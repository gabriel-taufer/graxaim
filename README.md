# graxaim

> **graxaim** (pronounced "gra-sha-EEM") — named after the pampas fox from southern Brazil, known for its adaptability in diverse environments. Just like the graxaim navigates different terrains, this tool helps you seamlessly switch between development environments.

A local-first CLI tool that manages named `.env` profiles per project.

## Features (Phase 1 - Complete)

- 🔄 **Profile Management**: Create, delete, rename, and list `.env` profiles
- 🔀 **Quick Switching**: Switch between profiles with symlink management
- 🎯 **Interactive Picker**: Fuzzy-find profiles when switching
- 🚀 **Run Commands**: Execute commands with profile environment loaded
- 📤 **Export**: Generate shell-specific export commands
- 🔧 **Editor Integration**: Edit profiles in your `$EDITOR`
- 📝 **direnv Support**: Auto-generates `.envrc` for direnv integration

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

## Project Structure

```
.
├── .graxaim/          # graxaim configuration
│   └── config.toml      # project config
├── .env                 # symlink to active profile
├── .env.local           # local development profile
├── .env.staging         # staging profile
└── .env.production      # production profile
```

## How It Works

1. **Profiles**: Each profile is stored as `.env.<name>` in your project root
2. **Active Profile**: A `.env` symlink points to the active profile
3. **Configuration**: Settings are stored in `.graxaim/config.toml`
4. **Git Safety**: `.env` and `.env.*` are automatically added to `.gitignore`

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
- [ ] **Phase 2**: Diffing profiles
- [ ] **Phase 3**: Schema validation
- [ ] **Phase 4**: Codebase audit
- [ ] **Phase 5**: Encryption (seal/unseal)

## License

MIT

## Contributing

Contributions are welcome! Please read [CLAUDE.md](CLAUDE.md) for implementation guidelines.
