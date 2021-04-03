use std::collections::{BTreeSet, HashSet};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use darling::FromMeta;
use darling::util::Flag;
use proc_macro2::{Ident, TokenStream};
use proc_macro_error::{emit_error, emit_warning};
use quote::ToTokens;
use syn::{Attribute, FnArg, GenericArgument, GenericParam, ImplItemMethod, Item, ItemImpl, ItemMod, ItemStruct, Lit, parse_quote, Pat, Path, PathArguments, PatIdent, PatType, Type, TypePath, TypeReference, Visibility, PathSegment};
use syn::{Error, ImplItem, Meta, Token};
use syn::fold::Fold;
use syn::parse::{Parse, Parser, ParseStream, ParseBuffer};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::visit::Visit;

use imported::ImportedMethodTransformer;

use crate::transformation::exported::ExportedMethodTransformer;
use crate::utils::{canonicalize_path, get_abi, unique_ident};
use crate::validation::JNIBridgeModule;
use crate::transformation::context::StructContext;
use std::fmt;

#[macro_use]
mod utils;
mod exported;
mod imported;
mod context;

#[derive(Copy, Clone)]
pub(crate) enum ImplItemType {
    Exported,
    Imported,
    Unexported,
}

pub(crate) struct ModTransformer {
    module: JNIBridgeModule,
}

impl ModTransformer {
    pub(crate) fn new(module: JNIBridgeModule) -> Self {
        ModTransformer { module }
    }

    pub(crate) fn transform_module(&mut self) -> TokenStream {
        let module_decl = self.module.module_decl.clone();
        self.fold_item_mod(module_decl).into_token_stream()
    }

    /// If the impl block is a standard impl block for a type, makes every exported fn a freestanding one
    fn transform_item_impl(&mut self, node: ItemImpl) -> TokenStream {
        let mut impl_export_visitor = ImplExportVisitor::default();
        impl_export_visitor.visit_item_impl(&node);

        let (preserved_items, transformed_items) = if let Type::Path(p) = &*node.self_ty {
            let canonical_path = canonicalize_path(&p.path);
            let struct_name = canonical_path
                .to_token_stream()
                .to_string()
                .replace(" ", ""); // TODO: Replace String-based struct name matching with something more robust
            let struct_package = self.module.package_map.get(&struct_name).cloned().flatten();

            if struct_package.is_none() {
                emit_error!(p.path, "can't find package for struct `{}`", struct_name);
                return node.to_token_stream();
            }

            let path_lifetimes: BTreeSet<String> = p.path.segments.iter().filter_map(|s: &PathSegment| {
                if let PathArguments::AngleBracketed(a) = &s.arguments {
                    Some(a.args.iter().filter_map(|g| {
                        match g {
                            GenericArgument::Lifetime(l) => Some(l.ident.to_string()),
                            _ => None
                        }
                    }))
                } else {
                    None
                }
            }).flatten().collect();

            let struct_lifetimes: Vec<_> = node.generics.params.iter().filter_map(|p| {
                match p {
                    GenericParam::Lifetime(l) if path_lifetimes.contains(&l.lifetime.ident.to_string()) => {
                        Some(l.clone())
                    }
                    _ => None
                }
            })
                .collect();

            let context = StructContext {
                struct_type: p.path.clone(),
                struct_name,
                struct_lifetimes,
                package: struct_package,
            };

            let mut exported_fns_transformer = ExportedMethodTransformer {
                struct_context: &context
            };
            let mut imported_fns_transformer = ImportedMethodTransformer {
                struct_context: &context
            };
            let mut impl_cleaner = ImplCleaner;

            let preserved = impl_export_visitor
                .items
                .iter()
                .map(|(i, t)| {
                    let item = (*i).clone();
                    match t {
                        ImplItemType::Exported => impl_cleaner.fold_impl_item(item),
                        ImplItemType::Imported => imported_fns_transformer
                            .fold_impl_item(impl_cleaner.fold_impl_item(item)),
                        ImplItemType::Unexported => item,
                    }
                })
                .collect();

            let transformed = impl_export_visitor
                .items
                .into_iter()
                .filter_map(|(i, t)| match t {
                    ImplItemType::Exported => Some(i),
                    _ => None,
                })
                .cloned()
                .map(|i| exported_fns_transformer.fold_impl_item(i))
                .collect();

            (preserved, transformed)
        } else {
            (node.items, Vec::new())
        };

        let preserved_impl = ItemImpl {
            attrs: node
                .attrs
                .into_iter()
                .map(|a| self.fold_attribute(a))
                .collect(),
            generics: self.fold_generics(node.generics),
            self_ty: Box::new(self.fold_type(*node.self_ty)),
            items: preserved_items
                .into_iter()
                .map(|i| self.fold_impl_item(i))
                .collect(),
            ..node
        };

        transformed_items.iter().map(|i| i.to_token_stream()).fold(
            preserved_impl.into_token_stream(),
            |item, mut stream| {
                item.to_tokens(&mut stream);
                stream
            },
        )
    }
}

impl Fold for ModTransformer {
    fn fold_item(&mut self, node: Item) -> Item {
        match node {
            Item::Const(c) => Item::Const(self.fold_item_const(c)),
            Item::Enum(e) => Item::Enum(self.fold_item_enum(e)),
            Item::ExternCrate(c) => Item::ExternCrate(self.fold_item_extern_crate(c)),
            Item::Fn(f) => Item::Fn(self.fold_item_fn(f)),
            Item::ForeignMod(m) => Item::ForeignMod(self.fold_item_foreign_mod(m)),
            Item::Impl(i) => Item::Verbatim(self.transform_item_impl(i)),
            Item::Macro(m) => Item::Macro(self.fold_item_macro(m)),
            Item::Macro2(m) => Item::Macro2(self.fold_item_macro2(m)),
            Item::Mod(m) => Item::Mod(self.fold_item_mod(m)),
            Item::Static(s) => Item::Static(self.fold_item_static(s)),
            Item::Struct(s) => Item::Struct(self.fold_item_struct(s)),
            Item::Trait(t) => Item::Trait(self.fold_item_trait(t)),
            Item::TraitAlias(t) => Item::TraitAlias(self.fold_item_trait_alias(t)),
            Item::Type(t) => Item::Type(self.fold_item_type(t)),
            Item::Union(u) => Item::Union(self.fold_item_union(u)),
            Item::Use(u) => Item::Use(self.fold_item_use(u)),
            Item::Verbatim(_) => node,
            _ => node,
        }
    }

    fn fold_item_mod(&mut self, mut node: ItemMod) -> ItemMod {
        let allow_non_snake_case: Attribute = parse_quote! { #![allow(non_snake_case)] };

        node.attrs
            .extend_from_slice(&[allow_non_snake_case]);

        ItemMod {
            attrs: node.attrs,
            vis: self.fold_visibility(node.vis),
            mod_token: node.mod_token,
            ident: self.fold_ident(node.ident),
            content: node.content.map(|(brace, items)| {
                (
                    brace,
                    items.into_iter().map(|i| self.fold_item(i)).collect(),
                )
            }),
            semi: node.semi,
        }
    }

    fn fold_item_struct(&mut self, node: ItemStruct) -> ItemStruct {
        let struct_attributes = {
            /* The `#[bridge]` attribute macro has to discard `#[package()]` attributes, because they don't exists in standard Rust
             * and currently there is no way for attribute macros to automatically introduce inert attributes (see: https://doc.rust-lang.org/reference/attributes.html#active-and-inert-attributes
             * and rust-lang/issues/#65823).
             * However, we want `#[package()]` to also be used in combination with `Signature` auto-derive, and it *needs* a `#[package]` attribute on the struct it's applied on.
             * If we remove the package attribute blindly `Signature` cannot see it, and if we keep it `Signature` cannot remove it (auto-derive macros cannot modify the existing token stream as proc macros).
             * Here we check wether the struct has a `#[derive(Signature)]` (crudely with a string comparison and hoping the user never writes `#[derive(::robusta_jni::convert::Signature)]`)
             * if it is present we don't remove `#[package]`, otherwise we remove it.
             * This works because `Signature` auto-derive macros also declares `#[package]` as a helper attribute
             */
            let attributes = node.attrs.clone();

            let has_derive_signature = node.attrs.iter()
                .any(|a| {
                    let is_derive = a.path.get_ident().map(ToString::to_string).as_deref() == Some("derive");
                    let derived_traits = a.parse_args_with(Punctuated::<Ident, Token![,]>::parse_terminated);
                    let has_signature = derived_traits.map(|p| p.iter().any(|i| i.to_string().as_str() == "Signature")).unwrap_or(false);

                    is_derive && has_signature
                });

            if !has_derive_signature {
                attributes
                    .into_iter()
                    .filter(|a| {
                        a.path.to_token_stream().to_string().as_str() != "package"
                    })
                    .collect()
            } else {
                attributes
            }
        };

        ItemStruct {
            attrs: struct_attributes,
            vis: node.vis,
            struct_token: node.struct_token,
            ident: node.ident,
            generics: self.fold_generics(node.generics),
            fields: self.fold_fields(node.fields),
            semi_token: node.semi_token,
        }
    }
}

#[derive(Default)]
pub struct ImplExportVisitor<'ast> {
    pub(crate) items: Vec<(&'ast ImplItem, ImplItemType)>,
}

impl<'ast> Visit<'ast> for ImplExportVisitor<'ast> {
    fn visit_impl_item(&mut self, node: &'ast ImplItem) {
        match node {
            ImplItem::Method(method) => {
                let abi = get_abi(&method.sig);

                match abi.as_deref() {
                    Some("jni") => self.items.push((node, ImplItemType::Exported)),
                    Some("java") => self.items.push((node, ImplItemType::Imported)),
                    _ => self.items.push((node, ImplItemType::Unexported)),
                }
            }
            _ => self.items.push((node, ImplItemType::Unexported)),
        }
    }
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq)]
pub(crate) struct JavaPath(String);

impl Display for JavaPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for JavaPath {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let input = s.to_string().replace(' ', "");
        if input.contains('-') {
            Err("package names can't contain dashes".into())
        } else {
            Ok(JavaPath(input))
        }
    }
}

impl JavaPath {
    pub fn to_snake_case(&self) -> String {
        self.0.replace('.', "_")
    }
}

impl Parse for JavaPath {
    fn parse<'a>(input: &'a ParseBuffer<'a>) -> syn::Result<Self> {
        let tokens = Punctuated::<Ident, Token![.]>::parse_terminated(input)?
            .to_token_stream();
        let package = tokens
            .to_string();

        JavaPath::from_str(&package).map_err(|e| Error::new_spanned(tokens, e))
    }
}

impl FromMeta for JavaPath {
    fn from_value(value: &Lit) -> darling::Result<Self> {
        use darling::Error;

        if let Lit::Str(literal) = value {
            let path = literal.value();
            Self::from_string(&path)
        } else {
            Err(Error::custom("invalid type"))
        }
    }

    fn from_string(path: &str) -> darling::Result<Self> {
        use darling::Error;
        if path.contains('-') {
            Err(Error::custom(
                "invalid path: packages and classes cannot contain dashes",
            ))
        } else {
            let tokens = TokenStream::from_str(&path).map_err(|_| {
                Error::custom("cannot create token stream for java path parsing")
            })?;
            let _parsed: Punctuated<Ident, Token![.]> =
                Punctuated::<Ident, Token![.]>::parse_separated_nonempty
                    .parse(tokens.into())
                    .map_err(|e| Error::custom(format!("cannot parse java path ({})", e)))?;

            Ok(JavaPath(path.into()))
        }
    }
}

pub(crate) struct AttributeFilter<'ast> {
    pub whitelist: HashSet<Path>,
    pub filtered_attributes: Vec<&'ast Attribute>,
}

impl<'ast> AttributeFilter<'ast> {
    pub(crate) fn with_whitelist(whitelist: HashSet<Path>) -> Self {
        AttributeFilter {
            whitelist,
            filtered_attributes: Vec::new(),
        }
    }
}

impl<'ast> Visit<'ast> for AttributeFilter<'ast> {
    fn visit_attribute(&mut self, attribute: &'ast Attribute) {
        if self.whitelist.contains(&attribute.path) {
            self.filtered_attributes.push(attribute);
        }
    }
}

struct ImplCleaner;

impl Fold for ImplCleaner {
    fn fold_impl_item_method(&mut self, mut node: ImplItemMethod) -> ImplItemMethod {
        let abi = node
            .sig
            .abi
            .as_ref()
            .and_then(|l| l.name.as_ref().map(|n| n.value()));

        match (&node.vis, &abi.as_deref()) {
            (Visibility::Public(_), Some("jni")) => {
                node.sig.abi = None;
                node.attrs = node
                    .attrs
                    .into_iter()
                    .filter(|a| a.path.get_ident().map_or(false, |i| i != "call_type"))
                    .collect();

                node
            }
            (_, _) => node,
        }
    }
}

struct FreestandingTransformer {
    struct_type: Path,
    struct_name: String,
    fn_name: String,
}

impl FreestandingTransformer {
    fn new(struct_type: Path, struct_name: String, fn_name: String) -> Self {
        FreestandingTransformer {
            struct_type,
            struct_name,
            fn_name,
        }
    }
}

impl Fold for FreestandingTransformer {
    fn fold_fn_arg(&mut self, arg: FnArg) -> FnArg {
        match arg {
            FnArg::Receiver(r) => {
                let receiver_span = r.span();

                let needs_env_lifetime = self.struct_type.segments.iter().any(|s| {
                    if let PathArguments::AngleBracketed(a) = &s.arguments {
                        a.args
                            .iter()
                            .filter_map(|g| match g {
                                GenericArgument::Lifetime(l) => Some(l),
                                _ => None,
                            })
                            .all(|l| l.ident != "env")
                    } else {
                        false
                    }
                });

                if needs_env_lifetime {
                    emit_warning!(self.struct_type, "must have one `'env` lifetime in impl to support self methods when using lifetime-parametrized struct");
                }

                let self_type = match r.reference.clone() {
                    Some((and_token, lifetime)) => Type::Reference(TypeReference {
                        and_token,
                        lifetime,
                        mutability: r.mutability,
                        elem: Box::new(Type::Path(TypePath {
                            qself: None,
                            path: self.struct_type.clone(),
                        })),
                    }),

                    None => Type::Path(TypePath {
                        qself: None,
                        path: self.struct_type.clone(),
                    }),
                };

                FnArg::Typed(PatType {
                    attrs: r.attrs,
                    pat: Box::new(Pat::Ident(PatIdent {
                        attrs: vec![],
                        by_ref: None,
                        mutability: None,
                        ident: unique_ident(
                            &format!("receiver_{}_{}", self.struct_name, self.fn_name),
                            receiver_span,
                        ),
                        subpat: None,
                    })),
                    colon_token: Token![:](receiver_span),
                    ty: Box::new(parse_quote! { #self_type }),
                })
            }

            FnArg::Typed(t) => match &*t.pat {
                Pat::Ident(ident) if ident.ident == "self" => {
                    let pat_span = t.span();
                    let self_type = &*t.ty;
                    FnArg::Typed(PatType {
                        attrs: t.attrs,
                        pat: Box::new(Pat::Ident(PatIdent {
                            attrs: ident.attrs.clone(),
                            by_ref: ident.by_ref,
                            mutability: ident.mutability,
                            ident: unique_ident(
                                &format!("receiver_{}_{}", self.struct_name, self.fn_name),
                                pat_span,
                            ),
                            subpat: ident.subpat.clone(),
                        })),
                        colon_token: t.colon_token,
                        ty: Box::new(parse_quote! { #self_type }),
                    })
                }
                _ => FnArg::Typed(t),
            },
        }
    }
}

#[derive(Clone, Default, FromMeta)]
#[darling(default)]
pub struct SafeParams {
    pub(crate) exception_class: Option<JavaPath>,
    pub(crate) message: Option<String>,
}

#[derive(Clone, FromMeta)]
pub enum CallType {
    Safe(Option<SafeParams>),
    Unchecked(Flag),
}

pub struct CallTypeAttribute {
    pub(crate) attr: Attribute,
    pub(crate) call_type: CallType,
}

impl Parse for CallTypeAttribute {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let attribute = input
            .call(Attribute::parse_outer)?
            .first()
            .cloned()
            .ok_or_else(|| Error::new(input.span(), "Invalid parsing of `call_type` attribute "))?;

        if attribute
            .path
            .get_ident()
            .ok_or_else(|| Error::new(attribute.path.span(), "expected identifier for attribute"))?
            != "call_type"
        {
            return Err(Error::new(
                attribute.path.span(),
                "expected identifier `call_type` for attribute",
            ));
        }

        let attr_meta: Meta = attribute.parse_meta()?;

        // Special-case `call_type(safe)` without further parentheses
        // TODO: Find out if it's possible to use darling to allow `call_type(safe)` *and* `call_type(safe(message = "foo"))` etc.
        if attr_meta.to_token_stream().to_string() == "call_type(safe)" {
            Ok(CallTypeAttribute {
                attr: attribute,
                call_type: CallType::Safe(None),
            })
        } else {
            CallType::from_meta(&attr_meta)
                .map_err(|e| {
                    Error::new(
                        attr_meta.span(),
                        format!("invalid `call_type` attribute options ({})", e),
                    )
                })
                .map(|c| {
                    CallTypeAttribute {
                        attr: attribute,
                        call_type: c,
                    }
                })
        }
    }
}
