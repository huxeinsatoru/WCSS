use serde::{Deserialize, Serialize};

/// Source location span for error reporting and source maps.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

/// Root node of a WCSS file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StyleSheet {
    pub rules: Vec<Rule>,
    pub span: Span,
}

/// A single style rule with selector, declarations, states, and responsive blocks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rule {
    pub selector: Selector,
    pub declarations: Vec<Declaration>,
    pub states: Vec<StateBlock>,
    pub responsive: Vec<ResponsiveBlock>,
    pub span: Span,
}

/// A CSS selector.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Selector {
    pub class_name: String,
    pub combinators: Vec<Combinator>,
    pub pseudo_elements: Vec<PseudoElement>,
    pub span: Span,
}

/// Selector combinators for descendant, child, adjacent, sibling relationships.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Combinator {
    Descendant(Box<Selector>),
    Child(Box<Selector>),
    Adjacent(Box<Selector>),
    Sibling(Box<Selector>),
}

/// CSS pseudo-elements (::before, ::after, etc.).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PseudoElement {
    Before,
    After,
    FirstLine,
    FirstLetter,
    Placeholder,
    Selection,
    Custom(String),
}

/// A property-value declaration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Declaration {
    pub property: Property,
    pub value: Value,
    pub important: bool,
    pub span: Span,
}

/// CSS property types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
}

/// Reference to a design token.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
            Self::Custom => "custom",
        }
    }
}

/// CSS units.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Unit {
    Px,
    Rem,
    Em,
    Percent,
    Vh,
    Vw,
    Fr,
    Deg,
    Ms,
    S,
}

impl Unit {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "px" => Some(Self::Px),
            "rem" => Some(Self::Rem),
            "em" => Some(Self::Em),
            "%" => Some(Self::Percent),
            "vh" => Some(Self::Vh),
            "vw" => Some(Self::Vw),
            "fr" => Some(Self::Fr),
            "deg" => Some(Self::Deg),
            "ms" => Some(Self::Ms),
            "s" => Some(Self::S),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Px => "px",
            Self::Rem => "rem",
            Self::Em => "em",
            Self::Percent => "%",
            Self::Vh => "vh",
            Self::Vw => "vw",
            Self::Fr => "fr",
            Self::Deg => "deg",
            Self::Ms => "ms",
            Self::S => "s",
        }
    }
}

/// Color representations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Color {
    Hex(String),
    Rgb(f64, f64, f64),
    Rgba(f64, f64, f64, f64),
    Hsl(f64, f64, f64),
    Hsla(f64, f64, f64, f64),
    Named(String),
}

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

/// State modifiers for pseudo-class selectors.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StateModifier {
    Hover,
    Focus,
    Active,
    Visited,
    Disabled,
    Checked,
    FirstChild,
    LastChild,
    Custom(String),
}

impl StateModifier {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "hover" => Some(Self::Hover),
            "focus" => Some(Self::Focus),
            "active" => Some(Self::Active),
            "visited" => Some(Self::Visited),
            "disabled" => Some(Self::Disabled),
            "checked" => Some(Self::Checked),
            "first-child" => Some(Self::FirstChild),
            "last-child" => Some(Self::LastChild),
            _ => Some(Self::Custom(s.to_string())),
        }
    }

    pub fn as_css_pseudo(&self) -> String {
        match self {
            Self::Hover => ":hover".to_string(),
            Self::Focus => ":focus".to_string(),
            Self::Active => ":active".to_string(),
            Self::Visited => ":visited".to_string(),
            Self::Disabled => ":disabled".to_string(),
            Self::Checked => ":checked".to_string(),
            Self::FirstChild => ":first-child".to_string(),
            Self::LastChild => ":last-child".to_string(),
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
