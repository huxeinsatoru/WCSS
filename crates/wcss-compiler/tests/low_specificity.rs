// Feature: wcss-framework, Property 18: Low Specificity CSS
// **Validates: Requirements 18.1, 18.4, 18.7**

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
        Just("width".to_string()),
        Just("height".to_string()),
        Just("display".to_string()),
        Just("font-size".to_string()),
        Just("font-family".to_string()),
        Just("border".to_string()),
        Just("border-color".to_string()),
        Just("opacity".to_string()),
        Just("transform".to_string()),
        Just("position".to_string()),
        Just("z-index".to_string()),
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
        Just(Value::Literal("block".to_string())),
        Just(Value::Literal("flex".to_string())),
        Just(Value::Literal("none".to_string())),
        Just(Value::Literal("auto".to_string())),
        (0.0..1000.0).prop_map(|n| Value::Number(n, Some(Unit::Px))),
        (0.0..100.0).prop_map(|n| Value::Number(n, Some(Unit::Rem))),
        (0.0..1.0).prop_map(|n| Value::Number(n, None)),
        "[0-9a-f]{6}".prop_map(|hex| Value::Color(Color::Hex(format!("#{}", hex)))),
    ]
}

/// Generate a single declaration (never with !important).
fn declaration_strategy() -> impl Strategy<Value = Declaration> {
    (property_strategy(), value_strategy()).prop_map(|(prop, val)| Declaration {
        property: Property::Standard(prop),
        value: val,
        important: false, // Always false for low specificity
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
        ],
        prop::collection::vec(declaration_strategy(), 1..=3),
    ).prop_map(|(breakpoint, declarations)| ResponsiveBlock {
        breakpoint,
        declarations,
        span: Span::empty(),
    })
}

/// Generate a rule with simple single-class selectors (no combinators).
fn rule_strategy() -> impl Strategy<Value = Rule> {
    (
        class_name_strategy(),
        prop::collection::vec(declaration_strategy(), 1..=5),
        prop::collection::vec(state_block_strategy(), 0..=2),
        prop::collection::vec(responsive_block_strategy(), 0..=2),
    ).prop_map(|(class_name, declarations, states, responsive)| Rule {
        selector: Selector {
            class_name,
            combinators: vec![], // No combinators = single class selector
            pseudo_elements: vec![],
            span: Span::empty(),
        },
        declarations,
        states,
        responsive,
        span: Span::empty(),
    })
}

/// Generate a stylesheet with multiple rules.
fn stylesheet_strategy() -> impl Strategy<Value = StyleSheet> {
    prop::collection::vec(rule_strategy(), 1..=20).prop_map(|rules| StyleSheet {
        rules,
        span: Span::empty(),
    })
}

/// Calculate CSS specificity for a selector.
/// Returns (a, b, c) where:
/// - a = number of ID selectors
/// - b = number of class selectors, attribute selectors, and pseudo-classes
/// - c = number of type selectors and pseudo-elements
///
/// For low specificity, we want (0, 1, 0) for single-class selectors.
fn calculate_specificity(selector: &str) -> (u32, u32, u32) {
    let mut ids = 0;
    let mut classes = 0;
    let mut types = 0;
    
    let mut chars = selector.chars().peekable();
    
    while let Some(ch) = chars.next() {
        match ch {
            '#' => {
                // ID selector
                ids += 1;
                // Skip the ID name
                while let Some(&next) = chars.peek() {
                    if next.is_alphanumeric() || next == '-' || next == '_' {
                        chars.next();
                    } else {
                        break;
                    }
                }
            }
            '.' => {
                // Class selector
                classes += 1;
                // Skip the class name
                while let Some(&next) = chars.peek() {
                    if next.is_alphanumeric() || next == '-' || next == '_' {
                        chars.next();
                    } else {
                        break;
                    }
                }
            }
            ':' => {
                // Pseudo-class or pseudo-element
                if chars.peek() == Some(&':') {
                    // Pseudo-element (::)
                    chars.next();
                    types += 1;
                } else {
                    // Pseudo-class (:)
                    classes += 1;
                }
                // Skip the pseudo name
                while let Some(&next) = chars.peek() {
                    if next.is_alphanumeric() || next == '-' {
                        chars.next();
                    } else {
                        break;
                    }
                }
            }
            '[' => {
                // Attribute selector
                classes += 1;
                // Skip until closing bracket
                while let Some(next) = chars.next() {
                    if next == ']' {
                        break;
                    }
                }
            }
            _ if ch.is_alphabetic() => {
                // Type selector (element name)
                // Only count if it's at the start or after a combinator
                types += 1;
                while let Some(&next) = chars.peek() {
                    if next.is_alphanumeric() || next == '-' {
                        chars.next();
                    } else {
                        break;
                    }
                }
            }
            _ => {}
        }
    }
    
    (ids, classes, types)
}

/// Extract all selectors from CSS (text before opening braces).
fn extract_selectors(css: &str) -> Vec<String> {
    let mut selectors = Vec::new();
    let mut current = String::new();
    let mut in_media_query = false;
    
    for ch in css.chars() {
        if ch == '@' {
            in_media_query = true;
        }
        
        if ch == '{' {
            let selector = current.trim().to_string();
            if !selector.is_empty() && !selector.starts_with('@') {
                if !in_media_query || selector.starts_with('.') {
                    selectors.push(selector);
                }
            }
            current.clear();
        } else if ch == '}' {
            if in_media_query {
                in_media_query = false;
            }
            current.clear();
        } else {
            current.push(ch);
        }
    }
    
    selectors
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    /// Property 18: Low Specificity CSS
    /// For any WCSS source, the generated CSS SHALL use single-class selectors
    /// (low specificity) to ensure JavaScript inline styles and library-driven
    /// styles can override WCSS styles.
    /// 
    /// Validates: Requirements 18.1, 18.4, 18.7
    #[test]
    fn prop_low_specificity_single_class_selectors(stylesheet in stylesheet_strategy()) {
        let config = CompilerConfig::default();
        let css = generate_css(&stylesheet, &config);
        
        let selectors = extract_selectors(&css);
        
        // Verify all selectors have low specificity
        for selector in &selectors {
            let (ids, classes, types) = calculate_specificity(selector);
            
            // Single-class selectors should have specificity (0, 1, 0)
            // With pseudo-classes, it becomes (0, 2+, 0) which is still low
            prop_assert_eq!(ids, 0,
                "Selector should not contain ID selectors: {}\nSpecificity: ({}, {}, {})",
                selector, ids, classes, types);
            
            prop_assert_eq!(types, 0,
                "Selector should not contain type selectors: {}\nSpecificity: ({}, {}, {})",
                selector, ids, classes, types);
            
            prop_assert!(classes >= 1,
                "Selector should contain at least one class: {}\nSpecificity: ({}, {}, {})",
                selector, ids, classes, types);
            
            // Verify selector starts with a class (not a type selector)
            prop_assert!(selector.trim().starts_with('.'),
                "Selector should start with a class: {}", selector);
        }
    }
    
    /// Property: No !important declarations
    /// For any WCSS source, the generated CSS SHALL not contain !important
    /// declarations to ensure JavaScript can override styles.
    /// 
    /// Validates: Requirement 18.4
    #[test]
    fn prop_no_important_declarations(stylesheet in stylesheet_strategy()) {
        let config = CompilerConfig::default();
        let css = generate_css(&stylesheet, &config);
        
        // Verify no !important in generated CSS
        prop_assert!(!css.contains("!important"),
            "CSS should not contain !important declarations\nGenerated CSS:\n{}", css);
    }
    
    /// Property: Single class selector specificity
    /// For any rule with a single class selector, the specificity SHALL be (0, 1, 0)
    /// for base rules and (0, 1+n, 0) for rules with n pseudo-classes.
    /// 
    /// Validates: Requirements 18.1, 18.7
    #[test]
    fn prop_single_class_specificity_calculation(
        class_name in class_name_strategy(),
        declarations in prop::collection::vec(declaration_strategy(), 1..=3)
    ) {
        let stylesheet = StyleSheet {
            rules: vec![Rule {
                selector: Selector {
                    class_name: class_name.clone(),
                    combinators: vec![],
                    pseudo_elements: vec![],
                    span: Span::empty(),
                },
                declarations,
                states: vec![],
                responsive: vec![],
                span: Span::empty(),
            }],
            span: Span::empty(),
        };
        
        let config = CompilerConfig::default();
        let css = generate_css(&stylesheet, &config);
        
        let selectors = extract_selectors(&css);
        
        // Should have exactly one selector (the base class)
        prop_assert!(!selectors.is_empty(), "Should have at least one selector");
        
        // Check the base selector
        let base_selector = format!(".{}", class_name);
        let matching_selectors: Vec<_> = selectors.iter()
            .filter(|s| s.starts_with(&base_selector) && !s.contains(':'))
            .collect();
        
        if !matching_selectors.is_empty() {
            let selector = matching_selectors[0];
            let (ids, classes, types) = calculate_specificity(selector);
            
            // Base selector should have specificity (0, 1, 0)
            prop_assert_eq!(ids, 0, "Base selector should have 0 ID selectors");
            prop_assert_eq!(classes, 1, "Base selector should have exactly 1 class selector");
            prop_assert_eq!(types, 0, "Base selector should have 0 type selectors");
        }
    }
    
    /// Property: Pseudo-class selectors maintain low specificity
    /// For any rule with pseudo-class selectors, the specificity SHALL remain low
    /// (0, 1+n, 0) where n is the number of pseudo-classes.
    /// 
    /// Validates: Requirements 18.1, 18.7
    #[test]
    fn prop_pseudo_class_low_specificity(
        class_name in class_name_strategy(),
        state_blocks in prop::collection::vec(state_block_strategy(), 1..=3)
    ) {
        let stylesheet = StyleSheet {
            rules: vec![Rule {
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
            }],
            span: Span::empty(),
        };
        
        let config = CompilerConfig::default();
        let css = generate_css(&stylesheet, &config);
        
        let selectors = extract_selectors(&css);
        
        // Verify all pseudo-class selectors have low specificity
        for selector in &selectors {
            let (ids, classes, types) = calculate_specificity(selector);
            
            // Should have no IDs or type selectors
            prop_assert_eq!(ids, 0,
                "Pseudo-class selector should have 0 ID selectors: {}", selector);
            prop_assert_eq!(types, 0,
                "Pseudo-class selector should have 0 type selectors: {}", selector);
            
            // Should have at least 1 class (the base class)
            prop_assert!(classes >= 1,
                "Pseudo-class selector should have at least 1 class: {}", selector);
        }
    }
    
    /// Property: Media query selectors maintain low specificity
    /// For any responsive block, the selectors inside media queries SHALL
    /// maintain low specificity (0, 1, 0).
    /// 
    /// Validates: Requirements 18.1, 18.7
    #[test]
    fn prop_media_query_low_specificity(
        class_name in class_name_strategy(),
        responsive_blocks in prop::collection::vec(responsive_block_strategy(), 1..=3)
    ) {
        let stylesheet = StyleSheet {
            rules: vec![Rule {
                selector: Selector {
                    class_name: class_name.clone(),
                    combinators: vec![],
                    pseudo_elements: vec![],
                    span: Span::empty(),
                },
                declarations: vec![],
                states: vec![],
                responsive: responsive_blocks,
                span: Span::empty(),
            }],
            span: Span::empty(),
        };
        
        let config = CompilerConfig::default();
        let css = generate_css(&stylesheet, &config);
        
        // Extract selectors from within media queries
        let mut in_media = false;
        let mut media_selectors = Vec::new();
        let mut current = String::new();
        
        for ch in css.chars() {
            if current.ends_with("@media") {
                in_media = true;
            }
            
            if ch == '{' && in_media {
                let selector = current.trim().to_string();
                if selector.starts_with('.') {
                    media_selectors.push(selector);
                }
                current.clear();
            } else if ch == '}' {
                current.clear();
            } else {
                current.push(ch);
            }
        }
        
        // Verify media query selectors have low specificity
        for selector in &media_selectors {
            let (ids, classes, types) = calculate_specificity(selector);
            
            prop_assert_eq!(ids, 0,
                "Media query selector should have 0 ID selectors: {}", selector);
            prop_assert_eq!(types, 0,
                "Media query selector should have 0 type selectors: {}", selector);
            prop_assert!(classes >= 1,
                "Media query selector should have at least 1 class: {}", selector);
        }
    }
    
    /// Property: All selectors start with class selector
    /// For any generated CSS, all selectors SHALL start with a class selector (.)
    /// to maintain low specificity and avoid type selectors.
    /// 
    /// Validates: Requirements 18.1, 18.7
    #[test]
    fn prop_all_selectors_start_with_class(stylesheet in stylesheet_strategy()) {
        let config = CompilerConfig::default();
        let css = generate_css(&stylesheet, &config);
        
        let selectors = extract_selectors(&css);
        
        for selector in &selectors {
            let trimmed = selector.trim();
            prop_assert!(trimmed.starts_with('.'),
                "Selector should start with a class (.): {}", selector);
            
            // Verify it doesn't start with a type selector
            prop_assert!(!trimmed.chars().next().unwrap().is_alphabetic() || trimmed.starts_with('.'),
                "Selector should not start with a type selector: {}", selector);
        }
    }
    
    /// Property: No descendant or child combinators increase specificity
    /// For any rule without combinators, the generated CSS SHALL not contain
    /// descendant (space) or child (>) combinators that would increase specificity.
    /// 
    /// Validates: Requirements 18.1, 18.7
    #[test]
    fn prop_no_combinator_specificity_increase(
        class_name in class_name_strategy(),
        declarations in prop::collection::vec(declaration_strategy(), 1..=3)
    ) {
        let stylesheet = StyleSheet {
            rules: vec![Rule {
                selector: Selector {
                    class_name: class_name.clone(),
                    combinators: vec![], // No combinators
                    pseudo_elements: vec![],
                    span: Span::empty(),
                },
                declarations,
                states: vec![],
                responsive: vec![],
                span: Span::empty(),
            }],
            span: Span::empty(),
        };
        
        let config = CompilerConfig::default();
        let css = generate_css(&stylesheet, &config);
        
        let selectors = extract_selectors(&css);
        
        for selector in &selectors {
            // Should not contain descendant combinator (space between selectors)
            // Check for patterns like ".class1 .class2"
            let parts: Vec<&str> = selector.split_whitespace().collect();
            prop_assert!(parts.len() <= 1 || !parts[0].starts_with('.') || !parts[1].starts_with('.'),
                "Selector should not contain descendant combinator: {}", selector);
            
            // Should not contain child combinator
            prop_assert!(!selector.contains(" > "),
                "Selector should not contain child combinator: {}", selector);
            
            // Should not contain adjacent sibling combinator
            prop_assert!(!selector.contains(" + "),
                "Selector should not contain adjacent sibling combinator: {}", selector);
            
            // Should not contain general sibling combinator
            prop_assert!(!selector.contains(" ~ "),
                "Selector should not contain general sibling combinator: {}", selector);
        }
    }
}
