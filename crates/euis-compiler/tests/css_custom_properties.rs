// Feature: euis-framework, Property 19: CSS Custom Property Generation
// **Validates: Requirements 18.6**

use proptest::prelude::*;
use euis_compiler::ast::*;
use euis_compiler::codegen::generate_css;
use euis_compiler::config::CompilerConfig;

/// Generate a valid class name.
fn class_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9-]{0,15}".prop_map(|s| s.to_string())
}

/// Generate a valid token name (alphanumeric with hyphens).
fn token_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9-]{0,15}".prop_map(|s| s.to_string())
}

/// Generate a random token category.
fn token_category_strategy() -> impl Strategy<Value = TokenCategory> {
    prop_oneof![
        Just(TokenCategory::Colors),
        Just(TokenCategory::Spacing),
        Just(TokenCategory::Typography),
        Just(TokenCategory::Breakpoints),
    ]
}

/// Generate a token reference (unresolved, for dynamic values).
fn token_ref_strategy() -> impl Strategy<Value = TokenRef> {
    (token_category_strategy(), token_name_strategy()).prop_map(|(category, name)| TokenRef {
        category,
        name,
        span: Span::empty(),
    })
}

/// Generate a valid CSS property name.
fn property_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("color".to_string()),
        Just("background-color".to_string()),
        Just("border-color".to_string()),
        Just("width".to_string()),
        Just("height".to_string()),
        Just("padding".to_string()),
        Just("margin".to_string()),
        Just("font-size".to_string()),
        Just("font-family".to_string()),
        Just("gap".to_string()),
        Just("transform".to_string()),
        Just("opacity".to_string()),
    ]
}

/// Generate a declaration with a token reference (dynamic value).
fn dynamic_declaration_strategy() -> impl Strategy<Value = Declaration> {
    (property_strategy(), token_ref_strategy()).prop_map(|(prop, token_ref)| Declaration {
        property: Property::Standard(prop),
        value: Value::Token(token_ref),
        important: false,
        span: Span::empty(),
    })
}

/// Generate a rule with dynamic value declarations.
fn dynamic_rule_strategy() -> impl Strategy<Value = Rule> {
    (
        class_name_strategy(),
        prop::collection::vec(dynamic_declaration_strategy(), 1..=5),
    )
        .prop_map(|(class_name, declarations)| Rule {
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
            declarations,
            states: vec![],
            responsive: vec![],
            nested_rules: vec![],
            nested_at_rules: vec![],
            span: Span::empty(),
        })
}

/// Generate a stylesheet with dynamic value declarations.
fn dynamic_stylesheet_strategy() -> impl Strategy<Value = StyleSheet> {
    prop::collection::vec(dynamic_rule_strategy(), 1..=10).prop_map(|rules| StyleSheet {
        rules,
        at_rules: vec![],
        span: Span::empty(),
    })
}

/// Validate that CSS custom properties are correctly formatted.
/// CSS custom properties should follow the pattern: var(--category-name)
fn validate_css_custom_property(css: &str, category: &TokenCategory, name: &str) -> bool {
    let expected = format!("var(--{}-{})", category.as_str(), name);
    css.contains(&expected)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property 19: CSS Custom Property Generation
    /// For any Euis declaration with dynamic values (token references),
    /// the compiler SHALL generate CSS custom properties that can be
    /// manipulated by JavaScript at runtime.
    ///
    /// Validates: Requirements 18.6
    #[test]
    fn prop_css_custom_properties_generated(stylesheet in dynamic_stylesheet_strategy()) {
        let config = CompilerConfig::default();
        let css = generate_css(&stylesheet, &config);

        // Verify CSS is generated
        prop_assert!(!css.trim().is_empty(), "Generated CSS should not be empty");

        // Verify each token reference generates a CSS custom property
        for rule in &stylesheet.rules {
            for decl in &rule.declarations {
                if let Value::Token(token_ref) = &decl.value {
                    let is_valid = validate_css_custom_property(
                        &css,
                        &token_ref.category,
                        &token_ref.name,
                    );
                    prop_assert!(
                        is_valid,
                        "CSS should contain custom property var(--{}-{}) for token reference",
                        token_ref.category.as_str(),
                        token_ref.name
                    );
                }
            }
        }
    }

    /// Property: CSS custom properties use correct naming convention
    /// For any token reference, the generated CSS custom property should
    /// follow the naming pattern: --{category}-{name}
    #[test]
    fn prop_css_custom_property_naming_convention(
        category in token_category_strategy(),
        name in token_name_strategy(),
        property in property_strategy(),
        class_name in class_name_strategy()
    ) {
        let token_ref = TokenRef {
            category: category.clone(),
            name: name.clone(),
            span: Span::empty(),
        };

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
                declarations: vec![Declaration {
                    property: Property::Standard(property.clone()),
                    value: Value::Token(token_ref.clone()),
                    important: false,
                    span: Span::empty(),
                }],
                states: vec![],
                responsive: vec![],
                nested_rules: vec![],
            nested_at_rules: vec![],
                span: Span::empty(),
            }],
            at_rules: vec![],
            span: Span::empty(),
        };

        let config = CompilerConfig::default();
        let css = generate_css(&stylesheet, &config);

        // Verify the CSS custom property follows the correct naming pattern
        let expected_var = format!("var(--{}-{})", category.as_str(), name);
        prop_assert!(
            css.contains(&expected_var),
            "CSS should contain custom property {} but got:\n{}",
            expected_var,
            css
        );

        // Verify the property name is present
        prop_assert!(
            css.contains(&property),
            "CSS should contain property name: {}",
            property
        );

        // Verify the class selector is present
        let class_selector = format!(".{}", class_name);
        prop_assert!(
            css.contains(&class_selector),
            "CSS should contain class selector: {}",
            class_selector
        );
    }

    /// Property: CSS custom properties work in state blocks
    /// For any token reference in a state block, the compiler should
    /// generate CSS custom properties correctly.
    #[test]
    fn prop_css_custom_properties_in_states(
        class_name in class_name_strategy(),
        token_ref in token_ref_strategy(),
        property in property_strategy()
    ) {
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
                states: vec![StateBlock {
                    modifiers: vec![StateModifier::Hover],
                    declarations: vec![Declaration {
                        property: Property::Standard(property.clone()),
                        value: Value::Token(token_ref.clone()),
                        important: false,
                        span: Span::empty(),
                    }],
                    span: Span::empty(),
                }],
                responsive: vec![],
                nested_rules: vec![],
            nested_at_rules: vec![],
                span: Span::empty(),
            }],
            at_rules: vec![],
            span: Span::empty(),
        };

        let config = CompilerConfig::default();
        let css = generate_css(&stylesheet, &config);

        // Verify the CSS custom property is generated
        let expected_var = format!("var(--{}-{})", token_ref.category.as_str(), token_ref.name);
        prop_assert!(
            css.contains(&expected_var),
            "CSS should contain custom property {} in state block",
            expected_var
        );

        // Verify the hover pseudo-class is present
        prop_assert!(
            css.contains(":hover"),
            "CSS should contain :hover pseudo-class"
        );
    }

    /// Property: CSS custom properties work in responsive blocks
    /// For any token reference in a responsive block, the compiler should
    /// generate CSS custom properties correctly within media queries.
    #[test]
    fn prop_css_custom_properties_in_responsive(
        class_name in class_name_strategy(),
        token_ref in token_ref_strategy(),
        property in property_strategy()
    ) {
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
                    breakpoint: "md".to_string(),
                    declarations: vec![Declaration {
                        property: Property::Standard(property.clone()),
                        value: Value::Token(token_ref.clone()),
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

        let config = CompilerConfig::default();
        let css = generate_css(&stylesheet, &config);

        // Verify the CSS custom property is generated
        let expected_var = format!("var(--{}-{})", token_ref.category.as_str(), token_ref.name);
        prop_assert!(
            css.contains(&expected_var),
            "CSS should contain custom property {} in responsive block",
            expected_var
        );

        // Verify the media query is present
        prop_assert!(
            css.contains("@media"),
            "CSS should contain @media query for responsive block"
        );
    }

    /// Property: Multiple CSS custom properties in single rule
    /// For any rule with multiple token references, all should generate
    /// correct CSS custom properties.
    #[test]
    fn prop_multiple_css_custom_properties(
        class_name in class_name_strategy(),
        token_refs in prop::collection::vec(
            (property_strategy(), token_ref_strategy()),
            2..=5
        )
    ) {
        let declarations: Vec<Declaration> = token_refs
            .iter()
            .map(|(prop, token_ref)| Declaration {
                property: Property::Standard(prop.clone()),
                value: Value::Token(token_ref.clone()),
                important: false,
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
                declarations,
                states: vec![],
                responsive: vec![],
                nested_rules: vec![],
            nested_at_rules: vec![],
                span: Span::empty(),
            }],
            at_rules: vec![],
            span: Span::empty(),
        };

        let config = CompilerConfig::default();
        let css = generate_css(&stylesheet, &config);

        // Verify all CSS custom properties are generated
        for (_prop, token_ref) in &token_refs {
            let expected_var = format!("var(--{}-{})", token_ref.category.as_str(), token_ref.name);
            prop_assert!(
                css.contains(&expected_var),
                "CSS should contain custom property {}",
                expected_var
            );
        }
    }

    /// Property: CSS custom properties are minification-safe
    /// For any token reference, the generated CSS custom property should
    /// remain valid even after minification.
    #[test]
    fn prop_css_custom_properties_minification_safe(
        class_name in class_name_strategy(),
        token_ref in token_ref_strategy(),
        property in property_strategy()
    ) {
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
                declarations: vec![Declaration {
                    property: Property::Standard(property.clone()),
                    value: Value::Token(token_ref.clone()),
                    important: false,
                    span: Span::empty(),
                }],
                states: vec![],
                responsive: vec![],
                nested_rules: vec![],
            nested_at_rules: vec![],
                span: Span::empty(),
            }],
            at_rules: vec![],
            span: Span::empty(),
        };

        // Generate minified CSS
        let mut config = CompilerConfig::default();
        config.minify = true;
        let css = generate_css(&stylesheet, &config);

        // Verify the CSS custom property is still present after minification
        let expected_var = format!("var(--{}-{})", token_ref.category.as_str(), token_ref.name);
        prop_assert!(
            css.contains(&expected_var),
            "Minified CSS should contain custom property {}",
            expected_var
        );
    }

    /// Property: CSS custom properties enable JavaScript manipulation
    /// For any token reference, the generated CSS custom property should
    /// use the var() function, which allows JavaScript to manipulate the value.
    #[test]
    fn prop_css_custom_properties_use_var_function(
        stylesheet in dynamic_stylesheet_strategy()
    ) {
        let config = CompilerConfig::default();
        let css = generate_css(&stylesheet, &config);

        // Count token references in the stylesheet
        let mut token_count = 0;
        for rule in &stylesheet.rules {
            for decl in &rule.declarations {
                if matches!(decl.value, Value::Token(_)) {
                    token_count += 1;
                }
            }
        }

        // If there are token references, verify var() is used
        if token_count > 0 {
            prop_assert!(
                css.contains("var(--"),
                "CSS should use var() function for token references"
            );

            // Count var() occurrences
            let var_count = css.matches("var(--").count();
            prop_assert!(
                var_count >= token_count,
                "CSS should have at least {} var() calls, found {}",
                token_count,
                var_count
            );
        }
    }
}
