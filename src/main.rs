mod cli;
mod commands;
mod core;
mod errors;
mod ui;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};

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
        } => commands::use_profile::execute(name, no_hooks, strict_hooks),

        Commands::List => commands::list::execute(),

        Commands::Current => commands::current::execute(),

        Commands::Create { name, from } => commands::create::execute(name, from),

        Commands::Delete { name, yes } => commands::delete::execute(name, yes),

        Commands::Rename { old_name, new_name } => commands::rename::execute(old_name, new_name),

        Commands::Edit { name } => commands::edit::execute(name),

        Commands::Run { command } => commands::run::execute(command),

        Commands::Export { shell } => commands::export::execute(shell),
    };

    if let Err(e) = result {
        ui::output::error(&e.to_string());
        std::process::exit(1);
    }

    Ok(())
}
