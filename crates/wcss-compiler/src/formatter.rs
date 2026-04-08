use crate::ast::*;

/// Format a StyleSheet AST back to WCSS source code.
pub fn format_stylesheet(stylesheet: &StyleSheet) -> String {
    let mut output = String::new();

    // Format at-rules
    for (i, at_rule) in stylesheet.at_rules.iter().enumerate() {
        if i > 0 || !stylesheet.rules.is_empty() {
            output.push('\n');
        }
        format_at_rule(&mut output, at_rule, 0);
    }

    // Format rules
    for (i, rule) in stylesheet.rules.iter().enumerate() {
        if i > 0 || !stylesheet.at_rules.is_empty() {
            output.push('\n');
        }
        format_rule(&mut output, rule, 0);
    }

    output
}

fn format_at_rule(output: &mut String, at_rule: &AtRule, indent: usize) {
    let prefix = "  ".repeat(indent);

    match at_rule {
        AtRule::Import(imp) => {
            output.push_str(&format!("{prefix}@import \"{}\"", imp.url));
            if let Some(ref media) = imp.media {
                output.push(' ');
                output.push_str(media);
            }
            output.push_str(";\n");
        }
        AtRule::Charset(charset, _) => {
            output.push_str(&format!("{prefix}@charset \"{charset}\";\n"));
        }
        AtRule::Namespace(ns, _) => {
            output.push_str(&format!("{prefix}@namespace {ns};\n"));
        }
        AtRule::Layer(layer) => {
            output.push_str(&format!("{prefix}@layer {}", layer.name));
            if let Some(rules) = &layer.rules {
                output.push_str(" {\n");
                for rule in rules {
                    format_rule(output, rule, indent + 1);
                }
                output.push_str(&format!("{prefix}}}\n"));
            } else {
                output.push_str(";\n");
            }
        }
        AtRule::Keyframes(kf) => {
            output.push_str(&format!("{prefix}@keyframes {} {{\n", kf.name));
            for keyframe in &kf.keyframes {
                let inner = "  ".repeat(indent + 1);
                output.push_str(&inner);
                for (i, sel) in keyframe.selectors.iter().enumerate() {
                    if i > 0 { output.push_str(", "); }
                    match sel {
                        KeyframeSelector::From => output.push_str("from"),
                        KeyframeSelector::To => output.push_str("to"),
                        KeyframeSelector::Percentage(p) => {
                            if *p == (*p as i64) as f64 {
                                output.push_str(&format!("{}%", *p as i64));
                            } else {
                                output.push_str(&format!("{p}%"));
                            }
                        }
                    }
                }
                output.push_str(" {\n");
                let decl_prefix = "  ".repeat(indent + 2);
                for decl in &keyframe.declarations {
                    format_declaration(output, decl, &decl_prefix);
                }
                output.push_str(&format!("{inner}}}\n"));
            }
            output.push_str(&format!("{prefix}}}\n"));
        }
        AtRule::FontFace(ff) => {
            output.push_str(&format!("{prefix}@font-face {{\n"));
            let inner = "  ".repeat(indent + 1);
            for decl in &ff.declarations {
                format_declaration(output, decl, &inner);
            }
            output.push_str(&format!("{prefix}}}\n"));
        }
        AtRule::Supports(s) => {
            output.push_str(&format!("{prefix}@supports ({}) {{\n", s.condition));
            for rule in &s.rules {
                format_rule(output, rule, indent + 1);
            }
            output.push_str(&format!("{prefix}}}\n"));
        }
        AtRule::Container(c) => {
            output.push_str(&format!("{prefix}@container "));
            if let Some(name) = &c.name {
                output.push_str(name);
                output.push(' ');
            }
            output.push_str(&format!("({}) {{\n", c.condition));
            for rule in &c.rules {
                format_rule(output, rule, indent + 1);
            }
            output.push_str(&format!("{prefix}}}\n"));
        }
        AtRule::Media(m) => {
            output.push_str(&format!("{prefix}@media {} {{\n", m.query));
            for rule in &m.rules {
                format_rule(output, rule, indent + 1);
            }
            output.push_str(&format!("{prefix}}}\n"));
        }
        AtRule::Property(p) => {
            output.push_str(&format!("{prefix}@property {} {{\n", p.name));
            let inner = "  ".repeat(indent + 1);
            if let Some(ref syntax) = p.syntax {
                output.push_str(&format!("{inner}syntax: \"{syntax}\";\n"));
            }
            if let Some(inherits) = p.inherits {
                output.push_str(&format!("{inner}inherits: {};\n", if inherits { "true" } else { "false" }));
            }
            if let Some(ref initial) = p.initial_value {
                output.push_str(&format!("{inner}initial-value: {initial};\n"));
            }
            output.push_str(&format!("{prefix}}}\n"));
        }
        AtRule::Scope(s) => {
            output.push_str(&format!("{prefix}@scope "));
            if !s.root.is_empty() {
                output.push_str(&format!("({}))", s.root));
            }
            if let Some(ref limit) = s.limit {
                output.push_str(&format!(" to ({limit})"));
            }
            output.push_str(" {\n");
            for rule in &s.rules {
                format_rule(output, rule, indent + 1);
            }
            output.push_str(&format!("{prefix}}}\n"));
        }
        AtRule::Tailwind(tw) => {
            let directive_name = match tw.directive_type {
                TailwindDirectiveType::Base => "base",
                TailwindDirectiveType::Components => "components",
                TailwindDirectiveType::Utilities => "utilities",
                TailwindDirectiveType::Variants => "variants",
                TailwindDirectiveType::Screens => "screens",
            };
            output.push_str(&format!("{prefix}@tailwind {directive_name};\n"));
        }
        AtRule::Theme(theme) => {
            if theme.content.is_empty() {
                output.push_str(&format!("{prefix}@theme;\n"));
            } else {
                output.push_str(&format!("{prefix}@theme {{\n"));
                for line in theme.content.lines() {
                    if !line.trim().is_empty() {
                        output.push_str(&format!("{}  {}\n", prefix, line.trim()));
                    }
                }
                output.push_str(&format!("{prefix}}}\n"));
            }
        }
        AtRule::Utility(u) => {
            output.push_str(&format!("{prefix}@utility {} {{\n", u.name));
            for line in u.content.lines() {
                if !line.trim().is_empty() {
                    output.push_str(&format!("{}  {}\n", prefix, line.trim()));
                }
            }
            output.push_str(&format!("{prefix}}}\n"));
        }
        AtRule::Variant(v) | AtRule::CustomVariant(v) => {
            let directive = if matches!(at_rule, AtRule::CustomVariant(_)) { "@custom-variant" } else { "@variant" };
            if v.content.is_empty() {
                output.push_str(&format!("{prefix}{directive} {};\n", v.name));
            } else {
                output.push_str(&format!("{prefix}{directive} {} {{\n", v.name));
                for line in v.content.lines() {
                    if !line.trim().is_empty() {
                        output.push_str(&format!("{}  {}\n", prefix, line.trim()));
                    }
                }
                output.push_str(&format!("{prefix}}}\n"));
            }
        }
        AtRule::Source(path, _) => {
            output.push_str(&format!("{prefix}@source \"{path}\";\n"));
        }
        AtRule::Plugin(name, _) => {
            output.push_str(&format!("{prefix}@plugin \"{name}\";\n"));
        }
        AtRule::Config(path, _) => {
            output.push_str(&format!("{prefix}@config \"{path}\";\n"));
        }
        AtRule::Page(page) => {
            output.push_str(&format!("{prefix}@page"));
            if let Some(ref sel) = page.selector {
                output.push_str(&format!(" {sel}"));
            }
            output.push_str(" {\n");
            let inner = "  ".repeat(indent + 1);
            for decl in &page.declarations {
                format_declaration(output, decl, &inner);
            }
            output.push_str(&format!("{prefix}}}\n"));
        }
    }
}

fn format_rule(output: &mut String, rule: &Rule, indent: usize) {
    let prefix = "  ".repeat(indent);

    // Selector
    output.push_str(&format!("{prefix}{}", format_full_selector(&rule.selector)));
    for sel in &rule.selectors {
        output.push_str(&format!(", {}", format_full_selector(sel)));
    }
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

    // Nested at-rules
    for nested_at in &rule.nested_at_rules {
        output.push('\n');
        let at_keyword = match nested_at.kind {
            NestedAtRuleKind::Media => "@media",
            NestedAtRuleKind::Supports => "@supports",
            NestedAtRuleKind::Container => "@container",
        };
        output.push_str(&format!("{inner_prefix}{} {} {{\n", at_keyword, nested_at.query));
        let nested_prefix = "  ".repeat(indent + 2);
        for decl in &nested_at.declarations {
            format_declaration(output, decl, &nested_prefix);
        }
        for nested_rule in &nested_at.nested_rules {
            format_rule(output, nested_rule, indent + 2);
        }
        output.push_str(&format!("{inner_prefix}}}\n"));
    }

    // Nested rules
    for nested in &rule.nested_rules {
        output.push('\n');
        format_rule(output, nested, indent + 1);
    }

    output.push_str(&format!("{prefix}}}\n"));
}

fn format_declaration(output: &mut String, decl: &Declaration, prefix: &str) {
    match &decl.property {
        Property::Apply(classes) => {
            // Format @apply directive
            output.push_str(&format!("{prefix}@apply {classes};\n"));
        }
        _ => {
            let prop = decl.property.name();
            let val = format_value(&decl.value);
            let important = if decl.important { " !important" } else { "" };
            output.push_str(&format!("{prefix}{prop}: {val}{important};\n"));
        }
    }
}

fn format_full_selector(selector: &Selector) -> String {
    let mut result = String::new();

    match selector.kind {
        SelectorKind::Class => {
            result.push('.');
            result.push_str(&selector.class_name);
        }
        SelectorKind::Id => {
            result.push('#');
            result.push_str(&selector.class_name);
        }
        SelectorKind::Tag => {
            result.push_str(&selector.class_name);
        }
        SelectorKind::Universal => {
            result.push('*');
        }
        SelectorKind::Nesting => {
            result.push('&');
        }
        SelectorKind::Attribute => {}
    }

    for attr in &selector.attributes {
        result.push('[');
        result.push_str(&attr.name);
        if let Some(ref op) = attr.operator {
            match op {
                AttributeOp::Equals => result.push('='),
                AttributeOp::Contains => result.push_str("~="),
                AttributeOp::DashMatch => result.push_str("|="),
                AttributeOp::Prefix => result.push_str("^="),
                AttributeOp::Suffix => result.push_str("$="),
                AttributeOp::Substring => result.push_str("*="),
            }
            if let Some(ref val) = attr.value {
                result.push('"');
                result.push_str(val);
                result.push('"');
            }
        }
        result.push(']');
    }

    for pc in &selector.pseudo_classes {
        result.push_str(&format_pseudo_class(pc));
    }

    for pe in &selector.pseudo_elements {
        result.push_str(&format!("::{}", pseudo_element_name(pe)));
    }

    for combinator in &selector.combinators {
        match combinator {
            Combinator::Descendant(sel) => {
                result.push_str(&format!(" {}", format_full_selector(sel)));
            }
            Combinator::Child(sel) => {
                result.push_str(&format!(" > {}", format_full_selector(sel)));
            }
            Combinator::Adjacent(sel) => {
                result.push_str(&format!(" + {}", format_full_selector(sel)));
            }
            Combinator::Sibling(sel) => {
                result.push_str(&format!(" ~ {}", format_full_selector(sel)));
            }
        }
    }

    result
}

fn format_pseudo_class(pc: &PseudoClass) -> String {
    match pc {
        PseudoClass::Hover => ":hover".to_string(),
        PseudoClass::Focus => ":focus".to_string(),
        PseudoClass::FocusVisible => ":focus-visible".to_string(),
        PseudoClass::FocusWithin => ":focus-within".to_string(),
        PseudoClass::Active => ":active".to_string(),
        PseudoClass::Visited => ":visited".to_string(),
        PseudoClass::Link => ":link".to_string(),
        PseudoClass::Disabled => ":disabled".to_string(),
        PseudoClass::Enabled => ":enabled".to_string(),
        PseudoClass::Checked => ":checked".to_string(),
        PseudoClass::Indeterminate => ":indeterminate".to_string(),
        PseudoClass::Required => ":required".to_string(),
        PseudoClass::Optional => ":optional".to_string(),
        PseudoClass::Valid => ":valid".to_string(),
        PseudoClass::Invalid => ":invalid".to_string(),
        PseudoClass::ReadOnly => ":read-only".to_string(),
        PseudoClass::ReadWrite => ":read-write".to_string(),
        PseudoClass::PlaceholderShown => ":placeholder-shown".to_string(),
        PseudoClass::Default => ":default".to_string(),
        PseudoClass::FirstChild => ":first-child".to_string(),
        PseudoClass::LastChild => ":last-child".to_string(),
        PseudoClass::OnlyChild => ":only-child".to_string(),
        PseudoClass::FirstOfType => ":first-of-type".to_string(),
        PseudoClass::LastOfType => ":last-of-type".to_string(),
        PseudoClass::OnlyOfType => ":only-of-type".to_string(),
        PseudoClass::Empty => ":empty".to_string(),
        PseudoClass::Root => ":root".to_string(),
        PseudoClass::Dark => ":dark".to_string(),
        PseudoClass::NthChild(s) => format!(":nth-child({s})"),
        PseudoClass::NthLastChild(s) => format!(":nth-last-child({s})"),
        PseudoClass::NthOfType(s) => format!(":nth-of-type({s})"),
        PseudoClass::NthLastOfType(s) => format!(":nth-last-of-type({s})"),
        PseudoClass::Not(s) => format!(":not({s})"),
        PseudoClass::Is(s) => format!(":is({s})"),
        PseudoClass::Where(s) => format!(":where({s})"),
        PseudoClass::Has(s) => format!(":has({s})"),
        PseudoClass::Lang(s) => format!(":lang({s})"),
        PseudoClass::Dir(s) => format!(":dir({s})"),
        PseudoClass::Custom(s) => format!(":{s}"),
    }
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
            Color::Hwb(h, w, b) => format!("hwb({h} {w}% {b}%)"),
            Color::Lab(l, a, b) => format!("lab({l} {a} {b})"),
            Color::Lch(l, c, h) => format!("lch({l} {c} {h})"),
            Color::Oklch(l, c, h) => format!("oklch({l} {c} {h})"),
            Color::Oklab(l, a, b) => format!("oklab({l} {a} {b})"),
            Color::Named(name) => name.clone(),
            Color::ColorMix(expr) => format!("color-mix({expr})"),
            Color::LightDark(light, dark) => {
                format!("light-dark({}, {})", format_color(light), format_color(dark))
            }
            Color::CurrentColor => "currentColor".to_string(),
            Color::Transparent => "transparent".to_string(),
        },
        Value::Computed(expr) => format_expr(expr),
        Value::List(values) => values.iter().map(format_value).collect::<Vec<_>>().join(" "),
        Value::Var(name, fallback) => {
            if let Some(fb) = fallback {
                format!("var({name}, {})", format_value(fb))
            } else {
                format!("var({name})")
            }
        }
        Value::Env(name, fallback) => {
            if let Some(fb) = fallback {
                format!("env({name}, {})", format_value(fb))
            } else {
                format!("env({name})")
            }
        }
    }
}

fn format_color(color: &Color) -> String {
    format_value(&Value::Color(color.clone()))
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
        StateModifier::FocusVisible => "focus-visible",
        StateModifier::FocusWithin => "focus-within",
        StateModifier::Active => "active",
        StateModifier::Visited => "visited",
        StateModifier::Disabled => "disabled",
        StateModifier::Enabled => "enabled",
        StateModifier::Checked => "checked",
        StateModifier::Indeterminate => "indeterminate",
        StateModifier::Required => "required",
        StateModifier::Optional => "optional",
        StateModifier::Valid => "valid",
        StateModifier::Invalid => "invalid",
        StateModifier::ReadOnly => "read-only",
        StateModifier::ReadWrite => "read-write",
        StateModifier::PlaceholderShown => "placeholder-shown",
        StateModifier::Default => "default",
        StateModifier::FirstChild => "first-child",
        StateModifier::LastChild => "last-child",
        StateModifier::OnlyChild => "only-child",
        StateModifier::FirstOfType => "first-of-type",
        StateModifier::LastOfType => "last-of-type",
        StateModifier::OnlyOfType => "only-of-type",
        StateModifier::Empty => "empty",
        StateModifier::Dark => "dark",
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
        PseudoElement::Marker => "marker",
        PseudoElement::Backdrop => "backdrop",
        PseudoElement::Cue => "cue",
        PseudoElement::CueRegion => "cue-region",
        PseudoElement::GrammarError => "grammar-error",
        PseudoElement::SpellingError => "spelling-error",
        PseudoElement::TargetText => "target-text",
        PseudoElement::FileSelectorButton => "file-selector-button",
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
                nested_at_rules: vec![],
                span: Span::empty(),
            }],
            at_rules: vec![],
            span: Span::empty(),
        };

        let formatted = format_stylesheet(&stylesheet);
        assert!(formatted.contains(".btn {"));
        assert!(formatted.contains("  color: red;"));
        assert!(formatted.contains("}"));
    }

    #[test]
    fn test_format_id_selector() {
        let stylesheet = StyleSheet {
            rules: vec![Rule {
                selector: Selector {
                    class_name: "main".to_string(),
                    kind: SelectorKind::Id,
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
                nested_at_rules: vec![],
                span: Span::empty(),
            }],
            at_rules: vec![],
            span: Span::empty(),
        };

        let formatted = format_stylesheet(&stylesheet);
        assert!(formatted.contains("#main {"));
    }
}
