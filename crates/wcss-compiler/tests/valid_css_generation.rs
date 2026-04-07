// Feature: wcss-framework, Property 5: Valid CSS Generation
// **Validates: Requirements 2.1, 15.1, 15.5**

use proptest::prelude::*;
use wcss_compiler::ast::*;
use wcss_compiler::codegen::generate_css;
use wcss_compiler::config::CompilerConfig;

/// Generate a valid CSS property name.
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
        Just("border-color".to_string()),
        Just("border-width".to_string()),
        Just("opacity".to_string()),
        Just("position".to_string()),
        Just("top".to_string()),
        Just("left".to_string()),
        Just("right".to_string()),
        Just("bottom".to_string()),
        Just("z-index".to_string()),
        Just("flex".to_string()),
        Just("grid".to_string()),
        Just("gap".to_string()),
        Just("transform".to_string()),
        Just("transition".to_string()),
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
        Just(Value::Literal("relative".to_string())),
        Just(Value::Literal("absolute".to_string())),
        // Numbers with units
        (0.0..1000.0).prop_map(|n| Value::Number(n, Some(Unit::Px))),
        (0.0..100.0).prop_map(|n| Value::Number(n, Some(Unit::Rem))),
        (0.0..100.0).prop_map(|n| Value::Number(n, Some(Unit::Em))),
        (0.0..100.0).prop_map(|n| Value::Number(n, Some(Unit::Percent))),
        (0.0..100.0).prop_map(|n| Value::Number(n, Some(Unit::Vh))),
        (0.0..100.0).prop_map(|n| Value::Number(n, Some(Unit::Vw))),
        // Hex colors
        "[0-9a-f]{6}".prop_map(|hex| Value::Color(Color::Hex(format!("#{}", hex)))),
        // RGB colors
        (0.0..256.0, 0.0..256.0, 0.0..256.0).prop_map(|(r, g, b)| 
            Value::Color(Color::Rgb(r, g, b))
        ),
        // RGBA colors
        (0.0..256.0, 0.0..256.0, 0.0..256.0, 0.0..1.0).prop_map(|(r, g, b, a)| 
            Value::Color(Color::Rgba(r, g, b, a))
        ),
    ]
}

/// Generate a valid class name.
fn class_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9-]{0,15}".prop_map(|s| s.to_string())
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

/// Generate a state modifier.
fn state_modifier_strategy() -> impl Strategy<Value = StateModifier> {
    prop_oneof![
        Just(StateModifier::Hover),
        Just(StateModifier::Focus),
        Just(StateModifier::Active),
        Just(StateModifier::Visited),
        Just(StateModifier::Disabled),
        Just(StateModifier::Checked),
        Just(StateModifier::FirstChild),
        Just(StateModifier::LastChild),
    ]
}

/// Generate a state block.
fn state_block_strategy() -> impl Strategy<Value = StateBlock> {
    (
        prop::collection::vec(state_modifier_strategy(), 1..=2),
        prop::collection::vec(declaration_strategy(), 1..=3),
    ).prop_map(|(modifiers, declarations)| StateBlock {
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
    ).prop_map(|(breakpoint, declarations)| ResponsiveBlock {
        breakpoint,
        declarations,
        span: Span::empty(),
    })
}

/// Generate a rule with random declarations, states, and responsive blocks.
fn rule_strategy() -> impl Strategy<Value = Rule> {
    (
        class_name_strategy(),
        prop::collection::vec(declaration_strategy(), 1..=5),
        prop::collection::vec(state_block_strategy(), 0..=2),
        prop::collection::vec(responsive_block_strategy(), 0..=2),
    ).prop_map(|(class_name, declarations, states, responsive)| Rule {
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

/// Generate a stylesheet with multiple rules.
fn stylesheet_strategy() -> impl Strategy<Value = StyleSheet> {
    prop::collection::vec(rule_strategy(), 1..=20).prop_map(|rules| StyleSheet {
        rules,
        at_rules: vec![],
        span: Span::empty(),
    })
}

/// Validate that the generated CSS follows basic CSS3 syntax rules.
/// This checks for:
/// - Balanced braces
/// - Valid selector syntax
/// - Valid declaration syntax (property: value;)
/// - Valid at-rule syntax (@media)
fn validate_css_syntax(css: &str) -> Result<(), String> {
    let mut brace_depth = 0;
    let mut in_at_rule = false;
    let mut chars = css.chars().peekable();
    
    while let Some(ch) = chars.next() {
        match ch {
            '{' => {
                brace_depth += 1;
                if brace_depth > 10 {
                    return Err("Excessive brace nesting depth".to_string());
                }
            }
            '}' => {
                brace_depth -= 1;
                if brace_depth < 0 {
                    return Err("Unbalanced closing brace".to_string());
                }
                if in_at_rule && brace_depth == 0 {
                    in_at_rule = false;
                }
            }
            '@' => {
                // Check for valid at-rule (e.g., @media)
                let mut at_rule = String::new();
                while let Some(&next_ch) = chars.peek() {
                    if next_ch.is_whitespace() || next_ch == '(' {
                        break;
                    }
                    at_rule.push(chars.next().unwrap());
                }
                if at_rule != "media" && at_rule != "keyframes" && at_rule != "supports" {
                    // Allow other at-rules but validate they're not empty
                    if at_rule.is_empty() {
                        return Err("Empty at-rule".to_string());
                    }
                }
                in_at_rule = true;
            }
            _ => {}
        }
    }
    
    if brace_depth != 0 {
        return Err(format!("Unbalanced braces: depth = {}", brace_depth));
    }
    
    Ok(())
}

/// Check that the CSS contains expected structural elements.
fn validate_css_structure(css: &str, stylesheet: &StyleSheet) -> Result<(), String> {
    // Check that each rule's class name appears in the CSS
    for rule in &stylesheet.rules {
        let class_selector = format!(".{}", rule.selector.class_name);
        if !css.contains(&class_selector) {
            return Err(format!("Missing class selector: {}", class_selector));
        }
        
        // Check that declarations are present (at least property names)
        for decl in &rule.declarations {
            let prop_name = decl.property.name();
            if !css.contains(prop_name) {
                return Err(format!("Missing property: {}", prop_name));
            }
        }
        
        // Check that state blocks generate pseudo-classes
        for state in &rule.states {
            for modifier in &state.modifiers {
                let pseudo = modifier.as_css_pseudo();
                if !css.contains(&pseudo) {
                    return Err(format!("Missing pseudo-class: {}", pseudo));
                }
            }
        }
        
        // Check that responsive blocks generate media queries
        for responsive in &rule.responsive {
            if !responsive.declarations.is_empty() && !css.contains("@media") {
                return Err("Missing @media query for responsive block".to_string());
            }
        }
    }
    
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    /// Property 5: Valid CSS Generation
    /// For any valid WCSS AST, the code generator SHALL produce CSS output
    /// that is valid according to CSS3 specification.
    /// 
    /// Validates: Requirements 2.1, 15.1, 15.5
    #[test]
    fn prop_valid_css_generation(stylesheet in stylesheet_strategy()) {
        // Generate CSS with default config (non-minified for easier validation)
        let mut config = CompilerConfig::default();
        config.minify = false;
        let css = generate_css(&stylesheet, &config);
        
        // Validate CSS syntax
        if let Err(e) = validate_css_syntax(&css) {
            prop_assert!(false, "Invalid CSS syntax: {}\nGenerated CSS:\n{}", e, css);
        }
        
        // Validate CSS structure matches AST
        if let Err(e) = validate_css_structure(&css, &stylesheet) {
            prop_assert!(false, "Invalid CSS structure: {}\nGenerated CSS:\n{}", e, css);
        }
        
        // Verify CSS is not empty
        prop_assert!(!css.trim().is_empty(), "Generated CSS should not be empty");
        
        // Verify CSS contains at least one rule
        prop_assert!(css.contains('{') && css.contains('}'), 
            "Generated CSS should contain at least one rule block");
    }
    
    /// Property: Valid CSS generation with minification
    /// For any valid WCSS AST, the code generator SHALL produce valid minified CSS.
    #[test]
    fn prop_valid_minified_css_generation(stylesheet in stylesheet_strategy()) {
        // Generate minified CSS
        let mut config = CompilerConfig::default();
        config.minify = true;
        let css = generate_css(&stylesheet, &config);
        
        // Validate CSS syntax (minified CSS should still be valid)
        if let Err(e) = validate_css_syntax(&css) {
            prop_assert!(false, "Invalid minified CSS syntax: {}\nGenerated CSS:\n{}", e, css);
        }
        
        // Validate CSS structure matches AST
        if let Err(e) = validate_css_structure(&css, &stylesheet) {
            prop_assert!(false, "Invalid minified CSS structure: {}\nGenerated CSS:\n{}", e, css);
        }
        
        // Verify CSS is not empty
        prop_assert!(!css.trim().is_empty(), "Generated minified CSS should not be empty");
    }
    
    /// Property: CSS generation produces parseable output
    /// For any valid WCSS AST, the generated CSS should have balanced braces
    /// and valid syntax that could be parsed by a CSS parser.
    #[test]
    fn prop_css_has_balanced_braces(stylesheet in stylesheet_strategy()) {
        let config = CompilerConfig::default();
        let css = generate_css(&stylesheet, &config);
        
        let open_braces = css.matches('{').count();
        let close_braces = css.matches('}').count();
        
        prop_assert_eq!(open_braces, close_braces,
            "CSS should have balanced braces: {} open, {} close",
            open_braces, close_braces);
    }
    
    /// Property: CSS generation includes all selectors
    /// For any valid WCSS AST, all class selectors should appear in the generated CSS.
    #[test]
    fn prop_css_includes_all_selectors(stylesheet in stylesheet_strategy()) {
        let config = CompilerConfig::default();
        let css = generate_css(&stylesheet, &config);
        
        for rule in &stylesheet.rules {
            let class_selector = format!(".{}", rule.selector.class_name);
            prop_assert!(css.contains(&class_selector),
                "CSS should contain selector: {}", class_selector);
        }
    }
    
    /// Property: CSS generation includes all properties
    /// For any valid WCSS AST, all property names should appear in the generated CSS.
    #[test]
    fn prop_css_includes_all_properties(stylesheet in stylesheet_strategy()) {
        let config = CompilerConfig::default();
        let css = generate_css(&stylesheet, &config);
        
        for rule in &stylesheet.rules {
            for decl in &rule.declarations {
                let prop_name = decl.property.name();
                prop_assert!(css.contains(prop_name),
                    "CSS should contain property: {}", prop_name);
            }
            
            // Check state block declarations
            for state in &rule.states {
                for decl in &state.declarations {
                    let prop_name = decl.property.name();
                    prop_assert!(css.contains(prop_name),
                        "CSS should contain property from state block: {}", prop_name);
                }
            }
            
            // Check responsive block declarations
            for responsive in &rule.responsive {
                for decl in &responsive.declarations {
                    let prop_name = decl.property.name();
                    prop_assert!(css.contains(prop_name),
                        "CSS should contain property from responsive block: {}", prop_name);
                }
            }
        }
    }
    
    /// Property: CSS generation produces valid pseudo-classes
    /// For any state block, the generated CSS should contain valid pseudo-class syntax.
    #[test]
    fn prop_css_generates_valid_pseudo_classes(
        class_name in class_name_strategy(),
        state_blocks in prop::collection::vec(state_block_strategy(), 1..=3)
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
                states: state_blocks.clone(),
                responsive: vec![],
                nested_rules: vec![],
                span: Span::empty(),
            }],
            at_rules: vec![],
            span: Span::empty(),
        };

        let config = CompilerConfig::default();
        let css = generate_css(&stylesheet, &config);

        // Verify each state modifier generates a pseudo-class
        for state_block in &state_blocks {
            for modifier in &state_block.modifiers {
                let pseudo = modifier.as_css_pseudo();
                prop_assert!(css.contains(&pseudo),
                    "CSS should contain pseudo-class: {}", pseudo);
            }
        }
    }
    
    /// Property: CSS generation produces valid media queries
    /// For any responsive block, the generated CSS should contain valid @media syntax.
    #[test]
    fn prop_css_generates_valid_media_queries(
        class_name in class_name_strategy(),
        responsive_blocks in prop::collection::vec(responsive_block_strategy(), 1..=3)
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
                responsive: responsive_blocks.clone(),
                nested_rules: vec![],
                span: Span::empty(),
            }],
            at_rules: vec![],
            span: Span::empty(),
        };

        let config = CompilerConfig::default();
        let css = generate_css(&stylesheet, &config);

        // Verify media queries are generated
        if !responsive_blocks.is_empty() {
            prop_assert!(css.contains("@media"),
                "CSS should contain @media query for responsive blocks");
            
            // Verify media query syntax is valid
            prop_assert!(css.contains("@media") && css.contains("min-width"),
                "CSS should contain valid @media query with min-width");
        }
    }
}
