// src/main.rs
//! Trait Winnower CLI binary.

#![deny(missing_docs)]

use clap::Parser;
use colored::Colorize;
use std::path::PathBuf;

use trait_winnower::analysis::ItemBounds;
use trait_winnower::cli;
use trait_winnower::config::Config;
use trait_winnower::discover::Discover;
use trait_winnower::error::TraitError;
use trait_winnower::info::TraitInfo;
use trait_winnower::target::TargetKind;

fn main() -> TraitError<()> {
    let args = cli::Cli::parse();
    let verbosity = args.verbose;

    match args.command {
        // init: initializes project config (e.g., default path);
        cli::Commands::Init { path, force } => {
            let mut root: PathBuf = path.unwrap_or_else(|| PathBuf::from("."));
            if root.is_file()
                && let Some(parent) = root.parent()
            {
                root = parent.to_path_buf();
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
            let kind = TargetKind::get_target(target)?;
            match &kind {
                TargetKind::SingleFile(p) => {
                    println!("(dry-run) would modify 1 file:\n  {}", p.display())
                }
                TargetKind::Crate(root) | TargetKind::Workspace(root) => {
                    let cfg = Config::default();
                    let files = Discover::discover_rs_files(root, &cfg.include, &cfg.exclude)?;
                    println!("(dry-run) would modify {} files", files.len());
                }
            }
        }
        // check: per-file items at -vv (capped by --top), global top-traits summary always.
        cli::Commands::Check { target, top } => {
            let kind = TargetKind::get_target(target)?;
            let top = match top.as_deref() {
                Some(s)
                    if s.eq_ignore_ascii_case("all")
                        || s.eq_ignore_ascii_case("max")
                        || s.eq_ignore_ascii_case("maxx") =>
                {
                    usize::MAX
                }
                Some(s) => s.parse::<usize>().unwrap_or(10),
                None => 10,
            };

            match &kind {
                TargetKind::SingleFile(p) => {
                    let file = ItemBounds::parse_file(p)?;
                    let items = ItemBounds::collect_items_in_file(&file)?;
                    if verbosity > 1 {
                        for item in items.iter_all_items().take(top) {
                            TraitInfo::show_item(item);
                            if verbosity > 2 {
                                TraitInfo::debug_print_itemref(&item.item);
                            }
                        }
                    }
                }
                TargetKind::Crate(root) | TargetKind::Workspace(root) => {
                    let cfg = Config::load_or_default(root)?;
                    let files = Discover::discover_rs_files(root, &cfg.include, &cfg.exclude)?;

                    for f in files.iter().take(top) {
                        let file = ItemBounds::parse_file(f)?;
                        let items = ItemBounds::collect_items_in_file(&file)?;

                        if verbosity > 1 {
                            println!(
                                "// {}:",
                                f.display().to_string().italic().truecolor(0x00, 0xA6, 0x52)
                            );
                            for item in items.iter_all_items().take(top) {
                                TraitInfo::show_item(item);
                                if verbosity > 2 {
                                    TraitInfo::debug_print_itemref(&item.item);
                                }
                            }
                            println!();
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
