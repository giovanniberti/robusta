use inflector::cases::camelcase::to_camel_case;
use proc_macro2::TokenStream;
use proc_macro_error::emit_error;
use quote::{quote, ToTokens};
use syn::fold::Fold;
use syn::parse_quote;
use syn::{FnArg, ImplItemMethod, Pat, PatIdent, ReturnType, Signature};

use crate::utils::{get_env_arg, is_self_method};

pub struct ImportedMethodTransformer {
    pub(crate) struct_name: String,
    pub(crate) package: Option<String>,
}

impl Fold for ImportedMethodTransformer {
    fn fold_impl_item_method(&mut self, node: ImplItemMethod) -> ImplItemMethod {
        let abi = node
            .sig
            .abi
            .as_ref()
            .and_then(|l| l.name.as_ref().map(|n| n.value()));
        match (&node.vis, &abi.as_deref()) {
            (_, Some("java")) => {
                if !node.block.stmts.is_empty() {
                    emit_error!(
                        node.block,
                        "`extern \"java\"` methods must have an empty body"
                    )
                }

                let original_signature = node.sig.clone();
                let self_method = is_self_method(&node.sig);
                let (signature, env_arg) = get_env_arg(node.sig);

                if !self_method && env_arg.is_none() {
                    emit_error!(
                        original_signature,
                        "static methods must have a parameter of type `&JNIEnv` as first parameter"
                    );
                    let dummy = ImplItemMethod {
                        sig: Signature {
                            abi: None,
                            ..original_signature
                        },
                        ..node
                    };

                    return dummy;
                }

                let jni_package_path = self
                    .package
                    .clone()
                    .map(|mut p| {
                        p.push('/');
                        p
                    })
                    .unwrap_or("".into())
                    .replace('.', "/");
                let java_class_path = format!("{}{}", jni_package_path, self.struct_name);
                let java_method_name = to_camel_case(&signature.ident.to_string());

                let input_types_conversions = signature
                    .inputs
                    .iter()
                    .filter_map(|i| match i {
                        FnArg::Typed(t) => match &*t.pat {
                            Pat::Ident(PatIdent { ident, .. }) if ident == "self" => None,
                            _ => Some(&t.ty),
                        },
                        FnArg::Receiver(_) => None,
                    })
                    .map(|t| {
                        quote! { <#t as TryIntoJavaValue>::SIG_TYPE, }
                    })
                    .fold(TokenStream::new(), |t, mut tok| {
                        t.to_tokens(&mut tok);
                        tok
                    });

                let output_conversion = match signature.output {
                    ReturnType::Default => quote!(""),
                    ReturnType::Type(_arrow, ref ty) => {
                        quote! { <#ty as TryIntoJavaValue>::SIG_TYPE }
                    }
                };

                let java_signature =
                    quote! { ["(", #input_types_conversions ")", #output_conversion].join("") };

                let input_conversions = signature.inputs.iter().fold(TokenStream::new(), |mut tok, input| {
                    match input {
                        FnArg::Receiver(_) => { tok }
                        FnArg::Typed(t) => {
                            let pat = &t.pat;
                            let ty = &t.ty;
                            let conversion: TokenStream = quote! { TryInto::try_into(<#ty as IntoJavaValue>::into(#pat, &env)).unwrap(), };
                            conversion.to_tokens(&mut tok);
                            tok
                        }
                    }
                });

                ImplItemMethod {
                    sig: Signature {
                        abi: None,
                        ..original_signature
                    },
                    block: if self_method {
                        parse_quote! {{
                            let env: ::robusta_jni::jni::JNIEnv = <Self as JNIEnvLink>::get_env(&self).clone();
                            let res = env.call_method(IntoJavaValue::into(self, &env).autobox(&env), #java_method_name, #java_signature, &[#input_conversions]).unwrap();
                            TryInto::try_into(JValueWrapper::from(res)).unwrap()
                        }}
                    } else {
                        let env_ident = match env_arg.unwrap() {
                            FnArg::Typed(t) => {
                                match *t.pat {
                                    Pat::Ident(PatIdent { ident, .. }) => ident,
                                    _ => panic!("non-ident pat in FnArg")
                                }
                            },
                            _ => panic!("Bug -- please report to library author. Expected env parameter, found receiver")
                        };

                        parse_quote! {{
                            let env: &::robusta_jni::jni::JNIEnv = #env_ident;
                            let res = env.call_static_method(#java_class_path, #java_method_name, #java_signature, &[#input_conversions]).unwrap();
                            TryInto::try_into(JValueWrapper::from(res)).unwrap()
                        }}
                    },
                    ..node
                }
            }

            _ => node,
        }
    }
}
