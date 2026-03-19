# Add New Command

Step-by-step guide for adding a new graxaim subcommand. Follow each step in order.

## Step 1: Add CLI variant in `src/cli.rs`

Add a new variant to the `Commands` enum:

```rust
#[derive(Subcommand, Debug)]
pub enum Commands {
    // ... existing commands ...

    /// Description of what the new command does
    NewCommand {
        /// Argument description
        #[arg(long)]
        some_flag: bool,

        /// Optional positional argument
        name: Option<String>,
    },
}
```

## Step 2: Create command module `src/commands/<name>.rs`

Create the command file following the standard module structure:

```rust
use anyhow::{Context, Result};
use crate::core::project::Project;

pub fn execute(some_flag: bool, name: Option<String>) -> Result<()> {
    let project = Project::discover()
        .context("failed to find graxaim project")?;

    // Command implementation here

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_happy_path() {
        // Test with tempdir setup
    }
}
```

## Step 3: Register module in `src/commands/mod.rs`

Add the public module declaration:

```rust
pub mod new_command;
```

## Step 4: Wire routing in `src/main.rs`

Add the match arm in the `main()` function:

```rust
fn main() -> Result<()> {
    let cli = Cli::parse();

    let result = match cli.command {
        // ... existing arms ...

        Commands::NewCommand { some_flag, name } => {
            commands::new_command::execute(some_flag, name)
        }
    };

    // ... error handling ...
}
```

## Step 5: Add core module if needed

If the command needs new core logic, create `src/core/<name>.rs`:

```rust
use anyhow::{Context, Result};

pub fn core_function(input: &str) -> Result<Output> {
    // Core logic separate from CLI concerns
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_function_happy_path() {
        let result = core_function("valid input");
        assert!(result.is_ok());
    }

    #[test]
    fn test_core_function_empty_input() {
        let result = core_function("");
        assert!(result.is_ok()); // or specific error, but never panic
    }
}
```

Then add to `src/core/mod.rs`:

```rust
pub mod new_module;
```

## Step 6: Write unit tests

In the command module's `#[cfg(test)] mod tests`:

- Happy path test
- Empty/missing input test
- Invalid input test (bad profile name, missing project, etc.)
- Edge cases specific to the command

## Step 7: Write integration test

Create `tests/integration/<name>_test.rs`:

```rust
use assert_cmd::Command;
use tempfile::TempDir;

fn setup_project(dir: &TempDir) {
    std::fs::create_dir_all(dir.path().join(".graxaim")).unwrap();
    std::fs::write(
        dir.path().join(".graxaim/config.toml"),
        "[project]\nname = \"test\"\n",
    ).unwrap();
}

#[test]
fn test_new_command_succeeds() {
    let dir = TempDir::new().unwrap();
    setup_project(&dir);

    Command::cargo_bin("graxaim")
        .unwrap()
        .current_dir(dir.path())
        .args(&["new-command", "--some-flag"])
        .assert()
        .success();
}

#[test]
fn test_new_command_without_init_fails() {
    let dir = TempDir::new().unwrap();

    Command::cargo_bin("graxaim")
        .unwrap()
        .current_dir(dir.path())
        .args(&["new-command"])
        .assert()
        .failure();
}
```

## Step 8: Quality gate

Run the full quality check:

```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test --all
```

All three must pass before committing.

## Checklist

- [ ] CLI variant added to `Commands` enum in `src/cli.rs`
- [ ] Command module created at `src/commands/<name>.rs`
- [ ] Module registered in `src/commands/mod.rs`
- [ ] Routing wired in `src/main.rs`
- [ ] Core module added (if needed) in `src/core/`
- [ ] Unit tests written and passing
- [ ] Integration test written and passing
- [ ] Quality gate: fmt + clippy + test = all green
