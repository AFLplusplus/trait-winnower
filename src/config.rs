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
    /// Load `.trait-winnower.toml` from `dir` (or its parent if `dir` is a file).
    /// If missing, return defaults. Ensures `include/exclude` are never empty.
    pub fn load_or_default(dir: &Path) -> TraitError<Self> {
        let base = if dir.is_file() {
            dir.parent().unwrap_or(dir)
        } else {
            dir
        };
        let file = base.join(".trait-winnower.toml");
        if file.exists() {
            let s = fs::read_to_string(&file)?;
            let mut cfg: Config = toml::from_str(&s)?;
            if cfg.include.is_empty() {
                cfg.include = Config::default().include;
            }
            if cfg.exclude.is_empty() {
                cfg.exclude = Config::default().exclude;
            }
            Ok(cfg)
        } else {
            Ok(Config::default())
        }
    }
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
