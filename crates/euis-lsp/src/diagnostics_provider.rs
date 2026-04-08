use tower_lsp::lsp_types::*;
use euis_compiler::error::CompilerError;

/// Parse the given Euis source and convert any compiler errors to LSP diagnostics.
pub fn get_diagnostics(uri: &str, source: &str) -> Vec<Diagnostic> {
    let parse_result = euis_compiler::parse(source);

    match parse_result {
        Ok(_ast) => {
            // Parsing succeeded. Run compilation to catch validation errors as well.
            let config = euis_compiler::config::CompilerConfig {
                minify: false,
                ..Default::default()
            };
            let compile_result = euis_compiler::compile(source, &config);

            let mut diagnostics = Vec::new();
            for err in &compile_result.errors {
                diagnostics.push(compiler_error_to_diagnostic(err, source));
            }
            for warn in &compile_result.warnings {
                diagnostics.push(Diagnostic {
                    range: span_to_range(&warn.span, source),
                    severity: Some(DiagnosticSeverity::WARNING),
                    source: Some("euis".to_string()),
                    message: warn.message.clone(),
                    code: None,
                    code_description: None,
                    related_information: None,
                    tags: None,
                    data: None,
                });
            }
            diagnostics
        }
        Err(errors) => {
            errors
                .iter()
                .map(|err| compiler_error_to_diagnostic(err, source))
                .collect()
        }
    }
}

/// Convert a `CompilerError` into an LSP `Diagnostic`.
fn compiler_error_to_diagnostic(err: &CompilerError, source: &str) -> Diagnostic {
    let severity = match err.kind {
        euis_compiler::error::ErrorKind::SyntaxError => DiagnosticSeverity::ERROR,
        euis_compiler::error::ErrorKind::ValidationError => DiagnosticSeverity::WARNING,
        euis_compiler::error::ErrorKind::TokenNotFound => DiagnosticSeverity::ERROR,
        euis_compiler::error::ErrorKind::CircularReference => DiagnosticSeverity::ERROR,
        _ => DiagnosticSeverity::ERROR,
    };

    let range = span_to_range(&err.span, source);

    let mut message = err.message.clone();
    if let Some(suggestion) = &err.suggestion {
        message.push_str(&format!("\nSuggestion: {suggestion}"));
    }

    let code = error_kind_to_code(&err.kind);

    Diagnostic {
        range,
        severity: Some(severity),
        source: Some("euis".to_string()),
        message,
        code: code.map(NumberOrString::String),
        code_description: None,
        related_information: None,
        tags: None,
        data: None,
    }
}

/// Map an `ErrorKind` to a string error code for the diagnostic.
fn error_kind_to_code(kind: &euis_compiler::error::ErrorKind) -> Option<String> {
    match kind {
        euis_compiler::error::ErrorKind::SyntaxError => Some("E003".to_string()),
        euis_compiler::error::ErrorKind::ValidationError => Some("E002".to_string()),
        euis_compiler::error::ErrorKind::TokenNotFound => Some("E001".to_string()),
        euis_compiler::error::ErrorKind::CircularReference => Some("E009".to_string()),
        _ => None,
    }
}

/// Convert an AST `Span` to an LSP `Range`.
///
/// The span stores 1-based line/column and byte offsets. LSP uses 0-based
/// line/character positions.
fn span_to_range(span: &euis_compiler::ast::Span, source: &str) -> Range {
    // Use byte offsets to compute accurate start/end positions.
    let start = offset_to_position(span.start, source);
    let end = if span.end > span.start {
        offset_to_position(span.end, source)
    } else {
        // If end == start, highlight at least the rest of the current line.
        Position::new(
            start.line,
            start.character + 1,
        )
    };

    Range { start, end }
}

/// Convert a byte offset into a 0-based LSP Position (line, character).
fn offset_to_position(offset: usize, source: &str) -> Position {
    let offset = offset.min(source.len());
    let mut line = 0u32;
    let mut col = 0u32;

    for (i, ch) in source.char_indices() {
        if i >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
    }

    Position::new(line, col)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_css_no_diagnostics() {
        let source = ".btn { color: red; }";
        let diags = get_diagnostics("file:///test.euis", source);
        assert!(diags.is_empty(), "Expected no diagnostics for valid CSS, got: {:?}", diags);
    }

    #[test]
    fn test_offset_to_position_basic() {
        let source = "abc\ndef\nghi";
        assert_eq!(offset_to_position(0, source), Position::new(0, 0));
        assert_eq!(offset_to_position(4, source), Position::new(1, 0));
        assert_eq!(offset_to_position(5, source), Position::new(1, 1));
    }
}
