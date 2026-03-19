use graxaim::core::encryption;
use secrecy::SecretString;
use std::path::PathBuf;

#[test]
fn test_encryption_round_trip() {
    let plaintext = b"DATABASE_URL=postgres://prod\nSECRET_KEY=abc123\n";
    let pass = SecretString::new("test-pass".to_string());

    let encrypted = encryption::seal(plaintext, &pass).unwrap();
    let decrypted = encryption::unseal(&encrypted, &pass).unwrap();
    assert_eq!(decrypted, plaintext);
}

#[test]
fn test_wrong_passphrase_fails() {
    let plaintext = b"SECRET=value\n";
    let pass = SecretString::new("correct-passphrase".to_string());
    let encrypted = encryption::seal(plaintext, &pass).unwrap();
    let wrong_pass = SecretString::new("wrong-passphrase".to_string());
    assert!(encryption::unseal(&encrypted, &wrong_pass).is_err());
}

#[test]
fn test_sealed_path_convention() {
    assert_eq!(
        encryption::sealed_path(&PathBuf::from(".env.staging")),
        PathBuf::from(".env.staging.sealed")
    );

    assert_eq!(
        encryption::sealed_path(&PathBuf::from(".env.production")),
        PathBuf::from(".env.production.sealed")
    );
}

#[test]
fn test_encrypted_differs_from_plaintext() {
    let plaintext = b"API_KEY=super-secret-value\n";
    let pass = SecretString::new("my-passphrase".to_string());
    let encrypted = encryption::seal(plaintext, &pass).unwrap();
    // Encrypted output must not equal plaintext
    assert_ne!(encrypted.as_slice(), plaintext.as_slice());
    // And must not contain the secret in clear text
    assert!(!encrypted
        .windows(plaintext.len())
        .any(|w| w == plaintext.as_slice()));
}
