extern crate proc_macro;

use proc_macro::TokenStream;
use std::collections::{BTreeMap, BTreeSet};

use proc_macro_error::proc_macro_error;
use proc_macro_error::{emit_error, emit_warning};
use quote::quote;
use syn::{Attribute, Error, Ident, Item, ItemImpl, ItemMod, ItemStruct, parse_macro_input, Result, Type, ImplItem};
use syn::export::ToTokens;
use syn::parse::{Parse, ParseBuffer, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::Token;

struct JNIBridgeModule {
    module_decl: ItemMod,
    package_map: BTreeMap<String, String>,
    bridged_structs: Vec<ItemStruct>,
    struct_impls: Vec<ItemImpl>,
}

impl Parse for JNIBridgeModule {
    fn parse(input: &ParseBuffer) -> Result<Self> {
        let module_decl: ItemMod = input.parse().map_err(|e| Error::new(e.span(), "`bridge` attribute is supported on mod items only"))?;

        let module_span = module_decl.span();
        let (_, module_items) = module_decl.clone().content.ok_or_else(|| Error::new(module_span, "Invalid module definition"))?;

        let module_annotated_items: Vec<&Item> = module_items.iter()
            .filter(|item| {
                let has_package_attribute = |a: &Attribute| a.path.segments.first().unwrap().ident.to_string() == "package";
                match item {
                    Item::Struct(_) => true,
                    Item::Const(i) if i.attrs.iter().any(has_package_attribute) => {
                        emit_error!(i.span(), "`package` attribute used on non-struct type");
                        false
                    }
                    Item::Enum(i) if i.attrs.iter().any(has_package_attribute) => {
                        emit_error!(i.span(), "`package` attribute used on non-struct type"; help = i.enum_token.span() => "replace `enum` with `struct`");
                        false
                    }
                    Item::ExternCrate(i) if i.attrs.iter().any(has_package_attribute) => {
                        emit_error!(i.span(), "`package` attribute used on non-struct type");
                        false
                    }
                    Item::Fn(i) if i.attrs.iter().any(has_package_attribute) => {
                        emit_error!(i.span(), "`package` attribute used on non-struct type");
                        false
                    }
                    Item::ForeignMod(i) => {
                        emit_error!(i.span(), "`package` attribute used on non-struct type");
                        false
                    }
                    Item::Impl(i) if i.attrs.iter().any(has_package_attribute) => {
                        emit_error!(i.span(), "`package` attribute used on non-struct type");
                        false
                    }
                    Item::Macro(i) if i.attrs.iter().any(has_package_attribute) => {
                        emit_error!(i.span(), "`package` attribute used on non-struct type");
                        false
                    }
                    Item::Macro2(i) if i.attrs.iter().any(has_package_attribute) => {
                        emit_error!(i.span(), "`package` attribute used on non-struct type");
                        false
                    }
                    Item::Mod(i) if i.attrs.iter().any(has_package_attribute) => {
                        emit_error!(i.span(), "`package` attribute used on non-struct type");
                        false
                    }
                    Item::Static(i) if i.attrs.iter().any(has_package_attribute) => {
                        emit_error!(i.span(), "`package` attribute used on non-struct type");
                        false
                    }
                    Item::Trait(i) if i.attrs.iter().any(has_package_attribute) => {
                        emit_error!(i.span(), "`package` attribute used on non-struct type");
                        false
                    }
                    Item::TraitAlias(i) if i.attrs.iter().any(has_package_attribute) => {
                        emit_error!(i.span(), "`package` attribute used on non-struct type");
                        false
                    }
                    Item::Type(i) if i.attrs.iter().any(has_package_attribute) => {
                        emit_error!(i.span(), "`package` attribute used on non-struct type");
                        false
                    }
                    Item::Union(i) if i.attrs.iter().any(has_package_attribute) => {
                        emit_error!(i.span(), "`package` attribute used on non-struct type");
                        false
                    }
                    Item::Use(i) if i.attrs.iter().any(has_package_attribute) => {
                        emit_error!(i.span(), "`package` attribute used on non-struct type");
                        false
                    }
                    Item::Verbatim(_) => false,
                    _ => false
                }
            })
            .collect();

        let module_structs: Vec<ItemStruct> = module_items.iter()
            .filter_map(|item| {
                match item {
                    Item::Struct(s) => {
                        Some(s.clone())
                    }
                    _ => None
                }
            })
            .collect();

        let bridged_structs: Vec<_> = module_structs.iter().cloned()
            .filter(|s| {
                s.attrs.iter()
                    .map(|a| &a.path)
                    .any(|p| p.segments.first().unwrap().ident.to_string() == "package")
            })
            .collect();

        let module_struct_names: BTreeSet<_> = module_structs.iter().map(|s| s.ident.to_string()).collect();
        let bridged_struct_names: BTreeSet<_> = bridged_structs.iter().map(|s| s.ident.to_string()).collect();

        let module_struct_impls: Vec<ItemImpl> = module_items.into_iter()
            .filter_map(|item| {
                match item {
                    Item::Impl(i) => Some(i),
                    _ => None
                }
            })
            .collect();

        let module_struct_impl_names: BTreeSet<_> = module_struct_impls.iter()
            .filter_map(|i| {
                match &*i.self_ty {
                    Type::Path(p) => Some(p.path.to_token_stream().to_string()),
                    _ => None
                }
            })
            .collect();

        let bridged_struct_impls: Vec<ItemImpl> = module_struct_impls.into_iter()
            .map(|i| {
                // Check wether impl is struct impl (`i.self_ty` is of variant `Path`) and that type corresponds to annotated/bridged struct

                match &*i.self_ty {
                    Type::Path(p) => {
                        let type_name = p.path.to_token_stream().to_string();

                        match (bridged_struct_names.contains(&type_name), module_struct_names.contains(&type_name)) {
                            (true, _) => Ok(Some(i)),
                            (false, true) => {
                                emit_error!(i, "struct {} is not marked with `package` attribute", type_name);
                                Ok(None)
                            }
                            (false, false) => {
                                emit_warning!(i, "ignoring impl for undeclared item {}", type_name);
                                Ok(None)
                            }
                        }
                    }
                    _ => Err(Error::new(i.span(), "2"))
                }
            })
            .collect::<Result<Vec<_>>>()?
            .into_iter().filter_map(|x| x)
            .collect();

        let bridged_struct_impl_names: BTreeSet<_> = bridged_struct_impls.iter().filter_map(|i| {
            match &*i.self_ty {
                Type::Path(p) => Some(p.path.to_token_stream().to_string()),
                _ => None
            }
        }).collect();

        module_structs.iter()
            .map(|s| (s, &s.ident))
            .filter(|(_, ident)| !bridged_struct_names.contains(&ident.to_string()))
            .filter(|(_, ident)| !module_struct_impl_names.contains(&ident.to_string()))
            .for_each(|(s, ident)| {
                emit_warning!(s, "ignoring struct {} declared without required `package` attribute", ident)
            });

        let package_map: BTreeMap<String, String> = bridged_structs.iter()
            .map(|s| {
                let package_params = s.attrs.iter()
                    .filter(|a| a.path.segments.first().unwrap().ident.to_string() == "package")
                    .next()
                    .map(|a| {
                        a.parse_args_with(|t: ParseStream| {
                            Punctuated::<Ident, Token![.]>::parse_separated_nonempty(t)
                        })
                            .unwrap()
                            .to_token_stream()
                            .to_string().replace(' ', "")
                    })
                    .unwrap();

                (s.ident.to_string(), package_params)
            })
            .collect();

        Ok(JNIBridgeModule {
            module_decl,
            package_map,
            bridged_structs,
            struct_impls: bridged_struct_impls,
        })
    }
}

/// Example of user-defined [procedural macro attribute][1].
///
/// [1]: https://doc.rust-lang.org/reference/procedural-macros.html#attribute-macros
#[proc_macro_error]
#[proc_macro_attribute]
pub fn bridge(_args: TokenStream, raw_input: TokenStream) -> TokenStream {
    let module_data = parse_macro_input!(raw_input as JNIBridgeModule);

    println!("Package map: {:?}", module_data.package_map);

    let tokens = quote! {};

    tokens.into()
}
