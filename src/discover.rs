// src/discover.rs
//! Discover files for analysis.

#![deny(missing_docs)]

use crate::error::TraitError;
use globset::{Glob, GlobSet, GlobSetBuilder};
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};

/// File discovery utilities.
pub struct Discover;

impl Discover {
    /// Find `.rs` files under `root`, applying `include` then subtracting `exclude` (exclude wins).
    /// Glob matching uses root-relative paths; returned file paths are absolute.
    pub fn discover_rs_files(
        root: &Path,
        include: &[String],
        exclude: &[String],
    ) -> TraitError<Vec<PathBuf>> {
        let inc = if include.is_empty() {
            vec!["**/*".into()]
        } else {
            include.to_vec()
        };
        let inc_set = Self::globset(&inc)?;
        let exc_set = Self::globset(exclude)?;

        let mut walk = WalkBuilder::new(root);
        walk.hidden(false)
            .ignore(true)
            .git_ignore(true)
            .git_exclude(true)
            .git_global(true)
            .follow_links(false);

        let mut out = Vec::new();
        for dent in walk.build() {
            let dent = match dent {
                Ok(d) => d,
                Err(_) => continue,
            };
            if !dent.file_type().map(|t| t.is_file()).unwrap_or(false) {
                continue;
            }
            if dent.path().extension().and_then(|s| s.to_str()) != Some("rs") {
                continue;
            }

            let path = dent.path();
            let rel = path.strip_prefix(root).unwrap_or(path);
            let rel_str = rel.to_string_lossy().replace('\\', "/");

            if !inc_set.is_match(&rel_str) {
                continue;
            }
            if exc_set.is_match(&rel_str) {
                continue;
            }

            out.push(path.to_path_buf());
        }
        Ok(out)
    }

    fn globset(patterns: &[String]) -> TraitError<GlobSet> {
        let mut b = GlobSetBuilder::new();
        for p in patterns {
            b.add(Glob::new(p)?);
        }
        Ok(b.build()?)
    }
}
