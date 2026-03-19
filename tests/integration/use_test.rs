use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn setup_project() -> TempDir {
    let temp = TempDir::new().unwrap();

    // Create profiles
    fs::write(temp.path().join(".env.local"), "KEY=local\n").unwrap();
    fs::write(temp.path().join(".env.staging"), "KEY=staging\n").unwrap();

    // Initialize
    Command::cargo_bin("graxaim")
        .unwrap()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success();

    temp
}

#[test]
fn test_use_profile() {
    let temp = setup_project();

    let mut cmd = Command::cargo_bin("graxaim").unwrap();
    cmd.current_dir(temp.path())
        .arg("use")
        .arg("staging")
        .assert()
        .success()
        .stderr(predicate::str::contains("Switched to profile 'staging'"))
        .stdout(predicate::str::contains("export KEY="));

    // Check symlink points to correct file
    let symlink = temp.path().join(".env");
    assert!(symlink.exists());
    let target = fs::read_link(&symlink).unwrap();
    assert_eq!(target.to_str().unwrap(), ".env.staging");
}

#[test]
fn test_use_nonexistent_profile() {
    let temp = setup_project();

    let mut cmd = Command::cargo_bin("graxaim").unwrap();
    cmd.current_dir(temp.path())
        .arg("use")
        .arg("nonexistent")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_use_outside_project() {
    let temp = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("graxaim").unwrap();
    cmd.current_dir(temp.path())
        .arg("use")
        .arg("local")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("not initialized")
                .or(predicate::str::contains("not found"))
                .or(predicate::str::contains("Cannot find project root")),
        );
}
