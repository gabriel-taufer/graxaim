use crate::core::profile::Profile;
use crate::core::project::Project;
use crate::errors::{GraxaimError, Result};

pub fn execute(shell: String) -> Result<()> {
    let project = Project::find()?;
    let profile = Profile::get_active(&project)?;

    // Load env file
    let env_file = profile.load_env_file()?;

    // Generate export commands based on shell
    match shell.as_str() {
        "bash" | "zsh" => {
            for entry in &env_file.entries {
                println!("export {}=\"{}\"", entry.key, escape_bash(&entry.value));
            }
        }
        "fish" => {
            for entry in &env_file.entries {
                println!("set -x {} \"{}\"", entry.key, escape_fish(&entry.value));
            }
        }
        _ => {
            return Err(GraxaimError::Custom(format!(
                "Unsupported shell: {}. Supported: bash, zsh, fish",
                shell
            )));
        }
    }

    Ok(())
}

/// Escape value for bash/zsh
fn escape_bash(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('$', "\\$")
}

/// Escape value for fish
fn escape_fish(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_bash() {
        assert_eq!(escape_bash("simple"), "simple");
        assert_eq!(escape_bash("with\"quote"), "with\\\"quote");
        assert_eq!(escape_bash("with$dollar"), "with\\$dollar");
        assert_eq!(escape_bash("with\\backslash"), "with\\\\backslash");
    }

    #[test]
    fn test_escape_fish() {
        assert_eq!(escape_fish("simple"), "simple");
        assert_eq!(escape_fish("with\"quote"), "with\\\"quote");
        assert_eq!(escape_fish("with\\backslash"), "with\\\\backslash");
    }
}
