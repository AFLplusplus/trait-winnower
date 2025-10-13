// src/dynamic_analysis/edit.rs
//! Edit dynamic analysis of trait bounds.

#![deny(missing_docs)]

use crate::config::CargoCheckConfig;
use crate::dynamic_analysis::common::{
    BoundCandidate, BoundRemovalOutcome, BoundRemovalResult, CargoCheck, HasGenerics,
};
use crate::error::TraitError;
use anyhow::Context;
use proc_macro2::Span;
use std::fs;
use syn::visit_mut::VisitMut;

/// Traversal that locates the *exact* target item by its anchor Span
pub struct BoundEditor<'a, T: HasGenerics> {
    target_ident: Option<&'a syn::Ident>,
    target_anchor: Span,
    candidate: &'a BoundCandidate,
    modified: bool,
    _phantom: std::marker::PhantomData<T>,
}

impl<'a, T: HasGenerics> BoundEditor<'a, T> {
    /// Construct a new editor for the given anchor/ident/candidate.
    pub fn new(
        target_ident: Option<&'a syn::Ident>,
        target_anchor: Span,
        candidate: &'a BoundCandidate,
    ) -> Self {
        Self {
            target_ident,
            target_anchor,
            candidate,
            modified: false,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Returns true if the item was modified.
    #[inline]
    pub fn modified(&self) -> bool {
        self.modified
    }

    /// Compare two spans for equality using byte ranges when available.
    #[inline]
    fn spans_equal(&self, span1: Span, span2: Span) -> bool {
        if span1.byte_range() == span2.byte_range() {
            return true;
        }
        if span1.file() != span2.file() {
            return false;
        }
        let start1 = span1.start();
        let start2 = span2.start();
        let end1 = span1.end();
        let end2 = span2.end();
        start1.line == start2.line
            && start1.column == start2.column
            && end1.line == end2.line
            && end1.column == end2.column
    }

    #[inline]
    fn try_edit_node<N: HasGenerics>(
        &mut self,
        node: &mut N,
        node_ident: Option<&syn::Ident>,
        node_anchor: Span,
    ) {
        if self.modified {
            return;
        }
        if !self.spans_equal(node_anchor, self.target_anchor) {
            return;
        }
        if let (Some(want), Some(got)) = (self.target_ident, node_ident)
            && *want != *got
        {
            return;
        }
        self.modified = crate::dynamic_analysis::common::Remove::apply_to_item_with_generics(
            node,
            self.candidate,
        );
    }
}

impl<'a, T: HasGenerics> VisitMut for BoundEditor<'a, T> {
    fn visit_item_mod_mut(&mut self, node: &mut syn::ItemMod) {
        if self.modified {
            return;
        }
        syn::visit_mut::visit_item_mod_mut(self, node);
    }

    fn visit_item_fn_mut(&mut self, node: &mut syn::ItemFn) {
        let id = node.sig.ident.clone();
        let anchor = id.span();
        self.try_edit_node(node, Some(&id), anchor);
    }

    fn visit_item_impl_mut(&mut self, node: &mut syn::ItemImpl) {
        let anchor = node.impl_token.span;
        self.try_edit_node(node, None, anchor);
        if !self.modified {
            syn::visit_mut::visit_item_impl_mut(self, node);
        }
    }

    fn visit_item_trait_mut(&mut self, node: &mut syn::ItemTrait) {
        let id = node.ident.clone();
        let anchor = id.span();
        self.try_edit_node(node, Some(&id), anchor);
        if !self.modified {
            syn::visit_mut::visit_item_trait_mut(self, node);
        }
    }

    fn visit_item_struct_mut(&mut self, node: &mut syn::ItemStruct) {
        let id = node.ident.clone();
        let anchor = id.span();
        self.try_edit_node(node, Some(&id), anchor);
    }

    fn visit_item_enum_mut(&mut self, node: &mut syn::ItemEnum) {
        let id = node.ident.clone();
        let anchor = id.span();
        self.try_edit_node(node, Some(&id), anchor);
    }

    fn visit_impl_item_fn_mut(&mut self, node: &mut syn::ImplItemFn) {
        let id = node.sig.ident.clone();
        let anchor = id.span();
        self.try_edit_node(node, Some(&id), anchor);
    }

    fn visit_trait_item_fn_mut(&mut self, node: &mut syn::TraitItemFn) {
        let id = node.sig.ident.clone();
        let anchor = id.span();
        self.try_edit_node(node, Some(&id), anchor);
    }
}

#[inline]
fn hash_bytes(s: &str) -> u32 {
    crc32fast::hash(s.as_bytes())
}
struct CandidateTrialConfig<'a> {
    file_path: &'a std::path::Path,
    crate_root: &'a std::path::Path,
    working: &'a syn::File,
    target_ident: Option<&'a syn::Ident>,
    target_anchor: Span,
    candidate: &'a BoundCandidate,
    current_src: &'a str,
    current_hash: u32,
    cargo_check_config: &'a CargoCheckConfig,
}
impl<'a> CandidateTrialConfig<'a> {
    fn try_candidate_once<T: HasGenerics>(
        config: CandidateTrialConfig<'_>,
    ) -> TraitError<(bool, BoundRemovalOutcome, String, u32)> {
        let mut try_working = config.working.clone();
        let mut editor =
            BoundEditor::<T>::new(config.target_ident, config.target_anchor, config.candidate);
        editor.visit_file_mut(&mut try_working);
        if !editor.modified() {
            return Ok((
                false,
                BoundRemovalOutcome::Skipped,
                config.current_src.to_owned(),
                config.current_hash,
            ));
        }

        let updated_src = prettyplease::unparse(&try_working);
        let updated_hash = hash_bytes(&updated_src);

        if updated_hash == config.current_hash {
            return Ok((
                false,
                BoundRemovalOutcome::Skipped,
                config.current_src.to_owned(),
                config.current_hash,
            ));
        }

        fs::write(config.file_path, &updated_src)
            .with_context(|| format!("writing updated {}", config.file_path.display()))?;
        let check = CargoCheck::run_cargo_check(config.crate_root, config.cargo_check_config)?;

        if check.status.success() {
            Ok((
                true,
                BoundRemovalOutcome::Removed { check },
                updated_src,
                updated_hash,
            ))
        } else {
            fs::write(config.file_path, config.current_src)
                .with_context(|| format!("reverting {}", config.file_path.display()))?;
            Ok((
                false,
                BoundRemovalOutcome::Retained { check },
                config.current_src.to_owned(),
                config.current_hash,
            ))
        }
    }
}
/// A trait for items that can be pruned.
pub struct PruneItem;

macro_rules! make_pruner {
    ( $( name: $name:ident, item_ty: $item_ty:ty, bounds_ty: $bounds_ty:ty, collect_candidates: $collect:expr $(,)? );+ $(;)? ) => {
        $(
            impl PruneItem {
                #[allow(missing_docs, reason = "macro-generated")]
                pub fn $name(
                    file_path: &std::path::Path,
                    crate_root: &std::path::Path,
                    syntax: &mut syn::File,
                    bounds: &mut Vec<$bounds_ty>,
                    cargo_check_config: &CargoCheckConfig,
                ) -> crate::error::TraitError<Vec<BoundRemovalResult>> {
                    let original_src = fs::read_to_string(file_path)
                        .with_context(|| format!("reading {}", file_path.display()))?;
                    let original_hash = hash_bytes(&original_src);
                    let mut outcomes = Vec::new();
                    let mut working = syntax.clone();
                    let mut current_src = original_src.clone();
                    let mut current_hash = original_hash;
                    let i = 0;

                    while i < bounds.len() {
                        let bounds_item = &bounds[i];
                        let item_key = bounds_item.item_key();
                        let target_ident = item_key.ident();
                        let target_anchor = item_key.span();

                        let candidates: Vec<BoundCandidate> = ($collect)(bounds_item);
                        let mut removed_any = false;

                        for candidate in &candidates {
                            let config = CandidateTrialConfig {
                                file_path,
                                crate_root,
                                working: &working,
                                target_ident,
                                target_anchor,
                                candidate,
                                current_src: &current_src,
                                current_hash,
                                cargo_check_config,
                            };
                            let (accepted, outcome, new_src, new_hash) = CandidateTrialConfig::try_candidate_once::<$item_ty>(config)?;
                            outcomes.push(BoundRemovalResult { candidate: candidate.clone(), outcome });

                            if accepted {
                                let mut tmp = working.clone();
                                let mut editor =
                                    BoundEditor::<$item_ty>::new(target_ident, target_anchor, candidate);
                                editor.visit_file_mut(&mut tmp);
                                debug_assert!(editor.modified());
                                working = tmp;
                                *syntax = working.clone();
                                current_src = new_src;
                                current_hash = new_hash;
                                removed_any = true;
                                break;
                            }
                        }

                        if removed_any {
                            continue;
                        } else {
                            bounds.remove(i);
                        }
                    }

                    Ok(outcomes)
                }
            }
        )+
    };
}

make_pruner! {
    name: prune_function_bounds,  item_ty: syn::ItemFn,  bounds_ty: crate::analysis::FnBounds<'_>,
    collect_candidates: |b: &crate::analysis::FnBounds<'_>| { BoundCandidate::collect_function_candidates(b) };

    name: prune_struct_bounds, item_ty: syn::ItemStruct, bounds_ty: crate::analysis::StructBounds<'_>,
    collect_candidates: |b: &crate::analysis::StructBounds<'_>| { BoundCandidate::collect_struct_candidates(b) };

    name: prune_enum_bounds, item_ty: syn::ItemEnum, bounds_ty: crate::analysis::EnumBounds<'_>,
    collect_candidates: |b: &crate::analysis::EnumBounds<'_>| { BoundCandidate::collect_enum_candidates(b)};

    name: prune_impl_bounds, item_ty: syn::ItemImpl, bounds_ty: crate::analysis::ImplBounds<'_>,
    collect_candidates: |b: &crate::analysis::ImplBounds<'_>| { BoundCandidate::collect_impl_candidates(b) };

    name: prune_trait_bounds, item_ty: syn::ItemTrait, bounds_ty: crate::analysis::TraitBounds<'_>,
    collect_candidates: |b: &crate::analysis::TraitBounds<'_>| { BoundCandidate::collect_trait_candidates(b) };

    name: prune_trait_method_bounds, item_ty: syn::TraitItemFn, bounds_ty: crate::analysis::TraitMethodBounds<'_>,
    collect_candidates: |b: &crate::analysis::TraitMethodBounds<'_>| { BoundCandidate::collect_trait_method_candidates(b) };

    name: prune_impl_method_bounds, item_ty: syn::ImplItemFn, bounds_ty: crate::analysis::ImplMethodBounds<'_>,
    collect_candidates: |b: &crate::analysis::ImplMethodBounds<'_>| { BoundCandidate::collect_impl_method_candidates(b) };
}
