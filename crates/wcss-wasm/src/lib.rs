use wasm_bindgen::prelude::*;
use wcss_compiler::config::CompilerConfig;

#[wasm_bindgen]
pub struct WCSSCompiler;

#[wasm_bindgen]
impl WCSSCompiler {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self
    }

    /// Compile WCSS source to CSS.
    pub fn compile(&self, source: &str, config_json: &str) -> JsValue {
        let config: CompilerConfig = serde_json::from_str(config_json)
            .unwrap_or_default();

        let result = wcss_compiler::compile(source, &config);
        serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
    }

    /// Parse WCSS source into AST (JSON).
    pub fn parse(&self, source: &str) -> JsValue {
        match wcss_compiler::parse(source) {
            Ok(ast) => serde_wasm_bindgen::to_value(&ast).unwrap_or(JsValue::NULL),
            Err(errors) => serde_wasm_bindgen::to_value(&errors).unwrap_or(JsValue::NULL),
        }
    }

    /// Format WCSS source code.
    pub fn format(&self, source: &str) -> Result<String, JsValue> {
        wcss_compiler::format(source)
            .map_err(|errors| {
                let msg = errors.iter()
                    .map(|e| e.message.clone())
                    .collect::<Vec<_>>()
                    .join("; ");
                JsValue::from_str(&msg)
            })
    }

    /// Validate WCSS source without full compilation.
    pub fn validate(&self, source: &str, config_json: &str) -> JsValue {
        let config: CompilerConfig = serde_json::from_str(config_json)
            .unwrap_or_default();

        match wcss_compiler::parse(source) {
            Ok(ast) => {
                let errors = wcss_compiler::validator::validate(&ast, &config);
                serde_wasm_bindgen::to_value(&errors).unwrap_or(JsValue::NULL)
            }
            Err(errors) => serde_wasm_bindgen::to_value(&errors).unwrap_or(JsValue::NULL),
        }
    }

    /// Compile W3C Design Tokens to platform-specific code.
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
            _ => PlatformTarget::CSS, // Default to CSS
        };

        match wcss_compiler::compile_w3c_tokens(json_content, platform_target) {
            Ok(output) => {
                let result = W3CTokensResult {
                    output,
                    errors: vec![],
                };
                serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
            }
            Err(errors) => {
                let result = W3CTokensResult {
                    output: std::collections::HashMap::new(),
                    errors,
                };
                serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
            }
        }
    }
}

/// Result of W3C Design Tokens compilation
#[derive(serde::Serialize)]
struct W3CTokensResult {
    output: std::collections::HashMap<String, String>,
    errors: Vec<wcss_compiler::error::CompilerError>,
}
