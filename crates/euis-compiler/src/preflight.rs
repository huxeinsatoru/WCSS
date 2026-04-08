use serde::{Deserialize, Serialize};

/// Configuration for the CSS preflight/reset generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreflightConfig {
    /// Whether preflight CSS is enabled at all.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Include modern-normalize base rules.
    #[serde(default = "default_true")]
    pub include_normalize: bool,
    /// Include form element resets.
    #[serde(default = "default_true")]
    pub include_forms: bool,
    /// Include typography defaults.
    #[serde(default = "default_true")]
    pub include_typography: bool,
    /// Override the default font family.
    #[serde(default)]
    pub base_font_family: Option<String>,
    /// Override the default font size.
    #[serde(default)]
    pub base_font_size: Option<String>,
    /// Override the default line height.
    #[serde(default)]
    pub base_line_height: Option<String>,
}

fn default_true() -> bool {
    true
}

impl Default for PreflightConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            include_normalize: true,
            include_forms: true,
            include_typography: true,
            base_font_family: None,
            base_font_size: None,
            base_line_height: None,
        }
    }
}

/// Generate a CSS preflight/reset string based on the given configuration.
///
/// The generated CSS is inspired by Tailwind's Preflight (which builds on
/// modern-normalize) and provides sensible defaults for cross-browser
/// consistency.
pub fn generate_preflight(config: &PreflightConfig) -> String {
    if !config.enabled {
        return String::new();
    }

    let mut sections: Vec<String> = Vec::new();

    // Always include box-sizing and margin/padding reset — these are
    // fundamental and not gated behind a sub-flag.
    sections.push(generate_box_sizing_reset());
    sections.push(generate_margin_padding_reset());

    if config.include_normalize {
        sections.push(generate_normalize(config));
    }

    if config.include_forms {
        sections.push(generate_form_reset());
    }

    sections.push(generate_media_defaults());

    if config.include_typography {
        sections.push(generate_typography_defaults(config));
    }

    sections.push(generate_interactive_defaults());

    // Join non-empty sections with a blank line separator.
    sections
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
}

// ---------------------------------------------------------------------------
// Individual reset sections
// ---------------------------------------------------------------------------

fn generate_box_sizing_reset() -> String {
    r#"/*! euis preflight | MIT License */

/*
 * 1. Use border-box by default, globally.
 * 2. Ensure consistent border style.
 */
*,
::before,
::after {
  box-sizing: border-box; /* 1 */
  border-width: 0; /* 2 */
  border-style: solid; /* 2 */
  border-color: currentColor; /* 2 */
}"#
    .to_string()
}

fn generate_margin_padding_reset() -> String {
    r#"/*
 * Remove default margin and padding from all elements.
 */
* {
  margin: 0;
  padding: 0;
}"#
    .to_string()
}

fn generate_normalize(config: &PreflightConfig) -> String {
    let font_family = config
        .base_font_family
        .as_deref()
        .unwrap_or(
            "ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, \"Segoe UI\", Roboto, \"Helvetica Neue\", Arial, \"Noto Sans\", sans-serif, \"Apple Color Emoji\", \"Segoe UI Emoji\", \"Segoe UI Symbol\", \"Noto Color Emoji\""
        );
    let font_size = config.base_font_size.as_deref().unwrap_or("100%");
    let line_height = config.base_line_height.as_deref().unwrap_or("1.5");

    format!(
        r#"/*
 * Modern normalize base.
 * 1. Correct the line height in all browsers.
 * 2. Prevent adjustments of font size after orientation changes in iOS.
 * 3. Use a more readable tab size.
 * 4. Use the configured font family.
 * 5. Use the configured font feature settings.
 * 6. Use the configured font variation settings.
 * 7. Disable tap highlights on iOS.
 */
html {{
  line-height: {line_height}; /* 1 */
  -webkit-text-size-adjust: 100%; /* 2 */
  -moz-tab-size: 4; /* 3 */
  tab-size: 4; /* 3 */
  font-family: {font_family}; /* 4 */
  font-size: {font_size};
  font-feature-settings: normal; /* 5 */
  font-variation-settings: normal; /* 6 */
  -webkit-tap-highlight-color: transparent; /* 7 */
}}

/*
 * 1. Remove the margin in all browsers.
 * 2. Inherit line-height from `html` so users can set it as a class on `html`.
 */
body {{
  line-height: inherit; /* 2 */
}}

/*
 * Correct the font size and margin on `h1` elements within `section` and
 * `article` contexts in Chrome, Firefox, and Safari.
 */
h1 {{
  font-size: 2em;
}}

/*
 * Add the correct font weight in Chrome and Safari.
 */
b,
strong {{
  font-weight: bolder;
}}

/*
 * 1. Use the configured mono font family.
 * 2. Correct the odd `em` font sizing in all browsers.
 */
code,
kbd,
samp,
pre {{
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace; /* 1 */
  font-size: 1em; /* 2 */
}}

/*
 * Add the correct font size in all browsers.
 */
small {{
  font-size: 80%;
}}

/*
 * Prevent `sub` and `sup` from affecting the line height in all browsers.
 */
sub,
sup {{
  font-size: 75%;
  line-height: 0;
  position: relative;
  vertical-align: baseline;
}}

sub {{
  bottom: -0.25em;
}}

sup {{
  top: -0.5em;
}}

/*
 * 1. Remove text indentation from table contents in Chrome and Safari.
 * 2. Correct table border color inheritance in Chrome and Safari.
 * 3. Remove gaps between table borders by default.
 */
table {{
  text-indent: 0; /* 1 */
  border-color: inherit; /* 2 */
  border-collapse: collapse; /* 3 */
}}

/*
 * Add the correct text decoration in Chrome, Edge, and Safari.
 */
abbr:where([title]) {{
  text-decoration: underline dotted;
}}

/*
 * Ensure horizontal rules are visible by default.
 */
hr {{
  height: 0;
  color: inherit;
  border-top-width: 1px;
}}"#
    )
}

fn generate_form_reset() -> String {
    r#"/*
 * Form element resets.
 * 1. Change the font styles in all browsers.
 * 2. Remove the margin in Firefox and Safari.
 */
button,
input,
optgroup,
select,
textarea {
  font-family: inherit; /* 1 */
  font-feature-settings: inherit; /* 1 */
  font-variation-settings: inherit; /* 1 */
  font-size: 100%; /* 1 */
  font-weight: inherit; /* 1 */
  line-height: inherit; /* 1 */
  letter-spacing: inherit;
  color: inherit;
}

/*
 * Remove default button styles.
 */
button,
[type='button'],
[type='reset'],
[type='submit'] {
  -webkit-appearance: button;
  background-color: transparent;
  background-image: none;
}

/*
 * Use the modern Firefox focus style for all focusable elements.
 */
:-moz-focusring {
  outline: auto;
}

/*
 * Remove the additional `:invalid` styles in Firefox.
 */
:-moz-ui-invalid {
  box-shadow: none;
}

/*
 * Correct the cursor style of increment and decrement buttons in Safari.
 */
::-webkit-inner-spin-button,
::-webkit-outer-spin-button {
  height: auto;
}

/*
 * 1. Correct the odd appearance in Chrome and Safari.
 * 2. Correct the outline style in Safari.
 */
[type='search'] {
  -webkit-appearance: textfield; /* 1 */
  outline-offset: -2px; /* 2 */
}

/*
 * Remove the inner padding in Chrome and Safari on macOS.
 */
::-webkit-search-decoration {
  -webkit-appearance: none;
}

/*
 * 1. Correct the inability to style clickable types in iOS and Safari.
 * 2. Change font properties to `inherit` in Safari.
 */
::-webkit-file-upload-button {
  -webkit-appearance: button; /* 1 */
  font: inherit; /* 2 */
}

/*
 * Add the correct display in Chrome and Safari.
 */
summary {
  display: list-item;
}

/*
 * Make textareas only resizable vertically by default.
 */
textarea {
  resize: vertical;
}

/*
 * Reset the default placeholder opacity in Firefox.
 */
::placeholder {
  opacity: 1;
  color: #9ca3af;
}

/*
 * Set the default cursor for disabled elements.
 */
:disabled {
  cursor: default;
}"#
    .to_string()
}

fn generate_media_defaults() -> String {
    r#"/*
 * 1. Make replaced elements display: block by default.
 * 2. Add `vertical-align: middle` to align replaced elements more sensibly.
 */
img,
svg,
video,
canvas,
audio,
iframe,
embed,
object {
  display: block; /* 1 */
  vertical-align: middle; /* 2 */
}

/*
 * Constrain images and videos to the parent width and preserve their
 * intrinsic aspect ratio.
 */
img,
video {
  max-width: 100%;
  height: auto;
}"#
    .to_string()
}

fn generate_typography_defaults(config: &PreflightConfig) -> String {
    let line_height = config.base_line_height.as_deref().unwrap_or("1.5");

    format!(
        r#"/*
 * Typography defaults.
 */
h1,
h2,
h3,
h4,
h5,
h6 {{
  font-size: inherit;
  font-weight: inherit;
}}

/*
 * Reset links to optimise opt-in rather than opt-out.
 */
a {{
  color: inherit;
  text-decoration: inherit;
}}

/*
 * Remove list styles (bullets/numbers) by default.
 */
ol,
ul,
menu {{
  list-style: none;
}}

/*
 * Prevent resizing textareas horizontally by default.
 */
textarea {{
  resize: vertical;
}}

/*
 * Ensure the default line-height is applied to paragraphs.
 */
p {{
  line-height: {line_height};
}}

/*
 * Ensure the cursor changes to indicate clickable targets.
 */
[role="button"],
button {{
  cursor: pointer;
}}

/*
 * Make sure disabled elements don't get the pointer cursor.
 */
:disabled {{
  cursor: default;
}}"#
    )
}

fn generate_interactive_defaults() -> String {
    r#"/*
 * Ensure hidden elements are actually hidden.
 */
[hidden] {
  display: none !important;
}"#
    .to_string()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disabled_produces_empty_string() {
        let config = PreflightConfig {
            enabled: false,
            ..Default::default()
        };
        let css = generate_preflight(&config);
        assert!(css.is_empty());
    }

    #[test]
    fn test_default_config_includes_box_sizing() {
        let css = generate_preflight(&PreflightConfig::default());
        assert!(css.contains("box-sizing: border-box"));
    }

    #[test]
    fn test_default_config_includes_margin_reset() {
        let css = generate_preflight(&PreflightConfig::default());
        assert!(css.contains("margin: 0"));
        assert!(css.contains("padding: 0"));
    }

    #[test]
    fn test_normalize_section_present() {
        let css = generate_preflight(&PreflightConfig::default());
        assert!(css.contains("-webkit-text-size-adjust: 100%"));
        assert!(css.contains("tab-size: 4"));
    }

    #[test]
    fn test_custom_font_family() {
        let config = PreflightConfig {
            base_font_family: Some("Inter, sans-serif".to_string()),
            ..Default::default()
        };
        let css = generate_preflight(&config);
        assert!(css.contains("font-family: Inter, sans-serif"));
    }

    #[test]
    fn test_custom_font_size() {
        let config = PreflightConfig {
            base_font_size: Some("16px".to_string()),
            ..Default::default()
        };
        let css = generate_preflight(&config);
        assert!(css.contains("font-size: 16px"));
    }

    #[test]
    fn test_custom_line_height() {
        let config = PreflightConfig {
            base_line_height: Some("1.75".to_string()),
            ..Default::default()
        };
        let css = generate_preflight(&config);
        assert!(css.contains("line-height: 1.75"));
    }

    #[test]
    fn test_form_reset_included() {
        let css = generate_preflight(&PreflightConfig::default());
        assert!(css.contains("button,\ninput,"));
        assert!(css.contains("-webkit-appearance: button"));
        assert!(css.contains("background-color: transparent"));
    }

    #[test]
    fn test_form_reset_excluded() {
        let config = PreflightConfig {
            include_forms: false,
            ..Default::default()
        };
        let css = generate_preflight(&config);
        // Form-specific selectors should be absent.
        assert!(!css.contains("::-webkit-search-decoration"));
        assert!(!css.contains("::-webkit-file-upload-button"));
    }

    #[test]
    fn test_media_defaults_present() {
        let css = generate_preflight(&PreflightConfig::default());
        assert!(css.contains("img,"));
        assert!(css.contains("display: block"));
        assert!(css.contains("max-width: 100%"));
        assert!(css.contains("height: auto"));
    }

    #[test]
    fn test_typography_defaults_present() {
        let css = generate_preflight(&PreflightConfig::default());
        assert!(css.contains("h1,\nh2,"));
        assert!(css.contains("text-decoration: inherit"));
        assert!(css.contains("list-style: none"));
    }

    #[test]
    fn test_typography_excluded() {
        let config = PreflightConfig {
            include_typography: false,
            ..Default::default()
        };
        let css = generate_preflight(&config);
        assert!(!css.contains("list-style: none"));
    }

    #[test]
    fn test_hidden_rule_always_present() {
        let css = generate_preflight(&PreflightConfig::default());
        assert!(css.contains("[hidden]"));
        assert!(css.contains("display: none !important"));
    }

    #[test]
    fn test_normalize_excluded() {
        let config = PreflightConfig {
            include_normalize: false,
            ..Default::default()
        };
        let css = generate_preflight(&config);
        assert!(!css.contains("tab-size: 4"));
        assert!(!css.contains("-webkit-text-size-adjust"));
    }

    #[test]
    fn test_serde_roundtrip() {
        let config = PreflightConfig {
            enabled: true,
            include_normalize: false,
            include_forms: true,
            include_typography: false,
            base_font_family: Some("Georgia, serif".to_string()),
            base_font_size: Some("18px".to_string()),
            base_line_height: Some("1.6".to_string()),
        };
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: PreflightConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.enabled, config.enabled);
        assert_eq!(deserialized.include_normalize, config.include_normalize);
        assert_eq!(deserialized.base_font_family, config.base_font_family);
        assert_eq!(deserialized.base_font_size, config.base_font_size);
        assert_eq!(deserialized.base_line_height, config.base_line_height);
    }

    #[test]
    fn test_default_serde_deserialization() {
        // An empty JSON object should produce a valid config with all
        // boolean fields defaulting to true.
        let config: PreflightConfig = serde_json::from_str("{}").unwrap();
        assert!(config.enabled);
        assert!(config.include_normalize);
        assert!(config.include_forms);
        assert!(config.include_typography);
        assert!(config.base_font_family.is_none());
    }
}
