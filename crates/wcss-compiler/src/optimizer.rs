use std::collections::{HashMap, HashSet};

use crate::ast::*;
use crate::config::CompilerConfig;

/// Optimize a stylesheet by tree-shaking, deduplication, and minification.
pub fn optimize(mut stylesheet: StyleSheet, config: &CompilerConfig) -> StyleSheet {
    if config.tree_shaking && !config.used_classes.is_empty() {
        stylesheet = tree_shake(stylesheet, &config.used_classes, &config.safelist);
    }
    if config.deduplicate {
        stylesheet = deduplicate(stylesheet);
    }
    stylesheet
}

/// Check if a class name matches a safelist pattern.
/// Supports glob patterns (e.g., "btn-*", "text-*") and simple prefix regex (e.g., "/^dynamic-/").
fn matches_safelist(class_name: &str, safelist: &[String]) -> bool {
    for pattern in safelist {
        if pattern.starts_with('/') && pattern.ends_with('/') && pattern.len() > 2 {
            // Simple regex-like pattern: support /^prefix/ for starts-with
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
            // Glob pattern: "btn-*", "text-*", or exact match
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
        // No wildcard — exact match
        return pattern == text;
    }

    let mut pos = 0;
    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }
        if let Some(found) = text[pos..].find(part) {
            if i == 0 && found != 0 {
                // First part must match at the start
                return false;
            }
            pos += found + part.len();
        } else {
            return false;
        }
    }

    // If pattern doesn't end with '*', text must end at pos
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
            })
            .collect(),
        span: stylesheet.span,
    }
}

/// Merge rules with identical declarations by combining selectors.
fn deduplicate(stylesheet: StyleSheet) -> StyleSheet {
    let mut seen: HashMap<u64, usize> = HashMap::new();
    let mut rules: Vec<Rule> = Vec::new();

    for rule in stylesheet.rules {
        // Only merge rules that have no states or responsive blocks
        if !rule.states.is_empty() || !rule.responsive.is_empty() {
            rules.push(rule);
            continue;
        }

        let decl_key = hash_declarations(&rule.declarations);

        if let Some(&existing_idx) = seen.get(&decl_key) {
            // Only merge if the existing rule also has no states/responsive
            if rules[existing_idx].states.is_empty() && rules[existing_idx].responsive.is_empty() {
                // Skip — the existing rule already covers these declarations.
                // The selector is different, but declarations are identical,
                // so the rule is deduplicated (each class still gets its own rule
                // in the current AST model where Rule has a single Selector).
                //
                // Note: True selector merging (.a, .b { ... }) would require
                // changing the AST to support multi-selectors. For now we keep
                // the first occurrence and drop duplicates.
                // Actually, we need to keep all rules because each has a unique selector.
                // The dedup should only remove rules with the SAME selector AND same declarations.
                let existing_class = &rules[existing_idx].selector.class_name;
                if existing_class == &rule.selector.class_name {
                    // Truly duplicate — same selector, same declarations. Skip.
                    continue;
                }
                // Different selector, same declarations — keep both
                // (can't merge selectors in current single-selector AST)
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
        span: stylesheet.span,
    }
}

/// Create a hash of declarations for comparison using a fast hasher.
fn hash_declarations(declarations: &[Declaration]) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    for d in declarations {
        d.property.name().hash(&mut hasher);
        // Hash the debug representation of the value for uniqueness
        format!("{:?}", d.value).hash(&mut hasher);
    }
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_rule(class: &str, prop: &str, val: &str) -> Rule {
        Rule {
            selector: Selector {
                class_name: class.to_string(),
                combinators: vec![],
                pseudo_elements: vec![],
                span: Span::empty(),
            },
            declarations: vec![Declaration {
                property: Property::Standard(prop.to_string()),
                value: Value::Literal(val.to_string()),
                important: false,
                span: Span::empty(),
            }],
            states: vec![],
            responsive: vec![],
            span: Span::empty(),
        }
    }

    #[test]
    fn test_tree_shake_removes_unused() {
        let stylesheet = StyleSheet {
            rules: vec![
                make_rule("used", "color", "red"),
                make_rule("unused", "color", "blue"),
            ],
            span: Span::empty(),
        };

        let result = tree_shake(stylesheet, &["used".to_string()], &[]);
        assert_eq!(result.rules.len(), 1);
        assert_eq!(result.rules[0].selector.class_name, "used");
    }

    #[test]
    fn test_tree_shake_keeps_all_used() {
        let stylesheet = StyleSheet {
            rules: vec![
                make_rule("a", "color", "red"),
                make_rule("b", "color", "blue"),
            ],
            span: Span::empty(),
        };

        let result = tree_shake(stylesheet, &["a".to_string(), "b".to_string()], &[]);
        assert_eq!(result.rules.len(), 2);
    }

    #[test]
    fn test_tree_shake_safelist_glob() {
        let stylesheet = StyleSheet {
            rules: vec![
                make_rule("btn-primary", "color", "red"),
                make_rule("btn-secondary", "color", "blue"),
                make_rule("text-lg", "font-size", "18px"),
            ],
            span: Span::empty(),
        };

        let safelist = vec!["btn-*".to_string()];
        let result = tree_shake(stylesheet, &["text-lg".to_string()], &safelist);
        assert_eq!(result.rules.len(), 3);
    }

    #[test]
    fn test_tree_shake_safelist_regex() {
        let stylesheet = StyleSheet {
            rules: vec![
                make_rule("dynamic-header", "color", "red"),
                make_rule("static-footer", "color", "blue"),
            ],
            span: Span::empty(),
        };

        let safelist = vec!["/^dynamic-/".to_string()];
        let result = tree_shake(stylesheet, &["static-footer".to_string()], &safelist);
        assert_eq!(result.rules.len(), 2);
    }

    #[test]
    fn test_dedup_removes_exact_duplicates() {
        let stylesheet = StyleSheet {
            rules: vec![
                make_rule("a", "color", "red"),
                make_rule("a", "color", "red"),
            ],
            span: Span::empty(),
        };

        let result = deduplicate(stylesheet);
        assert_eq!(result.rules.len(), 1);
    }

    #[test]
    fn test_dedup_keeps_different_selectors() {
        let stylesheet = StyleSheet {
            rules: vec![
                make_rule("a", "color", "red"),
                make_rule("b", "color", "red"),
            ],
            span: Span::empty(),
        };

        let result = deduplicate(stylesheet);
        assert_eq!(result.rules.len(), 2);
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
}
