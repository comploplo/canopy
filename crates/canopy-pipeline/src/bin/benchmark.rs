//! Benchmark runner for Canopy Pipeline
//!
//! This executable runs comprehensive benchmarks comparing UDPipe 1.2 and 2.15 models
//! across Layer 1, Layer 2, and full stack performance.

use canopy_pipeline::real_benchmarks::ModelBenchmarkSuite;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 Canopy Pipeline Benchmark Suite");
    println!("===================================");
    println!("Running comprehensive performance analysis...\n");

    // Run the benchmark suite
    match ModelBenchmarkSuite::run_model_comparison().await {
        Ok(results) => {
            println!("\n✅ Benchmark suite completed successfully!");
            println!("Collected results for {} models", results.len());

            // Save results to JSON for further analysis
            if let Ok(json) = serde_json::to_string_pretty(&results) {
                std::fs::write("benchmark_results.json", json)?;
                println!("📊 Results saved to benchmark_results.json");
            }
        }
        Err(e) => {
            eprintln!("❌ Benchmark failed: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}
