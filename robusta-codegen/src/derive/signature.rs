use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::{quote, quote_spanned};
use syn::{Data, DataStruct, DeriveInput};
use syn::spanned::Spanned;

use crate::transformation::JavaPath;

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
                    let signature = ["L", package.to_string().replace('.', "/").as_str(), "/", struct_name.to_string().as_str(), ";"].join("");

                    Ok(quote! {
                        #[automatically_derived]
                        impl ::robusta_jni::convert::Signature for #struct_name {
                            const SIG_TYPE: &'static str = #signature;
                        }

                        #[automatically_derived]
                        impl ::robusta_jni::convert::Signature for &#struct_name {
                            const SIG_TYPE: &'static str = <#struct_name as ::robusta_jni::convert::Signature>::SIG_TYPE;
                        }

                        #[automatically_derived]
                        impl ::robusta_jni::convert::Signature for &mut #struct_name {
                            const SIG_TYPE: &'static str = <#struct_name as ::robusta_jni::convert::Signature>::SIG_TYPE;
                        }
                    })
                }
            }
        },
        _ => abort!(input_span, "`Signature` auto-derive implemented for structs only"),
    }
}
