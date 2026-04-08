use std::fs;
use std::path::Path;

use colored::Colorize;
use euis_compiler::config::PlatformTarget;

/// Map a CLI platform string to the compiler's PlatformTarget enum.
fn parse_platform(platform: &str) -> Result<PlatformTarget, String> {
    match platform.to_lowercase().as_str() {
        "css" => Ok(PlatformTarget::CSS),
        "ios" => Ok(PlatformTarget::IOS),
        "android" => Ok(PlatformTarget::Android),
        "android-kotlin" => Ok(PlatformTarget::AndroidKotlin),
        "flutter" => Ok(PlatformTarget::Flutter),
        "typescript" => Ok(PlatformTarget::TypeScript),
        "docs" => Ok(PlatformTarget::Docs),
        _ => Err(format!(
            "Unknown platform: {platform}. Supported: css, ios, android, android-kotlin, flutter, typescript, docs"
        )),
    }
}

/// Run the tokens command.
pub fn run(input: &str, output: &str, platform: &str) -> Result<(), String> {
    // Validate input file exists.
    if !Path::new(input).exists() {
        return Err(format!("Token file not found: {input}"));
    }

    // Read and validate JSON.
    let token_content =
        fs::read_to_string(input).map_err(|e| format!("Failed to read {input}: {e}"))?;

    // Quick JSON validation.
    let _: serde_json::Value = serde_json::from_str(&token_content)
        .map_err(|e| format!("Invalid JSON in token file {input}: {e}"))?;

    let target = parse_platform(platform)?;

    println!(
        "{} Compiling W3C Design Tokens to {platform}...",
        "info:".cyan()
    );

    let result = euis_compiler::compile_w3c_tokens(&token_content, target).map_err(|errors| {
        let msgs: Vec<String> = errors.iter().map(|e| e.message.clone()).collect();
        format!("Token compilation errors:\n  {}", msgs.join("\n  "))
    })?;

    // Create output directory.
    fs::create_dir_all(output)
        .map_err(|e| format!("Failed to create output directory {output}: {e}"))?;

    // Write output files.
    for (filename, content) in &result {
        let output_path = Path::new(output).join(filename);
        fs::write(&output_path, content).map_err(|e| {
            format!(
                "Failed to write {}: {e}",
                output_path.to_string_lossy()
            )
        })?;
        println!(
            "  {} {} -> {}",
            "info:".cyan(),
            input,
            output_path.to_string_lossy()
        );
    }

    println!(
        "{} W3C Design Tokens compiled successfully to {platform}",
        "success:".green()
    );

    Ok(())
}
