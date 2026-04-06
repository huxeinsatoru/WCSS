// Feature: wcss-framework, Property 12: Typed OM Code Generation
// **Validates: Requirements 7.1**

use proptest::prelude::*;
use wcss_compiler::ast::*;
use wcss_compiler::codegen::generate_typed_om_js;

/// Generate a valid class name.
fn class_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9-]{0,15}".prop_map(|s| s.to_string())
}

/// Generate a declaration with numeric values suitable for Typed OM.
fn typed_om_declaration_strategy() -> impl Strategy<Value = Declaration> {
    (
        prop_oneof![
            Just("width".to_string()),
            Just("height".to_string()),
            Just("padding".to_string()),
            Just("margin".to_string()),
            Just("font-size".to_string()),
            Just("top".to_string()),
            Just("left".to_string()),
            Just("opacity".to_string()),
        ],
        prop_oneof![
            (0.0..1000.0).prop_map(|n| Value::Number(n, Some(Unit::Px))),
            (0.0..100.0).prop_map(|n| Value::Number(n, Some(Unit::Rem))),
            (0.0..100.0).prop_map(|n| Value::Number(n, Some(Unit::Em))),
            (0.0..100.0).prop_map(|n| Value::Number(n, Some(Unit::Percent))),
            (0.0..100.0).prop_map(|n| Value::Number(n, Some(Unit::Vh))),
            (0.0..100.0).prop_map(|n| Value::Number(n, Some(Unit::Vw))),
            (0.0..1.0).prop_map(|n| Value::Number(n, None)),
        ],
    )
        .prop_map(|(prop, val)| Declaration {
            property: Property::Standard(prop),
            value: val,
            important: false,
            span: Span::empty(),
        })
}

/// Generate a rule with Typed OM-compatible declarations.
fn typed_om_rule_strategy() -> impl Strategy<Value = Rule> {
    (
        class_name_strategy(),
        prop::collection::vec(typed_om_declaration_strategy(), 1..=5),
    )
        .prop_map(|(class_name, declarations)| Rule {
            selector: Selector {
                class_name,
                combinators: vec![],
                pseudo_elements: vec![],
                span: Span::empty(),
            },
            declarations,
            states: vec![],
            responsive: vec![],
            span: Span::empty(),
        })
}

/// Generate a stylesheet with Typed OM-compatible rules.
fn typed_om_stylesheet_strategy() -> impl Strategy<Value = StyleSheet> {
    prop::collection::vec(typed_om_rule_strategy(), 1..=10).prop_map(|rules| StyleSheet {
        rules,
        span: Span::empty(),
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property 12: Typed OM Code Generation
    /// For any WCSS source compiled with Typed OM runtime enabled,
    /// the generated JavaScript SHALL use CSS Typed OM API
    /// (CSSStyleValue objects) instead of string-based styles.
    ///
    /// Validates: Requirements 7.1
    #[test]
    fn prop_typed_om_uses_css_api(stylesheet in typed_om_stylesheet_strategy()) {
        // Generate Typed OM JavaScript
        let js = generate_typed_om_js(&stylesheet);

        // Verify JavaScript is not empty
        prop_assert!(!js.is_empty(), "Generated JavaScript should not be empty");

        // Verify it uses CSS Typed OM API calls
        prop_assert!(
            js.contains("CSS.px(") || js.contains("CSS.rem(") || js.contains("CSS.em(")
                || js.contains("CSS.percent(") || js.contains("CSS.vh(")
                || js.contains("CSS.vw(") || js.contains("CSS.number("),
            "Generated JavaScript should use CSS Typed OM API (CSS.px, CSS.rem, etc.)"
        );

        // Verify it uses attributeStyleMap
        prop_assert!(
            js.contains("attributeStyleMap"),
            "Generated JavaScript should use attributeStyleMap for Typed OM"
        );

        // Verify it does NOT use string-based style assignment
        prop_assert!(
            !js.contains("element.style."),
            "Generated JavaScript should NOT use element.style (string-based)"
        );
    }

    /// Property: Typed OM generates valid JavaScript
    /// For any stylesheet, the generated Typed OM code should be syntactically valid JavaScript.
    #[test]
    fn prop_typed_om_generates_valid_javascript(stylesheet in typed_om_stylesheet_strategy()) {
        let js = generate_typed_om_js(&stylesheet);

        // Basic JavaScript syntax checks
        // Verify balanced braces
        let open_braces = js.matches('{').count();
        let close_braces = js.matches('}').count();
        prop_assert_eq!(
            open_braces,
            close_braces,
            "JavaScript should have balanced braces"
        );

        // Verify balanced parentheses
        let open_parens = js.matches('(').count();
        let close_parens = js.matches(')').count();
        prop_assert_eq!(
            open_parens,
            close_parens,
            "JavaScript should have balanced parentheses"
        );

        // Verify it's a valid module (has export)
        prop_assert!(
            js.contains("export"),
            "Generated JavaScript should be a valid ES module with export"
        );
    }

    /// Property: Typed OM includes all class names
    /// For any stylesheet with N rules, the generated JavaScript
    /// should include all N class names in the _styles object.
    #[test]
    fn prop_typed_om_includes_all_classes(stylesheet in typed_om_stylesheet_strategy()) {
        let js = generate_typed_om_js(&stylesheet);

        // Verify each class name appears in the generated JavaScript
        for rule in &stylesheet.rules {
            let class_name = &rule.selector.class_name;
            prop_assert!(
                js.contains(&format!("'{}':", class_name)),
                "Generated JavaScript should include class name '{}'",
                class_name
            );
        }
    }

    /// Property: Typed OM includes all properties
    /// For any rule with declarations, the generated JavaScript
    /// should include all property names.
    #[test]
    fn prop_typed_om_includes_all_properties(stylesheet in typed_om_stylesheet_strategy()) {
        let js = generate_typed_om_js(&stylesheet);

        // Verify each property appears in the generated JavaScript
        for rule in &stylesheet.rules {
            for decl in &rule.declarations {
                let prop_name = decl.property.name();
                prop_assert!(
                    js.contains(&format!("'{}':", prop_name)),
                    "Generated JavaScript should include property '{}'",
                    prop_name
                );
            }
        }
    }

    /// Property: Typed OM provides runtime API
    /// The generated JavaScript should provide the required runtime API methods:
    /// apply, update, remove, get.
    #[test]
    fn prop_typed_om_provides_runtime_api(stylesheet in typed_om_stylesheet_strategy()) {
        let js = generate_typed_om_js(&stylesheet);

        // Verify all required API methods are present
        prop_assert!(
            js.contains("apply(element, className)"),
            "Generated JavaScript should provide apply() method"
        );
        prop_assert!(
            js.contains("update(element, property, value)"),
            "Generated JavaScript should provide update() method"
        );
        prop_assert!(
            js.contains("remove(element, properties)"),
            "Generated JavaScript should provide remove() method"
        );
        prop_assert!(
            js.contains("get(element, property)"),
            "Generated JavaScript should provide get() method"
        );
    }

    /// Property: Typed OM uses correct CSS unit functions
    /// For each numeric value with a unit, the generated JavaScript
    /// should use the corresponding CSS Typed OM unit function.
    #[test]
    fn prop_typed_om_uses_correct_unit_functions(
        unit in prop_oneof![
            Just(Unit::Px),
            Just(Unit::Rem),
            Just(Unit::Em),
            Just(Unit::Percent),
            Just(Unit::Vh),
            Just(Unit::Vw),
        ],
        value in 0.0..1000.0
    ) {
        let stylesheet = StyleSheet {
            rules: vec![Rule {
                selector: Selector {
                    class_name: "test".to_string(),
                    combinators: vec![],
                    pseudo_elements: vec![],
                    span: Span::empty(),
                },
                declarations: vec![Declaration {
                    property: Property::Standard("width".to_string()),
                    value: Value::Number(value, Some(unit.clone())),
                    important: false,
                    span: Span::empty(),
                }],
                states: vec![],
                responsive: vec![],
                span: Span::empty(),
            }],
            span: Span::empty(),
        };

        let js = generate_typed_om_js(&stylesheet);

        // Verify the correct CSS unit function is used
        let expected_function = match unit {
            Unit::Px => "CSS.px(",
            Unit::Rem => "CSS.rem(",
            Unit::Em => "CSS.em(",
            Unit::Percent => "CSS.percent(",
            Unit::Vh => "CSS.vh(",
            Unit::Vw => "CSS.vw(",
            _ => "CSS.",
        };

        prop_assert!(
            js.contains(expected_function),
            "Generated JavaScript should use {} for {:?} unit",
            expected_function,
            unit
        );
    }

    /// Property: Typed OM handles unitless numbers
    /// For numeric values without units, the generated JavaScript
    /// should use CSS.number().
    #[test]
    fn prop_typed_om_handles_unitless_numbers(value in 0.0..10.0) {
        let stylesheet = StyleSheet {
            rules: vec![Rule {
                selector: Selector {
                    class_name: "test".to_string(),
                    combinators: vec![],
                    pseudo_elements: vec![],
                    span: Span::empty(),
                },
                declarations: vec![Declaration {
                    property: Property::Standard("opacity".to_string()),
                    value: Value::Number(value, None),
                    important: false,
                    span: Span::empty(),
                }],
                states: vec![],
                responsive: vec![],
                span: Span::empty(),
            }],
            span: Span::empty(),
        };

        let js = generate_typed_om_js(&stylesheet);

        // Verify CSS.number() is used for unitless values
        prop_assert!(
            js.contains("CSS.number("),
            "Generated JavaScript should use CSS.number() for unitless values"
        );
    }

    /// Property: Typed OM is deterministic
    /// Generating Typed OM JavaScript multiple times for the same stylesheet
    /// should produce identical results.
    #[test]
    fn prop_typed_om_is_deterministic(stylesheet in typed_om_stylesheet_strategy()) {
        let js1 = generate_typed_om_js(&stylesheet);
        let js2 = generate_typed_om_js(&stylesheet);

        prop_assert_eq!(
            js1,
            js2,
            "Typed OM JavaScript generation should be deterministic"
        );
    }
}
