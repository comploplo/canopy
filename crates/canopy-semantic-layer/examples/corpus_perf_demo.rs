//! Layer 1 Corpus Performance Demo
//!
//! Benchmarks Layer 1 semantic analysis performance with a larger corpus of text
//! to demonstrate throughput and latency characteristics.

use canopy_pipeline::create_l1_analyzer;
use std::error::Error;
use std::fs;
use std::io::{self, Write};
use std::time::Instant;

fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    println!("⚡ Semantic Analysis Performance Demo");
    println!("===================================");

    let analyzer = create_l1_analyzer()?;
    let stats = analyzer.get_statistics();
    println!("✅ Engines loaded: {:?}\n", stats.active_engines);

    // Load Moby Dick corpus for realistic performance testing
    println!("📖 Loading Moby Dick corpus...");
    let corpus_text = fs::read_to_string("data/test-corpus/mobydick.txt")?;

    // Extract meaningful words (skip short words, punctuation, numbers)
    let words: Vec<&str> = corpus_text
        .split_whitespace()
        .filter(|word| {
            word.len() >= 3
                && word.chars().all(|c| c.is_alphabetic())
                && !word.chars().all(|c| c.is_uppercase()) // Skip chapter headers
        })
        // Full Moby Dick corpus - approximately 85,000 valid words
        .collect();

    println!(
        "📊 Full Corpus Performance Analysis ({} words)",
        words.len()
    );
    println!("{}", "=".repeat(50));

    // Estimate runtime based on observed performance characteristics
    let estimated_seconds = words.len() as f64 / 930.0; // ~930 words/sec observed
    println!(
        "⏱️  Estimated runtime: {:.0} seconds ({:.1} minutes)",
        estimated_seconds,
        estimated_seconds / 60.0
    );
    println!();

    // Main performance test
    println!(
        "🚀 Running performance analysis on {} words...",
        words.len()
    );
    print!("   🔄 Progress: ");

    let start_total = Instant::now();
    let mut individual_times = Vec::new();
    let mut successful_analyses = 0;
    let mut total_semantic_units = 0;

    for (i, word) in words.iter().enumerate() {
        let start = Instant::now();
        match analyzer.analyze(word) {
            Ok(result) => {
                let duration = start.elapsed();
                individual_times.push(duration.as_micros());
                successful_analyses += 1;

                // Count semantic units
                let vn_count = result
                    .verbnet
                    .as_ref()
                    .map(|v| v.verb_classes.len())
                    .unwrap_or(0);
                let fn_count = result
                    .framenet
                    .as_ref()
                    .map(|f| f.frames.len())
                    .unwrap_or(0);
                let wn_count = result
                    .wordnet
                    .as_ref()
                    .map(|w| w.synsets.len())
                    .unwrap_or(0);
                total_semantic_units += vn_count + fn_count + wn_count;

                // Clean progress indicator - show progress every 1000 words
                if (i + 1) % 1000 == 0 {
                    print!("█");
                    io::stdout().flush().unwrap();
                }
            }
            Err(_) => {
                // Count errors but don't spam output
                if (i + 1) % 1000 == 0 {
                    print!("▓");
                    io::stdout().flush().unwrap();
                }
            }
        }
    }

    let total_duration = start_total.elapsed();
    println!(" ✅ Complete");
    println!();

    // Performance statistics
    println!("📈 Performance Results");
    println!("{}", "=".repeat(25));

    let total_words = words.len();
    let throughput = (total_words as f64) / total_duration.as_secs_f64();

    let actual_seconds = total_duration.as_secs_f64();
    println!(
        "🏁 Total time: {:.1} seconds ({:.2} minutes)",
        actual_seconds,
        actual_seconds / 60.0
    );
    println!(
        "📝 Words processed: {}/{}",
        successful_analyses, total_words
    );
    println!("⚡ Throughput: {:.0} words/second", throughput);
    println!("📊 Semantic units found: {}", total_semantic_units);

    // Compare actual vs estimated runtime
    let accuracy = if estimated_seconds > 0.0 {
        (1.0 - (actual_seconds - estimated_seconds).abs() / estimated_seconds) * 100.0
    } else {
        0.0
    };
    println!(
        "🎯 Runtime accuracy: {:.1}% (estimated: {:.0}s, actual: {:.1}s)",
        accuracy, estimated_seconds, actual_seconds
    );

    if !individual_times.is_empty() {
        individual_times.sort();
        let min_time = individual_times[0];
        let max_time = individual_times[individual_times.len() - 1];
        let median_time = individual_times[individual_times.len() / 2];
        let avg_time: f64 =
            individual_times.iter().map(|&x| x as f64).sum::<f64>() / individual_times.len() as f64;

        println!("\n⏱️  Latency Statistics (per word)");
        println!("   Min:    {:.1}μs", min_time);
        println!("   Median: {:.1}μs", median_time);
        println!("   Average:{:.1}μs", avg_time);
        println!("   Max:    {:.1}μs", max_time);
    }

    // Engine-specific statistics
    println!("\n🔧 Engine Statistics");
    println!(
        "   Success rate: {:.1}%",
        (successful_analyses as f64 / total_words as f64) * 100.0
    );
    println!(
        "   Avg semantic units per word: {:.1}",
        total_semantic_units as f64 / successful_analyses as f64
    );

    println!("\n🎉 Performance demo complete!");

    Ok(())
}
