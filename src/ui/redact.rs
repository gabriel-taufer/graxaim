use crate::core::config::ProjectConfig;

/// Redact a value for display (e.g., sk_test_12345678 -> sk_test_...5678)
pub fn redact_value(value: &str, min_length: usize) -> String {
    // Don't redact short values
    if value.len() < min_length {
        return value.to_string();
    }

    // Extract suffix (last 3-4 characters)
    let suffix_len = if value.len() > 12 { 4 } else { 3 };
    let suffix = if value.len() > suffix_len {
        &value[value.len() - suffix_len..]
    } else {
        ""
    };

    // Extract prefix: find the last underscore in the first half of the string
    // to preserve token-style prefixes (e.g., "sk_test_" from "sk_test_12345678")
    let half = value.len() / 2;
    let prefix = if let Some(last_us) = value[..half].rfind('_') {
        let candidate = &value[..last_us + 1];
        if candidate.len() <= 8 {
            candidate
        } else {
            &value[..2]
        }
    } else if value.len() >= 3 {
        &value[..2]
    } else {
        ""
    };

    if prefix.is_empty() && suffix.is_empty() {
        return "***".to_string();
    }

    format!("{}...{}", prefix, suffix)
}

/// Determine if a key suggests it contains sensitive data
#[allow(dead_code)]
pub fn is_sensitive_key(key: &str) -> bool {
    let key_lower = key.to_lowercase();

    let sensitive_patterns = [
        "key",
        "token",
        "secret",
        "password",
        "passwd",
        "pwd",
        "auth",
        "api",
        "credential",
        "private",
        "salt",
        "hash",
    ];

    sensitive_patterns
        .iter()
        .any(|pattern| key_lower.contains(pattern))
}

/// Redact a value based on config and key name
#[allow(dead_code)]
pub fn redact_if_needed(value: &str, key: &str, config: &ProjectConfig) -> String {
    if !config.settings.redact_by_default {
        return value.to_string();
    }

    if is_sensitive_key(key) {
        redact_value(value, config.settings.redact_min_length)
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::ProjectConfig;

    #[test]
    fn test_redact_value() {
        assert_eq!(redact_value("short", 8), "short");
        assert_eq!(redact_value("sk_test_12345678", 8), "sk_test_...5678");
        assert_eq!(
            redact_value("very_long_secret_key_value_here", 8),
            "ve...here"
        );
        assert_eq!(redact_value("12345678", 8), "12...678");
    }

    #[test]
    fn test_is_sensitive_key() {
        assert!(is_sensitive_key("API_KEY"));
        assert!(is_sensitive_key("SECRET_TOKEN"));
        assert!(is_sensitive_key("database_password"));
        assert!(is_sensitive_key("JWT_SECRET"));
        assert!(is_sensitive_key("PRIVATE_KEY"));

        assert!(!is_sensitive_key("DATABASE_URL"));
        assert!(!is_sensitive_key("PORT"));
        assert!(!is_sensitive_key("NODE_ENV"));
    }

    #[test]
    fn test_redact_if_needed() {
        let config = ProjectConfig::default();

        let redacted = redact_if_needed("sk_test_12345678", "API_KEY", &config);
        assert_eq!(redacted, "sk_test_...5678");

        let not_redacted = redact_if_needed("localhost:5432", "DATABASE_HOST", &config);
        assert_eq!(not_redacted, "localhost:5432");
    }

    #[test]
    fn test_redact_with_config_disabled() {
        let mut config = ProjectConfig::default();
        config.settings.redact_by_default = false;

        let not_redacted = redact_if_needed("sk_test_12345678", "API_KEY", &config);
        assert_eq!(not_redacted, "sk_test_12345678");
    }
}
