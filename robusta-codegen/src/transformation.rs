use proc_macro2::{Ident, TokenStream};
use proc_macro_error::emit_error;
use quote::{quote_spanned, ToTokens};
use syn::{Abi, Attribute, Block, Expr, FnArg, ImplItemMethod, Item, ItemImpl, ItemMod, ItemStruct, LitStr, parse_quote, Pat, PatIdent, PatType, ReturnType, Signature, Type, TypeReference, Visibility, VisPublic};
use syn::fold::Fold;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::Token;
use syn::token::Extern;

use crate::utils::unique_ident;
use crate::validation::JNIBridgeModule;
use std::str::FromStr;

pub(crate) struct ModTransformer {
    module: JNIBridgeModule
}

impl ModTransformer {
    pub(crate) fn new(module: JNIBridgeModule) -> Self {
        ModTransformer {
            module
        }
    }

    pub(crate) fn transform_module(&mut self) -> TokenStream {
        let mut module_decl = self.module.module_decl.clone();
        if let Some((brace, mut items)) = module_decl.content {
            /* FIXME: Somehow, enabling `no_jni` on `robusta` Cargo.toml doesn't do anything.
                Will investigate, for now this doesn't do anything wrong ¯\_(ツ)_/¯ */
            let jni_path_prefix = if cfg!(no_jni) {
                ""
            } else {
                "::robusta"
            };

            let mut items_with_use: Vec<Item> = vec![
                parse_quote!{ use ::robusta::convert::{FromJavaValue, IntoJavaValue}; },
                syn::parse2(TokenStream::from_str(&format!("use {}::jni::JNIEnv;", jni_path_prefix)).unwrap()).unwrap(),
                syn::parse2(TokenStream::from_str(&format!("use {}::jni::objects::JClass;", jni_path_prefix)).unwrap()).unwrap()
            ];
            items_with_use.append(&mut items);

            module_decl.content = Some((brace, items_with_use));
        }

        self.fold_item_mod(module_decl).into_token_stream()
    }
}

impl ModTransformer {
    fn transform_item_impl(&mut self, node: ItemImpl) -> TokenStream {
        let transformed_item_impl = if let Type::Path(p) = &*node.self_ty {
            let struct_name = p.path.segments.last().unwrap().ident.to_string();
            let struct_package = self.module.package_map[&struct_name].clone();
            let mut impl_transformer = ImplFnTransformer { struct_name, package: struct_package };

            impl_transformer.fold_item_impl(node)
        } else {
            ItemImpl {
                attrs: node.attrs.into_iter().map(|a| self.fold_attribute(a)).collect(),
                defaultness: node.defaultness,
                unsafety: node.unsafety,
                impl_token: node.impl_token,
                generics: self.fold_generics(node.generics),
                trait_: node.trait_,
                self_ty: Box::new(self.fold_type(*node.self_ty)),
                brace_token: node.brace_token,
                items: node.items.into_iter().map(|i| self.fold_impl_item(i)).collect(),
            }
        };

        transformed_item_impl.items.iter()
            .map(|i| i.to_token_stream())
            .fold(TokenStream::new(), |item, mut stream| {
                item.to_tokens(&mut stream);
                stream
            })
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
            Item::Impl(i) => {
                Item::Verbatim(self.transform_item_impl(i))
            }
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

    fn fold_item_mod(&mut self, node: ItemMod) -> ItemMod {
        let allow_non_snake_case: Attribute = parse_quote! { #![allow(non_snake_case)] };
        let allow_unused: Attribute = parse_quote! { #![allow(unused)] };

        ItemMod {
            attrs: vec![allow_non_snake_case, allow_unused],
            vis: self.fold_visibility(node.vis),
            mod_token: node.mod_token,
            ident: self.fold_ident(node.ident),
            content: node.content.map(|(brace, items)| (brace, items.into_iter().map(|i| self.fold_item(i)).collect())),
            semi: node.semi,
        }
    }

    fn fold_item_struct(&mut self, node: ItemStruct) -> ItemStruct {
        ItemStruct {
            attrs: vec![],
            vis: node.vis,
            struct_token: node.struct_token,
            ident: node.ident,
            generics: self.fold_generics(node.generics),
            fields: self.fold_fields(node.fields),
            semi_token: node.semi_token,
        }
    }
}

struct ImplFnTransformer {
    pub(crate) struct_name: String,
    pub(crate) package: Option<String>,
}

impl ImplFnTransformer {
    fn wrap_fn_block(&mut self, mut signature: Signature, node: Block) -> Block {
        signature.ident = Ident::new("inner", signature.span());
        let call_inputs: Punctuated<Expr, Token![,]> = signature.inputs.iter().map(|a| match a {
            FnArg::Receiver(_) => panic!("Bug -- please report to library author. Found receiver type in freestanding signature!"),
            FnArg::Typed(t) => {
                match &*t.pat {
                    Pat::Ident(ident) => {
                        ident.ident.clone()
                    },
                    _ => panic!("Non-identifier argument pattern in function")
                }
            },
        }).map(|i: Ident| {
            let input_param: Expr = parse_quote!{ FromJavaValue::from(#i, &env) };
            input_param
        }).collect();

        let result: Block = parse_quote! {{
            #signature
                #node

            IntoJavaValue::into(inner(#call_inputs), &env)
        }};

        result
    }
}

impl Fold for ImplFnTransformer {
    fn fold_impl_item_method(&mut self, node: ImplItemMethod) -> ImplItemMethod {
        let no_mangle = parse_quote! { #[no_mangle] };
        ImplItemMethod {
            attrs: vec![no_mangle],
            vis: Visibility::Public(VisPublic {
                pub_token: Token![pub](node.span())
            }),
            defaultness: node.defaultness,
            sig: self.fold_signature(node.sig.clone()),
            block: self.wrap_fn_block(node.sig, node.block),
        }
    }

    fn fold_signature(&mut self, node: Signature) -> Signature {
        if node.ident.to_string().contains('_') {
            emit_error!(node.ident, "native methods cannot contain `_` character");
        }

        let jni_method_name = {
            let snake_case_package = self.package.clone().map(|s| {
                let mut s = s.replace('.', "_");
                s.push('_');
                s
            }).unwrap_or("".into());

            format!("Java_{}{}_{}", snake_case_package, self.struct_name, node.ident.to_string())
        };

        let freestanding_inputs = node.inputs.iter()
            .map(|arg| {
                match arg {
                    FnArg::Receiver(r) => {
                        let receiver_span = r.span();
                        let struct_type_ident = Type::Verbatim(Ident::new(&self.struct_name, receiver_span).to_token_stream());

                        let self_type = match r.reference.clone() {
                            Some((and_token, lifetime)) => {
                                Type::Reference(TypeReference {
                                    and_token,
                                    lifetime,
                                    mutability: r.mutability,
                                    elem: Box::new(struct_type_ident),
                                })
                            }

                            None => Type::Verbatim(struct_type_ident.to_token_stream())
                        };

                        FnArg::Typed(PatType {
                            attrs: r.attrs.clone(),
                            pat: Box::new(Pat::Ident(PatIdent {
                                attrs: vec![],
                                by_ref: None,
                                mutability: None,
                                ident: unique_ident(&format!("receiver_{}", self.struct_name), receiver_span),
                                subpat: None,
                            })),
                            colon_token: Token![:](receiver_span),
                            ty: Box::new(self_type),
                        })
                    }

                    FnArg::Typed(t) => {
                        if let Pat::Ident(ident) = &*t.pat {
                            if ident.ident == "self" {
                                FnArg::Typed(PatType {
                                    attrs: vec![],
                                    pat: Box::new(Pat::Ident(PatIdent {
                                        attrs: ident.attrs.clone(),
                                        by_ref: ident.by_ref,
                                        mutability: ident.mutability,
                                        ident: unique_ident(&format!("receiver_{}", self.struct_name), t.span()),
                                        subpat: ident.subpat.clone()
                                    })),
                                    colon_token: t.colon_token,
                                    ty: t.ty.clone()
                                })
                            } else {
                                arg.clone()
                            }
                        } else {
                            arg.clone()
                        }
                    }
                }
            });

        let jni_abi_inputs: Punctuated<FnArg, Token![,]> = {
            let mut res = Punctuated::new();
            res.push(parse_quote!(env: JNIEnv<'env>));
            res.push(parse_quote!(class: JClass));

            let jni_compatible_inputs: Punctuated<_, Token![,]> = freestanding_inputs.map(|input| {
                match input {
                    FnArg::Receiver(_) => panic!("Bug -- please report to library author. Found receiver input after freestanding conversion"),
                    FnArg::Typed(t) => {
                        let original_input_type = t.ty;

                        FnArg::Typed(PatType {
                            attrs: t.attrs,
                            pat: t.pat,
                            colon_token: t.colon_token,
                            ty: Box::new(syn::parse2(quote_spanned!{ original_input_type.span() => <#original_input_type as FromJavaValue<'env>>::Source }).unwrap())
                        })
                    },
                }
            }).collect();

            res.extend(jni_compatible_inputs);
            res
        };

        let jni_output = match &node.output {
            ReturnType::Default => node.output.clone(),
            ReturnType::Type(arrow, rtype) => {
                match &**rtype {
                    Type::Path(p) => {
                        ReturnType::Type(*arrow, syn::parse2(quote_spanned!{ p.span() => <#p as IntoJavaValue<'env>>::Target }).unwrap())
                    },
                    _ => {
                        let res = node.output.clone();
                        emit_error!(res, "Only type or type paths are permitted as type ascriptions in function params");
                        res
                    }
                }
            },
        };

        Signature {
            constness: node.constness,
            asyncness: node.asyncness,
            unsafety: node.unsafety,
            abi: Some(Abi {
                extern_token: Extern { span: node.span() },
                name: Some(LitStr::new("system", node.span())),
            }),
            fn_token: node.fn_token,
            ident: Ident::new(&jni_method_name, node.ident.span()),
            generics: parse_quote!{ <'env> },
            paren_token: node.paren_token,
            inputs: jni_abi_inputs,
            variadic: node.variadic,
            output: jni_output,
        }
    }
}