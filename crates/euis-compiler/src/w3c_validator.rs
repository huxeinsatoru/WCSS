use crate::error::CompilerError;
use crate::w3c_parser::{W3CToken, W3CTokenType, W3CTokenValue};
use crate::ast::Span;
use std::collections::HashMap;

/// Validator for W3C Design Token types.
pub struct TokenTypeValidator;

impl TokenTypeValidator {
    /// Validate a token against its declared type.
    pub fn validate(token: &W3CToken) -> Result<(), CompilerError> {
        if let Some(ref token_type) = token.token_type {
            Self::validate_value(&token.value, token_type, &token.path)
        } else {
            // No type declared, skip validation
            Ok(())
        }
    }

    /// Validate a token value against a specific type.
    fn validate_value(value: &W3CTokenValue, token_type: &W3CTokenType, path: &str) -> Result<(), CompilerError> {
        match value {
            W3CTokenValue::Literal(s) => {
                Self::validate_literal(s, token_type, path)
            }
            W3CTokenValue::Reference(_) => {
                // References are validated during resolution phase
                Ok(())
            }
            W3CTokenValue::Composite(map) => {
                Self::validate_composite(map, token_type, path)
            }
        }
    }

    /// Validate a literal value against a token type.
    fn validate_literal(value: &str, token_type: &W3CTokenType, path: &str) -> Result<(), CompilerError> {
        let result = match token_type {
            W3CTokenType::Color => Self::validate_color(value),
            W3CTokenType::Dimension => Self::validate_dimension(value),
            W3CTokenType::FontFamily => Self::validate_font_family(value),
            W3CTokenType::FontWeight => Self::validate_font_weight(value),
            W3CTokenType::Duration => Self::validate_duration(value),
            W3CTokenType::CubicBezier => Self::validate_cubic_bezier(value),
            W3CTokenType::Number => Self::validate_number(value),
            W3CTokenType::StrokeStyle => Self::validate_stroke_style(value),
            W3CTokenType::Border | W3CTokenType::Shadow | W3CTokenType::Typography => {
                Err("Composite types require object values".to_string())
            }
        };

        result.map_err(|msg| {
            CompilerError::validation(
                format!("Invalid {} value for token '{}': {}", token_type.as_str(), path, msg),
                Span::empty(),
            )
        })
    }

    /// Validate a composite value against a token type.
    fn validate_composite(map: &HashMap<String, W3CTokenValue>, token_type: &W3CTokenType, path: &str) -> Result<(), CompilerError> {
        match token_type {
            W3CTokenType::Typography => {
                Self::validate_typography_composite(map, path)
            }
            W3CTokenType::Shadow => {
                Self::validate_shadow_composite(map, path)
            }
            W3CTokenType::Border => {
                Self::validate_border_composite(map, path)
            }
            _ => {
                Err(CompilerError::validation(
                    format!("Token type '{}' does not support composite values", token_type.as_str()),
                    Span::empty(),
                ))
            }
        }
    }

    /// Validate color value (hex, rgb, rgba, hsl, hsla, named).
    fn validate_color(value: &str) -> Result<(), String> {
        let trimmed = value.trim();
        
        // Hex color
        if trimmed.starts_with('#') {
            let hex = &trimmed[1..];
            if hex.len() == 3 || hex.len() == 4 || hex.len() == 6 || hex.len() == 8 {
                if hex.chars().all(|c| c.is_ascii_hexdigit()) {
                    return Ok(());
                }
            }
            return Err(format!("Invalid hex color format (expected #RGB, #RGBA, #RRGGBB, or #RRGGBBAA): {}", value));
        }
        
        // RGB/RGBA - validate format more thoroughly
        if trimmed.starts_with("rgb(") && trimmed.ends_with(')') {
            let content = &trimmed[4..trimmed.len()-1];
            let parts: Vec<&str> = content.split(',').map(|s| s.trim()).collect();
            if parts.len() == 3 {
                for part in parts {
                    if part.parse::<u8>().is_err() && !part.ends_with('%') {
                        return Err(format!("Invalid rgb() format (expected 3 numeric values): {}", value));
                    }
                }
                return Ok(());
            }
            return Err(format!("Invalid rgb() format (expected 3 values): {}", value));
        }
        
        if trimmed.starts_with("rgba(") && trimmed.ends_with(')') {
            let content = &trimmed[5..trimmed.len()-1];
            let parts: Vec<&str> = content.split(',').map(|s| s.trim()).collect();
            if parts.len() == 4 {
                // Validate first 3 as RGB, last as alpha
                for (i, part) in parts.iter().enumerate() {
                    if i < 3 {
                        if part.parse::<u8>().is_err() && !part.ends_with('%') {
                            return Err(format!("Invalid rgba() format: {}", value));
                        }
                    } else if part.parse::<f64>().is_err() {
                        return Err(format!("Invalid rgba() alpha value: {}", value));
                    }
                }
                return Ok(());
            }
            return Err(format!("Invalid rgba() format (expected 4 values): {}", value));
        }
        
        // HSL/HSLA - validate format more thoroughly
        if trimmed.starts_with("hsl(") && trimmed.ends_with(')') {
            let content = &trimmed[4..trimmed.len()-1];
            let parts: Vec<&str> = content.split(',').map(|s| s.trim()).collect();
            if parts.len() == 3 {
                // Validate hue, saturation, lightness
                for (i, part) in parts.iter().enumerate() {
                    if i == 0 {
                        // Hue: number or number with deg
                        let hue_str = part.trim_end_matches("deg");
                        if hue_str.parse::<f64>().is_err() {
                            return Err(format!("Invalid hsl() hue value: {}", value));
                        }
                    } else if !part.ends_with('%') {
                        return Err(format!("Invalid hsl() format (saturation and lightness must be percentages): {}", value));
                    }
                }
                return Ok(());
            }
            return Err(format!("Invalid hsl() format (expected 3 values): {}", value));
        }
        
        if trimmed.starts_with("hsla(") && trimmed.ends_with(')') {
            let content = &trimmed[5..trimmed.len()-1];
            let parts: Vec<&str> = content.split(',').map(|s| s.trim()).collect();
            if parts.len() == 4 {
                // Validate hue, saturation, lightness, alpha
                for (i, part) in parts.iter().enumerate() {
                    if i == 0 {
                        let hue_str = part.trim_end_matches("deg");
                        if hue_str.parse::<f64>().is_err() {
                            return Err(format!("Invalid hsla() hue value: {}", value));
                        }
                    } else if i < 3 && !part.ends_with('%') {
                        return Err(format!("Invalid hsla() format: {}", value));
                    } else if i == 3 && part.parse::<f64>().is_err() {
                        return Err(format!("Invalid hsla() alpha value: {}", value));
                    }
                }
                return Ok(());
            }
            return Err(format!("Invalid hsla() format (expected 4 values): {}", value));
        }
        
        // OKLCH/OKLCHA - modern perceptual color space
        if trimmed.starts_with("oklch(") && trimmed.ends_with(')') {
            let content = &trimmed[6..trimmed.len()-1];
            // Handle both comma and space separated values, and slash for alpha
            let parts: Vec<&str> = if content.contains('/') {
                // New syntax: oklch(L C H / alpha)
                let without_alpha = content.split('/').next().unwrap_or("").trim();
                let alpha = content.split('/').nth(1).unwrap_or("1").trim();
                let mut parts: Vec<&str> = without_alpha.split_whitespace().collect();
                parts.push(alpha);
                parts
            } else if content.contains(',') {
                // Legacy comma syntax
                content.split(',').map(|s| s.trim()).collect()
            } else {
                // Modern space syntax: oklch(L C H)
                content.split_whitespace().collect()
            };
            
            if parts.len() >= 3 {
                // Validate lightness (0-100% or 0-1)
                let l = parts[0];
                let l_num = if l.ends_with('%') {
                    l.trim_end_matches('%').parse::<f64>().ok()
                } else {
                    l.parse::<f64>().ok()
                };
                if l_num.is_none() || l_num.unwrap() < 0.0 {
                    return Err(format!("Invalid oklch() lightness value: {}", value));
                }
                
                // Validate chroma (0+)
                if parts[1].parse::<f64>().is_err() {
                    return Err(format!("Invalid oklch() chroma value: {}", value));
                }
                
                // Validate hue (number, optionally with deg)
                let hue_str = parts[2].trim_end_matches("deg");
                if hue_str.parse::<f64>().is_err() {
                    return Err(format!("Invalid oklch() hue value: {}", value));
                }
                
                // Validate alpha if present
                if parts.len() >= 4 {
                    if parts[3].parse::<f64>().is_err() {
                        return Err(format!("Invalid oklch() alpha value: {}", value));
                    }
                }
                return Ok(());
            }
            return Err(format!("Invalid oklch() format (expected L C H [/ alpha]): {}", value));
        }
        
        // Named colors - comprehensive CSS named colors list
        let named_colors = [
            "aliceblue", "antiquewhite", "aqua", "aquamarine", "azure",
            "beige", "bisque", "black", "blanchedalmond", "blue", "blueviolet", "brown", "burlywood",
            "cadetblue", "chartreuse", "chocolate", "coral", "cornflowerblue", "cornsilk", "crimson", "cyan",
            "darkblue", "darkcyan", "darkgoldenrod", "darkgray", "darkgrey", "darkgreen", "darkkhaki",
            "darkmagenta", "darkolivegreen", "darkorange", "darkorchid", "darkred", "darksalmon",
            "darkseagreen", "darkslateblue", "darkslategray", "darkslategrey", "darkturquoise",
            "darkviolet", "deeppink", "deepskyblue", "dimgray", "dimgrey", "dodgerblue",
            "firebrick", "floralwhite", "forestgreen", "fuchsia",
            "gainsboro", "ghostwhite", "gold", "goldenrod", "gray", "grey", "green", "greenyellow",
            "honeydew", "hotpink", "indianred", "indigo", "ivory", "khaki",
            "lavender", "lavenderblush", "lawngreen", "lemonchiffon", "lightblue", "lightcoral",
            "lightcyan", "lightgoldenrodyellow", "lightgray", "lightgrey", "lightgreen", "lightpink",
            "lightsalmon", "lightseagreen", "lightskyblue", "lightslategray", "lightslategrey",
            "lightsteelblue", "lightyellow", "lime", "limegreen", "linen",
            "magenta", "maroon", "mediumaquamarine", "mediumblue", "mediumorchid", "mediumpurple",
            "mediumseagreen", "mediumslateblue", "mediumspringgreen", "mediumturquoise",
            "mediumvioletred", "midnightblue", "mintcream", "mistyrose", "moccasin",
            "navajowhite", "navy", "oldlace", "olive", "olivedrab", "orange", "orangered", "orchid",
            "palegoldenrod", "palegreen", "paleturquoise", "palevioletred", "papayawhip", "peachpuff",
            "peru", "pink", "plum", "powderblue", "purple",
            "rebeccapurple", "red", "rosybrown", "royalblue",
            "saddlebrown", "salmon", "sandybrown", "seagreen", "seashell", "sienna", "silver", "skyblue",
            "slateblue", "slategray", "slategrey", "snow", "springgreen", "steelblue",
            "tan", "teal", "thistle", "tomato", "transparent", "turquoise",
            "violet", "wheat", "white", "whitesmoke", "yellow", "yellowgreen",
        ];
        
        if named_colors.contains(&trimmed.to_lowercase().as_str()) {
            return Ok(());
        }
        
        Err(format!("Invalid color format (expected hex, rgb(), rgba(), hsl(), hsla(), oklch(), or named color): {}", value))
    }

    /// Validate dimension value (number with unit).
    fn validate_dimension(value: &str) -> Result<(), String> {
        let trimmed = value.trim();
        
        // Check for number followed by unit
        let valid_units = ["px", "rem", "em", "%", "vh", "vw", "vmin", "vmax", "ch", "ex", "cm", "mm", "in", "pt", "pc"];
        
        for unit in &valid_units {
            if trimmed.ends_with(unit) {
                let number_part = &trimmed[..trimmed.len() - unit.len()];
                if number_part.parse::<f64>().is_ok() {
                    return Ok(());
                }
            }
        }
        
        Err(format!("Invalid dimension format (expected number + unit): {}", value))
    }

    /// Validate font family value.
    fn validate_font_family(value: &str) -> Result<(), String> {
        // Font family can be any string or comma-separated list
        if value.trim().is_empty() {
            Err("Font family cannot be empty".to_string())
        } else {
            Ok(())
        }
    }

    /// Validate font weight value (100-900 or keywords).
    fn validate_font_weight(value: &str) -> Result<(), String> {
        let trimmed = value.trim();
        
        // Check for numeric weight (100-900)
        if let Ok(weight) = trimmed.parse::<u32>() {
            if (100..=900).contains(&weight) && weight % 100 == 0 {
                return Ok(());
            }
            return Err(format!("Font weight must be between 100-900 in increments of 100: {}", value));
        }
        
        // Check for keyword
        let valid_keywords = ["normal", "bold", "bolder", "lighter"];
        if valid_keywords.contains(&trimmed) {
            return Ok(());
        }
        
        Err(format!("Invalid font weight: {}", value))
    }

    /// Validate duration value (number with ms or s).
    fn validate_duration(value: &str) -> Result<(), String> {
        let trimmed = value.trim();
        
        if trimmed.ends_with("ms") {
            let number_part = &trimmed[..trimmed.len() - 2];
            if number_part.parse::<f64>().is_ok() {
                return Ok(());
            }
        } else if trimmed.ends_with('s') {
            let number_part = &trimmed[..trimmed.len() - 1];
            if number_part.parse::<f64>().is_ok() {
                return Ok(());
            }
        }
        
        Err(format!("Invalid duration format (expected number + 'ms' or 's'): {}", value))
    }

    /// Validate cubic bezier value (four comma-separated numbers).
    fn validate_cubic_bezier(value: &str) -> Result<(), String> {
        let parts: Vec<&str> = value.split(',').map(|s| s.trim()).collect();
        
        if parts.len() != 4 {
            return Err(format!("Cubic bezier requires exactly 4 values: {}", value));
        }
        
        for part in parts {
            if part.parse::<f64>().is_err() {
                return Err(format!("Invalid cubic bezier value (must be numbers): {}", value));
            }
        }
        
        Ok(())
    }

    /// Validate number value (unitless numeric).
    fn validate_number(value: &str) -> Result<(), String> {
        if value.trim().parse::<f64>().is_ok() {
            Ok(())
        } else {
            Err(format!("Invalid number format: {}", value))
        }
    }

    /// Validate stroke style value.
    fn validate_stroke_style(value: &str) -> Result<(), String> {
        let trimmed = value.trim();
        let valid_styles = ["solid", "dashed", "dotted", "double", "groove", "ridge", "inset", "outset", "none", "hidden"];
        
        if valid_styles.contains(&trimmed) {
            return Ok(());
        }
        
        // Also allow dash array patterns like "5 10" or "5, 10, 2"
        let parts: Vec<&str> = trimmed.split(|c: char| c == ',' || c.is_whitespace())
            .filter(|s| !s.is_empty())
            .collect();
        
        if !parts.is_empty() && parts.iter().all(|p| p.parse::<f64>().is_ok()) {
            return Ok(());
        }
        
        Err(format!("Invalid stroke style (expected keyword or dash pattern): {}", value))
    }

    /// Validate typography composite token.
    fn validate_typography_composite(map: &HashMap<String, W3CTokenValue>, path: &str) -> Result<(), CompilerError> {
        let required_fields = ["fontFamily", "fontSize", "fontWeight"];
        let mut missing = Vec::new();
        
        for field in &required_fields {
            if !map.contains_key(*field) {
                missing.push(*field);
            }
        }
        
        if !missing.is_empty() {
            return Err(CompilerError::validation(
                format!("Typography token '{}' is missing required properties: {}", path, missing.join(", ")),
                Span::empty(),
            ));
        }
        
        Ok(())
    }

    /// Validate shadow composite token.
    fn validate_shadow_composite(map: &HashMap<String, W3CTokenValue>, path: &str) -> Result<(), CompilerError> {
        let required_fields = ["offsetX", "offsetY", "blur", "color"];
        let mut missing = Vec::new();
        
        for field in &required_fields {
            if !map.contains_key(*field) {
                missing.push(*field);
            }
        }
        
        if !missing.is_empty() {
            return Err(CompilerError::validation(
                format!("Shadow token '{}' is missing required properties: {}", path, missing.join(", ")),
                Span::empty(),
            ));
        }
        
        Ok(())
    }

    /// Validate border composite token.
    fn validate_border_composite(map: &HashMap<String, W3CTokenValue>, path: &str) -> Result<(), CompilerError> {
        let required_fields = ["width", "style", "color"];
        let mut missing = Vec::new();
        
        for field in &required_fields {
            if !map.contains_key(*field) {
                missing.push(*field);
            }
        }
        
        if !missing.is_empty() {
            return Err(CompilerError::validation(
                format!("Border token '{}' is missing required properties: {}", path, missing.join(", ")),
                Span::empty(),
            ));
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== Color Validation Tests =====
    
    #[test]
    fn test_validate_hex_color_3_digit() {
        assert!(TokenTypeValidator::validate_color("#fff").is_ok());
        assert!(TokenTypeValidator::validate_color("#000").is_ok());
        assert!(TokenTypeValidator::validate_color("#abc").is_ok());
        assert!(TokenTypeValidator::validate_color("#ABC").is_ok());
    }

    #[test]
    fn test_validate_hex_color_4_digit() {
        assert!(TokenTypeValidator::validate_color("#ffff").is_ok());
        assert!(TokenTypeValidator::validate_color("#0000").is_ok());
        assert!(TokenTypeValidator::validate_color("#abcd").is_ok());
    }

    #[test]
    fn test_validate_hex_color_6_digit() {
        assert!(TokenTypeValidator::validate_color("#3b82f6").is_ok());
        assert!(TokenTypeValidator::validate_color("#ffffff").is_ok());
        assert!(TokenTypeValidator::validate_color("#000000").is_ok());
        assert!(TokenTypeValidator::validate_color("#ABCDEF").is_ok());
    }

    #[test]
    fn test_validate_hex_color_8_digit() {
        assert!(TokenTypeValidator::validate_color("#ff5733aa").is_ok());
        assert!(TokenTypeValidator::validate_color("#00000000").is_ok());
        assert!(TokenTypeValidator::validate_color("#ffffffff").is_ok());
    }

    #[test]
    fn test_validate_hex_color_invalid() {
        assert!(TokenTypeValidator::validate_color("#ff").is_err());
        assert!(TokenTypeValidator::validate_color("#fffff").is_err());
        assert!(TokenTypeValidator::validate_color("#gggggg").is_err());
        assert!(TokenTypeValidator::validate_color("3b82f6").is_err());
        assert!(TokenTypeValidator::validate_color("#").is_err());
    }

    #[test]
    fn test_validate_rgb_color() {
        assert!(TokenTypeValidator::validate_color("rgb(255, 0, 0)").is_ok());
        assert!(TokenTypeValidator::validate_color("rgb(0, 128, 255)").is_ok());
        assert!(TokenTypeValidator::validate_color("rgb(100%, 50%, 0%)").is_ok());
    }

    #[test]
    fn test_validate_rgba_color() {
        assert!(TokenTypeValidator::validate_color("rgba(255, 0, 0, 0.5)").is_ok());
        assert!(TokenTypeValidator::validate_color("rgba(0, 128, 255, 1)").is_ok());
        assert!(TokenTypeValidator::validate_color("rgba(100%, 50%, 0%, 0.8)").is_ok());
    }

    #[test]
    fn test_validate_hsl_color() {
        assert!(TokenTypeValidator::validate_color("hsl(120, 100%, 50%)").is_ok());
        assert!(TokenTypeValidator::validate_color("hsl(240deg, 50%, 75%)").is_ok());
        assert!(TokenTypeValidator::validate_color("hsl(0, 0%, 100%)").is_ok());
    }

    #[test]
    fn test_validate_hsla_color() {
        assert!(TokenTypeValidator::validate_color("hsla(120, 100%, 50%, 0.5)").is_ok());
        assert!(TokenTypeValidator::validate_color("hsla(240deg, 50%, 75%, 1)").is_ok());
        assert!(TokenTypeValidator::validate_color("hsla(0, 0%, 100%, 0.3)").is_ok());
    }

    #[test]
    fn test_validate_oklch_color() {
        // Modern space-separated syntax
        assert!(TokenTypeValidator::validate_color("oklch(70% 0.15 180)").is_ok());
        assert!(TokenTypeValidator::validate_color("oklch(0.7 0.15 180)").is_ok());
        assert!(TokenTypeValidator::validate_color("oklch(50% 0.2 240deg)").is_ok());
        
        // With alpha (slash syntax)
        assert!(TokenTypeValidator::validate_color("oklch(70% 0.15 180 / 0.5)").is_ok());
        assert!(TokenTypeValidator::validate_color("oklch(0.7 0.15 180 / 1)").is_ok());
        
        // Legacy comma syntax
        assert!(TokenTypeValidator::validate_color("oklch(70%, 0.15, 180)").is_ok());
    }

    #[test]
    fn test_validate_oklch_color_invalid() {
        // Missing components
        assert!(TokenTypeValidator::validate_color("oklch(70% 0.15)").is_err());
        // Invalid lightness
        assert!(TokenTypeValidator::validate_color("oklch(-10% 0.15 180)").is_err());
        // Invalid chroma
        assert!(TokenTypeValidator::validate_color("oklch(70% abc 180)").is_err());
        // Invalid hue
        assert!(TokenTypeValidator::validate_color("oklch(70% 0.15 abc)").is_err());
    }

    #[test]
    fn test_validate_named_colors() {
        assert!(TokenTypeValidator::validate_color("red").is_ok());
        assert!(TokenTypeValidator::validate_color("blue").is_ok());
        assert!(TokenTypeValidator::validate_color("transparent").is_ok());
        assert!(TokenTypeValidator::validate_color("rebeccapurple").is_ok());
        assert!(TokenTypeValidator::validate_color("cornflowerblue").is_ok());
        assert!(TokenTypeValidator::validate_color("darkslategray").is_ok());
    }

    #[test]
    fn test_validate_color_invalid_formats() {
        assert!(TokenTypeValidator::validate_color("invalid").is_err());
        assert!(TokenTypeValidator::validate_color("rgb(300, 0, 0)").is_err());
        assert!(TokenTypeValidator::validate_color("rgba(255, 0, 0)").is_err()); // Missing alpha
        assert!(TokenTypeValidator::validate_color("hsl(120, 100%)").is_err()); // Missing lightness
        assert!(TokenTypeValidator::validate_color("notacolor").is_err());
    }

    // ===== Dimension Validation Tests =====
    
    #[test]
    fn test_validate_dimension_px() {
        assert!(TokenTypeValidator::validate_dimension("16px").is_ok());
        assert!(TokenTypeValidator::validate_dimension("0px").is_ok());
        assert!(TokenTypeValidator::validate_dimension("100.5px").is_ok());
        assert!(TokenTypeValidator::validate_dimension("-10px").is_ok());
    }

    #[test]
    fn test_validate_dimension_rem_em() {
        assert!(TokenTypeValidator::validate_dimension("1.5rem").is_ok());
        assert!(TokenTypeValidator::validate_dimension("2em").is_ok());
        assert!(TokenTypeValidator::validate_dimension("0.875rem").is_ok());
    }

    #[test]
    fn test_validate_dimension_percentage() {
        assert!(TokenTypeValidator::validate_dimension("100%").is_ok());
        assert!(TokenTypeValidator::validate_dimension("50%").is_ok());
        assert!(TokenTypeValidator::validate_dimension("0%").is_ok());
    }

    #[test]
    fn test_validate_dimension_viewport_units() {
        assert!(TokenTypeValidator::validate_dimension("100vh").is_ok());
        assert!(TokenTypeValidator::validate_dimension("50vw").is_ok());
        assert!(TokenTypeValidator::validate_dimension("10vmin").is_ok());
        assert!(TokenTypeValidator::validate_dimension("20vmax").is_ok());
    }

    #[test]
    fn test_validate_dimension_absolute_units() {
        assert!(TokenTypeValidator::validate_dimension("1cm").is_ok());
        assert!(TokenTypeValidator::validate_dimension("10mm").is_ok());
        assert!(TokenTypeValidator::validate_dimension("1in").is_ok());
        assert!(TokenTypeValidator::validate_dimension("12pt").is_ok());
        assert!(TokenTypeValidator::validate_dimension("1pc").is_ok());
    }

    #[test]
    fn test_validate_dimension_font_relative() {
        assert!(TokenTypeValidator::validate_dimension("2ch").is_ok());
        assert!(TokenTypeValidator::validate_dimension("1.5ex").is_ok());
    }

    #[test]
    fn test_validate_dimension_invalid() {
        assert!(TokenTypeValidator::validate_dimension("invalid").is_err());
        assert!(TokenTypeValidator::validate_dimension("16").is_err()); // Missing unit
        assert!(TokenTypeValidator::validate_dimension("px16").is_err()); // Wrong order
        assert!(TokenTypeValidator::validate_dimension("16xyz").is_err()); // Invalid unit
    }

    // ===== Font Family Validation Tests =====
    
    #[test]
    fn test_validate_font_family_single() {
        assert!(TokenTypeValidator::validate_font_family("Arial").is_ok());
        assert!(TokenTypeValidator::validate_font_family("Helvetica").is_ok());
        assert!(TokenTypeValidator::validate_font_family("Inter").is_ok());
    }

    #[test]
    fn test_validate_font_family_stack() {
        assert!(TokenTypeValidator::validate_font_family("Arial, sans-serif").is_ok());
        assert!(TokenTypeValidator::validate_font_family("'Times New Roman', serif").is_ok());
        assert!(TokenTypeValidator::validate_font_family("Inter, Helvetica, Arial, sans-serif").is_ok());
    }

    #[test]
    fn test_validate_font_family_invalid() {
        assert!(TokenTypeValidator::validate_font_family("").is_err());
        assert!(TokenTypeValidator::validate_font_family("   ").is_err());
    }

    // ===== Font Weight Validation Tests =====
    
    #[test]
    fn test_validate_font_weight_numeric() {
        assert!(TokenTypeValidator::validate_font_weight("100").is_ok());
        assert!(TokenTypeValidator::validate_font_weight("400").is_ok());
        assert!(TokenTypeValidator::validate_font_weight("700").is_ok());
        assert!(TokenTypeValidator::validate_font_weight("900").is_ok());
    }

    #[test]
    fn test_validate_font_weight_keywords() {
        assert!(TokenTypeValidator::validate_font_weight("normal").is_ok());
        assert!(TokenTypeValidator::validate_font_weight("bold").is_ok());
        assert!(TokenTypeValidator::validate_font_weight("bolder").is_ok());
        assert!(TokenTypeValidator::validate_font_weight("lighter").is_ok());
    }

    #[test]
    fn test_validate_font_weight_invalid() {
        assert!(TokenTypeValidator::validate_font_weight("999").is_err());
        assert!(TokenTypeValidator::validate_font_weight("50").is_err());
        assert!(TokenTypeValidator::validate_font_weight("450").is_err()); // Not multiple of 100
        assert!(TokenTypeValidator::validate_font_weight("heavy").is_err());
        assert!(TokenTypeValidator::validate_font_weight("").is_err());
    }

    // ===== Duration Validation Tests =====
    
    #[test]
    fn test_validate_duration_milliseconds() {
        assert!(TokenTypeValidator::validate_duration("200ms").is_ok());
        assert!(TokenTypeValidator::validate_duration("0ms").is_ok());
        assert!(TokenTypeValidator::validate_duration("1000ms").is_ok());
        assert!(TokenTypeValidator::validate_duration("150.5ms").is_ok());
    }

    #[test]
    fn test_validate_duration_seconds() {
        assert!(TokenTypeValidator::validate_duration("1s").is_ok());
        assert!(TokenTypeValidator::validate_duration("0.5s").is_ok());
        assert!(TokenTypeValidator::validate_duration("2.5s").is_ok());
    }

    #[test]
    fn test_validate_duration_invalid() {
        assert!(TokenTypeValidator::validate_duration("invalid").is_err());
        assert!(TokenTypeValidator::validate_duration("200").is_err()); // Missing unit
        assert!(TokenTypeValidator::validate_duration("ms200").is_err()); // Wrong order
        assert!(TokenTypeValidator::validate_duration("200m").is_err()); // Wrong unit
    }

    // ===== Cubic Bezier Validation Tests =====
    
    #[test]
    fn test_validate_cubic_bezier_valid() {
        assert!(TokenTypeValidator::validate_cubic_bezier("0.25, 0.1, 0.25, 1").is_ok());
        assert!(TokenTypeValidator::validate_cubic_bezier("0, 0, 1, 1").is_ok());
        assert!(TokenTypeValidator::validate_cubic_bezier("0.42, 0, 0.58, 1").is_ok());
        assert!(TokenTypeValidator::validate_cubic_bezier("0.17, 0.67, 0.83, 0.67").is_ok());
    }

    #[test]
    fn test_validate_cubic_bezier_invalid() {
        assert!(TokenTypeValidator::validate_cubic_bezier("0.25, 0.1, 0.25").is_err()); // Only 3 values
        assert!(TokenTypeValidator::validate_cubic_bezier("0.25, 0.1, 0.25, 1, 0").is_err()); // 5 values
        assert!(TokenTypeValidator::validate_cubic_bezier("a, b, c, d").is_err()); // Non-numeric
        assert!(TokenTypeValidator::validate_cubic_bezier("").is_err());
    }

    // ===== Number Validation Tests =====
    
    #[test]
    fn test_validate_number_integers() {
        assert!(TokenTypeValidator::validate_number("42").is_ok());
        assert!(TokenTypeValidator::validate_number("0").is_ok());
        assert!(TokenTypeValidator::validate_number("-10").is_ok());
        assert!(TokenTypeValidator::validate_number("1000").is_ok());
    }

    #[test]
    fn test_validate_number_decimals() {
        assert!(TokenTypeValidator::validate_number("3.14").is_ok());
        assert!(TokenTypeValidator::validate_number("0.5").is_ok());
        assert!(TokenTypeValidator::validate_number("-2.5").is_ok());
        assert!(TokenTypeValidator::validate_number("1.618").is_ok());
    }

    #[test]
    fn test_validate_number_invalid() {
        assert!(TokenTypeValidator::validate_number("not-a-number").is_err());
        assert!(TokenTypeValidator::validate_number("42px").is_err()); // Has unit
        assert!(TokenTypeValidator::validate_number("").is_err());
        assert!(TokenTypeValidator::validate_number("abc").is_err());
    }

    // ===== Stroke Style Validation Tests =====
    
    #[test]
    fn test_validate_stroke_style_keywords() {
        assert!(TokenTypeValidator::validate_stroke_style("solid").is_ok());
        assert!(TokenTypeValidator::validate_stroke_style("dashed").is_ok());
        assert!(TokenTypeValidator::validate_stroke_style("dotted").is_ok());
        assert!(TokenTypeValidator::validate_stroke_style("double").is_ok());
        assert!(TokenTypeValidator::validate_stroke_style("none").is_ok());
    }

    #[test]
    fn test_validate_stroke_style_dash_patterns() {
        assert!(TokenTypeValidator::validate_stroke_style("5 10").is_ok());
        assert!(TokenTypeValidator::validate_stroke_style("5, 10, 2").is_ok());
        assert!(TokenTypeValidator::validate_stroke_style("10 5 2 5").is_ok());
        assert!(TokenTypeValidator::validate_stroke_style("1,2,3,4").is_ok());
    }

    #[test]
    fn test_validate_stroke_style_invalid() {
        assert!(TokenTypeValidator::validate_stroke_style("invalid-style").is_err());
        assert!(TokenTypeValidator::validate_stroke_style("5 abc").is_err());
        assert!(TokenTypeValidator::validate_stroke_style("").is_err());
    }

    // ===== Composite Token Validation Tests =====
    
    #[test]
    fn test_validate_typography_composite_valid() {
        let mut map = HashMap::new();
        map.insert("fontFamily".to_string(), W3CTokenValue::Literal("Inter".to_string()));
        map.insert("fontSize".to_string(), W3CTokenValue::Literal("16px".to_string()));
        map.insert("fontWeight".to_string(), W3CTokenValue::Literal("400".to_string()));
        
        assert!(TokenTypeValidator::validate_typography_composite(&map, "test.typography").is_ok());
    }

    #[test]
    fn test_validate_typography_composite_missing_fields() {
        let mut map = HashMap::new();
        map.insert("fontFamily".to_string(), W3CTokenValue::Literal("Inter".to_string()));
        // Missing fontSize and fontWeight
        
        let result = TokenTypeValidator::validate_typography_composite(&map, "test.typography");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_shadow_composite_valid() {
        let mut map = HashMap::new();
        map.insert("offsetX".to_string(), W3CTokenValue::Literal("2px".to_string()));
        map.insert("offsetY".to_string(), W3CTokenValue::Literal("4px".to_string()));
        map.insert("blur".to_string(), W3CTokenValue::Literal("8px".to_string()));
        map.insert("color".to_string(), W3CTokenValue::Literal("#000000".to_string()));
        
        assert!(TokenTypeValidator::validate_shadow_composite(&map, "test.shadow").is_ok());
    }

    #[test]
    fn test_validate_shadow_composite_missing_fields() {
        let mut map = HashMap::new();
        map.insert("offsetX".to_string(), W3CTokenValue::Literal("2px".to_string()));
        // Missing offsetY, blur, color
        
        let result = TokenTypeValidator::validate_shadow_composite(&map, "test.shadow");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_border_composite_valid() {
        let mut map = HashMap::new();
        map.insert("width".to_string(), W3CTokenValue::Literal("1px".to_string()));
        map.insert("style".to_string(), W3CTokenValue::Literal("solid".to_string()));
        map.insert("color".to_string(), W3CTokenValue::Literal("#000000".to_string()));
        
        assert!(TokenTypeValidator::validate_border_composite(&map, "test.border").is_ok());
    }

    #[test]
    fn test_validate_border_composite_missing_fields() {
        let mut map = HashMap::new();
        map.insert("width".to_string(), W3CTokenValue::Literal("1px".to_string()));
        // Missing style and color
        
        let result = TokenTypeValidator::validate_border_composite(&map, "test.border");
        assert!(result.is_err());
    }

    // ===== Integration Tests =====
    
    #[test]
    fn test_validate_token_with_type() {
        let token = W3CToken {
            path: "color.primary".to_string(),
            value: W3CTokenValue::Literal("#3b82f6".to_string()),
            token_type: Some(W3CTokenType::Color),
            description: None,
        };
        
        assert!(TokenTypeValidator::validate(&token).is_ok());
    }

    #[test]
    fn test_validate_token_type_mismatch() {
        let token = W3CToken {
            path: "color.primary".to_string(),
            value: W3CTokenValue::Literal("not-a-color".to_string()),
            token_type: Some(W3CTokenType::Color),
            description: None,
        };
        
        assert!(TokenTypeValidator::validate(&token).is_err());
    }

    #[test]
    fn test_validate_token_no_type() {
        let token = W3CToken {
            path: "some.token".to_string(),
            value: W3CTokenValue::Literal("any value".to_string()),
            token_type: None,
            description: None,
        };
        
        // Should pass validation when no type is declared
        assert!(TokenTypeValidator::validate(&token).is_ok());
    }

    #[test]
    fn test_validate_token_reference() {
        let token = W3CToken {
            path: "color.secondary".to_string(),
            value: W3CTokenValue::Reference("{color.primary}".to_string()),
            token_type: Some(W3CTokenType::Color),
            description: None,
        };
        
        // References should pass validation (resolved later)
        assert!(TokenTypeValidator::validate(&token).is_ok());
    }
}

    // ===== Type Mismatch Error Reporting Tests (Sub-task 3.3) =====
    
    #[test]
    fn test_type_mismatch_error_message_format() {
        // Sub-task 3.3: Type mismatch errors include expected type and found value
        let token = W3CToken {
            path: "spacing.base".to_string(),
            value: W3CTokenValue::Literal("not-a-dimension".to_string()),
            token_type: Some(W3CTokenType::Dimension),
            description: None,
        };
        
        let result = TokenTypeValidator::validate(&token);
        assert!(result.is_err());
        
        let error = result.unwrap_err();
        assert!(error.message.contains("dimension"), "Error should mention expected type");
        assert!(error.message.contains("spacing.base"), "Error should mention token path");
        assert!(error.message.contains("not-a-dimension"), "Error should mention found value");
    }

    #[test]
    fn test_composite_type_error_lists_missing_properties() {
        // Sub-task 3.2 & 3.3: Composite type errors list missing properties
        let mut map = HashMap::new();
        map.insert("fontFamily".to_string(), W3CTokenValue::Literal("Inter".to_string()));
        // Missing fontSize and fontWeight
        
        let result = TokenTypeValidator::validate_typography_composite(&map, "text.heading");
        assert!(result.is_err());
        
        let error = result.unwrap_err();
        assert!(error.message.contains("missing required properties"), "Error should mention missing properties");
        assert!(error.message.contains("fontSize") || error.message.contains("fontWeight"), 
                "Error should list specific missing properties");
    }

    #[test]
    fn test_primitive_type_error_provides_format_hint() {
        // Sub-task 3.3: Primitive type errors provide format hints
        let test_cases = vec![
            (W3CTokenType::Color, "invalid-color", "hex, rgb"),
            (W3CTokenType::Dimension, "no-unit", "number + unit"),
            (W3CTokenType::Duration, "100", "ms' or 's"),
            (W3CTokenType::CubicBezier, "0.5, 0.5", "4 values"),
        ];
        
        for (token_type, invalid_value, expected_hint) in test_cases {
            let token = W3CToken {
                path: "test.token".to_string(),
                value: W3CTokenValue::Literal(invalid_value.to_string()),
                token_type: Some(token_type.clone()),
                description: None,
            };
            
            let result = TokenTypeValidator::validate(&token);
            assert!(result.is_err(), "Should error for invalid {} value", token_type.as_str());
            
            let error = result.unwrap_err();
            assert!(
                error.message.to_lowercase().contains(expected_hint),
                "Error for {} should provide format hint containing '{}', got: {}",
                token_type.as_str(),
                expected_hint,
                error.message
            );
        }
    }
