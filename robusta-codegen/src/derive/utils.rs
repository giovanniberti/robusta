use syn::parse_quote;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::Token;
use syn::{
    AngleBracketedGenericArguments, ConstParam, GenericArgument, GenericParam, Generics, TypeParam,
};

pub(crate) fn generic_params_to_args(generics: Generics) -> AngleBracketedGenericArguments {
    let args: Punctuated<GenericArgument, Token![,]> = generics
        .params
        .iter()
        .map(|g| match g {
            GenericParam::Type(TypeParam { ident, .. }) => {
                GenericArgument::Type(parse_quote! { #ident })
            }
            GenericParam::Lifetime(l) => GenericArgument::Lifetime(l.lifetime.clone()),
            GenericParam::Const(ConstParam { ident, .. }) => {
                GenericArgument::Const(parse_quote! { #ident })
            }
        })
        .collect();

    AngleBracketedGenericArguments {
        colon2_token: None,
        lt_token: generics
            .lt_token
            .unwrap_or_else(|| Token![<](generics.span())),
        args,
        gt_token: generics
            .gt_token
            .unwrap_or_else(|| Token![>](generics.span())),
    }
}
