// src/error.rs
//! Source targets for trait-winnower.

#![deny(missing_docs)]

use crate::error::TraitError;
use anyhow::{Context, bail};
use std::fs;
use std::path::PathBuf;

/// The classification of a target path.
#[derive(Debug)]
pub enum TargetKind {
    /// A single `.rs` file.
    SingleFile(PathBuf),
    /// A crate with a `Cargo.toml`.
    Crate(PathBuf),
    /// A workspace root with a `Cargo.toml` that contains `[workspace]`.
    Workspace(PathBuf),
}

impl TargetKind {
    /// Resolve the user-provided target (file or directory).
    pub fn get_target(raw: Option<PathBuf>) -> TraitError<TargetKind> {
        let path = raw.unwrap_or_else(|| PathBuf::from("."));
        let meta =
            fs::metadata(&path).with_context(|| format!("target not found: {}", path.display()))?;

        if meta.is_file() {
            if path.extension().and_then(|s| s.to_str()) != Some("rs") {
                bail!("single-file mode requires a .rs file: {}", path.display());
            }
            return Ok(TargetKind::SingleFile(path));
        }

        let cargo = path.join("Cargo.toml");
        if !cargo.exists() {
            bail!(
                "no Cargo.toml in {}. Provide a crate root or a single Rust file",
                path.display()
            );
        }
        let toml = fs::read_to_string(&cargo).unwrap_or_default();
        if toml.contains("[workspace]") {
            Ok(TargetKind::Workspace(path))
        } else {
            Ok(TargetKind::Crate(path))
        }
    }
}
