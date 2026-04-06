use crate::w3c_parser::{W3CToken, W3CTokenType, W3CTokenValue};

/// Generator for iOS Swift code from W3C Design Tokens.
pub struct IOSGenerator;

impl IOSGenerator {
    /// Generate Swift code from W3C tokens.
    ///
    /// # Arguments
    /// * `tokens` - The W3C tokens to convert to Swift code
    ///
    /// # Returns
    /// A string containing Swift code with UIColor extensions, CGFloat constants,
    /// font helpers, and other design token representations.
    pub fn generate(tokens: &[W3CToken]) -> String {
        if tokens.is_empty() {
            return String::new();
        }

        let mut output = String::with_capacity(tokens.len() * 100);
        
        // Add file header
        output.push_str("import UIKit\n\n");
        output.push_str("// MARK: - Design Tokens\n");
        output.push_str("// Generated from W3C Design Tokens\n\n");
        
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
        
        // Generate UIColor extension for color tokens
        if !color_tokens.is_empty() {
            output.push_str("// MARK: - Colors\n");
            output.push_str("extension UIColor {\n");
            
            for token in color_tokens {
                Self::generate_color_token(&mut output, token);
            }
            
            output.push_str("}\n\n");
        }
        
        // Generate CGFloat constants for dimension tokens
        if !dimension_tokens.is_empty() {
            output.push_str("// MARK: - Dimensions\n");
            output.push_str("enum DesignTokens {\n");
            
            for token in dimension_tokens {
                Self::generate_dimension_token(&mut output, token);
            }
            
            output.push_str("}\n");
        }
        
        output
    }
    
    /// Generate a UIColor property for a color token.
    fn generate_color_token(output: &mut String, token: &W3CToken) {
        // Generate documentation comment if description exists
        if let Some(ref description) = token.description {
            Self::generate_documentation(output, description);
        }
        
        let swift_name = Self::token_to_swift_name(&token.path);
        let color_value = Self::extract_literal_value(&token.value);
        let uicolor_code = Self::hex_to_uicolor(&color_value);
        
        output.push_str("    static var ");
        output.push_str(&swift_name);
        output.push_str(": UIColor {\n");
        output.push_str("        ");
        output.push_str(&uicolor_code);
        output.push_str("\n    }\n");
    }
    
    /// Generate a CGFloat constant for a dimension token.
    fn generate_dimension_token(output: &mut String, token: &W3CToken) {
        // Generate documentation comment if description exists
        if let Some(ref description) = token.description {
            Self::generate_documentation(output, description);
        }
        
        let swift_name = Self::token_to_swift_name(&token.path);
        let dimension_value = Self::extract_literal_value(&token.value);
        let cgfloat_value = Self::dimension_to_cgfloat(&dimension_value);
        
        output.push_str("    static let ");
        output.push_str(&swift_name);
        output.push_str(": CGFloat = ");
        output.push_str(&cgfloat_value);
        output.push_str("\n");
    }
    
    /// Generate Swift documentation comment from a description.
    /// Creates a triple-slash comment (///) in Swift format.
    fn generate_documentation(output: &mut String, description: &str) {
        output.push_str("    /// ");
        output.push_str(description);
        output.push('\n');
    }
    
    /// Convert a token path to a Swift property name.
    /// Example: "color.primary.500" -> "colorPrimary500"
    pub fn token_to_swift_name(path: &str) -> String {
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
    
    /// Convert a hex color to UIColor RGB components.
    /// Supports #RGB, #RRGGBB, and #RRGGBBAA formats.
    pub fn hex_to_uicolor(hex: &str) -> String {
        let hex = hex.trim_start_matches('#');
        
        let (r, g, b, a) = match hex.len() {
            3 => {
                // #RGB format
                let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).unwrap_or(0);
                let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).unwrap_or(0);
                let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).unwrap_or(0);
                (r, g, b, 255)
            }
            6 => {
                // #RRGGBB format
                let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
                let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
                let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
                (r, g, b, 255)
            }
            8 => {
                // #RRGGBBAA format
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
        
        let r_float = r as f64 / 255.0;
        let g_float = g as f64 / 255.0;
        let b_float = b as f64 / 255.0;
        let a_float = a as f64 / 255.0;
        
        format!(
            "UIColor(red: {:.3}, green: {:.3}, blue: {:.3}, alpha: {:.3})",
            r_float, g_float, b_float, a_float
        )
    }
    
    /// Convert a dimension value to a CGFloat value.
    /// Strips the unit and returns just the numeric value.
    fn dimension_to_cgfloat(dimension: &str) -> String {
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
    
    /// Extract the literal value from a token value.
    fn extract_literal_value(value: &W3CTokenValue) -> String {
        match value {
            W3CTokenValue::Literal(s) => s.clone(),
            W3CTokenValue::Reference(r) => format!("{{{}}}", r),
            W3CTokenValue::Composite(map) => {
                // Serialize composite as a simple string representation
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
    fn test_token_to_swift_name() {
        assert_eq!(
            IOSGenerator::token_to_swift_name("color.primary.500"),
            "colorPrimary500"
        );
        assert_eq!(
            IOSGenerator::token_to_swift_name("spacing.base"),
            "spacingBase"
        );
        assert_eq!(
            IOSGenerator::token_to_swift_name("fontSize"),
            "fontSize"
        );
    }

    #[test]
    fn test_hex_to_uicolor() {
        // Test #RRGGBB format
        let result = IOSGenerator::hex_to_uicolor("#3b82f6");
        assert!(result.contains("UIColor"));
        assert!(result.contains("red:"));
        assert!(result.contains("green:"));
        assert!(result.contains("blue:"));
        assert!(result.contains("alpha:"));
        
        // Test #RGB format
        let result = IOSGenerator::hex_to_uicolor("#fff");
        assert!(result.contains("1.000")); // White should be 1.0 for all components
        
        // Test #RRGGBBAA format
        let result = IOSGenerator::hex_to_uicolor("#3b82f680");
        assert!(result.contains("alpha: 0.502")); // 0x80 / 255 ≈ 0.502
    }

    #[test]
    fn test_dimension_to_cgfloat() {
        assert_eq!(IOSGenerator::dimension_to_cgfloat("16px"), "16");
        assert_eq!(IOSGenerator::dimension_to_cgfloat("1.5rem"), "1.5");
        assert_eq!(IOSGenerator::dimension_to_cgfloat("24"), "24");
        assert_eq!(IOSGenerator::dimension_to_cgfloat("px"), "0.0");
    }

    #[test]
    fn test_generate_empty_tokens() {
        let tokens: Vec<W3CToken> = vec![];
        let result = IOSGenerator::generate(&tokens);
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
        
        let result = IOSGenerator::generate(&vec![token]);
        assert!(result.contains("import UIKit"));
        assert!(result.contains("extension UIColor"));
        assert!(result.contains("static var colorPrimary"));
        assert!(result.contains("UIColor(red:"));
    }

    #[test]
    fn test_generate_dimension_token() {
        let token = create_token(
            "spacing.base",
            W3CTokenValue::Literal("16px".to_string()),
            Some(W3CTokenType::Dimension),
            None,
        );
        
        let result = IOSGenerator::generate(&vec![token]);
        assert!(result.contains("enum DesignTokens"));
        assert!(result.contains("static let spacingBase: CGFloat = 16"));
    }

    #[test]
    fn test_generate_with_documentation() {
        let token = create_token(
            "color.primary",
            W3CTokenValue::Literal("#3b82f6".to_string()),
            Some(W3CTokenType::Color),
            Some("Primary brand color used for buttons and links".to_string()),
        );
        
        let result = IOSGenerator::generate(&vec![token]);
        assert!(result.contains("/// Primary brand color used for buttons and links"));
        assert!(result.contains("static var colorPrimary"));
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
        
        let result = IOSGenerator::generate(&tokens);
        assert!(result.contains("/// Primary color"));
        assert!(result.contains("colorPrimary"));
        assert!(result.contains("colorSecondary"));
        assert!(result.contains("/// Small spacing unit"));
        assert!(result.contains("spacingSmall"));
    }
}
