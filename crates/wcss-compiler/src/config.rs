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
    /// Browser targets for vendor prefixing.
    #[serde(default)]
    pub browser_targets: BrowserTargets,
    /// Enable vendor prefixing.
    #[serde(default)]
    pub autoprefixer: bool,
    /// Enable CSS shorthand property merging.
    #[serde(default)]
    pub merge_shorthands: bool,
    /// Dark mode strategy.
    #[serde(default)]
    pub dark_mode: DarkModeStrategy,
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
            browser_targets: BrowserTargets::default(),
            autoprefixer: false,
            merge_shorthands: false,
            dark_mode: DarkModeStrategy::Media,
        }
    }
}

// ---------------------------------------------------------------------------
// Design Tokens
// ---------------------------------------------------------------------------

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
    #[serde(default)]
    pub shadows: HashMap<String, TokenValue>,
    #[serde(default)]
    pub borders: HashMap<String, TokenValue>,
    #[serde(default)]
    pub radii: HashMap<String, TokenValue>,
    #[serde(default)]
    pub zindex: HashMap<String, TokenValue>,
    #[serde(default)]
    pub opacity: HashMap<String, TokenValue>,
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
            TokenCategory::Animation => None,
            TokenCategory::Shadows => self.shadows.get(name),
            TokenCategory::Borders => self.borders.get(name),
            TokenCategory::Radii => self.radii.get(name),
            TokenCategory::ZIndex => self.zindex.get(name),
            TokenCategory::Opacity => self.opacity.get(name),
            TokenCategory::Custom => self.spacing.get(name),
        }
    }

    /// Get all token names for a category (for validation suggestions).
    pub fn get_category_keys(&self, category: &crate::ast::TokenCategory) -> Vec<String> {
        use crate::ast::TokenCategory;
        match category {
            TokenCategory::Colors => self.colors.keys().cloned().collect(),
            TokenCategory::Spacing => self.spacing.keys().cloned().collect(),
            TokenCategory::Typography => self.typography.keys().cloned().collect(),
            TokenCategory::Breakpoints => self.breakpoints.keys().cloned().collect(),
            TokenCategory::Animation => Vec::new(),
            TokenCategory::Shadows => self.shadows.keys().cloned().collect(),
            TokenCategory::Borders => self.borders.keys().cloned().collect(),
            TokenCategory::Radii => self.radii.keys().cloned().collect(),
            TokenCategory::ZIndex => self.zindex.keys().cloned().collect(),
            TokenCategory::Opacity => self.opacity.keys().cloned().collect(),
            TokenCategory::Custom => self.spacing.keys().cloned().collect(),
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

// ---------------------------------------------------------------------------
// Source Maps
// ---------------------------------------------------------------------------

/// Source map output configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceMapConfig {
    Disabled,
    Inline,
    External,
}

// ---------------------------------------------------------------------------
// Platform Targets
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Browser Targets (for vendor prefixing)
// ---------------------------------------------------------------------------

/// Browser version targets for vendor prefix decisions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserTargets {
    /// Chrome version (e.g., 80)
    #[serde(default)]
    pub chrome: Option<u32>,
    /// Firefox version (e.g., 80)
    #[serde(default)]
    pub firefox: Option<u32>,
    /// Safari version (e.g., 14)
    #[serde(default)]
    pub safari: Option<u32>,
    /// Edge version (e.g., 80)
    #[serde(default)]
    pub edge: Option<u32>,
    /// iOS Safari version
    #[serde(default)]
    pub ios: Option<u32>,
    /// Android Chrome version
    #[serde(default)]
    pub android: Option<u32>,
    /// Samsung Internet version
    #[serde(default)]
    pub samsung: Option<u32>,
    /// Opera version
    #[serde(default)]
    pub opera: Option<u32>,
}

impl Default for BrowserTargets {
    fn default() -> Self {
        Self {
            chrome: None,
            firefox: None,
            safari: None,
            edge: None,
            ios: None,
            android: None,
            samsung: None,
            opera: None,
        }
    }
}

impl BrowserTargets {
    /// Create targets that need all prefixes (for maximum compat).
    pub fn default_with_prefixes() -> Self {
        Self {
            chrome: Some(80),
            firefox: Some(78),
            safari: Some(13),
            edge: Some(80),
            ios: Some(13),
            android: Some(80),
            samsung: Some(12),
            opera: Some(67),
        }
    }

    /// Create targets where no prefixes are needed.
    pub fn none() -> Self {
        Self {
            chrome: None,
            firefox: None,
            safari: None,
            edge: None,
            ios: None,
            android: None,
            samsung: None,
            opera: None,
        }
    }

    /// Whether any -webkit- prefix is needed.
    pub fn needs_webkit(&self) -> bool {
        // Safari and iOS Safari are the main drivers for -webkit-
        self.safari.map_or(false, |v| v <= 16)
            || self.ios.map_or(false, |v| v <= 16)
            || self.samsung.map_or(false, |v| v <= 20)
            || self.chrome.map_or(false, |v| v <= 90)
            || self.android.map_or(false, |v| v <= 90)
    }

    /// Whether any -moz- prefix is needed.
    pub fn needs_moz(&self) -> bool {
        self.firefox.map_or(false, |v| v <= 100)
    }

    /// Whether any -ms- prefix is needed.
    pub fn needs_ms(&self) -> bool {
        self.edge.map_or(false, |v| v <= 18) // EdgeHTML, not Chromium Edge
    }

    /// Whether any -o- prefix is needed.
    pub fn needs_o(&self) -> bool {
        self.opera.map_or(false, |v| v <= 15) // Pre-Blink Opera
    }
}

// ---------------------------------------------------------------------------
// Dark Mode
// ---------------------------------------------------------------------------

/// Dark mode strategy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DarkModeStrategy {
    /// Use @media (prefers-color-scheme: dark)
    #[default]
    Media,
    /// Use a CSS class on root element (e.g., .dark, [data-theme="dark"])
    Class(String),
    /// Use data attribute selector (e.g., [data-mode="dark"])
    Attribute(String),
}
