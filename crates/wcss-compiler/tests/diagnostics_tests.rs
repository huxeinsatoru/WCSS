// Integration tests for the diagnostics module.

use wcss_compiler::ast::Span;
use wcss_compiler::diagnostics::{
    levenshtein, suggest_at_rule, suggest_property, suggest_pseudo, suggest_value,
    Diagnostic, DiagnosticRenderer, ErrorCode, Severity, Suggestion,
};
use wcss_compiler::error::CompilerError;

// ============================================================================
// Levenshtein distance
// ============================================================================

#[test]
fn test_levenshtein_identical_strings() {
    assert_eq!(levenshtein("display", "display"), 0);
}

#[test]
fn test_levenshtein_single_insertion() {
    assert_eq!(levenshtein("colr", "color"), 1);
}

#[test]
fn test_levenshtein_single_deletion() {
    assert_eq!(levenshtein("padding", "paddin"), 1);
}

#[test]
fn test_levenshtein_single_substitution() {
    assert_eq!(levenshtein("margin", "margon"), 1);
}

#[test]
fn test_levenshtein_completely_different() {
    assert!(levenshtein("abc", "xyz") > 0);
    assert_eq!(levenshtein("abc", "xyz"), 3);
}

#[test]
fn test_levenshtein_empty_strings() {
    assert_eq!(levenshtein("", ""), 0);
    assert_eq!(levenshtein("abc", ""), 3);
    assert_eq!(levenshtein("", "abc"), 3);
}

#[test]
fn test_levenshtein_case_sensitive() {
    assert_eq!(levenshtein("Color", "color"), 1);
}

// ============================================================================
// Property suggestions (E001)
// ============================================================================

#[test]
fn test_suggest_property_color_typo() {
    let suggestions = suggest_property("colr");
    assert!(!suggestions.is_empty());
    assert_eq!(suggestions[0], "color");
}

#[test]
fn test_suggest_property_background_typo() {
    let suggestions = suggest_property("backgrond");
    assert!(!suggestions.is_empty());
    assert!(suggestions.iter().any(|s| s == "background"));
}

#[test]
fn test_suggest_property_margin_typo() {
    let suggestions = suggest_property("marign");
    assert!(!suggestions.is_empty());
    assert_eq!(suggestions[0], "margin");
}

#[test]
fn test_suggest_property_display_typo() {
    let suggestions = suggest_property("dispaly");
    assert!(!suggestions.is_empty());
    assert_eq!(suggestions[0], "display");
}

#[test]
fn test_suggest_property_padding_typo() {
    let suggestions = suggest_property("paddin");
    assert!(!suggestions.is_empty());
    assert_eq!(suggestions[0], "padding");
}

#[test]
fn test_suggest_property_nonsense_returns_empty() {
    let suggestions = suggest_property("zzzzzzzzzzz");
    assert!(suggestions.is_empty());
}

#[test]
fn test_suggest_property_max_three_results() {
    let suggestions = suggest_property("border");
    assert!(suggestions.len() <= 3);
}

// ============================================================================
// Value suggestions (E002)
// ============================================================================

#[test]
fn test_suggest_value_display_block() {
    let suggestions = suggest_value("display", "blok");
    assert!(!suggestions.is_empty());
    assert_eq!(suggestions[0], "block");
}

#[test]
fn test_suggest_value_display_flex() {
    let suggestions = suggest_value("display", "flx");
    assert!(!suggestions.is_empty());
    assert_eq!(suggestions[0], "flex");
}

#[test]
fn test_suggest_value_position_relative() {
    let suggestions = suggest_value("position", "relativ");
    assert!(!suggestions.is_empty());
    assert_eq!(suggestions[0], "relative");
}

#[test]
fn test_suggest_value_text_align_centr() {
    let suggestions = suggest_value("text-align", "centr");
    assert!(!suggestions.is_empty());
    assert_eq!(suggestions[0], "center");
}

#[test]
fn test_suggest_value_unknown_property() {
    let suggestions = suggest_value("fake-property", "anything");
    assert!(suggestions.is_empty());
}

#[test]
fn test_suggest_value_cursor_pointr() {
    let suggestions = suggest_value("cursor", "pointr");
    assert!(!suggestions.is_empty());
    assert_eq!(suggestions[0], "pointer");
}

#[test]
fn test_suggest_value_flex_direction() {
    let suggestions = suggest_value("flex-direction", "colum");
    assert!(!suggestions.is_empty());
    assert_eq!(suggestions[0], "column");
}

// ============================================================================
// Pseudo-class / pseudo-element suggestions (E010)
// ============================================================================

#[test]
fn test_suggest_pseudo_hover() {
    let suggestions = suggest_pseudo("hovr");
    assert!(!suggestions.is_empty());
    assert_eq!(suggestions[0], "hover");
}

#[test]
fn test_suggest_pseudo_focus() {
    let suggestions = suggest_pseudo("focsu");
    assert!(!suggestions.is_empty());
    assert_eq!(suggestions[0], "focus");
}

#[test]
fn test_suggest_pseudo_before() {
    let suggestions = suggest_pseudo("befor");
    assert!(!suggestions.is_empty());
    assert_eq!(suggestions[0], "before");
}

#[test]
fn test_suggest_pseudo_first_child() {
    let suggestions = suggest_pseudo("first-chld");
    assert!(!suggestions.is_empty());
    assert_eq!(suggestions[0], "first-child");
}

// ============================================================================
// At-rule suggestions (E006)
// ============================================================================

#[test]
fn test_suggest_at_rule_media() {
    let suggestions = suggest_at_rule("meda");
    assert!(!suggestions.is_empty());
    assert_eq!(suggestions[0], "media");
}

#[test]
fn test_suggest_at_rule_keyframes() {
    let suggestions = suggest_at_rule("keyframs");
    assert!(!suggestions.is_empty());
    assert_eq!(suggestions[0], "keyframes");
}

#[test]
fn test_suggest_at_rule_supports() {
    let suggestions = suggest_at_rule("suports");
    assert!(!suggestions.is_empty());
    assert_eq!(suggestions[0], "supports");
}

// ============================================================================
// Error codes
// ============================================================================

#[test]
fn test_error_code_e001_unknown_property() {
    let code = ErrorCode::E001;
    assert_eq!(code.as_str(), "E001");
    assert!(code.explanation().contains("property"));
}

#[test]
fn test_error_code_e002_invalid_value() {
    let code = ErrorCode::E002;
    assert_eq!(code.as_str(), "E002");
    assert!(code.explanation().contains("value"));
}

#[test]
fn test_error_code_e003_unclosed_bracket() {
    let code = ErrorCode::E003;
    assert_eq!(code.as_str(), "E003");
    assert!(code.explanation().contains("bracket") || code.explanation().contains("brace"));
}

#[test]
fn test_error_code_e004_invalid_selector() {
    let code = ErrorCode::E004;
    assert_eq!(code.as_str(), "E004");
    assert!(code.explanation().contains("selector"));
}

#[test]
fn test_error_code_e005_duplicate_property() {
    let code = ErrorCode::E005;
    assert_eq!(code.as_str(), "E005");
    assert!(code.explanation().contains("property"));
}

#[test]
fn test_error_code_e006_unknown_at_rule() {
    let code = ErrorCode::E006;
    assert_eq!(code.as_str(), "E006");
    assert!(code.explanation().contains("at-rule"));
}

#[test]
fn test_error_code_e007_invalid_color() {
    let code = ErrorCode::E007;
    assert_eq!(code.as_str(), "E007");
    assert!(code.explanation().contains("color"));
}

#[test]
fn test_error_code_e008_missing_semicolon() {
    let code = ErrorCode::E008;
    assert_eq!(code.as_str(), "E008");
    assert!(code.explanation().contains("semicolon"));
}

#[test]
fn test_error_code_e009_invalid_nesting() {
    let code = ErrorCode::E009;
    assert_eq!(code.as_str(), "E009");
    assert!(code.explanation().contains("esting")); // "Nesting"
}

#[test]
fn test_error_code_e010_unknown_pseudo() {
    let code = ErrorCode::E010;
    assert_eq!(code.as_str(), "E010");
    assert!(code.explanation().contains("pseudo"));
}

#[test]
fn test_all_error_codes_have_distinct_strings() {
    let codes = [
        ErrorCode::E001, ErrorCode::E002, ErrorCode::E003, ErrorCode::E004,
        ErrorCode::E005, ErrorCode::E006, ErrorCode::E007, ErrorCode::E008,
        ErrorCode::E009, ErrorCode::E010,
    ];
    for i in 0..codes.len() {
        for j in (i + 1)..codes.len() {
            assert_ne!(codes[i].as_str(), codes[j].as_str());
        }
    }
}

// ============================================================================
// Diagnostic construction
// ============================================================================

#[test]
fn test_diagnostic_error_severity() {
    let diag = Diagnostic::error("msg", Span::empty());
    assert_eq!(diag.severity, Severity::Error);
}

#[test]
fn test_diagnostic_warning_severity() {
    let diag = Diagnostic::warning("msg", Span::empty());
    assert_eq!(diag.severity, Severity::Warning);
}

#[test]
fn test_diagnostic_info_severity() {
    let diag = Diagnostic::info("msg", Span::empty());
    assert_eq!(diag.severity, Severity::Info);
}

#[test]
fn test_diagnostic_hint_severity() {
    let diag = Diagnostic::hint("msg", Span::empty());
    assert_eq!(diag.severity, Severity::Hint);
}

#[test]
fn test_diagnostic_with_code() {
    let diag = Diagnostic::error("msg", Span::empty()).with_code(ErrorCode::E001);
    assert_eq!(diag.code, Some(ErrorCode::E001));
}

#[test]
fn test_diagnostic_with_source_snippet() {
    let diag = Diagnostic::error("msg", Span::empty())
        .with_source_snippet("color: red;");
    assert_eq!(diag.source_snippet, Some("color: red;".to_string()));
}

#[test]
fn test_diagnostic_with_suggestion() {
    let diag = Diagnostic::error("msg", Span::new(0, 4, 1, 1))
        .with_suggestion(Suggestion {
            message: "Try this".into(),
            replacement: "color".into(),
            span: Span::new(0, 4, 1, 1),
        });
    assert_eq!(diag.suggestions.len(), 1);
    assert_eq!(diag.suggestions[0].replacement, "color");
}

#[test]
fn test_diagnostic_with_did_you_mean() {
    let span = Span::new(0, 4, 1, 1);
    let diag = Diagnostic::error("msg", span.clone())
        .with_did_you_mean("color", span);
    assert_eq!(diag.suggestions.len(), 1);
    assert!(diag.suggestions[0].message.contains("Did you mean"));
    assert!(diag.suggestions[0].message.contains("color"));
}

#[test]
fn test_diagnostic_multiple_suggestions() {
    let span = Span::new(0, 4, 1, 1);
    let diag = Diagnostic::error("msg", span.clone())
        .with_did_you_mean("color", span.clone())
        .with_suggestion(Suggestion {
            message: "Another fix".into(),
            replacement: "colour".into(),
            span,
        });
    assert_eq!(diag.suggestions.len(), 2);
}

// ============================================================================
// Diagnostic from CompilerError
// ============================================================================

#[test]
fn test_from_compiler_error_syntax_unclosed_brace() {
    let err = CompilerError::syntax("Unclosed brace", Span::new(0, 10, 1, 1));
    let source = ".test { color: red;";
    let diag = Diagnostic::from_compiler_error(&err, source);

    assert_eq!(diag.severity, Severity::Error);
    assert_eq!(diag.code, Some(ErrorCode::E003));
    assert_eq!(diag.source_snippet, Some(".test { color: red;".to_string()));
}

#[test]
fn test_from_compiler_error_syntax_missing_semicolon() {
    let err = CompilerError::syntax(
        "Expected ';' after declaration",
        Span::new(18, 19, 1, 19),
    );
    let source = ".test { color: red }";
    let diag = Diagnostic::from_compiler_error(&err, source);

    assert_eq!(diag.code, Some(ErrorCode::E008));
}

#[test]
fn test_from_compiler_error_preserves_suggestion() {
    let err = CompilerError::syntax("Bad selector", Span::new(0, 5, 1, 1))
        .with_suggestion("Use a class selector like .foo");
    let diag = Diagnostic::from_compiler_error(&err, ".test{}");

    assert_eq!(diag.suggestions.len(), 1);
    assert!(diag.suggestions[0].message.contains("class selector"));
}

#[test]
fn test_from_compiler_error_multiline_source() {
    let err = CompilerError::syntax("Error on line 2", Span::new(20, 25, 2, 3));
    let source = ".a { color: red; }\n  font-size: 12px;";
    let diag = Diagnostic::from_compiler_error(&err, source);

    assert_eq!(
        diag.source_snippet,
        Some("  font-size: 12px;".to_string()),
    );
}

// ============================================================================
// DiagnosticRenderer - plain output
// ============================================================================

#[test]
fn test_render_plain_error_with_code() {
    let diag = Diagnostic::error("Unknown property 'colr'", Span::new(0, 4, 1, 1))
        .with_code(ErrorCode::E001)
        .with_source_snippet("colr: red;");

    let renderer = DiagnosticRenderer::plain();
    let output = renderer.render(&diag, "style.wcss");

    assert!(output.contains("error[E001]"));
    assert!(output.contains("Unknown property 'colr'"));
    assert!(output.contains("style.wcss:1:1"));
    assert!(output.contains("colr: red;"));
    assert!(output.contains("^^^^"));
}

#[test]
fn test_render_plain_warning() {
    let diag = Diagnostic::warning("Duplicate property", Span::new(10, 15, 2, 3))
        .with_code(ErrorCode::E005);

    let renderer = DiagnosticRenderer::plain();
    let output = renderer.render(&diag, "style.wcss");

    assert!(output.contains("warning[E005]"));
}

#[test]
fn test_render_plain_includes_suggestion() {
    let span = Span::new(0, 4, 1, 1);
    let diag = Diagnostic::error("Unknown property", span.clone())
        .with_did_you_mean("color", span);

    let renderer = DiagnosticRenderer::plain();
    let output = renderer.render(&diag, "f.wcss");

    assert!(output.contains("Did you mean `color`?"));
}

#[test]
fn test_render_plain_includes_error_code_note() {
    let diag = Diagnostic::error("Bad value", Span::new(0, 3, 1, 1))
        .with_code(ErrorCode::E002);

    let renderer = DiagnosticRenderer::plain();
    let output = renderer.render(&diag, "f.wcss");

    assert!(output.contains("[E002]"));
    assert!(output.contains("not valid for the given property"));
}

#[test]
fn test_render_caret_width_matches_span() {
    let diag = Diagnostic::error("msg", Span::new(5, 12, 1, 6))
        .with_source_snippet("test colr: red;");

    let renderer = DiagnosticRenderer::plain();
    let output = renderer.render(&diag, "f.wcss");

    // Span length is 12 - 5 = 7, so 7 carets
    assert!(output.contains("^^^^^^^"));
}

#[test]
fn test_render_minimum_one_caret() {
    let diag = Diagnostic::error("msg", Span::new(5, 5, 1, 6))
        .with_source_snippet("hello");

    let renderer = DiagnosticRenderer::plain();
    let output = renderer.render(&diag, "f.wcss");

    // When start == end, should still render at least 1 caret
    assert!(output.contains("^"));
}

// ============================================================================
// DiagnosticRenderer - colored output
// ============================================================================

#[test]
fn test_render_colored_error_has_red() {
    let diag = Diagnostic::error("test", Span::new(0, 1, 1, 1))
        .with_source_snippet("x");

    let renderer = DiagnosticRenderer::new();
    let output = renderer.render(&diag, "f.wcss");

    assert!(output.contains("\x1b[31m"), "Error should use red ANSI");
}

#[test]
fn test_render_colored_warning_has_yellow() {
    let diag = Diagnostic::warning("test", Span::new(0, 1, 1, 1))
        .with_source_snippet("x");

    let renderer = DiagnosticRenderer::new();
    let output = renderer.render(&diag, "f.wcss");

    assert!(output.contains("\x1b[33m"), "Warning should use yellow ANSI");
}

#[test]
fn test_render_colored_info_has_blue() {
    let diag = Diagnostic::info("test", Span::new(0, 1, 1, 1))
        .with_source_snippet("x");

    let renderer = DiagnosticRenderer::new();
    let output = renderer.render(&diag, "f.wcss");

    assert!(output.contains("\x1b[34m"), "Info should use blue ANSI");
}

#[test]
fn test_render_colored_hint_has_green() {
    let diag = Diagnostic::hint("test", Span::new(0, 1, 1, 1))
        .with_source_snippet("x");

    let renderer = DiagnosticRenderer::new();
    let output = renderer.render(&diag, "f.wcss");

    assert!(output.contains("\x1b[32m"), "Hint should use green ANSI");
}

// ============================================================================
// DiagnosticRenderer - render_all
// ============================================================================

#[test]
fn test_render_all_multiple_diagnostics() {
    let diags = vec![
        Diagnostic::error("first error", Span::new(0, 5, 1, 1)),
        Diagnostic::error("second error", Span::new(20, 25, 3, 1)),
    ];

    let renderer = DiagnosticRenderer::plain();
    let output = renderer.render_all(&diags, "test.wcss", None);

    assert!(output.contains("first error"));
    assert!(output.contains("second error"));
    assert!(output.contains("2 error(s)"));
    assert!(output.contains("0 warning(s)"));
}

#[test]
fn test_render_all_mixed_severities() {
    let diags = vec![
        Diagnostic::error("err", Span::new(0, 1, 1, 1)),
        Diagnostic::warning("warn", Span::new(5, 6, 2, 1)),
        Diagnostic::hint("hint", Span::new(10, 11, 3, 1)),
    ];

    let renderer = DiagnosticRenderer::plain();
    let output = renderer.render_all(&diags, "test.wcss", None);

    assert!(output.contains("1 error(s)"));
    assert!(output.contains("1 warning(s)"));
}

#[test]
fn test_render_all_empty_list() {
    let diags: Vec<Diagnostic> = vec![];
    let renderer = DiagnosticRenderer::plain();
    let output = renderer.render_all(&diags, "test.wcss", None);

    // No summary for empty list
    assert!(!output.contains("error(s)"));
}

#[test]
fn test_render_all_uses_source_for_snippet() {
    let source = ".test { colr: red; }";
    let diag = Diagnostic::error("bad property", Span::new(8, 12, 1, 9));

    let renderer = DiagnosticRenderer::plain();
    let output = renderer.render_all(&[diag], "test.wcss", Some(source));

    assert!(
        output.contains(".test { colr: red; }"),
        "Should extract snippet from source when not already set",
    );
}

// ============================================================================
// Severity
// ============================================================================

#[test]
fn test_severity_labels() {
    assert_eq!(Severity::Error.label(), "error");
    assert_eq!(Severity::Warning.label(), "warning");
    assert_eq!(Severity::Info.label(), "info");
    assert_eq!(Severity::Hint.label(), "hint");
}

#[test]
fn test_severity_ordering() {
    assert!(Severity::Error < Severity::Warning);
    assert!(Severity::Warning < Severity::Info);
    assert!(Severity::Info < Severity::Hint);
}

#[test]
fn test_severity_display() {
    assert_eq!(format!("{}", Severity::Error), "error");
    assert_eq!(format!("{}", Severity::Warning), "warning");
}

// ============================================================================
// Suggestion accuracy - property typos
// ============================================================================

#[test]
fn test_suggest_property_font_size_typo() {
    let suggestions = suggest_property("font-siz");
    assert!(!suggestions.is_empty());
    assert_eq!(suggestions[0], "font-size");
}

#[test]
fn test_suggest_property_border_radius_typo() {
    let suggestions = suggest_property("border-radus");
    assert!(!suggestions.is_empty());
    assert_eq!(suggestions[0], "border-radius");
}

#[test]
fn test_suggest_property_justify_content_typo() {
    let suggestions = suggest_property("justify-conten");
    assert!(!suggestions.is_empty());
    assert_eq!(suggestions[0], "justify-content");
}

#[test]
fn test_suggest_property_align_items_typo() {
    let suggestions = suggest_property("align-item");
    assert!(!suggestions.is_empty());
    assert_eq!(suggestions[0], "align-items");
}

// ============================================================================
// Suggestion accuracy - value typos
// ============================================================================

#[test]
fn test_suggest_value_overflow_hidden() {
    let suggestions = suggest_value("overflow", "hiden");
    assert!(!suggestions.is_empty());
    assert_eq!(suggestions[0], "hidden");
}

#[test]
fn test_suggest_value_font_weight_bold() {
    let suggestions = suggest_value("font-weight", "blod");
    assert!(!suggestions.is_empty());
    assert_eq!(suggestions[0], "bold");
}

#[test]
fn test_suggest_value_justify_content_center() {
    let suggestions = suggest_value("justify-content", "centr");
    assert!(!suggestions.is_empty());
    assert_eq!(suggestions[0], "center");
}

// ============================================================================
// Edge cases
// ============================================================================

#[test]
fn test_diagnostic_zero_line_span() {
    // Line 0 means no location - should not crash renderer
    let diag = Diagnostic::error("msg", Span::new(0, 0, 0, 0));
    let renderer = DiagnosticRenderer::plain();
    let output = renderer.render(&diag, "f.wcss");
    assert!(output.contains("error"));
}

#[test]
fn test_suggest_property_single_char() {
    // Very short input - should not panic
    let suggestions = suggest_property("a");
    // May or may not return results, just should not panic
    let _ = suggestions;
}

#[test]
fn test_suggest_property_empty_string() {
    let suggestions = suggest_property("");
    // Empty input - should not panic
    let _ = suggestions;
}

#[test]
fn test_levenshtein_unicode() {
    // Ensure it handles non-ASCII gracefully
    let dist = levenshtein("cafe", "caf\u{00e9}");
    assert!(dist > 0);
}

#[test]
fn test_error_code_display_trait() {
    let code = ErrorCode::E007;
    let formatted = format!("{}", code);
    assert_eq!(formatted, "E007");
}
