use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use tempfile::TempDir;

/// Shared helper to set up a test project with .graxaim/, config, profiles, and symlink.
fn setup_test_project(dir: &std::path::Path) {
    // Create .graxaim/ directory and hooks subdirectory
    fs::create_dir_all(dir.join(".graxaim/hooks")).unwrap();

    // Create config.toml with hooks enabled
    fs::write(
        dir.join(".graxaim/config.toml"),
        "[project]\nactive_profile = \"local\"\n\n[settings]\nredact_by_default = true\n\n[hooks]\nenabled = true\nshell = \"/bin/sh\"\ntimeout = 30\n",
    )
    .unwrap();

    // Create profile files
    fs::write(dir.join(".env.local"), "DB_HOST=localhost\n").unwrap();
    fs::write(dir.join(".env.staging"), "DB_HOST=staging.example.com\n").unwrap();

    // Create .env symlink pointing to .env.local
    std::os::unix::fs::symlink(".env.local", dir.join(".env")).unwrap();
}

/// Helper to write a hook script and make it executable.
fn create_hook(dir: &std::path::Path, hook_name: &str, script_content: &str) {
    let hook_path = dir.join(".graxaim/hooks").join(hook_name);
    fs::write(&hook_path, script_content).unwrap();
    let mut perms = fs::metadata(&hook_path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&hook_path, perms).unwrap();
}

#[test]
fn test_post_hook_executes_after_switch() {
    let temp = TempDir::new().unwrap();
    setup_test_project(temp.path());

    let marker = temp.path().join("post_marker");

    create_hook(
        temp.path(),
        "staging.post.sh",
        &format!("#!/bin/sh\ntouch {}\n", marker.display()),
    );

    Command::cargo_bin("graxaim")
        .unwrap()
        .current_dir(temp.path())
        .arg("use")
        .arg("staging")
        .assert()
        .success()
        .stderr(predicate::str::contains("Switched to profile 'staging'"));

    assert!(marker.exists(), "Post-switch hook marker file should exist");
}

#[test]
fn test_pre_hook_executes_before_switch() {
    let temp = TempDir::new().unwrap();
    setup_test_project(temp.path());

    let marker = temp.path().join("pre_marker");

    create_hook(
        temp.path(),
        "staging.pre.sh",
        &format!("#!/bin/sh\ntouch {}\n", marker.display()),
    );

    Command::cargo_bin("graxaim")
        .unwrap()
        .current_dir(temp.path())
        .arg("use")
        .arg("staging")
        .assert()
        .success()
        .stderr(predicate::str::contains("Switched to profile 'staging'"));

    assert!(marker.exists(), "Pre-switch hook marker file should exist");
}

#[test]
fn test_global_hooks_execute() {
    let temp = TempDir::new().unwrap();
    setup_test_project(temp.path());

    let pre_marker = temp.path().join("global_pre_marker");
    let post_marker = temp.path().join("global_post_marker");

    create_hook(
        temp.path(),
        "_pre.sh",
        &format!("#!/bin/sh\ntouch {}\n", pre_marker.display()),
    );
    create_hook(
        temp.path(),
        "_post.sh",
        &format!("#!/bin/sh\ntouch {}\n", post_marker.display()),
    );

    Command::cargo_bin("graxaim")
        .unwrap()
        .current_dir(temp.path())
        .arg("use")
        .arg("staging")
        .assert()
        .success();

    assert!(
        pre_marker.exists(),
        "Global pre-switch hook marker should exist"
    );
    assert!(
        post_marker.exists(),
        "Global post-switch hook marker should exist"
    );
}

#[test]
fn test_no_hooks_flag_skips_all_hooks() {
    let temp = TempDir::new().unwrap();
    setup_test_project(temp.path());

    let pre_marker = temp.path().join("skipped_pre_marker");
    let post_marker = temp.path().join("skipped_post_marker");
    let global_pre_marker = temp.path().join("skipped_global_pre_marker");

    create_hook(
        temp.path(),
        "staging.pre.sh",
        &format!("#!/bin/sh\ntouch {}\n", pre_marker.display()),
    );
    create_hook(
        temp.path(),
        "staging.post.sh",
        &format!("#!/bin/sh\ntouch {}\n", post_marker.display()),
    );
    create_hook(
        temp.path(),
        "_pre.sh",
        &format!("#!/bin/sh\ntouch {}\n", global_pre_marker.display()),
    );

    Command::cargo_bin("graxaim")
        .unwrap()
        .current_dir(temp.path())
        .arg("use")
        .arg("staging")
        .arg("--no-hooks")
        .assert()
        .success()
        .stderr(predicate::str::contains("Switched to profile 'staging'"));

    assert!(
        !pre_marker.exists(),
        "Pre hook marker should NOT exist when --no-hooks is used"
    );
    assert!(
        !post_marker.exists(),
        "Post hook marker should NOT exist when --no-hooks is used"
    );
    assert!(
        !global_pre_marker.exists(),
        "Global pre hook marker should NOT exist when --no-hooks is used"
    );
}

#[test]
fn test_hook_receives_env_vars() {
    let temp = TempDir::new().unwrap();
    setup_test_project(temp.path());

    let env_dump = temp.path().join("env_dump.txt");

    create_hook(
        temp.path(),
        "staging.post.sh",
        &format!(
            "#!/bin/sh\necho \"PROFILE=$GRAXAIM_PROFILE\" > {0}\necho \"ROOT=$GRAXAIM_PROJECT_ROOT\" >> {0}\necho \"PREVIOUS=$GRAXAIM_PREVIOUS_PROFILE\" >> {0}\n",
            env_dump.display()
        ),
    );

    Command::cargo_bin("graxaim")
        .unwrap()
        .current_dir(temp.path())
        .arg("use")
        .arg("staging")
        .assert()
        .success();

    assert!(env_dump.exists(), "Environment dump file should exist");

    let contents = fs::read_to_string(&env_dump).unwrap();
    assert!(
        contents.contains("PROFILE=staging"),
        "Hook should receive GRAXAIM_PROFILE=staging, got: {}",
        contents
    );
    // Use canonicalized path for comparison (macOS resolves /var -> /private/var)
    let canonical_root = temp.path().canonicalize().unwrap();
    assert!(
        contents.contains(&format!("ROOT={}", canonical_root.display())),
        "Hook should receive GRAXAIM_PROJECT_ROOT={}, got: {}",
        canonical_root.display(),
        contents
    );
    assert!(
        contents.contains("PREVIOUS=local"),
        "Hook should receive GRAXAIM_PREVIOUS_PROFILE=local, got: {}",
        contents
    );
}

#[test]
fn test_leave_hook_executes_when_switching_away() {
    let temp = TempDir::new().unwrap();
    setup_test_project(temp.path());

    let leave_marker = temp.path().join("leave_local_marker");

    // The project is already set to active_profile = "local" with .env -> .env.local
    // Create a leave hook for "local"
    create_hook(
        temp.path(),
        "_leave_local.sh",
        &format!("#!/bin/sh\ntouch {}\n", leave_marker.display()),
    );

    // Switch away from local to staging
    Command::cargo_bin("graxaim")
        .unwrap()
        .current_dir(temp.path())
        .arg("use")
        .arg("staging")
        .assert()
        .success()
        .stderr(predicate::str::contains("Switched to profile 'staging'"));

    assert!(
        leave_marker.exists(),
        "Leave hook marker should exist when switching away from local"
    );
}

#[test]
fn test_non_executable_hook_is_skipped() {
    let temp = TempDir::new().unwrap();
    setup_test_project(temp.path());

    let marker = temp.path().join("non_exec_marker");

    // Write hook script but do NOT make it executable
    let hook_path = temp.path().join(".graxaim/hooks/staging.post.sh");
    fs::write(
        &hook_path,
        format!("#!/bin/sh\ntouch {}\n", marker.display()),
    )
    .unwrap();

    // Ensure it's NOT executable (read/write only)
    let mut perms = fs::metadata(&hook_path).unwrap().permissions();
    perms.set_mode(0o644);
    fs::set_permissions(&hook_path, perms).unwrap();

    // The command should still succeed — non-executable hooks are skipped
    Command::cargo_bin("graxaim")
        .unwrap()
        .current_dir(temp.path())
        .arg("use")
        .arg("staging")
        .assert()
        .success()
        .stderr(predicate::str::contains("Switched to profile 'staging'"));

    assert!(
        !marker.exists(),
        "Non-executable hook should be skipped, marker should NOT exist"
    );
}
