//! CLI argument parser for trait-winnower.

#![deny(missing_docs)]

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
    /// Set verbosity level: -v=1, -v=2, -v=3
    #[arg(
        short = 'v',
        long = "verbose",
        value_name = "LEVEL",
        default_value_t = 0,
        value_parser = clap::value_parser!(u8).range(0..=3),
        global = true
    )]
    pub verbose: u8,

    /// Silence all output (overrides -v).
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
        target: Option<PathBuf>,
    },

    /// Check target and report likely unnecessary trait bounds.
    Check {
        /// Target to check. Defaults to ".".
        target: Option<PathBuf>,

        /// Show only the top N trait bounds.
        #[arg(short, long)]
        top: Option<String>,
    },
}
