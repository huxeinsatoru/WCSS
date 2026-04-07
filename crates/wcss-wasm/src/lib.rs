use wasm_bindgen::prelude::*;
use wcss_compiler::config::CompilerConfig;
use serde::Serialize;

/// WCSS Compiler WebAssembly bindings.
/// 
/// This struct provides the JavaScript API for the WCSS compiler.
/// All methods are exposed to JavaScript via wasm-bindgen.
#[wasm_bindgen]
pub struct WCSSCompiler;

/// JavaScript-friendly error object for WASM boundary.
/// 
/// This struct ensures errors are properly serialized and don't expose
/// internal Rust panic messages to JavaScript consumers.
#[derive(Serialize)]
struct JsError {
    message: String,
    line: Option<usize>,
    column: Option<usize>,
    severity: String,
}

impl JsError {
    fn from_string(message: String) -> Self {
        Self {
            message,
            line: None,
            column: None,
            severity: "error".to_string(),
        }
    }

    fn from_compiler_error(error: &wcss_compiler::error::CompilerError) -> Self {
        Self {
            message: error.message.clone(),
            line: if error.span.line > 0 { Some(error.span.line) } else { None },
            column: if error.span.column > 0 { Some(error.span.column) } else { None },
            severity: "error".to_string(),
        }
    }
}

/// Result wrapper for WASM boundary that includes proper error handling.
#[derive(Serialize)]
struct JsResult<T> {
    success: bool,
    data: Option<T>,
    error: Option<JsError>,
}

#[wasm_bindgen]
impl WCSSCompiler {
    /// Create a new WCSS compiler instance.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self
    }

    /// Compile WCSS source to CSS.
    /// 
    /// # Arguments
    /// * `source` - WCSS source code
    /// * `config_json` - JSON string of CompilerConfig
    /// 
    /// # Returns
    /// CompileResult object with css, errors, warnings, source_map fields
    /// 
    /// **Validates: Requirements 7.1, 7.2, 1.3, 8.1, 8.2, 8.6**
    pub fn compile(&self, source: &str, config_json: &str) -> JsValue {
        // Parse configuration with proper error handling
        let config: CompilerConfig = match serde_json::from_str(config_json) {
            Ok(cfg) => cfg,
            Err(e) => {
                // Invalid config JSON - return error with helpful message
                let error = JsError::from_string(format!(
                    "Invalid configuration JSON: {}. Using default configuration.",
                    e
                ));
                // Log warning but continue with defaults
                web_sys::console::warn_1(&format!("WCSS: {}", error.message).into());
                CompilerConfig::default()
            }
        };

        // Compile with error handling
        let result = wcss_compiler::compile(source, &config);
        
        // Convert to JsValue with proper error handling
        match serde_wasm_bindgen::to_value(&result) {
            Ok(js_value) => js_value,
            Err(e) => {
                // Serialization failed - return structured error
                let error = JsError::from_string(format!(
                    "Failed to serialize compilation result: {}",
                    e
                ));
                let error_result = JsResult {
                    success: false,
                    data: None::<()>,
                    error: Some(error),
                };
                serde_wasm_bindgen::to_value(&error_result).unwrap_or(JsValue::NULL)
            }
        }
    }

    /// Parse WCSS source into AST (JSON).
    /// 
    /// # Arguments
    /// * `source` - WCSS source code
    /// 
    /// # Returns
    /// AST as JSON or parse errors
    /// 
    /// **Validates: Requirements 7.2, 8.1, 8.2, 8.6**
    pub fn parse(&self, source: &str) -> JsValue {
        match wcss_compiler::parse(source) {
            Ok(ast) => {
                match serde_wasm_bindgen::to_value(&ast) {
                    Ok(js_value) => js_value,
                    Err(e) => {
                        let error = JsError::from_string(format!(
                            "Failed to serialize AST: {}",
                            e
                        ));
                        let error_result = JsResult {
                            success: false,
                            data: None::<()>,
                            error: Some(error),
                        };
                        serde_wasm_bindgen::to_value(&error_result).unwrap_or(JsValue::NULL)
                    }
                }
            }
            Err(errors) => {
                // Convert compiler errors to JS-friendly format
                let js_errors: Vec<JsError> = errors.iter()
                    .map(JsError::from_compiler_error)
                    .collect();
                
                match serde_wasm_bindgen::to_value(&js_errors) {
                    Ok(js_value) => js_value,
                    Err(e) => {
                        let error = JsError::from_string(format!(
                            "Failed to serialize parse errors: {}",
                            e
                        ));
                        let error_result = JsResult {
                            success: false,
                            data: None::<()>,
                            error: Some(error),
                        };
                        serde_wasm_bindgen::to_value(&error_result).unwrap_or(JsValue::NULL)
                    }
                }
            }
        }
    }

    /// Format WCSS source code.
    /// 
    /// # Arguments
    /// * `source` - WCSS source code
    /// 
    /// # Returns
    /// Formatted source or error
    /// 
    /// **Validates: Requirements 7.3, 8.1, 8.2, 8.6**
    pub fn format(&self, source: &str) -> Result<String, JsValue> {
        wcss_compiler::format(source)
            .map_err(|errors| {
                // Convert compiler errors to JS-friendly format with line/column info
                let js_errors: Vec<JsError> = errors.iter()
                    .map(JsError::from_compiler_error)
                    .collect();
                
                // Create a descriptive error message
                let error_messages: Vec<String> = js_errors.iter()
                    .map(|e| {
                        if let (Some(line), Some(col)) = (e.line, e.column) {
                            format!("{} (line {}:{})", e.message, line, col)
                        } else {
                            e.message.clone()
                        }
                    })
                    .collect();
                
                let combined_message = format!(
                    "WCSS formatting failed:\n{}",
                    error_messages.join("\n")
                );
                
                JsValue::from_str(&combined_message)
            })
    }

    /// Validate WCSS source without full compilation.
    /// 
    /// # Arguments
    /// * `source` - WCSS source code
    /// * `config_json` - JSON string of CompilerConfig
    /// 
    /// # Returns
    /// Validation errors
    /// 
    /// **Validates: Requirements 7.4, 8.1, 8.2, 8.6**
    pub fn validate(&self, source: &str, config_json: &str) -> JsValue {
        // Parse configuration with proper error handling
        let config: CompilerConfig = match serde_json::from_str(config_json) {
            Ok(cfg) => cfg,
            Err(e) => {
                // Invalid config JSON - return error
                let error = JsError::from_string(format!(
                    "Invalid configuration JSON: {}. Using default configuration.",
                    e
                ));
                web_sys::console::warn_1(&format!("WCSS: {}", error.message).into());
                CompilerConfig::default()
            }
        };

        match wcss_compiler::parse(source) {
            Ok(ast) => {
                let errors = wcss_compiler::validator::validate(&ast, &config);
                
                // Convert to JS-friendly format
                let js_errors: Vec<JsError> = errors.iter()
                    .map(JsError::from_compiler_error)
                    .collect();
                
                match serde_wasm_bindgen::to_value(&js_errors) {
                    Ok(js_value) => js_value,
                    Err(e) => {
                        let error = JsError::from_string(format!(
                            "Failed to serialize validation errors: {}",
                            e
                        ));
                        let error_result = JsResult {
                            success: false,
                            data: None::<()>,
                            error: Some(error),
                        };
                        serde_wasm_bindgen::to_value(&error_result).unwrap_or(JsValue::NULL)
                    }
                }
            }
            Err(errors) => {
                // Parse errors - convert to JS-friendly format
                let js_errors: Vec<JsError> = errors.iter()
                    .map(JsError::from_compiler_error)
                    .collect();
                
                match serde_wasm_bindgen::to_value(&js_errors) {
                    Ok(js_value) => js_value,
                    Err(e) => {
                        let error = JsError::from_string(format!(
                            "Failed to serialize parse errors: {}",
                            e
                        ));
                        let error_result = JsResult {
                            success: false,
                            data: None::<()>,
                            error: Some(error),
                        };
                        serde_wasm_bindgen::to_value(&error_result).unwrap_or(JsValue::NULL)
                    }
                }
            }
        }
    }

    /// Compile W3C Design Tokens to platform-specific code.
    /// 
    /// # Arguments
    /// * `json_content` - W3C tokens JSON
    /// * `target` - Platform target (CSS, IOS, Android, AndroidKotlin, Flutter, TypeScript, Docs)
    /// 
    /// # Returns
    /// Generated code files
    /// 
    /// **Validates: Requirements 7.5, 8.1, 8.2, 8.6**
    pub fn compile_w3c_tokens(&self, json_content: &str, target: &str) -> JsValue {
        use wcss_compiler::config::PlatformTarget;

        let platform_target = match target {
            "CSS" => PlatformTarget::CSS,
            "IOS" => PlatformTarget::IOS,
            "Android" => PlatformTarget::Android,
            "AndroidKotlin" => PlatformTarget::AndroidKotlin,
            "Flutter" => PlatformTarget::Flutter,
            "TypeScript" => PlatformTarget::TypeScript,
            "Docs" => PlatformTarget::Docs,
            _ => {
                // Invalid target - return error
                let error = JsError::from_string(format!(
                    "Invalid platform target '{}'. Valid targets: CSS, IOS, Android, AndroidKotlin, Flutter, TypeScript, Docs",
                    target
                ));
                let error_result = W3CTokensResult {
                    output: std::collections::HashMap::new(),
                    errors: vec![],
                    error: Some(error),
                };
                return match serde_wasm_bindgen::to_value(&error_result) {
                    Ok(js_value) => js_value,
                    Err(e) => {
                        let error = JsError::from_string(format!(
                            "Failed to serialize error result: {}",
                            e
                        ));
                        let error_result = JsResult {
                            success: false,
                            data: None::<()>,
                            error: Some(error),
                        };
                        serde_wasm_bindgen::to_value(&error_result).unwrap_or(JsValue::NULL)
                    }
                };
            }
        };

        match wcss_compiler::compile_w3c_tokens(json_content, platform_target) {
            Ok(output) => {
                let result = W3CTokensResult {
                    output,
                    errors: vec![],
                    error: None,
                };
                match serde_wasm_bindgen::to_value(&result) {
                    Ok(js_value) => js_value,
                    Err(e) => {
                        let error = JsError::from_string(format!(
                            "Failed to serialize W3C tokens result: {}",
                            e
                        ));
                        let error_result = JsResult {
                            success: false,
                            data: None::<()>,
                            error: Some(error),
                        };
                        serde_wasm_bindgen::to_value(&error_result).unwrap_or(JsValue::NULL)
                    }
                }
            }
            Err(errors) => {
                // Convert compiler errors to JS-friendly format
                let js_errors: Vec<JsError> = errors.iter()
                    .map(JsError::from_compiler_error)
                    .collect();
                
                let result = W3CTokensResult {
                    output: std::collections::HashMap::new(),
                    errors: js_errors,
                    error: None,
                };
                match serde_wasm_bindgen::to_value(&result) {
                    Ok(js_value) => js_value,
                    Err(e) => {
                        let error = JsError::from_string(format!(
                            "Failed to serialize W3C tokens errors: {}",
                            e
                        ));
                        let error_result = JsResult {
                            success: false,
                            data: None::<()>,
                            error: Some(error),
                        };
                        serde_wasm_bindgen::to_value(&error_result).unwrap_or(JsValue::NULL)
                    }
                }
            }
        }
    }
}

/// Result of W3C Design Tokens compilation
#[derive(Serialize)]
struct W3CTokensResult {
    output: std::collections::HashMap<String, String>,
    errors: Vec<JsError>,
    error: Option<JsError>,
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compiler_struct_exists() {
        // Just verify the struct can be instantiated
        // Actual WASM functionality tests should be run with wasm-pack test
        let _compiler = WCSSCompiler::new();
    }

    #[test]
    fn test_js_error_from_string() {
        let error = JsError::from_string("Test error message".to_string());
        assert_eq!(error.message, "Test error message");
        assert_eq!(error.line, None);
        assert_eq!(error.column, None);
        assert_eq!(error.severity, "error");
    }

    #[test]
    fn test_js_error_from_compiler_error() {
        use wcss_compiler::error::CompilerError;
        use wcss_compiler::ast::Span;

        let span = Span::new(0, 5, 10, 15);
        let compiler_error = CompilerError::syntax("Syntax error test", span);
        
        let js_error = JsError::from_compiler_error(&compiler_error);
        assert_eq!(js_error.message, "Syntax error test");
        assert_eq!(js_error.line, Some(10));
        assert_eq!(js_error.column, Some(15));
        assert_eq!(js_error.severity, "error");
    }

    #[test]
    fn test_js_error_with_zero_line_column() {
        use wcss_compiler::error::CompilerError;
        use wcss_compiler::ast::Span;

        let span = Span::new(0, 5, 0, 0);
        let compiler_error = CompilerError::syntax("Error with no location", span);
        
        let js_error = JsError::from_compiler_error(&compiler_error);
        assert_eq!(js_error.message, "Error with no location");
        assert_eq!(js_error.line, None);
        assert_eq!(js_error.column, None);
    }

    // Note: Full integration tests for WASM bindings should be run with:
    // wasm-pack test --node
    // or
    // wasm-pack test --headless --firefox
    //
    // These tests verify the API surface exists and compiles correctly.
}
