use atty::Stream;
use owo_colors::OwoColorize;

/// Check if we should use colors on stdout
pub fn should_use_colors() -> bool {
    atty::is(Stream::Stdout)
}

/// Check if we should use colors on stderr
pub fn should_use_colors_stderr() -> bool {
    atty::is(Stream::Stderr)
}

/// Print a success message to stdout
pub fn success(message: &str) {
    if should_use_colors() {
        println!("{} {}", "✓".green().bold(), message);
    } else {
        println!("✓ {}", message);
    }
}

/// Print a success message to stderr
pub fn success_err(message: &str) {
    if should_use_colors_stderr() {
        eprintln!("{} {}", "✓".green().bold(), message);
    } else {
        eprintln!("✓ {}", message);
    }
}

/// Print an error message
pub fn error(message: &str) {
    if should_use_colors_stderr() {
        eprintln!("{} {}", "✗".red().bold(), message);
    } else {
        eprintln!("✗ {}", message);
    }
}

/// Print an info message to stdout
pub fn info(message: &str) {
    if should_use_colors() {
        println!("{} {}", "ℹ".blue().bold(), message);
    } else {
        println!("ℹ {}", message);
    }
}

/// Print an info message to stderr
pub fn info_err(message: &str) {
    if should_use_colors_stderr() {
        eprintln!("{} {}", "ℹ".blue().bold(), message);
    } else {
        eprintln!("ℹ {}", message);
    }
}

/// Print a warning message
pub fn warning(message: &str) {
    if should_use_colors() {
        println!("{} {}", "⚠".yellow().bold(), message);
    } else {
        println!("⚠ {}", message);
    }
}

/// Format a profile name with highlighting
pub fn format_profile_name(name: &str, is_active: bool) -> String {
    if !should_use_colors() {
        if is_active {
            return format!("{} (active)", name);
        } else {
            return name.to_string();
        }
    }

    if is_active {
        format!("{} {}", name.cyan().bold(), "(active)".green())
    } else {
        name.to_string()
    }
}

/// Format a key-value pair
#[allow(dead_code)]
pub fn format_key_value(key: &str, value: &str) -> String {
    if should_use_colors() {
        format!("{}={}", key.yellow(), value)
    } else {
        format!("{}={}", key, value)
    }
}

/// Format a section header
#[allow(dead_code)]
pub fn format_section_header(title: &str) -> String {
    if should_use_colors() {
        title.bold().underline().to_string()
    } else {
        format!("=== {} ===", title)
    }
}

/// Print a list item
pub fn list_item(marker: &str, content: &str) {
    if should_use_colors() {
        println!("  {} {}", marker.cyan(), content);
    } else {
        println!("  {} {}", marker, content);
    }
}

/// Print a bulleted list item
pub fn bullet(content: &str) {
    list_item("•", content);
}

/// Print a numbered list item
#[allow(dead_code)]
pub fn numbered(number: usize, content: &str) {
    if should_use_colors() {
        println!("  {} {}", format!("{}.", number).cyan(), content);
    } else {
        println!("  {}. {}", number, content);
    }
}

/// Get a dimmed style for less important text
#[allow(dead_code)]
pub fn dimmed_text(text: &str) -> String {
    if should_use_colors() {
        text.dimmed().to_string()
    } else {
        text.to_string()
    }
}

/// Format a file path
#[allow(dead_code)]
pub fn format_path(path: &str) -> String {
    if should_use_colors() {
        path.cyan().to_string()
    } else {
        path.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_profile_name() {
        // These tests just ensure the functions don't panic
        let active = format_profile_name("local", true);
        assert!(active.contains("local"));

        let inactive = format_profile_name("staging", false);
        assert!(inactive.contains("staging"));
    }

    #[test]
    fn test_format_key_value() {
        let formatted = format_key_value("KEY", "value");
        assert!(formatted.contains("KEY"));
        assert!(formatted.contains("value"));
        assert!(formatted.contains("="));
    }
}
