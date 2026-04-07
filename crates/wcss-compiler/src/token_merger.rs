use crate::error::{CompilerError, ErrorKind, W3CErrorKind};
use crate::config::{DesignTokens, TokenValue};
use crate::ast::TokenCategory;
use crate::w3c_parser::{W3CToken, W3CTokenType, W3CTokenValue};
use std::collections::HashMap;

/// Merger for combining W3C Design Tokens with existing WCSS tokens.
pub struct TokenMerger;

/// Result of a token merge operation.
#[derive(Debug, Clone)]
pub struct MergedTokens {
    /// The merged design tokens
    pub tokens: DesignTokens,
    /// Any warnings generated during merge
    pub warnings: Vec<String>,
}

impl TokenMerger {
    /// Merge W3C tokens with existing WCSS tokens.
    ///
    /// # Arguments
    /// * `w3c_tokens` - Tokens parsed from W3C Design Tokens JSON
    /// * `wcss_tokens` - Existing WCSS tokens from the config
    ///
    /// # Returns
    /// Result containing merged tokens or a vector of errors
    pub fn merge(
        w3c_tokens: Vec<W3CToken>,
        wcss_tokens: &DesignTokens,
    ) -> Result<MergedTokens, Vec<CompilerError>> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Check for conflicts first
        let conflicts = Self::detect_conflicts(&w3c_tokens, wcss_tokens);
        if !conflicts.is_empty() {
            errors.extend(conflicts);
            return Err(errors);
        }

        // Convert W3C tokens to WCSS format
        let mut merged = DesignTokens {
            colors: wcss_tokens.colors.clone(),
            spacing: wcss_tokens.spacing.clone(),
            typography: wcss_tokens.typography.clone(),
            breakpoints: wcss_tokens.breakpoints.clone(),
            shadows: wcss_tokens.shadows.clone(),
            borders: wcss_tokens.borders.clone(),
            radii: wcss_tokens.radii.clone(),
            zindex: wcss_tokens.zindex.clone(),
            opacity: wcss_tokens.opacity.clone(),
        };

        // Add W3C tokens to the appropriate categories
        for token in &w3c_tokens {
            match Self::convert_w3c_to_wcss(token) {
                Some((category, name, value)) => {
                    if let Err(e) = Self::add_token_to_category(&mut merged, category, name, value) {
                        warnings.push(format!(
                            "Failed to add token '{}': {}",
                            token.path, e
                        ));
                    }
                }
                None => {
                    warnings.push(format!(
                        "Token '{}' could not be converted to WCSS format",
                        token.path
                    ));
                }
            }
        }

        Ok(MergedTokens {
            tokens: merged,
            warnings,
        })
    }

    /// Detect naming conflicts between W3C and WCSS tokens.
    fn detect_conflicts(
        w3c_tokens: &[W3CToken],
        wcss_tokens: &DesignTokens,
    ) -> Vec<CompilerError> {
        let mut conflicts = Vec::new();

        for token in w3c_tokens {
            let token_name = token.path.replace('.', "-");

            // Check each WCSS category for conflicts
            if wcss_tokens.colors.contains_key(&token_name) {
                conflicts.push(CompilerError {
                    kind: ErrorKind::W3C(W3CErrorKind::ConflictingToken),
                    message: format!(
                        "Token name conflict: '{}' exists in both W3C tokens and WCSS colors",
                        token_name
                    ),
                    span: crate::ast::Span::empty(),
                    suggestion: Some(format!(
                        "Rename either the W3C token '{}' or the WCSS color '{}'",
                        token.path, token_name
                    )),
                });
            }

            if wcss_tokens.spacing.contains_key(&token_name) {
                conflicts.push(CompilerError {
                    kind: ErrorKind::W3C(W3CErrorKind::ConflictingToken),
                    message: format!(
                        "Token name conflict: '{}' exists in both W3C tokens and WCSS spacing",
                        token_name
                    ),
                    span: crate::ast::Span::empty(),
                    suggestion: Some(format!(
                        "Rename either the W3C token '{}' or the WCSS spacing '{}'",
                        token.path, token_name
                    )),
                });
            }

            if wcss_tokens.typography.contains_key(&token_name) {
                conflicts.push(CompilerError {
                    kind: ErrorKind::W3C(W3CErrorKind::ConflictingToken),
                    message: format!(
                        "Token name conflict: '{}' exists in both W3C tokens and WCSS typography",
                        token_name
                    ),
                    span: crate::ast::Span::empty(),
                    suggestion: Some(format!(
                        "Rename either the W3C token '{}' or the WCSS typography '{}'",
                        token.path, token_name
                    )),
                });
            }

            if wcss_tokens.breakpoints.contains_key(&token_name) {
                conflicts.push(CompilerError {
                    kind: ErrorKind::W3C(W3CErrorKind::ConflictingToken),
                    message: format!(
                        "Token name conflict: '{}' exists in both W3C tokens and WCSS breakpoints",
                        token_name
                    ),
                    span: crate::ast::Span::empty(),
                    suggestion: Some(format!(
                        "Rename either the W3C token '{}' or the WCSS breakpoint '{}'",
                        token.path, token_name
                    )),
                });
            }
        }

        conflicts
    }

    /// Convert a W3C token to WCSS format.
    fn convert_w3c_to_wcss(
        token: &W3CToken,
    ) -> Option<(TokenCategory, String, TokenValue)> {
        let name = token.path.replace('.', "-");
        let value = Self::extract_w3c_value(&token.value);

        // Map W3C types to WCSS categories
        let category = match &token.token_type {
            Some(W3CTokenType::Color) => TokenCategory::Colors,
            Some(W3CTokenType::Dimension) => TokenCategory::Spacing,
            Some(W3CTokenType::FontFamily) => TokenCategory::Typography,
            Some(W3CTokenType::FontWeight) => TokenCategory::Typography,
            Some(W3CTokenType::Duration) => TokenCategory::Animation,
            Some(W3CTokenType::Number) => TokenCategory::Spacing,
            _ => TokenCategory::Custom,
        };

        Some((category, name, TokenValue::Literal(value)))
    }

    /// Extract the string value from a W3C token value.
    fn extract_w3c_value(value: &W3CTokenValue) -> String {
        match value {
            W3CTokenValue::Literal(s) => s.clone(),
            W3CTokenValue::Reference(r) => format!("var(--{})", r.replace('.', "-")),
            W3CTokenValue::Composite(map) => {
                // Serialize composite values as JSON-like string
                let parts: Vec<String> = map
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, Self::extract_w3c_value(v)))
                    .collect();
                format!("{{{}}}", parts.join(", "))
            }
        }
    }

    /// Add a token to the appropriate category in the merged tokens.
    fn add_token_to_category(
        tokens: &mut DesignTokens,
        category: TokenCategory,
        name: String,
        value: TokenValue,
    ) -> Result<(), String> {
        match category {
            TokenCategory::Colors => {
                tokens.colors.insert(name, value);
            }
            TokenCategory::Spacing => {
                tokens.spacing.insert(name, value);
            }
            TokenCategory::Typography => {
                tokens.typography.insert(name, value);
            }
            TokenCategory::Breakpoints => {
                tokens.breakpoints.insert(name, value);
            }
            _ => {
                // Custom tokens go into spacing for now
                tokens.spacing.insert(name, value);
            }
        }
        Ok(())
    }

    /// Convert W3C tokens to a flat HashMap for easier lookup.
    pub fn w3c_to_hashmap(tokens: &[W3CToken]) -> HashMap<String, String> {
        let mut map = HashMap::new();

        for token in tokens {
            let value = Self::extract_w3c_value(&token.value);
            map.insert(token.path.clone(), value);
        }

        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_w3c_token(path: &str, value: &str, token_type: W3CTokenType) -> W3CToken {
        W3CToken {
            path: path.to_string(),
            value: W3CTokenValue::Literal(value.to_string()),
            token_type: Some(token_type),
            description: None,
        }
    }

    fn create_wcss_tokens() -> DesignTokens {
        DesignTokens {
            colors: [(
                "primary".to_string(),
                TokenValue::Literal("#3b82f6".to_string()),
            )]
            .into_iter()
            .collect(),
            spacing: [("base".to_string(), TokenValue::Literal("16px".to_string()))]
                .into_iter()
                .collect(),
            typography: HashMap::new(),
            breakpoints: HashMap::new(),
            shadows: HashMap::new(),
            borders: HashMap::new(),
            radii: HashMap::new(),
            zindex: HashMap::new(),
            opacity: HashMap::new(),
        }
    }

    #[test]
    fn test_merge_w3c_with_wcss() {
        let w3c_tokens = vec![
            create_w3c_token("color.secondary", "#64748b", W3CTokenType::Color),
            create_w3c_token("spacing.large", "24px", W3CTokenType::Dimension),
        ];

        let wcss_tokens = create_wcss_tokens();
        let result = TokenMerger::merge(w3c_tokens, &wcss_tokens);

        assert!(result.is_ok());
        let merged = result.unwrap();

        // Should have both WCSS and W3C tokens
        assert!(merged.tokens.colors.contains_key("primary")); // From WCSS
        assert!(merged.tokens.colors.contains_key("color-secondary")); // From W3C
        assert!(merged.tokens.spacing.contains_key("base")); // From WCSS
        assert!(merged.tokens.spacing.contains_key("spacing-large")); // From W3C
    }

    #[test]
    fn test_detect_conflicts() {
        // Create W3C token with same name as WCSS token after conversion
        // WCSS has "primary" in colors, so we need a W3C token that converts to "primary"
        // A W3C token with path "primary" (no dots) becomes "primary" after conversion
        let w3c_tokens = vec![create_w3c_token("primary", "#ff0000", W3CTokenType::Color)];

        let wcss_tokens = create_wcss_tokens();
        let result = TokenMerger::merge(w3c_tokens, &wcss_tokens);

        assert!(result.is_err());
    }

    #[test]
    fn test_convert_w3c_to_wcss() {
        let color_token = create_w3c_token("color.primary", "#3b82f6", W3CTokenType::Color);
        let result = TokenMerger::convert_w3c_to_wcss(&color_token);

        assert!(result.is_some());
        let (category, name, value) = result.unwrap();
        assert!(matches!(category, TokenCategory::Colors));
        assert_eq!(name, "color-primary");
        assert_eq!(value, TokenValue::Literal("#3b82f6".to_string()));
    }

    #[test]
    fn test_extract_w3c_value_literal() {
        let value = W3CTokenValue::Literal("#3b82f6".to_string());
        assert_eq!(TokenMerger::extract_w3c_value(&value), "#3b82f6");
    }

    #[test]
    fn test_extract_w3c_value_reference() {
        let value = W3CTokenValue::Reference("color.primary".to_string());
        assert_eq!(
            TokenMerger::extract_w3c_value(&value),
            "var(--color-primary)"
        );
    }

    #[test]
    fn test_w3c_to_hashmap() {
        let tokens = vec![
            create_w3c_token("color.primary", "#3b82f6", W3CTokenType::Color),
            create_w3c_token("spacing.base", "16px", W3CTokenType::Dimension),
        ];

        let map = TokenMerger::w3c_to_hashmap(&tokens);

        assert_eq!(map.get("color.primary"), Some(&"#3b82f6".to_string()));
        assert_eq!(map.get("spacing.base"), Some(&"16px".to_string()));
    }

    #[test]
    fn test_empty_w3c_tokens() {
        let w3c_tokens: Vec<W3CToken> = vec![];
        let wcss_tokens = create_wcss_tokens();

        let result = TokenMerger::merge(w3c_tokens, &wcss_tokens);

        assert!(result.is_ok());
        let merged = result.unwrap();

        // Should only have WCSS tokens
        assert!(merged.tokens.colors.contains_key("primary"));
        assert!(merged.tokens.spacing.contains_key("base"));
    }

    #[test]
    fn test_dimension_token_mapping() {
        let token = create_w3c_token("spacing.large", "24px", W3CTokenType::Dimension);
        let result = TokenMerger::convert_w3c_to_wcss(&token);

        assert!(result.is_some());
        let (category, _, _) = result.unwrap();
        assert!(matches!(category, TokenCategory::Spacing));
    }

    #[test]
    fn test_typography_token_mapping() {
        let token = create_w3c_token("font.family.base", "Inter", W3CTokenType::FontFamily);
        let result = TokenMerger::convert_w3c_to_wcss(&token);

        assert!(result.is_some());
        let (category, _, _) = result.unwrap();
        assert!(matches!(category, TokenCategory::Typography));
    }
}
