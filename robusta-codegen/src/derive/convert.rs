use std::collections::HashMap;

use proc_macro2::{TokenStream, Ident};
use proc_macro_error::{abort, emit_error, emit_warning};
use quote::{quote, quote_spanned};
use syn::{Data, DataStruct, DeriveInput, GenericParam, LifetimeDef, Type, TypePath, PathArguments, GenericArgument, AngleBracketedGenericArguments, Generics};
use syn::spanned::Spanned;
use crate::derive::utils::generic_params_to_args;

struct TraitAutoDeriveData {
    instance_field_type_assertion: TokenStream,
    ident: Ident,
    generics: Generics,
    instance_ident: Ident,
    generic_args: AngleBracketedGenericArguments
}

pub(crate) fn into_java_value_macro_derive(input: DeriveInput) -> TokenStream {
    let input_span = input.span();
    match into_java_value_macro_derive_impl(input) {
        Ok(t) => t,
        Err(_) => quote_spanned! { input_span => }
    }
}

fn into_java_value_macro_derive_impl(input: DeriveInput) -> syn::Result<TokenStream> {
    let TraitAutoDeriveData {
        instance_field_type_assertion,
        ident,
        generics,
        instance_ident,
        generic_args
    } = get_trait_impl_components("IntoJavaValue", input);

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

pub(crate) fn tryinto_java_value_macro_derive(input: DeriveInput) -> TokenStream {
    let input_span = input.span();
    match tryinto_java_value_macro_derive_impl(input) {
        Ok(t) => t,
        Err(_) => quote_spanned! { input_span => }
    }
}

fn tryinto_java_value_macro_derive_impl(input: DeriveInput) -> syn::Result<TokenStream> {
    let TraitAutoDeriveData {
        instance_field_type_assertion,
        ident,
        generics,
        instance_ident,
        generic_args
    } = get_trait_impl_components("TryIntoJavaValue", input);

    Ok(quote! {
        #instance_field_type_assertion

        #[automatically_derived]
        impl#generics ::robusta_jni::convert::TryIntoJavaValue<'env> for #ident#generic_args {
            type Target = ::robusta_jni::jni::objects::JObject<'env>;

            fn try_into(self, env: &::robusta_jni::jni::JNIEnv<'env>) -> ::robusta_jni::jni::errors::Result<Self::Target> {
                ::robusta_jni::convert::TryIntoJavaValue::try_into(self, env)
            }
        }

        #[automatically_derived]
        impl#generics ::robusta_jni::convert::TryIntoJavaValue<'env> for &#ident#generic_args {
            type Target = ::robusta_jni::jni::objects::JObject<'env>;

            fn try_into(self, env: &::robusta_jni::jni::JNIEnv<'env>) -> ::robusta_jni::jni::errors::Result<Self::Target> {
                Ok(self.#instance_ident.as_obj())
            }
        }

        #[automatically_derived]
        impl#generics ::robusta_jni::convert::TryIntoJavaValue<'env> for &mut #ident#generic_args {
            type Target = ::robusta_jni::jni::objects::JObject<'env>;

            fn try_into(self, env: &::robusta_jni::jni::JNIEnv<'env>) -> ::robusta_jni::jni::errors::Result<Self::Target> {
                ::robusta_jni::convert::TryIntoJavaValue::try_into(self, env)
            }
        }
    })
}

fn get_trait_impl_components(trait_name: &str, input: DeriveInput) -> TraitAutoDeriveData {
    let input_span = input.span();

    match input.data {
        Data::Struct(DataStruct { fields, .. }) => {
            let lifetimes: HashMap<String, &LifetimeDef> = input.generics.params.iter().filter_map(|g| match g {
                GenericParam::Lifetime(l) => Some(l),
                _ => None
            }).map(|l| {
                (l.lifetime.ident.to_string(), l)
            }).collect();

            match (lifetimes.get("env"), lifetimes.get("borrow")) {
                (Some(env_lifetime), Some(borrow_lifetime)) => {
                    if !env_lifetime.bounds.iter().any(|l| *l == borrow_lifetime.lifetime) {
                        emit_error!(env_lifetime, "`'env` lifetime must have a `'borrow` lifetime bound";
                                    help = "try adding `'env: 'borrow`")
                    }
                },
                _ => emit_error!(input_span, "deriving struct must have `'env` and `'borrow` lifetime parameters")
            }

            let instance_fields: Vec<_> = fields.iter().filter_map(|f| {
                let attr = f.attrs.iter().find(|a| a.path.get_ident().map(|i| i.to_string()).as_deref() == Some("instance"));
                attr.map(|a| (f, a))
            }).collect();

            if instance_fields.len() > 1 {
                emit_error!(input_span, "cannot have more than one `#[instance]` attribute")
            }

            let instance_field_data = instance_fields.get(0);

            match instance_field_data {
                None => abort!(input_span, "missing `#[instance] field attribute"),
                Some((instance, attr)) => {
                    if !attr.tokens.is_empty() {
                        emit_warning!(attr.tokens, "`#[instance]` attribute doesn't have any arguments")
                    }

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
                    let instance_span = instance.span();
                    let instance_ident = instance.ident.as_ref().unwrap_or_else(|| {
                        abort!(instance_span, "instance field must have a name")
                    });

                    let generic_args = generic_params_to_args(generics.clone());

                    TraitAutoDeriveData {
                        instance_field_type_assertion,
                        ident,
                        generics,
                        instance_ident: instance_ident.clone(),
                        generic_args
                    }
                }
            }
        },
        _ => abort!(input, "`{}` auto-derive implemented for structs only", trait_name),
    }
}
