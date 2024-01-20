use core::option::Option::{None, Some};
use core::result::Result::{Err, Ok};
use std::collections::BTreeMap;

use proc_macro_error::{emit_error, emit_warning};
use quote::ToTokens;
use syn::parse::{Parse, ParseBuffer};
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{Attribute, Error, GenericParam, Item, ItemImpl, ItemMod, ItemStruct, Result, Type};

use crate::transformation::JavaPath;

struct AttribItemChecker {
    valid: bool,
}

impl AttribItemChecker {
    fn new() -> Self {
        AttribItemChecker { valid: true }
    }
}

impl<'ast> Visit<'ast> for AttribItemChecker {
    fn visit_item(&mut self, node: &'ast Item) {
        let has_package_attribute =
            |a: &Attribute| a.path().segments.first().unwrap().ident == "package";
        match node {
            Item::Struct(_) => {}
            Item::Const(i) if i.attrs.iter().any(has_package_attribute) => {
                emit_error!(i.span(), "`package` attribute used on non-struct type");
                self.valid = false;
            }
            Item::Enum(i) if i.attrs.iter().any(has_package_attribute) => {
                emit_error!(i.span(), "`package` attribute used on non-struct type"; help = i.enum_token.span() => "replace `enum` with `struct`");
                self.valid = false;
            }
            Item::ExternCrate(i) if i.attrs.iter().any(has_package_attribute) => {
                emit_error!(i.span(), "`package` attribute used on non-struct type");
                self.valid = false;
            }
            Item::Fn(i) if i.attrs.iter().any(has_package_attribute) => {
                emit_error!(i.span(), "`package` attribute used on non-struct type");
                self.valid = false;
            }
            Item::ForeignMod(i) => {
                emit_error!(i.span(), "`package` attribute used on non-struct type");
                self.valid = false;
            }
            Item::Impl(i) if i.attrs.iter().any(has_package_attribute) => {
                emit_error!(i.span(), "`package` attribute used on non-struct type");
                self.valid = false;
            }
            Item::Macro(i) if i.attrs.iter().any(has_package_attribute) => {
                emit_error!(i.span(), "`package` attribute used on non-struct type");
                self.valid = false;
            }
            Item::Mod(i) if i.attrs.iter().any(has_package_attribute) => {
                emit_error!(i.span(), "`package` attribute used on non-struct type");
                self.valid = false;
            }
            Item::Static(i) if i.attrs.iter().any(has_package_attribute) => {
                emit_error!(i.span(), "`package` attribute used on non-struct type");
                self.valid = false;
            }
            Item::Trait(i) if i.attrs.iter().any(has_package_attribute) => {
                emit_error!(i.span(), "`package` attribute used on non-struct type");
                self.valid = false;
            }
            Item::TraitAlias(i) if i.attrs.iter().any(has_package_attribute) => {
                emit_error!(i.span(), "`package` attribute used on non-struct type");
                self.valid = false;
            }
            Item::Type(i) if i.attrs.iter().any(has_package_attribute) => {
                emit_error!(i.span(), "`package` attribute used on non-struct type");
                self.valid = false;
            }
            Item::Union(i) if i.attrs.iter().any(has_package_attribute) => {
                emit_error!(i.span(), "`package` attribute used on non-struct type");
                self.valid = false;
            }
            Item::Use(i) if i.attrs.iter().any(has_package_attribute) => {
                emit_error!(i.span(), "`package` attribute used on non-struct type");
                self.valid = false;
            }
            Item::Verbatim(_) => {}
            _ => {}
        }
    }
}

#[derive(Default)]
struct ImplAccumulator<'ast> {
    impls: Vec<&'ast ItemImpl>,
}

impl<'ast> Visit<'ast> for ImplAccumulator<'ast> {
    fn visit_item_impl(&mut self, node: &'ast ItemImpl) {
        self.impls.push(node);
    }
}

enum StructDeclarationKind {
    // structs with `package` attrib and impl
    Bridged,
    // structs with `package` attrib but no impl
    UnImpl,
    // structs without `package` attrib but with impl
    UnAttrib,
    // structs without `package` attrib and no impl
    Bare,
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
        let has_package_attrib = node
            .attrs
            .iter()
            .any(|a| a.path().segments.first().unwrap().ident == "package");
        let has_impl = self
            .module_impls
            .iter()
            .filter_map(|i| match &*i.self_ty {
                Type::Path(p) => Some(p.path.segments.last().unwrap().ident.to_string()),
                _ => None,
            })
            .any(|s| s == struct_name);

        let declaration_kind = match (has_package_attrib, has_impl) {
            (true, true) => StructDeclarationKind::Bridged,
            (true, false) => StructDeclarationKind::UnImpl,
            (false, true) => StructDeclarationKind::UnAttrib,
            (false, false) => StructDeclarationKind::Bare,
        };

        self.module_structs.push((node, declaration_kind))
    }
}

pub(crate) struct JNIBridgeModule {
    pub(crate) module_decl: ItemMod,
    pub(crate) package_map: BTreeMap<String, Option<JavaPath>>,
}

impl Parse for JNIBridgeModule {
    fn parse(input: &ParseBuffer) -> Result<Self> {
        let mut valid_input;
        let module_decl: ItemMod = input.parse().map_err(|e| {
            Error::new(
                e.span(),
                "`bridge` attribute is supported on mod items only",
            )
        })?;

        let mut attribute_checker = AttribItemChecker::new();
        attribute_checker.visit_item_mod(&module_decl);
        valid_input = attribute_checker.valid;

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
                        valid_input = false;
                        None
                    }
                    StructDeclarationKind::Bare => {
                        emit_warning!(struct_item, "ignoring struct with no `package` attribute and no implementation";
                            help = struct_item.span() => "add a #[package(...)] attribute";
                            note = "structs with declared methods require package attribute for correct translation");
                        None
                    }
                }
            })
            .collect();

        let structs_idents: Vec<_> = bridged_structs.iter().map(|s| &s.ident).collect();
        let bridged_impls: Vec<_> = mod_visitor
            .module_impls
            .iter()
            .filter_map(|item_impl| match &*item_impl.self_ty {
                Type::Path(p) => structs_idents
                    .iter()
                    .position(|id| *id == &p.path.segments.last().unwrap().ident)
                    .map(|pos| (bridged_structs[pos], *item_impl)),
                _ => None,
            })
            .map(|(s, i)| (s.clone(), i.clone()))
            .collect();

        mod_visitor
            .module_impls
            .into_iter()
            .filter(|i| {
                if let Type::Path(p) = &*i.self_ty {
                    let impl_struct_name = p.path.segments.last().unwrap().ident.to_string();
                    let has_generics = i
                        .generics
                        .params
                        .iter()
                        .filter_map(|g| match g {
                            GenericParam::Type(t) => Some(&t.ident),
                            _ => None,
                        })
                        .next()
                        .is_some();

                    !bridged_impls
                        .iter()
                        .map(|(_, i)| i)
                        .filter_map(|i| {
                            // *Very* conservative check to avoid hassles with checking struct name in where clauses
                            // Should refactor into something proper or just delete this
                            if !has_generics {
                                match &*i.self_ty {
                                    Type::Path(p) => {
                                        Some(p.path.segments.last().unwrap().ident.to_string())
                                    }
                                    _ => None,
                                }
                            } else {
                                Some(impl_struct_name.clone()) // ignore this impl item
                            }
                        })
                        .any(|struct_name| struct_name == impl_struct_name)
                } else {
                    false
                }
            })
            .for_each(|lone_impl| {
                emit_error!(
                    lone_impl,
                    "impl declared without corresponding struct \"{}\"",
                    lone_impl.self_ty.to_token_stream()
                );
                valid_input = false;
            });

        let package_map: BTreeMap<String, Option<JavaPath>> = bridged_structs
            .iter()
            .map(|s| {
                let name = s.ident.to_string();
                let package_path = s
                    .attrs
                    .iter()
                    .filter(|a| a.path().segments.last().unwrap().ident == "package")
                    .map(|a| a.parse_args::<JavaPath>().unwrap())
                    .next()
                    .unwrap();

                let package = Some(package_path);

                (name, package)
            })
            .collect();

        if !valid_input {
            Err(Error::new(
                module_decl.span(),
                "`bridge` macro expansion failed due to previous errors",
            ))
        } else {
            Ok(JNIBridgeModule {
                module_decl,
                package_map,
            })
        }
    }
}
