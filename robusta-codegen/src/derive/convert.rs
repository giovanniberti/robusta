use std::collections::HashMap;

use crate::derive::utils::generic_params_to_args;
use crate::transformation::JavaPath;
use proc_macro2::{Ident, TokenStream};
use proc_macro_error::{abort, emit_error, emit_warning};
use quote::{quote, quote_spanned, ToTokens};
use syn::spanned::Spanned;
use syn::{
    AngleBracketedGenericArguments, Data, DataStruct, DeriveInput, Field, GenericArgument,
    GenericParam, Generics, LifetimeParam, PathArguments, Type, TypePath,
};

struct TraitAutoDeriveData {
    instance_field_type_assertion: TokenStream,
    impl_target: Ident,
    classpath_path: String,
    generics: Generics,
    instance_ident: Ident,
    generic_args: AngleBracketedGenericArguments,
    data_fields: Vec<Field>,
    class_fields: Vec<Field>,
}

pub(crate) fn into_java_value_macro_derive(input: DeriveInput) -> TokenStream {
    let input_span = input.span();
    match into_java_value_macro_derive_impl(input) {
        Ok(t) => t,
        Err(_) => quote_spanned! { input_span => },
    }
}

fn into_java_value_macro_derive_impl(input: DeriveInput) -> syn::Result<TokenStream> {
    let TraitAutoDeriveData {
        instance_field_type_assertion,
        impl_target,
        generics,
        instance_ident,
        generic_args,
        ..
    } = get_trait_impl_components("IntoJavaValue", input);

    Ok(quote! {
        #instance_field_type_assertion

        #[automatically_derived]
        impl#generics ::robusta_jni::convert::IntoJavaValue<'env> for #impl_target#generic_args {
            type Target = ::robusta_jni::jni::objects::JObject<'env>;

            fn into(self, env: &::robusta_jni::jni::JNIEnv<'env>) -> Self::Target {
                <&#impl_target as ::robusta_jni::convert::IntoJavaValue>::into(&self, env)
            }
        }

        #[automatically_derived]
        impl#generics ::robusta_jni::convert::IntoJavaValue<'env> for &#impl_target#generic_args {
            type Target = ::robusta_jni::jni::objects::JObject<'env>;

            fn into(self, env: &::robusta_jni::jni::JNIEnv<'env>) -> Self::Target {
                self.#instance_ident.as_obj()
            }
        }

        #[automatically_derived]
        impl#generics ::robusta_jni::convert::IntoJavaValue<'env> for &mut #impl_target#generic_args {
            type Target = ::robusta_jni::jni::objects::JObject<'env>;

            fn into(self, env: &::robusta_jni::jni::JNIEnv<'env>) -> Self::Target {
                <&#impl_target as ::robusta_jni::convert::IntoJavaValue>::into(self, env)
            }
        }
    })
}

pub(crate) fn tryinto_java_value_macro_derive(input: DeriveInput) -> TokenStream {
    let input_span = input.span();
    match tryinto_java_value_macro_derive_impl(input) {
        Ok(t) => t,
        Err(_) => quote_spanned! { input_span => },
    }
}

fn tryinto_java_value_macro_derive_impl(input: DeriveInput) -> syn::Result<TokenStream> {
    let TraitAutoDeriveData {
        instance_field_type_assertion,
        impl_target,
        generics,
        instance_ident,
        generic_args,
        ..
    } = get_trait_impl_components("TryIntoJavaValue", input);

    Ok(quote! {
        #instance_field_type_assertion

        #[automatically_derived]
        impl#generics ::robusta_jni::convert::TryIntoJavaValue<'env> for #impl_target#generic_args {
            type Target = ::robusta_jni::jni::objects::JObject<'env>;

            fn try_into(self, env: &::robusta_jni::jni::JNIEnv<'env>) -> ::robusta_jni::jni::errors::Result<Self::Target> {
                <&#impl_target as ::robusta_jni::convert::TryIntoJavaValue>::try_into(&self, env)
            }
        }

        #[automatically_derived]
        impl#generics ::robusta_jni::convert::TryIntoJavaValue<'env> for &#impl_target#generic_args {
            type Target = ::robusta_jni::jni::objects::JObject<'env>;

            fn try_into(self, env: &::robusta_jni::jni::JNIEnv<'env>) -> ::robusta_jni::jni::errors::Result<Self::Target> {
                Ok(self.#instance_ident.as_obj())
            }
        }

        #[automatically_derived]
        impl#generics ::robusta_jni::convert::TryIntoJavaValue<'env> for &mut #impl_target#generic_args {
            type Target = ::robusta_jni::jni::objects::JObject<'env>;

            fn try_into(self, env: &::robusta_jni::jni::JNIEnv<'env>) -> ::robusta_jni::jni::errors::Result<Self::Target> {
                <&#impl_target as ::robusta_jni::convert::TryIntoJavaValue>::try_into(self, env)
            }
        }
    })
}

pub(crate) fn from_java_value_macro_derive(input: DeriveInput) -> TokenStream {
    let input_span = input.span();
    match from_java_value_macro_derive_impl(input) {
        Ok(t) => t,
        Err(_) => quote_spanned! { input_span => },
    }
}

fn from_java_value_macro_derive_impl(input: DeriveInput) -> syn::Result<TokenStream> {
    let TraitAutoDeriveData {
        instance_field_type_assertion,
        impl_target,
        classpath_path,
        generics,
        instance_ident,
        generic_args,
        data_fields,
        class_fields,
    } = get_trait_impl_components("FromJavaValue", input);

    let data_fields_struct_init: Vec<_> = data_fields
        .iter()
        .map(|f| f.ident.as_ref().unwrap())
        .collect();
    let data_fields_env_init: Vec<_> = data_fields.iter().map(|f| {
        let field_ident = f.ident.as_ref().unwrap();
        let field_name = field_ident.to_string();
        let field_type = &f.ty;
        let field_type_sig = quote_spanned! { field_type.span() =>
            <#field_type as Signature>::SIG_TYPE
        };
        quote_spanned! { f.span() =>
            let #field_ident: #field_type = ::robusta_jni::convert::FromJavaValue::from(::core::convert::TryInto::try_into(::robusta_jni::convert::JValueWrapper::from(env.get_field(source, #field_name, #field_type_sig).unwrap())).unwrap(), env);
        }
    }).collect();

    let class_fields_struct_init: Vec<_> = class_fields
        .iter()
        .map(|f| f.ident.as_ref().unwrap())
        .collect();
    let class_fields_env_init: Vec<_> = class_fields
        .iter()
        .map(|f| {
            let field_ident = f.ident.as_ref().unwrap();
            let field_name = field_ident.to_string();
            let field_type = &f.ty;

            quote_spanned! { f.span() =>
                let #field_ident: #field_type = ::robusta_jni::convert::Field::field_from(source,
                    #classpath_path,
                    #field_name,
                    env);
            }
        })
        .collect();

    let mut fields_struct_init = data_fields_struct_init.clone();
    fields_struct_init.extend(class_fields_struct_init);

    Ok(quote! {
        #instance_field_type_assertion

        #[automatically_derived]
        impl#generics ::robusta_jni::convert::FromJavaValue<'env, 'borrow> for #impl_target#generic_args {
            type Source = ::robusta_jni::jni::objects::JObject<'env>;

            fn from(source: Self::Source, env: &'borrow ::robusta_jni::jni::JNIEnv<'env>) -> Self {
                #(#data_fields_env_init)*
                #(#class_fields_env_init)*

                Self {
                    #instance_ident: ::robusta_jni::convert::Local::new(env, source),
                    #(#fields_struct_init),*
                }
            }
        }
    })
}

pub(crate) fn tryfrom_java_value_macro_derive(input: DeriveInput) -> TokenStream {
    let input_span = input.span();
    match tryfrom_java_value_macro_derive_impl(input) {
        Ok(t) => t,
        Err(_) => quote_spanned! { input_span => },
    }
}

fn tryfrom_java_value_macro_derive_impl(input: DeriveInput) -> syn::Result<TokenStream> {
    let TraitAutoDeriveData {
        instance_field_type_assertion,
        impl_target,
        classpath_path,
        generics,
        instance_ident,
        generic_args,
        data_fields,
        class_fields,
    } = get_trait_impl_components("FromJavaValue", input);

    let data_fields_struct_init: Vec<_> = data_fields
        .iter()
        .map(|f| f.ident.as_ref().unwrap())
        .collect();
    let data_fields_env_init: Vec<_> = data_fields.iter().map(|f| {
        let field_ident = f.ident.as_ref().unwrap();
        let field_name = field_ident.to_string();
        let field_type = &f.ty;
        let field_type_sig = quote_spanned! { field_type.span() =>
            <#field_type as Signature>::SIG_TYPE
        };
        quote_spanned! { f.span() =>
            let #field_ident: #field_type = ::robusta_jni::convert::TryFromJavaValue::try_from(::core::convert::TryInto::try_into(::robusta_jni::convert::JValueWrapper::from(env.get_field(source, #field_name, #field_type_sig)?))?, env)?;
        }
    }).collect();

    let class_fields_struct_init: Vec<_> = class_fields
        .iter()
        .map(|f| f.ident.as_ref().unwrap())
        .collect();
    let class_fields_env_init: Vec<_> = class_fields.iter().map(|f| {
        let field_ident = f.ident.as_ref().unwrap();
        let field_name = field_ident.to_string();
        let field_type = &f.ty;

        quote_spanned! { f.span() =>
            let #field_ident: #field_type = ::robusta_jni::convert::Field::field_try_from(source,
                #classpath_path,
                #field_name,
                env)?;
        }
    }).collect();

    let mut fields_struct_init = data_fields_struct_init.clone();
    fields_struct_init.extend(class_fields_struct_init);

    Ok(quote! {
        #instance_field_type_assertion

        #[automatically_derived]
        impl#generics ::robusta_jni::convert::TryFromJavaValue<'env, 'borrow> for #impl_target#generic_args {
            type Source = ::robusta_jni::jni::objects::JObject<'env>;

            fn try_from(source: Self::Source, env: &'borrow ::robusta_jni::jni::JNIEnv<'env>) -> ::robusta_jni::jni::errors::Result<Self> {
                #(#data_fields_env_init)*
                #(#class_fields_env_init)*

                Ok(Self {
                    #instance_ident: ::robusta_jni::convert::Local::new(env, source),
                    #(#fields_struct_init),*
                })
            }
        }
    })
}

fn get_trait_impl_components(trait_name: &str, input: DeriveInput) -> TraitAutoDeriveData {
    let input_span = input.span();
    let input_ident = &input.ident;

    match input.data {
        Data::Struct(DataStruct { fields, .. }) => {
            let package_attr = input.attrs.iter().find(|a| {
                a.path().get_ident().map(ToString::to_string).as_deref() == Some("package")
            });
            if package_attr.is_none() {
                abort!(input_span, "missing `#[package]` attribute")
            }

            let classpath_path = package_attr
                .unwrap()
                .parse_args()
                .map(|p: JavaPath| p.to_classpath_path())
                .map(|s| {
                    let mut s = s.clone();
                    if !s.is_empty() {
                        s.push('/');
                    }
                    s.push_str(&input_ident.to_string());
                    s
                })
                .unwrap_or_else(|_| {
                    emit_error!(package_attr, "invalid Java class path");
                    "".to_string()
                });

            let lifetimes: HashMap<String, &LifetimeParam> = input
                .generics
                .params
                .iter()
                .filter_map(|g| match g {
                    GenericParam::Lifetime(l) => Some(l),
                    _ => None,
                })
                .map(|l| (l.lifetime.ident.to_string(), l))
                .collect();

            match (lifetimes.get("env"), lifetimes.get("borrow")) {
                (Some(env_lifetime), Some(borrow_lifetime)) => {
                    if !env_lifetime
                        .bounds
                        .iter()
                        .any(|l| *l == borrow_lifetime.lifetime)
                    {
                        emit_error!(env_lifetime, "`'env` lifetime must have a `'borrow` lifetime bound";
                                    help = "try adding `'env: 'borrow`")
                    }
                }
                _ => emit_error!(
                    input_span,
                    "deriving struct must have `'env` and `'borrow` lifetime parameters"
                ),
            }

            let instance_fields: Vec<_> = fields
                .iter()
                .filter_map(|f| {
                    let attr = f.attrs.iter().find(|a| {
                        a.path().get_ident().map(|i| i.to_string()).as_deref() == Some("instance")
                    });
                    attr.map(|a| (f, a))
                })
                .collect();

            let class_fields: Vec<_> = fields
                .iter()
                .filter(|f| {
                    let attr = f.attrs.iter().find(|a| {
                        a.path().get_ident().map(|i| i.to_string()).as_deref() == Some("field")
                    });
                    attr.is_some()
                })
                .collect();

            if instance_fields.len() > 1 {
                emit_error!(
                    input_span,
                    "cannot have more than one `#[instance]` attribute"
                )
            }

            let instance_field_data = instance_fields.get(0);

            match instance_field_data {
                None => abort!(input_span, "missing `#[instance] field attribute"),
                Some((instance, attr)) => {
                    if attr
                        .meta
                        .require_list()
                        .is_ok_and(|meta_list| !meta_list.tokens.is_empty())
                    {
                        emit_warning!(
                            attr.to_token_stream(),
                            "`#[instance]` attribute doesn't have any arguments"
                        )
                    }

                    let ty = {
                        let mut t = instance.ty.clone();
                        if let Type::Path(TypePath { path, .. }) = &mut t {
                            path.segments.iter_mut().for_each(|s| {
                                if let PathArguments::AngleBracketed(a) = &mut s.arguments {
                                    a.args.iter_mut().for_each(|g| {
                                        if let GenericArgument::Lifetime(l) = g {
                                            l.ident = Ident::new("static", l.span());
                                        }
                                    })
                                }
                            });
                        }

                        t
                    };

                    let instance_field_type_assertion = quote_spanned! { ty.span() =>
                        ::robusta_jni::assert_type_eq_all!(#ty, ::robusta_jni::convert::Local<'static, 'static>);
                    };

                    let generics = input.generics;
                    let instance_span = instance.span();
                    let instance_ident = instance.ident.as_ref().unwrap_or_else(|| {
                        abort!(instance_span, "instance field must have a name")
                    });

                    let generic_args = generic_params_to_args(generics.clone());

                    let data_fields: Vec<_> = fields
                        .iter()
                        .filter(|f| {
                            f.ident.as_ref() != Some(instance_ident)
                                && class_fields.iter().all(|g| g != f)
                        })
                        .cloned()
                        .collect();

                    TraitAutoDeriveData {
                        instance_field_type_assertion,
                        impl_target: input.ident,
                        classpath_path,
                        generics,
                        instance_ident: instance_ident.clone(),
                        generic_args,
                        data_fields,
                        class_fields: class_fields.into_iter().cloned().collect(),
                    }
                }
            }
        }
        _ => abort!(
            input,
            "`{}` auto-derive implemented for structs only",
            trait_name
        ),
    }
}
