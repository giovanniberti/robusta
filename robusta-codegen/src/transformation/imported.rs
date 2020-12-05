use inflector::cases::camelcase::to_camel_case;
use proc_macro2::TokenStream;
use proc_macro_error::{abort, emit_error};
use quote::{quote, ToTokens};
use syn::fold::Fold;
use syn::spanned::Spanned;
use syn::{parse_quote, TypePath, Type, PathArguments, GenericArgument};
use syn::{FnArg, ImplItemMethod, Pat, PatIdent, ReturnType, Signature};

use crate::utils::{get_abi, get_env_arg, is_self_method};
use crate::transformation::utils::get_call_type;
use crate::transformation::{CallType, CallTypeAttribute, SafeParams};
use std::collections::HashSet;

pub struct ImportedMethodTransformer {
    pub(crate) struct_name: String,
    pub(crate) package: Option<String>,
}

impl Fold for ImportedMethodTransformer {
    fn fold_impl_item_method(&mut self, node: ImplItemMethod) -> ImplItemMethod {
        let abi = get_abi(&node.sig);
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
                let (signature, env_arg) = get_env_arg(node.sig.clone());

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

                let call_type_attribute = get_call_type(&node);
                let call_type = call_type_attribute.as_ref().map(|c| &c.call_type).unwrap_or(&CallType::Safe(None));

                if let Some(CallTypeAttribute { attr, ..}) = &call_type_attribute {
                    if let CallType::Safe(Some(params)) = call_type {
                        if let SafeParams { message: Some(_), .. } | SafeParams { exception_class: Some(_), .. } = params {
                            abort!(attr, "can't have exception message or exception class for imported methods")
                        }
                    }
                }

                let jni_package_path = self
                    .package
                    .clone()
                    .filter(|p| !p.is_empty())
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
                        if let CallType::Safe(_) = call_type {
                            quote! { <#t as ::robusta_jni::convert::TryIntoJavaValue>::SIG_TYPE, }
                        } else {
                            quote! { <#t as ::robusta_jni::convert::IntoJavaValue>::SIG_TYPE, }
                        }
                    })
                    .fold(TokenStream::new(), |t, mut tok| {
                        t.to_tokens(&mut tok);
                        tok
                    });

                let output_conversion = match signature.output {
                    ReturnType::Default => quote!(""),
                    ReturnType::Type(_arrow, ref ty) => {
                        if let CallType::Safe(_) = call_type {
                            let inner_result_ty = match &**ty {
                                Type::Path(TypePath { path, .. }) => {
                                    path.segments.last().map(|s| match &s.arguments {
                                        PathArguments::AngleBracketed(a) => {
                                            match &a.args.first().expect("return type must be `::robusta_jni::jni::errors::Result` when using \"java\" ABI with an implicit or \"safe\" `call_type`") {
                                                GenericArgument::Type(t) => t,
                                                _ => abort!(a, "first generic argument in return type must be a type")
                                            }
                                        },
                                        PathArguments::None => {
                                            let user_attribute_message = call_type_attribute.as_ref().map(|_| "because of this attribute");
                                            abort!(s, "return type must be `::robusta_jni::jni::errors::Result` when using \"java\" ABI with an implicit or \"safe\" `call_type`";
                                                                        help = "replace `{}` with `Result<{}>`", s.ident, s.ident;
                                                                        help =? call_type_attribute.as_ref().map(|c| c.attr.span()).unwrap() => user_attribute_message)
                                        },
                                        _ => abort!(s, "return type must be `::robusta_jni::jni::errors::Result` when using \"java\" ABI with an implicit or \"safe\" `call_type`")
                                    })
                                },
                                _ => abort!(ty, "return type must be `::robusta_jni::jni::errors::Result` when using \"java\" ABI with an implicit or \"safe\" `call_type`")
                            }.unwrap();

                            quote! { <#inner_result_ty as ::robusta_jni::convert::TryIntoJavaValue>::SIG_TYPE }
                        } else {
                            quote! { <#ty as ::robusta_jni::convert::IntoJavaValue>::SIG_TYPE }
                        }
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
                            let conversion: TokenStream = if let CallType::Safe(_) = call_type {
                                quote! { ::std::convert::Into::into(<#ty as ::robusta_jni::convert::TryIntoJavaValue>::try_into(#pat, &env)?), }
                            } else {
                                quote! { ::std::convert::Into::into(<#ty as ::robusta_jni::convert::IntoJavaValue>::into(#pat, &env)), }
                            };
                            conversion.to_tokens(&mut tok);
                            tok
                        }
                    }
                });

                let return_expr= if let CallType::Safe(_) = call_type {
                    quote! {
                        res.and_then(|v| ::std::convert::TryInto::try_into(::robusta_jni::convert::JValueWrapper::from(v)))
                           .and_then(|v| ::robusta_jni::convert::TryFromJavaValue::try_from(v, &env))
                    }
                } else {
                    quote! {
                        ::std::convert::TryInto::try_into(::robusta_jni::convert::JValueWrapper::from(res))
                            .and_then(|v| ::robusta_jni::convert::TryFromJavaValue::try_from(v, &env))
                            .unwrap()
                    }
                };

                let impl_item_attributes = {
                    let discarded_known_attributes: HashSet<&str> = {
                        let mut h = HashSet::new();
                        h.insert("call_type");
                        h
                    };

                    node.attrs
                        .into_iter()
                        .filter(|a| {
                            !discarded_known_attributes
                                .contains(&a.path.segments.to_token_stream().to_string().as_str())
                        })
                        .collect()
                };

                ImplItemMethod {
                    sig: Signature {
                        abi: None,
                        ..original_signature.clone()
                    },
                    block: if self_method {
                        let self_span = node.sig.inputs.iter().next().unwrap().span();
                        match call_type {
                            CallType::Safe(_) => {
                                parse_quote_spanned! { self_span => {
                                    let env: ::robusta_jni::jni::JNIEnv = <Self as ::robusta_jni::convert::JNIEnvLink>::get_env(&self).clone();
                                    let res = env.call_method(::robusta_jni::convert::JavaValue::autobox(::robusta_jni::convert::IntoJavaValue::into(self, &env), &env), #java_method_name, #java_signature, &[#input_conversions]);
                                    #return_expr
                                }}
                            }
                            CallType::Unchecked(_) => {
                                parse_quote_spanned! { self_span => {
                                    let env: ::robusta_jni::jni::JNIEnv = <Self as ::robusta_jni::convert::JNIEnvLink>::get_env(&self).clone();
                                    let res = env.call_method(::robusta_jni::convert::JavaValue::autobox(::robusta_jni::convert::IntoJavaValue::into(self, &env), &env), #java_method_name, #java_signature, &[#input_conversions]).unwrap();
                                    #return_expr
                                }}
                            }
                        }
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

                        match call_type {
                            CallType::Safe(_) => {
                                parse_quote! {{
                                    let env: &::robusta_jni::jni::JNIEnv = #env_ident;
                                    let res = env.call_static_method(#java_class_path, #java_method_name, #java_signature, &[#input_conversions]);
                                    #return_expr
                                }}
                            },
                            CallType::Unchecked(_) => {
                                parse_quote! {{
                                    let env: &::robusta_jni::jni::JNIEnv = #env_ident;
                                    let res = env.call_static_method(#java_class_path, #java_method_name, #java_signature, &[#input_conversions]).unwrap();
                                    #return_expr
                                }}
                            }
                        }
                    },
                    attrs: impl_item_attributes,
                    ..node
                }
            }

            _ => node,
        }
    }
}
