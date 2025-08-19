//! Performance benchmark demonstration
//!
//! This example shows how to use the performance benchmarking system
//! to compare different UDPipe model configurations.

use canopy_semantics::PerformanceBenchmark;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a benchmark with reduced iterations for demo
    let benchmark = PerformanceBenchmark::new()
        .with_warmup_iterations(1)
        .with_benchmark_iterations(5);

    println!("🚀 Running UDPipe Performance Benchmarks");
    println!("=========================================");

    // Run benchmarks for all performance modes
    let results = benchmark.benchmark_udpipe_models()?;

    // Display results
    for result in &results {
        println!("\n📊 Model: {}", result.model_name);
        println!("   Sentences processed: {}", result.sentences_processed);
        println!("   Words processed: {}", result.words_processed);
        println!(
            "   Total time: {:.2}ms",
            result.total_time.as_secs_f64() * 1000.0
        );
        println!(
            "   Avg per sentence: {:.2}ms",
            result.avg_time_per_sentence.as_secs_f64() * 1000.0
        );
        println!(
            "   Avg per word: {:.2}ms",
            result.avg_time_per_word.as_secs_f64() * 1000.0
        );
        println!("   Sentences/sec: {:.1}", result.sentences_per_second);
        println!("   Words/sec: {:.1}", result.words_per_second);

        println!("   📈 Component Breakdown:");
        println!(
            "     - Theta assignment: {}ms",
            result.component_breakdown.theta_assignment_ms
        );
        println!(
            "     - Event construction: {}ms",
            result.component_breakdown.event_construction_ms
        );
        println!(
            "     - Total breakdown: {}ms",
            result.component_breakdown.total_time_ms()
        );

        println!("   💾 Memory:");
        println!(
            "     - Peak memory: {:.2}MB",
            result.memory_stats.peak_memory_mb
        );
        println!(
            "     - Per sentence: {:.2}KB",
            result.memory_stats.memory_per_sentence_kb
        );
    }

    // Compare performance modes
    println!("\n🔬 Performance Mode Comparison");
    println!("==============================");

    if results.len() >= 3 {
        let speed_result = &results[0]; // UDPipe-Speed
        let balanced_result = &results[1]; // UDPipe-Balanced
        let accuracy_result = &results[2]; // UDPipe-Accuracy

        println!("Speed vs Accuracy:");
        let speed_ratio =
            accuracy_result.total_time.as_secs_f64() / speed_result.total_time.as_secs_f64();
        println!(
            "  Accuracy mode is {:.2}x slower than Speed mode",
            speed_ratio
        );

        println!("Balanced Performance:");
        let balanced_vs_speed =
            balanced_result.total_time.as_secs_f64() / speed_result.total_time.as_secs_f64();
        let balanced_vs_accuracy =
            accuracy_result.total_time.as_secs_f64() / balanced_result.total_time.as_secs_f64();
        println!("  Balanced is {:.2}x slower than Speed", balanced_vs_speed);
        println!(
            "  Accuracy is {:.2}x slower than Balanced",
            balanced_vs_accuracy
        );
    }

    println!("\n✅ Benchmark completed successfully!");

    Ok(())
}
