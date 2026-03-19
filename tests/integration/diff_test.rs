use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn setup_project_with_profiles() -> TempDir {
    let temp = TempDir::new().unwrap();

    // Create profiles with different content
    fs::write(
        temp.path().join(".env.local"),
        "PORT=3000\nDATABASE_URL=postgres://localhost/mydb\nDEBUG=true\nLOCAL_ONLY=localvalue\n",
    )
    .unwrap();
    fs::write(
        temp.path().join(".env.staging"),
        "PORT=3000\nDATABASE_URL=postgres://staging.db.com/mydb\nDEBUG=false\nSTAGING_ONLY=stagingvalue\n",
    )
    .unwrap();

    // Initialize graxaim
    Command::cargo_bin("graxaim")
        .unwrap()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success();

    // Set active profile to local
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
fn test_diff_two_profiles() {
    let temp = setup_project_with_profiles();

    let mut cmd = Command::cargo_bin("graxaim").unwrap();
    cmd.current_dir(temp.path())
        .arg("diff")
        .arg("local")
        .arg("staging")
        .assert()
        .success()
        .stdout(predicate::str::contains("Comparing"))
        .stdout(predicate::str::contains("local"))
        .stdout(predicate::str::contains("staging"))
        .stdout(predicate::str::contains("LOCAL_ONLY"))
        .stdout(predicate::str::contains("STAGING_ONLY"))
        .stdout(predicate::str::contains("DATABASE_URL"))
        .stdout(predicate::str::contains("DEBUG"));
}

#[test]
fn test_diff_with_no_redact() {
    let temp = setup_project_with_profiles();

    let mut cmd = Command::cargo_bin("graxaim").unwrap();
    cmd.current_dir(temp.path())
        .arg("diff")
        .arg("local")
        .arg("staging")
        .arg("--no-redact")
        .assert()
        .success()
        .stdout(predicate::str::contains("postgres://localhost/mydb"))
        .stdout(predicate::str::contains("postgres://staging.db.com/mydb"))
        .stdout(predicate::str::contains("localvalue"))
        .stdout(predicate::str::contains("stagingvalue"));
}

#[test]
fn test_diff_against_active_profile() {
    let temp = setup_project_with_profiles();

    // Diff staging against active (local)
    let mut cmd = Command::cargo_bin("graxaim").unwrap();
    cmd.current_dir(temp.path())
        .arg("diff")
        .arg("staging")
        .assert()
        .success()
        .stdout(predicate::str::contains("Comparing"))
        .stdout(predicate::str::contains("staging"))
        .stdout(predicate::str::contains("local"))
        .stdout(predicate::str::contains("DATABASE_URL"));
}

#[test]
fn test_diff_identical_profiles() {
    let temp = setup_project_with_profiles();

    let mut cmd = Command::cargo_bin("graxaim").unwrap();
    cmd.current_dir(temp.path())
        .arg("diff")
        .arg("local")
        .arg("local")
        .assert()
        .success()
        .stdout(predicate::str::contains("identical").or(predicate::str::contains("Identical")));
}

#[test]
fn test_diff_nonexistent_profile() {
    let temp = setup_project_with_profiles();

    let mut cmd = Command::cargo_bin("graxaim").unwrap();
    cmd.current_dir(temp.path())
        .arg("diff")
        .arg("local")
        .arg("nonexistent")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}
