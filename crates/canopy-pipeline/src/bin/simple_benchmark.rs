//! Simple benchmark runner for UDPipe models
//!
//! This executable runs direct benchmarks without the complex API to get performance numbers.

use canopy_core::Word;
use canopy_parser::{Layer1Config, Layer1Parser, UDPipeEngine};
use canopy_semantics::Layer2Analyzer;
use std::time::{Duration, Instant};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Canopy Direct Model Benchmark");
    println!("=================================");

    let test_sentences = vec![
        "The cat sits.",
        "John runs quickly.",
        "Mary loves books.",
        "The student finished the homework assignment.",
        "My sister bought a new car yesterday.",
        "The book that John recommended was absolutely fascinating.",
    ];

    // Benchmark UDPipe 1.2
    if let Some(results_12) = benchmark_model(
        "/Users/gabe/projects/canopy/models/english-ud-1.2-160523.udpipe",
        "UDPipe 1.2",
        &test_sentences,
    )
    .await?
    {
        println!("\n📊 UDPipe 1.2 Results:");
        print_results(&results_12);
    }

    // Benchmark UDPipe 2.15
    if let Some(results_215) = benchmark_model(
        "/Users/gabe/projects/canopy/models/english-ewt-ud-2.12-230717.udpipe",
        "UDPipe 2.15",
        &test_sentences,
    )
    .await?
    {
        println!("\n📊 UDPipe 2.15 Results:");
        print_results(&results_215);
    }

    Ok(())
}

async fn benchmark_model(
    model_path: &str,
    model_name: &str,
    test_sentences: &[&str],
) -> Result<Option<BenchmarkResults>, Box<dyn std::error::Error>> {
    if !std::path::Path::new(model_path).exists() {
        println!("⚠️  Model not found: {}", model_path);
        return Ok(None);
    }

    let model_size_mb = std::fs::metadata(model_path)?.len() as f64 / 1024.0 / 1024.0;
    println!("\n🔍 Loading {} ({:.1} MB)...", model_name, model_size_mb);

    // Load model
    let udpipe_engine = UDPipeEngine::load(model_path)?;
    let layer1_parser = Layer1Parser::with_config(udpipe_engine, Layer1Config::default());
    let mut layer2_analyzer = Layer2Analyzer::new();

    // Warmup
    println!("🔥 Warming up...");
    for _ in 0..3 {
        for sentence in &test_sentences[..3] {
            let _ = layer1_parser.parse_document(sentence);
        }
    }

    // Benchmark Layer 1
    println!("⚡ Benchmarking Layer 1...");
    let mut layer1_times = Vec::new();
    let mut total_words = 0;
    let mut total_sentences = 0;

    let iterations = 20;
    let overall_start = Instant::now();

    for _ in 0..iterations {
        for sentence in test_sentences {
            let start = Instant::now();
            match layer1_parser.parse_document(sentence) {
                Ok(words) => {
                    let duration = start.elapsed();
                    layer1_times.push(duration.as_secs_f64() * 1000.0);
                    total_words += words.len();
                    total_sentences += 1;
                }
                Err(_) => {}
            }
        }
    }

    let total_time = overall_start.elapsed();

    // Benchmark Layer 2
    println!("🧠 Benchmarking Layer 2...");
    let sample_words = create_sample_words();
    let mut layer2_times = Vec::new();

    let layer2_start = Instant::now();
    for _ in 0..iterations {
        for _ in test_sentences {
            let start = Instant::now();
            if let Ok(_) = layer2_analyzer.analyze(sample_words.clone()) {
                let duration = start.elapsed();
                layer2_times.push(duration.as_secs_f64() * 1000.0);
            }
        }
    }
    let layer2_total = layer2_start.elapsed();

    // Calculate statistics
    layer1_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    layer2_times.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let results = BenchmarkResults {
        model_name: model_name.to_string(),
        model_size_mb,
        layer1_avg_ms: layer1_times.iter().sum::<f64>() / layer1_times.len() as f64,
        layer1_p95_ms: layer1_times[(layer1_times.len() as f64 * 0.95) as usize],
        layer1_throughput: total_sentences as f64 / total_time.as_secs_f64(),
        layer1_words_per_sec: total_words as f64 / total_time.as_secs_f64(),
        layer2_avg_ms: layer2_times.iter().sum::<f64>() / layer2_times.len() as f64,
        layer2_throughput: (layer2_times.len() as f64) / layer2_total.as_secs_f64(),
        sentences_processed: total_sentences,
        words_processed: total_words,
    };

    Ok(Some(results))
}

fn create_sample_words() -> Vec<Word> {
    use canopy_core::{DepRel, MorphFeatures, UPos};

    vec![
        Word {
            id: 1,
            text: "John".to_string(),
            lemma: "John".to_string(),
            upos: UPos::Propn,
            xpos: None,
            feats: MorphFeatures::default(),
            head: Some(2),
            deprel: DepRel::Nsubj,
            deps: None,
            misc: None,
            start: 0,
            end: 4,
        },
        Word {
            id: 2,
            text: "gave".to_string(),
            lemma: "give".to_string(),
            upos: UPos::Verb,
            xpos: None,
            feats: MorphFeatures::default(),
            head: Some(0),
            deprel: DepRel::Root,
            deps: None,
            misc: None,
            start: 5,
            end: 9,
        },
    ]
}

fn print_results(results: &BenchmarkResults) {
    println!(
        "  Model: {} ({:.1} MB)",
        results.model_name, results.model_size_mb
    );
    println!("  Layer 1:");
    println!("    Avg latency: {:.2}ms", results.layer1_avg_ms);
    println!("    P95 latency: {:.2}ms", results.layer1_p95_ms);
    println!("    Throughput: {:.1} sent/sec", results.layer1_throughput);
    println!("    Words/sec: {:.0}", results.layer1_words_per_sec);
    println!("  Layer 2:");
    println!("    Avg latency: {:.2}ms", results.layer2_avg_ms);
    println!("    Throughput: {:.1} sent/sec", results.layer2_throughput);
    println!(
        "  Total: {} sentences, {} words",
        results.sentences_processed, results.words_processed
    );

    let efficiency = results.layer1_throughput / results.model_size_mb;
    println!("  Efficiency: {:.2} sent/sec/MB", efficiency);
}

#[derive(Debug)]
struct BenchmarkResults {
    model_name: String,
    model_size_mb: f64,
    layer1_avg_ms: f64,
    layer1_p95_ms: f64,
    layer1_throughput: f64,
    layer1_words_per_sec: f64,
    layer2_avg_ms: f64,
    layer2_throughput: f64,
    sentences_processed: usize,
    words_processed: usize,
}
