use crate::core::config::ProjectConfig;
use crate::core::hooks::HookRunner;
use crate::core::profile::Profile;
use crate::core::project::Project;
use crate::errors::Result;
use crate::ui::{output, picker};

pub fn execute(name: Option<String>, no_hooks: bool, strict_hooks: bool) -> Result<()> {
    let project = Project::find()?;
    let config = ProjectConfig::load(&project)?;

    // Get previous profile name
    let previous_profile = config.active_profile().map(|s| s.to_string());

    let profile_name = match name {
        Some(n) => n,
        None => {
            // Open interactive picker
            let profiles = Profile::list_all(&project)?;
            picker::pick_profile(&profiles)?
        }
    };

    // Get the target profile
    let profile = Profile::get(&project, &profile_name)?;

    // Run pre-switch hooks if enabled
    if !no_hooks && config.hooks.enabled {
        let hooks_dir = project.hooks_dir();
        let hook_runner = HookRunner::new(
            hooks_dir,
            config.hooks.shell.clone(),
            config.hooks.timeout,
            true, // enabled
        );

        // Load the new profile's env vars for hooks
        let new_env = profile.load_env_file()?;

        // Run hooks (includes Leave, GlobalPre, ProfilePre phases)
        // Note: ProfilePost and GlobalPost run after symlink switch (inside run_switch_hooks)
        hook_runner.run_switch_hooks(
            previous_profile.as_deref(),
            &profile_name,
            &new_env,
            &project.root,
            strict_hooks,
        )?;
    }

    // Actually switch the symlink
    Profile::switch_to(&project, &profile_name)?;

    output::success(&format!("Switched to profile '{}'", profile.name));
    output::info(&format!(".env now points to .env.{}", profile.name));

    Ok(())
}
