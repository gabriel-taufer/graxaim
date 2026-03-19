use crate::core::project::Project;
use crate::errors::Result;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectConfig {
    #[serde(default)]
    pub project: ProjectSection,
    #[serde(default)]
    pub settings: SettingsSection,
    #[serde(default)]
    pub hooks: HooksSection,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectSection {
    pub active_profile: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsSection {
    #[serde(default = "default_redact_by_default")]
    pub redact_by_default: bool,

    #[serde(default = "default_redact_min_length")]
    pub redact_min_length: usize,

    #[serde(default = "default_gitignore_plaintext")]
    pub gitignore_plaintext: bool,

    #[serde(default = "default_envrc_integration")]
    pub envrc_integration: bool,
}

fn default_redact_by_default() -> bool {
    true
}

fn default_redact_min_length() -> usize {
    8
}

fn default_gitignore_plaintext() -> bool {
    true
}

fn default_envrc_integration() -> bool {
    true
}

impl Default for SettingsSection {
    fn default() -> Self {
        Self {
            redact_by_default: default_redact_by_default(),
            redact_min_length: default_redact_min_length(),
            gitignore_plaintext: default_gitignore_plaintext(),
            envrc_integration: default_envrc_integration(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HooksSection {
    #[serde(default = "default_hooks_enabled")]
    pub enabled: bool,

    #[serde(default = "default_hooks_shell")]
    pub shell: String,

    #[serde(default = "default_hooks_timeout")]
    pub timeout: u64,
}

fn default_hooks_enabled() -> bool {
    true
}

fn default_hooks_shell() -> String {
    "/bin/sh".to_string()
}

fn default_hooks_timeout() -> u64 {
    30
}

impl Default for HooksSection {
    fn default() -> Self {
        Self {
            enabled: default_hooks_enabled(),
            shell: default_hooks_shell(),
            timeout: default_hooks_timeout(),
        }
    }
}

impl ProjectConfig {
    /// Load config from the project
    pub fn load(project: &Project) -> Result<Self> {
        let config_path = project.config_path();

        if !config_path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&config_path)?;
        let config: ProjectConfig = toml::from_str(&content)?;
        Ok(config)
    }

    /// Save config to the project
    pub fn save(&self, project: &Project) -> Result<()> {
        let config_path = project.config_path();
        let content = toml::to_string_pretty(self)?;
        fs::write(&config_path, content)?;
        Ok(())
    }

    /// Set the active profile
    pub fn set_active_profile(&mut self, profile: Option<String>) {
        self.project.active_profile = profile;
    }

    /// Get the active profile
    pub fn active_profile(&self) -> Option<&str> {
        self.project.active_profile.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = ProjectConfig::default();
        assert!(config.settings.redact_by_default);
        assert_eq!(config.settings.redact_min_length, 8);
        assert!(config.settings.gitignore_plaintext);
        assert!(config.settings.envrc_integration);
        assert_eq!(config.project.active_profile, None);
    }

    #[test]
    fn test_save_and_load() {
        let temp = TempDir::new().unwrap();
        let project = Project::init(temp.path()).unwrap();

        let mut config = ProjectConfig::default();
        config.set_active_profile(Some("staging".to_string()));
        config.save(&project).unwrap();

        let loaded = ProjectConfig::load(&project).unwrap();
        assert_eq!(loaded.active_profile(), Some("staging"));
    }

    #[test]
    fn test_load_missing_config() {
        let temp = TempDir::new().unwrap();
        let project = Project::init(temp.path()).unwrap();

        let config = ProjectConfig::load(&project).unwrap();
        assert_eq!(config.active_profile(), None);
    }

    #[test]
    fn test_round_trip() {
        let temp = TempDir::new().unwrap();
        let project = Project::init(temp.path()).unwrap();

        let mut config = ProjectConfig::default();
        config.set_active_profile(Some("production".to_string()));
        config.settings.redact_min_length = 16;
        config.save(&project).unwrap();

        let loaded = ProjectConfig::load(&project).unwrap();
        assert_eq!(loaded.active_profile(), Some("production"));
        assert_eq!(loaded.settings.redact_min_length, 16);
    }
}
