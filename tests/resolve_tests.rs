use assert_fs::prelude::*;
use std::path::PathBuf;
use trait_winnower::target::TargetKind;

#[test]
fn resolves_single_rs_file() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = assert_fs::TempDir::new()?;
    let f = tmp.child("main.rs");
    f.write_str("// hi")?;
    let kind = TargetKind::get_target(Some(PathBuf::from(f.path())))?;
    match kind {
        TargetKind::SingleFile(p) => assert_eq!(p, PathBuf::from(f.path())),
        _ => panic!("expected SingleFile"),
    }
    tmp.close()?;
    Ok(())
}

#[test]
fn rejects_non_rs_file() {
    let tmp = assert_fs::TempDir::new().unwrap();
    let f = tmp.child("README.txt");
    f.write_str("x").unwrap();
    let err = TargetKind::get_target(Some(PathBuf::from(f.path()))).unwrap_err();
    let msg = format!("{err}");
    assert!(msg.contains(".rs file"), "got: {msg}");
}

#[test]
fn resolves_crate_root() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = assert_fs::TempDir::new()?;
    tmp.child("Cargo.toml")
        .write_str("[package]\nname=\"x\"\nversion=\"0.1.0\"\n")?;
    let kind = TargetKind::get_target(Some(PathBuf::from(tmp.path())))?;
    matches!(kind, TargetKind::Crate(_))
        .then_some(())
        .ok_or("not Crate".into())
}

#[test]
fn resolves_workspace_root() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = assert_fs::TempDir::new()?;
    tmp.child("Cargo.toml")
        .write_str("[workspace]\nmembers=[]\n")?;
    let kind = TargetKind::get_target(Some(PathBuf::from(tmp.path())))?;
    matches!(kind, TargetKind::Workspace(_))
        .then_some(())
        .ok_or("not Workspace".into())
}
