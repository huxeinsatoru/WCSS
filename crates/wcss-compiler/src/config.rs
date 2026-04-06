use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;


/// Main compiler configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilerConfig {
    pub tokens: DesignTokens,
    pub minify: bool,
    pub source_maps: SourceMapConfig,
    pub typed_om: bool,
    pub tree_shaking: bool,
    /// Enable rule deduplication (merge rules with identical declarations).
    #[serde(default = "default_true")]
    pub deduplicate: bool,
    pub output_path: Option<PathBuf>,
    /// List of used class names for tree shaking.
    #[serde(default)]
    pub used_classes: Vec<String>,
    /// Source file paths to scan for class usage.
    #[serde(default)]
    pub content_paths: Vec<PathBuf>,
    /// Class name patterns to always preserve (glob patterns like "btn-*" or regex like "/^dynamic-/").
    #[serde(default)]
    pub safelist: Vec<String>,
    /// Path to W3C Design Tokens JSON file (optional).
    #[serde(default)]
    pub w3c_tokens_path: Option<PathBuf>,
    /// Platform target for W3C token code generation.
    #[serde(default)]
    pub platform_target: PlatformTarget,
}

fn default_true() -> bool {
    true
}

impl Default for CompilerConfig {
    fn default() -> Self {
        Self {
            tokens: DesignTokens::default(),
            minify: true,
            source_maps: SourceMapConfig::Disabled,
            typed_om: false,
            tree_shaking: false,
            deduplicate: true,
            output_path: None,
            used_classes: Vec::new(),
            content_paths: Vec::new(),
            safelist: Vec::new(),
            w3c_tokens_path: None,
            platform_target: PlatformTarget::CSS,
        }
    }
}

/// Design token definitions for colors, spacing, typography, and breakpoints.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DesignTokens {
    #[serde(default)]
    pub colors: HashMap<String, TokenValue>,
    #[serde(default)]
    pub spacing: HashMap<String, TokenValue>,
    #[serde(default)]
    pub typography: HashMap<String, TokenValue>,
    #[serde(default)]
    pub breakpoints: HashMap<String, TokenValue>,
}

impl DesignTokens {
    /// Look up a token by category and name.
    pub fn get(&self, category: &crate::ast::TokenCategory, name: &str) -> Option<&TokenValue> {
        use crate::ast::TokenCategory;
        match category {
            TokenCategory::Colors => self.colors.get(name),
            TokenCategory::Spacing => self.spacing.get(name),
            TokenCategory::Typography => self.typography.get(name),
            TokenCategory::Breakpoints => self.breakpoints.get(name),
            TokenCategory::Animation => None, // Not yet implemented
            TokenCategory::Custom => self.spacing.get(name), // Fallback to spacing
        }
    }
}

/// A token value can be a literal string or a reference to another token.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TokenValue {
    Literal(String),
    Reference(String), // "$category.name" format
}

impl TokenValue {
    pub fn as_literal(&self) -> Option<&str> {
        match self {
            TokenValue::Literal(s) => Some(s),
            TokenValue::Reference(_) => None,
        }
    }
}

/// Source map output configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceMapConfig {
    Disabled,
    Inline,
    External,
}

/// Platform target for W3C Design Tokens code generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PlatformTarget {
    /// Generate CSS custom properties
    #[default]
    CSS,
    /// Generate iOS Swift code
    IOS,
    /// Generate Android XML resources
    Android,
    /// Generate Android Kotlin code
    AndroidKotlin,
    /// Generate Flutter Dart code
    Flutter,
    /// Generate TypeScript type definitions
    TypeScript,
    /// Generate HTML documentation
    Docs,
}
