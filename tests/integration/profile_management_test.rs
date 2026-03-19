use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn setup_project() -> TempDir {
    let temp = TempDir::new().unwrap();

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
fn test_create_profile() {
    let temp = setup_project();

    let mut cmd = Command::cargo_bin("graxaim").unwrap();
    cmd.current_dir(temp.path())
        .arg("create")
        .arg("local")
        .assert()
        .success()
        .stdout(predicate::str::contains("Created profile 'local'"));

    assert!(temp.path().join(".env.local").exists());
}

#[test]
fn test_create_duplicate_profile() {
    let temp = setup_project();

    // Create once
    Command::cargo_bin("graxaim")
        .unwrap()
        .current_dir(temp.path())
        .arg("create")
        .arg("local")
        .assert()
        .success();

    // Try to create again
    let mut cmd = Command::cargo_bin("graxaim").unwrap();
    cmd.current_dir(temp.path())
        .arg("create")
        .arg("local")
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}

#[test]
fn test_create_with_from() {
    let temp = setup_project();

    // Create source profile
    fs::write(temp.path().join(".env.local"), "KEY1=value1\nKEY2=value2\n").unwrap();

    let mut cmd = Command::cargo_bin("graxaim").unwrap();
    cmd.current_dir(temp.path())
        .arg("create")
        .arg("staging")
        .arg("--from")
        .arg("local")
        .assert()
        .success()
        .stdout(predicate::str::contains("copied from"));

    let content = fs::read_to_string(temp.path().join(".env.staging")).unwrap();
    assert!(content.contains("KEY1=value1"));
    assert!(content.contains("KEY2=value2"));
}

#[test]
fn test_delete_profile() {
    let temp = setup_project();

    // Create profiles
    fs::write(temp.path().join(".env.local"), "KEY=local\n").unwrap();
    fs::write(temp.path().join(".env.staging"), "KEY=staging\n").unwrap();

    // Switch to local (so we can delete staging)
    Command::cargo_bin("graxaim")
        .unwrap()
        .current_dir(temp.path())
        .arg("use")
        .arg("local")
        .assert()
        .success();

    // Delete staging
    let mut cmd = Command::cargo_bin("graxaim").unwrap();
    cmd.current_dir(temp.path())
        .arg("delete")
        .arg("staging")
        .arg("--yes")
        .assert()
        .success()
        .stdout(predicate::str::contains("Deleted profile 'staging'"));

    assert!(!temp.path().join(".env.staging").exists());
}

#[test]
fn test_delete_active_profile() {
    let temp = setup_project();

    // Create and activate profile
    fs::write(temp.path().join(".env.local"), "KEY=local\n").unwrap();
    Command::cargo_bin("graxaim")
        .unwrap()
        .current_dir(temp.path())
        .arg("use")
        .arg("local")
        .assert()
        .success();

    // Try to delete active profile
    let mut cmd = Command::cargo_bin("graxaim").unwrap();
    cmd.current_dir(temp.path())
        .arg("delete")
        .arg("local")
        .arg("--yes")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Cannot delete the active profile"));
}

#[test]
fn test_rename_profile() {
    let temp = setup_project();

    // Create profile
    fs::write(temp.path().join(".env.local"), "KEY=local\n").unwrap();

    let mut cmd = Command::cargo_bin("graxaim").unwrap();
    cmd.current_dir(temp.path())
        .arg("rename")
        .arg("local")
        .arg("development")
        .assert()
        .success()
        .stdout(predicate::str::contains("Renamed"));

    assert!(!temp.path().join(".env.local").exists());
    assert!(temp.path().join(".env.development").exists());
}

#[test]
fn test_current_no_active() {
    let temp = setup_project();

    let mut cmd = Command::cargo_bin("graxaim").unwrap();
    cmd.current_dir(temp.path())
        .arg("current")
        .assert()
        .success()
        .stdout(predicate::str::contains("No active profile"));
}

#[test]
fn test_current_with_active() {
    let temp = setup_project();

    // Create and activate profile
    fs::write(temp.path().join(".env.local"), "KEY=local\n").unwrap();
    Command::cargo_bin("graxaim")
        .unwrap()
        .current_dir(temp.path())
        .arg("use")
        .arg("local")
        .assert()
        .success();

    let mut cmd = Command::cargo_bin("graxaim").unwrap();
    cmd.current_dir(temp.path())
        .arg("current")
        .assert()
        .success()
        .stdout(predicate::str::contains("local").and(predicate::str::contains("(active)").not()));
}
