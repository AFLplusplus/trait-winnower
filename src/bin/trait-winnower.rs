// src/bin/trait-winnower.rs
//! Trait Winnower CLI binary.

#![deny(missing_docs)]

use clap::Parser;
use std::path::PathBuf;

use trait_winnower::analysis::ItemBounds;
use trait_winnower::cli;
use trait_winnower::config::Config;
use trait_winnower::discover::Discover;
use trait_winnower::dynamic_analysis::edit::PruneItem;
use trait_winnower::error::TraitError;
use trait_winnower::info::TraitInfo;
use trait_winnower::target::TargetKind;

fn main() -> TraitError<()> {
    let args = cli::Cli::parse();
    let verbosity = args.verbose;
    let brute_force = args.brute_force;
    let top = match args.number_of_items.as_deref() {
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

    let target_type = args.target_type;

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
                TargetKind::SingleFile(_p) => {
                    if brute_force {
                        eprintln!("Brute force is not supported for single files");
                        std::process::exit(1);
                    }
                }
                TargetKind::Crate(root) | TargetKind::Workspace(root) => {
                    let cfg = Config::load_or_default(root)?;
                    let files = Discover::discover_rs_files(root, &cfg.include, &cfg.exclude)?;
                    if brute_force {
                        for f in files.iter().take(top) {
                            // Avoid extra allocations by borrowing path directly
                            let file = ItemBounds::parse_file(f)?;
                            let mut items = ItemBounds::collect_items_in_file(&file)?;

                            // Execute pruning based on the specified target
                            match target_type {
                                cli::TargetType::All => {
                                    PruneItem::prune_function_bounds(
                                        f,
                                        root,
                                        &mut file.clone(),
                                        items.fns_mut(),
                                        &cfg.cargo_check,
                                    )?;
                                    PruneItem::prune_impl_bounds(
                                        f,
                                        root,
                                        &mut file.clone(),
                                        items.impls_mut(),
                                        &cfg.cargo_check,
                                    )?;
                                    PruneItem::prune_trait_bounds(
                                        f,
                                        root,
                                        &mut file.clone(),
                                        items.traits_mut(),
                                        &cfg.cargo_check,
                                    )?;
                                    PruneItem::prune_trait_method_bounds(
                                        f,
                                        root,
                                        &mut file.clone(),
                                        items.trait_methods_mut(),
                                        &cfg.cargo_check,
                                    )?;
                                    PruneItem::prune_impl_method_bounds(
                                        f,
                                        root,
                                        &mut file.clone(),
                                        items.impl_methods_mut(),
                                        &cfg.cargo_check,
                                    )?;
                                    PruneItem::prune_enum_bounds(
                                        f,
                                        root,
                                        &mut file.clone(),
                                        items.enums_mut(),
                                        &cfg.cargo_check,
                                    )?;
                                    PruneItem::prune_struct_bounds(
                                        f,
                                        root,
                                        &mut file.clone(),
                                        items.structs_mut(),
                                        &cfg.cargo_check,
                                    )?;
                                }
                                cli::TargetType::Function => {
                                    PruneItem::prune_function_bounds(
                                        f,
                                        root,
                                        &mut file.clone(),
                                        items.fns_mut(),
                                        &cfg.cargo_check,
                                    )?;
                                }
                                cli::TargetType::Impl => {
                                    PruneItem::prune_impl_bounds(
                                        f,
                                        root,
                                        &mut file.clone(),
                                        items.impls_mut(),
                                        &cfg.cargo_check,
                                    )?;
                                }
                                cli::TargetType::Trait => {
                                    PruneItem::prune_trait_bounds(
                                        f,
                                        root,
                                        &mut file.clone(),
                                        items.traits_mut(),
                                        &cfg.cargo_check,
                                    )?;
                                }
                                cli::TargetType::TraitMethod => {
                                    PruneItem::prune_trait_method_bounds(
                                        f,
                                        root,
                                        &mut file.clone(),
                                        items.trait_methods_mut(),
                                        &cfg.cargo_check,
                                    )?;
                                }
                                cli::TargetType::ImplMethod => {
                                    PruneItem::prune_impl_method_bounds(
                                        f,
                                        root,
                                        &mut file.clone(),
                                        items.impl_methods_mut(),
                                        &cfg.cargo_check,
                                    )?;
                                }
                                cli::TargetType::Enum => {
                                    PruneItem::prune_enum_bounds(
                                        f,
                                        root,
                                        &mut file.clone(),
                                        items.enums_mut(),
                                        &cfg.cargo_check,
                                    )?;
                                }
                                cli::TargetType::Struct => {
                                    PruneItem::prune_struct_bounds(
                                        f,
                                        root,
                                        &mut file.clone(),
                                        items.structs_mut(),
                                        &cfg.cargo_check,
                                    )?;
                                }
                            }
                        }
                    }
                }
            }
        }
        // check: per-file items at -vv (capped by --top), global top-traits summary always.
        cli::Commands::Check { target } => {
            let kind = TargetKind::get_target(target)?;

            match &kind {
                TargetKind::SingleFile(p) => {
                    let file = ItemBounds::parse_file(p)?;
                    let items = ItemBounds::collect_items_in_file(&file)?;
                    if verbosity > 1 {
                        for item in items.fns().iter().take(top) {
                            TraitInfo::show_item(item.item_key());
                            if verbosity > 2 {
                                TraitInfo::debug_print_itemref(item.item_key().item());
                            }
                        }
                    }
                }
                TargetKind::Crate(root) | TargetKind::Workspace(root) => {
                    let cfg = Config::load_or_default(root)?;
                    let files = Discover::discover_rs_files(root, &cfg.include, &cfg.exclude)?;

                    for file in files.iter().take(top) {
                        let file = ItemBounds::parse_file(file)?;
                        let items = ItemBounds::collect_items_in_file(&file)?;
                        if verbosity > 1 {
                            for item in items.fns().iter().take(top) {
                                TraitInfo::show_item(item.item_key());
                                if verbosity > 2 {
                                    TraitInfo::debug_print_itemref(item.item_key().item());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
