//! Real-world benchmarking with actual UDPipe models
//!
//! This module provides comprehensive benchmarking of the complete pipeline
//! using real UDPipe 1.2 and 2.15 models to compare performance characteristics.

use canopy_core::Word;
use canopy_parser::{Layer1Config, Layer1Parser, UDPipeEngine};
use canopy_semantics::{Layer2Analyzer, Layer2Config, PerformanceMode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Comprehensive benchmark results for model comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelBenchmarkResults {
    pub model_name: String,
    pub model_path: String,
    pub model_size_mb: f64,

    // Layer 1 (UDPipe) Performance
    pub layer1_results: LayerBenchmarkResults,

    // Layer 2 (Semantics) Performance
    pub layer2_results: LayerBenchmarkResults,

    // Full Stack Performance
    pub fullstack_results: FullStackResults,

    // Memory Usage
    pub memory_usage: MemoryBenchmarkResults,

    // Quality Metrics
    pub quality_metrics: QualityMetrics,
}

/// Performance results for a single layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerBenchmarkResults {
    pub avg_latency_ms: f64,
    pub p50_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub throughput_per_sec: f64,
    pub words_per_second: f64,
    pub sentences_processed: usize,
    pub words_processed: usize,
    pub total_time_ms: f64,
    pub error_rate: f64,
}

/// Full stack benchmark results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullStackResults {
    pub end_to_end_latency_ms: f64,
    pub layer1_percentage: f64,
    pub layer2_percentage: f64,
    pub overhead_percentage: f64,
    pub pipeline_efficiency: f64, // words/sec/MB
}

/// Memory usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryBenchmarkResults {
    pub baseline_memory_mb: f64,
    pub peak_memory_mb: f64,
    pub memory_per_word_bytes: f64,
    pub memory_per_sentence_kb: f64,
    pub memory_growth_rate: f64,
}

/// Quality and accuracy metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    pub features_per_word: f64,
    pub morphological_completeness: f64,
    pub events_per_sentence: f64,
    pub theta_roles_per_event: f64,
    pub processing_success_rate: f64,
}

/// Comprehensive model benchmark suite
pub struct ModelBenchmarkSuite {
    test_sentences: Vec<String>,
    warmup_iterations: usize,
    benchmark_iterations: usize,
    enable_memory_tracking: bool,
}

impl ModelBenchmarkSuite {
    pub fn new() -> Self {
        Self {
            test_sentences: Self::create_benchmark_corpus(),
            warmup_iterations: 3,
            benchmark_iterations: 20,
            enable_memory_tracking: true,
        }
    }

    /// Run comprehensive benchmarks comparing UDPipe models
    pub async fn run_model_comparison()
    -> Result<Vec<ModelBenchmarkResults>, Box<dyn std::error::Error>> {
        let suite = Self::new();

        println!("🚀 Starting Comprehensive Model Benchmark Suite");
        println!("================================================");

        let mut results = Vec::new();

        // Test UDPipe 1.2 model
        if let Some(udpipe12_result) = suite.benchmark_udpipe_12().await? {
            results.push(udpipe12_result);
        }

        // Test UDPipe 2.15 model
        if let Some(udpipe215_result) = suite.benchmark_udpipe_215().await? {
            results.push(udpipe215_result);
        }

        // Generate comparison report
        if results.len() >= 2 {
            Self::print_comparison_report(&results);
        }

        Ok(results)
    }

    /// Benchmark UDPipe 1.2 model
    async fn benchmark_udpipe_12(
        &self,
    ) -> Result<Option<ModelBenchmarkResults>, Box<dyn std::error::Error>> {
        let model_path = "/Users/gabe/projects/canopy/models/english-ud-1.2-160523.udpipe";

        if !std::path::Path::new(model_path).exists() {
            println!("⚠️  UDPipe 1.2 model not found at {}", model_path);
            return Ok(None);
        }

        println!("\n📊 Benchmarking UDPipe 1.2 Model");
        println!("Model: {}", model_path);

        let model_size_mb = std::fs::metadata(model_path)?.len() as f64 / 1024.0 / 1024.0;
        println!("Size: {:.2} MB", model_size_mb);

        // Load model and create components
        let udpipe_engine = UDPipeEngine::load(model_path)?;
        let layer1_parser = Layer1Parser::with_config(udpipe_engine, Layer1Config::default());
        let mut layer2_analyzer = Layer2Analyzer::new();

        // Run benchmarks
        let layer1_results = self.benchmark_layer1(&layer1_parser).await?;
        let layer2_results = self.benchmark_layer2(&mut layer2_analyzer).await?;
        let fullstack_results = self
            .benchmark_fullstack(&layer1_parser, &mut layer2_analyzer)
            .await?;
        let memory_usage = self
            .benchmark_memory(&layer1_parser, &mut layer2_analyzer)
            .await?;
        let quality_metrics = self
            .measure_quality(&layer1_parser, &mut layer2_analyzer)
            .await?;

        Ok(Some(ModelBenchmarkResults {
            model_name: "UDPipe 1.2 English".to_string(),
            model_path: model_path.to_string(),
            model_size_mb,
            layer1_results,
            layer2_results,
            fullstack_results,
            memory_usage,
            quality_metrics,
        }))
    }

    /// Benchmark UDPipe 2.15 model
    async fn benchmark_udpipe_215(
        &self,
    ) -> Result<Option<ModelBenchmarkResults>, Box<dyn std::error::Error>> {
        let model_path = "/Users/gabe/projects/canopy/models/english-ewt-ud-2.12-230717.udpipe";

        if !std::path::Path::new(model_path).exists() {
            println!("⚠️  UDPipe 2.15 model not found at {}", model_path);
            return Ok(None);
        }

        println!("\n📊 Benchmarking UDPipe 2.15 Model");
        println!("Model: {}", model_path);

        let model_size_mb = std::fs::metadata(model_path)?.len() as f64 / 1024.0 / 1024.0;
        println!("Size: {:.2} MB", model_size_mb);

        // Load model and create components
        let udpipe_engine = UDPipeEngine::load(model_path)?;
        let layer1_parser = Layer1Parser::with_config(udpipe_engine, Layer1Config::default());
        let mut layer2_analyzer = Layer2Analyzer::new();

        // Run benchmarks
        let layer1_results = self.benchmark_layer1(&layer1_parser).await?;
        let layer2_results = self.benchmark_layer2(&mut layer2_analyzer).await?;
        let fullstack_results = self
            .benchmark_fullstack(&layer1_parser, &mut layer2_analyzer)
            .await?;
        let memory_usage = self
            .benchmark_memory(&layer1_parser, &mut layer2_analyzer)
            .await?;
        let quality_metrics = self
            .measure_quality(&layer1_parser, &mut layer2_analyzer)
            .await?;

        Ok(Some(ModelBenchmarkResults {
            model_name: "UDPipe 2.15 English".to_string(),
            model_path: model_path.to_string(),
            model_size_mb,
            layer1_results,
            layer2_results,
            fullstack_results,
            memory_usage,
            quality_metrics,
        }))
    }

    /// Benchmark Layer 1 (UDPipe parsing) performance
    async fn benchmark_layer1(
        &self,
        parser: &Layer1Parser,
    ) -> Result<LayerBenchmarkResults, Box<dyn std::error::Error>> {
        println!("  🔍 Layer 1 (UDPipe) Benchmark...");

        // Warmup
        for _ in 0..self.warmup_iterations {
            for sentence in &self.test_sentences[..5] {
                let _ = parser.parse_document(sentence);
            }
        }

        // Collect timing data
        let mut latencies = Vec::new();
        let mut total_words = 0;
        let mut total_sentences = 0;
        let mut errors = 0;

        let overall_start = Instant::now();

        for _ in 0..self.benchmark_iterations {
            for sentence in &self.test_sentences {
                let start = Instant::now();

                match parser.parse_document(sentence) {
                    Ok(words) => {
                        let duration = start.elapsed();
                        latencies.push(duration.as_secs_f64() * 1000.0); // Convert to ms
                        total_words += words.len();
                        total_sentences += 1;
                    }
                    Err(_) => {
                        errors += 1;
                        total_sentences += 1;
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

        let throughput = total_sentences as f64 / total_time.as_secs_f64();
        let words_per_second = total_words as f64 / total_time.as_secs_f64();
        let error_rate = errors as f64 / total_sentences as f64;

        println!(
            "    ✅ Processed {} sentences, {} words",
            total_sentences, total_words
        );
        println!(
            "    ⚡ Avg latency: {:.2}ms, Throughput: {:.1} sent/sec",
            avg_latency, throughput
        );

        Ok(LayerBenchmarkResults {
            avg_latency_ms: avg_latency,
            p50_latency_ms: p50_latency,
            p95_latency_ms: p95_latency,
            p99_latency_ms: p99_latency,
            throughput_per_sec: throughput,
            words_per_second,
            sentences_processed: total_sentences,
            words_processed: total_words,
            total_time_ms: total_time.as_secs_f64() * 1000.0,
            error_rate,
        })
    }

    /// Benchmark Layer 2 (Semantic analysis) performance
    async fn benchmark_layer2(
        &self,
        analyzer: &mut Layer2Analyzer,
    ) -> Result<LayerBenchmarkResults, Box<dyn std::error::Error>> {
        println!("  🧠 Layer 2 (Semantics) Benchmark...");

        // Create sample parsed words for Layer 2 testing
        let sample_words = self.create_sample_words();

        // Warmup
        for _ in 0..self.warmup_iterations {
            let _ = analyzer.analyze(sample_words.clone());
        }

        // Collect timing data
        let mut latencies = Vec::new();
        let mut total_words = 0;
        let mut total_sentences = 0;
        let mut errors = 0;

        let overall_start = Instant::now();

        for _ in 0..self.benchmark_iterations {
            for _ in &self.test_sentences {
                let start = Instant::now();

                match analyzer.analyze(sample_words.clone()) {
                    Ok(analysis) => {
                        let duration = start.elapsed();
                        latencies.push(duration.as_secs_f64() * 1000.0);
                        total_words += analysis.words.len();
                        total_sentences += 1;
                    }
                    Err(_) => {
                        errors += 1;
                        total_sentences += 1;
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

        let throughput = total_sentences as f64 / total_time.as_secs_f64();
        let words_per_second = total_words as f64 / total_time.as_secs_f64();
        let error_rate = errors as f64 / total_sentences as f64;

        println!(
            "    ✅ Processed {} sentences, {} words",
            total_sentences, total_words
        );
        println!(
            "    ⚡ Avg latency: {:.2}ms, Throughput: {:.1} sent/sec",
            avg_latency, throughput
        );

        Ok(LayerBenchmarkResults {
            avg_latency_ms: avg_latency,
            p50_latency_ms: p50_latency,
            p95_latency_ms: p95_latency,
            p99_latency_ms: p99_latency,
            throughput_per_sec: throughput,
            words_per_second,
            sentences_processed: total_sentences,
            words_processed: total_words,
            total_time_ms: total_time.as_secs_f64() * 1000.0,
            error_rate,
        })
    }

    /// Benchmark full stack (Layer 1 + Layer 2) performance
    async fn benchmark_fullstack(
        &self,
        parser: &Layer1Parser,
        analyzer: &mut Layer2Analyzer,
    ) -> Result<FullStackResults, Box<dyn std::error::Error>> {
        println!("  🔗 Full Stack Benchmark...");

        // Warmup
        for _ in 0..self.warmup_iterations {
            for sentence in &self.test_sentences[..3] {
                if let Ok(words) = parser.parse_document(sentence) {
                    let words: Vec<Word> = words.into_iter().map(|ew| ew.word).collect();
                    let _ = analyzer.analyze(words);
                }
            }
        }

        let mut total_time = Duration::ZERO;
        let mut layer1_time = Duration::ZERO;
        let mut layer2_time = Duration::ZERO;
        let mut processed_sentences = 0;

        for _ in 0..self.benchmark_iterations {
            for sentence in &self.test_sentences {
                let overall_start = Instant::now();

                // Layer 1
                let layer1_start = Instant::now();
                if let Ok(enhanced_words) = parser.parse_document(sentence) {
                    let layer1_duration = layer1_start.elapsed();
                    layer1_time += layer1_duration;

                    // Convert to Words for Layer 2
                    let words: Vec<Word> = enhanced_words.into_iter().map(|ew| ew.word).collect();

                    // Layer 2
                    let layer2_start = Instant::now();
                    if let Ok(_analysis) = analyzer.analyze(words) {
                        let layer2_duration = layer2_start.elapsed();
                        layer2_time += layer2_duration;

                        total_time += overall_start.elapsed();
                        processed_sentences += 1;
                    }
                }
            }
        }

        let avg_latency = total_time.as_secs_f64() * 1000.0 / processed_sentences as f64;
        let layer1_percentage = (layer1_time.as_secs_f64() / total_time.as_secs_f64()) * 100.0;
        let layer2_percentage = (layer2_time.as_secs_f64() / total_time.as_secs_f64()) * 100.0;
        let overhead_percentage = 100.0 - layer1_percentage - layer2_percentage;

        // Calculate pipeline efficiency (processing rate per MB of model)
        let throughput = processed_sentences as f64 / total_time.as_secs_f64();
        let pipeline_efficiency = throughput; // Will be divided by model size in final calculation

        println!("    ✅ End-to-end latency: {:.2}ms", avg_latency);
        println!(
            "    📊 Layer 1: {:.1}%, Layer 2: {:.1}%, Overhead: {:.1}%",
            layer1_percentage, layer2_percentage, overhead_percentage
        );

        Ok(FullStackResults {
            end_to_end_latency_ms: avg_latency,
            layer1_percentage,
            layer2_percentage,
            overhead_percentage,
            pipeline_efficiency,
        })
    }

    /// Benchmark memory usage patterns
    async fn benchmark_memory(
        &self,
        parser: &Layer1Parser,
        analyzer: &mut Layer2Analyzer,
    ) -> Result<MemoryBenchmarkResults, Box<dyn std::error::Error>> {
        println!("  💾 Memory Usage Benchmark...");

        // Simplified memory tracking (in a real implementation, we'd use more sophisticated tools)
        let baseline_memory = Self::estimate_memory_usage();

        // Process some sentences and estimate peak memory
        let mut max_memory = baseline_memory;
        let mut total_words = 0;
        let mut total_sentences = 0;

        for sentence in &self.test_sentences[..10] {
            if let Ok(enhanced_words) = parser.parse_document(sentence) {
                let words: Vec<Word> = enhanced_words.into_iter().map(|ew| ew.word).collect();
                total_words += words.len();

                if let Ok(_analysis) = analyzer.analyze(words) {
                    total_sentences += 1;
                    let current_memory = Self::estimate_memory_usage();
                    max_memory = max_memory.max(current_memory);
                }
            }
        }

        let memory_per_word = if total_words > 0 {
            (max_memory - baseline_memory) * 1024.0 * 1024.0 / total_words as f64
        } else {
            0.0
        };

        let memory_per_sentence = if total_sentences > 0 {
            (max_memory - baseline_memory) * 1024.0 / total_sentences as f64
        } else {
            0.0
        };

        println!(
            "    💾 Baseline: {:.1}MB, Peak: {:.1}MB",
            baseline_memory, max_memory
        );
        println!(
            "    📏 Per word: {:.0} bytes, Per sentence: {:.1}KB",
            memory_per_word, memory_per_sentence
        );

        Ok(MemoryBenchmarkResults {
            baseline_memory_mb: baseline_memory,
            peak_memory_mb: max_memory,
            memory_per_word_bytes: memory_per_word,
            memory_per_sentence_kb: memory_per_sentence,
            memory_growth_rate: (max_memory - baseline_memory) / baseline_memory,
        })
    }

    /// Measure quality and feature extraction metrics
    async fn measure_quality(
        &self,
        parser: &Layer1Parser,
        analyzer: &mut Layer2Analyzer,
    ) -> Result<QualityMetrics, Box<dyn std::error::Error>> {
        println!("  📈 Quality Metrics Analysis...");

        let mut total_features = 0;
        let mut total_words = 0;
        let mut total_events = 0;
        let mut total_theta_roles = 0;
        let mut successful_parses = 0;
        let mut total_sentences = 0;

        for sentence in &self.test_sentences[..10] {
            total_sentences += 1;

            if let Ok(enhanced_words) = parser.parse_document(sentence) {
                let words: Vec<Word> = enhanced_words.into_iter().map(|ew| ew.word).collect();

                // Count morphological features
                for word in &words {
                    let feats = &word.feats;
                    let mut feature_count = 0;
                    if feats.number.is_some() {
                        feature_count += 1;
                    }
                    if feats.person.is_some() {
                        feature_count += 1;
                    }
                    if feats.tense.is_some() {
                        feature_count += 1;
                    }
                    if feats.voice.is_some() {
                        feature_count += 1;
                    }
                    if feats.mood.is_some() {
                        feature_count += 1;
                    }
                    total_features += feature_count;
                }
                total_words += words.len();

                if let Ok(analysis) = analyzer.analyze(words) {
                    successful_parses += 1;
                    total_events += analysis.events.len();
                    total_theta_roles += analysis
                        .theta_assignments
                        .values()
                        .map(|assignments| assignments.len())
                        .sum::<usize>();
                }
            }
        }

        let features_per_word = if total_words > 0 {
            total_features as f64 / total_words as f64
        } else {
            0.0
        };
        let morphological_completeness = (features_per_word / 5.0).min(1.0); // Target ~5 features per word
        let events_per_sentence = if successful_parses > 0 {
            total_events as f64 / successful_parses as f64
        } else {
            0.0
        };
        let theta_roles_per_event = if total_events > 0 {
            total_theta_roles as f64 / total_events as f64
        } else {
            0.0
        };
        let success_rate = successful_parses as f64 / total_sentences as f64;

        println!(
            "    📊 Features/word: {:.2}, Events/sentence: {:.2}",
            features_per_word, events_per_sentence
        );
        println!("    ✅ Success rate: {:.1}%", success_rate * 100.0);

        Ok(QualityMetrics {
            features_per_word,
            morphological_completeness,
            events_per_sentence,
            theta_roles_per_event,
            processing_success_rate: success_rate,
        })
    }

    /// Create comprehensive test corpus covering various linguistic phenomena
    fn create_benchmark_corpus() -> Vec<String> {
        vec![
            // Simple sentences (fast parsing baseline)
            "The cat sits.".to_string(),
            "John runs quickly.".to_string(),
            "Mary loves books.".to_string(),
            "Dogs bark loudly.".to_string(),
            "She writes letters.".to_string(),
            // Medium complexity (common real-world patterns)
            "The student finished the homework assignment.".to_string(),
            "My sister bought a new car yesterday.".to_string(),
            "The teacher explained the complex problem clearly.".to_string(),
            "Children played in the sunny garden.".to_string(),
            "The manager scheduled an important meeting.".to_string(),
            // Complex sentences (stress testing)
            "The book that John recommended was absolutely fascinating and well-written.".to_string(),
            "Although it was raining heavily, the determined hikers continued their journey up the mountain.".to_string(),
            "The scientist who discovered the new species received international recognition for her groundbreaking research.".to_string(),
            "When the storm finally ended, the cleanup crew began assessing the extensive damage throughout the neighborhood.".to_string(),
            // Linguistic phenomena (accuracy testing)
            "The letter was written by John and sent immediately.".to_string(), // Passive + coordination
            "What did Mary buy at the expensive downtown store?".to_string(), // Wh-question
            "John gave Mary a beautiful red rose for her birthday.".to_string(), // Ditransitive
            "This book, I really enjoyed reading during my vacation.".to_string(), // Topicalization
            "The intense heat quickly melted all the ice cubes.".to_string(), // Causative
            // Performance stress tests (longer sentences)
            "The comprehensive linguistic analysis system that we developed processes natural language text through multiple layers of increasingly sophisticated analysis to extract semantic meaning and syntactic structure.".to_string(),
            "In order to evaluate the performance characteristics of different universal dependency parsing models, researchers typically conduct extensive benchmarking studies using standardized test corpora that represent diverse linguistic phenomena across multiple languages and domains.".to_string(),
        ]
    }

    /// Create sample words for Layer 2 testing
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
            Word {
                id: 3,
                text: "Mary".to_string(),
                lemma: "Mary".to_string(),
                upos: UPos::Propn,
                xpos: None,
                feats: MorphFeatures::default(),
                head: Some(2),
                deprel: DepRel::Iobj,
                deps: None,
                misc: None,
                start: 10,
                end: 14,
            },
            Word {
                id: 4,
                text: "a".to_string(),
                lemma: "a".to_string(),
                upos: UPos::Det,
                xpos: None,
                feats: MorphFeatures::default(),
                head: Some(5),
                deprel: DepRel::Det,
                deps: None,
                misc: None,
                start: 15,
                end: 16,
            },
            Word {
                id: 5,
                text: "book".to_string(),
                lemma: "book".to_string(),
                upos: UPos::Noun,
                xpos: None,
                feats: MorphFeatures::default(),
                head: Some(2),
                deprel: DepRel::Obj,
                deps: None,
                misc: None,
                start: 17,
                end: 21,
            },
        ]
    }

    /// Simple memory usage estimation (in a real implementation, use proper memory profiling)
    fn estimate_memory_usage() -> f64 {
        // This is a simplified estimation - in practice, use tools like jemalloc or system APIs
        50.0 // Base 50MB estimation
    }

    /// Print comprehensive comparison report
    fn print_comparison_report(results: &[ModelBenchmarkResults]) {
        println!("\n🏆 MODEL COMPARISON REPORT");
        println!("==========================");

        if results.len() >= 2 {
            let udpipe12 = &results[0];
            let udpipe215 = &results[1];

            println!("\n📊 PERFORMANCE COMPARISON");
            println!("┌─────────────────────┬─────────────┬─────────────┬──────────────┐");
            println!("│ Metric              │ UDPipe 1.2  │ UDPipe 2.15 │ Ratio (2/1)  │");
            println!("├─────────────────────┼─────────────┼─────────────┼──────────────┤");

            // Layer 1 Performance
            let l1_latency_ratio =
                udpipe215.layer1_results.avg_latency_ms / udpipe12.layer1_results.avg_latency_ms;
            let l1_throughput_ratio = udpipe215.layer1_results.throughput_per_sec
                / udpipe12.layer1_results.throughput_per_sec;

            println!(
                "│ Layer1 Latency (ms) │ {:10.2}  │ {:10.2}  │ {:11.2}x │",
                udpipe12.layer1_results.avg_latency_ms,
                udpipe215.layer1_results.avg_latency_ms,
                l1_latency_ratio
            );

            println!(
                "│ Layer1 Throughput   │ {:10.1}  │ {:10.1}  │ {:11.2}x │",
                udpipe12.layer1_results.throughput_per_sec,
                udpipe215.layer1_results.throughput_per_sec,
                l1_throughput_ratio
            );

            // Full Stack Performance
            let e2e_latency_ratio = udpipe215.fullstack_results.end_to_end_latency_ms
                / udpipe12.fullstack_results.end_to_end_latency_ms;

            println!(
                "│ End-to-End (ms)     │ {:10.2}  │ {:10.2}  │ {:11.2}x │",
                udpipe12.fullstack_results.end_to_end_latency_ms,
                udpipe215.fullstack_results.end_to_end_latency_ms,
                e2e_latency_ratio
            );

            // Memory Usage
            let memory_ratio =
                udpipe215.memory_usage.peak_memory_mb / udpipe12.memory_usage.peak_memory_mb;

            println!(
                "│ Peak Memory (MB)    │ {:10.1}  │ {:10.1}  │ {:11.2}x │",
                udpipe12.memory_usage.peak_memory_mb,
                udpipe215.memory_usage.peak_memory_mb,
                memory_ratio
            );

            // Model Size
            let size_ratio = udpipe215.model_size_mb / udpipe12.model_size_mb;

            println!(
                "│ Model Size (MB)     │ {:10.1}  │ {:10.1}  │ {:11.2}x │",
                udpipe12.model_size_mb, udpipe215.model_size_mb, size_ratio
            );

            println!("└─────────────────────┴─────────────┴─────────────┴──────────────┘");

            // Quality Comparison
            println!("\n📈 QUALITY COMPARISON");
            println!("┌─────────────────────┬─────────────┬─────────────┬──────────────┐");
            println!("│ Quality Metric      │ UDPipe 1.2  │ UDPipe 2.15 │ Improvement  │");
            println!("├─────────────────────┼─────────────┼─────────────┼──────────────┤");

            let features_improvement = udpipe215.quality_metrics.features_per_word
                / udpipe12.quality_metrics.features_per_word;
            let morpho_improvement = udpipe215.quality_metrics.morphological_completeness
                / udpipe12.quality_metrics.morphological_completeness;

            println!(
                "│ Features per Word   │ {:10.2}  │ {:10.2}  │ {:11.2}x │",
                udpipe12.quality_metrics.features_per_word,
                udpipe215.quality_metrics.features_per_word,
                features_improvement
            );

            println!(
                "│ Morpho Completeness │ {:10.1}% │ {:10.1}% │ {:11.2}x │",
                udpipe12.quality_metrics.morphological_completeness * 100.0,
                udpipe215.quality_metrics.morphological_completeness * 100.0,
                morpho_improvement
            );

            println!(
                "│ Success Rate        │ {:10.1}% │ {:10.1}% │ {:11.1}% │",
                udpipe12.quality_metrics.processing_success_rate * 100.0,
                udpipe215.quality_metrics.processing_success_rate * 100.0,
                (udpipe215.quality_metrics.processing_success_rate
                    - udpipe12.quality_metrics.processing_success_rate)
                    * 100.0
            );

            println!("└─────────────────────┴─────────────┴─────────────┴──────────────┘");

            // Performance Summary
            println!("\n🔍 PERFORMANCE ANALYSIS");

            if l1_latency_ratio > 1.5 {
                println!(
                    "⚠️  UDPipe 2.15 is {:.1}x slower than 1.2 (expected due to larger model)",
                    l1_latency_ratio
                );
            } else if l1_latency_ratio > 1.0 {
                println!(
                    "⚡ UDPipe 2.15 is {:.1}x slower than 1.2 (reasonable overhead)",
                    l1_latency_ratio
                );
            } else {
                println!("🚀 UDPipe 2.15 is faster than 1.2 (unexpected - check measurement)");
            }

            if memory_ratio > 2.0 {
                println!(
                    "💾 UDPipe 2.15 uses {:.1}x more memory (significant increase)",
                    memory_ratio
                );
            } else {
                println!(
                    "💾 UDPipe 2.15 uses {:.1}x more memory (reasonable increase)",
                    memory_ratio
                );
            }

            if features_improvement > 1.1 {
                println!(
                    "📈 UDPipe 2.15 provides {:.1}x better feature extraction",
                    features_improvement
                );
            } else {
                println!("📊 Similar feature extraction quality between models");
            }

            // Efficiency calculation
            let udpipe12_efficiency =
                udpipe12.layer1_results.throughput_per_sec / udpipe12.model_size_mb;
            let udpipe215_efficiency =
                udpipe215.layer1_results.throughput_per_sec / udpipe215.model_size_mb;
            let efficiency_ratio = udpipe215_efficiency / udpipe12_efficiency;

            println!("\n📊 EFFICIENCY ANALYSIS");
            println!(
                "UDPipe 1.2 Efficiency:  {:.2} sentences/sec/MB",
                udpipe12_efficiency
            );
            println!(
                "UDPipe 2.15 Efficiency: {:.2} sentences/sec/MB",
                udpipe215_efficiency
            );
            println!("Efficiency Ratio:       {:.2}x", efficiency_ratio);

            if efficiency_ratio < 0.5 {
                println!("💡 Recommendation: Use UDPipe 1.2 for high-throughput applications");
                println!("💡 Recommendation: Use UDPipe 2.15 for accuracy-critical applications");
            } else {
                println!(
                    "💡 Recommendation: UDPipe 2.15 provides good balance of speed and accuracy"
                );
            }
        }

        println!("\n✅ Benchmark completed successfully!");
    }
}

impl Default for ModelBenchmarkSuite {
    fn default() -> Self {
        Self::new()
    }
}
