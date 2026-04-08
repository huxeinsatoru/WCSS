use std::time::Instant;
use euis_compiler::config::CompilerConfig;

fn main() {
    // Create small input - single rule
    let input = ".button { padding: 1rem; background: #3b82f6; }";
    
    let config = CompilerConfig::default();
    
    println!("Small Euis test: {} bytes, 1 rule", input.len());
    
    // Single compile first
    let result = euis_compiler::compile(input, &config);
    println!("Output: '{}' ({} bytes)", result.css, result.css.len());
    println!("Errors: {:?}", result.errors);
    
    // Warm up
    for _ in 0..100 {
        let _ = euis_compiler::compile(input, &config);
    }
    
    // Benchmark
    let iterations = 10000;
    let start = Instant::now();
    
    for _ in 0..iterations {
        let _ = euis_compiler::compile(input, &config);
    }
    
    let elapsed = start.elapsed();
    let avg_micros = elapsed.as_micros() / iterations;
    
    println!("Warm average: {}μs ({} iterations)", avg_micros, iterations);
}
