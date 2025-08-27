// src/cli.rs
//! CLI argument parser for trait-winnower.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Reduce unnecessary Rust trait requirements.
#[derive(Parser, Debug)]
#[command(
    name = "trait-winnower",
    version,
    about = "Reduce unnecessary Rust trait requirements",
    disable_help_subcommand = true
)]
pub struct Cli {
    /// Increase verbosity (-v, -vv).
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,

    /// Silence all output.
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Subcommand to run.
    #[command(subcommand)]
    pub command: Commands,
}

/// Top-level subcommands supported by the CLI.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize project configuration.
    Init {
        /// Directory where configuration should live (defaults to pwd).
        #[arg(short, long)]
        path: Option<PathBuf>,

        /// Overwrite existing configuration if present.
        #[arg(long)]
        force: bool,
    },

    /// Prune undue/overly-strong trait bounds.
    Prune {
        /// Target to operate on. Defaults to ".".
        target: Option<String>,
    },

    /// Check target and report likely unnecessary trait bounds.
    Check {
        /// Target to check. Defaults to ".".
        target: Option<String>,
    },
}
