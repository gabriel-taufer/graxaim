# graxaim `.claude/` Directory Plan

**Source:** Adapted from `~/Documents/personal/rtk/.claude/`  
**Purpose:** Create project-specific rules, skills, and commands for AI-assisted development of graxaim

---

## Directory Structure

```
.claude/
├── rules/
│   ├── graxaim-patterns.md       # Rust coding patterns specific to graxaim
│   ├── env-file-safety.md        # .env file handling security rules
│   └── testing-strategy.md       # Testing conventions and requirements
├── skills/
│   ├── tdd-graxaim/
│   │   └── SKILL.md              # TDD workflow for graxaim features
│   ├── design-patterns/
│   │   └── SKILL.md              # Design patterns for graxaim modules
│   └── ship/
│       └── SKILL.md              # Release/ship workflow
└── commands/
    ├── verify-build.md           # Build verification command
    └── add-command.md            # How to add a new graxaim subcommand
```

---

## Rules

### 1. `rules/graxaim-patterns.md`

Adapted from rtk's `rules/rust-patterns.md`. graxaim-specific constraints:

**Content outline:**

```markdown
# graxaim Rust Patterns

## Non-Negotiable Rules

1. **No `unwrap()` in production** — Use `.context("description")?` or `.with_context(|| format!(...))`. 
   Tests: use `expect("reason")`.

2. **Lazy regex** — `Regex::new()` inside a function recompiles every call. 
   Always use `lazy_static!` or `std::sync::LazyLock` (Rust 1.80+).

3. **No async** — graxaim is single-threaded, synchronous. No tokio, no async-std.

4. **Error context on every `?`** — Every `?` must have `.context()` or `.with_context()`.

5. **`thiserror` for library errors, `anyhow` only in `main.rs`** — Typed errors in 
   `errors.rs`, anyhow only at the entry point.

6. **`&str` over `&String`** — Function signatures should accept `&str`, not `&String`.

## Profile Name Validation

Always use the `ProfileName` newtype for user-provided profile names:
- Validates `[a-zA-Z0-9_-]` at construction time
- Prevents shell injection in hook scripts
- Compile-time safety against mixing profile names with file paths

## .env File Operations

- Parse with our custom parser (`core/env_file.rs`), never external crates
- Preserve formatting for round-trip operations
- Handle: Windows line endings, quoted values, escape sequences, comments
- Always read files with explicit UTF-8 handling

## Symlink Safety

- Always check if symlink target exists before operating
- Handle broken symlinks gracefully (dangling `.env` -> deleted profile)
- Use `std::os::unix::fs::symlink` with proper error context
- Never follow symlinks when listing profiles (avoid infinite loops)

## Hook Execution Safety

- Execute hooks via `Command::new(shell).arg(script_path)` — never shell interpolation
- Verify script is executable before running
- Always set timeout (default 30s)
- Inject env vars via `Command::env()`, not shell variable expansion
- Kill child process on timeout, don't just wait

## Module Structure

Every `commands/*.rs` follows this pattern:
1. Imports
2. Public `execute()` function
3. Private helper functions
4. `#[cfg(test)] mod tests` (unit tests in same file)

## Anti-Patterns

| Pattern | Problem | Fix |
|---------|---------|-----|
| `Regex::new()` in function body | Recompiles every call | `lazy_static!` |
| `unwrap()` in production code | Panic crashes CLI | `.context()?` |
| `&String` in function signature | Unnecessary restriction | `&str` |
| Silent `match Err(_) => {}` | Swallows errors | `eprintln!` + handle |
| `clone()` of large strings | Extra allocation | Borrow with `&str` |
| Nested match > 2 levels | Hard to read | Early returns |
```

### 2. `rules/env-file-safety.md`

graxaim-specific security rules for handling environment files:

**Content outline:**

```markdown
# .env File Safety Rules

## Never Do

1. **Never print values to stdout by default** — Always redact unless `--no-redact` is explicit
2. **Never log .env values** — Not in debug output, not in error messages
3. **Never include values in error context strings** — Say "failed to parse KEY" not "failed to parse KEY=secret123"
4. **Never write .env values to temp files** — Operations should be in-memory
5. **Never store passphrases in plain text** — Use `secrecy::SecretString`

## Always Do

1. **Redact by default** — `sk_test_...5678` format for display
2. **Check file permissions** — Warn if .env files are world-readable
3. **Validate profile names** — `[a-zA-Z0-9_-]` only, via `ProfileName` newtype
4. **Add to .gitignore** — On `init`, add `.env` and `.env.*` (except `.env.*.sealed`)
5. **Preserve file format** — Round-trip must not alter comments, blank lines, quoting style

## Edge Cases

- Empty values: `KEY=` vs `KEY=""` vs missing KEY — these are all different
- Values with `=`: `BASE64=abc=def==` — only split on first `=`
- Values with quotes: preserve original quoting style
- Windows line endings: handle `\r\n` transparently
- No trailing newline: don't add one if not present
- BOM markers: handle UTF-8 BOM gracefully
```

### 3. `rules/testing-strategy.md`

**Content outline:**

```markdown
# graxaim Testing Strategy

## Test Pyramid

1. **Unit tests** (highest count) — In `#[cfg(test)] mod tests` of the same file
2. **Integration tests** (medium count) — In `tests/integration/`, use `assert_cmd` + `tempfile`
3. **Manual verification** (lowest count) — Only for interactive picker and editor integration

## Unit Test Requirements

Every function in `core/` MUST have tests for:
- [ ] Happy path — normal expected usage
- [ ] Empty input — empty string, empty file, no profiles
- [ ] Malformed input — corrupted config, invalid TOML, binary data
- [ ] Edge cases — special characters, very long values, unicode
- [ ] Error paths — file not found, permission denied, invalid profile name

## Integration Test Pattern

```rust
use assert_cmd::Command;
use tempfile::TempDir;

fn setup_project(dir: &TempDir) {
    // Create .graxaim/ directory
    // Create config.toml
    // Create .env.* profile files
}

#[test]
fn test_command_name() {
    let dir = TempDir::new().unwrap();
    setup_project(&dir);
    
    Command::cargo_bin("graxaim")
        .unwrap()
        .current_dir(dir.path())
        .args(&["command", "args"])
        .assert()
        .success()
        .stdout(predicates::str::contains("expected output"));
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
- `test_init_discovers_existing_profiles`
- `test_use_switches_symlink`

## Quality Gate

Before every commit:
```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test --all
```

All three must pass. Zero clippy warnings.
```

---

## Skills

### 1. `skills/tdd-graxaim/SKILL.md`

Adapted from rtk's TDD workflow:

**Content outline:**

```markdown
---
name: tdd-graxaim
description: TDD workflow for graxaim feature development. Red-Green-Refactor with 
  .env-specific patterns. Auto-triggers on new command implementation.
triggers:
  - "new command"
  - "implement feature"
  - "add command"
  - "write tests for"
---

# graxaim TDD Workflow

## The Loop

1. RED   — Write failing test with realistic .env fixture
2. GREEN — Implement minimum code to pass
3. REFACTOR — Clean up, verify still passing
4. EDGE  — Add empty/malformed/unicode edge case tests
5. GATE  — cargo fmt && cargo clippy && cargo test

## Step 1: Create Fixture

Use realistic .env content, not synthetic:

```toml
# tests/fixtures/sample.env
DATABASE_URL=postgres://localhost:5432/mydb
API_KEY=sk_test_12345678901234567890
PORT=3000
DEBUG=true
EMPTY_VALUE=
QUOTED_VALUE="hello world"
# This is a comment
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
    }

    #[test]
    fn test_new_feature_empty_input() {
        let result = new_feature("");
        assert!(result.is_ok()); // Must not panic
    }

    #[test]
    fn test_new_feature_malformed_input() {
        let result = new_feature("\x00binary\xffdata");
        // Either Ok with best-effort or specific error — never panic
        assert!(result.is_ok() || result.is_err());
    }
}
```

## Step 3: Implement (Green)

Write minimum code to make tests pass.

## Step 4: Quality Gate

```bash
cargo fmt --all && cargo clippy --all-targets -- -D warnings && cargo test
```

All three must pass before moving on.

## What "Done" Looks Like

- [ ] Happy path test passing
- [ ] Empty input test passing (no panic)
- [ ] Malformed input test passing (no panic)
- [ ] Edge case tests (unicode, Windows line endings, quoted values)
- [ ] Integration test with `assert_cmd`
- [ ] cargo fmt + clippy + test = all green
```

### 2. `skills/design-patterns/SKILL.md`

Adapted from rtk, focused on graxaim's domain:

**Key patterns:**
- **Newtype** — `ProfileName`, `SchemaType` for type safety
- **Builder** — `DiffConfig`, `AuditConfig`, `CheckConfig` for complex options
- **State Machine** — `.env` parser states, diff output rendering
- **Strategy** — Validation strategies per `VarType` in schema
- **RAII** — Symlink management, temp file cleanup in tests

### 3. `skills/ship/SKILL.md`

Release workflow for graxaim:

**Steps:**
1. Quality gate: `cargo fmt + clippy + test`
2. Version bump in `Cargo.toml`
3. Update `CHANGELOG.md`
4. Build release binary: `cargo build --release`
5. Create git tag: `v0.X.0`
6. Push tag to trigger GitHub Actions release

---

## Commands

### 1. `commands/verify-build.md`

Quick command to verify everything compiles and tests pass:

```markdown
# Verify Build

Run the full quality check:

1. `cargo fmt --all --check`
2. `cargo clippy --all-targets -- -D warnings`
3. `cargo test --all`
4. `cargo build --release`
5. `target/release/graxaim --version`

Report results with pass/fail for each step.
```

### 2. `commands/add-command.md`

Step-by-step guide for adding a new graxaim subcommand:

```markdown
# Add New Command

When adding a new graxaim subcommand:

1. Add variant to `Commands` enum in `src/cli.rs`
2. Create `src/commands/<name>.rs` with `pub fn execute(...) -> Result<()>`
3. Add `pub mod <name>;` to `src/commands/mod.rs`
4. Wire routing in `src/main.rs` match block
5. If new core module needed: create in `src/core/`, add to `src/core/mod.rs`
6. Write unit tests in the module's `#[cfg(test)] mod tests`
7. Write integration test in `tests/integration/<name>_test.rs`
8. Run quality gate: `cargo fmt && cargo clippy && cargo test`
```

---

## Implementation Priority

This `.claude/` directory should be created as part of **Step 3** (repo setup), before the initial commit. This way every future AI-assisted development session will have these guardrails from day one.

**Order of creation:**
1. `rules/graxaim-patterns.md` — most impactful, sets coding standards
2. `rules/env-file-safety.md` — security-critical for a tool handling secrets
3. `rules/testing-strategy.md` — ensures quality from the start
4. `commands/add-command.md` — used immediately when implementing Phases 2-5
5. `commands/verify-build.md` — quick reference
6. `skills/tdd-graxaim/SKILL.md` — used during feature development
7. `skills/design-patterns/SKILL.md` — reference material
8. `skills/ship/SKILL.md` — used when releasing
