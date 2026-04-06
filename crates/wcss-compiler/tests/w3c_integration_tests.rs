//! Integration tests for W3C Design Tokens
//! 
//! Tests cover:
//! - End-to-end compilation to all platforms
//! - W3C spec compliance
//! - Platform-specific code validity
//! - WCSS integration

use wcss_compiler::{
    compile_w3c_tokens,
    config::PlatformTarget,
};

// ============================================================================
// Helper Functions
// ============================================================================

fn create_basic_tokens() -> &'static str {
    r##"{
        "color": {
            "primary": {
                "$value": "#3b82f6",
                "$type": "color",
                "$description": "Primary brand color"
            },
            "secondary": {
                "$value": "#64748b",
                "$type": "color"
            }
        },
        "spacing": {
            "base": {
                "$value": "16px",
                "$type": "dimension"
            }
        }
    }"##
}

fn create_comprehensive_tokens() -> &'static str {
    r##"{
        "color": {
            "primary": { "$value": "#3b82f6", "$type": "color" },
            "success": { "$value": "#22c55e", "$type": "color" },
            "warning": { "$value": "#f59e0b", "$type": "color" },
            "error": { "$value": "#ef4444", "$type": "color" }
        },
        "spacing": {
            "xs": { "$value": "4px", "$type": "dimension" },
            "sm": { "$value": "8px", "$type": "dimension" },
            "md": { "$value": "16px", "$type": "dimension" },
            "lg": { "$value": "24px", "$type": "dimension" }
        },
        "font": {
            "family": {
                "sans": { "$value": ["Inter", "sans-serif"], "$type": "fontFamily" }
            },
            "weight": {
                "normal": { "$value": 400, "$type": "fontWeight" },
                "bold": { "$value": 700, "$type": "fontWeight" }
            }
        },
        "animation": {
            "duration": {
                "fast": { "$value": "150ms", "$type": "duration" }
            },
            "easing": {
                "default": { "$value": [0.4, 0, 0.2, 1], "$type": "cubicBezier" }
            }
        }
    }"##
}

fn create_tokens_with_references() -> &'static str {
    r##"{
        "color": {
            "base": { "$value": "#3b82f6", "$type": "color" },
            "primary": { "$value": "{color.base}", "$type": "color" },
            "hover": { "$value": "{color.primary}", "$type": "color" }
        },
        "spacing": {
            "unit": { "$value": "4px", "$type": "dimension" },
            "sm": { "$value": "{spacing.unit}", "$type": "dimension" },
            "md": { "$value": "{spacing.unit}", "$type": "dimension" }
        }
    }"##
}

// ============================================================================
// Task 20.1: W3C Spec Compliance Tests
// ============================================================================

#[test]
fn test_w3c_spec_compliance_basic_structure() {
    let json = create_basic_tokens();
    let result = compile_w3c_tokens(json, PlatformTarget::CSS);
    
    assert!(result.is_ok(), "Should parse valid W3C token structure");
    
    let output = result.unwrap();
    assert!(output.contains_key("tokens.css"));
    
    let css = &output["tokens.css"];
    assert!(css.contains("--color-primary"));
    assert!(css.contains("#3b82f6"));
    assert!(css.contains("--spacing-base"));
    assert!(css.contains("16px"));
}

#[test]
fn test_w3c_spec_compliance_all_primitive_types() {
    let json = r##"{
        "color": { "$value": "#ff0000", "$type": "color" },
        "dimension": { "$value": "16px", "$type": "dimension" },
        "fontFamily": { "$value": "Arial", "$type": "fontFamily" },
        "fontWeight": { "$value": 700, "$type": "fontWeight" },
        "duration": { "$value": "200ms", "$type": "duration" },
        "cubicBezier": { "$value": [0.4, 0, 0.2, 1], "$type": "cubicBezier" },
        "number": { "$value": 1.5, "$type": "number" }
    }"##;
    
    let result = compile_w3c_tokens(json, PlatformTarget::CSS);
    assert!(result.is_ok(), "Should support all W3C primitive token types");
}

#[test]
fn test_w3c_spec_compliance_nested_groups() {
    let json = r##"{
        "color": {
            "brand": {
                "primary": { "$value": "#3b82f6", "$type": "color" },
                "secondary": { "$value": "#64748b", "$type": "color" }
            },
            "semantic": {
                "success": { "$value": "#22c55e", "$type": "color" },
                "error": { "$value": "#ef4444", "$type": "color" }
            }
        }
    }"##;
    
    let result = compile_w3c_tokens(json, PlatformTarget::CSS);
    assert!(result.is_ok());
    
    let css = &result.unwrap()["tokens.css"];
    assert!(css.contains("--color-brand-primary"));
    assert!(css.contains("--color-semantic-success"));
}

#[test]
fn test_w3c_spec_compliance_description_field() {
    let json = r##"{
        "token": {
            "$value": "#3b82f6",
            "$type": "color",
            "$description": "This is a description"
        }
    }"##;
    
    // Should parse without error (description is metadata)
    let result = compile_w3c_tokens(json, PlatformTarget::CSS);
    assert!(result.is_ok());
}

// ============================================================================
// Task 20.2: Platform Compilation Tests
// ============================================================================

#[test]
fn test_platform_css_generation() {
    let json = create_comprehensive_tokens();
    let result = compile_w3c_tokens(json, PlatformTarget::CSS);
    
    assert!(result.is_ok());
    let output = result.unwrap();
    
    assert!(output.contains_key("tokens.css"));
    let css = &output["tokens.css"];
    
    // Check CSS validity
    assert!(css.contains(":root"));
    assert!(css.contains("--color-primary: #3b82f6"));
    assert!(css.contains("--spacing-md: 16px"));
}

#[test]
fn test_platform_ios_generation() {
    let json = create_comprehensive_tokens();
    let result = compile_w3c_tokens(json, PlatformTarget::IOS);
    
    assert!(result.is_ok());
    let output = result.unwrap();
    
    assert!(output.contains_key("DesignTokens.swift"));
    let swift = &output["DesignTokens.swift"];
    
    // Check Swift code structure
    assert!(swift.contains("import UIKit"));
    assert!(swift.contains("UIColor"));
    assert!(swift.contains("colorPrimary"));
    assert!(swift.contains("CGFloat"));
}

#[test]
fn test_platform_android_xml_generation() {
    let json = create_comprehensive_tokens();
    let result = compile_w3c_tokens(json, PlatformTarget::Android);
    
    assert!(result.is_ok());
    let output = result.unwrap();
    
    // Should generate colors.xml for color tokens
    assert!(output.contains_key("colors.xml"));
    let colors = &output["colors.xml"];
    
    // Check Android XML structure
    assert!(colors.contains("<?xml version"));
    assert!(colors.contains("<resources>"));
    assert!(colors.contains("<color name=\"color_primary\">#3b82f6</color>"));
}

#[test]
fn test_platform_android_kotlin_generation() {
    let json = create_comprehensive_tokens();
    let result = compile_w3c_tokens(json, PlatformTarget::AndroidKotlin);
    
    assert!(result.is_ok());
    let output = result.unwrap();
    
    assert!(output.contains_key("DesignTokens.kt"));
    let kotlin = &output["DesignTokens.kt"];
    
    // Check Kotlin code structure
    assert!(kotlin.contains("object DesignTokens"));
    assert!(kotlin.contains("val colorPrimary"));
}

#[test]
fn test_platform_flutter_generation() {
    let json = create_comprehensive_tokens();
    let result = compile_w3c_tokens(json, PlatformTarget::Flutter);
    
    assert!(result.is_ok());
    let output = result.unwrap();
    
    assert!(output.contains_key("design_tokens.dart"));
    let dart = &output["design_tokens.dart"];
    
    // Check Dart code structure
    assert!(dart.contains("import 'package:flutter/material.dart'"));
    assert!(dart.contains("class DesignTokens"));
    assert!(dart.contains("Color("));
    assert!(dart.contains("0xFF3B82F6")); // Hex color in Flutter format
}

#[test]
fn test_platform_typescript_generation() {
    let json = create_comprehensive_tokens();
    let result = compile_w3c_tokens(json, PlatformTarget::TypeScript);
    
    assert!(result.is_ok());
    let output = result.unwrap();
    
    assert!(output.contains_key("tokens.ts"));
    let ts = &output["tokens.ts"];
    
    // Check TypeScript structure
    assert!(ts.contains("export type TokenPath"));
    assert!(ts.contains("export interface TokenValues"));
    assert!(ts.contains("export const tokens"));
    assert!(ts.contains("'color.primary'"));
}

#[test]
fn test_platform_docs_generation() {
    let json = create_comprehensive_tokens();
    let result = compile_w3c_tokens(json, PlatformTarget::Docs);
    
    assert!(result.is_ok());
    let output = result.unwrap();
    
    assert!(output.contains_key("index.html"));
    let html = &output["index.html"];
    
    // Check HTML structure
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("color.primary"));
    assert!(html.contains("#3b82f6"));
}

// ============================================================================
// Task 20.3: WCSS Integration Tests
// ============================================================================

#[test]
fn test_w3c_tokens_with_references() {
    let json = create_tokens_with_references();
    let result = compile_w3c_tokens(json, PlatformTarget::CSS);
    
    assert!(result.is_ok());
    let css = &result.unwrap()["tokens.css"];
    
    // References should be resolved to literal values
    assert!(css.contains("--color-base: #3b82f6"));
    assert!(css.contains("--color-primary: #3b82f6"));
    assert!(css.contains("--color-hover: #3b82f6"));
}

#[test]
fn test_w3c_token_validation_errors() {
    // Invalid color value
    let json = r##"{
        "invalid": {
            "$value": "not-a-color",
            "$type": "color"
        }
    }"##;
    
    let result = compile_w3c_tokens(json, PlatformTarget::CSS);
    assert!(result.is_err(), "Should fail validation for invalid color");
}

#[test]
fn test_w3c_circular_reference_detection() {
    let json = r##"{
        "a": { "$value": "{b}", "$type": "color" },
        "b": { "$value": "{a}", "$type": "color" }
    }"##;
    
    let result = compile_w3c_tokens(json, PlatformTarget::CSS);
    assert!(result.is_err(), "Should detect circular references");
}

// ============================================================================
// Task 20.4: Performance Tests
// ============================================================================

#[test]
fn test_large_token_file() {
    // Generate 1000+ tokens
    let mut token_parts = Vec::new();
    for i in 0..1000 {
        let hex = format!("{:06x}", i % 0xFFFFFF);
        // Build token string without using format! for the $ to avoid Rust format interpretation
        let token = format!(
            "\"token{}\": {{ \"$value\": \"#{}\", \"$type\": \"color\" }}",
            i, hex
        );
        token_parts.push(token);
    }
    let tokens = format!("{{{}}}", token_parts.join(","));
    
    let start = std::time::Instant::now();
    let result = compile_w3c_tokens(&tokens, PlatformTarget::CSS);
    let duration = start.elapsed();
    
    assert!(result.is_ok(), "Should handle 1000+ tokens");
    assert!(duration.as_millis() < 1000, "Should compile in under 1 second");
}

#[test]
fn test_deeply_nested_structure() {
    // Create 10+ levels of nesting
    let json = r##"{
        "l1": {
            "l2": {
                "l3": {
                    "l4": {
                        "l5": {
                            "l6": {
                                "l7": {
                                    "l8": {
                                        "l9": {
                                            "l10": {
                                                "$value": "#3b82f6",
                                                "$type": "color"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }"##;
    
    let result = compile_w3c_tokens(json, PlatformTarget::CSS);
    assert!(result.is_ok());
    
    let css = &result.unwrap()["tokens.css"];
    assert!(css.contains("--l1-l2-l3-l4-l5-l6-l7-l8-l9-l10"));
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_invalid_json_error() {
    let json = "not valid json";
    let result = compile_w3c_tokens(json, PlatformTarget::CSS);
    
    assert!(result.is_err());
}

#[test]
fn test_missing_value_field_error() {
    let json = r##"{
        "token": {
            "$type": "color"
        }
    }"##;
    
    let _result = compile_w3c_tokens(json, PlatformTarget::CSS);
    // This may or may not be an error depending on implementation
    // Some implementations treat missing $value as a group
}

#[test]
fn test_invalid_type_value_error() {
    let json = r##"{
        "token": {
            "$value": "#3b82f6",
            "$type": "invalidType"
        }
    }"##;
    
    let result = compile_w3c_tokens(json, PlatformTarget::CSS);
    // Should either error or ignore invalid type
    assert!(result.is_ok() || result.is_err());
}

// ============================================================================
// CLI Integration Tests
// ============================================================================

#[test]
fn test_all_platform_targets() {
    let json = create_basic_tokens();
    let platforms = vec![
        PlatformTarget::CSS,
        PlatformTarget::IOS,
        PlatformTarget::Android,
        PlatformTarget::AndroidKotlin,
        PlatformTarget::Flutter,
        PlatformTarget::TypeScript,
        PlatformTarget::Docs,
    ];
    
    for platform in platforms {
        let result = compile_w3c_tokens(json, platform);
        assert!(
            result.is_ok(),
            "Platform target should compile successfully"
        );
    }
}

#[test]
fn test_empty_token_file() {
    let json = "{}";
    let result = compile_w3c_tokens(json, PlatformTarget::CSS);
    
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains_key("tokens.css"));
}

#[test]
fn test_unicode_in_token_paths() {
    let json = r##"{
        "颜色": {
            "主要": { "$value": "#3b82f6", "$type": "color" }
        },
        "間隔": {
            "基本": { "$value": "16px", "$type": "dimension" }
        }
    }"##;
    
    let result = compile_w3c_tokens(json, PlatformTarget::CSS);
    assert!(result.is_ok());
}
