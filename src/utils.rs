use syn::Ident;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use proc_macro2::Span;

pub fn unique_ident(prefix: &str, span: Span) -> Ident {
    /* Identifier generation with a UUID (or `gensym` crate) might be more robust, but these 5 random characters should be more than enough */
    let string_repr = format!("__{}_{}", prefix, thread_rng().sample_iter(&Alphanumeric).take(5).collect::<String>());

    Ident::new(&string_repr, span)
}