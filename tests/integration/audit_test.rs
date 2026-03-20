#![allow(deprecated)]
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Helper: initialise a graxaim project inside `dir` and optionally activate a
/// profile whose .env file contains `env_content`.
fn setup_project(dir: &TempDir, profile_name: &str, env_content: &str) {
    // Write the profile file before `init` so it's picked up
    fs::write(
        dir.path().join(format!(".env.{}", profile_name)),
        env_content,
    )
    .unwrap();

    Command::cargo_bin("graxaim")
        .unwrap()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    Command::cargo_bin("graxaim")
        .unwrap()
        .current_dir(dir.path())
        .arg("use")
        .arg(profile_name)
        .assert()
        .success();
}

// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_audit_finds_vars_in_code() {
    let tmp = TempDir::new().unwrap();

    // Profile does NOT contain MY_VAR
    setup_project(&tmp, "local", "OTHER_VAR=hello\n");

    // Source file references MY_VAR
    fs::write(tmp.path().join("app.js"), "const v = process.env.MY_VAR;\n").unwrap();

    Command::cargo_bin("graxaim")
        .unwrap()
        .current_dir(tmp.path())
        .arg("audit")
        .assert()
        .success()
        .stdout(predicate::str::contains("MY_VAR"))
        .stdout(
            predicate::str::contains("MISSING from profiles")
                .or(predicate::str::contains("missing from profiles")),
        );
}

#[test]
fn test_audit_finds_dead_vars() {
    let tmp = TempDir::new().unwrap();

    // Profile contains DEAD_VAR but no source file references it
    setup_project(&tmp, "local", "DEAD_VAR=old_value\n");

    // Source file references a completely different var that IS in the profile
    // (so we avoid a "missing" hit that would muddy the output)
    // In this test we want NO source references at all.

    Command::cargo_bin("graxaim")
        .unwrap()
        .current_dir(tmp.path())
        .arg("audit")
        .assert()
        .success()
        .stdout(predicate::str::contains("DEAD_VAR"))
        .stdout(
            predicate::str::contains("NOT referenced in code")
                .or(predicate::str::contains("not referenced in code"))
                .or(predicate::str::contains("potentially dead")),
        );
}

#[test]
fn test_audit_no_issues() {
    let tmp = TempDir::new().unwrap();

    // Profile contains DATABASE_URL
    setup_project(&tmp, "local", "DATABASE_URL=postgres://localhost/db\n");

    // Source file references DATABASE_URL
    fs::write(
        tmp.path().join("server.js"),
        "const db = process.env.DATABASE_URL;\n",
    )
    .unwrap();

    Command::cargo_bin("graxaim")
        .unwrap()
        .current_dir(tmp.path())
        .arg("audit")
        .assert()
        .success()
        // Should report a clean audit
        .stdout(
            predicate::str::contains("clean")
                .or(predicate::str::contains("no issues"))
                .or(predicate::str::contains("0 missing"))
                .or(predicate::str::contains("✓")),
        );
}

#[test]
fn test_audit_skips_node_modules() {
    let tmp = TempDir::new().unwrap();

    // Profile has no vars that match INSIDE_NODE_MODULES
    setup_project(&tmp, "local", "REAL_VAR=value\n");

    // Put a JS file inside node_modules that references a var not in profile
    let nm = tmp.path().join("node_modules/some-pkg");
    fs::create_dir_all(&nm).unwrap();
    fs::write(
        nm.join("index.js"),
        "const x = process.env.INSIDE_NODE_MODULES;\n",
    )
    .unwrap();

    // Top-level source only references REAL_VAR which IS in profile
    fs::write(
        tmp.path().join("app.js"),
        "const r = process.env.REAL_VAR;\n",
    )
    .unwrap();

    let output = Command::cargo_bin("graxaim")
        .unwrap()
        .current_dir(tmp.path())
        .arg("audit")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    // INSIDE_NODE_MODULES must NOT appear — it's inside node_modules/
    assert!(
        !stdout.contains("INSIDE_NODE_MODULES"),
        "audit should skip node_modules; got:\n{}",
        stdout
    );
}

#[test]
fn test_audit_specific_profile() {
    let tmp = TempDir::new().unwrap();

    // Two profiles
    setup_project(&tmp, "local", "LOCAL_ONLY=x\n");
    fs::write(tmp.path().join(".env.staging"), "STAGING_ONLY=y\n").unwrap();

    // Source code references STAGING_ONLY
    fs::write(
        tmp.path().join("deploy.js"),
        "const s = process.env.STAGING_ONLY;\n",
    )
    .unwrap();

    // Audit only the "local" profile — STAGING_ONLY should appear as missing
    Command::cargo_bin("graxaim")
        .unwrap()
        .current_dir(tmp.path())
        .arg("audit")
        .arg("--profile")
        .arg("local")
        .assert()
        .success()
        .stdout(predicate::str::contains("STAGING_ONLY"));
}
