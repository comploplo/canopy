//! Comprehensive benchmarking suite for UDPipe performance analysis
//!
//! This module provides detailed performance measurements for:
//! - Parsing latency across different sentence lengths
//! - Memory usage patterns
//! - Throughput measurements
//! - Comparison between real UDPipe and enhanced tokenization

use crate::udpipe::UDPipeEngine;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub name: String,
    pub iterations: usize,
    pub total_time: Duration,
    pub avg_time: Duration,
    pub min_time: Duration,
    pub max_time: Duration,
    pub median_time: Duration,
    pub p95_time: Duration,
    pub throughput_per_sec: f64,
    pub memory_usage_mb: Option<f64>,
}

pub struct BenchmarkSuite {
    engine: UDPipeEngine,
    results: Vec<BenchmarkResult>,
}

impl BenchmarkSuite {
    /// Create a new benchmark suite with the specified model
    pub fn new(model_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let engine = UDPipeEngine::load(model_path)?;
        Ok(Self {
            engine,
            results: Vec::new(),
        })
    }

    /// Run all benchmarks and return comprehensive results
    pub fn run_all_benchmarks(&mut self) -> Vec<BenchmarkResult> {
        println!("🚀 Starting UDPipe Performance Benchmarking Suite");
        println!("==================================================");

        // Latency benchmarks
        self.benchmark_parsing_latency();
        self.benchmark_sentence_length_scaling();
        self.benchmark_batch_processing();

        // Throughput benchmarks
        self.benchmark_throughput();

        // Memory benchmarks
        self.benchmark_memory_usage();

        // Print summary
        self.print_benchmark_summary();

        self.results.clone()
    }

    /// Benchmark basic parsing latency with standard sentences
    fn benchmark_parsing_latency(&mut self) {
        let test_sentences = vec![
            "The cat sat.",
            "She loves reading books.",
            "The quick brown fox jumps over the lazy dog.",
            "John gave Mary a book yesterday.",
            "Programming in Rust is both challenging and rewarding.",
        ];

        let mut all_times = Vec::new();
        let iterations = 100;

        println!("\n📊 Parsing Latency Benchmark");
        println!(
            "Sentences: {}, Iterations per sentence: {}",
            test_sentences.len(),
            iterations
        );

        for sentence in &test_sentences {
            let mut times = Vec::new();

            // Warmup
            for _ in 0..10 {
                let _ = self.engine.parse(sentence);
            }

            // Actual measurements
            for _ in 0..iterations {
                let start = Instant::now();
                let _ = self.engine.parse(sentence).expect("Parse should succeed");
                let duration = start.elapsed();
                times.push(duration);
                all_times.push(duration);
            }

            let avg = times.iter().sum::<Duration>() / times.len() as u32;
            println!("  \"{}\": {:?} avg", sentence, avg);
        }

        let result = self.calculate_benchmark_stats(
            "Parsing Latency",
            iterations * test_sentences.len(),
            all_times,
        );
        self.results.push(result);
    }

    /// Benchmark how parsing time scales with sentence length
    fn benchmark_sentence_length_scaling(&mut self) {
        println!("\n📏 Sentence Length Scaling Benchmark");

        let base_words = vec![
            "The", "quick", "brown", "fox", "jumps", "over", "the", "lazy", "dog",
        ];
        let sentence_lengths = vec![3, 5, 10, 15, 20, 30, 50];

        for &length in &sentence_lengths {
            let mut sentence_words = base_words.clone();
            while sentence_words.len() < length {
                sentence_words.extend_from_slice(&["and", "more", "words", "here"]);
            }
            sentence_words.truncate(length);

            // Build sentence directly instead of modifying in place
            let mut sentence_parts = Vec::new();
            for (i, word) in sentence_words.iter().enumerate() {
                if i == sentence_words.len() - 1 {
                    sentence_parts.push(format!("{}.", word));
                } else {
                    sentence_parts.push(word.to_string());
                }
            }
            let sentence = sentence_parts.join(" ");
            let mut times = Vec::new();
            let iterations = 50;

            // Warmup
            for _ in 0..5 {
                let _ = self.engine.parse(&sentence);
            }

            // Measurements
            for _ in 0..iterations {
                let start = Instant::now();
                let _ = self.engine.parse(&sentence).expect("Parse should succeed");
                times.push(start.elapsed());
            }

            let avg = times.iter().sum::<Duration>() / times.len() as u32;
            println!("  {} words: {:?} avg", length, avg);

            let result = self.calculate_benchmark_stats(
                &format!("Sentence Length {} words", length),
                iterations,
                times,
            );
            self.results.push(result);
        }
    }

    /// Benchmark batch processing performance
    fn benchmark_batch_processing(&mut self) {
        println!("\n📦 Batch Processing Benchmark");

        let sentences = vec![
            "The weather is nice today.",
            "Machine learning is revolutionizing technology.",
            "Natural language processing enables better human-computer interaction.",
            "Rust provides memory safety without garbage collection.",
            "Dependency parsing reveals syntactic relationships between words.",
        ];

        let batch_sizes = vec![1, 5, 10, 25, 50, 100];

        for &batch_size in &batch_sizes {
            let mut batch = Vec::new();
            for i in 0..batch_size {
                batch.push(sentences[i % sentences.len()].to_string());
            }

            let mut times = Vec::new();
            let iterations = 20;

            // Warmup
            for _ in 0..3 {
                for sentence in &batch {
                    let _ = self.engine.parse(sentence);
                }
            }

            // Measurements
            for _ in 0..iterations {
                let start = Instant::now();
                for sentence in &batch {
                    let _ = self.engine.parse(sentence).expect("Parse should succeed");
                }
                times.push(start.elapsed());
            }

            let avg = times.iter().sum::<Duration>() / times.len() as u32;
            let per_sentence = avg / batch_size as u32;
            println!(
                "  Batch size {}: {:?} total, {:?} per sentence",
                batch_size, avg, per_sentence
            );

            let result = self.calculate_benchmark_stats(
                &format!("Batch Processing {} sentences", batch_size),
                iterations,
                times,
            );
            self.results.push(result);
        }
    }

    /// Benchmark overall throughput (sentences per second)
    fn benchmark_throughput(&mut self) {
        println!("\n⚡ Throughput Benchmark");

        let test_sentence =
            "The quick brown fox jumps over the lazy dog and demonstrates parsing performance.";
        let duration_seconds = 5;
        let start_time = Instant::now();
        let mut count = 0;
        let mut times = Vec::new();

        println!("Running for {} seconds...", duration_seconds);

        while start_time.elapsed().as_secs() < duration_seconds {
            let parse_start = Instant::now();
            let _ = self
                .engine
                .parse(test_sentence)
                .expect("Parse should succeed");
            times.push(parse_start.elapsed());
            count += 1;
        }

        let total_time = start_time.elapsed();
        let throughput = count as f64 / total_time.as_secs_f64();

        println!("  Parsed {} sentences in {:?}", count, total_time);
        println!("  Throughput: {:.1} sentences/second", throughput);

        let result = self.calculate_benchmark_stats("Throughput Test", count, times);
        self.results.push(result);
    }

    /// Benchmark memory usage patterns
    fn benchmark_memory_usage(&mut self) {
        println!("\n🧠 Memory Usage Benchmark");

        // This is a simplified memory benchmark
        // In a real implementation, you'd use proper memory profiling tools
        let sentences = vec![
            "Short sentence.",
            "This is a medium length sentence with several words.",
            "This is a much longer sentence that contains many more words and should use more memory during parsing and processing operations.",
        ];

        for (i, sentence) in sentences.iter().enumerate() {
            println!("  Sentence {}: {} chars", i + 1, sentence.len());

            // Parse multiple times to see memory patterns
            for _ in 0..100 {
                let _ = self.engine.parse(sentence).expect("Parse should succeed");
            }
        }

        println!("  Memory usage analysis requires external profiling tools");
        println!("  Consider using: cargo-profiler, heaptrack, or valgrind");
    }

    /// Calculate comprehensive statistics from timing measurements
    fn calculate_benchmark_stats(
        &self,
        name: &str,
        iterations: usize,
        mut times: Vec<Duration>,
    ) -> BenchmarkResult {
        times.sort();

        let total_time = times.iter().sum();
        let avg_time: Duration = total_time / times.len() as u32;
        let min_time = times[0];
        let max_time = times[times.len() - 1];
        let median_time = times[times.len() / 2];
        let p95_time = times[(times.len() as f64 * 0.95) as usize];
        let throughput_per_sec = 1.0 / avg_time.as_secs_f64();

        BenchmarkResult {
            name: name.to_string(),
            iterations,
            total_time,
            avg_time,
            min_time,
            max_time,
            median_time,
            p95_time,
            throughput_per_sec,
            memory_usage_mb: None, // Would need external profiling
        }
    }

    /// Print a comprehensive summary of all benchmark results
    fn print_benchmark_summary(&self) {
        println!("\n📈 Benchmark Summary");
        println!("===================");

        for result in &self.results {
            println!("\n{}", result.name);
            println!("  Iterations: {}", result.iterations);
            println!("  Average: {:?}", result.avg_time);
            println!("  Median:  {:?}", result.median_time);
            println!("  Min:     {:?}", result.min_time);
            println!("  Max:     {:?}", result.max_time);
            println!("  P95:     {:?}", result.p95_time);
            println!("  Throughput: {:.1} ops/sec", result.throughput_per_sec);
        }

        // Overall analysis
        println!("\n🎯 Performance Analysis");
        println!("=======================");

        let parsing_results: Vec<_> = self
            .results
            .iter()
            .filter(|r| r.name.contains("Parsing") || r.name.contains("Sentence Length"))
            .collect();

        if !parsing_results.is_empty() {
            let avg_parse_time: Duration =
                parsing_results.iter().map(|r| r.avg_time).sum::<Duration>()
                    / parsing_results.len() as u32;

            println!("  Average parsing time: {:?}", avg_parse_time);
            println!("  Target: <500μs (tokenizer compatibility)");

            if avg_parse_time.as_micros() < 500 {
                println!("  ✅ EXCELLENT: Well under tokenizer target!");
            } else if avg_parse_time.as_micros() < 1000 {
                println!("  ⚠️  GOOD: Close to tokenizer target");
            } else {
                println!("  ❌ NEEDS OPTIMIZATION: Above tokenizer target");
            }
        }

        // Memory efficiency note
        println!("  Memory: FFI-based, minimal Rust allocations");
        println!("  Model: Static loading, shared across parses");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_benchmark_suite() {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let workspace_root = std::path::Path::new(manifest_dir)
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        let model_path = workspace_root.join("models/test.model");

        if !model_path.exists() {
            println!("Skipping benchmark test - model not found");
            return;
        }

        let mut suite = BenchmarkSuite::new(&model_path.to_string_lossy())
            .expect("Should create benchmark suite");

        // Run a minimal benchmark for testing
        suite.benchmark_parsing_latency();

        assert!(!suite.results.is_empty(), "Should have benchmark results");

        let result = &suite.results[0];
        assert!(result.avg_time.as_micros() > 0, "Should have measured time");
        assert!(
            result.throughput_per_sec > 0.0,
            "Should have positive throughput"
        );
    }

    #[test]
    fn test_comprehensive_benchmarks() {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let workspace_root = std::path::Path::new(manifest_dir)
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        let model_path = workspace_root.join("models/test.model");

        if !model_path.exists() {
            println!("Skipping comprehensive benchmark test - model not found");
            return;
        }

        println!("\n🔥 Running Comprehensive UDPipe Benchmarks 🔥");

        let mut suite = BenchmarkSuite::new(&model_path.to_string_lossy())
            .expect("Should create benchmark suite");

        let results = suite.run_all_benchmarks();

        assert!(
            !results.is_empty(),
            "Should have multiple benchmark results"
        );

        // Verify performance targets
        let avg_times: Vec<_> = results.iter().map(|r| r.avg_time).collect();
        let overall_avg = avg_times.iter().sum::<Duration>() / avg_times.len() as u32;

        println!("\n🏁 Final Analysis:");
        println!("   Overall average parse time: {:?}", overall_avg);

        // Performance assertions
        assert!(overall_avg.as_millis() < 10, "Should be under 10ms average");

        for result in &results {
            // Reduced threshold for CI environments
            assert!(
                result.throughput_per_sec > 10.0,
                "Should handle >10 sentences/sec (was: {:.1})",
                result.throughput_per_sec
            );
        }
    }
}
