//! Content scanning for automatic class name extraction.
//!
//! This module scans source files (HTML, JSX, TSX, Vue, Svelte) to automatically
//! detect which CSS classes are used, enabling automatic tree-shaking without
//! manual configuration.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use regex::Regex;
use glob::glob;

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Configuration for content scanning.
#[derive(Debug, Clone)]
pub struct ScanConfig {
    /// Glob patterns for files to scan (e.g., "src/**/*.{js,jsx,tsx,vue}").
    pub patterns: Vec<String>,
    /// Regex patterns for safelist (classes matching these are always kept).
    pub safelist_patterns: Vec<Regex>,
    /// Whether to scan recursively.
    pub recursive: bool,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            patterns: vec![
                "**/*.html".to_string(),
                "**/*.jsx".to_string(),
                "**/*.tsx".to_string(),
                "**/*.vue".to_string(),
                "**/*.svelte".to_string(),
            ],
            safelist_patterns: Vec::new(),
            recursive: true,
        }
    }
}

// ---------------------------------------------------------------------------
// Scanner
// ---------------------------------------------------------------------------

/// Content scanner that extracts class names from source files.
pub struct ContentScanner {
    config: ScanConfig,
    /// Regex for extracting class names from various syntaxes.
    class_extractors: Vec<ClassExtractor>,
}

/// A regex-based class name extractor with a name for debugging.
struct ClassExtractor {
    name: &'static str,
    regex: Regex,
    /// Which capture group contains the class names (default: 1).
    capture_group: usize,
}

impl ContentScanner {
    /// Create a new content scanner with the given configuration.
    pub fn new(config: ScanConfig) -> Self {
        let class_extractors = vec![
            // HTML/JSX: class="btn primary"
            ClassExtractor {
                name: "class attribute",
                regex: Regex::new(r#"class(?:Name)?=["']([^"']+)["']"#).unwrap(),
                capture_group: 1,
            },
            // HTML/JSX: class={`btn ${variant}`}
            ClassExtractor {
                name: "template literal",
                regex: Regex::new(r#"class(?:Name)?=\{`([^`]+)`\}"#).unwrap(),
                capture_group: 1,
            },
            // Vue: :class="['btn', 'primary']"
            ClassExtractor {
                name: "vue array class",
                regex: Regex::new(r#":class="\[([^\]]+)\]"#).unwrap(),
                capture_group: 1,
            },
            // Svelte: class:btn={true}
            ClassExtractor {
                name: "svelte class directive",
                regex: Regex::new(r#"class:([a-zA-Z0-9_-]+)"#).unwrap(),
                capture_group: 1,
            },
            // JSX: className={'btn'}
            ClassExtractor {
                name: "jsx single quote",
                regex: Regex::new(r#"className=\{'([^']+)'\}"#).unwrap(),
                capture_group: 1,
            },
            // Tailwind @apply: @apply btn primary
            ClassExtractor {
                name: "css apply",
                regex: Regex::new(r#"@apply\s+([^;]+)"#).unwrap(),
                capture_group: 1,
            },
            // clsx/classnames: clsx('btn', 'primary')
            ClassExtractor {
                name: "clsx function",
                regex: Regex::new(r#"(?:clsx|classnames|cn)\s*\([^)]*['"]([^'"]+)['"]"#).unwrap(),
                capture_group: 1,
            },
        ];

        Self {
            config,
            class_extractors,
        }
    }

    /// Scan all configured paths and return a set of used class names.
    pub fn scan(&self) -> Result<HashSet<String>, ScanError> {
        let mut classes = HashSet::new();

        // Resolve all glob patterns to file paths
        let files = self.resolve_files()?;

        // Scan each file
        for file in files {
            match self.scan_file(&file) {
                Ok(file_classes) => {
                    classes.extend(file_classes);
                }
                Err(e) => {
                    eprintln!("Warning: Failed to scan {}: {}", file.display(), e);
                }
            }
        }

        // Add safelist patterns (these are kept even if not found)
        // Note: Safelist patterns are applied during tree-shaking, not here

        Ok(classes)
    }

    /// Resolve glob patterns to actual file paths.
    fn resolve_files(&self) -> Result<Vec<PathBuf>, ScanError> {
        let mut files = Vec::new();

        for pattern in &self.config.patterns {
            match glob(pattern) {
                Ok(paths) => {
                    for entry in paths {
                        match entry {
                            Ok(path) => {
                                if path.is_file() {
                                    files.push(path);
                                }
                            }
                            Err(e) => {
                                eprintln!("Warning: Glob error: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    return Err(ScanError::GlobPattern(format!(
                        "Invalid glob pattern '{}': {}",
                        pattern, e
                    )));
                }
            }
        }

        Ok(files)
    }

    /// Scan a single file and extract class names.
    fn scan_file(&self, path: &Path) -> Result<HashSet<String>, ScanError> {
        let content = fs::read_to_string(path)
            .map_err(|e| ScanError::FileRead(path.to_path_buf(), e))?;

        Ok(self.extract_classes(&content))
    }

    /// Extract class names from file content using all extractors.
    fn extract_classes(&self, content: &str) -> HashSet<String> {
        let mut classes = HashSet::new();

        for extractor in &self.class_extractors {
            for cap in extractor.regex.captures_iter(content) {
                if let Some(matched) = cap.get(extractor.capture_group) {
                    let class_string = matched.as_str();
                    // Split by whitespace and filter out empty strings
                    for class in class_string.split_whitespace() {
                        // Clean up quotes, commas, and other noise
                        let cleaned = class
                            .trim_matches(|c: char| {
                                c == '\'' || c == '"' || c == ',' || c == '{' || c == '}'
                            })
                            .trim();

                        if !cleaned.is_empty() && is_valid_class_name(cleaned) {
                            classes.insert(cleaned.to_string());
                        }
                    }
                }
            }
        }

        classes
    }

    /// Check if a class name matches any safelist pattern.
    pub fn matches_safelist(&self, class_name: &str) -> bool {
        self.config
            .safelist_patterns
            .iter()
            .any(|pattern| pattern.is_match(class_name))
    }
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

/// Check if a string is a valid CSS class name.
fn is_valid_class_name(s: &str) -> bool {
    // Must not be empty
    if s.is_empty() {
        return false;
    }

    // Must not start with a digit
    if s.chars().next().unwrap().is_ascii_digit() {
        return false;
    }

    // Must only contain valid CSS identifier characters (including colon for pseudo-classes)
    s.chars().all(|c| {
        c.is_alphanumeric() || c == '-' || c == '_' || c == ':'
    })
}

// ---------------------------------------------------------------------------
// Error handling
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum ScanError {
    GlobPattern(String),
    FileRead(PathBuf, std::io::Error),
}

impl std::fmt::Display for ScanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScanError::GlobPattern(msg) => write!(f, "Glob pattern error: {}", msg),
            ScanError::FileRead(path, err) => {
                write!(f, "Failed to read {}: {}", path.display(), err)
            }
        }
    }
}

impl std::error::Error for ScanError {}

// ---------------------------------------------------------------------------
// Public API helpers
// ---------------------------------------------------------------------------

/// Scan content paths and return used class names.
///
/// This is a convenience function that creates a scanner with default config
/// and scans the given paths.
pub fn scan_content_paths(patterns: Vec<String>) -> Result<HashSet<String>, ScanError> {
    let config = ScanConfig {
        patterns,
        ..Default::default()
    };
    let scanner = ContentScanner::new(config);
    scanner.scan()
}

/// Scan content paths with safelist patterns.
pub fn scan_with_safelist(
    patterns: Vec<String>,
    safelist_patterns: Vec<String>,
) -> Result<HashSet<String>, ScanError> {
    let safelist_regexes: Vec<Regex> = safelist_patterns
        .iter()
        .filter_map(|pattern| Regex::new(pattern).ok())
        .collect();

    let config = ScanConfig {
        patterns,
        safelist_patterns: safelist_regexes,
        recursive: true,
    };

    let scanner = ContentScanner::new(config);
    scanner.scan()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_class_name() {
        assert!(is_valid_class_name("btn"));
        assert!(is_valid_class_name("btn-primary"));
        assert!(is_valid_class_name("btn_large"));
        assert!(is_valid_class_name("text-2xl"));

        assert!(!is_valid_class_name(""));
        assert!(!is_valid_class_name("2xl"));
        assert!(!is_valid_class_name("btn primary")); // contains space
    }

    #[test]
    fn test_extract_classes_html() {
        let scanner = ContentScanner::new(ScanConfig::default());
        let content = r#"<div class="btn primary">Click me</div>"#;
        let classes = scanner.extract_classes(content);

        assert!(classes.contains("btn"));
        assert!(classes.contains("primary"));
        assert_eq!(classes.len(), 2);
    }

    #[test]
    fn test_extract_classes_jsx() {
        let scanner = ContentScanner::new(ScanConfig::default());
        let content = r#"<button className="btn-large hover:bg-blue">Click</button>"#;
        let classes = scanner.extract_classes(content);

        assert!(classes.contains("btn-large"));
        assert!(classes.contains("hover:bg-blue"));
    }

    #[test]
    fn test_extract_classes_template_literal() {
        let scanner = ContentScanner::new(ScanConfig::default());
        let content = r#"<div className={`btn ${variant}`}>Text</div>"#;
        let classes = scanner.extract_classes(content);

        assert!(classes.contains("btn"));
        // Note: ${variant} is not extracted as it's dynamic
    }

    #[test]
    fn test_extract_classes_vue() {
        let scanner = ContentScanner::new(ScanConfig::default());
        let content = r#"<div :class="['btn', 'primary']">Text</div>"#;
        let classes = scanner.extract_classes(content);

        assert!(classes.contains("btn"));
        assert!(classes.contains("primary"));
    }

    #[test]
    fn test_extract_classes_svelte() {
        let scanner = ContentScanner::new(ScanConfig::default());
        let content = r#"<div class:active={isActive} class:btn>Text</div>"#;
        let classes = scanner.extract_classes(content);

        assert!(classes.contains("active"));
        assert!(classes.contains("btn"));
    }

    #[test]
    fn test_extract_classes_clsx() {
        let scanner = ContentScanner::new(ScanConfig::default());
        let content = r#"const classes = clsx('btn', 'primary');"#;
        let classes = scanner.extract_classes(content);

        assert!(classes.contains("btn"));
        assert!(classes.contains("primary"));
    }

    #[test]
    fn test_safelist_matching() {
        let config = ScanConfig {
            patterns: vec![],
            safelist_patterns: vec![
                Regex::new(r"^btn-").unwrap(),
                Regex::new(r"^text-").unwrap(),
            ],
            recursive: true,
        };

        let scanner = ContentScanner::new(config);

        assert!(scanner.matches_safelist("btn-primary"));
        assert!(scanner.matches_safelist("btn-large"));
        assert!(scanner.matches_safelist("text-xl"));
        assert!(!scanner.matches_safelist("card"));
        assert!(!scanner.matches_safelist("primary"));
    }
}
