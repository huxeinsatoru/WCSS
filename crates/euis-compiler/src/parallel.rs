//! Parallel processing utilities for the Euis compiler.
//!
//! Uses Rayon to parallelize compilation, parsing, and optimization
//! across multiple files or stylesheets.

use std::path::PathBuf;

use rayon::prelude::*;

use crate::ast::StyleSheet;
use crate::config::CompilerConfig;
use crate::error::CompileResult;
use crate::{compile, optimizer, parser};

/// Compile multiple Euis source files in parallel using Rayon's `par_iter`.
///
/// Each file is read from disk and compiled independently. The results are
/// returned in the same order as the input paths.
pub fn parallel_compile_files(files: Vec<PathBuf>, config: &CompilerConfig) -> Vec<CompileResult> {
    files
        .par_iter()
        .map(|path| {
            match std::fs::read_to_string(path) {
                Ok(source) => compile(&source, config),
                Err(err) => CompileResult {
                    css: String::new(),
                    js: None,
                    source_map: None,
                    errors: vec![crate::error::CompilerError::syntax(
                        format!("Failed to read file {}: {}", path.display(), err),
                        crate::ast::Span::empty(),
                    )],
                    warnings: Vec::new(),
                    stats: crate::error::CompilationStats::default(),
                },
            }
        })
        .collect()
}

/// Optimize multiple stylesheets in parallel.
///
/// Each stylesheet is optimized independently using the provided configuration.
/// Results are returned in the same order as the input stylesheets.
pub fn parallel_optimize(
    stylesheets: Vec<StyleSheet>,
    config: &CompilerConfig,
) -> Vec<StyleSheet> {
    stylesheets
        .into_par_iter()
        .map(|stylesheet| optimizer::optimize(stylesheet, config))
        .collect()
}

/// Parse multiple Euis source strings in parallel.
///
/// Each source string is parsed independently. On parse failure the error
/// is converted into an empty stylesheet so the Vec always has the same
/// length as the input.
pub fn parallel_parse(sources: Vec<String>) -> Vec<StyleSheet> {
    sources
        .par_iter()
        .map(|source| {
            parser::parse(source).unwrap_or_else(|_| StyleSheet {
                rules: Vec::new(),
                at_rules: Vec::new(),
                span: crate::ast::Span::empty(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_config() -> CompilerConfig {
        CompilerConfig {
            minify: false,
            ..Default::default()
        }
    }

    #[test]
    fn test_parallel_parse_basic() {
        let sources = vec![
            ".a { color: red; }".to_string(),
            ".b { margin: 0; }".to_string(),
            ".c { padding: 10px; }".to_string(),
        ];

        let results = parallel_parse(sources);
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].rules.len(), 1);
        assert_eq!(results[1].rules.len(), 1);
        assert_eq!(results[2].rules.len(), 1);
    }

    #[test]
    fn test_parallel_parse_with_errors() {
        let sources = vec![
            ".valid { color: red; }".to_string(),
            "{{{{ invalid".to_string(),
        ];

        let results = parallel_parse(sources);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].rules.len(), 1);
        // Failed parse returns empty stylesheet
        assert_eq!(results[1].rules.len(), 0);
    }

    #[test]
    fn test_parallel_optimize_basic() {
        let sources = vec![
            ".a { color: red; }".to_string(),
            ".b { margin: 0; }".to_string(),
        ];

        let stylesheets: Vec<StyleSheet> = sources
            .iter()
            .map(|s| parser::parse(s).unwrap())
            .collect();

        let config = default_config();
        let results = parallel_optimize(stylesheets, &config);
        assert_eq!(results.len(), 2);
    }
}
