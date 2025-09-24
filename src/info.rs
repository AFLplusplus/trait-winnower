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
                let file = File {
                    shebang: None,
                    attrs: vec![],
                    items: vec![Item::Fn((**f).clone())],
                };
                println!("{}", prettyplease::unparse(&file));
            }
            ItemRef::Struct(s) => {
                let file = File {
                    shebang: None,
                    attrs: vec![],
                    items: vec![Item::Struct((**s).clone())],
                };
                println!("{}", prettyplease::unparse(&file));
            }
            ItemRef::Enum(e) => {
                let file = File {
                    shebang: None,
                    attrs: vec![],
                    items: vec![Item::Enum((**e).clone())],
                };
                println!("{}", prettyplease::unparse(&file));
            }
            ItemRef::Trait(t) => {
                let file = File {
                    shebang: None,
                    attrs: vec![],
                    items: vec![Item::Trait((**t).clone())],
                };
                println!("{}", prettyplease::unparse(&file));
            }
            ItemRef::Impl(i) => {
                let file = File {
                    shebang: None,
                    attrs: vec![],
                    items: vec![Item::Impl((**i).clone())],
                };
                println!("{}", prettyplease::unparse(&file));
            }
            ItemRef::ImplMethod { method, .. } => {
                println!("{}", method.to_token_stream());
            }
            ItemRef::TraitMethod { method, .. } => {
                println!("{}", method.to_token_stream());
            }
        }
    }
}
