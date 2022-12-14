use std::iter;

use syn::{
    parse_quote, FnArg, Pat, PatIdent, PatType, Path, PathArguments, Signature, Type,
    TypeReference,
};
use proc_macro_error::emit_error;

pub fn canonicalize_path(path: &Path) -> Path {
    let mut result = path.clone();
    result.segments = result
        .segments
        .into_iter()
        .map(|mut seg| {
            seg.arguments = PathArguments::None;
            seg
        })
        .collect();

    result
}

pub fn is_self_method(signature: &Signature) -> bool {
    signature.inputs.iter().any(|i| match i {
        FnArg::Receiver(_) => true,
        FnArg::Typed(t) => match &*t.pat {
            Pat::Ident(PatIdent { ident, .. }) => ident == "self",
            _ => false,
        },
    })
}

pub fn get_env_arg(signature: Signature) -> (Signature, Option<FnArg>) {
    let self_method = is_self_method(&signature);

    // Check whether second argument (first exluding self) is of type &JNIEnv, if so we take it out from the signature
    let possible_env_arg = if !self_method {
        signature.inputs.iter().next()
    } else {
        signature.inputs.iter().nth(1)
    };

    let has_explicit_env_arg = if let Some(FnArg::Typed(PatType { ty, .. })) = possible_env_arg {
        if let Type::Reference(TypeReference { elem, .. }) = &**ty {
            if let Type::Path(t) = &**elem {
                let full_path: Path = parse_quote! { ::robusta_jni::jni::JNIEnv };
                let imported_path: Path = parse_quote! { JNIEnv };
                let canonicalized_type_path = canonicalize_path(&t.path);

                canonicalized_type_path == imported_path || canonicalized_type_path == full_path
            } else {
                false
            }
        } else if let Type::Path(t) = &**ty {
            /* If the user has input `env: JNIEnv` instead of `env: &JNIEnv`, we let her know. */
            let full_path: Path = parse_quote! { ::robusta_jni::jni::JNIEnv };
            let imported_path: Path = parse_quote! { JNIEnv };
            let canonicalized_type_path = canonicalize_path(&t.path);

            if canonicalized_type_path == imported_path || canonicalized_type_path == full_path {
                emit_error!(t, "explicit environment parameter must be of type `&JNIEnv`");
            }

            false
        } else {
            false
        }
    } else {
        false
    };

    let (transformed_signature, env_arg): (Signature, Option<FnArg>) = if has_explicit_env_arg {
        let mut inner_signature = signature;

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
        (signature, None)
    };

    (transformed_signature, env_arg)
}

pub fn get_class_arg_if_any(signature: Signature) -> (Signature, Option<FnArg>) {
    let has_explicit_class_ref_arg = if let Some(FnArg::Typed(PatType { ty, .. })) = signature.inputs.iter().next() {
        if let Type::Reference(TypeReference { elem, .. }) = &**ty {
            if let Type::Path(t) = &**elem {
                let full_path: Path = parse_quote! { ::robusta_jni::jni::objects::GlobalRef };
                let imported_path: Path = parse_quote! { GlobalRef };
                let canonicalized_type_path = canonicalize_path(&t.path);

                canonicalized_type_path == imported_path || canonicalized_type_path == full_path
            } else {
                false
            }
        } else if let Type::Path(t) = &**ty {
            /* If the user has input `class_ref: GlobalRef` instead of `class_ref: &GlobalRef`, we let her know. */
            let full_path: Path = parse_quote! { ::robusta_jni::jni::objects::GlobalRef };
            let imported_path: Path = parse_quote! { GlobalRef };
            let canonicalized_type_path = canonicalize_path(&t.path);

            if canonicalized_type_path == imported_path || canonicalized_type_path == full_path {
                emit_error!(t, "explicit environment parameter must be of type `&GlobalRef`");
            }

            false
        } else {
            false
        }
    } else {
        false
    };

    let (transformed_signature, class_arg): (Signature, Option<FnArg>) = if has_explicit_class_ref_arg {
        let mut inner_signature = signature;

        let mut iter = inner_signature.inputs.into_iter();

        let class_arg = iter.next();

        inner_signature.inputs = iter.collect();

        (inner_signature, class_arg)

    } else {
        (signature, None)
    };

    (transformed_signature, class_arg)
}

pub fn get_abi(sig: &Signature) -> Option<String> {
    sig
        .abi
        .as_ref()
        .and_then(|l| l.name.as_ref().map(|n| n.value()))
}