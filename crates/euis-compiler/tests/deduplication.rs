// Feature: euis-framework, Property 7: Deduplication Correctness
// Validates: Requirements 2.4

use proptest::prelude::*;
use std::collections::{HashMap, HashSet};
use euis_compiler::ast::*;
use euis_compiler::config::CompilerConfig;
use euis_compiler::optimizer::optimize;

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

/// Generate a list of declarations (1-5 declarations).
fn declarations_strategy() -> impl Strategy<Value = Vec<Declaration>> {
    prop::collection::vec(declaration_strategy(), 1..=5)
}

/// Generate a rule with a given class name and declarations.
fn rule_strategy(class_name: String, declarations: Vec<Declaration>) -> Rule {
    Rule {
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
    }
}

/// Hash declarations to identify duplicates.
fn hash_declarations(declarations: &[Declaration]) -> String {
    declarations
        .iter()
        .map(|d| format!("{}:{:?}", d.property.name(), d.value))
        .collect::<Vec<_>>()
        .join(";")
}

/// Generate a stylesheet with duplicate rules.
/// Returns (stylesheet, duplicate_groups) where duplicate_groups maps
/// declaration hash -> list of class names with those declarations.
fn stylesheet_with_duplicates_strategy() -> impl Strategy<Value = (StyleSheet, HashMap<String, Vec<String>>)> {
    // Generate 3-10 unique declaration sets
    prop::collection::vec(declarations_strategy(), 3..=10).prop_flat_map(|decl_sets| {
        // For each declaration set, generate 2-5 class names
        let class_strategies: Vec<_> = decl_sets
            .iter()
            .map(|_| prop::collection::vec(class_name_strategy(), 2..=5))
            .collect();
        
        // Combine all strategies
        class_strategies.into_iter()
            .collect::<Vec<_>>()
            .prop_map(move |class_groups| {
                let mut rules = Vec::new();
                let mut duplicate_groups: HashMap<String, Vec<String>> = HashMap::new();
                
                // Create rules for each declaration set with multiple class names
                for (decl_set, class_names) in decl_sets.iter().zip(class_groups.iter()) {
                    let decl_hash = hash_declarations(decl_set);
                    
                    // Ensure unique class names within this group
                    let unique_classes: Vec<_> = class_names.iter()
                        .collect::<HashSet<_>>()
                        .into_iter()
                        .cloned()
                        .collect();
                    
                    for class_name in &unique_classes {
                        rules.push(rule_strategy(class_name.clone(), decl_set.clone()));
                    }
                    
                    duplicate_groups.insert(decl_hash, unique_classes);
                }
                
                let stylesheet = StyleSheet {
                    rules,
                    at_rules: vec![],
                    span: Span::empty(),
                };
                
                (stylesheet, duplicate_groups)
            })
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    /// Property: Deduplication combines selectors for identical rules
    /// For any stylesheet with duplicate style rules (same declarations),
    /// deduplication should preserve the styling effect by keeping all selectors.
    #[test]
    fn prop_deduplication_preserves_all_selectors(
        (stylesheet, _duplicate_groups) in stylesheet_with_duplicates_strategy()
    ) {
        // Create config with deduplication enabled (tree shaking disabled)
        let mut config = CompilerConfig::default();
        config.tree_shaking = false;
        
        // Collect all original class names
        let original_classes: HashSet<_> = stylesheet
            .rules
            .iter()
            .map(|r| r.selector.class_name.as_str())
            .collect();
        
        // Apply optimization (which includes deduplication)
        let optimized = optimize(stylesheet.clone(), &config);
        
        // Collect all optimized class names (including merged selectors)
        let mut optimized_classes: HashSet<&str> = HashSet::new();
        for rule in &optimized.rules {
            optimized_classes.insert(rule.selector.class_name.as_str());
            for sel in &rule.selectors {
                optimized_classes.insert(sel.class_name.as_str());
            }
        }
        
        // Verify all original class names are preserved
        for original_class in &original_classes {
            prop_assert!(
                optimized_classes.contains(original_class),
                "Class '{}' should be preserved after deduplication",
                original_class
            );
        }
        
        prop_assert_eq!(
            original_classes.len(),
            optimized_classes.len(),
            "All {} unique class names should be preserved",
            original_classes.len()
        );
    }
    
    /// Property: Deduplication reduces or maintains rule count
    /// For any stylesheet, deduplication should reduce the number of rules
    /// (when duplicates exist) or maintain the same count (when no duplicates).
    #[test]
    fn prop_deduplication_reduces_or_maintains_count(
        (stylesheet, _duplicate_groups) in stylesheet_with_duplicates_strategy()
    ) {
        let mut config = CompilerConfig::default();
        config.tree_shaking = false;
        
        let original_count = stylesheet.rules.len();
        let optimized = optimize(stylesheet, &config);
        let optimized_count = optimized.rules.len();
        
        prop_assert!(
            optimized_count <= original_count,
            "Deduplication should reduce or maintain rule count: {} -> {}",
            original_count,
            optimized_count
        );
    }
    
    /// Property: Deduplication preserves declaration content
    /// For any rule in the optimized stylesheet, its declarations should match
    /// the declarations of at least one rule in the original stylesheet.
    #[test]
    fn prop_deduplication_preserves_declarations(
        (stylesheet, _duplicate_groups) in stylesheet_with_duplicates_strategy()
    ) {
        let mut config = CompilerConfig::default();
        config.tree_shaking = false;
        
        // Build a map of original declaration hashes
        let original_decl_hashes: HashSet<_> = stylesheet
            .rules
            .iter()
            .map(|r| hash_declarations(&r.declarations))
            .collect();
        
        let optimized = optimize(stylesheet, &config);
        
        // Verify each optimized rule's declarations existed in original
        for opt_rule in &optimized.rules {
            let opt_hash = hash_declarations(&opt_rule.declarations);
            prop_assert!(
                original_decl_hashes.contains(&opt_hash),
                "Optimized rule declarations should match original declarations"
            );
        }
    }
    
    /// Property: Deduplication maintains styling semantics
    /// For any class name in the original stylesheet, the declarations
    /// associated with it should be preserved in the optimized stylesheet.
    #[test]
    fn prop_deduplication_maintains_styling_semantics(
        (stylesheet, _duplicate_groups) in stylesheet_with_duplicates_strategy()
    ) {
        let mut config = CompilerConfig::default();
        config.tree_shaking = false;
        
        // Build a map of class name -> declaration hash
        let original_class_decls: HashMap<_, _> = stylesheet
            .rules
            .iter()
            .map(|r| (r.selector.class_name.clone(), hash_declarations(&r.declarations)))
            .collect();
        
        let optimized = optimize(stylesheet, &config);
        
        // Build a map for optimized stylesheet (including merged selectors)
        let mut optimized_class_decls: HashMap<String, String> = HashMap::new();
        for rule in &optimized.rules {
            let decl_hash = hash_declarations(&rule.declarations);
            optimized_class_decls.insert(rule.selector.class_name.clone(), decl_hash.clone());
            for sel in &rule.selectors {
                optimized_class_decls.insert(sel.class_name.clone(), decl_hash.clone());
            }
        }
        
        // Verify each original class has the same declarations in optimized
        for (class_name, original_hash) in &original_class_decls {
            let optimized_hash = optimized_class_decls.get(class_name);
            prop_assert_eq!(
                optimized_hash,
                Some(original_hash),
                "Class '{}' should have the same declarations after deduplication",
                class_name
            );
        }
    }
    
    /// Property: Deduplication with no duplicates is identity
    /// For any stylesheet where all rules have unique declarations,
    /// deduplication should preserve all rules unchanged.
    #[test]
    fn prop_deduplication_identity_for_unique_rules(
        class_names in prop::collection::vec(class_name_strategy(), 5..=20)
    ) {
        // Generate unique declarations for each class
        let mut rules = Vec::new();
        let mut seen_hashes = HashSet::new();
        
        for class_name in class_names {
            // Generate declarations until we get a unique set
            let declarations = vec![
                Declaration {
                    property: Property::Standard("color".to_string()),
                    value: Value::Literal(format!("#{:06x}", rules.len())),
                    important: false,
                    span: Span::empty(),
                }
            ];
            
            let hash = hash_declarations(&declarations);
            if !seen_hashes.contains(&hash) {
                seen_hashes.insert(hash);
                rules.push(rule_strategy(class_name, declarations));
            }
        }
        
        let stylesheet = StyleSheet {
            rules: rules.clone(),
            at_rules: vec![],
            span: Span::empty(),
        };
        
        let mut config = CompilerConfig::default();
        config.tree_shaking = false;
        
        let original_count = stylesheet.rules.len();
        let optimized = optimize(stylesheet, &config);
        
        // All rules should be preserved since they're unique
        prop_assert_eq!(
            optimized.rules.len(),
            original_count,
            "Deduplication should preserve all {} unique rules",
            original_count
        );
    }
}
