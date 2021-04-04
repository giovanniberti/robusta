use std::collections::HashMap;

use proc_macro2::{TokenStream, Ident};
use proc_macro_error::{abort, emit_error};
use quote::{quote, quote_spanned};
use syn::{Data, DataStruct, DeriveInput, GenericParam, LifetimeDef, Type, TypePath, PathArguments, GenericArgument};
use syn::spanned::Spanned;
use crate::derive::utils::generic_params_to_args;

pub(crate) fn into_java_value_macro_derive(input: DeriveInput) -> TokenStream {
    let input_span = input.span();
    match into_java_value_macro_derive_impl(input) {
        Ok(t) => t,
        Err(_) => quote_spanned! { input_span => }
    }
}

fn into_java_value_macro_derive_impl(input: DeriveInput) -> syn::Result<TokenStream> {
    let input_span = input.span();

    let lifetimes: HashMap<String, &LifetimeDef> = input.generics.params.iter().filter_map(|g| match g {
        GenericParam::Lifetime(l) => Some(l),
        _ => None
    }).map(|l| {
        (l.lifetime.ident.to_string(), l)
    }).collect();

    match (lifetimes.get("env"), lifetimes.get("borrow")) {
        (Some(env_lifetime), Some(borrow_lifetime)) => {
            if env_lifetime.bounds.iter().find(|l| **l == borrow_lifetime.lifetime).is_none() {
                emit_error!(env_lifetime, "`'env` lifetime must have a `'borrow` lifetime bound";
                    help = "try adding `'env: 'borrow`")
            }
        },
        _ => emit_error!(input, "deriving struct must have `'env` and `'borrow` lifetime parameters")
    }

    match input.data {
        Data::Struct(DataStruct { fields, .. }) => {

            let instance_field = fields.iter().find(|f| {
                f.attrs.iter().find(|a| a.path.get_ident().map(|i| i.to_string()).as_deref() == Some("instance")).is_some()
            });

            match instance_field {
                None => abort!(input_span, "missing `#[instance] field attribute"),
                Some(instance) => {
                    let ty = {
                        let mut t = instance.ty.clone();
                        if let Type::Path(TypePath { path, .. }) = &mut t {
                            path.segments.iter_mut().for_each(|s|
                                if let PathArguments::AngleBracketed(a) = &mut s.arguments {
                                    a.args.iter_mut().for_each(|g| {
                                        if let GenericArgument::Lifetime(l) = g {
                                            l.ident = Ident::new("static", l.span());
                                        }
                                    })
                                });
                        }

                        t
                    };

                    let instance_field_type_assertion = quote_spanned! { ty.span() =>
                        ::robusta_jni::assert_type_eq_all!(#ty, ::robusta_jni::jni::objects::AutoLocal<'static, 'static>);
                    };

                    let ident = input.ident;
                    let generics = input.generics;
                    let instance_ident = instance.ident.as_ref().unwrap_or_else(|| {
                        abort!(instance, "instance field must have a name")
                    });

                    let generic_args = generic_params_to_args(generics.clone());

                    Ok(quote! {
                        #instance_field_type_assertion

                        #[automatically_derived]
                        impl#generics ::robusta_jni::convert::IntoJavaValue<'env> for #ident#generic_args {
                            type Target = ::robusta_jni::jni::objects::JObject<'env>;

                            fn into(self, env: &::robusta_jni::jni::JNIEnv<'env>) -> Self::Target {
                                ::robusta_jni::convert::IntoJavaValue::into(self, env)
                            }
                        }

                        #[automatically_derived]
                        impl#generics ::robusta_jni::convert::IntoJavaValue<'env> for &#ident#generic_args {
                            type Target = ::robusta_jni::jni::objects::JObject<'env>;

                            fn into(self, env: &::robusta_jni::jni::JNIEnv<'env>) -> Self::Target {
                                self.#instance_ident.as_obj()
                            }
                        }

                        #[automatically_derived]
                        impl#generics ::robusta_jni::convert::IntoJavaValue<'env> for &mut #ident#generic_args {
                            type Target = ::robusta_jni::jni::objects::JObject<'env>;

                            fn into(self, env: &::robusta_jni::jni::JNIEnv<'env>) -> Self::Target {
                                ::robusta_jni::convert::IntoJavaValue::into(self, env)
                            }
                        }
                    })
                }
            }
        },
        _ => abort!(input_span, "`IntoJavaValue` auto-derive implemented for structs only"),
    }
}