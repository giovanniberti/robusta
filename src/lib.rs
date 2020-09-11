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
use syn::visit::Visit;

struct AttribItemChecker;

impl<'ast> Visit<'ast> for AttribItemChecker {
    fn visit_item(&mut self, node: &'ast Item) {
        let has_package_attribute = |a: &Attribute| a.path.segments.first().unwrap().ident.to_string() == "package";
        match node {
            Item::Struct(_) => {}
            Item::Const(i) if i.attrs.iter().any(has_package_attribute) => {
                emit_error!(i.span(), "`package` attribute used on non-struct type");
            }
            Item::Enum(i) if i.attrs.iter().any(has_package_attribute) => {
                emit_error!(i.span(), "`package` attribute used on non-struct type"; help = i.enum_token.span() => "replace `enum` with `struct`");
            }
            Item::ExternCrate(i) if i.attrs.iter().any(has_package_attribute) => {
                emit_error!(i.span(), "`package` attribute used on non-struct type");
            }
            Item::Fn(i) if i.attrs.iter().any(has_package_attribute) => {
                emit_error!(i.span(), "`package` attribute used on non-struct type");
            }
            Item::ForeignMod(i) => {
                emit_error!(i.span(), "`package` attribute used on non-struct type");
            }
            Item::Impl(i) if i.attrs.iter().any(has_package_attribute) => {
                emit_error!(i.span(), "`package` attribute used on non-struct type");
            }
            Item::Macro(i) if i.attrs.iter().any(has_package_attribute) => {
                emit_error!(i.span(), "`package` attribute used on non-struct type");
            }
            Item::Macro2(i) if i.attrs.iter().any(has_package_attribute) => {
                emit_error!(i.span(), "`package` attribute used on non-struct type");
            }
            Item::Mod(i) if i.attrs.iter().any(has_package_attribute) => {
                emit_error!(i.span(), "`package` attribute used on non-struct type");
            }
            Item::Static(i) if i.attrs.iter().any(has_package_attribute) => {
                emit_error!(i.span(), "`package` attribute used on non-struct type");
            }
            Item::Trait(i) if i.attrs.iter().any(has_package_attribute) => {
                emit_error!(i.span(), "`package` attribute used on non-struct type");
            }
            Item::TraitAlias(i) if i.attrs.iter().any(has_package_attribute) => {
                emit_error!(i.span(), "`package` attribute used on non-struct type");
            }
            Item::Type(i) if i.attrs.iter().any(has_package_attribute) => {
                emit_error!(i.span(), "`package` attribute used on non-struct type");
            }
            Item::Union(i) if i.attrs.iter().any(has_package_attribute) => {
                emit_error!(i.span(), "`package` attribute used on non-struct type");
            }
            Item::Use(i) if i.attrs.iter().any(has_package_attribute) => {
                emit_error!(i.span(), "`package` attribute used on non-struct type");
            }
            Item::Verbatim(_) => {}
            _ => {}
        }
    }
}

#[derive(Default)]
struct ImplAccumulator<'ast> {
    impls: Vec<&'ast ItemImpl>
}

impl<'ast> Visit<'ast> for ImplAccumulator<'ast> {
    fn visit_item_impl(&mut self, node: &'ast ItemImpl) {
        self.impls.push(node);
    }
}

enum StructDeclarationKind {
    Bridged,
    // structs with `package` attrib and impl
    UnImpl,
    // structs with `package` attrib but no impl
    UnAttrib,
    // structs without `package` attrib but with impl
    Bare, // structs without `package` attrib and no impl
}

struct StructDeclVisitor<'ast> {
    module_structs: Vec<(&'ast ItemStruct, StructDeclarationKind)>,
    // all module impls
    module_impls: Vec<&'ast ItemImpl>,
}

impl<'ast> StructDeclVisitor<'ast> {
    fn new(module_impls: Vec<&'ast ItemImpl>) -> Self {
        StructDeclVisitor {
            module_structs: Vec::new(),
            module_impls,
        }
    }
}

impl<'ast> Visit<'ast> for StructDeclVisitor<'ast> {
    fn visit_item_struct(&mut self, node: &'ast ItemStruct) {
        let struct_name = node.ident.to_string();
        let has_package_attrib = node.attrs.iter().any(|a| a.path.segments.first().unwrap().ident.to_string() == "package");
        let has_impl = self.module_impls.iter().filter_map(|i| {
            match &*i.self_ty {
                Type::Path(p) => Some(p.path.segments.last().unwrap().ident.to_string()),
                _ => None
            }
        }).any(|s| s == struct_name);

        let declaration_kind = match (has_package_attrib, has_impl) {
            (true, true) => StructDeclarationKind::Bridged,
            (true, false) => StructDeclarationKind::UnImpl,
            (false, true) => StructDeclarationKind::UnAttrib,
            (false, false) => StructDeclarationKind::Bare,
        };

        self.module_structs.push((node, declaration_kind))
    }
}

struct JNIBridgeModule {
    module_decl: ItemMod,
    package_map: BTreeMap<String, String>,
    bridged_impls: Vec<(ItemStruct, ItemImpl)>,
}

impl Parse for JNIBridgeModule {
    fn parse(input: &ParseBuffer) -> Result<Self> {
        let module_decl: ItemMod = input.parse().map_err(|e| Error::new(e.span(), "`bridge` attribute is supported on mod items only"))?;

        let mut attribute_checker = AttribItemChecker;
        attribute_checker.visit_item_mod(&module_decl);

        let mut impl_visitor = ImplAccumulator::default();
        impl_visitor.visit_item_mod(&module_decl);

        let mut mod_visitor = StructDeclVisitor::new(impl_visitor.impls);
        mod_visitor.visit_item_mod(&module_decl);

        let bridged_structs: Vec<_> = mod_visitor.module_structs.into_iter()
            .filter_map(|(struct_item, decl_kind)| {
                match decl_kind {
                    StructDeclarationKind::Bridged => Some(struct_item),
                    StructDeclarationKind::UnImpl => {
                        emit_warning!(struct_item, "ignoring struct without declared methods"; help = "add methods using an `impl` block");
                        None
                    }
                    StructDeclarationKind::UnAttrib => {
                        emit_error!(struct_item, "struct without required `package` attribute");
                        None
                    }
                    StructDeclarationKind::Bare => {
                        emit_warning!(struct_item, "ignoring struct with no `package` attribute";
                            help = struct_item.span() => "add a #[package(...)] attribute");
                        None
                    }
                }
            })
            .collect();

        let structs_idents: Vec<_> = bridged_structs.iter().map(|s| &s.ident).collect();
        let bridged_impls: Vec<_> = mod_visitor.module_impls.into_iter()
            .filter_map(|item_impl| {
                match &*item_impl.self_ty {
                    Type::Path(p) => {
                        if let Some(pos) = structs_idents.iter().position(|id| *id == &p.path.segments.last().unwrap().ident) {
                            Some((bridged_structs[pos], item_impl))
                        } else {
                            None
                        }
                    }
                    _ => None
                }
            })
            .map(|(s, i)| (s.clone(), i.clone()))
            .collect();

        let package_map: BTreeMap<String, String> = bridged_structs.iter()
            .map(|s| {
                let name = s.ident.to_string();
                let package = s.attrs.iter()
                    .filter(|a| a.path.segments.last().unwrap().ident.to_string() == "package")
                    .map(|a| {
                        a.parse_args_with(|t: ParseStream| { Punctuated::<Ident, Token![.]>::parse_separated_nonempty(t) })
                            .unwrap()
                            .to_token_stream()
                            .to_string().replace(' ', "")
                    })
                    .next().unwrap();
                (name, package)
            })
            .collect();

        Ok(JNIBridgeModule {
            module_decl,
            package_map,
            bridged_impls,
        })
    }
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn bridge(_args: TokenStream, raw_input: TokenStream) -> TokenStream {
    let module_data = parse_macro_input!(raw_input as JNIBridgeModule);

    println!("Package map: {:?}", module_data.package_map);
    let cleared_structs: Vec<_> = module_data.bridged_impls.into_iter()
        .map(|(s, i)| {
            s.clone().attrs.clear();
            s
        })
        .collect();

    let tokens = quote! {};
    tokens.into()
}
