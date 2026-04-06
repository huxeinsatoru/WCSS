use crate::ast::*;
use crate::config::CompilerConfig;

/// Generate CSS from an optimized stylesheet AST.
pub fn generate_css(stylesheet: &StyleSheet, config: &CompilerConfig) -> String {
    // Pre-allocate: ~60 bytes per rule is a reasonable estimate
    let estimated_size = stylesheet.rules.len() * 60;
    let mut output = String::with_capacity(estimated_size);
    let minify = config.minify;

    for (i, rule) in stylesheet.rules.iter().enumerate() {
        if i > 0 && !minify {
            output.push('\n');
        }
        generate_rule(&mut output, rule, config, minify);
    }

    output
}

#[inline]
fn write_value(output: &mut String, value: &Value, minify: bool) {
    if minify {
        write_value_minified(output, value);
    } else {
        write_value_normal(output, value);
    }
}

fn write_value_minified(output: &mut String, value: &Value) {
    match value {
        Value::Color(color) => write_color_minified(output, color),
        Value::Number(n, unit) => {
            if *n == 0.0 && unit.is_some() {
                output.push('0');
                return;
            }
            write_number(output, *n, true);
            if let Some(u) = unit {
                output.push_str(u.as_str());
            }
        }
        Value::Literal(s) => {
            // Inline hex shortening for literal values containing hex colors
            if s.len() == 7 && s.starts_with('#') {
                let b = s.as_bytes();
                if b[1] == b[2] && b[3] == b[4] && b[5] == b[6]
                    && b[1..].iter().all(|c| c.is_ascii_hexdigit())
                {
                    output.push('#');
                    output.push(b[1] as char);
                    output.push(b[3] as char);
                    output.push(b[5] as char);
                    return;
                }
            }
            output.push_str(s);
        }
        Value::Token(token_ref) => {
            output.push_str("var(--");
            output.push_str(token_ref.category.as_str());
            output.push('-');
            output.push_str(&token_ref.name);
            output.push(')');
        }
        Value::Computed(expr) => write_expr(output, expr),
        Value::List(values) => {
            for (i, v) in values.iter().enumerate() {
                if i > 0 { output.push(' '); }
                write_value_minified(output, v);
            }
        }
    }
}

fn write_value_normal(output: &mut String, value: &Value) {
    match value {
        Value::Token(token_ref) => {
            output.push_str("var(--");
            output.push_str(token_ref.category.as_str());
            output.push('-');
            output.push_str(&token_ref.name);
            output.push(')');
        }
        Value::Literal(s) => output.push_str(s),
        Value::Number(n, unit) => {
            write_number(output, *n, false);
            if let Some(u) = unit {
                output.push_str(u.as_str());
            }
        }
        Value::Color(color) => write_color_normal(output, color),
        Value::Computed(expr) => write_expr(output, expr),
        Value::List(values) => {
            for (i, v) in values.iter().enumerate() {
                if i > 0 { output.push(' '); }
                write_value_normal(output, v);
            }
        }
    }
}

#[inline]
fn write_number(output: &mut String, n: f64, minify: bool) {
    use std::fmt::Write;
    if n == (n as i64) as f64 {
        let _ = write!(output, "{}", n as i64);
    } else if minify {
        let s = format!("{n}");
        if s.starts_with("0.") {
            output.push_str(&s[1..]);
        } else if s.starts_with("-0.") {
            output.push('-');
            output.push_str(&s[2..]);
        } else {
            output.push_str(&s);
        }
    } else {
        let _ = write!(output, "{n}");
    }
}

fn write_color_minified(output: &mut String, color: &Color) {
    match color {
        Color::Hex(hex) => {
            let b = hex.as_bytes();
            if b.len() == 7 && b[0] == b'#'
                && b[1] == b[2] && b[3] == b[4] && b[5] == b[6]
            {
                let short = [b'#', b[1], b[3], b[5]];
                // Check for keyword shortcut
                match &short {
                    b"#f00" | b"#F00" => { output.push_str("red"); return; }
                    b"#0f0" | b"#0F0" => { output.push_str("lime"); return; }
                    b"#00f" | b"#00F" => { output.push_str("blue"); return; }
                    _ => {
                        output.push('#');
                        output.push(b[1] as char);
                        output.push(b[3] as char);
                        output.push(b[5] as char);
                        return;
                    }
                }
            }
            output.push_str(hex);
        }
        Color::Rgb(r, g, b) => {
            use std::fmt::Write;
            let ri = *r as u8;
            let gi = *g as u8;
            let bi = *b as u8;
            // Check if shortenable
            if ri >> 4 == (ri & 0xf) && gi >> 4 == (gi & 0xf) && bi >> 4 == (bi & 0xf) {
                output.push('#');
                let _ = write!(output, "{:x}{:x}{:x}", ri & 0xf, gi & 0xf, bi & 0xf);
            } else {
                let _ = write!(output, "#{:02x}{:02x}{:02x}", ri, gi, bi);
            }
        }
        Color::Named(name) => output.push_str(name),
        _ => write_color_normal(output, color),
    }
}

fn write_color_normal(output: &mut String, color: &Color) {
    use std::fmt::Write;
    match color {
        Color::Hex(hex) => output.push_str(hex),
        Color::Rgb(r, g, b) => { let _ = write!(output, "rgb({r}, {g}, {b})"); }
        Color::Rgba(r, g, b, a) => { let _ = write!(output, "rgba({r}, {g}, {b}, {a})"); }
        Color::Hsl(h, s, l) => { let _ = write!(output, "hsl({h}, {s}%, {l}%)"); }
        Color::Hsla(h, s, l, a) => { let _ = write!(output, "hsla({h}, {s}%, {l}%, {a})"); }
        Color::Named(name) => output.push_str(name),
    }
}

fn write_expr(output: &mut String, expr: &Expr) {
    match expr {
        Expr::Value(v) => write_value_normal(output, v),
        Expr::Add(a, b) => {
            output.push_str("calc(");
            write_expr(output, a);
            output.push_str(" + ");
            write_expr(output, b);
            output.push(')');
        }
        Expr::Sub(a, b) => {
            output.push_str("calc(");
            write_expr(output, a);
            output.push_str(" - ");
            write_expr(output, b);
            output.push(')');
        }
        Expr::Mul(a, b) => {
            output.push_str("calc(");
            write_expr(output, a);
            output.push_str(" * ");
            write_expr(output, b);
            output.push(')');
        }
        Expr::Div(a, b) => {
            output.push_str("calc(");
            write_expr(output, a);
            output.push_str(" / ");
            write_expr(output, b);
            output.push(')');
        }
        Expr::Function(name, args) => {
            output.push_str(name);
            output.push('(');
            for (i, arg) in args.iter().enumerate() {
                if i > 0 { output.push_str(", "); }
                write_expr(output, arg);
            }
            output.push(')');
        }
    }
}

fn generate_rule(output: &mut String, rule: &Rule, config: &CompilerConfig, minify: bool) {
    let selector = &rule.selector;

    if minify {
        // === MINIFIED PATH ===
        // Base declarations
        if !rule.declarations.is_empty() {
            output.push('.');
            write_selector(output, selector);
            output.push('{');
            for decl in &rule.declarations {
                output.push_str(decl.property.name());
                output.push(':');
                write_value(output, &decl.value, true);
                if decl.important { output.push_str(" !important"); }
                output.push(';');
            }
            output.push('}');
        }

        // State blocks
        for state in &rule.states {
            output.push('.');
            write_selector(output, selector);
            for m in &state.modifiers {
                write_pseudo(output, m);
            }
            output.push('{');
            for decl in &state.declarations {
                output.push_str(decl.property.name());
                output.push(':');
                write_value(output, &decl.value, true);
                if decl.important { output.push_str(" !important"); }
                output.push(';');
            }
            output.push('}');
        }

        // Responsive blocks
        for responsive in &rule.responsive {
            let bp_value = config.tokens.breakpoints
                .get(&responsive.breakpoint)
                .and_then(|v| v.as_literal())
                .unwrap_or("0px");
            output.push_str("@media(min-width:");
            output.push_str(bp_value);
            output.push_str("){.");
            write_selector(output, selector);
            output.push('{');
            for decl in &responsive.declarations {
                output.push_str(decl.property.name());
                output.push(':');
                write_value(output, &decl.value, true);
                if decl.important { output.push_str(" !important"); }
                output.push(';');
            }
            output.push_str("}}");
        }
    } else {
        // === PRETTY PATH ===
        if !rule.declarations.is_empty() {
            output.push('.');
            write_selector(output, selector);
            output.push_str(" {\n");
            for decl in &rule.declarations {
                output.push_str("  ");
                output.push_str(decl.property.name());
                output.push_str(": ");
                write_value(output, &decl.value, false);
                if decl.important { output.push_str(" !important"); }
                output.push_str(";\n");
            }
            output.push_str("}\n");
        }

        for state in &rule.states {
            output.push('.');
            write_selector(output, selector);
            for m in &state.modifiers {
                write_pseudo(output, m);
            }
            output.push_str(" {\n");
            for decl in &state.declarations {
                output.push_str("  ");
                output.push_str(decl.property.name());
                output.push_str(": ");
                write_value(output, &decl.value, false);
                if decl.important { output.push_str(" !important"); }
                output.push_str(";\n");
            }
            output.push_str("}\n");
        }

        for responsive in &rule.responsive {
            let bp_value = config.tokens.breakpoints
                .get(&responsive.breakpoint)
                .and_then(|v| v.as_literal())
                .unwrap_or("0px");
            output.push_str("@media (min-width: ");
            output.push_str(bp_value);
            output.push_str(") {\n  .");
            write_selector(output, selector);
            output.push_str(" {\n");
            for decl in &responsive.declarations {
                output.push_str("    ");
                output.push_str(decl.property.name());
                output.push_str(": ");
                write_value(output, &decl.value, false);
                if decl.important { output.push_str(" !important"); }
                output.push_str(";\n");
            }
            output.push_str("  }\n}\n");
        }
    }
}

fn write_selector(output: &mut String, selector: &Selector) {
    output.push_str(&selector.class_name);

    for combinator in &selector.combinators {
        match combinator {
            Combinator::Descendant(sel) => {
                output.push_str(" .");
                write_selector(output, sel);
            }
            Combinator::Child(sel) => {
                output.push_str(" > .");
                write_selector(output, sel);
            }
            Combinator::Adjacent(sel) => {
                output.push_str(" + .");
                write_selector(output, sel);
            }
            Combinator::Sibling(sel) => {
                output.push_str(" ~ .");
                write_selector(output, sel);
            }
        }
    }

    for pseudo_element in &selector.pseudo_elements {
        match pseudo_element {
            PseudoElement::Before => output.push_str("::before"),
            PseudoElement::After => output.push_str("::after"),
            PseudoElement::FirstLine => output.push_str("::first-line"),
            PseudoElement::FirstLetter => output.push_str("::first-letter"),
            PseudoElement::Placeholder => output.push_str("::placeholder"),
            PseudoElement::Selection => output.push_str("::selection"),
            PseudoElement::Custom(name) => {
                output.push_str("::");
                output.push_str(name);
            }
        }
    }
}

#[inline]
fn write_pseudo(output: &mut String, modifier: &StateModifier) {
    match modifier {
        StateModifier::Hover => output.push_str(":hover"),
        StateModifier::Focus => output.push_str(":focus"),
        StateModifier::Active => output.push_str(":active"),
        StateModifier::Visited => output.push_str(":visited"),
        StateModifier::Disabled => output.push_str(":disabled"),
        StateModifier::Checked => output.push_str(":checked"),
        StateModifier::FirstChild => output.push_str(":first-child"),
        StateModifier::LastChild => output.push_str(":last-child"),
        StateModifier::Custom(s) => {
            output.push(':');
            output.push_str(s);
        }
    }
}

// Used by unit tests below
#[allow(dead_code)]
fn generate_value(value: &Value) -> String {
    let mut s = String::new();
    write_value_normal(&mut s, value);
    s
}

#[allow(dead_code)]
fn generate_selector(selector: &Selector) -> String {
    let mut s = String::new();
    write_selector(&mut s, selector);
    s
}

#[allow(dead_code)]
fn shorten_hex_colors(css: &str) -> String {
    let bytes = css.as_bytes();
    let mut result = String::with_capacity(css.len());
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'#' && i + 7 <= bytes.len() {
            let hex = &bytes[i + 1..i + 7];
            if hex.iter().all(|b| b.is_ascii_hexdigit())
                && hex[0] == hex[1]
                && hex[2] == hex[3]
                && hex[4] == hex[5]
            {
                result.push('#');
                result.push(hex[0] as char);
                result.push(hex[2] as char);
                result.push(hex[4] as char);
                i += 7;
                continue;
            }
        }
        result.push(bytes[i] as char);
        i += 1;
    }

    result
}

/// Generate JavaScript code using CSS Typed OM API.
pub fn generate_typed_om_js(stylesheet: &StyleSheet) -> String {
    let mut output = String::with_capacity(stylesheet.rules.len() * 100 + 500);

    output.push_str("// WCSS Typed OM Runtime - Auto-generated\nconst wcss = {\n  apply(element, className) {\n    const styles = this._styles[className];\n    if (!styles || !element.attributeStyleMap) return;\n    for (const [prop, value] of Object.entries(styles)) {\n      element.attributeStyleMap.set(prop, value);\n    }\n  },\n  update(element, property, value) {\n    if (element.attributeStyleMap) {\n      element.attributeStyleMap.set(property, value);\n    }\n  },\n  remove(element, properties) {\n    if (!element.attributeStyleMap) return;\n    for (const prop of properties) {\n      element.attributeStyleMap.delete(prop);\n    }\n  },\n  get(element, property) {\n    return element.attributeStyleMap?.get(property) ?? null;\n  },\n  _styles: {\n");

    for rule in &stylesheet.rules {
        use std::fmt::Write;
        let _ = write!(output, "    '{}': {{\n", rule.selector.class_name);
        for decl in &rule.declarations {
            output.push_str("      '");
            output.push_str(decl.property.name());
            output.push_str("': ");
            write_typed_om_value(&mut output, &decl.value);
            output.push_str(",\n");
        }
        output.push_str("    },\n");
    }

    output.push_str("  },\n};\nexport default wcss;\n");
    output
}

fn write_typed_om_value(output: &mut String, value: &Value) {
    use std::fmt::Write;
    match value {
        Value::Number(n, Some(Unit::Px)) => { let _ = write!(output, "CSS.px({n})"); }
        Value::Number(n, Some(Unit::Rem)) => { let _ = write!(output, "CSS.rem({n})"); }
        Value::Number(n, Some(Unit::Em)) => { let _ = write!(output, "CSS.em({n})"); }
        Value::Number(n, Some(Unit::Percent)) => { let _ = write!(output, "CSS.percent({n})"); }
        Value::Number(n, Some(Unit::Vh)) => { let _ = write!(output, "CSS.vh({n})"); }
        Value::Number(n, Some(Unit::Vw)) => { let _ = write!(output, "CSS.vw({n})"); }
        Value::Number(n, None) => { let _ = write!(output, "CSS.number({n})"); }
        _ => {
            output.push('\'');
            write_value_normal(output, value);
            output.push('\'');
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::CompilerConfig;

    #[test]
    fn test_generate_simple_css() {
        let stylesheet = StyleSheet {
            rules: vec![Rule {
                selector: Selector {
                    class_name: "button".to_string(),
                    combinators: vec![],
                    pseudo_elements: vec![],
                    span: Span::empty(),
                },
                declarations: vec![
                    Declaration {
                        property: Property::Standard("color".to_string()),
                        value: Value::Literal("red".to_string()),
                        important: false,
                        span: Span::empty(),
                    },
                    Declaration {
                        property: Property::Standard("padding".to_string()),
                        value: Value::Number(10.0, Some(Unit::Px)),
                        important: false,
                        span: Span::empty(),
                    },
                ],
                states: vec![],
                responsive: vec![],
                span: Span::empty(),
            }],
            span: Span::empty(),
        };

        let config = CompilerConfig { minify: false, ..Default::default() };
        let css = generate_css(&stylesheet, &config);
        assert!(css.contains(".button"));
        assert!(css.contains("color: red;"));
        assert!(css.contains("padding: 10px;"));
    }

    #[test]
    fn test_generate_pseudo_class() {
        let stylesheet = StyleSheet {
            rules: vec![Rule {
                selector: Selector {
                    class_name: "btn".to_string(),
                    combinators: vec![],
                    pseudo_elements: vec![],
                    span: Span::empty(),
                },
                declarations: vec![],
                states: vec![StateBlock {
                    modifiers: vec![StateModifier::Hover],
                    declarations: vec![Declaration {
                        property: Property::Standard("color".to_string()),
                        value: Value::Literal("blue".to_string()),
                        important: false,
                        span: Span::empty(),
                    }],
                    span: Span::empty(),
                }],
                responsive: vec![],
                span: Span::empty(),
            }],
            span: Span::empty(),
        };

        let config = CompilerConfig { minify: false, ..Default::default() };
        let css = generate_css(&stylesheet, &config);
        assert!(css.contains(".btn:hover"));
    }

    #[test]
    fn test_shorten_hex_colors() {
        assert_eq!(shorten_hex_colors("#ffffff"), "#fff");
        assert_eq!(shorten_hex_colors("#aabbcc"), "#abc");
        assert_eq!(shorten_hex_colors("#abcdef"), "#abcdef");
    }
}
