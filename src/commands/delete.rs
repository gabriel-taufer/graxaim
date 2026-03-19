use crate::core::profile::Profile;
use crate::core::project::Project;
use crate::errors::Result;
use crate::ui::output;
use std::io::{self, Write};

pub fn execute(name: String, yes: bool) -> Result<()> {
    let project = Project::find()?;

    // Check if profile exists
    let _profile = Profile::get(&project, &name)?;

    // Confirm deletion unless --yes
    if !yes {
        print!("Delete profile '{}'? [y/N] ", name);
        io::stdout().flush()?;

        let mut response = String::new();
        io::stdin().read_line(&mut response)?;

        let confirmed = response.trim().eq_ignore_ascii_case("y")
            || response.trim().eq_ignore_ascii_case("yes");

        if !confirmed {
            output::info("Cancelled");
            return Ok(());
        }
    }

    // Delete the profile
    Profile::delete(&project, &name)?;

    output::success(&format!("Deleted profile '{}'", name));

    Ok(())
}
