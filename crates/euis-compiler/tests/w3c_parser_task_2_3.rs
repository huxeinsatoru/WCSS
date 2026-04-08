// Test file for Task 2.3: Implement error reporting with location information
// Requirements: 1.6, 11.1, 11.2, 11.4, 11.6

use euis_compiler::w3c_parser::W3CTokenParser;
use euis_compiler::error::{ErrorKind, W3CErrorKind};

#[test]
fn test_invalid_json_has_location() {
    // Requirement 11.1: Invalid JSON returns error with line and column information
    let json = r##"{
        "color": {
            "primary": {
                "$value": "#3b82f6"
                "$type": "color"
            }
        }
    }"##;

    let result = W3CTokenParser::parse(json);
    assert!(result.is_err(), "Should error on invalid JSON");
    
    let errors = result.unwrap_err();
    assert_eq!(errors.len(), 1);
    
    let error = &errors[0];
    assert!(matches!(error.kind, ErrorKind::W3C(W3CErrorKind::InvalidJSON)));
    assert!(error.span.line > 0, "Error should have line number");
    assert!(error.span.column > 0, "Error should have column number");
    assert!(error.suggestion.is_some(), "Error should have suggestion");
}

#[test]
fn test_missing_value_field_has_location() {
    // Requirement 11.2: Missing $value field returns error with location
    let json = r##"{
        "color": {
            "primary": {
                "$type": "color"
            }
        }
    }"##;

    let result = W3CTokenParser::parse(json);
    assert!(result.is_err(), "Should error on missing $value");
    
    let errors = result.unwrap_err();
    assert_eq!(errors.len(), 1);
    
    let error = &errors[0];
    assert!(matches!(error.kind, ErrorKind::W3C(W3CErrorKind::MissingField)));
    assert!(error.message.contains("missing required $value field"));
    assert!(error.span.line >= 1, "Error should have line number");
    assert!(error.suggestion.is_some(), "Error should have suggestion");
}

#[test]
fn test_invalid_structure_has_location() {
    // Requirement 11.4: Invalid structure returns error with location
    let json = r##"{
        "color": {
            "primary": "not an object"
        }
    }"##;

    let result = W3CTokenParser::parse(json);
    assert!(result.is_err(), "Should error on invalid structure");
    
    let errors = result.unwrap_err();
    assert_eq!(errors.len(), 1);
    
    let error = &errors[0];
    assert!(matches!(error.kind, ErrorKind::W3C(W3CErrorKind::InvalidStructure)));
    assert!(error.message.contains("Invalid token structure"));
    assert!(error.span.line >= 1, "Error should have line number");
    assert!(error.suggestion.is_some(), "Error should have suggestion");
}

#[test]
fn test_mixed_token_group_has_location() {
    // Requirement 11.4: Mixed token/group object returns error with location
    let json = r##"{
        "color": {
            "primary": {
                "$value": "#3b82f6",
                "$type": "color",
                "nested": {
                    "$value": "#000000",
                    "$type": "color"
                }
            }
        }
    }"##;

    let result = W3CTokenParser::parse(json);
    assert!(result.is_err(), "Should error on mixed token/group");
    
    let errors = result.unwrap_err();
    assert!(errors.len() >= 1);
    
    let error = &errors[0];
    assert!(matches!(error.kind, ErrorKind::W3C(W3CErrorKind::InvalidStructure)));
    assert!(error.message.contains("both token properties") || error.message.contains("nested tokens"));
    assert!(error.span.line >= 1, "Error should have line number");
    assert!(error.suggestion.is_some(), "Error should have suggestion");
}

#[test]
fn test_non_object_root_has_location() {
    // Requirement 1.6: Non-conforming JSON returns descriptive error with location
    let json = r##"["array", "not", "object"]"##;

    let result = W3CTokenParser::parse(json);
    assert!(result.is_err(), "Should error when root is not an object");
    
    let errors = result.unwrap_err();
    assert_eq!(errors.len(), 1);
    
    let error = &errors[0];
    assert!(matches!(error.kind, ErrorKind::W3C(W3CErrorKind::InvalidStructure)));
    assert!(error.message.contains("must be a JSON object"));
    assert!(error.span.line >= 1, "Error should have line number");
    assert!(error.suggestion.is_some(), "Error should have suggestion");
}

#[test]
fn test_array_with_invalid_values_has_location() {
    // Test that array value errors include location information
    let json = r##"{
        "font": {
            "stack": {
                "$value": ["Inter", true, "Arial"],
                "$type": "fontFamily"
            }
        }
    }"##;

    let result = W3CTokenParser::parse(json);
    assert!(result.is_err(), "Should error on invalid array values");
    
    let errors = result.unwrap_err();
    assert!(errors.len() >= 1);
    
    let error = &errors[0];
    assert!(matches!(error.kind, ErrorKind::W3C(W3CErrorKind::InvalidStructure)));
    assert!(error.span.line >= 1, "Error should have line number");
}

#[test]
fn test_error_message_includes_path() {
    // Requirement 11.6: Error messages include token path for context
    let json = r##"{
        "theme": {
            "dark": {
                "color": {
                    "background": {
                        "$type": "color"
                    }
                }
            }
        }
    }"##;

    let result = W3CTokenParser::parse(json);
    assert!(result.is_err(), "Should error on missing $value");
    
    let errors = result.unwrap_err();
    let error = &errors[0];
    
    // Error message should include the full path
    assert!(
        error.message.contains("theme.dark.color.background"),
        "Error message should include full token path"
    );
}

#[test]
fn test_multiple_errors_collected() {
    // Requirement 11.5: Multiple validation errors reported in single pass
    let json = r##"{
        "color": {
            "primary": {
                "$type": "color"
            },
            "secondary": "not an object"
        }
    }"##;

    let result = W3CTokenParser::parse(json);
    assert!(result.is_err(), "Should error on multiple issues");
    
    let errors = result.unwrap_err();
    assert!(errors.len() >= 2, "Should collect multiple errors");
    
    // Verify each error has location information
    for error in &errors {
        assert!(error.span.line >= 1, "Each error should have line number");
    }
}

#[test]
fn test_error_suggestions_are_helpful() {
    // Requirement 11.6: Errors provide suggested fixes
    let json = r##"{
        "color": {
            "primary": {
                "$type": "color"
            }
        }
    }"##;

    let result = W3CTokenParser::parse(json);
    let errors = result.unwrap_err();
    let error = &errors[0];
    
    assert!(error.suggestion.is_some(), "Error should have suggestion");
    let suggestion = error.suggestion.as_ref().unwrap();
    assert!(
        suggestion.contains("$value") || suggestion.contains("value"),
        "Suggestion should mention the missing field"
    );
}

#[test]
fn test_deeply_nested_error_has_accurate_location() {
    // Test that errors in deeply nested structures have accurate location
    let json = r##"{
        "theme": {
            "light": {
                "color": {
                    "background": {
                        "primary": {
                            "$type": "color"
                        }
                    }
                }
            }
        }
    }"##;

    let result = W3CTokenParser::parse(json);
    assert!(result.is_err());
    
    let errors = result.unwrap_err();
    let error = &errors[0];
    
    // Should have valid location information
    assert!(error.span.line > 0);
    assert!(error.span.column > 0);
    assert!(error.message.contains("theme.light.color.background.primary"));
}

#[test]
fn test_json_syntax_error_location() {
    // Test that JSON syntax errors report accurate line/column
    let json = r##"{
        "color": {
            "primary": {
                "$value": "#3b82f6",
                "$type": "color",
            }
        }
    }"##; // Trailing comma before closing brace

    let result = W3CTokenParser::parse(json);
    assert!(result.is_err(), "Should error on JSON syntax error");
    
    let errors = result.unwrap_err();
    let error = &errors[0];
    
    assert!(matches!(error.kind, ErrorKind::W3C(W3CErrorKind::InvalidJSON)));
    assert!(error.span.line > 0, "Should have line number");
    assert!(error.span.column > 0, "Should have column number");
}

#[test]
fn test_error_span_has_valid_range() {
    // Test that error spans have valid start/end positions
    let json = r##"{
        "color": {
            "primary": {
                "$type": "color"
            }
        }
    }"##;

    let result = W3CTokenParser::parse(json);
    let errors = result.unwrap_err();
    let error = &errors[0];
    
    // Span should have valid range
    assert!(error.span.start <= error.span.end, "Span start should be <= end");
    assert!(error.span.line > 0, "Line should be positive");
    assert!(error.span.column > 0, "Column should be positive");
}

#[test]
fn test_format_error_with_source() {
    // Test that errors can be formatted with source code context
    let json = r##"{
        "color": {
            "primary": {
                "$type": "color"
            }
        }
    }"##;

    let result = W3CTokenParser::parse(json);
    let errors = result.unwrap_err();
    let error = &errors[0];
    
    // Test the format_with_source method
    let formatted = error.format_with_source(json, "tokens.json");
    
    assert!(formatted.contains("Error:"), "Should contain error label");
    assert!(formatted.contains("tokens.json"), "Should contain file path");
    assert!(formatted.contains(&error.span.line.to_string()), "Should contain line number");
}

#[test]
fn test_invalid_token_value_type_has_location() {
    // Test that invalid token value types report location
    let json = r##"{
        "value": {
            "test": {
                "$value": null,
                "$type": "color"
            }
        }
    }"##;

    let result = W3CTokenParser::parse(json);
    assert!(result.is_err(), "Should error on null value");
    
    let errors = result.unwrap_err();
    let error = &errors[0];
    
    assert!(matches!(error.kind, ErrorKind::W3C(W3CErrorKind::InvalidStructure)));
    assert!(error.span.line >= 1);
    assert!(error.suggestion.is_some());
}

#[test]
fn test_error_kind_is_w3c_specific() {
    // Test that W3C parser errors use W3C-specific error kinds
    let test_cases = vec![
        (r##"{ invalid }"##, W3CErrorKind::InvalidJSON),
        (r##"{"color": {"primary": {"$type": "color"}}}"##, W3CErrorKind::MissingField),
        (r##"{"color": {"primary": "string"}}"##, W3CErrorKind::InvalidStructure),
    ];

    for (json, expected_kind) in test_cases {
        let result = W3CTokenParser::parse(json);
        assert!(result.is_err());
        
        let errors = result.unwrap_err();
        let error = &errors[0];
        
        match &error.kind {
            ErrorKind::W3C(kind) => {
                assert_eq!(kind, &expected_kind, "Error kind should match expected");
            }
            _ => panic!("Expected W3C error kind"),
        }
    }
}

#[test]
fn test_invalid_type_value_error() {
    // Requirement 11.3: Invalid $type value returns error with list of valid types
    let json = r##"{
        "color": {
            "primary": {
                "$value": "#3b82f6",
                "$type": "invalidType"
            }
        }
    }"##;

    let result = W3CTokenParser::parse(json);
    assert!(result.is_err(), "Should error on invalid $type value");
    
    let errors = result.unwrap_err();
    assert_eq!(errors.len(), 1);
    
    let error = &errors[0];
    assert!(matches!(error.kind, ErrorKind::W3C(W3CErrorKind::InvalidType)));
    assert!(error.message.contains("Invalid $type value"));
    assert!(error.message.contains("invalidType"));
    assert!(error.message.contains("Valid types are:"));
    assert!(error.span.line > 0, "Error should have line number");
    assert!(error.suggestion.is_some(), "Error should have suggestion");
    
    let suggestion = error.suggestion.as_ref().unwrap();
    assert!(suggestion.contains("color"), "Suggestion should list valid types");
    assert!(suggestion.contains("dimension"), "Suggestion should list valid types");
}

#[test]
fn test_invalid_type_lists_all_valid_types() {
    // Requirement 11.3: Error message should list all valid W3C token types
    let json = r##"{
        "test": {
            "$value": "value",
            "$type": "wrongType"
        }
    }"##;

    let result = W3CTokenParser::parse(json);
    assert!(result.is_err());
    
    let errors = result.unwrap_err();
    let error = &errors[0];
    
    // Check that all valid types are mentioned in the error message
    let valid_types = vec![
        "color", "dimension", "fontFamily", "fontWeight",
        "duration", "cubicBezier", "number", "strokeStyle",
        "border", "shadow", "typography"
    ];
    
    for valid_type in valid_types {
        assert!(
            error.message.contains(valid_type) || error.suggestion.as_ref().unwrap().contains(valid_type),
            "Error should mention valid type: {}", valid_type
        );
    }
}

#[test]
fn test_valid_type_does_not_error() {
    // Ensure valid $type values don't trigger the error
    let json = r##"{
        "color": {
            "primary": {
                "$value": "#3b82f6",
                "$type": "color"
            }
        }
    }"##;

    let result = W3CTokenParser::parse(json);
    assert!(result.is_ok(), "Should not error on valid $type value");
    
    let tokens = result.unwrap();
    assert_eq!(tokens.len(), 1);
    assert!(tokens[0].token_type.is_some());
}
