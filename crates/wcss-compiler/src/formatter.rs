use crate::ast::*;

/// Format a StyleSheet AST back to WCSS source code.
pub fn format_stylesheet(stylesheet: &StyleSheet) -> String {
    let mut output = String::new();

    for (i, rule) in stylesheet.rules.iter().enumerate() {
        if i > 0 {
            output.push('\n');
        }
        format_rule(&mut output, rule, 0);
    }

    output
}

fn format_rule(output: &mut String, rule: &Rule, indent: usize) {
    let prefix = "  ".repeat(indent);

    // Selector
    output.push_str(&format!("{prefix}.{}", format_selector(&rule.selector)));
    output.push_str(" {\n");

    let inner_prefix = "  ".repeat(indent + 1);

    // Declarations
    for decl in &rule.declarations {
        format_declaration(output, decl, &inner_prefix);
    }

    // State blocks
    for state in &rule.states {
        output.push('\n');
        let pseudo = state
            .modifiers
            .iter()
            .map(|m| format!(":{}", state_modifier_name(m)))
            .collect::<String>();
        output.push_str(&format!("{inner_prefix}&{pseudo} {{\n"));
        let state_prefix = "  ".repeat(indent + 2);
        for decl in &state.declarations {
            format_declaration(output, decl, &state_prefix);
        }
        output.push_str(&format!("{inner_prefix}}}\n"));
    }

    // Responsive blocks
    for responsive in &rule.responsive {
        output.push('\n');
        output.push_str(&format!("{inner_prefix}@{} {{\n", responsive.breakpoint));
        let resp_prefix = "  ".repeat(indent + 2);
        for decl in &responsive.declarations {
            format_declaration(output, decl, &resp_prefix);
        }
        output.push_str(&format!("{inner_prefix}}}\n"));
    }

    output.push_str(&format!("{prefix}}}\n"));
}

fn format_declaration(output: &mut String, decl: &Declaration, prefix: &str) {
    let prop = decl.property.name();
    let val = format_value(&decl.value);
    let important = if decl.important { " !important" } else { "" };
    output.push_str(&format!("{prefix}{prop}: {val}{important};\n"));
}

fn format_selector(selector: &Selector) -> String {
    let mut result = selector.class_name.clone();

    for combinator in &selector.combinators {
        match combinator {
            Combinator::Descendant(sel) => {
                result.push_str(&format!(" .{}", format_selector(sel)));
            }
            Combinator::Child(sel) => {
                result.push_str(&format!(" > .{}", format_selector(sel)));
            }
            Combinator::Adjacent(sel) => {
                result.push_str(&format!(" + .{}", format_selector(sel)));
            }
            Combinator::Sibling(sel) => {
                result.push_str(&format!(" ~ .{}", format_selector(sel)));
            }
        }
    }

    for pe in &selector.pseudo_elements {
        result.push_str(&format!("::{}", pseudo_element_name(pe)));
    }

    result
}

fn format_value(value: &Value) -> String {
    match value {
        Value::Token(token_ref) => {
            format!("${}.{}", token_ref.category.as_str(), token_ref.name)
        }
        Value::Literal(s) => s.clone(),
        Value::Number(n, unit) => {
            let num = if *n == (*n as i64) as f64 {
                format!("{}", *n as i64)
            } else {
                format!("{n}")
            };
            match unit {
                Some(u) => format!("{num}{}", u.as_str()),
                None => num,
            }
        }
        Value::Color(color) => match color {
            Color::Hex(hex) => hex.clone(),
            Color::Rgb(r, g, b) => format!("rgb({r}, {g}, {b})"),
            Color::Rgba(r, g, b, a) => format!("rgba({r}, {g}, {b}, {a})"),
            Color::Hsl(h, s, l) => format!("hsl({h}, {s}%, {l}%)"),
            Color::Hsla(h, s, l, a) => format!("hsla({h}, {s}%, {l}%, {a})"),
            Color::Named(name) => name.clone(),
        },
        Value::Computed(expr) => format_expr(expr),
        Value::List(values) => values.iter().map(format_value).collect::<Vec<_>>().join(" "),
    }
}

fn format_expr(expr: &Expr) -> String {
    match expr {
        Expr::Value(v) => format_value(v),
        Expr::Add(a, b) => format!("{} + {}", format_expr(a), format_expr(b)),
        Expr::Sub(a, b) => format!("{} - {}", format_expr(a), format_expr(b)),
        Expr::Mul(a, b) => format!("{} * {}", format_expr(a), format_expr(b)),
        Expr::Div(a, b) => format!("{} / {}", format_expr(a), format_expr(b)),
        Expr::Function(name, args) => {
            let arg_strs: Vec<String> = args.iter().map(format_expr).collect();
            format!("{name}({})", arg_strs.join(", "))
        }
    }
}

fn state_modifier_name(modifier: &StateModifier) -> &str {
    match modifier {
        StateModifier::Hover => "hover",
        StateModifier::Focus => "focus",
        StateModifier::Active => "active",
        StateModifier::Visited => "visited",
        StateModifier::Disabled => "disabled",
        StateModifier::Checked => "checked",
        StateModifier::FirstChild => "first-child",
        StateModifier::LastChild => "last-child",
        StateModifier::Custom(s) => s,
    }
}

fn pseudo_element_name(pe: &PseudoElement) -> &str {
    match pe {
        PseudoElement::Before => "before",
        PseudoElement::After => "after",
        PseudoElement::FirstLine => "first-line",
        PseudoElement::FirstLetter => "first-letter",
        PseudoElement::Placeholder => "placeholder",
        PseudoElement::Selection => "selection",
        PseudoElement::Custom(s) => s,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_simple_rule() {
        let stylesheet = StyleSheet {
            rules: vec![Rule {
                selector: Selector {
                    class_name: "btn".to_string(),
                    combinators: vec![],
                    pseudo_elements: vec![],
                    span: Span::empty(),
                },
                declarations: vec![Declaration {
                    property: Property::Standard("color".to_string()),
                    value: Value::Literal("red".to_string()),
                    important: false,
                    span: Span::empty(),
                }],
                states: vec![],
                responsive: vec![],
                span: Span::empty(),
            }],
            span: Span::empty(),
        };

        let formatted = format_stylesheet(&stylesheet);
        assert!(formatted.contains(".btn {"));
        assert!(formatted.contains("  color: red;"));
        assert!(formatted.contains("}"));
    }
}
