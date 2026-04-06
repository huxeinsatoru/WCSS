// Feature: wcss-framework, Property 15: State Selector Generation
// **Validates: Requirements 10.1, 10.2, 10.3, 10.4**

use proptest::prelude::*;
use wcss_compiler::ast::*;
use wcss_compiler::codegen::generate_css;
use wcss_compiler::config::CompilerConfig;

/// Generate a valid class name.
fn class_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9-]{0,15}".prop_map(|s| s.to_string())
}

/// Generate a valid CSS property name.
fn property_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("color".to_string()),
        Just("background".to_string()),
        Just("background-color".to_string()),
        Just("padding".to_string()),
        Just("margin".to_string()),
        Just("border".to_string()),
        Just("border-color".to_string()),
        Just("opacity".to_string()),
        Just("transform".to_string()),
        Just("box-shadow".to_string()),
        Just("outline".to_string()),
        Just("text-decoration".to_string()),
    ]
}

/// Generate a valid CSS value.
fn value_strategy() -> impl Strategy<Value = Value> {
    prop_oneof![
        Just(Value::Literal("red".to_string())),
        Just(Value::Literal("blue".to_string())),
        Just(Value::Literal("green".to_string())),
        Just(Value::Literal("white".to_string())),
        Just(Value::Literal("black".to_string())),
        Just(Value::Literal("transparent".to_string())),
        Just(Value::Literal("none".to_string())),
        Just(Value::Literal("underline".to_string())),
        (0.0..100.0).prop_map(|n| Value::Number(n, Some(Unit::Px))),
        (0.0..1.0).prop_map(|n| Value::Number(n, None)),
        "[0-9a-f]{6}".prop_map(|hex| Value::Color(Color::Hex(format!("#{}", hex)))),
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

/// Generate a state block with one or more modifiers (for state combinations).
fn state_block_strategy() -> impl Strategy<Value = StateBlock> {
    (
        prop::collection::vec(state_modifier_strategy(), 1..=3),
        prop::collection::vec(declaration_strategy(), 1..=4),
    ).prop_map(|(modifiers, declarations)| StateBlock {
        modifiers,
        declarations,
        span: Span::empty(),
    })
}

/// Generate a rule with state blocks.
fn rule_with_states_strategy() -> impl Strategy<Value = Rule> {
    (
        class_name_strategy(),
        prop::collection::vec(declaration_strategy(), 0..=3),
        prop::collection::vec(state_block_strategy(), 1..=5),
    ).prop_map(|(class_name, declarations, states)| Rule {
        selector: Selector {
            class_name,
            combinators: vec![],
            pseudo_elements: vec![],
            span: Span::empty(),
        },
        declarations,
        states,
        responsive: vec![],
        span: Span::empty(),
    })
}

/// Validate that a pseudo-class selector is correctly formatted.
fn validate_pseudo_class_syntax(selector: &str) -> bool {
    // Should start with a dot (class selector)
    if !selector.starts_with('.') {
        return false;
    }
    
    // Should contain at least one colon for pseudo-class
    if !selector.contains(':') {
        return false;
    }
    
    // Should not have spaces between class and pseudo-class
    let parts: Vec<&str> = selector.split('{').collect();
    if parts.is_empty() {
        return false;
    }
    
    let selector_part = parts[0].trim();
    
    // Check that there are no spaces between class name and pseudo-classes
    if selector_part.contains(" :") {
        return false;
    }
    
    true
}

/// Extract all selectors from CSS (text before opening braces).
fn extract_selectors(css: &str) -> Vec<String> {
    let mut selectors = Vec::new();
    let mut current = String::new();
    
    for ch in css.chars() {
        if ch == '{' {
            let selector = current.trim().to_string();
            if !selector.is_empty() && !selector.starts_with('@') {
                selectors.push(selector);
            }
            current.clear();
        } else if ch == '}' {
            current.clear();
        } else {
            current.push(ch);
        }
    }
    
    selectors
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    /// Property 15: State Selector Generation
    /// For any state selector declaration (hover, focus, active, etc.),
    /// the compiler SHALL generate the appropriate CSS pseudo-class selector.
    /// 
    /// Validates: Requirements 10.1, 10.2
    #[test]
    fn prop_state_selectors_generate_pseudo_classes(rule in rule_with_states_strategy()) {
        let stylesheet = StyleSheet {
            rules: vec![rule.clone()],
            span: Span::empty(),
        };
        
        let config = CompilerConfig::default();
        let css = generate_css(&stylesheet, &config);
        
        // Verify each state block generates a pseudo-class selector
        for state_block in &rule.states {
            for modifier in &state_block.modifiers {
                let pseudo = modifier.as_css_pseudo();
                prop_assert!(css.contains(&pseudo),
                    "CSS should contain pseudo-class: {}\nGenerated CSS:\n{}", pseudo, css);
            }
            
            // Verify the selector format is correct
            let class_name = &rule.selector.class_name;
            let pseudo_selector = format!(".{}", class_name);
            prop_assert!(css.contains(&pseudo_selector),
                "CSS should contain class selector: {}", pseudo_selector);
        }
    }
    
    /// Property: State combinations generate combined pseudo-class selectors
    /// For any state block with multiple modifiers, the compiler SHALL generate
    /// a selector with all pseudo-classes combined.
    /// 
    /// Validates: Requirement 10.3
    #[test]
    fn prop_state_combinations_generate_correctly(
        class_name in class_name_strategy(),
        modifiers in prop::collection::vec(state_modifier_strategy(), 2..=3),
        declarations in prop::collection::vec(declaration_strategy(), 1..=3)
    ) {
        let state_block = StateBlock {
            modifiers: modifiers.clone(),
            declarations,
            span: Span::empty(),
        };
        
        let rule = Rule {
            selector: Selector {
                class_name: class_name.clone(),
                combinators: vec![],
                pseudo_elements: vec![],
                span: Span::empty(),
            },
            declarations: vec![],
            states: vec![state_block],
            responsive: vec![],
            span: Span::empty(),
        };
        
        let stylesheet = StyleSheet {
            rules: vec![rule],
            span: Span::empty(),
        };
        
        let config = CompilerConfig::default();
        let css = generate_css(&stylesheet, &config);
        
        // Verify all modifiers appear in the CSS
        for modifier in &modifiers {
            let pseudo = modifier.as_css_pseudo();
            prop_assert!(css.contains(&pseudo),
                "CSS should contain pseudo-class: {}\nGenerated CSS:\n{}", pseudo, css);
        }
        
        // Verify the combined selector exists (all pseudo-classes together)
        let combined_pseudo: String = modifiers.iter()
            .map(|m| m.as_css_pseudo())
            .collect();
        let full_selector = format!(".{}{}", class_name, combined_pseudo);
        prop_assert!(css.contains(&full_selector),
            "CSS should contain combined selector: {}\nGenerated CSS:\n{}", full_selector, css);
    }
    
    /// Property: Each state modifier generates correct pseudo-class syntax
    /// For any individual state modifier, the compiler SHALL generate the
    /// correct CSS pseudo-class syntax.
    /// 
    /// Validates: Requirement 10.2
    #[test]
    fn prop_individual_state_modifiers_correct_syntax(
        class_name in class_name_strategy(),
        modifier in state_modifier_strategy(),
        declarations in prop::collection::vec(declaration_strategy(), 1..=2)
    ) {
        let state_block = StateBlock {
            modifiers: vec![modifier.clone()],
            declarations,
            span: Span::empty(),
        };
        
        let rule = Rule {
            selector: Selector {
                class_name: class_name.clone(),
                combinators: vec![],
                pseudo_elements: vec![],
                span: Span::empty(),
            },
            declarations: vec![],
            states: vec![state_block],
            responsive: vec![],
            span: Span::empty(),
        };
        
        let stylesheet = StyleSheet {
            rules: vec![rule],
            span: Span::empty(),
        };
        
        let config = CompilerConfig::default();
        let css = generate_css(&stylesheet, &config);
        
        // Verify the correct pseudo-class is generated
        let expected_pseudo = modifier.as_css_pseudo();
        let expected_selector = format!(".{}{}", class_name, expected_pseudo);
        
        prop_assert!(css.contains(&expected_selector),
            "CSS should contain selector: {}\nGenerated CSS:\n{}", expected_selector, css);
        
        // Verify the selector syntax is valid
        let selectors = extract_selectors(&css);
        let matching_selectors: Vec<_> = selectors.iter()
            .filter(|s| s.contains(&class_name) && s.contains(&expected_pseudo))
            .collect();
        
        prop_assert!(!matching_selectors.is_empty(),
            "Should find at least one matching selector");
        
        for selector in matching_selectors {
            prop_assert!(validate_pseudo_class_syntax(selector),
                "Selector should have valid pseudo-class syntax: {}", selector);
        }
    }
    
    /// Property: State declarations are included in generated CSS
    /// For any state block with declarations, all declarations SHALL appear
    /// in the generated CSS within the pseudo-class rule.
    /// 
    /// Validates: Requirements 10.1, 10.2
    #[test]
    fn prop_state_declarations_included(
        class_name in class_name_strategy(),
        state_blocks in prop::collection::vec(state_block_strategy(), 1..=4)
    ) {
        let rule = Rule {
            selector: Selector {
                class_name: class_name.clone(),
                combinators: vec![],
                pseudo_elements: vec![],
                span: Span::empty(),
            },
            declarations: vec![],
            states: state_blocks.clone(),
            responsive: vec![],
            span: Span::empty(),
        };
        
        let stylesheet = StyleSheet {
            rules: vec![rule],
            span: Span::empty(),
        };
        
        let config = CompilerConfig::default();
        let css = generate_css(&stylesheet, &config);
        
        // Verify all state block declarations appear in CSS
        for state_block in &state_blocks {
            for declaration in &state_block.declarations {
                let prop_name = declaration.property.name();
                prop_assert!(css.contains(prop_name),
                    "CSS should contain property from state block: {}\nGenerated CSS:\n{}", 
                    prop_name, css);
            }
        }
    }
    
    /// Property: Multiple state blocks generate separate rules
    /// For any rule with multiple state blocks, each state block SHALL generate
    /// a separate CSS rule with its own pseudo-class selector.
    /// 
    /// Validates: Requirements 10.1, 10.3
    #[test]
    fn prop_multiple_state_blocks_generate_separate_rules(
        class_name in class_name_strategy(),
        state_blocks in prop::collection::vec(state_block_strategy(), 2..=5)
    ) {
        let rule = Rule {
            selector: Selector {
                class_name: class_name.clone(),
                combinators: vec![],
                pseudo_elements: vec![],
                span: Span::empty(),
            },
            declarations: vec![],
            states: state_blocks.clone(),
            responsive: vec![],
            span: Span::empty(),
        };
        
        let stylesheet = StyleSheet {
            rules: vec![rule],
            span: Span::empty(),
        };
        
        let config = CompilerConfig::default();
        let css = generate_css(&stylesheet, &config);
        
        // Count the number of rule blocks (opening braces)
        let rule_count = css.matches('{').count();
        
        // Should have at least as many rules as state blocks
        prop_assert!(rule_count >= state_blocks.len(),
            "CSS should have at least {} rules, found {}\nGenerated CSS:\n{}", 
            state_blocks.len(), rule_count, css);
        
        // Verify each state block's selector appears
        for state_block in &state_blocks {
            let combined_pseudo: String = state_block.modifiers.iter()
                .map(|m| m.as_css_pseudo())
                .collect();
            let selector = format!(".{}{}", class_name, combined_pseudo);
            prop_assert!(css.contains(&selector),
                "CSS should contain selector: {}\nGenerated CSS:\n{}", selector, css);
        }
    }
    
    /// Property: State selectors work with base declarations
    /// For any rule with both base declarations and state blocks,
    /// the compiler SHALL generate both the base rule and state rules.
    /// 
    /// Validates: Requirements 10.1, 10.2
    #[test]
    fn prop_state_selectors_with_base_declarations(
        class_name in class_name_strategy(),
        base_declarations in prop::collection::vec(declaration_strategy(), 1..=3),
        state_blocks in prop::collection::vec(state_block_strategy(), 1..=3)
    ) {
        let rule = Rule {
            selector: Selector {
                class_name: class_name.clone(),
                combinators: vec![],
                pseudo_elements: vec![],
                span: Span::empty(),
            },
            declarations: base_declarations.clone(),
            states: state_blocks.clone(),
            responsive: vec![],
            span: Span::empty(),
        };
        
        let stylesheet = StyleSheet {
            rules: vec![rule],
            span: Span::empty(),
        };
        
        let config = CompilerConfig::default();
        let css = generate_css(&stylesheet, &config);
        
        // Verify base selector exists
        let base_selector = format!(".{}", class_name);
        prop_assert!(css.contains(&base_selector),
            "CSS should contain base selector: {}", base_selector);
        
        // Verify base declarations are present
        for declaration in &base_declarations {
            let prop_name = declaration.property.name();
            prop_assert!(css.contains(prop_name),
                "CSS should contain base property: {}", prop_name);
        }
        
        // Verify state selectors exist
        for state_block in &state_blocks {
            for modifier in &state_block.modifiers {
                let pseudo = modifier.as_css_pseudo();
                prop_assert!(css.contains(&pseudo),
                    "CSS should contain pseudo-class: {}", pseudo);
            }
        }
    }
    
    /// Property: All standard state modifiers generate valid CSS
    /// For each standard state modifier (hover, focus, active, visited, disabled,
    /// checked, first-child, last-child), the compiler SHALL generate valid CSS.
    /// 
    /// Validates: Requirements 10.1, 10.2
    #[test]
    fn prop_all_standard_state_modifiers_valid(
        class_name in class_name_strategy(),
        declarations in prop::collection::vec(declaration_strategy(), 1..=2)
    ) {
        let all_modifiers = vec![
            StateModifier::Hover,
            StateModifier::Focus,
            StateModifier::Active,
            StateModifier::Visited,
            StateModifier::Disabled,
            StateModifier::Checked,
            StateModifier::FirstChild,
            StateModifier::LastChild,
        ];
        
        let state_blocks: Vec<StateBlock> = all_modifiers.iter().map(|modifier| {
            StateBlock {
                modifiers: vec![modifier.clone()],
                declarations: declarations.clone(),
                span: Span::empty(),
            }
        }).collect();
        
        let rule = Rule {
            selector: Selector {
                class_name: class_name.clone(),
                combinators: vec![],
                pseudo_elements: vec![],
                span: Span::empty(),
            },
            declarations: vec![],
            states: state_blocks,
            responsive: vec![],
            span: Span::empty(),
        };
        
        let stylesheet = StyleSheet {
            rules: vec![rule],
            span: Span::empty(),
        };
        
        let config = CompilerConfig::default();
        let css = generate_css(&stylesheet, &config);
        
        // Verify each standard modifier generates correct pseudo-class
        let expected_pseudos = vec![
            ":hover", ":focus", ":active", ":visited",
            ":disabled", ":checked", ":first-child", ":last-child"
        ];
        
        for pseudo in expected_pseudos {
            prop_assert!(css.contains(pseudo),
                "CSS should contain pseudo-class: {}\nGenerated CSS:\n{}", pseudo, css);
        }
    }
}
