use crate::core::env_file::EnvFile;
use crate::errors::{GraxaimError, Result};
use std::collections::HashMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum HookPhase {
    Leave(String),      // _leave_{profile}.sh — leaving a profile
    GlobalPre,          // _pre.sh
    ProfilePre(String), // {profile}.pre.sh
    // --- symlink switch happens here ---
    ProfilePost(String), // {profile}.post.sh
    GlobalPost,          // _post.sh
}

impl HookPhase {
    fn script_name(&self) -> String {
        match self {
            HookPhase::Leave(profile) => format!("_leave_{}.sh", profile),
            HookPhase::GlobalPre => "_pre.sh".to_string(),
            HookPhase::ProfilePre(profile) => format!("{}.pre.sh", profile),
            HookPhase::ProfilePost(profile) => format!("{}.post.sh", profile),
            HookPhase::GlobalPost => "_post.sh".to_string(),
        }
    }

    #[allow(dead_code)]
    fn description(&self) -> String {
        match self {
            HookPhase::Leave(profile) => format!("Leaving profile '{}'", profile),
            HookPhase::GlobalPre => "Global pre-switch".to_string(),
            HookPhase::ProfilePre(profile) => format!("Pre-switch for '{}'", profile),
            HookPhase::ProfilePost(profile) => format!("Post-switch for '{}'", profile),
            HookPhase::GlobalPost => "Global post-switch".to_string(),
        }
    }
}

pub struct HookRunner {
    pub hooks_dir: PathBuf,
    pub shell: String,
    #[allow(dead_code)]
    pub timeout: Duration,
    pub enabled: bool,
    pub redirect_stdout_to_stderr: bool,
}

impl HookRunner {
    pub fn new(hooks_dir: PathBuf, shell: String, timeout_secs: u64, enabled: bool) -> Self {
        Self {
            hooks_dir,
            shell,
            timeout: Duration::from_secs(timeout_secs),
            enabled,
            redirect_stdout_to_stderr: false,
        }
    }

    /// Check if a hook script exists and is executable
    pub fn hook_exists(&self, phase: &HookPhase) -> bool {
        let script_path = self.hooks_dir.join(phase.script_name());

        if !script_path.exists() {
            return false;
        }

        // Check if executable (Unix only)
        #[cfg(unix)]
        {
            if let Ok(metadata) = fs::metadata(&script_path) {
                let permissions = metadata.permissions();
                return permissions.mode() & 0o111 != 0; // Check any execute bit
            }
        }

        #[cfg(not(unix))]
        {
            // On Windows, just check if file exists
            return true;
        }

        false
    }

    /// Run all hooks for a profile switch
    pub fn run_switch_hooks(
        &self,
        previous_profile: Option<&str>,
        new_profile: &str,
        new_env: &EnvFile,
        project_root: &Path,
        strict: bool,
    ) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        // Build the phases in order
        let mut phases = Vec::new();

        // 1. Leave hook (if switching from another profile)
        if let Some(prev) = previous_profile {
            phases.push(HookPhase::Leave(prev.to_string()));
        }

        // 2. Global pre
        phases.push(HookPhase::GlobalPre);

        // 3. Profile-specific pre
        phases.push(HookPhase::ProfilePre(new_profile.to_string()));

        // --- symlink switch happens in between ---

        // 4. Profile-specific post
        phases.push(HookPhase::ProfilePost(new_profile.to_string()));

        // 5. Global post
        phases.push(HookPhase::GlobalPost);

        // Execute each phase
        for phase in phases {
            if self.hook_exists(&phase) {
                self.execute_hook(
                    &phase,
                    new_profile,
                    previous_profile,
                    new_env,
                    project_root,
                    strict,
                )?;
            }
        }

        Ok(())
    }

    /// Execute a single hook script
    fn execute_hook(
        &self,
        phase: &HookPhase,
        new_profile: &str,
        previous_profile: Option<&str>,
        new_env: &EnvFile,
        project_root: &Path,
        strict: bool,
    ) -> Result<()> {
        let script_path = self.hooks_dir.join(phase.script_name());

        // Build environment variables
        let mut env_vars: HashMap<String, String> = HashMap::new();

        // Add all variables from the new profile
        for entry in &new_env.entries {
            env_vars.insert(entry.key.clone(), entry.value.clone());
        }

        // Add graxaim-specific variables
        env_vars.insert("GRAXAIM_PROFILE".to_string(), new_profile.to_string());
        env_vars.insert(
            "GRAXAIM_PREVIOUS_PROFILE".to_string(),
            previous_profile.unwrap_or("").to_string(),
        );
        env_vars.insert(
            "GRAXAIM_PROJECT_ROOT".to_string(),
            project_root.to_string_lossy().to_string(),
        );

        let stdout_cfg = if self.redirect_stdout_to_stderr {
            Stdio::piped()
        } else {
            Stdio::inherit()
        };

        let output = Command::new(&self.shell)
            .arg(&script_path)
            .envs(&env_vars)
            .current_dir(project_root)
            .stdin(Stdio::null())
            .stdout(stdout_cfg)
            .stderr(Stdio::inherit())
            .output();

        match output {
            Ok(output) => {
                if self.redirect_stdout_to_stderr && !output.stdout.is_empty() {
                    use std::io::Write;
                    std::io::stderr().write_all(&output.stdout).ok();
                }

                if !output.status.success() {
                    let exit_code = output.status.code().unwrap_or(-1);
                    let error_msg = format!(
                        "Hook '{}' failed with exit code {}",
                        phase.script_name(),
                        exit_code
                    );

                    if strict {
                        return Err(GraxaimError::Custom(error_msg));
                    } else {
                        eprintln!("⚠ Warning: {}", error_msg);
                    }
                }
            }
            Err(e) => {
                let error_msg = format!("Failed to execute hook '{}': {}", phase.script_name(), e);
                if strict {
                    return Err(GraxaimError::Custom(error_msg));
                } else {
                    eprintln!("⚠ Warning: {}", error_msg);
                }
            }
        }

        Ok(())
    }

    /// Check if a profile has any hooks
    pub fn profile_has_hooks(&self, profile_name: &str) -> bool {
        let pre_hook = HookPhase::ProfilePre(profile_name.to_string());
        let post_hook = HookPhase::ProfilePost(profile_name.to_string());
        let leave_hook = HookPhase::Leave(profile_name.to_string());

        self.hook_exists(&pre_hook) || self.hook_exists(&post_hook) || self.hook_exists(&leave_hook)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_hook_phase_names() {
        assert_eq!(HookPhase::GlobalPre.script_name(), "_pre.sh");
        assert_eq!(HookPhase::GlobalPost.script_name(), "_post.sh");
        assert_eq!(
            HookPhase::ProfilePre("local".to_string()).script_name(),
            "local.pre.sh"
        );
        assert_eq!(
            HookPhase::ProfilePost("staging".to_string()).script_name(),
            "staging.post.sh"
        );
        assert_eq!(
            HookPhase::Leave("production".to_string()).script_name(),
            "_leave_production.sh"
        );
    }

    #[test]
    fn test_hook_exists() {
        let temp = TempDir::new().unwrap();
        let hooks_dir = temp.path().join("hooks");
        fs::create_dir(&hooks_dir).unwrap();

        let runner = HookRunner::new(hooks_dir.clone(), "/bin/sh".to_string(), 30, true);

        // Initially no hooks exist
        assert!(!runner.hook_exists(&HookPhase::GlobalPre));

        // Create a hook file (but not executable)
        let hook_path = hooks_dir.join("_pre.sh");
        fs::write(&hook_path, "#!/bin/sh\necho 'test'\n").unwrap();

        // Still not executable
        #[cfg(unix)]
        assert!(!runner.hook_exists(&HookPhase::GlobalPre));

        // Make it executable
        #[cfg(unix)]
        {
            let mut perms = fs::metadata(&hook_path).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&hook_path, perms).unwrap();

            // Now it should exist
            assert!(runner.hook_exists(&HookPhase::GlobalPre));
        }
    }

    #[test]
    fn test_profile_has_hooks() {
        let temp = TempDir::new().unwrap();
        let hooks_dir = temp.path().join("hooks");
        fs::create_dir(&hooks_dir).unwrap();

        let runner = HookRunner::new(hooks_dir.clone(), "/bin/sh".to_string(), 30, true);

        // No hooks initially
        assert!(!runner.profile_has_hooks("staging"));

        // Create a post hook
        let hook_path = hooks_dir.join("staging.post.sh");
        fs::write(&hook_path, "#!/bin/sh\necho 'staging post'\n").unwrap();

        #[cfg(unix)]
        {
            let mut perms = fs::metadata(&hook_path).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&hook_path, perms).unwrap();

            // Now staging has hooks
            assert!(runner.profile_has_hooks("staging"));
            assert!(!runner.profile_has_hooks("production"));
        }
    }
}
