use age::{Decryptor, Encryptor};
use anyhow::{Context, Result};
use secrecy::{ExposeSecret, SecretString};
use std::io::{Cursor, Read, Write};
use std::path::Path;

/// Seals a plaintext byte slice with the given passphrase.
/// Returns the encrypted bytes (binary age format).
pub fn seal(plaintext: &[u8], passphrase: &SecretString) -> Result<Vec<u8>> {
    let encryptor =
        Encryptor::with_user_passphrase(SecretString::new(passphrase.expose_secret().clone()));

    let mut output = Vec::new();
    let mut writer = encryptor
        .wrap_output(&mut output)
        .context("Failed to create age encryption writer")?;

    writer
        .write_all(plaintext)
        .context("Failed to write plaintext to age stream")?;

    writer
        .finish()
        .context("Failed to finalize age encryption")?;

    Ok(output)
}

/// Unseals an age-encrypted byte slice with the given passphrase.
/// Returns the original plaintext bytes.
pub fn unseal(ciphertext: &[u8], passphrase: &SecretString) -> Result<Vec<u8>> {
    let decryptor =
        Decryptor::new(Cursor::new(ciphertext)).context("Failed to parse age ciphertext")?;

    match decryptor {
        Decryptor::Passphrase(d) => {
            let pass = SecretString::new(passphrase.expose_secret().clone());
            let mut reader = d
                .decrypt(&pass, None)
                .context("Failed to decrypt: wrong passphrase or corrupted data")?;

            let mut plaintext = Vec::new();
            reader
                .read_to_end(&mut plaintext)
                .context("Failed to read decrypted data")?;

            Ok(plaintext)
        }
        _ => anyhow::bail!("Unexpected age format: expected passphrase-encrypted file"),
    }
}

/// Returns the sealed file path for a profile path.
/// e.g. `.env.production` → `.env.production.sealed`
pub fn sealed_path(profile_path: &Path) -> std::path::PathBuf {
    let mut p = profile_path.to_path_buf();
    let ext = p
        .extension()
        .map(|e| format!("{}.sealed", e.to_string_lossy()))
        .unwrap_or_else(|| "sealed".to_string());
    p.set_extension(&ext);
    p
}

/// Prompts for a passphrase from stdin (no echo).
pub fn prompt_passphrase(prompt: &str) -> Result<SecretString> {
    let pass = rpassword::prompt_password(prompt).context("Failed to read passphrase")?;
    Ok(SecretString::new(pass))
}

#[cfg(test)]
mod tests {
    use super::*;
    use secrecy::SecretString;

    #[test]
    fn test_round_trip() {
        let plaintext = b"DATABASE_URL=postgres://localhost/mydb\nAPI_KEY=secret123\n";
        let pass = SecretString::new("test-passphrase-123".to_string());
        let encrypted = seal(plaintext, &pass).unwrap();
        assert_ne!(encrypted.as_slice(), plaintext.as_slice());
        let decrypted = unseal(&encrypted, &pass).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_wrong_passphrase_fails() {
        let plaintext = b"SECRET=value\n";
        let pass = SecretString::new("correct-passphrase".to_string());
        let encrypted = seal(plaintext, &pass).unwrap();
        let wrong_pass = SecretString::new("wrong-passphrase".to_string());
        assert!(unseal(&encrypted, &wrong_pass).is_err());
    }

    #[test]
    fn test_sealed_path() {
        use std::path::PathBuf;
        let p = PathBuf::from(".env.production");
        let sealed = sealed_path(&p);
        assert_eq!(sealed, PathBuf::from(".env.production.sealed"));
    }
}
