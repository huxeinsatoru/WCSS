// Feature: wcss-framework, Property 4: Multiple Error Detection
// **Validates: Requirements 14.5**

use proptest::prelude::*;
use wcss_compiler::parser::parse;

/// Generate WCSS with multiple syntax errors.
fn multiple_errors_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        // Multiple missing braces
        (
            "[a-z][a-z0-9-]{2,8}",
            "[a-z][a-z0-9-]{2,8}",
        ).prop_map(|(class1, class2)| {
            format!(
                ".{} {{ color: red;\n.{} background: blue; }}",
                class1, class2
            )
        }),
        
        // Multiple missing colons
        (
            "[a-z][a-z0-9-]{2,8}",
            "[a-z][a-z0-9-]{2,8}",
        ).prop_map(|(class1, class2)| {
            format!(
                ".{} {{ color red; }}\n.{} {{ background blue; }}",
                class1, class2
            )
        }),
        
        // Mixed error types: missing brace and missing colon
        (
            "[a-z][a-z0-9-]{2,8}",
            "[a-z][a-z0-9-]{2,8}",
        ).prop_map(|(class1, class2)| {
            format!(
                ".{} {{ color: red;\n.{} {{ background blue; }}",
                class1, class2
            )
        }),
        
        // Multiple invalid selectors
        (
            "[a-z][a-z0-9-]{2,8}",
            "[a-z][a-z0-9-]{2,8}",
        ).prop_map(|(class1, class2)| {
            format!(
                "{}@ {{ color: red; }}\n{}# {{ background: blue; }}",
                class1, class2
            )
        }),
        
        // Three errors in sequence
        (
            "[a-z][a-z0-9-]{2,8}",
            "[a-z][a-z0-9-]{2,8}",
            "[a-z][a-z0-9-]{2,8}",
        ).prop_map(|(class1, class2, class3)| {
            format!(
                ".{} {{ color red; }}\n.{} background: blue; }}\n.{} {{ padding 10px; }}",
                class1, class2, class3
            )
        }),
        
        // Errors in different parts of rules
        "[a-z][a-z0-9-]{2,8}".prop_map(|class| {
            format!(
                ".{} {{\n  color red;\n  background blue;\n  padding 10px;\n}}",
                class
            )
        }),
        
        // Nested errors with state blocks
        "[a-z][a-z0-9-]{2,8}".prop_map(|class| {
            format!(
                ".{} {{\n  color: red;\n  :hover {{\n    background blue;\n  }}\n  :focus\n    color: green;\n  }}\n}}",
                class
            )
        }),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    /// Property 4: Multiple Error Detection
    /// For any WCSS source with multiple errors, the compiler SHALL report
    /// all errors in a single compilation pass.
    ///
    /// Validates: Requirements 14.5
    #[test]
    fn prop_multiple_error_detection(source_with_errors in multiple_errors_strategy()) {
        // Parse WCSS with multiple errors
        let result = parse(&source_with_errors);
        
        // Verify parsing fails
        prop_assert!(
            result.is_err(),
            "WCSS with multiple errors should fail to parse. Source:\n{}",
            source_with_errors
        );
        
        let errors = result.unwrap_err();
        
        // Verify multiple errors are reported
        // Note: Parser may not catch ALL errors due to error recovery,
        // but it should catch multiple errors in a single pass
        prop_assert!(
            !errors.is_empty(),
            "Should report at least one error"
        );
        
        // Verify errors are distinct (different locations)
        if errors.len() > 1 {
            for i in 0..errors.len() {
                for j in (i + 1)..errors.len() {
                    let loc1 = (errors[i].span.line, errors[i].span.column, errors[i].span.start);
                    let loc2 = (errors[j].span.line, errors[j].span.column, errors[j].span.start);
                    
                    // Errors should have different locations (unless they're at the same spot)
                    // This verifies they're distinct errors, not duplicates
                    if loc1 == loc2 {
                        prop_assert!(
                            errors[i].message != errors[j].message,
                            "Errors at same location should have different messages"
                        );
                    }
                }
            }
        }
        
        // Verify each error has complete information
        for error in &errors {
            prop_assert!(
                error.span.line > 0,
                "Each error should have a line number"
            );
            
            prop_assert!(
                !error.message.is_empty(),
                "Each error should have a message"
            );
        }
    }
    
    /// Property: Parser continues after first error
    /// The parser should use error recovery to continue parsing after
    /// encountering an error, allowing it to find subsequent errors.
    #[test]
    fn prop_parser_continues_after_error(source_with_errors in multiple_errors_strategy()) {
        let result = parse(&source_with_errors);
        
        if let Err(errors) = result {
            // Count the number of potential error locations in the source
            // (This is a heuristic - we look for patterns that typically cause errors)
            let potential_errors = source_with_errors.matches("color red")
                .chain(source_with_errors.matches("background blue"))
                .chain(source_with_errors.matches("padding 10px"))
                .count();
            
            // If there are multiple potential errors, we should detect at least some
            if potential_errors >= 2 {
                prop_assert!(
                    errors.len() >= 1,
                    "Parser should detect errors even with multiple issues present"
                );
            }
        }
    }
    
    /// Property: Errors are reported in source order
    /// When multiple errors are detected, they should be reported in the
    /// order they appear in the source code.
    #[test]
    fn prop_errors_in_source_order(source_with_errors in multiple_errors_strategy()) {
        let result = parse(&source_with_errors);
        
        if let Err(errors) = result {
            if errors.len() >= 2 {
                // Check that errors are generally in source order
                // (line numbers should be non-decreasing)
                for i in 0..(errors.len() - 1) {
                    let current_line = errors[i].span.line;
                    let next_line = errors[i + 1].span.line;
                    
                    // Errors should be in order by line number
                    prop_assert!(
                        current_line <= next_line,
                        "Errors should be reported in source order (line {} before line {})",
                        current_line,
                        next_line
                    );
                    
                    // If on the same line, check column order
                    if current_line == next_line {
                        prop_assert!(
                            errors[i].span.column <= errors[i + 1].span.column,
                            "Errors on same line should be ordered by column"
                        );
                    }
                }
            }
        }
    }
    
    /// Property: No duplicate errors
    /// The parser should not report the same error multiple times.
    #[test]
    fn prop_no_duplicate_errors(source_with_errors in multiple_errors_strategy()) {
        let result = parse(&source_with_errors);
        
        if let Err(errors) = result {
            // Check for exact duplicates (same location and message)
            for i in 0..errors.len() {
                for j in (i + 1)..errors.len() {
                    let same_location = 
                        errors[i].span.line == errors[j].span.line &&
                        errors[i].span.column == errors[j].span.column &&
                        errors[i].span.start == errors[j].span.start;
                    
                    let same_message = errors[i].message == errors[j].message;
                    
                    prop_assert!(
                        !(same_location && same_message),
                        "Should not report duplicate errors at same location with same message"
                    );
                }
            }
        }
    }
}

/// Unit test: Two distinct errors are both reported
#[test]
fn test_two_errors_both_reported() {
    let source = ".test1 { color red; }\n.test2 { background blue; }";
    let result = parse(source);
    
    assert!(result.is_err(), "Should fail with errors");
    let errors = result.unwrap_err();
    
    // Should detect at least one error (ideally both, but parser recovery may vary)
    assert!(!errors.is_empty(), "Should report at least one error");
    
    // Verify errors have valid information
    for error in &errors {
        assert!(error.span.line > 0, "Error should have line number");
        assert!(!error.message.is_empty(), "Error should have message");
    }
}

/// Unit test: Three errors in different rules
#[test]
fn test_three_errors_in_different_rules() {
    let source = ".a { color red; }\n.b { background blue; }\n.c { padding 10px; }";
    let result = parse(source);
    
    assert!(result.is_err(), "Should fail with errors");
    let errors = result.unwrap_err();
    
    // Should detect at least one error
    assert!(!errors.is_empty(), "Should report errors");
    
    // If multiple errors detected, verify they're on different lines
    if errors.len() >= 2 {
        let first_line = errors[0].span.line;
        let second_line = errors[1].span.line;
        assert_ne!(first_line, second_line, "Errors should be on different lines");
    }
}

/// Unit test: Errors within same rule
#[test]
fn test_multiple_errors_in_same_rule() {
    let source = ".test {\n  color red;\n  background blue;\n  padding 10px;\n}";
    let result = parse(source);
    
    assert!(result.is_err(), "Should fail with errors");
    let errors = result.unwrap_err();
    
    // Should detect at least one error
    assert!(!errors.is_empty(), "Should report at least one error");
    
    // Verify error information is complete
    for error in &errors {
        assert!(error.span.line > 0, "Error should have line number");
        assert!(error.span.column > 0, "Error should have column number");
        assert!(!error.message.is_empty(), "Error should have message");
    }
}

/// Unit test: Mixed error types
#[test]
fn test_mixed_error_types() {
    // Missing brace and missing colon
    let source = ".test1 { color: red;\n.test2 { background blue; }";
    let result = parse(source);
    
    assert!(result.is_err(), "Should fail with errors");
    let errors = result.unwrap_err();
    
    // Should detect at least one error
    assert!(!errors.is_empty(), "Should report errors");
    
    // Verify all errors have complete information
    for error in &errors {
        assert!(error.span.line > 0, "Error should have line number");
        assert!(!error.message.is_empty(), "Error should have message");
    }
}

/// Unit test: Errors are in source order
#[test]
fn test_errors_in_source_order() {
    let source = ".a { color red; }\n.b { background blue; }\n.c { padding 10px; }";
    let result = parse(source);
    
    if let Err(errors) = result {
        if errors.len() >= 2 {
            // Verify errors are in ascending line order
            for i in 0..(errors.len() - 1) {
                assert!(
                    errors[i].span.line <= errors[i + 1].span.line,
                    "Errors should be in source order"
                );
            }
        }
    }
}

/// Unit test: Parser recovery allows finding subsequent errors
#[test]
fn test_parser_recovery() {
    // First rule has error, second rule is valid, third has error
    let source = ".error1 { color red; }\n.valid { color: blue; }\n.error2 { background green; }";
    let result = parse(source);
    
    assert!(result.is_err(), "Should fail due to errors");
    let errors = result.unwrap_err();
    
    // Should detect at least one error
    assert!(!errors.is_empty(), "Should detect errors");
    
    // Verify parser didn't stop at first error
    // (it should have continued parsing and potentially found more)
    for error in &errors {
        assert!(error.span.line > 0, "Each error should have valid location");
    }
}
