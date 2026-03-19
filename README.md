# graxaim

> **graxaim** (pronounced "gra-sha-EEM") — named after the pampas fox from southern Brazil, known for its adaptability in diverse environments. Just like the graxaim navigates different terrains, this tool helps you seamlessly switch between development environments.

A local-first CLI tool that manages named `.env` profiles per project.

## Features

- 🔄 **Profile Management**: Create, delete, rename, and list `.env` profiles
- 🔀 **Quick Switching**: Switch between profiles with symlink management
- 🎯 **Interactive Picker**: Fuzzy-find profiles when switching
- 🚀 **Run Commands**: Execute commands with profile environment loaded
- 📤 **Export**: Generate shell-specific export commands
- 🔧 **Editor Integration**: Edit profiles in your `$EDITOR`
- 📝 **direnv Support**: Auto-generates `.envrc` for direnv integration
- 🔍 **Diff**: Compare any two profiles side-by-side
- ✅ **Schema Validation**: Define and enforce types and constraints on your env vars
- 🔬 **Audit**: Detect missing or dead env vars by scanning your source code
- 🔒 **Encryption**: Seal and unseal profiles with passphrase-based encryption (age)

## Installation

### One-Line Install (Recommended)

```bash
curl -sSL https://raw.githubusercontent.com/gabriel-taufer/graxaim/main/install.sh | bash
```

Or clone and run locally:

```bash
git clone https://github.com/gabriel-taufer/graxaim.git
cd graxaim
./install.sh
```

The install script will:
- ✅ Check if Rust is installed (and offer to install it)
- ✅ Build graxaim from source
- ✅ Install to `~/.cargo/bin`
- ✅ Verify the installation

### Manual Install

If you prefer to install manually:

```bash
# 1. Install Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. Clone and build
git clone https://github.com/gabriel-taufer/graxaim.git
cd graxaim
cargo install --path .
```

## Quick Start

```bash
# Initialize in your project (detects existing .env.* files)
graxaim init

# Create a new profile
graxaim create local

# Switch to a profile
graxaim use local

# Switch with interactive picker
graxaim use

# List all profiles
graxaim list

# Show current profile
graxaim current

# Edit a profile
graxaim edit local

# Run a command with the active profile's environment
graxaim run -- npm start

# Export environment variables for your shell
eval "$(graxaim export --shell bash)"
```

## Commands

### Profile Management

- `graxaim init` - Initialize graxaim in your project
- `graxaim create <name>` - Create a new profile
- `graxaim create <name> --from <source>` - Create by copying another profile
- `graxaim delete <name>` - Delete a profile
- `graxaim rename <old> <new>` - Rename a profile
- `graxaim list` - List all profiles
- `graxaim current` - Show the active profile

### Switching & Usage

- `graxaim use <name>` - Switch to a profile
- `graxaim use` - Interactive profile picker
- `graxaim edit [name]` - Edit a profile in `$EDITOR`
- `graxaim run -- <command>` - Run command with profile environment
- `graxaim export [--shell bash|zsh|fish]` - Export as shell commands

### Diff

Compare two profiles and highlight keys that are added, removed, or changed.

```bash
# Compare two named profiles
graxaim diff local staging

# Compare a profile against the currently active profile
graxaim diff production

# Show full values instead of redacted output
graxaim diff local staging --no-redact

# Also show keys that are identical in both profiles
graxaim diff local staging --show-same
```

### Schema Validation

Define types and constraints for your env vars and validate profiles against them.

```bash
# Infer a schema from the active profile and write .graxaim/schema.toml
graxaim schema init

# Generate a .env.example file from the schema
graxaim schema generate-example

# Validate the active profile against the schema
graxaim check

# Validate a specific profile
graxaim check staging

# Validate all profiles at once
graxaim check --all
```

**Schema file (`.graxaim/schema.toml`) example:**

```toml
[vars.DATABASE_URL]
type = "url"
schemes = ["postgres", "postgresql"]
required = true
description = "Primary database connection string"

[vars.PORT]
type = "port"
required = false
default = "3000"

[vars.LOG_LEVEL]
type = "enum"
values = ["debug", "info", "warn", "error"]
required = true

[vars.API_SECRET]
type = "string"
required = true
sensitive = true
min_length = 32
```

Supported types: `string`, `integer`, `port`, `boolean`, `url`, `email`, `enum`, `path`.

### Audit

Scan your source code to find env vars referenced in code but missing from profiles, and vars defined in profiles but never used in code.

```bash
# Audit all profiles against the codebase
graxaim audit

# Audit a single profile
graxaim audit --profile staging
```

Scanned languages: JavaScript/TypeScript, Python, Rust, Go, Ruby, PHP, YAML/Docker.

### Encryption (Seal / Unseal)

Encrypt profiles at rest using [age](https://age-encryption.org/) passphrase encryption. Useful for storing sensitive profiles in version control.

```bash
# Encrypt a profile (prompts for passphrase twice)
graxaim seal production

# Encrypt and delete the plaintext original
graxaim seal production --delete

# Decrypt a sealed profile back to its original location
graxaim unseal production

# Decrypt to a custom path
graxaim unseal production --output /tmp/.env.production
```

Sealed files are stored alongside the profile with a `.sealed` extension (e.g. `.env.production.sealed`).

## Project Structure

```
.
├── .graxaim/                  # graxaim configuration
│   ├── config.toml            # project config
│   ├── schema.toml            # schema definition (optional)
│   └── hooks/                 # lifecycle hook scripts (optional)
│       ├── _pre.sh            # runs before every profile switch
│       ├── _post.sh           # runs after every profile switch
│       ├── staging.pre.sh     # runs before switching TO staging
│       ├── staging.post.sh    # runs after switching TO staging
│       └── _leave_local.sh    # runs when leaving the local profile
├── .env                       # symlink to active profile
├── .env.local                 # local development profile
├── .env.staging               # staging profile
├── .env.production            # production profile
└── .env.production.sealed     # encrypted production profile (age)
```

## How It Works

1. **Profiles**: Each profile is stored as `.env.<name>` in your project root
2. **Active Profile**: A `.env` symlink points to the active profile
3. **Configuration**: Settings are stored in `.graxaim/config.toml`
4. **Git Safety**: `.env` and `.env.*` are automatically added to `.gitignore`
5. **Hooks**: Executable shell scripts in `.graxaim/hooks/` run at lifecycle events
6. **Schema**: Optional `.graxaim/schema.toml` enforces types and constraints across all profiles
7. **Encryption**: Sealed profiles (`.sealed`) can be safely committed to version control

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run integration tests only
cargo test --test '*'

# Run unit tests only
cargo test --lib

# Run specific test
cargo test test_init_in_empty_directory
```

### Code Quality

```bash
# Format code
cargo fmt

# Lint code
cargo clippy

# Check without building
cargo check
```

## Roadmap

- [x] **Phase 1**: Core profile management (init, use, list, create, delete, rename, current, edit, run, export)
- [x] **Phase 2**: Diff profiles
- [x] **Phase 3**: Schema validation (infer, validate, generate-example)
- [x] **Phase 4**: Codebase audit
- [x] **Phase 5**: Encryption (seal/unseal)

## License

MIT

## Contributing

Contributions are welcome! Please read [CLAUDE.md](CLAUDE.md) for implementation guidelines.
