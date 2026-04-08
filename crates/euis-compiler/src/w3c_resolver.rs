use std::collections::{HashMap, HashSet};
use crate::error::{CompilerError, ErrorKind, W3CErrorKind};
use crate::w3c_parser::{W3CToken, W3CTokenValue};
use crate::ast::Span;

/// Resolver for W3C Design Token references.
pub struct TokenReferenceResolver {
    tokens: HashMap<String, W3CToken>,
    original_tokens: HashMap<String, W3CToken>,
}

impl TokenReferenceResolver {
    /// Create a new resolver with the given tokens.
    pub fn new(tokens: Vec<W3CToken>) -> Self {
        let token_map: HashMap<String, W3CToken> = tokens
            .iter()
            .map(|token| (token.path.clone(), token.clone()))
            .collect();
        
        Self {
            tokens: token_map.clone(),
            original_tokens: token_map,
        }
    }

    /// Resolve all token references in the token collection.
    pub fn resolve_all(&mut self) -> Result<(), Vec<CompilerError>> {
        let mut errors = Vec::new();
        let paths: Vec<String> = self.tokens.keys().cloned().collect();

        for path in paths {
            let mut seen = HashSet::new();
            if let Err(e) = self.resolve_token_value(&path, &mut seen) {
                errors.push(e);
            }
        }

        // Validate type compatibility after resolution
        for (path, token) in &self.tokens {
            if let Err(e) = self.validate_type_compatibility(path, token) {
                errors.push(e);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Get the resolved tokens after resolution.
    pub fn get_resolved_tokens(&self) -> Vec<W3CToken> {
        self.tokens.values().cloned().collect()
    }

    /// Resolve a single token's value, following reference chains.
    fn resolve_token_value(&mut self, path: &str, seen: &mut HashSet<String>) -> Result<(), CompilerError> {
        // Check for circular reference
        if seen.contains(path) {
            let chain: Vec<String> = seen.iter().cloned().collect();
            return Err(CompilerError::circular_reference(&chain, Span::empty()));
        }

        seen.insert(path.to_string());

        // Get the token (we need to clone to avoid borrow checker issues)
        let token = match self.tokens.get(path) {
            Some(t) => t.clone(),
            None => return Ok(()), // Token doesn't exist, will be caught elsewhere
        };

        // Resolve the value
        let resolved_value = self.resolve_value(&token.value, seen)?;

        // Update the token with resolved value
        if let Some(token_mut) = self.tokens.get_mut(path) {
            token_mut.value = resolved_value;
        }

        seen.remove(path);
        Ok(())
    }

    /// Resolve a token value (handles references recursively).
    fn resolve_value(&self, value: &W3CTokenValue, seen: &mut HashSet<String>) -> Result<W3CTokenValue, CompilerError> {
        match value {
            W3CTokenValue::Literal(s) => {
                // Check if the literal contains embedded references
                let references = Self::extract_references(s);
                if references.is_empty() {
                    Ok(W3CTokenValue::Literal(s.clone()))
                } else {
                    // Resolve embedded references
                    let mut resolved_string = s.clone();
                    for ref_path in references {
                        let resolved_value = self.resolve_reference(&ref_path, seen)?;
                        // Extract the literal value from the resolved reference
                        let replacement = match resolved_value {
                            W3CTokenValue::Literal(lit) => lit,
                            _ => {
                                return Err(CompilerError {
                                    kind: ErrorKind::W3C(W3CErrorKind::InvalidStructure),
                                    message: format!("Cannot embed composite token '{}' in a string", ref_path),
                                    span: Span::empty(),
                                    suggestion: Some("Only literal values can be embedded in strings".to_string()),
                                });
                            }
                        };
                        // Replace {ref_path} with the resolved value
                        let pattern = format!("{{{}}}", ref_path);
                        resolved_string = resolved_string.replace(&pattern, &replacement);
                    }
                    Ok(W3CTokenValue::Literal(resolved_string))
                }
            }
            W3CTokenValue::Reference(ref_path) => {
                self.resolve_reference(ref_path, seen)
            }
            W3CTokenValue::Composite(map) => {
                let mut resolved_map = HashMap::new();
                for (key, val) in map {
                    let resolved = self.resolve_value(val, seen)?;
                    resolved_map.insert(key.clone(), resolved);
                }
                Ok(W3CTokenValue::Composite(resolved_map))
            }
        }
    }

    /// Resolve a reference to another token.
    fn resolve_reference(&self, ref_path: &str, seen: &mut HashSet<String>) -> Result<W3CTokenValue, CompilerError> {
        // Check for circular reference
        if seen.contains(ref_path) {
            let mut chain: Vec<String> = seen.iter().cloned().collect();
            chain.push(ref_path.to_string());
            return Err(CompilerError::circular_reference(&chain, Span::empty()));
        }

        // Look up the referenced token
        let referenced_token = match self.tokens.get(ref_path) {
            Some(t) => t,
            None => {
                let suggestion = self.find_similar_token(ref_path);
                return Err(CompilerError::token_not_found(ref_path, Span::empty(), suggestion));
            }
        };

        seen.insert(ref_path.to_string());

        // Recursively resolve the referenced token's value
        let resolved = self.resolve_value(&referenced_token.value, seen)?;

        seen.remove(ref_path);

        Ok(resolved)
    }

    /// Find a similar token name for suggestions using Levenshtein distance.
    fn find_similar_token(&self, target: &str) -> Option<String> {
        let candidates: Vec<&String> = self.tokens.keys().collect();
        
        candidates.into_iter()
            .min_by_key(|candidate| levenshtein_distance(target, candidate))
            .filter(|candidate| levenshtein_distance(target, candidate) <= 3)
            .map(|s| format!("Did you mean '{}'?", s))
    }

    /// Validate that resolved token types are compatible.
    /// This checks that if a token has a type and references another token,
    /// the referenced token's type is compatible.
    fn validate_type_compatibility(&self, _path: &str, token: &W3CToken) -> Result<(), CompilerError> {
        // If the token has no type, no validation needed
        if token.token_type.is_none() {
            return Ok(());
        }

        // Look up the original token value before resolution
        let original_token = self.original_tokens.get(&token.path).unwrap();
        
        // Check if the original value contains references
        self.validate_value_type_compatibility(&original_token.value, &token.token_type, &token.path)
    }

    /// Recursively validate type compatibility in a token value.
    fn validate_value_type_compatibility(
        &self,
        value: &W3CTokenValue,
        expected_type: &Option<crate::w3c_parser::W3CTokenType>,
        token_path: &str,
    ) -> Result<(), CompilerError> {
        match value {
            W3CTokenValue::Reference(ref_path) => {
                // Check if the referenced token exists and has a compatible type
                if let Some(referenced_token) = self.original_tokens.get(ref_path) {
                    if let (Some(expected), Some(actual)) = (expected_type, &referenced_token.token_type) {
                        if expected != actual {
                            return Err(CompilerError {
                                kind: ErrorKind::W3C(W3CErrorKind::TypeMismatch),
                                message: format!(
                                    "Type mismatch: token '{}' has type '{}' but references token '{}' with type '{}'",
                                    token_path,
                                    expected.as_str(),
                                    ref_path,
                                    actual.as_str()
                                ),
                                span: Span::empty(),
                                suggestion: Some(format!(
                                    "Ensure the referenced token '{}' has type '{}' or remove the type constraint",
                                    ref_path,
                                    expected.as_str()
                                )),
                            });
                        }
                    }
                }
            }
            W3CTokenValue::Composite(map) => {
                // Recursively validate composite values
                for val in map.values() {
                    self.validate_value_type_compatibility(val, &None, token_path)?;
                }
            }
            W3CTokenValue::Literal(s) => {
                // Check for embedded references in literals
                let references = Self::extract_references(s);
                for ref_path in references {
                    if let Some(referenced_token) = self.original_tokens.get(&ref_path) {
                        if let (Some(expected), Some(actual)) = (expected_type, &referenced_token.token_type) {
                            // For embedded references, we allow any type since they're being stringified
                            // But we could add stricter validation here if needed
                            let _ = (expected, actual); // Suppress unused warning
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Extract all references from a token value.
    pub fn extract_references(value: &str) -> Vec<String> {
        let mut references = Vec::new();
        let mut chars = value.chars().peekable();
        let mut current_ref = String::new();
        let mut in_reference = false;

        while let Some(ch) = chars.next() {
            if ch == '{' {
                in_reference = true;
                current_ref.clear();
            } else if ch == '}' && in_reference {
                in_reference = false;
                if !current_ref.is_empty() {
                    references.push(current_ref.clone());
                }
            } else if in_reference {
                current_ref.push(ch);
            }
        }

        references
    }
}

/// Calculate Levenshtein distance between two strings.
fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_len = a.len();
    let b_len = b.len();

    if a_len == 0 { return b_len; }
    if b_len == 0 { return a_len; }

    let mut prev_row: Vec<usize> = (0..=b_len).collect();
    let mut curr_row = vec![0usize; b_len + 1];

    for (i, a_ch) in a.chars().enumerate() {
        curr_row[0] = i + 1;
        for (j, b_ch) in b.chars().enumerate() {
            let cost = if a_ch == b_ch { 0 } else { 1 };
            curr_row[j + 1] = (prev_row[j + 1] + 1)
                .min(curr_row[j] + 1)
                .min(prev_row[j] + cost);
        }
        std::mem::swap(&mut prev_row, &mut curr_row);
    }

    prev_row[b_len]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::w3c_parser::{W3CToken, W3CTokenType};

    fn create_token(path: &str, value: W3CTokenValue) -> W3CToken {
        W3CToken {
            path: path.to_string(),
            value,
            token_type: Some(W3CTokenType::Color),
            description: None,
        }
    }

    #[test]
    fn test_resolve_literal() {
        let tokens = vec![
            create_token("color.primary", W3CTokenValue::Literal("#3b82f6".to_string())),
        ];

        let mut resolver = TokenReferenceResolver::new(tokens);
        assert!(resolver.resolve_all().is_ok());
    }

    #[test]
    fn test_resolve_simple_reference() {
        let tokens = vec![
            create_token("color.primary", W3CTokenValue::Literal("#3b82f6".to_string())),
            create_token("color.secondary", W3CTokenValue::Reference("color.primary".to_string())),
        ];

        let mut resolver = TokenReferenceResolver::new(tokens);
        assert!(resolver.resolve_all().is_ok());

        let secondary = resolver.tokens.get("color.secondary").unwrap();
        match &secondary.value {
            W3CTokenValue::Literal(s) => assert_eq!(s, "#3b82f6"),
            _ => panic!("Expected literal value after resolution"),
        }
    }

    #[test]
    fn test_resolve_reference_chain() {
        let tokens = vec![
            create_token("color.base", W3CTokenValue::Literal("#3b82f6".to_string())),
            create_token("color.primary", W3CTokenValue::Reference("color.base".to_string())),
            create_token("color.secondary", W3CTokenValue::Reference("color.primary".to_string())),
        ];

        let mut resolver = TokenReferenceResolver::new(tokens);
        assert!(resolver.resolve_all().is_ok());

        let secondary = resolver.tokens.get("color.secondary").unwrap();
        match &secondary.value {
            W3CTokenValue::Literal(s) => assert_eq!(s, "#3b82f6"),
            _ => panic!("Expected literal value after resolution"),
        }
    }

    #[test]
    fn test_detect_circular_reference() {
        let tokens = vec![
            create_token("color.a", W3CTokenValue::Reference("color.b".to_string())),
            create_token("color.b", W3CTokenValue::Reference("color.a".to_string())),
        ];

        let mut resolver = TokenReferenceResolver::new(tokens);
        let result = resolver.resolve_all();
        assert!(result.is_err());
    }

    #[test]
    fn test_reference_not_found() {
        let tokens = vec![
            create_token("color.primary", W3CTokenValue::Reference("color.nonexistent".to_string())),
        ];

        let mut resolver = TokenReferenceResolver::new(tokens);
        let result = resolver.resolve_all();
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_references() {
        let refs = TokenReferenceResolver::extract_references("{color.primary}");
        assert_eq!(refs, vec!["color.primary"]);

        let refs = TokenReferenceResolver::extract_references("solid 1px {color.border}");
        assert_eq!(refs, vec!["color.border"]);
    }

    #[test]
    fn test_resolve_embedded_reference() {
        let tokens = vec![
            create_token("color.border", W3CTokenValue::Literal("#cccccc".to_string())),
            create_token("border.default", W3CTokenValue::Literal("solid 1px {color.border}".to_string())),
        ];

        let mut resolver = TokenReferenceResolver::new(tokens);
        assert!(resolver.resolve_all().is_ok());

        let border = resolver.tokens.get("border.default").unwrap();
        match &border.value {
            W3CTokenValue::Literal(s) => assert_eq!(s, "solid 1px #cccccc"),
            _ => panic!("Expected literal value after resolution"),
        }
    }

    #[test]
    fn test_resolve_multiple_embedded_references() {
        let tokens = vec![
            create_token("color.primary", W3CTokenValue::Literal("#3b82f6".to_string())),
            create_token("color.secondary", W3CTokenValue::Literal("#10b981".to_string())),
            create_token("gradient.default", W3CTokenValue::Literal("linear-gradient({color.primary}, {color.secondary})".to_string())),
        ];

        let mut resolver = TokenReferenceResolver::new(tokens);
        assert!(resolver.resolve_all().is_ok());

        let gradient = resolver.tokens.get("gradient.default").unwrap();
        match &gradient.value {
            W3CTokenValue::Literal(s) => assert_eq!(s, "linear-gradient(#3b82f6, #10b981)"),
            _ => panic!("Expected literal value after resolution"),
        }
    }

    #[test]
    fn test_type_compatibility_validation() {
        // Create tokens with incompatible types
        let mut color_token = create_token("color.primary", W3CTokenValue::Literal("#3b82f6".to_string()));
        color_token.token_type = Some(W3CTokenType::Color);
        
        let mut dimension_token = create_token("spacing.base", W3CTokenValue::Literal("8px".to_string()));
        dimension_token.token_type = Some(W3CTokenType::Dimension);
        
        let mut invalid_ref_token = create_token("color.secondary", W3CTokenValue::Reference("spacing.base".to_string()));
        invalid_ref_token.token_type = Some(W3CTokenType::Color);
        
        let tokens = vec![color_token, dimension_token, invalid_ref_token];

        let mut resolver = TokenReferenceResolver::new(tokens);
        let result = resolver.resolve_all();
        
        // Should fail due to type mismatch
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e.kind, ErrorKind::W3C(W3CErrorKind::TypeMismatch))));
    }

    #[test]
    fn test_type_compatibility_same_type() {
        // Create tokens with compatible types
        let mut color1 = create_token("color.primary", W3CTokenValue::Literal("#3b82f6".to_string()));
        color1.token_type = Some(W3CTokenType::Color);
        
        let mut color2 = create_token("color.secondary", W3CTokenValue::Reference("color.primary".to_string()));
        color2.token_type = Some(W3CTokenType::Color);
        
        let tokens = vec![color1, color2];

        let mut resolver = TokenReferenceResolver::new(tokens);
        let result = resolver.resolve_all();
        
        // Should succeed - same types
        assert!(result.is_ok());
    }

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(levenshtein_distance("primary", "primary"), 0);
        assert_eq!(levenshtein_distance("primary", "primay"), 1);
        assert_eq!(levenshtein_distance("primary", "secondary"), 6);
    }
}
