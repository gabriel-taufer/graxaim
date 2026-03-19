use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GraxaimError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to parse .env file at {path}: {message}")]
    EnvFileParse { path: PathBuf, message: String },

    #[error("Failed to parse config file: {0}")]
    ConfigParse(#[from] toml::de::Error),

    #[error("Failed to serialize config: {0}")]
    ConfigSerialize(#[from] toml::ser::Error),

    #[error("Project not initialized. Run 'graxaim init' first.")]
    ProjectNotInitialized,

    #[error("Profile '{0}' not found")]
    ProfileNotFound(String),

    #[error("Profile '{0}' already exists")]
    ProfileAlreadyExists(String),

    #[error("Invalid profile name '{0}'. Only alphanumeric characters, hyphens, and underscores are allowed.")]
    InvalidProfileName(String),

    #[error("No active profile set")]
    NoActiveProfile,

    #[error("Active profile '{0}' symlink points to non-existent file")]
    #[allow(dead_code)]
    BrokenSymlink(String),

    #[error("Cannot find project root. Not in a git repository or .graxaim/ directory.")]
    ProjectRootNotFound,

    #[error("Cannot delete the active profile '{0}'. Switch to another profile first.")]
    CannotDeleteActiveProfile(String),

    #[error("No profiles found")]
    NoProfiles,

    #[error("User cancelled selection")]
    Cancelled,

    #[error("Editor error: {0}")]
    EditorError(String),

    #[error("Encryption error: {0}")]
    EncryptionError(String),

    #[error("{0}")]
    Custom(String),
}

pub type Result<T> = std::result::Result<T, GraxaimError>;
