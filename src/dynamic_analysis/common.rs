// src/dynamic_analysis/common.rs
//! Common types and logic for dynamic analysis of trait bounds.

#![deny(missing_docs)]

use crate::analysis::{
    EnumBounds, FnBounds, ImplBounds, ImplMethodBounds, StructBounds, TraitBounds,
    TraitMethodBounds, TypeParamBounds, WhereTypeBounds,
};
use crate::config::CargoCheckConfig;
use crate::error::TraitError;

use anyhow::Context;
use quote::ToTokens;
use std::path::Path;
use std::process::{Command, ExitStatus};
use syn::GenericParam;
use syn::{Ident, Type, TypeParamBound};
use syn::{WherePredicate, punctuated::Punctuated, token::Comma};

/// A structural coordinate describing precisely and concretely the location of a trait/lifetime bound
#[derive(Clone)]
pub enum BoundSite {
    /// Bound is on a type parameter like T: Display + Debug.<br>
    /// For example, fn foo<T: Clone>() { ... }
    TypeParam {
        /// The type parameter identifier (T).
        ident: Ident,
        /// Index of the type parameter in generics (e.g. T is 0 in <T, U>).
        param_index: usize,
        /// Index of the bound for this type param (0 for first, etc.).
        bound_index: usize,
    },
    /// Bound is in a where clause predicate, like where T: Debug + Send.
    /// For example, fn foo<T>(...) where T: Clone, MyTy: Debug
    WhereClause {
        /// The type/lifetime being bounded (e.g. T or MyType<A>).
        ty: Box<Type>,
        /// Index of predicate in the where-clause predicate list.
        pred_index: usize,
        /// Index of bound within that where-clause predicate.
        bound_index: usize,
    },
}

impl core::fmt::Debug for BoundSite {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            BoundSite::TypeParam {
                ident,
                param_index,
                bound_index,
            } => f
                .debug_struct("TypeParam")
                .field("ident", &ident.to_string())
                .field("param_index", param_index)
                .field("bound_index", bound_index)
                .finish(),
            BoundSite::WhereClause {
                ty,
                pred_index,
                bound_index,
            } => f
                .debug_struct("WhereClause")
                .field("ty", &ty.to_token_stream())
                .field("pred_index", pred_index)
                .field("bound_index", bound_index)
                .finish(),
        }
    }
}

/// Represents a possible (removable) trait/lifetime bound on an item (function, impl, ...).
#[derive(Clone)]
pub struct BoundCandidate {
    /// The coordinate describing the structural and logical location (type param vs where, index, etc.).
    pub site: BoundSite,
    /// The bound atom itself (e.g., Clone, ?Sized, 'a).
    pub bound: TypeParamBound,
}

impl std::fmt::Debug for BoundCandidate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BoundCandidate")
            .field("site", &self.site)
            .field("bound", &Self::to_tokens_string(&self.bound))
            .finish()
    }
}

impl BoundCandidate {
    #[inline]
    fn to_tokens_string(bound: &TypeParamBound) -> String {
        bound.to_token_stream().to_string()
    }

    #[inline]
    fn push_type_param_candidates(out: &mut Vec<BoundCandidate>, tp: &TypeParamBounds) {
        for (bound_index, bound) in tp.bounds().iter().cloned().enumerate() {
            out.push(BoundCandidate {
                site: BoundSite::TypeParam {
                    ident: tp.ident().clone(),
                    param_index: tp.param_index(),
                    bound_index,
                },
                bound,
            });
        }
    }

    #[inline]
    fn push_where_candidates(out: &mut Vec<BoundCandidate>, wb: &WhereTypeBounds) {
        for (bound_index, bound) in wb.bounds().iter().cloned().enumerate() {
            out.push(BoundCandidate {
                site: BoundSite::WhereClause {
                    ty: Box::new(wb.bounded_ty().clone()),
                    pred_index: wb.pred_index(),
                    bound_index,
                },
                bound,
            });
        }
    }
}

/// Macro generating a collect_*_candidates() function for each analysis struct.
macro_rules! define_collect_candidate_fns {
    ( $( ($func:ident, $bounds:ident) ),+ $(,)? ) => {
        $(
            impl BoundCandidate {
                #[allow(missing_docs, reason = "macro-generated code")]
                pub fn $func(bounds: &$bounds<'_>) -> Vec<Self> {
                    let mut out = Vec::new();
                    for tp in bounds.type_param_bounds() {
                        Self::push_type_param_candidates(&mut out, tp);
                    }
                    for wb in bounds.where_bounds() {
                        Self::push_where_candidates(&mut out, wb);
                    }
                    out
                }
            }
        )+
    };
}

define_collect_candidate_fns! {
    (collect_function_candidates, FnBounds),
    (collect_trait_method_candidates, TraitMethodBounds),
    (collect_impl_method_candidates, ImplMethodBounds),
    (collect_trait_candidates, TraitBounds),
    (collect_impl_candidates, ImplBounds),
    (collect_enum_candidates, EnumBounds),
    (collect_struct_candidates, StructBounds),
}

/// A stateless utility for removing a bound from a generics block in-place.
pub struct Remove;

impl Remove {
    /// Remove a single trait/lifetime bound from a generic block by its coordinates.
    pub fn apply_to_item_with_generics<T: HasGenerics>(
        item: &mut T,
        candidate: &BoundCandidate,
    ) -> bool {
        match &candidate.site {
            BoundSite::TypeParam {
                param_index,
                bound_index,
                ..
            } => Self::remove_tp_bound_by_index(item.generics_mut(), *param_index, *bound_index),
            BoundSite::WhereClause {
                pred_index,
                bound_index,
                ..
            } => Self::remove_where_bound_by_index(item.generics_mut(), *pred_index, *bound_index),
        }
    }

    fn remove_tp_bound_by_index(
        generics: &mut syn::Generics,
        param_index: usize,
        bound_index: usize,
    ) -> bool {
        let Some(GenericParam::Type(tp)) = generics.params.iter_mut().nth(param_index) else {
            return false;
        };
        let removed = Self::remove_punctuated_at(&mut tp.bounds, bound_index);
        if removed && tp.bounds.is_empty() {
            tp.colon_token = None;
        }
        removed
    }

    fn remove_where_bound_by_index(
        generics: &mut syn::Generics,
        pred_index: usize,
        bound_index: usize,
    ) -> bool {
        let Some(wc) = generics.where_clause.as_mut() else {
            return false;
        };
        let Some(pred) = wc.predicates.iter_mut().nth(pred_index) else {
            return false;
        };
        if let syn::WherePredicate::Type(tp) = pred {
            let removed = Self::remove_punctuated_at(&mut tp.bounds, bound_index);
            if removed && tp.bounds.is_empty() {
                wc.predicates =
                    Self::drop_predicate_at(std::mem::take(&mut wc.predicates), pred_index);
                if wc.predicates.is_empty() {
                    generics.where_clause = None;
                }
            }
            removed
        } else {
            false
        }
    }

    fn remove_punctuated_at<T, P>(list: &mut Punctuated<T, P>, idx: usize) -> bool
    where
        T: Clone,
        P: Default,
    {
        if idx >= list.len() {
            return false;
        }
        let mut kept = Vec::with_capacity(list.len().saturating_sub(1));
        for (i, val) in list.iter().cloned().enumerate() {
            if i != idx {
                kept.push(val);
            }
        }
        *list = {
            let mut out = Punctuated::new();
            let mut it = kept.into_iter();
            if let Some(first) = it.next() {
                out.push_value(first);
                for v in it {
                    out.push_punct(P::default());
                    out.push_value(v);
                }
            }
            out
        };
        true
    }

    fn drop_predicate_at(
        preds: Punctuated<WherePredicate, Comma>,
        idx: usize,
    ) -> Punctuated<WherePredicate, Comma> {
        if idx >= preds.len() {
            return preds;
        }
        let mut kept = Vec::with_capacity(preds.len() - 1);
        for (i, p) in preds.into_pairs().enumerate() {
            if i != idx {
                kept.push(p.into_value());
            }
        }
        let mut out = Punctuated::new();
        let mut it = kept.into_iter();
        if let Some(first) = it.next() {
            out.push_value(first);
            for v in it {
                out.push_punct(Comma::default());
                out.push_value(v);
            }
        }
        out
    }
}
/// A result of running cargo check.
#[derive(Debug)]
pub struct CommandOutput {
    /// The status of the cargo check.
    pub status: ExitStatus,
    /// The stdout of the cargo check.
    pub stdout: String,
    /// The stderr of the cargo check.
    pub stderr: String,
}

/// A result of removing a bound.
#[derive(Debug)]
pub enum BoundRemovalOutcome {
    /// The bound was removed and cargo check was successful.
    Removed {
        /// The output of the cargo check.
        check: CommandOutput,
    },
    /// The bound was retained and cargo check was successful.  
    Retained {
        /// The output of the cargo check.
        check: CommandOutput,
    },
    /// The bound was skipped.
    Skipped,
}

/// A result of removing a bound.
#[derive(Debug)]
pub struct BoundRemovalResult {
    /// The candidate that was removed.
    pub candidate: BoundCandidate,
    /// The outcome of the removal attempt.
    pub outcome: BoundRemovalOutcome,
}

/// A utility for running cargo check.
pub struct CargoCheck;

impl CargoCheck {
    /// Run cargo check with the given configuration.
    pub fn run_cargo_check(root: &Path, config: &CargoCheckConfig) -> TraitError<CommandOutput> {
        let mut command = Command::new("cargo");
        command.arg("check");
        for arg in &config.args {
            command.arg(arg);
        }
        let output = command
            .current_dir(root)
            .output()
            .with_context(|| format!("running cargo check in {}", Self::display(root)))?;
        Ok(CommandOutput {
            status: output.status,
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        })
    }

    #[inline]
    fn display(path: &Path) -> String {
        path.to_string_lossy().into_owned()
    }
}

/// A trait for items that have generics.
pub trait HasGenerics {
    /// Get a mutable reference to the generics of the item.
    fn generics_mut(&mut self) -> &mut syn::Generics;
}

macro_rules! impl_has_generics {
    ($($t:ty => ($($access:tt)+)),* $(,)?) => {
        $(
            impl HasGenerics for $t {
                fn generics_mut(&mut self) -> &mut syn::Generics {
                    &mut self $($access)+
                }
            }
        )*
    };
}

impl_has_generics! {
    syn::ItemFn => (.sig.generics),
    syn::ItemImpl => (.generics),
    syn::ItemTrait => (.generics),
    syn::ItemStruct => (.generics),
    syn::ImplItemFn => (.sig.generics),
    syn::TraitItemFn => (.sig.generics),
    syn::ItemEnum => (.generics),
}
