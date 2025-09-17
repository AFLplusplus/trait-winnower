// tests/cli.rs
//! Trait Winnower CLI tests.

use assert_cmd::Command;
use assert_fs::assert::PathAssert;
use assert_fs::fixture::FileWriteStr;
use assert_fs::fixture::PathChild;
use assert_fs::fixture::PathCreateDir;
use predicates::str::contains;
use trait_winnower::config::Config;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn dies_no_args() -> TestResult {
    let mut cmd = Command::cargo_bin("trait-winnower")?;
    cmd.env("CLICOLOR", "0");

    cmd.assert()
        .failure()
        .stderr(contains("Usage:"))
        .stderr(contains("[OPTIONS] <COMMAND>"))
        .stderr(contains("Commands:"));

    Ok(())
}

#[test]
fn init_writes_default_config_in_cwd() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = assert_fs::TempDir::new()?;

    Command::cargo_bin("trait-winnower")?
        .current_dir(&tmp)
        .arg("init")
        .assert()
        .success();

    let cfg_path = tmp.child(".trait-winnower.toml");
    cfg_path.assert(predicates::path::exists());

    let s = std::fs::read_to_string(cfg_path.path())?;
    let cfg: Config = toml::from_str(&s)?;
    let def = Config::default();
    assert_eq!(cfg.include, def.include);
    assert_eq!(cfg.exclude, def.exclude);

    tmp.close()?;
    Ok(())
}

#[test]
fn check_dry_run_on_crate_root_succeeds() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = assert_fs::TempDir::new()?;
    tmp.child("Cargo.toml")
        .write_str("[package]\nname=\"x\"\nversion=\"0.1.0\"\n")?;
    tmp.child("src").create_dir_all()?;
    tmp.child("src/lib.rs").write_str("// lib\n")?;

    Command::cargo_bin("trait-winnower")?
        .current_dir(&tmp)
        .args(["check", "."])
        .assert()
        .success();

    tmp.close()?;
    Ok(())
}

#[test]
fn prune_dry_run_on_crate_root_succeeds() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = assert_fs::TempDir::new()?;
    tmp.child("Cargo.toml")
        .write_str("[package]\nname=\"x\"\nversion=\"0.1.0\"\n")?;
    tmp.child("src").create_dir_all()?;
    tmp.child("src/lib.rs").write_str("// lib\n")?;

    Command::cargo_bin("trait-winnower")?
        .current_dir(&tmp)
        .args(["prune", "."])
        .assert()
        .success();

    tmp.close()?;
    Ok(())
}
