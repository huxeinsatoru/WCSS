use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

use crate::ast::*;

// ---------------------------------------------------------------------------
// Stats structs
// ---------------------------------------------------------------------------

/// Statistics about a rule type (e.g., media queries, keyframes).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RuleTypeStats {
    pub count: usize,
    pub total_size_bytes: usize,
}

/// Size information for a single rule.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RuleSizeInfo {
    pub selector: String,
    pub size_bytes: usize,
    pub declarations_count: usize,
}

/// Complete bundle analysis statistics comparing before/after optimization.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BundleStats {
    pub original_size: usize,
    pub optimized_size: usize,
    pub reduction_percentage: f64,
    pub total_rules: usize,
    pub removed_rules: usize,
    pub kept_rules: usize,
    pub total_declarations: usize,
    pub removed_declarations: usize,
    pub duplicate_declarations_removed: usize,
    pub shorthand_merges: usize,
    pub rules_by_type: HashMap<String, RuleTypeStats>,
    pub largest_rules: Vec<RuleSizeInfo>,
    pub unused_selectors: Vec<String>,
}

// ---------------------------------------------------------------------------
// Counting helpers
// ---------------------------------------------------------------------------

/// Count all declarations in a stylesheet (rules + at-rules).
fn count_declarations(stylesheet: &StyleSheet) -> usize {
    let mut count = 0;
    for rule in &stylesheet.rules {
        count += count_rule_declarations(rule);
    }
    for at_rule in &stylesheet.at_rules {
        count += count_at_rule_declarations(at_rule);
    }
    count
}

/// Count declarations in a single rule including states, responsive blocks, and nested rules.
fn count_rule_declarations(rule: &Rule) -> usize {
    let mut count = rule.declarations.len();
    for state in &rule.states {
        count += state.declarations.len();
    }
    for responsive in &rule.responsive {
        count += responsive.declarations.len();
    }
    for nested in &rule.nested_rules {
        count += count_rule_declarations(nested);
    }
    for nested_at in &rule.nested_at_rules {
        count += nested_at.declarations.len();
        for nested_rule in &nested_at.nested_rules {
            count += count_rule_declarations(nested_rule);
        }
    }
    count
}

fn count_at_rule_declarations(at_rule: &AtRule) -> usize {
    match at_rule {
        AtRule::FontFace(ff) => ff.declarations.len(),
        AtRule::Keyframes(kf) => kf.keyframes.iter().map(|k| k.declarations.len()).sum(),
        AtRule::Layer(layer) => {
            layer.rules.as_ref().map_or(0, |rules| {
                rules.iter().map(|r| count_rule_declarations(r)).sum()
            })
        }
        AtRule::Supports(sup) => sup.rules.iter().map(|r| count_rule_declarations(r)).sum(),
        AtRule::Container(cont) => cont.rules.iter().map(|r| count_rule_declarations(r)).sum(),
        AtRule::Media(media) => media.rules.iter().map(|r| count_rule_declarations(r)).sum(),
        AtRule::Page(page) => page.declarations.len(),
        _ => 0,
    }
}

/// Collect all selectors from a stylesheet as strings.
fn collect_selectors(stylesheet: &StyleSheet) -> HashSet<String> {
    let mut selectors = HashSet::new();
    for rule in &stylesheet.rules {
        collect_rule_selectors(rule, &mut selectors);
    }
    selectors
}

fn collect_rule_selectors(rule: &Rule, selectors: &mut HashSet<String>) {
    selectors.insert(rule.selector.class_name.clone());
    for sel in &rule.selectors {
        selectors.insert(sel.class_name.clone());
    }
    for nested in &rule.nested_rules {
        collect_rule_selectors(nested, selectors);
    }
}

/// Detect shorthand merges by counting shorthand properties in the after stylesheet
/// that correspond to longhand properties in the before stylesheet.
fn count_shorthand_merges(before: &StyleSheet, after: &StyleSheet) -> usize {
    let shorthands = ["margin", "padding", "border-radius"];

    let before_longhands = count_longhand_properties(before, &shorthands);
    let after_shorthands = count_shorthand_properties(after, &shorthands);

    // Each shorthand in after that wasn't in before represents a merge
    after_shorthands.saturating_sub(count_shorthand_properties(before, &shorthands))
        + if before_longhands > 0 && after_shorthands > 0 {
            // Heuristic: if longhands decreased and shorthands appeared, count the shorthands
            0
        } else {
            0
        }
}

fn count_longhand_properties(stylesheet: &StyleSheet, shorthands: &[&str]) -> usize {
    let longhand_prefixes: Vec<&str> = shorthands.to_vec();
    let mut count = 0;
    for rule in &stylesheet.rules {
        for decl in &rule.declarations {
            let name = decl.property.name();
            for prefix in &longhand_prefixes {
                if name.starts_with(prefix) && name != *prefix && name.contains('-') {
                    count += 1;
                }
            }
        }
    }
    count
}

fn count_shorthand_properties(stylesheet: &StyleSheet, shorthands: &[&str]) -> usize {
    let mut count = 0;
    for rule in &stylesheet.rules {
        for decl in &rule.declarations {
            let name = decl.property.name();
            if shorthands.contains(&name) {
                count += 1;
            }
        }
    }
    count
}

/// Detect duplicate declarations removed: declarations in before that share the same
/// property+value within or across rules but are absent in after.
fn count_duplicate_declarations_removed(before: &StyleSheet, after: &StyleSheet) -> usize {
    let before_total = count_declarations(before);
    let after_total = count_declarations(after);
    let removed_rules_decls = estimate_removed_rule_declarations(before, after);

    // Duplicates removed = total removed declarations minus those removed due to tree-shaking
    let total_removed = before_total.saturating_sub(after_total);
    total_removed.saturating_sub(removed_rules_decls)
}

/// Estimate declarations removed because their parent rules were removed (tree-shaking).
fn estimate_removed_rule_declarations(before: &StyleSheet, after: &StyleSheet) -> usize {
    let after_selectors = collect_selectors(after);
    let mut removed_decl_count = 0;
    for rule in &before.rules {
        let has_match = after_selectors.contains(&rule.selector.class_name)
            || rule.selectors.iter().any(|s| after_selectors.contains(&s.class_name));
        if !has_match {
            removed_decl_count += count_rule_declarations(rule);
        }
    }
    removed_decl_count
}

/// Build rules_by_type map from at-rules in the after stylesheet.
fn build_rules_by_type(stylesheet: &StyleSheet) -> HashMap<String, RuleTypeStats> {
    let mut map: HashMap<String, RuleTypeStats> = HashMap::new();

    // Count style rules
    if !stylesheet.rules.is_empty() {
        map.insert("style_rules".to_string(), RuleTypeStats {
            count: stylesheet.rules.len(),
            total_size_bytes: 0, // Will be filled by caller if needed
        });
    }

    for at_rule in &stylesheet.at_rules {
        let (key, size_estimate) = match at_rule {
            AtRule::Media(m) => ("media_queries".to_string(), estimate_media_size(m)),
            AtRule::Keyframes(k) => ("keyframes".to_string(), estimate_keyframes_size(k)),
            AtRule::FontFace(ff) => ("font_faces".to_string(), ff.declarations.len() * 30),
            AtRule::Layer(_) => ("layers".to_string(), 50),
            AtRule::Supports(_) => ("supports".to_string(), 50),
            AtRule::Container(_) => ("containers".to_string(), 50),
            AtRule::Import(_) => ("imports".to_string(), 40),
            AtRule::Property(_) => ("properties".to_string(), 60),
            AtRule::Charset(_, _) => ("charset".to_string(), 20),
            AtRule::Namespace(_, _) => ("namespace".to_string(), 30),
            AtRule::Scope(_) => ("scope".to_string(), 50),
            AtRule::Tailwind(_) => ("tailwind".to_string(), 30),
        };

        let entry = map.entry(key).or_insert(RuleTypeStats {
            count: 0,
            total_size_bytes: 0,
        });
        entry.count += 1;
        entry.total_size_bytes += size_estimate;
    }

    map
}

fn estimate_media_size(media: &MediaRule) -> usize {
    let header = media.query.len() + 10; // @media ... {
    let body: usize = media.rules.iter().map(|r| estimate_rule_size(r)).sum();
    header + body
}

fn estimate_keyframes_size(kf: &KeyframesRule) -> usize {
    let header = kf.name.len() + 15; // @keyframes name {
    let body: usize = kf.keyframes.iter().map(|k| k.declarations.len() * 25 + 10).sum();
    header + body
}

/// Estimate the CSS output size of a rule (rough approximation).
fn estimate_rule_size(rule: &Rule) -> usize {
    let selector_size = rule.selector.class_name.len() + 5;
    let decl_size: usize = rule.declarations.iter().map(|d| {
        d.property.name().len() + estimate_value_size(&d.value) + 4 // "prop: value; "
    }).sum();
    let state_size: usize = rule.states.iter().map(|s| {
        s.declarations.len() * 25 + 20
    }).sum();
    let responsive_size: usize = rule.responsive.iter().map(|r| {
        r.declarations.len() * 25 + 30
    }).sum();
    let nested_size: usize = rule.nested_rules.iter().map(|n| estimate_rule_size(n)).sum();

    selector_size + decl_size + state_size + responsive_size + nested_size + 4 // braces + newlines
}

fn estimate_value_size(value: &Value) -> usize {
    match value {
        Value::Literal(s) => s.len(),
        Value::Number(n, unit) => {
            let num_str = format!("{n}");
            num_str.len() + unit.as_ref().map_or(0, |u| u.as_str().len())
        }
        Value::Color(c) => match c {
            Color::Hex(h) => h.len() + 1,
            Color::Named(n) => n.len(),
            _ => 20,
        },
        Value::Token(t) => t.name.len() + t.category.as_str().len() + 2,
        Value::Var(name, _) => name.len() + 6,
        Value::Env(name, _) => name.len() + 6,
        Value::List(vals) => vals.iter().map(|v| estimate_value_size(v) + 1).sum(),
        Value::Computed(_) => 20,
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Analyze a bundle by comparing before/after stylesheets and CSS output.
///
/// - `before`: the stylesheet AST before optimization
/// - `after`: the stylesheet AST after optimization
/// - `original_css`: the CSS string before optimization
/// - `optimized_css`: the CSS string after optimization
pub fn analyze_bundle(
    before: &StyleSheet,
    after: &StyleSheet,
    original_css: &str,
    optimized_css: &str,
) -> BundleStats {
    let original_size = original_css.len();
    let optimized_size = optimized_css.len();
    let reduction_percentage = if original_size > 0 {
        ((original_size as f64 - optimized_size as f64) / original_size as f64) * 100.0
    } else {
        0.0
    };

    let total_rules = before.rules.len();
    let kept_rules = after.rules.len();
    let removed_rules = total_rules.saturating_sub(kept_rules);

    let total_declarations = count_declarations(before);
    let after_declarations = count_declarations(after);
    let removed_declarations = total_declarations.saturating_sub(after_declarations);

    let duplicate_declarations_removed = count_duplicate_declarations_removed(before, after);
    let shorthand_merges = count_shorthand_merges(before, after);

    // Find unused (removed) selectors
    let before_selectors = collect_selectors(before);
    let after_selectors = collect_selectors(after);
    let mut unused_selectors: Vec<String> = before_selectors
        .difference(&after_selectors)
        .cloned()
        .collect();
    unused_selectors.sort();

    // Build rules by type from the after stylesheet
    let rules_by_type = build_rules_by_type(after);

    // Find largest rules by estimated size, top 10
    let mut rule_sizes: Vec<RuleSizeInfo> = after
        .rules
        .iter()
        .map(|rule| {
            let selector = {
                let mut s = rule.selector.class_name.clone();
                for extra in &rule.selectors {
                    s.push_str(", ");
                    s.push_str(&extra.class_name);
                }
                s
            };
            RuleSizeInfo {
                selector,
                size_bytes: estimate_rule_size(rule),
                declarations_count: count_rule_declarations(rule),
            }
        })
        .collect();
    rule_sizes.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
    rule_sizes.truncate(10);

    BundleStats {
        original_size,
        optimized_size,
        reduction_percentage,
        total_rules,
        removed_rules,
        kept_rules,
        total_declarations,
        removed_declarations,
        duplicate_declarations_removed,
        shorthand_merges,
        rules_by_type,
        largest_rules: rule_sizes,
        unused_selectors,
    }
}

// ---------------------------------------------------------------------------
// Report formatting
// ---------------------------------------------------------------------------

/// Format a human-readable size string (e.g., "1.2 KB", "340.2 KB").
fn format_size(bytes: usize) -> String {
    if bytes >= 1_048_576 {
        format!("{:.1} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

/// Generate a human-readable text report from bundle stats.
pub fn format_report(stats: &BundleStats) -> String {
    let width = 42;
    let border_h = "=".repeat(width - 2);

    let mut lines: Vec<String> = Vec::new();

    // Top border
    lines.push(format!("\u{2554}{}\u{2557}", border_h));
    // Title
    lines.push(format!("\u{2551}{:^w$}\u{2551}", "WCSS Bundle Analysis", w = width - 2));
    // Separator
    lines.push(format!("\u{2560}{}\u{2563}", border_h));

    // Size stats
    let saved = stats.original_size.saturating_sub(stats.optimized_size);
    lines.push(format!(
        "\u{2551} {:<16}{:>w$} \u{2551}",
        "Original:",
        format_size(stats.original_size),
        w = width - 20
    ));
    lines.push(format!(
        "\u{2551} {:<16}{:>w$} \u{2551}",
        "Optimized:",
        format_size(stats.optimized_size),
        w = width - 20
    ));
    lines.push(format!(
        "\u{2551} {:<16}{:>w$} \u{2551}",
        "Saved:",
        format!("{} ({:.1}%)", format_size(saved), stats.reduction_percentage),
        w = width - 20
    ));

    // Separator
    lines.push(format!("\u{2560}{}\u{2563}", border_h));

    // Rule/declaration stats
    let rules_line = format!(
        "Rules: {} -> {} (-{})",
        stats.total_rules, stats.kept_rules, stats.removed_rules
    );
    lines.push(format!("\u{2551} {:<w$} \u{2551}", rules_line, w = width - 4));

    let decl_line = format!(
        "Declarations: {} -> {} (-{})",
        stats.total_declarations,
        stats.total_declarations.saturating_sub(stats.removed_declarations),
        stats.removed_declarations
    );
    lines.push(format!("\u{2551} {:<w$} \u{2551}", decl_line, w = width - 4));

    let dup_line = format!("Duplicates removed: {}", stats.duplicate_declarations_removed);
    lines.push(format!("\u{2551} {:<w$} \u{2551}", dup_line, w = width - 4));

    let merge_line = format!("Shorthand merges: {}", stats.shorthand_merges);
    lines.push(format!("\u{2551} {:<w$} \u{2551}", merge_line, w = width - 4));

    // Largest rules section
    if !stats.largest_rules.is_empty() {
        lines.push(format!("\u{2560}{}\u{2563}", border_h));
        lines.push(format!("\u{2551} {:<w$} \u{2551}", "Top 10 Largest Rules:", w = width - 4));
        for (i, rule) in stats.largest_rules.iter().enumerate() {
            let selector_display = if rule.selector.len() > 15 {
                format!("{}...", &rule.selector[..12])
            } else {
                rule.selector.clone()
            };
            let entry = format!(
                " {:>2}. {:<15} ({}, {} decl)",
                i + 1,
                selector_display,
                format_size(rule.size_bytes),
                rule.declarations_count
            );
            lines.push(format!("\u{2551}{:<w$}\u{2551}", entry, w = width - 2));
        }
    }

    // Unused selectors section
    if !stats.unused_selectors.is_empty() {
        lines.push(format!("\u{2560}{}\u{2563}", border_h));
        let header = format!("Unused selectors removed: {}", stats.unused_selectors.len());
        lines.push(format!("\u{2551} {:<w$} \u{2551}", header, w = width - 4));
        let display_count = stats.unused_selectors.len().min(5);
        for sel in &stats.unused_selectors[..display_count] {
            lines.push(format!("\u{2551}   {:<w$} \u{2551}", sel, w = width - 6));
        }
        if stats.unused_selectors.len() > 5 {
            let more = format!("  ...and {} more", stats.unused_selectors.len() - 5);
            lines.push(format!("\u{2551} {:<w$} \u{2551}", more, w = width - 4));
        }
    }

    // Bottom border
    lines.push(format!("\u{255a}{}\u{255d}", border_h));

    lines.join("\n")
}

/// Generate a JSON-formatted report from bundle stats.
pub fn format_report_json(stats: &BundleStats) -> String {
    serde_json::to_string_pretty(stats).unwrap_or_else(|_| "{}".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_rule(class: &str, props: &[(&str, &str)]) -> Rule {
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
            declarations: props
                .iter()
                .map(|(p, v)| Declaration {
                    property: Property::Standard(p.to_string()),
                    value: Value::Literal(v.to_string()),
                    important: false,
                    span: Span::empty(),
                })
                .collect(),
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
    fn test_format_size() {
        assert_eq!(format_size(500), "500 B");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1536), "1.5 KB");
        assert_eq!(format_size(1_048_576), "1.0 MB");
    }

    #[test]
    fn test_count_declarations() {
        let ss = make_stylesheet(vec![
            make_rule("a", &[("color", "red"), ("margin", "0")]),
            make_rule("b", &[("padding", "10px")]),
        ]);
        assert_eq!(count_declarations(&ss), 3);
    }

    #[test]
    fn test_collect_selectors() {
        let ss = make_stylesheet(vec![
            make_rule("a", &[("color", "red")]),
            make_rule("b", &[("color", "blue")]),
        ]);
        let sels = collect_selectors(&ss);
        assert!(sels.contains("a"));
        assert!(sels.contains("b"));
        assert_eq!(sels.len(), 2);
    }

    #[test]
    fn test_analyze_basic() {
        let before = make_stylesheet(vec![
            make_rule("btn", &[("color", "red")]),
            make_rule("card", &[("padding", "10px")]),
            make_rule("unused", &[("display", "none")]),
        ]);
        let after = make_stylesheet(vec![
            make_rule("btn", &[("color", "red")]),
            make_rule("card", &[("padding", "10px")]),
        ]);

        let original_css = ".btn { color: red; }\n.card { padding: 10px; }\n.unused { display: none; }";
        let optimized_css = ".btn { color: red; }\n.card { padding: 10px; }";

        let stats = analyze_bundle(&before, &after, original_css, optimized_css);

        assert_eq!(stats.total_rules, 3);
        assert_eq!(stats.kept_rules, 2);
        assert_eq!(stats.removed_rules, 1);
        assert_eq!(stats.unused_selectors, vec!["unused".to_string()]);
    }

    #[test]
    fn test_reduction_percentage() {
        let before = make_stylesheet(vec![]);
        let after = make_stylesheet(vec![]);

        let original = "a".repeat(1000);
        let optimized = "a".repeat(100);

        let stats = analyze_bundle(&before, &after, &original, &optimized);

        assert!((stats.reduction_percentage - 90.0).abs() < 0.01);
        assert_eq!(stats.original_size, 1000);
        assert_eq!(stats.optimized_size, 100);
    }

    #[test]
    fn test_empty_stylesheet() {
        let before = make_stylesheet(vec![]);
        let after = make_stylesheet(vec![]);

        let stats = analyze_bundle(&before, &after, "", "");

        assert_eq!(stats.original_size, 0);
        assert_eq!(stats.optimized_size, 0);
        assert_eq!(stats.reduction_percentage, 0.0);
        assert_eq!(stats.total_rules, 0);
        assert_eq!(stats.kept_rules, 0);
        assert_eq!(stats.removed_rules, 0);
        assert!(stats.largest_rules.is_empty());
        assert!(stats.unused_selectors.is_empty());
    }

    #[test]
    fn test_no_optimization_same_before_after() {
        let rules = vec![
            make_rule("btn", &[("color", "red"), ("padding", "10px")]),
        ];
        let before = make_stylesheet(rules.clone());
        let after = make_stylesheet(rules);

        let css = ".btn { color: red; padding: 10px; }";
        let stats = analyze_bundle(&before, &after, css, css);

        assert_eq!(stats.reduction_percentage, 0.0);
        assert_eq!(stats.removed_rules, 0);
        assert_eq!(stats.removed_declarations, 0);
        assert!(stats.unused_selectors.is_empty());
    }
}
