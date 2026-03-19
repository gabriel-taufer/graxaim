use crate::core::profile::Profile;
use crate::errors::{GraxaimError, Result};
use skim::prelude::*;
use std::io::Cursor;

/// Show an interactive fuzzy picker for selecting a profile
pub fn pick_profile(profiles: &[Profile]) -> Result<String> {
    if profiles.is_empty() {
        return Err(GraxaimError::NoProfiles);
    }

    // If only one profile, return it
    if profiles.len() == 1 {
        return Ok(profiles[0].name.clone());
    }

    // Build the input string for skim
    let mut items = Vec::new();
    for profile in profiles {
        let marker = if profile.is_active { " (active)" } else { "" };
        items.push(format!("{}{}", profile.name, marker));
    }
    let input = items.join("\n");

    // Configure skim options
    let options = SkimOptionsBuilder::default()
        .height(Some("40%"))
        .multi(false)
        .prompt(Some("Select profile: "))
        .build()
        .map_err(|e| GraxaimError::Custom(format!("Picker error: {}", e)))?;

    // Run skim
    let item_reader = SkimItemReader::default();
    let items = item_reader.of_bufread(Cursor::new(input));

    let output = Skim::run_with(&options, Some(items)).ok_or(GraxaimError::Cancelled)?;

    // Handle user cancellation
    if output.is_abort {
        return Err(GraxaimError::Cancelled);
    }

    // Extract selected item
    let selected = output
        .selected_items
        .first()
        .ok_or(GraxaimError::Cancelled)?;

    let selected_text = selected.output().to_string();

    // Remove the " (active)" suffix if present
    let profile_name = selected_text
        .split(" (active)")
        .next()
        .unwrap_or(&selected_text)
        .to_string();

    Ok(profile_name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_pick_profile_empty() {
        let profiles = vec![];
        let result = pick_profile(&profiles);
        assert!(matches!(result, Err(GraxaimError::NoProfiles)));
    }

    #[test]
    fn test_pick_profile_single() {
        use crate::core::project::Project;
        use tempfile::TempDir;

        let temp = TempDir::new().unwrap();
        let project = Project::init(temp.path()).unwrap();

        let profiles = vec![Profile::new(
            "local".to_string(),
            PathBuf::from(".env.local"),
            false,
            &project,
        )];

        let result = pick_profile(&profiles).unwrap();
        assert_eq!(result, "local");
    }

    // Interactive tests would require mocking stdin/stdout
    // which is complex with skim, so we skip them here
}
