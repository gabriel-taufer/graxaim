use crate::core::env_file::EnvFile;
use crate::errors::{GraxaimError, Result};

pub fn escape_bash(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('$', "\\$")
        .replace('`', "\\`")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

pub fn escape_fish(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

pub fn print_exports(env_file: &EnvFile, shell: &str) -> Result<()> {
    match shell {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_bash() {
        assert_eq!(escape_bash("simple"), "simple");
        assert_eq!(escape_bash("with\"quote"), "with\\\"quote");
        assert_eq!(escape_bash("with$dollar"), "with\\$dollar");
        assert_eq!(escape_bash("with\\backslash"), "with\\\\backslash");
        assert_eq!(escape_bash("with`backtick"), "with\\`backtick");
        assert_eq!(escape_bash("with\nnewline"), "with\\nnewline");
        assert_eq!(escape_bash("with\rcarriage"), "with\\rcarriage");
    }

    #[test]
    fn test_escape_fish() {
        assert_eq!(escape_fish("simple"), "simple");
        assert_eq!(escape_fish("with\"quote"), "with\\\"quote");
        assert_eq!(escape_fish("with\\backslash"), "with\\\\backslash");
    }
}
