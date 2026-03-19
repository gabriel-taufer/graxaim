mod cli;
mod commands;
mod core;
mod errors;
mod ui;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands, SchemaCommands};

fn main() -> Result<()> {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Init {
            no_gitignore,
            no_envrc,
        } => commands::init::execute(no_gitignore, no_envrc),

        Commands::Use {
            name,
            no_hooks,
            strict_hooks,
            no_export,
            shell,
        } => commands::use_profile::execute(name, no_hooks, strict_hooks, no_export, shell),

        Commands::List => commands::list::execute(),

        Commands::Current => commands::current::execute(),

        Commands::Create { name, from } => commands::create::execute(name, from),

        Commands::Delete { name, yes } => commands::delete::execute(name, yes),

        Commands::Rename { old_name, new_name } => commands::rename::execute(old_name, new_name),

        Commands::Edit { name } => commands::edit::execute(name),

        Commands::Run { command } => commands::run::execute(command),

        Commands::Export { shell } => commands::export::execute(shell),

        Commands::Diff {
            profile_a,
            profile_b,
            no_redact,
            show_same,
        } => commands::diff::execute(profile_a, profile_b, !no_redact, show_same),

        Commands::Check { name, all } => commands::check::execute(name, all),

        Commands::Schema(subcmd) => match subcmd {
            SchemaCommands::Init => commands::schema::execute_init(),
            SchemaCommands::GenerateExample => commands::schema::execute_generate_example(),
        },

        Commands::Audit { profile } => commands::audit::execute(profile),

        Commands::Seal { profile, delete, force } => commands::seal::execute(&profile, delete, force),

        Commands::Unseal { profile, output, force } => {
            commands::unseal::execute(&profile, output.as_deref(), force)
        }
    };

    if let Err(e) = result {
        ui::output::error(&e.to_string());
        std::process::exit(1);
    }

    Ok(())
}
