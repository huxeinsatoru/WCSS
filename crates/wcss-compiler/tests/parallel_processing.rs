use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

use wcss_compiler::config::CompilerConfig;
use wcss_compiler::{compile, compile_multiple, parallel_compile_files, parallel_optimize, parallel_parse};

fn default_config() -> CompilerConfig {
    CompilerConfig {
        minify: false,
        ..Default::default()
    }
}

// ---------------------------------------------------------------------------
// Correctness: parallel compilation produces same results as sequential
// ---------------------------------------------------------------------------

#[test]
fn test_parallel_compile_matches_sequential() {
    let sources: Vec<&str> = vec![
        ".a { color: red; }",
        ".b { margin: 0; padding: 10px; }",
        ".c { display: flex; align-items: center; }",
        "#main { background: white; }",
        "div { font-size: 16px; }",
    ];

    let config = default_config();

    // Sequential
    let sequential: Vec<_> = sources.iter().map(|s| compile(s, &config)).collect();

    // Parallel via compile_multiple
    let parallel = compile_multiple(&sources, &config);

    assert_eq!(sequential.len(), parallel.len());
    for (seq, par) in sequential.iter().zip(parallel.iter()) {
        assert_eq!(seq.css, par.css, "CSS output mismatch");
        assert_eq!(seq.errors.len(), par.errors.len(), "Error count mismatch");
    }
}

#[test]
fn test_parallel_compile_single_file_matches_sequential() {
    let sources: Vec<&str> = vec![".single { color: blue; }"];
    let config = default_config();

    let sequential = compile(sources[0], &config);
    let parallel = compile_multiple(&sources, &config);

    assert_eq!(parallel.len(), 1);
    assert_eq!(sequential.css, parallel[0].css);
}

#[test]
fn test_parallel_compile_empty_input() {
    let sources: Vec<&str> = vec![];
    let config = default_config();
    let results = compile_multiple(&sources, &config);
    assert!(results.is_empty());
}

// ---------------------------------------------------------------------------
// Parallel parse correctness
// ---------------------------------------------------------------------------

#[test]
fn test_parallel_parse_correctness() {
    let sources = vec![
        ".x { color: red; }".to_string(),
        ".y { margin: 0; }".to_string(),
        ".z { padding: 5px; }".to_string(),
    ];

    let sequential: Vec<_> = sources
        .iter()
        .map(|s| wcss_compiler::parse(s).unwrap())
        .collect();

    let parallel = parallel_parse(sources);

    assert_eq!(sequential.len(), parallel.len());
    for (seq, par) in sequential.iter().zip(parallel.iter()) {
        assert_eq!(seq.rules.len(), par.rules.len());
        assert_eq!(
            seq.rules[0].selector.class_name,
            par.rules[0].selector.class_name,
        );
    }
}

// ---------------------------------------------------------------------------
// Parallel optimize correctness
// ---------------------------------------------------------------------------

#[test]
fn test_parallel_optimize_correctness() {
    let sources = vec![
        ".a { color: red; }".to_string(),
        ".b { margin: 0; }".to_string(),
    ];

    let config = default_config();

    let stylesheets: Vec<_> = sources
        .iter()
        .map(|s| wcss_compiler::parse(s).unwrap())
        .collect();

    // Sequential optimize
    let sequential: Vec<_> = stylesheets
        .clone()
        .into_iter()
        .map(|ss| wcss_compiler::optimizer::optimize(ss, &config))
        .collect();

    // Parallel optimize
    let parallel = parallel_optimize(stylesheets, &config);

    assert_eq!(sequential.len(), parallel.len());
    for (seq, par) in sequential.iter().zip(parallel.iter()) {
        assert_eq!(seq.rules.len(), par.rules.len());
    }
}

// ---------------------------------------------------------------------------
// Parallel file compilation
// ---------------------------------------------------------------------------

#[test]
fn test_parallel_compile_files() {
    let dir = tempfile::tempdir().unwrap();

    let mut files: Vec<PathBuf> = Vec::new();
    for i in 0..5 {
        let path = dir.path().join(format!("test_{}.wcss", i));
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(f, ".item-{} {{ color: red; }}", i).unwrap();
        files.push(path);
    }

    let config = default_config();
    let results = parallel_compile_files(files, &config);

    assert_eq!(results.len(), 5);
    for result in &results {
        assert!(result.errors.is_empty(), "Unexpected errors: {:?}", result.errors);
        assert!(!result.css.is_empty(), "CSS should not be empty");
    }
}

#[test]
fn test_parallel_compile_files_missing_file() {
    let config = default_config();
    let files = vec![PathBuf::from("/nonexistent/path/file.wcss")];
    let results = parallel_compile_files(files, &config);

    assert_eq!(results.len(), 1);
    assert!(!results[0].errors.is_empty(), "Should have an error for missing file");
}

// ---------------------------------------------------------------------------
// Performance: parallel should not be slower than sequential for many files
// ---------------------------------------------------------------------------

#[test]
fn test_parallel_performance_with_many_sources() {
    // Generate a moderately large workload
    let sources: Vec<&str> = (0..200)
        .map(|_| ".perf-test { color: red; margin: 10px; padding: 20px; display: flex; }")
        .collect();

    let config = default_config();

    // Warm up
    let _ = compile_multiple(&sources, &config);

    // Sequential timing
    let seq_start = Instant::now();
    let _sequential: Vec<_> = sources.iter().map(|s| compile(s, &config)).collect();
    let seq_duration = seq_start.elapsed();

    // Parallel timing
    let par_start = Instant::now();
    let _parallel = compile_multiple(&sources, &config);
    let par_duration = par_start.elapsed();

    // We don't assert strict speedup because CI environments vary,
    // but we verify parallel is not catastrophically slower (< 5x sequential).
    assert!(
        par_duration < seq_duration * 5,
        "Parallel ({:?}) should not be vastly slower than sequential ({:?})",
        par_duration,
        seq_duration,
    );

    // Print timings for informational purposes
    eprintln!(
        "Performance: sequential={:?}, parallel={:?}, ratio={:.2}x",
        seq_duration,
        par_duration,
        seq_duration.as_secs_f64() / par_duration.as_secs_f64(),
    );
}
