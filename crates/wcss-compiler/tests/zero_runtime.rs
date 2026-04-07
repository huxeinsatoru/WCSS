// Feature: wcss-framework, Property 13: Zero Runtime Overhead
// **Validates: Requirements 7.5**

use proptest::prelude::*;
use wcss_compiler::ast::*;
use wcss_compiler::codegen::{generate_css, generate_typed_om_js};
use wcss_compiler::config::CompilerConfig;

/// Generate a valid class name.
fn class_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9-]{0,15}".prop_map(|s| s.to_string())
}

/// Generate a simple declaration.
fn declaration_strategy() -> impl Strategy<Value = Declaration> {
    (
        prop_oneof![
            Just("color".to_string()),
            Just("background".to_string()),
            Just("padding".to_string()),
            Just("margin".to_string()),
            Just("width".to_string()),
            Just("height".to_string()),
            Just("display".to_string()),
            Just("font-size".to_string()),
        ],
        prop_oneof![
            Just(Value::Literal("red".to_string())),
            Just(Value::Literal("blue".to_string())),
            Just(Value::Literal("block".to_string())),
            Just(Value::Literal("flex".to_string())),
            (0.0..1000.0).prop_map(|n| Value::Number(n, Some(Unit::Px))),
            (0.0..100.0).prop_map(|n| Value::Number(n, Some(Unit::Rem))),
        ],
    )
        .prop_map(|(prop, val)| Declaration {
            property: Property::Standard(prop),
            value: val,
            important: false,
            span: Span::empty(),
        })
}

/// Generate a rule.
fn rule_strategy() -> impl Strategy<Value = Rule> {
    (
        class_name_strategy(),
        prop::collection::vec(declaration_strategy(), 1..=5),
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
            span: Span::empty(),
        })
}

/// Generate a stylesheet.
fn stylesheet_strategy() -> impl Strategy<Value = StyleSheet> {
    prop::collection::vec(rule_strategy(), 1..=20).prop_map(|rules| StyleSheet {
        rules,
        at_rules: vec![],
        span: Span::empty(),
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property 13: Zero Runtime Overhead
    /// For any WCSS source compiled with Typed OM runtime disabled,
    /// the output SHALL contain only CSS with no JavaScript runtime code.
    ///
    /// Validates: Requirements 7.5
    #[test]
    fn prop_zero_runtime_no_javascript(stylesheet in stylesheet_strategy()) {
        // Generate CSS with Typed OM disabled (default)
        let mut config = CompilerConfig::default();
        config.typed_om = false;

        let css = generate_css(&stylesheet, &config);

        // Verify CSS is generated
        prop_assert!(!css.is_empty(), "CSS output should not be empty");

        // Verify CSS contains expected selectors
        prop_assert!(
            css.contains('.'),
            "CSS output should contain class selectors"
        );

        // Verify CSS does NOT contain JavaScript code
        prop_assert!(
            !css.contains("function"),
            "CSS output should not contain JavaScript functions"
        );
        prop_assert!(
            !css.contains("const "),
            "CSS output should not contain JavaScript const declarations"
        );
        prop_assert!(
            !css.contains("let "),
            "CSS output should not contain JavaScript let declarations"
        );
        prop_assert!(
            !css.contains("var "),
            "CSS output should not contain JavaScript var declarations"
        );
        prop_assert!(
            !css.contains("=>"),
            "CSS output should not contain JavaScript arrow functions"
        );
        prop_assert!(
            !css.contains("attributeStyleMap"),
            "CSS output should not contain Typed OM API references"
        );
        prop_assert!(
            !css.contains("CSS.px"),
            "CSS output should not contain CSS Typed OM unit functions"
        );
    }

    /// Property: Zero runtime produces only valid CSS
    /// For any stylesheet with Typed OM disabled, the output
    /// should be valid CSS with no JavaScript.
    #[test]
    fn prop_zero_runtime_produces_valid_css(stylesheet in stylesheet_strategy()) {
        let mut config = CompilerConfig::default();
        config.typed_om = false;
        config.minify = false;

        let css = generate_css(&stylesheet, &config);

        // Verify CSS syntax elements are present
        prop_assert!(css.contains('{'), "CSS should contain opening braces");
        prop_assert!(css.contains('}'), "CSS should contain closing braces");
        prop_assert!(css.contains(':'), "CSS should contain property-value separators");
        prop_assert!(css.contains(';'), "CSS should contain declaration terminators");

        // Verify balanced braces
        let open_braces = css.matches('{').count();
        let close_braces = css.matches('}').count();
        prop_assert_eq!(
            open_braces,
            close_braces,
            "CSS should have balanced braces"
        );
    }

    /// Property: Zero runtime includes all selectors
    /// For any stylesheet with Typed OM disabled, all class selectors
    /// should appear in the CSS output.
    #[test]
    fn prop_zero_runtime_includes_all_selectors(stylesheet in stylesheet_strategy()) {
        let mut config = CompilerConfig::default();
        config.typed_om = false;

        let css = generate_css(&stylesheet, &config);

        // Verify each class name appears in the CSS
        for rule in &stylesheet.rules {
            let class_selector = format!(".{}", rule.selector.class_name);
            prop_assert!(
                css.contains(&class_selector),
                "CSS should contain selector: {}",
                class_selector
            );
        }
    }

    /// Property: Zero runtime includes all properties
    /// For any stylesheet with Typed OM disabled, all property names
    /// should appear in the CSS output.
    #[test]
    fn prop_zero_runtime_includes_all_properties(stylesheet in stylesheet_strategy()) {
        let mut config = CompilerConfig::default();
        config.typed_om = false;

        let css = generate_css(&stylesheet, &config);

        // Verify each property appears in the CSS
        for rule in &stylesheet.rules {
            for decl in &rule.declarations {
                let prop_name = decl.property.name();
                prop_assert!(
                    css.contains(prop_name),
                    "CSS should contain property: {}",
                    prop_name
                );
            }
        }
    }

    /// Property: Zero runtime output is pure CSS
    /// The output should not contain any JavaScript-specific syntax.
    #[test]
    fn prop_zero_runtime_is_pure_css(stylesheet in stylesheet_strategy()) {
        let mut config = CompilerConfig::default();
        config.typed_om = false;

        let css = generate_css(&stylesheet, &config);

        // List of JavaScript-specific patterns that should NOT appear
        let js_patterns = vec![
            "export",
            "import",
            "class ",
            "extends",
            "constructor",
            "this.",
            "new ",
            "return ",
            "if (",
            "for (",
            "while (",
            "switch (",
            "case ",
            "break",
            "continue",
            "typeof",
            "instanceof",
            "async",
            "await",
            "Promise",
            "console.",
            "document.",
            "window.",
        ];

        for pattern in js_patterns {
            prop_assert!(
                !css.contains(pattern),
                "CSS output should not contain JavaScript pattern: '{}'",
                pattern
            );
        }
    }

    /// Property: Zero runtime is deterministic
    /// Generating CSS multiple times with Typed OM disabled
    /// should produce identical results.
    #[test]
    fn prop_zero_runtime_is_deterministic(stylesheet in stylesheet_strategy()) {
        let mut config = CompilerConfig::default();
        config.typed_om = false;

        let css1 = generate_css(&stylesheet, &config);
        let css2 = generate_css(&stylesheet, &config);

        prop_assert_eq!(
            css1,
            css2,
            "CSS generation with zero runtime should be deterministic"
        );
    }

    /// Property: Typed OM disabled vs enabled produces different output
    /// When Typed OM is disabled, the output should be different from
    /// when it's enabled (CSS-only vs CSS+JS).
    #[test]
    fn prop_typed_om_disabled_differs_from_enabled(stylesheet in stylesheet_strategy()) {
        // Generate CSS with Typed OM disabled
        let mut config_disabled = CompilerConfig::default();
        config_disabled.typed_om = false;
        let css_only = generate_css(&stylesheet, &config_disabled);

        // Generate JavaScript with Typed OM enabled
        let js_with_typed_om = generate_typed_om_js(&stylesheet);

        // Verify CSS doesn't contain Typed OM patterns
        prop_assert!(
            !css_only.contains("CSS.px") && !css_only.contains("CSS.rem") && !css_only.contains("attributeStyleMap"),
            "CSS-only output should not contain Typed OM patterns"
        );

        // Verify CSS is pure CSS (contains CSS syntax)
        prop_assert!(
            css_only.contains('.') && css_only.contains('{') && css_only.contains('}'),
            "CSS-only output should contain valid CSS syntax"
        );
        
        // Verify JS output is different from CSS output (if JS is generated)
        if !js_with_typed_om.is_empty() {
            prop_assert_ne!(
                &css_only,
                &js_with_typed_om,
                "CSS-only output should differ from Typed OM JavaScript output"
            );
        }
    }

    /// Property: Zero runtime with minification
    /// Even with minification enabled, the output should still be pure CSS
    /// with no JavaScript when Typed OM is disabled.
    #[test]
    fn prop_zero_runtime_with_minification(stylesheet in stylesheet_strategy()) {
        let mut config = CompilerConfig::default();
        config.typed_om = false;
        config.minify = true;

        let css = generate_css(&stylesheet, &config);

        // Verify CSS is generated
        prop_assert!(!css.is_empty(), "Minified CSS output should not be empty");

        // Verify no JavaScript patterns
        prop_assert!(
            !css.contains("function") && !css.contains("const "),
            "Minified CSS should not contain JavaScript"
        );

        // Verify CSS patterns are present
        prop_assert!(
            css.contains('.') && css.contains('{') && css.contains('}'),
            "Minified output should still be valid CSS"
        );
    }
}
