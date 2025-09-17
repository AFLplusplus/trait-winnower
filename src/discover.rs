// src/error.rs
//! Source targets for trait-winnower.

#![deny(missing_docs)]

use crate::error::TraitError;
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};

/// Discover struct to keep
pub struct Discover();

impl Discover {
    /// Find the files to operate on.
    pub fn discover_rs_files(root: &Path) -> TraitError<Vec<PathBuf>> {
        let mut paths = Vec::new();
        let mut builder = WalkBuilder::new(root);

        builder
            .hidden(false)
            .ignore(true)
            .git_ignore(true)
            .git_exclude(true)
            .git_global(true)
            .follow_links(false)
            .max_depth(None);

        builder.add_ignore(".git");
        builder.add_ignore("target");
        builder.add_ignore("node_modules");
        builder.add_ignore("tests");

        for res in builder.build() {
            let dent = match res {
                Ok(d) => d,
                Err(_) => continue,
            };

            if !dent.file_type().map(|t| t.is_file()).unwrap_or(false) {
                continue;
            }

            if dent.path().extension().and_then(|s| s.to_str()) == Some("rs") {
                paths.push(dent.into_path());
            }
        }

        Ok(paths)
    }
}
