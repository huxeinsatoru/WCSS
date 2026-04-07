use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

/// Source location span for error reporting and source maps.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

impl Span {
    pub fn new(start: usize, end: usize, line: usize, column: usize) -> Self {
        Self { start, end, line, column }
    }

    pub fn empty() -> Self {
        Self { start: 0, end: 0, line: 0, column: 0 }
    }
}

// ---------------------------------------------------------------------------
// Root AST
// ---------------------------------------------------------------------------

/// Root node of a WCSS file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StyleSheet {
    pub rules: Vec<Rule>,
    /// Top-level at-rules (@import, @layer, @font-face, @keyframes, etc.)
    #[serde(default)]
    pub at_rules: Vec<AtRule>,
    pub span: Span,
}

// ---------------------------------------------------------------------------
// At-Rules (full CSS spec)
// ---------------------------------------------------------------------------

/// Represents all CSS at-rules.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AtRule {
    /// @import url("...") [layer] [supports(...)] [media];
    Import(ImportRule),
    /// @layer name { ... } or @layer name;
    Layer(LayerRule),
    /// @keyframes name { from/to/% { ... } }
    Keyframes(KeyframesRule),
    /// @font-face { ... }
    FontFace(FontFaceRule),
    /// @supports (condition) { ... }
    Supports(SupportsRule),
    /// @container [name] (condition) { ... }
    Container(ContainerRule),
    /// @media (condition) { ... } — top-level media queries
    Media(MediaRule),
    /// @property --name { ... } — CSS Houdini custom properties
    Property(PropertyRule),
    /// @charset "UTF-8"; (preserved for compatibility)
    Charset(String, Span),
    /// @namespace ...
    Namespace(String, Span),
    /// @scope (.root) to (.limit) { ... }
    Scope(ScopeRule),
}

/// @import rule
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ImportRule {
    pub url: String,
    pub layer: Option<String>,
    pub supports: Option<String>,
    pub media: Option<String>,
    pub span: Span,
}

/// @layer rule
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayerRule {
    pub name: String,
    /// None = declaration-only (@layer name;), Some = block
    pub rules: Option<Vec<Rule>>,
    pub span: Span,
}

/// @keyframes rule
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyframesRule {
    pub name: String,
    pub keyframes: Vec<Keyframe>,
    pub span: Span,
}

/// A single keyframe stop (from, to, or percentage).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Keyframe {
    pub selectors: Vec<KeyframeSelector>,
    pub declarations: Vec<Declaration>,
    pub span: Span,
}

/// Keyframe selector: from, to, or a percentage.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum KeyframeSelector {
    From,
    To,
    Percentage(f64),
}

/// Custom Hash for KeyframeSelector (f64 doesn't impl Hash)
impl Hash for KeyframeSelector {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            KeyframeSelector::From => 0u8.hash(state),
            KeyframeSelector::To => 1u8.hash(state),
            KeyframeSelector::Percentage(p) => {
                2u8.hash(state);
                p.to_bits().hash(state);
            }
        }
    }
}

/// @font-face rule
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FontFaceRule {
    pub declarations: Vec<Declaration>,
    pub span: Span,
}

/// @supports rule
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SupportsRule {
    pub condition: String,
    pub rules: Vec<Rule>,
    pub span: Span,
}

/// @container rule
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContainerRule {
    pub name: Option<String>,
    pub condition: String,
    pub rules: Vec<Rule>,
    pub span: Span,
}

/// @media rule (top-level)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MediaRule {
    pub query: String,
    pub rules: Vec<Rule>,
    pub span: Span,
}

/// @property rule (CSS Houdini)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PropertyRule {
    pub name: String,
    pub syntax: Option<String>,
    pub inherits: Option<bool>,
    pub initial_value: Option<String>,
    pub span: Span,
}

/// @scope rule: `@scope (.root) to (.limit) { ... }` or `@scope (.root) { ... }` or `@scope { ... }`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScopeRule {
    /// The root selector (e.g., ".card"). Empty string for implicit scope.
    pub root: String,
    /// Optional limit selector (e.g., ".card__content").
    pub limit: Option<String>,
    /// Rules inside the scope block.
    pub rules: Vec<Rule>,
    pub span: Span,
}

// ---------------------------------------------------------------------------
// Style Rules
// ---------------------------------------------------------------------------

/// A single style rule with selector(s), declarations, states, and responsive blocks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rule {
    /// Primary selector (backward compat)
    pub selector: Selector,
    /// Additional selectors for grouped rules (.a, .b { ... })
    #[serde(default)]
    pub selectors: Vec<Selector>,
    pub declarations: Vec<Declaration>,
    pub states: Vec<StateBlock>,
    pub responsive: Vec<ResponsiveBlock>,
    /// Nested rules (CSS nesting spec)
    #[serde(default)]
    pub nested_rules: Vec<Rule>,
    pub span: Span,
}

impl Rule {
    /// Get all selectors (primary + additional) for this rule.
    pub fn all_selectors(&self) -> Vec<&Selector> {
        let mut all = vec![&self.selector];
        all.extend(self.selectors.iter());
        all
    }
}

// ---------------------------------------------------------------------------
// Selectors (full CSS spec)
// ---------------------------------------------------------------------------

/// A CSS selector — now supports all selector types.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Selector {
    /// The primary selector part
    pub class_name: String,
    /// Selector type (class, id, tag, attribute, universal, nesting)
    #[serde(default)]
    pub kind: SelectorKind,
    pub combinators: Vec<Combinator>,
    pub pseudo_elements: Vec<PseudoElement>,
    /// Pseudo-classes directly on this selector (not in state blocks)
    #[serde(default)]
    pub pseudo_classes: Vec<PseudoClass>,
    /// Attribute selectors: [attr], [attr=value], etc.
    #[serde(default)]
    pub attributes: Vec<AttributeSelector>,
    pub span: Span,
}

/// The kind of selector.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum SelectorKind {
    #[default]
    Class,        // .class
    Id,           // #id
    Tag,          // div, p, span
    Universal,    // *
    Nesting,      // & (CSS nesting)
    Attribute,    // [attr]
}

/// Attribute selector ([attr], [attr=value], [attr~=value], etc.)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AttributeSelector {
    pub name: String,
    pub operator: Option<AttributeOp>,
    pub value: Option<String>,
    pub modifier: Option<AttributeModifier>,
}

/// Attribute selector operators.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AttributeOp {
    Equals,        // =
    Contains,      // ~=
    DashMatch,     // |=
    Prefix,        // ^=
    Suffix,        // $=
    Substring,     // *=
}

/// Attribute selector modifiers.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AttributeModifier {
    CaseInsensitive, // i
    CaseSensitive,   // s
}

/// Pseudo-class selectors (:hover, :nth-child(n), etc.)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PseudoClass {
    Hover,
    Focus,
    FocusVisible,
    FocusWithin,
    Active,
    Visited,
    Link,
    Disabled,
    Enabled,
    Checked,
    Indeterminate,
    Required,
    Optional,
    Valid,
    Invalid,
    ReadOnly,
    ReadWrite,
    PlaceholderShown,
    Default,
    FirstChild,
    LastChild,
    OnlyChild,
    FirstOfType,
    LastOfType,
    OnlyOfType,
    Empty,
    Root,
    /// :nth-child(An+B)
    NthChild(String),
    /// :nth-last-child(An+B)
    NthLastChild(String),
    /// :nth-of-type(An+B)
    NthOfType(String),
    /// :nth-last-of-type(An+B)
    NthLastOfType(String),
    /// :not(selector)
    Not(String),
    /// :is(selector)
    Is(String),
    /// :where(selector)
    Where(String),
    /// :has(selector)
    Has(String),
    /// :lang(language)
    Lang(String),
    /// :dir(ltr|rtl)
    Dir(String),
    /// Dark mode: prefers-color-scheme: dark
    Dark,
    /// Custom pseudo-class
    Custom(String),
}

/// Selector combinators for descendant, child, adjacent, sibling relationships.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Combinator {
    Descendant(Box<Selector>),
    Child(Box<Selector>),
    Adjacent(Box<Selector>),
    Sibling(Box<Selector>),
}

/// CSS pseudo-elements (::before, ::after, etc.).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PseudoElement {
    Before,
    After,
    FirstLine,
    FirstLetter,
    Placeholder,
    Selection,
    Marker,
    Backdrop,
    Cue,
    CueRegion,
    GrammarError,
    SpellingError,
    TargetText,
    FileSelectorButton,
    Custom(String),
}

// ---------------------------------------------------------------------------
// Declarations
// ---------------------------------------------------------------------------

/// A property-value declaration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Declaration {
    pub property: Property,
    pub value: Value,
    pub important: bool,
    pub span: Span,
}

/// CSS property types.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Property {
    Standard(String),
    Custom(String), // --custom-property
}

impl Property {
    pub fn name(&self) -> &str {
        match self {
            Property::Standard(name) => name,
            Property::Custom(name) => name,
        }
    }
}

// ---------------------------------------------------------------------------
// Values
// ---------------------------------------------------------------------------

/// Style values that can appear in declarations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    /// Reference to a design token: $category.name
    Token(TokenRef),
    /// Raw string literal
    Literal(String),
    /// Numeric value with optional unit
    Number(f64, Option<Unit>),
    /// Color value
    Color(Color),
    /// Computed expression
    Computed(Box<Expr>),
    /// Multiple values (e.g., shorthand properties)
    List(Vec<Value>),
    /// CSS var() function reference
    Var(String, Option<Box<Value>>),
    /// CSS env() function reference
    Env(String, Option<Box<Value>>),
}

/// Implement Hash for Value (f64 doesn't impl Hash natively).
impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            Value::Token(t) => t.hash(state),
            Value::Literal(s) => s.hash(state),
            Value::Number(n, u) => {
                n.to_bits().hash(state);
                u.hash(state);
            }
            Value::Color(c) => c.hash(state),
            Value::Computed(e) => e.hash(state),
            Value::List(v) => v.hash(state),
            Value::Var(name, fallback) => {
                name.hash(state);
                fallback.hash(state);
            }
            Value::Env(name, fallback) => {
                name.hash(state);
                fallback.hash(state);
            }
        }
    }
}

/// Reference to a design token.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TokenRef {
    pub category: TokenCategory,
    pub name: String,
    pub span: Span,
}

/// Categories of design tokens.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TokenCategory {
    Colors,
    Spacing,
    Typography,
    Breakpoints,
    Animation,
    Shadows,
    Borders,
    Radii,
    ZIndex,
    Opacity,
    Custom,
}

impl TokenCategory {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "colors" => Some(Self::Colors),
            "spacing" => Some(Self::Spacing),
            "typography" => Some(Self::Typography),
            "breakpoints" => Some(Self::Breakpoints),
            "animation" => Some(Self::Animation),
            "shadows" => Some(Self::Shadows),
            "borders" => Some(Self::Borders),
            "radii" => Some(Self::Radii),
            "zindex" | "z-index" => Some(Self::ZIndex),
            "opacity" => Some(Self::Opacity),
            "custom" => Some(Self::Custom),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Colors => "colors",
            Self::Spacing => "spacing",
            Self::Typography => "typography",
            Self::Breakpoints => "breakpoints",
            Self::Animation => "animation",
            Self::Shadows => "shadows",
            Self::Borders => "borders",
            Self::Radii => "radii",
            Self::ZIndex => "zindex",
            Self::Opacity => "opacity",
            Self::Custom => "custom",
        }
    }
}

// ---------------------------------------------------------------------------
// CSS Units (comprehensive)
// ---------------------------------------------------------------------------

/// CSS units — full spec coverage.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Unit {
    // Absolute lengths
    Px,
    Cm,
    Mm,
    In,
    Pt,
    Pc,
    // Relative lengths
    Rem,
    Em,
    Ex,
    Ch,
    Lh,
    Rlh,
    Cap,
    Ic,
    // Viewport units
    Vh,
    Vw,
    Vmin,
    Vmax,
    // Dynamic viewport units
    Dvh,
    Dvw,
    Dvmin,
    Dvmax,
    // Small viewport units
    Svh,
    Svw,
    Svmin,
    Svmax,
    // Large viewport units
    Lvh,
    Lvw,
    Lvmin,
    Lvmax,
    // Container query units
    Cqi,
    Cqb,
    Cqw,
    Cqh,
    Cqmin,
    Cqmax,
    // Percentage
    Percent,
    // Grid
    Fr,
    // Angles
    Deg,
    Rad,
    Grad,
    Turn,
    // Time
    Ms,
    S,
    // Frequency
    Hz,
    Khz,
    // Resolution
    Dpi,
    Dpcm,
    Dppx,
    X, // alias for dppx
}

impl Unit {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            // Absolute
            "px" => Some(Self::Px),
            "cm" => Some(Self::Cm),
            "mm" => Some(Self::Mm),
            "in" => Some(Self::In),
            "pt" => Some(Self::Pt),
            "pc" => Some(Self::Pc),
            // Relative
            "rem" => Some(Self::Rem),
            "em" => Some(Self::Em),
            "ex" => Some(Self::Ex),
            "ch" => Some(Self::Ch),
            "lh" => Some(Self::Lh),
            "rlh" => Some(Self::Rlh),
            "cap" => Some(Self::Cap),
            "ic" => Some(Self::Ic),
            // Viewport
            "vh" => Some(Self::Vh),
            "vw" => Some(Self::Vw),
            "vmin" => Some(Self::Vmin),
            "vmax" => Some(Self::Vmax),
            // Dynamic viewport
            "dvh" => Some(Self::Dvh),
            "dvw" => Some(Self::Dvw),
            "dvmin" => Some(Self::Dvmin),
            "dvmax" => Some(Self::Dvmax),
            // Small viewport
            "svh" => Some(Self::Svh),
            "svw" => Some(Self::Svw),
            "svmin" => Some(Self::Svmin),
            "svmax" => Some(Self::Svmax),
            // Large viewport
            "lvh" => Some(Self::Lvh),
            "lvw" => Some(Self::Lvw),
            "lvmin" => Some(Self::Lvmin),
            "lvmax" => Some(Self::Lvmax),
            // Container query
            "cqi" => Some(Self::Cqi),
            "cqb" => Some(Self::Cqb),
            "cqw" => Some(Self::Cqw),
            "cqh" => Some(Self::Cqh),
            "cqmin" => Some(Self::Cqmin),
            "cqmax" => Some(Self::Cqmax),
            // Percentage
            "%" => Some(Self::Percent),
            // Grid
            "fr" => Some(Self::Fr),
            // Angles
            "deg" => Some(Self::Deg),
            "rad" => Some(Self::Rad),
            "grad" => Some(Self::Grad),
            "turn" => Some(Self::Turn),
            // Time
            "ms" => Some(Self::Ms),
            "s" => Some(Self::S),
            // Frequency
            "hz" | "Hz" => Some(Self::Hz),
            "khz" | "kHz" => Some(Self::Khz),
            // Resolution
            "dpi" => Some(Self::Dpi),
            "dpcm" => Some(Self::Dpcm),
            "dppx" => Some(Self::Dppx),
            "x" => Some(Self::X),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Px => "px",
            Self::Cm => "cm",
            Self::Mm => "mm",
            Self::In => "in",
            Self::Pt => "pt",
            Self::Pc => "pc",
            Self::Rem => "rem",
            Self::Em => "em",
            Self::Ex => "ex",
            Self::Ch => "ch",
            Self::Lh => "lh",
            Self::Rlh => "rlh",
            Self::Cap => "cap",
            Self::Ic => "ic",
            Self::Vh => "vh",
            Self::Vw => "vw",
            Self::Vmin => "vmin",
            Self::Vmax => "vmax",
            Self::Dvh => "dvh",
            Self::Dvw => "dvw",
            Self::Dvmin => "dvmin",
            Self::Dvmax => "dvmax",
            Self::Svh => "svh",
            Self::Svw => "svw",
            Self::Svmin => "svmin",
            Self::Svmax => "svmax",
            Self::Lvh => "lvh",
            Self::Lvw => "lvw",
            Self::Lvmin => "lvmin",
            Self::Lvmax => "lvmax",
            Self::Cqi => "cqi",
            Self::Cqb => "cqb",
            Self::Cqw => "cqw",
            Self::Cqh => "cqh",
            Self::Cqmin => "cqmin",
            Self::Cqmax => "cqmax",
            Self::Percent => "%",
            Self::Fr => "fr",
            Self::Deg => "deg",
            Self::Rad => "rad",
            Self::Grad => "grad",
            Self::Turn => "turn",
            Self::Ms => "ms",
            Self::S => "s",
            Self::Hz => "Hz",
            Self::Khz => "kHz",
            Self::Dpi => "dpi",
            Self::Dpcm => "dpcm",
            Self::Dppx => "dppx",
            Self::X => "x",
        }
    }
}

// ---------------------------------------------------------------------------
// Colors
// ---------------------------------------------------------------------------

/// Color representations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Color {
    Hex(String),
    Rgb(f64, f64, f64),
    Rgba(f64, f64, f64, f64),
    Hsl(f64, f64, f64),
    Hsla(f64, f64, f64, f64),
    Hwb(f64, f64, f64),
    Lab(f64, f64, f64),
    Lch(f64, f64, f64),
    Oklch(f64, f64, f64),
    Oklab(f64, f64, f64),
    Named(String),
    /// color-mix() function
    ColorMix(String),
    /// light-dark() function
    LightDark(Box<Color>, Box<Color>),
    CurrentColor,
    Transparent,
}

/// Implement Hash for Color (f64 fields).
impl Hash for Color {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            Color::Hex(h) => h.hash(state),
            Color::Rgb(r, g, b) => {
                r.to_bits().hash(state);
                g.to_bits().hash(state);
                b.to_bits().hash(state);
            }
            Color::Rgba(r, g, b, a) => {
                r.to_bits().hash(state);
                g.to_bits().hash(state);
                b.to_bits().hash(state);
                a.to_bits().hash(state);
            }
            Color::Hsl(h, s, l) => {
                h.to_bits().hash(state);
                s.to_bits().hash(state);
                l.to_bits().hash(state);
            }
            Color::Hsla(h, s, l, a) => {
                h.to_bits().hash(state);
                s.to_bits().hash(state);
                l.to_bits().hash(state);
                a.to_bits().hash(state);
            }
            Color::Hwb(h, w, b) => {
                h.to_bits().hash(state);
                w.to_bits().hash(state);
                b.to_bits().hash(state);
            }
            Color::Lab(l, a, b) | Color::Lch(l, a, b)
            | Color::Oklch(l, a, b) | Color::Oklab(l, a, b) => {
                l.to_bits().hash(state);
                a.to_bits().hash(state);
                b.to_bits().hash(state);
            }
            Color::Named(n) => n.hash(state),
            Color::ColorMix(s) => s.hash(state),
            Color::LightDark(l, d) => { l.hash(state); d.hash(state); }
            Color::CurrentColor => {}
            Color::Transparent => {}
        }
    }
}

// ---------------------------------------------------------------------------
// Expressions
// ---------------------------------------------------------------------------

/// Expression for computed values.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expr {
    Value(Value),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Function(String, Vec<Expr>),
}

/// Implement Hash for Expr.
impl Hash for Expr {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            Expr::Value(v) => v.hash(state),
            Expr::Add(a, b) | Expr::Sub(a, b) | Expr::Mul(a, b) | Expr::Div(a, b) => {
                a.hash(state);
                b.hash(state);
            }
            Expr::Function(name, args) => {
                name.hash(state);
                args.hash(state);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// State & Responsive Blocks
// ---------------------------------------------------------------------------

/// State modifiers for pseudo-class selectors.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StateModifier {
    Hover,
    Focus,
    FocusVisible,
    FocusWithin,
    Active,
    Visited,
    Disabled,
    Enabled,
    Checked,
    Indeterminate,
    Required,
    Optional,
    Valid,
    Invalid,
    ReadOnly,
    ReadWrite,
    PlaceholderShown,
    Default,
    FirstChild,
    LastChild,
    OnlyChild,
    FirstOfType,
    LastOfType,
    OnlyOfType,
    Empty,
    /// Dark mode modifier (generates @media (prefers-color-scheme: dark))
    Dark,
    Custom(String),
}

impl StateModifier {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "hover" => Some(Self::Hover),
            "focus" => Some(Self::Focus),
            "focus-visible" => Some(Self::FocusVisible),
            "focus-within" => Some(Self::FocusWithin),
            "active" => Some(Self::Active),
            "visited" => Some(Self::Visited),
            "disabled" => Some(Self::Disabled),
            "enabled" => Some(Self::Enabled),
            "checked" => Some(Self::Checked),
            "indeterminate" => Some(Self::Indeterminate),
            "required" => Some(Self::Required),
            "optional" => Some(Self::Optional),
            "valid" => Some(Self::Valid),
            "invalid" => Some(Self::Invalid),
            "read-only" => Some(Self::ReadOnly),
            "read-write" => Some(Self::ReadWrite),
            "placeholder-shown" => Some(Self::PlaceholderShown),
            "default" => Some(Self::Default),
            "first-child" => Some(Self::FirstChild),
            "last-child" => Some(Self::LastChild),
            "only-child" => Some(Self::OnlyChild),
            "first-of-type" => Some(Self::FirstOfType),
            "last-of-type" => Some(Self::LastOfType),
            "only-of-type" => Some(Self::OnlyOfType),
            "empty" => Some(Self::Empty),
            "dark" => Some(Self::Dark),
            _ => Some(Self::Custom(s.to_string())),
        }
    }

    pub fn as_css_pseudo(&self) -> String {
        match self {
            Self::Hover => ":hover".to_string(),
            Self::Focus => ":focus".to_string(),
            Self::FocusVisible => ":focus-visible".to_string(),
            Self::FocusWithin => ":focus-within".to_string(),
            Self::Active => ":active".to_string(),
            Self::Visited => ":visited".to_string(),
            Self::Disabled => ":disabled".to_string(),
            Self::Enabled => ":enabled".to_string(),
            Self::Checked => ":checked".to_string(),
            Self::Indeterminate => ":indeterminate".to_string(),
            Self::Required => ":required".to_string(),
            Self::Optional => ":optional".to_string(),
            Self::Valid => ":valid".to_string(),
            Self::Invalid => ":invalid".to_string(),
            Self::ReadOnly => ":read-only".to_string(),
            Self::ReadWrite => ":read-write".to_string(),
            Self::PlaceholderShown => ":placeholder-shown".to_string(),
            Self::Default => ":default".to_string(),
            Self::FirstChild => ":first-child".to_string(),
            Self::LastChild => ":last-child".to_string(),
            Self::OnlyChild => ":only-child".to_string(),
            Self::FirstOfType => ":first-of-type".to_string(),
            Self::LastOfType => ":last-of-type".to_string(),
            Self::OnlyOfType => ":only-of-type".to_string(),
            Self::Empty => ":empty".to_string(),
            Self::Dark => ":dark".to_string(), // special: handled in codegen
            Self::Custom(s) => format!(":{s}"),
        }
    }
}

/// A block of declarations for a specific state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StateBlock {
    pub modifiers: Vec<StateModifier>,
    pub declarations: Vec<Declaration>,
    pub span: Span,
}

/// A block of declarations for a specific breakpoint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResponsiveBlock {
    pub breakpoint: String,
    pub declarations: Vec<Declaration>,
    pub span: Span,
}

// ---------------------------------------------------------------------------
// Vendor Prefix
// ---------------------------------------------------------------------------

/// Vendor prefix targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VendorPrefix {
    None,
    Webkit,
    Moz,
    Ms,
    O,
}

impl VendorPrefix {
    pub fn as_str(&self) -> &str {
        match self {
            Self::None => "",
            Self::Webkit => "-webkit-",
            Self::Moz => "-moz-",
            Self::Ms => "-ms-",
            Self::O => "-o-",
        }
    }
}
