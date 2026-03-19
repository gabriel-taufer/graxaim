use crate::errors::{GraxaimError, Result};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub const ENVSWITCH_DIR: &str = ".graxaim";
pub const CONFIG_FILE: &str = "config.toml";
pub const ENV_SYMLINK: &str = ".env";

pub struct Project {
    pub root: PathBuf,
}

impl Project {
    /// Find the project root by walking up the directory tree
    /// looking for .graxaim/ or .git/
    pub fn find() -> Result<Self> {
        let current_dir = env::current_dir().map_err(GraxaimError::Io)?;
        Self::find_from(&current_dir)
    }

    /// Find project root starting from a specific directory
    pub fn find_from(start: &Path) -> Result<Self> {
        let mut current = start.to_path_buf();

        loop {
            let graxaim_path = current.join(ENVSWITCH_DIR);
            if graxaim_path.exists() && graxaim_path.is_dir() {
                return Ok(Project { root: current });
            }

            let git_path = current.join(".git");
            if git_path.exists() {
                // Check if .graxaim exists
                let graxaim_path = current.join(ENVSWITCH_DIR);
                if graxaim_path.exists() && graxaim_path.is_dir() {
                    return Ok(Project { root: current });
                } else {
                    // Found git but no .graxaim
                    return Err(GraxaimError::ProjectNotInitialized);
                }
            }

            // Move up to parent directory
            match current.parent() {
                Some(parent) => current = parent.to_path_buf(),
                None => return Err(GraxaimError::ProjectRootNotFound),
            }
        }
    }

    /// Get the .graxaim directory path
    pub fn graxaim_dir(&self) -> PathBuf {
        self.root.join(ENVSWITCH_DIR)
    }

    /// Get the hooks directory path
    pub fn hooks_dir(&self) -> PathBuf {
        self.graxaim_dir().join("hooks")
    }

    /// Get the config file path
    pub fn config_path(&self) -> PathBuf {
        self.graxaim_dir().join(CONFIG_FILE)
    }

    /// Get the .env symlink path
    pub fn env_symlink_path(&self) -> PathBuf {
        self.root.join(ENV_SYMLINK)
    }

    /// Initialize a new project
    pub fn init(root: &Path) -> Result<Self> {
        let graxaim_dir = root.join(ENVSWITCH_DIR);

        // Create .graxaim directory
        if !graxaim_dir.exists() {
            fs::create_dir(&graxaim_dir).map_err(GraxaimError::Io)?;
        }

        let project = Project {
            root: root.to_path_buf(),
        };

        // Create hooks directory
        let hooks_dir = project.hooks_dir();
        if !hooks_dir.exists() {
            fs::create_dir(&hooks_dir).map_err(GraxaimError::Io)?;
        }

        Ok(project)
    }

    /// Check if project is initialized
    pub fn is_initialized(&self) -> bool {
        self.graxaim_dir().exists()
    }

    /// Discover existing .env.* files in the project root
    pub fn discover_profiles(&self) -> Result<Vec<String>> {
        let mut profiles = Vec::new();

        let entries = fs::read_dir(&self.root).map_err(GraxaimError::Io)?;

        for entry in entries {
            let entry = entry.map_err(GraxaimError::Io)?;
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
                // Extract profile name
                let profile_name = file_name.strip_prefix(".env.").unwrap().to_string();
                if !profile_name.is_empty() {
                    profiles.push(profile_name);
                }
            }
        }

        profiles.sort();
        Ok(profiles)
    }

    /// Add .env files to .gitignore
    pub fn update_gitignore(&self) -> Result<()> {
        let gitignore_path = self.root.join(".gitignore");

        let mut content = if gitignore_path.exists() {
            fs::read_to_string(&gitignore_path).map_err(GraxaimError::Io)?
        } else {
            String::new()
        };

        // Check if .env is already ignored
        let needs_env = !content.lines().any(|line| {
            let trimmed = line.trim();
            trimmed == ".env" || trimmed == "/.env"
        });

        let needs_env_star = !content.lines().any(|line| {
            let trimmed = line.trim();
            trimmed == ".env.*" || trimmed == "/.env.*"
        });

        if needs_env || needs_env_star {
            // Add graxaim section
            if !content.is_empty() && !content.ends_with('\n') {
                content.push('\n');
            }

            if !content.contains("# graxaim") {
                content.push_str("\n# graxaim\n");
            }

            if needs_env {
                content.push_str(".env\n");
            }
            if needs_env_star {
                content.push_str(".env.*\n");
            }

            fs::write(&gitignore_path, content).map_err(GraxaimError::Io)?;
        }

        Ok(())
    }

    /// Generate .envrc for direnv integration
    pub fn generate_envrc(&self) -> Result<()> {
        let envrc_path = self.root.join(".envrc");

        // Don't overwrite existing .envrc
        if envrc_path.exists() {
            return Ok(());
        }

        let content = "# Auto-generated by graxaim\n\
                       # Load .env file if it exists (managed by graxaim)\n\
                       dotenv_if_exists .env\n";

        fs::write(&envrc_path, content).map_err(GraxaimError::Io)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_find_project_with_graxaim() {
        let temp = TempDir::new().unwrap();
        let graxaim_dir = temp.path().join(ENVSWITCH_DIR);
        fs::create_dir(&graxaim_dir).unwrap();

        let project = Project::find_from(temp.path()).unwrap();
        assert_eq!(project.root, temp.path());
    }

    #[test]
    fn test_find_project_with_git_but_no_graxaim() {
        let temp = TempDir::new().unwrap();
        let git_dir = temp.path().join(".git");
        fs::create_dir(&git_dir).unwrap();

        let result = Project::find_from(temp.path());
        assert!(matches!(result, Err(GraxaimError::ProjectNotInitialized)));
    }

    #[test]
    fn test_find_project_walks_up() {
        let temp = TempDir::new().unwrap();
        let graxaim_dir = temp.path().join(ENVSWITCH_DIR);
        fs::create_dir(&graxaim_dir).unwrap();

        let subdir = temp.path().join("subdir").join("nested");
        fs::create_dir_all(&subdir).unwrap();

        let project = Project::find_from(&subdir).unwrap();
        assert_eq!(project.root, temp.path());
    }

    #[test]
    fn test_discover_profiles() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join(".env.local"), "KEY=value").unwrap();
        fs::write(temp.path().join(".env.staging"), "KEY=value").unwrap();
        fs::write(temp.path().join(".env.production"), "KEY=value").unwrap();
        fs::write(temp.path().join(".env"), "KEY=value").unwrap(); // should be ignored
        fs::write(temp.path().join("other.txt"), "content").unwrap(); // should be ignored

        let project = Project::init(temp.path()).unwrap();
        let profiles = project.discover_profiles().unwrap();

        assert_eq!(profiles.len(), 3);
        assert!(profiles.contains(&"local".to_string()));
        assert!(profiles.contains(&"staging".to_string()));
        assert!(profiles.contains(&"production".to_string()));
    }

    #[test]
    fn test_update_gitignore() {
        let temp = TempDir::new().unwrap();
        let project = Project::init(temp.path()).unwrap();

        project.update_gitignore().unwrap();

        let gitignore_path = temp.path().join(".gitignore");
        let content = fs::read_to_string(&gitignore_path).unwrap();

        assert!(content.contains(".env"));
        assert!(content.contains(".env.*"));
    }

    #[test]
    fn test_update_gitignore_existing() {
        let temp = TempDir::new().unwrap();
        let gitignore_path = temp.path().join(".gitignore");
        fs::write(&gitignore_path, "node_modules/\n").unwrap();

        let project = Project::init(temp.path()).unwrap();
        project.update_gitignore().unwrap();

        let content = fs::read_to_string(&gitignore_path).unwrap();
        assert!(content.contains("node_modules/"));
        assert!(content.contains(".env"));
    }

    #[test]
    fn test_generate_envrc() {
        let temp = TempDir::new().unwrap();
        let project = Project::init(temp.path()).unwrap();

        project.generate_envrc().unwrap();

        let envrc_path = temp.path().join(".envrc");
        assert!(envrc_path.exists());
        let content = fs::read_to_string(&envrc_path).unwrap();
        assert!(content.contains("dotenv_if_exists .env"));
    }
}
