#![allow(deprecated)]
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_list_profiles() {
    let temp = TempDir::new().unwrap();

    // Create profiles
    fs::write(temp.path().join(".env.local"), "KEY=local\n").unwrap();
    fs::write(temp.path().join(".env.staging"), "KEY=staging\n").unwrap();
    fs::write(temp.path().join(".env.production"), "KEY=prod\n").unwrap();

    // Initialize
    Command::cargo_bin("graxaim")
        .unwrap()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success();

    // Activate a profile so the list output shows "(active)"
    Command::cargo_bin("graxaim")
        .unwrap()
        .current_dir(temp.path())
        .arg("use")
        .arg("local")
        .assert()
        .success();

    let mut cmd = Command::cargo_bin("graxaim").unwrap();
    cmd.current_dir(temp.path())
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("local"))
        .stdout(predicate::str::contains("staging"))
        .stdout(predicate::str::contains("production"))
        .stdout(predicate::str::contains("(active)"));
}

#[test]
fn test_list_no_profiles() {
    let temp = TempDir::new().unwrap();

    // Initialize without profiles
    Command::cargo_bin("graxaim")
        .unwrap()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success();

    let mut cmd = Command::cargo_bin("graxaim").unwrap();
    cmd.current_dir(temp.path())
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("No profiles found"));
}
