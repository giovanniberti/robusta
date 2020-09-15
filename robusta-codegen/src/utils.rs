use syn::Ident;
use rand::{SeedableRng, Rng};
use rand::rngs::StdRng;
use rand::distributions::Alphanumeric;
use proc_macro2::Span;
use quote::format_ident;
use std::convert::TryInto;

pub fn unique_ident(prefix: &str, span: Span) -> Ident {
    /* Identifier generation with a UUID (or `gensym` crate) might be more robust, but these 5 random characters should be more than enough */
    let seed: [u8; 32] = {
        let mut res = vec![0u8; 28];
        res.append(&mut vec![span.start().column as u8, span.start().line as u8, span.end().column as u8, span.start().line as u8]);
        res.as_slice().try_into().unwrap()
    };

    format_ident!("__{}_{}", prefix, StdRng::from_seed(seed).sample_iter(&Alphanumeric).take(5).collect::<String>(), span = span)
}