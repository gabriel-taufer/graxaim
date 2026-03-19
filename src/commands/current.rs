use crate::core::profile::Profile;
use crate::core::project::Project;
use crate::errors::Result;
use crate::ui::output;

pub fn execute() -> Result<()> {
    let project = Project::find()?;

    match Profile::get_active(&project) {
        Ok(profile) => {
            println!("{}", profile.name);
            Ok(())
        }
        Err(_) => {
            output::warning("No active profile set");
            output::info("Switch to a profile with: graxaim use <name>");
            Ok(())
        }
    }
}
