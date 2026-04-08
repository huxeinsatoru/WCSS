use std::fs;

use colored::Colorize;

use super::build::resolve_input_files;

/// Run the format command.
pub fn run(input: &str, write: bool) -> Result<(), String> {
    let files = resolve_input_files(input)?;

    if files.is_empty() {
        return Err(format!("No Euis files found matching: {input}"));
    }

    println!(
        "{} Formatting {} file{}...",
        "info:".cyan(),
        files.len(),
        if files.len() > 1 { "s" } else { "" }
    );

    let mut error_count = 0usize;

    for file in &files {
        if let Err(e) = format_file(file, write) {
            eprintln!("{} Error formatting {}: {}", "error:".red(), file, e);
            error_count += 1;
        }
    }

    if error_count > 0 {
        return Err(format!(
            "Formatting failed for {error_count} file{}",
            if error_count > 1 { "s" } else { "" }
        ));
    }

    if write {
        println!(
            "{} Formatted {} file{}",
            "success:".green(),
            files.len(),
            if files.len() > 1 { "s" } else { "" }
        );
    } else {
        println!("{} Run with --write to save changes", "info:".cyan());
    }

    Ok(())
}

/// Format a single Euis file.
fn format_file(file_path: &str, write: bool) -> Result<(), String> {
    let source =
        fs::read_to_string(file_path).map_err(|e| format!("Failed to read {file_path}: {e}"))?;

    let formatted = euis_compiler::format(&source).map_err(|errors| {
        let msgs: Vec<String> = errors.iter().map(|e| e.message.clone()).collect();
        msgs.join(", ")
    })?;

    if write {
        fs::write(file_path, &formatted)
            .map_err(|e| format!("Failed to write {file_path}: {e}"))?;
        println!("  {} Formatted: {}", "info:".cyan(), file_path);
    } else {
        println!("\n{}:", file_path);
        println!("{}", formatted);
    }

    Ok(())
}
