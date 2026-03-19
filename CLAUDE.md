# CLAUDE.md — graxaim Implementation Guide

> This file is the source of truth for Claude Code when working on graxaim.
> Read this before making any changes to the codebase.

## What is graxaim

**graxaim** (pronounced "gra-sha-EEM") is a local-first CLI tool that manages named `.env` profiles per project. It handles switching between profiles, running post-switch hooks, diffing profiles, validating them against a schema, and auditing them against source code references.

Named after the *graxaim-do-campo* (Lycalopex gymnocercus), the pampas fox from southern Brazil — adaptable, clever, thrives in any environment.

Read `docs/PRD.md` for the full product requirements document.

---

## Tech Stack

- **Language:** Rust (2021 edition)
- **CLI framework:** `clap` v4 (derive API)
- **Config format:** TOML (`toml` crate)
- **Colored output:** `owo-colors`
- **Fuzzy picker:** `skim` (fzf-like, pure Rust, no external dependency)
- **Encryption:** `age` crate
- **Regex (for audit):** `regex` crate
- **File watching (future):** `notify` crate
- **Testing:** built-in `#[test]`, `assert_cmd` for CLI integration tests, `tempfile` for temp dirs

---

## Project Structure

```
graxaim/
├── CLAUDE.md                  # this file
├── Cargo.toml
├── docs/
│   └── PRD.md                 # product requirements document
├── src/
│   ├── main.rs                # entry point, clap CLI definition
│   ├── cli.rs                 # clap derive structs for all commands/subcommands
│   ├── commands/
│   │   ├── mod.rs
│   │   ├── init.rs            # graxaim init
│   │   ├── use_profile.rs     # graxaim use [name]
│   │   ├── list.rs            # graxaim list
│   │   ├── create.rs          # graxaim create <n>
│   │   ├── delete.rs          # graxaim delete <n>
│   │   ├── rename.rs          # graxaim rename <old> <new>
│   │   ├── current.rs         # graxaim current
│   │   ├── edit.rs            # graxaim edit [name]
│   │   ├── diff.rs            # graxaim diff <a> <b>
│   │   ├── check.rs           # graxaim check [name] --all
│   │   ├── audit.rs           # graxaim audit
│   │   ├── seal.rs            # graxaim seal <n>
│   │   ├── unseal.rs          # graxaim unseal <n>
│   │   ├── run.rs             # graxaim run -- <command>
│   │   ├── export.rs          # graxaim export
│   │   └── schema.rs          # graxaim schema init | generate-example
│   ├── core/
│   │   ├── mod.rs
│   │   ├── profile.rs         # Profile struct, load/save/list/symlink logic
│   │   ├── config.rs          # .graxaim/config.toml read/write
│   │   ├── hooks.rs           # hook discovery, execution, output capture
│   │   ├── schema.rs          # schema.toml parsing, type definitions, validation engine
│   │   ├── env_file.rs        # .env file parser (key=value, comments, blank lines)
│   │   ├── differ.rs          # diff engine: compare two EnvFile instances
│   │   ├── auditor.rs         # source code scanner, pattern matching, cross-reference
│   │   ├── encryption.rs      # age seal/unseal wrappers
│   │   └── project.rs         # project root detection, .graxaim/ directory management
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── output.rs          # colored/formatted terminal output helpers
│   │   ├── picker.rs          # interactive fuzzy profile picker (skim)
│   │   └── redact.rs          # value redaction logic (sk_...4xf)
│   └── errors.rs              # error types (thiserror)
├── tests/
│   ├── integration/
│   │   ├── init_test.rs
│   │   ├── use_test.rs
│   │   ├── hooks_test.rs
│   │   ├── diff_test.rs
│   │   ├── check_test.rs
│   │   └── audit_test.rs
│   └── fixtures/
│       ├── sample_project/    # a fake project with .env files for testing
│       └── sample_schema.toml
└── .github/
    └── workflows/
        └── ci.yml             # cargo test, cargo clippy, cargo fmt --check
```

---

## Key Data Structures

### EnvFile

Represents a parsed `.env` file. Preserves ordering and comments for round-tripping.

```rust
pub struct EnvEntry {
    pub key: String,
    pub value: String,
    pub comment: Option<String>,  // inline comment
    pub line_number: usize,
}

pub struct EnvFile {
    pub entries: Vec<EnvEntry>,
    pub header_comments: Vec<String>,  // comments before first key
}
```

### Profile

```rust
pub struct Profile {
    pub name: String,           // e.g. "staging"
    pub path: PathBuf,          // e.g. /project/.env.staging
    pub is_active: bool,
    pub is_sealed: bool,        // .env.staging.sealed exists
    pub has_hook: bool,         // .graxaim/hooks/staging.post.sh exists
}
```

### ProjectConfig (.graxaim/config.toml)

```toml
[project]
active_profile = "local"

[settings]
redact_by_default = true
redact_min_length = 8        # only redact values longer than this
gitignore_plaintext = true   # auto-add .env.* to .gitignore on init
envrc_integration = true     # generate .envrc on init

[hooks]
enabled = true               # master switch for hooks
shell = "/bin/sh"            # shell to execute hooks with (default: /bin/sh)
timeout = 30                 # max seconds a hook can run before being killed
```

### Hook System

Hooks are shell scripts that run at specific points during profile switching. They live in `.graxaim/hooks/` and follow a naming convention:

```
.graxaim/hooks/
├── _pre.sh                  # runs BEFORE any profile switch
├── _post.sh                 # runs AFTER any profile switch
├── production.pre.sh        # runs BEFORE switching TO production
├── production.post.sh       # runs AFTER switching TO production
├── staging.post.sh          # runs AFTER switching TO staging
└── _leave_staging.sh        # runs when LEAVING staging (before switch)
```

**Execution order for `graxaim use production` (switching from staging):**

1. `_leave_staging.sh` (if exists) — cleanup for the profile being left
2. `_pre.sh` (if exists) — global pre-switch
3. `production.pre.sh` (if exists) — profile-specific pre-switch
4. **symlink switch happens** (.env → .env.production)
5. `production.post.sh` (if exists) — profile-specific post-switch
6. `_post.sh` (if exists) — global post-switch

**Hook environment:** Hooks are executed with the NEW profile's env vars already available in the environment (injected by graxaim), plus these special variables:

```bash
GRAXAIM_PROFILE=production        # the profile being switched TO
GRAXAIM_PREVIOUS_PROFILE=staging  # the profile being switched FROM
GRAXAIM_PROJECT_ROOT=/path/to/project
```

**Real-world example — AWS profile switch:**

```bash
#!/bin/sh
# .graxaim/hooks/production.post.sh

# Clear cached AWS sessions
rm -rf ~/.aws/cli/cache/*

# Re-authenticate with SSO
aws sso login --profile production

# Confirm identity
echo "AWS Identity:"
aws sts get-caller-identity

# Switch kubectl context to match
kubectl config use-context production-cluster
```

**Hook rules:**
- Hooks must be executable (`chmod +x`)
- A hook that exits non-zero prints a warning but does NOT abort the switch (env vars are already exported). Use `--strict-hooks` to abort on hook failure.
- Hook stdout/stderr is passed through to the terminal
- Hooks have a configurable timeout (default: 30s)
- `graxaim use --no-hooks` skips all hooks

```rust
pub struct HookRunner {
    pub hooks_dir: PathBuf,
    pub shell: String,
    pub timeout: Duration,
    pub enabled: bool,
}

pub enum HookPhase {
    Leave(String),    // _leave_{profile}.sh — leaving a profile
    GlobalPre,        // _pre.sh
    ProfilePre(String),  // {profile}.pre.sh
    // --- symlink switch happens here ---
    ProfilePost(String), // {profile}.post.sh
    GlobalPost,       // _post.sh
}
```

### Schema types

```rust
pub enum VarType {
    String { min_length: Option<usize>, max_length: Option<usize>, pattern: Option<String> },
    Integer { min: Option<i64>, max: Option<i64> },
    Port,
    Boolean,
    Url { schemes: Option<Vec<String>> },
    Email,
    Enum { values: Vec<String> },
    List { item_type: Option<Box<VarType>> },
    Path { must_exist: bool },
}

pub struct VarSchema {
    pub key: String,
    pub var_type: VarType,
    pub required: bool,
    pub sensitive: bool,
    pub default: Option<String>,
    pub description: Option<String>,
    pub example: Option<String>,
    pub depends_on: Option<String>,
}
```

---

## Implementation Phases

Build in this order. Each phase should be fully working and tested before moving to the next.

### Phase 1: Foundation + Core Profile Management + Hooks

**Goal:** Replace the shell function with a proper CLI, with hook support from day one.

1. Set up the Rust project with `clap` derive API
2. Implement `core/env_file.rs` — parse and write `.env` files
3. Implement `core/project.rs` — find project root (walk up looking for `.graxaim/` or `.git/`)
4. Implement `core/profile.rs` — list profiles, read active symlink, switch symlink
5. Implement `core/config.rs` — read/write `.graxaim/config.toml`
6. Implement `core/hooks.rs` — discover hooks, execute them in order, handle timeouts and errors
7. Implement commands: `init`, `use`, `list`, `create`, `delete`, `rename`, `current`, `edit`
8. Implement `run` and `export` commands
9. Implement `ui/picker.rs` — interactive picker for `use` with no args
10. Write integration tests for all commands including hooks

**Acceptance criteria:**
- `graxaim init` in a directory with existing `.env.*` files detects them all, creates `.graxaim/` with `hooks/` subdirectory
- `graxaim use staging` switches the symlink, runs hooks in correct order, prints confirmation
- `graxaim use` with no args opens a picker
- `graxaim use production --no-hooks` switches without running hooks
- `graxaim list` shows all profiles with active marker and hook indicator
- `graxaim run -- echo $DATABASE_URL` prints the value from the active profile
- A `production.post.sh` hook that runs a command works correctly after switching to production
- Hook timeout kills a hanging script after the configured duration
- Non-zero hook exit prints a warning but does not abort the switch (unless `--strict-hooks`)

### Phase 2: Diffing

**Goal:** Make it easy to compare profiles.

1. Implement `core/differ.rs` — compare two `EnvFile` instances, categorize keys
2. Implement `ui/redact.rs` — redaction logic
3. Implement `diff` command with `--redact` (default), `--no-redact`, `--all`, `--show-same`
4. Write tests for diff edge cases (empty profiles, identical profiles, one-sided)

**Acceptance criteria:**
- `graxaim diff local staging` shows grouped, colored output
- Values are redacted by default
- `graxaim diff --all` shows a matrix summary

### Phase 3: Schema Validation

**Goal:** Catch configuration errors before they hit runtime.

1. Implement `core/schema.rs` — parse `schema.toml`, validate an `EnvFile` against it
2. Implement all type validators (string, integer, port, boolean, url, email, enum, list, path)
3. Implement `depends_on` conditional requirement logic
4. Implement `check` command (single profile and `--all`)
5. Implement `schema init` (infer types from active profile values)
6. Implement `schema generate-example` (produce `.env.example` from schema)
7. Write comprehensive validation tests

**Acceptance criteria:**
- `graxaim check` reports missing required vars, type errors, constraint violations
- `graxaim check --all` produces a summary matrix
- `graxaim schema init` produces a reasonable schema from existing values
- Generated `.env.example` has comments with descriptions and example values

### Phase 4: Codebase Audit

**Goal:** Find dead and missing env vars by scanning source code.

1. Implement `core/auditor.rs` — regex-based scanner for env var patterns per language
2. Implement file discovery (respect `.gitignore`, skip `node_modules`, `target/`, etc.)
3. Cross-reference found vars with profiles and schema
4. Implement `audit` command
5. Write tests with fixture projects

**Acceptance criteria:**
- `graxaim audit` correctly identifies vars in code that are missing from profiles
- `graxaim audit` correctly identifies vars in profiles that are not referenced in code
- Scans 10k files in under 1 second
- Respects `.gitignore`

### Phase 5: Encryption

**Goal:** Allow profiles to be safely committed to git.

1. Implement `core/encryption.rs` — age encrypt/decrypt wrappers
2. Implement `seal` and `unseal` commands
3. Support both passphrase and age identity file
4. Write tests

**Acceptance criteria:**
- `graxaim seal production` produces `.env.production.sealed`
- `graxaim unseal production` restores the plaintext file
- Sealed files are deterministic given the same input and key (for git diff)

---

## Coding Conventions

- Use `thiserror` for error types, `anyhow` in `main.rs` only
- All commands return `Result<()>` and print their own output (don't return strings)
- Use `owo-colors` for terminal colors, always check `atty::is(Stream::Stdout)` before coloring
- Parse `.env` files ourselves (don't depend on external dotenv crates — they vary in behavior)
- All file operations use `std::fs` with explicit error context via `.with_context()`
- No `unwrap()` in library code, only in tests
- Run `cargo fmt` and `cargo clippy` before every commit
- Integration tests use `assert_cmd` and `tempfile` to create isolated project directories

## Test Strategy

- **Unit tests:** Every function in `core/` has tests in the same file (`#[cfg(test)] mod tests`)
- **Integration tests:** Every command has an integration test that runs the actual binary against a temp directory with fixture files
- **Hook tests:** Integration tests that create executable hook scripts, run `graxaim use`, and verify hooks executed in correct order with correct env vars
- **Fixture projects:** `tests/fixtures/sample_project/` contains a realistic project layout with multiple `.env.*` files, a `schema.toml`, hook scripts, and source files with env var references

---

## Edge Cases to Handle

- Profile names with special characters (only allow `[a-zA-Z0-9_-]`)
- `.env` file with no trailing newline
- `.env` file with Windows line endings (`\r\n`)
- Values with `=` signs in them (e.g., base64 encoded strings)
- Values with quotes (single, double) — preserve quoting style on round-trip
- Empty values (`KEY=` vs `KEY=""` vs missing key)
- Multiline values (quoted values with `\n`)
- Symlink already exists but points to a deleted file
- Running `graxaim` outside any project (clear error message)
- `.graxaim/` directory exists but `config.toml` is corrupted
- Two profiles with overlapping names (e.g., `prod` and `production`)
- Hook script exists but is not executable (warn and skip)
- Hook script hangs indefinitely (timeout and kill)
- Hook script fails (warn but continue, unless `--strict-hooks`)
- No previous profile on first switch (skip `_leave_*` hook, `GRAXAIM_PREVIOUS_PROFILE` is empty)
- Hooks directory doesn't exist (skip silently, don't error)
- Windows: hooks use `.cmd`/`.ps1` extensions instead of `.sh`