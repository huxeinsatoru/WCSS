use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

use crate::ast::*;
use crate::config::CompilerConfig;

/// Optimize a stylesheet by tree-shaking, deduplication, shorthand merging, and minification.
pub fn optimize(mut stylesheet: StyleSheet, config: &CompilerConfig) -> StyleSheet {
    if config.tree_shaking && !config.used_classes.is_empty() {
        stylesheet = tree_shake(stylesheet, &config.used_classes, &config.safelist);
    }
    if config.deduplicate {
        stylesheet = deduplicate(stylesheet);
        stylesheet = merge_selectors(stylesheet);
    }
    if config.merge_shorthands {
        stylesheet = merge_shorthand_properties(stylesheet);
    }
    stylesheet
}

// ---------------------------------------------------------------------------
// Tree Shaking
// ---------------------------------------------------------------------------

/// Check if a class name matches a safelist pattern.
fn matches_safelist(class_name: &str, safelist: &[String]) -> bool {
    for pattern in safelist {
        if pattern.starts_with('/') && pattern.ends_with('/') && pattern.len() > 2 {
            let regex_str = &pattern[1..pattern.len() - 1];
            if let Some(prefix) = regex_str.strip_prefix('^') {
                if class_name.starts_with(prefix) {
                    return true;
                }
            } else if let Some(suffix) = regex_str.strip_suffix('$') {
                if class_name.ends_with(suffix) {
                    return true;
                }
            } else if class_name.contains(regex_str) {
                return true;
            }
        } else {
            if glob_match(pattern, class_name) {
                return true;
            }
        }
    }
    false
}

/// Simple glob matching supporting '*' wildcard.
fn glob_match(pattern: &str, text: &str) -> bool {
    let parts: Vec<&str> = pattern.split('*').collect();
    if parts.len() == 1 {
        return pattern == text;
    }

    let mut pos = 0;
    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }
        if let Some(found) = text[pos..].find(part) {
            if i == 0 && found != 0 {
                return false;
            }
            pos += found + part.len();
        } else {
            return false;
        }
    }

    if !pattern.ends_with('*') {
        return pos == text.len();
    }

    true
}

/// Remove rules whose selectors are not in the list of used classes.
fn tree_shake(stylesheet: StyleSheet, used_classes: &[String], safelist: &[String]) -> StyleSheet {
    let used_set: HashSet<&str> = used_classes.iter().map(|s| s.as_str()).collect();
    StyleSheet {
        rules: stylesheet
            .rules
            .into_iter()
            .filter(|rule| {
                used_set.contains(rule.selector.class_name.as_str())
                    || matches_safelist(&rule.selector.class_name, safelist)
                    // Keep all selectors from multi-selector rules if any match
                    || rule.selectors.iter().any(|s| {
                        used_set.contains(s.class_name.as_str())
                            || matches_safelist(&s.class_name, safelist)
                    })
            })
            .collect(),
        at_rules: stylesheet.at_rules,
        span: stylesheet.span,
    }
}

// ---------------------------------------------------------------------------
// Deduplication (using Hash trait instead of format!("{:?}"))
// ---------------------------------------------------------------------------

/// Merge rules with identical declarations by combining selectors.
fn deduplicate(stylesheet: StyleSheet) -> StyleSheet {
    let mut seen: HashMap<u64, usize> = HashMap::new();
    let mut rules: Vec<Rule> = Vec::new();

    for rule in stylesheet.rules {
        if !rule.states.is_empty() || !rule.responsive.is_empty() || !rule.nested_rules.is_empty() || !rule.nested_at_rules.is_empty() {
            rules.push(rule);
            continue;
        }

        let decl_key = hash_declarations(&rule.declarations);

        if let Some(&existing_idx) = seen.get(&decl_key) {
            if rules[existing_idx].states.is_empty()
                && rules[existing_idx].responsive.is_empty()
                && rules[existing_idx].nested_rules.is_empty()
                && rules[existing_idx].nested_at_rules.is_empty()
            {
                let existing_class = &rules[existing_idx].selector.class_name;
                if existing_class == &rule.selector.class_name {
                    continue; // Truly duplicate
                }
                // Different selector, same declarations — keep both (merge later)
                rules.push(rule);
            } else {
                rules.push(rule);
            }
        } else {
            seen.insert(decl_key, rules.len());
            rules.push(rule);
        }
    }

    StyleSheet {
        rules,
        at_rules: stylesheet.at_rules,
        span: stylesheet.span,
    }
}

/// Create a hash of declarations using the Hash trait directly (no format!).
fn hash_declarations(declarations: &[Declaration]) -> u64 {
    use std::collections::hash_map::DefaultHasher;

    let mut hasher = DefaultHasher::new();
    for d in declarations {
        d.property.hash(&mut hasher);
        d.value.hash(&mut hasher);
        d.important.hash(&mut hasher);
    }
    hasher.finish()
}

// ---------------------------------------------------------------------------
// Selector Merging
// ---------------------------------------------------------------------------

/// Merge rules with identical declarations into multi-selector rules.
fn merge_selectors(stylesheet: StyleSheet) -> StyleSheet {
    let mut merged: Vec<Rule> = Vec::new();
    let mut decl_map: HashMap<u64, Vec<usize>> = HashMap::new(); // Changed to Vec<usize> to track all matching rules

    for rule in stylesheet.rules {
        if !rule.states.is_empty() || !rule.responsive.is_empty() || !rule.nested_rules.is_empty() || !rule.nested_at_rules.is_empty() {
            merged.push(rule);
            continue;
        }

        let hash = hash_declarations(&rule.declarations);

        if let Some(existing_indices) = decl_map.get_mut(&hash) {
            // Try to find an existing rule that doesn't already have this class name
            let mut merged_into_existing = false;
            
            for &existing_idx in existing_indices.iter() {
                let existing = &merged[existing_idx];
                
                // Check if this class name is already in the existing rule
                let class_already_exists = existing.selector.class_name == rule.selector.class_name
                    || existing.selectors.iter().any(|s| s.class_name == rule.selector.class_name);
                
                if !class_already_exists {
                    // Merge: add this rule's selector(s) to the existing rule
                    let existing = &mut merged[existing_idx];
                    
                    // Add primary selector
                    existing.selectors.push(rule.selector.clone());
                    
                    // Add additional selectors
                    for sel in &rule.selectors {
                        if !existing.selectors.iter().any(|s| s.class_name == sel.class_name)
                            && existing.selector.class_name != sel.class_name
                        {
                            existing.selectors.push(sel.clone());
                        }
                    }
                    
                    merged_into_existing = true;
                    break;
                }
            }
            
            if !merged_into_existing {
                // This is a new rule with the same declarations but different class name
                // that couldn't be merged into existing rules (maybe due to duplicates)
                existing_indices.push(merged.len());
                merged.push(rule);
            }
        } else {
            decl_map.insert(hash, vec![merged.len()]);
            merged.push(rule);
        }
    }

    StyleSheet {
        rules: merged,
        at_rules: stylesheet.at_rules,
        span: stylesheet.span,
    }
}

// ---------------------------------------------------------------------------
// CSS Shorthand Property Merging
// ---------------------------------------------------------------------------

/// Merge longhand properties into shorthand equivalents.
fn merge_shorthand_properties(mut stylesheet: StyleSheet) -> StyleSheet {
    for rule in &mut stylesheet.rules {
        rule.declarations = merge_declarations(std::mem::take(&mut rule.declarations));

        for state in &mut rule.states {
            state.declarations = merge_declarations(std::mem::take(&mut state.declarations));
        }
        for responsive in &mut rule.responsive {
            responsive.declarations = merge_declarations(std::mem::take(&mut responsive.declarations));
        }
    }
    stylesheet
}

fn merge_declarations(declarations: Vec<Declaration>) -> Vec<Declaration> {
    let mut result = Vec::with_capacity(declarations.len());
    let mut margin_parts: [Option<(Value, bool, Span)>; 4] = [None, None, None, None]; // top, right, bottom, left
    let mut padding_parts: [Option<(Value, bool, Span)>; 4] = [None, None, None, None];
    let mut border_radius_parts: [Option<(Value, bool, Span)>; 4] = [None, None, None, None];
    let mut other_decls: Vec<Declaration> = Vec::new();

    for decl in declarations {
        match decl.property.name() {
            "margin-top" => margin_parts[0] = Some((decl.value, decl.important, decl.span)),
            "margin-right" => margin_parts[1] = Some((decl.value, decl.important, decl.span)),
            "margin-bottom" => margin_parts[2] = Some((decl.value, decl.important, decl.span)),
            "margin-left" => margin_parts[3] = Some((decl.value, decl.important, decl.span)),
            "padding-top" => padding_parts[0] = Some((decl.value, decl.important, decl.span)),
            "padding-right" => padding_parts[1] = Some((decl.value, decl.important, decl.span)),
            "padding-bottom" => padding_parts[2] = Some((decl.value, decl.important, decl.span)),
            "padding-left" => padding_parts[3] = Some((decl.value, decl.important, decl.span)),
            "border-top-left-radius" => border_radius_parts[0] = Some((decl.value, decl.important, decl.span)),
            "border-top-right-radius" => border_radius_parts[1] = Some((decl.value, decl.important, decl.span)),
            "border-bottom-right-radius" => border_radius_parts[2] = Some((decl.value, decl.important, decl.span)),
            "border-bottom-left-radius" => border_radius_parts[3] = Some((decl.value, decl.important, decl.span)),
            _ => other_decls.push(decl),
        }
    }

    // Try to merge margin
    if let Some(merged) = try_merge_shorthand("margin", &margin_parts) {
        result.push(merged);
    } else {
        push_longhand_parts(&mut result, "margin", &["top", "right", "bottom", "left"], &margin_parts);
    }

    // Try to merge padding
    if let Some(merged) = try_merge_shorthand("padding", &padding_parts) {
        result.push(merged);
    } else {
        push_longhand_parts(&mut result, "padding", &["top", "right", "bottom", "left"], &padding_parts);
    }

    // Try to merge border-radius
    if let Some(merged) = try_merge_shorthand("border-radius", &border_radius_parts) {
        result.push(merged);
    } else {
        let sides = ["top-left", "top-right", "bottom-right", "bottom-left"];
        for (i, part) in border_radius_parts.iter().enumerate() {
            if let Some((value, important, span)) = part {
                result.push(Declaration {
                    property: Property::Standard(format!("border-{}-radius", sides[i])),
                    value: value.clone(),
                    important: *important,
                    span: span.clone(),
                });
            }
        }
    }

    result.extend(other_decls);
    result
}

fn try_merge_shorthand(
    shorthand: &str,
    parts: &[Option<(Value, bool, Span)>; 4],
) -> Option<Declaration> {
    // All 4 parts must be present and have the same importance
    let vals: Vec<&(Value, bool, Span)> = parts.iter().filter_map(|p| p.as_ref()).collect();
    if vals.len() != 4 {
        return None;
    }

    let important = vals[0].1;
    if !vals.iter().all(|v| v.1 == important) {
        return None;
    }

    let top = &vals[0].0;
    let right = &vals[1].0;
    let bottom = &vals[2].0;
    let left = &vals[3].0;

    // Generate optimal shorthand
    let value = if top == right && right == bottom && bottom == left {
        // All same: margin: X
        top.clone()
    } else if top == bottom && right == left {
        // margin: Y X
        Value::List(vec![top.clone(), right.clone()])
    } else if right == left {
        // margin: T X B
        Value::List(vec![top.clone(), right.clone(), bottom.clone()])
    } else {
        // margin: T R B L
        Value::List(vec![top.clone(), right.clone(), bottom.clone(), left.clone()])
    };

    Some(Declaration {
        property: Property::Standard(shorthand.to_string()),
        value,
        important,
        span: vals[0].2.clone(),
    })
}

fn push_longhand_parts(
    result: &mut Vec<Declaration>,
    prefix: &str,
    sides: &[&str; 4],
    parts: &[Option<(Value, bool, Span)>; 4],
) {
    for (i, part) in parts.iter().enumerate() {
        if let Some((value, important, span)) = part {
            result.push(Declaration {
                property: Property::Standard(format!("{prefix}-{}", sides[i])),
                value: value.clone(),
                important: *important,
                span: span.clone(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_rule(class: &str, prop: &str, val: &str) -> Rule {
        Rule {
            selector: Selector {
                class_name: class.to_string(),
                kind: SelectorKind::Class,
                combinators: vec![],
                pseudo_elements: vec![],
                pseudo_classes: vec![],
                attributes: vec![],
                span: Span::empty(),
            },
            selectors: vec![],
            declarations: vec![Declaration {
                property: Property::Standard(prop.to_string()),
                value: Value::Literal(val.to_string()),
                important: false,
                span: Span::empty(),
            }],
            states: vec![],
            responsive: vec![],
            nested_rules: vec![],
            nested_at_rules: vec![],
            span: Span::empty(),
        }
    }

    fn make_stylesheet(rules: Vec<Rule>) -> StyleSheet {
        StyleSheet {
            rules,
            at_rules: vec![],
            span: Span::empty(),
        }
    }

    #[test]
    fn test_tree_shake_removes_unused() {
        let stylesheet = make_stylesheet(vec![
            make_rule("used", "color", "red"),
            make_rule("unused", "color", "blue"),
        ]);

        let result = tree_shake(stylesheet, &["used".to_string()], &[]);
        assert_eq!(result.rules.len(), 1);
        assert_eq!(result.rules[0].selector.class_name, "used");
    }

    #[test]
    fn test_tree_shake_keeps_all_used() {
        let stylesheet = make_stylesheet(vec![
            make_rule("a", "color", "red"),
            make_rule("b", "color", "blue"),
        ]);

        let result = tree_shake(stylesheet, &["a".to_string(), "b".to_string()], &[]);
        assert_eq!(result.rules.len(), 2);
    }

    #[test]
    fn test_tree_shake_safelist_glob() {
        let stylesheet = make_stylesheet(vec![
            make_rule("btn-primary", "color", "red"),
            make_rule("btn-secondary", "color", "blue"),
            make_rule("text-lg", "font-size", "18px"),
        ]);

        let safelist = vec!["btn-*".to_string()];
        let result = tree_shake(stylesheet, &["text-lg".to_string()], &safelist);
        assert_eq!(result.rules.len(), 3);
    }

    #[test]
    fn test_tree_shake_safelist_regex() {
        let stylesheet = make_stylesheet(vec![
            make_rule("dynamic-header", "color", "red"),
            make_rule("static-footer", "color", "blue"),
        ]);

        let safelist = vec!["/^dynamic-/".to_string()];
        let result = tree_shake(stylesheet, &["static-footer".to_string()], &safelist);
        assert_eq!(result.rules.len(), 2);
    }

    #[test]
    fn test_dedup_removes_exact_duplicates() {
        let stylesheet = make_stylesheet(vec![
            make_rule("a", "color", "red"),
            make_rule("a", "color", "red"),
        ]);

        let result = deduplicate(stylesheet);
        assert_eq!(result.rules.len(), 1);
    }

    #[test]
    fn test_dedup_keeps_different_selectors() {
        let stylesheet = make_stylesheet(vec![
            make_rule("a", "color", "red"),
            make_rule("b", "color", "red"),
        ]);

        let result = deduplicate(stylesheet);
        assert_eq!(result.rules.len(), 2);
    }

    #[test]
    fn test_merge_selectors_same_declarations() {
        let stylesheet = make_stylesheet(vec![
            make_rule("a", "color", "red"),
            make_rule("b", "color", "red"),
        ]);

        let result = merge_selectors(stylesheet);
        assert_eq!(result.rules.len(), 1);
        assert_eq!(result.rules[0].selector.class_name, "a");
        assert_eq!(result.rules[0].selectors.len(), 1);
        assert_eq!(result.rules[0].selectors[0].class_name, "b");
    }

    #[test]
    fn test_shorthand_merge_margin() {
        let decls = vec![
            Declaration {
                property: Property::Standard("margin-top".to_string()),
                value: Value::Number(10.0, Some(Unit::Px)),
                important: false,
                span: Span::empty(),
            },
            Declaration {
                property: Property::Standard("margin-right".to_string()),
                value: Value::Number(20.0, Some(Unit::Px)),
                important: false,
                span: Span::empty(),
            },
            Declaration {
                property: Property::Standard("margin-bottom".to_string()),
                value: Value::Number(10.0, Some(Unit::Px)),
                important: false,
                span: Span::empty(),
            },
            Declaration {
                property: Property::Standard("margin-left".to_string()),
                value: Value::Number(20.0, Some(Unit::Px)),
                important: false,
                span: Span::empty(),
            },
        ];

        let result = merge_declarations(decls);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].property.name(), "margin");
        // Should be "10px 20px" (top==bottom, right==left)
        match &result[0].value {
            Value::List(vals) => assert_eq!(vals.len(), 2),
            _ => panic!("Expected list value"),
        }
    }

    #[test]
    fn test_glob_match() {
        assert!(glob_match("btn-*", "btn-primary"));
        assert!(glob_match("btn-*", "btn-secondary"));
        assert!(!glob_match("btn-*", "text-lg"));
        assert!(glob_match("*-lg", "text-lg"));
        assert!(glob_match("exact", "exact"));
        assert!(!glob_match("exact", "inexact"));
    }

    #[test]
    fn test_hash_declarations_fast() {
        // Verify Hash-based approach works correctly
        let decl1 = vec![Declaration {
            property: Property::Standard("color".to_string()),
            value: Value::Literal("red".to_string()),
            important: false,
            span: Span::empty(),
        }];
        let decl2 = vec![Declaration {
            property: Property::Standard("color".to_string()),
            value: Value::Literal("red".to_string()),
            important: false,
            span: Span::empty(),
        }];
        let decl3 = vec![Declaration {
            property: Property::Standard("color".to_string()),
            value: Value::Literal("blue".to_string()),
            important: false,
            span: Span::empty(),
        }];

        assert_eq!(hash_declarations(&decl1), hash_declarations(&decl2));
        assert_ne!(hash_declarations(&decl1), hash_declarations(&decl3));
    }
}
