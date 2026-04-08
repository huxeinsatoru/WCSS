// Feature: euis-framework, Property 3: Error Reporting Completeness
// **Validates: Requirements 1.2, 14.1, 14.2, 14.3**

use proptest::prelude::*;
use euis_compiler::parser::parse;

/// Generate invalid Euis syntax patterns that definitely produce errors.
fn invalid_euis_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        // Missing closing brace (confirmed to produce error)
        "[a-z][a-z0-9-]{2,8}".prop_map(|class| format!(".{} {{ color: red;", class)),
        
        // Missing colon in declaration
        "[a-z][a-z0-9-]{2,8}".prop_map(|class| format!(".{} {{ color red; }}", class)),
        
        // Unterminated string
        "[a-z][a-z0-9-]{2,8}".prop_map(|class| format!(".{} {{ content: \"unterminated; }}", class)),
        
        // Invalid token reference (missing dot in category)
        "[a-z][a-z0-9-]{2,8}".prop_map(|class| format!(".{} {{ color: $primary; }}", class)),
        
        // Missing opening brace
        "[a-z][a-z0-9-]{2,8}".prop_map(|class| format!(".{} color: red; }}", class)),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    /// Property 3: Error Reporting Completeness
    /// For any invalid Euis syntax, the compiler SHALL return errors that
    /// include file path (via span), line number, column number, and a
    /// descriptive message with code snippet capability.
    ///
    /// Validates: Requirements 1.2, 14.1, 14.2, 14.3
    #[test]
    fn prop_error_reporting_completeness(invalid_source in invalid_euis_strategy()) {
        // Parse invalid Euis
        let result = parse(&invalid_source);
        
        // Verify parsing fails
        prop_assert!(
            result.is_err(),
            "Invalid Euis should produce errors. Source:\n{}",
            invalid_source
        );
        
        let errors = result.unwrap_err();
        
        // Verify at least one error is reported
        prop_assert!(
            !errors.is_empty(),
            "Should report at least one error for invalid syntax"
        );
        
        // Check each error for completeness
        for error in &errors {
            // Requirement 14.1: Error includes line number
            prop_assert!(
                error.span.line > 0,
                "Error should include line number (got {})",
                error.span.line
            );
            
            // Requirement 14.2: Error includes column number
            prop_assert!(
                error.span.column > 0,
                "Error should include column number (got {})",
                error.span.column
            );
            
            // Requirement 14.3: Error includes descriptive message
            prop_assert!(
                !error.message.is_empty(),
                "Error should include a descriptive message"
            );
            
            // Verify message is meaningful (not just generic)
            prop_assert!(
                error.message.len() > 5,
                "Error message should be descriptive (got: '{}')",
                error.message
            );
            
            // Verify span has valid positions
            prop_assert!(
                error.span.start <= error.span.end,
                "Error span should have valid start/end positions"
            );
            
            // Requirement 14.1: Verify error can be formatted with source snippet
            let formatted = error.format_with_source(&invalid_source, "test.euis");
            
            // Verify formatted error includes key components
            prop_assert!(
                formatted.contains("Error:"),
                "Formatted error should include 'Error:' label"
            );
            
            prop_assert!(
                formatted.contains("test.euis"),
                "Formatted error should include file path"
            );
            
            prop_assert!(
                formatted.contains(&error.span.line.to_string()),
                "Formatted error should include line number"
            );
            
            prop_assert!(
                formatted.contains(&error.span.column.to_string()),
                "Formatted error should include column number"
            );
            
            // Verify code snippet is included (should contain part of source)
            let source_lines: Vec<&str> = invalid_source.lines().collect();
            if error.span.line > 0 && error.span.line <= source_lines.len() {
                let error_line = source_lines[error.span.line - 1];
                if !error_line.is_empty() {
                    prop_assert!(
                        formatted.contains(error_line),
                        "Formatted error should include code snippet from source"
                    );
                }
            }
        }
    }
    
    /// Property: Error messages are descriptive
    /// For any parsing error, the message should provide context about what went wrong.
    #[test]
    fn prop_error_messages_are_descriptive(invalid_source in invalid_euis_strategy()) {
        let result = parse(&invalid_source);
        
        if let Err(errors) = result {
            for error in &errors {
                // Error message should contain helpful keywords
                let message_lower = error.message.to_lowercase();
                let has_helpful_keyword = 
                    message_lower.contains("expected") ||
                    message_lower.contains("unexpected") ||
                    message_lower.contains("missing") ||
                    message_lower.contains("invalid") ||
                    message_lower.contains("unterminated") ||
                    message_lower.contains("unclosed") ||
                    message_lower.contains("found");
                
                prop_assert!(
                    has_helpful_keyword,
                    "Error message should contain helpful keywords. Got: '{}'",
                    error.message
                );
            }
        }
    }
    
    /// Property: Error spans point to valid source locations
    /// For any parsing error, the span should reference a valid location in the source.
    #[test]
    fn prop_error_spans_are_valid(invalid_source in invalid_euis_strategy()) {
        let result = parse(&invalid_source);
        
        if let Err(errors) = result {
            let source_len = invalid_source.len();
            let line_count = invalid_source.lines().count();
            
            for error in &errors {
                // Span positions should be within source bounds
                prop_assert!(
                    error.span.start <= source_len,
                    "Error span start should be within source bounds"
                );
                
                prop_assert!(
                    error.span.end <= source_len,
                    "Error span end should be within source bounds"
                );
                
                // Line number should be within source line count
                prop_assert!(
                    error.span.line <= line_count || line_count == 0,
                    "Error line number should be within source line count"
                );
            }
        }
    }
    
    /// Property: Formatted errors are human-readable
    /// For any error, the formatted output should be readable and well-structured.
    #[test]
    fn prop_formatted_errors_are_readable(invalid_source in invalid_euis_strategy()) {
        let result = parse(&invalid_source);
        
        if let Err(errors) = result {
            for error in &errors {
                let formatted = error.format_with_source(&invalid_source, "test.euis");
                
                // Should have multiple lines (header, location, snippet, etc.)
                let line_count = formatted.lines().count();
                prop_assert!(
                    line_count >= 3,
                    "Formatted error should have multiple lines for readability"
                );
                
                // Should use box-drawing characters for visual structure
                prop_assert!(
                    formatted.contains('┌') || formatted.contains('│') || formatted.contains('└'),
                    "Formatted error should use box-drawing characters for structure"
                );
                
                // Should not be excessively long
                prop_assert!(
                    formatted.len() < 2000,
                    "Formatted error should be concise (not excessively long)"
                );
            }
        }
    }
}

/// Unit test: Verify specific error components
#[test]
fn test_error_includes_all_required_components() {
    let invalid_source = ".test { color red; }"; // Missing colon
    let result = parse(invalid_source);
    
    assert!(result.is_err(), "Should produce error for invalid syntax");
    let errors = result.unwrap_err();
    assert!(!errors.is_empty(), "Should have at least one error");
    
    let error = &errors[0];
    
    // Verify all required components
    assert!(error.span.line > 0, "Should have line number");
    assert!(error.span.column > 0, "Should have column number");
    assert!(!error.message.is_empty(), "Should have error message");
    assert!(error.span.start <= error.span.end, "Should have valid span");
    
    // Verify formatted output
    let formatted = error.format_with_source(invalid_source, "test.euis");
    assert!(formatted.contains("Error:"), "Should include error label");
    assert!(formatted.contains("test.euis"), "Should include file path");
    assert!(formatted.contains(&error.span.line.to_string()), "Should include line number");
}

/// Unit test: Verify error snippet extraction
#[test]
fn test_error_includes_code_snippet() {
    let invalid_source = ".button {\n  color: red\n  background: blue;\n}"; // Missing semicolon
    let result = parse(invalid_source);
    
    if let Err(errors) = result {
        assert!(!errors.is_empty(), "Should have errors");
        
        let error = &errors[0];
        let formatted = error.format_with_source(invalid_source, "test.euis");
        
        // Should include the line with the error
        let source_lines: Vec<&str> = invalid_source.lines().collect();
        if error.span.line > 0 && error.span.line <= source_lines.len() {
            let error_line = source_lines[error.span.line - 1];
            assert!(
                formatted.contains(error_line),
                "Formatted error should include the source line with the error"
            );
        }
    }
}

/// Unit test: Multiple errors on different lines
#[test]
fn test_multiple_errors_have_different_locations() {
    let invalid_source = ".test1 { color red; }\n.test2 { background blue; }"; // Two missing colons
    let result = parse(invalid_source);
    
    if let Err(errors) = result {
        if errors.len() >= 2 {
            // Errors should have different line numbers or positions
            let first_location = (errors[0].span.line, errors[0].span.column);
            let second_location = (errors[1].span.line, errors[1].span.column);
            
            assert_ne!(
                first_location, second_location,
                "Different errors should have different locations"
            );
        }
    }
}
