use std::collections::HashMap;
use serde_json::Value as JsonValue;
use crate::error::{CompilerError, ErrorKind, W3CErrorKind};
use crate::ast::Span;

/// Parser for W3C Design Tokens JSON format.
pub struct W3CTokenParser {
    source: String,
    current_path: Vec<String>,
    /// Map of JSON paths to their byte positions in the source
    position_map: HashMap<String, (usize, usize, usize)>, // (byte_offset, line, column)
}

/// A parsed W3C Design Token.
#[derive(Debug, Clone, PartialEq)]
pub struct W3CToken {
    /// Flattened path using dot notation (e.g., "color.primary.500")
    pub path: String,
    /// Token value (literal, reference, or composite)
    pub value: W3CTokenValue,
    /// Optional W3C token type
    pub token_type: Option<W3CTokenType>,
    /// Optional description for documentation
    pub description: Option<String>,
}

/// W3C token value types.
#[derive(Debug, Clone, PartialEq)]
pub enum W3CTokenValue {
    /// Literal string value
    Literal(String),
    /// Reference to another token (e.g., "{color.primary}")
    Reference(String),
    /// Composite value with multiple properties (for typography, shadow, border)
    Composite(HashMap<String, W3CTokenValue>),
}

/// W3C Design Token types as defined in the specification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum W3CTokenType {
    Color,
    Dimension,
    FontFamily,
    FontWeight,
    Duration,
    CubicBezier,
    Number,
    StrokeStyle,
    Border,
    Shadow,
    Typography,
}

impl W3CTokenType {
    /// Parse a W3C token type from a string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "color" => Some(Self::Color),
            "dimension" => Some(Self::Dimension),
            "fontFamily" => Some(Self::FontFamily),
            "fontWeight" => Some(Self::FontWeight),
            "duration" => Some(Self::Duration),
            "cubicBezier" => Some(Self::CubicBezier),
            "number" => Some(Self::Number),
            "strokeStyle" => Some(Self::StrokeStyle),
            "border" => Some(Self::Border),
            "shadow" => Some(Self::Shadow),
            "typography" => Some(Self::Typography),
            _ => None,
        }
    }

    /// Convert token type to string representation.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Color => "color",
            Self::Dimension => "dimension",
            Self::FontFamily => "fontFamily",
            Self::FontWeight => "fontWeight",
            Self::Duration => "duration",
            Self::CubicBezier => "cubicBezier",
            Self::Number => "number",
            Self::StrokeStyle => "strokeStyle",
            Self::Border => "border",
            Self::Shadow => "shadow",
            Self::Typography => "typography",
        }
    }
}

impl W3CTokenParser {
    /// Create a new parser for the given JSON source.
    pub fn new(source: String) -> Self {
        Self {
            source: source.clone(),
            current_path: Vec::new(),
            position_map: HashMap::new(),
        }
    }

    /// Parse W3C Design Tokens JSON into a list of tokens.
    pub fn parse(json: &str) -> Result<Vec<W3CToken>, Vec<CompilerError>> {
        let mut parser = Self::new(json.to_string());
        
        // Parse JSON
        let root: JsonValue = match serde_json::from_str(json) {
            Ok(value) => value,
            Err(e) => {
                // Calculate byte offset from line and column
                let byte_offset = parser.calculate_byte_offset(e.line(), e.column());
                let span = Span::new(byte_offset, byte_offset + 1, e.line(), e.column());
                let error = CompilerError {
                    kind: ErrorKind::W3C(W3CErrorKind::InvalidJSON),
                    message: format!("Invalid JSON: {}", e),
                    span,
                    suggestion: Some("Check JSON syntax for missing commas, brackets, or quotes".to_string()),
                };
                return Err(vec![error]);
            }
        };

        // Build position map for better error reporting
        parser.build_position_map(json);

        // Parse token groups
        if let JsonValue::Object(obj) = root {
            parser.parse_token_group(&obj)
        } else {
            let span = Span::new(0, json.len().min(50), 1, 1);
            let error = CompilerError {
                kind: ErrorKind::W3C(W3CErrorKind::InvalidStructure),
                message: "W3C Design Tokens file must be a JSON object".to_string(),
                span,
                suggestion: Some("Wrap your tokens in a top-level JSON object: { ... }".to_string()),
            };
            Err(vec![error])
        }
    }

    /// Calculate byte offset from line and column numbers.
    fn calculate_byte_offset(&self, line: usize, column: usize) -> usize {
        let mut current_line = 1;
        let mut current_col = 1;
        
        for (idx, ch) in self.source.chars().enumerate() {
            if current_line == line && current_col == column {
                return idx;
            }
            if ch == '\n' {
                current_line += 1;
                current_col = 1;
            } else {
                current_col += 1;
            }
        }
        
        self.source.len()
    }

    /// Build a map of JSON paths to their positions in the source.
    /// This is a simplified approach that estimates positions based on string search.
    fn build_position_map(&mut self, _json: &str) {
        // Store root position
        self.position_map.insert("".to_string(), (0, 1, 1));
    }

    /// Get the span for the current path.
    fn get_current_span(&self) -> Span {
        // Try to find the position in the source by searching for the key
        if let Some(last_key) = self.current_path.last() {
            // Search for the key in the source
            let search_pattern = format!("\"{}\"", last_key);
            if let Some(pos) = self.source.find(&search_pattern) {
                let (line, col) = self.calculate_line_col(pos);
                return Span::new(pos, pos + search_pattern.len(), line, col);
            }
        }
        
        // Fallback to a default span
        Span::new(0, 0, 1, 1)
    }

    /// Calculate line and column from byte offset.
    fn calculate_line_col(&self, byte_offset: usize) -> (usize, usize) {
        let mut line = 1;
        let mut col = 1;
        
        for (idx, ch) in self.source.chars().enumerate() {
            if idx >= byte_offset {
                break;
            }
            if ch == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }
        
        (line, col)
    }

    /// Parse a token group (object that may contain tokens or nested groups).
    fn parse_token_group(&mut self, obj: &serde_json::Map<String, JsonValue>) -> Result<Vec<W3CToken>, Vec<CompilerError>> {
        let mut tokens = Vec::new();
        let mut errors = Vec::new();

        for (key, value) in obj {
            // Skip keys starting with $ at the group level
            if key.starts_with('$') {
                continue;
            }

            self.current_path.push(key.clone());

            match value {
                JsonValue::Object(nested_obj) => {
                    // Check if this is a token (has $value or $type) or a group
                    let has_value = nested_obj.contains_key("$value");
                    let has_type = nested_obj.contains_key("$type");
                    let has_nested = nested_obj.iter().any(|(k, v)| !k.starts_with('$') && v.is_object());
                    
                    // Check for mixed token/group structure
                    if (has_value || has_type) && has_nested {
                        let span = self.get_current_span();
                        let error = CompilerError {
                            kind: ErrorKind::W3C(W3CErrorKind::InvalidStructure),
                            message: format!(
                                "Token '{}' has both token properties ($value/$type) and nested tokens. A token object cannot be both a token and a group.",
                                self.flatten_path()
                            ),
                            span,
                            suggestion: Some("Either remove the $value/$type fields to make this a group, or remove nested tokens to make this a token".to_string()),
                        };
                        errors.push(error);
                    } else if has_value || has_type {
                        // This is a token
                        match self.parse_token(nested_obj) {
                            Ok(token) => tokens.push(token),
                            Err(mut errs) => errors.append(&mut errs),
                        }
                    } else {
                        // This is a nested group
                        match self.parse_token_group(nested_obj) {
                            Ok(mut nested_tokens) => tokens.append(&mut nested_tokens),
                            Err(mut errs) => errors.append(&mut errs),
                        }
                    }
                }
                _ => {
                    let span = self.get_current_span();
                    let error = CompilerError {
                        kind: ErrorKind::W3C(W3CErrorKind::InvalidStructure),
                        message: format!("Invalid token structure at path '{}': expected an object", self.flatten_path()),
                        span,
                        suggestion: Some("Token definitions must be JSON objects with $value and optional $type fields".to_string()),
                    };
                    errors.push(error);
                }
            }

            self.current_path.pop();
        }

        if errors.is_empty() {
            Ok(tokens)
        } else {
            Err(errors)
        }
    }

    /// Parse a single token object.
    fn parse_token(&self, obj: &serde_json::Map<String, JsonValue>) -> Result<W3CToken, Vec<CompilerError>> {
        let path = self.flatten_path();
        let span = self.get_current_span();
        
        // Extract $value (required)
        let value = match obj.get("$value") {
            Some(v) => self.parse_token_value(v)?,
            None => {
                let error = CompilerError {
                    kind: ErrorKind::W3C(W3CErrorKind::MissingField),
                    message: format!("Token '{}' is missing required $value field", path),
                    span,
                    suggestion: Some("Add a $value field to define the token's value, e.g., \"$value\": \"#3b82f6\"".to_string()),
                };
                return Err(vec![error]);
            }
        };

        // Extract $type (optional)
        let token_type = if let Some(JsonValue::String(type_str)) = obj.get("$type") {
            match W3CTokenType::from_str(type_str) {
                Some(t) => Some(t),
                None => {
                    // Invalid $type value - report error with list of valid types
                    let valid_types = vec![
                        "color", "dimension", "fontFamily", "fontWeight", 
                        "duration", "cubicBezier", "number", "strokeStyle",
                        "border", "shadow", "typography"
                    ];
                    let error = CompilerError {
                        kind: ErrorKind::W3C(W3CErrorKind::InvalidType),
                        message: format!(
                            "Invalid $type value '{}' for token '{}'. Valid types are: {}",
                            type_str,
                            path,
                            valid_types.join(", ")
                        ),
                        span,
                        suggestion: Some(format!(
                            "Use one of the W3C Design Token types: {}",
                            valid_types.join(", ")
                        )),
                    };
                    return Err(vec![error]);
                }
            }
        } else {
            None
        };

        // Extract $description (optional)
        let description = if let Some(JsonValue::String(desc)) = obj.get("$description") {
            Some(desc.clone())
        } else {
            None
        };

        Ok(W3CToken {
            path,
            value,
            token_type,
            description,
        })
    }

    /// Parse a token value (can be string, number, object, or array).
    fn parse_token_value(&self, value: &JsonValue) -> Result<W3CTokenValue, Vec<CompilerError>> {
        match value {
            JsonValue::String(s) => {
                // Check if it's a reference
                if s.starts_with('{') && s.ends_with('}') {
                    let reference = s[1..s.len()-1].to_string();
                    Ok(W3CTokenValue::Reference(reference))
                } else {
                    Ok(W3CTokenValue::Literal(s.clone()))
                }
            }
            JsonValue::Number(n) => {
                Ok(W3CTokenValue::Literal(n.to_string()))
            }
            JsonValue::Object(obj) => {
                // Composite value
                let mut composite = HashMap::new();
                for (key, val) in obj {
                    let parsed_val = self.parse_token_value(val)?;
                    composite.insert(key.clone(), parsed_val);
                }
                Ok(W3CTokenValue::Composite(composite))
            }
            JsonValue::Array(arr) => {
                // Convert array to comma-separated string
                let items: Result<Vec<String>, Vec<CompilerError>> = arr.iter()
                    .map(|v| match v {
                        JsonValue::String(s) => Ok(s.clone()),
                        JsonValue::Number(n) => Ok(n.to_string()),
                        _ => {
                            let span = self.get_current_span();
                            let error = CompilerError {
                                kind: ErrorKind::W3C(W3CErrorKind::InvalidStructure),
                                message: "Array values must be strings or numbers".to_string(),
                                span,
                                suggestion: Some("Use only string or number values in arrays".to_string()),
                            };
                            Err(vec![error])
                        }
                    })
                    .collect();
                let items = items?;
                Ok(W3CTokenValue::Literal(items.join(", ")))
            }
            _ => {
                let span = self.get_current_span();
                let error = CompilerError {
                    kind: ErrorKind::W3C(W3CErrorKind::InvalidStructure),
                    message: format!("Invalid token value type at path '{}'", self.flatten_path()),
                    span,
                    suggestion: Some("Token values must be strings, numbers, objects, or arrays".to_string()),
                };
                Err(vec![error])
            }
        }
    }

    /// Flatten the current path to a dot-separated string.
    fn flatten_path(&self) -> String {
        self.current_path.join(".")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_token() {
        let json = r##"{
            "color": {
                "primary": {
                    "$value": "#3b82f6",
                    "$type": "color"
                }
            }
        }"##;

        let tokens = W3CTokenParser::parse(json).unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].path, "color.primary");
        assert_eq!(tokens[0].token_type, Some(W3CTokenType::Color));
    }

    #[test]
    fn test_parse_nested_tokens() {
        let json = r##"{
            "color": {
                "primary": {
                    "500": {
                        "$value": "#3b82f6",
                        "$type": "color"
                    }
                }
            }
        }"##;

        let tokens = W3CTokenParser::parse(json).unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].path, "color.primary.500");
    }

    #[test]
    fn test_parse_token_reference() {
        let json = r##"{
            "color": {
                "primary": {
                    "$value": "#3b82f6",
                    "$type": "color"
                },
                "secondary": {
                    "$value": "{color.primary}",
                    "$type": "color"
                }
            }
        }"##;

        let tokens = W3CTokenParser::parse(json).unwrap();
        assert_eq!(tokens.len(), 2);
        
        let secondary = tokens.iter().find(|t| t.path == "color.secondary").unwrap();
        match &secondary.value {
            W3CTokenValue::Reference(r) => assert_eq!(r, "color.primary"),
            _ => panic!("Expected reference value"),
        }
    }

    #[test]
    fn test_parse_missing_value() {
        let json = r##"{
            "color": {
                "primary": {
                    "$type": "color"
                }
            }
        }"##;

        let result = W3CTokenParser::parse(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_json() {
        let json = r##"{ invalid json }"##;
        let result = W3CTokenParser::parse(json);
        assert!(result.is_err());
    }
}
