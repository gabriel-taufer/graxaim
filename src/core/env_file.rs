use crate::errors::{GraxaimError, Result};
use std::fmt;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub struct EnvEntry {
    pub key: String,
    pub value: String,
    pub comment: Option<String>, // inline comment
    pub line_number: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnvFile {
    pub entries: Vec<EnvEntry>,
    pub header_comments: Vec<String>, // comments before first key
}

impl Default for EnvFile {
    fn default() -> Self {
        Self::new()
    }
}

impl EnvFile {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            header_comments: Vec::new(),
        }
    }

    /// Parse a .env file from a path
    pub fn from_path(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path).map_err(GraxaimError::Io)?;
        Self::parse(&content, path)
    }

    /// Parse .env file content
    pub fn parse(content: &str, path: &Path) -> Result<Self> {
        let mut env_file = EnvFile::new();
        let mut in_header = true;
        let mut line_number = 0;

        // Normalize line endings (handle \r\n)
        let content = content.replace("\r\n", "\n");

        for line in content.lines() {
            line_number += 1;
            let trimmed = line.trim();

            // Empty line
            if trimmed.is_empty() {
                continue;
            }

            // Comment line
            if trimmed.starts_with('#') {
                if in_header {
                    env_file.header_comments.push(line.to_string());
                }
                // We don't preserve non-header comments for now
                continue;
            }

            // Now we have a key=value line
            in_header = false;

            // Find the first '=' to split key and value
            let Some(eq_pos) = line.find('=') else {
                return Err(GraxaimError::EnvFileParse {
                    path: path.to_path_buf(),
                    message: format!("Line {}: Missing '=' separator", line_number),
                });
            };

            let key_part = &line[..eq_pos];
            let value_part = &line[eq_pos + 1..];

            let key = key_part.trim().to_string();
            if key.is_empty() {
                return Err(GraxaimError::EnvFileParse {
                    path: path.to_path_buf(),
                    message: format!("Line {}: Empty key", line_number),
                });
            }

            // Check for inline comment (only if not inside quotes)
            let (value_str, comment) = Self::extract_value_and_comment(value_part);

            let value = Self::parse_value(&value_str);

            env_file.entries.push(EnvEntry {
                key,
                value,
                comment,
                line_number,
            });
        }

        Ok(env_file)
    }

    /// Extract value and inline comment, respecting quotes
    fn extract_value_and_comment(value_part: &str) -> (String, Option<String>) {
        let trimmed = value_part.trim();

        // If value starts with quote, we need to find the matching end quote
        if trimmed.starts_with('"') || trimmed.starts_with('\'') {
            let quote_char = trimmed.chars().next().unwrap();
            let mut escaped = false;
            let mut end_quote_pos = None;

            for (i, ch) in trimmed.chars().enumerate().skip(1) {
                if escaped {
                    escaped = false;
                    continue;
                }
                if ch == '\\' {
                    escaped = true;
                    continue;
                }
                if ch == quote_char {
                    end_quote_pos = Some(i);
                    break;
                }
            }

            if let Some(pos) = end_quote_pos {
                let value = trimmed[..=pos].to_string();
                let rest = trimmed[pos + 1..].trim();
                let comment = if rest.starts_with('#') {
                    Some(rest.to_string())
                } else {
                    None
                };
                return (value, comment);
            }
        }

        // No quotes or unclosed quotes - look for # comment
        if let Some(hash_pos) = trimmed.find('#') {
            let value = trimmed[..hash_pos].trim_end().to_string();
            let comment = Some(trimmed[hash_pos..].to_string());
            (value, comment)
        } else {
            (trimmed.to_string(), None)
        }
    }

    /// Parse the value, handling quotes and escape sequences
    fn parse_value(value_str: &str) -> String {
        let trimmed = value_str.trim();

        // Empty value
        if trimmed.is_empty() {
            return String::new();
        }

        // Check if value is quoted
        if (trimmed.starts_with('"') && trimmed.ends_with('"'))
            || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
        {
            if trimmed.len() < 2 {
                return String::new();
            }

            let inner = &trimmed[1..trimmed.len() - 1];

            // Handle escape sequences for double quotes
            if trimmed.starts_with('"') {
                return Self::unescape(inner);
            } else {
                // Single quotes: no escape processing
                return inner.to_string();
            }
        }

        // Unquoted value
        trimmed.to_string()
    }

    /// Unescape common escape sequences
    fn unescape(s: &str) -> String {
        let mut result = String::new();
        let mut chars = s.chars();

        while let Some(ch) = chars.next() {
            if ch == '\\' {
                if let Some(next) = chars.next() {
                    match next {
                        'n' => result.push('\n'),
                        'r' => result.push('\r'),
                        't' => result.push('\t'),
                        '\\' => result.push('\\'),
                        '"' => result.push('"'),
                        _ => {
                            result.push('\\');
                            result.push(next);
                        }
                    }
                } else {
                    result.push('\\');
                }
            } else {
                result.push(ch);
            }
        }

        result
    }

    /// Write the env file to a path
    pub fn write_to_path(&self, path: &Path) -> Result<()> {
        let content = self.to_string();
        fs::write(path, content).map_err(GraxaimError::Io)?;
        Ok(())
    }

    /// Format a value for writing (add quotes if necessary)
    fn format_value(value: &str) -> String {
        // Empty value
        if value.is_empty() {
            return String::new();
        }

        // Check if value needs quoting (contains spaces, special chars, or starts/ends with whitespace)
        let needs_quotes = value.contains(' ')
            || value.contains('\n')
            || value.contains('\t')
            || value.contains('#')
            || value.starts_with('"')
            || value.starts_with('\'')
            || value != value.trim();

        if needs_quotes {
            // Escape special characters
            let escaped = value
                .replace('\\', "\\\\")
                .replace('"', "\\\"")
                .replace('\n', "\\n")
                .replace('\r', "\\r")
                .replace('\t', "\\t");
            format!("\"{}\"", escaped)
        } else {
            value.to_string()
        }
    }

    /// Get value by key
    pub fn get(&self, key: &str) -> Option<&str> {
        self.entries
            .iter()
            .find(|e| e.key == key)
            .map(|e| e.value.as_str())
    }

    /// Set or update a value
    #[allow(dead_code)]
    pub fn set(&mut self, key: String, value: String) {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.key == key) {
            entry.value = value;
        } else {
            self.entries.push(EnvEntry {
                key,
                value,
                comment: None,
                line_number: self.entries.len() + 1,
            });
        }
    }

    /// Remove a key
    #[allow(dead_code)]
    pub fn remove(&mut self, key: &str) -> bool {
        let initial_len = self.entries.len();
        self.entries.retain(|e| e.key != key);
        self.entries.len() < initial_len
    }
}

impl fmt::Display for EnvFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut lines = Vec::new();

        // Write header comments
        for comment in &self.header_comments {
            lines.push(comment.clone());
        }

        if !self.header_comments.is_empty() && !self.entries.is_empty() {
            lines.push(String::new()); // blank line after header
        }

        // Write entries
        for entry in &self.entries {
            let formatted_value = EnvFile::format_value(&entry.value);
            let line = if let Some(comment) = &entry.comment {
                format!("{}={} {}", entry.key, formatted_value, comment)
            } else {
                format!("{}={}", entry.key, formatted_value)
            };
            lines.push(line);
        }

        write!(f, "{}", lines.join("\n") + "\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let content = "KEY=value\n";
        let env = EnvFile::parse(content, Path::new("test.env")).unwrap();
        assert_eq!(env.entries.len(), 1);
        assert_eq!(env.entries[0].key, "KEY");
        assert_eq!(env.entries[0].value, "value");
    }

    #[test]
    fn test_parse_with_spaces() {
        let content = "KEY = value with spaces \n";
        let env = EnvFile::parse(content, Path::new("test.env")).unwrap();
        assert_eq!(env.entries[0].key, "KEY");
        assert_eq!(env.entries[0].value, "value with spaces");
    }

    #[test]
    fn test_parse_quoted_value() {
        let content = "KEY=\"quoted value\"\n";
        let env = EnvFile::parse(content, Path::new("test.env")).unwrap();
        assert_eq!(env.entries[0].value, "quoted value");
    }

    #[test]
    fn test_parse_value_with_equals() {
        let content = "KEY=value=with=equals\n";
        let env = EnvFile::parse(content, Path::new("test.env")).unwrap();
        assert_eq!(env.entries[0].value, "value=with=equals");
    }

    #[test]
    fn test_parse_empty_value() {
        let content = "KEY=\n";
        let env = EnvFile::parse(content, Path::new("test.env")).unwrap();
        assert_eq!(env.entries[0].value, "");
    }

    #[test]
    fn test_parse_comment() {
        let content = "# This is a comment\nKEY=value\n";
        let env = EnvFile::parse(content, Path::new("test.env")).unwrap();
        assert_eq!(env.header_comments.len(), 1);
        assert_eq!(env.entries.len(), 1);
    }

    #[test]
    fn test_parse_inline_comment() {
        let content = "KEY=value # inline comment\n";
        let env = EnvFile::parse(content, Path::new("test.env")).unwrap();
        assert_eq!(env.entries[0].value, "value");
        assert_eq!(env.entries[0].comment, Some("# inline comment".to_string()));
    }

    #[test]
    fn test_parse_multiline_value() {
        let content = "KEY=\"line1\\nline2\"\n";
        let env = EnvFile::parse(content, Path::new("test.env")).unwrap();
        assert_eq!(env.entries[0].value, "line1\nline2");
    }

    #[test]
    fn test_parse_windows_line_endings() {
        let content = "KEY=value\r\n";
        let env = EnvFile::parse(content, Path::new("test.env")).unwrap();
        assert_eq!(env.entries.len(), 1);
        assert_eq!(env.entries[0].value, "value");
    }

    #[test]
    fn test_round_trip() {
        let content = "# Header comment\nKEY1=value1\nKEY2=\"quoted value\"\n";
        let env = EnvFile::parse(content, Path::new("test.env")).unwrap();
        let output = env.to_string();
        let reparsed = EnvFile::parse(&output, Path::new("test.env")).unwrap();
        // Compare semantic content (key/value/comment), not line numbers
        // since Display adds a blank separator line after header comments
        assert_eq!(env.entries.len(), reparsed.entries.len());
        for (orig, reparsed) in env.entries.iter().zip(reparsed.entries.iter()) {
            assert_eq!(orig.key, reparsed.key);
            assert_eq!(orig.value, reparsed.value);
            assert_eq!(orig.comment, reparsed.comment);
        }
        assert_eq!(env.header_comments, reparsed.header_comments);
    }

    #[test]
    fn test_get_set_remove() {
        let mut env = EnvFile::new();
        assert_eq!(env.get("KEY"), None);

        env.set("KEY".to_string(), "value".to_string());
        assert_eq!(env.get("KEY"), Some("value"));

        env.set("KEY".to_string(), "new_value".to_string());
        assert_eq!(env.get("KEY"), Some("new_value"));

        assert!(env.remove("KEY"));
        assert_eq!(env.get("KEY"), None);
        assert!(!env.remove("KEY"));
    }
}
