// Feature: euis-framework, Property 17: Error Suggestions
// **Validates: Requirements 14.4**

use proptest::prelude::*;
use std::collections::HashMap;
use euis_compiler::ast::*;
use euis_compiler::config::{CompilerConfig, DesignTokens, TokenValue};
use euis_compiler::error::ErrorKind;
use euis_compiler::validator::validate;

/// Helper to create config with color tokens
fn create_config_with_tokens(tokens: Vec<(&str, &str)>) -> CompilerConfig {
    let mut color_map = HashMap::new();
    for (name, value) in tokens {
        color_map.insert(name.to_string(), TokenValue::Literal(value.to_string()));
    }
    CompilerConfig {
        tokens: DesignTokens {
            colors: color_map,
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Helper to create config with spacing tokens
fn create_config_with_spacing(tokens: Vec<(&str, &str)>) -> CompilerConfig {
    let mut spacing_map = HashMap::new();
    for (name, value) in tokens {
        spacing_map.insert(name.to_string(), TokenValue::Literal(value.to_string()));
    }
    CompilerConfig {
        tokens: DesignTokens {
            spacing: spacing_map,
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Helper to create config with breakpoints
fn create_config_with_breakpoints(tokens: Vec<(&str, &str)>) -> CompilerConfig {
    let mut breakpoint_map = HashMap::new();
    for (name, value) in tokens {
        breakpoint_map.insert(name.to_string(), TokenValue::Literal(value.to_string()));
    }
    CompilerConfig {
        tokens: DesignTokens {
            breakpoints: breakpoint_map,
            ..Default::default()
        },
        ..Default::default()
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    /// Property 17: Error Suggestions
    /// For any common error pattern, the compiler SHALL provide helpful
    /// suggestions for fixes when applicable.
    ///
    /// Validates: Requirements 14.4
    #[test]
    fn prop_error_suggestions_for_typos(
        base_name in "[a-z]{5,10}",
        mutation in 0..3usize,
    ) {
        // Create a typo by mutating the base name
        let typo = match mutation {
            0 => {
                // Delete one character
                if base_name.len() > 1 {
                    let idx = base_name.len() / 2;
                    format!("{}{}", &base_name[..idx], &base_name[idx+1..])
                } else {
                    base_name.clone()
                }
            },
            1 => {
                // Change one character
                let mut chars: Vec<char> = base_name.chars().collect();
                if !chars.is_empty() {
                    let idx = chars.len() / 2;
                    chars[idx] = 'x';
                }
                chars.into_iter().collect()
            },
            _ => {
                // Add one character
                let idx = base_name.len() / 2;
                format!("{}x{}", &base_name[..idx], &base_name[idx..])
            }
        };
        
        // Skip if typo is same as original
        prop_assume!(typo != base_name);
        
        // Create config with the correct token name
        let config = create_config_with_tokens(vec![(&base_name, "#3b82f6")]);
        
        // Create stylesheet with typo
        let stylesheet = StyleSheet {
            rules: vec![Rule {
                selector: Selector {
                    class_name: "test".to_string(),
                    kind: SelectorKind::Class,
                    pseudo_classes: vec![],
                    attributes: vec![],
                    combinators: vec![],
                    pseudo_elements: vec![],
                    span: Span::empty(),
                },
                declarations: vec![Declaration {
                    property: Property::Standard("color".to_string()),
                    value: Value::Token(TokenRef {
                        category: TokenCategory::Colors,
                        name: typo.clone(),
                        span: Span::new(10, 20, 1, 10),
                    }),
                    important: false,
                    span: Span::empty(),
                }],
                selectors: vec![],
                nested_rules: vec![],
            nested_at_rules: vec![],
                states: vec![],
                responsive: vec![],
                span: Span::empty(),
            }],
            at_rules: vec![],
            span: Span::empty(),
        };
        
        // Validate and get errors
        let errors = validate(&stylesheet, &config);
        
        // Should have an error for undefined token
        prop_assert!(!errors.is_empty(), "Should report undefined token error");
        
        let error = &errors[0];
        prop_assert_eq!(&error.kind, &ErrorKind::TokenNotFound);
        
        // Error should include a suggestion
        prop_assert!(
            error.suggestion.is_some(),
            "Error should include a suggestion for similar token name"
        );
        
        if let Some(ref suggestion) = error.suggestion {
            // Suggestion should mention the similar token name
            prop_assert!(
                suggestion.contains(&base_name),
                "Suggestion should mention the similar token name '{}'. Got: '{}'",
                base_name,
                suggestion
            );
        }
    }
    
    /// Property: Suggestions are helpful
    /// Error suggestions should provide actionable guidance.
    #[test]
    fn prop_suggestions_are_helpful(
        base_name in "[a-z]{5,10}",
        mutation in 0..2usize,
    ) {
        // Create a typo
        let typo = if mutation == 0 && base_name.len() > 1 {
            // Delete last character
            base_name[..base_name.len()-1].to_string()
        } else {
            // Change last character
            let mut chars: Vec<char> = base_name.chars().collect();
            if let Some(last) = chars.last_mut() {
                *last = 'z';
            }
            chars.into_iter().collect()
        };
        
        prop_assume!(typo != base_name);
        
        let config = create_config_with_tokens(vec![(&base_name, "#3b82f6")]);
        
        let stylesheet = StyleSheet {
            rules: vec![Rule {
                selector: Selector {
                    class_name: "test".to_string(),
                    kind: SelectorKind::Class,
                    pseudo_classes: vec![],
                    attributes: vec![],
                    combinators: vec![],
                    pseudo_elements: vec![],
                    span: Span::empty(),
                },
                declarations: vec![Declaration {
                    property: Property::Standard("color".to_string()),
                    value: Value::Token(TokenRef {
                        category: TokenCategory::Colors,
                        name: typo.clone(),
                        span: Span::new(10, 20, 1, 10),
                    }),
                    important: false,
                    span: Span::empty(),
                }],
                selectors: vec![],
                nested_rules: vec![],
            nested_at_rules: vec![],
                states: vec![],
                responsive: vec![],
                span: Span::empty(),
            }],
            at_rules: vec![],
            span: Span::empty(),
        };
        
        let errors = validate(&stylesheet, &config);
        
        if let Some(error) = errors.first() {
            if let Some(ref suggestion) = error.suggestion {
                // Suggestion should be non-empty
                prop_assert!(!suggestion.is_empty(), "Suggestion should not be empty");
                
                // Suggestion should be reasonably short (not a wall of text)
                prop_assert!(
                    suggestion.len() < 500,
                    "Suggestion should be concise"
                );
                
                // Suggestion should contain helpful keywords
                let suggestion_lower = suggestion.to_lowercase();
                let has_helpful_keyword = 
                    suggestion_lower.contains("did you mean") ||
                    suggestion_lower.contains("available") ||
                    suggestion_lower.contains("try") ||
                    suggestion_lower.contains("use") ||
                    suggestion_lower.contains("instead");
                
                prop_assert!(
                    has_helpful_keyword,
                    "Suggestion should contain helpful keywords. Got: '{}'",
                    suggestion
                );
            }
        }
    }
}

/// Unit test: Typo in color token suggests correct name
#[test]
fn test_color_token_typo_suggestion() {
    let config = create_config_with_tokens(vec![
        ("primary", "#3b82f6"),
        ("secondary", "#8b5cf6"),
        ("danger", "#ef4444"),
    ]);
    
    // Use "primry" instead of "primary"
    let stylesheet = StyleSheet {
        rules: vec![Rule {
            selector: Selector {
                class_name: "test".to_string(),
                kind: SelectorKind::Class,
                pseudo_classes: vec![],
                attributes: vec![],
                combinators: vec![],
                pseudo_elements: vec![],
                span: Span::empty(),
            },
            declarations: vec![Declaration {
                property: Property::Standard("color".to_string()),
                value: Value::Token(TokenRef {
                    category: TokenCategory::Colors,
                    name: "primry".to_string(),
                    span: Span::new(10, 20, 1, 10),
                }),
                important: false,
                span: Span::empty(),
            }],
            selectors: vec![],
            nested_rules: vec![],
            nested_at_rules: vec![],
            states: vec![],
            responsive: vec![],
            span: Span::empty(),
        }],
        at_rules: vec![],
        span: Span::empty(),
    };
    
    let errors = validate(&stylesheet, &config);
    
    assert!(!errors.is_empty(), "Should report error");
    assert_eq!(&errors[0].kind, &ErrorKind::TokenNotFound);
    assert!(errors[0].suggestion.is_some(), "Should include suggestion");
    
    let suggestion = errors[0].suggestion.as_ref().unwrap();
    assert!(
        suggestion.contains("primary"),
        "Suggestion should mention 'primary'. Got: '{}'",
        suggestion
    );
}

/// Unit test: Typo in spacing token suggests correct name
#[test]
fn test_spacing_token_typo_suggestion() {
    let config = create_config_with_spacing(vec![
        ("small", "0.5rem"),
        ("medium", "1rem"),
        ("large", "1.5rem"),
    ]);
    
    // Use "smal" instead of "small"
    let stylesheet = StyleSheet {
        rules: vec![Rule {
            selector: Selector {
                class_name: "test".to_string(),
                kind: SelectorKind::Class,
                pseudo_classes: vec![],
                attributes: vec![],
                combinators: vec![],
                pseudo_elements: vec![],
                span: Span::empty(),
            },
            declarations: vec![Declaration {
                property: Property::Standard("padding".to_string()),
                value: Value::Token(TokenRef {
                    category: TokenCategory::Spacing,
                    name: "smal".to_string(),
                    span: Span::new(15, 25, 2, 5),
                }),
                important: false,
                span: Span::empty(),
            }],
            selectors: vec![],
            nested_rules: vec![],
            nested_at_rules: vec![],
            states: vec![],
            responsive: vec![],
            span: Span::empty(),
        }],
        at_rules: vec![],
        span: Span::empty(),
    };
    
    let errors = validate(&stylesheet, &config);
    
    assert!(!errors.is_empty(), "Should report error");
    assert!(errors[0].suggestion.is_some(), "Should include suggestion");
    
    let suggestion = errors[0].suggestion.as_ref().unwrap();
    assert!(
        suggestion.contains("small"),
        "Suggestion should mention 'small'. Got: '{}'",
        suggestion
    );
}

/// Unit test: Undefined breakpoint suggests available breakpoints
#[test]
fn test_breakpoint_suggestion() {
    let config = create_config_with_breakpoints(vec![
        ("sm", "640px"),
        ("md", "768px"),
        ("lg", "1024px"),
    ]);
    
    // Use undefined breakpoint "xl"
    let stylesheet = StyleSheet {
        rules: vec![Rule {
            selector: Selector {
                class_name: "test".to_string(),
                kind: SelectorKind::Class,
                pseudo_classes: vec![],
                attributes: vec![],
                combinators: vec![],
                pseudo_elements: vec![],
                span: Span::empty(),
            },
            declarations: vec![],
            selectors: vec![],
            nested_rules: vec![],
            nested_at_rules: vec![],
            states: vec![],
            responsive: vec![ResponsiveBlock {
                breakpoint: "xl".to_string(),
                declarations: vec![],
                span: Span::new(20, 30, 3, 5),
            }],
            span: Span::empty(),
        }],
        at_rules: vec![],
        span: Span::empty(),
    };
    
    let errors = validate(&stylesheet, &config);
    
    assert!(!errors.is_empty(), "Should report error");
    assert!(errors[0].suggestion.is_some(), "Should include suggestion");
    
    let suggestion = errors[0].suggestion.as_ref().unwrap();
    // Should list available breakpoints
    assert!(
        suggestion.contains("sm") || suggestion.contains("md") || suggestion.contains("lg"),
        "Suggestion should list available breakpoints. Got: '{}'",
        suggestion
    );
}

/// Unit test: Close typo gets better suggestion than distant typo
#[test]
fn test_suggestion_quality() {
    let config = create_config_with_tokens(vec![
        ("primary", "#3b82f6"),
        ("secondary", "#8b5cf6"),
        ("tertiary", "#10b981"),
    ]);
    
    // "primry" is closer to "primary" than to "secondary" or "tertiary"
    let stylesheet = StyleSheet {
        rules: vec![Rule {
            selector: Selector {
                class_name: "test".to_string(),
                kind: SelectorKind::Class,
                pseudo_classes: vec![],
                attributes: vec![],
                combinators: vec![],
                pseudo_elements: vec![],
                span: Span::empty(),
            },
            declarations: vec![Declaration {
                property: Property::Standard("color".to_string()),
                value: Value::Token(TokenRef {
                    category: TokenCategory::Colors,
                    name: "primry".to_string(),
                    span: Span::new(10, 20, 1, 10),
                }),
                important: false,
                span: Span::empty(),
            }],
            selectors: vec![],
            nested_rules: vec![],
            nested_at_rules: vec![],
            states: vec![],
            responsive: vec![],
            span: Span::empty(),
        }],
        at_rules: vec![],
        span: Span::empty(),
    };
    
    let errors = validate(&stylesheet, &config);
    
    assert!(!errors.is_empty(), "Should report error");
    assert!(errors[0].suggestion.is_some(), "Should include suggestion");
    
    let suggestion = errors[0].suggestion.as_ref().unwrap();
    // Should suggest "primary" (closest match)
    assert!(
        suggestion.contains("primary"),
        "Should suggest closest match 'primary'. Got: '{}'",
        suggestion
    );
}

/// Unit test: Multiple similar tokens listed in suggestion
#[test]
fn test_multiple_suggestions() {
    let config = create_config_with_tokens(vec![
        ("primary-light", "#60a5fa"),
        ("primary", "#3b82f6"),
        ("primary-dark", "#2563eb"),
    ]);
    
    // "prim" could match multiple tokens
    let stylesheet = StyleSheet {
        rules: vec![Rule {
            selector: Selector {
                class_name: "test".to_string(),
                kind: SelectorKind::Class,
                pseudo_classes: vec![],
                attributes: vec![],
                combinators: vec![],
                pseudo_elements: vec![],
                span: Span::empty(),
            },
            declarations: vec![Declaration {
                property: Property::Standard("color".to_string()),
                value: Value::Token(TokenRef {
                    category: TokenCategory::Colors,
                    name: "prim".to_string(),
                    span: Span::new(10, 20, 1, 10),
                }),
                important: false,
                span: Span::empty(),
            }],
            selectors: vec![],
            nested_rules: vec![],
            nested_at_rules: vec![],
            states: vec![],
            responsive: vec![],
            span: Span::empty(),
        }],
        at_rules: vec![],
        span: Span::empty(),
    };
    
    let errors = validate(&stylesheet, &config);
    
    assert!(!errors.is_empty(), "Should report error");
    assert!(errors[0].suggestion.is_some(), "Should include suggestion");
    
    let suggestion = errors[0].suggestion.as_ref().unwrap();
    // Should mention at least one of the similar tokens
    let has_similar = suggestion.contains("primary") ||
                      suggestion.contains("primary-light") ||
                      suggestion.contains("primary-dark");
    
    assert!(
        has_similar,
        "Should suggest similar token names. Got: '{}'",
        suggestion
    );
}

/// Unit test: No suggestion for completely unrelated token name
#[test]
fn test_no_suggestion_for_unrelated_name() {
    let config = create_config_with_tokens(vec![
        ("primary", "#3b82f6"),
        ("secondary", "#8b5cf6"),
    ]);
    
    // "xyz" is not similar to any defined token
    let stylesheet = StyleSheet {
        rules: vec![Rule {
            selector: Selector {
                class_name: "test".to_string(),
                kind: SelectorKind::Class,
                pseudo_classes: vec![],
                attributes: vec![],
                combinators: vec![],
                pseudo_elements: vec![],
                span: Span::empty(),
            },
            declarations: vec![Declaration {
                property: Property::Standard("color".to_string()),
                value: Value::Token(TokenRef {
                    category: TokenCategory::Colors,
                    name: "completely-unrelated-name-xyz".to_string(),
                    span: Span::new(10, 20, 1, 10),
                }),
                important: false,
                span: Span::empty(),
            }],
            selectors: vec![],
            nested_rules: vec![],
            nested_at_rules: vec![],
            states: vec![],
            responsive: vec![],
            span: Span::empty(),
        }],
        at_rules: vec![],
        span: Span::empty(),
    };
    
    let errors = validate(&stylesheet, &config);
    
    assert!(!errors.is_empty(), "Should report error");
    // May or may not have suggestion depending on implementation
    // If there is a suggestion, it should list available tokens
    if let Some(ref suggestion) = errors[0].suggestion {
        assert!(
            suggestion.contains("primary") || suggestion.contains("secondary"),
            "If suggestion provided, should list available tokens"
        );
    }
}
