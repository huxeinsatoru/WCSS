use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// How themes are applied in the generated CSS.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ThemeStrategy {
    /// Use CSS custom properties on `:root` and override selectors.
    CSSVariables,
    /// Switch themes via a `data-*` attribute (e.g. `[data-theme="dark"]`).
    DataAttribute(String),
    /// Switch themes via a CSS class (e.g. `.theme-dark`).
    ClassName(String),
    /// Use `prefers-color-scheme` media queries for light/dark switching.
    MediaQuery,
}

impl Default for ThemeStrategy {
    fn default() -> Self {
        ThemeStrategy::CSSVariables
    }
}

/// Top-level theme configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    /// Named themes keyed by identifier (e.g. "light", "dark", "brand").
    pub themes: HashMap<String, Theme>,
    /// Which theme name is the default (rendered on `:root`).
    pub default_theme: String,
    /// Strategy used to apply/switch themes in CSS.
    #[serde(default)]
    pub strategy: ThemeStrategy,
}

/// A single theme definition containing design-token maps.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Theme {
    /// Color tokens (e.g. "primary" -> "#3b82f6").
    #[serde(default)]
    pub colors: HashMap<String, String>,
    /// Spacing tokens (e.g. "sm" -> "0.5rem").
    #[serde(default)]
    pub spacing: HashMap<String, String>,
    /// Typography tokens (e.g. "body" -> "1rem/1.5 sans-serif").
    #[serde(default)]
    pub typography: HashMap<String, String>,
    /// Shadow tokens (e.g. "md" -> "0 4px 6px rgba(0,0,0,.1)").
    #[serde(default)]
    pub shadows: HashMap<String, String>,
    /// Border tokens (e.g. "default" -> "1px solid #e5e7eb").
    #[serde(default)]
    pub borders: HashMap<String, String>,
    /// Arbitrary custom CSS variables.
    #[serde(default)]
    pub custom: HashMap<String, String>,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Generate the complete theme CSS for every theme in the configuration.
///
/// The default theme's variables are placed on `:root`. Non-default themes
/// are scoped according to the chosen [`ThemeStrategy`].
pub fn generate_theme_css(config: &ThemeConfig) -> String {
    let mut parts: Vec<String> = Vec::new();

    // Render the default theme first, on :root.
    if let Some(default_theme) = config.themes.get(&config.default_theme) {
        parts.push(generate_theme_variables(default_theme, ":root"));
    }

    // Collect and sort non-default theme names for deterministic output.
    let mut other_names: Vec<&String> = config
        .themes
        .keys()
        .filter(|name| *name != &config.default_theme)
        .collect();
    other_names.sort();

    for name in other_names {
        let theme = &config.themes[name];
        let selector = build_selector(name, &config.strategy);
        parts.push(generate_theme_variables(theme, &selector));
    }

    parts.join("\n\n")
}

/// Generate CSS variable declarations for a single theme inside the given
/// selector.
///
/// ```css
/// :root {
///   --color-primary: #3b82f6;
///   --spacing-sm: 0.5rem;
/// }
/// ```
pub fn generate_theme_variables(theme: &Theme, selector: &str) -> String {
    let mut declarations: Vec<String> = Vec::new();

    append_prefixed_vars(&mut declarations, "color", &theme.colors);
    append_prefixed_vars(&mut declarations, "spacing", &theme.spacing);
    append_prefixed_vars(&mut declarations, "typography", &theme.typography);
    append_prefixed_vars(&mut declarations, "shadow", &theme.shadows);
    append_prefixed_vars(&mut declarations, "border", &theme.borders);
    append_prefixed_vars(&mut declarations, "", &theme.custom);

    if declarations.is_empty() {
        return format!("{} {{}}", selector);
    }

    let body = declarations
        .iter()
        .map(|d| format!("  {}", d))
        .collect::<Vec<_>>()
        .join("\n");

    format!("{} {{\n{}\n}}", selector, body)
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Build the CSS selector string for a non-default theme based on the
/// strategy.
fn build_selector(theme_name: &str, strategy: &ThemeStrategy) -> String {
    match strategy {
        ThemeStrategy::CSSVariables => {
            // For plain CSS-variable strategy, non-default themes use a
            // data-theme attribute by convention.
            format!("[data-theme=\"{}\"]", theme_name)
        }
        ThemeStrategy::DataAttribute(attr) => {
            format!("[{}=\"{}\"]", attr, theme_name)
        }
        ThemeStrategy::ClassName(prefix) => {
            if prefix.is_empty() {
                format!(".{}", theme_name)
            } else {
                format!(".{}-{}", prefix, theme_name)
            }
        }
        ThemeStrategy::MediaQuery => {
            // Map well-known theme names to prefers-color-scheme values.
            let scheme = match theme_name {
                "dark" => "dark",
                "light" => "light",
                other => other,
            };
            format!("@media (prefers-color-scheme: {})", scheme)
        }
    }
}

/// Collect CSS variable declarations from a token map, sorting keys for
/// deterministic output.
fn append_prefixed_vars(
    out: &mut Vec<String>,
    prefix: &str,
    map: &HashMap<String, String>,
) {
    let mut keys: Vec<&String> = map.keys().collect();
    keys.sort();

    for key in keys {
        let var_name = if prefix.is_empty() {
            format!("--{}", key)
        } else {
            format!("--{}-{}", prefix, key)
        };
        out.push(format!("{}: {};", var_name, map[key]));
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: build a minimal light/dark theme config.
    fn sample_config(strategy: ThemeStrategy) -> ThemeConfig {
        let mut light = Theme::default();
        light.colors.insert("primary".into(), "#3b82f6".into());
        light.colors.insert("secondary".into(), "#8b5cf6".into());
        light.spacing.insert("sm".into(), "0.5rem".into());

        let mut dark = Theme::default();
        dark.colors.insert("primary".into(), "#60a5fa".into());
        dark.colors.insert("secondary".into(), "#a78bfa".into());
        dark.spacing.insert("sm".into(), "0.5rem".into());

        let mut themes = HashMap::new();
        themes.insert("light".into(), light);
        themes.insert("dark".into(), dark);

        ThemeConfig {
            themes,
            default_theme: "light".into(),
            strategy,
        }
    }

    // -- Basic generation tests --

    #[test]
    fn test_css_variables_strategy() {
        let config = sample_config(ThemeStrategy::CSSVariables);
        let css = generate_theme_css(&config);

        // Default theme on :root
        assert!(css.contains(":root {"));
        assert!(css.contains("--color-primary: #3b82f6;"));
        assert!(css.contains("--color-secondary: #8b5cf6;"));
        assert!(css.contains("--spacing-sm: 0.5rem;"));

        // Dark theme uses data-theme attribute
        assert!(css.contains("[data-theme=\"dark\"]"));
        assert!(css.contains("--color-primary: #60a5fa;"));
    }

    #[test]
    fn test_data_attribute_strategy() {
        let config = sample_config(ThemeStrategy::DataAttribute("data-mode".into()));
        let css = generate_theme_css(&config);

        assert!(css.contains(":root {"));
        assert!(css.contains("[data-mode=\"dark\"]"));
    }

    #[test]
    fn test_class_name_strategy() {
        let config = sample_config(ThemeStrategy::ClassName("theme".into()));
        let css = generate_theme_css(&config);

        assert!(css.contains(":root {"));
        assert!(css.contains(".theme-dark {"));
    }

    #[test]
    fn test_class_name_strategy_empty_prefix() {
        let config = sample_config(ThemeStrategy::ClassName("".into()));
        let css = generate_theme_css(&config);

        assert!(css.contains(".dark {"));
    }

    #[test]
    fn test_media_query_strategy() {
        let config = sample_config(ThemeStrategy::MediaQuery);
        let css = generate_theme_css(&config);

        assert!(css.contains(":root {"));
        assert!(css.contains("@media (prefers-color-scheme: dark)"));
    }

    // -- Single theme variable generation --

    #[test]
    fn test_generate_theme_variables_basic() {
        let mut theme = Theme::default();
        theme.colors.insert("primary".into(), "#ff0000".into());
        theme.spacing.insert("md".into(), "1rem".into());

        let css = generate_theme_variables(&theme, ":root");
        assert!(css.contains(":root {"));
        assert!(css.contains("--color-primary: #ff0000;"));
        assert!(css.contains("--spacing-md: 1rem;"));
    }

    #[test]
    fn test_generate_theme_variables_all_categories() {
        let mut theme = Theme::default();
        theme.colors.insert("bg".into(), "#fff".into());
        theme.spacing.insert("xs".into(), "0.25rem".into());
        theme.typography.insert("body".into(), "1rem".into());
        theme.shadows.insert("sm".into(), "0 1px 2px rgba(0,0,0,0.05)".into());
        theme.borders.insert("default".into(), "1px solid #e5e7eb".into());
        theme.custom.insert("radius".into(), "0.375rem".into());

        let css = generate_theme_variables(&theme, ".test");
        assert!(css.contains("--color-bg: #fff;"));
        assert!(css.contains("--spacing-xs: 0.25rem;"));
        assert!(css.contains("--typography-body: 1rem;"));
        assert!(css.contains("--shadow-sm: 0 1px 2px rgba(0,0,0,0.05);"));
        assert!(css.contains("--border-default: 1px solid #e5e7eb;"));
        assert!(css.contains("--radius: 0.375rem;"));
    }

    #[test]
    fn test_empty_theme_produces_empty_block() {
        let theme = Theme::default();
        let css = generate_theme_variables(&theme, ":root");
        assert_eq!(css, ":root {}");
    }

    #[test]
    fn test_custom_vars_no_prefix() {
        let mut theme = Theme::default();
        theme.custom.insert("my-var".into(), "42px".into());
        let css = generate_theme_variables(&theme, ":root");
        assert!(css.contains("--my-var: 42px;"));
    }

    // -- Deterministic ordering --

    #[test]
    fn test_variables_are_sorted_within_category() {
        let mut theme = Theme::default();
        theme.colors.insert("zebra".into(), "#000".into());
        theme.colors.insert("alpha".into(), "#fff".into());

        let css = generate_theme_variables(&theme, ":root");
        let alpha_pos = css.find("--color-alpha").unwrap();
        let zebra_pos = css.find("--color-zebra").unwrap();
        assert!(
            alpha_pos < zebra_pos,
            "Variables should be sorted alphabetically"
        );
    }

    #[test]
    fn test_non_default_themes_sorted() {
        let mut themes = HashMap::new();
        themes.insert("light".into(), Theme::default());

        let mut dark = Theme::default();
        dark.colors.insert("bg".into(), "#000".into());
        themes.insert("dark".into(), dark);

        let mut brand = Theme::default();
        brand.colors.insert("bg".into(), "#00f".into());
        themes.insert("brand".into(), brand);

        let config = ThemeConfig {
            themes,
            default_theme: "light".into(),
            strategy: ThemeStrategy::CSSVariables,
        };
        let css = generate_theme_css(&config);

        // "brand" should come after "dark" alphabetically (b < d).
        let brand_pos = css.find("data-theme=\"brand\"").unwrap();
        let dark_pos = css.find("data-theme=\"dark\"").unwrap();
        assert!(
            brand_pos < dark_pos,
            "Themes should be sorted alphabetically: brand before dark"
        );
    }

    // -- Serde tests --

    #[test]
    fn test_theme_config_serde_roundtrip() {
        let config = sample_config(ThemeStrategy::DataAttribute("data-theme".into()));
        let json = serde_json::to_string_pretty(&config).unwrap();
        let deserialized: ThemeConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.default_theme, "light");
        assert_eq!(
            deserialized.strategy,
            ThemeStrategy::DataAttribute("data-theme".into())
        );
        assert!(deserialized.themes.contains_key("light"));
        assert!(deserialized.themes.contains_key("dark"));
    }

    #[test]
    fn test_theme_strategy_default() {
        let strategy: ThemeStrategy = Default::default();
        assert_eq!(strategy, ThemeStrategy::CSSVariables);
    }

    #[test]
    fn test_serde_strategy_variants() {
        let variants = vec![
            ThemeStrategy::CSSVariables,
            ThemeStrategy::DataAttribute("data-mode".into()),
            ThemeStrategy::ClassName("theme".into()),
            ThemeStrategy::MediaQuery,
        ];
        for v in variants {
            let json = serde_json::to_string(&v).unwrap();
            let back: ThemeStrategy = serde_json::from_str(&json).unwrap();
            assert_eq!(back, v);
        }
    }

    // -- Edge cases --

    #[test]
    fn test_missing_default_theme() {
        let config = ThemeConfig {
            themes: HashMap::new(),
            default_theme: "nonexistent".into(),
            strategy: ThemeStrategy::CSSVariables,
        };
        // Should not panic, just produces empty output.
        let css = generate_theme_css(&config);
        assert!(css.is_empty());
    }

    #[test]
    fn test_single_theme_only() {
        let mut themes = HashMap::new();
        let mut light = Theme::default();
        light.colors.insert("bg".into(), "#fff".into());
        themes.insert("light".into(), light);

        let config = ThemeConfig {
            themes,
            default_theme: "light".into(),
            strategy: ThemeStrategy::CSSVariables,
        };
        let css = generate_theme_css(&config);
        assert!(css.contains(":root {"));
        assert!(css.contains("--color-bg: #fff;"));
        // No other selectors.
        assert!(!css.contains("[data-theme"));
    }

    #[test]
    fn test_media_query_custom_scheme_name() {
        let mut themes = HashMap::new();
        themes.insert("light".into(), Theme::default());

        let mut high_contrast = Theme::default();
        high_contrast
            .colors
            .insert("bg".into(), "#000".into());
        themes.insert("high-contrast".into(), high_contrast);

        let config = ThemeConfig {
            themes,
            default_theme: "light".into(),
            strategy: ThemeStrategy::MediaQuery,
        };
        let css = generate_theme_css(&config);
        assert!(css.contains("@media (prefers-color-scheme: high-contrast)"));
    }
}
