use std::collections::HashSet;
use std::str::FromStr;

use proc_macro2::{Ident, TokenStream};
use proc_macro_error::emit_error;
use quote::{quote_spanned, ToTokens};
use syn::{Abi, Attribute, Block, Error, Expr, FnArg, GenericParam, Generics, ImplItemMethod, Item, ItemImpl, ItemMod, ItemStruct, Lifetime, LifetimeDef, Lit, LitStr, parse_quote, Pat, PatIdent, PatType, ReturnType, Signature, Type, TypeReference, Visibility, VisPublic, Meta, Path, ImplItem};
use syn::fold::Fold;
use syn::parse::{Parse, ParseStream, Parser};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::Token;
use syn::token::Extern;
use syn::visit::Visit;

use darling::FromMeta;

use proc_macro_error::{emit_warning};

use crate::utils::unique_ident;
use crate::validation::JNIBridgeModule;
use darling::util::Flag;

#[derive(Default)]
struct ImplExportVisitor<'ast> {
    exported: Vec<&'ast ImplItem>,
    unexported: Vec<&'ast ImplItem>
}

impl<'ast> Visit<'ast> for ImplExportVisitor<'ast> {
    fn visit_impl_item(&mut self, node: &'ast ImplItem) {
        match node {
            ImplItem::Method(method) => {
                if let Some("jni") = method.sig.abi.as_ref().and_then(|a| a.name.as_ref().map(|l| l.value())).as_deref() {
                    self.exported.push(node)
                } else {
                    self.unexported.push(node)
                }
            }
            _ => self.unexported.push(node)
        }
    }
}

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
            let jni_path_prefix = if cfg!(feature = "no_jni") {
                ""
            } else {
                "::robusta_jni"
            };

            let mut items_with_use: Vec<Item> = vec![
                parse_quote! { use ::robusta_jni::convert::{FromJavaValue, IntoJavaValue, TryFromJavaValue, TryIntoJavaValue}; },
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
    /// If the impl block is a standard impl block for a type, makes every exported fn a freestanding one
    fn transform_item_impl(&mut self, node: ItemImpl) -> TokenStream {
        let mut impl_export_visitor = ImplExportVisitor::default();
        impl_export_visitor.visit_item_impl(&node);

        let (preserved_items, transformed_items) = if let Type::Path(p) = &*node.self_ty {
            let struct_name = p.path.get_ident().expect(&format!("expected identifier in item impl, got: {}", p.path.to_token_stream())).to_string();
            let struct_package = self.module.package_map[&struct_name].clone();
            let mut impl_transformer = ImplTransformer { struct_name, package: struct_package };

            let preserved = impl_export_visitor.unexported.into_iter().cloned().collect();
            let transformed = impl_export_visitor.exported.into_iter().cloned().map(|i| impl_transformer.fold_impl_item(i)).collect();

            (preserved, transformed)
        } else {
            (node.items, Vec::new())
        };

        let preserved_impl = ItemImpl {
            attrs: node.attrs.into_iter().map(|a| self.fold_attribute(a)).collect(),
            defaultness: node.defaultness,
            unsafety: node.unsafety,
            impl_token: node.impl_token,
            generics: self.fold_generics(node.generics),
            trait_: node.trait_,
            self_ty: Box::new(self.fold_type(*node.self_ty)),
            brace_token: node.brace_token,
            items: preserved_items.into_iter().map(|i| self.fold_impl_item(i)).collect(),
        };

        transformed_items.iter()
            .map(|i| i.to_token_stream())
            .fold(preserved_impl.into_token_stream(), |item, mut stream| {
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

    fn fold_item_mod(&mut self, mut node: ItemMod) -> ItemMod {
        let allow_non_snake_case: Attribute = parse_quote! { #![allow(non_snake_case)] };
        let allow_unused: Attribute = parse_quote! { #![allow(unused)] };

        node.attrs.extend_from_slice(&[allow_non_snake_case, allow_unused]);

        ItemMod {
            attrs: node.attrs,
            vis: self.fold_visibility(node.vis),
            mod_token: node.mod_token,
            ident: self.fold_ident(node.ident),
            content: node.content.map(|(brace, items)| (brace, items.into_iter().map(|i| self.fold_item(i)).collect())),
            semi: node.semi,
        }
    }

    fn fold_item_struct(&mut self, node: ItemStruct) -> ItemStruct {
        let discarded_known_attributes = {
            let mut known = HashSet::new();
            known.insert("package");
            known
        };

        let struct_attributes = {
            let attributes = node.attrs;
            attributes.into_iter().filter(|a| !discarded_known_attributes.contains(&a.path.to_token_stream().to_string().as_str())).collect()
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

#[derive(Clone)]
struct JavaPath(String);

impl FromMeta for JavaPath {
    fn from_value(value: &Lit) -> darling::Result<Self> {
        use darling::Error;

        if let Lit::Str(literal) = value {
            let path = literal.value();
            if path.contains('-') {
                Err(Error::custom("invalid path: packages and classes cannot contain dashes"))
            } else {
                let tokens = TokenStream::from_str(&path).map_err(|_| Error::custom("cannot create token stream for java path parsing"))?;
                let _parsed: Punctuated<Ident, Token![.]> = Punctuated::<Ident, Token![.]>::parse_separated_nonempty.parse(tokens.into()).map_err(|e| Error::custom(format!("cannot parse java path ({})", e)))?;

                Ok(JavaPath(path))
            }
        } else {
            Err(Error::custom("invalid type"))
        }
    }
}

#[derive(Clone, Default, FromMeta)]
#[darling(default)]
struct SafeParams {
    exception_class: Option<JavaPath>,
    message: Option<String>,
}

#[derive(Clone, FromMeta)]
enum CallType {
    Safe(Option<SafeParams>),
    Unchecked(Flag),
}

impl Parse for CallType {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let attribute = input.call(Attribute::parse_outer)?.first().cloned().ok_or_else(|| Error::new(input.span(), "Invalid parsing of `call_type` attribute "))?;

        if attribute.path.get_ident().ok_or_else(|| Error::new(attribute.path.span(), "expected identifier for attribute"))? != "call_type" {
            return Err(Error::new(attribute.path.span(), "expected identifier `call_type` for attribute"));
        }

        let attr_meta: Meta = attribute.parse_meta()?;

        // Special-case `call_type(safe)` without further parentheses
        // TODO: Find out if it's possible to use darling to allow `call_type(safe)` *and* `call_type(safe(message = "foo"))` etc.
        if attr_meta.to_token_stream().to_string() == "call_type(safe)" {
            Ok(CallType::Safe(None))
        } else {
            CallType::from_meta(&attr_meta).map_err(|e| Error::new(attr_meta.span(), format!("invalid `call_type` attribute options ({})", e)))
        }
    }
}

struct AttributeFilter<'ast> {
    pub whitelist: HashSet<Path>,
    pub filtered_attributes: Vec<&'ast Attribute>,
}

impl<'ast> AttributeFilter<'ast> {
    fn with_whitelist(whitelist: HashSet<Path>) -> Self {
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

struct ImplTransformer {
    pub(crate) struct_name: String,
    pub(crate) package: Option<String>,
}

impl Fold for ImplTransformer {
    fn fold_impl_item_method(&mut self, node: ImplItemMethod) -> ImplItemMethod {
        let abi = node.sig.abi.as_ref().and_then(|l| l.name.as_ref().map(|n| n.value()));
        match (&node.vis, &abi.as_deref()) {
            (Visibility::Public(_), Some("jni")) => {
                let whitelist = {
                    let mut f = HashSet::new();
                    f.insert(syn::parse2(TokenStream::from_str("call_type").unwrap()).unwrap());
                    f
                };

                let mut attributes_collector = AttributeFilter::with_whitelist(whitelist);
                attributes_collector.visit_impl_item_method(&node);

                let call_type_attribute = attributes_collector.filtered_attributes.first().and_then(|call_type_attr| {
                    syn::parse2(call_type_attr.to_token_stream()).map_err(|e| {
                        emit_warning!(e.span(), format!("invalid parsing of `call_type` attribute, defaulting to #[call_type(safe)]. {}", e));
                        e
                    }).ok()
                }).unwrap_or(CallType::Safe(None));

                let mut impl_method_transformer = ImplMethodTransformer::new(self.struct_name.clone(), self.package.clone(), call_type_attribute);
                impl_method_transformer.fold_impl_item_method(node)
            },
            _ => node
        }
    }
}

struct ImplMethodTransformer {
    struct_name: String,
    package: Option<String>,
    call_type: CallType,
}

impl ImplMethodTransformer {
    fn new(struct_name: String, package: Option<String>, call_type: CallType) -> Self {
        ImplMethodTransformer {
            struct_name,
            package,
            call_type,
        }
    }
}

impl Fold for ImplMethodTransformer {
    fn fold_impl_item_method(&mut self, node: ImplItemMethod) -> ImplItemMethod {
        let signature = node.sig.clone();

        let mut jni_signature_transformer = JNISignatureTransformer::new(self.struct_name.clone(), signature.ident.to_string(), self.call_type.clone());

        let call_inputs_idents: Punctuated<Expr, Token![,]> = signature.inputs.iter().cloned()
            .map(|arg| jni_signature_transformer.fold_fn_arg(arg))
            .map::<Expr, _>(|a| match a {
                FnArg::Receiver(_) => panic!("Bug -- please report to library author. Found receiver type in freestanding signature!"),
                FnArg::Typed(t) => {
                    match &*t.pat {
                        Pat::Ident(ident) => {
                            let ident = ident.ident.clone();
                            parse_quote!(#ident)
                        }
                        _ => panic!("Non-identifier argument pattern in function")
                    }
                }
            })
            .collect();

        let outer_call_inputs = {
            let mut result = call_inputs_idents.clone();

            if let CallType::Safe(_) = self.call_type {
                result.push(parse_quote!(&env))
            }

            result
        };

        let inner_call_inputs: Punctuated<Expr, Token![,]> = call_inputs_idents.into_iter()
            .map(|arg| {
                let input_param: Expr = {
                    match self.call_type {
                        CallType::Safe(_) => parse_quote! { TryFromJavaValue::try_from(#arg, &env)? },
                        CallType::Unchecked { .. } => parse_quote! { FromJavaValue::from(#arg, &env) }
                    }
                };
                input_param
            }).collect();

        let inner_fn_ident = Ident::new("inner", signature.span());

        let mut freestanding_transformer = FreestandingTransformer::new(self.struct_name.clone(), inner_fn_ident.to_string());
        let inner_fn_inputs: Punctuated<FnArg, Token![,]> = signature.inputs.iter().cloned()
            .map(|a| freestanding_transformer.fold_fn_arg(a))
            .collect();

        let inner_signature = {
            let mut s = signature.clone();
            s.ident = inner_fn_ident;
            s.inputs = inner_fn_inputs;
            s.abi = None;
            s
        };

        let outer_signature = {
            let mut s = jni_signature_transformer.fold_signature(signature.clone());
            s.ident = Ident::new("outer", signature.ident.span());

            if let CallType::Safe(_) = self.call_type {
                s.inputs.push(FnArg::Typed(PatType {
                    attrs: vec![],
                    pat: Box::new(Pat::Ident(PatIdent {
                        attrs: vec![],
                        by_ref: None,
                        mutability: None,
                        ident: Ident::new("env", s.inputs.span()),
                        subpat: None,
                    })),
                    colon_token: Token![:](s.inputs.span()),
                    ty: Box::new(parse_quote! { &JNIEnv<'env> }),
                }))
            }

            let outer_signature_span = s.span();
            let outer_output_type: Type = match s.output {
                ReturnType::Default => parse_quote!(()),
                ReturnType::Type(_, ty) => *ty
            };

            s.output = ReturnType::Type(Token![->](outer_signature_span), Box::new(parse_quote!(::jni::errors::Result<#outer_output_type>)));
            s.abi = None;
            s
        };

        let node_span = node.span();
        let method_impl = node.block;

        let new_block: Block = match &self.call_type {
            CallType::Unchecked { .. } => parse_quote! {{
                #inner_signature
                    #method_impl

                IntoJavaValue::into(inner(#inner_call_inputs), &env)
            }},

            CallType::Safe(exception_details) => {
                let (default_exception_class, default_message) = ("java/lang/RuntimeException", "JNI conversion error!");
                let (exception_class, message) = match exception_details {
                    Some(SafeParams { exception_class, message }) => {
                        let exception_class_result = exception_class.as_ref().map(|v| &v.0).map(AsRef::as_ref).unwrap_or(default_exception_class);
                        let message_result = message.as_ref().map(AsRef::as_ref).unwrap_or(default_message);

                        (exception_class_result, message_result)
                    }
                    None => (default_exception_class, default_message)
                };

                parse_quote! {{
                    #outer_signature {
                        #inner_signature
                            #method_impl

                        TryIntoJavaValue::try_into(inner(#inner_call_inputs), &env)
                    }

                    match outer(#outer_call_inputs) {
                        Ok(result) => result,
                        Err(_) => {
                            env.throw_new(#exception_class, #message).unwrap();

                            /* We never hand out Rust references and the object returned is ignored
                             * by the JVM, so it should be safe to just return zeroed memory.
                             * Also, all primitives have a valid zero representation and because objects
                             * are represented as pointers this should not have any unsafe side effects.
                             * (Uninitialized memory would probably work as well)
                             */
                            unsafe { ::std::mem::zeroed() }
                        }
                    }
                }}
            }
        };

        let no_mangle = parse_quote! { #[no_mangle] };
        let impl_item_attributes = {
            let mut attributes = node.attrs;
            attributes.push(no_mangle);

            let discarded_known_attributes: HashSet<&str> = {
                let mut h = HashSet::new();
                h.insert("call_type");
                h
            };

            attributes.into_iter().filter(|a| {
                !discarded_known_attributes.contains(&a.path.segments.to_token_stream().to_string().as_str())
            }).collect()
        };

        ImplItemMethod {
            attrs: impl_item_attributes,
            vis: Visibility::Public(VisPublic {
                pub_token: Token![pub](node_span)
            }),
            defaultness: node.defaultness,
            sig: self.fold_signature(node.sig),
            block: new_block,
        }
    }

    /// Transform original signature in JNI-ready one, including JClass and JNIEnv parameters into the function signature.
    fn fold_signature(&mut self, node: Signature) -> Signature {
        if node.ident.to_string().contains('_') {
            emit_error!(node.ident, "JNI methods cannot contain `_` character");
        }

        let jni_method_name = {
            let snake_case_package = self.package.clone().map(|s| {
                let mut s = s.replace('.', "_");
                s.push('_');
                s
            }).unwrap_or_else(|| "".into());

            format!("Java_{}{}_{}", snake_case_package, self.struct_name, node.ident.to_string())
        };

        let mut jni_signature_transformer = JNISignatureTransformer::new(self.struct_name.clone(), node.ident.to_string(), self.call_type.clone());

        let jni_abi_inputs: Punctuated<FnArg, Token![,]> = {
            let mut res = Punctuated::new();
            res.push(parse_quote!(env: JNIEnv<'env>));
            res.push(parse_quote!(class: JClass));

            let jni_compatible_inputs: Punctuated<_, Token![,]> = node.inputs.iter().cloned().map(|input| {
                jni_signature_transformer.fold_fn_arg(input)
            }).collect();

            res.extend(jni_compatible_inputs);
            res
        };

        let jni_output = jni_signature_transformer.fold_return_type(node.output.clone());

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
            generics: parse_quote! { <'env> },
            paren_token: node.paren_token,
            inputs: jni_abi_inputs,
            variadic: node.variadic,
            output: jni_output,
        }
    }
}

struct FreestandingTransformer {
    struct_name: String,
    fn_name: String,
}

impl FreestandingTransformer {
    fn new(struct_name: String, fn_name: String) -> Self {
        FreestandingTransformer {
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
                    attrs: r.attrs,
                    pat: Box::new(Pat::Ident(PatIdent {
                        attrs: vec![],
                        by_ref: None,
                        mutability: None,
                        ident: unique_ident(&format!("receiver_{}_{}", self.struct_name, self.fn_name), receiver_span),
                        subpat: None,
                    })),
                    colon_token: Token![:](receiver_span),
                    ty: Box::new(self_type),
                })
            }

            FnArg::Typed(t) => {
                match &*t.pat {
                    Pat::Ident(ident) if ident.ident == "self" => {
                        let pat_span = t.span();
                        FnArg::Typed(PatType {
                            attrs: t.attrs,
                            pat: Box::new(Pat::Ident(PatIdent {
                                attrs: ident.attrs.clone(),
                                by_ref: ident.by_ref,
                                mutability: ident.mutability,
                                ident: unique_ident(&format!("receiver_{}_{}", self.struct_name, self.fn_name), pat_span),
                                subpat: ident.subpat.clone(),
                            })),
                            colon_token: t.colon_token,
                            ty: t.ty.clone(),
                        })
                    },

                    _ => FnArg::Typed(t)
                }
            }
        }
    }
}

struct JNISignatureTransformer {
    struct_name: String,
    fn_name: String,
    call_type: CallType,
}

impl JNISignatureTransformer {
    fn new(struct_name: String, fn_name: String, call_type: CallType) -> Self {
        JNISignatureTransformer {
            struct_name,
            fn_name,
            call_type,
        }
    }
}

impl Fold for JNISignatureTransformer {
    fn fold_fn_arg(&mut self, arg: FnArg) -> FnArg {
        let mut freestanding_transformer = FreestandingTransformer::new(self.struct_name.clone(), self.fn_name.clone());

        match freestanding_transformer.fold_fn_arg(arg) {
            FnArg::Receiver(_) => panic!("Bug -- please report to library author. Found receiver input after freestanding conversion"),
            FnArg::Typed(t) => {
                let original_input_type = t.ty;

                let jni_conversion_type: Type = match self.call_type {
                    CallType::Safe(_) => syn::parse2(quote_spanned! { original_input_type.span() => <#original_input_type as TryFromJavaValue<'env>>::Source }).unwrap(),
                    CallType::Unchecked { .. } => syn::parse2(quote_spanned! { original_input_type.span() => <#original_input_type as FromJavaValue<'env>>::Source }).unwrap(),
                };

                FnArg::Typed(PatType {
                    attrs: t.attrs,
                    pat: t.pat,
                    colon_token: t.colon_token,
                    ty: Box::new(jni_conversion_type),
                })
            }
        }
    }

    fn fold_generics(&mut self, mut generics: Generics) -> Generics {
        generics.params.push(GenericParam::Lifetime(LifetimeDef {
            attrs: vec![],
            lifetime: Lifetime {
                apostrophe: generics.span(),
                ident: Ident::new("env", generics.span()),
            },
            colon_token: None,
            bounds: Default::default(),
        }));

        generics
    }

    fn fold_return_type(&mut self, return_type: ReturnType) -> ReturnType {
        match return_type {
            ReturnType::Default => return_type,
            ReturnType::Type(ref arrow, ref rtype) => {
                match (&**rtype, self.call_type.clone()) {
                    (Type::Path(p), CallType::Unchecked { .. }) => {
                        ReturnType::Type(*arrow, syn::parse2(quote_spanned! { p.span() => <#p as IntoJavaValue<'env>>::Target }).unwrap())
                    }

                    (Type::Path(p), CallType::Safe(_)) => {
                        ReturnType::Type(*arrow, syn::parse2(quote_spanned! { p.span() => <#p as TryIntoJavaValue<'env>>::Target }).unwrap())
                    }

                    (Type::Reference(r), CallType::Unchecked { .. }) => {
                        ReturnType::Type(*arrow, syn::parse2(quote_spanned! { r.span() => <#r as IntoJavaValue<'env>>::Target }).unwrap())
                    }

                    (Type::Reference(r), CallType::Safe(_)) => {
                        ReturnType::Type(*arrow, syn::parse2(quote_spanned! { r.span() => <#r as TryIntoJavaValue<'env>>::Target }).unwrap())
                    }
                    _ => {
                        emit_error!(return_type, "Only type or type paths are permitted as type ascriptions in function params");
                        return_type
                    }
                }
            }
        }
    }
}
