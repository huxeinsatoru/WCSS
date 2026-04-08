// Test file for Task 2.1: Implement JSON parsing and token extraction
// Requirements: 1.1, 1.2, 1.3, 1.7

use euis_compiler::w3c_parser::{W3CTokenParser, W3CTokenValue, W3CTokenType};

#[test]
fn test_parse_json_string() {
    // Requirement 1.1: Parse valid W3C Design Tokens JSON file
    let json = r##"{
        "color": {
            "primary": {
                "$value": "#3b82f6",
                "$type": "color"
            }
        }
    }"##;

    let result = W3CTokenParser::parse(json);
    assert!(result.is_ok(), "Should parse valid JSON");
    let tokens = result.unwrap();
    assert_eq!(tokens.len(), 1);
}

#[test]
fn test_extract_value_field() {
    // Requirement 1.2: Extract $value field
    let json = r##"{
        "color": {
            "primary": {
                "$value": "#3b82f6",
                "$type": "color"
            }
        }
    }"##;

    let tokens = W3CTokenParser::parse(json).unwrap();
    match &tokens[0].value {
        W3CTokenValue::Literal(v) => assert_eq!(v, "#3b82f6"),
        _ => panic!("Expected literal value"),
    }
}

#[test]
fn test_extract_type_field() {
    // Requirement 1.3: Extract $type field
    let json = r##"{
        "color": {
            "primary": {
                "$value": "#3b82f6",
                "$type": "color"
            }
        }
    }"##;

    let tokens = W3CTokenParser::parse(json).unwrap();
    assert_eq!(tokens[0].token_type, Some(W3CTokenType::Color));
}

#[test]
fn test_extract_description_field() {
    // Requirement 1.7: Extract $description field
    let json = r##"{
        "color": {
            "primary": {
                "$value": "#3b82f6",
                "$type": "color",
                "$description": "Primary brand color"
            }
        }
    }"##;

    let tokens = W3CTokenParser::parse(json).unwrap();
    assert_eq!(tokens[0].description, Some("Primary brand color".to_string()));
}

#[test]
fn test_recursive_token_group_traversal() {
    // Test parse_token_group() recursively traverses nested groups
    let json = r##"{
        "color": {
            "primary": {
                "100": {
                    "$value": "#e0f2fe",
                    "$type": "color"
                },
                "500": {
                    "$value": "#3b82f6",
                    "$type": "color"
                },
                "900": {
                    "$value": "#1e3a8a",
                    "$type": "color"
                }
            }
        }
    }"##;

    let tokens = W3CTokenParser::parse(json).unwrap();
    assert_eq!(tokens.len(), 3, "Should extract all nested tokens");
    
    let paths: Vec<&str> = tokens.iter().map(|t| t.path.as_str()).collect();
    assert!(paths.contains(&"color.primary.100"));
    assert!(paths.contains(&"color.primary.500"));
    assert!(paths.contains(&"color.primary.900"));
}

#[test]
fn test_distinguish_token_from_group() {
    // Test that parser distinguishes between token objects ($value present) and token groups
    let json = r##"{
        "color": {
            "primary": {
                "$value": "#3b82f6",
                "$type": "color"
            },
            "secondary": {
                "light": {
                    "$value": "#f0f9ff",
                    "$type": "color"
                },
                "dark": {
                    "$value": "#0c4a6e",
                    "$type": "color"
                }
            }
        }
    }"##;

    let tokens = W3CTokenParser::parse(json).unwrap();
    assert_eq!(tokens.len(), 3, "Should extract 3 tokens");
    
    // color.primary is a token (has $value)
    let primary = tokens.iter().find(|t| t.path == "color.primary");
    assert!(primary.is_some(), "Should find color.primary token");
    
    // color.secondary is a group (no $value), but contains tokens
    let secondary_light = tokens.iter().find(|t| t.path == "color.secondary.light");
    let secondary_dark = tokens.iter().find(|t| t.path == "color.secondary.dark");
    assert!(secondary_light.is_some(), "Should find color.secondary.light token");
    assert!(secondary_dark.is_some(), "Should find color.secondary.dark token");
}

#[test]
fn test_token_with_type_but_no_value() {
    // Test that parser correctly identifies token by $type presence
    let json = r##"{
        "color": {
            "primary": {
                "$type": "color"
            }
        }
    }"##;

    let result = W3CTokenParser::parse(json);
    assert!(result.is_err(), "Should error on missing $value");
}

#[test]
fn test_parse_reference_value() {
    // Test that parser extracts reference values
    let json = r##"{
        "color": {
            "primary": {
                "$value": "#3b82f6",
                "$type": "color"
            },
            "secondary": {
                "$value": "{color.primary}",
                "$type": "color"
            }
        }
    }"##;

    let tokens = W3CTokenParser::parse(json).unwrap();
    let secondary = tokens.iter().find(|t| t.path == "color.secondary").unwrap();
    
    match &secondary.value {
        W3CTokenValue::Reference(r) => assert_eq!(r, "color.primary"),
        _ => panic!("Expected reference value"),
    }
}

#[test]
fn test_parse_composite_value() {
    // Test that parser extracts composite values (for typography, shadow, border)
    let json = r##"{
        "typography": {
            "heading": {
                "$value": {
                    "fontFamily": "Inter",
                    "fontSize": "24px",
                    "fontWeight": 700
                },
                "$type": "typography"
            }
        }
    }"##;

    let tokens = W3CTokenParser::parse(json).unwrap();
    match &tokens[0].value {
        W3CTokenValue::Composite(map) => {
            assert!(map.contains_key("fontFamily"));
            assert!(map.contains_key("fontSize"));
            assert!(map.contains_key("fontWeight"));
        }
        _ => panic!("Expected composite value"),
    }
}

#[test]
fn test_parse_array_value() {
    // Test that parser handles array values (e.g., font families)
    let json = r##"{
        "font": {
            "stack": {
                "$value": ["Inter", "Helvetica", "Arial", "sans-serif"],
                "$type": "fontFamily"
            }
        }
    }"##;

    let tokens = W3CTokenParser::parse(json).unwrap();
    match &tokens[0].value {
        W3CTokenValue::Literal(v) => {
            assert_eq!(v, "Inter, Helvetica, Arial, sans-serif");
        }
        _ => panic!("Expected literal value from array"),
    }
}

#[test]
fn test_parse_numeric_value() {
    // Test that parser handles numeric values
    let json = r##"{
        "opacity": {
            "half": {
                "$value": 0.5,
                "$type": "number"
            }
        }
    }"##;

    let tokens = W3CTokenParser::parse(json).unwrap();
    match &tokens[0].value {
        W3CTokenValue::Literal(v) => assert_eq!(v, "0.5"),
        _ => panic!("Expected literal value"),
    }
}

#[test]
fn test_skip_dollar_prefixed_keys_at_group_level() {
    // Test that parser skips $ keys at group level (like $extensions)
    let json = r##"{
        "$schema": "https://example.com/schema",
        "color": {
            "primary": {
                "$value": "#3b82f6",
                "$type": "color"
            }
        }
    }"##;

    let tokens = W3CTokenParser::parse(json).unwrap();
    assert_eq!(tokens.len(), 1, "Should skip $schema and only parse tokens");
}

#[test]
fn test_deeply_nested_groups() {
    // Test parser handles deeply nested token groups
    let json = r##"{
        "theme": {
            "light": {
                "color": {
                    "background": {
                        "primary": {
                            "$value": "#ffffff",
                            "$type": "color"
                        }
                    }
                }
            }
        }
    }"##;

    let tokens = W3CTokenParser::parse(json).unwrap();
    assert_eq!(tokens[0].path, "theme.light.color.background.primary");
}

#[test]
fn test_invalid_json_error() {
    // Test that parser returns error for invalid JSON
    let json = r##"{ invalid json }"##;
    let result = W3CTokenParser::parse(json);
    assert!(result.is_err(), "Should error on invalid JSON");
}

#[test]
fn test_non_object_root_error() {
    // Test that parser returns error when root is not an object
    let json = r##"["array", "not", "object"]"##;
    let result = W3CTokenParser::parse(json);
    assert!(result.is_err(), "Should error when root is not an object");
}

#[test]
fn test_invalid_token_structure_error() {
    // Test that parser returns error for invalid token structure (non-object value)
    let json = r##"{
        "color": {
            "primary": "not an object"
        }
    }"##;
    let result = W3CTokenParser::parse(json);
    assert!(result.is_err(), "Should error on invalid token structure");
}

#[test]
fn test_multiple_tokens_extraction() {
    // Test that parser extracts all tokens from a complex structure
    let json = r##"{
        "color": {
            "primary": {
                "$value": "#3b82f6",
                "$type": "color"
            },
            "secondary": {
                "$value": "#8b5cf6",
                "$type": "color"
            }
        },
        "spacing": {
            "small": {
                "$value": "8px",
                "$type": "dimension"
            },
            "medium": {
                "$value": "16px",
                "$type": "dimension"
            }
        }
    }"##;

    let tokens = W3CTokenParser::parse(json).unwrap();
    assert_eq!(tokens.len(), 4, "Should extract all 4 tokens");
}
