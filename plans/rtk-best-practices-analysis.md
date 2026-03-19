# RTK Best Practices — Analysis for graxaim

**Source:** `~/Documents/personal/rtk/.claude/` (rules, skills, agents)  
**Purpose:** Identify what to incorporate into graxaim's codebase and development workflow

---

## Summary: What to Adopt vs. Skip

### ✅ ADOPT — High value for graxaim

| Practice | Source | Why adopt | Priority |
|----------|--------|-----------|----------|
| Lazy static regex | `rules/rust-patterns.md` | Phase 4 audit uses regex heavily — avoid recompilation | 🔴 Critical |
| `.context()` on every `?` | `rules/rust-patterns.md` | graxaim already uses `thiserror` but some `?` lack context | 🔴 Critical |
| No `unwrap()` in production | `rules/rust-patterns.md` | Already in CLAUDE.md but need to audit existing code | 🔴 Critical |
| Iterator chains over loops | `rules/rust-patterns.md`, `skills/code-simplifier` | Cleaner, more idiomatic Rust | 🟡 Important |
| Edge case tests | `rules/cli-testing.md` | Empty input, malformed input, unicode — all relevant to `.env` parsing | 🔴 Critical |
| CI pipeline structure | `.github/workflows/ci.yml` | fmt → clippy → test cascade, cross-platform, security scan | 🔴 Critical |
| `cargo audit` in CI | `.github/workflows/ci.yml` | graxaim uses `age` encryption crate — must check for CVEs | 🟡 Important |
| CONTRIBUTING.md | `CONTRIBUTING.md` | Public repo needs contribution guidelines | 🟡 Important |
| SECURITY.md | `SECURITY.md` | Project handles secrets — needs security policy | 🟡 Important |
| CHANGELOG.md | `skills/ship.md` | Track changes across versions | 🟡 Important |
| Dangerous pattern scanning | `.github/workflows/ci.yml` | Detect `unwrap()`, `unsafe`, shell injection in CI | 🟡 Important |
| Ship/release workflow | `skills/ship.md` | Versioned releases with tags, binary builds | 🟢 Later |
| Performance benchmarks | `skills/performance.md` | Startup time <50ms, memory profiling | 🟢 Later |
| Newtype pattern | `skills/design-patterns` | Profile names should be validated newtypes | 🟡 Important |
| State machine for parsing | `skills/design-patterns` | `.env` parser could benefit from explicit states | 🟢 Later |
| Builder pattern for config | `skills/design-patterns` | `FilterConfig` / `DiffConfig` with many options | 🟡 Important |

### ⏭️ SKIP — Not applicable to graxaim

| Practice | Source | Why skip |
|----------|--------|----------|
| Token savings tests | `rules/cli-testing.md` | RTK-specific — graxaim doesn't filter/compress output |
| Snapshot testing with `insta` | `rules/cli-testing.md`, `skills/tdd-rust` | Good but adds another dependency; `assert_cmd` is enough for now |
| Fallback pattern | `rules/rust-patterns.md` | RTK proxies commands — graxaim is standalone, no fallback needed |
| Exit code propagation | `rules/rust-patterns.md` | Only relevant for `graxaim run`, already handled |
| No async | `rules/rust-patterns.md` | graxaim already has no async — already aligned |
| TOML filter DSL | Various | RTK-specific feature |
| AI doc review in CI | `ci.yml` | Overkill for graxaim at this stage |

---

## Detailed Recommendations

### 1. Codebase Audit — Apply rust-patterns rules

**Action:** Before first commit, audit the existing codebase against rtk patterns.

**Specific checks:**
- [ ] Search for `unwrap()` outside of tests and `lazy_static!` inits — replace with `.context()?`
- [ ] Search for bare `?` without `.context()` or `.with_context()` — add context strings
- [ ] Search for `Regex::new()` inside functions (Phase 4 audit will use this heavily)
- [ ] Verify all functions use `&str` instead of `&String` in signatures
- [ ] Check for unnecessary `.clone()` calls

**Current graxaim CLAUDE.md already says:**
> No `unwrap()` in library code, only in tests

This is good, but let's verify compliance.

### 2. CI Pipeline — Adopt rtk's tiered structure

**Adapt from rtk's [`.github/workflows/ci.yml`](rtk CI):**

```yaml
# Graxaim CI — adapted from rtk
jobs:
  fmt:           # Fast gate — fail early
  clippy:        # Needs: fmt
  test:          # Needs: clippy, matrix: [ubuntu, macos]
  security:      # Needs: clippy — cargo audit + dangerous patterns scan
```

**Key additions over a basic CI:**
- **`cargo audit`** — checks for known CVEs in dependencies (critical since we use `age` encryption)
- **Dangerous patterns scan** — grep the diff for `unwrap()`, `panic!()`, `unsafe`, `Command::new("sh")`
- **Rust cache** via `Swatinem/rust-cache@v2` — significantly speeds up CI
- **Cross-platform testing** — ubuntu + macos at minimum (graxaim uses Unix symlinks)

**What to drop from rtk's CI:**
- Windows testing (graxaim uses Unix symlinks — explicitly not supported)
- Benchmark job (premature for graxaim)
- AI doc review (overkill)
- DCO check (unnecessary for a solo project)

### 3. Lazy Static Regex — Critical for Phase 4

Phase 4 (Codebase Audit) will compile regex patterns for 7+ languages. Without `lazy_static!`, each `graxaim audit` call would recompile all patterns.

**Action:** Add `lazy_static` to `Cargo.toml` dependencies and use it in `core/auditor.rs`.

```rust
// src/core/auditor.rs
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref JS_PROCESS_ENV: Regex = Regex::new(r#"process\.env\.([A-Z_][A-Z0-9_]*)"#).unwrap();
    static ref PYTHON_GETENV: Regex = Regex::new(r#"os\.getenv\(['\"]([A-Z_][A-Z0-9_]*)['\"]"#).unwrap();
    static ref RUST_ENV_VAR: Regex = Regex::new(r#"env::var\(['\"]([A-Z_][A-Z0-9_]*)['\"]"#).unwrap();
    // ... more patterns
}
```

### 4. Newtype Pattern for Profile Names

graxaim validates profile names with `[a-zA-Z0-9_-]`. This should be a newtype to prevent passing raw strings:

```rust
pub struct ProfileName(String);

impl ProfileName {
    pub fn new(name: &str) -> Result<Self> {
        if !name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            return Err(GraxaimError::InvalidProfileName(name.to_string()));
        }
        Ok(Self(name.to_string()))
    }
    pub fn as_str(&self) -> &str { &self.0 }
}
```

**Where to use:** All command functions that accept profile names (`create`, `delete`, `rename`, `use`, `edit`, `diff`, `check`, `seal`, `unseal`).

### 5. Security Considerations

From `skills/security-guardian.md` — relevant to graxaim:

- **Hook security:** graxaim executes user-provided shell scripts as hooks. The current `HookRunner` already uses `Command::new(shell).arg(script_path)` which is safe. But we should:
  - Verify hooks are executable (already done)
  - Log which hooks are being executed
  - Never pass hook names through a shell interpolation

- **Shell command execution in `run`:** `graxaim run -- <command>` executes user commands. Current implementation in `run.rs` should be verified to use `Command::new()` safely.

- **Encryption key handling:** Phase 5 uses `age` crate. Must:
  - Never log passphrases
  - Clear passphrase from memory after use (use `secrecy` crate — already in Cargo.toml)
  - Verify `secrecy::SecretString` is used for passphrase input

### 6. Edge Case Testing Strategy

From `rules/cli-testing.md` — add these test patterns:

```rust
// For every core function, test:
#[test] fn test_empty_input() { ... }       // Empty .env file
#[test] fn test_malformed_input() { ... }   // Corrupted .env file
#[test] fn test_unicode_values() { ... }    // UTF-8 values in .env
#[test] fn test_huge_file() { ... }         // 10k line .env file
```

### 7. Documentation Files for Public Repo

From rtk's repo structure:

| File | Purpose | Adapt from |
|------|---------|------------|
| `CONTRIBUTING.md` | How to contribute | rtk's CONTRIBUTING.md, simplified |
| `SECURITY.md` | Security policy | New, focused on env var handling |
| `CHANGELOG.md` | Version history | Start with Phase 1 entry |
| `.github/ISSUE_TEMPLATE/` | Bug/feature templates | Optional, add later |

### 8. Builder Pattern for Complex Configs

For Phase 2+ features with many options:

```rust
// Diff configuration
pub struct DiffConfig {
    redact: bool,
    show_same: bool,
    all_profiles: bool,
}

impl DiffConfig {
    pub fn new() -> Self { Self { redact: true, show_same: false, all_profiles: false } }
    pub fn no_redact(mut self) -> Self { self.redact = false; self }
    pub fn show_same(mut self) -> Self { self.show_same = true; self }
}
```

---

## Implementation Priority

**Before initial commit (Step 0 in roadmap):**
1. Audit for `unwrap()` in production code
2. Audit for bare `?` without `.context()`
3. Add `lazy_static` to Cargo.toml

**During repo setup (Steps 2-9 in roadmap):**
4. Create CONTRIBUTING.md
5. Create SECURITY.md  
6. Create CHANGELOG.md
7. Implement tiered CI pipeline with security scanning

**During Phase 2-5 development:**
8. Use `lazy_static!` for all regex in auditor
9. Introduce `ProfileName` newtype
10. Apply builder pattern for complex config structs
11. Add edge case tests for every new module

---

## What This Means for graxaim's `.claude/` Directory

Consider creating a similar `.claude/` structure for graxaim:

```
.claude/
├── rules/
│   └── graxaim-patterns.md     # Adapted from rtk rust-patterns
├── skills/
│   └── env-file-testing.md     # .env-specific testing patterns
└── commands/
    └── ship.md                 # Release workflow
```

This is optional but would maintain consistency with the rtk project and provide guardrails for future contributors (human or AI).
