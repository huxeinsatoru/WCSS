use wcss_compiler::{compile, config::CompilerConfig};

#[test]
fn test_modern_hsl_syntax() {
    let source = r#"
        .test {
            background: hsl(240 2% 12% / 0.82);
            color: hsl(120 100% 50%);
        }
    "#;
    
    let config = CompilerConfig::default();
    let result = compile(source, &config);
    
    assert!(result.errors.is_empty(), "Should compile without errors");
    assert!(result.css.contains("hsla(240, 2%, 12%, 0.82)"));
    assert!(result.css.contains("hsl(120, 100%, 50%)"));
}

#[test]
fn test_legacy_hsl_syntax() {
    let source = r#"
        .test {
            background: hsla(240, 2%, 12%, 0.82);
            color: hsl(120, 100%, 50%);
        }
    "#;
    
    let config = CompilerConfig::default();
    let result = compile(source, &config);
    
    assert!(result.errors.is_empty());
    assert!(result.css.contains("hsla(240, 2%, 12%, 0.82)"));
    assert!(result.css.contains("hsl(120, 100%, 50%)"));
}

#[test]
fn test_modern_rgb_syntax() {
    let source = r#"
        .test {
            color: rgb(255 0 0 / 0.5);
            background: rgb(0 128 255);
        }
    "#;
    
    let config = CompilerConfig::default();
    let result = compile(source, &config);
    
    assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);
    // Normalize: remove all whitespace for comparison
    let normalized = result.css.chars().filter(|c| !c.is_whitespace()).collect::<String>();
    assert!(normalized.contains("rgba(255,0,0,0.5)") || normalized.contains("color:rgba(255,0,0,0.5)"), "CSS output: {}", result.css);
    assert!(normalized.contains("rgb(0,128,255)") || normalized.contains("background:rgb(0,128,255)"), "CSS output: {}", result.css);
}

#[test]
fn test_legacy_rgb_syntax() {
    let source = r#"
        .test {
            color: rgba(255, 0, 0, 0.5);
            background: rgb(0, 128, 255);
        }
    "#;
    
    let config = CompilerConfig::default();
    let result = compile(source, &config);
    
    assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);
    let normalized = result.css.chars().filter(|c| !c.is_whitespace()).collect::<String>();
    assert!(normalized.contains("rgba(255,0,0,0.5)") || normalized.contains("color:rgba(255,0,0,0.5)"), "CSS output: {}", result.css);
    assert!(normalized.contains("rgb(0,128,255)") || normalized.contains("background:rgb(0,128,255)"), "CSS output: {}", result.css);
}

#[test]
fn test_webkit_vendor_prefix() {
    let source = r#"
        .test {
            -webkit-backdrop-filter: blur(48px);
            -webkit-transform: translateY(0);
        }
    "#;
    
    let config = CompilerConfig::default();
    let result = compile(source, &config);
    
    assert!(result.errors.is_empty());
    assert!(result.css.contains("-webkit-backdrop-filter"));
    assert!(result.css.contains("-webkit-transform"));
}

#[test]
fn test_moz_vendor_prefix() {
    let source = r#"
        .test {
            -moz-transform: scale(1.2);
            -moz-user-select: none;
        }
    "#;
    
    let config = CompilerConfig::default();
    let result = compile(source, &config);
    
    assert!(result.errors.is_empty());
    assert!(result.css.contains("-moz-transform"));
    assert!(result.css.contains("-moz-user-select"));
}

#[test]
fn test_ms_vendor_prefix() {
    let source = r#"
        .test {
            -ms-transform: rotate(45deg);
            -ms-filter: blur(5px);
        }
    "#;
    
    let config = CompilerConfig::default();
    let result = compile(source, &config);
    
    assert!(result.errors.is_empty());
    assert!(result.css.contains("-ms-transform"));
    assert!(result.css.contains("-ms-filter"));
}

#[test]
fn test_o_vendor_prefix() {
    let source = r#"
        .test {
            -o-transform: skew(10deg);
        }
    "#;
    
    let config = CompilerConfig::default();
    let result = compile(source, &config);
    
    assert!(result.errors.is_empty());
    assert!(result.css.contains("-o-transform"));
}

#[test]
fn test_inset_property() {
    let source = r#"
        .test {
            inset: 0;
        }
    "#;
    
    let config = CompilerConfig::default();
    let result = compile(source, &config);
    
    assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);
    let normalized = result.css.replace(char::is_whitespace, "");
    assert!(normalized.contains("inset:0"), "CSS output: {}", result.css);
}

#[test]
fn test_will_change_property() {
    let source = r#"
        .test {
            will-change: transform;
        }
    "#;
    
    let config = CompilerConfig::default();
    let result = compile(source, &config);
    
    assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);
    let normalized = result.css.replace(char::is_whitespace, "");
    assert!(normalized.contains("will-change:transform"), "CSS output: {}", result.css);
}

#[test]
fn test_transform_origin_property() {
    let source = r#"
        .test {
            transform-origin: center bottom;
        }
    "#;
    
    let config = CompilerConfig::default();
    let result = compile(source, &config);
    
    assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);
    let normalized = result.css.replace(char::is_whitespace, "");
    assert!(normalized.contains("transform-origin:centerbottom"), "CSS output: {}", result.css);
}

#[test]
fn test_box_sizing_property() {
    let source = r#"
        .test {
            box-sizing: content-box;
        }
    "#;
    
    let config = CompilerConfig::default();
    let result = compile(source, &config);
    
    assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);
    let normalized = result.css.replace(char::is_whitespace, "");
    assert!(normalized.contains("box-sizing:content-box"), "CSS output: {}", result.css);
}

#[test]
fn test_complex_box_shadow() {
    let source = r#"
        .test {
            box-shadow: 0 4px 6px rgba(0,0,0,0.1), 0 2px 4px rgba(0,0,0,0.06);
        }
    "#;
    
    let config = CompilerConfig::default();
    let result = compile(source, &config);
    
    assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);
    let normalized = result.css.chars().filter(|c| !c.is_whitespace()).collect::<String>();
    assert!(normalized.contains("box-shadow"), "CSS output: {}", result.css);
    // Box shadow is parsed as literal, so just check it's there
    assert!(result.css.contains("box-shadow"), "CSS output: {}", result.css);
}

#[test]
fn test_mixed_modern_and_vendor() {
    let source = r#"
        .test {
            -webkit-backdrop-filter: blur(48px);
            backdrop-filter: blur(48px);
            background: hsl(240 2% 12% / 0.82);
            inset: 0;
            will-change: transform;
        }
    "#;
    
    let config = CompilerConfig::default();
    let result = compile(source, &config);
    
    assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);
    let normalized = result.css.replace(char::is_whitespace, "");
    assert!(normalized.contains("-webkit-backdrop-filter"), "CSS output: {}", result.css);
    assert!(normalized.contains("backdrop-filter"), "CSS output: {}", result.css);
    assert!(normalized.contains("hsla(240,2%,12%,0.82)"), "CSS output: {}", result.css);
    assert!(normalized.contains("inset:0"), "CSS output: {}", result.css);
    assert!(normalized.contains("will-change:transform"), "CSS output: {}", result.css);
}
