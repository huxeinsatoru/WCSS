// Feature: wcss-framework, Property 8: Minification Equivalence
// **Validates: Requirements 2.5**

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
        Just("border".to_string()),
        Just("border-color".to_string()),
        Just("opacity".to_string()),
    ]
}

/// Generate a valid CSS value with various formats.
fn value_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        // 6-digit hex colors (can be shortened)
        Just("#ffffff".to_string()),
        Just("#000000".to_string()),
        Just("#ff0000".to_string()),
        Just("#00ff00".to_string()),
        Just("#0000ff".to_string()),
        Just("#aabbcc".to_string()),
        // 6-digit hex colors that cannot be shortened
        Just("#123456".to_string()),
        Just("#abcdef".to_string()),
        // RGB colors
        "rgb\\([0-9]{1,3}, [0-9]{1,3}, [0-9]{1,3}\\)".prop_map(|s| s.to_string()),
        // Sizes
        "[0-9]{1,3}(px|rem|em|%)".prop_map(|s| s.to_string()),
        // Keywords
        Just("block".to_string()),
        Just("flex".to_string()),
        Just("none".to_string()),
        Just("auto".to_string()),
        Just("solid".to_string()),
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

/// Generate a rule with random declarations.
fn rule_strategy() -> impl Strategy<Value = Rule> {
    (class_name_strategy(), prop::collection::vec(declaration_strategy(), 1..=8))
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
            nested_at_rules: vec![],
            span: Span::empty(),
        })
}

/// Generate a stylesheet with multiple rules.
fn stylesheet_strategy() -> impl Strategy<Value = StyleSheet> {
    prop::collection::vec(rule_strategy(), 5..=30).prop_map(|rules| StyleSheet {
        rules,
        at_rules: vec![],
        span: Span::empty(),
    })
}

/// Normalize CSS for semantic comparison by removing all whitespace.
/// This allows us to compare the actual CSS content ignoring formatting.
fn normalize_css(css: &str) -> String {
    css.chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>()
}

/// Normalize hex colors to their canonical 6-digit form for comparison.
/// Converts #fff -> #ffffff, #abc -> #aabbcc, etc.
fn normalize_hex_colors(css: &str) -> String {
    let mut result = String::with_capacity(css.len());
    let chars: Vec<char> = css.chars().collect();
    let mut i = 0;
    
    while i < chars.len() {
        if chars[i] == '#' {
            // Check if we have at least 4 characters (#abc)
            if i + 4 <= chars.len() {
                let potential_short: String = chars[i + 1..i + 4].iter().collect();
                // Check if it's a 3-digit hex color
                if potential_short.len() == 3 && potential_short.chars().all(|c| c.is_ascii_hexdigit()) {
                    // Check if the next character is NOT a hex digit (to avoid matching #ffffff as #fff)
                    if i + 4 >= chars.len() || !chars[i + 4].is_ascii_hexdigit() {
                        // Expand to 6 digits
                        result.push('#');
                        result.push(chars[i + 1]);
                        result.push(chars[i + 1]);
                        result.push(chars[i + 2]);
                        result.push(chars[i + 2]);
                        result.push(chars[i + 3]);
                        result.push(chars[i + 3]);
                        i += 4;
                        continue;
                    }
                }
            }
        }
        result.push(chars[i]);
        i += 1;
    }
    
    result
}

/// Extract all CSS declarations from a CSS string.
/// Returns a set of "selector{property:value}" strings for comparison.
fn extract_declarations(css: &str) -> Vec<String> {
    let normalized = normalize_css(css);
    let normalized = normalize_hex_colors(&normalized);
    let mut declarations = Vec::new();
    
    // Simple extraction: find all "selector{...}" blocks
    let mut current = String::new();
    let mut in_block = false;
    let mut brace_count = 0;
    
    for ch in normalized.chars() {
        current.push(ch);
        if ch == '{' {
            in_block = true;
            brace_count += 1;
        } else if ch == '}' {
            brace_count -= 1;
            if brace_count == 0 && in_block {
                declarations.push(current.clone());
                current.clear();
                in_block = false;
            }
        }
    }
    
    declarations.sort();
    declarations
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    /// Property: Minification produces smaller or equal output
    /// For any CSS output, minification should reduce the size or maintain it
    /// (in cases where the CSS is already minimal).
    #[test]
    fn prop_minification_reduces_size(stylesheet in stylesheet_strategy()) {
        // Generate non-minified CSS
        let mut config_normal = CompilerConfig::default();
        config_normal.minify = false;
        let normal_css = generate_css(&stylesheet, &config_normal);
        
        // Generate minified CSS
        let mut config_minified = CompilerConfig::default();
        config_minified.minify = true;
        let minified_css = generate_css(&stylesheet, &config_minified);
        
        // Minified should be smaller or equal
        prop_assert!(
            minified_css.len() <= normal_css.len(),
            "Minified CSS ({} bytes) should be <= normal CSS ({} bytes)",
            minified_css.len(),
            normal_css.len()
        );
    }
    
    /// Property: Minification preserves semantic equivalence
    /// For any CSS output, the minified version should have the same
    /// semantic meaning (same selectors and declarations).
    #[test]
    fn prop_minification_preserves_semantics(stylesheet in stylesheet_strategy()) {
        // Generate non-minified CSS
        let mut config_normal = CompilerConfig::default();
        config_normal.minify = false;
        let normal_css = generate_css(&stylesheet, &config_normal);
        
        // Generate minified CSS
        let mut config_minified = CompilerConfig::default();
        config_minified.minify = true;
        let minified_css = generate_css(&stylesheet, &config_minified);
        
        // Extract declarations from both
        let normal_decls = extract_declarations(&normal_css);
        let minified_decls = extract_declarations(&minified_css);
        
        // Should have the same declarations (ignoring whitespace)
        prop_assert_eq!(
            normal_decls,
            minified_decls,
            "Minified CSS should have the same declarations as normal CSS"
        );
    }
    
    /// Property: Minification removes whitespace
    /// For any CSS with whitespace, minification should remove unnecessary whitespace.
    #[test]
    fn prop_minification_removes_whitespace(stylesheet in stylesheet_strategy()) {
        // Skip empty stylesheets
        prop_assume!(!stylesheet.rules.is_empty());
        
        // Generate non-minified CSS (should have whitespace)
        let mut config_normal = CompilerConfig::default();
        config_normal.minify = false;
        let normal_css = generate_css(&stylesheet, &config_normal);
        
        // Generate minified CSS
        let mut config_minified = CompilerConfig::default();
        config_minified.minify = true;
        let minified_css = generate_css(&stylesheet, &config_minified);
        
        // Count whitespace characters
        let normal_whitespace = normal_css.chars().filter(|c| c.is_whitespace()).count();
        let minified_whitespace = minified_css.chars().filter(|c| c.is_whitespace()).count();
        
        // Minified should have less or equal whitespace
        prop_assert!(
            minified_whitespace <= normal_whitespace,
            "Minified CSS should have <= whitespace: {} vs {}",
            minified_whitespace,
            normal_whitespace
        );
    }
    
    /// Property: Minification shortens hex colors
    /// For any CSS with 6-digit hex colors that can be shortened (e.g., #ffffff -> #fff),
    /// minification should shorten them.
    #[test]
    fn prop_minification_shortens_hex_colors(
        class_name in class_name_strategy()
    ) {
        // Create a stylesheet with shortenable hex colors
        let stylesheet = StyleSheet {
            rules: vec![
                Rule {
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
                    declarations: vec![
                        Declaration {
                            property: Property::Standard("color".to_string()),
                            value: Value::Literal("#ffffff".to_string()),
                            important: false,
                            span: Span::empty(),
                        },
                        Declaration {
                            property: Property::Standard("background".to_string()),
                            value: Value::Literal("#000000".to_string()),
                            important: false,
                            span: Span::empty(),
                        },
                    ],
                    states: vec![],
                    responsive: vec![],
                    nested_rules: vec![],
            nested_at_rules: vec![],
                    span: Span::empty(),
                },
            ],
            at_rules: vec![],
            span: Span::empty(),
        };
        
        // Generate minified CSS
        let mut config = CompilerConfig::default();
        config.minify = true;
        let minified_css = generate_css(&stylesheet, &config);
        
        // Should contain shortened hex colors
        prop_assert!(
            minified_css.contains("#fff") || minified_css.contains("#FFF"),
            "Minified CSS should contain shortened #fff (from #ffffff)"
        );
        prop_assert!(
            minified_css.contains("#000"),
            "Minified CSS should contain shortened #000 (from #000000)"
        );
        
        // Should NOT contain the long forms
        prop_assert!(
            !minified_css.contains("#ffffff") && !minified_css.contains("#FFFFFF"),
            "Minified CSS should not contain long form #ffffff"
        );
        prop_assert!(
            !minified_css.contains("#000000"),
            "Minified CSS should not contain long form #000000"
        );
    }
    
    /// Property: Minification is idempotent
    /// Minifying already-minified CSS should produce the same result.
    #[test]
    fn prop_minification_is_idempotent(stylesheet in stylesheet_strategy()) {
        // Generate minified CSS
        let mut config = CompilerConfig::default();
        config.minify = true;
        let minified_once = generate_css(&stylesheet, &config);
        
        // The minification happens during generation, so we can't directly
        // minify the output again. Instead, we verify that generating with
        // minify=true produces consistent output.
        let minified_twice = generate_css(&stylesheet, &config);
        
        prop_assert_eq!(
            minified_once,
            minified_twice,
            "Minification should be deterministic"
        );
    }
    
    /// Property: Minification preserves rule count
    /// Minification should not add or remove CSS rules, only optimize their representation.
    #[test]
    fn prop_minification_preserves_rule_count(stylesheet in stylesheet_strategy()) {
        // Count rules in original stylesheet
        let original_rule_count = stylesheet.rules.len();
        
        // Generate non-minified CSS
        let mut config_normal = CompilerConfig::default();
        config_normal.minify = false;
        let normal_css = generate_css(&stylesheet, &config_normal);
        
        // Generate minified CSS
        let mut config_minified = CompilerConfig::default();
        config_minified.minify = true;
        let minified_css = generate_css(&stylesheet, &config_minified);
        
        // Count CSS rules (count opening braces as a proxy)
        let normal_rule_count = normal_css.matches('{').count();
        let minified_rule_count = minified_css.matches('{').count();
        
        prop_assert_eq!(
            normal_rule_count,
            minified_rule_count,
            "Minification should preserve rule count: {} rules",
            original_rule_count
        );
    }
}
