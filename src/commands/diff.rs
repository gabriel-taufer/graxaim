use crate::core::config::ProjectConfig;
use crate::core::differ;
use crate::core::env_file::EnvFile;
use crate::core::project::Project;
use crate::errors::{GraxaimError, Result};
use crate::ui::output::should_use_colors;
use crate::ui::redact::redact_value;
use owo_colors::OwoColorize;

pub fn execute(
    profile_a: String,
    profile_b: Option<String>,
    redact: bool,
    show_same: bool,
) -> Result<()> {
    let project = Project::find()?;

    // If profile_b is None, diff profile_a against the active profile
    let name_b = match profile_b {
        Some(name) => name,
        None => {
            let config = ProjectConfig::load(&project)?;
            config
                .active_profile()
                .ok_or_else(|| {
                    GraxaimError::Custom("No active profile to diff against".to_string())
                })?
                .to_string()
        }
    };

    // Load both profiles
    let path_a = project.root.join(format!(".env.{}", profile_a));
    let path_b = project.root.join(format!(".env.{}", name_b));

    if !path_a.exists() {
        return Err(GraxaimError::ProfileNotFound(profile_a));
    }
    if !path_b.exists() {
        return Err(GraxaimError::ProfileNotFound(name_b));
    }

    let env_a = EnvFile::from_path(&path_a)?;
    let env_b = EnvFile::from_path(&path_b)?;

    // Compute diff
    let result = differ::diff_env_files(&env_a, &env_b);

    let colors = should_use_colors();

    // Header
    if colors {
        println!(
            "\nComparing {} {} {}",
            profile_a.cyan().bold(),
            "↔".dimmed(),
            name_b.cyan().bold()
        );
    } else {
        println!("\nComparing {} <-> {}", profile_a, name_b);
    }

    if result.is_empty() && result.same.is_empty() {
        println!("\n  Both profiles are empty.");
        return Ok(());
    }

    if result.is_empty() {
        println!("\n  Profiles are identical ({} keys).", result.same.len());
        if show_same {
            print_same_section(&result.same, redact, colors);
        }
        return Ok(());
    }

    // Only in A
    if !result.only_in_a.is_empty() {
        println!();
        if colors {
            println!("  {}:", format!("Only in {}", profile_a).green().bold());
        } else {
            println!("  Only in {}:", profile_a);
        }
        for (key, value) in &result.only_in_a {
            let display_value = maybe_redact(value, redact);
            if colors {
                println!(
                    "    {} {} = {}",
                    "+".green(),
                    key.green(),
                    display_value.green()
                );
            } else {
                println!("    + {} = {}", key, display_value);
            }
        }
    }

    // Only in B
    if !result.only_in_b.is_empty() {
        println!();
        if colors {
            println!("  {}:", format!("Only in {}", name_b).red().bold());
        } else {
            println!("  Only in {}:", name_b);
        }
        for (key, value) in &result.only_in_b {
            let display_value = maybe_redact(value, redact);
            if colors {
                println!("    {} {} = {}", "-".red(), key.red(), display_value.red());
            } else {
                println!("    - {} = {}", key, display_value);
            }
        }
    }

    // Different values
    if !result.different.is_empty() {
        println!();
        if colors {
            println!("  {}:", "Different values".yellow().bold());
        } else {
            println!("  Different values:");
        }
        for (key, val_a, val_b) in &result.different {
            let display_a = maybe_redact(val_a, redact);
            let display_b = maybe_redact(val_b, redact);
            if colors {
                println!("    {} {}", "~".yellow(), key.yellow());
                println!("        {}: {}", profile_a.dimmed(), display_a);
                println!("        {}: {}", name_b.dimmed(), display_b);
            } else {
                println!("    ~ {}", key);
                println!("        {}: {}", profile_a, display_a);
                println!("        {}: {}", name_b, display_b);
            }
        }
    }

    // Same (only if requested)
    if show_same {
        print_same_section(&result.same, redact, colors);
    }

    println!();
    Ok(())
}

fn print_same_section(same: &[(String, String)], redact: bool, colors: bool) {
    if same.is_empty() {
        return;
    }

    println!();
    if colors {
        println!("  {}:", format!("Identical ({} keys)", same.len()).dimmed());
    } else {
        println!("  Identical ({} keys):", same.len());
    }
    for (key, value) in same {
        let display_value = maybe_redact(value, redact);
        if colors {
            println!(
                "    {} {} = {}",
                "=".dimmed(),
                key.dimmed(),
                display_value.dimmed()
            );
        } else {
            println!("    = {} = {}", key, display_value);
        }
    }
}

fn maybe_redact(value: &str, redact: bool) -> String {
    if redact {
        redact_value(value, 8)
    } else {
        value.to_string()
    }
}
