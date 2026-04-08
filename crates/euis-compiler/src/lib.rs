pub mod ast;
pub mod config;
pub mod content_scanner;
pub mod diagnostics;
pub mod error;
pub mod parser;
pub mod validator;
pub mod optimizer;
pub mod codegen;
pub mod tokens;
pub mod formatter;
pub mod sourcemap;
pub mod prefixer;
pub mod plugin;
pub mod cache;
pub mod parallel;
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
pub mod preflight;
pub mod tailwind_migration;
pub mod theming;

pub use parallel::{parallel_compile_files, parallel_optimize, parallel_parse};

use config::CompilerConfig;
use error::CompileResult;
use plugin::PluginRegistry;

/// Compile Euis source code to CSS.
pub fn compile(source: &str, config: &CompilerConfig) -> CompileResult {
    compile_with_plugins(source, config, &PluginRegistry::new())
}

/// Compile Euis source code to CSS with plugin support.
pub fn compile_with_plugins(source: &str, config: &CompilerConfig, plugins: &PluginRegistry) -> CompileResult {
    let mut source_owned = source.to_string();

    // Auto-scan content paths if tree-shaking is enabled but no used_classes provided
    let mut config_with_scanned = config.clone();
    if config.tree_shaking && config.used_classes.is_empty() && !config.content_paths.is_empty() {
        // Convert PathBuf to String patterns
        let patterns: Vec<String> = config.content_paths
            .iter()
            .filter_map(|p| p.to_str().map(|s| s.to_string()))
            .collect();

        if !patterns.is_empty() {
            match content_scanner::scan_content_paths(patterns) {
                Ok(scanned_classes) => {
                    config_with_scanned.used_classes = scanned_classes.into_iter().collect();
                }
                Err(e) => {
                    eprintln!("Warning: Content scanning failed: {}", e);
                }
            }
        }
    }

    let config = &config_with_scanned;

    // Plugin: before_parse
    if plugins.has_plugins() {
        if let Err(e) = plugins.run_before_parse(&mut source_owned) {
            return CompileResult {
                css: String::new(),
                js: None,
                source_map: None,
                errors: vec![error::CompilerError::syntax(e.message, ast::Span::empty())],
                warnings: Vec::new(),
                stats: error::CompilationStats::default(),
            };
        }
    }

    let parse_result = parser::parse(&source_owned);

    let mut errors = Vec::new();
    let warnings = Vec::new();

    let mut stylesheet = match parse_result {
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

    // Plugin: after_parse
    if plugins.has_plugins() {
        if let Err(e) = plugins.run_after_parse(&mut stylesheet) {
            errors.push(error::CompilerError::syntax(e.message, ast::Span::empty()));
        }
    }

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

    let input_rule_count = stylesheet.rules.len();

    // Resolve tokens (takes ownership, no clone)
    let mut resolved = tokens::resolve(stylesheet, &config.tokens);

    // Plugin: before_optimize
    if plugins.has_plugins() {
        let _ = plugins.run_before_optimize(&mut resolved);
    }

    // Optimize
    let mut optimized = optimizer::optimize(resolved, config);

    // Vendor prefixing
    if config.autoprefixer {
        optimized = prefixer::prefix(optimized, &config.browser_targets);
    }

    // Plugin: after_optimize
    if plugins.has_plugins() {
        let _ = plugins.run_after_optimize(&mut optimized);
    }

    // Plugin: before_codegen
    if plugins.has_plugins() {
        let _ = plugins.run_before_codegen(&mut optimized);
    }

    // Generate CSS
    let mut css_output = codegen::generate_css(&optimized, config);

    // Plugin: after_codegen
    if plugins.has_plugins() {
        let _ = plugins.run_after_codegen(&mut css_output);
    }

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

/// Compile multiple Euis source strings, using parallel processing when there
/// are more than one source to compile.
///
/// Results are returned in the same order as the input sources.
pub fn compile_multiple(sources: &[&str], config: &CompilerConfig) -> Vec<CompileResult> {
    if sources.len() <= 1 {
        // Sequential for a single file — avoid rayon overhead
        sources.iter().map(|s| compile(s, config)).collect()
    } else {
        use rayon::prelude::*;
        sources
            .par_iter()
            .map(|s| compile(s, config))
            .collect()
    }
}

/// Parse Euis source code into an AST.
pub fn parse(source: &str) -> Result<ast::StyleSheet, Vec<error::CompilerError>> {
    parser::parse(source)
}

/// Format Euis AST back to source code.
pub fn format(source: &str) -> Result<String, Vec<error::CompilerError>> {
    let ast = parser::parse(source)?;
    Ok(formatter::format_stylesheet(&ast))
}

/// Compile W3C Design Tokens to the specified platform target.
pub fn compile_w3c_tokens(
    json_content: &str,
    target: config::PlatformTarget,
) -> Result<std::collections::HashMap<String, String>, Vec<error::CompilerError>> {
    use w3c_parser::W3CTokenParser;
    use w3c_validator::TokenTypeValidator;
    use w3c_resolver::TokenReferenceResolver;
    use config::PlatformTarget;
    use android_generator::{AndroidGenerator, AndroidFormat};

    let mut tokens = W3CTokenParser::parse(json_content)?;

    for token in &tokens {
        if let Err(e) = TokenTypeValidator::validate(token) {
            return Err(vec![e]);
        }
    }

    let mut resolver = TokenReferenceResolver::new(tokens);
    resolver.resolve_all()?;
    tokens = resolver.get_resolved_tokens();

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
    fn test_compile_basic() {
        let source = ".btn { color: red; }";
        let config = CompilerConfig { minify: false, ..Default::default() };
        let result = compile(source, &config);
        assert!(result.errors.is_empty());
        assert!(result.css.contains(".btn"));
        assert!(result.css.contains("color: red"));
    }

    #[test]
    fn test_compile_with_keyframes() {
        let source = r#"
            @keyframes fadeIn {
                from { opacity: 0; }
                to { opacity: 1; }
            }
            .fade { animation: fadeIn 1s; }
        "#;
        let config = CompilerConfig { minify: false, ..Default::default() };
        let result = compile(source, &config);
        assert!(result.errors.is_empty());
        assert!(result.css.contains("@keyframes fadeIn"));
    }

    #[test]
    fn test_compile_with_layer() {
        let source = r#"
            @layer utilities {
                .hidden { display: none; }
            }
        "#;
        let config = CompilerConfig { minify: false, ..Default::default() };
        let result = compile(source, &config);
        assert!(result.errors.is_empty());
        assert!(result.css.contains("@layer utilities"));
    }

    #[test]
    fn test_compile_with_dark_mode() {
        let source = r#"
            .card {
                background: white;
                &:dark {
                    background: black;
                }
            }
        "#;
        let config = CompilerConfig { minify: false, ..Default::default() };
        let result = compile(source, &config);
        assert!(result.errors.is_empty());
        assert!(result.css.contains("prefers-color-scheme: dark"));
    }

    #[test]
    fn test_compile_multi_selector() {
        let source = ".a, .b { color: red; }";
        let config = CompilerConfig { minify: false, ..Default::default() };
        let result = compile(source, &config);
        assert!(result.errors.is_empty());
        assert!(result.css.contains(".a, .b"));
    }

    #[test]
    fn test_compile_id_and_tag() {
        let source = r#"
            #main { color: red; }
            div { margin: 0; }
        "#;
        let config = CompilerConfig { minify: false, ..Default::default() };
        let result = compile(source, &config);
        assert!(result.errors.is_empty());
        assert!(result.css.contains("#main"));
        assert!(result.css.contains("div"));
    }

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
    }

    #[test]
    fn test_compile_with_plugin() {
        use plugin::{Plugin, PluginError, PluginRegistry};

        struct AddCommentPlugin;
        impl Plugin for AddCommentPlugin {
            fn name(&self) -> &str { "add-comment" }
            fn after_codegen(&self, css: &mut String) -> Result<(), PluginError> {
                css.insert_str(0, "/* Generated by Euis */\n");
                Ok(())
            }
        }

        let mut registry = PluginRegistry::new();
        registry.add(Box::new(AddCommentPlugin));

        let source = ".btn { color: red; }";
        let config = CompilerConfig { minify: false, ..Default::default() };
        let result = compile_with_plugins(source, &config, &registry);
        assert!(result.css.starts_with("/* Generated by Euis */"));
    }
}
