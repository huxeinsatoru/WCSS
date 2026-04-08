use std::path::Path;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use colored::Colorize;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use super::build;

/// Run the watch command.
pub fn run(
    input: &str,
    output: Option<&str>,
    minify: bool,
    source_maps: Option<&str>,
    typed_om: bool,
    tree_shaking: bool,
) -> Result<(), String> {
    println!("{} Watching for changes in: {}", "info:".cyan(), input);

    // Run an initial build.
    let _ = build::run(input, output, minify, source_maps, typed_om, tree_shaking);

    // Determine the directory to watch.
    let watch_path = if Path::new(input).is_dir() {
        input.to_string()
    } else if Path::new(input).is_file() {
        Path::new(input)
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| ".".to_string())
    } else {
        // For glob patterns, watch the current directory.
        ".".to_string()
    };

    let (tx, rx) = mpsc::channel();

    let mut watcher = RecommendedWatcher::new(tx, Config::default())
        .map_err(|e| format!("Failed to create watcher: {e}"))?;

    watcher
        .watch(Path::new(&watch_path), RecursiveMode::Recursive)
        .map_err(|e| format!("Failed to watch {watch_path}: {e}"))?;

    println!("{} Press Ctrl+C to stop\n", "info:".cyan());

    let debounce = Duration::from_millis(100);
    let mut last_build = Instant::now() - debounce;

    // Block and process events.
    for event_result in rx {
        match event_result {
            Ok(Event {
                kind: EventKind::Modify(_) | EventKind::Create(_),
                paths,
                ..
            }) => {
                let euis_paths: Vec<_> = paths
                    .iter()
                    .filter(|p| p.extension().map_or(false, |ext| ext == "euis"))
                    .collect();

                if !euis_paths.is_empty() && last_build.elapsed() >= debounce {
                    for p in &euis_paths {
                        println!("{} File changed: {}", "info:".cyan(), p.display());
                    }
                    println!("{} Recompiling...\n", "info:".cyan());

                    let _ = build::run(
                        input,
                        output,
                        minify,
                        source_maps,
                        typed_om,
                        tree_shaking,
                    );
                    last_build = Instant::now();
                }
            }
            Ok(Event {
                kind: EventKind::Remove(_),
                paths,
                ..
            }) => {
                for p in &paths {
                    if p.extension().map_or(false, |ext| ext == "euis") {
                        println!("{} File removed: {}", "info:".cyan(), p.display());
                    }
                }
            }
            Ok(_) => {}
            Err(e) => {
                eprintln!("{} Watcher error: {}", "error:".red(), e);
            }
        }
    }

    Ok(())
}
