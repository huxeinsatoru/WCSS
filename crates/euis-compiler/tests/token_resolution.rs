// Feature: euis-framework, Property 10: Design Token Resolution
// Validates: Requirements 6.2, 6.3

use proptest::prelude::*;
use std::collections::{HashMap, HashSet};
use euis_compiler::ast::{Span, TokenCategory, TokenRef};
use euis_compiler::config::{DesignTokens, TokenValue};
use euis_compiler::tokens::resolve_token;

/// Generate a random token category.
fn token_category_strategy() -> impl Strategy<Value = TokenCategory> {
    prop_oneof![
        Just(TokenCategory::Colors),
        Just(TokenCategory::Spacing),
        Just(TokenCategory::Typography),
        Just(TokenCategory::Breakpoints),
    ]
}

/// Generate a valid token name (alphanumeric with hyphens).
fn token_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9-]{0,15}".prop_map(|s| s.to_string())
}

/// Generate a literal token value (color, spacing, etc.).
fn literal_value_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        // Colors
        "#[0-9a-f]{6}".prop_map(|s| s.to_string()),
        "rgb\\([0-9]{1,3}, [0-9]{1,3}, [0-9]{1,3}\\)".prop_map(|s| s.to_string()),
        // Spacing
        "[0-9]{1,2}(px|rem|em)".prop_map(|s| s.to_string()),
        // Typography
        "[0-9]{1,2}px".prop_map(|s| s.to_string()),
        "(sans-serif|serif|monospace)".prop_map(|s| s.to_string()),
        // Breakpoints
        "[0-9]{3,4}px".prop_map(|s| s.to_string()),
    ]
}

/// Generate a design token configuration with random tokens.
/// Returns (tokens, defined_names) where defined_names is a map of category -> set of names.
fn design_tokens_strategy() -> impl Strategy<Value = (DesignTokens, HashMap<TokenCategory, HashSet<String>>)> {
    // Generate 5-20 tokens per category
    let tokens_per_category = 5..=20usize;
    
    (
        prop::collection::hash_map(token_name_strategy(), literal_value_strategy(), tokens_per_category.clone()),
        prop::collection::hash_map(token_name_strategy(), literal_value_strategy(), tokens_per_category.clone()),
        prop::collection::hash_map(token_name_strategy(), literal_value_strategy(), tokens_per_category.clone()),
        prop::collection::hash_map(token_name_strategy(), literal_value_strategy(), tokens_per_category),
    ).prop_map(|(colors, spacing, typography, breakpoints)| {
        let mut defined_names = HashMap::new();
        defined_names.insert(TokenCategory::Colors, colors.keys().cloned().collect());
        defined_names.insert(TokenCategory::Spacing, spacing.keys().cloned().collect());
        defined_names.insert(TokenCategory::Typography, typography.keys().cloned().collect());
        defined_names.insert(TokenCategory::Breakpoints, breakpoints.keys().cloned().collect());
        
        let tokens = DesignTokens {
            colors: colors.into_iter().map(|(k, v)| (k, TokenValue::Literal(v))).collect(),
            spacing: spacing.into_iter().map(|(k, v)| (k, TokenValue::Literal(v))).collect(),
            typography: typography.into_iter().map(|(k, v)| (k, TokenValue::Literal(v))).collect(),
            breakpoints: breakpoints.into_iter().map(|(k, v)| (k, TokenValue::Literal(v))).collect(),
            shadows: HashMap::new(),
            borders: HashMap::new(),
            radii: HashMap::new(),
            zindex: HashMap::new(),
            opacity: HashMap::new(),
        };
        
        (tokens, defined_names)
    })
}

/// Generate a token reference that is guaranteed to be defined in the given tokens.
fn defined_token_ref_strategy(
    defined_names: HashMap<TokenCategory, HashSet<String>>
) -> impl Strategy<Value = (TokenRef, String)> {
    // Pick a random category that has tokens
    let categories: Vec<_> = defined_names.keys().cloned().collect();
    
    prop::sample::select(categories).prop_flat_map(move |category| {
        let names: Vec<_> = defined_names.get(&category).unwrap().iter().cloned().collect();
        prop::sample::select(names).prop_map(move |name| {
            let token_ref = TokenRef {
                category: category.clone(),
                name: name.clone(),
                span: Span::empty(),
            };
            (token_ref, name)
        })
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    /// Property: Defined tokens resolve correctly
    /// For any design token that is defined in the configuration,
    /// resolving it should return the correct literal value.
    #[test]
    fn prop_defined_tokens_resolve_correctly(
        data in design_tokens_strategy()
            .prop_flat_map(|(t, d)| {
                let t_clone = t.clone();
                defined_token_ref_strategy(d).prop_map(move |(r, _)| (t_clone.clone(), r))
            })
    ) {
        let (tokens, token_ref) = data;
        let result = resolve_token(&token_ref, &tokens, &mut HashSet::new());
        
        // Should resolve successfully
        prop_assert!(result.is_ok(), "Token {:?}.{} should resolve successfully", 
            token_ref.category, token_ref.name);
        
        // Should return a non-empty value
        let resolved_value = result.unwrap();
        prop_assert!(!resolved_value.is_empty(), "Resolved value should not be empty");
        
        // Should match the expected value from the configuration
        let expected = tokens.get(&token_ref.category, &token_ref.name)
            .and_then(|tv| tv.as_literal());
        prop_assert_eq!(Some(resolved_value.as_str()), expected,
            "Resolved value should match configuration");
    }
    
    /// Property: Undefined tokens produce errors
    /// For any design token that is NOT defined in the configuration,
    /// attempting to resolve it should return an error.
    #[test]
    fn prop_undefined_tokens_produce_errors(
        (tokens, defined_names) in design_tokens_strategy(),
        category in token_category_strategy(),
        name in "[a-z][a-z0-9-]{0,15}".prop_map(|s| s.to_string())
    ) {
        // Check if this token is defined
        let is_defined = defined_names
            .get(&category)
            .map(|names| names.contains(&name))
            .unwrap_or(false);
        
        // Skip this test case if the token is defined
        prop_assume!(!is_defined);
        
        let token_ref = TokenRef {
            category,
            name: name.clone(),
            span: Span::empty(),
        };
        
        let result = resolve_token(&token_ref, &tokens, &mut HashSet::new());
        
        // Should fail with an error
        prop_assert!(result.is_err(), 
            "Undefined token {:?}.{} should produce an error", 
            token_ref.category, token_ref.name);
    }
    
    /// Property: Token resolution is deterministic
    /// Resolving the same token multiple times should always return the same result.
    #[test]
    fn prop_token_resolution_is_deterministic(
        data in design_tokens_strategy()
            .prop_flat_map(|(t, d)| {
                let t_clone = t.clone();
                defined_token_ref_strategy(d).prop_map(move |(r, _)| (t_clone.clone(), r))
            })
    ) {
        let (tokens, token_ref) = data;
        let result1 = resolve_token(&token_ref, &tokens, &mut HashSet::new());
        let result2 = resolve_token(&token_ref, &tokens, &mut HashSet::new());
        
        prop_assert_eq!(result1.is_ok(), result2.is_ok(), 
            "Resolution should be deterministic (same success/failure)");
        
        if let (Ok(val1), Ok(val2)) = (result1, result2) {
            prop_assert_eq!(val1, val2, "Resolved values should be identical");
        }
    }
    
    /// Property: All defined tokens can be resolved
    /// Every token that exists in the configuration should be resolvable.
    #[test]
    fn prop_all_defined_tokens_resolvable(
        (tokens, _defined_names) in design_tokens_strategy()
    ) {
        // Test all colors
        for (name, value) in &tokens.colors {
            if let TokenValue::Literal(expected) = value {
                let token_ref = TokenRef {
                    category: TokenCategory::Colors,
                    name: name.clone(),
                    span: Span::empty(),
                };
                let result = resolve_token(&token_ref, &tokens, &mut HashSet::new());
                prop_assert!(result.is_ok(), "Color token '{}' should resolve", name);
                prop_assert_eq!(&result.unwrap(), expected);
            }
        }
        
        // Test all spacing
        for (name, value) in &tokens.spacing {
            if let TokenValue::Literal(expected) = value {
                let token_ref = TokenRef {
                    category: TokenCategory::Spacing,
                    name: name.clone(),
                    span: Span::empty(),
                };
                let result = resolve_token(&token_ref, &tokens, &mut HashSet::new());
                prop_assert!(result.is_ok(), "Spacing token '{}' should resolve", name);
                prop_assert_eq!(&result.unwrap(), expected);
            }
        }
        
        // Test all typography
        for (name, value) in &tokens.typography {
            if let TokenValue::Literal(expected) = value {
                let token_ref = TokenRef {
                    category: TokenCategory::Typography,
                    name: name.clone(),
                    span: Span::empty(),
                };
                let result = resolve_token(&token_ref, &tokens, &mut HashSet::new());
                prop_assert!(result.is_ok(), "Typography token '{}' should resolve", name);
                prop_assert_eq!(&result.unwrap(), expected);
            }
        }
        
        // Test all breakpoints
        for (name, value) in &tokens.breakpoints {
            if let TokenValue::Literal(expected) = value {
                let token_ref = TokenRef {
                    category: TokenCategory::Breakpoints,
                    name: name.clone(),
                    span: Span::empty(),
                };
                let result = resolve_token(&token_ref, &tokens, &mut HashSet::new());
                prop_assert!(result.is_ok(), "Breakpoint token '{}' should resolve", name);
                prop_assert_eq!(&result.unwrap(), expected);
            }
        }
    }
}
