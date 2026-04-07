// Feature: wcss-framework, Property 1: Parse-Format Round-Trip Preservation
// **Validates: Requirements 1.1, 13.1, 13.4**

use proptest::prelude::*;
use wcss_compiler::ast::*;
use wcss_compiler::formatter::format_stylesheet;
use wcss_compiler::parser::parse;

/// Generate a valid class name.
fn class_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9-]{0,15}".prop_map(|s| s.to_string())
}

/// Generate a valid property name.
fn property_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("color".to_string()),
        Just("background".to_string()),
        Just("background-color".to_string()),
        Just("padding".to_string()),
        Just("margin".to_string()),
        Just("width".to_string()),
        Just("height".to_string()),
        Just("display".to_string()),
        Just("font-size".to_string()),
        Just("font-family".to_string()),
        Just("border".to_string()),
        Just("opacity".to_string()),
    ]
}

/// Generate a valid CSS value.
fn value_strategy() -> impl Strategy<Value = Value> {
    prop_oneof![
        // Literal strings
        Just(Value::Literal("red".to_string())),
        Just(Value::Literal("blue".to_string())),
        Just(Value::Literal("green".to_string())),
        Just(Value::Literal("white".to_string())),
        Just(Value::Literal("black".to_string())),
        Just(Value::Literal("block".to_string())),
        Just(Value::Literal("flex".to_string())),
        Just(Value::Literal("none".to_string())),
        Just(Value::Literal("auto".to_string())),
        Just(Value::Literal("solid".to_string())),
        // Numbers with units
        (0.0..1000.0).prop_map(|n| Value::Number(n, Some(Unit::Px))),
        (0.0..100.0).prop_map(|n| Value::Number(n, Some(Unit::Rem))),
        (0.0..100.0).prop_map(|n| Value::Number(n, Some(Unit::Em))),
        (0.0..100.0).prop_map(|n| Value::Number(n, Some(Unit::Percent))),
        // Hex colors
        "[0-9a-f]{6}".prop_map(|hex| Value::Color(Color::Hex(format!("#{}", hex)))),
        // Token references
        (
            prop_oneof![
                Just(TokenCategory::Colors),
                Just(TokenCategory::Spacing),
                Just(TokenCategory::Typography),
            ],
            "[a-z][a-z0-9-]{0,10}".prop_map(|s| s.to_string()),
        ).prop_map(|(category, name)| Value::Token(TokenRef {
            category,
            name,
            span: Span::empty(),
        })),
    ]
}

/// Generate a declaration.
fn declaration_strategy() -> impl Strategy<Value = Declaration> {
    (property_strategy(), value_strategy()).prop_map(|(prop, val)| Declaration {
        property: Property::Standard(prop),
        value: val,
        important: false,
        span: Span::empty(),
    })
}

/// Generate a state block.
fn state_block_strategy() -> impl Strategy<Value = StateBlock> {
    (
        prop::collection::vec(
            prop_oneof![
                Just(StateModifier::Hover),
                Just(StateModifier::Focus),
                Just(StateModifier::Active),
            ],
            1..=2,
        ),
        prop::collection::vec(declaration_strategy(), 1..=3),
    )
        .prop_map(|(modifiers, declarations)| StateBlock {
            modifiers,
            declarations,
            span: Span::empty(),
        })
}

/// Generate a responsive block.
fn responsive_block_strategy() -> impl Strategy<Value = ResponsiveBlock> {
    (
        prop_oneof![
            Just("sm".to_string()),
            Just("md".to_string()),
            Just("lg".to_string()),
            Just("xl".to_string()),
        ],
        prop::collection::vec(declaration_strategy(), 1..=3),
    )
        .prop_map(|(breakpoint, declarations)| ResponsiveBlock {
            breakpoint,
            declarations,
            span: Span::empty(),
        })
}

/// Generate a rule.
fn rule_strategy() -> impl Strategy<Value = Rule> {
    (
        class_name_strategy(),
        prop::collection::vec(declaration_strategy(), 1..=5),
        prop::collection::vec(state_block_strategy(), 0..=2),
        prop::collection::vec(responsive_block_strategy(), 0..=2),
    )
        .prop_map(|(class_name, declarations, states, responsive)| Rule {
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
            states,
            responsive,
            nested_rules: vec![],
            span: Span::empty(),
        })
}

/// Generate a stylesheet.
fn stylesheet_strategy() -> impl Strategy<Value = StyleSheet> {
    prop::collection::vec(rule_strategy(), 1..=10).prop_map(|rules| StyleSheet {
        rules,
        at_rules: vec![],
        span: Span::empty(),
    })
}

/// Normalize a value by removing span information from TokenRefs.
fn normalize_value(value: &Value) -> Value {
    match value {
        Value::Token(token_ref) => Value::Token(TokenRef {
            category: token_ref.category.clone(),
            name: token_ref.name.clone(),
            span: Span::empty(),
        }),
        Value::List(values) => Value::List(values.iter().map(normalize_value).collect()),
        Value::Computed(expr) => Value::Computed(Box::new(normalize_expr(expr))),
        _ => value.clone(),
    }
}

/// Normalize an expression by removing span information.
fn normalize_expr(expr: &Expr) -> Expr {
    match expr {
        Expr::Value(v) => Expr::Value(normalize_value(v)),
        Expr::Add(left, right) => Expr::Add(
            Box::new(normalize_expr(left)),
            Box::new(normalize_expr(right)),
        ),
        Expr::Sub(left, right) => Expr::Sub(
            Box::new(normalize_expr(left)),
            Box::new(normalize_expr(right)),
        ),
        Expr::Mul(left, right) => Expr::Mul(
            Box::new(normalize_expr(left)),
            Box::new(normalize_expr(right)),
        ),
        Expr::Div(left, right) => Expr::Div(
            Box::new(normalize_expr(left)),
            Box::new(normalize_expr(right)),
        ),
        Expr::Function(name, args) => Expr::Function(
            name.clone(),
            args.iter().map(normalize_expr).collect(),
        ),
    }
}

/// Normalize AST by removing span information for comparison.
fn normalize_ast(stylesheet: &StyleSheet) -> StyleSheet {
    StyleSheet {
        rules: stylesheet
            .rules
            .iter()
            .map(|rule| Rule {
                selector: Selector {
                    class_name: rule.selector.class_name.clone(),
                    kind: rule.selector.kind.clone(),
                    combinators: rule.selector.combinators.clone(),
                    pseudo_elements: rule.selector.pseudo_elements.clone(),
                    pseudo_classes: rule.selector.pseudo_classes.clone(),
                    attributes: rule.selector.attributes.clone(),
                    span: Span::empty(),
                },
                selectors: rule.selectors.clone(),
                declarations: rule
                    .declarations
                    .iter()
                    .map(|decl| Declaration {
                        property: decl.property.clone(),
                        value: normalize_value(&decl.value),
                        important: decl.important,
                        span: Span::empty(),
                    })
                    .collect(),
                states: rule
                    .states
                    .iter()
                    .map(|state| StateBlock {
                        modifiers: state.modifiers.clone(),
                        declarations: state
                            .declarations
                            .iter()
                            .map(|decl| Declaration {
                                property: decl.property.clone(),
                                value: normalize_value(&decl.value),
                                important: decl.important,
                                span: Span::empty(),
                            })
                            .collect(),
                        span: Span::empty(),
                    })
                    .collect(),
                responsive: rule
                    .responsive
                    .iter()
                    .map(|resp| ResponsiveBlock {
                        breakpoint: resp.breakpoint.clone(),
                        declarations: resp
                            .declarations
                            .iter()
                            .map(|decl| Declaration {
                                property: decl.property.clone(),
                                value: normalize_value(&decl.value),
                                important: decl.important,
                                span: Span::empty(),
                            })
                            .collect(),
                        span: Span::empty(),
                    })
                    .collect(),
                nested_rules: vec![],
                span: Span::empty(),
            })
            .collect(),
        at_rules: vec![],
        span: Span::empty(),
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property 1: Parse-Format Round-Trip Preservation
    /// For any valid WCSS source code, parsing to an AST, formatting back
    /// to source, and parsing again SHALL produce an equivalent abstract
    /// syntax tree.
    ///
    /// Validates: Requirements 1.1, 13.1, 13.4
    #[test]
    fn prop_parse_format_roundtrip(stylesheet in stylesheet_strategy()) {
        // Format the AST to WCSS source
        let formatted_source = format_stylesheet(&stylesheet);

        // Parse the formatted source back to AST
        let parse_result = parse(&formatted_source);

        // Verify parsing succeeds
        prop_assert!(
            parse_result.is_ok(),
            "Formatted WCSS should parse successfully. Errors: {:?}\nFormatted source:\n{}",
            parse_result.err(),
            formatted_source
        );

        let parsed_stylesheet = parse_result.unwrap();

        // Normalize both ASTs (remove span information)
        let normalized_original = normalize_ast(&stylesheet);
        let normalized_parsed = normalize_ast(&parsed_stylesheet);

        // Verify ASTs are equivalent
        prop_assert_eq!(
            normalized_original,
            normalized_parsed,
            "Round-trip ASTs should be equivalent.\nFormatted source:\n{}",
            formatted_source
        );
    }

    /// Property: Format produces valid WCSS
    /// For any AST, formatting should produce valid WCSS that can be parsed.
    #[test]
    fn prop_format_produces_valid_wcss(stylesheet in stylesheet_strategy()) {
        let formatted = format_stylesheet(&stylesheet);

        // Verify formatted output is not empty
        prop_assert!(!formatted.is_empty(), "Formatted output should not be empty");

        // Verify it can be parsed
        let parse_result = parse(&formatted);
        prop_assert!(
            parse_result.is_ok(),
            "Formatted WCSS should be parseable. Errors: {:?}\nFormatted:\n{}",
            parse_result.err(),
            formatted
        );
    }

    /// Property: Format is idempotent
    /// Formatting an AST, parsing it, and formatting again should produce
    /// the same output.
    #[test]
    fn prop_format_is_idempotent(stylesheet in stylesheet_strategy()) {
        // First format
        let formatted1 = format_stylesheet(&stylesheet);

        // Parse and format again
        let parsed = parse(&formatted1).unwrap();
        let formatted2 = format_stylesheet(&parsed);

        // Verify outputs are identical
        prop_assert_eq!(
            formatted1,
            formatted2,
            "Formatting should be idempotent"
        );
    }

    /// Property: Round-trip preserves class names
    /// For any stylesheet, all class names should be preserved through
    /// the parse-format-parse cycle.
    #[test]
    fn prop_roundtrip_preserves_class_names(stylesheet in stylesheet_strategy()) {
        let formatted = format_stylesheet(&stylesheet);
        let parsed = parse(&formatted).unwrap();

        // Collect class names from original
        let original_classes: Vec<_> = stylesheet
            .rules
            .iter()
            .map(|r| r.selector.class_name.as_str())
            .collect();

        // Collect class names from parsed
        let parsed_classes: Vec<_> = parsed
            .rules
            .iter()
            .map(|r| r.selector.class_name.as_str())
            .collect();

        prop_assert_eq!(
            original_classes,
            parsed_classes,
            "Class names should be preserved through round-trip"
        );
    }

    /// Property: Round-trip preserves property names
    /// For any stylesheet, all property names should be preserved through
    /// the parse-format-parse cycle.
    #[test]
    fn prop_roundtrip_preserves_properties(stylesheet in stylesheet_strategy()) {
        let formatted = format_stylesheet(&stylesheet);
        let parsed = parse(&formatted).unwrap();

        // Collect property names from original
        let original_props: Vec<_> = stylesheet
            .rules
            .iter()
            .flat_map(|r| r.declarations.iter())
            .map(|d| d.property.name())
            .collect();

        // Collect property names from parsed
        let parsed_props: Vec<_> = parsed
            .rules
            .iter()
            .flat_map(|r| r.declarations.iter())
            .map(|d| d.property.name())
            .collect();

        prop_assert_eq!(
            original_props,
            parsed_props,
            "Property names should be preserved through round-trip"
        );
    }

    /// Property: Round-trip preserves state modifiers
    /// For any stylesheet with state blocks, all state modifiers should be
    /// preserved through the parse-format-parse cycle.
    #[test]
    fn prop_roundtrip_preserves_states(
        stylesheet in stylesheet_strategy().prop_filter(
            "Has state blocks",
            |s| s.rules.iter().any(|r| !r.states.is_empty())
        )
    ) {
        let formatted = format_stylesheet(&stylesheet);
        let parsed = parse(&formatted).unwrap();

        // Collect state modifiers from original
        let original_states: Vec<_> = stylesheet
            .rules
            .iter()
            .flat_map(|r| r.states.iter())
            .flat_map(|s| s.modifiers.iter())
            .collect();

        // Collect state modifiers from parsed
        let parsed_states: Vec<_> = parsed
            .rules
            .iter()
            .flat_map(|r| r.states.iter())
            .flat_map(|s| s.modifiers.iter())
            .collect();

        prop_assert_eq!(
            original_states,
            parsed_states,
            "State modifiers should be preserved through round-trip"
        );
    }

    /// Property: Round-trip preserves responsive blocks
    /// For any stylesheet with responsive blocks, all breakpoint names should be
    /// preserved through the parse-format-parse cycle.
    #[test]
    fn prop_roundtrip_preserves_responsive(
        stylesheet in stylesheet_strategy().prop_filter(
            "Has responsive blocks",
            |s| s.rules.iter().any(|r| !r.responsive.is_empty())
        )
    ) {
        let formatted = format_stylesheet(&stylesheet);
        let parsed = parse(&formatted).unwrap();

        // Collect breakpoint names from original
        let original_breakpoints: Vec<_> = stylesheet
            .rules
            .iter()
            .flat_map(|r| r.responsive.iter())
            .map(|resp| resp.breakpoint.as_str())
            .collect();

        // Collect breakpoint names from parsed
        let parsed_breakpoints: Vec<_> = parsed
            .rules
            .iter()
            .flat_map(|r| r.responsive.iter())
            .map(|resp| resp.breakpoint.as_str())
            .collect();

        prop_assert_eq!(
            original_breakpoints,
            parsed_breakpoints,
            "Breakpoint names should be preserved through round-trip"
        );
    }

    /// Property: Round-trip preserves rule count
    /// For any stylesheet, the number of rules should be preserved through
    /// the parse-format-parse cycle.
    #[test]
    fn prop_roundtrip_preserves_rule_count(stylesheet in stylesheet_strategy()) {
        let formatted = format_stylesheet(&stylesheet);
        let parsed = parse(&formatted).unwrap();

        prop_assert_eq!(
            stylesheet.rules.len(),
            parsed.rules.len(),
            "Rule count should be preserved through round-trip"
        );
    }
}
