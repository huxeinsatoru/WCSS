// Feature: wcss-framework, Property 16: Formatting Consistency
// **Validates: Requirements 13.2**

use proptest::prelude::*;
use wcss_compiler::ast::*;
use wcss_compiler::formatter::format_stylesheet;

/// Generate a valid class name.
fn class_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9-]{0,15}".prop_map(|s| s.to_string())
}

/// Generate a valid property name.
fn property_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("color".to_string()),
        Just("background".to_string()),
        Just("padding".to_string()),
        Just("margin".to_string()),
        Just("width".to_string()),
        Just("height".to_string()),
        Just("display".to_string()),
        Just("font-size".to_string()),
    ]
}

/// Generate a valid CSS value.
fn value_strategy() -> impl Strategy<Value = Value> {
    prop_oneof![
        Just(Value::Literal("red".to_string())),
        Just(Value::Literal("blue".to_string())),
        Just(Value::Literal("block".to_string())),
        Just(Value::Literal("flex".to_string())),
        (0.0..1000.0).prop_map(|n| Value::Number(n, Some(Unit::Px))),
        (0.0..100.0).prop_map(|n| Value::Number(n, Some(Unit::Rem))),
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
                combinators: vec![],
                pseudo_elements: vec![],
                span: Span::empty(),
            },
            declarations,
            states,
            responsive,
            span: Span::empty(),
        })
}

/// Generate a stylesheet.
fn stylesheet_strategy() -> impl Strategy<Value = StyleSheet> {
    prop::collection::vec(rule_strategy(), 1..=10).prop_map(|rules| StyleSheet {
        rules,
        span: Span::empty(),
    })
}

/// Check if a string uses consistent indentation (2 spaces per level).
fn has_consistent_indentation(source: &str) -> bool {
    for line in source.lines() {
        if line.is_empty() {
            continue;
        }

        // Count leading spaces
        let leading_spaces = line.chars().take_while(|&c| c == ' ').count();

        // Indentation should be a multiple of 2
        if leading_spaces % 2 != 0 {
            return false;
        }
    }
    true
}

/// Check if declarations have consistent spacing (property: value;).
fn has_consistent_declaration_spacing(source: &str) -> bool {
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.contains(':') && !trimmed.starts_with('@') && !trimmed.starts_with('&') {
            // Should have format: "property: value;"
            if let Some(colon_pos) = trimmed.find(':') {
                // Check for space after colon
                if colon_pos + 1 < trimmed.len() {
                    let after_colon = &trimmed[colon_pos + 1..colon_pos + 2];
                    if after_colon != " " {
                        return false;
                    }
                }
            }
        }
    }
    true
}

/// Check if braces have consistent spacing.
fn has_consistent_brace_spacing(source: &str) -> bool {
    for line in source.lines() {
        let trimmed = line.trim();

        // Opening brace should be preceded by a space
        if trimmed.contains(" {") || trimmed == "{" {
            // Valid
        } else if trimmed.contains('{') && !trimmed.starts_with('{') {
            // Opening brace without preceding space (invalid)
            return false;
        }

        // Closing brace should be on its own line or at end
        if trimmed.contains('}') && trimmed != "}" && !trimmed.ends_with('}') {
            // Closing brace not at end of line (invalid)
            return false;
        }
    }
    true
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property 16: Formatting Consistency
    /// For any WCSS AST, formatting SHALL produce output with consistent
    /// indentation and spacing according to defined formatting rules.
    ///
    /// Validates: Requirements 13.2
    #[test]
    fn prop_formatting_has_consistent_indentation(stylesheet in stylesheet_strategy()) {
        let formatted = format_stylesheet(&stylesheet);

        prop_assert!(
            has_consistent_indentation(&formatted),
            "Formatted output should have consistent indentation (2 spaces per level).\nFormatted:\n{}",
            formatted
        );
    }

    /// Property: Formatting has consistent declaration spacing
    /// For any AST, formatted declarations should have consistent spacing
    /// around colons (property: value;).
    #[test]
    fn prop_formatting_has_consistent_declaration_spacing(stylesheet in stylesheet_strategy()) {
        let formatted = format_stylesheet(&stylesheet);

        prop_assert!(
            has_consistent_declaration_spacing(&formatted),
            "Formatted output should have consistent declaration spacing (property: value;).\nFormatted:\n{}",
            formatted
        );
    }

    /// Property: Formatting has consistent brace spacing
    /// For any AST, formatted output should have consistent spacing around braces.
    #[test]
    fn prop_formatting_has_consistent_brace_spacing(stylesheet in stylesheet_strategy()) {
        let formatted = format_stylesheet(&stylesheet);

        prop_assert!(
            has_consistent_brace_spacing(&formatted),
            "Formatted output should have consistent brace spacing.\nFormatted:\n{}",
            formatted
        );
    }

    /// Property: Formatting uses newlines consistently
    /// For any AST with multiple rules, formatted output should separate
    /// rules with newlines.
    #[test]
    fn prop_formatting_uses_newlines_consistently(
        stylesheet in stylesheet_strategy().prop_filter("Has multiple rules", |s| s.rules.len() > 1)
    ) {
        let formatted = format_stylesheet(&stylesheet);

        // Count closing braces (should match rule count)
        let closing_braces = formatted.matches('}').count();
        let rule_count = stylesheet.rules.len();

        // Each rule should have at least one closing brace
        prop_assert!(
            closing_braces >= rule_count,
            "Formatted output should have at least one closing brace per rule"
        );

        // Rules should be separated by newlines
        let lines: Vec<_> = formatted.lines().collect();
        prop_assert!(
            lines.len() > rule_count,
            "Formatted output should have multiple lines for multiple rules"
        );
    }

    /// Property: Formatting is deterministic
    /// Formatting the same AST multiple times should produce identical output.
    #[test]
    fn prop_formatting_is_deterministic(stylesheet in stylesheet_strategy()) {
        let formatted1 = format_stylesheet(&stylesheet);
        let formatted2 = format_stylesheet(&stylesheet);

        prop_assert_eq!(
            formatted1,
            formatted2,
            "Formatting should be deterministic"
        );
    }

    /// Property: Formatting produces non-empty output
    /// For any non-empty AST, formatting should produce non-empty output.
    #[test]
    fn prop_formatting_produces_nonempty_output(stylesheet in stylesheet_strategy()) {
        let formatted = format_stylesheet(&stylesheet);

        prop_assert!(
            !formatted.trim().is_empty(),
            "Formatted output should not be empty for non-empty AST"
        );
    }

    /// Property: Formatting includes all class names
    /// For any AST, all class names should appear in the formatted output.
    #[test]
    fn prop_formatting_includes_all_class_names(stylesheet in stylesheet_strategy()) {
        let formatted = format_stylesheet(&stylesheet);

        for rule in &stylesheet.rules {
            let class_name = &rule.selector.class_name;
            prop_assert!(
                formatted.contains(&format!(".{}", class_name)),
                "Formatted output should include class name: {}",
                class_name
            );
        }
    }

    /// Property: Formatting includes all properties
    /// For any AST, all property names should appear in the formatted output.
    #[test]
    fn prop_formatting_includes_all_properties(stylesheet in stylesheet_strategy()) {
        let formatted = format_stylesheet(&stylesheet);

        for rule in &stylesheet.rules {
            for decl in &rule.declarations {
                let prop_name = decl.property.name();
                prop_assert!(
                    formatted.contains(prop_name),
                    "Formatted output should include property: {}",
                    prop_name
                );
            }
        }
    }

    /// Property: Formatting preserves state modifiers
    /// For any AST with state blocks, all state modifiers should appear
    /// in the formatted output.
    #[test]
    fn prop_formatting_preserves_state_modifiers(
        stylesheet in stylesheet_strategy().prop_filter(
            "Has state blocks",
            |s| s.rules.iter().any(|r| !r.states.is_empty())
        )
    ) {
        let formatted = format_stylesheet(&stylesheet);

        for rule in &stylesheet.rules {
            for state in &rule.states {
                for modifier in &state.modifiers {
                    let state_name = match modifier {
                        StateModifier::Hover => "hover",
                        StateModifier::Focus => "focus",
                        StateModifier::Active => "active",
                        StateModifier::Visited => "visited",
                        StateModifier::Disabled => "disabled",
                        StateModifier::Checked => "checked",
                        StateModifier::FirstChild => "first-child",
                        StateModifier::LastChild => "last-child",
                        StateModifier::Custom(s) => s.as_str(),
                    };
                    prop_assert!(
                        formatted.contains(&format!(":{}", state_name)),
                        "Formatted output should include state modifier: :{}",
                        state_name
                    );
                }
            }
        }
    }

    /// Property: Formatting preserves responsive blocks
    /// For any AST with responsive blocks, all breakpoint names should appear
    /// in the formatted output.
    #[test]
    fn prop_formatting_preserves_responsive_blocks(
        stylesheet in stylesheet_strategy().prop_filter(
            "Has responsive blocks",
            |s| s.rules.iter().any(|r| !r.responsive.is_empty())
        )
    ) {
        let formatted = format_stylesheet(&stylesheet);

        for rule in &stylesheet.rules {
            for responsive in &rule.responsive {
                let breakpoint = &responsive.breakpoint;
                prop_assert!(
                    formatted.contains(&format!("@{}", breakpoint)),
                    "Formatted output should include breakpoint: @{}",
                    breakpoint
                );
            }
        }
    }

    /// Property: Formatting uses consistent line endings
    /// For any AST, formatted output should use consistent line endings (LF).
    #[test]
    fn prop_formatting_uses_consistent_line_endings(stylesheet in stylesheet_strategy()) {
        let formatted = format_stylesheet(&stylesheet);

        // Should not contain CRLF
        prop_assert!(
            !formatted.contains("\r\n"),
            "Formatted output should use LF line endings, not CRLF"
        );

        // Should contain LF for multi-line output
        if stylesheet.rules.len() > 1 || stylesheet.rules.iter().any(|r| r.declarations.len() > 1) {
            prop_assert!(
                formatted.contains('\n'),
                "Formatted output should contain newlines for multi-line content"
            );
        }
    }

    /// Property: Formatting indents nested blocks
    /// For any AST with state or responsive blocks, nested content should be indented.
    #[test]
    fn prop_formatting_indents_nested_blocks(
        stylesheet in stylesheet_strategy().prop_filter(
            "Has nested blocks",
            |s| s.rules.iter().any(|r| !r.states.is_empty() || !r.responsive.is_empty())
        )
    ) {
        let formatted = format_stylesheet(&stylesheet);

        // Check that nested blocks have increased indentation
        let lines: Vec<_> = formatted.lines().collect();
        let mut found_nested_indentation = false;

        for (i, line) in lines.iter().enumerate() {
            // Look for state or responsive block markers
            if line.trim().starts_with('&') || line.trim().starts_with('@') {
                // Next lines should have more indentation
                if i + 1 < lines.len() {
                    let current_indent = line.chars().take_while(|&c| c == ' ').count();
                    let next_indent = lines[i + 1].chars().take_while(|&c| c == ' ').count();
                    if next_indent > current_indent {
                        found_nested_indentation = true;
                        break;
                    }
                }
            }
        }

        prop_assert!(
            found_nested_indentation,
            "Formatted output should indent nested blocks.\nFormatted:\n{}",
            formatted
        );
    }
}
