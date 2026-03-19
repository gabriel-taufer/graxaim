use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_use_exports_can_be_evaluated() {
    let temp = TempDir::new().unwrap();

    // Initialize project
    Command::cargo_bin("graxaim")
        .unwrap()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success();

    // Create a profile with specific env vars
    let env_content = "TEST_VAR=hello\nANOTHER_VAR=world\n";
    fs::write(temp.path().join(".env.test"), env_content).unwrap();

    // Get the export commands
    let output = Command::cargo_bin("graxaim")
        .unwrap()
        .current_dir(temp.path())
        .arg("use")
        .arg("test")
        .arg("--no-hooks")
        .output()
        .unwrap();

    assert!(output.status.success());

    // Stdout should contain export commands
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("export TEST_VAR="));
    assert!(stdout.contains("export ANOTHER_VAR="));

    // Verify the export commands are valid bash syntax
    // by actually executing them in a shell
    let test_script = format!(
        r#"#!/bin/bash
{}
echo "TEST_VAR=$TEST_VAR"
echo "ANOTHER_VAR=$ANOTHER_VAR"
"#,
        stdout
    );

    let script_path = temp.path().join("test_exports.sh");
    fs::write(&script_path, test_script).unwrap();

    // Make executable and run
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&script_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms).unwrap();
    }

    let result = std::process::Command::new("bash")
        .arg(&script_path)
        .output()
        .unwrap();

    let result_stdout = String::from_utf8(result.stdout).unwrap();
    assert!(
        result_stdout.contains("TEST_VAR=hello"),
        "Expected TEST_VAR=hello, got: {}",
        result_stdout
    );
    assert!(
        result_stdout.contains("ANOTHER_VAR=world"),
        "Expected ANOTHER_VAR=world, got: {}",
        result_stdout
    );
}

#[test]
fn test_use_with_special_characters_in_values() {
    let temp = TempDir::new().unwrap();

    Command::cargo_bin("graxaim")
        .unwrap()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success();

    // Create a profile with special characters
    let env_content = r#"URL=https://example.com/path?query=value&other=123
QUOTED="value with spaces"
DOLLAR_SIGN=price_$100
BACKTICK=no`command`here
"#;
    fs::write(temp.path().join(".env.special"), env_content).unwrap();

    let output = Command::cargo_bin("graxaim")
        .unwrap()
        .current_dir(temp.path())
        .arg("use")
        .arg("special")
        .arg("--no-hooks")
        .output()
        .unwrap();

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();

    // Test that the exports can be evaluated
    let test_script = format!(
        r#"#!/bin/bash
{}
echo "URL=$URL"
echo "QUOTED=$QUOTED"
echo "DOLLAR_SIGN=$DOLLAR_SIGN"
echo "BACKTICK=$BACKTICK"
"#,
        stdout
    );

    let script_path = temp.path().join("test_special.sh");
    fs::write(&script_path, test_script).unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&script_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms).unwrap();
    }

    let result = std::process::Command::new("bash")
        .arg(&script_path)
        .output()
        .unwrap();

    let result_stdout = String::from_utf8(result.stdout).unwrap();

    // Verify values are preserved correctly
    assert!(result_stdout.contains("URL=https://example.com/path?query=value&other=123"));
    assert!(
        result_stdout.contains("QUOTED=\"value with spaces\"")
            || result_stdout.contains("QUOTED=value with spaces")
    );
    assert!(
        result_stdout.contains("DOLLAR_SIGN=price_$100")
            || result_stdout.contains("DOLLAR_SIGN=price_\\$100")
    );
    assert!(
        result_stdout.contains("BACKTICK=no`command`here")
            || result_stdout.contains("BACKTICK=no\\`command\\`here")
    );
}
