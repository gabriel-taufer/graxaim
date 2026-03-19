# graxaim Testing Strategy

## Test Pyramid

1. **Unit tests** (highest count) — In `#[cfg(test)] mod tests` of the same file
2. **Integration tests** (medium count) — In `tests/integration/`, use `assert_cmd` + `tempfile`
3. **Manual verification** (lowest count) — Only for interactive picker (`src/ui/picker.rs`) and editor integration (`src/commands/edit.rs`)

## Unit Test Requirements

Every function in `src/core/` MUST have tests for:
- **Happy path** — normal expected usage
- **Empty input** — empty string, empty file, no profiles
- **Malformed input** — corrupted config, invalid TOML, binary data
- **Edge cases** — special characters, very long values, unicode, Windows line endings
- **Error paths** — file not found, permission denied, invalid profile name

## Integration Test Pattern

```rust
use assert_cmd::Command;
use tempfile::TempDir;

fn setup_project(dir: &TempDir) {
    // Create .graxaim/ directory
    std::fs::create_dir_all(dir.path().join(".graxaim")).unwrap();
    // Create config.toml
    std::fs::write(
        dir.path().join(".graxaim/config.toml"),
        "[project]\nname = \"test\"\n",
    ).unwrap();
    // Create .env.* profile files
    std::fs::write(
        dir.path().join(".env.development"),
        "DATABASE_URL=postgres://localhost:5432/dev\n",
    ).unwrap();
}

#[test]
fn test_list_shows_profiles() {
    let dir = TempDir::new().unwrap();
    setup_project(&dir);

    Command::cargo_bin("graxaim")
        .unwrap()
        .current_dir(dir.path())
        .arg("list")
        .assert()
        .success()
        .stdout(predicates::str::contains("development"));
}
```

## Test Naming Convention

```
test_{function}_{scenario}
test_{function}_{input_type}
test_{command}_{expected_behavior}
```

Examples:
- `test_parse_env_file_empty_input`
- `test_parse_env_file_windows_line_endings`
- `test_parse_env_file_quoted_values`
- `test_init_discovers_existing_profiles`
- `test_use_switches_symlink`
- `test_create_rejects_invalid_name`
- `test_delete_refuses_active_profile`
- `test_export_bash_format`

## Quality Gate

Before every commit:

```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test --all
```

All three must pass. Zero clippy warnings. No skipped tests.

## Test Fixtures

Place reusable test fixtures in `tests/fixtures/`:
- `sample.env` — realistic .env with various value types
- `windows.env` — file with `\r\n` line endings
- `quoted.env` — file with single/double quoted values
- `edge-cases.env` — empty values, BOM, no trailing newline

For inline test data, use `const` strings or `include_str!()`:

```rust
const SAMPLE_ENV: &str = "\
DATABASE_URL=postgres://localhost:5432/mydb
API_KEY=sk_test_12345678901234567890
PORT=3000
EMPTY_VALUE=
QUOTED_VALUE=\"hello world\"
# This is a comment
";
```
