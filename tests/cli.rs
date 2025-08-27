// tests/cli.rs
//! Trait Winnower CLI tests.

use assert_cmd::Command;
use predicates::str::contains;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn dies_no_args() -> TestResult {
    let mut cmd = Command::cargo_bin("trait-winnower")?;
    cmd.env("CLICOLOR", "0");

    cmd.assert()
        .failure()
        .stderr(contains("Usage: trait-winnower [OPTIONS] <COMMAND>"));
    Ok(())
}
