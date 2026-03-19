use crate::core::env_file::EnvFile;
use crate::errors::{GraxaimError, Result};
use lazy_static::lazy_static;
use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt;
use std::path::Path;

lazy_static! {
    /// Pre-compiled regex for URL validation
    static ref URL_PATTERN: Regex =
        Regex::new(r"^[a-zA-Z][a-zA-Z0-9+.-]*://[^\s/$.?#].[^\s]*$").unwrap();

    /// Pre-compiled regex for email validation (RFC 5322-inspired)
    static ref EMAIL_PATTERN: Regex =
        Regex::new(r"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$").unwrap();
}

// ---------------------------------------------------------------------------
// Data structures
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum VarType {
    String {
        #[serde(default)]
        min_length: Option<usize>,
        #[serde(default)]
        max_length: Option<usize>,
        #[serde(default)]
        pattern: Option<String>,
    },
    Integer {
        #[serde(default)]
        min: Option<i64>,
        #[serde(default)]
        max: Option<i64>,
    },
    Port,
    Boolean,
    Url {
        #[serde(default)]
        schemes: Option<Vec<String>>,
    },
    Email,
    Enum {
        values: Vec<String>,
    },
    Path {
        #[serde(default)]
        must_exist: bool,
    },
}

impl fmt::Display for VarType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VarType::String { .. } => write!(f, "string"),
            VarType::Integer { .. } => write!(f, "integer"),
            VarType::Port => write!(f, "port"),
            VarType::Boolean => write!(f, "boolean"),
            VarType::Url { .. } => write!(f, "url"),
            VarType::Email => write!(f, "email"),
            VarType::Enum { .. } => write!(f, "enum"),
            VarType::Path { .. } => write!(f, "path"),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct VarSchema {
    #[serde(flatten)]
    pub var_type: VarType,
    #[serde(default = "default_true")]
    pub required: bool,
    #[serde(default)]
    pub sensitive: bool,
    #[serde(default)]
    pub default: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub example: Option<String>,
    #[serde(default)]
    pub depends_on: Option<String>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize)]
pub struct Schema {
    #[serde(default)]
    pub vars: HashMap<String, VarSchema>,
    /// Pre-compiled regex objects, keyed by var name.
    /// Populated by [`Schema::load()`]; skipped during deserialization.
    /// Tests that construct a `Schema` directly via [`toml::from_str`] will
    /// have an empty map and fall back to on-the-fly compilation.
    #[serde(skip, default)]
    compiled_patterns: HashMap<String, Regex>,
}

// ---------------------------------------------------------------------------
// Validation errors & result
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum ValidationError {
    Missing {
        key: String,
        schema: VarSchema,
    },
    TypeError {
        key: String,
        value: String,
        expected: String,
        got: String,
    },
    ConstraintViolation {
        key: String,
        value: String,
        constraint: String,
    },
    Unknown {
        key: String,
    },
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::Missing { key, schema } => {
                let type_name = schema.var_type.to_string();
                let mut details = Vec::new();
                if schema.required {
                    details.push("required".to_string());
                }
                details.push(type_name);
                // Add constraint hints
                if let VarType::String {
                    min_length: Some(min),
                    ..
                } = &schema.var_type
                {
                    details.push(format!("min_length={}", min));
                }
                write!(f, "MISSING    {} ({})", key, details.join(", "))
            }
            ValidationError::TypeError {
                key,
                value,
                expected,
                got,
            } => {
                write!(
                    f,
                    "TYPE       {} = \"{}\" (expected: {}, got: {})",
                    key, value, expected, got
                )
            }
            ValidationError::ConstraintViolation {
                key,
                value,
                constraint,
            } => {
                write!(f, "CONSTRAINT {} = \"{}\" ({})", key, value, constraint)
            }
            ValidationError::Unknown { key } => {
                write!(f, "UNKNOWN    {} (not defined in schema)", key)
            }
        }
    }
}

#[derive(Debug)]
pub struct ValidationResult {
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationError>,
    pub passed: usize,
}

impl ValidationResult {
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Helper: check if a value is truthy
// ---------------------------------------------------------------------------

fn is_truthy(value: &str) -> bool {
    matches!(value.to_lowercase().as_str(), "true" | "1" | "yes")
}

// ---------------------------------------------------------------------------
// Schema implementation
// ---------------------------------------------------------------------------

impl Schema {
    /// Load a schema from a TOML file.
    ///
    /// Validates and pre-compiles all regex patterns eagerly so that
    /// subsequent calls to [`Schema::validate`] pay zero compilation cost.
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path).map_err(GraxaimError::Io)?;
        let mut schema: Schema = toml::from_str(&content)?;
        schema.compile_patterns()?;
        Ok(schema)
    }

    /// Validate and cache compiled [`Regex`] objects for every `String` var
    /// that carries a `pattern` constraint.  Reports the first invalid pattern
    /// as an error.
    fn compile_patterns(&mut self) -> Result<()> {
        for (key, var) in &self.vars {
            if let VarType::String {
                pattern: Some(pat), ..
            } = &var.var_type
            {
                let re = Regex::new(pat).map_err(|e| {
                    GraxaimError::Custom(format!("Invalid regex pattern for '{}': {}", key, e))
                })?;
                self.compiled_patterns.insert(key.clone(), re);
            }
        }
        Ok(())
    }

    /// Validate an env file against this schema
    pub fn validate(&self, env: &EnvFile) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut passed: usize = 0;

        // Check each key defined in the schema
        for (key, var_schema) in &self.vars {
            let value = env.get(key);

            // Handle depends_on: skip required check if dependency not met
            if let Some(dep_key) = &var_schema.depends_on {
                let dep_value = env.get(dep_key);
                let dep_met = dep_value.is_some_and(is_truthy);
                if !dep_met {
                    // Dependency not met — skip validation entirely for this key.
                    // Do not increment passed; skipped keys are neither pass nor fail.
                    continue;
                }
            }

            match value {
                None => {
                    if var_schema.required {
                        errors.push(ValidationError::Missing {
                            key: key.clone(),
                            schema: var_schema.clone(),
                        });
                    }
                    // If not required and missing, that's fine
                }
                Some(val) => {
                    let compiled_re = self.compiled_patterns.get(key).map(|re| re as &Regex);
                    let validation_errors = validate_value(key, val, var_schema, compiled_re);
                    if validation_errors.is_empty() {
                        passed += 1;
                    } else {
                        errors.extend(validation_errors);
                    }
                }
            }
        }

        // Check for unknown keys (in env but not in schema)
        for entry in &env.entries {
            if !self.vars.contains_key(&entry.key) {
                warnings.push(ValidationError::Unknown {
                    key: entry.key.clone(),
                });
            }
        }

        ValidationResult {
            errors,
            warnings,
            passed,
        }
    }

    /// Infer a schema from an existing env file
    pub fn infer_schema(env: &EnvFile) -> Schema {
        let mut vars = HashMap::new();

        for entry in &env.entries {
            let var_type = infer_type(&entry.key, &entry.value);
            let sensitive = is_sensitive_key(&entry.key);

            vars.insert(
                entry.key.clone(),
                VarSchema {
                    var_type,
                    required: true,
                    sensitive,
                    default: None,
                    description: None,
                    example: Some(if sensitive {
                        "<REDACTED>".to_string()
                    } else {
                        entry.value.clone()
                    }),
                    depends_on: None,
                },
            );
        }

        Schema {
            vars,
            compiled_patterns: HashMap::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Type validation
// ---------------------------------------------------------------------------

/// `compiled_pattern` must be `Some` whenever `schema.var_type` is
/// `VarType::String { pattern: Some(_), .. }` — this is guaranteed by
/// [`Schema::validate`], which looks up pre-compiled regexes built during
/// [`Schema::load`].  Tests that construct a schema via `toml::from_str`
/// directly may pass `None`, in which case the regex is compiled on the fly
/// as a fallback (patterns are still guaranteed valid by the schema TOML).
fn validate_value(
    key: &str,
    value: &str,
    schema: &VarSchema,
    compiled_pattern: Option<&Regex>,
) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    match &schema.var_type {
        VarType::String {
            min_length,
            max_length,
            pattern,
        } => {
            if let Some(min) = min_length {
                if value.len() < *min {
                    errors.push(ValidationError::ConstraintViolation {
                        key: key.to_string(),
                        value: value.to_string(),
                        constraint: format!("min_length={}, got {}", min, value.len()),
                    });
                }
            }
            if let Some(max) = max_length {
                if value.len() > *max {
                    errors.push(ValidationError::ConstraintViolation {
                        key: key.to_string(),
                        value: value.to_string(),
                        constraint: format!("max_length={}, got {}", max, value.len()),
                    });
                }
            }
            if let Some(pat) = pattern {
                // Use the pre-compiled Regex when available (normal production
                // path).  Fall back to on-the-fly compilation only for unit
                // tests that bypass Schema::load.
                let matches = if let Some(re) = compiled_pattern {
                    re.is_match(value)
                } else {
                    Regex::new(pat)
                        .map(|re| re.is_match(value))
                        .unwrap_or(false)
                };
                if !matches {
                    errors.push(ValidationError::ConstraintViolation {
                        key: key.to_string(),
                        value: value.to_string(),
                        constraint: format!("pattern \"{}\" did not match", pat),
                    });
                }
            }
        }

        VarType::Integer { min, max } => match value.parse::<i64>() {
            Ok(n) => {
                if let Some(min_val) = min {
                    if n < *min_val {
                        errors.push(ValidationError::ConstraintViolation {
                            key: key.to_string(),
                            value: value.to_string(),
                            constraint: format!("min={}, got {}", min_val, n),
                        });
                    }
                }
                if let Some(max_val) = max {
                    if n > *max_val {
                        errors.push(ValidationError::ConstraintViolation {
                            key: key.to_string(),
                            value: value.to_string(),
                            constraint: format!("max={}, got {}", max_val, n),
                        });
                    }
                }
            }
            Err(_) => {
                errors.push(ValidationError::TypeError {
                    key: key.to_string(),
                    value: value.to_string(),
                    expected: "integer".to_string(),
                    got: "non-numeric string".to_string(),
                });
            }
        },

        VarType::Port => match value.parse::<i64>() {
            Ok(n) => {
                if !(1..=65535).contains(&n) {
                    errors.push(ValidationError::ConstraintViolation {
                        key: key.to_string(),
                        value: value.to_string(),
                        constraint: format!("port must be 1-65535, got {}", n),
                    });
                }
            }
            Err(_) => {
                errors.push(ValidationError::TypeError {
                    key: key.to_string(),
                    value: value.to_string(),
                    expected: "port".to_string(),
                    got: "non-numeric string".to_string(),
                });
            }
        },

        VarType::Boolean => {
            let lower = value.to_lowercase();
            if !matches!(lower.as_str(), "true" | "false" | "1" | "0" | "yes" | "no") {
                errors.push(ValidationError::TypeError {
                    key: key.to_string(),
                    value: value.to_string(),
                    expected: "boolean".to_string(),
                    got: format!(
                        "\"{}\" (expected one of: true, false, 1, 0, yes, no)",
                        value
                    ),
                });
            }
        }

        VarType::Url { schemes } => {
            // Basic URL validation: check format with regex
            // Pattern: scheme://[user[:pass]@]host[:port][/path][?query][#fragment]
            if !URL_PATTERN.is_match(value) {
                errors.push(ValidationError::TypeError {
                    key: key.to_string(),
                    value: value.to_string(),
                    expected: "url".to_string(),
                    got: "invalid URL format".to_string(),
                });
            } else if let Some(allowed_schemes) = schemes {
                if let Some(scheme) = value.split("://").next() {
                    if !allowed_schemes.iter().any(|s| s == scheme) {
                        errors.push(ValidationError::ConstraintViolation {
                            key: key.to_string(),
                            value: value.to_string(),
                            constraint: format!(
                                "scheme \"{}\" not in allowed: [{}]",
                                scheme,
                                allowed_schemes.join(", ")
                            ),
                        });
                    }
                }
            }
        }

        VarType::Email => {
            // RFC 5322-inspired email validation (simplified but robust)
            if !EMAIL_PATTERN.is_match(value) {
                errors.push(ValidationError::TypeError {
                    key: key.to_string(),
                    value: value.to_string(),
                    expected: "email".to_string(),
                    got: "invalid email format".to_string(),
                });
            }
        }

        VarType::Enum { values } => {
            if !values.contains(&value.to_string()) {
                errors.push(ValidationError::TypeError {
                    key: key.to_string(),
                    value: value.to_string(),
                    expected: format!("enum (one of: {})", values.join(", ")),
                    got: format!("\"{}\"", value),
                });
            }
        }

        VarType::Path { must_exist } => {
            if *must_exist && !std::path::Path::new(value).exists() {
                errors.push(ValidationError::ConstraintViolation {
                    key: key.to_string(),
                    value: value.to_string(),
                    constraint: "path must exist".to_string(),
                });
            }
        }
    }

    errors
}

// ---------------------------------------------------------------------------
// Type inference
// ---------------------------------------------------------------------------

fn infer_type(key: &str, value: &str) -> VarType {
    // Boolean check
    if matches!(
        value.to_lowercase().as_str(),
        "true" | "false" | "1" | "0" | "yes" | "no"
    ) {
        return VarType::Boolean;
    }

    // Integer / port check
    if let Ok(n) = value.parse::<i64>() {
        let upper_key = key.to_uppercase();
        let looks_like_port =
            upper_key == "PORT" || upper_key.ends_with("_PORT") || upper_key.contains("PORT");
        if looks_like_port && (1..=65535).contains(&n) {
            return VarType::Port;
        }
        return VarType::Integer {
            min: None,
            max: None,
        };
    }

    // URL check
    if value.contains("://") {
        return VarType::Url { schemes: None };
    }

    // Email check
    if value.contains('@') {
        let parts: Vec<&str> = value.splitn(2, '@').collect();
        if parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty() {
            return VarType::Email;
        }
    }

    // Default to string
    VarType::String {
        min_length: None,
        max_length: None,
        pattern: None,
    }
}

fn is_sensitive_key(key: &str) -> bool {
    let upper = key.to_uppercase();
    let sensitive_suffixes = [
        "_KEY",
        "_SECRET",
        "_PASSWORD",
        "_TOKEN",
        "_PASS",
        "_PASSPHRASE",
    ];
    // Also flag keys that _begin_ with a sensitive word (e.g. SECRET_VALUE).
    let sensitive_prefixes = ["SECRET_", "PASSWORD_", "TOKEN_"];
    let exact_matches = ["PASSWORD", "SECRET", "TOKEN", "KEY"];
    sensitive_suffixes.iter().any(|s| upper.ends_with(s))
        || sensitive_prefixes.iter().any(|p| upper.starts_with(p))
        || exact_matches.contains(&upper.as_str())
}

// ---------------------------------------------------------------------------
// Schema serialization helpers (for `schema init` output)
// ---------------------------------------------------------------------------

/// Escape a string for use inside a TOML basic string (double-quoted).
fn toml_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

impl Schema {
    /// Serialize the schema to a human-readable TOML string
    pub fn to_toml_string(&self) -> String {
        let mut output = String::new();
        output.push_str("# Schema generated by graxaim schema init\n");
        output.push_str("# Review and adjust types, constraints, and descriptions.\n\n");

        // Sort keys for deterministic output
        let mut keys: Vec<&String> = self.vars.keys().collect();
        keys.sort();

        for key in keys {
            let var = &self.vars[key];
            output.push_str(&format!("[vars.{}]\n", key));
            // Type
            match &var.var_type {
                VarType::String {
                    min_length,
                    max_length,
                    pattern,
                } => {
                    output.push_str("type = \"string\"\n");
                    if let Some(min) = min_length {
                        output.push_str(&format!("min_length = {}\n", min));
                    }
                    if let Some(max) = max_length {
                        output.push_str(&format!("max_length = {}\n", max));
                    }
                    if let Some(pat) = pattern {
                        output.push_str(&format!("pattern = \"{}\"\n", toml_escape(pat)));
                    }
                }
                VarType::Integer { min, max } => {
                    output.push_str("type = \"integer\"\n");
                    if let Some(m) = min {
                        output.push_str(&format!("min = {}\n", m));
                    }
                    if let Some(m) = max {
                        output.push_str(&format!("max = {}\n", m));
                    }
                }
                VarType::Port => {
                    output.push_str("type = \"port\"\n");
                }
                VarType::Boolean => {
                    output.push_str("type = \"boolean\"\n");
                }
                VarType::Url { schemes } => {
                    output.push_str("type = \"url\"\n");
                    if let Some(s) = schemes {
                        output.push_str(&format!(
                            "schemes = [{}]\n",
                            s.iter()
                                .map(|v| format!("\"{}\"", toml_escape(v)))
                                .collect::<Vec<_>>()
                                .join(", ")
                        ));
                    }
                }
                VarType::Email => {
                    output.push_str("type = \"email\"\n");
                }
                VarType::Enum { values } => {
                    output.push_str("type = \"enum\"\n");
                    output.push_str(&format!(
                        "values = [{}]\n",
                        values
                            .iter()
                            .map(|v| format!("\"{}\"", toml_escape(v)))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ));
                }
                VarType::Path { must_exist } => {
                    output.push_str("type = \"path\"\n");
                    if *must_exist {
                        output.push_str("must_exist = true\n");
                    }
                }
            }

            // Common fields
            output.push_str(&format!("required = {}\n", var.required));
            if var.sensitive {
                output.push_str("sensitive = true\n");
            }
            if let Some(default) = &var.default {
                output.push_str(&format!("default = \"{}\"\n", default));
            }
            if let Some(desc) = &var.description {
                output.push_str(&format!("description = \"{}\"\n", desc));
            }
            if let Some(example) = &var.example {
                output.push_str(&format!("example = \"{}\"\n", example));
            }
            if let Some(dep) = &var.depends_on {
                output.push_str(&format!("depends_on = \"{}\"\n", dep));
            }
            output.push('\n');
        }

        output
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::env_file::EnvFile;
    use std::path::Path;

    fn parse_env(content: &str) -> EnvFile {
        EnvFile::parse(content, Path::new("test.env")).unwrap()
    }

    fn load_schema(toml_content: &str) -> Schema {
        toml::from_str(toml_content).unwrap()
    }

    // -----------------------------------------------------------------------
    // String validation
    // -----------------------------------------------------------------------

    #[test]
    fn test_validate_string_pass() {
        let schema = load_schema(
            r#"
            [vars.NAME]
            type = "string"
            required = true
            "#,
        );
        let env = parse_env("NAME=hello\n");
        let result = schema.validate(&env);
        assert_eq!(result.passed, 1);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validate_string_min_length_fail() {
        let schema = load_schema(
            r#"
            [vars.API_SECRET]
            type = "string"
            required = true
            min_length = 32
            "#,
        );
        let env = parse_env("API_SECRET=short\n");
        let result = schema.validate(&env);
        assert_eq!(result.errors.len(), 1);
        assert!(matches!(
            &result.errors[0],
            ValidationError::ConstraintViolation { constraint, .. }
            if constraint.contains("min_length")
        ));
    }

    #[test]
    fn test_validate_string_max_length_fail() {
        let schema = load_schema(
            r#"
            [vars.CODE]
            type = "string"
            required = true
            max_length = 4
            "#,
        );
        let env = parse_env("CODE=toolong\n");
        let result = schema.validate(&env);
        assert_eq!(result.errors.len(), 1);
        assert!(matches!(
            &result.errors[0],
            ValidationError::ConstraintViolation { constraint, .. }
            if constraint.contains("max_length")
        ));
    }

    #[test]
    fn test_validate_string_pattern_pass() {
        let schema = load_schema(
            r#"
            [vars.VERSION]
            type = "string"
            required = true
            pattern = "^v\\d+\\.\\d+\\.\\d+$"
            "#,
        );
        let env = parse_env("VERSION=v1.2.3\n");
        let result = schema.validate(&env);
        assert!(result.errors.is_empty());
        assert_eq!(result.passed, 1);
    }

    #[test]
    fn test_validate_string_pattern_fail() {
        let schema = load_schema(
            r#"
            [vars.VERSION]
            type = "string"
            required = true
            pattern = "^v\\d+\\.\\d+\\.\\d+$"
            "#,
        );
        let env = parse_env("VERSION=latest\n");
        let result = schema.validate(&env);
        assert_eq!(result.errors.len(), 1);
        assert!(matches!(
            &result.errors[0],
            ValidationError::ConstraintViolation { constraint, .. }
            if constraint.contains("pattern")
        ));
    }

    // -----------------------------------------------------------------------
    // Integer validation
    // -----------------------------------------------------------------------

    #[test]
    fn test_validate_integer_pass() {
        let schema = load_schema(
            r#"
            [vars.COUNT]
            type = "integer"
            required = true
            "#,
        );
        let env = parse_env("COUNT=42\n");
        let result = schema.validate(&env);
        assert_eq!(result.passed, 1);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validate_integer_non_numeric() {
        let schema = load_schema(
            r#"
            [vars.COUNT]
            type = "integer"
            required = true
            "#,
        );
        let env = parse_env("COUNT=abc\n");
        let result = schema.validate(&env);
        assert_eq!(result.errors.len(), 1);
        assert!(matches!(
            &result.errors[0],
            ValidationError::TypeError { expected, .. }
            if expected == "integer"
        ));
    }

    #[test]
    fn test_validate_integer_min_fail() {
        let schema = load_schema(
            r#"
            [vars.WORKERS]
            type = "integer"
            required = true
            min = 1
            "#,
        );
        let env = parse_env("WORKERS=0\n");
        let result = schema.validate(&env);
        assert_eq!(result.errors.len(), 1);
        assert!(matches!(
            &result.errors[0],
            ValidationError::ConstraintViolation { constraint, .. }
            if constraint.contains("min=1")
        ));
    }

    #[test]
    fn test_validate_integer_max_fail() {
        let schema = load_schema(
            r#"
            [vars.WORKERS]
            type = "integer"
            required = true
            max = 16
            "#,
        );
        let env = parse_env("WORKERS=100\n");
        let result = schema.validate(&env);
        assert_eq!(result.errors.len(), 1);
        assert!(matches!(
            &result.errors[0],
            ValidationError::ConstraintViolation { constraint, .. }
            if constraint.contains("max=16")
        ));
    }

    // -----------------------------------------------------------------------
    // Port validation
    // -----------------------------------------------------------------------

    #[test]
    fn test_validate_port_pass() {
        let schema = load_schema(
            r#"
            [vars.PORT]
            type = "port"
            required = true
            "#,
        );
        let env = parse_env("PORT=3000\n");
        let result = schema.validate(&env);
        assert_eq!(result.passed, 1);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validate_port_non_numeric() {
        let schema = load_schema(
            r#"
            [vars.PORT]
            type = "port"
            required = true
            "#,
        );
        let env = parse_env("PORT=not_a_number\n");
        let result = schema.validate(&env);
        assert_eq!(result.errors.len(), 1);
        assert!(matches!(
            &result.errors[0],
            ValidationError::TypeError { expected, .. }
            if expected == "port"
        ));
    }

    #[test]
    fn test_validate_port_out_of_range() {
        let schema = load_schema(
            r#"
            [vars.PORT]
            type = "port"
            required = true
            "#,
        );
        let env = parse_env("PORT=70000\n");
        let result = schema.validate(&env);
        assert_eq!(result.errors.len(), 1);
        assert!(matches!(
            &result.errors[0],
            ValidationError::ConstraintViolation { constraint, .. }
            if constraint.contains("1-65535")
        ));
    }

    #[test]
    fn test_validate_port_zero() {
        let schema = load_schema(
            r#"
            [vars.PORT]
            type = "port"
            required = true
            "#,
        );
        let env = parse_env("PORT=0\n");
        let result = schema.validate(&env);
        assert_eq!(result.errors.len(), 1);
    }

    // -----------------------------------------------------------------------
    // Boolean validation
    // -----------------------------------------------------------------------

    #[test]
    fn test_validate_boolean_pass() {
        let schema_toml = r#"
            [vars.DEBUG]
            type = "boolean"
            required = true
        "#;
        for val in &[
            "true", "false", "1", "0", "yes", "no", "TRUE", "False", "YES",
        ] {
            let schema: Schema = toml::from_str(schema_toml).unwrap();
            let env = parse_env(&format!("DEBUG={}\n", val));
            let result = schema.validate(&env);
            assert!(
                result.errors.is_empty(),
                "Expected {} to pass boolean validation",
                val
            );
        }
    }

    #[test]
    fn test_validate_boolean_fail() {
        let schema = load_schema(
            r#"
            [vars.DEBUG]
            type = "boolean"
            required = true
            "#,
        );
        let env = parse_env("DEBUG=maybe\n");
        let result = schema.validate(&env);
        assert_eq!(result.errors.len(), 1);
        assert!(matches!(
            &result.errors[0],
            ValidationError::TypeError { expected, .. }
            if expected == "boolean"
        ));
    }

    // -----------------------------------------------------------------------
    // URL validation
    // -----------------------------------------------------------------------

    #[test]
    fn test_validate_url_pass() {
        let schema = load_schema(
            r#"
            [vars.API_URL]
            type = "url"
            required = true
            "#,
        );
        let env = parse_env("API_URL=https://example.com\n");
        let result = schema.validate(&env);
        assert_eq!(result.passed, 1);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validate_url_fail_no_scheme() {
        let schema = load_schema(
            r#"
            [vars.API_URL]
            type = "url"
            required = true
            "#,
        );
        let env = parse_env("API_URL=example.com\n");
        let result = schema.validate(&env);
        assert_eq!(result.errors.len(), 1);
        assert!(matches!(
            &result.errors[0],
            ValidationError::TypeError { expected, .. }
            if expected == "url"
        ));
    }

    #[test]
    fn test_validate_url_scheme_constraint() {
        let schema = load_schema(
            r#"
            [vars.DATABASE_URL]
            type = "url"
            required = true
            schemes = ["postgres", "mysql"]
            "#,
        );
        let env = parse_env("DATABASE_URL=postgres://localhost/mydb\n");
        let result = schema.validate(&env);
        assert!(result.errors.is_empty());

        let env_bad = parse_env("DATABASE_URL=http://localhost/mydb\n");
        let result_bad = schema.validate(&env_bad);
        assert_eq!(result_bad.errors.len(), 1);
        assert!(matches!(
            &result_bad.errors[0],
            ValidationError::ConstraintViolation { constraint, .. }
            if constraint.contains("scheme")
        ));
    }

    // -----------------------------------------------------------------------
    // Email validation
    // -----------------------------------------------------------------------

    #[test]
    fn test_validate_email_pass() {
        let schema = load_schema(
            r#"
            [vars.ADMIN_EMAIL]
            type = "email"
            required = true
            "#,
        );
        let env = parse_env("ADMIN_EMAIL=admin@example.com\n");
        let result = schema.validate(&env);
        assert_eq!(result.passed, 1);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validate_email_fail() {
        let schema = load_schema(
            r#"
            [vars.ADMIN_EMAIL]
            type = "email"
            required = true
            "#,
        );
        let env = parse_env("ADMIN_EMAIL=not-an-email\n");
        let result = schema.validate(&env);
        assert_eq!(result.errors.len(), 1);
        assert!(matches!(
            &result.errors[0],
            ValidationError::TypeError { expected, .. }
            if expected == "email"
        ));
    }

    #[test]
    fn test_validate_email_no_text_before_at() {
        let schema = load_schema(
            r#"
            [vars.ADMIN_EMAIL]
            type = "email"
            required = true
            "#,
        );
        let env = parse_env("ADMIN_EMAIL=@example.com\n");
        let result = schema.validate(&env);
        assert_eq!(result.errors.len(), 1);
    }

    #[test]
    fn test_validate_email_no_text_after_at() {
        let schema = load_schema(
            r#"
            [vars.ADMIN_EMAIL]
            type = "email"
            required = true
            "#,
        );
        let env = parse_env("ADMIN_EMAIL=admin@\n");
        let result = schema.validate(&env);
        assert_eq!(result.errors.len(), 1);
    }

    // -----------------------------------------------------------------------
    // Enum validation
    // -----------------------------------------------------------------------

    #[test]
    fn test_validate_enum_pass() {
        let schema = load_schema(
            r#"
            [vars.LOG_LEVEL]
            type = "enum"
            required = true
            values = ["debug", "info", "warn", "error"]
            "#,
        );
        let env = parse_env("LOG_LEVEL=info\n");
        let result = schema.validate(&env);
        assert_eq!(result.passed, 1);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validate_enum_fail() {
        let schema = load_schema(
            r#"
            [vars.LOG_LEVEL]
            type = "enum"
            required = true
            values = ["debug", "info", "warn", "error"]
            "#,
        );
        let env = parse_env("LOG_LEVEL=verbose\n");
        let result = schema.validate(&env);
        assert_eq!(result.errors.len(), 1);
        assert!(matches!(
            &result.errors[0],
            ValidationError::TypeError { expected, .. }
            if expected.contains("enum")
        ));
    }

    // -----------------------------------------------------------------------
    // Path validation
    // -----------------------------------------------------------------------

    #[test]
    fn test_validate_path_pass_no_exist_check() {
        let schema = load_schema(
            r#"
            [vars.DATA_DIR]
            type = "path"
            required = true
            "#,
        );
        let env = parse_env("DATA_DIR=/some/nonexistent/path\n");
        let result = schema.validate(&env);
        assert_eq!(result.passed, 1);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validate_path_must_exist_fail() {
        let schema = load_schema(
            r#"
            [vars.DATA_DIR]
            type = "path"
            required = true
            must_exist = true
            "#,
        );
        let env = parse_env("DATA_DIR=/definitely/not/a/real/path/xyzzy\n");
        let result = schema.validate(&env);
        assert_eq!(result.errors.len(), 1);
        assert!(matches!(
            &result.errors[0],
            ValidationError::ConstraintViolation { constraint, .. }
            if constraint.contains("must exist")
        ));
    }

    #[test]
    fn test_validate_path_must_exist_pass() {
        let schema = load_schema(
            r#"
            [vars.DATA_DIR]
            type = "path"
            required = true
            must_exist = true
            "#,
        );
        // /tmp always exists
        let env = parse_env("DATA_DIR=/tmp\n");
        let result = schema.validate(&env);
        assert!(result.errors.is_empty());
    }

    // -----------------------------------------------------------------------
    // Missing required
    // -----------------------------------------------------------------------

    #[test]
    fn test_validate_missing_required() {
        let schema = load_schema(
            r#"
            [vars.API_KEY]
            type = "string"
            required = true
            "#,
        );
        let env = parse_env("OTHER=value\n");
        let result = schema.validate(&env);
        assert_eq!(result.errors.len(), 1);
        assert!(matches!(
            &result.errors[0],
            ValidationError::Missing { key, .. }
            if key == "API_KEY"
        ));
    }

    #[test]
    fn test_validate_missing_optional() {
        let schema = load_schema(
            r#"
            [vars.OPTIONAL_VAR]
            type = "string"
            required = false
            "#,
        );
        let env = parse_env("OTHER=value\n");
        let result = schema.validate(&env);
        assert!(result.errors.is_empty());
    }

    // -----------------------------------------------------------------------
    // Unknown keys
    // -----------------------------------------------------------------------

    #[test]
    fn test_validate_unknown_keys() {
        let schema = load_schema(
            r#"
            [vars.KNOWN]
            type = "string"
            required = true
            "#,
        );
        let env = parse_env("KNOWN=value\nUNKNOWN=other\n");
        let result = schema.validate(&env);
        assert_eq!(result.warnings.len(), 1);
        assert!(matches!(
            &result.warnings[0],
            ValidationError::Unknown { key }
            if key == "UNKNOWN"
        ));
    }

    // -----------------------------------------------------------------------
    // depends_on
    // -----------------------------------------------------------------------

    #[test]
    fn test_depends_on_truthy_present() {
        let schema = load_schema(
            r#"
            [vars.SMTP_ENABLED]
            type = "boolean"
            required = true

            [vars.SMTP_PASSWORD]
            type = "string"
            required = true
            depends_on = "SMTP_ENABLED"
            "#,
        );
        // Dependency met (truthy), but SMTP_PASSWORD missing → error
        let env = parse_env("SMTP_ENABLED=true\n");
        let result = schema.validate(&env);
        assert!(result.errors.iter().any(|e| matches!(e,
            ValidationError::Missing { key, .. } if key == "SMTP_PASSWORD"
        )));
    }

    #[test]
    fn test_depends_on_falsy() {
        let schema = load_schema(
            r#"
            [vars.SMTP_ENABLED]
            type = "boolean"
            required = true

            [vars.SMTP_PASSWORD]
            type = "string"
            required = true
            depends_on = "SMTP_ENABLED"
            "#,
        );
        // Dependency NOT met (false) → SMTP_PASSWORD not required
        let env = parse_env("SMTP_ENABLED=false\n");
        let result = schema.validate(&env);
        let missing_smtp_pw = result.errors.iter().any(|e| {
            matches!(e,
                ValidationError::Missing { key, .. } if key == "SMTP_PASSWORD"
            )
        });
        assert!(!missing_smtp_pw);
    }

    #[test]
    fn test_depends_on_missing_dependency() {
        let schema = load_schema(
            r#"
            [vars.SMTP_PASSWORD]
            type = "string"
            required = true
            depends_on = "SMTP_ENABLED"
            "#,
        );
        // SMTP_ENABLED doesn't exist → dependency not met → skip
        let env = parse_env("OTHER=value\n");
        let result = schema.validate(&env);
        let missing_smtp_pw = result.errors.iter().any(|e| {
            matches!(e,
                ValidationError::Missing { key, .. } if key == "SMTP_PASSWORD"
            )
        });
        assert!(!missing_smtp_pw);
    }

    // -----------------------------------------------------------------------
    // Type inference
    // -----------------------------------------------------------------------

    #[test]
    fn test_infer_boolean() {
        let env = parse_env("DEBUG=true\n");
        let schema = Schema::infer_schema(&env);
        assert!(matches!(
            schema.vars.get("DEBUG").unwrap().var_type,
            VarType::Boolean
        ));
    }

    #[test]
    fn test_infer_port() {
        let env = parse_env("PORT=3000\n");
        let schema = Schema::infer_schema(&env);
        assert!(matches!(
            schema.vars.get("PORT").unwrap().var_type,
            VarType::Port
        ));
    }

    #[test]
    fn test_infer_integer() {
        let env = parse_env("BIG_NUMBER=100000\n");
        let schema = Schema::infer_schema(&env);
        assert!(matches!(
            schema.vars.get("BIG_NUMBER").unwrap().var_type,
            VarType::Integer { .. }
        ));
    }

    #[test]
    fn test_infer_url() {
        let env = parse_env("DATABASE_URL=postgres://localhost/mydb\n");
        let schema = Schema::infer_schema(&env);
        assert!(matches!(
            schema.vars.get("DATABASE_URL").unwrap().var_type,
            VarType::Url { .. }
        ));
    }

    #[test]
    fn test_infer_email() {
        let env = parse_env("ADMIN_EMAIL=admin@example.com\n");
        let schema = Schema::infer_schema(&env);
        assert!(matches!(
            schema.vars.get("ADMIN_EMAIL").unwrap().var_type,
            VarType::Email
        ));
    }

    #[test]
    fn test_infer_string() {
        let env = parse_env("APP_NAME=myapp\n");
        let schema = Schema::infer_schema(&env);
        assert!(matches!(
            schema.vars.get("APP_NAME").unwrap().var_type,
            VarType::String { .. }
        ));
    }

    #[test]
    fn test_infer_sensitive() {
        let env =
            parse_env("DB_PASSWORD=secret123\nAPI_KEY=abc\nACCESS_TOKEN=tok\nSECRET_VALUE=val\n");
        let schema = Schema::infer_schema(&env);
        assert!(schema.vars.get("DB_PASSWORD").unwrap().sensitive);
        assert!(schema.vars.get("API_KEY").unwrap().sensitive);
        assert!(schema.vars.get("ACCESS_TOKEN").unwrap().sensitive);
        assert!(schema.vars.get("SECRET_VALUE").unwrap().sensitive);
    }

    #[test]
    fn test_infer_non_sensitive() {
        let env = parse_env("APP_NAME=myapp\nPORT=3000\n");
        let schema = Schema::infer_schema(&env);
        assert!(!schema.vars.get("APP_NAME").unwrap().sensitive);
        assert!(!schema.vars.get("PORT").unwrap().sensitive);
    }

    // -----------------------------------------------------------------------
    // Schema to TOML roundtrip
    // -----------------------------------------------------------------------

    #[test]
    fn test_schema_toml_roundtrip() {
        let toml_input = r#"
[vars.PORT]
type = "port"
required = true
default = "3000"

[vars.LOG_LEVEL]
type = "enum"
required = false
values = ["debug", "info", "warn", "error"]
default = "info"
        "#;
        let schema: Schema = toml::from_str(toml_input).unwrap();
        assert!(schema.vars.contains_key("PORT"));
        assert!(schema.vars.contains_key("LOG_LEVEL"));
        assert!(matches!(
            schema.vars.get("PORT").unwrap().var_type,
            VarType::Port
        ));
        assert!(matches!(
            schema.vars.get("LOG_LEVEL").unwrap().var_type,
            VarType::Enum { .. }
        ));
    }

    // -----------------------------------------------------------------------
    // Full integration-style validation
    // -----------------------------------------------------------------------

    #[test]
    fn test_full_validation_mixed() {
        let schema = load_schema(
            r#"
            [vars.PORT]
            type = "port"
            required = true

            [vars.DATABASE_URL]
            type = "url"
            required = true
            schemes = ["postgres", "mysql"]

            [vars.LOG_LEVEL]
            type = "enum"
            required = false
            values = ["debug", "info", "warn", "error"]

            [vars.DEBUG]
            type = "boolean"
            required = false
            "#,
        );
        let env = parse_env(
            "PORT=3000\nDATABASE_URL=postgres://localhost/mydb\nLOG_LEVEL=info\nDEBUG=true\nLEGACY=old\n",
        );
        let result = schema.validate(&env);
        assert_eq!(result.passed, 4);
        assert!(result.errors.is_empty());
        assert_eq!(result.warnings.len(), 1); // LEGACY is unknown
    }
}
