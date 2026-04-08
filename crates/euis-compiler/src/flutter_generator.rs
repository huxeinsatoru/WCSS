use crate::w3c_parser::{W3CToken, W3CTokenType, W3CTokenValue};

/// Generator for Flutter Dart code from W3C Design Tokens.
pub struct FlutterGenerator;

impl FlutterGenerator {
    /// Generate Dart code from W3C tokens.
    ///
    /// # Arguments
    /// * `tokens` - The W3C tokens to convert to Dart code
    ///
    /// # Returns
    /// A string containing Dart code with Color and double constants
    /// organized in a class for namespacing.
    pub fn generate(tokens: &[W3CToken]) -> String {
        if tokens.is_empty() {
            return String::new();
        }

        let mut output = String::with_capacity(tokens.len() * 100);

        // Add file header
        output.push_str("import 'package:flutter/material.dart';\n\n");
        output.push_str("/// Design Tokens\n");
        output.push_str("/// Generated from W3C Design Tokens\n");
        output.push_str("class DesignTokens {\n");
        output.push_str("  const DesignTokens._();\n\n");

        // Group tokens by type
        let mut color_tokens = Vec::new();
        let mut dimension_tokens = Vec::new();
        let mut other_tokens = Vec::new();

        for token in tokens {
            match &token.token_type {
                Some(W3CTokenType::Color) => color_tokens.push(token),
                Some(W3CTokenType::Dimension) => dimension_tokens.push(token),
                _ => other_tokens.push(token),
            }
        }

        // Generate color constants
        if !color_tokens.is_empty() {
            output.push_str("  // Colors\n");
            for token in color_tokens {
                Self::generate_color_token(&mut output, token);
            }
            output.push('\n');
        }

        // Generate dimension constants
        if !dimension_tokens.is_empty() {
            output.push_str("  // Dimensions\n");
            for token in dimension_tokens {
                Self::generate_dimension_token(&mut output, token);
            }
            output.push('\n');
        }

        // Handle other tokens
        if !other_tokens.is_empty() {
            output.push_str("  // Other tokens\n");
            for token in other_tokens {
                Self::generate_generic_token(&mut output, token);
            }
        }

        output.push_str("}\n");
        output
    }

    /// Generate a Color constant for a color token.
    fn generate_color_token(output: &mut String, token: &W3CToken) {
        // Generate documentation comment if description exists
        if let Some(ref description) = token.description {
            Self::generate_documentation(output, description, 2);
        }

        let dart_name = Self::token_to_dart_name(&token.path);
        let color_value = Self::extract_literal_value(&token.value);
        let flutter_color = Self::hex_to_flutter_color(&color_value);

        output.push_str("  static const Color ");
        output.push_str(&dart_name);
        output.push_str(" = ");
        output.push_str(&flutter_color);
        output.push_str(";\n");
    }

    /// Generate a double constant for a dimension token.
    fn generate_dimension_token(output: &mut String, token: &W3CToken) {
        // Generate documentation comment if description exists
        if let Some(ref description) = token.description {
            Self::generate_documentation(output, description, 2);
        }

        let dart_name = Self::token_to_dart_name(&token.path);
        let dimension_value = Self::extract_literal_value(&token.value);
        let double_value = Self::dimension_to_double(&dimension_value);

        output.push_str("  static const double ");
        output.push_str(&dart_name);
        output.push_str(" = ");
        output.push_str(&double_value);
        output.push_str(";\n");
    }

    /// Generate a generic constant for other token types.
    fn generate_generic_token(output: &mut String, token: &W3CToken) {
        // Generate documentation comment if description exists
        if let Some(ref description) = token.description {
            Self::generate_documentation(output, description, 2);
        }

        let dart_name = Self::token_to_dart_name(&token.path);
        let value = Self::extract_literal_value(&token.value);

        // Determine the type based on token type
        let (type_name, dart_value) = match &token.token_type {
            Some(W3CTokenType::FontFamily) => ("String", format!("'{}'", value)),
            Some(W3CTokenType::FontWeight) => ("int", value.clone()),
            Some(W3CTokenType::Duration) => ("Duration", Self::duration_to_dart(&value)),
            Some(W3CTokenType::Number) => ("double", value.clone()),
            _ => ("String", format!("'{}'", value)),
        };

        output.push_str("  static const ");
        output.push_str(type_name);
        output.push(' ');
        output.push_str(&dart_name);
        output.push_str(" = ");
        output.push_str(&dart_value);
        output.push_str(";\n");
    }

    /// Generate Dart documentation comment from a description.
    fn generate_documentation(output: &mut String, description: &str, indent_level: usize) {
        let indent = "  ".repeat(indent_level);
        output.push_str(&indent);
        output.push_str("/// ");
        output.push_str(description);
        output.push('\n');
    }

    /// Convert a token path to a Dart constant name.
    /// Example: "color.primary.500" -> "colorPrimary500"
    pub fn token_to_dart_name(path: &str) -> String {
        let parts: Vec<&str> = path.split('.').collect();
        if parts.is_empty() {
            return String::new();
        }

        let mut result = String::with_capacity(path.len());

        // First part is lowercase
        result.push_str(parts[0]);

        // Subsequent parts are capitalized
        for part in &parts[1..] {
            if let Some(first_char) = part.chars().next() {
                result.push(first_char.to_ascii_uppercase());
                result.push_str(&part[1..]);
            }
        }

        result
    }

    /// Convert a hex color to Flutter Color format (0xAARRGGBB).
    /// Example: "#FF5733" -> "Color(0xFFFF5733)"
    pub fn hex_to_flutter_color(hex: &str) -> String {
        let hex = hex.trim().trim_start_matches('#');

        let (r, g, b, a) = match hex.len() {
            3 => {
                // #RGB format
                let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).unwrap_or(0);
                let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).unwrap_or(0);
                let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).unwrap_or(0);
                (r, g, b, 255)
            }
            4 => {
                // #RGBA format
                let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).unwrap_or(0);
                let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).unwrap_or(0);
                let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).unwrap_or(0);
                let a = u8::from_str_radix(&hex[3..4].repeat(2), 16).unwrap_or(255);
                (r, g, b, a)
            }
            6 => {
                // #RRGGBB format
                let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
                let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
                let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
                (r, g, b, 255)
            }
            8 => {
                // #RRGGBBAA format - Flutter uses ARGB
                let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
                let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
                let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
                let a = u8::from_str_radix(&hex[6..8], 16).unwrap_or(255);
                (r, g, b, a)
            }
            _ => {
                // Invalid format, return black
                (0, 0, 0, 255)
            }
        };

        // Flutter uses ARGB format
        format!("Color(0x{:02X}{:02X}{:02X}{:02X})", a, r, g, b)
    }

    /// Convert a dimension value to a double value.
    fn dimension_to_double(dimension: &str) -> String {
        // Extract numeric part from dimension (e.g., "16px" -> "16")
        let numeric_part: String = dimension
            .chars()
            .take_while(|c| c.is_numeric() || *c == '.' || *c == '-')
            .collect();

        if numeric_part.is_empty() {
            "0.0".to_string()
        } else {
            numeric_part
        }
    }

    /// Convert a duration value to Dart Duration.
    fn duration_to_dart(duration: &str) -> String {
        let trimmed = duration.trim();

        if trimmed.ends_with("ms") {
            let ms = &trimmed[..trimmed.len() - 2];
            format!("Duration(milliseconds: {})", ms)
        } else if trimmed.ends_with('s') {
            let s = &trimmed[..trimmed.len() - 1];
            if let Ok(seconds) = s.parse::<f64>() {
                let ms = (seconds * 1000.0) as i32;
                format!("Duration(milliseconds: {})", ms)
            } else {
                "Duration.zero".to_string()
            }
        } else {
            "Duration.zero".to_string()
        }
    }

    /// Extract the literal value from a token value.
    fn extract_literal_value(value: &W3CTokenValue) -> String {
        match value {
            W3CTokenValue::Literal(s) => s.clone(),
            W3CTokenValue::Reference(r) => format!("{{{}}}", r),
            W3CTokenValue::Composite(map) => {
                format!("{:?}", map)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_token(
        path: &str,
        value: W3CTokenValue,
        token_type: Option<W3CTokenType>,
        description: Option<String>,
    ) -> W3CToken {
        W3CToken {
            path: path.to_string(),
            value,
            token_type,
            description,
        }
    }

    #[test]
    fn test_token_to_dart_name() {
        assert_eq!(
            FlutterGenerator::token_to_dart_name("color.primary.500"),
            "colorPrimary500"
        );
        assert_eq!(
            FlutterGenerator::token_to_dart_name("spacing.base"),
            "spacingBase"
        );
        assert_eq!(
            FlutterGenerator::token_to_dart_name("fontSize"),
            "fontSize"
        );
    }

    #[test]
    fn test_hex_to_flutter_color() {
        // Test #RRGGBB format
        let result = FlutterGenerator::hex_to_flutter_color("#3b82f6");
        assert_eq!(result, "Color(0xFF3B82F6)");

        // Test #RGB format
        let result = FlutterGenerator::hex_to_flutter_color("#fff");
        assert_eq!(result, "Color(0xFFFFFFFF)");

        // Test #RRGGBBAA format with alpha
        let result = FlutterGenerator::hex_to_flutter_color("#3b82f680");
        assert_eq!(result, "Color(0x803B82F6)");
    }

    #[test]
    fn test_dimension_to_double() {
        assert_eq!(FlutterGenerator::dimension_to_double("16px"), "16");
        assert_eq!(FlutterGenerator::dimension_to_double("1.5rem"), "1.5");
        assert_eq!(FlutterGenerator::dimension_to_double("24"), "24");
        assert_eq!(FlutterGenerator::dimension_to_double("px"), "0.0");
    }

    #[test]
    fn test_duration_to_dart() {
        assert_eq!(
            FlutterGenerator::duration_to_dart("200ms"),
            "Duration(milliseconds: 200)"
        );
        assert_eq!(
            FlutterGenerator::duration_to_dart("1s"),
            "Duration(milliseconds: 1000)"
        );
        assert_eq!(
            FlutterGenerator::duration_to_dart("0.5s"),
            "Duration(milliseconds: 500)"
        );
    }

    #[test]
    fn test_generate_empty_tokens() {
        let tokens: Vec<W3CToken> = vec![];
        let result = FlutterGenerator::generate(&tokens);
        assert_eq!(result, "");
    }

    #[test]
    fn test_generate_color_token() {
        let token = create_token(
            "color.primary",
            W3CTokenValue::Literal("#3b82f6".to_string()),
            Some(W3CTokenType::Color),
            None,
        );

        let result = FlutterGenerator::generate(&vec![token]);
        assert!(result.contains("import 'package:flutter/material.dart';"));
        assert!(result.contains("class DesignTokens"));
        assert!(result.contains("static const Color colorPrimary"));
        assert!(result.contains("Color(0xFF3B82F6)"));
    }

    #[test]
    fn test_generate_dimension_token() {
        let token = create_token(
            "spacing.base",
            W3CTokenValue::Literal("16px".to_string()),
            Some(W3CTokenType::Dimension),
            None,
        );

        let result = FlutterGenerator::generate(&vec![token]);
        assert!(result.contains("static const double spacingBase"));
        assert!(result.contains("= 16;"));
    }

    #[test]
    fn test_generate_with_documentation() {
        let token = create_token(
            "color.primary",
            W3CTokenValue::Literal("#3b82f6".to_string()),
            Some(W3CTokenType::Color),
            Some("Primary brand color used for buttons and links".to_string()),
        );

        let result = FlutterGenerator::generate(&vec![token]);
        assert!(result.contains("/// Primary brand color used for buttons and links"));
        assert!(result.contains("static const Color colorPrimary"));
    }

    #[test]
    fn test_generate_multiple_tokens() {
        let tokens = vec![
            create_token(
                "color.primary",
                W3CTokenValue::Literal("#3b82f6".to_string()),
                Some(W3CTokenType::Color),
                Some("Primary color".to_string()),
            ),
            create_token(
                "color.secondary",
                W3CTokenValue::Literal("#64748b".to_string()),
                Some(W3CTokenType::Color),
                None,
            ),
            create_token(
                "spacing.small",
                W3CTokenValue::Literal("8px".to_string()),
                Some(W3CTokenType::Dimension),
                Some("Small spacing unit".to_string()),
            ),
        ];

        let result = FlutterGenerator::generate(&tokens);
        assert!(result.contains("/// Primary color"));
        assert!(result.contains("colorPrimary"));
        assert!(result.contains("colorSecondary"));
        assert!(result.contains("/// Small spacing unit"));
        assert!(result.contains("spacingSmall"));
        assert!(result.contains("const DesignTokens._();"));
    }

    #[test]
    fn test_generate_font_weight_token() {
        let token = create_token(
            "font.weight.bold",
            W3CTokenValue::Literal("700".to_string()),
            Some(W3CTokenType::FontWeight),
            None,
        );

        let result = FlutterGenerator::generate(&vec![token]);
        assert!(result.contains("static const int fontWeightBold"));
        assert!(result.contains("= 700;"));
    }

    #[test]
    fn test_generate_duration_token() {
        let token = create_token(
            "animation.fast",
            W3CTokenValue::Literal("200ms".to_string()),
            Some(W3CTokenType::Duration),
            None,
        );

        let result = FlutterGenerator::generate(&vec![token]);
        assert!(result.contains("static const Duration animationFast"));
        assert!(result.contains("Duration(milliseconds: 200)"));
    }
}
