use crate::core::profile::Profile;
use crate::core::project::Project;
use crate::errors::Result;
use crate::ui::output;

pub fn execute() -> Result<()> {
    let project = Project::find()?;
    let profiles = Profile::list_all(&project)?;

    if profiles.is_empty() {
        output::info("No profiles found");
        output::info("Create your first profile with: graxaim create <name>");
        return Ok(());
    }

    println!("Profiles:");
    for profile in profiles {
        let formatted = output::format_profile_name(&profile.name, profile.is_active);
        let sealed_marker = if profile.is_sealed { " [sealed]" } else { "" };
        let hook_marker = if profile.has_hook { " 🪝" } else { "" };
        output::bullet(&format!("{}{}{}", formatted, sealed_marker, hook_marker));
    }

    Ok(())
}
