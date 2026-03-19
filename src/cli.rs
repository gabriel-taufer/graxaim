use clap::{Parser, Subcommand};

#[derive(Subcommand, Debug)]
pub enum SchemaCommands {
    /// Generate schema from active profile
    Init,
    /// Generate .env.example from schema
    GenerateExample,
}

#[derive(Parser, Debug)]
#[command(name = "graxaim")]
#[command(about = "Manage named .env profiles per project", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize graxaim in the current project
    Init {
        /// Don't update .gitignore
        #[arg(long)]
        no_gitignore: bool,

        /// Don't generate .envrc
        #[arg(long)]
        no_envrc: bool,
    },

    /// Switch to a profile (creates symlink to .env)
    #[command(name = "use")]
    Use {
        /// Profile name (opens picker if not specified)
        name: Option<String>,

        /// Skip all hooks
        #[arg(long)]
        no_hooks: bool,

        /// Abort switch if any hook fails (default: warn and continue)
        #[arg(long)]
        strict_hooks: bool,

        /// Don't output export commands to stdout
        #[arg(long)]
        no_export: bool,

        /// Shell format for export commands (bash, zsh, fish)
        #[arg(long, default_value = "bash")]
        shell: String,
    },

    /// List all profiles
    List,

    /// Show the currently active profile
    Current,

    /// Create a new profile
    Create {
        /// Profile name
        name: String,

        /// Copy from existing profile
        #[arg(long)]
        from: Option<String>,
    },

    /// Delete a profile
    Delete {
        /// Profile name
        name: String,

        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },

    /// Rename a profile
    Rename {
        /// Old profile name
        old_name: String,

        /// New profile name
        new_name: String,
    },

    /// Edit a profile in $EDITOR
    Edit {
        /// Profile name (uses active profile if not specified)
        name: Option<String>,
    },

    /// Run a command with the active profile's environment
    Run {
        /// Command to run
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        command: Vec<String>,
    },

    /// Export the active profile as shell commands
    Export {
        /// Shell format (bash, zsh, fish)
        #[arg(long, default_value = "bash")]
        shell: String,
    },

    /// Compare two profiles
    Diff {
        /// First profile name
        profile_a: String,

        /// Second profile name (defaults to active profile)
        profile_b: Option<String>,

        /// Show full values (default: redacted)
        #[arg(long)]
        no_redact: bool,

        /// Show identical keys too
        #[arg(long)]
        show_same: bool,
    },

    /// Validate profiles against schema
    Check {
        /// Profile name (defaults to active)
        name: Option<String>,

        /// Validate all profiles
        #[arg(long)]
        all: bool,
    },

    /// Schema management
    #[command(subcommand)]
    Schema(SchemaCommands),

    /// Audit env vars against source code
    Audit {
        /// Audit specific profile (default: all profiles)
        #[arg(long)]
        profile: Option<String>,
    },

    /// Encrypt a profile with a passphrase
    Seal {
        /// Profile name to seal
        profile: String,
        /// Delete the original plaintext profile after sealing
        #[arg(long)]
        delete: bool,
        /// Overwrite an existing sealed file without prompting
        #[arg(long)]
        force: bool,
    },

    /// Decrypt a sealed profile
    Unseal {
        /// Profile name to unseal
        profile: String,
        /// Output file path (default: overwrites original profile)
        #[arg(long)]
        output: Option<String>,
        /// Overwrite existing file without confirmation
        #[arg(short, long)]
        force: bool,
    },
}
