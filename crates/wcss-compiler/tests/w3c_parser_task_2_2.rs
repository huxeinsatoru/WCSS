// Test file for Task 2.2: Implement hierarchical path flattening
// Requirements: 1.4

use wcss_compiler::w3c_parser::{W3CTokenParser, W3CTokenType};

#[test]
fn test_flatten_single_level_path() {
    // Requirement 1.4: Flatten nested paths to dot notation
    let json = r##"{
        "color": {
            "$value": "#3b82f6",
            "$type": "color"
        }
    }"##;

    let tokens = W3CTokenParser::parse(json).unwrap();
    assert_eq!(tokens[0].path, "color");
}

#[test]
fn test_flatten_two_level_path() {
    // Requirement 1.4: Flatten nested paths to dot notation
    let json = r##"{
        "color": {
            "primary": {
                "$value": "#3b82f6",
                "$type": "color"
            }
        }
    }"##;

    let tokens = W3CTokenParser::parse(json).unwrap();
    assert_eq!(tokens[0].path, "color.primary");
}

#[test]
fn test_flatten_three_level_path() {
    // Requirement 1.4: Flatten nested paths to dot notation
    let json = r##"{
        "color": {
            "primary": {
                "500": {
                    "$value": "#3b82f6",
                    "$type": "color"
                }
            }
        }
    }"##;

    let tokens = W3CTokenParser::parse(json).unwrap();
    assert_eq!(tokens[0].path, "color.primary.500");
}

#[test]
fn test_flatten_deeply_nested_path() {
    // Requirement 1.4: Flatten deeply nested structures
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
fn test_flatten_multiple_paths_same_group() {
    // Requirement 1.4: Track current path during recursive traversal
    let json = r##"{
        "color": {
            "primary": {
                "$value": "#3b82f6",
                "$type": "color"
            },
            "secondary": {
                "$value": "#8b5cf6",
                "$type": "color"
            },
            "tertiary": {
                "$value": "#ec4899",
                "$type": "color"
            }
        }
    }"##;

    let tokens = W3CTokenParser::parse(json).unwrap();
    assert_eq!(tokens.len(), 3);
    
    let paths: Vec<&str> = tokens.iter().map(|t| t.path.as_str()).collect();
    assert!(paths.contains(&"color.primary"));
    assert!(paths.contains(&"color.secondary"));
    assert!(paths.contains(&"color.tertiary"));
}

#[test]
fn test_flatten_mixed_depth_paths() {
    // Requirement 1.4: Handle mixed nesting depths correctly
    let json = r##"{
        "color": {
            "brand": {
                "$value": "#3b82f6",
                "$type": "color"
            },
            "primary": {
                "500": {
                    "$value": "#3b82f6",
                    "$type": "color"
                },
                "600": {
                    "$value": "#2563eb",
                    "$type": "color"
                }
            }
        }
    }"##;

    let tokens = W3CTokenParser::parse(json).unwrap();
    assert_eq!(tokens.len(), 3);
    
    let paths: Vec<&str> = tokens.iter().map(|t| t.path.as_str()).collect();
    assert!(paths.contains(&"color.brand"));
    assert!(paths.contains(&"color.primary.500"));
    assert!(paths.contains(&"color.primary.600"));
}

#[test]
fn test_flatten_numeric_path_segments() {
    // Requirement 1.4: Handle numeric path segments (like "500", "600")
    let json = r##"{
        "color": {
            "blue": {
                "100": {
                    "$value": "#dbeafe",
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
    assert_eq!(tokens.len(), 3);
    
    let paths: Vec<&str> = tokens.iter().map(|t| t.path.as_str()).collect();
    assert!(paths.contains(&"color.blue.100"));
    assert!(paths.contains(&"color.blue.500"));
    assert!(paths.contains(&"color.blue.900"));
}

#[test]
fn test_flatten_special_characters_in_path() {
    // Requirement 1.4: Handle special characters in path segments
    let json = r##"{
        "color": {
            "primary-brand": {
                "$value": "#3b82f6",
                "$type": "color"
            },
            "secondary_accent": {
                "$value": "#8b5cf6",
                "$type": "color"
            }
        }
    }"##;

    let tokens = W3CTokenParser::parse(json).unwrap();
    assert_eq!(tokens.len(), 2);
    
    let paths: Vec<&str> = tokens.iter().map(|t| t.path.as_str()).collect();
    assert!(paths.contains(&"color.primary-brand"));
    assert!(paths.contains(&"color.secondary_accent"));
}

#[test]
fn test_flatten_path_with_multiple_token_types() {
    // Requirement 1.4: Flatten paths for different token types
    let json = r##"{
        "color": {
            "primary": {
                "$value": "#3b82f6",
                "$type": "color"
            }
        },
        "spacing": {
            "small": {
                "$value": "8px",
                "$type": "dimension"
            }
        },
        "typography": {
            "heading": {
                "$value": {
                    "fontFamily": "Inter",
                    "fontSize": "24px"
                },
                "$type": "typography"
            }
        }
    }"##;

    let tokens = W3CTokenParser::parse(json).unwrap();
    assert_eq!(tokens.len(), 3);
    
    let color_token = tokens.iter().find(|t| t.path == "color.primary").unwrap();
    assert_eq!(color_token.token_type, Some(W3CTokenType::Color));
    
    let spacing_token = tokens.iter().find(|t| t.path == "spacing.small").unwrap();
    assert_eq!(spacing_token.token_type, Some(W3CTokenType::Dimension));
    
    let typography_token = tokens.iter().find(|t| t.path == "typography.heading").unwrap();
    assert_eq!(typography_token.token_type, Some(W3CTokenType::Typography));
}

#[test]
fn test_flatten_sibling_groups() {
    // Requirement 1.4: Track current path correctly when traversing sibling groups
    let json = r##"{
        "light": {
            "color": {
                "background": {
                    "$value": "#ffffff",
                    "$type": "color"
                }
            }
        },
        "dark": {
            "color": {
                "background": {
                    "$value": "#000000",
                    "$type": "color"
                }
            }
        }
    }"##;

    let tokens = W3CTokenParser::parse(json).unwrap();
    assert_eq!(tokens.len(), 2);
    
    let paths: Vec<&str> = tokens.iter().map(|t| t.path.as_str()).collect();
    assert!(paths.contains(&"light.color.background"));
    assert!(paths.contains(&"dark.color.background"));
}

#[test]
fn test_flatten_path_order_preservation() {
    // Requirement 1.4: Preserve order of path segments
    let json = r##"{
        "a": {
            "b": {
                "c": {
                    "d": {
                        "$value": "test",
                        "$type": "color"
                    }
                }
            }
        }
    }"##;

    let tokens = W3CTokenParser::parse(json).unwrap();
    assert_eq!(tokens[0].path, "a.b.c.d");
}

#[test]
fn test_flatten_empty_path_segments() {
    // Requirement 1.4: Handle edge case of root-level token (empty path)
    // Note: This is technically invalid W3C format, but tests robustness
    let json = r##"{
        "token": {
            "$value": "#3b82f6",
            "$type": "color"
        }
    }"##;

    let tokens = W3CTokenParser::parse(json).unwrap();
    assert_eq!(tokens[0].path, "token");
}

#[test]
fn test_flatten_complex_nested_structure() {
    // Requirement 1.4: Test complex real-world structure
    let json = r##"{
        "theme": {
            "light": {
                "color": {
                    "background": {
                        "primary": {
                            "$value": "#ffffff",
                            "$type": "color"
                        },
                        "secondary": {
                            "$value": "#f3f4f6",
                            "$type": "color"
                        }
                    },
                    "text": {
                        "primary": {
                            "$value": "#111827",
                            "$type": "color"
                        }
                    }
                }
            },
            "dark": {
                "color": {
                    "background": {
                        "primary": {
                            "$value": "#111827",
                            "$type": "color"
                        }
                    }
                }
            }
        }
    }"##;

    let tokens = W3CTokenParser::parse(json).unwrap();
    assert_eq!(tokens.len(), 4);
    
    let paths: Vec<&str> = tokens.iter().map(|t| t.path.as_str()).collect();
    assert!(paths.contains(&"theme.light.color.background.primary"));
    assert!(paths.contains(&"theme.light.color.background.secondary"));
    assert!(paths.contains(&"theme.light.color.text.primary"));
    assert!(paths.contains(&"theme.dark.color.background.primary"));
}

#[test]
fn test_flatten_path_with_references() {
    // Requirement 1.4: Flatten paths for tokens with references
    let json = r##"{
        "color": {
            "base": {
                "blue": {
                    "$value": "#3b82f6",
                    "$type": "color"
                }
            },
            "semantic": {
                "primary": {
                    "$value": "{color.base.blue}",
                    "$type": "color"
                }
            }
        }
    }"##;

    let tokens = W3CTokenParser::parse(json).unwrap();
    assert_eq!(tokens.len(), 2);
    
    let paths: Vec<&str> = tokens.iter().map(|t| t.path.as_str()).collect();
    assert!(paths.contains(&"color.base.blue"));
    assert!(paths.contains(&"color.semantic.primary"));
}

#[test]
fn test_flatten_path_consistency_across_types() {
    // Requirement 1.4: Ensure path flattening is consistent across all token types
    let json = r##"{
        "design": {
            "color": {
                "primary": {
                    "$value": "#3b82f6",
                    "$type": "color"
                }
            },
            "spacing": {
                "base": {
                    "$value": "8px",
                    "$type": "dimension"
                }
            },
            "font": {
                "family": {
                    "$value": "Inter",
                    "$type": "fontFamily"
                }
            }
        }
    }"##;

    let tokens = W3CTokenParser::parse(json).unwrap();
    assert_eq!(tokens.len(), 3);
    
    // All paths should start with "design" and have consistent dot notation
    for token in &tokens {
        assert!(token.path.starts_with("design."));
        assert_eq!(token.path.matches('.').count(), 2); // Two dots for 3-level path
    }
}

#[test]
fn test_flatten_path_with_unicode_characters() {
    // Requirement 1.4: Handle Unicode characters in path segments
    let json = r##"{
        "color": {
            "主要": {
                "$value": "#3b82f6",
                "$type": "color"
            },
            "couleur-primaire": {
                "$value": "#8b5cf6",
                "$type": "color"
            }
        }
    }"##;

    let tokens = W3CTokenParser::parse(json).unwrap();
    assert_eq!(tokens.len(), 2);
    
    let paths: Vec<&str> = tokens.iter().map(|t| t.path.as_str()).collect();
    assert!(paths.contains(&"color.主要"));
    assert!(paths.contains(&"color.couleur-primaire"));
}

#[test]
fn test_flatten_path_large_structure() {
    // Requirement 1.4: Test path flattening with many tokens
    let json = r##"{
        "color": {
            "red": {
                "100": { "$value": "#fee2e2", "$type": "color" },
                "200": { "$value": "#fecaca", "$type": "color" },
                "300": { "$value": "#fca5a5", "$type": "color" },
                "400": { "$value": "#f87171", "$type": "color" },
                "500": { "$value": "#ef4444", "$type": "color" }
            },
            "blue": {
                "100": { "$value": "#dbeafe", "$type": "color" },
                "200": { "$value": "#bfdbfe", "$type": "color" },
                "300": { "$value": "#93c5fd", "$type": "color" },
                "400": { "$value": "#60a5fa", "$type": "color" },
                "500": { "$value": "#3b82f6", "$type": "color" }
            }
        }
    }"##;

    let tokens = W3CTokenParser::parse(json).unwrap();
    assert_eq!(tokens.len(), 10);
    
    // Verify all paths are correctly flattened
    for token in &tokens {
        assert!(token.path.starts_with("color."));
        assert!(token.path.contains(".100") || 
                token.path.contains(".200") || 
                token.path.contains(".300") || 
                token.path.contains(".400") || 
                token.path.contains(".500"));
    }
}
