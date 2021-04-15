use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::{quote, quote_spanned};
use syn::{Data, DataStruct, DeriveInput};
use syn::spanned::Spanned;

use crate::transformation::JavaPath;

use super::utils::generic_params_to_args;

pub(crate) fn signature_macro_derive(input: DeriveInput) -> TokenStream {
    let input_span = input.span();
    match signature_macro_derive_impl(input) {
        Ok(t) => t,
        Err(_) => quote_spanned! { input_span => }
    }
}

fn signature_macro_derive_impl(input: DeriveInput) -> syn::Result<TokenStream> {
    let input_span = input.span();

    match input.data {
        Data::Struct(DataStruct { .. }) => {
            let package_attr = input.attrs.iter().find(|a| a.path.get_ident().map(ToString::to_string).as_deref() == Some("package"));

            match package_attr {
                None => abort!(input_span, "missing `#[package()]` attribute"),
                Some(attr) => {
                    let struct_name = input.ident;
                    let package = attr.parse_args::<JavaPath>()?;
                    let package_str = {
                        let mut s = package.to_classpath_path();
                        if !s.is_empty() {
                            s.push('/')
                        }
                        s
                    };

                    let signature = ["L", package_str.as_str(), struct_name.to_string().as_str(), ";"].join("");
                    let generics = input.generics.clone();
                    let generic_args = generic_params_to_args(input.generics);

                    Ok(quote! {
                        #[automatically_derived]
                        impl#generics ::robusta_jni::convert::Signature for #struct_name#generic_args {
                            const SIG_TYPE: &'static str = #signature;
                        }

                        #[automatically_derived]
                        impl#generics ::robusta_jni::convert::Signature for &#struct_name#generic_args {
                            const SIG_TYPE: &'static str = <#struct_name as ::robusta_jni::convert::Signature>::SIG_TYPE;
                        }

                        #[automatically_derived]
                        impl#generics ::robusta_jni::convert::Signature for &mut #struct_name#generic_args {
                            const SIG_TYPE: &'static str = <#struct_name as ::robusta_jni::convert::Signature>::SIG_TYPE;
                        }
                    })
                }
            }
        },
        _ => abort!(input_span, "`Signature` auto-derive implemented for structs only"),
    }
}
