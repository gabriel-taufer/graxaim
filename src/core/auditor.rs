use crate::errors::Result;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

lazy_static! {
    // JavaScript/TypeScript
    static ref JS_PROCESS_ENV: Regex =
        Regex::new(r#"process\.env\.([A-Z_][A-Z0-9_]*)"#).unwrap();
    static ref JS_PROCESS_ENV_BRACKET: Regex =
        Regex::new(r#"process\.env\[['"]([A-Z_][A-Z0-9_]*)['"]\]"#).unwrap();
    static ref JS_IMPORT_META_ENV: Regex =
        Regex::new(r#"import\.meta\.env\.([A-Z_][A-Z0-9_]*)"#).unwrap();

    // Python
    static ref PY_OS_ENVIRON: Regex =
        Regex::new(r#"os\.environ\[['"]([A-Z_][A-Z0-9_]*)['"]\]"#).unwrap();
    static ref PY_OS_GETENV: Regex =
        Regex::new(r#"os\.getenv\(['"]([A-Z_][A-Z0-9_]*)['"]\)"#).unwrap();
    static ref PY_ENVIRON_GET: Regex =
        Regex::new(r#"os\.environ\.get\(['"]([A-Z_][A-Z0-9_]*)['"]\)"#).unwrap();

    // Rust
    static ref RUST_ENV_VAR: Regex =
        Regex::new(r#"env::var\(['"]([A-Z_][A-Z0-9_]*)['"]\)"#).unwrap();
    static ref RUST_ENV_MACRO: Regex =
        Regex::new(r#"env!\(['"]([A-Z_][A-Z0-9_]*)['"]\)"#).unwrap();

    // Go
    static ref GO_GETENV: Regex =
        Regex::new(r#"os\.Getenv\(['"]([A-Z_][A-Z0-9_]*)['"]\)"#).unwrap();

    // Ruby
    static ref RUBY_ENV: Regex =
        Regex::new(r#"ENV\[['"]([A-Z_][A-Z0-9_]*)['"]\]"#).unwrap();
    static ref RUBY_ENV_FETCH: Regex =
        Regex::new(r#"ENV\.fetch\(['"]([A-Z_][A-Z0-9_]*)['"]\)"#).unwrap();

    // PHP
    static ref PHP_GETENV: Regex =
        Regex::new(r#"getenv\(['"]([A-Z_][A-Z0-9_]*)['"]\)"#).unwrap();
    static ref PHP_ENV: Regex =
        Regex::new(r#"\$_ENV\[['"]([A-Z_][A-Z0-9_]*)['"]\]"#).unwrap();

    // Docker/Generic (shell interpolation)
    static ref DOCKER_VAR: Regex =
        Regex::new(r#"\$\{([A-Z_][A-Z0-9_]*)\}"#).unwrap();

    /// All patterns collected for a single scan pass
    static ref ALL_PATTERNS: Vec<&'static Regex> = vec![
        &JS_PROCESS_ENV,
        &JS_PROCESS_ENV_BRACKET,
        &JS_IMPORT_META_ENV,
        &PY_OS_ENVIRON,
        &PY_OS_GETENV,
        &PY_ENVIRON_GET,
        &RUST_ENV_VAR,
        &RUST_ENV_MACRO,
        &GO_GETENV,
        &RUBY_ENV,
        &RUBY_ENV_FETCH,
        &PHP_GETENV,
        &PHP_ENV,
        &DOCKER_VAR,
    ];
}

/// Directories to skip during scanning
const SKIP_DIRS: &[&str] = &[
    "node_modules",
    "target",
    ".git",
    "vendor",
    "__pycache__",
    ".venv",
    "venv",
    "dist",
    "build",
    ".next",
    ".nuxt",
    "coverage",
    ".graxaim",
];

/// File extensions to scan
const SCAN_EXTENSIONS: &[&str] = &[
    "js", "ts", "jsx", "tsx", "mjs", "cjs", "py", "pyw", "rs", "go", "rb", "erb", "php", "yml",
    "yaml", "env", "envrc",
];

/// A reference to an env var found in source code
#[derive(Debug, Clone)]
pub struct CodeReference {
    pub var_name: String,
    pub file: PathBuf,
    pub line: usize,
}

/// A var present in profiles but not referenced in code
#[derive(Debug, Clone)]
pub struct ProfileReference {
    pub var_name: String,
    /// Which profiles contain this var
    pub profiles: Vec<String>,
}

/// Result of a full codebase audit
#[derive(Debug)]
pub struct AuditResult {
    /// Vars referenced in code but absent from all profiles
    pub in_code_missing_from_profiles: Vec<CodeReference>,
    /// Vars in profiles but not referenced anywhere in code
    pub in_profiles_not_in_code: Vec<ProfileReference>,
    /// Number of source files scanned
    pub files_scanned: usize,
}

/// Run a full audit: scan source files and cross-reference with profile vars.
///
/// `profile_vars` maps profile name → set of var keys defined in that profile.
pub fn audit(
    project_root: &Path,
    profile_vars: &HashMap<String, HashSet<String>>,
) -> Result<AuditResult> {
    let files = walk_directory(project_root);
    let files_scanned = files.len();

    // Collect every code reference: (var_name → Vec<CodeReference>)
    let mut code_refs: HashMap<String, Vec<CodeReference>> = HashMap::new();

    for file in &files {
        for (var_name, line_no) in scan_file(file) {
            code_refs
                .entry(var_name.clone())
                .or_default()
                .push(CodeReference {
                    var_name: var_name.clone(),
                    file: file.clone(),
                    line: line_no,
                });
        }
    }

    // Build a flat set of all vars that exist in at least one profile
    let mut all_profile_vars: HashMap<String, Vec<String>> = HashMap::new();
    for (profile_name, vars) in profile_vars {
        for var in vars {
            all_profile_vars
                .entry(var.clone())
                .or_default()
                .push(profile_name.clone());
        }
    }

    // 1. Vars in code but missing from all profiles
    let mut in_code_missing_from_profiles: Vec<CodeReference> = Vec::new();
    for (var_name, refs) in &code_refs {
        if !all_profile_vars.contains_key(var_name) {
            // Use only the first occurrence to keep output concise
            if let Some(first) = refs.first() {
                in_code_missing_from_profiles.push(first.clone());
            }
        }
    }
    in_code_missing_from_profiles.sort_by(|a, b| a.var_name.cmp(&b.var_name));

    // 2. Vars in profiles but not referenced in code
    let mut in_profiles_not_in_code: Vec<ProfileReference> = Vec::new();
    for (var_name, mut profiles) in all_profile_vars {
        if !code_refs.contains_key(&var_name) {
            profiles.sort();
            in_profiles_not_in_code.push(ProfileReference { var_name, profiles });
        }
    }
    in_profiles_not_in_code.sort_by(|a, b| a.var_name.cmp(&b.var_name));

    Ok(AuditResult {
        in_code_missing_from_profiles,
        in_profiles_not_in_code,
        files_scanned,
    })
}

/// Scan a single file for env var references.
/// Returns a vec of `(var_name, line_number)` pairs (1-based).
pub fn scan_file(path: &Path) -> Vec<(String, usize)> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Vec::new(), // skip binary / unreadable files
    };

    let mut results = Vec::new();

    for (line_idx, line) in content.lines().enumerate() {
        let line_no = line_idx + 1;
        for pattern in ALL_PATTERNS.iter() {
            for cap in pattern.captures_iter(line) {
                if let Some(m) = cap.get(1) {
                    results.push((m.as_str().to_string(), line_no));
                }
            }
        }
    }

    results
}

/// Recursively walk `root`, skipping [`SKIP_DIRS`], and return files whose
/// extensions are in [`SCAN_EXTENSIONS`].
pub fn walk_directory(root: &Path) -> Vec<PathBuf> {
    let mut result = Vec::new();
    walk_inner(root, &mut result);
    result
}

fn walk_inner(dir: &Path, result: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_dir() {
            let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if SKIP_DIRS.contains(&dir_name) {
                continue;
            }
            walk_inner(&path, result);
        } else if path.is_file() {
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            // Also match extensionless files like ".envrc" by checking the filename
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if SCAN_EXTENSIONS.contains(&ext) || SCAN_EXTENSIONS.contains(&file_name) {
                result.push(path);
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Unit tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn tmp_file(dir: &TempDir, name: &str, content: &str) -> PathBuf {
        let path = dir.path().join(name);
        fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn test_scan_javascript() {
        let tmp = TempDir::new().unwrap();
        let path = tmp_file(
            &tmp,
            "app.js",
            r#"
const key = process.env.API_KEY;
const url = process.env['DATABASE_URL'];
const flag = import.meta.env.VITE_FLAG;
"#,
        );

        let results = scan_file(&path);
        let names: Vec<&str> = results.iter().map(|(n, _)| n.as_str()).collect();
        assert!(
            names.contains(&"API_KEY"),
            "should detect process.env.API_KEY"
        );
        assert!(
            names.contains(&"DATABASE_URL"),
            "should detect process.env['DATABASE_URL']"
        );
        assert!(
            names.contains(&"VITE_FLAG"),
            "should detect import.meta.env.VITE_FLAG"
        );
    }

    #[test]
    fn test_scan_python() {
        let tmp = TempDir::new().unwrap();
        let path = tmp_file(
            &tmp,
            "app.py",
            r#"
import os
secret = os.getenv("SECRET_KEY")
url = os.environ["DATABASE_URL"]
flag = os.environ.get("FEATURE_FLAG")
"#,
        );

        let results = scan_file(&path);
        let names: Vec<&str> = results.iter().map(|(n, _)| n.as_str()).collect();
        assert!(names.contains(&"SECRET_KEY"), "should detect os.getenv");
        assert!(
            names.contains(&"DATABASE_URL"),
            "should detect os.environ[]"
        );
        assert!(
            names.contains(&"FEATURE_FLAG"),
            "should detect os.environ.get"
        );
    }

    #[test]
    fn test_scan_rust() {
        let tmp = TempDir::new().unwrap();
        let path = tmp_file(
            &tmp,
            "main.rs",
            r#"
let key = std::env::var("API_KEY").unwrap();
let compile_time = env!("BUILD_HASH");
"#,
        );

        let results = scan_file(&path);
        let names: Vec<&str> = results.iter().map(|(n, _)| n.as_str()).collect();
        assert!(names.contains(&"API_KEY"), "should detect env::var");
        assert!(names.contains(&"BUILD_HASH"), "should detect env! macro");
    }

    #[test]
    fn test_scan_docker() {
        let tmp = TempDir::new().unwrap();
        let path = tmp_file(
            &tmp,
            "docker-compose.yml",
            r#"
environment:
  - DATABASE_URL=${DATABASE_URL}
  - REDIS_URL=${REDIS_URL}
"#,
        );

        let results = scan_file(&path);
        let names: Vec<&str> = results.iter().map(|(n, _)| n.as_str()).collect();
        assert!(
            names.contains(&"DATABASE_URL"),
            "should detect ${{DATABASE_URL}}"
        );
        assert!(names.contains(&"REDIS_URL"), "should detect ${{REDIS_URL}}");
    }

    #[test]
    fn test_skip_directories() {
        let tmp = TempDir::new().unwrap();

        // Create a file inside node_modules
        let nm = tmp.path().join("node_modules");
        fs::create_dir_all(&nm).unwrap();
        fs::write(nm.join("lib.js"), "process.env.SHOULD_SKIP\n").unwrap();

        // Create a legit source file
        fs::write(tmp.path().join("index.js"), "process.env.SHOULD_FIND\n").unwrap();

        let files = walk_directory(tmp.path());
        let file_names: Vec<&str> = files
            .iter()
            .filter_map(|p| p.file_name()?.to_str())
            .collect();

        assert!(
            file_names.contains(&"index.js"),
            "should include top-level source files"
        );
        // node_modules should be skipped, so the file inside it should not appear
        let has_nm_file = files.iter().any(|p| p.starts_with(&nm));
        assert!(!has_nm_file, "should skip node_modules directory");
    }

    #[test]
    fn test_audit_cross_reference() {
        let tmp = TempDir::new().unwrap();

        // Source file references API_KEY and DATABASE_URL
        fs::write(
            tmp.path().join("server.js"),
            "const k = process.env.API_KEY;\nconst u = process.env.DATABASE_URL;\n",
        )
        .unwrap();

        // Profile only has DATABASE_URL (API_KEY is missing)
        let mut profile_vars: HashMap<String, HashSet<String>> = HashMap::new();
        let mut local_vars = HashSet::new();
        local_vars.insert("DATABASE_URL".to_string());
        local_vars.insert("DEAD_VAR".to_string()); // in profile, not in code
        profile_vars.insert("local".to_string(), local_vars);

        let result = audit(tmp.path(), &profile_vars).unwrap();

        let missing_names: Vec<&str> = result
            .in_code_missing_from_profiles
            .iter()
            .map(|r| r.var_name.as_str())
            .collect();
        assert!(
            missing_names.contains(&"API_KEY"),
            "API_KEY should be missing from profiles"
        );
        assert!(
            !missing_names.contains(&"DATABASE_URL"),
            "DATABASE_URL is in profile, should not be flagged"
        );

        let dead_names: Vec<&str> = result
            .in_profiles_not_in_code
            .iter()
            .map(|r| r.var_name.as_str())
            .collect();
        assert!(
            dead_names.contains(&"DEAD_VAR"),
            "DEAD_VAR should be flagged as dead"
        );
        assert!(
            !dead_names.contains(&"DATABASE_URL"),
            "DATABASE_URL is in code, should not be flagged as dead"
        );
    }
}
