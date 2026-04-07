// Task 5.4: Write unit tests for validation errors
// Tests undefined token errors, invalid property-value errors, and undefined breakpoint errors
//
// NOTE: Property-value validation (checking if CSS property names are valid and if values
// match the expected types for those properties) is not yet implemented in the validator.
// The current validator implementation focuses on:
// 1. Undefined token errors (TokenNotFound)
// 2. Undefined breakpoint errors (ValidationError)
//
// Once property-value validation is implemented in the validator (task 5.2), additional
// tests should be added here to cover invalid property-value combinations.

use std::collections::HashMap;
use wcss_compiler::ast::*;
use wcss_compiler::config::{CompilerConfig, DesignTokens, TokenValue};
use wcss_compiler::error::ErrorKind;
use wcss_compiler::validator::validate;

/// Helper to create a minimal compiler config with specific tokens
fn config_with_colors(colors: Vec<(&str, &str)>) -> CompilerConfig {
    let mut color_map = HashMap::new();
    for (name, value) in colors {
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

/// Helper to create a config with breakpoints
fn config_with_breakpoints(breakpoints: Vec<(&str, &str)>) -> CompilerConfig {
    let mut breakpoint_map = HashMap::new();
    for (name, value) in breakpoints {
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

/// Helper to create a simple rule with a single declaration
fn rule_with_declaration(property: &str, value: Value) -> Rule {
    Rule {
        selector: Selector {
            class_name: "test".to_string(),
            kind: SelectorKind::Class,
            combinators: vec![],
            pseudo_elements: vec![],
            pseudo_classes: vec![],
            attributes: vec![],
            span: Span::empty(),
        },
        selectors: vec![],
        declarations: vec![Declaration {
            property: Property::Standard(property.to_string()),
            value,
            important: false,
            span: Span::new(10, 20, 1, 10),
        }],
        states: vec![],
        responsive: vec![],
        nested_rules: vec![],
        span: Span::empty(),
    }
}

// ============================================================================
// Test undefined token errors
// ============================================================================

#[test]
fn test_undefined_color_token() {
    let config = config_with_colors(vec![("primary", "#3b82f6")]);
    
    let stylesheet = StyleSheet {
        rules: vec![rule_with_declaration(
            "color",
            Value::Token(TokenRef {
                category: TokenCategory::Colors,
                name: "danger".to_string(),
                span: Span::new(10, 20, 1, 10),
            }),
        )],
        at_rules: vec![],
        span: Span::empty(),
    };

    let errors = validate(&stylesheet, &config);
    
    assert_eq!(errors.len(), 1, "Should have exactly one error");
    assert_eq!(errors[0].kind, ErrorKind::TokenNotFound);
    assert!(
        errors[0].message.contains("danger"),
        "Error message should mention the undefined token name"
    );
}

#[test]
fn test_undefined_spacing_token() {
    let config = CompilerConfig {
        tokens: DesignTokens {
            spacing: {
                let mut map = HashMap::new();
                map.insert("sm".to_string(), TokenValue::Literal("0.5rem".to_string()));
                map
            },
            ..Default::default()
        },
        ..Default::default()
    };
    
    let stylesheet = StyleSheet {
        rules: vec![rule_with_declaration(
            "padding",
            Value::Token(TokenRef {
                category: TokenCategory::Spacing,
                name: "xl".to_string(),
                span: Span::new(15, 25, 2, 5),
            }),
        )],
        at_rules: vec![],
        span: Span::empty(),
    };

    let errors = validate(&stylesheet, &config);
    
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].kind, ErrorKind::TokenNotFound);
    assert!(errors[0].message.contains("xl"));
}

#[test]
fn test_undefined_typography_token() {
    let config = CompilerConfig {
        tokens: DesignTokens {
            typography: {
                let mut map = HashMap::new();
                map.insert("base".to_string(), TokenValue::Literal("16px".to_string()));
                map
            },
            ..Default::default()
        },
        ..Default::default()
    };
    
    let stylesheet = StyleSheet {
        rules: vec![rule_with_declaration(
            "font-size",
            Value::Token(TokenRef {
                category: TokenCategory::Typography,
                name: "large".to_string(),
                span: Span::new(20, 30, 3, 8),
            }),
        )],
        at_rules: vec![],
        span: Span::empty(),
    };

    let errors = validate(&stylesheet, &config);
    
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].kind, ErrorKind::TokenNotFound);
    assert!(errors[0].message.contains("large"));
}

#[test]
fn test_undefined_token_with_empty_config() {
    let config = CompilerConfig::default();
    
    let stylesheet = StyleSheet {
        rules: vec![rule_with_declaration(
            "color",
            Value::Token(TokenRef {
                category: TokenCategory::Colors,
                name: "primary".to_string(),
                span: Span::new(5, 15, 1, 5),
            }),
        )],
        at_rules: vec![],
        span: Span::empty(),
    };

    let errors = validate(&stylesheet, &config);
    
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].kind, ErrorKind::TokenNotFound);
}

#[test]
fn test_multiple_undefined_tokens() {
    let config = config_with_colors(vec![("primary", "#3b82f6")]);
    
    let stylesheet = StyleSheet {
        rules: vec![
            rule_with_declaration(
                "color",
                Value::Token(TokenRef {
                    category: TokenCategory::Colors,
                    name: "danger".to_string(),
                    span: Span::new(10, 20, 1, 10),
                }),
            ),
            rule_with_declaration(
                "background",
                Value::Token(TokenRef {
                    category: TokenCategory::Colors,
                    name: "warning".to_string(),
                    span: Span::new(30, 40, 2, 10),
                }),
            ),
        ],
        at_rules: vec![],
        span: Span::empty(),
    };

    let errors = validate(&stylesheet, &config);
    
    assert_eq!(errors.len(), 2, "Should report both undefined tokens");
    assert!(errors[0].message.contains("danger"));
    assert!(errors[1].message.contains("warning"));
}

#[test]
fn test_undefined_token_in_value_list() {
    let config = config_with_colors(vec![("primary", "#3b82f6")]);
    
    let stylesheet = StyleSheet {
        rules: vec![Rule {
            selector: Selector {
                class_name: "test".to_string(),
                kind: SelectorKind::Class,
                combinators: vec![],
                pseudo_elements: vec![],
                pseudo_classes: vec![],
                attributes: vec![],
                span: Span::empty(),
            },
            selectors: vec![],
            declarations: vec![Declaration {
                property: Property::Standard("border".to_string()),
                value: Value::List(vec![
                    Value::Literal("1px".to_string()),
                    Value::Literal("solid".to_string()),
                    Value::Token(TokenRef {
                        category: TokenCategory::Colors,
                        name: "border-color".to_string(),
                        span: Span::new(25, 40, 1, 25),
                    }),
                ]),
                important: false,
                span: Span::empty(),
            }],
            states: vec![],
            responsive: vec![],
            nested_rules: vec![],
            span: Span::empty(),
        }],
        at_rules: vec![],
        span: Span::empty(),
    };

    let errors = validate(&stylesheet, &config);
    
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].kind, ErrorKind::TokenNotFound);
    assert!(errors[0].message.contains("border-color"));
}

#[test]
fn test_defined_token_no_error() {
    let config = config_with_colors(vec![
        ("primary", "#3b82f6"),
        ("secondary", "#8b5cf6"),
    ]);
    
    let stylesheet = StyleSheet {
        rules: vec![rule_with_declaration(
            "color",
            Value::Token(TokenRef {
                category: TokenCategory::Colors,
                name: "primary".to_string(),
                span: Span::new(10, 20, 1, 10),
            }),
        )],
        at_rules: vec![],
        span: Span::empty(),
    };

    let errors = validate(&stylesheet, &config);
    
    assert_eq!(errors.len(), 0, "Defined tokens should not produce errors");
}

// ============================================================================
// Test undefined breakpoint errors
// ============================================================================

#[test]
fn test_undefined_breakpoint() {
    let config = config_with_breakpoints(vec![
        ("sm", "640px"),
        ("md", "768px"),
    ]);
    
    let stylesheet = StyleSheet {
        rules: vec![Rule {
            selector: Selector {
                class_name: "test".to_string(),
                kind: SelectorKind::Class,
                combinators: vec![],
                pseudo_elements: vec![],
                pseudo_classes: vec![],
                attributes: vec![],
                span: Span::empty(),
            },
            selectors: vec![],
            declarations: vec![],
            states: vec![],
            responsive: vec![ResponsiveBlock {
                breakpoint: "lg".to_string(),
                declarations: vec![Declaration {
                    property: Property::Standard("width".to_string()),
                    value: Value::Literal("100%".to_string()),
                    important: false,
                    span: Span::empty(),
                }],
                span: Span::new(30, 50, 3, 5),
            }],
            nested_rules: vec![],
            span: Span::empty(),
        }],
        at_rules: vec![],
        span: Span::empty(),
    };

    let errors = validate(&stylesheet, &config);
    
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].kind, ErrorKind::ValidationError);
    assert!(
        errors[0].message.contains("lg"),
        "Error message should mention the undefined breakpoint"
    );
    assert!(
        errors[0].message.contains("breakpoint"),
        "Error message should indicate it's a breakpoint error"
    );
}

#[test]
fn test_undefined_breakpoint_with_empty_config() {
    let config = CompilerConfig::default();
    
    let stylesheet = StyleSheet {
        rules: vec![Rule {
            selector: Selector {
                class_name: "test".to_string(),
                kind: SelectorKind::Class,
                combinators: vec![],
                pseudo_elements: vec![],
                pseudo_classes: vec![],
                attributes: vec![],
                span: Span::empty(),
            },
            selectors: vec![],
            declarations: vec![],
            states: vec![],
            responsive: vec![ResponsiveBlock {
                breakpoint: "md".to_string(),
                declarations: vec![],
                span: Span::new(20, 30, 2, 5),
            }],
            nested_rules: vec![],
            span: Span::empty(),
        }],
        at_rules: vec![],
        span: Span::empty(),
    };

    let errors = validate(&stylesheet, &config);

    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].kind, ErrorKind::ValidationError);
    assert!(errors[0].message.contains("md"));
}

#[test]
fn test_multiple_undefined_breakpoints() {
    let config = config_with_breakpoints(vec![("sm", "640px")]);
    
    let stylesheet = StyleSheet {
        rules: vec![Rule {
            selector: Selector {
                class_name: "test".to_string(),
                kind: SelectorKind::Class,
                combinators: vec![],
                pseudo_elements: vec![],
                pseudo_classes: vec![],
                attributes: vec![],
                span: Span::empty(),
            },
            selectors: vec![],
            declarations: vec![],
            states: vec![],
            responsive: vec![
                ResponsiveBlock {
                    breakpoint: "md".to_string(),
                    declarations: vec![],
                    span: Span::new(20, 30, 2, 5),
                },
                ResponsiveBlock {
                    breakpoint: "lg".to_string(),
                    declarations: vec![],
                    span: Span::new(40, 50, 4, 5),
                },
            ],
            nested_rules: vec![],
            span: Span::empty(),
        }],
        at_rules: vec![],
        span: Span::empty(),
    };

    let errors = validate(&stylesheet, &config);
    
    assert_eq!(errors.len(), 2, "Should report both undefined breakpoints");
    assert!(errors[0].message.contains("md"));
    assert!(errors[1].message.contains("lg"));
}

#[test]
fn test_defined_breakpoint_no_error() {
    let config = config_with_breakpoints(vec![
        ("sm", "640px"),
        ("md", "768px"),
        ("lg", "1024px"),
    ]);
    
    let stylesheet = StyleSheet {
        rules: vec![Rule {
            selector: Selector {
                class_name: "test".to_string(),
                kind: SelectorKind::Class,
                combinators: vec![],
                pseudo_elements: vec![],
                pseudo_classes: vec![],
                attributes: vec![],
                span: Span::empty(),
            },
            selectors: vec![],
            declarations: vec![],
            states: vec![],
            responsive: vec![ResponsiveBlock {
                breakpoint: "md".to_string(),
                declarations: vec![],
                span: Span::new(20, 30, 2, 5),
            }],
            nested_rules: vec![],
            span: Span::empty(),
        }],
        at_rules: vec![],
        span: Span::empty(),
    };

    let errors = validate(&stylesheet, &config);

    assert_eq!(errors.len(), 0, "Defined breakpoints should not produce errors");
}

#[test]
fn test_breakpoint_error_includes_suggestions() {
    let config = config_with_breakpoints(vec![
        ("sm", "640px"),
        ("md", "768px"),
    ]);
    
    let stylesheet = StyleSheet {
        rules: vec![Rule {
            selector: Selector {
                class_name: "test".to_string(),
                kind: SelectorKind::Class,
                combinators: vec![],
                pseudo_elements: vec![],
                pseudo_classes: vec![],
                attributes: vec![],
                span: Span::empty(),
            },
            selectors: vec![],
            declarations: vec![],
            states: vec![],
            responsive: vec![ResponsiveBlock {
                breakpoint: "lg".to_string(),
                declarations: vec![],
                span: Span::new(20, 30, 2, 5),
            }],
            nested_rules: vec![],
            span: Span::empty(),
        }],
        at_rules: vec![],
        span: Span::empty(),
    };

    let errors = validate(&stylesheet, &config);

    assert_eq!(errors.len(), 1);
    assert!(
        errors[0].suggestion.is_some(),
        "Error should include a suggestion"
    );
    let suggestion = errors[0].suggestion.as_ref().unwrap();
    assert!(
        suggestion.contains("sm") || suggestion.contains("md"),
        "Suggestion should list available breakpoints"
    );
}

// ============================================================================
// Test undefined tokens in state blocks
// ============================================================================

#[test]
fn test_undefined_token_in_state_block() {
    let config = config_with_colors(vec![("primary", "#3b82f6")]);
    
    let stylesheet = StyleSheet {
        rules: vec![Rule {
            selector: Selector {
                class_name: "button".to_string(),
                kind: SelectorKind::Class,
                combinators: vec![],
                pseudo_elements: vec![],
                pseudo_classes: vec![],
                attributes: vec![],
                span: Span::empty(),
            },
            selectors: vec![],
            declarations: vec![],
            states: vec![StateBlock {
                modifiers: vec![StateModifier::Hover],
                declarations: vec![Declaration {
                    property: Property::Standard("background".to_string()),
                    value: Value::Token(TokenRef {
                        category: TokenCategory::Colors,
                        name: "hover-bg".to_string(),
                        span: Span::new(35, 50, 4, 10),
                    }),
                    important: false,
                    span: Span::empty(),
                }],
                span: Span::empty(),
            }],
            responsive: vec![],
            nested_rules: vec![],
            span: Span::empty(),
        }],
        at_rules: vec![],
        span: Span::empty(),
    };

    let errors = validate(&stylesheet, &config);
    
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].kind, ErrorKind::TokenNotFound);
    assert!(errors[0].message.contains("hover-bg"));
}

// ============================================================================
// Test undefined tokens in responsive blocks
// ============================================================================

#[test]
fn test_undefined_token_in_responsive_block() {
    let config = CompilerConfig {
        tokens: DesignTokens {
            colors: {
                let mut map = HashMap::new();
                map.insert("primary".to_string(), TokenValue::Literal("#3b82f6".to_string()));
                map
            },
            breakpoints: {
                let mut map = HashMap::new();
                map.insert("md".to_string(), TokenValue::Literal("768px".to_string()));
                map
            },
            ..Default::default()
        },
        ..Default::default()
    };
    
    let stylesheet = StyleSheet {
        rules: vec![Rule {
            selector: Selector {
                class_name: "test".to_string(),
                kind: SelectorKind::Class,
                combinators: vec![],
                pseudo_elements: vec![],
                pseudo_classes: vec![],
                attributes: vec![],
                span: Span::empty(),
            },
            selectors: vec![],
            declarations: vec![],
            states: vec![],
            responsive: vec![ResponsiveBlock {
                breakpoint: "md".to_string(),
                declarations: vec![Declaration {
                    property: Property::Standard("color".to_string()),
                    value: Value::Token(TokenRef {
                        category: TokenCategory::Colors,
                        name: "secondary".to_string(),
                        span: Span::new(40, 55, 5, 10),
                    }),
                    important: false,
                    span: Span::empty(),
                }],
                span: Span::new(30, 60, 4, 5),
            }],
            nested_rules: vec![],
            span: Span::empty(),
        }],
        at_rules: vec![],
        span: Span::empty(),
    };

    let errors = validate(&stylesheet, &config);
    
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].kind, ErrorKind::TokenNotFound);
    assert!(errors[0].message.contains("secondary"));
}

// ============================================================================
// Test combined errors (tokens + breakpoints)
// ============================================================================

#[test]
fn test_combined_undefined_token_and_breakpoint_errors() {
    let config = config_with_colors(vec![("primary", "#3b82f6")]);
    
    let stylesheet = StyleSheet {
        rules: vec![Rule {
            selector: Selector {
                class_name: "test".to_string(),
                kind: SelectorKind::Class,
                combinators: vec![],
                pseudo_elements: vec![],
                pseudo_classes: vec![],
                attributes: vec![],
                span: Span::empty(),
            },
            selectors: vec![],
            declarations: vec![Declaration {
                property: Property::Standard("color".to_string()),
                value: Value::Token(TokenRef {
                    category: TokenCategory::Colors,
                    name: "danger".to_string(),
                    span: Span::new(10, 20, 1, 10),
                }),
                important: false,
                span: Span::empty(),
            }],
            states: vec![],
            responsive: vec![ResponsiveBlock {
                breakpoint: "lg".to_string(),
                declarations: vec![],
                span: Span::new(30, 40, 3, 5),
            }],
            nested_rules: vec![],
            span: Span::empty(),
        }],
        at_rules: vec![],
        span: Span::empty(),
    };

    let errors = validate(&stylesheet, &config);
    
    assert_eq!(errors.len(), 2, "Should report both token and breakpoint errors");
    
    // Check that we have one of each error type
    let token_errors: Vec<_> = errors.iter().filter(|e| e.kind == ErrorKind::TokenNotFound).collect();
    let validation_errors: Vec<_> = errors.iter().filter(|e| e.kind == ErrorKind::ValidationError).collect();
    
    assert_eq!(token_errors.len(), 1, "Should have one token error");
    assert_eq!(validation_errors.len(), 1, "Should have one validation error");
}

#[test]
fn test_error_span_information() {
    let config = config_with_colors(vec![("primary", "#3b82f6")]);
    
    let span = Span::new(15, 30, 2, 8);
    let stylesheet = StyleSheet {
        rules: vec![rule_with_declaration(
            "color",
            Value::Token(TokenRef {
                category: TokenCategory::Colors,
                name: "undefined".to_string(),
                span: span.clone(),
            }),
        )],
        at_rules: vec![],
        span: Span::empty(),
    };

    let errors = validate(&stylesheet, &config);
    
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].span.start, span.start);
    assert_eq!(errors[0].span.end, span.end);
    assert_eq!(errors[0].span.line, span.line);
    assert_eq!(errors[0].span.column, span.column);
}
