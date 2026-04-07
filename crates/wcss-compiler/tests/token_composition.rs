// Feature: wcss-framework, Property 11: Token Composition Correctness
// Validates: Requirements 6.5

use proptest::prelude::*;
use std::collections::{HashMap, HashSet};
use wcss_compiler::ast::{Span, TokenCategory, TokenRef};
use wcss_compiler::config::{DesignTokens, TokenValue};
use wcss_compiler::tokens::resolve_token;

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

/// Generate a token chain: a sequence of tokens where each references the next.
/// Returns (tokens, chain_start_ref, expected_value)
/// Example: a -> b -> c -> "literal_value"
fn token_chain_strategy() -> impl Strategy<Value = (DesignTokens, TokenRef, String)> {
    (2..=5usize, token_category_strategy(), literal_value_strategy())
        .prop_flat_map(|(chain_length, category, final_value)| {
            // Generate unique names for the chain
            prop::collection::vec(token_name_strategy(), chain_length)
                .prop_map(move |names| {
                    // Ensure all names are unique
                    let mut unique_names: Vec<String> = names.into_iter().collect::<HashSet<_>>().into_iter().collect();
                    if unique_names.len() < chain_length {
                        // Pad with generated names if we don't have enough unique ones
                        for i in unique_names.len()..chain_length {
                            unique_names.push(format!("token{}", i));
                        }
                    }
                    (category.clone(), unique_names, final_value.clone())
                })
        })
        .prop_map(|(category, names, final_value)| {
            let mut tokens_map = HashMap::new();
            
            // Build the chain: each token references the next
            for i in 0..names.len() - 1 {
                let reference = format!("${}.{}", category.as_str(), names[i + 1]);
                tokens_map.insert(names[i].clone(), TokenValue::Reference(reference));
            }
            
            // Last token in chain has the literal value
            tokens_map.insert(names[names.len() - 1].clone(), TokenValue::Literal(final_value.clone()));
            
            // Create DesignTokens with the chain in the appropriate category
            let tokens = match category {
                TokenCategory::Colors => DesignTokens {
                    colors: tokens_map,
                    ..Default::default()
                },
                TokenCategory::Spacing => DesignTokens {
                    spacing: tokens_map,
                    ..Default::default()
                },
                TokenCategory::Typography => DesignTokens {
                    typography: tokens_map,
                    ..Default::default()
                },
                TokenCategory::Breakpoints => DesignTokens {
                    breakpoints: tokens_map,
                    ..Default::default()
                },
                _ => unreachable!(),
            };
            
            // Create a reference to the first token in the chain
            let start_ref = TokenRef {
                category,
                name: names[0].clone(),
                span: Span::empty(),
            };
            
            (tokens, start_ref, final_value)
        })
}

/// Generate a circular reference: a -> b -> c -> a
/// Returns (tokens, start_ref)
fn circular_reference_strategy() -> impl Strategy<Value = (DesignTokens, TokenRef)> {
    (2..=4usize, token_category_strategy())
        .prop_flat_map(|(cycle_length, category)| {
            // Generate unique names for the cycle
            prop::collection::vec(token_name_strategy(), cycle_length)
                .prop_map(move |names| {
                    // Ensure all names are unique
                    let mut unique_names: Vec<String> = names.into_iter().collect::<HashSet<_>>().into_iter().collect();
                    if unique_names.len() < cycle_length {
                        // Pad with generated names if we don't have enough unique ones
                        for i in unique_names.len()..cycle_length {
                            unique_names.push(format!("cycle{}", i));
                        }
                    }
                    (category.clone(), unique_names)
                })
        })
        .prop_map(|(category, names)| {
            let mut tokens_map = HashMap::new();
            
            // Build the cycle: each token references the next, last references first
            for i in 0..names.len() {
                let next_idx = (i + 1) % names.len();
                let reference = format!("${}.{}", category.as_str(), names[next_idx]);
                tokens_map.insert(names[i].clone(), TokenValue::Reference(reference));
            }
            
            // Create DesignTokens with the cycle in the appropriate category
            let tokens = match category {
                TokenCategory::Colors => DesignTokens {
                    colors: tokens_map,
                    ..Default::default()
                },
                TokenCategory::Spacing => DesignTokens {
                    spacing: tokens_map,
                    ..Default::default()
                },
                TokenCategory::Typography => DesignTokens {
                    typography: tokens_map,
                    ..Default::default()
                },
                TokenCategory::Breakpoints => DesignTokens {
                    breakpoints: tokens_map,
                    ..Default::default()
                },
                _ => unreachable!(),
            };
            
            // Create a reference to the first token in the cycle
            let start_ref = TokenRef {
                category,
                name: names[0].clone(),
                span: Span::empty(),
            };
            
            (tokens, start_ref)
        })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    /// Property: Token chains resolve to final literal value
    /// For any token chain where tokens reference other tokens,
    /// resolving the first token should follow the chain and return the final literal value.
    #[test]
    fn prop_token_chains_resolve_correctly(
        (tokens, start_ref, expected_value) in token_chain_strategy()
    ) {
        let result = resolve_token(&start_ref, &tokens, &mut HashSet::new());
        
        // Should resolve successfully
        prop_assert!(result.is_ok(), 
            "Token chain starting at {:?}.{} should resolve successfully", 
            start_ref.category, start_ref.name);
        
        // Should resolve to the final literal value
        let resolved_value = result.unwrap();
        prop_assert_eq!(resolved_value, expected_value,
            "Token chain should resolve to the final literal value");
    }
    
    /// Property: Circular references are detected
    /// For any circular token reference (a -> b -> c -> a),
    /// attempting to resolve should return an error indicating circular reference.
    #[test]
    fn prop_circular_references_detected(
        (tokens, start_ref) in circular_reference_strategy()
    ) {
        let result = resolve_token(&start_ref, &tokens, &mut HashSet::new());
        
        // Should fail with an error
        prop_assert!(result.is_err(), 
            "Circular reference starting at {:?}.{} should be detected", 
            start_ref.category, start_ref.name);
        
        // Verify it's a circular reference error (not some other error)
        let err = result.unwrap_err();
        let err_msg = format!("{:?}", err);
        prop_assert!(err_msg.contains("circular") || err_msg.contains("Circular"),
            "Error should indicate circular reference, got: {}", err_msg);
    }
    
    /// Property: Token composition is deterministic
    /// Resolving the same token chain multiple times should always return the same result.
    #[test]
    fn prop_token_composition_is_deterministic(
        (tokens, start_ref, _expected) in token_chain_strategy()
    ) {
        let result1 = resolve_token(&start_ref, &tokens, &mut HashSet::new());
        let result2 = resolve_token(&start_ref, &tokens, &mut HashSet::new());
        
        prop_assert_eq!(result1.is_ok(), result2.is_ok(), 
            "Token composition should be deterministic (same success/failure)");
        
        if let (Ok(val1), Ok(val2)) = (result1, result2) {
            prop_assert_eq!(val1, val2, "Resolved values should be identical");
        }
    }
    
    /// Property: Mixed chains with literals and references
    /// For any token configuration with both literal values and reference chains,
    /// all tokens should resolve correctly.
    #[test]
    fn prop_mixed_tokens_resolve_correctly(
        category in token_category_strategy(),
        literal_tokens in prop::collection::hash_map(token_name_strategy(), literal_value_strategy(), 3..=5),
        chain_data in token_chain_strategy()
    ) {
        let (chain_tokens, chain_ref, chain_expected) = chain_data;
        
        // Merge literal tokens into the chain tokens
        let merged_tokens = match category {
            TokenCategory::Colors => {
                let mut colors = chain_tokens.colors.clone();
                for (name, value) in literal_tokens {
                    // Only add if not already present (avoid overwriting chain)
                    if !colors.contains_key(&name) {
                        colors.insert(name, TokenValue::Literal(value));
                    }
                }
                DesignTokens {
                    colors,
                    spacing: chain_tokens.spacing,
                    typography: chain_tokens.typography,
                    breakpoints: chain_tokens.breakpoints,
                    shadows: chain_tokens.shadows,
                    borders: chain_tokens.borders,
                    radii: chain_tokens.radii,
                    zindex: chain_tokens.zindex,
                    opacity: chain_tokens.opacity,
                }
            },
            TokenCategory::Spacing => {
                let mut spacing = chain_tokens.spacing.clone();
                for (name, value) in literal_tokens {
                    if !spacing.contains_key(&name) {
                        spacing.insert(name, TokenValue::Literal(value));
                    }
                }
                DesignTokens {
                    colors: chain_tokens.colors,
                    spacing,
                    typography: chain_tokens.typography,
                    breakpoints: chain_tokens.breakpoints,
                    shadows: chain_tokens.shadows,
                    borders: chain_tokens.borders,
                    radii: chain_tokens.radii,
                    zindex: chain_tokens.zindex,
                    opacity: chain_tokens.opacity,
                }
            },
            TokenCategory::Typography => {
                let mut typography = chain_tokens.typography.clone();
                for (name, value) in literal_tokens {
                    if !typography.contains_key(&name) {
                        typography.insert(name, TokenValue::Literal(value));
                    }
                }
                DesignTokens {
                    colors: chain_tokens.colors,
                    spacing: chain_tokens.spacing,
                    typography,
                    breakpoints: chain_tokens.breakpoints,
                    shadows: chain_tokens.shadows,
                    borders: chain_tokens.borders,
                    radii: chain_tokens.radii,
                    zindex: chain_tokens.zindex,
                    opacity: chain_tokens.opacity,
                }
            },
            TokenCategory::Breakpoints => {
                let mut breakpoints = chain_tokens.breakpoints.clone();
                for (name, value) in literal_tokens {
                    if !breakpoints.contains_key(&name) {
                        breakpoints.insert(name, TokenValue::Literal(value));
                    }
                }
                DesignTokens {
                    colors: chain_tokens.colors,
                    spacing: chain_tokens.spacing,
                    typography: chain_tokens.typography,
                    breakpoints,
                    shadows: chain_tokens.shadows,
                    borders: chain_tokens.borders,
                    radii: chain_tokens.radii,
                    zindex: chain_tokens.zindex,
                    opacity: chain_tokens.opacity,
                }
            },
            _ => unreachable!(),
        };
        
        // The chain should still resolve correctly
        let result = resolve_token(&chain_ref, &merged_tokens, &mut HashSet::new());
        prop_assert!(result.is_ok(), "Chain should resolve in mixed token configuration");
        prop_assert_eq!(result.unwrap(), chain_expected, "Chain should resolve to expected value");
    }
    
    /// Property: Self-referencing token is detected as circular
    /// A token that references itself should be detected as a circular reference.
    #[test]
    fn prop_self_reference_detected(
        category in token_category_strategy(),
        name in token_name_strategy()
    ) {
        let reference = format!("${}.{}", category.as_str(), name);
        let mut tokens_map = HashMap::new();
        tokens_map.insert(name.clone(), TokenValue::Reference(reference));
        
        let tokens = match category {
            TokenCategory::Colors => DesignTokens {
                colors: tokens_map,
                ..Default::default()
            },
            TokenCategory::Spacing => DesignTokens {
                spacing: tokens_map,
                ..Default::default()
            },
            TokenCategory::Typography => DesignTokens {
                typography: tokens_map,
                ..Default::default()
            },
            TokenCategory::Breakpoints => DesignTokens {
                breakpoints: tokens_map,
                ..Default::default()
            },
            _ => unreachable!(),
        };
        
        let token_ref = TokenRef {
            category,
            name,
            span: Span::empty(),
        };
        
        let result = resolve_token(&token_ref, &tokens, &mut HashSet::new());
        
        // Should detect circular reference
        prop_assert!(result.is_err(), "Self-referencing token should be detected as circular");
    }
}
