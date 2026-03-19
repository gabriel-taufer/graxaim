use crate::core::profile::Profile;
use crate::core::project::Project;
use crate::errors::{GraxaimError, Result};
use crate::ui::output;
use std::env;
use std::process::Command;

pub fn execute(name: Option<String>) -> Result<()> {
    let project = Project::find()?;

    // Determine which profile to edit
    let profile = match name {
        Some(n) => Profile::get(&project, &n)?,
        None => Profile::get_active(&project)?,
    };

    // Get the editor from environment
    let editor = env::var("EDITOR")
        .or_else(|_| env::var("VISUAL"))
        .unwrap_or_else(|_| {
            // Default editors based on platform
            if cfg!(target_os = "windows") {
                "notepad".to_string()
            } else {
                "vi".to_string()
            }
        });

    // Open the editor
    let status = Command::new(&editor)
        .arg(&profile.path)
        .status()
        .map_err(|e| GraxaimError::EditorError(format!("Failed to launch {}: {}", editor, e)))?;

    if !status.success() {
        return Err(GraxaimError::EditorError(format!(
            "Editor exited with status: {}",
            status
        )));
    }

    output::success(&format!("Edited profile '{}'", profile.name));

    Ok(())
}
