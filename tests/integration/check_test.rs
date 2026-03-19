use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn setup_project(env_content: &str) -> TempDir {
    let temp = TempDir::new().unwrap();

    // Create a profile
    fs::write(temp.path().join(".env.local"), env_content).unwrap();

    // Initialize graxaim
    Command::cargo_bin("graxaim")
        .unwrap()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success();

    // Set active profile
    Command::cargo_bin("graxaim")
        .unwrap()
        .current_dir(temp.path())
        .arg("use")
        .arg("local")
        .assert()
        .success();

    temp
}

fn write_schema(temp: &TempDir, schema_content: &str) {
    let schema_path = temp.path().join(".graxaim/schema.toml");
    fs::write(schema_path, schema_content).unwrap();
}

#[test]
fn test_check_valid_profile() {
    let temp = setup_project("PORT=3000\nDEBUG=true\nAPP_NAME=myapp\n");

    write_schema(
        &temp,
        r#"
[vars.PORT]
type = "port"
required = true

[vars.DEBUG]
type = "boolean"
required = true

[vars.APP_NAME]
type = "string"
required = true
"#,
    );

    Command::cargo_bin("graxaim")
        .unwrap()
        .current_dir(temp.path())
        .arg("check")
        .assert()
        .success()
        .stdout(predicate::str::contains("3 variables passed validation"))
        .stdout(predicate::str::contains("0 errors"));
}

#[test]
fn test_check_missing_required_var() {
    let temp = setup_project("PORT=3000\n");

    write_schema(
        &temp,
        r#"
[vars.PORT]
type = "port"
required = true

[vars.API_KEY]
type = "string"
required = true
"#,
    );

    Command::cargo_bin("graxaim")
        .unwrap()
        .current_dir(temp.path())
        .arg("check")
        .assert()
        .failure()
        .stdout(predicate::str::contains("MISSING"))
        .stdout(predicate::str::contains("API_KEY"));
}

#[test]
fn test_check_type_error() {
    let temp = setup_project("PORT=not_a_number\n");

    write_schema(
        &temp,
        r#"
[vars.PORT]
type = "port"
required = true
"#,
    );

    Command::cargo_bin("graxaim")
        .unwrap()
        .current_dir(temp.path())
        .arg("check")
        .assert()
        .failure()
        .stdout(predicate::str::contains("TYPE"))
        .stdout(predicate::str::contains("PORT"))
        .stdout(predicate::str::contains("not_a_number"));
}

#[test]
fn test_check_no_schema_error() {
    let temp = setup_project("PORT=3000\n");

    // Don't write a schema — it should fail
    Command::cargo_bin("graxaim")
        .unwrap()
        .current_dir(temp.path())
        .arg("check")
        .assert()
        .failure()
        .stderr(predicate::str::contains("No schema found"));
}

#[test]
fn test_schema_init_creates_schema() {
    let temp = setup_project("PORT=3000\nDATABASE_URL=postgres://localhost/mydb\nDEBUG=true\n");

    Command::cargo_bin("graxaim")
        .unwrap()
        .current_dir(temp.path())
        .arg("schema")
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains("Schema generated"));

    // Verify schema file was created
    let schema_path = temp.path().join(".graxaim/schema.toml");
    assert!(schema_path.exists());

    let content = fs::read_to_string(schema_path).unwrap();
    assert!(content.contains("[vars.PORT]"));
    assert!(content.contains("[vars.DATABASE_URL]"));
    assert!(content.contains("[vars.DEBUG]"));
}

#[test]
fn test_check_all_profiles() {
    let temp = TempDir::new().unwrap();

    // Create two profiles
    fs::write(temp.path().join(".env.local"), "PORT=3000\nDEBUG=true\n").unwrap();
    fs::write(temp.path().join(".env.staging"), "PORT=4000\nDEBUG=false\n").unwrap();

    // Initialize graxaim
    Command::cargo_bin("graxaim")
        .unwrap()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success();

    // Set active profile
    Command::cargo_bin("graxaim")
        .unwrap()
        .current_dir(temp.path())
        .arg("use")
        .arg("local")
        .assert()
        .success();

    write_schema(
        &temp,
        r#"
[vars.PORT]
type = "port"
required = true

[vars.DEBUG]
type = "boolean"
required = true
"#,
    );

    Command::cargo_bin("graxaim")
        .unwrap()
        .current_dir(temp.path())
        .arg("check")
        .arg("--all")
        .assert()
        .success()
        .stdout(predicate::str::contains("Validating local"))
        .stdout(predicate::str::contains("Validating staging"))
        .stdout(predicate::str::contains("All profiles passed"));
}

#[test]
fn test_check_enum_error() {
    let temp = setup_project("LOG_LEVEL=verbose\n");

    write_schema(
        &temp,
        r#"
[vars.LOG_LEVEL]
type = "enum"
required = true
values = ["debug", "info", "warn", "error"]
"#,
    );

    Command::cargo_bin("graxaim")
        .unwrap()
        .current_dir(temp.path())
        .arg("check")
        .assert()
        .failure()
        .stdout(predicate::str::contains("LOG_LEVEL"))
        .stdout(predicate::str::contains("verbose"));
}

#[test]
fn test_schema_generate_example() {
    let temp = setup_project("PORT=3000\n");

    write_schema(
        &temp,
        r#"
[vars.PORT]
type = "port"
required = true
default = "3000"
description = "Application port"

[vars.DEBUG]
type = "boolean"
required = false
default = "false"
"#,
    );

    Command::cargo_bin("graxaim")
        .unwrap()
        .current_dir(temp.path())
        .arg("schema")
        .arg("generate-example")
        .assert()
        .success()
        .stdout(predicate::str::contains("Generated .env.example"));

    let example_path = temp.path().join(".env.example");
    assert!(example_path.exists());

    let content = fs::read_to_string(example_path).unwrap();
    assert!(content.contains("PORT="));
    assert!(content.contains("DEBUG="));
    assert!(content.contains("Application port"));
}
