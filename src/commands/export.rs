use crate::core::profile::Profile;
use crate::core::project::Project;
use crate::core::shell;
use crate::errors::Result;

pub fn execute(shell_name: String) -> Result<()> {
    let project = Project::find()?;
    let profile = Profile::get_active(&project)?;
    let env_file = profile.load_env_file()?;

    shell::print_exports(&env_file, &shell_name)
}
