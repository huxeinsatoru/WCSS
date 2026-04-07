use std::collections::HashSet;

use crate::ast::*;
use crate::config::{DesignTokens, TokenValue};
use crate::error::CompilerError;

/// Resolve all token references in a stylesheet, replacing them with literal values.
/// Takes ownership to avoid cloning.
pub fn resolve(mut stylesheet: StyleSheet, tokens: &DesignTokens) -> StyleSheet {
    for rule in &mut stylesheet.rules {
        resolve_declarations(&mut rule.declarations, tokens);
        for state in &mut rule.states {
            resolve_declarations(&mut state.declarations, tokens);
        }
        for responsive in &mut rule.responsive {
            resolve_declarations(&mut responsive.declarations, tokens);
        }
    }
    stylesheet
}

fn resolve_declarations(declarations: &mut [Declaration], tokens: &DesignTokens) {
    for decl in declarations.iter_mut() {
        resolve_value_in_place(&mut decl.value, tokens);
    }
}

fn resolve_value_in_place(value: &mut Value, tokens: &DesignTokens) {
    match value {
        Value::Token(token_ref) => {
            match resolve_token(token_ref, tokens, &mut HashSet::new()) {
                Ok(resolved) => *value = Value::Literal(resolved),
                Err(_) => {} // Keep unresolved (validator will catch it)
            }
        }
        Value::List(values) => {
            for v in values.iter_mut() {
                resolve_value_in_place(v, tokens);
            }
        }
        Value::Var(_, Some(fallback)) | Value::Env(_, Some(fallback)) => {
            resolve_value_in_place(fallback, tokens);
        }
        _ => {} // No-op for non-token values
    }
}

/// Resolve a single token reference, following reference chains.
/// Detects circular references.
pub fn resolve_token(
    token_ref: &TokenRef,
    tokens: &DesignTokens,
    seen: &mut HashSet<String>,
) -> Result<String, CompilerError> {
    let key = format!("{}.{}", token_ref.category.as_str(), token_ref.name);

    if seen.contains(&key) {
        let chain: Vec<String> = seen.iter().cloned().collect();
        return Err(CompilerError::circular_reference(&chain, token_ref.span.clone()));
    }
    seen.insert(key);

    match tokens.get(&token_ref.category, &token_ref.name) {
        Some(TokenValue::Literal(value)) => Ok(value.clone()),
        Some(TokenValue::Reference(ref_str)) => {
            let ref_str = ref_str.trim_start_matches('$');
            if let Some((cat, name)) = ref_str.split_once('.') {
                if let Some(category) = TokenCategory::from_str(cat) {
                    let nested_ref = TokenRef {
                        category,
                        name: name.to_string(),
                        span: token_ref.span.clone(),
                    };
                    resolve_token(&nested_ref, tokens, seen)
                } else {
                    Err(CompilerError::token_not_found(ref_str, token_ref.span.clone(), None))
                }
            } else {
                Err(CompilerError::token_not_found(ref_str, token_ref.span.clone(), None))
            }
        }
        None => {
            let suggestion = find_similar_token(tokens, &token_ref.category, &token_ref.name);
            Err(CompilerError::token_not_found(&token_ref.name, token_ref.span.clone(), suggestion))
        }
    }
}

/// Find a similar token name for suggestions.
fn find_similar_token(tokens: &DesignTokens, category: &TokenCategory, name: &str) -> Option<String> {
    let candidates: Vec<&String> = match category {
        TokenCategory::Colors => tokens.colors.keys().collect(),
        TokenCategory::Spacing => tokens.spacing.keys().collect(),
        TokenCategory::Typography => tokens.typography.keys().collect(),
        TokenCategory::Breakpoints => tokens.breakpoints.keys().collect(),
        TokenCategory::Animation => Vec::new(),
        TokenCategory::Shadows => tokens.shadows.keys().collect(),
        TokenCategory::Borders => tokens.borders.keys().collect(),
        TokenCategory::Radii => tokens.radii.keys().collect(),
        TokenCategory::ZIndex => tokens.zindex.keys().collect(),
        TokenCategory::Opacity => tokens.opacity.keys().collect(),
        TokenCategory::Custom => tokens.spacing.keys().collect(),
    };

    candidates.into_iter()
        .min_by_key(|candidate| levenshtein_distance(name, candidate))
        .filter(|candidate| levenshtein_distance(name, candidate) <= 3)
        .map(|s| format!("Did you mean '{s}'?"))
}

fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_len = a.len();
    let b_len = b.len();

    // Early exit for obvious cases
    if a_len == 0 { return b_len; }
    if b_len == 0 { return a_len; }
    let diff = if a_len > b_len { a_len - b_len } else { b_len - a_len };
    if diff > 3 { return diff; }

    // Use single-row optimization instead of full matrix
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
    use crate::config::TokenValue;
    use std::collections::HashMap;

    fn test_tokens() -> DesignTokens {
        let mut colors = HashMap::new();
        colors.insert("primary".to_string(), TokenValue::Literal("#3b82f6".to_string()));
        colors.insert("secondary".to_string(), TokenValue::Literal("#8b5cf6".to_string()));
        colors.insert("primary-dark".to_string(), TokenValue::Reference("$colors.primary".to_string()));

        let mut spacing = HashMap::new();
        spacing.insert("md".to_string(), TokenValue::Literal("1rem".to_string()));

        DesignTokens {
            colors,
            spacing,
            ..Default::default()
        }
    }

    #[test]
    fn test_resolve_literal_token() {
        let tokens = test_tokens();
        let token_ref = TokenRef {
            category: TokenCategory::Colors,
            name: "primary".to_string(),
            span: Span::empty(),
        };
        let result = resolve_token(&token_ref, &tokens, &mut HashSet::new());
        assert_eq!(result.unwrap(), "#3b82f6");
    }

    #[test]
    fn test_resolve_reference_chain() {
        let tokens = test_tokens();
        let token_ref = TokenRef {
            category: TokenCategory::Colors,
            name: "primary-dark".to_string(),
            span: Span::empty(),
        };
        let result = resolve_token(&token_ref, &tokens, &mut HashSet::new());
        assert_eq!(result.unwrap(), "#3b82f6");
    }

    #[test]
    fn test_resolve_undefined_token() {
        let tokens = test_tokens();
        let token_ref = TokenRef {
            category: TokenCategory::Colors,
            name: "nonexistent".to_string(),
            span: Span::empty(),
        };
        let result = resolve_token(&token_ref, &tokens, &mut HashSet::new());
        assert!(result.is_err());
    }

    #[test]
    fn test_circular_reference_detection() {
        let mut colors = HashMap::new();
        colors.insert("a".to_string(), TokenValue::Reference("$colors.b".to_string()));
        colors.insert("b".to_string(), TokenValue::Reference("$colors.a".to_string()));
        let tokens = DesignTokens { colors, ..Default::default() };

        let token_ref = TokenRef {
            category: TokenCategory::Colors,
            name: "a".to_string(),
            span: Span::empty(),
        };
        let result = resolve_token(&token_ref, &tokens, &mut HashSet::new());
        assert!(result.is_err());
    }

    #[test]
    fn test_similar_token_suggestion() {
        let tokens = test_tokens();
        let suggestion = find_similar_token(&tokens, &TokenCategory::Colors, "primay");
        assert!(suggestion.is_some());
        assert!(suggestion.unwrap().contains("primary"));
    }
}
