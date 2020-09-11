use proc_macro2::{Ident, TokenStream};
use quote::ToTokens;
use syn::{Attribute, ItemImpl, ItemMod, ItemStruct, parse_quote, Signature, Type};
use syn::fold::Fold;

use crate::validation::JNIBridgeModule;

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
        let module_decl = self.module.module_decl.clone();
        self.fold_item_mod(module_decl).into_token_stream()
    }
}

impl Fold for ModTransformer {
    fn fold_item_impl(&mut self, node: ItemImpl) -> ItemImpl {
        if let Type::Path(p) =  &*node.self_ty {
            let struct_name = p.path.segments.last().unwrap().ident.to_string();
            let struct_package = self.module.package_map[&struct_name].clone();
            let mut struct_transformer = ImplTransformer { struct_name, package: struct_package };

            struct_transformer.fold_item_impl(node)
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
                items: node.items.into_iter().map(|i| self.fold_impl_item(i)).collect()
            }
        }
    }

    fn fold_item_mod(&mut self, node: ItemMod) -> ItemMod {
        let allow_non_snake_case: Attribute = parse_quote!{ #![allow(non_snake_case)] };
        let allow_unused: Attribute = parse_quote!{ #![allow(unused)] };

        ItemMod {
            attrs: vec![allow_non_snake_case, allow_unused],
            vis: self.fold_visibility(node.vis),
            mod_token: node.mod_token,
            ident: self.fold_ident(node.ident),
            content: node.content.map(|(brace, items)| (brace, items.into_iter().map(|i| self.fold_item(i)).collect())),
            semi: node.semi
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
            semi_token: node.semi_token
        }
    }
}

struct ImplTransformer {
    pub(crate) struct_name: String,
    pub(crate) package: String
}

impl Fold for ImplTransformer {
    fn fold_signature(&mut self, node: Signature) -> Signature {
        let jni_method_name = {
            let mut res = self.package.clone().replace('.', "_");
            res.push('_');
            res.push_str(&self.struct_name);
            res
        };

        Signature {
            constness: node.constness,
            asyncness: node.asyncness,
            unsafety: node.unsafety,
            abi: node.abi,
            fn_token: node.fn_token,
            ident: Ident::new(&jni_method_name, node.ident.span()),
            generics: node.generics,
            paren_token: node.paren_token,
            inputs: node.inputs,
            variadic: node.variadic,
            output: node.output
        }
    }
}