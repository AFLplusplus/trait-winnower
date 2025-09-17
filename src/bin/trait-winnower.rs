// src/main.rs
//! Trait Winnower CLI binary.

#![deny(missing_docs)]

use clap::Parser;
use std::path::PathBuf;

use trait_winnower::cli;
use trait_winnower::config::Config;
use trait_winnower::error::TraitError;
use trait_winnower::target::TargetKind;

fn main() -> TraitError<()> {
    let args = cli::Cli::parse();

    match args.command {
        // init: initializes project config (e.g., default path);
        cli::Commands::Init { path, force } => {
            // Default to current directory if not provided.
            let mut root: PathBuf = path.unwrap_or_else(|| PathBuf::from("."));

            if root.is_file() {
                if let Some(parent) = root.parent() {
                    root = parent.to_path_buf();
                }
            }

            let path_written = Config::write_default_config_at(root.as_path(), force)?;
            println!(
                "{} .trait-winnower.toml at {}",
                if force { "Overwrote" } else { "Initialized" },
                path_written.display()
            );
        }

        // prune: prunes undue/overly-strong trait bounds while preserving correctness.
        cli::Commands::Prune { target } => {
            let _kind = TargetKind::get_target(target)?;
            // todo!();
        }

        // check: scans and warns about likely unnecessary trait bounds (no edits).
        cli::Commands::Check { target } => {
            let _kind = TargetKind::get_target(target)?;
            // todo!();
        }
    }

    Ok(())
}
