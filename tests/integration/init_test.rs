#![allow(deprecated)]
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_init_in_empty_directory() {
    let temp = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("graxaim").unwrap();
    cmd.current_dir(temp.path())
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialized graxaim"));

    // Check that .graxaim directory was created
    assert!(temp.path().join(".graxaim").exists());
    assert!(temp.path().join(".graxaim/config.toml").exists());
    assert!(temp.path().join(".gitignore").exists());
    assert!(temp.path().join(".envrc").exists());
}

#[test]
fn test_init_with_existing_profiles() {
    let temp = TempDir::new().unwrap();

    // Create some existing .env.* files
    fs::write(temp.path().join(".env.local"), "KEY=value1\n").unwrap();
    fs::write(temp.path().join(".env.staging"), "KEY=value2\n").unwrap();

    let mut cmd = Command::cargo_bin("graxaim").unwrap();
    cmd.current_dir(temp.path())
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains("Discovered 2 profile(s)"))
        .stdout(predicate::str::contains("local"))
        .stdout(predicate::str::contains("staging"));

    // Check that .env symlink was created
    assert!(temp.path().join(".env").exists());
}

#[test]
fn test_init_no_gitignore() {
    let temp = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("graxaim").unwrap();
    cmd.current_dir(temp.path())
        .arg("init")
        .arg("--no-gitignore")
        .assert()
        .success();

    assert!(!temp.path().join(".gitignore").exists());
}

#[test]
fn test_init_no_envrc() {
    let temp = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("graxaim").unwrap();
    cmd.current_dir(temp.path())
        .arg("init")
        .arg("--no-envrc")
        .assert()
        .success();

    assert!(!temp.path().join(".envrc").exists());
}

#[test]
fn test_init_already_initialized() {
    let temp = TempDir::new().unwrap();

    // Initialize once
    Command::cargo_bin("graxaim")
        .unwrap()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success();

    // Initialize again
    let mut cmd = Command::cargo_bin("graxaim").unwrap();
    cmd.current_dir(temp.path())
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains("already initialized"));
}
