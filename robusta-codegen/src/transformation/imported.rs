use inflector::cases::camelcase::to_camel_case;
use proc_macro2::{TokenStream, TokenTree};
use proc_macro_error::{abort, emit_error, emit_warning};
use quote::{quote_spanned, ToTokens};
use syn::fold::Fold;
use syn::spanned::Spanned;
use syn::{parse_quote, GenericArgument, PathArguments, Type, TypePath};
use syn::{FnArg, ImplItemFn, Lit, Pat, PatIdent, ReturnType, Signature};

use crate::transformation::context::StructContext;
use crate::transformation::utils::{get_call_type, get_output_type_override};
use crate::transformation::{CallType, CallTypeAttribute, SafeParams};
use crate::utils::{get_abi, get_class_arg_if_any, get_env_arg, is_self_method};
use std::collections::HashSet;

pub struct ImportedMethodTransformer<'ctx> {
    pub(crate) struct_context: &'ctx StructContext,
}

impl<'ctx> Fold for ImportedMethodTransformer<'ctx> {
    fn fold_impl_item_fn(&mut self, node: ImplItemFn) -> ImplItemFn {
        let abi = get_abi(&node.sig);
        match (&node.vis, &abi.as_deref()) {
            (_, Some("java")) => {
                let constructor_attribute =
                    node.attrs.iter().find(|a| a.path().is_ident("constructor"));
                let is_constructor = {
                    match constructor_attribute {
                        Some(a) => {
                            if a.meta
                                .require_list()
                                .is_ok_and(|meta_list| !meta_list.tokens.is_empty())
                            {
                                emit_warning!(
                                    a.to_token_stream(),
                                    "#[constructor] attribute does not take parameters"
                                )
                            }
                            true
                        }
                        None => false,
                    }
                };

                if !node.block.stmts.is_empty() {
                    emit_error!(
                        node.block,
                        "`extern \"java\"` methods must have an empty body"
                    )
                }

                let mut original_signature = node.sig.clone();
                let self_method = is_self_method(&node.sig);
                let (signature, env_arg) = get_env_arg(node.sig.clone());
                let (mut signature, class_ref_arg) = get_class_arg_if_any(signature.clone());

                let impl_item_attributes: Vec<_> = {
                    let discarded_known_attributes: HashSet<&str> = {
                        let mut h = HashSet::new();
                        h.insert("call_type");
                        h.insert("output_type");

                        if is_constructor {
                            h.insert("constructor");
                        }
                        h
                    };

                    node.clone()
                        .attrs
                        .into_iter()
                        .filter(|a| {
                            !discarded_known_attributes
                                .contains(&a.path().segments.to_token_stream().to_string().as_str())
                        })
                        .collect()
                };

                let dummy = ImplItemFn {
                    sig: Signature {
                        abi: None,
                        ..original_signature.clone()
                    },
                    block: parse_quote! {{
                        unimplemented!()
                    }},
                    attrs: impl_item_attributes.clone(),
                    ..node.clone()
                };

                if is_constructor && self_method {
                    emit_error!(
                        original_signature,
                        "cannot have self methods declared as constructors"
                    );

                    return dummy;
                }

                if env_arg.is_none() {
                    if !self_method {
                        emit_error!(
                            original_signature,
                            "imported static methods must have a parameter of type `&JNIEnv` as first parameter"
                        );
                    } else {
                        emit_error!(
                            original_signature,
                            "imported self methods must have a parameter of type `&JNIEnv` as second parameter"
                        );
                    }
                    return dummy;
                }

                let call_type_attribute = get_call_type(&node);
                let call_type = call_type_attribute
                    .as_ref()
                    .map(|c| &c.call_type)
                    .unwrap_or(&CallType::Safe(None));

                if let Some(CallTypeAttribute { attr, .. }) = &call_type_attribute {
                    if let CallType::Safe(Some(params)) = call_type {
                        if let SafeParams {
                            message: Some(_), ..
                        }
                        | SafeParams {
                            exception_class: Some(_),
                            ..
                        } = params
                        {
                            abort!(attr, "can't have exception message or exception class for imported methods")
                        }
                    }
                }

                let jni_package_path = self
                    .struct_context
                    .package
                    .as_ref()
                    .map(|p| p.to_string())
                    .filter(|s| !s.is_empty())
                    .unwrap_or_else(|| "".into())
                    .replace('.', "/");

                let java_class_path = [jni_package_path, self.struct_context.struct_name.clone()]
                    .iter()
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_owned())
                    .collect::<Vec<_>>()
                    .join("/");
                let java_method_name = to_camel_case(&signature.ident.to_string());

                let input_types_conversions = signature
                    .inputs
                    .iter_mut()
                    .rev()
                    .filter_map(|i| match i {
                        FnArg::Typed(t) => match &*t.pat {
                            Pat::Ident(PatIdent { ident, .. }) if ident == "self" => None,
                            _ => Some((&t.ty, t.ty.span(), &mut t.attrs))
                        },
                        FnArg::Receiver(_) => None,
                    })
                    .map(|(t, span, attrs)| {
                        let override_input_type = attrs.iter().find(|attr| {
                            attr.path().segments.iter().find(|seg| seg.ident.to_string().as_str() == "input_type").is_some()
                        }).and_then(|a| {
                            if let Ok(meta_list) = a.meta.require_list() {
                                let token_tree_lit: Lit = syn::parse2::<Lit>(meta_list.clone().tokens).unwrap();

                                if let Lit::Str(literal) = token_tree_lit {
                                    Some(literal)
                                } else {
                                    None
                                }
                            } else {
                                abort!(a, "Missing argument for `#[input_type]`")
                            }
                        });

                        if let Some(override_input_type) = override_input_type {
                            quote_spanned! { span => #override_input_type, }
                        } else {
                            if let CallType::Safe(_) = call_type {
                                quote_spanned! { span => <#t as ::robusta_jni::convert::TryIntoJavaValue>::SIG_TYPE, }
                            } else {
                                quote_spanned! { span => <#t as ::robusta_jni::convert::IntoJavaValue>::SIG_TYPE, }
                            }
                        }
                    })
                    .fold(TokenStream::new(), |t, mut tok| {
                        t.to_tokens(&mut tok);
                        tok
                    });

                let output_type_span = {
                    match &signature.output {
                        ReturnType::Default => signature.output.span(),
                        ReturnType::Type(_arrow, ref ty) => ty.span(),
                    }
                };

                let override_output_type = get_output_type_override(&node);
                let output_conversion = if let Some(override_output_type) = override_output_type {
                    quote_spanned! { output_type_span => #override_output_type }
                } else {
                    match signature.output {
                        ReturnType::Default => quote_spanned!(signature.output.span() => ),
                        ReturnType::Type(_arrow, ref ty) => {
                            if is_constructor {
                                quote_spanned! { output_type_span => "V" }
                            } else {
                                match call_type {
                                    CallType::Safe(_) => {
                                        let inner_result_ty = match &**ty {
                                            Type::Path(TypePath { path, .. }) => {
                                                path.segments.last().map(|s| match &s.arguments {
                                                    PathArguments::AngleBracketed(a) => {
                                                        match &a.args.first().expect("return type must be `::robusta_jni::jni::errors::Result` when using \"java\" ABI with an implicit or \"safe\" `call_type`") {
                                                            GenericArgument::Type(t) => t,
                                                            _ => abort!(a, "first generic argument in return type must be a type")
                                                        }
                                                    }
                                                    PathArguments::None => {
                                                        let user_attribute_message = call_type_attribute.as_ref().map(|_| "because of this attribute");
                                                        abort!(s, "return type must be `::robusta_jni::jni::errors::Result` when using \"java\" ABI with an implicit or \"safe\" `call_type`";
                                                                        help = "replace `{}` with `Result<{}>`", s.ident, s.ident;
                                                                        help =? call_type_attribute.as_ref().map(|c| c.attr.span()).unwrap() => user_attribute_message)
                                                    }
                                                    _ => abort!(s, "return type must be `::robusta_jni::jni::errors::Result` when using \"java\" ABI with an implicit or \"safe\" `call_type`")
                                                })
                                            }
                                            _ => abort!(ty, "return type must be `::robusta_jni::jni::errors::Result` when using \"java\" ABI with an implicit or \"safe\" `call_type`")
                                        }.unwrap();

                                        quote_spanned! { output_type_span => <#inner_result_ty as ::robusta_jni::convert::TryIntoJavaValue>::SIG_TYPE }
                                    }
                                    CallType::Unchecked(_) => {
                                        if let Type::Path(TypePath { path, .. }) = ty.as_ref() {
                                            if let Some(r) =
                                                path.segments.last().filter(|i| i.ident == "Result")
                                            {
                                                if let PathArguments::AngleBracketed(_) = r.arguments {
                                                    let call_type_span = call_type_attribute
                                                        .as_ref()
                                                        .map(|c| c.attr.span());
                                                    let call_type_hint = call_type_span.map(|_| {
                                                        "maybe you meant `#[call_type(safe)]`?"
                                                    });

                                                    emit_warning!(ty, "using a `Result` type in a `#[call_type(unchecked)]` method";
                                            hint =? call_type_span.unwrap() => call_type_hint)
                                                }
                                            }
                                        }
                                        quote_spanned! { output_type_span => <#ty as ::robusta_jni::convert::IntoJavaValue>::SIG_TYPE }
                                    }
                                }
                            }
                        }
                    }
                };

                let java_signature = quote_spanned! { signature.span() => ["(", #input_types_conversions ")", #output_conversion].join("") };

                let input_conversions = signature.inputs.iter().fold(TokenStream::new(), |mut tok, input| {
                    match input {
                        FnArg::Receiver(_) => { tok }
                        FnArg::Typed(t) => {
                            let ty = &t.ty;
                            let pat: TokenStream = {
                                // The things we do for ~love~ nice compiler errors...
                                // TODO: Check whether there is a better way to force spans onto token streams
                                let pat = &t.pat;
                                let mut p: TokenTree = parse_quote! { #pat };
                                p.set_span(ty.span());
                                p.into()
                            };

                            let conversion: TokenStream = if let CallType::Safe(_) = call_type {
                                quote_spanned! { ty.span() => ::std::convert::Into::into(<#ty as ::robusta_jni::convert::TryIntoJavaValue>::try_into(#pat, &env)?), }
                            } else {
                                quote_spanned! { ty.span() => ::std::convert::Into::into(<#ty as ::robusta_jni::convert::IntoJavaValue>::into(#pat, &env)), }
                            };
                            conversion.to_tokens(&mut tok);
                            tok
                        }
                    }
                });

                let return_expr = match call_type {
                    CallType::Safe(_) => {
                        if is_constructor {
                            quote_spanned! { output_type_span =>
                                res.and_then(|v| ::robusta_jni::convert::TryFromJavaValue::try_from(v, &env))
                            }
                        } else {
                            quote_spanned! { output_type_span =>
                                res.and_then(|v| ::std::convert::TryInto::try_into(::robusta_jni::convert::JValueWrapper::from(v)))
                                   .and_then(|v| ::robusta_jni::convert::TryFromJavaValue::try_from(v, &env))
                            }
                        }
                    }
                    CallType::Unchecked(_) => {
                        if is_constructor {
                            quote_spanned! { output_type_span =>
                                ::robusta_jni::convert::FromJavaValue::from(res, &env)
                            }
                        } else {
                            quote_spanned! { output_type_span =>
                                ::std::convert::TryInto::try_into(::robusta_jni::convert::JValueWrapper::from(res))
                                    .map(|v| ::robusta_jni::convert::FromJavaValue::from(v, &env))
                                    .unwrap()
                            }
                        }
                    }
                };

                let env_ident = match env_arg.unwrap() {
                    FnArg::Typed(t) => {
                        match *t.pat {
                            Pat::Ident(PatIdent { ident, .. }) => ident,
                            _ => panic!("non-ident pat in FnArg")
                        }
                    }
                    _ => panic!("Bug -- please report to library author. Expected env parameter, found receiver")
                };

                let sig_discarded_known_attributes: HashSet<&str> = {
                    let mut h = HashSet::new();
                    h.insert("input_type");

                    h
                };

                let class_arg_ident = if let Some(class_ref_arg) = class_ref_arg {
                    match class_ref_arg {
                        FnArg::Typed(t) => {
                            match *t.pat {
                                Pat::Ident(PatIdent { ident, .. }) => Some(ident),
                                _ => panic!("non-ident pat in FnArg")
                            }
                        },
                        _ => panic!("Bug -- please report to library author. Expected env parameter, found receiver")
                    }
                } else {
                    None
                };

                original_signature.inputs.iter_mut().for_each(|i| match i {
                    FnArg::Typed(t) => match &*t.pat {
                        Pat::Ident(PatIdent { ident, .. }) if ident == "self" => {}
                        _ => {
                            t.attrs = t
                                .attrs
                                .clone()
                                .into_iter()
                                .filter(|a| {
                                    !a.path()
                                        .segments
                                        .iter()
                                        .find(|s| {
                                            sig_discarded_known_attributes
                                                .iter()
                                                .any(|d| s.ident.to_string().contains(d))
                                        })
                                        .is_some()
                                })
                                .collect()
                        }
                    },
                    FnArg::Receiver(_) => {}
                });

                ImplItemFn {
                    sig: Signature {
                        abi: None,
                        ..original_signature
                    },
                    block: if self_method {
                        let self_span = node.sig.inputs.iter().next().unwrap().span();
                        match call_type {
                            CallType::Safe(_) => {
                                parse_quote_spanned! { self_span => {
                                    let env: &'_ ::robusta_jni::jni::JNIEnv<'_> = #env_ident;
                                    let res = env.call_method(::robusta_jni::convert::JavaValue::autobox(::robusta_jni::convert::TryIntoJavaValue::try_into(self, &env)?, &env), #java_method_name, #java_signature, &[#input_conversions]);
                                    #return_expr
                                }}
                            }
                            CallType::Unchecked(_) => {
                                parse_quote_spanned! { self_span => {
                                    let env: &'_ ::robusta_jni::jni::JNIEnv<'_> = #env_ident;
                                    let res = env.call_method(::robusta_jni::convert::JavaValue::autobox(::robusta_jni::convert::IntoJavaValue::into(self, &env), &env), #java_method_name, #java_signature, &[#input_conversions]).unwrap();
                                    #return_expr
                                }}
                            }
                        }
                    } else {
                        match call_type {
                            CallType::Safe(_) => {
                                if is_constructor {
                                    if let Some(class_arg_ident) = class_arg_ident {
                                        parse_quote! {{
                                            let env: &'_ ::robusta_jni::jni::JNIEnv<'_> = #env_ident;
                                            let res = env.new_object(#class_arg_ident, #java_signature, &[#input_conversions]);
                                            #return_expr
                                        }}
                                    } else {
                                        parse_quote! {{
                                            let env: &'_ ::robusta_jni::jni::JNIEnv<'_> = #env_ident;
                                            let res = env.new_object(#java_class_path, #java_signature, &[#input_conversions]);
                                            #return_expr
                                        }}
                                    }
                                } else {
                                    if let Some(class_arg_ident) = class_arg_ident {
                                        parse_quote! {{
                                            let env: &'_ ::robusta_jni::jni::JNIEnv<'_> = #env_ident;
                                            let res = env.call_static_method(#class_arg_ident, #java_method_name, #java_signature, &[#input_conversions]);
                                            #return_expr
                                        }}
                                    } else {
                                        parse_quote! {{
                                            let env: &'_ ::robusta_jni::jni::JNIEnv<'_> = #env_ident;
                                            let res = env.call_static_method(#java_class_path, #java_method_name, #java_signature, &[#input_conversions]);
                                            #return_expr
                                        }}
                                    }
                                }
                            }
                            CallType::Unchecked(_) => {
                                if is_constructor {
                                    if let Some(class_arg_ident) = class_arg_ident {
                                        parse_quote! {{
                                            let env: &'_ ::robusta_jni::jni::JNIEnv<'_> = #env_ident;
                                            let res = env.new_object(#class_arg_ident, #java_signature, &[#input_conversions]).unwrap();
                                            #return_expr
                                        }}
                                    } else {
                                        parse_quote! {{
                                            let env: &'_ ::robusta_jni::jni::JNIEnv<'_> = #env_ident;
                                            let res = env.new_object(#java_class_path, #java_signature, &[#input_conversions]).unwrap();
                                            #return_expr
                                        }}
                                    }
                                } else {
                                    if let Some(class_arg_ident) = class_arg_ident {
                                        parse_quote! {{
                                            let env: &'_ ::robusta_jni::jni::JNIEnv<'_> = #env_ident;
                                            let res = env.call_static_method(#class_arg_ident, #java_method_name, #java_signature, &[#input_conversions]).unwrap();
                                            #return_expr
                                        }}
                                    } else {
                                        parse_quote! {{
                                            let env: &'_ ::robusta_jni::jni::JNIEnv<'_> = #env_ident;
                                            let res = env.call_static_method(#java_class_path, #java_method_name, #java_signature, &[#input_conversions]).unwrap();
                                            #return_expr
                                        }}
                                    }
                                }
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
