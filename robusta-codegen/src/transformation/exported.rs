use crate::transformation::{JNISignature, CallType, SafeParams};
use proc_macro2::Ident;
use proc_macro_error::emit_error;
use quote::ToTokens;
use std::collections::HashSet;
use syn::fold::Fold;
use syn::{parse_quote, LifetimeDef};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Extern;
use syn::Token;
use syn::{
    Abi, Block, Expr, FnArg, ImplItemMethod, LitStr, Pat,
    PatIdent, PatType, Path, ReturnType, Signature, Type, VisPublic, Visibility,
};
use crate::transformation::utils::get_call_type;
use crate::utils::{get_abi, is_self_method};

pub struct ExportedMethodTransformer {
    pub(crate) struct_type: Path,
    pub(crate) struct_name: String,
    pub(crate) struct_lifetimes: Vec<LifetimeDef>,
    pub(crate) package: Option<String>,
}

impl Fold for ExportedMethodTransformer {
    fn fold_impl_item_method(&mut self, node: ImplItemMethod) -> ImplItemMethod {
        let abi = get_abi(&node.sig);
        match (&node.vis, &abi.as_deref()) {
            (Visibility::Public(_), Some("jni")) => {
                let call_type_attribute = get_call_type(&node).map(|c| c.call_type).unwrap_or(CallType::Safe(None));

                let mut jni_method_transformer = ExternJNIMethodTransformer::new(
                    self.struct_type.clone(),
                    self.struct_name.clone(),
                    self.struct_lifetimes.clone(),
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
    struct_lifetimes: Vec<LifetimeDef>,
    package: Option<String>,
    call_type: CallType,
}

impl ExternJNIMethodTransformer {
    fn new(
        struct_type: Path,
        struct_name: String,
        struct_lifetimes: Vec<LifetimeDef>,
        package: Option<String>,
        call_type: CallType,
    ) -> Self {
        ExternJNIMethodTransformer {
            struct_type,
            struct_name,
            struct_lifetimes,
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
            self.struct_lifetimes.clone(),
            self.call_type.clone(),
        );

        let transformed_jni_signature = jni_signature.transformed_signature();
        let method_call = jni_signature.signature_call();

        let new_block: Block = match &self.call_type {
            CallType::Unchecked { .. } => {
                parse_quote! {{
                    ::robusta_jni::convert::IntoJavaValue::into(#method_call, &env)
                }}
            }

            CallType::Safe(exception_details) => {
                let outer_call_inputs = {
                    let mut inputs: Punctuated<Expr, Token![,]> = jni_signature.args_iter()
                        .map(|p| -> Expr {
                            let PatType { pat, .. } = p;

                            match &**pat {
                                Pat::Ident(PatIdent { ident, ..}) => {
                                    parse_quote!(#ident)
                                }
                                _ => panic!("Non-identifier argument pattern in function")
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
                        ty: Box::new(parse_quote! { &'borrow ::robusta_jni::jni::JNIEnv<'env> }),
                    }));

                    let outer_signature_span = s.span();
                    let outer_output_type: Type = match s.output {
                        ReturnType::Default => parse_quote!(()),
                        ReturnType::Type(_, ty) => *ty,
                    };

                    s.output = ReturnType::Type(
                        Token![->](outer_signature_span),
                        Box::new(parse_quote!(::robusta_jni::jni::errors::Result<#outer_output_type>)),
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
                        ::robusta_jni::convert::TryIntoJavaValue::try_into(#method_call, &env)
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
            sig: self.fold_signature(node.sig),
            block: new_block,
        }
    }

    /// Transform original signature in JNI-ready one, including JClass and JNIEnv parameters into the function signature.
    fn fold_signature(&mut self, node: Signature) -> Signature {
        let jni_signature = JNISignature::new(
            node.clone(),
            self.struct_type.clone(),
            self.struct_name.clone(),
            self.struct_lifetimes.clone(),
            self.call_type.clone(),
        );

        let mut sig = jni_signature.transformed_signature;

        if sig.ident.to_string().contains('_') {
            emit_error!(sig.ident, "JNI methods cannot contain `_` character");
        }

        let jni_method_name = {
            let snake_case_package = self
                .package
                .clone()
                .filter(|s| !s.is_empty())
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
                sig.ident.to_string()
            )
        };

        sig.inputs = {
            let mut res = Punctuated::new();
            res.push(parse_quote!(env: ::robusta_jni::jni::JNIEnv<'env>));

            if !is_self_method(&node) {
                res.push(parse_quote!(class: ::robusta_jni::jni::objects::JClass));
            }

            res.extend(sig.inputs);
            res
        };

        sig.ident = Ident::new(&jni_method_name, sig.ident.span());
        sig.abi = Some(Abi {
            extern_token: Extern { span: sig.span() },
            name: Some(LitStr::new("system", sig.span())),
        });

        sig
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use proc_macro2::TokenStream;
    use std::str::FromStr;

    fn setup_package(package: Option<String>, struct_name: String, method_name: String) -> ImplItemMethod {
        let struct_name_token_stream = TokenStream::from_str(&struct_name).unwrap();
        let method_name_token_stream = TokenStream::from_str(&method_name).unwrap();

        let method: ImplItemMethod = parse_quote! { pub extern "jni" fn #method_name_token_stream() {} };
        let mut transformer = ExternJNIMethodTransformer {
            struct_type: parse_quote! { #struct_name_token_stream },
            struct_name,
            struct_lifetimes: vec![],
            package,
            call_type: CallType::Safe(None)
        };

        transformer.fold_impl_item_method(method)
    }

    #[test]
    fn jni_method_is_public() {
        let output = setup_package(None, "Foo".into(), "foo".into());
        assert!(matches!(output.vis, Visibility::Public(_)))
    }

    #[test]
    fn jni_method_follows_naming_scheme() {
        let output_no_package = setup_package(None, "Foo".into(), "foo".into());
        assert_eq!(output_no_package.sig.ident.to_string(), format!("Java_Foo_foo"));

        let output_with_package = setup_package(Some("com.bar.quux".into()), "Foo".into(), "foo".into());
        assert_eq!(output_with_package.sig.ident.to_string(), format!("Java_com_bar_quux_Foo_foo"));
    }

    #[test]
    fn jni_method_has_no_mangle() {
        let output = setup_package(None, "Foo".into(), "foo".into());
        let no_mangle = parse_quote! { #[no_mangle] };
        assert!(output.attrs.contains(&no_mangle));
    }

    #[test]
    fn jni_method_has_system_abi() {
        let output = setup_package(None, "Foo".into(), "foo".into());
        assert_eq!(output.sig.abi.unwrap().name.unwrap().value(), "system")
    }

    fn setup_with_params(params: TokenStream, struct_name: String) -> ImplItemMethod {
        let package = None;
        let method_name = "foo".to_string();
        let struct_name_token_stream = TokenStream::from_str(&struct_name).unwrap();
        let method_name_token_stream = TokenStream::from_str(&method_name).unwrap();

        let method: ImplItemMethod = parse_quote! {
                pub extern "jni" fn #method_name_token_stream(#params) -> i32 {}
            };

        let mut transformer = ExternJNIMethodTransformer {
            struct_type: parse_quote! { #struct_name_token_stream },
            struct_name,
            struct_lifetimes: vec![],
            package,
            call_type: CallType::Safe(None)
        };

        transformer.fold_impl_item_method(method)
    }

    #[test]
    fn static_method_params() {
        use quote::quote;

        let param_type_1: TokenStream = parse_quote! { i32 };
        let param_type_2: TokenStream = parse_quote! { FooBar };
        let output = setup_with_params(quote! { _1: #param_type_1, _2: #param_type_2 }, "Foo".to_string());

        let env_type: Type = parse_quote! { ::robusta_jni::jni::JNIEnv<'env> };
        let class_type: Type = parse_quote! { ::robusta_jni::jni::objects::JClass };
        let conv_type_1: Type = parse_quote! { <#param_type_1 as ::robusta_jni::convert::TryFromJavaValue<'env, 'borrow>>::Source };
        let conv_type_2: Type = parse_quote! { <#param_type_2 as ::robusta_jni::convert::TryFromJavaValue<'env, 'borrow>>::Source };


        let args: &[FnArg] = &output.sig.inputs.into_iter().collect::<Vec<_>>();
        match args {
            [FnArg::Typed(PatType { ty: ty_env, .. }),
            FnArg::Typed(PatType { ty: ty_class, .. }),
            FnArg::Typed(PatType { ty: ty_1, .. }),
            FnArg::Typed(PatType { ty: ty_2, .. })] => {
                assert_eq!(ty_env.to_token_stream().to_string(), env_type.to_token_stream().to_string());
                assert_eq!(ty_class.to_token_stream().to_string(), class_type.to_token_stream().to_string());
                assert_eq!(ty_1.to_token_stream().to_string(), conv_type_1.to_token_stream().to_string());
                assert_eq!(ty_2.to_token_stream().to_string(), conv_type_2.to_token_stream().to_string());
            },

            _ => assert!(false)
        }
    }

    #[test]
    fn self_method_params() {
        use quote::quote;

        let struct_name = "Foo".to_string();
        let struct_name_toks = TokenStream::from_str(&struct_name).unwrap();

        let param_type_1: TokenStream = parse_quote! { i32 };
        let param_type_2: TokenStream = parse_quote! { FooBar };
        let output = setup_with_params(quote! { self, _1: #param_type_1, _2: #param_type_2 }, struct_name.clone());

        let env_type: Type = parse_quote! { ::robusta_jni::jni::JNIEnv<'env> };
        let self_conv_type: Type = parse_quote! { <#struct_name_toks as ::robusta_jni::convert::TryFromJavaValue<'env, 'borrow>>::Source };
        let conv_type_1: Type = parse_quote! { <#param_type_1 as ::robusta_jni::convert::TryFromJavaValue<'env, 'borrow>>::Source };
        let conv_type_2: Type = parse_quote! { <#param_type_2 as ::robusta_jni::convert::TryFromJavaValue<'env, 'borrow>>::Source };

        let args: &[FnArg] = &output.sig.inputs.into_iter().collect::<Vec<_>>();
        match args {
            [FnArg::Typed(PatType { ty: ty_env, .. }),
            FnArg::Typed(PatType { ty: ty_self, .. }),
            FnArg::Typed(PatType { ty: ty_1, .. }),
            FnArg::Typed(PatType { ty: ty_2, .. })] => {
                assert_eq!(ty_env.to_token_stream().to_string(), env_type.to_token_stream().to_string());
                assert_eq!(ty_self.to_token_stream().to_string(), self_conv_type.to_token_stream().to_string());
                assert_eq!(ty_1.to_token_stream().to_string(), conv_type_1.to_token_stream().to_string());
                assert_eq!(ty_2.to_token_stream().to_string(), conv_type_2.to_token_stream().to_string());
            },

            _ => assert!(false)
        }
    }
}