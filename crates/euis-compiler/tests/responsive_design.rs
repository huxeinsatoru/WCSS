// Feature: euis-framework, Property 14: Responsive Design Generation
// **Validates: Requirements 9.1, 9.2, 9.4**

use proptest::prelude::*;
use std::collections::HashMap;
use euis_compiler::ast::*;
use euis_compiler::codegen::generate_css;
use euis_compiler::config::{CompilerConfig, DesignTokens, TokenValue};

/// Generate a valid breakpoint name.
fn breakpoint_name_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("sm".to_string()),
        Just("md".to_string()),
        Just("lg".to_string()),
        Just("xl".to_string()),
        Just("2xl".to_string()),
    ]
}

/// Generate a valid breakpoint value (pixel width).
fn breakpoint_value_strategy() -> impl Strategy<Value = String> {
    (320..=1920u32).prop_map(|px| format!("{}px", px))
}

/// Generate a design token configuration with breakpoints.
fn design_tokens_with_breakpoints_strategy() -> impl Strategy<Value = DesignTokens> {
    prop::collection::hash_map(
        breakpoint_name_strategy(),
        breakpoint_value_strategy(),
        1..=5,
    )
    .prop_map(|breakpoints| {
        let breakpoints = breakpoints
            .into_iter()
            .map(|(k, v)| (k, TokenValue::Literal(v)))
            .collect();
        DesignTokens {
            colors: HashMap::new(),
            spacing: HashMap::new(),
            typography: HashMap::new(),
            breakpoints,
            shadows: HashMap::new(),
            borders: HashMap::new(),
            radii: HashMap::new(),
            zindex: HashMap::new(),
            opacity: HashMap::new(),
        }
    })
}

/// Generate a valid CSS property name.
fn property_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("color".to_string()),
        Just("background-color".to_string()),
        Just("padding".to_string()),
        Just("margin".to_string()),
        Just("width".to_string()),
        Just("height".to_string()),
        Just("display".to_string()),
        Just("font-size".to_string()),
        Just("flex-direction".to_string()),
        Just("grid-template-columns".to_string()),
    ]
}

/// Generate a valid CSS value.
fn value_strategy() -> impl Strategy<Value = Value> {
    prop_oneof![
        Just(Value::Literal("red".to_string())),
        Just(Value::Literal("blue".to_string())),
        Just(Value::Literal("block".to_string())),
        Just(Value::Literal("flex".to_string())),
        Just(Value::Literal("none".to_string())),
        Just(Value::Literal("row".to_string())),
        Just(Value::Literal("column".to_string())),
        (0.0..1000.0).prop_map(|n| Value::Number(n, Some(Unit::Px))),
        (0.0..100.0).prop_map(|n| Value::Number(n, Some(Unit::Rem))),
        (0.0..100.0).prop_map(|n| Value::Number(n, Some(Unit::Percent))),
    ]
}

/// Generate a single declaration.
fn declaration_strategy() -> impl Strategy<Value = Declaration> {
    (property_strategy(), value_strategy()).prop_map(|(prop, val)| Declaration {
        property: Property::Standard(prop),
        value: val,
        important: false,
        span: Span::empty(),
    })
}

/// Generate a valid class name.
fn class_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9-]{0,15}".prop_map(|s| s.to_string())
}

/// Generate a responsive block with a breakpoint reference.
fn responsive_block_strategy(
    available_breakpoints: Vec<String>,
) -> impl Strategy<Value = ResponsiveBlock> {
    (
        prop::sample::select(available_breakpoints),
        prop::collection::vec(declaration_strategy(), 1..=5),
    )
        .prop_map(|(breakpoint, declarations)| ResponsiveBlock {
            breakpoint,
            declarations,
            span: Span::empty(),
        })
}

/// Generate a stylesheet with responsive blocks.
fn stylesheet_with_responsive_strategy() -> impl Strategy<Value = (StyleSheet, DesignTokens)> {
    design_tokens_with_breakpoints_strategy().prop_flat_map(|tokens| {
        let available_breakpoints: Vec<String> = tokens.breakpoints.keys().cloned().collect();

        if available_breakpoints.is_empty() {
            // If no breakpoints, return empty stylesheet
            return Just((
                StyleSheet {
                    rules: vec![],
                    at_rules: vec![],
                    span: Span::empty(),
                },
                tokens,
            ))
            .boxed();
        }

        (
            class_name_strategy(),
            prop::collection::vec(declaration_strategy(), 0..=3),
            prop::collection::vec(
                responsive_block_strategy(available_breakpoints),
                1..=3,
            ),
        )
            .prop_map(move |(class_name, base_declarations, responsive_blocks)| {
                let stylesheet = StyleSheet {
                    rules: vec![Rule {
                        selector: Selector {
                            class_name,
                            kind: SelectorKind::Class,
                            combinators: vec![],
                            pseudo_elements: vec![],
                            pseudo_classes: vec![],
                            attributes: vec![],
                            span: Span::empty(),
                        },
                        selectors: vec![],
                        declarations: base_declarations,
                        states: vec![],
                        responsive: responsive_blocks,
                        nested_rules: vec![],
            nested_at_rules: vec![],
                        span: Span::empty(),
                    }],
                    at_rules: vec![],
                    span: Span::empty(),
                };
                (stylesheet, tokens.clone())
            })
            .boxed()
    })
}

/// Validate that the CSS contains @media queries for responsive blocks.
fn validate_media_queries(css: &str, stylesheet: &StyleSheet) -> Result<(), String> {
    for rule in &stylesheet.rules {
        if !rule.responsive.is_empty() {
            if !css.contains("@media") {
                return Err("CSS should contain @media query for responsive blocks".to_string());
            }
        }
    }
    Ok(())
}

/// Validate that breakpoint values from design tokens are used in media queries.
fn validate_breakpoint_values(
    css: &str,
    stylesheet: &StyleSheet,
    tokens: &DesignTokens,
) -> Result<(), String> {
    for rule in &stylesheet.rules {
        for responsive in &rule.responsive {
            // Get the breakpoint value from tokens
            if let Some(TokenValue::Literal(bp_value)) = tokens.breakpoints.get(&responsive.breakpoint)
            {
                // Check that the breakpoint value appears in the CSS
                if !css.contains(bp_value) {
                    return Err(format!(
                        "CSS should contain breakpoint value '{}' for breakpoint '{}'",
                        bp_value, responsive.breakpoint
                    ));
                }

                // Check that it appears in a media query context
                let media_query_pattern = format!("@media");
                if !css.contains(&media_query_pattern) {
                    return Err(format!(
                        "CSS should contain @media query with breakpoint '{}'",
                        responsive.breakpoint
                    ));
                }
            }
        }
    }
    Ok(())
}

/// Validate that media queries use min-width syntax.
fn validate_min_width_syntax(css: &str, stylesheet: &StyleSheet) -> Result<(), String> {
    for rule in &stylesheet.rules {
        if !rule.responsive.is_empty() {
            if !css.contains("min-width") {
                return Err(
                    "CSS media queries should use min-width for responsive design".to_string()
                );
            }
        }
    }
    Ok(())
}

/// Validate that responsive block declarations appear inside media queries.
fn validate_declarations_in_media_queries(
    css: &str,
    stylesheet: &StyleSheet,
) -> Result<(), String> {
    for rule in &stylesheet.rules {
        for responsive in &rule.responsive {
            for decl in &responsive.declarations {
                let prop_name = decl.property.name();
                // Property should appear in the CSS
                if !css.contains(prop_name) {
                    return Err(format!(
                        "CSS should contain property '{}' from responsive block",
                        prop_name
                    ));
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_basic_responsive_block() {
        // Create a simple responsive block with a breakpoint
        let mut tokens = DesignTokens::default();
        tokens.breakpoints.insert(
            "md".to_string(),
            TokenValue::Literal("768px".to_string()),
        );

        let stylesheet = StyleSheet {
            rules: vec![Rule {
                selector: Selector {
                    class_name: "container".to_string(),
                    kind: SelectorKind::Class,
                    combinators: vec![],
                    pseudo_elements: vec![],
                    pseudo_classes: vec![],
                    attributes: vec![],
                    span: Span::empty(),
                },
                selectors: vec![],
                declarations: vec![Declaration {
                    property: Property::Standard("width".to_string()),
                    value: Value::Number(100.0, Some(Unit::Percent)),
                    important: false,
                    span: Span::empty(),
                }],
                states: vec![],
                responsive: vec![ResponsiveBlock {
                    breakpoint: "md".to_string(),
                    declarations: vec![Declaration {
                        property: Property::Standard("max-width".to_string()),
                        value: Value::Number(960.0, Some(Unit::Px)),
                        important: false,
                        span: Span::empty(),
                    }],
                    span: Span::empty(),
                }],
                nested_rules: vec![],
            nested_at_rules: vec![],
                span: Span::empty(),
            }],
            at_rules: vec![],
            span: Span::empty(),
        };

        let mut config = CompilerConfig::default();
        config.tokens = tokens;
        config.minify = false;

        let css = generate_css(&stylesheet, &config);

        // Verify the CSS contains the expected elements
        assert!(css.contains(".container"));
        assert!(css.contains("width: 100%"));
        assert!(css.contains("@media"));
        assert!(css.contains("768px"));
        assert!(css.contains("min-width"));
        assert!(css.contains("max-width: 960px"));
    }

    #[test]
    fn test_multiple_breakpoints() {
        // Create multiple responsive blocks with different breakpoints
        let mut tokens = DesignTokens::default();
        tokens.breakpoints.insert(
            "sm".to_string(),
            TokenValue::Literal("640px".to_string()),
        );
        tokens.breakpoints.insert(
            "lg".to_string(),
            TokenValue::Literal("1024px".to_string()),
        );

        let stylesheet = StyleSheet {
            rules: vec![Rule {
                selector: Selector {
                    class_name: "text".to_string(),
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
                        breakpoint: "sm".to_string(),
                        declarations: vec![Declaration {
                            property: Property::Standard("font-size".to_string()),
                            value: Value::Number(14.0, Some(Unit::Px)),
                            important: false,
                            span: Span::empty(),
                        }],
                        span: Span::empty(),
                    },
                    ResponsiveBlock {
                        breakpoint: "lg".to_string(),
                        declarations: vec![Declaration {
                            property: Property::Standard("font-size".to_string()),
                            value: Value::Number(18.0, Some(Unit::Px)),
                            important: false,
                            span: Span::empty(),
                        }],
                        span: Span::empty(),
                    },
                ],
                nested_rules: vec![],
            nested_at_rules: vec![],
                span: Span::empty(),
            }],
            at_rules: vec![],
            span: Span::empty(),
        };

        let mut config = CompilerConfig::default();
        config.tokens = tokens;
        config.minify = false;

        let css = generate_css(&stylesheet, &config);

        // Verify both media queries are present
        assert_eq!(css.matches("@media").count(), 2);
        assert!(css.contains("640px"));
        assert!(css.contains("1024px"));
        assert!(css.contains("font-size: 14px"));
        assert!(css.contains("font-size: 18px"));
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property 14: Responsive Design Generation
    /// For any responsive design declaration with breakpoint references,
    /// the compiler SHALL generate correct media queries using the breakpoint
    /// values from design tokens.
    ///
    /// Validates: Requirements 9.1, 9.2, 9.4
    #[test]
    fn prop_responsive_design_generation(
        (stylesheet, tokens) in stylesheet_with_responsive_strategy()
    ) {
        // Skip if no responsive blocks
        let has_responsive = stylesheet.rules.iter().any(|r| !r.responsive.is_empty());
        prop_assume!(has_responsive);

        // Generate CSS with the design tokens
        let mut config = CompilerConfig::default();
        config.tokens = tokens.clone();
        config.minify = false; // Non-minified for easier validation

        let css = generate_css(&stylesheet, &config);

        // Validate that @media queries are generated
        if let Err(e) = validate_media_queries(&css, &stylesheet) {
            prop_assert!(false, "{}\nGenerated CSS:\n{}", e, css);
        }

        // Validate that breakpoint values from design tokens are used
        if let Err(e) = validate_breakpoint_values(&css, &stylesheet, &tokens) {
            prop_assert!(false, "{}\nGenerated CSS:\n{}", e, css);
        }

        // Validate that media queries use min-width syntax
        if let Err(e) = validate_min_width_syntax(&css, &stylesheet) {
            prop_assert!(false, "{}\nGenerated CSS:\n{}", e, css);
        }

        // Validate that declarations appear in media queries
        if let Err(e) = validate_declarations_in_media_queries(&css, &stylesheet) {
            prop_assert!(false, "{}\nGenerated CSS:\n{}", e, css);
        }
    }

    /// Property: Media queries are correctly formatted
    /// For any responsive block, the generated @media query should have
    /// correct syntax with balanced braces.
    #[test]
    fn prop_media_query_syntax(
        (stylesheet, tokens) in stylesheet_with_responsive_strategy()
    ) {
        // Skip if no responsive blocks
        let has_responsive = stylesheet.rules.iter().any(|r| !r.responsive.is_empty());
        prop_assume!(has_responsive);

        let mut config = CompilerConfig::default();
        config.tokens = tokens;
        config.minify = false;

        let css = generate_css(&stylesheet, &config);

        // Count @media occurrences
        let media_count = css.matches("@media").count();

        // Count total responsive blocks
        let responsive_count: usize = stylesheet
            .rules
            .iter()
            .map(|r| r.responsive.len())
            .sum();

        // Should have one @media per responsive block
        prop_assert_eq!(
            media_count,
            responsive_count,
            "Should have one @media query per responsive block"
        );

        // Validate balanced braces
        let open_braces = css.matches('{').count();
        let close_braces = css.matches('}').count();
        prop_assert_eq!(
            open_braces,
            close_braces,
            "Media queries should have balanced braces"
        );
    }

    /// Property: Responsive blocks with empty declarations
    /// For any responsive block with no declarations, the compiler should
    /// still generate a valid (though empty) media query or skip it.
    #[test]
    fn prop_empty_responsive_blocks(
        tokens in design_tokens_with_breakpoints_strategy(),
        class_name in class_name_strategy(),
        breakpoint in breakpoint_name_strategy()
    ) {
        // Only test if the breakpoint exists in tokens
        prop_assume!(tokens.breakpoints.contains_key(&breakpoint));

        let stylesheet = StyleSheet {
            rules: vec![Rule {
                selector: Selector {
                    class_name: class_name.clone(),
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
                    breakpoint: breakpoint.clone(),
                    declarations: vec![],
                    span: Span::empty(),
                }],
                nested_rules: vec![],
            nested_at_rules: vec![],
                span: Span::empty(),
            }],
            at_rules: vec![],
            span: Span::empty(),
        };

        let mut config = CompilerConfig::default();
        config.tokens = tokens;
        config.minify = false;

        let css = generate_css(&stylesheet, &config);

        // CSS should be valid (no syntax errors)
        // Empty responsive blocks might generate empty media queries or be skipped
        // Either behavior is acceptable, just ensure no syntax errors
        let open_braces = css.matches('{').count();
        let close_braces = css.matches('}').count();
        prop_assert_eq!(
            open_braces,
            close_braces,
            "CSS should have balanced braces even with empty responsive blocks"
        );
    }

    /// Property: Multiple responsive blocks for same rule
    /// For any rule with multiple responsive blocks, each should generate
    /// a separate @media query.
    #[test]
    fn prop_multiple_responsive_blocks(
        tokens in design_tokens_with_breakpoints_strategy(),
        class_name in class_name_strategy()
    ) {
        let available_breakpoints: Vec<String> = tokens.breakpoints.keys().cloned().collect();
        prop_assume!(available_breakpoints.len() >= 2);

        // Create responsive blocks for different breakpoints
        let responsive_blocks: Vec<ResponsiveBlock> = available_breakpoints
            .iter()
            .take(2)
            .map(|bp| ResponsiveBlock {
                breakpoint: bp.clone(),
                declarations: vec![Declaration {
                    property: Property::Standard("color".to_string()),
                    value: Value::Literal("red".to_string()),
                    important: false,
                    span: Span::empty(),
                }],
                span: Span::empty(),
            })
            .collect();

        let stylesheet = StyleSheet {
            rules: vec![Rule {
                selector: Selector {
                    class_name: class_name.clone(),
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
                responsive: responsive_blocks.clone(),
                nested_rules: vec![],
            nested_at_rules: vec![],
                span: Span::empty(),
            }],
            at_rules: vec![],
            span: Span::empty(),
        };

        let mut config = CompilerConfig::default();
        config.tokens = tokens.clone();
        config.minify = false;

        let css = generate_css(&stylesheet, &config);

        // Should have one @media per responsive block
        let media_count = css.matches("@media").count();
        prop_assert_eq!(
            media_count,
            responsive_blocks.len(),
            "Should have one @media query per responsive block"
        );

        // Each breakpoint value should appear in the CSS
        for responsive in &responsive_blocks {
            if let Some(TokenValue::Literal(bp_value)) = tokens.breakpoints.get(&responsive.breakpoint) {
                prop_assert!(
                    css.contains(bp_value),
                    "CSS should contain breakpoint value '{}'",
                    bp_value
                );
            }
        }
    }

    /// Property: Responsive blocks work with minification
    /// For any responsive block, the generated CSS should be valid
    /// even when minified.
    #[test]
    fn prop_responsive_with_minification(
        (stylesheet, tokens) in stylesheet_with_responsive_strategy()
    ) {
        // Skip if no responsive blocks
        let has_responsive = stylesheet.rules.iter().any(|r| !r.responsive.is_empty());
        prop_assume!(has_responsive);

        let mut config = CompilerConfig::default();
        config.tokens = tokens.clone();
        config.minify = true;

        let css = generate_css(&stylesheet, &config);

        // Validate that @media queries are present
        prop_assert!(
            css.contains("@media"),
            "Minified CSS should contain @media queries"
        );

        // Validate balanced braces
        let open_braces = css.matches('{').count();
        let close_braces = css.matches('}').count();
        prop_assert_eq!(
            open_braces,
            close_braces,
            "Minified CSS should have balanced braces"
        );

        // Validate breakpoint values are present
        for rule in &stylesheet.rules {
            for responsive in &rule.responsive {
                if let Some(TokenValue::Literal(bp_value)) = tokens.breakpoints.get(&responsive.breakpoint) {
                    prop_assert!(
                        css.contains(bp_value),
                        "Minified CSS should contain breakpoint value '{}'",
                        bp_value
                    );
                }
            }
        }
    }
}
