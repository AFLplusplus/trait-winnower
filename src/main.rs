// src/main.rs
//! Trait Winnower CLI binary.

use anyhow::Result;
use clap::Parser;
use trait_winnower::cli;

fn main() -> Result<()> {
    let args = cli::Cli::parse();

    match args.command {
        // init: initializes project config (e.g., default path);
        cli::Commands::Init { .. } => {
            // todo!();
        }

        // prune: prunes undue/overly-strong trait bounds while preserving correctness.
        cli::Commands::Prune { target } => {
            let _target = target.unwrap_or_else(|| ".".to_string());
            // todo!();
        }

        // check: scans and warns about likely unnecessary trait bounds (no edits).
        cli::Commands::Check { target } => {
            let _target = target.unwrap_or_else(|| ".".to_string());
            // todo!();
        }
    }

    Ok(())
}
