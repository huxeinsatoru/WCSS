use crate::w3c_parser::{W3CToken, W3CTokenType, W3CTokenValue};
use std::collections::HashMap;

/// Format for Android code generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AndroidFormat {
    /// Generate XML resource files (colors.xml, dimens.xml)
    XML,
    /// Generate Kotlin code with constants
    Kotlin,
}

/// Generator for Android resources/code from W3C Design Tokens.
pub struct AndroidGenerator;

impl AndroidGenerator {
    /// Generate Android output from W3C tokens.
    ///
    /// Returns a map of filename to content.
    pub fn generate(tokens: &[W3CToken], format: AndroidFormat) -> HashMap<String, String> {
        let mut output = HashMap::new();

        match format {
            AndroidFormat::XML => {
                // Generate colors.xml
                let color_tokens: Vec<&W3CToken> = tokens
                    .iter()
                    .filter(|t| matches!(t.token_type, Some(W3CTokenType::Color)))
                    .collect();

                if !color_tokens.is_empty() {
                    let colors_xml = Self::generate_colors_xml(&color_tokens);
                    output.insert("colors.xml".to_string(), colors_xml);
                }

                // Generate dimens.xml
                let dimension_tokens: Vec<&W3CToken> = tokens
                    .iter()
                    .filter(|t| matches!(t.token_type, Some(W3CTokenType::Dimension)))
                    .collect();

                if !dimension_tokens.is_empty() {
                    let dimens_xml = Self::generate_dimens_xml(&dimension_tokens);
                    output.insert("dimens.xml".to_string(), dimens_xml);
                }
            }
            AndroidFormat::Kotlin => {
                let kotlin_code = Self::generate_kotlin(tokens);
                output.insert("DesignTokens.kt".to_string(), kotlin_code);
            }
        }

        output
    }

    /// Generate colors.xml file for color tokens.
    fn generate_colors_xml(tokens: &[&W3CToken]) -> String {
        let mut output = String::new();

        output.push_str("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n");
        output.push_str("<resources>\n");

        for token in tokens {
            let resource_name = Self::token_to_resource_name(&token.path);
            let color_value = Self::extract_literal_value(&token.value);
            let android_color = Self::hex_to_android_color(&color_value);

            // Add description as comment if available
            if let Some(ref description) = token.description {
                output.push_str(&format!("    <!-- {} -->\n", description));
            }

            output.push_str(&format!(
                "    <color name=\"{}\">{}</color>\n",
                resource_name, android_color
            ));
        }

        output.push_str("</resources>\n");
        output
    }

    /// Generate dimens.xml file for dimension tokens.
    fn generate_dimens_xml(tokens: &[&W3CToken]) -> String {
        let mut output = String::new();

        output.push_str("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n");
        output.push_str("<resources>\n");

        for token in tokens {
            let resource_name = Self::token_to_resource_name(&token.path);
            let dimension_value = Self::extract_literal_value(&token.value);
            let android_dimension = Self::dimension_to_android(&dimension_value);

            // Add description as comment if available
            if let Some(ref description) = token.description {
                output.push_str(&format!("    <!-- {} -->\n", description));
            }

            output.push_str(&format!(
                "    <dimen name=\"{}\">{}</dimen>\n",
                resource_name, android_dimension
            ));
        }

        output.push_str("</resources>\n");
        output
    }

    /// Generate Kotlin object with design token constants.
    fn generate_kotlin(tokens: &[W3CToken]) -> String {
        let mut output = String::new();

        // File header
        output.push_str("package com.example.design.tokens\n\n");
        output.push_str("import androidx.compose.ui.graphics.Color\n");
        output.push_str("import androidx.compose.ui.unit.dp\n");
        output.push_str("import androidx.compose.ui.unit.sp\n\n");

        // Object declaration
        output.push_str("/**
 * Design Tokens
 * Generated from W3C Design Tokens
 */\n");
        output.push_str("object DesignTokens {\n");

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
            output.push_str("\n    // Colors\n");
            for token in color_tokens {
                Self::generate_kotlin_color(&mut output, token);
            }
        }

        // Generate dimension constants
        if !dimension_tokens.is_empty() {
            output.push_str("\n    // Dimensions\n");
            for token in dimension_tokens {
                Self::generate_kotlin_dimension(&mut output, token);
            }
        }

        output.push_str("}\n");
        output
    }

    /// Generate a Kotlin color constant.
    fn generate_kotlin_color(output: &mut String, token: &W3CToken) {
        // Add KDoc comment if description exists
        if let Some(ref description) = token.description {
            output.push_str(&format!("    /** {} */\n", description));
        }

        let kotlin_name = Self::token_to_kotlin_name(&token.path);
        let color_value = Self::extract_literal_value(&token.value);
        let kotlin_color = Self::hex_to_kotlin_color(&color_value);

        output.push_str(&format!(
            "    val {} = {}\n",
            kotlin_name, kotlin_color
        ));
    }

    /// Generate a Kotlin dimension constant.
    fn generate_kotlin_dimension(output: &mut String, token: &W3CToken) {
        // Add KDoc comment if description exists
        if let Some(ref description) = token.description {
            output.push_str(&format!("    /** {} */\n", description));
        }

        let kotlin_name = Self::token_to_kotlin_name(&token.path);
        let dimension_value = Self::extract_literal_value(&token.value);
        let kotlin_dimension = Self::dimension_to_kotlin(&dimension_value);

        output.push_str(&format!(
            "    val {} = {}\n",
            kotlin_name, kotlin_dimension
        ));
    }

    /// Convert a token path to an Android resource name.
    /// Example: "color.primary.500" -> "color_primary_500"
    pub fn token_to_resource_name(path: &str) -> String {
        path.replace('.', "_")
            .replace('-', "_")
            .to_lowercase()
    }

    /// Convert a token path to a Kotlin constant name.
    /// Example: "color.primary.500" -> "colorPrimary500"
    pub fn token_to_kotlin_name(path: &str) -> String {
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

    /// Convert a hex color to Android color format.
    /// Android uses #AARRGGBB or #RRGGBB format.
    fn hex_to_android_color(hex: &str) -> String {
        let hex = hex.trim().trim_start_matches('#');

        match hex.len() {
            3 => {
                // #RGB -> #RRGGBB
                let r = hex[0..1].repeat(2);
                let g = hex[1..2].repeat(2);
                let b = hex[2..3].repeat(2);
                format!("#{}{}{}", r, g, b)
            }
            4 => {
                // #RGBA -> #AARRGGBB
                let r = hex[0..1].repeat(2);
                let g = hex[1..2].repeat(2);
                let b = hex[2..3].repeat(2);
                let a = hex[3..4].repeat(2);
                format!("#{}{}{}{}", a, r, g, b)
            }
            6 => {
                // #RRGGBB
                format!("#{}", hex)
            }
            8 => {
                // #RRGGBBAA -> #AARRGGBB (Android uses ARGB)
                let r = &hex[0..2];
                let g = &hex[2..4];
                let b = &hex[4..6];
                let a = &hex[6..8];
                format!("#{}{}{}{}", a, r, g, b)
            }
            _ => {
                // Invalid format, return black
                "#000000".to_string()
            }
        }
    }

    /// Convert a hex color to Kotlin Compose Color.
    fn hex_to_kotlin_color(hex: &str) -> String {
        let hex = hex.trim().trim_start_matches('#');

        let (r, g, b, a) = match hex.len() {
            3 => {
                // #RGB
                let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).unwrap_or(0);
                let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).unwrap_or(0);
                let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).unwrap_or(0);
                (r, g, b, 255)
            }
            6 => {
                // #RRGGBB
                let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
                let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
                let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
                (r, g, b, 255)
            }
            8 => {
                // #RRGGBBAA
                let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
                let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
                let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
                let a = u8::from_str_radix(&hex[6..8], 16).unwrap_or(255);
                (r, g, b, a)
            }
            _ => (0, 0, 0, 255),
        };

        if a == 255 {
            format!("Color(0xFF{:02X}{:02X}{:02X})", r, g, b)
        } else {
            format!("Color(0x{:02X}{:02X}{:02X}{:02X})", a, r, g, b)
        }
    }

    /// Convert a dimension value to Android format.
    fn dimension_to_android(dimension: &str) -> String {
        // Android uses the same format as CSS for most units
        // Just need to ensure proper formatting
        dimension.trim().to_string()
    }

    /// Convert a dimension value to Kotlin format.
    fn dimension_to_kotlin(dimension: &str) -> String {
        let trimmed = dimension.trim();

        // Extract numeric part and unit
        let numeric_part: String = trimmed
            .chars()
            .take_while(|c| c.is_numeric() || *c == '.' || *c == '-')
            .collect();

        let unit = &trimmed[numeric_part.len()..];

        match unit {
            "dp" => format!("{}dp", numeric_part),
            "sp" => format!("{}sp", numeric_part),
            "px" => format!("{}px", numeric_part),
            _ => {
                // Default to dp for unknown units
                if let Ok(num) = numeric_part.parse::<f64>() {
                    format!("{}dp", num)
                } else {
                    "0.dp".to_string()
                }
            }
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
    fn test_token_to_resource_name() {
        assert_eq!(
            AndroidGenerator::token_to_resource_name("color.primary.500"),
            "color_primary_500"
        );
        assert_eq!(
            AndroidGenerator::token_to_resource_name("spacing.base"),
            "spacing_base"
        );
        assert_eq!(
            AndroidGenerator::token_to_resource_name("font-size"),
            "font_size"
        );
    }

    #[test]
    fn test_token_to_kotlin_name() {
        assert_eq!(
            AndroidGenerator::token_to_kotlin_name("color.primary.500"),
            "colorPrimary500"
        );
        assert_eq!(
            AndroidGenerator::token_to_kotlin_name("spacing.base"),
            "spacingBase"
        );
        assert_eq!(
            AndroidGenerator::token_to_kotlin_name("fontSize"),
            "fontSize"
        );
    }

    #[test]
    fn test_hex_to_android_color() {
        // Test #RGB format
        assert_eq!(AndroidGenerator::hex_to_android_color("#fff"), "#ffffff");
        assert_eq!(AndroidGenerator::hex_to_android_color("#000"), "#000000");

        // Test #RRGGBB format
        assert_eq!(
            AndroidGenerator::hex_to_android_color("#3b82f6"),
            "#3b82f6"
        );

        // Test #RRGGBBAA format (should convert to #AARRGGBB)
        assert_eq!(
            AndroidGenerator::hex_to_android_color("#3b82f6ff"),
            "#ff3b82f6"
        );
        assert_eq!(
            AndroidGenerator::hex_to_android_color("#3b82f680"),
            "#803b82f6"
        );
    }

    #[test]
    fn test_hex_to_kotlin_color() {
        // Test #RRGGBB format
        let result = AndroidGenerator::hex_to_kotlin_color("#3b82f6");
        assert!(result.contains("Color(0xFF"));

        // Test #RRGGBBAA format with alpha
        let result = AndroidGenerator::hex_to_kotlin_color("#3b82f680");
        assert!(result.contains("Color(0x80"));
    }

    #[test]
    fn test_dimension_to_kotlin() {
        assert_eq!(AndroidGenerator::dimension_to_kotlin("16dp"), "16dp");
        assert_eq!(AndroidGenerator::dimension_to_kotlin("12sp"), "12sp");
        assert_eq!(AndroidGenerator::dimension_to_kotlin("8px"), "8px");
    }

    #[test]
    fn test_generate_colors_xml() {
        let token = create_token(
            "color.primary",
            W3CTokenValue::Literal("#3b82f6".to_string()),
            Some(W3CTokenType::Color),
            Some("Primary brand color".to_string()),
        );

        let xml = AndroidGenerator::generate_colors_xml(&vec![&token]);

        assert!(xml.contains("<?xml version=\"1.0\" encoding=\"utf-8\"?>"));
        assert!(xml.contains("<resources>"));
        assert!(xml.contains("<!-- Primary brand color -->"));
        assert!(xml.contains("<color name=\"color_primary\">#3b82f6</color>"));
        assert!(xml.contains("</resources>"));
    }

    #[test]
    fn test_generate_dimens_xml() {
        let token = create_token(
            "spacing.base",
            W3CTokenValue::Literal("16px".to_string()),
            Some(W3CTokenType::Dimension),
            Some("Base spacing unit".to_string()),
        );

        let xml = AndroidGenerator::generate_dimens_xml(&vec![&token]);

        assert!(xml.contains("<?xml version=\"1.0\" encoding=\"utf-8\"?>"));
        assert!(xml.contains("<resources>"));
        assert!(xml.contains("<!-- Base spacing unit -->"));
        assert!(xml.contains("<dimen name=\"spacing_base\">16px</dimen>"));
        assert!(xml.contains("</resources>"));
    }

    #[test]
    fn test_generate_kotlin() {
        let tokens = vec![
            create_token(
                "color.primary",
                W3CTokenValue::Literal("#3b82f6".to_string()),
                Some(W3CTokenType::Color),
                Some("Primary color".to_string()),
            ),
            create_token(
                "spacing.base",
                W3CTokenValue::Literal("16dp".to_string()),
                Some(W3CTokenType::Dimension),
                None,
            ),
        ];

        let kotlin = AndroidGenerator::generate_kotlin(&tokens);

        assert!(kotlin.contains("package com.example.design.tokens"));
        assert!(kotlin.contains("object DesignTokens"));
        assert!(kotlin.contains("/** Primary color */"));
        assert!(kotlin.contains("val colorPrimary"));
        assert!(kotlin.contains("val spacingBase"));
    }

    #[test]
    fn test_generate_xml_format() {
        let tokens = vec![
            create_token(
                "color.primary",
                W3CTokenValue::Literal("#3b82f6".to_string()),
                Some(W3CTokenType::Color),
                None,
            ),
            create_token(
                "spacing.base",
                W3CTokenValue::Literal("16px".to_string()),
                Some(W3CTokenType::Dimension),
                None,
            ),
        ];

        let output = AndroidGenerator::generate(&tokens, AndroidFormat::XML);

        assert!(output.contains_key("colors.xml"));
        assert!(output.contains_key("dimens.xml"));
        assert!(output["colors.xml"].contains("<color name=\"color_primary\">"));
        assert!(output["dimens.xml"].contains("<dimen name=\"spacing_base\">"));
    }

    #[test]
    fn test_generate_kotlin_format() {
        let tokens = vec![
            create_token(
                "color.primary",
                W3CTokenValue::Literal("#3b82f6".to_string()),
                Some(W3CTokenType::Color),
                None,
            ),
        ];

        let output = AndroidGenerator::generate(&tokens, AndroidFormat::Kotlin);

        assert!(output.contains_key("DesignTokens.kt"));
        assert!(output["DesignTokens.kt"].contains("object DesignTokens"));
    }
}
