use std::collections::HashSet;
use std::str::FromStr;

use proc_macro2::TokenStream;
use proc_macro_error::emit_warning;
use quote::ToTokens;
use syn::visit::Visit;
use syn::ImplItemMethod;

use crate::transformation::{AttributeFilter, CallTypeAttribute};

pub(crate) fn get_call_type(node: &ImplItemMethod) -> Option<CallTypeAttribute> {
    let whitelist = {
        let mut f = HashSet::new();
        f.insert(syn::parse2(TokenStream::from_str("call_type").unwrap()).unwrap());
        f
    };

    let mut attributes_collector = AttributeFilter::with_whitelist(whitelist);
    attributes_collector.visit_impl_item_method(&node);

    let call_type_attribute = attributes_collector.filtered_attributes.first().and_then(|call_type_attr| {
        syn::parse2(call_type_attr.to_token_stream()).map_err(|e| {
            emit_warning!(e.span(), format!("invalid parsing of `call_type` attribute, defaulting to #[call_type(safe)]. {}", e));
            e
        }).ok()
    });

    call_type_attribute
}

macro_rules! parse_quote_spanned {
    ($span:expr => $($tt:tt)*) => {
        syn::parse2(quote::quote_spanned!($span => $($tt)*)).unwrap_or_else(|e| panic!("{}", e))
    };
}
