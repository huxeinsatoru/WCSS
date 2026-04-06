use crate::error::{CompilerError, ErrorKind, W3CErrorKind};
use crate::w3c_parser::{W3CToken, W3CTokenType, W3CTokenValue};
use crate::ast::Span;

/// Configuration for token transformations.
#[derive(Debug, Clone)]
pub struct TransformConfig {
    pub rules: Vec<TransformRule>,
}

impl Default for TransformConfig {
    fn default() -> Self {
        Self {
            rules: Vec::new(),
        }
    }
}

impl TransformConfig {
    /// Create a new transform config with the given rules.
    pub fn new(rules: Vec<TransformRule>) -> Self {
        Self { rules }
    }

    /// Apply all transformation rules to the given tokens.
    pub fn apply(&self, tokens: &mut [W3CToken]) -> Result<(), Vec<CompilerError>> {
        let mut errors = Vec::new();

        for rule in &self.rules {
            for token in tokens.iter_mut() {
                if rule.matcher.matches(token) {
                    if let Err(e) = rule.transformation.apply(token) {
                        errors.push(e);
                    }
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// A single transformation rule with a matcher and transformation.
#[derive(Debug, Clone)]
pub struct TransformRule {
    pub matcher: TokenMatcher,
    pub transformation: Transformation,
}

impl TransformRule {
    /// Create a new transformation rule.
    pub fn new(matcher: TokenMatcher, transformation: Transformation) -> Self {
        Self {
            matcher,
            transformation,
        }
    }
}

/// Matcher for selecting tokens to transform.
#[derive(Debug, Clone)]
pub enum TokenMatcher {
    /// Match by exact path
    Path(String),
    /// Match by token type
    Type(W3CTokenType),
    /// Match by path pattern (glob-style)
    PathPattern(String),
}

impl TokenMatcher {
    /// Check if this matcher matches the given token.
    pub fn matches(&self, token: &W3CToken) -> bool {
        match self {
            TokenMatcher::Path(path) => token.path == *path,
            TokenMatcher::Type(token_type) => {
                token.token_type.as_ref() == Some(token_type)
            }
            TokenMatcher::PathPattern(pattern) => {
                glob_match(&token.path, pattern)
            }
        }
    }
}

/// Simple glob matching for path patterns.
fn glob_match(path: &str, pattern: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if pattern.starts_with("*") && pattern.ends_with("*") {
        let middle = &pattern[1..pattern.len()-1];
        return path.contains(middle);
    }
    if pattern.ends_with("*") {
        let prefix = &pattern[..pattern.len()-1];
        return path.starts_with(prefix);
    }
    if pattern.starts_with("*") {
        let suffix = &pattern[1..];
        return path.ends_with(suffix);
    }
    path == pattern
}

/// A transformation to apply to a token value.
#[derive(Debug, Clone, Copy)]
pub enum Transformation {
    /// Lighten a color by a percentage (0-100)
    ColorLighten(f64),
    /// Darken a color by a percentage (0-100)
    ColorDarken(f64),
    /// Saturate a color by a percentage (0-100)
    ColorSaturate(f64),
    /// Desaturate a color by a percentage (0-100)
    ColorDesaturate(f64),
    /// Adjust alpha by a factor (0-1)
    ColorAdjustAlpha(f64),
    /// Scale a dimension by a factor
    DimensionScale(f64),
    /// Add a value to a dimension
    DimensionAdd(f64),
    /// Subtract a value from a dimension
    DimensionSubtract(f64),
}

impl Transformation {
    /// Apply this transformation to a token.
    pub fn apply(&self, token: &mut W3CToken) -> Result<(), CompilerError> {
        match self {
            Transformation::ColorLighten(percent) => {
                Self::transform_color(token, |color| lighten_color(color, *percent))
            }
            Transformation::ColorDarken(percent) => {
                Self::transform_color(token, |color| darken_color(color, *percent))
            }
            Transformation::ColorSaturate(percent) => {
                Self::transform_color(token, |color| saturate_color(color, *percent))
            }
            Transformation::ColorDesaturate(percent) => {
                Self::transform_color(token, |color| desaturate_color(color, *percent))
            }
            Transformation::ColorAdjustAlpha(factor) => {
                Self::transform_color(token, |color| adjust_alpha(color, *factor))
            }
            Transformation::DimensionScale(factor) => {
                Self::transform_dimension(token, |dim| scale_dimension(dim, *factor))
            }
            Transformation::DimensionAdd(value) => {
                Self::transform_dimension(token, |dim| add_to_dimension(dim, *value))
            }
            Transformation::DimensionSubtract(value) => {
                Self::transform_dimension(token, |dim| subtract_from_dimension(dim, *value))
            }
        }
    }

    /// Transform a color token value.
    fn transform_color<F>(token: &mut W3CToken, transform: F) -> Result<(), CompilerError>
    where
        F: FnOnce(&str) -> Result<String, String>,
    {
        // Check if this is a color token
        let is_color = token.token_type.as_ref() == Some(&W3CTokenType::Color);
        
        match &mut token.value {
            W3CTokenValue::Literal(color) if is_color => {
                match transform(color) {
                    Ok(new_color) => {
                        *color = new_color;
                        Ok(())
                    }
                    Err(msg) => Err(CompilerError {
                        kind: ErrorKind::W3C(W3CErrorKind::InvalidTransformation),
                        message: format!("Color transformation failed for '{}': {}", token.path, msg),
                        span: Span::empty(),
                        suggestion: None,
                    }),
                }
            }
            _ => Ok(()), // Not a color, skip
        }
    }

    /// Transform a dimension token value.
    fn transform_dimension<F>(token: &mut W3CToken, transform: F) -> Result<(), CompilerError>
    where
        F: FnOnce(&str) -> Result<String, String>,
    {
        let is_dimension = token.token_type.as_ref() == Some(&W3CTokenType::Dimension);
        
        match &mut token.value {
            W3CTokenValue::Literal(dim) if is_dimension => {
                match transform(dim) {
                    Ok(new_dim) => {
                        *dim = new_dim;
                        Ok(())
                    }
                    Err(msg) => Err(CompilerError {
                        kind: ErrorKind::W3C(W3CErrorKind::InvalidTransformation),
                        message: format!("Dimension transformation failed for '{}': {}", token.path, msg),
                        span: Span::empty(),
                        suggestion: None,
                    }),
                }
            }
            _ => Ok(()), // Not a dimension, skip
        }
    }
}

/// Lighten a hex color by a percentage.
fn lighten_color(color: &str, percent: f64) -> Result<String, String> {
    let (r, g, b, a) = parse_hex_color(color)?;
    
    let factor = percent / 100.0;
    let new_r = (r as f64 + (255.0 - r as f64) * factor) as u8;
    let new_g = (g as f64 + (255.0 - g as f64) * factor) as u8;
    let new_b = (b as f64 + (255.0 - b as f64) * factor) as u8;
    
    if a == 255 {
        Ok(format!("#{:02X}{:02X}{:02X}", new_r, new_g, new_b))
    } else {
        Ok(format!("#{:02X}{:02X}{:02X}{:02X}", new_r, new_g, new_b, a))
    }
}

/// Darken a hex color by a percentage.
fn darken_color(color: &str, percent: f64) -> Result<String, String> {
    let (r, g, b, a) = parse_hex_color(color)?;
    
    let factor = 1.0 - (percent / 100.0);
    let new_r = (r as f64 * factor) as u8;
    let new_g = (g as f64 * factor) as u8;
    let new_b = (b as f64 * factor) as u8;
    
    if a == 255 {
        Ok(format!("#{:02X}{:02X}{:02X}", new_r, new_g, new_b))
    } else {
        Ok(format!("#{:02X}{:02X}{:02X}{:02X}", new_r, new_g, new_b, a))
    }
}

/// Saturate a color (placeholder - would need HSL conversion).
fn saturate_color(color: &str, _percent: f64) -> Result<String, String> {
    // For simplicity, just return the original color
    // Real implementation would convert to HSL, adjust saturation, convert back
    Ok(color.to_string())
}

/// Desaturate a color (placeholder - would need HSL conversion).
fn desaturate_color(color: &str, _percent: f64) -> Result<String, String> {
    // For simplicity, just return the original color
    Ok(color.to_string())
}

/// Adjust alpha of a color.
fn adjust_alpha(color: &str, factor: f64) -> Result<String, String> {
    let (r, g, b, a) = parse_hex_color(color)?;
    
    let new_a = ((a as f64 * factor).clamp(0.0, 255.0)) as u8;
    
    Ok(format!("#{:02X}{:02X}{:02X}{:02X}", r, g, b, new_a))
}

/// Parse a hex color into RGBA components.
fn parse_hex_color(color: &str) -> Result<(u8, u8, u8, u8), String> {
    let hex = color.trim().trim_start_matches('#');
    
    match hex.len() {
        3 => {
            let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).unwrap_or(0);
            let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).unwrap_or(0);
            let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).unwrap_or(0);
            Ok((r, g, b, 255))
        }
        4 => {
            let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).unwrap_or(0);
            let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).unwrap_or(0);
            let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).unwrap_or(0);
            let a = u8::from_str_radix(&hex[3..4].repeat(2), 16).unwrap_or(255);
            Ok((r, g, b, a))
        }
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
            Ok((r, g, b, 255))
        }
        8 => {
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
            let a = u8::from_str_radix(&hex[6..8], 16).unwrap_or(255);
            Ok((r, g, b, a))
        }
        _ => Err(format!("Invalid hex color format: {}", color)),
    }
}

/// Scale a dimension value.
fn scale_dimension(dim: &str, factor: f64) -> Result<String, String> {
    let numeric_part: String = dim.chars()
        .take_while(|c| c.is_numeric() || *c == '.' || *c == '-')
        .collect();
    
    let unit = &dim[numeric_part.len()..];
    
    if let Ok(num) = numeric_part.parse::<f64>() {
        let new_num = num * factor;
        Ok(format!("{}{}", new_num, unit))
    } else {
        Err(format!("Invalid dimension: {}", dim))
    }
}

/// Add a value to a dimension.
fn add_to_dimension(dim: &str, value: f64) -> Result<String, String> {
    let numeric_part: String = dim.chars()
        .take_while(|c| c.is_numeric() || *c == '.' || *c == '-')
        .collect();
    
    let unit = &dim[numeric_part.len()..];
    
    if let Ok(num) = numeric_part.parse::<f64>() {
        let new_num = num + value;
        Ok(format!("{}{}", new_num, unit))
    } else {
        Err(format!("Invalid dimension: {}", dim))
    }
}

/// Subtract a value from a dimension.
fn subtract_from_dimension(dim: &str, value: f64) -> Result<String, String> {
    add_to_dimension(dim, -value)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_color_token(path: &str, value: &str) -> W3CToken {
        W3CToken {
            path: path.to_string(),
            value: W3CTokenValue::Literal(value.to_string()),
            token_type: Some(W3CTokenType::Color),
            description: None,
        }
    }

    fn create_dimension_token(path: &str, value: &str) -> W3CToken {
        W3CToken {
            path: path.to_string(),
            value: W3CTokenValue::Literal(value.to_string()),
            token_type: Some(W3CTokenType::Dimension),
            description: None,
        }
    }

    #[test]
    fn test_glob_match() {
        assert!(glob_match("color.primary", "*"));
        assert!(glob_match("color.primary", "color.*"));
        assert!(glob_match("color.primary", "*primary"));
        assert!(glob_match("color.primary", "color.primary"));
        assert!(!glob_match("spacing.base", "color.*"));
    }

    #[test]
    fn test_token_matcher() {
        let token = create_color_token("color.primary", "#3b82f6");
        
        let path_matcher = TokenMatcher::Path("color.primary".to_string());
        assert!(path_matcher.matches(&token));
        
        let type_matcher = TokenMatcher::Type(W3CTokenType::Color);
        assert!(type_matcher.matches(&token));
        
        let pattern_matcher = TokenMatcher::PathPattern("color.*".to_string());
        assert!(pattern_matcher.matches(&token));
    }

    #[test]
    fn test_lighten_color() {
        let result = lighten_color("#000000", 50.0).unwrap();
        assert_eq!(result, "#7F7F7F"); // ~50% gray
        
        let result = lighten_color("#000000", 100.0).unwrap();
        assert_eq!(result, "#FFFFFF"); // White
    }

    #[test]
    fn test_darken_color() {
        let result = darken_color("#FFFFFF", 50.0).unwrap();
        assert_eq!(result, "#7F7F7F"); // ~50% gray
        
        let result = darken_color("#FFFFFF", 100.0).unwrap();
        assert_eq!(result, "#000000"); // Black
    }

    #[test]
    fn test_adjust_alpha() {
        let result = adjust_alpha("#3b82f6", 0.5).unwrap();
        assert_eq!(result, "#3B82F67F"); // 50% alpha
    }

    #[test]
    fn test_scale_dimension() {
        let result = scale_dimension("16px", 2.0).unwrap();
        assert_eq!(result, "32px");
        
        let result = scale_dimension("1.5rem", 2.0).unwrap();
        assert_eq!(result, "3rem");
    }

    #[test]
    fn test_add_to_dimension() {
        let result = add_to_dimension("16px", 4.0).unwrap();
        assert_eq!(result, "20px");
    }

    #[test]
    fn test_transformation_apply_color() {
        let mut token = create_color_token("color.primary", "#3b82f6");
        let transform = Transformation::ColorLighten(20.0);
        
        assert!(transform.apply(&mut token).is_ok());
        // Color should be lightened (exact value depends on implementation)
        match &token.value {
            W3CTokenValue::Literal(v) => assert_ne!(v, "#3b82f6"),
            _ => panic!("Expected literal value"),
        }
    }

    #[test]
    fn test_transformation_apply_dimension() {
        let mut token = create_dimension_token("spacing.base", "16px");
        let transform = Transformation::DimensionScale(2.0);
        
        assert!(transform.apply(&mut token).is_ok());
        match &token.value {
            W3CTokenValue::Literal(v) => assert_eq!(v, "32px"),
            _ => panic!("Expected literal value"),
        }
    }

    #[test]
    fn test_transform_config_apply() {
        let mut tokens = vec![
            create_color_token("color.primary", "#3b82f6"),
            create_dimension_token("spacing.base", "16px"),
        ];
        
        let config = TransformConfig::new(vec![
            TransformRule::new(
                TokenMatcher::Type(W3CTokenType::Dimension),
                Transformation::DimensionScale(2.0),
            ),
        ]);
        
        assert!(config.apply(&mut tokens).is_ok());
        
        // Dimension should be scaled
        match &tokens[1].value {
            W3CTokenValue::Literal(v) => assert_eq!(v, "32px"),
            _ => panic!("Expected literal value"),
        }
    }
}
