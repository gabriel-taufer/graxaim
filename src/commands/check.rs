use crate::core::config::ProjectConfig;
use crate::core::env_file::EnvFile;
use crate::core::profile::Profile;
use crate::core::project::Project;
use crate::core::schema::{Schema, ValidationError, ValidationResult};
use crate::errors::{GraxaimError, Result};
use crate::ui;

pub fn execute(name: Option<String>, all: bool) -> Result<()> {
    let project = Project::find()?;
    let schema_path = project.root.join(".graxaim/schema.toml");

    if !schema_path.exists() {
        return Err(GraxaimError::Custom(
            "No schema found. Run 'graxaim schema init' to create one.".to_string(),
        ));
    }

    let schema = Schema::load(&schema_path)?;

    if all {
        execute_all(&project, &schema)?;
    } else {
        let profile_name = match name {
            Some(n) => n,
            None => {
                let config = ProjectConfig::load(&project)?;
                config
                    .active_profile()
                    .ok_or(GraxaimError::NoActiveProfile)?
                    .to_string()
            }
        };
        execute_single(&project, &schema, &profile_name)?;
    }

    Ok(())
}

fn execute_single(project: &Project, schema: &Schema, profile_name: &str) -> Result<()> {
    let profile = Profile::get(project, profile_name)?;
    let env = EnvFile::from_path(&profile.path)?;

    println!("Validating {} against schema...\n", profile_name);

    let result = schema.validate(&env);
    print_validation_result(&result);

    if result.has_errors() {
        Err(GraxaimError::Custom(format!(
            "Validation failed with {} error(s)",
            result.errors.len()
        )))
    } else {
        Ok(())
    }
}

fn execute_all(project: &Project, schema: &Schema) -> Result<()> {
    let profiles = Profile::list_all(project)?;

    if profiles.is_empty() {
        return Err(GraxaimError::NoProfiles);
    }

    let mut any_failed = false;

    for profile in &profiles {
        let env = EnvFile::from_path(&profile.path)?;

        println!("Validating {} against schema...\n", profile.name);

        let result = schema.validate(&env);
        print_validation_result(&result);

        if result.has_errors() {
            any_failed = true;
        }

        println!();
    }

    if any_failed {
        Err(GraxaimError::Custom(
            "One or more profiles failed validation".to_string(),
        ))
    } else {
        ui::output::success("All profiles passed validation");
        Ok(())
    }
}

fn print_validation_result(result: &ValidationResult) {
    for error in &result.errors {
        match error {
            ValidationError::Missing { .. } => {
                println!("  ✗ {}", error);
            }
            ValidationError::TypeError { .. } => {
                println!("  ✗ {}", error);
            }
            ValidationError::ConstraintViolation { .. } => {
                println!("  ✗ {}", error);
            }
            ValidationError::Unknown { .. } => {
                // Unknowns go in warnings, not errors
            }
        }
    }

    for warning in &result.warnings {
        println!("  ⚠ {}", warning);
    }

    if result.passed > 0 {
        println!(
            "  ✓ {} variable{} passed validation",
            result.passed,
            if result.passed == 1 { "" } else { "s" }
        );
    }

    let error_count = result.errors.len();
    let warning_count = result.warnings.len();
    println!(
        "\n  Result: {} error{}, {} warning{}",
        error_count,
        if error_count == 1 { "" } else { "s" },
        warning_count,
        if warning_count == 1 { "" } else { "s" },
    );
}
