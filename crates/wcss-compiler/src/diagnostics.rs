//! Rich diagnostics system for the WCSS compiler.
//!
//! This module provides structured, user-friendly error reporting that complements
//! the existing `error` module. It adds error codes, severity levels, source snippets,
//! Levenshtein-based suggestions, and color-coded rendering.

use crate::ast::Span;
use crate::error::{CompilerError, ErrorKind};

// ---------------------------------------------------------------------------
// Error codes
// ---------------------------------------------------------------------------

/// Standardized error codes for common WCSS mistakes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCode {
    /// E001: Unknown CSS property name.
    E001,
    /// E002: Invalid value for a known property.
    E002,
    /// E003: Unclosed bracket or brace.
    E003,
    /// E004: Invalid selector syntax.
    E004,
    /// E005: Duplicate property in the same rule.
    E005,
    /// E006: Unknown at-rule.
    E006,
    /// E007: Invalid color value.
    E007,
    /// E008: Missing semicolon.
    E008,
    /// E009: Invalid nesting.
    E009,
    /// E010: Unknown pseudo-class or pseudo-element.
    E010,
}

impl ErrorCode {
    /// Short identifier suitable for display, e.g. `"E001"`.
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorCode::E001 => "E001",
            ErrorCode::E002 => "E002",
            ErrorCode::E003 => "E003",
            ErrorCode::E004 => "E004",
            ErrorCode::E005 => "E005",
            ErrorCode::E006 => "E006",
            ErrorCode::E007 => "E007",
            ErrorCode::E008 => "E008",
            ErrorCode::E009 => "E009",
            ErrorCode::E010 => "E010",
        }
    }

    /// Human-readable explanation of the error category.
    pub fn explanation(&self) -> &'static str {
        match self {
            ErrorCode::E001 => "The property name is not a recognized CSS property.",
            ErrorCode::E002 => "The value is not valid for the given property.",
            ErrorCode::E003 => "A bracket, brace, or parenthesis was opened but never closed.",
            ErrorCode::E004 => "The selector contains invalid syntax.",
            ErrorCode::E005 => "The same property appears more than once in a single rule.",
            ErrorCode::E006 => "The at-rule is not recognized.",
            ErrorCode::E007 => "The color value is malformed or out of range.",
            ErrorCode::E008 => "A semicolon is expected after a declaration.",
            ErrorCode::E009 => "Nesting is used in an invalid context.",
            ErrorCode::E010 => "The pseudo-class or pseudo-element is not recognized.",
        }
    }
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// ---------------------------------------------------------------------------
// Severity
// ---------------------------------------------------------------------------

/// Severity level of a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Severity {
    /// Something that prevents compilation.
    Error,
    /// Potential problem that does not block compilation.
    Warning,
    /// Informational note attached to another diagnostic.
    Info,
    /// A hint for improvement.
    Hint,
}

impl Severity {
    /// Label used when rendering, e.g. `"error"`.
    pub fn label(&self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "info",
            Severity::Hint => "hint",
        }
    }

    /// ANSI color code prefix (red, yellow, blue, green).
    pub fn color_code(&self) -> &'static str {
        match self {
            Severity::Error => "\x1b[31m",   // red
            Severity::Warning => "\x1b[33m", // yellow
            Severity::Info => "\x1b[34m",    // blue
            Severity::Hint => "\x1b[32m",    // green
        }
    }
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

// ---------------------------------------------------------------------------
// Suggestion
// ---------------------------------------------------------------------------

/// A concrete fix suggestion that can be applied to the source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Suggestion {
    /// Human-readable description of the suggestion.
    pub message: String,
    /// The replacement text.  If empty, the suggestion is a deletion.
    pub replacement: String,
    /// The source span that should be replaced.
    pub span: Span,
}

// ---------------------------------------------------------------------------
// Diagnostic
// ---------------------------------------------------------------------------

/// A rich diagnostic message produced by the compiler.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// Severity level.
    pub severity: Severity,
    /// Primary human-readable message.
    pub message: String,
    /// Location in source.
    pub span: Span,
    /// The original source line (or small snippet) for context.
    pub source_snippet: Option<String>,
    /// Machine-readable error code.
    pub code: Option<ErrorCode>,
    /// Zero or more concrete fix suggestions.
    pub suggestions: Vec<Suggestion>,
}

impl Diagnostic {
    /// Create a new error-level diagnostic.
    pub fn error(message: impl Into<String>, span: Span) -> Self {
        Self {
            severity: Severity::Error,
            message: message.into(),
            span,
            source_snippet: None,
            code: None,
            suggestions: Vec::new(),
        }
    }

    /// Create a new warning-level diagnostic.
    pub fn warning(message: impl Into<String>, span: Span) -> Self {
        Self {
            severity: Severity::Warning,
            message: message.into(),
            span,
            source_snippet: None,
            code: None,
            suggestions: Vec::new(),
        }
    }

    /// Create a new info-level diagnostic.
    pub fn info(message: impl Into<String>, span: Span) -> Self {
        Self {
            severity: Severity::Info,
            message: message.into(),
            span,
            source_snippet: None,
            code: None,
            suggestions: Vec::new(),
        }
    }

    /// Create a new hint-level diagnostic.
    pub fn hint(message: impl Into<String>, span: Span) -> Self {
        Self {
            severity: Severity::Hint,
            message: message.into(),
            span,
            source_snippet: None,
            code: None,
            suggestions: Vec::new(),
        }
    }

    /// Attach an error code.
    pub fn with_code(mut self, code: ErrorCode) -> Self {
        self.code = Some(code);
        self
    }

    /// Attach a source snippet (typically the offending line).
    pub fn with_source_snippet(mut self, snippet: impl Into<String>) -> Self {
        self.source_snippet = Some(snippet.into());
        self
    }

    /// Add a suggestion.
    pub fn with_suggestion(mut self, suggestion: Suggestion) -> Self {
        self.suggestions.push(suggestion);
        self
    }

    /// Convenience: add a "did you mean?" suggestion.
    pub fn with_did_you_mean(mut self, candidate: &str, span: Span) -> Self {
        self.suggestions.push(Suggestion {
            message: format!("Did you mean `{}`?", candidate),
            replacement: candidate.to_string(),
            span,
        });
        self
    }

    // -- Conversion helpers --------------------------------------------------

    /// Build a `Diagnostic` from an existing `CompilerError`, enriching it with
    /// an error code and source context when possible.
    pub fn from_compiler_error(err: &CompilerError, source: &str) -> Self {
        let (code, severity) = classify_error(err);
        let snippet = extract_source_line(source, err.span.line);

        let mut diag = Diagnostic {
            severity,
            message: err.message.clone(),
            span: err.span.clone(),
            source_snippet: snippet,
            code,
            suggestions: Vec::new(),
        };

        if let Some(ref suggestion_text) = err.suggestion {
            diag.suggestions.push(Suggestion {
                message: suggestion_text.clone(),
                replacement: String::new(),
                span: err.span.clone(),
            });
        }

        diag
    }
}

/// Map an existing `ErrorKind` + message heuristics to an `ErrorCode`.
fn classify_error(err: &CompilerError) -> (Option<ErrorCode>, Severity) {
    let msg = err.message.to_lowercase();
    let code = match &err.kind {
        ErrorKind::SyntaxError => {
            if msg.contains("unclosed") || msg.contains("brace") || msg.contains("bracket") {
                Some(ErrorCode::E003)
            } else if msg.contains("selector") {
                Some(ErrorCode::E004)
            } else if msg.contains("semicolon") || msg.contains("expected ';'") || msg.contains("expected ':'") {
                Some(ErrorCode::E008)
            } else if msg.contains("nesting") {
                Some(ErrorCode::E009)
            } else {
                None
            }
        }
        ErrorKind::ValidationError => {
            if msg.contains("property") && (msg.contains("unknown") || msg.contains("invalid")) {
                Some(ErrorCode::E001)
            } else if msg.contains("value") {
                Some(ErrorCode::E002)
            } else if msg.contains("duplicate") {
                Some(ErrorCode::E005)
            } else if msg.contains("at-rule") || msg.contains("@") {
                Some(ErrorCode::E006)
            } else if msg.contains("color") {
                Some(ErrorCode::E007)
            } else if msg.contains("pseudo") {
                Some(ErrorCode::E010)
            } else {
                None
            }
        }
        ErrorKind::TokenNotFound => None,
        ErrorKind::CircularReference => None,
        ErrorKind::W3C(_) => None,
    };
    (code, Severity::Error)
}

/// Extract the source line at a given 1-based line number.
fn extract_source_line(source: &str, line: usize) -> Option<String> {
    if line == 0 {
        return None;
    }
    source.lines().nth(line - 1).map(|s| s.to_string())
}

// ---------------------------------------------------------------------------
// DiagnosticRenderer
// ---------------------------------------------------------------------------

/// Formats diagnostics for terminal display.
pub struct DiagnosticRenderer {
    /// When true, ANSI color escape codes are emitted.
    pub use_color: bool,
}

impl DiagnosticRenderer {
    /// Create a renderer with color enabled.
    pub fn new() -> Self {
        Self { use_color: true }
    }

    /// Create a renderer with color disabled (for testing / CI).
    pub fn plain() -> Self {
        Self { use_color: false }
    }

    /// Render a single diagnostic to a `String`.
    pub fn render(&self, diag: &Diagnostic, file_path: &str) -> String {
        self.render_with_source(diag, file_path, None)
    }

    /// Render a single diagnostic, optionally using the full source to extract
    /// context lines if `diag.source_snippet` is not already populated.
    pub fn render_with_source(
        &self,
        diag: &Diagnostic,
        file_path: &str,
        source: Option<&str>,
    ) -> String {
        let reset = if self.use_color { "\x1b[0m" } else { "" };
        let bold = if self.use_color { "\x1b[1m" } else { "" };
        let severity_color = if self.use_color {
            diag.severity.color_code()
        } else {
            ""
        };

        let mut out = String::new();

        // -- Header: severity[code]: message --------------------------------
        out.push_str(&format!(
            "{bold}{severity_color}{severity}{reset}",
            severity = diag.severity.label(),
        ));
        if let Some(ref code) = diag.code {
            out.push_str(&format!("[{code}]"));
        }
        out.push_str(&format!(
            "{bold}: {msg}{reset}\n",
            msg = diag.message,
        ));

        // -- Location -------------------------------------------------------
        out.push_str(&format!(
            "  \u{250c}\u{2500} {file}:{line}:{col}\n",
            file = file_path,
            line = diag.span.line,
            col = diag.span.column,
        ));
        out.push_str("  \u{2502}\n");

        // -- Source snippet with caret underline -----------------------------
        let snippet = diag.source_snippet.as_deref().or_else(|| {
            source.and_then(|s| {
                if diag.span.line > 0 {
                    s.lines().nth(diag.span.line - 1)
                } else {
                    None
                }
            })
        });

        if let Some(line_text) = snippet {
            let line_num = diag.span.line;
            out.push_str(&format!(
                "{:>3} \u{2502}   {}\n",
                line_num, line_text,
            ));

            let underline_start = if diag.span.column > 0 {
                diag.span.column - 1
            } else {
                0
            };
            let underline_len = if diag.span.end > diag.span.start {
                diag.span.end - diag.span.start
            } else {
                1
            };
            out.push_str(&format!(
                "    \u{2502}   {}{}{}{}\n",
                " ".repeat(underline_start),
                severity_color,
                "^".repeat(underline_len),
                reset,
            ));
        }

        // -- Suggestions ----------------------------------------------------
        for suggestion in &diag.suggestions {
            out.push_str(&format!("  = help: {}\n", suggestion.message));
        }

        // -- Error code explanation -----------------------------------------
        if let Some(ref code) = diag.code {
            out.push_str(&format!(
                "  = note: [{}] {}\n",
                code.as_str(),
                code.explanation(),
            ));
        }

        out
    }

    /// Render a collection of diagnostics.
    pub fn render_all(
        &self,
        diagnostics: &[Diagnostic],
        file_path: &str,
        source: Option<&str>,
    ) -> String {
        let mut out = String::new();
        for (i, diag) in diagnostics.iter().enumerate() {
            if i > 0 {
                out.push('\n');
            }
            out.push_str(&self.render_with_source(diag, file_path, source));
        }

        // Summary line
        let error_count = diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .count();
        let warning_count = diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Warning)
            .count();

        if !diagnostics.is_empty() {
            out.push_str(&format!(
                "\n{}: {} error(s), {} warning(s) emitted\n",
                if self.use_color {
                    format!("{}\x1b[1m{}\x1b[0m", diag_summary_color(error_count), "summary")
                } else {
                    "summary".to_string()
                },
                error_count,
                warning_count,
            ));
        }

        out
    }
}

fn diag_summary_color(error_count: usize) -> &'static str {
    if error_count > 0 {
        "\x1b[31m" // red
    } else {
        "\x1b[33m" // yellow (warnings only)
    }
}

impl Default for DiagnosticRenderer {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Levenshtein distance & suggestion helpers
// ---------------------------------------------------------------------------

/// Compute the Levenshtein edit-distance between two strings.
pub fn levenshtein(a: &str, b: &str) -> usize {
    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();
    let a_len = a_bytes.len();
    let b_len = b_bytes.len();

    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    let mut prev: Vec<usize> = (0..=b_len).collect();
    let mut curr = vec![0usize; b_len + 1];

    for i in 1..=a_len {
        curr[0] = i;
        for j in 1..=b_len {
            let cost = if a_bytes[i - 1] == b_bytes[j - 1] { 0 } else { 1 };
            curr[j] = (prev[j] + 1)
                .min(curr[j - 1] + 1)
                .min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    prev[b_len]
}

/// Suggest CSS property names similar to `input` using Levenshtein distance.
///
/// Returns up to 3 candidates sorted by distance (closest first).
pub fn suggest_property(input: &str) -> Vec<String> {
    suggest_from_list(input, &CSS_PROPERTIES, 3)
}

/// Suggest valid values for a given CSS property, ranked by similarity to `input`.
///
/// Returns up to 3 candidates sorted by distance (closest first).
pub fn suggest_value(property: &str, input: &str) -> Vec<String> {
    let candidates = known_values_for_property(property);
    if candidates.is_empty() {
        return Vec::new();
    }
    suggest_from_list(input, &candidates, 3)
}

/// Generic helper: find the closest strings to `input` from `candidates`.
fn suggest_from_list(input: &str, candidates: &[&str], max: usize) -> Vec<String> {
    let input_lower = input.to_lowercase();
    let threshold = (input_lower.len() / 2).max(2);

    let mut scored: Vec<(usize, &str)> = candidates
        .iter()
        .map(|c| (levenshtein(&input_lower, &c.to_lowercase()), *c))
        .filter(|(d, _)| *d <= threshold)
        .collect();

    scored.sort_by_key(|(d, _)| *d);
    scored.truncate(max);
    scored.into_iter().map(|(_, s)| s.to_string()).collect()
}

/// Well-known CSS property names (a representative subset).
const CSS_PROPERTIES: [&str; 80] = [
    "color",
    "background",
    "background-color",
    "background-image",
    "background-position",
    "background-size",
    "background-repeat",
    "border",
    "border-color",
    "border-width",
    "border-style",
    "border-radius",
    "margin",
    "margin-top",
    "margin-right",
    "margin-bottom",
    "margin-left",
    "padding",
    "padding-top",
    "padding-right",
    "padding-bottom",
    "padding-left",
    "width",
    "height",
    "min-width",
    "min-height",
    "max-width",
    "max-height",
    "display",
    "position",
    "top",
    "right",
    "bottom",
    "left",
    "float",
    "clear",
    "overflow",
    "overflow-x",
    "overflow-y",
    "z-index",
    "font",
    "font-family",
    "font-size",
    "font-weight",
    "font-style",
    "line-height",
    "text-align",
    "text-decoration",
    "text-transform",
    "letter-spacing",
    "word-spacing",
    "white-space",
    "vertical-align",
    "opacity",
    "visibility",
    "cursor",
    "box-shadow",
    "text-shadow",
    "transition",
    "transform",
    "animation",
    "animation-name",
    "animation-duration",
    "flex",
    "flex-direction",
    "flex-wrap",
    "justify-content",
    "align-items",
    "align-self",
    "gap",
    "grid",
    "grid-template-columns",
    "grid-template-rows",
    "grid-column",
    "grid-row",
    "grid-gap",
    "outline",
    "list-style",
    "content",
    "pointer-events",
];

/// Return well-known values for a CSS property.
fn known_values_for_property(property: &str) -> Vec<&'static str> {
    match property {
        "display" => vec![
            "block", "inline", "inline-block", "flex", "inline-flex", "grid",
            "inline-grid", "none", "contents", "table", "list-item", "flow-root",
        ],
        "position" => vec!["static", "relative", "absolute", "fixed", "sticky"],
        "overflow" | "overflow-x" | "overflow-y" => {
            vec!["visible", "hidden", "scroll", "auto", "clip"]
        }
        "visibility" => vec!["visible", "hidden", "collapse"],
        "float" => vec!["left", "right", "none", "inline-start", "inline-end"],
        "clear" => vec!["left", "right", "both", "none", "inline-start", "inline-end"],
        "text-align" => vec!["left", "right", "center", "justify", "start", "end"],
        "text-decoration" => vec!["none", "underline", "overline", "line-through"],
        "text-transform" => vec!["none", "capitalize", "uppercase", "lowercase", "full-width"],
        "font-style" => vec!["normal", "italic", "oblique"],
        "font-weight" => vec![
            "normal", "bold", "bolder", "lighter",
            "100", "200", "300", "400", "500", "600", "700", "800", "900",
        ],
        "cursor" => vec![
            "auto", "default", "pointer", "wait", "text", "move", "not-allowed",
            "crosshair", "grab", "grabbing", "col-resize", "row-resize", "help",
        ],
        "flex-direction" => vec!["row", "row-reverse", "column", "column-reverse"],
        "flex-wrap" => vec!["nowrap", "wrap", "wrap-reverse"],
        "justify-content" => vec![
            "flex-start", "flex-end", "center", "space-between", "space-around",
            "space-evenly", "start", "end",
        ],
        "align-items" => vec![
            "flex-start", "flex-end", "center", "baseline", "stretch",
            "start", "end", "self-start", "self-end",
        ],
        "align-self" => vec![
            "auto", "flex-start", "flex-end", "center", "baseline", "stretch",
        ],
        "white-space" => vec!["normal", "nowrap", "pre", "pre-wrap", "pre-line", "break-spaces"],
        "pointer-events" => vec!["auto", "none"],
        "border-style" => vec![
            "none", "hidden", "dotted", "dashed", "solid", "double", "groove",
            "ridge", "inset", "outset",
        ],
        "list-style" => vec!["none", "disc", "circle", "square", "decimal"],
        _ => Vec::new(),
    }
}

// ---------------------------------------------------------------------------
// Pseudo-class / pseudo-element knowledge base for E010
// ---------------------------------------------------------------------------

/// Well-known CSS pseudo-classes.
pub const PSEUDO_CLASSES: [&str; 24] = [
    "hover", "focus", "active", "visited", "link", "checked", "disabled",
    "enabled", "first-child", "last-child", "nth-child", "nth-last-child",
    "first-of-type", "last-of-type", "nth-of-type", "nth-last-of-type",
    "only-child", "only-of-type", "empty", "not", "is", "where", "has",
    "focus-visible",
];

/// Well-known CSS pseudo-elements.
pub const PSEUDO_ELEMENTS: [&str; 6] = [
    "before", "after", "first-line", "first-letter", "placeholder", "selection",
];

/// Suggest a pseudo-class or pseudo-element similar to `input`.
pub fn suggest_pseudo(input: &str) -> Vec<String> {
    let all: Vec<&str> = PSEUDO_CLASSES
        .iter()
        .chain(PSEUDO_ELEMENTS.iter())
        .copied()
        .collect();
    suggest_from_list(input, &all, 3)
}

// ---------------------------------------------------------------------------
// Well-known at-rules for E006
// ---------------------------------------------------------------------------

/// Well-known CSS at-rules.
pub const AT_RULES: [&str; 12] = [
    "media", "keyframes", "font-face", "import", "charset", "supports",
    "layer", "container", "page", "namespace", "counter-style", "property",
];

/// Suggest an at-rule similar to `input`.
pub fn suggest_at_rule(input: &str) -> Vec<String> {
    suggest_from_list(input, &AT_RULES, 3)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Levenshtein --

    #[test]
    fn test_levenshtein_identical() {
        assert_eq!(levenshtein("color", "color"), 0);
    }

    #[test]
    fn test_levenshtein_empty() {
        assert_eq!(levenshtein("", "abc"), 3);
        assert_eq!(levenshtein("abc", ""), 3);
        assert_eq!(levenshtein("", ""), 0);
    }

    #[test]
    fn test_levenshtein_substitution() {
        assert_eq!(levenshtein("cat", "bat"), 1);
    }

    #[test]
    fn test_levenshtein_insertion() {
        assert_eq!(levenshtein("clor", "color"), 1);
    }

    #[test]
    fn test_levenshtein_deletion() {
        assert_eq!(levenshtein("colour", "color"), 1);
    }

    // -- Property suggestions --

    #[test]
    fn test_suggest_property_close_typo() {
        let suggestions = suggest_property("colr");
        assert!(!suggestions.is_empty(), "Should suggest something for 'colr'");
        assert_eq!(suggestions[0], "color");
    }

    #[test]
    fn test_suggest_property_backgroud() {
        let suggestions = suggest_property("backgroud");
        assert!(!suggestions.is_empty());
        assert!(
            suggestions.iter().any(|s| s == "background"),
            "Should suggest 'background', got: {:?}",
            suggestions,
        );
    }

    #[test]
    fn test_suggest_property_no_match() {
        let suggestions = suggest_property("xyzzyplugh");
        assert!(suggestions.is_empty(), "Completely unrelated input should yield no suggestions");
    }

    // -- Value suggestions --

    #[test]
    fn test_suggest_value_display() {
        let suggestions = suggest_value("display", "blok");
        assert!(!suggestions.is_empty());
        assert_eq!(suggestions[0], "block");
    }

    #[test]
    fn test_suggest_value_unknown_property() {
        let suggestions = suggest_value("made-up-property", "foo");
        assert!(suggestions.is_empty());
    }

    // -- Pseudo suggestions --

    #[test]
    fn test_suggest_pseudo_hover_typo() {
        let suggestions = suggest_pseudo("hovr");
        assert!(!suggestions.is_empty());
        assert_eq!(suggestions[0], "hover");
    }

    // -- At-rule suggestions --

    #[test]
    fn test_suggest_at_rule_keyframe() {
        let suggestions = suggest_at_rule("keyframs");
        assert!(!suggestions.is_empty());
        assert_eq!(suggestions[0], "keyframes");
    }

    // -- ErrorCode --

    #[test]
    fn test_error_code_display() {
        assert_eq!(ErrorCode::E001.as_str(), "E001");
        assert_eq!(ErrorCode::E010.as_str(), "E010");
    }

    #[test]
    fn test_error_code_explanation_nonempty() {
        let codes = [
            ErrorCode::E001, ErrorCode::E002, ErrorCode::E003, ErrorCode::E004,
            ErrorCode::E005, ErrorCode::E006, ErrorCode::E007, ErrorCode::E008,
            ErrorCode::E009, ErrorCode::E010,
        ];
        for code in &codes {
            assert!(!code.explanation().is_empty(), "{} should have an explanation", code);
        }
    }

    // -- Diagnostic construction --

    #[test]
    fn test_diagnostic_error_builder() {
        let diag = Diagnostic::error("test error", Span::new(0, 5, 1, 1))
            .with_code(ErrorCode::E001)
            .with_source_snippet("colr: red;");

        assert_eq!(diag.severity, Severity::Error);
        assert_eq!(diag.code, Some(ErrorCode::E001));
        assert!(diag.source_snippet.is_some());
    }

    #[test]
    fn test_diagnostic_from_compiler_error() {
        let err = CompilerError::syntax("Unclosed brace", Span::new(0, 10, 1, 1));
        let source = ".test { color: red;";
        let diag = Diagnostic::from_compiler_error(&err, source);

        assert_eq!(diag.severity, Severity::Error);
        assert_eq!(diag.code, Some(ErrorCode::E003));
        assert!(diag.source_snippet.is_some());
    }

    // -- DiagnosticRenderer --

    #[test]
    fn test_render_plain_contains_essential_parts() {
        let diag = Diagnostic::error("Unknown property 'colr'", Span::new(0, 4, 1, 1))
            .with_code(ErrorCode::E001)
            .with_source_snippet("colr: red;")
            .with_did_you_mean("color", Span::new(0, 4, 1, 1));

        let renderer = DiagnosticRenderer::plain();
        let output = renderer.render(&diag, "test.wcss");

        assert!(output.contains("error"), "Should contain severity label");
        assert!(output.contains("E001"), "Should contain error code");
        assert!(output.contains("Unknown property 'colr'"), "Should contain message");
        assert!(output.contains("test.wcss:1:1"), "Should contain file location");
        assert!(output.contains("colr: red;"), "Should contain source snippet");
        assert!(output.contains("^^^^"), "Should contain caret underline");
        assert!(output.contains("Did you mean `color`?"), "Should contain suggestion");
        assert!(output.contains("E001"), "Should contain error code in note");
    }

    #[test]
    fn test_render_colored_contains_ansi() {
        let diag = Diagnostic::error("test", Span::new(0, 1, 1, 1))
            .with_source_snippet("x");

        let renderer = DiagnosticRenderer::new();
        let output = renderer.render(&diag, "f.wcss");

        assert!(output.contains("\x1b[31m"), "Should contain red ANSI code for error");
        assert!(output.contains("\x1b[0m"), "Should contain ANSI reset");
    }

    #[test]
    fn test_render_all_summary() {
        let diags = vec![
            Diagnostic::error("err1", Span::new(0, 1, 1, 1)),
            Diagnostic::warning("warn1", Span::new(5, 6, 2, 1)),
            Diagnostic::error("err2", Span::new(10, 11, 3, 1)),
        ];

        let renderer = DiagnosticRenderer::plain();
        let output = renderer.render_all(&diags, "test.wcss", None);

        assert!(output.contains("2 error(s)"), "Summary should count 2 errors");
        assert!(output.contains("1 warning(s)"), "Summary should count 1 warning");
    }

    // -- Severity ordering --

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Error < Severity::Warning);
        assert!(Severity::Warning < Severity::Info);
        assert!(Severity::Info < Severity::Hint);
    }
}
