// Feature: wcss-framework, Property 6: Tree Shaking Correctness
// Validates: Requirements 2.2

use proptest::prelude::*;
use std::collections::HashSet;
use wcss_compiler::ast::*;
use wcss_compiler::config::CompilerConfig;
use wcss_compiler::optimizer::optimize;

/// Generate a valid CSS property name.
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
        Just("border".to_string()),
        Just("opacity".to_string()),
    ]
}

/// Generate a valid CSS value.
fn value_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        // Colors
        "#[0-9a-f]{6}".prop_map(|s| s.to_string()),
        "rgb\\([0-9]{1,3}, [0-9]{1,3}, [0-9]{1,3}\\)".prop_map(|s| s.to_string()),
        // Sizes
        "[0-9]{1,3}(px|rem|em|%)".prop_map(|s| s.to_string()),
        // Keywords
        Just("block".to_string()),
        Just("flex".to_string()),
        Just("none".to_string()),
        Just("auto".to_string()),
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
        value: Value::Literal(val),
        important: false,
        span: Span::empty(),
    })
}

/// Generate a rule with a given class name.
fn rule_with_class_strategy(class_name: String) -> impl Strategy<Value = Rule> {
    prop::collection::vec(declaration_strategy(), 1..=5).prop_map(move |declarations| Rule {
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
        span: Span::empty(),
    })
}

/// Generate a stylesheet with marked used and unused rules.
/// Returns (stylesheet, used_classes, unused_classes).
fn stylesheet_with_usage_strategy() -> impl Strategy<Value = (StyleSheet, Vec<String>, Vec<String>)> {
    // Generate 5-20 used classes and 5-20 unused classes
    (
        prop::collection::vec(class_name_strategy(), 5..=20),
        prop::collection::vec(class_name_strategy(), 5..=20),
    )
        .prop_flat_map(|(used_classes, unused_classes)| {
            // Ensure no overlap between used and unused
            let used_set: HashSet<_> = used_classes.iter().cloned().collect();
            let unused_classes: Vec<_> = unused_classes
                .into_iter()
                .filter(|c| !used_set.contains(c))
                .collect();
            
            // Generate one rule per class
            let all_classes: Vec<_> = used_classes.iter().chain(unused_classes.iter()).cloned().collect();
            
            // Generate rules for all classes
            let rule_strategies: Vec<_> = all_classes
                .iter()
                .map(|c| rule_with_class_strategy(c.clone()))
                .collect();
            
            // Use a tuple strategy to generate all rules at once
            prop::collection::vec(
                prop::strategy::Union::new(rule_strategies),
                all_classes.len()..=all_classes.len()
            )
                .prop_map(move |mut rules| {
                    // Ensure we have exactly one rule per class by deduplicating
                    // and filling in missing ones
                    let mut seen_classes = HashSet::new();
                    rules.retain(|r| seen_classes.insert(r.selector.class_name.clone()));
                    
                    // Add missing classes with simple rules
                    for class in &all_classes {
                        if !seen_classes.contains(class) {
                            rules.push(Rule {
                                selector: Selector {
                                    class_name: class.clone(),
                                    kind: SelectorKind::Class,
                                    combinators: vec![],
                                    pseudo_elements: vec![],
                                    pseudo_classes: vec![],
                                    attributes: vec![],
                                    span: Span::empty(),
                                },
                                selectors: vec![],
                                declarations: vec![Declaration {
                                    property: Property::Standard("color".to_string()),
                                    value: Value::Literal("red".to_string()),
                                    important: false,
                                    span: Span::empty(),
                                }],
                                states: vec![],
                                responsive: vec![],
                                nested_rules: vec![],
                                span: Span::empty(),
                            });
                        }
                    }
                    
                    let stylesheet = StyleSheet {
                        rules,
                        at_rules: vec![],
                        span: Span::empty(),
                    };
                    (stylesheet, used_classes.clone(), unused_classes.clone())
                })
        })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    /// Property: Tree shaking removes only unused rules
    /// For any stylesheet with marked used and unused rules,
    /// tree shaking should remove only the unused rules while preserving all used rules.
    #[test]
    fn prop_tree_shaking_removes_only_unused(
        (stylesheet, used_classes, unused_classes) in stylesheet_with_usage_strategy()
    ) {
        // Create config with tree shaking enabled
        let mut config = CompilerConfig::default();
        config.tree_shaking = true;
        config.deduplicate = false; // Disable deduplication to test tree shaking in isolation
        config.used_classes = used_classes.clone();
        
        // Count original rules by class
        let original_used_count = stylesheet
            .rules
            .iter()
            .filter(|r| used_classes.contains(&r.selector.class_name))
            .count();
        
        // Apply optimization (which includes tree shaking)
        let optimized = optimize(stylesheet.clone(), &config);
        
        // Collect class names from optimized stylesheet
        let optimized_classes: HashSet<_> = optimized
            .rules
            .iter()
            .map(|r| r.selector.class_name.as_str())
            .collect();
        
        // Verify all used classes are preserved
        for used_class in &used_classes {
            prop_assert!(
                optimized_classes.contains(used_class.as_str()),
                "Used class '{}' should be preserved after tree shaking",
                used_class
            );
        }
        
        // Verify all unused classes are removed
        for unused_class in &unused_classes {
            prop_assert!(
                !optimized_classes.contains(unused_class.as_str()),
                "Unused class '{}' should be removed after tree shaking",
                unused_class
            );
        }
        
        // Verify the count matches (all used rules preserved, all unused removed)
        prop_assert_eq!(
            optimized.rules.len(),
            original_used_count,
            "Optimized stylesheet should contain exactly {} rules (all used rules)",
            original_used_count
        );
        
        // Verify no unused rules remain
        let unused_in_optimized = optimized
            .rules
            .iter()
            .filter(|r| unused_classes.contains(&r.selector.class_name))
            .count();
        prop_assert_eq!(
            unused_in_optimized,
            0,
            "No unused rules should remain after tree shaking"
        );
    }
    
    /// Property: Tree shaking preserves rule content
    /// For any used rule, tree shaking should preserve its declarations exactly.
    #[test]
    fn prop_tree_shaking_preserves_rule_content(
        (stylesheet, used_classes, _unused_classes) in stylesheet_with_usage_strategy()
    ) {
        // Create config with tree shaking enabled
        let mut config = CompilerConfig::default();
        config.tree_shaking = true;
        config.deduplicate = false; // Disable deduplication to test tree shaking in isolation
        config.used_classes = used_classes.clone();
        
        // Build a list of original used rules
        let original_used_rules: Vec<_> = stylesheet
            .rules
            .iter()
            .filter(|r| used_classes.contains(&r.selector.class_name))
            .cloned()
            .collect();
        
        // Apply optimization
        let optimized = optimize(stylesheet, &config);
        
        // Verify all optimized rules match original used rules
        prop_assert_eq!(
            optimized.rules.len(),
            original_used_rules.len(),
            "Optimized should have same number of rules as original used rules"
        );
        
        // Verify each optimized rule exists in original used rules
        for opt_rule in &optimized.rules {
            let matching_original = original_used_rules.iter().find(|orig| {
                orig.selector.class_name == opt_rule.selector.class_name
                    && orig.declarations.len() == opt_rule.declarations.len()
                    && orig.declarations.iter().zip(opt_rule.declarations.iter()).all(|(o, n)| {
                        o.property.name() == n.property.name() && o.value == n.value
                    })
            });
            
            prop_assert!(
                matching_original.is_some(),
                "Optimized rule for class '{}' should match an original used rule",
                opt_rule.selector.class_name
            );
        }
    }
    
    /// Property: Tree shaking with empty used list keeps all rules
    /// When tree shaking is enabled but no classes are marked as used,
    /// tree shaking is skipped and all rules are preserved (safe default).
    #[test]
    fn prop_tree_shaking_empty_used_keeps_all(
        (stylesheet, _used_classes, _unused_classes) in stylesheet_with_usage_strategy()
    ) {
        // Create config with tree shaking enabled but empty used_classes
        let mut config = CompilerConfig::default();
        config.tree_shaking = true;
        config.deduplicate = false; // Disable deduplication to preserve all rules
        config.used_classes = vec![];
        
        let original_count = stylesheet.rules.len();
        
        // Apply optimization
        let optimized = optimize(stylesheet, &config);
        
        // Verify all rules are preserved (tree shaking is skipped when used_classes is empty)
        prop_assert_eq!(
            optimized.rules.len(),
            original_count,
            "Tree shaking with empty used list should preserve all {} rules (safe default)",
            original_count
        );
    }
    
    /// Property: Tree shaking disabled preserves all rules
    /// When tree shaking is disabled, all rules should be preserved.
    #[test]
    fn prop_tree_shaking_disabled_preserves_all(
        (stylesheet, _used_classes, _unused_classes) in stylesheet_with_usage_strategy()
    ) {
        // Create config with tree shaking disabled
        let mut config = CompilerConfig::default();
        config.tree_shaking = false;
        config.deduplicate = false; // Also disable deduplication to preserve all rules
        config.used_classes = vec![]; // Even with empty used list
        
        let original_count = stylesheet.rules.len();
        
        // Apply optimization
        let optimized = optimize(stylesheet, &config);
        
        // Verify all rules are preserved
        prop_assert_eq!(
            optimized.rules.len(),
            original_count,
            "Tree shaking disabled should preserve all {} rules",
            original_count
        );
    }
}
