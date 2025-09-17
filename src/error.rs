// src/error.rs
//! Error handling for trait-winnower.

#![deny(missing_docs)]

/// TraitError is alias for anyhow
pub type TraitError<T> = anyhow::Result<T>;
