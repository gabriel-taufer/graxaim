---
name: design-patterns
description: >
  Design patterns for graxaim modules. Newtype, Builder, State Machine, Strategy,
  and RAII patterns adapted for .env profile management.
triggers:
  - "design pattern"
  - "refactor module"
  - "type safety"
  - "newtype"
  - "builder pattern"
---

# graxaim Design Patterns

## Newtype Pattern — Type Safety at the Boundary

Use newtypes to prevent mixing semantically different strings.

### `ProfileName` — Validated profile identifier

```rust
pub struct ProfileName(String);

impl ProfileName {
    pub fn new(name: &str) -> Result<Self> {
        let re = regex!(r"^[a-zA-Z0-9_-]+$");
        if re.is_match(name) {
            Ok(Self(name.to_string()))
        } else {
            Err(GraxaimError::InvalidProfileName(name.to_string()))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
```

**Why:** Prevents shell injection in hook scripts. A `ProfileName` is guaranteed
safe to interpolate into `Command::env()`. A raw `String` is not.

### When to use Newtype

- User-provided identifiers (profile names, schema types)
- File paths that need validation (project root, config path)
- Values with format constraints (version strings, semver)

## Builder Pattern — Complex Configuration

Use builders when a struct has 4+ optional fields.

### `DiffConfig` — Diff display options

```rust
pub struct DiffConfig {
    pub color: bool,
    pub context_lines: usize,
    pub show_values: bool,
    pub ignore_comments: bool,
}

impl Default for DiffConfig {
    fn default() -> Self {
        Self {
            color: true,
            context_lines: 3,
            show_values: false, // redact by default
            ignore_comments: false,
        }
    }
}
```

### `AuditConfig` — Audit log settings

```rust
pub struct AuditConfig {
    pub enabled: bool,
    pub log_path: Option<PathBuf>,
    pub include_timestamps: bool,
    pub redact_values: bool,
}
```

**When to use:** Any configuration struct with more than 3 fields where most
have sensible defaults. Use `Default` trait + field overrides instead of
a full builder if the struct is simple enough.

## State Machine — Parser States

Use explicit state machines for parsing .env files.

```rust
enum ParserState {
    LineStart,
    InKey,
    AfterEquals,
    InUnquotedValue,
    InDoubleQuotedValue,
    InSingleQuotedValue,
    InComment,
    InEscapeSequence { return_to: Box<ParserState> },
}
```

**Why:** Makes it impossible to forget a state transition. Each state
only accepts valid transitions, caught at compile time via exhaustive `match`.

### Diff Output Rendering

```rust
enum DiffLine {
    Added(String),
    Removed(String),
    Modified { key: String, old: String, new: String },
    Unchanged(String),
    Comment(String),
}
```

## Strategy Pattern — Validation per Type

Different validation logic per `VarType` in schema validation:

```rust
pub enum VarType {
    String,
    Integer,
    Boolean,
    Url,
    Email,
    Custom(String), // regex pattern
}

impl VarType {
    pub fn validate(&self, value: &str) -> Result<()> {
        match self {
            VarType::String => Ok(()), // anything goes
            VarType::Integer => value.parse::<i64>()
                .map(|_| ())
                .context("expected integer value"),
            VarType::Boolean => match value {
                "true" | "false" | "1" | "0" => Ok(()),
                _ => Err(anyhow!("expected boolean value")),
            },
            VarType::Url => validate_url(value),
            VarType::Email => validate_email(value),
            VarType::Custom(pattern) => validate_regex(value, pattern),
        }
    }
}
```

## RAII — Resource Cleanup

### Symlink Management

```rust
struct SymlinkGuard {
    path: PathBuf,
    original_target: Option<PathBuf>,
}

impl Drop for SymlinkGuard {
    fn drop(&mut self) {
        if let Some(ref target) = self.original_target {
            // Restore original symlink on failure
            let _ = std::fs::remove_file(&self.path);
            let _ = std::os::unix::fs::symlink(target, &self.path);
        }
    }
}
```

### Temp File Cleanup in Tests

```rust
// TempDir from tempfile crate auto-cleans on drop
let dir = TempDir::new().expect("failed to create temp dir");
// dir is cleaned up when it goes out of scope
```

## Pattern Selection Guide

| Scenario | Pattern | Example |
|----------|---------|---------|
| User-provided identifier | Newtype | `ProfileName`, `SchemaType` |
| 4+ optional config fields | Builder / Default | `DiffConfig`, `AuditConfig` |
| Multi-state parsing | State Machine | `.env` parser, diff renderer |
| Behavior varies by type | Strategy | `VarType::validate()` |
| Resource needs cleanup | RAII | `SymlinkGuard`, `TempDir` |
| Transform pipeline | Iterator chain | `env_vars.iter().filter().map()` |

## Anti-Patterns

| Anti-Pattern | Problem | Alternative |
|-------------|---------|-------------|
| Over-engineering with traits | graxaim is small; don't abstract prematurely | Start concrete, extract trait if 2+ impls needed |
| Singleton / global state | Hard to test, hidden coupling | Pass config as parameter |
| Async traits | graxaim is sync-only; adds complexity for no gain | Synchronous functions |
| Deep inheritance via trait objects | Rust isn't OOP; composition > inheritance | Enum dispatch or generics |
| `Arc<Mutex<T>>` for shared state | No threads in graxaim | Pass `&mut T` directly |
