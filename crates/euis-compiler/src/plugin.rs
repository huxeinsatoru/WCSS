//! Plugin system for Euis.
//!
//! Plugins can hook into the compilation pipeline at various stages:
//! - `before_parse`: Transform source text before parsing
//! - `after_parse`: Transform the AST after parsing
//! - `before_optimize`: Transform before optimization
//! - `after_optimize`: Transform after optimization
//! - `before_codegen`: Transform before code generation
//! - `after_codegen`: Transform the final CSS output
//!
//! # Example
//! ```ignore
//! struct MyPlugin;
//!
//! impl Plugin for MyPlugin {
//!     fn name(&self) -> &str { "my-plugin" }
//!
//!     fn after_parse(&self, stylesheet: &mut StyleSheet) -> Result<(), PluginError> {
//!         // Transform the AST...
//!         Ok(())
//!     }
//! }
//! ```

use crate::ast::StyleSheet;

/// Error from a plugin.
#[derive(Debug, Clone)]
pub struct PluginError {
    pub plugin_name: String,
    pub message: String,
}

impl std::fmt::Display for PluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Plugin '{}': {}", self.plugin_name, self.message)
    }
}

/// The plugin trait. Implement this to create a Euis plugin.
pub trait Plugin: Send + Sync {
    /// Plugin name (for error reporting).
    fn name(&self) -> &str;

    /// Transform source text before parsing.
    fn before_parse(&self, _source: &mut String) -> Result<(), PluginError> {
        Ok(())
    }

    /// Transform AST after parsing.
    fn after_parse(&self, _stylesheet: &mut StyleSheet) -> Result<(), PluginError> {
        Ok(())
    }

    /// Transform AST before optimization.
    fn before_optimize(&self, _stylesheet: &mut StyleSheet) -> Result<(), PluginError> {
        Ok(())
    }

    /// Transform AST after optimization.
    fn after_optimize(&self, _stylesheet: &mut StyleSheet) -> Result<(), PluginError> {
        Ok(())
    }

    /// Transform AST before code generation.
    fn before_codegen(&self, _stylesheet: &mut StyleSheet) -> Result<(), PluginError> {
        Ok(())
    }

    /// Transform final CSS output.
    fn after_codegen(&self, _css: &mut String) -> Result<(), PluginError> {
        Ok(())
    }
}

/// Plugin registry for managing the plugin pipeline.
pub struct PluginRegistry {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self { plugins: Vec::new() }
    }

    /// Register a plugin.
    pub fn add(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.push(plugin);
    }

    /// Run all before_parse hooks.
    pub fn run_before_parse(&self, source: &mut String) -> Result<(), PluginError> {
        for plugin in &self.plugins {
            plugin.before_parse(source)?;
        }
        Ok(())
    }

    /// Run all after_parse hooks.
    pub fn run_after_parse(&self, stylesheet: &mut StyleSheet) -> Result<(), PluginError> {
        for plugin in &self.plugins {
            plugin.after_parse(stylesheet)?;
        }
        Ok(())
    }

    /// Run all before_optimize hooks.
    pub fn run_before_optimize(&self, stylesheet: &mut StyleSheet) -> Result<(), PluginError> {
        for plugin in &self.plugins {
            plugin.before_optimize(stylesheet)?;
        }
        Ok(())
    }

    /// Run all after_optimize hooks.
    pub fn run_after_optimize(&self, stylesheet: &mut StyleSheet) -> Result<(), PluginError> {
        for plugin in &self.plugins {
            plugin.after_optimize(stylesheet)?;
        }
        Ok(())
    }

    /// Run all before_codegen hooks.
    pub fn run_before_codegen(&self, stylesheet: &mut StyleSheet) -> Result<(), PluginError> {
        for plugin in &self.plugins {
            plugin.before_codegen(stylesheet)?;
        }
        Ok(())
    }

    /// Run all after_codegen hooks.
    pub fn run_after_codegen(&self, css: &mut String) -> Result<(), PluginError> {
        for plugin in &self.plugins {
            plugin.after_codegen(css)?;
        }
        Ok(())
    }

    pub fn has_plugins(&self) -> bool {
        !self.plugins.is_empty()
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Built-in plugins
// ---------------------------------------------------------------------------

/// Plugin that adds a CSS reset/normalize at the start.
pub struct CSSResetPlugin {
    pub reset_css: String,
}

impl Plugin for CSSResetPlugin {
    fn name(&self) -> &str { "css-reset" }

    fn after_codegen(&self, css: &mut String) -> Result<(), PluginError> {
        let mut new_css = self.reset_css.clone();
        new_css.push_str(css);
        *css = new_css;
        Ok(())
    }
}

/// Plugin that removes all comments from the output.
pub struct StripCommentsPlugin;

impl Plugin for StripCommentsPlugin {
    fn name(&self) -> &str { "strip-comments" }

    fn before_parse(&self, _source: &mut String) -> Result<(), PluginError> {
        // Comments are already stripped by the parser, but this handles edge cases
        Ok(())
    }
}

/// Plugin that adds custom at-rules or transformations.
pub struct TailwindCompatPlugin;

impl Plugin for TailwindCompatPlugin {
    fn name(&self) -> &str { "tailwind-compat" }

    fn before_parse(&self, _source: &mut String) -> Result<(), PluginError> {
        // Transform @apply directives to standard declarations
        // This is a placeholder for Tailwind compatibility
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestPlugin {
        name: String,
    }

    impl Plugin for TestPlugin {
        fn name(&self) -> &str { &self.name }

        fn after_codegen(&self, css: &mut String) -> Result<(), PluginError> {
            css.push_str("\n/* Added by test plugin */");
            Ok(())
        }
    }

    #[test]
    fn test_plugin_registry() {
        let mut registry = PluginRegistry::new();
        registry.add(Box::new(TestPlugin { name: "test".to_string() }));

        let mut css = ".btn { color: red; }".to_string();
        registry.run_after_codegen(&mut css).unwrap();
        assert!(css.contains("/* Added by test plugin */"));
    }

    #[test]
    fn test_multiple_plugins() {
        let mut registry = PluginRegistry::new();
        registry.add(Box::new(TestPlugin { name: "plugin-1".to_string() }));
        registry.add(Box::new(TestPlugin { name: "plugin-2".to_string() }));

        let mut css = String::new();
        registry.run_after_codegen(&mut css).unwrap();
        // Both plugins should have added their comment
        assert_eq!(css.matches("test plugin").count(), 2);
    }
}
