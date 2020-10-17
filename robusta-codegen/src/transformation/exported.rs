use syn::{ImplItem, Attribute, Error, Meta, Path, ImplItemMethod, Visibility, Expr, FnArg, Pat, PatType, PatIdent, Type, ReturnType, Block, VisPublic, Signature, Abi, Generics, LitStr, TypeReference};
use crate::transformation::{ImplItemType, JavaPath, AttributeFilter, JNISignatureTransformer};
use darling::util::Flag;
use syn::parse::{Parse, ParseStream};
use syn::fold::Fold;
use std::collections::HashSet;
use proc_macro2::{TokenStream, Ident};
use syn::punctuated::Punctuated;
use syn::token::Extern;
use syn::visit::Visit;
use quote::ToTokens;
use syn::spanned::Spanned;
use syn::Token;
use darling::FromMeta;
use proc_macro_error::{emit_error, emit_warning};
use syn::parse_quote;
use std::str::FromStr;
use std::iter;
use std::iter::FromIterator;
use crate::utils::{canonicalize_path, is_self_method};

#[derive(Default)]
pub struct ImplExportVisitor<'ast> {
    pub(crate) items: Vec<(&'ast ImplItem, ImplItemType)>
}

impl<'ast> Visit<'ast> for ImplExportVisitor<'ast> {
    fn visit_impl_item(&mut self, node: &'ast ImplItem) {
        match node {
            ImplItem::Method(method) => {
                let abi = method.sig.abi.as_ref().and_then(|a| a.name.as_ref()).map(|a| a.value());

                match abi.as_deref() {
                    Some("jni") => self.items.push((node, ImplItemType::Exported)),
                    Some("java") => self.items.push((node, ImplItemType::Imported)),
                    _ => self.items.push((node, ImplItemType::Unexported))
                }
            }
            _ => self.items.push((node, ImplItemType::Unexported))
        }
    }
}

#[derive(Clone, Default, FromMeta)]
#[darling(default)]
pub(crate) struct SafeParams {
    exception_class: Option<JavaPath>,
    message: Option<String>,
}

#[derive(Clone, FromMeta)]
pub(crate) enum CallType {
    Safe(Option<SafeParams>),
    Unchecked(Flag),
}

impl Parse for CallType {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let attribute = input.call(Attribute::parse_outer)?.first().cloned().ok_or_else(|| Error::new(input.span(), "Invalid parsing of `call_type` attribute "))?;

        if attribute.path.get_ident().ok_or_else(|| Error::new(attribute.path.span(), "expected identifier for attribute"))? != "call_type" {
            return Err(Error::new(attribute.path.span(), "expected identifier `call_type` for attribute"));
        }

        let attr_meta: Meta = attribute.parse_meta()?;

        // Special-case `call_type(safe)` without further parentheses
        // TODO: Find out if it's possible to use darling to allow `call_type(safe)` *and* `call_type(safe(message = "foo"))` etc.
        if attr_meta.to_token_stream().to_string() == "call_type(safe)" {
            Ok(CallType::Safe(None))
        } else {
            CallType::from_meta(&attr_meta).map_err(|e| Error::new(attr_meta.span(), format!("invalid `call_type` attribute options ({})", e)))
        }
    }
}

pub struct ExportedMethodTransformer {
    pub(crate) struct_type: Path,
    pub(crate) struct_name: String,
    pub(crate) package: Option<String>,
}

impl Fold for ExportedMethodTransformer {
    fn fold_impl_item_method(&mut self, node: ImplItemMethod) -> ImplItemMethod {
        let abi = node.sig.abi.as_ref().and_then(|l| l.name.as_ref().map(|n| n.value()));
        match (&node.vis, &abi.as_deref()) {
            (Visibility::Public(_), Some("jni")) => {
                let whitelist = {
                    let mut f = HashSet::new();
                    f.insert(syn::parse2(TokenStream::from_str("call_type").unwrap()).unwrap());
                    f
                };

                let mut attributes_collector = AttributeFilter::with_whitelist(whitelist);
                attributes_collector.visit_impl_item_method(&node);

                let call_type_attribute = attributes_collector.filtered_attributes.first().and_then(|call_type_attr| {
                    syn::parse2(call_type_attr.to_token_stream()).map_err(|e| {
                        emit_warning!(e.span(), format!("invalid parsing of `call_type` attribute, defaulting to #[call_type(safe)]. {}", e));
                        e
                    }).ok()
                }).unwrap_or(CallType::Safe(None));

                let mut jni_method_transformer = ExternJNIMethodTransformer::new(self.struct_type.clone(), self.struct_name.clone(), self.package.clone(), call_type_attribute);
                jni_method_transformer.fold_impl_item_method(node)
            }
            _ => node
        }
    }
}

struct ExternJNIMethodTransformer {
    struct_type: Path,
    struct_name: String,
    package: Option<String>,
    call_type: CallType,
}

impl ExternJNIMethodTransformer {
    fn new(struct_type: Path, struct_name: String, package: Option<String>, call_type: CallType) -> Self {
        ExternJNIMethodTransformer {
            struct_type,
            struct_name,
            package,
            call_type,
        }
    }
}

impl Fold for ExternJNIMethodTransformer {
    fn fold_impl_item_method(&mut self, node: ImplItemMethod) -> ImplItemMethod {
        let original_signature = node.sig.clone();

        let mut jni_signature_transformer = JNISignatureTransformer::new(self.struct_type.clone(), self.struct_name.clone(), original_signature.ident.to_string(), self.call_type.clone());

        let self_method = is_self_method(&node.sig);

        // Check whether second argument (first exluding self) is of type &JNIEnv, if so we don't transform it
        let possible_env_arg = if !self_method {
            original_signature.inputs.iter().next()
        } else {
            original_signature.inputs.iter().nth(1)
        };

        let has_explicit_env_arg = if let Some(FnArg::Typed(PatType { ty, ..})) = possible_env_arg {
            if let Type::Reference(TypeReference { elem, .. }) = &**ty {
                if let Type::Path(t) = &**elem {
                    let full_path: Path = parse_quote! { ::robusta_jni::jni::JNIEnv };
                    let imported_path: Path = parse_quote! { JNIEnv };
                    let canonicalized_type_path = canonicalize_path(&t.path);

                    canonicalized_type_path == imported_path || canonicalized_type_path == full_path
                } else { false }
            } else { false }
        } else { false };

        let (signature, explicit_env_arg): (Signature, Option<FnArg>) = if has_explicit_env_arg {
            let mut inner_signature = original_signature;

            let mut iter = inner_signature.inputs.into_iter();

            if self_method {
                let self_arg = iter.next();
                let env_arg = iter.next();

                inner_signature.inputs = iter::once(self_arg.unwrap()).chain(iter).collect();
                (inner_signature, env_arg)
            } else {
                let env_arg = iter.next();
                inner_signature.inputs = iter.collect();

                (inner_signature, env_arg)
            }
        } else {
            (original_signature, None)
        };

        let transformed_jni_signature = jni_signature_transformer.fold_signature(signature.clone());

        let call_inputs_idents: Punctuated<Expr, Token![,]> = transformed_jni_signature.inputs.iter().cloned()
            .map::<Expr, _>(|a| match a {
                FnArg::Receiver(_) => panic!("Bug -- please report to library author. Found receiver type in freestanding signature!"),
                FnArg::Typed(t) => {
                    match &*t.pat {
                        Pat::Ident(ident) => {
                            let ident = ident.ident.clone();
                            parse_quote!(#ident)
                        }
                        _ => panic!("Non-identifier argument pattern in function")
                    }
                }
            })
            .collect();

        let outer_call_inputs = {
            let mut result = call_inputs_idents.clone();

            if let CallType::Safe(_) = self.call_type {
                result.push(parse_quote!(&env))
            }

            result
        };

        let method_call_inputs: Punctuated<Expr, Token![,]> = {
            let mut result: Vec<_> = transformed_jni_signature.inputs.iter()
                .map(|arg| {
                    if let FnArg::Typed(PatType { pat, .. }) = arg {
                        if let Pat::Ident(PatIdent { ident, .. }) = &**pat {
                            let input_param: Expr = {
                                match self.call_type {
                                    CallType::Safe(_) => parse_quote! { TryFromJavaValue::try_from(#ident, &env)? },
                                    CallType::Unchecked { .. } => parse_quote! { FromJavaValue::from(#ident, &env) }
                                }
                            };
                            input_param
                        } else {
                            panic!("Bug -- please report to library author. Found non-ident FnArg pattern");
                        }
                    } else {
                        panic!("Bug -- please report to library author. Found receiver FnArg after freestanding transform");
                    }
                }).collect();

            if explicit_env_arg.is_some() {
                // because `self` is kept in the transformed JNI signature, if this is a `self` method we put `env` *after* self, otherwise the env parameter must be first
                let idx = if self_method { 1 } else { 0 };
                result.insert(idx, parse_quote!(&env));
            }

            Punctuated::from_iter(result.into_iter())
        };

        let outer_signature = {
            let mut s = transformed_jni_signature;
            s.ident = Ident::new("outer", signature.ident.span());

            s.inputs.push(FnArg::Typed(PatType {
                attrs: vec![],
                pat: Box::new(Pat::Ident(PatIdent {
                    attrs: vec![],
                    by_ref: None,
                    mutability: None,
                    ident: Ident::new("env", s.inputs.span()),
                    subpat: None,
                })),
                colon_token: Token![:](s.inputs.span()),
                ty: Box::new(parse_quote! { &::robusta_jni::jni::JNIEnv<'env> }),
            }));

            let outer_signature_span = s.span();
            let outer_output_type: Type = match s.output {
                ReturnType::Default => parse_quote!(()),
                ReturnType::Type(_, ty) => *ty
            };

            s.output = ReturnType::Type(Token![->](outer_signature_span), Box::new(parse_quote!(::jni::errors::Result<#outer_output_type>)));
            s.abi = None;
            s
        };

        let node_span = node.span();
        let struct_name = Ident::new(&self.struct_name, node_span);
        let method_name = signature.ident.clone();

        let new_block: Block = match &self.call_type {
            CallType::Unchecked { .. } => {
                parse_quote! {{
                    IntoJavaValue::into(#struct_name::#method_name(#method_call_inputs), &env)
                }}
            }

            CallType::Safe(exception_details) => {
                let (default_exception_class, default_message) = ("java/lang/RuntimeException", "JNI conversion error!");
                let (exception_class, message) = match exception_details {
                    Some(SafeParams { exception_class, message }) => {
                        let exception_class_result = exception_class.as_ref().map(|v| &v.0).map(AsRef::as_ref).unwrap_or(default_exception_class);
                        let message_result = message.as_ref().map(AsRef::as_ref).unwrap_or(default_message);

                        (exception_class_result, message_result)
                    }
                    None => (default_exception_class, default_message)
                };

                parse_quote! {{
                    #outer_signature {
                        TryIntoJavaValue::try_into(#struct_name::#method_name(#method_call_inputs), &env)
                    }

                    match outer(#outer_call_inputs) {
                        Ok(result) => result,
                        Err(_) => {
                            env.throw_new(#exception_class, #message).unwrap();

                            /* We never hand out Rust references and the object returned is ignored
                             * by the JVM, so it should be safe to just return zeroed memory.
                             * Also, all primitives have a valid zero representation and because objects
                             * are represented as pointers this should not have any unsafe side effects.
                             * (Uninitialized memory would probably work as well)
                             */
                            unsafe { ::std::mem::zeroed() }
                        }
                    }
                }}
            }
        };

        let no_mangle = parse_quote! { #[no_mangle] };
        let impl_item_attributes = {
            let mut attributes = node.attrs;
            attributes.push(no_mangle);

            let discarded_known_attributes: HashSet<&str> = {
                let mut h = HashSet::new();
                h.insert("call_type");
                h
            };

            attributes.into_iter().filter(|a| {
                !discarded_known_attributes.contains(&a.path.segments.to_token_stream().to_string().as_str())
            }).collect()
        };

        ImplItemMethod {
            attrs: impl_item_attributes,
            vis: Visibility::Public(VisPublic {
                pub_token: Token![pub](node_span)
            }),
            defaultness: node.defaultness,
            sig: self.fold_signature(signature),
            block: new_block,
        }
    }

    /// Transform original signature in JNI-ready one, including JClass and JNIEnv parameters into the function signature.
    fn fold_signature(&mut self, node: Signature) -> Signature {
        if node.ident.to_string().contains('_') {
            emit_error!(node.ident, "JNI methods cannot contain `_` character");
        }

        let jni_method_name = {
            let snake_case_package = self.package.clone().map(|s| {
                let mut s = s.replace('.', "_");
                s.push('_');
                s
            }).unwrap_or_else(|| "".into());

            format!("Java_{}{}_{}", snake_case_package, self.struct_name, node.ident.to_string())
        };

        let mut jni_signature_transformer = JNISignatureTransformer::new(self.struct_type.clone(), self.struct_name.clone(), node.ident.to_string(), self.call_type.clone());

        let jni_abi_inputs: Punctuated<FnArg, Token![,]> = {
            let mut res = Punctuated::new();
            res.push(parse_quote!(env: ::robusta_jni::jni::JNIEnv<'env>));
            res.push(parse_quote!(class: JClass));

            let jni_compatible_inputs: Punctuated<_, Token![,]> = node.inputs.iter().cloned().map(|input| {
                jni_signature_transformer.fold_fn_arg(input)
            }).collect();

            res.extend(jni_compatible_inputs);
            res
        };

        let jni_output = jni_signature_transformer.fold_return_type(node.output.clone());

        let self_method = is_self_method(&node);

        Signature {
            constness: node.constness,
            asyncness: node.asyncness,
            unsafety: node.unsafety,
            abi: Some(Abi {
                extern_token: Extern { span: node.span() },
                name: Some(LitStr::new("system", node.span())),
            }),
            fn_token: node.fn_token,
            ident: Ident::new(&jni_method_name, node.ident.span()),
            generics: jni_signature_transformer.transform_generics(Generics::default(), self_method),
            paren_token: node.paren_token,
            inputs: jni_abi_inputs,
            variadic: node.variadic,
            output: jni_output,
        }
    }
}

