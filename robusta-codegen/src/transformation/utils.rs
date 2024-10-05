use std::collections::HashSet;
use std::str::FromStr;

use proc_macro2::TokenStream;
use proc_macro_error::{abort, emit_warning};
use quote::ToTokens;
use syn::visit::Visit;
use syn::{ImplItemFn, Lit, LitStr};

use crate::transformation::{AttributeFilter, CallTypeAttribute};

pub(crate) fn get_call_type(node: &ImplItemFn) -> Option<CallTypeAttribute> {
    let attributes_collector = get_attributes_collector(&node, "call_type");

    let call_type_attribute = attributes_collector.filtered_attributes.first().and_then(|call_type_attr| {
        syn::parse2(call_type_attr.to_token_stream()).map_err(|e| {
            emit_warning!(e.span(), format!("invalid parsing of `call_type` attribute, defaulting to #[call_type(safe)]. {}", e));
            e
        }).ok()
    });

    call_type_attribute
}

pub(crate) fn get_output_type_override(node: &ImplItemFn) -> Option<LitStr> {
    let attributes_collector = get_attributes_collector(&node, "output_type");

    let output_type_attribute = attributes_collector.filtered_attributes.first().and_then(|a| {
        if let Ok(meta_list) = a.meta.require_list() {
            let token_tree_lit: Lit = syn::parse2::<Lit>(meta_list.clone().tokens).unwrap();

            if let Lit::Str(literal) = token_tree_lit {
                Some(literal)
            } else {
                None
            }
        } else {
            abort!(a, "Missing argument for `#[output_type]`")
        }
    });

    output_type_attribute
}

fn get_attributes_collector<'a>(node: &'a ImplItemFn, name: &'a str) -> AttributeFilter<'a> {
    let whitelist = {
        let mut f = HashSet::new();
        f.insert(syn::parse2(TokenStream::from_str(name).unwrap()).unwrap());
        f
    };

    let mut attributes_collector = AttributeFilter::with_whitelist(whitelist);
    attributes_collector.visit_impl_item_fn(&node);
    attributes_collector
}

macro_rules! parse_quote_spanned {
    ($span:expr => $($tt:tt)*) => {
        syn::parse2(quote::quote_spanned!($span => $($tt)*)).unwrap_or_else(|e| panic!("{}", e))
    };
}
