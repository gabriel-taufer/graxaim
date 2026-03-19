use crate::core::auditor;
use crate::core::env_file::EnvFile;
use crate::core::profile::Profile;
use crate::core::project::Project;
use crate::errors::Result;
use crate::ui;
use owo_colors::OwoColorize;
use std::collections::{HashMap, HashSet};

/// `graxaim audit [--profile <name>]`
///
/// Scans source code for env var references and cross-references with profiles.
pub fn execute(profile: Option<String>) -> Result<()> {
    let project = Project::find()?;

    // Collect vars from all profiles (or a specific one)
    let all_profiles = Profile::list_all(&project)?;

    let profiles_to_scan: Vec<&Profile> = if let Some(ref name) = profile {
        all_profiles.iter().filter(|p| &p.name == name).collect()
    } else {
        all_profiles.iter().collect()
    };

    if profiles_to_scan.is_empty() {
        if let Some(name) = &profile {
            return Err(crate::errors::GraxaimError::ProfileNotFound(name.clone()));
        }
        ui::output::info("No profiles found — nothing to audit.");
        return Ok(());
    }

    // Build profile_vars: profile_name → set of var keys
    let mut profile_vars: HashMap<String, HashSet<String>> = HashMap::new();
    for p in &profiles_to_scan {
        let env = EnvFile::from_path(&p.path)?;
        let vars: HashSet<String> = env.entries.iter().map(|e| e.key.clone()).collect();
        profile_vars.insert(p.name.clone(), vars);
    }

    eprint!("Scanning source files...");
    let result = auditor::audit(&project.root, &profile_vars)?;
    // Clear the progress indicator
    eprintln!(" {} files scanned", result.files_scanned);
    println!();

    let missing_count = result.in_code_missing_from_profiles.len();
    let dead_count = result.in_profiles_not_in_code.len();

    // ── Referenced in code but MISSING from profiles ──────────────────────────
    if missing_count > 0 {
        println!(
            "  {}",
            "Referenced in code but MISSING from profiles:"
                .yellow()
                .bold()
        );
        for r in &result.in_code_missing_from_profiles {
            let location = format!(
                "found in: {}:{}",
                r.file
                    .strip_prefix(&project.root)
                    .unwrap_or(&r.file)
                    .display(),
                r.line
            );
            println!("    {:<32} {}", r.var_name.red().bold(), location.dimmed());
        }
        println!();
    } else {
        println!(
            "  {}",
            "✓ All code references are covered by profiles.".green()
        );
        println!();
    }

    // ── In profiles but NOT referenced in code ────────────────────────────────
    if dead_count > 0 {
        println!(
            "  {}",
            "In profiles but NOT referenced in code:".yellow().bold()
        );
        for r in &result.in_profiles_not_in_code {
            let profile_list = r.profiles.join(", ");
            println!(
                "    {:<32} {}",
                r.var_name.yellow(),
                format!("present in: {}", profile_list).dimmed()
            );
        }
        println!();
    } else {
        println!("  {}", "✓ No dead variables found in profiles.".green());
        println!();
    }

    // ── Summary ───────────────────────────────────────────────────────────────
    if missing_count == 0 && dead_count == 0 {
        ui::output::success("Audit clean — no issues found.");
    } else {
        println!(
            "  Summary: {} missing from profiles, {} potentially dead variable{}",
            missing_count.to_string().red().bold(),
            dead_count.to_string().yellow().bold(),
            if dead_count == 1 { "" } else { "s" }
        );
    }

    Ok(())
}
