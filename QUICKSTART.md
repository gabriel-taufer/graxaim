# graxaim Quick Start Guide

Get up and running with graxaim in 5 minutes.

## 1. Install Rust

If you don't have Rust installed:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

Verify installation:
```bash
cargo --version
rustc --version
```

## 2. Build graxaim

```bash
cd /Users/gabriel/Documents/personal/graxaim

# Build in release mode (optimized, faster)
cargo build --release

# The binary will be at: target/release/graxaim
```

## 3. Run Tests

```bash
# Run all tests (should see ~64 tests pass)
cargo test

# Run with output to see details
cargo test -- --nocapture
```

Expected output:
```
running 64 tests
test core::config::tests::test_default_config ... ok
test core::env_file::tests::test_parse_simple ... ok
...
test result: ok. 64 passed; 0 failed; 0 ignored; 0 measured
```

## 4. Install Locally

```bash
# Install to ~/.cargo/bin (adds to PATH)
cargo install --path .

# Verify installation
graxaim --version
```

## 5. Try It Out

### Option A: Use the test project

Create a test project:
```bash
mkdir ~/graxaim-demo && cd ~/graxaim-demo

# Create some .env files
echo "DATABASE_URL=postgres://localhost/dev" > .env.local
echo "DATABASE_URL=postgres://staging.example.com/db" > .env.staging
echo "DATABASE_URL=postgres://prod.example.com/db" > .env.production

# Initialize graxaim
graxaim init

# Expected output:
# ✓ Initialized graxaim
#
# Discovered 3 profile(s):
#   • local
#   • production
#   • staging
```

### Option B: Use an existing project

If you already have a project with `.env` files:
```bash
cd ~/your-existing-project
graxaim init
```

## 6. Common Commands

### Switch profiles
```bash
# Switch to a specific profile
graxaim use staging

# Interactive picker (use arrow keys to select)
graxaim use
```

### List profiles
```bash
graxaim list

# Output:
# Profiles:
#   • local
#   • production
#   • staging (active)
```

### Show current profile
```bash
graxaim current
# Output: staging
```

### Create a new profile
```bash
# Create empty profile
graxaim create development

# Create by copying from another
graxaim create qa --from staging
```

### Edit a profile
```bash
# Edit specific profile
graxaim edit local

# Edit current active profile
graxaim edit
```

### Run commands with environment loaded
```bash
# Run any command with the active profile's env vars
graxaim run -- npm start
graxaim run -- echo $DATABASE_URL
graxaim run -- python app.py
```

### Export for shell
```bash
# Export to current shell (bash/zsh)
eval "$(graxaim export --shell bash)"

# Export for fish shell
eval (graxaim export --shell fish)
```

### Delete a profile
```bash
# With confirmation
graxaim delete development

# Skip confirmation
graxaim delete development --yes
```

### Rename a profile
```bash
graxaim rename staging stage
```

## 7. Integration with direnv (Optional)

If you use direnv, graxaim has already created a `.envrc` file:

```bash
# Allow direnv to load .env
direnv allow

# Now whenever you `cd` to this directory, your env loads automatically
# And when you run `graxaim use`, direnv will reload
```

## 8. Next Steps

- Read `README.md` for full documentation
- Check `PHASE1_VERIFICATION.md` for detailed testing scenarios
- See `docs/PRD.md` for the full product vision
- Explore upcoming features (diff, schema validation, audit, encryption)

## Troubleshooting

### "command not found: cargo"

Rust is not installed. Follow step 1 above.

### "command not found: graxaim"

After `cargo install --path .`, make sure `~/.cargo/bin` is in your PATH:
```bash
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

### Build errors

Make sure you're using a recent Rust version:
```bash
rustup update
```

### Tests fail

Make sure you're in the project directory:
```bash
cd /Users/gabriel/Documents/personal/graxaim
cargo test
```

## Getting Help

- Check the `--help` for any command: `graxaim --help` or `graxaim use --help`
- Read the code documentation in `CLAUDE.md`
- File an issue if you find a bug

---

**Enjoy using graxaim! 🚀**
