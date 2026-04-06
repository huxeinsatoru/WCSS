use crate::w3c_parser::{W3CToken, W3CTokenType, W3CTokenValue};
use std::collections::HashMap;

/// Generator for CSS custom properties from W3C Design Tokens.
pub struct CSSGenerator;

impl CSSGenerator {
    /// Generate CSS custom properties from W3C tokens.
    /// 
    /// # Arguments
    /// * `tokens` - The W3C tokens to convert to CSS
    /// * `minify` - Whether to minify the output (remove whitespace)
    /// 
    /// # Returns
    /// A string containing CSS custom properties wrapped in a :root selector
    pub fn generate(tokens: &[W3CToken], minify: bool) -> String {
        if tokens.is_empty() {
            return if minify {
                ":root{}".to_string()
            } else {
                ":root {\n}\n".to_string()
            };
        }

        let mut output = String::with_capacity(tokens.len() * 50);
        
        if minify {
            output.push_str(":root{");
        } else {
            output.push_str(":root {\n");
        }

        for token in tokens {
            Self::generate_token_properties(&mut output, token, minify);
        }

        if minify {
            output.push('}');
        } else {
            output.push_str("}\n");
        }

        output
    }

    /// Generate CSS custom properties for a single token.
    /// Composite tokens (typography, shadow, border) generate multiple properties.
    fn generate_token_properties(output: &mut String, token: &W3CToken, minify: bool) {
        match &token.value {
            W3CTokenValue::Literal(value) => {
                // Check if this is a composite token that should be decomposed
                if let Some(ref token_type) = token.token_type {
                    match token_type {
                        W3CTokenType::Typography => {
                            // Typography tokens should have composite values, not literals
                            // If we get here, just output as a single property
                            Self::write_property(output, &token.path, value, minify);
                        }
                        _ => {
                            // Regular token - single property
                            Self::write_property(output, &token.path, value, minify);
                        }
                    }
                } else {
                    // No type specified - output as single property
                    Self::write_property(output, &token.path, value, minify);
                }
            }
            W3CTokenValue::Reference(_) => {
                // References should have been resolved before generation
                // If we encounter one, treat it as a literal
                if let W3CTokenValue::Reference(ref_path) = &token.value {
                    let value = format!("{{{}}}", ref_path);
                    Self::write_property(output, &token.path, &value, minify);
                }
            }
            W3CTokenValue::Composite(map) => {
                // Composite tokens generate multiple properties or a single combined value
                if let Some(ref token_type) = token.token_type {
                    match token_type {
                        W3CTokenType::Typography => {
                            Self::generate_typography_properties(output, &token.path, map, minify);
                        }
                        W3CTokenType::Shadow => {
                            Self::generate_shadow_property(output, &token.path, map, minify);
                        }
                        W3CTokenType::Border => {
                            Self::generate_border_property(output, &token.path, map, minify);
                        }
                        _ => {
                            // Unknown composite type - serialize as JSON-like string
                            let value = Self::serialize_composite(map);
                            Self::write_property(output, &token.path, &value, minify);
                        }
                    }
                } else {
                    // No type - serialize as JSON-like string
                    let value = Self::serialize_composite(map);
                    Self::write_property(output, &token.path, &value, minify);
                }
            }
        }
    }

    /// Write a single CSS custom property.
    fn write_property(output: &mut String, path: &str, value: &str, minify: bool) {
        let var_name = Self::token_to_css_var(path);
        
        if minify {
            output.push_str(&var_name);
            output.push(':');
            output.push_str(value);
            output.push(';');
        } else {
            output.push_str("  ");
            output.push_str(&var_name);
            output.push_str(": ");
            output.push_str(value);
            output.push_str(";\n");
        }
    }

    /// Convert a token path to a CSS custom property name.
    /// Example: "color.primary.500" -> "--color-primary-500"
    pub fn token_to_css_var(path: &str) -> String {
        let mut result = String::with_capacity(path.len() + 2);
        result.push_str("--");
        
        for ch in path.chars() {
            if ch == '.' {
                result.push('-');
            } else {
                result.push(ch);
            }
        }
        
        result
    }

    /// Generate multiple CSS custom properties for a typography token.
    /// Creates separate properties for font-family, font-size, font-weight, line-height, letter-spacing.
    fn generate_typography_properties(
        output: &mut String,
        path: &str,
        map: &HashMap<String, W3CTokenValue>,
        minify: bool,
    ) {
        // Generate individual properties for each typography sub-property
        let sub_properties = [
            "fontFamily",
            "fontSize",
            "fontWeight",
            "lineHeight",
            "letterSpacing",
        ];

        for sub_prop in &sub_properties {
            if let Some(value) = map.get(*sub_prop) {
                let sub_path = format!("{}-{}", path, Self::camel_to_kebab(sub_prop));
                let value_str = Self::extract_literal_value(value);
                Self::write_property(output, &sub_path, &value_str, minify);
            }
        }
    }

    /// Generate a single CSS box-shadow property for a shadow token.
    fn generate_shadow_property(
        output: &mut String,
        path: &str,
        map: &HashMap<String, W3CTokenValue>,
        minify: bool,
    ) {
        // Extract shadow components
        let offset_x = map.get("offsetX").map(Self::extract_literal_value).unwrap_or_else(|| "0".to_string());
        let offset_y = map.get("offsetY").map(Self::extract_literal_value).unwrap_or_else(|| "0".to_string());
        let blur = map.get("blur").map(Self::extract_literal_value).unwrap_or_else(|| "0".to_string());
        let spread = map.get("spread").map(Self::extract_literal_value);
        let color = map.get("color").map(Self::extract_literal_value).unwrap_or_else(|| "#000".to_string());

        // Build box-shadow value
        let value = if let Some(spread_val) = spread {
            format!("{} {} {} {} {}", offset_x, offset_y, blur, spread_val, color)
        } else {
            format!("{} {} {} {}", offset_x, offset_y, blur, color)
        };

        Self::write_property(output, path, &value, minify);
    }

    /// Generate a single CSS border property for a border token.
    fn generate_border_property(
        output: &mut String,
        path: &str,
        map: &HashMap<String, W3CTokenValue>,
        minify: bool,
    ) {
        // Extract border components
        let width = map.get("width").map(Self::extract_literal_value).unwrap_or_else(|| "1px".to_string());
        let style = map.get("style").map(Self::extract_literal_value).unwrap_or_else(|| "solid".to_string());
        let color = map.get("color").map(Self::extract_literal_value).unwrap_or_else(|| "#000".to_string());

        // Build border value
        let value = format!("{} {} {}", width, style, color);

        Self::write_property(output, path, &value, minify);
    }

    /// Extract a literal string value from a W3CTokenValue.
    fn extract_literal_value(value: &W3CTokenValue) -> String {
        match value {
            W3CTokenValue::Literal(s) => s.clone(),
            W3CTokenValue::Reference(r) => format!("{{{}}}", r),
            W3CTokenValue::Composite(map) => Self::serialize_composite(map),
        }
    }

    /// Serialize a composite value to a string representation.
    fn serialize_composite(map: &HashMap<String, W3CTokenValue>) -> String {
        let mut parts: Vec<String> = map
            .iter()
            .map(|(k, v)| format!("{}: {}", k, Self::extract_literal_value(v)))
            .collect();
        parts.sort();
        format!("{{{}}}", parts.join(", "))
    }

    /// Convert camelCase to kebab-case.
    fn camel_to_kebab(s: &str) -> String {
        let mut result = String::with_capacity(s.len() + 5);
        
        for (i, ch) in s.chars().enumerate() {
            if ch.is_uppercase() {
                if i > 0 {
                    result.push('-');
                }
                result.push(ch.to_ascii_lowercase());
            } else {
                result.push(ch);
            }
        }
        
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_token(path: &str, value: W3CTokenValue, token_type: Option<W3CTokenType>) -> W3CToken {
        W3CToken {
            path: path.to_string(),
            value,
            token_type,
            description: None,
        }
    }

    #[test]
    fn test_token_to_css_var() {
        assert_eq!(CSSGenerator::token_to_css_var("color.primary"), "--color-primary");
        assert_eq!(CSSGenerator::token_to_css_var("color.primary.500"), "--color-primary-500");
        assert_eq!(CSSGenerator::token_to_css_var("spacing.base"), "--spacing-base");
    }

    #[test]
    fn test_generate_simple_color_token() {
        let tokens = vec![
            create_token(
                "color.primary",
                W3CTokenValue::Literal("#3b82f6".to_string()),
                Some(W3CTokenType::Color),
            ),
        ];

        let css = CSSGenerator::generate(&tokens, false);
        assert!(css.contains(":root"));
        assert!(css.contains("--color-primary: #3b82f6;"));
    }

    #[test]
    fn test_generate_dimension_token() {
        let tokens = vec![
            create_token(
                "spacing.base",
                W3CTokenValue::Literal("8px".to_string()),
                Some(W3CTokenType::Dimension),
            ),
        ];

        let css = CSSGenerator::generate(&tokens, false);
        assert!(css.contains("--spacing-base: 8px;"));
    }

    #[test]
    fn test_generate_typography_composite() {
        let mut map = HashMap::new();
        map.insert("fontFamily".to_string(), W3CTokenValue::Literal("Inter".to_string()));
        map.insert("fontSize".to_string(), W3CTokenValue::Literal("16px".to_string()));
        map.insert("fontWeight".to_string(), W3CTokenValue::Literal("400".to_string()));
        map.insert("lineHeight".to_string(), W3CTokenValue::Literal("1.5".to_string()));

        let tokens = vec![
            create_token(
                "typography.body",
                W3CTokenValue::Composite(map),
                Some(W3CTokenType::Typography),
            ),
        ];

        let css = CSSGenerator::generate(&tokens, false);
        assert!(css.contains("--typography-body-font-family: Inter;"));
        assert!(css.contains("--typography-body-font-size: 16px;"));
        assert!(css.contains("--typography-body-font-weight: 400;"));
        assert!(css.contains("--typography-body-line-height: 1.5;"));
    }

    #[test]
    fn test_generate_shadow_composite() {
        let mut map = HashMap::new();
        map.insert("offsetX".to_string(), W3CTokenValue::Literal("0".to_string()));
        map.insert("offsetY".to_string(), W3CTokenValue::Literal("4px".to_string()));
        map.insert("blur".to_string(), W3CTokenValue::Literal("8px".to_string()));
        map.insert("spread".to_string(), W3CTokenValue::Literal("0".to_string()));
        map.insert("color".to_string(), W3CTokenValue::Literal("rgba(0, 0, 0, 0.1)".to_string()));

        let tokens = vec![
            create_token(
                "shadow.default",
                W3CTokenValue::Composite(map),
                Some(W3CTokenType::Shadow),
            ),
        ];

        let css = CSSGenerator::generate(&tokens, false);
        assert!(css.contains("--shadow-default: 0 4px 8px 0 rgba(0, 0, 0, 0.1);"));
    }

    #[test]
    fn test_generate_border_composite() {
        let mut map = HashMap::new();
        map.insert("width".to_string(), W3CTokenValue::Literal("1px".to_string()));
        map.insert("style".to_string(), W3CTokenValue::Literal("solid".to_string()));
        map.insert("color".to_string(), W3CTokenValue::Literal("#cccccc".to_string()));

        let tokens = vec![
            create_token(
                "border.default",
                W3CTokenValue::Composite(map),
                Some(W3CTokenType::Border),
            ),
        ];

        let css = CSSGenerator::generate(&tokens, false);
        assert!(css.contains("--border-default: 1px solid #cccccc;"));
    }

    #[test]
    fn test_generate_minified() {
        let tokens = vec![
            create_token(
                "color.primary",
                W3CTokenValue::Literal("#3b82f6".to_string()),
                Some(W3CTokenType::Color),
            ),
            create_token(
                "spacing.base",
                W3CTokenValue::Literal("8px".to_string()),
                Some(W3CTokenType::Dimension),
            ),
        ];

        let css = CSSGenerator::generate(&tokens, true);
        assert!(css.contains(":root{"));
        assert!(css.contains("--color-primary:#3b82f6;"));
        assert!(css.contains("--spacing-base:8px;"));
        assert!(!css.contains('\n'));
        assert!(!css.contains("  "));
    }

    #[test]
    fn test_generate_empty_tokens() {
        let tokens: Vec<W3CToken> = vec![];
        let css = CSSGenerator::generate(&tokens, false);
        assert_eq!(css, ":root {\n}\n");
    }

    #[test]
    fn test_generate_multiple_tokens() {
        let tokens = vec![
            create_token(
                "color.primary",
                W3CTokenValue::Literal("#3b82f6".to_string()),
                Some(W3CTokenType::Color),
            ),
            create_token(
                "color.secondary",
                W3CTokenValue::Literal("#10b981".to_string()),
                Some(W3CTokenType::Color),
            ),
            create_token(
                "spacing.small",
                W3CTokenValue::Literal("4px".to_string()),
                Some(W3CTokenType::Dimension),
            ),
        ];

        let css = CSSGenerator::generate(&tokens, false);
        assert!(css.contains("--color-primary: #3b82f6;"));
        assert!(css.contains("--color-secondary: #10b981;"));
        assert!(css.contains("--spacing-small: 4px;"));
    }

    #[test]
    fn test_camel_to_kebab() {
        assert_eq!(CSSGenerator::camel_to_kebab("fontFamily"), "font-family");
        assert_eq!(CSSGenerator::camel_to_kebab("fontSize"), "font-size");
        assert_eq!(CSSGenerator::camel_to_kebab("lineHeight"), "line-height");
        assert_eq!(CSSGenerator::camel_to_kebab("letterSpacing"), "letter-spacing");
    }
}
