// Feature: euis-framework, Property 2: Parser Syntax Coverage
// Validates: Requirements 1.3, 1.4, 1.5, 1.6

use proptest::prelude::*;
use euis_compiler::ast::*;
use euis_compiler::parser::parse;

// Generators for Euis syntax features

/// Generate valid class names
fn class_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9-]{0,10}".prop_map(|s| s.to_string())
}

/// Generate valid CSS property names
fn property_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("color".to_string()),
        Just("background".to_string()),
        Just("padding".to_string()),
        Just("margin".to_string()),
        Just("width".to_string()),
        Just("height".to_string()),
        Just("display".to_string()),
        Just("flex-direction".to_string()),
        Just("justify-content".to_string()),
        Just("align-items".to_string()),
        Just("gap".to_string()),
        Just("border-radius".to_string()),
    ]
}

/// Generate design token references (Requirement 1.4)
fn token_ref_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("$colors.primary".to_string()),
        Just("$colors.secondary".to_string()),
        Just("$spacing.sm".to_string()),
        Just("$spacing.md".to_string()),
        Just("$spacing.lg".to_string()),
        Just("$typography.font-sans".to_string()),
        Just("$typography.text-base".to_string()),
    ]
}

/// Generate literal values
fn literal_value_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("red".to_string()),
        Just("blue".to_string()),
        Just("white".to_string()),
        Just("black".to_string()),
        Just("#ff0000".to_string()),
        Just("#00ff00".to_string()),
        Just("10px".to_string()),
        Just("1rem".to_string()),
        Just("100%".to_string()),
        Just("auto".to_string()),
        Just("flex".to_string()),
        Just("block".to_string()),
        Just("center".to_string()),
    ]
}

/// Generate values (tokens or literals)
fn value_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        token_ref_strategy(),
        literal_value_strategy(),
    ]
}

/// Generate state modifiers (Requirement 1.6)
fn state_modifier_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("hover".to_string()),
        Just("focus".to_string()),
        Just("active".to_string()),
        Just("visited".to_string()),
        Just("disabled".to_string()),
        Just("checked".to_string()),
        Just("first-child".to_string()),
        Just("last-child".to_string()),
    ]
}

/// Generate breakpoint names (Requirement 1.5)
fn breakpoint_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("sm".to_string()),
        Just("md".to_string()),
        Just("lg".to_string()),
        Just("xl".to_string()),
    ]
}

/// Generate shorthand patterns (Requirement 1.3)
fn shorthand_pattern_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        // Centering shorthand
        Just("center: true".to_string()),
        // Stacking shorthand
        Just("stack: vertical".to_string()),
        Just("stack: horizontal".to_string()),
        // Spacing shorthand
        Just("spacing: $spacing.md".to_string()),
    ]
}

/// Generate a single declaration
fn declaration_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        // Regular property-value pairs
        (property_strategy(), value_strategy())
            .prop_map(|(prop, val)| format!("  {prop}: {val};")),
        // Shorthand patterns (Requirement 1.3)
        shorthand_pattern_strategy()
            .prop_map(|s| format!("  {s};")),
    ]
}

/// Generate a state block (Requirement 1.6)
fn state_block_strategy() -> impl Strategy<Value = String> {
    (
        state_modifier_strategy(),
        prop::collection::vec(declaration_strategy(), 1..3),
    )
        .prop_map(|(state, decls)| {
            let decls_str = decls.join("\n");
            format!("  &:{state} {{\n{decls_str}\n  }}")
        })
}

/// Generate a responsive block (Requirement 1.5)
fn responsive_block_strategy() -> impl Strategy<Value = String> {
    (
        breakpoint_strategy(),
        prop::collection::vec(declaration_strategy(), 1..3),
    )
        .prop_map(|(breakpoint, decls)| {
            let decls_str = decls.join("\n");
            format!("  @{breakpoint} {{\n{decls_str}\n  }}")
        })
}

/// Generate a complete Euis rule with all syntax features
fn euis_rule_strategy() -> impl Strategy<Value = String> {
    (
        class_name_strategy(),
        prop::collection::vec(declaration_strategy(), 1..4),
        prop::option::of(state_block_strategy()),
        prop::option::of(responsive_block_strategy()),
    )
        .prop_map(|(class, decls, state, responsive)| {
            let mut rule = format!(".{class} {{\n");
            rule.push_str(&decls.join("\n"));
            rule.push('\n');
            
            if let Some(state_block) = state {
                rule.push('\n');
                rule.push_str(&state_block);
                rule.push('\n');
            }
            
            if let Some(resp_block) = responsive {
                rule.push('\n');
                rule.push_str(&resp_block);
                rule.push('\n');
            }
            
            rule.push_str("}\n");
            rule
        })
}

// Property Tests

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property 2: Parser Syntax Coverage
    /// For any valid Euis syntax feature, the parser SHALL produce a correct AST representation.
    /// Validates: Requirements 1.3, 1.4, 1.5, 1.6
    #[test]
    fn prop_parser_handles_all_syntax_features(
        euis in euis_rule_strategy()
    ) {
        // Parse the generated Euis
        let result = parse(&euis);
        
        // Parser should succeed
        prop_assert!(result.is_ok(), "Parser failed on valid Euis: {}\nError: {:?}", euis, result.err());
        
        let stylesheet = result.unwrap();
        
        // Should have at least one rule
        prop_assert!(!stylesheet.rules.is_empty(), "Parser produced empty stylesheet for: {}", euis);
        
        let rule = &stylesheet.rules[0];
        
        // Should have a valid selector
        prop_assert!(!rule.selector.class_name.is_empty(), "Parser produced empty class name");
        
        // Should have declarations
        prop_assert!(!rule.declarations.is_empty(), "Parser produced no declarations for: {}", euis);
    }

    /// Test that design token references are parsed correctly (Requirement 1.4)
    #[test]
    fn prop_parser_handles_token_references(
        class in class_name_strategy(),
        prop in property_strategy(),
        token in token_ref_strategy()
    ) {
        let euis = format!(".{class} {{\n  {prop}: {token};\n}}\n");
        
        let result = parse(&euis);
        prop_assert!(result.is_ok(), "Parser failed on token reference: {}", euis);
        
        let stylesheet = result.unwrap();
        let rule = &stylesheet.rules[0];
        
        // Should have a declaration with a token value
        prop_assert!(!rule.declarations.is_empty());
        
        let decl = &rule.declarations[0];
        prop_assert_eq!(decl.property.name(), prop.as_str());
        
        // Verify it's a token reference
        match &decl.value {
            Value::Token(token_ref) => {
                // Token should have valid category and name
                prop_assert!(!token_ref.name.is_empty(), "Token name is empty");
            }
            _ => prop_assert!(false, "Expected Token value, got: {:?}", decl.value),
        }
    }

    /// Test that state selectors are parsed correctly (Requirement 1.6)
    #[test]
    fn prop_parser_handles_state_selectors(
        class in class_name_strategy(),
        state in state_modifier_strategy(),
        prop in property_strategy(),
        val in literal_value_strategy()
    ) {
        let euis = format!(".{class} {{\n  &:{state} {{\n    {prop}: {val};\n  }}\n}}\n");
        
        let result = parse(&euis);
        prop_assert!(result.is_ok(), "Parser failed on state selector: {}", euis);
        
        let stylesheet = result.unwrap();
        let rule = &stylesheet.rules[0];
        
        // Should have a state block
        prop_assert!(!rule.states.is_empty(), "Parser produced no state blocks for: {}", euis);
        
        let state_block = &rule.states[0];
        
        // State block should have modifiers
        prop_assert!(!state_block.modifiers.is_empty(), "State block has no modifiers");
        
        // State block should have declarations
        prop_assert!(!state_block.declarations.is_empty(), "State block has no declarations");
    }

    /// Test that responsive blocks are parsed correctly (Requirement 1.5)
    #[test]
    fn prop_parser_handles_responsive_blocks(
        class in class_name_strategy(),
        breakpoint in breakpoint_strategy(),
        prop in property_strategy(),
        val in literal_value_strategy()
    ) {
        let euis = format!(".{class} {{\n  @{breakpoint} {{\n    {prop}: {val};\n  }}\n}}\n");
        
        let result = parse(&euis);
        prop_assert!(result.is_ok(), "Parser failed on responsive block: {}", euis);
        
        let stylesheet = result.unwrap();
        let rule = &stylesheet.rules[0];
        
        // Should have a responsive block
        prop_assert!(!rule.responsive.is_empty(), "Parser produced no responsive blocks for: {}", euis);
        
        let resp_block = &rule.responsive[0];
        
        // Responsive block should have breakpoint name
        prop_assert_eq!(&resp_block.breakpoint, &breakpoint);
        
        // Responsive block should have declarations
        prop_assert!(!resp_block.declarations.is_empty(), "Responsive block has no declarations");
    }

    /// Test that shorthand patterns are parsed (Requirement 1.3)
    /// Note: Shorthand expansion happens in the parser, so we verify the parser accepts them
    #[test]
    fn prop_parser_handles_shorthand_syntax(
        class in class_name_strategy(),
        shorthand in shorthand_pattern_strategy()
    ) {
        let euis = format!(".{class} {{\n  {shorthand};\n}}\n");
        
        let result = parse(&euis);
        
        // Parser should either:
        // 1. Successfully parse and expand the shorthand, OR
        // 2. Parse it as a regular property-value pair
        // Either way, it should not fail
        prop_assert!(result.is_ok(), "Parser failed on shorthand: {}", euis);
        
        let stylesheet = result.unwrap();
        prop_assert!(!stylesheet.rules.is_empty(), "Parser produced empty stylesheet");
    }
}
