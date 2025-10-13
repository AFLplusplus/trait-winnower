//! Integration tests for trait-winnower prune functionality.

use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;
use trait_winnower::analysis::ItemBounds;
use trait_winnower::config::Config;
use trait_winnower::discover::Discover;

/// Helper function to copy directory recursively
fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    if src.is_dir() {
        fs::create_dir_all(dst)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());

            if src_path.is_dir() {
                copy_dir_recursive(&src_path, &dst_path)?;
            } else {
                fs::copy(&src_path, &dst_path)?;
            }
        }
    }
    Ok(())
}

/// Helper function to collect all bounds from a directory
fn collect_bounds_from_dir(dir_path: &Path) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let cfg = Config::load_or_default(dir_path)?;
    let files = Discover::discover_rs_files(dir_path, &cfg.include, &cfg.exclude)?;

    let mut all_bounds = Vec::new();

    for file in files {
        let parsed_file = ItemBounds::parse_file(&file)?;
        let items = ItemBounds::collect_items_in_file(&parsed_file)?;

        // Collect all items with their bounds
        for item in items.iter_all_items() {
            all_bounds.push(item.to_string());
        }
    }

    all_bounds.sort();
    Ok(all_bounds)
}

#[test]
fn test_prune_trait_sandbox() -> Result<(), Box<dyn std::error::Error>> {
    // Setup paths
    let test_files_dir = Path::new("tests/test_files/trait_sandbox");
    let expected_dir = Path::new("tests/expected/trait_sandbox");

    assert!(test_files_dir.exists(), "Missing {:?}", test_files_dir);
    assert!(expected_dir.exists(), "Missing {:?}", expected_dir);

    // Temporary working directory
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();
    copy_dir_recursive(test_files_dir, temp_path)?;

    // Ensure the binary exists
    Command::new("cargo")
        .args(&["build", "--bin", "trait-winnower"])
        .status()
        .expect("Failed to build trait-winnower binary before running test");

    let binary_path = if cfg!(debug_assertions) {
        "target/debug/trait-winnower"
    } else {
        "target/release/trait-winnower"
    };

    // Run the prune command
    let output = Command::new(binary_path)
        .args(&["prune", "-n", "all", "-t", "all", "--brute-force"])
        .arg(temp_path)
        .output()?;

    assert!(
        output.status.success(),
        "trait-winnower prune failed\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    // Compare the collected bounds
    let pruned_bounds = collect_bounds_from_dir(temp_path)?;
    let expected_bounds = collect_bounds_from_dir(expected_dir)?;

    assert_eq!(
        pruned_bounds.len(),
        expected_bounds.len(),
        "Number of bounds differs. Pruned: {}, Expected: {}",
        pruned_bounds.len(),
        expected_bounds.len()
    );

    for (i, (pruned, expected)) in pruned_bounds.iter().zip(expected_bounds.iter()).enumerate() {
        assert_eq!(
            pruned, expected,
            "Bound at index {} differs.\nPruned:   {:?}\nExpected: {:?}",
            i, pruned, expected
        );
    }

    println!("[+] All bounds and file contents match expected results!");
    Ok(())
}