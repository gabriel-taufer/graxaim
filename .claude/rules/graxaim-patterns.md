# graxaim Rust Patterns

## Non-Negotiable Rules

1. **No `unwrap()` in production** — Use `.context("description")?` or `.with_context(|| format!(...))`.
   Tests may use `expect("reason")` with an explanatory message.

2. **Lazy regex** — `Regex::new()` inside a function recompiles every call.
   Always use `lazy_static!` or `std::sync::LazyLock` (Rust 1.80+).

3. **No async** — graxaim is single-threaded, synchronous. No tokio, no async-std, no futures.

4. **Error context on every `?`** — Every `?` must have `.context()` or `.with_context()`.
   Bad: `fs::read_to_string(path)?`
   Good: `fs::read_to_string(path).context("failed to read .env file")?`

5. **`thiserror` for library errors, `anyhow` only in `main.rs`** — Typed errors live in
   `src/errors.rs` using `thiserror`. `anyhow::Result` is only used in `src/main.rs`.

6. **`&str` over `&String`** — Function signatures should accept `&str`, not `&String`.

## Profile Name Validation

Always use the `ProfileName` newtype for user-provided profile names:
- Validates `[a-zA-Z0-9_-]` at construction time
- Prevents shell injection in hook scripts
- Compile-time safety against mixing profile names with file paths

```rust
// Good — validated at the boundary
let name = ProfileName::new(user_input)?;
execute_hook(&name);

// Bad — raw string passed around
execute_hook(&user_input); // could contain "; rm -rf /"
```

## .env File Operations

- Parse with our custom parser (`src/core/env_file.rs`), never external crates like `dotenv`
- Preserve formatting for round-trip operations (comments, blank lines, quoting style)
- Handle: Windows line endings (`\r\n`), quoted values, escape sequences, comments
- Always read files with explicit UTF-8 handling
- Only split on the first `=` sign: `BASE64=abc=def==` → key=`BASE64`, value=`abc=def==`

## Symlink Safety

- Always check if symlink target exists before operating
- Handle broken symlinks gracefully (dangling `.env` → deleted profile)
- Use `std::os::unix::fs::symlink` with proper error context
- Never follow symlinks when listing profiles (avoid infinite loops)

```rust
// Good
std::os::unix::fs::symlink(&target, &link_path)
    .with_context(|| format!("failed to create symlink {} -> {}", link_path.display(), target.display()))?;

// Bad
std::os::unix::fs::symlink(&target, &link_path)?;
```

## Hook Execution Safety

- Execute hooks via `Command::new(shell).arg(script_path)` — never shell interpolation
- Verify script is executable before running
- Always set timeout (default 30s)
- Inject env vars via `Command::env()`, not shell variable expansion
- Kill child process on timeout, don't just wait

```rust
// Good — safe execution
Command::new("sh")
    .arg(&script_path)
    .env("GRAXAIM_PROFILE", name.as_ref())
    .current_dir(&project_root)

// Bad — shell interpolation
Command::new("sh")
    .arg("-c")
    .arg(format!("PROFILE={} {}", name, script_path.display()))
```

## Module Structure

Every `src/commands/*.rs` follows this pattern:

```rust
// 1. Imports
use anyhow::{Context, Result};
use crate::core::project::Project;

// 2. Public execute() function
pub fn execute(args: ...) -> Result<()> {
    // command logic
}

// 3. Private helper functions
fn helper_function() -> Result<()> {
    // ...
}

// 4. Unit tests in same file
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_happy_path() {
        // ...
    }
}
```

## Anti-Patterns

| Pattern | Problem | Fix |
|---------|---------|-----|
| `Regex::new()` in function body | Recompiles every call | `lazy_static!` or `LazyLock` |
| `unwrap()` in production code | Panic crashes CLI | `.context()?` |
| `&String` in function signature | Unnecessary restriction | `&str` |
| Silent `match Err(_) => {}` | Swallows errors | Log with `eprintln!` or propagate |
| `clone()` of large strings | Extra allocation | Borrow with `&str` |
| Nested match > 2 levels | Hard to read | Early returns or `?` operator |
