use std::collections::HashSet;

use proc_macro2::Ident;
use proc_macro_error::{emit_error, emit_warning};
use quote::ToTokens;
use syn::fold::Fold;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Extern;
use syn::Lifetime;
use syn::Token;
use syn::{parse_quote, GenericParam, Generics, LifetimeParam, TypeTuple};
use syn::{
    Abi, Block, Expr, FnArg, ImplItemFn, LitStr, Pat, PatIdent, PatType, ReturnType, Signature,
    Type, Visibility,
};

use crate::transformation::context::StructContext;
use crate::transformation::utils::get_call_type;
use crate::transformation::{CallType, FreestandingTransformer, SafeParams};
use crate::utils::{get_abi, get_env_arg, is_self_method};
use std::iter::FromIterator;

pub struct ExportedMethodTransformer<'ctx> {
    pub(crate) struct_context: &'ctx StructContext,
}

impl<'ctx> Fold for ExportedMethodTransformer<'ctx> {
    fn fold_impl_item_fn(&mut self, node: ImplItemFn) -> ImplItemFn {
        let abi = get_abi(&node.sig);
        match (&node.vis, &abi.as_deref()) {
            (Visibility::Public(_), Some("jni")) => {
                let call_type_attribute = get_call_type(&node)
                    .map(|c| c.call_type)
                    .unwrap_or(CallType::Safe(None));

                let mut jni_method_transformer =
                    ExternJNIMethodTransformer::new(self.struct_context, call_type_attribute);
                jni_method_transformer.fold_impl_item_fn(node)
            }
            _ => node,
        }
    }
}

struct ExternJNIMethodTransformer<'ctx> {
    struct_context: &'ctx StructContext,
    call_type: CallType,
}

impl<'ctx> ExternJNIMethodTransformer<'ctx> {
    fn new(struct_context: &'ctx StructContext, call_type: CallType) -> Self {
        ExternJNIMethodTransformer {
            struct_context,
            call_type,
        }
    }
}

impl<'ctx> Fold for ExternJNIMethodTransformer<'ctx> {
    fn fold_impl_item_fn(&mut self, node: ImplItemFn) -> ImplItemFn {
        let jni_signature = JNISignature::new(
            node.sig.clone(),
            &self.struct_context,
            self.call_type.clone(),
        );

        let transformed_jni_signature = jni_signature.transformed_signature();
        let method_call = jni_signature.signature_call();

        let new_block: Block = match &self.call_type {
            CallType::Unchecked { .. } => {
                parse_quote_spanned! { node.span() => {
                    ::robusta_jni::convert::IntoJavaValue::into(#method_call, &env)
                }}
            }

            CallType::Safe(exception_details) => {
                let outer_call_inputs = {
                    let mut inputs: Punctuated<Expr, Token![,]> = jni_signature
                        .args_iter()
                        .map(|p| -> Expr {
                            let PatType { pat, .. } = p;

                            match &**pat {
                                Pat::Ident(PatIdent { ident, .. }) => {
                                    parse_quote_spanned!(ident.span() => #ident)
                                }
                                _ => panic!("Non-identifier argument pattern in function"),
                            }
                        })
                        .collect();

                    inputs.push(parse_quote!(&env));
                    inputs
                };
                let outer_signature = {
                    let mut s = transformed_jni_signature.clone();
                    s.ident = Ident::new("outer", s.ident.span());

                    s.inputs.iter_mut().for_each(|i| {
                        if let FnArg::Typed(PatType { pat, .. }) = i {
                            if let Pat::Ident(PatIdent { mutability, .. }) = pat.as_mut() {
                                *mutability = None
                            }
                        }
                    });

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
                        Box::new(
                            parse_quote_spanned!(outer_output_type.span() => ::robusta_jni::jni::errors::Result<#outer_output_type>),
                        ),
                    );
                    s.abi = None;
                    s
                };

                let (default_exception_class, default_message) = (
                    "java.lang.RuntimeException".parse().unwrap(),
                    "JNI call error!",
                );
                let (exception_class, message) = match exception_details {
                    Some(SafeParams {
                        exception_class,
                        message,
                    }) => {
                        let exception_class_result =
                            exception_class.as_ref().unwrap_or(&default_exception_class);
                        let message_result = message.as_deref().unwrap_or(default_message);

                        (exception_class_result, message_result)
                    }
                    None => (&default_exception_class, default_message),
                };

                let exception_classpath_path = exception_class.to_classpath_path();

                parse_quote_spanned! { node.span() => {
                    #outer_signature {
                        ::robusta_jni::convert::TryIntoJavaValue::try_into(#method_call, &env)
                    }

                    match outer(#outer_call_inputs) {
                        Ok(result) => result,
                        Err(e) => {
                            let r = env.throw_new(#exception_classpath_path, format!("{}. Cause: {}", #message, e));

                            if let Err(e) = r {
                                println!("Error while throwing Java exception: {}", e);
                            }

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
                        .contains(&a.path().segments.to_token_stream().to_string().as_str())
                })
                .collect()
        };

        let node_span = node.span();
        ImplItemFn {
            attrs: impl_item_attributes,
            vis: Visibility::Public(Token![pub](node_span)),
            defaultness: node.defaultness,
            sig: self.fold_signature(node.sig),
            block: new_block,
        }
    }

    /// Transform original signature in JNI-ready one, including JClass and JNIEnv parameters into the function signature.
    fn fold_signature(&mut self, node: Signature) -> Signature {
        let jni_signature =
            JNISignature::new(node.clone(), &self.struct_context, self.call_type.clone());

        let mut sig = jni_signature.transformed_signature;

        if sig.ident.to_string().contains('_') {
            emit_error!(sig.ident, "JNI methods cannot contain `_` character");
        }

        let jni_method_name = {
            let snake_case_package = self
                .struct_context
                .package
                .as_ref()
                .map(|s| s.to_snake_case())
                .unwrap_or_else(|| "".into());

            [
                "Java",
                &snake_case_package,
                &self.struct_context.struct_name,
                &sig.ident.to_string(),
            ]
            .iter()
            .filter(|s| !s.is_empty())
            .map(|s| s.to_owned())
            .collect::<Vec<_>>()
            .join("_")
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
    use std::str::FromStr;

    use proc_macro2::TokenStream;

    use super::*;
    use crate::transformation::JavaPath;

    fn setup_package(
        package: Option<JavaPath>,
        struct_name: String,
        method_name: String,
    ) -> ImplItemFn {
        let struct_name_token_stream = TokenStream::from_str(&struct_name).unwrap();
        let method_name_token_stream = TokenStream::from_str(&method_name).unwrap();

        let method: ImplItemFn =
            parse_quote! { pub extern "jni" fn #method_name_token_stream() {} };
        let struct_context = StructContext {
            struct_type: parse_quote! { #struct_name_token_stream },
            struct_name,
            struct_lifetimes: vec![],
            package,
        };
        let mut transformer = ExternJNIMethodTransformer {
            struct_context: &struct_context,
            call_type: CallType::Safe(None),
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
        assert_eq!(
            output_no_package.sig.ident.to_string(),
            format!("Java_Foo_foo")
        );

        let output_with_package = setup_package(
            Some(JavaPath::from_str("com.bar.quux").unwrap()),
            "Foo".into(),
            "foo".into(),
        );
        assert_eq!(
            output_with_package.sig.ident.to_string(),
            format!("Java_com_bar_quux_Foo_foo")
        );
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

    fn setup_with_params(params: TokenStream, struct_name: String) -> ImplItemFn {
        let package = None;
        let method_name = "foo".to_string();
        let struct_name_token_stream = TokenStream::from_str(&struct_name).unwrap();
        let method_name_token_stream = TokenStream::from_str(&method_name).unwrap();

        let method: ImplItemFn = parse_quote! {
            pub extern "jni" fn #method_name_token_stream(#params) -> i32 {}
        };

        let struct_context = StructContext {
            struct_type: parse_quote! { #struct_name_token_stream },
            struct_name,
            struct_lifetimes: vec![],
            package,
        };
        let mut transformer = ExternJNIMethodTransformer {
            struct_context: &struct_context,
            call_type: CallType::Safe(None),
        };

        transformer.fold_impl_item_method(method)
    }

    #[test]
    fn static_method_params() {
        use quote::quote;

        let param_type_1: TokenStream = parse_quote! { i32 };
        let param_type_2: TokenStream = parse_quote! { FooBar };
        let output = setup_with_params(
            quote! { _1: #param_type_1, _2: #param_type_2 },
            "Foo".to_string(),
        );

        let env_type: Type = parse_quote! { ::robusta_jni::jni::JNIEnv<'env> };
        let class_type: Type = parse_quote! { ::robusta_jni::jni::objects::JClass };
        let conv_type_1: Type = parse_quote! { <#param_type_1 as ::robusta_jni::convert::TryFromJavaValue<'env, 'borrow>>::Source };
        let conv_type_2: Type = parse_quote! { <#param_type_2 as ::robusta_jni::convert::TryFromJavaValue<'env, 'borrow>>::Source };

        let args: &[FnArg] = &output.sig.inputs.into_iter().collect::<Vec<_>>();
        match args {
            [FnArg::Typed(PatType { ty: ty_env, .. }), FnArg::Typed(PatType { ty: ty_class, .. }), FnArg::Typed(PatType { ty: ty_1, .. }), FnArg::Typed(PatType { ty: ty_2, .. })] =>
            {
                assert_eq!(
                    ty_env.to_token_stream().to_string(),
                    env_type.to_token_stream().to_string()
                );
                assert_eq!(
                    ty_class.to_token_stream().to_string(),
                    class_type.to_token_stream().to_string()
                );
                assert_eq!(
                    ty_1.to_token_stream().to_string(),
                    conv_type_1.to_token_stream().to_string()
                );
                assert_eq!(
                    ty_2.to_token_stream().to_string(),
                    conv_type_2.to_token_stream().to_string()
                );
            }

            _ => assert!(false),
        }
    }

    #[test]
    fn self_method_params() {
        use quote::quote;

        let struct_name = "Foo".to_string();
        let struct_name_toks = TokenStream::from_str(&struct_name).unwrap();

        let param_type_1: TokenStream = parse_quote! { i32 };
        let param_type_2: TokenStream = parse_quote! { FooBar };
        let output = setup_with_params(
            quote! { self, _1: #param_type_1, _2: #param_type_2 },
            struct_name.clone(),
        );

        let env_type: Type = parse_quote! { ::robusta_jni::jni::JNIEnv<'env> };
        let self_conv_type: Type = parse_quote! { <#struct_name_toks as ::robusta_jni::convert::TryFromJavaValue<'env, 'borrow>>::Source };
        let conv_type_1: Type = parse_quote! { <#param_type_1 as ::robusta_jni::convert::TryFromJavaValue<'env, 'borrow>>::Source };
        let conv_type_2: Type = parse_quote! { <#param_type_2 as ::robusta_jni::convert::TryFromJavaValue<'env, 'borrow>>::Source };

        let args: &[FnArg] = &output.sig.inputs.into_iter().collect::<Vec<_>>();
        match args {
            [FnArg::Typed(PatType { ty: ty_env, .. }), FnArg::Typed(PatType { ty: ty_self, .. }), FnArg::Typed(PatType { ty: ty_1, .. }), FnArg::Typed(PatType { ty: ty_2, .. })] =>
            {
                assert_eq!(
                    ty_env.to_token_stream().to_string(),
                    env_type.to_token_stream().to_string()
                );
                assert_eq!(
                    ty_self.to_token_stream().to_string(),
                    self_conv_type.to_token_stream().to_string()
                );
                assert_eq!(
                    ty_1.to_token_stream().to_string(),
                    conv_type_1.to_token_stream().to_string()
                );
                assert_eq!(
                    ty_2.to_token_stream().to_string(),
                    conv_type_2.to_token_stream().to_string()
                );
            }

            _ => assert!(false),
        }
    }
}

struct JNISignatureTransformer {
    struct_freestanding_transformer: FreestandingTransformer,
    struct_lifetimes: Vec<LifetimeParam>,
    call_type: CallType,
}

impl JNISignatureTransformer {
    fn new(
        struct_freestanding_transformer: FreestandingTransformer,
        struct_lifetimes: Vec<LifetimeParam>,
        call_type: CallType,
    ) -> Self {
        JNISignatureTransformer {
            struct_freestanding_transformer,
            struct_lifetimes,
            call_type,
        }
    }

    fn transform_generics(&mut self, mut generics: Generics) -> Generics {
        let generics_span = generics.span();
        generics.params.extend(
            self.struct_lifetimes
                .iter()
                .cloned()
                .map(GenericParam::Lifetime),
        );

        let (env_lifetime, borrow_lifetime) = generics.params.iter_mut().fold((None, None), |acc, l| {
            match l {
                GenericParam::Lifetime(l) => {
                    if l.lifetime.ident == "env" {
                        if l.bounds.iter().any(|b| b.ident != "borrow") {
                            emit_warning!(l, "using JNI-reserved `'env` lifetime with non `'borrow` bounds";
                                note = "If you need to access to the lifetime of the `JNIEnv`, please use `'borrow` instead")
                        }

                        (Some(l), acc.1)
                    } else if l.lifetime.ident == "borrow" {
                        (acc.0, Some(l))
                    } else {
                        acc
                    }
                },
                _ => acc
            }
        });

        match (env_lifetime, borrow_lifetime) {
            (Some(_), Some(_)) => {}
            (Some(e), None) => {
                let borrow_lifetime_value = Lifetime {
                    apostrophe: generics_span,
                    ident: Ident::new("borrow", generics_span),
                };

                e.bounds.push(borrow_lifetime_value.clone());

                generics.params.push(GenericParam::Lifetime(LifetimeParam {
                    attrs: vec![],
                    lifetime: borrow_lifetime_value,
                    colon_token: None,
                    bounds: Default::default(),
                }))
            }
            (None, Some(l)) => {
                emit_error!(l, "Can't use JNI-reserved `'borrow` lifetime without accompanying `'env: 'borrow` lifetime";
                    help = "Add `'env: 'borrow` lifetime here")
            }
            (None, None) => {
                let borrow_lifetime_value = Lifetime {
                    apostrophe: generics_span,
                    ident: Ident::new("borrow", generics_span),
                };

                generics.params.push(GenericParam::Lifetime(LifetimeParam {
                    attrs: vec![],
                    lifetime: Lifetime {
                        apostrophe: generics_span,
                        ident: Ident::new("env", generics_span),
                    },
                    colon_token: None,
                    bounds: {
                        let mut p = Punctuated::new();
                        p.push(borrow_lifetime_value.clone());
                        p
                    },
                }));

                generics.params.push(GenericParam::Lifetime(LifetimeParam {
                    attrs: vec![],
                    lifetime: borrow_lifetime_value,
                    colon_token: None,
                    bounds: Default::default(),
                }))
            }
        }

        generics
    }
}

impl Fold for JNISignatureTransformer {
    fn fold_fn_arg(&mut self, arg: FnArg) -> FnArg {
        match self.struct_freestanding_transformer.fold_fn_arg(arg) {
            FnArg::Receiver(_) => panic!("Bug -- please report to library author. Found receiver input after freestanding conversion"),
            FnArg::Typed(mut t) => {
                let original_input_type = t.ty;

                let jni_conversion_type: Type = match self.call_type {
                    CallType::Safe(_) => parse_quote_spanned! { original_input_type.span() => <#original_input_type as ::robusta_jni::convert::TryFromJavaValue<'env, 'borrow>>::Source },
                    CallType::Unchecked { .. } => parse_quote_spanned! { original_input_type.span() => <#original_input_type as ::robusta_jni::convert::FromJavaValue<'env, 'borrow>>::Source },
                };

                if let Pat::Ident(PatIdent { mutability, .. }) = t.pat.as_mut() {
                    *mutability = None
                }

                FnArg::Typed(PatType {
                    attrs: t.attrs,
                    pat: t.pat,
                    colon_token: t.colon_token,
                    ty: Box::new(jni_conversion_type),
                })
            }
        }
    }

    fn fold_return_type(&mut self, return_type: ReturnType) -> ReturnType {
        match return_type {
            ReturnType::Default => return_type,
            ReturnType::Type(ref arrow, ref rtype) => match (&**rtype, self.call_type.clone()) {
                (Type::Path(p), CallType::Unchecked { .. }) => ReturnType::Type(
                    *arrow,
                    parse_quote_spanned! { p.span() => <#p as ::robusta_jni::convert::IntoJavaValue<'env>>::Target },
                ),

                (Type::Path(p), CallType::Safe(_)) => ReturnType::Type(
                    *arrow,
                    parse_quote_spanned! { p.span() => <#p as ::robusta_jni::convert::TryIntoJavaValue<'env>>::Target },
                ),

                (Type::Reference(r), CallType::Unchecked { .. }) => ReturnType::Type(
                    *arrow,
                    parse_quote_spanned! { r.span() => <#r as ::robusta_jni::convert::IntoJavaValue<'env>>::Target },
                ),

                (Type::Reference(r), CallType::Safe(_)) => ReturnType::Type(
                    *arrow,
                    parse_quote_spanned! { r.span() => <#r as ::robusta_jni::convert::TryIntoJavaValue<'env>>::Target },
                ),
                (Type::Tuple(TypeTuple { elems, .. }), _) if elems.is_empty() => {
                    ReturnType::Default
                }
                _ => {
                    emit_error!(return_type, "Only type or type paths are permitted as type ascriptions in function params");
                    return_type
                }
            },
        }
    }

    fn fold_signature(&mut self, node: Signature) -> Signature {
        Signature {
            abi: node.abi.map(|a| self.fold_abi(a)),
            ident: self.fold_ident(node.ident),
            generics: self.transform_generics(node.generics),
            inputs: node
                .inputs
                .into_iter()
                .map(|f| self.fold_fn_arg(f))
                .collect(),
            variadic: node.variadic.map(|v| self.fold_variadic(v)),
            output: self.fold_return_type(node.output),
            ..node
        }
    }
}

struct JNISignature {
    transformed_signature: Signature,
    call_type: CallType,
    struct_name: String,
    self_method: bool,
    env_arg: Option<FnArg>,
}

impl JNISignature {
    fn new(
        signature: Signature,
        struct_context: &StructContext,
        call_type: CallType,
    ) -> JNISignature {
        let freestanding_transformer =
            FreestandingTransformer::new(struct_context.struct_type.clone());
        let mut jni_signature_transformer = JNISignatureTransformer::new(
            freestanding_transformer,
            struct_context.struct_lifetimes.clone(),
            call_type.clone(),
        );

        let self_method = is_self_method(&signature);
        let (transformed_signature, env_arg) = get_env_arg(signature);

        let transformed_signature = jni_signature_transformer.fold_signature(transformed_signature);

        JNISignature {
            transformed_signature,
            call_type,
            struct_name: struct_context.struct_name.clone(),
            self_method,
            env_arg,
        }
    }

    fn args_iter(&self) -> impl Iterator<Item = &PatType> {
        self.transformed_signature.inputs.iter()
            .map(|a| match a {
                FnArg::Receiver(_) => panic!("Bug -- please report to library author. Found receiver type in freestanding signature!"),
                FnArg::Typed(p) => p
            })
    }

    fn signature_call(&self) -> Expr {
        let method_call_inputs: Punctuated<Expr, Token![,]> = {
            let mut result: Vec<_> = self.args_iter()
                .map(|p| {
                    match p.pat.as_ref() {
                        Pat::Ident(PatIdent { ident, .. }) => {
                            let input_param: Expr = {
                                match self.call_type {
                                    CallType::Safe(_) => parse_quote_spanned! { ident.span() => ::robusta_jni::convert::TryFromJavaValue::try_from(#ident, &env)? },
                                    CallType::Unchecked { .. } => parse_quote_spanned! { ident.span() => ::robusta_jni::convert::FromJavaValue::from(#ident, &env) }
                                }
                            };
                            input_param
                        }
                        _ => panic!("Bug -- please report to library author. Found non-ident FnArg pattern")
                    }
            }).collect();

            if let Some(ref e) = self.env_arg {
                // because `self` is kept in the transformed JNI signature, if this is a `self` method we put `env` *after* self, otherwise the env parameter must be first
                let idx = if self.self_method { 1 } else { 0 };
                let env_span = e.span();
                result.insert(idx, parse_quote_spanned!(env_span => &env));
            }

            Punctuated::from_iter(result.into_iter())
        };

        let signature_span = self.transformed_signature.span();
        let struct_name = Ident::new(&self.struct_name, signature_span);
        let method_name = self.transformed_signature.ident.clone();

        parse_quote_spanned! { signature_span =>
            #struct_name::#method_name(#method_call_inputs)
        }
    }

    fn transformed_signature(&self) -> &Signature {
        &self.transformed_signature
    }
}
