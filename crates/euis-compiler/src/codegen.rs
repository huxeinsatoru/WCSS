use crate::ast::*;
use crate::config::{CompilerConfig, DarkModeStrategy};

/// Generate CSS from an optimized stylesheet AST.
pub fn generate_css(stylesheet: &StyleSheet, config: &CompilerConfig) -> String {
    let estimated_size = stylesheet.rules.len() * 60 + stylesheet.at_rules.len() * 100;
    let mut output = String::with_capacity(estimated_size);
    let minify = config.minify;

    // Generate at-rules first (@import, @charset, @namespace must come first)
    for at_rule in &stylesheet.at_rules {
        generate_at_rule(&mut output, at_rule, config, minify, 0);
        if !minify {
            output.push('\n');
        }
    }

    // Generate style rules
    for (i, rule) in stylesheet.rules.iter().enumerate() {
        if i > 0 && !minify {
            output.push('\n');
        }
        generate_rule(&mut output, rule, config, minify, 0);
    }

    output
}

// ---------------------------------------------------------------------------
// At-rule generation
// ---------------------------------------------------------------------------

fn generate_at_rule(output: &mut String, at_rule: &AtRule, config: &CompilerConfig, minify: bool, indent: usize) {
    let prefix = if minify { String::new() } else { "  ".repeat(indent) };

    match at_rule {
        AtRule::Import(imp) => {
            output.push_str(&prefix);
            output.push_str("@import ");
            output.push_str(&format!("url(\"{}\")", imp.url));
            if let Some(ref media) = imp.media {
                output.push(' ');
                output.push_str(media);
            }
            output.push(';');
            if !minify { output.push('\n'); }
        }

        AtRule::Charset(charset, _) => {
            output.push_str(&prefix);
            output.push_str(&format!("@charset \"{charset}\";"));
            if !minify { output.push('\n'); }
        }

        AtRule::Namespace(ns, _) => {
            output.push_str(&prefix);
            output.push_str(&format!("@namespace {ns};"));
            if !minify { output.push('\n'); }
        }

        AtRule::Layer(layer) => {
            output.push_str(&prefix);
            output.push_str("@layer ");
            output.push_str(&layer.name);
            if let Some(rules) = &layer.rules {
                if minify {
                    output.push('{');
                } else {
                    output.push_str(" {\n");
                }
                for rule in rules {
                    generate_rule(output, rule, config, minify, indent + 1);
                }
                output.push_str(&prefix);
                output.push('}');
            } else {
                output.push(';');
            }
            if !minify { output.push('\n'); }
        }

        AtRule::Keyframes(kf) => {
            output.push_str(&prefix);
            if minify {
                output.push_str(&format!("@keyframes {}", kf.name));
                output.push('{');
            } else {
                output.push_str(&format!("@keyframes {} {{\n", kf.name));
            }

            for keyframe in &kf.keyframes {
                let inner_prefix = if minify { String::new() } else { "  ".repeat(indent + 1) };
                output.push_str(&inner_prefix);

                // Keyframe selectors
                for (i, sel) in keyframe.selectors.iter().enumerate() {
                    if i > 0 {
                        if minify { output.push(','); } else { output.push_str(", "); }
                    }
                    match sel {
                        KeyframeSelector::From => output.push_str("from"),
                        KeyframeSelector::To => output.push_str("to"),
                        KeyframeSelector::Percentage(p) => {
                            write_number(output, *p, minify);
                            output.push('%');
                        }
                    }
                }

                if minify {
                    output.push('{');
                } else {
                    output.push_str(" {\n");
                }

                let decl_prefix = if minify { String::new() } else { "  ".repeat(indent + 2) };
                for decl in &keyframe.declarations {
                    output.push_str(&decl_prefix);
                    write_declaration(output, decl, minify);
                    if !minify { output.push('\n'); }
                }

                output.push_str(&inner_prefix);
                output.push('}');
                if !minify { output.push('\n'); }
            }

            output.push_str(&prefix);
            output.push('}');
            if !minify { output.push('\n'); }
        }

        AtRule::FontFace(ff) => {
            output.push_str(&prefix);
            if minify {
                output.push_str("@font-face{");
            } else {
                output.push_str("@font-face {\n");
            }

            let inner_prefix = if minify { String::new() } else { "  ".repeat(indent + 1) };
            for decl in &ff.declarations {
                output.push_str(&inner_prefix);
                write_declaration(output, decl, minify);
                if !minify { output.push('\n'); }
            }

            output.push_str(&prefix);
            output.push('}');
            if !minify { output.push('\n'); }
        }

        AtRule::Supports(s) => {
            output.push_str(&prefix);
            if minify {
                output.push_str(&format!("@supports({})", s.condition));
                output.push('{');
            } else {
                output.push_str(&format!("@supports ({}) {{\n", s.condition));
            }

            for rule in &s.rules {
                generate_rule(output, rule, config, minify, indent + 1);
            }

            output.push_str(&prefix);
            output.push('}');
            if !minify { output.push('\n'); }
        }

        AtRule::Container(c) => {
            output.push_str(&prefix);
            output.push_str("@container ");
            if let Some(name) = &c.name {
                output.push_str(name);
                output.push(' ');
            }
            if minify {
                output.push_str(&format!("({})", c.condition));
                output.push('{');
            } else {
                output.push_str(&format!("({}) {{\n", c.condition));
            }

            for rule in &c.rules {
                generate_rule(output, rule, config, minify, indent + 1);
            }

            output.push_str(&prefix);
            output.push('}');
            if !minify { output.push('\n'); }
        }

        AtRule::Media(m) => {
            output.push_str(&prefix);
            if minify {
                output.push_str(&format!("@media {}", m.query));
                output.push('{');
            } else {
                output.push_str(&format!("@media {} {{\n", m.query));
            }

            for rule in &m.rules {
                generate_rule(output, rule, config, minify, indent + 1);
            }

            output.push_str(&prefix);
            output.push('}');
            if !minify { output.push('\n'); }
        }

        AtRule::Property(p) => {
            output.push_str(&prefix);
            if minify {
                output.push_str(&format!("@property {}", p.name));
                output.push('{');
            } else {
                output.push_str(&format!("@property {} {{\n", p.name));
            }

            let inner_prefix = if minify { String::new() } else { "  ".repeat(indent + 1) };
            if let Some(ref syntax) = p.syntax {
                output.push_str(&inner_prefix);
                output.push_str("syntax:");
                if !minify { output.push(' '); }
                output.push_str(&format!("\"{syntax}\""));
                output.push(';');
                if !minify { output.push('\n'); }
            }
            if let Some(inherits) = p.inherits {
                output.push_str(&inner_prefix);
                output.push_str("inherits:");
                if !minify { output.push(' '); }
                output.push_str(if inherits { "true" } else { "false" });
                output.push(';');
                if !minify { output.push('\n'); }
            }
            if let Some(ref initial) = p.initial_value {
                output.push_str(&inner_prefix);
                output.push_str("initial-value:");
                if !minify { output.push(' '); }
                output.push_str(initial);
                output.push(';');
                if !minify { output.push('\n'); }
            }

            output.push_str(&prefix);
            output.push('}');
            if !minify { output.push('\n'); }
        }

        AtRule::Scope(s) => {
            output.push_str(&prefix);
            output.push_str("@scope ");
            if !s.root.is_empty() {
                output.push('(');
                output.push_str(&s.root);
                output.push(')');
            }
            if let Some(ref limit) = s.limit {
                output.push_str(" to (");
                output.push_str(limit);
                output.push(')');
            }
            if minify {
                output.push('{');
            } else {
                output.push_str(" {\n");
            }

            for rule in &s.rules {
                generate_rule(output, rule, config, minify, indent + 1);
            }

            output.push_str(&prefix);
            output.push('}');
            if !minify { output.push('\n'); }
        }

        AtRule::Tailwind(tw) => {
            // Generate Tailwind directive as-is (pass-through)
            output.push_str(&prefix);
            output.push_str("@tailwind ");
            let directive_name = match tw.directive_type {
                TailwindDirectiveType::Base => "base",
                TailwindDirectiveType::Components => "components",
                TailwindDirectiveType::Utilities => "utilities",
                TailwindDirectiveType::Variants => "variants",
                TailwindDirectiveType::Screens => "screens",
            };
            output.push_str(directive_name);
            output.push(';');
            if !minify { output.push('\n'); }
        }

        AtRule::Theme(theme) => {
            output.push_str(&prefix);
            if theme.content.is_empty() {
                output.push_str("@theme;");
            } else if minify {
                output.push_str("@theme{");
                output.push_str(&theme.content);
                output.push('}');
            } else {
                output.push_str("@theme {\n");
                // Indent content lines
                for line in theme.content.lines() {
                    if !line.trim().is_empty() {
                        output.push_str(&"  ".repeat(indent + 1));
                        output.push_str(line.trim());
                        output.push('\n');
                    }
                }
                output.push_str(&prefix);
                output.push_str("}\n");
            }
        }

        AtRule::Utility(u) => {
            output.push_str(&prefix);
            if minify {
                output.push_str(&format!("@utility {}", u.name));
                output.push('{');
                output.push_str(&u.content);
                output.push('}');
            } else {
                output.push_str(&format!("@utility {} {{\n", u.name));
                for line in u.content.lines() {
                    if !line.trim().is_empty() {
                        output.push_str(&"  ".repeat(indent + 1));
                        output.push_str(line.trim());
                        output.push('\n');
                    }
                }
                output.push_str(&prefix);
                output.push_str("}\n");
            }
        }

        AtRule::Variant(v) | AtRule::CustomVariant(v) => {
            let directive = if matches!(at_rule, AtRule::CustomVariant(_)) { "@custom-variant" } else { "@variant" };
            output.push_str(&prefix);
            if v.content.is_empty() {
                output.push_str(&format!("{} {};", directive, v.name));
            } else if minify {
                output.push_str(&format!("{} {}", directive, v.name));
                output.push('{');
                output.push_str(&v.content);
                output.push('}');
            } else {
                output.push_str(&format!("{} {} {{\n", directive, v.name));
                for line in v.content.lines() {
                    if !line.trim().is_empty() {
                        output.push_str(&"  ".repeat(indent + 1));
                        output.push_str(line.trim());
                        output.push('\n');
                    }
                }
                output.push_str(&prefix);
                output.push_str("}\n");
            }
        }

        AtRule::Source(path, _) => {
            output.push_str(&prefix);
            output.push_str(&format!("@source \"{path}\";"));
            if !minify { output.push('\n'); }
        }

        AtRule::Plugin(name, _) => {
            output.push_str(&prefix);
            output.push_str(&format!("@plugin \"{name}\";"));
            if !minify { output.push('\n'); }
        }

        AtRule::Config(path, _) => {
            output.push_str(&prefix);
            output.push_str(&format!("@config \"{path}\";"));
            if !minify { output.push('\n'); }
        }

        AtRule::Page(page) => {
            output.push_str(&prefix);
            output.push_str("@page");
            if let Some(ref sel) = page.selector {
                output.push(' ');
                output.push_str(sel);
            }
            if minify { output.push('{'); } else { output.push_str(" {\n"); }
            let inner_prefix = if minify { String::new() } else { "  ".repeat(indent + 1) };
            for decl in &page.declarations {
                output.push_str(&inner_prefix);
                write_declaration(output, decl, minify);
                if !minify { output.push('\n'); }
            }
            output.push_str(&prefix);
            output.push('}');
            if !minify { output.push('\n'); }
        }
    }
}

// ---------------------------------------------------------------------------
// Rule generation (with multi-selector and dark mode support)
// ---------------------------------------------------------------------------

fn generate_rule(output: &mut String, rule: &Rule, config: &CompilerConfig, minify: bool, indent: usize) {
    let prefix = if minify { String::new() } else { "  ".repeat(indent) };

    // === Base declarations ===
    if !rule.declarations.is_empty() {
        output.push_str(&prefix);
        write_selector_list(output, rule, minify);
        if minify { output.push('{'); } else { output.push_str(" {\n"); }

        let inner_prefix = if minify { String::new() } else { "  ".repeat(indent + 1) };
        for decl in &rule.declarations {
            output.push_str(&inner_prefix);
            write_declaration(output, decl, minify);
            if !minify { output.push('\n'); }
        }

        output.push_str(&prefix);
        output.push('}');
        if !minify { output.push('\n'); }
    }

    // === State blocks ===
    for state in &rule.states {
        // Check for dark mode modifier
        let has_dark = state.modifiers.iter().any(|m| matches!(m, StateModifier::Dark));
        let non_dark_modifiers: Vec<&StateModifier> = state.modifiers.iter()
            .filter(|m| !matches!(m, StateModifier::Dark))
            .collect();

        if has_dark {
            // Generate dark mode wrapper
            let dark_selector = match &config.dark_mode {
                DarkModeStrategy::Media => {
                    // Wrap in @media (prefers-color-scheme: dark)
                    if minify {
                        output.push_str("@media(prefers-color-scheme:dark){");
                    } else {
                        output.push_str(&prefix);
                        output.push_str("@media (prefers-color-scheme: dark) {\n");
                    }
                    output.push_str(&prefix);
                    if !minify { output.push_str("  "); }
                    write_selector_list(output, rule, minify);
                    for m in &non_dark_modifiers {
                        write_pseudo(output, m);
                    }
                    None
                }
                DarkModeStrategy::Class(class) => {
                    output.push_str(&prefix);
                    output.push_str(&format!(".{class} "));
                    write_selector_list(output, rule, minify);
                    for m in &non_dark_modifiers {
                        write_pseudo(output, m);
                    }
                    Some(())
                }
                DarkModeStrategy::Attribute(attr) => {
                    output.push_str(&prefix);
                    output.push_str(&format!("[{attr}] "));
                    write_selector_list(output, rule, minify);
                    for m in &non_dark_modifiers {
                        write_pseudo(output, m);
                    }
                    Some(())
                }
            };

            if minify { output.push('{'); } else { output.push_str(" {\n"); }
            let inner = if minify { String::new() } else {
                if dark_selector.is_none() { "  ".repeat(indent + 2) } else { "  ".repeat(indent + 1) }
            };
            for decl in &state.declarations {
                output.push_str(&inner);
                write_declaration(output, decl, minify);
                if !minify { output.push('\n'); }
            }

            if dark_selector.is_none() {
                // Close both media and rule blocks
                if minify {
                    output.push_str("}}");
                } else {
                    output.push_str(&prefix);
                    output.push_str("  }\n");
                    output.push_str(&prefix);
                    output.push_str("}\n");
                }
            } else {
                output.push_str(&prefix);
                output.push('}');
                if !minify { output.push('\n'); }
            }
        } else {
            // Regular state block
            output.push_str(&prefix);
            write_selector_list(output, rule, minify);
            for m in &state.modifiers {
                write_pseudo(output, m);
            }
            if minify { output.push('{'); } else { output.push_str(" {\n"); }

            let inner_prefix = if minify { String::new() } else { "  ".repeat(indent + 1) };
            for decl in &state.declarations {
                output.push_str(&inner_prefix);
                write_declaration(output, decl, minify);
                if !minify { output.push('\n'); }
            }

            output.push_str(&prefix);
            output.push('}');
            if !minify { output.push('\n'); }
        }
    }

    // === Responsive blocks ===
    for responsive in &rule.responsive {
        let bp_value = config.tokens.breakpoints
            .get(&responsive.breakpoint)
            .and_then(|v| v.as_literal())
            .unwrap_or("0px");

        if minify {
            output.push_str(&format!("@media(min-width:{bp_value}){{"));
        } else {
            output.push_str(&prefix);
            output.push_str(&format!("@media (min-width: {bp_value}) {{\n"));
        }

        output.push_str(&prefix);
        if !minify { output.push_str("  "); }
        write_selector_list(output, rule, minify);
        if minify { output.push('{'); } else { output.push_str(" {\n"); }

        let decl_prefix = if minify { String::new() } else { "  ".repeat(indent + 2) };
        for decl in &responsive.declarations {
            output.push_str(&decl_prefix);
            write_declaration(output, decl, minify);
            if !minify { output.push('\n'); }
        }

        if minify {
            output.push_str("}}");
        } else {
            output.push_str(&prefix);
            output.push_str("  }\n");
            output.push_str(&prefix);
            output.push_str("}\n");
        }
    }

    // === Nested at-rules (@media, @supports, @container inside rules) ===
    for nested_at in &rule.nested_at_rules {
        let at_keyword = match nested_at.kind {
            NestedAtRuleKind::Media => "@media",
            NestedAtRuleKind::Supports => "@supports",
            NestedAtRuleKind::Container => "@container",
        };
        output.push_str(&prefix);
        if minify {
            output.push_str(&format!("{} {}", at_keyword, nested_at.query));
            output.push('{');
        } else {
            output.push_str(&format!("  {} {} {{\n", at_keyword, nested_at.query));
        }

        let inner_prefix = if minify { String::new() } else { "  ".repeat(indent + 2) };
        for decl in &nested_at.declarations {
            output.push_str(&inner_prefix);
            write_declaration(output, decl, minify);
            if !minify { output.push('\n'); }
        }
        for nested_rule in &nested_at.nested_rules {
            generate_rule(output, nested_rule, config, minify, indent + 2);
        }

        if minify {
            output.push('}');
        } else {
            output.push_str(&prefix);
            output.push_str("  }\n");
        }
    }

    // === Nested rules ===
    for nested in &rule.nested_rules {
        generate_rule(output, nested, config, minify, indent + 1);
    }
}

// ---------------------------------------------------------------------------
// Selector list generation (multi-selector support)
// ---------------------------------------------------------------------------

fn write_selector_list(output: &mut String, rule: &Rule, minify: bool) {
    write_full_selector(output, &rule.selector);

    for (_i, sel) in rule.selectors.iter().enumerate() {
        if minify {
            output.push(',');
        } else {
            output.push_str(", ");
        }
        write_full_selector(output, sel);
    }
}

fn write_full_selector(output: &mut String, selector: &Selector) {
    match selector.kind {
        SelectorKind::Class => {
            output.push('.');
            output.push_str(&selector.class_name);
        }
        SelectorKind::Id => {
            output.push('#');
            output.push_str(&selector.class_name);
        }
        SelectorKind::Tag => {
            output.push_str(&selector.class_name);
        }
        SelectorKind::Universal => {
            output.push('*');
        }
        SelectorKind::Nesting => {
            output.push('&');
        }
        SelectorKind::Attribute => {
            // Attribute-only selector (no element name)
        }
    }

    // Attribute selectors
    for attr in &selector.attributes {
        output.push('[');
        output.push_str(&attr.name);
        if let Some(ref op) = attr.operator {
            match op {
                AttributeOp::Equals => output.push('='),
                AttributeOp::Contains => output.push_str("~="),
                AttributeOp::DashMatch => output.push_str("|="),
                AttributeOp::Prefix => output.push_str("^="),
                AttributeOp::Suffix => output.push_str("$="),
                AttributeOp::Substring => output.push_str("*="),
            }
            if let Some(ref val) = attr.value {
                output.push('"');
                output.push_str(val);
                output.push('"');
            }
        }
        if let Some(ref modifier) = attr.modifier {
            output.push(' ');
            match modifier {
                AttributeModifier::CaseInsensitive => output.push('i'),
                AttributeModifier::CaseSensitive => output.push('s'),
            }
        }
        output.push(']');
    }

    // Pseudo-classes
    for pc in &selector.pseudo_classes {
        write_pseudo_class(output, pc);
    }

    // Pseudo-elements
    for pe in &selector.pseudo_elements {
        write_pseudo_element(output, pe);
    }

    // Combinators
    for combinator in &selector.combinators {
        match combinator {
            Combinator::Descendant(sel) => {
                output.push(' ');
                write_full_selector(output, sel);
            }
            Combinator::Child(sel) => {
                output.push_str(" > ");
                write_full_selector(output, sel);
            }
            Combinator::Adjacent(sel) => {
                output.push_str(" + ");
                write_full_selector(output, sel);
            }
            Combinator::Sibling(sel) => {
                output.push_str(" ~ ");
                write_full_selector(output, sel);
            }
        }
    }
}

fn write_pseudo_class(output: &mut String, pc: &PseudoClass) {
    match pc {
        PseudoClass::Hover => output.push_str(":hover"),
        PseudoClass::Focus => output.push_str(":focus"),
        PseudoClass::FocusVisible => output.push_str(":focus-visible"),
        PseudoClass::FocusWithin => output.push_str(":focus-within"),
        PseudoClass::Active => output.push_str(":active"),
        PseudoClass::Visited => output.push_str(":visited"),
        PseudoClass::Link => output.push_str(":link"),
        PseudoClass::Disabled => output.push_str(":disabled"),
        PseudoClass::Enabled => output.push_str(":enabled"),
        PseudoClass::Checked => output.push_str(":checked"),
        PseudoClass::Indeterminate => output.push_str(":indeterminate"),
        PseudoClass::Required => output.push_str(":required"),
        PseudoClass::Optional => output.push_str(":optional"),
        PseudoClass::Valid => output.push_str(":valid"),
        PseudoClass::Invalid => output.push_str(":invalid"),
        PseudoClass::ReadOnly => output.push_str(":read-only"),
        PseudoClass::ReadWrite => output.push_str(":read-write"),
        PseudoClass::PlaceholderShown => output.push_str(":placeholder-shown"),
        PseudoClass::Default => output.push_str(":default"),
        PseudoClass::FirstChild => output.push_str(":first-child"),
        PseudoClass::LastChild => output.push_str(":last-child"),
        PseudoClass::OnlyChild => output.push_str(":only-child"),
        PseudoClass::FirstOfType => output.push_str(":first-of-type"),
        PseudoClass::LastOfType => output.push_str(":last-of-type"),
        PseudoClass::OnlyOfType => output.push_str(":only-of-type"),
        PseudoClass::Empty => output.push_str(":empty"),
        PseudoClass::Root => output.push_str(":root"),
        PseudoClass::Dark => {} // Handled by dark mode codegen
        PseudoClass::NthChild(s) => { output.push_str(":nth-child("); output.push_str(s); output.push(')'); }
        PseudoClass::NthLastChild(s) => { output.push_str(":nth-last-child("); output.push_str(s); output.push(')'); }
        PseudoClass::NthOfType(s) => { output.push_str(":nth-of-type("); output.push_str(s); output.push(')'); }
        PseudoClass::NthLastOfType(s) => { output.push_str(":nth-last-of-type("); output.push_str(s); output.push(')'); }
        PseudoClass::Not(s) => { output.push_str(":not("); output.push_str(s); output.push(')'); }
        PseudoClass::Is(s) => { output.push_str(":is("); output.push_str(s); output.push(')'); }
        PseudoClass::Where(s) => { output.push_str(":where("); output.push_str(s); output.push(')'); }
        PseudoClass::Has(s) => { output.push_str(":has("); output.push_str(s); output.push(')'); }
        PseudoClass::Lang(s) => { output.push_str(":lang("); output.push_str(s); output.push(')'); }
        PseudoClass::Dir(s) => { output.push_str(":dir("); output.push_str(s); output.push(')'); }
        PseudoClass::Custom(s) => { output.push(':'); output.push_str(s); }
    }
}

fn write_pseudo_element(output: &mut String, pe: &PseudoElement) {
    match pe {
        PseudoElement::Before => output.push_str("::before"),
        PseudoElement::After => output.push_str("::after"),
        PseudoElement::FirstLine => output.push_str("::first-line"),
        PseudoElement::FirstLetter => output.push_str("::first-letter"),
        PseudoElement::Placeholder => output.push_str("::placeholder"),
        PseudoElement::Selection => output.push_str("::selection"),
        PseudoElement::Marker => output.push_str("::marker"),
        PseudoElement::Backdrop => output.push_str("::backdrop"),
        PseudoElement::Cue => output.push_str("::cue"),
        PseudoElement::CueRegion => output.push_str("::cue-region"),
        PseudoElement::GrammarError => output.push_str("::grammar-error"),
        PseudoElement::SpellingError => output.push_str("::spelling-error"),
        PseudoElement::TargetText => output.push_str("::target-text"),
        PseudoElement::FileSelectorButton => output.push_str("::file-selector-button"),
        PseudoElement::Custom(name) => {
            output.push_str("::");
            output.push_str(name);
        }
    }
}

// ---------------------------------------------------------------------------
// Declaration / Value writing
// ---------------------------------------------------------------------------

fn write_declaration(output: &mut String, decl: &Declaration, minify: bool) {
    match &decl.property {
        Property::Apply(classes) => {
            // Write @apply directive
            output.push_str("@apply ");
            output.push_str(classes);
            output.push(';');
        }
        _ => {
            output.push_str(decl.property.name());
            if minify { output.push(':'); } else { output.push_str(": "); }
            write_value(output, &decl.value, minify);
            if decl.important {
                if minify { output.push_str(" !important"); } else { output.push_str(" !important"); }
            }
            output.push(';');
        }
    }
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
        Value::Var(name, fallback) => {
            output.push_str("var(");
            output.push_str(name);
            if let Some(fb) = fallback {
                output.push(',');
                write_value_minified(output, fb);
            }
            output.push(')');
        }
        Value::Env(name, fallback) => {
            output.push_str("env(");
            output.push_str(name);
            if let Some(fb) = fallback {
                output.push(',');
                write_value_minified(output, fb);
            }
            output.push(')');
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
        Value::Var(name, fallback) => {
            output.push_str("var(");
            output.push_str(name);
            if let Some(fb) = fallback {
                output.push_str(", ");
                write_value_normal(output, fb);
            }
            output.push(')');
        }
        Value::Env(name, fallback) => {
            output.push_str("env(");
            output.push_str(name);
            if let Some(fb) = fallback {
                output.push_str(", ");
                write_value_normal(output, fb);
            }
            output.push(')');
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
            if ri >> 4 == (ri & 0xf) && gi >> 4 == (gi & 0xf) && bi >> 4 == (bi & 0xf) {
                output.push('#');
                let _ = write!(output, "{:x}{:x}{:x}", ri & 0xf, gi & 0xf, bi & 0xf);
            } else {
                let _ = write!(output, "#{:02x}{:02x}{:02x}", ri, gi, bi);
            }
        }
        Color::CurrentColor => output.push_str("currentColor"),
        Color::Transparent => output.push_str("transparent"),
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
        Color::Hwb(h, w, b) => { let _ = write!(output, "hwb({h} {w}% {b}%)"); }
        Color::Lab(l, a, b) => { let _ = write!(output, "lab({l} {a} {b})"); }
        Color::Lch(l, c, h) => { let _ = write!(output, "lch({l} {c} {h})"); }
        Color::Oklch(l, c, h) => { let _ = write!(output, "oklch({l} {c} {h})"); }
        Color::Oklab(l, a, b) => { let _ = write!(output, "oklab({l} {a} {b})"); }
        Color::Named(name) => output.push_str(name),
        Color::ColorMix(expr) => { output.push_str("color-mix("); output.push_str(expr); output.push(')'); }
        Color::LightDark(light, dark) => {
            output.push_str("light-dark(");
            write_color_normal(output, light);
            output.push_str(", ");
            write_color_normal(output, dark);
            output.push(')');
        }
        Color::CurrentColor => output.push_str("currentColor"),
        Color::Transparent => output.push_str("transparent"),
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

#[inline]
fn write_pseudo(output: &mut String, modifier: &StateModifier) {
    match modifier {
        StateModifier::Hover => output.push_str(":hover"),
        StateModifier::Focus => output.push_str(":focus"),
        StateModifier::FocusVisible => output.push_str(":focus-visible"),
        StateModifier::FocusWithin => output.push_str(":focus-within"),
        StateModifier::Active => output.push_str(":active"),
        StateModifier::Visited => output.push_str(":visited"),
        StateModifier::Disabled => output.push_str(":disabled"),
        StateModifier::Enabled => output.push_str(":enabled"),
        StateModifier::Checked => output.push_str(":checked"),
        StateModifier::Indeterminate => output.push_str(":indeterminate"),
        StateModifier::Required => output.push_str(":required"),
        StateModifier::Optional => output.push_str(":optional"),
        StateModifier::Valid => output.push_str(":valid"),
        StateModifier::Invalid => output.push_str(":invalid"),
        StateModifier::ReadOnly => output.push_str(":read-only"),
        StateModifier::ReadWrite => output.push_str(":read-write"),
        StateModifier::PlaceholderShown => output.push_str(":placeholder-shown"),
        StateModifier::Default => output.push_str(":default"),
        StateModifier::FirstChild => output.push_str(":first-child"),
        StateModifier::LastChild => output.push_str(":last-child"),
        StateModifier::OnlyChild => output.push_str(":only-child"),
        StateModifier::FirstOfType => output.push_str(":first-of-type"),
        StateModifier::LastOfType => output.push_str(":last-of-type"),
        StateModifier::OnlyOfType => output.push_str(":only-of-type"),
        StateModifier::Empty => output.push_str(":empty"),
        StateModifier::Dark => {} // handled by dark mode codegen
        StateModifier::Custom(s) => {
            output.push(':');
            output.push_str(s);
        }
    }
}

// ---------------------------------------------------------------------------
// Typed OM
// ---------------------------------------------------------------------------

/// Generate JavaScript code using CSS Typed OM API.
pub fn generate_typed_om_js(stylesheet: &StyleSheet) -> String {
    let mut output = String::with_capacity(stylesheet.rules.len() * 100 + 500);

    output.push_str("// Euis Typed OM Runtime - Auto-generated\nconst euis = {\n  apply(element, className) {\n    const styles = this._styles[className];\n    if (!styles || !element.attributeStyleMap) return;\n    for (const [prop, value] of Object.entries(styles)) {\n      element.attributeStyleMap.set(prop, value);\n    }\n  },\n  update(element, property, value) {\n    if (element.attributeStyleMap) {\n      element.attributeStyleMap.set(property, value);\n    }\n  },\n  remove(element, properties) {\n    if (!element.attributeStyleMap) return;\n    for (const prop of properties) {\n      element.attributeStyleMap.delete(prop);\n    }\n  },\n  get(element, property) {\n    return element.attributeStyleMap?.get(property) ?? null;\n  },\n  _styles: {\n");

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

    output.push_str("  },\n};\nexport default euis;\n");
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
        Value::Number(n, Some(Unit::Dvh)) => { let _ = write!(output, "CSS.dvh({n})"); }
        Value::Number(n, Some(Unit::Dvw)) => { let _ = write!(output, "CSS.dvw({n})"); }
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
                nested_rules: vec![],
                nested_at_rules: vec![],
                span: Span::empty(),
            }],
            at_rules: vec![],
            span: Span::empty(),
        };

        let config = CompilerConfig { minify: false, ..Default::default() };
        let css = generate_css(&stylesheet, &config);
        assert!(css.contains(".button"));
        assert!(css.contains("color: red;"));
        assert!(css.contains("padding: 10px;"));
    }

    #[test]
    fn test_generate_multi_selector() {
        let stylesheet = StyleSheet {
            rules: vec![Rule {
                selector: Selector {
                    class_name: "a".to_string(),
                    kind: SelectorKind::Class,
                    combinators: vec![],
                    pseudo_elements: vec![],
                    pseudo_classes: vec![],
                    attributes: vec![],
                    span: Span::empty(),
                },
                selectors: vec![
                    Selector {
                        class_name: "b".to_string(),
                        kind: SelectorKind::Class,
                        combinators: vec![],
                        pseudo_elements: vec![],
                        pseudo_classes: vec![],
                        attributes: vec![],
                        span: Span::empty(),
                    },
                ],
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

        let config = CompilerConfig { minify: false, ..Default::default() };
        let css = generate_css(&stylesheet, &config);
        assert!(css.contains(".a, .b"));
    }

    #[test]
    fn test_generate_keyframes() {
        let stylesheet = StyleSheet {
            rules: vec![],
            at_rules: vec![AtRule::Keyframes(KeyframesRule {
                name: "fadeIn".to_string(),
                keyframes: vec![
                    Keyframe {
                        selectors: vec![KeyframeSelector::From],
                        declarations: vec![Declaration {
                            property: Property::Standard("opacity".to_string()),
                            value: Value::Number(0.0, None),
                            important: false,
                            span: Span::empty(),
                        }],
                        span: Span::empty(),
                    },
                    Keyframe {
                        selectors: vec![KeyframeSelector::To],
                        declarations: vec![Declaration {
                            property: Property::Standard("opacity".to_string()),
                            value: Value::Number(1.0, None),
                            important: false,
                            span: Span::empty(),
                        }],
                        span: Span::empty(),
                    },
                ],
                span: Span::empty(),
            })],
            span: Span::empty(),
        };

        let config = CompilerConfig { minify: false, ..Default::default() };
        let css = generate_css(&stylesheet, &config);
        assert!(css.contains("@keyframes fadeIn"));
        assert!(css.contains("from"));
        assert!(css.contains("to"));
    }
}
