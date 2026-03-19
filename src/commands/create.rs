use crate::core::profile::Profile;
use crate::core::project::Project;
use crate::errors::Result;
use crate::ui::output;

pub fn execute(name: String, from: Option<String>) -> Result<()> {
    let project = Project::find()?;

    // Create the profile
    let profile = Profile::create(&project, &name)?;

    // If --from is specified, copy content from source profile
    if let Some(source_name) = from {
        let source_profile = Profile::get(&project, &source_name)?;
        let source_env = source_profile.load_env_file()?;
        source_env.write_to_path(&profile.path)?;

        output::success(&format!(
            "Created profile '{}' (copied from '{}')",
            name, source_name
        ));
    } else {
        output::success(&format!("Created profile '{}'", name));
    }

    output::info(&format!("Edit it with: graxaim edit {}", name));

    Ok(())
}
