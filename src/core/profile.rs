use crate::core::config::ProjectConfig;
use crate::core::env_file::EnvFile;
use crate::core::hooks::HookRunner;
use crate::core::project::Project;
use crate::errors::{GraxaimError, Result};
use std::fs;
use std::os::unix::fs as unix_fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Profile {
    pub name: String,
    pub path: PathBuf,
    pub is_active: bool,
    pub is_sealed: bool,
    pub has_hook: bool,
}

impl Profile {
    /// Create a new profile reference
    pub fn new(name: String, path: PathBuf, is_active: bool, project: &Project) -> Self {
        let sealed_path = path.with_extension("sealed");
        let is_sealed = sealed_path.exists();

        // Check if profile has hooks
        let config = ProjectConfig::load(project).unwrap_or_default();
        let hooks_dir = project.hooks_dir();
        let hook_runner = HookRunner::new(
            hooks_dir,
            config.hooks.shell.clone(),
            config.hooks.timeout,
            config.hooks.enabled,
        );
        let has_hook = hook_runner.profile_has_hooks(&name);

        Self {
            name,
            path,
            is_active,
            is_sealed,
            has_hook,
        }
    }

    /// List all profiles in the project
    pub fn list_all(project: &Project) -> Result<Vec<Profile>> {
        let config = ProjectConfig::load(project)?;
        let active_profile = config.active_profile();

        let mut profiles = Vec::new();

        let entries = fs::read_dir(&project.root)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            let file_name = match path.file_name() {
                Some(name) => name.to_string_lossy(),
                None => continue,
            };

            // Match .env.* pattern (but not .env itself or .env.sealed)
            if file_name.starts_with(".env.") && !file_name.ends_with(".sealed") {
                let profile_name = file_name.strip_prefix(".env.").unwrap().to_string();
                if !profile_name.is_empty() {
                    let is_active = active_profile == Some(profile_name.as_str());
                    profiles.push(Profile::new(profile_name, path, is_active, project));
                }
            }
        }

        profiles.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(profiles)
    }

    /// Get a specific profile by name
    pub fn get(project: &Project, name: &str) -> Result<Profile> {
        Self::validate_name(name)?;

        let path = project.root.join(format!(".env.{}", name));
        if !path.exists() {
            return Err(GraxaimError::ProfileNotFound(name.to_string()));
        }

        let config = ProjectConfig::load(project)?;
        let is_active = config.active_profile() == Some(name);

        Ok(Profile::new(name.to_string(), path, is_active, project))
    }

    /// Get the currently active profile
    pub fn get_active(project: &Project) -> Result<Profile> {
        let config = ProjectConfig::load(project)?;

        let active_name = config
            .active_profile()
            .ok_or(GraxaimError::NoActiveProfile)?;

        Self::get(project, active_name)
    }

    /// Create a new profile
    pub fn create(project: &Project, name: &str) -> Result<Profile> {
        Self::validate_name(name)?;

        let path = project.root.join(format!(".env.{}", name));
        if path.exists() {
            return Err(GraxaimError::ProfileAlreadyExists(name.to_string()));
        }

        // Create empty env file
        let env_file = EnvFile::new();
        env_file.write_to_path(&path)?;

        Ok(Profile::new(name.to_string(), path, false, project))
    }

    /// Delete a profile
    pub fn delete(project: &Project, name: &str) -> Result<()> {
        let profile = Self::get(project, name)?;

        // Check if it's the active profile
        if profile.is_active {
            return Err(GraxaimError::CannotDeleteActiveProfile(name.to_string()));
        }

        // Delete the profile file
        fs::remove_file(&profile.path)?;

        // Also delete sealed version if it exists
        let sealed_path = profile.path.with_extension("sealed");
        if sealed_path.exists() {
            fs::remove_file(&sealed_path)?;
        }

        Ok(())
    }

    /// Rename a profile
    pub fn rename(project: &Project, old_name: &str, new_name: &str) -> Result<()> {
        Self::validate_name(new_name)?;

        let old_profile = Self::get(project, old_name)?;

        let new_path = project.root.join(format!(".env.{}", new_name));
        if new_path.exists() {
            return Err(GraxaimError::ProfileAlreadyExists(new_name.to_string()));
        }

        // Rename the file
        fs::rename(&old_profile.path, &new_path)?;

        // Also rename sealed version if it exists
        let old_sealed = old_profile.path.with_extension("sealed");
        if old_sealed.exists() {
            let new_sealed = new_path.with_extension("sealed");
            fs::rename(&old_sealed, &new_sealed)?;
        }

        // Update config if this was the active profile
        if old_profile.is_active {
            let mut config = ProjectConfig::load(project)?;
            config.set_active_profile(Some(new_name.to_string()));
            config.save(project)?;
        }

        Ok(())
    }

    /// Switch to a profile (update symlink and config)
    pub fn switch_to(project: &Project, name: &str) -> Result<Profile> {
        let profile = Self::get(project, name)?;

        // Update symlink
        Self::update_symlink(project, &profile.path)?;

        // Update config
        let mut config = ProjectConfig::load(project)?;
        config.set_active_profile(Some(name.to_string()));
        config.save(project)?;

        Ok(profile)
    }

    /// Update the .env symlink to point to a profile
    fn update_symlink(project: &Project, target: &Path) -> Result<()> {
        let symlink_path = project.env_symlink_path();

        // Remove existing symlink if it exists
        if symlink_path.exists() || symlink_path.read_link().is_ok() {
            fs::remove_file(&symlink_path)?;
        }

        // Create new symlink (relative path)
        let target_name = target.file_name().unwrap();
        unix_fs::symlink(target_name, &symlink_path)?;

        Ok(())
    }

    /// Read the current symlink target
    pub fn read_symlink_target(project: &Project) -> Result<Option<String>> {
        let symlink_path = project.env_symlink_path();

        if !symlink_path.exists() {
            return Ok(None);
        }

        let target = fs::read_link(&symlink_path)?;
        let target_name = target
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| GraxaimError::Custom("Invalid symlink target".to_string()))?;

        if let Some(profile_name) = target_name.strip_prefix(".env.") {
            Ok(Some(profile_name.to_string()))
        } else {
            Ok(None)
        }
    }

    /// Validate profile name (only allow alphanumeric, hyphens, underscores)
    pub fn validate_name(name: &str) -> Result<()> {
        if name.is_empty() {
            return Err(GraxaimError::InvalidProfileName(name.to_string()));
        }

        let is_valid = name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_');

        if !is_valid {
            return Err(GraxaimError::InvalidProfileName(name.to_string()));
        }

        Ok(())
    }

    /// Load the env file for this profile
    pub fn load_env_file(&self) -> Result<EnvFile> {
        EnvFile::from_path(&self.path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_validate_name() {
        assert!(Profile::validate_name("local").is_ok());
        assert!(Profile::validate_name("staging-01").is_ok());
        assert!(Profile::validate_name("prod_v2").is_ok());

        assert!(Profile::validate_name("").is_err());
        assert!(Profile::validate_name("prod/dev").is_err());
        assert!(Profile::validate_name("prod.dev").is_err());
        assert!(Profile::validate_name("prod dev").is_err());
    }

    #[test]
    fn test_create_and_list() {
        let temp = TempDir::new().unwrap();
        let project = Project::init(temp.path()).unwrap();

        Profile::create(&project, "local").unwrap();
        Profile::create(&project, "staging").unwrap();

        let profiles = Profile::list_all(&project).unwrap();
        assert_eq!(profiles.len(), 2);
        assert_eq!(profiles[0].name, "local");
        assert_eq!(profiles[1].name, "staging");
    }

    #[test]
    fn test_create_duplicate() {
        let temp = TempDir::new().unwrap();
        let project = Project::init(temp.path()).unwrap();

        Profile::create(&project, "local").unwrap();
        let result = Profile::create(&project, "local");
        assert!(matches!(result, Err(GraxaimError::ProfileAlreadyExists(_))));
    }

    #[test]
    fn test_get_profile() {
        let temp = TempDir::new().unwrap();
        let project = Project::init(temp.path()).unwrap();

        Profile::create(&project, "local").unwrap();
        let profile = Profile::get(&project, "local").unwrap();
        assert_eq!(profile.name, "local");
    }

    #[test]
    fn test_get_nonexistent() {
        let temp = TempDir::new().unwrap();
        let project = Project::init(temp.path()).unwrap();

        let result = Profile::get(&project, "nonexistent");
        assert!(matches!(result, Err(GraxaimError::ProfileNotFound(_))));
    }

    #[test]
    fn test_switch_profile() {
        let temp = TempDir::new().unwrap();
        let project = Project::init(temp.path()).unwrap();

        Profile::create(&project, "local").unwrap();
        Profile::create(&project, "staging").unwrap();

        Profile::switch_to(&project, "local").unwrap();
        let config = ProjectConfig::load(&project).unwrap();
        assert_eq!(config.active_profile(), Some("local"));

        let profiles = Profile::list_all(&project).unwrap();
        assert!(
            profiles
                .iter()
                .find(|p| p.name == "local")
                .unwrap()
                .is_active
        );
        assert!(
            !profiles
                .iter()
                .find(|p| p.name == "staging")
                .unwrap()
                .is_active
        );

        Profile::switch_to(&project, "staging").unwrap();
        let config = ProjectConfig::load(&project).unwrap();
        assert_eq!(config.active_profile(), Some("staging"));
    }

    #[test]
    fn test_delete_profile() {
        let temp = TempDir::new().unwrap();
        let project = Project::init(temp.path()).unwrap();

        Profile::create(&project, "local").unwrap();
        Profile::delete(&project, "local").unwrap();

        let result = Profile::get(&project, "local");
        assert!(matches!(result, Err(GraxaimError::ProfileNotFound(_))));
    }

    #[test]
    fn test_delete_active_profile() {
        let temp = TempDir::new().unwrap();
        let project = Project::init(temp.path()).unwrap();

        Profile::create(&project, "local").unwrap();
        Profile::switch_to(&project, "local").unwrap();

        let result = Profile::delete(&project, "local");
        assert!(matches!(
            result,
            Err(GraxaimError::CannotDeleteActiveProfile(_))
        ));
    }

    #[test]
    fn test_rename_profile() {
        let temp = TempDir::new().unwrap();
        let project = Project::init(temp.path()).unwrap();

        Profile::create(&project, "local").unwrap();
        Profile::rename(&project, "local", "development").unwrap();

        let result = Profile::get(&project, "local");
        assert!(matches!(result, Err(GraxaimError::ProfileNotFound(_))));

        let profile = Profile::get(&project, "development").unwrap();
        assert_eq!(profile.name, "development");
    }

    #[test]
    fn test_rename_active_profile() {
        let temp = TempDir::new().unwrap();
        let project = Project::init(temp.path()).unwrap();

        Profile::create(&project, "local").unwrap();
        Profile::switch_to(&project, "local").unwrap();
        Profile::rename(&project, "local", "dev").unwrap();

        let config = ProjectConfig::load(&project).unwrap();
        assert_eq!(config.active_profile(), Some("dev"));
    }
}
