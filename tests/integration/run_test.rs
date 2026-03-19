use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn setup_project() -> TempDir {
    let temp = TempDir::new().unwrap();

    // Create profile with env vars
    fs::write(
        temp.path().join(".env.local"),
        "DATABASE_URL=postgres://localhost/test\nAPI_KEY=secret123\n",
    )
    .unwrap();

    // Initialize and activate
    Command::cargo_bin("graxaim")
        .unwrap()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success();

    Command::cargo_bin("graxaim")
        .unwrap()
        .current_dir(temp.path())
        .arg("use")
        .arg("local")
        .assert()
        .success();

    temp
}

#[test]
fn test_run_command() {
    let temp = setup_project();

    let mut cmd = Command::cargo_bin("graxaim").unwrap();
    cmd.current_dir(temp.path())
        .arg("run")
        .arg("--")
        .arg("sh")
        .arg("-c")
        .arg("echo $DATABASE_URL")
        .assert()
        .success()
        .stdout(predicate::str::contains("postgres://localhost/test"));
}

#[test]
fn test_run_no_command() {
    let temp = setup_project();

    let mut cmd = Command::cargo_bin("graxaim").unwrap();
    cmd.current_dir(temp.path())
        .arg("run")
        .assert()
        .failure()
        .stderr(predicate::str::contains("No command specified"));
}

#[test]
fn test_export_bash() {
    let temp = setup_project();

    let mut cmd = Command::cargo_bin("graxaim").unwrap();
    cmd.current_dir(temp.path())
        .arg("export")
        .arg("--shell")
        .arg("bash")
        .assert()
        .success()
        .stdout(predicate::str::contains("export DATABASE_URL="))
        .stdout(predicate::str::contains("export API_KEY="));
}

#[test]
fn test_export_fish() {
    let temp = setup_project();

    let mut cmd = Command::cargo_bin("graxaim").unwrap();
    cmd.current_dir(temp.path())
        .arg("export")
        .arg("--shell")
        .arg("fish")
        .assert()
        .success()
        .stdout(predicate::str::contains("set -x DATABASE_URL"))
        .stdout(predicate::str::contains("set -x API_KEY"));
}
