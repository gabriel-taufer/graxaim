use crate::core::profile::Profile;
use crate::core::project::Project;
use crate::errors::Result;
use crate::ui::output;

pub fn execute(old_name: String, new_name: String) -> Result<()> {
    let project = Project::find()?;

    Profile::rename(&project, &old_name, &new_name)?;

    output::success(&format!("Renamed profile '{}' to '{}'", old_name, new_name));

    Ok(())
}
