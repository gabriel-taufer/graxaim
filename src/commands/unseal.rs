use crate::core::encryption;
use crate::core::project::Project;
use crate::errors::{GraxaimError, Result};
use crate::ui::output;
use std::io::{self, Write};

pub fn execute(profile_name: &str, output_path: Option<&str>, force: bool) -> Result<()> {
    let project = Project::find()?;
    let profile_path = project.root.join(format!(".env.{}", profile_name));
    let sealed = encryption::sealed_path(&profile_path);

    if !sealed.exists() {
        return Err(GraxaimError::Custom(format!(
            "No sealed file found for profile '{}' (expected: {})",
            profile_name,
            sealed.display()
        )));
    }

    // Read ciphertext
    let ciphertext = std::fs::read(&sealed).map_err(GraxaimError::Io)?;

    // Prompt passphrase
    let pass = encryption::prompt_passphrase("Enter passphrase: ")
        .map_err(|e| GraxaimError::EncryptionError(e.to_string()))?;

    // Decrypt
    let plaintext = encryption::unseal(&ciphertext, &pass)
        .map_err(|e| GraxaimError::EncryptionError(format!("Decryption failed — {}", e)))?;

    // Write output
    let dest = output_path
        .map(std::path::PathBuf::from)
        .unwrap_or(profile_path);

    // Safety check: prompt for confirmation if destination already exists
    if dest.exists() && !force {
        output::warning(&format!("File '{}' already exists.", dest.display()));

        print!("Overwrite? (y/N): ");
        io::stdout()
            .flush()
            .map_err(|e| GraxaimError::Custom(format!("Failed to prompt user: {}", e)))?;

        let mut response = String::new();
        io::stdin()
            .read_line(&mut response)
            .map_err(|e| GraxaimError::Custom(format!("Failed to read user input: {}", e)))?;

        let response = response.trim().to_lowercase();
        if response != "y" && response != "yes" {
            output::info("Unseal cancelled.");
            return Ok(());
        }
    }

    std::fs::write(&dest, &plaintext).map_err(GraxaimError::Io)?;

    output::success(&format!(
        "Profile '{}' unsealed → {}",
        profile_name,
        dest.display()
    ));

    Ok(())
}
