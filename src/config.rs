// src/config.rs
//! Configuration file for trait-winnower

#![deny(missing_docs)]

use crate::error::TraitError;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path, path::PathBuf};

/// Config struct for trait-winnower.
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// Include files.
    pub include: Vec<String>,
    /// Exclude files.
    pub exclude: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            include: vec!["**/*.rs".into()],
            exclude: vec![
                "target/**".into(),
                "**/.git/**".into(),
                "**/tests/**".into(),
            ],
        }
    }
}

impl Config {
    /// Write default configs to .trait-winnower.toml
    pub fn write_default_config_at(dir: &Path, force: bool) -> TraitError<PathBuf> {
        let base = if dir.is_file() {
            dir.parent().unwrap_or(dir)
        } else {
            dir
        };
        let file = base.join(".trait-winnower.toml");
        if !file.exists() || force {
            let s = toml::to_string_pretty(&Self::default())?;
            fs::write(&file, s)?;
        }
        Ok(file)
    }
}
