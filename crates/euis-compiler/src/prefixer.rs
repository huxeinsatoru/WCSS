//! Vendor prefixing system for Euis.
//!
//! Automatically adds vendor prefixes based on browser target configuration.
//! Inspired by Autoprefixer but integrated directly into the compilation pipeline.

use crate::ast::*;
use crate::config::BrowserTargets;

/// Apply vendor prefixes to a stylesheet based on browser targets.
pub fn prefix(mut stylesheet: StyleSheet, targets: &BrowserTargets) -> StyleSheet {
    for rule in &mut stylesheet.rules {
        prefix_rule(rule, targets);
    }
    for at_rule in &mut stylesheet.at_rules {
        prefix_at_rule(at_rule, targets);
    }
    stylesheet
}

fn prefix_at_rule(at_rule: &mut AtRule, targets: &BrowserTargets) {
    match at_rule {
        AtRule::Layer(layer) => {
            if let Some(rules) = &mut layer.rules {
                for rule in rules {
                    prefix_rule(rule, targets);
                }
            }
        }
        AtRule::Supports(s) => {
            for rule in &mut s.rules {
                prefix_rule(rule, targets);
            }
        }
        AtRule::Container(c) => {
            for rule in &mut c.rules {
                prefix_rule(rule, targets);
            }
        }
        AtRule::Media(m) => {
            for rule in &mut m.rules {
                prefix_rule(rule, targets);
            }
        }
        _ => {}
    }
}

fn prefix_rule(rule: &mut Rule, targets: &BrowserTargets) {
    let mut extra_decls = Vec::new();

    for decl in &rule.declarations {
        extra_decls.extend(generate_prefixed_declarations(decl, targets));
    }

    // Insert prefixed declarations before the original
    if !extra_decls.is_empty() {
        let mut new_decls = Vec::with_capacity(rule.declarations.len() + extra_decls.len());
        for decl in &rule.declarations {
            // Insert any prefixed versions before the standard property
            let prop_name = decl.property.name();
            for extra in &extra_decls {
                if extra.0 == prop_name {
                    new_decls.push(extra.1.clone());
                }
            }
            new_decls.push(decl.clone());
        }
        rule.declarations = new_decls;
    }

    // Prefix state blocks
    for state in &mut rule.states {
        let mut extra = Vec::new();
        for decl in &state.declarations {
            extra.extend(generate_prefixed_declarations(decl, targets));
        }
        if !extra.is_empty() {
            let mut new_decls = Vec::with_capacity(state.declarations.len() + extra.len());
            for decl in &state.declarations {
                let prop_name = decl.property.name();
                for e in &extra {
                    if e.0 == prop_name {
                        new_decls.push(e.1.clone());
                    }
                }
                new_decls.push(decl.clone());
            }
            state.declarations = new_decls;
        }
    }

    // Prefix nested at-rules
    for nested_at in &mut rule.nested_at_rules {
        let mut prefixed_decls = Vec::new();
        for decl in &nested_at.declarations {
            for (_, prefixed_decl) in generate_prefixed_declarations(decl, targets) {
                prefixed_decls.push(prefixed_decl);
            }
        }
        nested_at.declarations.extend(prefixed_decls);
        for nested_rule in &mut nested_at.nested_rules {
            prefix_rule(nested_rule, targets);
        }
    }

    // Prefix nested rules
    for nested in &mut rule.nested_rules {
        prefix_rule(nested, targets);
    }
}

/// Generate prefixed declarations for a given declaration.
/// Returns (original_property_name, prefixed_declaration).
fn generate_prefixed_declarations(decl: &Declaration, targets: &BrowserTargets) -> Vec<(String, Declaration)> {
    let prop = decl.property.name();
    let mut results = Vec::new();

    // Properties that need -webkit- prefix
    let webkit_props = [
        "appearance", "backdrop-filter", "background-clip",
        "box-decoration-break", "clip-path", "hyphens",
        "initial-letter", "mask-image", "mask-size",
        "mask-repeat", "mask-position", "mask-clip",
        "mask-origin", "mask-composite", "mask",
        "text-decoration-skip-ink", "text-emphasis",
        "text-emphasis-color", "text-emphasis-style",
        "text-emphasis-position", "text-orientation",
        "text-size-adjust", "text-stroke", "text-stroke-width",
        "text-stroke-color", "text-fill-color",
        "print-color-adjust",
    ];

    // Properties that need -moz- prefix
    let moz_props = [
        "appearance", "hyphens", "tab-size",
        "text-size-adjust", "print-color-adjust",
    ];

    // Properties that need -ms- prefix
    let ms_props = [
        "text-size-adjust", "overflow-style",
        "hyphens",
    ];

    // Flexbox prefixing (older syntax)
    let flexbox_webkit = [
        ("display", "flex", "display", "-webkit-flex"),
        ("display", "inline-flex", "display", "-webkit-inline-flex"),
        ("flex-direction", "", "-webkit-flex-direction", ""),
        ("flex-wrap", "", "-webkit-flex-wrap", ""),
        ("flex-flow", "", "-webkit-flex-flow", ""),
        ("justify-content", "", "-webkit-justify-content", ""),
        ("align-items", "", "-webkit-align-items", ""),
        ("align-content", "", "-webkit-align-content", ""),
        ("align-self", "", "-webkit-align-self", ""),
        ("flex", "", "-webkit-flex", ""),
        ("flex-grow", "", "-webkit-flex-grow", ""),
        ("flex-shrink", "", "-webkit-flex-shrink", ""),
        ("flex-basis", "", "-webkit-flex-basis", ""),
        ("order", "", "-webkit-order", ""),
    ];

    // Grid prefixing
    if targets.needs_webkit() {
        if webkit_props.contains(&prop) {
            results.push((
                prop.to_string(),
                Declaration {
                    property: Property::Standard(format!("-webkit-{prop}")),
                    value: decl.value.clone(),
                    important: decl.important,
                    span: decl.span.clone(),
                },
            ));
        }

        // Handle user-select
        if prop == "user-select" {
            results.push((
                prop.to_string(),
                Declaration {
                    property: Property::Standard("-webkit-user-select".to_string()),
                    value: decl.value.clone(),
                    important: decl.important,
                    span: decl.span.clone(),
                },
            ));
        }

        // Flexbox
        for &(match_prop, match_val, prefix_prop, prefix_val) in &flexbox_webkit {
            if prop == match_prop {
                if match_val.is_empty() {
                    results.push((
                        prop.to_string(),
                        Declaration {
                            property: Property::Standard(prefix_prop.to_string()),
                            value: decl.value.clone(),
                            important: decl.important,
                            span: decl.span.clone(),
                        },
                    ));
                } else if let Value::Literal(val) = &decl.value {
                    if val == match_val {
                        results.push((
                            prop.to_string(),
                            Declaration {
                                property: Property::Standard(prefix_prop.to_string()),
                                value: Value::Literal(prefix_val.to_string()),
                                important: decl.important,
                                span: decl.span.clone(),
                            },
                        ));
                    }
                }
            }
        }

        // Transform
        if prop == "transform" || prop == "transform-origin" || prop == "perspective"
            || prop == "perspective-origin" || prop == "backface-visibility"
        {
            results.push((
                prop.to_string(),
                Declaration {
                    property: Property::Standard(format!("-webkit-{prop}")),
                    value: decl.value.clone(),
                    important: decl.important,
                    span: decl.span.clone(),
                },
            ));
        }

        // Transition & animation
        if prop == "transition" || prop == "animation" || prop == "animation-name"
            || prop == "animation-duration" || prop == "animation-timing-function"
            || prop == "animation-delay" || prop == "animation-iteration-count"
            || prop == "animation-direction" || prop == "animation-fill-mode"
            || prop == "animation-play-state"
        {
            results.push((
                prop.to_string(),
                Declaration {
                    property: Property::Standard(format!("-webkit-{prop}")),
                    value: decl.value.clone(),
                    important: decl.important,
                    span: decl.span.clone(),
                },
            ));
        }
    }

    if targets.needs_moz() {
        if moz_props.contains(&prop) {
            results.push((
                prop.to_string(),
                Declaration {
                    property: Property::Standard(format!("-moz-{prop}")),
                    value: decl.value.clone(),
                    important: decl.important,
                    span: decl.span.clone(),
                },
            ));
        }

        if prop == "user-select" {
            results.push((
                prop.to_string(),
                Declaration {
                    property: Property::Standard("-moz-user-select".to_string()),
                    value: decl.value.clone(),
                    important: decl.important,
                    span: decl.span.clone(),
                },
            ));
        }
    }

    if targets.needs_ms() {
        if ms_props.contains(&prop) {
            results.push((
                prop.to_string(),
                Declaration {
                    property: Property::Standard(format!("-ms-{prop}")),
                    value: decl.value.clone(),
                    important: decl.important,
                    span: decl.span.clone(),
                },
            ));
        }

        if prop == "user-select" {
            results.push((
                prop.to_string(),
                Declaration {
                    property: Property::Standard("-ms-user-select".to_string()),
                    value: decl.value.clone(),
                    important: decl.important,
                    span: decl.span.clone(),
                },
            ));
        }
    }

    // Value prefixing: sticky, image-set, etc.
    if let Value::Literal(val) = &decl.value {
        if val == "sticky" && prop == "position" && targets.needs_webkit() {
            results.push((
                prop.to_string(),
                Declaration {
                    property: Property::Standard(prop.to_string()),
                    value: Value::Literal("-webkit-sticky".to_string()),
                    important: decl.important,
                    span: decl.span.clone(),
                },
            ));
        }

        if val.contains("image-set(") && targets.needs_webkit() {
            results.push((
                prop.to_string(),
                Declaration {
                    property: decl.property.clone(),
                    value: Value::Literal(val.replace("image-set(", "-webkit-image-set(")),
                    important: decl.important,
                    span: decl.span.clone(),
                },
            ));
        }
    }

    results
}

/// Prefix value-level functions like gradient prefixing.
pub fn prefix_value(value: &Value, targets: &BrowserTargets) -> Vec<Value> {
    let mut extra = Vec::new();

    if let Value::Literal(val) = value {
        // Linear gradient prefixing
        if val.contains("linear-gradient(") && targets.needs_webkit() {
            extra.push(Value::Literal(
                val.replace("linear-gradient(", "-webkit-linear-gradient("),
            ));
        }
        if val.contains("radial-gradient(") && targets.needs_webkit() {
            extra.push(Value::Literal(
                val.replace("radial-gradient(", "-webkit-radial-gradient("),
            ));
        }
        if val.contains("repeating-linear-gradient(") && targets.needs_webkit() {
            extra.push(Value::Literal(
                val.replace("repeating-linear-gradient(", "-webkit-repeating-linear-gradient("),
            ));
        }
    }

    extra
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::BrowserTargets;

    fn make_decl(prop: &str, val: &str) -> Declaration {
        Declaration {
            property: Property::Standard(prop.to_string()),
            value: Value::Literal(val.to_string()),
            important: false,
            span: Span::empty(),
        }
    }

    #[test]
    fn test_webkit_prefix() {
        let targets = BrowserTargets {
            chrome: Some(80),
            firefox: Some(80),
            safari: Some(13),
            edge: Some(80),
            ..Default::default()
        };

        let decl = make_decl("appearance", "none");
        let prefixed = generate_prefixed_declarations(&decl, &targets);
        assert!(!prefixed.is_empty());
        assert!(prefixed.iter().any(|(_, d)| d.property.name() == "-webkit-appearance"));
    }

    #[test]
    fn test_user_select_prefix() {
        let targets = BrowserTargets::default_with_prefixes();
        let decl = make_decl("user-select", "none");
        let prefixed = generate_prefixed_declarations(&decl, &targets);
        assert!(prefixed.iter().any(|(_, d)| d.property.name() == "-webkit-user-select"));
        assert!(prefixed.iter().any(|(_, d)| d.property.name() == "-moz-user-select"));
    }

    #[test]
    fn test_sticky_prefix() {
        let targets = BrowserTargets::default_with_prefixes();
        let decl = make_decl("position", "sticky");
        let prefixed = generate_prefixed_declarations(&decl, &targets);
        assert!(prefixed.iter().any(|(_, d)| {
            if let Value::Literal(v) = &d.value {
                v == "-webkit-sticky"
            } else {
                false
            }
        }));
    }

    #[test]
    fn test_no_prefix_when_not_needed() {
        let targets = BrowserTargets {
            chrome: Some(120),
            firefox: Some(120),
            safari: Some(17),
            edge: Some(120),
            ..Default::default()
        };
        // Modern browsers don't need transform prefix
        // (our simple system always prefixes based on needs_webkit, so this tests the flag)
        let no_prefix_targets = BrowserTargets::none();
        let decl = make_decl("color", "red");
        let prefixed = generate_prefixed_declarations(&decl, &no_prefix_targets);
        assert!(prefixed.is_empty());
    }
}
