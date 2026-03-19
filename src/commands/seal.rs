use crate::core::encryption;
use crate::core::project::Project;
use crate::errors::{GraxaimError, Result};
use crate::ui::output;
use secrecy::ExposeSecret;

pub fn execute(profile_name: &str, delete: bool, force: bool) -> Result<()> {
    let project = Project::find()?;
    let profile_path = project.root.join(format!(".env.{}", profile_name));

    if !profile_path.exists() {
        return Err(GraxaimError::ProfileNotFound(profile_name.to_string()));
    }

    let sealed = encryption::sealed_path(&profile_path);
    if sealed.exists() {
        if !force {
            return Err(GraxaimError::Custom(format!(
                "Sealed file already exists: {}\n  Use --force to overwrite it.",
                sealed.display()
            )));
        } else {
            output::warning(&format!(
                "Overwriting existing sealed file: {}",
                sealed.display()
            ));
        }
    }

    // Read plaintext
    let plaintext = std::fs::read(&profile_path).map_err(GraxaimError::Io)?;

    // Prompt passphrase (twice for confirmation)
    let pass1 = encryption::prompt_passphrase("Enter passphrase: ")
        .map_err(|e| GraxaimError::EncryptionError(e.to_string()))?;
    let pass2 = encryption::prompt_passphrase("Confirm passphrase: ")
        .map_err(|e| GraxaimError::EncryptionError(e.to_string()))?;

    if pass1.expose_secret() != pass2.expose_secret() {
        return Err(GraxaimError::EncryptionError(
            "Passphrases do not match".to_string(),
        ));
    }

    // Encrypt and write
    let ciphertext = encryption::seal(&plaintext, &pass1)
        .map_err(|e| GraxaimError::EncryptionError(e.to_string()))?;

    std::fs::write(&sealed, &ciphertext).map_err(GraxaimError::Io)?;

    output::success(&format!(
        "Profile '{}' sealed → {}",
        profile_name,
        sealed.display()
    ));

    if delete {
        std::fs::remove_file(&profile_path).map_err(GraxaimError::Io)?;
        output::success(&format!(
            "Plaintext profile '{}' deleted.",
            profile_path.display()
        ));
    } else {
        output::warning(&format!(
            "Plaintext profile still exists on disk: {}\n  Run with --delete to remove it, or delete it manually.",
            profile_path.display()
        ));
    }

    Ok(())
}
