use crate::transformation::{AttributeFilter, ImplItemType, JNISignature, JavaPath};
use darling::util::Flag;
use darling::FromMeta;
use proc_macro2::{Ident, TokenStream};
use proc_macro_error::{emit_error, emit_warning};
use quote::ToTokens;
use std::collections::HashSet;
use std::str::FromStr;
use syn::fold::Fold;
use syn::parse::{Parse, ParseStream};
use syn::parse_quote;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Extern;
use syn::visit::Visit;
use syn::Token;
use syn::{
    Abi, Attribute, Block, Error, Expr, FnArg, ImplItem, ImplItemMethod, LitStr, Meta, Pat,
    PatIdent, PatType, Path, ReturnType, Signature, Type, VisPublic, Visibility,
};

#[derive(Default)]
pub struct ImplExportVisitor<'ast> {
    pub(crate) items: Vec<(&'ast ImplItem, ImplItemType)>,
}

impl<'ast> Visit<'ast> for ImplExportVisitor<'ast> {
    fn visit_impl_item(&mut self, node: &'ast ImplItem) {
        match node {
            ImplItem::Method(method) => {
                let abi = method
                    .sig
                    .abi
                    .as_ref()
                    .and_then(|a| a.name.as_ref())
                    .map(|a| a.value());

                match abi.as_deref() {
                    Some("jni") => self.items.push((node, ImplItemType::Exported)),
                    Some("java") => self.items.push((node, ImplItemType::Imported)),
                    _ => self.items.push((node, ImplItemType::Unexported)),
                }
            }
            _ => self.items.push((node, ImplItemType::Unexported)),
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
        let attribute = input
            .call(Attribute::parse_outer)?
            .first()
            .cloned()
            .ok_or_else(|| Error::new(input.span(), "Invalid parsing of `call_type` attribute "))?;

        if attribute
            .path
            .get_ident()
            .ok_or_else(|| Error::new(attribute.path.span(), "expected identifier for attribute"))?
            != "call_type"
        {
            return Err(Error::new(
                attribute.path.span(),
                "expected identifier `call_type` for attribute",
            ));
        }

        let attr_meta: Meta = attribute.parse_meta()?;

        // Special-case `call_type(safe)` without further parentheses
        // TODO: Find out if it's possible to use darling to allow `call_type(safe)` *and* `call_type(safe(message = "foo"))` etc.
        if attr_meta.to_token_stream().to_string() == "call_type(safe)" {
            Ok(CallType::Safe(None))
        } else {
            CallType::from_meta(&attr_meta).map_err(|e| {
                Error::new(
                    attr_meta.span(),
                    format!("invalid `call_type` attribute options ({})", e),
                )
            })
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
        let abi = node
            .sig
            .abi
            .as_ref()
            .and_then(|l| l.name.as_ref().map(|n| n.value()));
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

                let mut jni_method_transformer = ExternJNIMethodTransformer::new(
                    self.struct_type.clone(),
                    self.struct_name.clone(),
                    self.package.clone(),
                    call_type_attribute,
                );
                jni_method_transformer.fold_impl_item_method(node)
            }
            _ => node,
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
    fn new(
        struct_type: Path,
        struct_name: String,
        package: Option<String>,
        call_type: CallType,
    ) -> Self {
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
        let jni_signature = JNISignature::new(
            node.sig.clone(),
            self.struct_type.clone(),
            self.struct_name.clone(),
            self.call_type.clone(),
        );

        let transformed_jni_signature = jni_signature.transformed_signature();
        let method_call = jni_signature.signature_call();

        let new_block: Block = match &self.call_type {
            CallType::Unchecked { .. } => {
                parse_quote! {{
                    IntoJavaValue::into(#method_call, &env)
                }}
            }

            CallType::Safe(exception_details) => {
                let outer_call_inputs = {
                    let mut inputs: Punctuated<Expr, Token![,]> = transformed_jni_signature.inputs.iter()
                        .map::<Expr, _>(|a| match a {
                            FnArg::Receiver(_) => panic!("Bug -- please report to library author. Found receiver type in freestanding signature!"),
                            FnArg::Typed(t) => {
                                match &*t.pat {
                                    Pat::Ident(PatIdent { ident, ..}) => {
                                        parse_quote!(#ident)
                                    }
                                    _ => panic!("Non-identifier argument pattern in function")
                                }
                            }
                        })
                        .collect();

                    inputs.push(parse_quote!(&env));
                    inputs
                };
                let outer_signature = {
                    let mut s = transformed_jni_signature.clone();
                    s.ident = Ident::new("outer", s.ident.span());

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
                        ReturnType::Type(_, ty) => *ty,
                    };

                    s.output = ReturnType::Type(
                        Token![->](outer_signature_span),
                        Box::new(parse_quote!(::jni::errors::Result<#outer_output_type>)),
                    );
                    s.abi = None;
                    s
                };

                let (default_exception_class, default_message) =
                    ("java/lang/RuntimeException", "JNI conversion error!");
                let (exception_class, message) = match exception_details {
                    Some(SafeParams {
                        exception_class,
                        message,
                    }) => {
                        let exception_class_result = exception_class
                            .as_ref()
                            .map(|v| &v.0)
                            .map(AsRef::as_ref)
                            .unwrap_or(default_exception_class);
                        let message_result = message
                            .as_ref()
                            .map(AsRef::as_ref)
                            .unwrap_or(default_message);

                        (exception_class_result, message_result)
                    }
                    None => (default_exception_class, default_message),
                };

                parse_quote! {{
                    #outer_signature {
                        TryIntoJavaValue::try_into(#method_call, &env)
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
            let mut attributes = node.attrs.clone();
            attributes.push(no_mangle);

            let discarded_known_attributes: HashSet<&str> = {
                let mut h = HashSet::new();
                h.insert("call_type");
                h
            };

            attributes
                .into_iter()
                .filter(|a| {
                    !discarded_known_attributes
                        .contains(&a.path.segments.to_token_stream().to_string().as_str())
                })
                .collect()
        };

        let node_span = node.span();
        ImplItemMethod {
            attrs: impl_item_attributes,
            vis: Visibility::Public(VisPublic {
                pub_token: Token![pub](node_span),
            }),
            defaultness: node.defaultness,
            sig: self.fold_signature(transformed_jni_signature.clone()),
            block: new_block,
        }
    }

    /// Transform original signature in JNI-ready one, including JClass and JNIEnv parameters into the function signature.
    fn fold_signature(&mut self, mut node: Signature) -> Signature {
        if node.ident.to_string().contains('_') {
            emit_error!(node.ident, "JNI methods cannot contain `_` character");
        }

        let jni_method_name = {
            let snake_case_package = self
                .package
                .clone()
                .map(|s| {
                    let mut s = s.replace('.', "_");
                    s.push('_');
                    s
                })
                .unwrap_or_else(|| "".into());

            format!(
                "Java_{}{}_{}",
                snake_case_package,
                self.struct_name,
                node.ident.to_string()
            )
        };

        node.inputs = {
            let mut res = Punctuated::new();
            res.push(parse_quote!(env: ::robusta_jni::jni::JNIEnv<'env>));
            res.push(parse_quote!(class: JClass));

            res.extend(node.inputs);
            res
        };

        node.ident = Ident::new(&jni_method_name, node.ident.span());
        node.abi = Some(Abi {
            extern_token: Extern { span: node.span() },
            name: Some(LitStr::new("system", node.span())),
        });

        node
    }
}
