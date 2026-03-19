use crate::core::profile::Profile;
use crate::core::project::Project;
use crate::errors::{GraxaimError, Result};
use std::process::Command;

pub fn execute(command: Vec<String>) -> Result<()> {
    if command.is_empty() {
        return Err(GraxaimError::Custom(
            "No command specified. Usage: graxaim run -- <command>".to_string(),
        ));
    }

    let project = Project::find()?;
    let profile = Profile::get_active(&project)?;

    // Load env file
    let env_file = profile.load_env_file()?;

    // Build command
    let program = &command[0];
    let args = &command[1..];

    // Execute command with environment variables
    let status = Command::new(program)
        .args(args)
        .envs(env_file.entries.iter().map(|e| (&e.key, &e.value)))
        .status()
        .map_err(|e| {
            GraxaimError::Custom(format!("Failed to execute command '{}': {}", program, e))
        })?;

    // Exit with the same code as the child process
    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}
