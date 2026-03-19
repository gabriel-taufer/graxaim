---
name: tdd-graxaim
description: >
  TDD workflow for graxaim feature development. Red-Green-Refactor with
  .env-specific patterns. Auto-triggers on new command implementation.
triggers:
  - "new command"
  - "implement feature"
  - "add command"
  - "write tests for"
---

# graxaim TDD Workflow

## The Loop

1. **RED** — Write failing test with realistic .env fixture
2. **GREEN** — Implement minimum code to pass
3. **REFACTOR** — Clean up, verify still passing
4. **EDGE** — Add empty/malformed/unicode edge case tests
5. **GATE** — `cargo fmt && cargo clippy && cargo test`

Repeat for each function or behavior.

## Step 1: Create Fixture

Use realistic .env content, not synthetic placeholder data:

```
# tests/fixtures/sample.env
DATABASE_URL=postgres://localhost:5432/mydb
API_KEY=sk_test_12345678901234567890
PORT=3000
DEBUG=true
EMPTY_VALUE=
QUOTED_VALUE="hello world"
# This is a comment
MULTIEQUAL=base64=abc=def==
```

Or use inline constants for self-contained tests:

```rust
const FIXTURE: &str = "\
DATABASE_URL=postgres://localhost:5432/mydb
API_KEY=sk_test_12345678901234567890
PORT=3000
";
```

## Step 2: Write Failing Test (Red)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_feature_happy_path() {
        // Arrange
        let env_content = include_str!("../../tests/fixtures/sample.env");

        // Act
        let result = new_feature(env_content);

        // Assert
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.len(), 7); // 7 variables in fixture
    }

    #[test]
    fn test_new_feature_empty_input() {
        let result = new_feature("");
        assert!(result.is_ok()); // Must not panic on empty
    }

    #[test]
    fn test_new_feature_malformed_input() {
        let result = new_feature("\x00binary\xffdata");
        // Either Ok with best-effort or specific error — never panic
        assert!(result.is_ok() || result.is_err());
    }
}
```

Follow **Arrange-Act-Assert** pattern in every test.

## Step 3: Implement (Green)

Write the minimum code to make the tests pass. Don't optimize yet.

```rust
pub fn new_feature(content: &str) -> Result<Vec<EnvVar>> {
    // Minimum viable implementation
    let mut vars = Vec::new();
    for line in content.lines() {
        if let Some((key, value)) = line.split_once('=') {
            vars.push(EnvVar { key: key.to_string(), value: value.to_string() });
        }
    }
    Ok(vars)
}
```

## Step 4: Refactor

Clean up the implementation while keeping tests green:
- Extract helpers
- Add proper error handling with `.context()`
- Replace `unwrap()` with `?`
- Ensure module structure follows: imports → `execute()` → helpers → tests

## Step 5: Edge Case Tests

Add tests for every edge case relevant to .env handling:

```rust
#[test]
fn test_new_feature_windows_line_endings() {
    let content = "KEY1=value1\r\nKEY2=value2\r\n";
    let result = new_feature(content);
    assert!(result.is_ok());
}

#[test]
fn test_new_feature_unicode_values() {
    let content = "GREETING=こんにちは\nEMOJI=🦡\n";
    let result = new_feature(content);
    assert!(result.is_ok());
}

#[test]
fn test_new_feature_value_with_equals() {
    let content = "BASE64=abc=def==\n";
    let result = new_feature(content);
    let vars = result.unwrap();
    assert_eq!(vars[0].value, "abc=def==");
}
```

## Step 6: Quality Gate

```bash
cargo fmt --all && cargo clippy --all-targets -- -D warnings && cargo test
```

All three must pass before moving on.

## What "Done" Looks Like

- [ ] Happy path test passing
- [ ] Empty input test passing (no panic)
- [ ] Malformed input test passing (no panic)
- [ ] Edge case tests (unicode, Windows line endings, quoted values)
- [ ] Integration test with `assert_cmd` if adding a CLI command
- [ ] `cargo fmt` + `cargo clippy` + `cargo test` = all green
- [ ] No `unwrap()` in production code (only in tests with `expect("reason")`)
