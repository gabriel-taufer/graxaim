use crate::core::config::ProjectConfig;
use crate::core::hooks::HookRunner;
use crate::core::profile::Profile;
use crate::core::project::Project;
use crate::core::shell;
use crate::errors::Result;
use crate::ui::{output, picker};

pub fn execute(
    name: Option<String>,
    no_hooks: bool,
    strict_hooks: bool,
    no_export: bool,
    shell_name: String,
) -> Result<()> {
    let project = Project::find()?;
    let config = ProjectConfig::load(&project)?;

    let previous_profile = config.active_profile().map(|s| s.to_string());

    let profile_name = match name {
        Some(n) => n,
        None => {
            let profiles = Profile::list_all(&project)?;
            picker::pick_profile(&profiles)?
        }
    };

    let profile = Profile::get(&project, &profile_name)?;

    if !no_hooks && config.hooks.enabled {
        let hooks_dir = project.hooks_dir();
        let mut hook_runner = HookRunner::new(
            hooks_dir,
            config.hooks.shell.clone(),
            config.hooks.timeout,
            true,
        );
        hook_runner.redirect_stdout_to_stderr = !no_export;

        let new_env = profile.load_env_file()?;

        hook_runner.run_switch_hooks(
            previous_profile.as_deref(),
            &profile_name,
            &new_env,
            &project.root,
            strict_hooks,
        )?;
    }

    Profile::switch_to(&project, &profile_name)?;

    output::success_err(&format!("Switched to profile '{}'", profile.name));
    output::info_err(&format!(".env now points to .env.{}", profile.name));

    if !no_export {
        let env_file = profile.load_env_file()?;
        shell::print_exports(&env_file, &shell_name)?;
    }

    Ok(())
}
