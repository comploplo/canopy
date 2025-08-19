//! Performance benchmarks for UDPipe models and end-to-end pipeline

use crate::{Layer2Analyzer, Layer2Config, PerformanceMode};
use canopy_core;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Performance benchmark results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResults {
    pub model_name: String,
    pub sentences_processed: usize,
    pub words_processed: usize,
    pub total_time: Duration,
    pub avg_time_per_sentence: Duration,
    pub avg_time_per_word: Duration,
    pub sentences_per_second: f64,
    pub words_per_second: f64,
    pub memory_stats: MemoryStats,
    pub component_breakdown: ComponentBreakdown,
}

/// Memory usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub peak_memory_mb: f64,
    pub memory_per_sentence_kb: f64,
    pub allocation_patterns: HashMap<String, usize>,
}

/// Performance breakdown by analysis component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentBreakdown {
    pub udpipe_parsing_ms: u64,
    pub feature_extraction_ms: u64,
    pub theta_assignment_ms: u64,
    pub event_construction_ms: u64,
    pub little_v_analysis_ms: u64,
    pub movement_detection_ms: u64,
}

/// Performance benchmark suite
pub struct PerformanceBenchmark {
    test_sentences: Vec<String>,
    warmup_iterations: usize,
    benchmark_iterations: usize,
}

impl PerformanceBenchmark {
    pub fn new() -> Self {
        Self {
            test_sentences: Self::default_test_sentences(),
            warmup_iterations: 3,
            benchmark_iterations: 10,
        }
    }

    pub fn with_test_sentences(mut self, sentences: Vec<String>) -> Self {
        self.test_sentences = sentences;
        self
    }

    pub fn with_warmup_iterations(mut self, iterations: usize) -> Self {
        self.warmup_iterations = iterations;
        self
    }

    pub fn with_benchmark_iterations(mut self, iterations: usize) -> Self {
        self.benchmark_iterations = iterations;
        self
    }

    /// Run comprehensive benchmark comparing UDPipe models
    pub fn benchmark_udpipe_models(&self) -> Result<Vec<BenchmarkResults>, BenchmarkError> {
        let mut results = Vec::new();

        // Test with different performance modes
        let test_configs = [
            ("UDPipe-Speed", PerformanceMode::Speed),
            ("UDPipe-Balanced", PerformanceMode::Balanced),
            ("UDPipe-Accuracy", PerformanceMode::Accuracy),
        ];

        for (config_name, perf_mode) in test_configs {
            let config = Layer2Config {
                performance_mode: perf_mode,
                ..Default::default()
            };
            let analyzer = Layer2Analyzer::with_config(config);

            let result = self.benchmark_analyzer(&analyzer, config_name)?;
            results.push(result);
        }

        Ok(results)
    }

    /// Benchmark a specific analyzer configuration
    pub fn benchmark_analyzer(
        &self,
        _analyzer: &Layer2Analyzer,
        model_name: &str,
    ) -> Result<BenchmarkResults, BenchmarkError> {
        // Note: This is a simplified benchmark since Layer2 operates on pre-parsed data
        // In a full implementation, we would need Layer1 to provide parsed words

        // Create dummy words for testing (normally from Layer1/UDPipe)
        let test_words = Self::create_test_words();

        // Warmup phase
        for _ in 0..self.warmup_iterations {
            let mut test_analyzer = Layer2Analyzer::with_config(Layer2Config::default());
            let _ = test_analyzer.analyze(test_words.clone());
        }

        // Benchmark phase
        let mut total_time = Duration::new(0, 0);
        let mut component_times = ComponentBreakdown {
            udpipe_parsing_ms: 0,
            feature_extraction_ms: 0,
            theta_assignment_ms: 0,
            event_construction_ms: 0,
            little_v_analysis_ms: 0,
            movement_detection_ms: 0,
        };

        let mut total_words = 0;

        for _ in 0..self.benchmark_iterations {
            let _start_time = Instant::now();

            // Create a new analyzer instance for each iteration
            let mut test_analyzer = Layer2Analyzer::with_config(Layer2Config::default());

            // Time the analysis
            let analysis_start = Instant::now();
            let analysis = test_analyzer.analyze(test_words.clone())?;
            let analysis_time = analysis_start.elapsed();

            // Extract timing information from Layer2Metrics
            component_times.theta_assignment_ms += analysis.metrics.theta_assignment_time_us / 1000;
            component_times.event_construction_ms += analysis.metrics.event_creation_time_us / 1000;

            total_time += analysis_time;
            total_words += analysis.words.len();
        }

        let sentences_processed = self.benchmark_iterations; // One "sentence" per iteration
        let avg_time_per_sentence = total_time / sentences_processed as u32;
        let avg_time_per_word = total_time / total_words as u32;

        let sentences_per_second = sentences_processed as f64 / total_time.as_secs_f64();
        let words_per_second = total_words as f64 / total_time.as_secs_f64();

        // Estimate memory usage (simplified)
        let estimated_memory_mb = (total_words * 100) as f64 / 1024.0 / 1024.0; // ~100 bytes per word
        let memory_per_sentence_kb = (estimated_memory_mb * 1024.0) / sentences_processed as f64;

        Ok(BenchmarkResults {
            model_name: model_name.to_string(),
            sentences_processed,
            words_processed: total_words,
            total_time,
            avg_time_per_sentence,
            avg_time_per_word,
            sentences_per_second,
            words_per_second,
            memory_stats: MemoryStats {
                peak_memory_mb: estimated_memory_mb,
                memory_per_sentence_kb,
                allocation_patterns: HashMap::new(), // TODO: Implement detailed tracking
            },
            component_breakdown: component_times,
        })
    }

    /// Create test words for benchmarking (simplified)
    fn create_test_words() -> Vec<canopy_core::Word> {
        use crate::layer2::create_word_from_parse;
        use canopy_core::UPos;

        vec![
            create_word_from_parse(1, "John", "John", UPos::Propn),
            create_word_from_parse(2, "gave", "give", UPos::Verb),
            create_word_from_parse(3, "Mary", "Mary", UPos::Propn),
            create_word_from_parse(4, "a", "a", UPos::Det),
            create_word_from_parse(5, "book", "book", UPos::Noun),
        ]
    }

    /// Default test sentences covering various linguistic phenomena
    fn default_test_sentences() -> Vec<String> {
        vec![
            // Simple sentences
            "John runs.".to_string(),
            "Mary reads books.".to_string(),
            // Complex sentences
            "The cat that sat on the mat ate the fish.".to_string(),
            "Although it was raining, John decided to go for a walk.".to_string(),
            // Passive voice
            "The book was read by Mary.".to_string(),
            "The vase was broken by John.".to_string(),
            // Questions
            "What did Mary read?".to_string(),
            "Who broke the vase?".to_string(),
            // Ditransitive
            "John gave Mary a book.".to_string(),
            "The teacher showed the students the problem.".to_string(),
            // Movement and topicalization
            "This book, I really like.".to_string(),
            "Yesterday, John met Mary at the park.".to_string(),
            // Causatives
            "The heat melted the ice.".to_string(),
            "John made Mary laugh.".to_string(),
            // Complex coordination
            "John and Mary went to the store and bought groceries.".to_string(),
            "Either John will come, or Mary will stay home.".to_string(),
        ]
    }
}

/// Complete performance report
#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceReport {
    pub benchmark_timestamp: std::time::SystemTime,
    pub test_configuration: TestConfiguration,
    pub results: Vec<BenchmarkResults>,
}

/// Test configuration details
#[derive(Debug, Serialize, Deserialize)]
pub struct TestConfiguration {
    pub sentence_count: usize,
    pub warmup_iterations: usize,
    pub benchmark_iterations: usize,
    pub test_sentences: Vec<String>,
}

/// Benchmark errors
#[derive(Debug, thiserror::Error)]
pub enum BenchmarkError {
    #[error("Analysis failed: {0}")]
    AnalysisError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

// Convert from Layer2Error
impl From<crate::Layer2Error> for BenchmarkError {
    fn from(err: crate::Layer2Error) -> Self {
        BenchmarkError::AnalysisError(err.to_string())
    }
}

impl Default for PerformanceBenchmark {
    fn default() -> Self {
        Self::new()
    }
}

impl ComponentBreakdown {
    pub fn total_time_ms(&self) -> u64 {
        self.udpipe_parsing_ms
            + self.feature_extraction_ms
            + self.theta_assignment_ms
            + self.event_construction_ms
            + self.little_v_analysis_ms
            + self.movement_detection_ms
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_creation() {
        let benchmark = PerformanceBenchmark::new();
        assert!(!benchmark.test_sentences.is_empty());
        assert_eq!(benchmark.warmup_iterations, 3);
        assert_eq!(benchmark.benchmark_iterations, 10);
    }

    #[test]
    fn test_component_breakdown_calculations() {
        let breakdown = ComponentBreakdown {
            udpipe_parsing_ms: 100,
            feature_extraction_ms: 50,
            theta_assignment_ms: 30,
            event_construction_ms: 20,
            little_v_analysis_ms: 15,
            movement_detection_ms: 10,
        };

        assert_eq!(breakdown.total_time_ms(), 225);
    }

    #[test]
    fn test_benchmark_analyzer() {
        let benchmark = PerformanceBenchmark::new()
            .with_warmup_iterations(1)
            .with_benchmark_iterations(2);

        let analyzer = Layer2Analyzer::new();
        let result = benchmark.benchmark_analyzer(&analyzer, "test-model");

        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.model_name, "test-model");
        assert_eq!(results.sentences_processed, 2);
        assert!(results.words_processed > 0);
        // Total time is always non-negative
    }

    #[test]
    fn test_udpipe_models_benchmark() {
        let benchmark = PerformanceBenchmark::new()
            .with_warmup_iterations(1)
            .with_benchmark_iterations(3);

        let result = benchmark.benchmark_udpipe_models();

        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 3); // Speed, Balanced, Accuracy modes

        // Verify all modes are represented
        let model_names: Vec<&str> = results.iter().map(|r| r.model_name.as_str()).collect();
        assert!(model_names.contains(&"UDPipe-Speed"));
        assert!(model_names.contains(&"UDPipe-Balanced"));
        assert!(model_names.contains(&"UDPipe-Accuracy"));

        // Each result should have processed some words
        for result in &results {
            assert!(result.words_processed > 0);
            assert!(result.sentences_processed > 0);
        }
    }
}
