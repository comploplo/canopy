//! UDPipe model comparison benchmark
//!
//! Compares performance between UDPipe 1.2 and 2.15 models using real parsing.

// use canopy_core::Word; // Unused in current benchmark implementation
use canopy_parser::{Layer1Config, Layer1Parser, UDPipeEngine};
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 UDPipe Model Performance Comparison");
    println!("======================================");

    let test_sentences = vec![
        "The cat sits.",
        "John runs quickly.",
        "Mary loves books.",
        "Dogs bark loudly.",
        "She writes letters.",
        "The student finished the homework assignment.",
        "My sister bought a new car yesterday.",
        "The teacher explained the complex problem clearly.",
        "Children played in the sunny garden.",
        "The manager scheduled an important meeting.",
        "The book that John recommended was fascinating.",
        "Although it was raining heavily, the hikers continued.",
        "The scientist who discovered the species received recognition.",
        "When the storm ended, cleanup crews began work.",
        "The letter was written by John and sent immediately.",
        "What did Mary buy at the expensive store?",
        "John gave Mary a beautiful rose for her birthday.",
        "This book, I really enjoyed reading during vacation.",
        "The intense heat quickly melted all the ice cubes.",
        "The comprehensive analysis system processes multiple layers.",
    ];

    let mut results = Vec::new();

    // Benchmark UDPipe 1.2
    if let Some(result) = benchmark_model(
        "/Users/gabe/projects/canopy/models/udpipe-ud-1.2-160523/english-ud-1.2-160523.udpipe",
        "UDPipe 1.2",
        &test_sentences,
    )? {
        results.push(result);
    }

    // Benchmark UDPipe 2.15
    if let Some(result) = benchmark_model(
        "/Users/gabe/projects/canopy/models/udpipe2-ud-2.15-241121/en_all-ud-2.15-241121.model",
        "UDPipe 2.15",
        &test_sentences,
    )? {
        results.push(result);
    }

    // Print comparison
    if results.len() >= 2 {
        print_comparison(&results[0], &results[1]);
    }

    println!("\n✅ Benchmark completed successfully!");
    Ok(())
}

fn benchmark_model(
    model_path: &str,
    model_name: &str,
    test_sentences: &[&str],
) -> Result<Option<ModelResult>, Box<dyn std::error::Error>> {
    if !std::path::Path::new(model_path).exists() {
        println!("⚠️  Model not found: {}", model_path);
        return Ok(None);
    }

    let model_size_mb = std::fs::metadata(model_path)?.len() as f64 / 1024.0 / 1024.0;
    println!("\n📊 Benchmarking {} ({:.1} MB)", model_name, model_size_mb);
    println!("Model: {}", model_path);

    // Load model
    let load_start = Instant::now();
    let udpipe_engine = UDPipeEngine::load(model_path)?;
    let parser = Layer1Parser::with_config(udpipe_engine, Layer1Config::default());
    let load_time = load_start.elapsed();
    println!("  ⚡ Model loaded in {:.3}s", load_time.as_secs_f64());

    // Warmup
    println!("  🔥 Warming up...");
    for _ in 0..3 {
        for sentence in &test_sentences[..5] {
            let _ = parser.parse_document(sentence);
        }
    }

    // Benchmark
    println!("  ⚡ Running benchmark...");
    let mut latencies = Vec::new();
    let mut total_words = 0;
    let mut successful_parses = 0;
    let mut errors = 0;

    let iterations = 20;
    let overall_start = Instant::now();

    for _ in 0..iterations {
        for sentence in test_sentences {
            let start = Instant::now();

            match parser.parse_document(sentence) {
                Ok(enhanced_words) => {
                    let duration = start.elapsed();
                    latencies.push(duration.as_secs_f64() * 1000.0); // Convert to ms

                    total_words += enhanced_words.len();
                    successful_parses += 1;
                }
                Err(_) => {
                    errors += 1;
                }
            }
        }
    }

    let total_time = overall_start.elapsed();

    // Calculate statistics
    latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let avg_latency = latencies.iter().sum::<f64>() / latencies.len() as f64;
    let p50_latency = latencies[latencies.len() / 2];
    let p95_latency = latencies[(latencies.len() as f64 * 0.95) as usize];
    let p99_latency = latencies[(latencies.len() as f64 * 0.99) as usize];

    let throughput = successful_parses as f64 / total_time.as_secs_f64();
    let words_per_second = total_words as f64 / total_time.as_secs_f64();
    let error_rate = errors as f64 / (successful_parses + errors) as f64;
    let efficiency = throughput / model_size_mb;

    println!(
        "    ✅ Processed {} sentences, {} words",
        successful_parses, total_words
    );
    println!(
        "    ⚡ Avg latency: {:.2}ms (P50: {:.2}ms, P95: {:.2}ms, P99: {:.2}ms)",
        avg_latency, p50_latency, p95_latency, p99_latency
    );
    println!("    📈 Throughput: {:.1} sent/sec", throughput);
    println!("    📝 Words/sec: {:.0}", words_per_second);
    println!("    ⚖️  Efficiency: {:.2} sent/sec/MB", efficiency);
    println!("    ❌ Error rate: {:.1}%", error_rate * 100.0);

    Ok(Some(ModelResult {
        name: model_name.to_string(),
        model_size_mb,
        load_time_sec: load_time.as_secs_f64(),
        avg_latency_ms: avg_latency,
        p50_latency_ms: p50_latency,
        p95_latency_ms: p95_latency,
        p99_latency_ms: p99_latency,
        throughput_per_sec: throughput,
        words_per_second,
        efficiency,
        error_rate,
        sentences_processed: successful_parses,
        words_processed: total_words,
    }))
}

fn print_comparison(udpipe12: &ModelResult, udpipe215: &ModelResult) {
    println!("\n🏆 MODEL COMPARISON REPORT");
    println!("==========================");

    println!("\n📊 PERFORMANCE COMPARISON");
    println!("┌─────────────────────┬─────────────┬─────────────┬──────────────┐");
    println!("│ Metric              │ UDPipe 1.2  │ UDPipe 2.15 │ Ratio (2/1)  │");
    println!("├─────────────────────┼─────────────┼─────────────┼──────────────┤");

    let latency_ratio = udpipe215.avg_latency_ms / udpipe12.avg_latency_ms;
    let throughput_ratio = udpipe215.throughput_per_sec / udpipe12.throughput_per_sec;
    let size_ratio = udpipe215.model_size_mb / udpipe12.model_size_mb;
    let load_ratio = udpipe215.load_time_sec / udpipe12.load_time_sec;
    let efficiency_ratio = udpipe215.efficiency / udpipe12.efficiency;

    println!(
        "│ Avg Latency (ms)    │ {:10.2}  │ {:10.2}  │ {:11.2}x │",
        udpipe12.avg_latency_ms, udpipe215.avg_latency_ms, latency_ratio
    );
    println!(
        "│ P95 Latency (ms)    │ {:10.2}  │ {:10.2}  │ {:11.2}x │",
        udpipe12.p95_latency_ms,
        udpipe215.p95_latency_ms,
        udpipe215.p95_latency_ms / udpipe12.p95_latency_ms
    );
    println!(
        "│ Throughput (sent/s) │ {:10.1}  │ {:10.1}  │ {:11.2}x │",
        udpipe12.throughput_per_sec, udpipe215.throughput_per_sec, throughput_ratio
    );
    println!(
        "│ Words/sec           │ {:10.0}  │ {:10.0}  │ {:11.2}x │",
        udpipe12.words_per_second,
        udpipe215.words_per_second,
        udpipe215.words_per_second / udpipe12.words_per_second
    );
    println!(
        "│ Model Size (MB)     │ {:10.1}  │ {:10.1}  │ {:11.2}x │",
        udpipe12.model_size_mb, udpipe215.model_size_mb, size_ratio
    );
    println!(
        "│ Load Time (sec)     │ {:10.3}  │ {:10.3}  │ {:11.2}x │",
        udpipe12.load_time_sec, udpipe215.load_time_sec, load_ratio
    );
    println!(
        "│ Efficiency (s/s/MB) │ {:10.2}  │ {:10.2}  │ {:11.2}x │",
        udpipe12.efficiency, udpipe215.efficiency, efficiency_ratio
    );

    println!("└─────────────────────┴─────────────┴─────────────┴──────────────┘");

    println!("\n🔍 PERFORMANCE ANALYSIS");

    if latency_ratio > 2.0 {
        println!(
            "⚠️  UDPipe 2.15 is significantly slower ({:.1}x) - model complexity trade-off",
            latency_ratio
        );
    } else if latency_ratio > 1.0 {
        println!(
            "⚡ UDPipe 2.15 has reasonable overhead ({:.1}x slower)",
            latency_ratio
        );
    } else {
        println!("🚀 UDPipe 2.15 is faster than 1.2 (unexpected - check measurement)");
    }

    if size_ratio > 3.0 {
        println!(
            "💾 UDPipe 2.15 is much larger ({:.1}x) - significant memory impact",
            size_ratio
        );
    } else {
        println!(
            "💾 UDPipe 2.15 size increase is reasonable ({:.1}x)",
            size_ratio
        );
    }

    if load_ratio > 2.0 {
        println!(
            "🐌 UDPipe 2.15 takes significantly longer to load ({:.1}x)",
            load_ratio
        );
        println!("💡 Consider model caching/preloading for production systems");
    }

    if efficiency_ratio > 0.7 {
        println!("✅ UDPipe 2.15 maintains good efficiency despite larger size");
    } else {
        println!("⚠️  UDPipe 2.15 efficiency is significantly lower - consider use case");
    }

    println!("\n💡 RECOMMENDATIONS");
    if efficiency_ratio < 0.5 {
        println!("• Use UDPipe 1.2 for high-throughput, latency-sensitive applications");
        println!("• Use UDPipe 2.15 for accuracy-critical applications");
        println!(
            "• Consider hybrid approach: UDPipe 1.2 for preprocessing, 2.15 for critical analysis"
        );
    } else {
        println!("• UDPipe 2.15 provides good balance of accuracy and performance");
        println!("• Model choice depends on specific accuracy requirements");
    }

    if load_ratio > 2.0 {
        println!("• Implement model caching for UDPipe 2.15 in production");
        println!("• Consider warming up models during application startup");
    }
}

#[derive(Debug)]
#[allow(dead_code)] // Benchmark results struct for future performance analysis
struct ModelResult {
    name: String,
    model_size_mb: f64,
    load_time_sec: f64,
    avg_latency_ms: f64,
    #[allow(dead_code)]
    p50_latency_ms: f64,
    p95_latency_ms: f64,
    #[allow(dead_code)]
    p99_latency_ms: f64,
    throughput_per_sec: f64,
    words_per_second: f64,
    efficiency: f64,
    #[allow(dead_code)]
    error_rate: f64,
    #[allow(dead_code)]
    sentences_processed: usize,
    #[allow(dead_code)]
    words_processed: usize,
}
