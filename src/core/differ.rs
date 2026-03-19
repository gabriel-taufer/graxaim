use crate::core::env_file::EnvFile;
use std::collections::BTreeSet;

/// Result of comparing two env files
#[derive(Debug, Clone)]
pub struct DiffResult {
    /// Keys only in profile A (key, value)
    pub only_in_a: Vec<(String, String)>,
    /// Keys only in profile B (key, value)
    pub only_in_b: Vec<(String, String)>,
    /// Keys with different values (key, value_a, value_b)
    pub different: Vec<(String, String, String)>,
    /// Keys with identical values (key, value)
    pub same: Vec<(String, String)>,
}

impl DiffResult {
    /// Returns true if there are no differences between the two profiles
    pub fn is_empty(&self) -> bool {
        self.only_in_a.is_empty() && self.only_in_b.is_empty() && self.different.is_empty()
    }
}

/// Compare two `EnvFile` instances and categorize keys into four groups
pub fn diff_env_files(a: &EnvFile, b: &EnvFile) -> DiffResult {
    let keys_a: BTreeSet<&str> = a.entries.iter().map(|e| e.key.as_str()).collect();
    let keys_b: BTreeSet<&str> = b.entries.iter().map(|e| e.key.as_str()).collect();

    let mut only_in_a = Vec::new();
    let mut only_in_b = Vec::new();
    let mut different = Vec::new();
    let mut same = Vec::new();

    // All keys from both files, sorted
    let all_keys: BTreeSet<&str> = keys_a.union(&keys_b).copied().collect();

    for key in all_keys {
        let val_a = a.get(key);
        let val_b = b.get(key);

        match (val_a, val_b) {
            (Some(va), Some(vb)) => {
                if va == vb {
                    same.push((key.to_string(), va.to_string()));
                } else {
                    different.push((key.to_string(), va.to_string(), vb.to_string()));
                }
            }
            (Some(va), None) => {
                only_in_a.push((key.to_string(), va.to_string()));
            }
            (None, Some(vb)) => {
                only_in_b.push((key.to_string(), vb.to_string()));
            }
            (None, None) => unreachable!(),
        }
    }

    DiffResult {
        only_in_a,
        only_in_b,
        different,
        same,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::env_file::EnvFile;
    use std::path::Path;

    fn parse_env(content: &str) -> EnvFile {
        EnvFile::parse(content, Path::new("test.env")).unwrap()
    }

    #[test]
    fn test_diff_identical_files() {
        let a = parse_env("KEY1=value1\nKEY2=value2\n");
        let b = parse_env("KEY1=value1\nKEY2=value2\n");

        let result = diff_env_files(&a, &b);

        assert!(result.only_in_a.is_empty());
        assert!(result.only_in_b.is_empty());
        assert!(result.different.is_empty());
        assert_eq!(result.same.len(), 2);
        assert!(result.is_empty());
    }

    #[test]
    fn test_diff_empty_files() {
        let a = EnvFile::new();
        let b = EnvFile::new();

        let result = diff_env_files(&a, &b);

        assert!(result.only_in_a.is_empty());
        assert!(result.only_in_b.is_empty());
        assert!(result.different.is_empty());
        assert!(result.same.is_empty());
        assert!(result.is_empty());
    }

    #[test]
    fn test_diff_one_sided() {
        let a = parse_env("KEY1=value1\nKEY2=value2\n");
        let b = EnvFile::new();

        let result = diff_env_files(&a, &b);

        assert_eq!(result.only_in_a.len(), 2);
        assert!(result.only_in_b.is_empty());
        assert!(result.different.is_empty());
        assert!(result.same.is_empty());
        assert!(!result.is_empty());

        // Reverse direction
        let result2 = diff_env_files(&b, &a);

        assert!(result2.only_in_a.is_empty());
        assert_eq!(result2.only_in_b.len(), 2);
        assert!(!result2.is_empty());
    }

    #[test]
    fn test_diff_mixed() {
        let a = parse_env("SHARED=same\nDIFF=val_a\nONLY_A=aaa\n");
        let b = parse_env("SHARED=same\nDIFF=val_b\nONLY_B=bbb\n");

        let result = diff_env_files(&a, &b);

        assert_eq!(result.only_in_a.len(), 1);
        assert_eq!(result.only_in_a[0].0, "ONLY_A");
        assert_eq!(result.only_in_a[0].1, "aaa");

        assert_eq!(result.only_in_b.len(), 1);
        assert_eq!(result.only_in_b[0].0, "ONLY_B");
        assert_eq!(result.only_in_b[0].1, "bbb");

        assert_eq!(result.different.len(), 1);
        assert_eq!(result.different[0].0, "DIFF");
        assert_eq!(result.different[0].1, "val_a");
        assert_eq!(result.different[0].2, "val_b");

        assert_eq!(result.same.len(), 1);
        assert_eq!(result.same[0].0, "SHARED");
        assert_eq!(result.same[0].1, "same");

        assert!(!result.is_empty());
    }

    #[test]
    fn test_diff_same_keys_different_values() {
        let a = parse_env("DB_HOST=localhost\nDB_PORT=5432\nDB_NAME=mydb\n");
        let b = parse_env("DB_HOST=staging.db.com\nDB_PORT=5433\nDB_NAME=mydb\n");

        let result = diff_env_files(&a, &b);

        assert!(result.only_in_a.is_empty());
        assert!(result.only_in_b.is_empty());
        assert_eq!(result.different.len(), 2);
        assert_eq!(result.same.len(), 1);
        assert_eq!(result.same[0].0, "DB_NAME");
        assert!(!result.is_empty());
    }
}
