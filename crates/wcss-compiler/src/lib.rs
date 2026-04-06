pub mod ast;
pub mod config;
pub mod error;
pub mod parser;
pub mod validator;
pub mod optimizer;
pub mod codegen;
pub mod tokens;
pub mod formatter;
pub mod sourcemap;
pub mod w3c_parser;
pub mod w3c_validator;
pub mod w3c_resolver;
pub mod w3c_css_generator;
pub mod ios_generator;
pub mod android_generator;
pub mod flutter_generator;
pub mod token_merger;
pub mod w3c_transform;
pub mod docs_generator;
pub mod w3c_optimizer;
pub mod typescript_generator;

use config::CompilerConfig;
use error::CompileResult;

/// Compile WCSS source code to CSS.
pub fn compile(source: &str, config: &CompilerConfig) -> CompileResult {
    let parse_result = parser::parse(source);

    let mut errors = Vec::new();
    let warnings = Vec::new();

    let stylesheet = match parse_result {
        Ok(stylesheet) => stylesheet,
        Err(parse_errors) => {
            return CompileResult {
                css: String::new(),
                js: None,
                source_map: None,
                errors: parse_errors,
                warnings: Vec::new(),
                stats: error::CompilationStats::default(),
            };
        }
    };

    // Validate
    let validation_errors = validator::validate(&stylesheet, config);
    errors.extend(validation_errors);

    if !errors.is_empty() {
        return CompileResult {
            css: String::new(),
            js: None,
            source_map: None,
            errors,
            warnings,
            stats: error::CompilationStats::default(),
        };
    }

    // Track rule count before optimization
    let input_rule_count = stylesheet.rules.len();

    // Resolve tokens (takes ownership, no clone)
    let resolved = tokens::resolve(stylesheet, &config.tokens);

    // Optimize
    let optimized = optimizer::optimize(resolved, config);

    // Generate CSS
    let css_output = codegen::generate_css(&optimized, config);

    // Generate JS (Typed OM) if enabled
    let js_output = if config.typed_om {
        Some(codegen::generate_typed_om_js(&optimized))
    } else {
        None
    };

    // Generate source map if enabled
    let source_map = match config.source_maps {
        config::SourceMapConfig::Disabled => None,
        _ => Some(sourcemap::generate(&optimized, source)),
    };

    let stats = error::CompilationStats {
        input_size: source.len(),
        output_size: css_output.len(),
        compile_time_us: 0,
        rules_processed: optimized.rules.len(),
        rules_eliminated: input_rule_count.saturating_sub(optimized.rules.len()),
    };

    CompileResult {
        css: css_output,
        js: js_output,
        source_map,
        errors,
        warnings,
        stats,
    }
}

/// Parse WCSS source code into an AST.
pub fn parse(source: &str) -> Result<ast::StyleSheet, Vec<error::CompilerError>> {
    parser::parse(source)
}

/// Format WCSS AST back to source code.
pub fn format(source: &str) -> Result<String, Vec<error::CompilerError>> {
    let ast = parser::parse(source)?;
    Ok(formatter::format_stylesheet(&ast))
}

/// Compile W3C Design Tokens to the specified platform target.
///
/// # Arguments
/// * `json_content` - The W3C Design Tokens JSON content
/// * `target` - The platform target (CSS, iOS, Android, Flutter, etc.)
///
/// # Returns
/// A map of filename to content, or a vector of errors if compilation fails.
pub fn compile_w3c_tokens(
    json_content: &str,
    target: config::PlatformTarget,
) -> Result<std::collections::HashMap<String, String>, Vec<error::CompilerError>> {
    use w3c_parser::W3CTokenParser;
    use w3c_validator::TokenTypeValidator;
    use w3c_resolver::TokenReferenceResolver;
    use config::PlatformTarget;
    use android_generator::{AndroidGenerator, AndroidFormat};

    // Parse W3C tokens
    let mut tokens = W3CTokenParser::parse(json_content)?;

    // Validate token types
    for token in &tokens {
        if let Err(e) = TokenTypeValidator::validate(token) {
            return Err(vec![e]);
        }
    }

    // Resolve references
    let mut resolver = TokenReferenceResolver::new(tokens);
    resolver.resolve_all()?;
    tokens = resolver.get_resolved_tokens();

    // Generate output based on target
    let mut output = std::collections::HashMap::new();

    match target {
        PlatformTarget::CSS => {
            let css = w3c_css_generator::CSSGenerator::generate(&tokens, false);
            output.insert("tokens.css".to_string(), css);
        }
        PlatformTarget::IOS => {
            let swift = ios_generator::IOSGenerator::generate(&tokens);
            output.insert("DesignTokens.swift".to_string(), swift);
        }
        PlatformTarget::Android => {
            let files = AndroidGenerator::generate(&tokens, AndroidFormat::XML);
            output.extend(files);
        }
        PlatformTarget::AndroidKotlin => {
            let files = AndroidGenerator::generate(&tokens, AndroidFormat::Kotlin);
            output.extend(files);
        }
        PlatformTarget::Flutter => {
            let dart = flutter_generator::FlutterGenerator::generate(&tokens);
            output.insert("design_tokens.dart".to_string(), dart);
        }
        PlatformTarget::TypeScript => {
            let ts = typescript_generator::TypeScriptGenerator::generate(&tokens);
            output.insert("tokens.ts".to_string(), ts);
        }
        PlatformTarget::Docs => {
            let docs = docs_generator::DocsGenerator::generate(&tokens);
            output.insert("index.html".to_string(), docs);
        }
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_w3c_tokens_to_css() {
        let json = r##"{
            "color": {
                "primary": {
                    "$value": "#3b82f6",
                    "$type": "color"
                }
            }
        }"##;

        let result = compile_w3c_tokens(json, config::PlatformTarget::CSS);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.contains_key("tokens.css"));
        assert!(output["tokens.css"].contains("--color-primary"));
        assert!(output["tokens.css"].contains("#3b82f6"));
    }

    #[test]
    fn test_compile_w3c_tokens_to_ios() {
        let json = r##"{
            "color": {
                "primary": {
                    "$value": "#3b82f6",
                    "$type": "color"
                }
            }
        }"##;

        let result = compile_w3c_tokens(json, config::PlatformTarget::IOS);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.contains_key("DesignTokens.swift"));
        assert!(output["DesignTokens.swift"].contains("UIColor"));
    }

    #[test]
    fn test_compile_w3c_tokens_to_android_xml() {
        let json = r##"{
            "color": {
                "primary": {
                    "$value": "#3b82f6",
                    "$type": "color"
                }
            }
        }"##;

        let result = compile_w3c_tokens(json, config::PlatformTarget::Android);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.contains_key("colors.xml"));
        assert!(output["colors.xml"].contains("color_primary"));
    }

    #[test]
    fn test_compile_w3c_tokens_to_flutter() {
        let json = r##"{
            "color": {
                "primary": {
                    "$value": "#3b82f6",
                    "$type": "color"
                }
            }
        }"##;

        let result = compile_w3c_tokens(json, config::PlatformTarget::Flutter);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.contains_key("design_tokens.dart"));
        assert!(output["design_tokens.dart"].contains("Color(0xFF3B82F6)"));
    }

    #[test]
    fn test_compile_w3c_tokens_invalid_json() {
        let json = "invalid json";

        let result = compile_w3c_tokens(json, config::PlatformTarget::CSS);
        assert!(result.is_err());
    }

    #[test]
    fn test_compile_w3c_tokens_to_typescript() {
        let json = r##"{
            "color": {
                "primary": {
                    "$value": "#3b82f6",
                    "$type": "color"
                }
            }
        }"##;

        let result = compile_w3c_tokens(json, config::PlatformTarget::TypeScript);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.contains_key("tokens.ts"));
        assert!(output["tokens.ts"].contains("export type TokenPath"));
        assert!(output["tokens.ts"].contains("export const tokens"));
    }

    #[test]
    fn test_compile_w3c_tokens_to_docs() {
        let json = r##"{
            "color": {
                "primary": {
                    "$value": "#3b82f6",
                    "$type": "color"
                }
            }
        }"##;

        let result = compile_w3c_tokens(json, config::PlatformTarget::Docs);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.contains_key("index.html"));
        assert!(output["index.html"].contains("<!DOCTYPE html>"));
        assert!(output["index.html"].contains("color.primary"));
    }
}
