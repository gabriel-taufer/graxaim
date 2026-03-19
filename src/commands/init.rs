use crate::core::config::ProjectConfig;
use crate::core::profile::Profile;
use crate::core::project::Project;
use crate::errors::Result;
use crate::ui::output;
use std::env;

pub fn execute(no_gitignore: bool, no_envrc: bool) -> Result<()> {
    let current_dir = env::current_dir()?;

    // Check if already initialized
    let graxaim_dir = current_dir.join(".graxaim");
    if graxaim_dir.exists() {
        output::info("Project already initialized");
        return Ok(());
    }

    // Initialize project
    let project = Project::init(&current_dir)?;

    // Discover existing profiles
    let discovered = project.discover_profiles()?;

    if discovered.is_empty() {
        output::info("Initialized graxaim");
        output::info("No existing .env.* files found");
        output::info("Create your first profile with: graxaim create <name>");
    } else {
        output::success("Initialized graxaim");
        println!();
        println!("Discovered {} profile(s):", discovered.len());
        for profile_name in &discovered {
            output::bullet(profile_name);
        }
        println!();

        // Set the first discovered profile as active
        let first_profile = &discovered[0];
        Profile::switch_to(&project, first_profile)?;
        output::success(&format!("Set '{}' as the active profile", first_profile));
    }

    // Update .gitignore unless disabled
    if !no_gitignore {
        project.update_gitignore()?;
        output::info("Updated .gitignore");
    }

    // Generate .envrc unless disabled
    if !no_envrc {
        project.generate_envrc()?;
        let envrc_path = project.root.join(".envrc");
        if envrc_path.exists() {
            output::info("Generated .envrc for direnv integration");
        }
    }

    // Save initial config
    let config = ProjectConfig::default();
    config.save(&project)?;

    println!();
    output::info("Next steps:");
    println!("  • Switch profiles: graxaim use <name>");
    println!("  • Create a new profile: graxaim create <name>");
    println!("  • List all profiles: graxaim list");

    Ok(())
}
