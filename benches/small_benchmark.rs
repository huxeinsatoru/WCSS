use std::fs;
use std::time::Instant;
use euis_compiler::config::CompilerConfig;

fn main() {
    let input = fs::read_to_string("small_test.euis")
        .expect("Failed to read file");
    
    let config = CompilerConfig::default();
    
    println!("Small Euis test: {} bytes, 2 rules", input.len());
    
    // Single run first
    let start = Instant::now();
    let result = euis_compiler::compile(&input, &config);
    let single_time = start.elapsed();
    
    println!("Single compile: {}μs", single_time.as_micros());
    println!("Output: {} bytes", result.css.len());
    
    // Warm benchmark
    let iterations = 1000;
    let start = Instant::now();
    
    for _ in 0..iterations {
        let _ = euis_compiler::compile(&input, &config);
    }
    
    let elapsed = start.elapsed();
    let avg_micros = elapsed.as_micros() / iterations;
    
    println!("Warm average: {}μs", avg_micros);
}
