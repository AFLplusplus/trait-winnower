// src/config.rs
//! Configuration file for trait-winnower

#![deny(missing_docs)]

use crate::error::TraitError;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path, path::PathBuf};

/// Configuration for cargo check execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CargoCheckConfig {
    /// Cargo check arguments (e.g., ["--workspace", "--all-features", "--all-targets", "--quiet"]).
    pub args: Vec<String>,
}

impl Default for CargoCheckConfig {
    fn default() -> Self {
        Self {
            args: vec![
                "--workspace".into(),
                "--all-features".into(),
                "--all-targets".into(),
                "--quiet".into(),
            ],
        }
    }
}

/// Config struct for trait-winnower.
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// Include files.
    pub include: Vec<String>,
    /// Exclude files.
    pub exclude: Vec<String>,
    /// Cargo check configuration.
    pub cargo_check: CargoCheckConfig,
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
            cargo_check: CargoCheckConfig::default(),
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
            // If cargo_check is not specified in the config, use defaults
            if cfg.cargo_check.args.is_empty() {
                cfg.cargo_check = CargoCheckConfig::default();
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
