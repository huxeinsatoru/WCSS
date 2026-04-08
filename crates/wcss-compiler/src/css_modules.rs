//! CSS Modules support for the WCSS compiler.
//!
//! Transforms class selectors by appending a content-based hash to provide
//! local scoping, and generates export maps for JS/JSON consumption.

use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use crate::ast::*;

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Configuration for CSS Modules transformation.
#[derive(Debug, Clone)]
pub struct CSSModulesConfig {
    /// Prefix added before the hash (e.g. the project name).
    pub hash_prefix: String,
    /// Number of hex characters in the generated hash (default 6).
    pub hash_length: usize,
    /// Whether to emit a source map alongside the transformed CSS.
    pub generate_sourcemap: bool,
}

impl Default for CSSModulesConfig {
    fn default() -> Self {
        Self {
            hash_prefix: String::new(),
            hash_length: 6,
            generate_sourcemap: false,
        }
    }
}

// ---------------------------------------------------------------------------
// Result
// ---------------------------------------------------------------------------

/// The result of a CSS Modules transformation.
#[derive(Debug, Clone)]
pub struct CSSModulesResult {
    /// The transformed CSS with hashed class names.
    pub css: String,
    /// Map from original class name to hashed class name.
    pub exports: HashMap<String, String>,
    /// Optional source map (placeholder for future implementation).
    pub source_map: Option<String>,
}

// ---------------------------------------------------------------------------
// Composition entry
// ---------------------------------------------------------------------------

/// A parsed `composes:` declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct ComposesDirective {
    /// The class names being composed.
    pub class_names: Vec<String>,
    /// The source file, if any (`from './other.module.css'`).
    pub from: Option<String>,
}

/// Parse a `composes` value string.
///
/// Supported formats:
/// - `composes: className`
/// - `composes: className otherClass`
/// - `composes: className from './other.module.css'`
pub fn parse_composes(value: &str) -> ComposesDirective {
    let value = value.trim().trim_end_matches(';').trim();
    if let Some(idx) = value.find(" from ") {
        let classes_part = &value[..idx];
        let from_part = value[idx + 6..].trim();
        // Strip quotes
        let from_path = from_part
            .trim_matches('\'')
            .trim_matches('"')
            .to_string();
        let class_names = classes_part
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();
        ComposesDirective {
            class_names,
            from: Some(from_path),
        }
    } else {
        let class_names = value
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();
        ComposesDirective {
            class_names,
            from: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Hash helper
// ---------------------------------------------------------------------------

/// Generate a deterministic short hash from a file path and class name.
fn generate_hash(file_path: &str, class_name: &str, config: &CSSModulesConfig) -> String {
    // Use Rust's default SipHash via DefaultHasher for deterministic hashing.
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    config.hash_prefix.hash(&mut hasher);
    file_path.hash(&mut hasher);
    class_name.hash(&mut hasher);
    let hash_value = hasher.finish();
    let hex = format!("{:016x}", hash_value);
    hex[..config.hash_length.min(16)].to_string()
}

/// Build the hashed class name: `originalName_hash`.
fn hashed_class_name(class_name: &str, file_path: &str, config: &CSSModulesConfig) -> String {
    let hash = generate_hash(file_path, class_name, config);
    format!("{}_{}", class_name, hash)
}

// ---------------------------------------------------------------------------
// Selector analysis helpers
// ---------------------------------------------------------------------------

/// Scope mode for a class name.
#[derive(Debug, Clone, Copy, PartialEq)]
enum ScopeMode {
    /// The default: class names are hashed (equivalent to `:local()`).
    Local,
    /// Explicitly global: class names are left as-is.
    Global,
}

/// Determine the scope mode of a class name based on :local() / :global() wrapping.
/// Returns `(mode, bare_class_name)`.
fn detect_scope(class_name: &str) -> (ScopeMode, &str) {
    if let Some(inner) = class_name.strip_prefix(":global(").and_then(|s| s.strip_suffix(')')) {
        // Strip leading dot if present inside :global(.foo)
        let inner = inner.strip_prefix('.').unwrap_or(inner);
        (ScopeMode::Global, inner)
    } else if let Some(inner) = class_name.strip_prefix(":local(").and_then(|s| s.strip_suffix(')')) {
        let inner = inner.strip_prefix('.').unwrap_or(inner);
        (ScopeMode::Local, inner)
    } else {
        (ScopeMode::Local, class_name)
    }
}

// ---------------------------------------------------------------------------
// AST walking & transformation
// ---------------------------------------------------------------------------

/// Transform a parsed `StyleSheet` using CSS Modules rules.
///
/// * Class selectors are hashed (unless wrapped in `:global()`).
/// * An exports map is built mapping original -> hashed names.
/// * `composes:` declarations are parsed and reflected in the exports map.
/// * The resulting CSS string is regenerated from the (cloned) AST.
pub fn transform_css_modules(
    stylesheet: &StyleSheet,
    config: &CSSModulesConfig,
    file_path: &str,
) -> CSSModulesResult {
    let mut exports: HashMap<String, String> = HashMap::new();
    let mut compositions: HashMap<String, Vec<ComposesDirective>> = HashMap::new();

    // Clone the stylesheet so we can mutate selectors.
    let mut ss = stylesheet.clone();

    // Process all top-level rules.
    for rule in &mut ss.rules {
        process_rule(rule, config, file_path, &mut exports, &mut compositions);
    }

    // Process rules inside at-rules that contain nested rules.
    for at_rule in &mut ss.at_rules {
        match at_rule {
            AtRule::Layer(layer) => {
                if let Some(rules) = &mut layer.rules {
                    for rule in rules {
                        process_rule(rule, config, file_path, &mut exports, &mut compositions);
                    }
                }
            }
            AtRule::Supports(supports) => {
                for rule in &mut supports.rules {
                    process_rule(rule, config, file_path, &mut exports, &mut compositions);
                }
            }
            AtRule::Container(container) => {
                for rule in &mut container.rules {
                    process_rule(rule, config, file_path, &mut exports, &mut compositions);
                }
            }
            AtRule::Media(media) => {
                for rule in &mut media.rules {
                    process_rule(rule, config, file_path, &mut exports, &mut compositions);
                }
            }
            _ => {}
        }
    }

    // Fold compositions into the exports map.
    for (class, directives) in &compositions {
        let mut composed_value = exports
            .get(class)
            .cloned()
            .unwrap_or_else(|| class.clone());
        for directive in directives {
            for composed_class in &directive.class_names {
                let resolved = if directive.from.is_some() {
                    // External compositions: use the raw class name (would need
                    // a resolver in a full implementation; for now keep the name).
                    composed_class.clone()
                } else {
                    // Local composition: use the hashed name if available.
                    exports
                        .get(composed_class)
                        .cloned()
                        .unwrap_or_else(|| composed_class.clone())
                };
                composed_value = format!("{} {}", composed_value, resolved);
            }
        }
        exports.insert(class.clone(), composed_value);
    }

    // Regenerate CSS from the mutated AST.
    let css = regenerate_css(&ss);

    let source_map = if config.generate_sourcemap {
        Some(String::from("/* source map placeholder */"))
    } else {
        None
    };

    CSSModulesResult {
        css,
        exports,
        source_map,
    }
}

/// Recursively process a single rule: hash its selectors and collect exports.
fn process_rule(
    rule: &mut Rule,
    config: &CSSModulesConfig,
    file_path: &str,
    exports: &mut HashMap<String, String>,
    compositions: &mut HashMap<String, Vec<ComposesDirective>>,
) {
    // Current rule's original class name (before hashing) for composition tracking.
    let original_class = rule.selector.class_name.clone();

    // Hash primary selector.
    hash_selector(&mut rule.selector, config, file_path, exports);

    // Hash additional selectors.
    for sel in &mut rule.selectors {
        hash_selector(sel, config, file_path, exports);
    }

    // Scan declarations for `composes:`.
    let mut composes_indices = Vec::new();
    for (i, decl) in rule.declarations.iter().enumerate() {
        if decl.property.name() == "composes" {
            if let Value::Literal(ref val) = decl.value {
                let directive = parse_composes(val);
                compositions
                    .entry(original_class.clone())
                    .or_default()
                    .push(directive);
                composes_indices.push(i);
            }
        }
    }
    // Remove composes declarations from output (they are virtual).
    for i in composes_indices.into_iter().rev() {
        rule.declarations.remove(i);
    }

    // Recurse into nested rules.
    for nested in &mut rule.nested_rules {
        process_rule(nested, config, file_path, exports, compositions);
    }

    // Recurse into nested at-rules.
    for nested_at in &mut rule.nested_at_rules {
        for nested_rule in &mut nested_at.nested_rules {
            process_rule(nested_rule, config, file_path, exports, compositions);
        }
    }
}

/// Hash a single selector if it is a class selector and not :global().
fn hash_selector(
    selector: &mut Selector,
    config: &CSSModulesConfig,
    file_path: &str,
    exports: &mut HashMap<String, String>,
) {
    if selector.kind == SelectorKind::Class {
        let (mode, bare) = detect_scope(&selector.class_name);
        match mode {
            ScopeMode::Local => {
                let bare = bare.to_string();
                let hashed = hashed_class_name(&bare, file_path, config);
                exports.insert(bare.clone(), hashed.clone());
                selector.class_name = hashed;
            }
            ScopeMode::Global => {
                let bare = bare.to_string();
                exports.insert(bare.clone(), bare.clone());
                selector.class_name = bare;
            }
        }
    }

    // Also process combinators which may contain class selectors.
    for combinator in &mut selector.combinators {
        let inner = match combinator {
            Combinator::Descendant(s) => s.as_mut(),
            Combinator::Child(s) => s.as_mut(),
            Combinator::Adjacent(s) => s.as_mut(),
            Combinator::Sibling(s) => s.as_mut(),
        };
        hash_selector(inner, config, file_path, exports);
    }
}

// ---------------------------------------------------------------------------
// CSS regeneration (minimal, for CSS Modules output)
// ---------------------------------------------------------------------------

/// Regenerate CSS text from a StyleSheet AST.
fn regenerate_css(stylesheet: &StyleSheet) -> String {
    let mut out = String::new();

    for at_rule in &stylesheet.at_rules {
        regenerate_at_rule(&mut out, at_rule, 0);
    }

    for rule in &stylesheet.rules {
        regenerate_rule(&mut out, rule, 0);
    }

    out
}

fn regenerate_at_rule(out: &mut String, at_rule: &AtRule, indent: usize) {
    let pad = "  ".repeat(indent);
    match at_rule {
        AtRule::Import(imp) => {
            out.push_str(&format!("{}@import url(\"{}\");\n", pad, imp.url));
        }
        AtRule::Layer(layer) => {
            if let Some(rules) = &layer.rules {
                out.push_str(&format!("{}@layer {} {{\n", pad, layer.name));
                for rule in rules {
                    regenerate_rule(out, rule, indent + 1);
                }
                out.push_str(&format!("{}}}\n", pad));
            } else {
                out.push_str(&format!("{}@layer {};\n", pad, layer.name));
            }
        }
        AtRule::Media(media) => {
            out.push_str(&format!("{}@media {} {{\n", pad, media.query));
            for rule in &media.rules {
                regenerate_rule(out, rule, indent + 1);
            }
            out.push_str(&format!("{}}}\n", pad));
        }
        AtRule::Supports(supports) => {
            out.push_str(&format!("{}@supports {} {{\n", pad, supports.condition));
            for rule in &supports.rules {
                regenerate_rule(out, rule, indent + 1);
            }
            out.push_str(&format!("{}}}\n", pad));
        }
        AtRule::Container(container) => {
            let name = container.name.as_deref().unwrap_or("");
            out.push_str(&format!("{}@container {} ({}) {{\n", pad, name, container.condition));
            for rule in &container.rules {
                regenerate_rule(out, rule, indent + 1);
            }
            out.push_str(&format!("{}}}\n", pad));
        }
        AtRule::Keyframes(kf) => {
            out.push_str(&format!("{}@keyframes {} {{\n", pad, kf.name));
            for frame in &kf.keyframes {
                let sels: Vec<String> = frame.selectors.iter().map(|s| match s {
                    KeyframeSelector::From => "from".to_string(),
                    KeyframeSelector::To => "to".to_string(),
                    KeyframeSelector::Percentage(p) => format!("{}%", p),
                }).collect();
                out.push_str(&format!("{}  {} {{\n", pad, sels.join(", ")));
                for decl in &frame.declarations {
                    out.push_str(&format!("{}    ", pad));
                    write_decl(out, decl);
                    out.push('\n');
                }
                out.push_str(&format!("{}  }}\n", pad));
            }
            out.push_str(&format!("{}}}\n", pad));
        }
        AtRule::FontFace(ff) => {
            out.push_str(&format!("{}@font-face {{\n", pad));
            for decl in &ff.declarations {
                out.push_str(&format!("{}  ", pad));
                write_decl(out, decl);
                out.push('\n');
            }
            out.push_str(&format!("{}}}\n", pad));
        }
        AtRule::Charset(cs, _) => {
            out.push_str(&format!("{}@charset \"{}\";\n", pad, cs));
        }
        AtRule::Namespace(ns, _) => {
            out.push_str(&format!("{}@namespace {};\n", pad, ns));
        }
        AtRule::Property(prop) => {
            out.push_str(&format!("{}@property {} {{\n", pad, prop.name));
            if let Some(ref syn) = prop.syntax {
                out.push_str(&format!("{}  syntax: \"{}\";\n", pad, syn));
            }
            if let Some(inh) = prop.inherits {
                out.push_str(&format!("{}  inherits: {};\n", pad, inh));
            }
            if let Some(ref iv) = prop.initial_value {
                out.push_str(&format!("{}  initial-value: {};\n", pad, iv));
            }
            out.push_str(&format!("{}}}\n", pad));
        }
        AtRule::Scope(scope) => {
            out.push_str(&format!("{}@scope ", pad));
            if !scope.root.is_empty() {
                out.push_str(&format!("({})", scope.root));
            }
            if let Some(ref limit) = scope.limit {
                out.push_str(&format!(" to ({})", limit));
            }
            out.push_str(" {\n");
            for rule in &scope.rules {
                regenerate_rule(out, rule, indent + 1);
            }
            out.push_str(&format!("{}}}\n", pad));
        }
        AtRule::Tailwind(tw) => {
            let directive_name = match tw.directive_type {
                TailwindDirectiveType::Base => "base",
                TailwindDirectiveType::Components => "components",
                TailwindDirectiveType::Utilities => "utilities",
                TailwindDirectiveType::Variants => "variants",
                TailwindDirectiveType::Screens => "screens",
            };
            out.push_str(&format!("{}@tailwind {};\n", pad, directive_name));
        }
    }
}

fn regenerate_rule(out: &mut String, rule: &Rule, indent: usize) {
    let pad = "  ".repeat(indent);

    if !rule.declarations.is_empty() || !rule.nested_rules.is_empty() {
        out.push_str(&pad);
        write_selector_text(out, &rule.selector);
        for sel in &rule.selectors {
            out.push_str(", ");
            write_selector_text(out, sel);
        }
        out.push_str(" {\n");

        let inner_pad = "  ".repeat(indent + 1);
        for decl in &rule.declarations {
            out.push_str(&inner_pad);
            write_decl(out, decl);
            out.push('\n');
        }
        for nested in &rule.nested_rules {
            regenerate_rule(out, nested, indent + 1);
        }

        out.push_str(&pad);
        out.push_str("}\n");
    }

    // State blocks
    for state in &rule.states {
        out.push_str(&pad);
        write_selector_text(out, &rule.selector);
        for m in &state.modifiers {
            out.push_str(&m.as_css_pseudo());
        }
        out.push_str(" {\n");
        let inner_pad = "  ".repeat(indent + 1);
        for decl in &state.declarations {
            out.push_str(&inner_pad);
            write_decl(out, decl);
            out.push('\n');
        }
        out.push_str(&pad);
        out.push_str("}\n");
    }

    // Responsive blocks
    for resp in &rule.responsive {
        out.push_str(&format!("{}@media (min-width: {}) {{\n", pad, resp.breakpoint));
        out.push_str(&format!("{}  ", pad));
        write_selector_text(out, &rule.selector);
        out.push_str(" {\n");
        let inner_pad = "  ".repeat(indent + 2);
        for decl in &resp.declarations {
            out.push_str(&inner_pad);
            write_decl(out, decl);
            out.push('\n');
        }
        out.push_str(&format!("{}  }}\n", pad));
        out.push_str(&format!("{}}}\n", pad));
    }
}

fn write_selector_text(out: &mut String, selector: &Selector) {
    match selector.kind {
        SelectorKind::Class => {
            out.push('.');
            out.push_str(&selector.class_name);
        }
        SelectorKind::Id => {
            out.push('#');
            out.push_str(&selector.class_name);
        }
        SelectorKind::Tag => {
            out.push_str(&selector.class_name);
        }
        SelectorKind::Universal => {
            out.push('*');
        }
        SelectorKind::Nesting => {
            out.push('&');
        }
        SelectorKind::Attribute => {
            if !selector.attributes.is_empty() {
                write_attribute_selector(out, &selector.attributes[0]);
            }
        }
    }

    // Attribute selectors
    for attr in &selector.attributes {
        if selector.kind != SelectorKind::Attribute {
            write_attribute_selector(out, attr);
        }
    }

    // Pseudo-classes
    for pc in &selector.pseudo_classes {
        write_pseudo_class(out, pc);
    }

    // Pseudo-elements
    for pe in &selector.pseudo_elements {
        write_pseudo_element(out, pe);
    }

    // Combinators
    for comb in &selector.combinators {
        match comb {
            Combinator::Descendant(s) => {
                out.push(' ');
                write_selector_text(out, s);
            }
            Combinator::Child(s) => {
                out.push_str(" > ");
                write_selector_text(out, s);
            }
            Combinator::Adjacent(s) => {
                out.push_str(" + ");
                write_selector_text(out, s);
            }
            Combinator::Sibling(s) => {
                out.push_str(" ~ ");
                write_selector_text(out, s);
            }
        }
    }
}

fn write_attribute_selector(out: &mut String, attr: &AttributeSelector) {
    out.push('[');
    out.push_str(&attr.name);
    if let Some(ref op) = attr.operator {
        match op {
            AttributeOp::Equals => out.push('='),
            AttributeOp::Contains => out.push_str("~="),
            AttributeOp::DashMatch => out.push_str("|="),
            AttributeOp::Prefix => out.push_str("^="),
            AttributeOp::Suffix => out.push_str("$="),
            AttributeOp::Substring => out.push_str("*="),
        }
        if let Some(ref val) = attr.value {
            out.push('"');
            out.push_str(val);
            out.push('"');
        }
    }
    out.push(']');
}

fn write_pseudo_class(out: &mut String, pc: &PseudoClass) {
    match pc {
        PseudoClass::Hover => out.push_str(":hover"),
        PseudoClass::Focus => out.push_str(":focus"),
        PseudoClass::Active => out.push_str(":active"),
        PseudoClass::NthChild(n) => out.push_str(&format!(":nth-child({})", n)),
        PseudoClass::Not(s) => out.push_str(&format!(":not({})", s)),
        PseudoClass::Is(s) => out.push_str(&format!(":is({})", s)),
        PseudoClass::Where(s) => out.push_str(&format!(":where({})", s)),
        PseudoClass::Has(s) => out.push_str(&format!(":has({})", s)),
        PseudoClass::Custom(s) => out.push_str(&format!(":{}", s)),
        _ => {
            // Use Debug format as fallback for less common pseudo-classes.
            let name = format!("{:?}", pc);
            let kebab = camel_to_kebab(&name);
            out.push(':');
            out.push_str(&kebab);
        }
    }
}

fn write_pseudo_element(out: &mut String, pe: &PseudoElement) {
    match pe {
        PseudoElement::Before => out.push_str("::before"),
        PseudoElement::After => out.push_str("::after"),
        PseudoElement::FirstLine => out.push_str("::first-line"),
        PseudoElement::FirstLetter => out.push_str("::first-letter"),
        PseudoElement::Placeholder => out.push_str("::placeholder"),
        PseudoElement::Selection => out.push_str("::selection"),
        PseudoElement::Marker => out.push_str("::marker"),
        PseudoElement::Custom(s) => {
            out.push_str("::");
            out.push_str(s);
        }
        _ => {
            let name = format!("{:?}", pe);
            let kebab = camel_to_kebab(&name);
            out.push_str("::");
            out.push_str(&kebab);
        }
    }
}

/// Convert CamelCase to kebab-case.
fn camel_to_kebab(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            result.push('-');
        }
        result.push(c.to_ascii_lowercase());
    }
    result
}

fn write_decl(out: &mut String, decl: &Declaration) {
    match &decl.property {
        Property::Standard(name) => {
            out.push_str(name);
            out.push_str(": ");
            write_value(out, &decl.value);
            if decl.important {
                out.push_str(" !important");
            }
            out.push(';');
        }
        Property::Custom(name) => {
            out.push_str(name);
            out.push_str(": ");
            write_value(out, &decl.value);
            if decl.important {
                out.push_str(" !important");
            }
            out.push(';');
        }
        Property::Apply(classes) => {
            out.push_str("@apply ");
            out.push_str(classes);
            out.push(';');
        }
    }
}

fn write_value(out: &mut String, value: &Value) {
    match value {
        Value::Literal(s) => out.push_str(s),
        Value::Number(n, unit) => {
            if *n == (*n as i64) as f64 {
                out.push_str(&format!("{}", *n as i64));
            } else {
                out.push_str(&format!("{}", n));
            }
            if let Some(u) = unit {
                out.push_str(u.as_str());
            }
        }
        Value::Color(c) => write_color(out, c),
        Value::Token(t) => out.push_str(&format!("${}.{}", t.category.as_str(), t.name)),
        Value::Var(name, fallback) => {
            out.push_str(&format!("var(--{})", name));
            if let Some(fb) = fallback {
                out.push_str(", ");
                write_value(out, fb);
                out.push(')');
            }
        }
        Value::Env(name, fallback) => {
            out.push_str(&format!("env({})", name));
            if let Some(fb) = fallback {
                out.push_str(", ");
                write_value(out, fb);
                out.push(')');
            }
        }
        Value::List(values) => {
            for (i, v) in values.iter().enumerate() {
                if i > 0 {
                    out.push(' ');
                }
                write_value(out, v);
            }
        }
        Value::Computed(expr) => write_expr(out, expr),
    }
}

fn write_color(out: &mut String, color: &Color) {
    match color {
        Color::Hex(h) => {
            out.push('#');
            out.push_str(h);
        }
        Color::Named(n) => out.push_str(n),
        Color::Rgb(r, g, b) => out.push_str(&format!("rgb({}, {}, {})", r, g, b)),
        Color::Rgba(r, g, b, a) => out.push_str(&format!("rgba({}, {}, {}, {})", r, g, b, a)),
        Color::Hsl(h, s, l) => out.push_str(&format!("hsl({}, {}%, {}%)", h, s, l)),
        Color::Hsla(h, s, l, a) => out.push_str(&format!("hsla({}, {}%, {}%, {})", h, s, l, a)),
        Color::CurrentColor => out.push_str("currentColor"),
        Color::Transparent => out.push_str("transparent"),
        _ => out.push_str(&format!("{:?}", color)),
    }
}

fn write_expr(out: &mut String, expr: &Expr) {
    match expr {
        Expr::Value(v) => write_value(out, v),
        Expr::Add(a, b) => {
            out.push_str("calc(");
            write_expr(out, a);
            out.push_str(" + ");
            write_expr(out, b);
            out.push(')');
        }
        Expr::Sub(a, b) => {
            out.push_str("calc(");
            write_expr(out, a);
            out.push_str(" - ");
            write_expr(out, b);
            out.push(')');
        }
        Expr::Mul(a, b) => {
            out.push_str("calc(");
            write_expr(out, a);
            out.push_str(" * ");
            write_expr(out, b);
            out.push(')');
        }
        Expr::Div(a, b) => {
            out.push_str("calc(");
            write_expr(out, a);
            out.push_str(" / ");
            write_expr(out, b);
            out.push(')');
        }
        Expr::Function(name, args) => {
            out.push_str(name);
            out.push('(');
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                write_expr(out, arg);
            }
            out.push(')');
        }
    }
}

// ---------------------------------------------------------------------------
// Export generators
// ---------------------------------------------------------------------------

/// Generate a JSON string from the exports map.
///
/// ```json
/// { "button": "button_a1b2c3", "header": "header_d4e5f6" }
/// ```
pub fn generate_exports_json(exports: &HashMap<String, String>) -> String {
    // Sort keys for deterministic output.
    let mut keys: Vec<&String> = exports.keys().collect();
    keys.sort();

    let mut out = String::from("{\n");
    for (i, key) in keys.iter().enumerate() {
        let value = &exports[*key];
        out.push_str(&format!("  \"{}\": \"{}\"", key, value));
        if i < keys.len() - 1 {
            out.push(',');
        }
        out.push('\n');
    }
    out.push('}');
    out
}

/// Generate a JavaScript ES module exporting the class name map.
///
/// ```js
/// export default { "button": "button_a1b2c3" };
/// ```
pub fn generate_exports_js(exports: &HashMap<String, String>) -> String {
    let mut keys: Vec<&String> = exports.keys().collect();
    keys.sort();

    let mut out = String::from("export default {\n");
    for (i, key) in keys.iter().enumerate() {
        let value = &exports[*key];
        out.push_str(&format!("  \"{}\": \"{}\"", key, value));
        if i < keys.len() - 1 {
            out.push(',');
        }
        out.push('\n');
    }
    out.push_str("};\n");
    out
}

// ---------------------------------------------------------------------------
// Tests (unit)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_hash_deterministic() {
        let config = CSSModulesConfig::default();
        let h1 = generate_hash("src/app.module.css", "button", &config);
        let h2 = generate_hash("src/app.module.css", "button", &config);
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 6);
    }

    #[test]
    fn test_generate_hash_varies_by_file() {
        let config = CSSModulesConfig::default();
        let h1 = generate_hash("a.css", "button", &config);
        let h2 = generate_hash("b.css", "button", &config);
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_detect_scope_local_default() {
        let (mode, bare) = detect_scope("button");
        assert_eq!(mode, ScopeMode::Local);
        assert_eq!(bare, "button");
    }

    #[test]
    fn test_detect_scope_global() {
        let (mode, bare) = detect_scope(":global(.button)");
        assert_eq!(mode, ScopeMode::Global);
        assert_eq!(bare, "button");
    }

    #[test]
    fn test_detect_scope_local_explicit() {
        let (mode, bare) = detect_scope(":local(.button)");
        assert_eq!(mode, ScopeMode::Local);
        assert_eq!(bare, "button");
    }

    #[test]
    fn test_parse_composes_local() {
        let d = parse_composes("base primary");
        assert_eq!(d.class_names, vec!["base", "primary"]);
        assert!(d.from.is_none());
    }

    #[test]
    fn test_parse_composes_external() {
        let d = parse_composes("heading from './typography.module.css'");
        assert_eq!(d.class_names, vec!["heading"]);
        assert_eq!(d.from, Some("./typography.module.css".to_string()));
    }

    #[test]
    fn test_exports_json() {
        let mut exports = HashMap::new();
        exports.insert("a".to_string(), "a_123456".to_string());
        let json = generate_exports_json(&exports);
        assert!(json.contains("\"a\": \"a_123456\""));
    }

    #[test]
    fn test_exports_js() {
        let mut exports = HashMap::new();
        exports.insert("btn".to_string(), "btn_abcdef".to_string());
        let js = generate_exports_js(&exports);
        assert!(js.starts_with("export default {"));
        assert!(js.contains("\"btn\": \"btn_abcdef\""));
    }
}
