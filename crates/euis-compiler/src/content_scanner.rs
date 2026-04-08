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
    #[allow(dead_code)]
    name: &'static str,
    regex: Regex,
    /// Which capture group contains the class names (default: 1).
    capture_group: usize,
}

// ---------------------------------------------------------------------------
// Standalone helper functions for deep extraction
// ---------------------------------------------------------------------------

/// Extract all string literals (single-quoted, double-quoted, and backtick) from
/// a block of code and return those that look like valid CSS class names.
pub fn extract_string_literals(content: &str) -> HashSet<String> {
    let re = Regex::new(r#"(?:'([^']*)'|"([^"]*)")"#).unwrap();
    let mut classes = HashSet::new();

    for cap in re.captures_iter(content) {
        let value = cap.get(1).or_else(|| cap.get(2));
        if let Some(m) = value {
            for token in m.as_str().split_whitespace() {
                if is_valid_class_name(token) {
                    classes.insert(token.to_string());
                }
            }
        }
    }

    classes
}

/// Extract the body of a balanced function call starting at the opening paren.
/// Returns `None` if no balanced match is found.
fn extract_balanced_parens(content: &str, start: usize) -> Option<&str> {
    let bytes = content.as_bytes();
    if start >= bytes.len() || bytes[start] != b'(' {
        return None;
    }
    let mut depth: usize = 0;
    for (i, &b) in bytes[start..].iter().enumerate() {
        match b {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    // Return the inner content (excluding outer parens)
                    return Some(&content[start + 1..start + i]);
                }
            }
            _ => {}
        }
    }
    None
}

/// Deep extraction of class names from dynamic utility-function calls.
///
/// This handles:
/// - clsx / classnames / cn / cva / twMerge / twJoin / tv calls with balanced
///   parentheses (including multi-line and nested parens).
/// - Object-syntax keys: `{ 'btn-active': condition }` extracts `btn-active`.
/// - Ternary expressions: `cond ? 'a' : 'b'` extracts both `a` and `b`.
/// - All quoted string literals within the call body.
pub fn extract_from_dynamic_patterns(content: &str) -> HashSet<String> {
    let mut classes = HashSet::new();

    // Match any of the supported function names followed by `(`
    let fn_re = Regex::new(
        r"(?:clsx|classnames|cn|cva|twMerge|twJoin|tv)\s*\("
    ).unwrap();

    for m in fn_re.find_iter(content) {
        // `m.end() - 1` points to the opening paren
        let paren_start = m.end() - 1;
        if let Some(body) = extract_balanced_parens(content, paren_start) {
            // Extract every quoted string literal from the body
            classes.extend(extract_string_literals(body));

            // Extract object keys that look like class names: { 'key': ... }
            let obj_key_re = Regex::new(
                r#"(?:'([^']+)'|"([^"]+)")\s*:"#
            ).unwrap();
            for cap in obj_key_re.captures_iter(body) {
                let value = cap.get(1).or_else(|| cap.get(2));
                if let Some(v) = value {
                    for token in v.as_str().split_whitespace() {
                        if is_valid_class_name(token) {
                            classes.insert(token.to_string());
                        }
                    }
                }
            }
        }
    }

    classes
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

    /// Extract class names from file content using all extractors plus deep
    /// dynamic-pattern extraction.
    fn extract_classes(&self, content: &str) -> HashSet<String> {
        let mut classes = HashSet::new();

        // 1. Regex-based extractors for static patterns
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

        // 2. Deep extraction from dynamic utility-function calls
        classes.extend(extract_from_dynamic_patterns(content));

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

    // Must only contain valid CSS identifier characters.
    // Includes colon (pseudo-classes / Tailwind modifiers), forward-slash
    // (fractional values like w-1/2), brackets (arbitrary values like bg-[#fff]),
    // hash (colors inside brackets), dot (decimal values), percent, and comma
    // (for multi-value arbitrary properties).
    s.chars().all(|c| {
        c.is_alphanumeric()
            || c == '-'
            || c == '_'
            || c == ':'
            || c == '/'
            || c == '['
            || c == ']'
            || c == '#'
            || c == '.'
            || c == '%'
            || c == ','
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
    fn test_clsx_nested_parens() {
        let scanner = ContentScanner::new(ScanConfig::default());
        let content = r#"clsx('btn', isActive && 'active', isBig && 'large')"#;
        let classes = scanner.extract_classes(content);

        assert!(classes.contains("btn"));
        assert!(classes.contains("active"));
        assert!(classes.contains("large"));
    }

    #[test]
    fn test_clsx_multiline() {
        let scanner = ContentScanner::new(ScanConfig::default());
        let content = "clsx(\n  'flex',\n  'items-center',\n  isOpen && 'visible'\n)";
        let classes = scanner.extract_classes(content);

        assert!(classes.contains("flex"));
        assert!(classes.contains("items-center"));
        assert!(classes.contains("visible"));
    }

    #[test]
    fn test_clsx_object_syntax() {
        let scanner = ContentScanner::new(ScanConfig::default());
        let content = r#"clsx({ 'btn-active': isActive, 'btn-disabled': isDisabled })"#;
        let classes = scanner.extract_classes(content);

        assert!(classes.contains("btn-active"));
        assert!(classes.contains("btn-disabled"));
    }

    #[test]
    fn test_cn_ternary() {
        let scanner = ContentScanner::new(ScanConfig::default());
        let content = r#"cn(condition ? 'class-a' : 'class-b', 'always')"#;
        let classes = scanner.extract_classes(content);

        assert!(classes.contains("class-a"));
        assert!(classes.contains("class-b"));
        assert!(classes.contains("always"));
    }

    #[test]
    fn test_cva_extraction() {
        let scanner = ContentScanner::new(ScanConfig::default());
        let content = r#"cva('base-class flex', { variants: { size: { sm: 'text-sm p-1', lg: 'text-lg p-4' } } })"#;
        let classes = scanner.extract_classes(content);

        assert!(classes.contains("base-class"));
        assert!(classes.contains("flex"));
        assert!(classes.contains("text-sm"));
        assert!(classes.contains("p-1"));
        assert!(classes.contains("text-lg"));
        assert!(classes.contains("p-4"));
    }

    #[test]
    fn test_cva_multiline() {
        let scanner = ContentScanner::new(ScanConfig::default());
        let content = r#"const button = cva(
  'inline-flex items-center',
  {
    variants: {
      intent: {
        primary: 'bg-blue-500 text-white',
        secondary: 'bg-gray-200 text-gray-800',
      },
      size: {
        sm: 'text-sm px-2',
        lg: 'text-lg px-6',
      },
    },
  }
)"#;
        let classes = scanner.extract_classes(content);

        assert!(classes.contains("inline-flex"));
        assert!(classes.contains("items-center"));
        assert!(classes.contains("bg-blue-500"));
        assert!(classes.contains("text-white"));
        assert!(classes.contains("bg-gray-200"));
        assert!(classes.contains("text-gray-800"));
        assert!(classes.contains("text-sm"));
        assert!(classes.contains("px-2"));
        assert!(classes.contains("text-lg"));
        assert!(classes.contains("px-6"));
    }

    #[test]
    fn test_tw_merge() {
        let scanner = ContentScanner::new(ScanConfig::default());
        let content = r#"twMerge('p-4 bg-red-500', 'bg-blue-500')"#;
        let classes = scanner.extract_classes(content);

        assert!(classes.contains("p-4"));
        assert!(classes.contains("bg-red-500"));
        assert!(classes.contains("bg-blue-500"));
    }

    #[test]
    fn test_tw_join() {
        let scanner = ContentScanner::new(ScanConfig::default());
        let content = r#"twJoin('flex', isOpen && 'block')"#;
        let classes = scanner.extract_classes(content);

        assert!(classes.contains("flex"));
        assert!(classes.contains("block"));
    }

    #[test]
    fn test_tv_extraction() {
        let scanner = ContentScanner::new(ScanConfig::default());
        let content = r#"const card = tv({ base: 'rounded-lg shadow', variants: { color: { primary: 'bg-blue-500' } } })"#;
        let classes = scanner.extract_classes(content);

        assert!(classes.contains("rounded-lg"));
        assert!(classes.contains("shadow"));
        assert!(classes.contains("bg-blue-500"));
    }

    #[test]
    fn test_extract_string_literals() {
        let result = extract_string_literals(r#"'btn', "primary", 'text-lg hover:underline'"#);

        assert!(result.contains("btn"));
        assert!(result.contains("primary"));
        assert!(result.contains("text-lg"));
        assert!(result.contains("hover:underline"));
    }

    #[test]
    fn test_extract_from_dynamic_patterns_standalone() {
        let content = r#"
            const cls = clsx('base', isActive && 'active');
            const merged = twMerge('p-4', props.className);
            const variant = cva('btn', { variants: { size: { sm: 'text-sm' } } });
        "#;
        let classes = extract_from_dynamic_patterns(content);

        assert!(classes.contains("base"));
        assert!(classes.contains("active"));
        assert!(classes.contains("p-4"));
        assert!(classes.contains("btn"));
        assert!(classes.contains("text-sm"));
    }

    #[test]
    fn test_clsx_template_literal_with_ternary() {
        let scanner = ContentScanner::new(ScanConfig::default());
        let content = r#"cn('flex', isOpen ? 'block' : 'hidden')"#;
        let classes = scanner.extract_classes(content);

        assert!(classes.contains("flex"));
        assert!(classes.contains("block"));
        assert!(classes.contains("hidden"));
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
