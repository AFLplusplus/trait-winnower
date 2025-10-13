// src/analysis.rs
//! Analysis of Rust code.

#![deny(missing_docs)]

use crate::error::TraitError;
use syn::{
    Ident, ImplItemFn, Item, ItemEnum, ItemFn, ItemImpl, ItemStruct, ItemTrait, Path as SynPath,
    TraitItemFn, Type, TypeParamBound, punctuated::Punctuated, token::Plus, visit::Visit,
};

use paste::paste;
use proc_macro2::Span;

/// Reference to a Rust item in the AST.
pub enum ItemRef<'ast> {
    /// A free-standing function.
    Func(&'ast ItemFn),
    /// A struct definition.
    Struct(&'ast ItemStruct),
    /// An enum definition.
    Enum(&'ast ItemEnum),
    /// A trait definition.
    Trait(&'ast ItemTrait),
    /// An impl block.
    Impl(&'ast ItemImpl),
    /// A method in an impl block (inherent or trait impl).
    ImplMethod {
        /// The type being implemented for.
        self_ty: &'ast Type,
        /// The trait path, if this is a trait impl.
        trait_path: Option<&'ast SynPath>,
        /// The method itself.
        method: &'ast ImplItemFn,
    },
    /// A method in a trait definition.
    TraitMethod {
        /// The trait's identifier.
        trait_ident: &'ast Ident,
        /// The method itself.
        method: &'ast TraitItemFn,
    },
}

/// A lightweight identity/label for an inspected item.
pub struct ItemKey<'ast> {
    item: ItemRef<'ast>,
    label: String,
    span: Span,
}

/// Generate label-formatting helpers on `ItemKey`.
/// Each arm is `$fn_name: (args...) => "fmt";` and returns `String`.
macro_rules! define_item_labels {
    ( $( $fn_name:ident ( $($arg:ident),* ) => $fmt:expr ; )* ) => {
        impl<'ast> ItemKey<'ast> {
            $(
                #[allow(missing_docs, reason = "macro-generated code")]
                #[inline]
                pub fn $fn_name ( $( $arg: &str ),* ) -> String {
                    format!($fmt, $( $arg ),*)
                }
            )*
        }
    };
}

define_item_labels! {
    fn_label            (name)                => "// fn {}";
    struct_label        (name)                => "// struct {}";
    enum_label          (name)                => "// enum {}";
    trait_label         (name)                => "// trait {}";
    impl_inherent_label (self_ty)             => "// impl {}";
    impl_trait_label    (trait_path, self_ty) => "// impl {} for {}";
    impl_method_label   (owner, method)       => "// {}::{}";
    trait_method_label  (trait_name, method)  => "// trait {}::{}";
}

impl<'ast> ItemKey<'ast> {
    /// Convenience: require an ident or explain why not.
    #[inline]
    pub fn ident(&self) -> Option<&'ast syn::Ident> {
        self.ident_opt()
    }

    /// Get the item.
    #[inline]
    pub fn item(&self) -> &ItemRef<'ast> {
        &self.item
    }

    /// Get the span of the item.
    #[inline]
    pub fn span(&self) -> Span {
        self.span
    }

    #[inline]
    fn ident_opt(&self) -> Option<&'ast syn::Ident> {
        match self.item {
            ItemRef::Func(f) => Some(&f.sig.ident),
            ItemRef::Struct(s) => Some(&s.ident),
            ItemRef::Enum(e) => Some(&e.ident),
            ItemRef::Trait(t) => Some(&t.ident),
            ItemRef::Impl(_) => None,
            ItemRef::ImplMethod { method, .. } => Some(&method.sig.ident),
            ItemRef::TraitMethod { method, .. } => Some(&method.sig.ident),
        }
    }
}

impl<'ast> std::fmt::Display for ItemKey<'ast> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.label)
    }
}

macro_rules! define_bounds_types {
    ( $( $name:ident,)+ $(,)? ) => {
        $(
            #[allow(missing_docs, reason = "macro-generated code")]
            pub struct $name<'ast> {
                item: ItemKey<'ast>,
                type_params: Vec<TypeParamBounds>,
                where_preds: Vec<WhereTypeBounds>,
            }

            impl<'ast> $name<'ast> {
                #[allow(missing_docs, reason = "macro-generated code")]
                pub fn type_param_bounds(&self) -> &[TypeParamBounds] { &self.type_params }

                #[allow(missing_docs, reason = "macro-generated code")]
                pub fn where_bounds(&self) -> &[WhereTypeBounds] { &self.where_preds }

                #[allow(missing_docs, reason = "macro-generated code")]
                pub fn item_key(&self) -> &ItemKey<'ast> { &self.item }

            }
        )+
    };
}

define_bounds_types! {
    FnBounds,
    TraitMethodBounds,
    ImplMethodBounds,
    TraitBounds,
    ImplBounds,
    EnumBounds,
    StructBounds,
}

/// A collection of items found in a file.
pub struct ItemBounds<'ast> {
    fns: Vec<FnBounds<'ast>>,
    traits: Vec<TraitBounds<'ast>>,
    impls: Vec<ImplBounds<'ast>>,
    trait_methods: Vec<TraitMethodBounds<'ast>>,
    impl_methods: Vec<ImplMethodBounds<'ast>>,
    enums: Vec<EnumBounds<'ast>>,
    structs: Vec<StructBounds<'ast>>,
}

macro_rules! define_bounds_slice {
    ( $( $method:ident, $field:ident, $ty:ty )+ ) => {
        paste! {
            $(
                impl<'ast> ItemBounds<'ast> {
                    #[allow(missing_docs, reason = "macro-generated")]
                    pub fn $method(&self) -> &$ty {
                        &self.$field
                    }

                    #[allow(missing_docs, reason = "macro-generated")]
                    pub fn [< $method _mut >](&mut self) -> &mut $ty {
                        &mut self.$field
                    }
                }
            )+
        }
    };
}

paste! {
    define_bounds_slice! {
        fns, fns, Vec<FnBounds<'ast>>
        traits, traits, Vec<TraitBounds<'ast>>
        trait_methods, trait_methods, Vec<TraitMethodBounds<'ast>>
        impl_methods, impl_methods, Vec<ImplMethodBounds<'ast>>
        enums, enums, Vec<EnumBounds<'ast>>
        structs, structs, Vec<StructBounds<'ast>>
        impls, impls, Vec<ImplBounds<'ast>>
    }
}

impl<'ast> ItemBounds<'ast> {
    /// Parse a file from disk.
    pub fn parse_file(path: &std::path::Path) -> TraitError<syn::File> {
        let src = std::fs::read_to_string(path)?;
        Ok(syn::parse_file(&src)?)
    }

    /// Main entry: parse a file from disk and collect items.
    pub fn collect_items_in_file(file: &'ast syn::File) -> TraitError<ItemBounds<'ast>> {
        Self::collect_items_from_src(file)
    }

    /// Iterate over all items.
    pub fn iter_all_items(&self) -> impl Iterator<Item = &ItemKey<'ast>> {
        self.fns
            .iter()
            .map(|f| &f.item)
            .chain(self.traits.iter().map(|t| &t.item))
            .chain(self.impls.iter().map(|i| &i.item))
            .chain(self.trait_methods.iter().map(|t| &t.item))
            .chain(self.impl_methods.iter().map(|i| &i.item))
            .chain(self.enums.iter().map(|e| &e.item))
            .chain(self.structs.iter().map(|s| &s.item))
    }

    fn collect_items_from_src(file: &'ast syn::File) -> TraitError<ItemBounds<'ast>> {
        let mut v = Collector {
            out: ItemBounds::empty(),
        };
        v.visit_file(file);
        Ok(v.out)
    }

    fn empty() -> Self {
        Self {
            fns: Vec::new(),
            traits: Vec::new(),
            impls: Vec::new(),
            trait_methods: Vec::new(),
            impl_methods: Vec::new(),
            enums: Vec::new(),
            structs: Vec::new(),
        }
    }
}

struct Collector<'ast> {
    out: ItemBounds<'ast>,
}

/// Where a bound lives on a type parameter in the function's generic list.
pub struct TypeParamBounds {
    ident: Ident,
    bounds: Punctuated<TypeParamBound, Plus>,
    param_index: usize,
}

impl TypeParamBounds {
    /// The identifier of the type parameter.
    #[inline]
    pub fn ident(&self) -> &Ident {
        &self.ident
    }

    /// The bounds of the type parameter.
    #[inline]
    pub fn bounds(&self) -> &Punctuated<TypeParamBound, Plus> {
        &self.bounds
    }

    /// The index of the type parameter in the generic list.
    #[inline]
    pub fn param_index(&self) -> usize {
        self.param_index
    }
}

/// Where a bound lives on a type parameter in the function's generic list.
pub struct WhereTypeBounds {
    ty: Box<Type>,
    bounds: Punctuated<TypeParamBound, Plus>,
    pred_index: usize,
}

impl WhereTypeBounds {
    /// The bounded type.
    #[inline]
    pub fn bounded_ty(&self) -> &Type {
        &self.ty
    }

    /// The bounds of the type parameter.
    #[inline]
    pub fn bounds(&self) -> &Punctuated<TypeParamBound, Plus> {
        &self.bounds
    }

    /// The index of the type parameter in the generic list.
    #[inline]
    pub fn pred_index(&self) -> usize {
        self.pred_index
    }
}

impl<'ast> Collector<'ast> {
    fn type_param_bounds(&self, gens: &syn::Generics) -> Vec<TypeParamBounds> {
        use syn::{GenericParam, TypeParam};
        gens.params
            .iter()
            .enumerate()
            .filter_map(|(idx, p)| match p {
                GenericParam::Type(TypeParam { ident, bounds, .. }) if !bounds.is_empty() => {
                    Some(TypeParamBounds {
                        ident: ident.clone(),
                        bounds: bounds.clone(),
                        param_index: idx,
                    })
                }
                _ => None,
            })
            .collect()
    }

    fn where_bounds(&self, gens: &syn::Generics) -> Vec<WhereTypeBounds> {
        let mut out = Vec::new();
        if let Some(wc) = &gens.where_clause {
            for (pred_index, pred) in wc.predicates.iter().enumerate() {
                if let syn::WherePredicate::Type(t) = pred
                    && !t.bounds.is_empty()
                {
                    out.push(WhereTypeBounds {
                        ty: Box::new(t.bounded_ty.clone()),
                        bounds: t.bounds.clone(),
                        pred_index,
                    });
                }
            }
        }
        out
    }

    fn push_if_any<F>(&mut self, gens: &syn::Generics, mut push: F)
    where
        F: FnMut(&mut Self, Vec<TypeParamBounds>, Vec<WhereTypeBounds>),
    {
        let tp = self.type_param_bounds(gens);
        let wb = self.where_bounds(gens);
        if !tp.is_empty() || !wb.is_empty() {
            push(self, tp, wb);
        }
    }
}

impl<'ast> Visit<'ast> for Collector<'ast> {
    fn visit_item(&mut self, i: &'ast Item) {
        match i {
            Item::Fn(f) => {
                let name = f.sig.ident.to_string();
                let label = ItemKey::fn_label(&name);
                self.push_if_any(&f.sig.generics, |this, tp, wb| {
                    this.out.fns.push(FnBounds {
                        item: ItemKey {
                            item: ItemRef::Func(f),
                            label: label.clone(),
                            span: f.sig.ident.span(),
                        },
                        type_params: tp,
                        where_preds: wb,
                    });
                });
            }

            Item::Struct(s) => {
                let name = s.ident.to_string();
                let label = ItemKey::struct_label(&name);
                self.push_if_any(&s.generics, |this, tp, wb| {
                    this.out.structs.push(StructBounds {
                        item: ItemKey {
                            item: ItemRef::Struct(s),
                            label: label.clone(),
                            span: s.ident.span(),
                        },
                        type_params: tp,
                        where_preds: wb,
                    });
                });
            }

            Item::Enum(e) => {
                let name = e.ident.to_string();
                let label = ItemKey::enum_label(&name);
                self.push_if_any(&e.generics, |this, tp, wb| {
                    this.out.enums.push(EnumBounds {
                        item: ItemKey {
                            item: ItemRef::Enum(e),
                            label: label.clone(),
                            span: e.ident.span(),
                        },
                        type_params: tp,
                        where_preds: wb,
                    });
                });
            }

            Item::Trait(t) => {
                let trait_name = t.ident.to_string();
                let label = ItemKey::trait_label(&trait_name);
                self.push_if_any(&t.generics, |this, tp, wb| {
                    this.out.traits.push(TraitBounds {
                        item: ItemKey {
                            item: ItemRef::Trait(t),
                            label: label.clone(),
                            span: t.ident.span(),
                        },
                        type_params: tp,
                        where_preds: wb,
                    });
                });

                // Trait methods: generics live on the method *signature*.
                for it in &t.items {
                    if let syn::TraitItem::Fn(m) = it {
                        let trait_name = t.ident.to_string();
                        let mlabel =
                            ItemKey::trait_method_label(&trait_name, &m.sig.ident.to_string());
                        self.push_if_any(&m.sig.generics, |this, tp, wb| {
                            this.out.trait_methods.push(TraitMethodBounds {
                                item: ItemKey {
                                    item: ItemRef::TraitMethod {
                                        trait_ident: &t.ident,
                                        method: m,
                                    },
                                    label: mlabel.clone(),
                                    span: m.sig.ident.span(),
                                },
                                type_params: tp,
                                where_preds: wb,
                            });
                        });
                    }
                }
            }

            Item::Impl(im) => {
                use quote::ToTokens;
                let trait_path_ref: Option<&'ast syn::Path> = im.trait_.as_ref().map(|(_, p, _)| p);
                let self_ty_str = im.self_ty.to_token_stream().to_string();
                let impl_label = if let Some(tp) = trait_path_ref {
                    ItemKey::impl_trait_label(&tp.to_token_stream().to_string(), &self_ty_str)
                } else {
                    ItemKey::impl_inherent_label(&self_ty_str)
                };

                self.push_if_any(&im.generics, |this, tp, wb| {
                    this.out.impls.push(ImplBounds {
                        item: ItemKey {
                            item: ItemRef::Impl(im),
                            label: impl_label.clone(),
                            span: im.impl_token.span,
                        },
                        type_params: tp,
                        where_preds: wb,
                    });
                });

                // Impl methods (method generics are on the signature)
                for ii in &im.items {
                    if let syn::ImplItem::Fn(m) = ii {
                        let owner = trait_path_ref
                            .map(|tp| format!("{} for {}", tp.to_token_stream(), self_ty_str))
                            .unwrap_or_else(|| self_ty_str.clone());
                        let mlabel = ItemKey::impl_method_label(&owner, &m.sig.ident.to_string());

                        self.push_if_any(&m.sig.generics, |this, tp, wb| {
                            this.out.impl_methods.push(ImplMethodBounds {
                                item: ItemKey {
                                    item: ItemRef::ImplMethod {
                                        self_ty: &im.self_ty,
                                        trait_path: trait_path_ref,
                                        method: m,
                                    },
                                    label: mlabel.clone(),
                                    span: m.sig.ident.span(),
                                },
                                type_params: tp,
                                where_preds: wb,
                            });
                        });
                    }
                }
            }

            _ => {}
        }

        syn::visit::visit_item(self, i);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt;

    enum Label<'a> {
        Eq(&'a str),
        StartsWith(&'a str),
        Contains(&'a str),
    }

    impl<'a> fmt::Display for Label<'a> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Label::Eq(s) => write!(f, "== {:?}", s),
                Label::StartsWith(s) => write!(f, "starts_with {:?}", s),
                Label::Contains(s) => write!(f, "contains {:?}", s),
            }
        }
    }

    /// Helper: collect all item labels from a source string.
    fn labels_from_src(src: &str) -> TraitError<Vec<String>> {
        let file = syn::parse_file(src)?;
        let items = ItemBounds::collect_items_in_file(&file)?;
        Ok(items.iter_all_items().map(|i| i.label.clone()).collect())
    }

    fn assert_has(labels: &[String], expected: &[Label<'_>]) {
        for want in expected {
            let ok = match want {
                Label::Eq(s) => labels.iter().any(|l| l == s),
                Label::StartsWith(s) => labels.iter().any(|l| l.starts_with(s)),
                Label::Contains(s) => labels.iter().any(|l| l.contains(s)),
            };
            assert!(ok, "missing label that {}", want);
        }
    }

    fn assert_none(labels: &[String]) {
        assert!(
            labels.is_empty(),
            "expected no interesting items, but got: {:?}",
            labels
        );
    }

    #[test]
    fn item_bounds_fn() -> TraitError<()> {
        let src = r#"
        fn foo<T: Copy>() where T: Clone {
            let x: i32 = 1;
        }
        "#;
        let labels = labels_from_src(src)?;
        assert_eq!(labels.len(), 1);
        assert_has(&labels, &[Label::Eq("// fn foo")]);
        Ok(())
    }

    #[test]
    fn item_bounds_fn_no_bounds() -> TraitError<()> {
        let src = r#"
        fn bar() {}
        "#;
        let labels = labels_from_src(src)?;
        assert_none(&labels);
        Ok(())
    }

    #[test]
    fn item_bounds_fn_in_module_records_path() -> TraitError<()> {
        let src = r#"
        mod outer {
            fn foo<T: Copy>() {}
        }
        "#;
        let file = syn::parse_file(src)?;
        let items = ItemBounds::collect_items_in_file(&file)?;
        assert_eq!(items.fns().len(), 1);
        let info = &items.fns()[0];
        assert_eq!(info.item.label, "// fn foo");
        Ok(())
    }

    #[test]
    fn item_bounds_struct() -> TraitError<()> {
        let src = r#"
        struct Bar<T: Copy> where T: Clone {
            a: T,
        }
        "#;
        let labels = labels_from_src(src)?;
        assert_eq!(labels.len(), 1);
        assert_has(&labels, &[Label::Eq("// struct Bar")]);
        Ok(())
    }

    #[test]
    fn item_bounds_struct_no_bounds() -> TraitError<()> {
        let src = r#"
        struct Baz {
            a: i32,
        }
        "#;
        let labels = labels_from_src(src)?;
        assert_none(&labels);
        Ok(())
    }

    #[test]
    fn item_bounds_enum() -> TraitError<()> {
        let src = r#"
        enum Baz<T: Copy> where T: Clone {
            A(T),
            B,
        }
        "#;
        let labels = labels_from_src(src)?;
        assert_eq!(labels.len(), 1);
        assert_has(&labels, &[Label::Eq("// enum Baz")]);
        Ok(())
    }

    #[test]
    fn item_bounds_enum_no_bounds() -> TraitError<()> {
        let src = r#"
        enum Qux {
            A,
            B,
        }
        "#;
        let labels = labels_from_src(src)?;
        assert_none(&labels);
        Ok(())
    }

    #[test]
    fn item_bounds_trait_and_methods() -> TraitError<()> {
        let src = r#"
        trait Qux<T: Copy> where T: Clone {
            fn a(&self) where T: Default;
            fn b(&self) -> i32;
        }
        "#;
        let labels = labels_from_src(src)?;
        assert_has(
            &labels,
            &[Label::Eq("// trait Qux"), Label::Eq("// trait Qux::a")],
        );
        // Should not collect trait method b (no bounds)
        assert!(!labels.iter().any(|l| l == "// trait Qux::b"));
        Ok(())
    }

    #[test]
    fn item_bounds_trait_and_methods_no_bounds() -> TraitError<()> {
        let src = r#"
        trait Empty {
            fn a(&self);
            fn b(&self) -> i32;
        }
        "#;
        let labels = labels_from_src(src)?;
        assert_none(&labels);
        Ok(())
    }

    #[test]
    fn item_bounds_impl_trait_and_methods() -> TraitError<()> {
        let src = r#"
        trait T { fn m(&self); }
        struct S;
        impl<T: Copy> T for S where T: Clone {
            fn m(&self) where T: Default {}
            fn n(&self) {}
        }
        "#;
        let labels = labels_from_src(src)?;
        assert_has(
            &labels,
            &[Label::StartsWith("// impl T for S"), Label::Contains("::m")],
        );
        assert!(!labels.iter().any(|l| l.contains("::n")));
        Ok(())
    }

    #[test]
    fn item_bounds_impl_trait_and_methods_no_bounds() -> TraitError<()> {
        let src = r#"
        trait T { fn m(&self); }
        struct S;
        impl T for S {
            fn m(&self) {}
        }
        "#;
        let labels = labels_from_src(src)?;
        assert_none(&labels);
        Ok(())
    }

    #[test]
    fn item_bounds_impl_inherent_and_methods() -> TraitError<()> {
        let src = r#"
        struct S;
        impl<T: Copy> S where T: Clone {
            fn foo(&self) where T: Default {}
            fn bar(&self) {}
        }
        "#;
        let labels = labels_from_src(src)?;
        assert_has(
            &labels,
            &[Label::StartsWith("// impl S"), Label::Contains("::foo")],
        );
        assert!(!labels.iter().any(|l| l.contains("::bar")));
        Ok(())
    }

    #[test]
    fn item_bounds_impl_inherent_and_methods_no_bounds() -> TraitError<()> {
        let src = r#"
        struct S;
        impl S {
            fn foo(&self) {}
            fn bar(&self) {}
        }
        "#;
        let labels = labels_from_src(src)?;
        assert_none(&labels);
        Ok(())
    }
}

// TODO: Check supertraits and their methods.
