use serde::{Deserialize, Serialize};

use crate::ast::Span;

/// Result of a WCSS compilation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompileResult {
    pub css: String,
    pub js: Option<String>,
    pub source_map: Option<String>,
    pub errors: Vec<CompilerError>,
    pub warnings: Vec<CompilerWarning>,
    pub stats: CompilationStats,
}

/// A compiler error with location and suggestion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilerError {
    pub kind: ErrorKind,
    pub message: String,
    pub span: Span,
    pub suggestion: Option<String>,
}

impl CompilerError {
    pub fn syntax(message: impl Into<String>, span: Span) -> Self {
        Self {
            kind: ErrorKind::SyntaxError,
            message: message.into(),
            span,
            suggestion: None,
        }
    }

    pub fn validation(message: impl Into<String>, span: Span) -> Self {
        Self {
            kind: ErrorKind::ValidationError,
            message: message.into(),
            span,
            suggestion: None,
        }
    }

    pub fn token_not_found(name: &str, span: Span, suggestion: Option<String>) -> Self {
        Self {
            kind: ErrorKind::TokenNotFound,
            message: format!("Undefined token '{name}'"),
            span,
            suggestion,
        }
    }

    pub fn circular_reference(chain: &[String], span: Span) -> Self {
        Self {
            kind: ErrorKind::CircularReference,
            message: format!("Circular token reference: {}", chain.join(" -> ")),
            span,
            suggestion: Some("Remove the circular dependency in your token definitions".to_string()),
        }
    }

    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    /// Format error with code snippet for display.
    pub fn format_with_source(&self, source: &str, file_path: &str) -> String {
        let lines: Vec<&str> = source.lines().collect();
        let line_num = self.span.line;
        let col = self.span.column;

        let mut output = format!(
            "Error: {}\n  \u{250c}\u{2500} {}:{}:{}\n  \u{2502}\n",
            self.message, file_path, line_num, col
        );

        if line_num > 0 && line_num <= lines.len() {
            let line_content = lines[line_num - 1];
            output.push_str(&format!("{:>3}\u{2502}   {}\n", line_num, line_content));

            let underline_start = if col > 0 { col - 1 } else { 0 };
            let underline_len = (self.span.end - self.span.start).max(1);
            output.push_str(&format!(
                "   \u{2502}   {}{}\n",
                " ".repeat(underline_start),
                "^".repeat(underline_len)
            ));
        }

        if let Some(ref suggestion) = self.suggestion {
            output.push_str(&format!("  = help: {}\n", suggestion));
        }

        output
    }
}

/// Categories of compiler errors.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorKind {
    SyntaxError,
    ValidationError,
    TokenNotFound,
    CircularReference,
    W3C(W3CErrorKind),
}

/// W3C Design Tokens specific error kinds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum W3CErrorKind {
    InvalidJSON,
    MissingField,
    InvalidType,
    InvalidStructure,
    ReferenceNotFound,
    TypeMismatch,
    ConflictingToken,
    InvalidTransformation,
}

/// A compiler warning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilerWarning {
    pub message: String,
    pub span: Span,
}

/// Statistics from a compilation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CompilationStats {
    pub input_size: usize,
    pub output_size: usize,
    pub compile_time_us: u64,
    pub rules_processed: usize,
    pub rules_eliminated: usize,
}
