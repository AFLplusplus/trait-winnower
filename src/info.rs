// src/print.rs
//! Print trait bounds.

#![deny(missing_docs)]

use crate::analysis::ItemKey;
use crate::analysis::ItemRef;
use quote::ToTokens;
use syn::File;
use syn::Item;

/// Print trait bounds.
pub struct TraitInfo();

impl TraitInfo {
    /// Print a single item.
    pub fn show_item(it: &ItemKey) {
        print!("{}", it);
        println!();
    }

    /// Debug utility: print an `ItemRef` AST to stdout, nicely formatted.
    pub fn debug_print_itemref(item: &ItemRef) {
        match item {
            ItemRef::Func(f) => {
                Self::unparse_item(Item::Fn((**f).clone()));
            }
            ItemRef::Struct(s) => {
                Self::unparse_item(Item::Struct((**s).clone()));
            }
            ItemRef::Enum(e) => {
                Self::unparse_item(Item::Enum((**e).clone()));
            }
            ItemRef::Trait(t) => {
                Self::unparse_item(Item::Trait((**t).clone()));
            }
            ItemRef::Impl(i) => {
                Self::unparse_item(Item::Impl((**i).clone()));
            }
            ItemRef::ImplMethod { method, .. } => {
                println!("{}", method.to_token_stream());
            }
            ItemRef::TraitMethod { method, .. } => {
                println!("{}", method.to_token_stream());
            }
        }
    }

    #[inline]
    fn unparse_item(item: Item) {
        let file = File {
            shebang: None,
            attrs: vec![],
            items: vec![item],
        };
        println!("{}", prettyplease::unparse(&file));
    }
}
