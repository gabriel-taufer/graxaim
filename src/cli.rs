use clap::{Parser, Subcommand};

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
}
