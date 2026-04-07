use std::fs;
use std::path::Path;

use colored::Colorize;
use wcss_compiler::config::{CompilerConfig, SourceMapConfig};

/// Build a compiler config from CLI flags.
pub fn make_config(
    minify: bool,
    source_maps: Option<&str>,
    typed_om: bool,
    tree_shaking: bool,
) -> CompilerConfig {
    let source_maps_cfg = match source_maps {
        Some("inline") => SourceMapConfig::Inline,
        Some("external") | Some(_) => SourceMapConfig::External,
        None => SourceMapConfig::Disabled,
    };

    CompilerConfig {
        minify,
        source_maps: source_maps_cfg,
        typed_om,
        tree_shaking,
        ..Default::default()
    }
}

/// Resolve input files from a path or glob pattern.
pub fn resolve_input_files(input: &str) -> Result<Vec<String>, String> {
    let path = Path::new(input);

    // If input is a directory, search for .wcss files inside it.
    let pattern = if path.is_dir() {
        format!("{}/**/*.wcss", input.trim_end_matches('/'))
    } else {
        input.to_string()
    };

    let entries = glob::glob(&pattern).map_err(|e| format!("Invalid glob pattern: {e}"))?;

    let mut files: Vec<String> = Vec::new();
    for entry in entries {
        match entry {
            Ok(p) => {
                let s = p.to_string_lossy().to_string();
                // Skip node_modules and dist directories.
                if !s.contains("node_modules") && !s.contains("/dist/") {
                    files.push(s);
                }
            }
            Err(e) => eprintln!("{}: {}", "warning".yellow(), e),
        }
    }

    Ok(files)
}

/// Run the build command.
pub fn run(
    input: &str,
    output: Option<&str>,
    minify: bool,
    source_maps: Option<&str>,
    typed_om: bool,
    tree_shaking: bool,
) -> Result<(), String> {
    let config = make_config(minify, source_maps, typed_om, tree_shaking);
    let files = resolve_input_files(input)?;

    if files.is_empty() {
        return Err(format!("No WCSS files found matching: {input}"));
    }

    println!(
        "{} Compiling {} file{}...",
        "info:".cyan(),
        files.len(),
        if files.len() > 1 { "s" } else { "" }
    );

    let mut total_errors = 0usize;
    let mut total_warnings = 0usize;

    for file in &files {
        let (errors, warnings) = compile_file(file, output, &config)?;
        total_errors += errors;
        total_warnings += warnings;
    }

    if total_errors > 0 {
        return Err(format!(
            "Compilation failed with {total_errors} error{}",
            if total_errors > 1 { "s" } else { "" }
        ));
    }

    if total_warnings > 0 {
        println!(
            "{} Compilation completed with {total_warnings} warning{}",
            "warning:".yellow(),
            if total_warnings > 1 { "s" } else { "" }
        );
    } else {
        println!("{} Compilation completed successfully", "success:".green());
    }

    Ok(())
}

/// Compile a single WCSS file, write outputs, return (error_count, warning_count).
fn compile_file(
    input_path: &str,
    output_path: Option<&str>,
    config: &CompilerConfig,
) -> Result<(usize, usize), String> {
    let source =
        fs::read_to_string(input_path).map_err(|e| format!("Failed to read {input_path}: {e}"))?;

    let result = wcss_compiler::compile(&source, config);

    // Display errors.
    if !result.errors.is_empty() {
        eprintln!("\n{} in {}:", "Errors".red(), input_path);
        for err in &result.errors {
            eprintln!("  {} {}", "error:".red(), err.message);
            if let Some(ref suggestion) = err.suggestion {
                eprintln!("  {} {}", "hint:".cyan(), suggestion);
            }
        }
        return Ok((result.errors.len(), result.warnings.len()));
    }

    // Display warnings.
    if !result.warnings.is_empty() {
        eprintln!("\n{} in {}:", "Warnings".yellow(), input_path);
        for w in &result.warnings {
            eprintln!("  {} {}", "warning:".yellow(), w.message);
        }
    }

    // Determine output path.
    let css_output = match output_path {
        Some(p) => p.to_string(),
        None => input_path.replace(".wcss", ".css"),
    };

    // Create parent directories.
    if let Some(parent) = Path::new(&css_output).parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create output directory: {e}"))?;
    }

    // Write CSS.
    fs::write(&css_output, &result.css)
        .map_err(|e| format!("Failed to write {css_output}: {e}"))?;

    // Write JS (Typed OM) if present.
    if let Some(ref js) = result.js {
        let js_output = css_output.replace(".css", ".js");
        fs::write(&js_output, js).map_err(|e| format!("Failed to write {js_output}: {e}"))?;
    }

    // Write source map if external.
    if let Some(ref sm) = result.source_map {
        if matches!(config.source_maps, SourceMapConfig::External) {
            let map_output = format!("{css_output}.map");
            fs::write(&map_output, sm)
                .map_err(|e| format!("Failed to write {map_output}: {e}"))?;
        }
    }

    // Print stats.
    let stats = &result.stats;
    if stats.input_size > 0 {
        let reduction = (1.0 - stats.output_size as f64 / stats.input_size as f64) * 100.0;
        println!(
            "  {} {} -> {} ({:.1}% reduction)",
            "info:".cyan(),
            input_path,
            css_output,
            reduction
        );
    }

    Ok((0, result.warnings.len()))
}
