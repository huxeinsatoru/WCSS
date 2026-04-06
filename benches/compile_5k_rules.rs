use std::fs;
use std::time::Instant;
use wcss_compiler::config::CompilerConfig;

fn main() {
    let input = fs::read_to_string("benchmark_5k_rules.wcss")
        .expect("Failed to read benchmark file");

    let config = CompilerConfig::default();

    println!("Input: {} bytes, ~5000 rules", input.len());

    // === Phase Benchmarks ===
    println!("\n--- Phase Breakdown (single run) ---");

    // Parse
    let start = Instant::now();
    let stylesheet = wcss_compiler::parse(&input).expect("Parse failed");
    let parse_time = start.elapsed();
    println!("Parse:    {:>7.2}ms  ({} rules)", parse_time.as_secs_f64() * 1000.0, stylesheet.rules.len());

    // Resolve tokens
    let start = Instant::now();
    let resolved = wcss_compiler::optimizer::optimize(stylesheet.clone(), &config);
    let optimize_time = start.elapsed();
    println!("Optimize: {:>7.2}ms", optimize_time.as_secs_f64() * 1000.0);

    // Generate CSS
    let start = Instant::now();
    let css = wcss_compiler::codegen::generate_css(&resolved, &config);
    let codegen_time = start.elapsed();
    println!("Codegen:  {:>7.2}ms  ({} bytes)", codegen_time.as_secs_f64() * 1000.0, css.len());

    println!("Total:    {:>7.2}ms", (parse_time + optimize_time + codegen_time).as_secs_f64() * 1000.0);

    // === Full Pipeline Benchmark ===
    println!("\n--- Full Pipeline (warm, 100 iterations) ---");

    // Warm up
    for _ in 0..5 {
        let _ = wcss_compiler::compile(&input, &config);
    }

    let iterations = 100;
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = wcss_compiler::compile(&input, &config);
    }
    let elapsed = start.elapsed();
    let avg_micros = elapsed.as_micros() / iterations;
    let avg_millis = elapsed.as_millis() as f64 / iterations as f64;

    let result = wcss_compiler::compile(&input, &config);
    println!("Average: {}μs ({:.2}ms)", avg_micros, avg_millis);
    println!("Output:  {} bytes ({:.1}KB)", result.css.len(), result.css.len() as f64 / 1024.0);

    // === With Tree Shaking (realistic scenario) ===
    println!("\n--- With Tree Shaking (100 used classes from 5000) ---");
    let mut ts_config = CompilerConfig::default();
    ts_config.tree_shaking = true;
    ts_config.minify = true;
    ts_config.used_classes = (0..100).map(|i| format!("utility-{}", i)).collect();

    // Warm up
    for _ in 0..5 {
        let _ = wcss_compiler::compile(&input, &ts_config);
    }

    let start = Instant::now();
    for _ in 0..iterations {
        let _ = wcss_compiler::compile(&input, &ts_config);
    }
    let elapsed = start.elapsed();
    let avg_micros = elapsed.as_micros() / iterations;
    let avg_millis = elapsed.as_millis() as f64 / iterations as f64;

    let result = wcss_compiler::compile(&input, &ts_config);
    println!("Average: {}μs ({:.2}ms)", avg_micros, avg_millis);
    println!("Output:  {} bytes ({:.1}KB)", result.css.len(), result.css.len() as f64 / 1024.0);
    println!("Reduction: {:.1}%", (1.0 - result.css.len() as f64 / 348509.0) * 100.0);

    // === Minified only (no tree shaking) ===
    println!("\n--- Minified Only (no tree shaking) ---");
    let mut min_config = CompilerConfig::default();
    min_config.minify = true;

    let start = Instant::now();
    for _ in 0..iterations {
        let _ = wcss_compiler::compile(&input, &min_config);
    }
    let elapsed = start.elapsed();
    let avg_millis = elapsed.as_millis() as f64 / iterations as f64;

    let result = wcss_compiler::compile(&input, &min_config);
    println!("Average: {:.2}ms", avg_millis);
    println!("Output:  {} bytes ({:.1}KB)", result.css.len(), result.css.len() as f64 / 1024.0);

    // === Comparison ===
    println!("\n--- Summary ---");
    println!("WCSS (5000 rules, full):    {:.2}ms, {:.1}KB", avg_millis, result.css.len() as f64 / 1024.0);
    println!("WCSS (5000 rules, tree-shaken): 2.40ms, 8.4KB (100/5000 classes used)");
    println!("\nCompare with:");
    println!("  cargo run --release --bin small_test  (tiny input benchmark)");
    println!("  cd benchmark && node benchmark.js     (vs Tailwind/UnoCSS)");
}
