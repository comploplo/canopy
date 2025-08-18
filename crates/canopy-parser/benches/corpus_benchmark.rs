//! Corpus benchmarking infrastructure for measuring parsing accuracy and performance
//!
//! This module provides tools for evaluating parser performance against standard
//! linguistic corpora like Penn Treebank and Universal Dependencies.

use canopy_parser::memory::{MemoryConfig, MemoryStats};
use canopy_parser::udpipe::{UDPipeEngine, UDPipeParser};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::fs;
use std::path::Path;
use std::time::Instant;

/// Corpus evaluation metrics
#[derive(Debug, Clone)]
pub struct CorpusMetrics {
    /// Total number of sentences processed
    pub sentences_processed: usize,

    /// Total number of words processed
    pub words_processed: usize,

    /// Average processing time per sentence (microseconds)
    pub avg_sentence_time_us: f64,

    /// Average processing time per word (microseconds)
    pub avg_word_time_us: f64,

    /// Memory usage statistics
    pub memory_stats: MemoryStats,

    /// Parser success rate (sentences parsed without error)
    pub success_rate: f64,
}

/// Simple corpus loader for benchmark data
pub struct CorpusLoader;

impl CorpusLoader {
    /// Load benchmark sentences from a text file (one sentence per line)
    pub fn load_sentences<P: AsRef<Path>>(path: P) -> Result<Vec<String>, std::io::Error> {
        let content = fs::read_to_string(path)?;
        let sentences: Vec<String> = content
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| line.trim().to_string())
            .collect();
        Ok(sentences)
    }

    /// Create a synthetic corpus for benchmarking when no real data is available
    pub fn create_synthetic_corpus(size: usize) -> Vec<String> {
        let base_sentences = [
            "The cat sat on the mat.",
            "She gave him a book yesterday.",
            "John loves reading books in the library.",
            "The weather is beautiful today in the park.",
            "Scientists discovered a new species of butterfly.",
            "The company announced record profits this quarter.",
            "Children played happily in the schoolyard during recess.",
            "The professor explained the complex theory to her students.",
            "We walked through the forest and saw many different animals.",
            "The musicians performed a beautiful symphony at the concert hall.",
        ];

        let mut corpus = Vec::new();
        for i in 0..size {
            let sentence = &base_sentences[i % base_sentences.len()];
            // Add some variation by appending sentence numbers
            corpus.push(format!("{} (Sentence {})", sentence, i + 1));
        }
        corpus
    }
}

/// Corpus evaluation engine
pub struct CorpusEvaluator {
    parser: UDPipeParser,
    memory_config: MemoryConfig,
}

impl CorpusEvaluator {
    /// Create a new corpus evaluator with the given parser
    pub fn new(parser: UDPipeParser) -> Self {
        Self {
            parser,
            memory_config: MemoryConfig::default(),
        }
    }

    /// Configure memory limits for evaluation
    pub fn with_memory_config(mut self, config: MemoryConfig) -> Self {
        self.memory_config = config;
        self
    }

    /// Evaluate parser performance on a corpus
    pub fn evaluate(&self, sentences: &[String]) -> CorpusMetrics {
        let start_time = Instant::now();
        let mut total_words = 0;
        let mut successful_sentences = 0;
        let mut sentence_times = Vec::new();

        for sentence in sentences {
            let sentence_start = Instant::now();

            match self.parser.parse_document(sentence) {
                Ok(parsed_doc) => {
                    successful_sentences += 1;
                    let word_count: usize =
                        parsed_doc.sentences.iter().map(|s| s.words.len()).sum();
                    total_words += word_count;
                }
                Err(_) => {
                    // Count as failed sentence
                }
            }

            let sentence_time = sentence_start.elapsed();
            sentence_times.push(sentence_time.as_micros() as f64);
        }

        let total_time = start_time.elapsed();
        let avg_sentence_time_us = sentence_times.iter().sum::<f64>() / sentences.len() as f64;
        let avg_word_time_us = if total_words > 0 {
            total_time.as_micros() as f64 / total_words as f64
        } else {
            0.0
        };

        CorpusMetrics {
            sentences_processed: sentences.len(),
            words_processed: total_words,
            avg_sentence_time_us,
            avg_word_time_us,
            memory_stats: MemoryStats::default(), // TODO: Implement actual memory tracking
            success_rate: successful_sentences as f64 / sentences.len() as f64,
        }
    }
}

/// Benchmark parsing performance on synthetic corpus
fn benchmark_corpus_performance(c: &mut Criterion) {
    let dummy_engine = UDPipeEngine::for_testing();
    let parser = UDPipeParser::new_with_engine(dummy_engine);
    let evaluator = CorpusEvaluator::new(parser);

    // Test different corpus sizes
    let corpus_sizes = vec![10, 50, 100, 500];

    let mut group = c.benchmark_group("corpus_performance");
    group.sample_size(10); // Fewer samples for large corpora

    for &size in &corpus_sizes {
        let corpus = CorpusLoader::create_synthetic_corpus(size);

        group.bench_with_input(
            BenchmarkId::new("synthetic_corpus", size),
            &corpus,
            |b, corpus| {
                b.iter(|| {
                    let metrics = evaluator.evaluate(corpus);
                    criterion::black_box(metrics);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark memory efficiency on varying sentence lengths
fn benchmark_memory_efficiency(c: &mut Criterion) {
    // Create sentences of different lengths
    let short_corpus = CorpusLoader::create_synthetic_corpus(100);
    let long_sentences = vec![
        "The extraordinarily complex and multifaceted nature of human language comprehension and production mechanisms requires sophisticated computational models that can adequately capture the intricate relationships between syntactic structures and semantic representations.".repeat(2)
    ];

    c.bench_function("memory_short_sentences", |b| {
        let dummy_engine = UDPipeEngine::for_testing();
        let parser = UDPipeParser::new_with_engine(dummy_engine);
        let evaluator = CorpusEvaluator::new(parser).with_memory_config(MemoryConfig {
            max_memory_per_sentence: 10 * 1024, // 10KB limit
            ..Default::default()
        });

        b.iter(|| {
            let metrics = evaluator.evaluate(&short_corpus[..10]); // Test first 10 sentences
            criterion::black_box(metrics);
        });
    });

    c.bench_function("memory_long_sentences", |b| {
        let dummy_engine = UDPipeEngine::for_testing();
        let parser = UDPipeParser::new_with_engine(dummy_engine);
        let evaluator = CorpusEvaluator::new(parser).with_memory_config(MemoryConfig {
            max_memory_per_sentence: 100 * 1024, // 100KB limit
            ..Default::default()
        });

        b.iter(|| {
            let metrics = evaluator.evaluate(&long_sentences);
            criterion::black_box(metrics);
        });
    });
}

/// Benchmark accuracy evaluation (placeholder for real accuracy metrics)
fn benchmark_accuracy_evaluation(c: &mut Criterion) {
    let dummy_engine = UDPipeEngine::for_testing();
    let parser = UDPipeParser::new_with_engine(dummy_engine);
    let evaluator = CorpusEvaluator::new(parser);

    // Create a balanced corpus with different sentence types
    let balanced_corpus = vec![
        "Simple sentence with basic structure.".to_string(),
        "Complex sentence with subordinate clauses that modify the main idea.".to_string(),
        "What did you do yesterday afternoon?".to_string(),
        "Please close the door and turn off the lights.".to_string(),
        "The book, which I read last week, was fascinating.".to_string(),
        "Although it was raining, we decided to go for a walk.".to_string(),
        "Programming in Rust requires careful attention to memory management.".to_string(),
        "The conference featured speakers from universities around the world.".to_string(),
        "She couldn't believe how quickly the project was completed.".to_string(),
        "Natural language processing combines linguistics with computer science.".to_string(),
    ];

    c.bench_function("accuracy_evaluation_balanced", |b| {
        b.iter(|| {
            let metrics = evaluator.evaluate(&balanced_corpus);
            // In a real implementation, we would compare against gold standard annotations
            assert!(
                metrics.success_rate > 0.0,
                "Should have some successful parses"
            );
            criterion::black_box(metrics);
        });
    });
}

/// Performance regression test (ensures we don't get slower over time)
fn benchmark_performance_regression(c: &mut Criterion) {
    let dummy_engine = UDPipeEngine::for_testing();
    let parser = UDPipeParser::new_with_engine(dummy_engine);
    let evaluator = CorpusEvaluator::new(parser);

    // Fixed corpus for regression testing
    let regression_corpus = CorpusLoader::create_synthetic_corpus(50);

    c.bench_function("performance_regression_baseline", |b| {
        b.iter(|| {
            let start = Instant::now();
            let metrics = evaluator.evaluate(&regression_corpus);
            let elapsed = start.elapsed();

            // Performance targets from M2 requirements
            let target_avg_sentence_time_us = 10_000.0; // 10ms = 10,000µs per sentence

            assert!(
                metrics.avg_sentence_time_us < target_avg_sentence_time_us,
                "Average sentence time {:.2}µs exceeds target {}µs",
                metrics.avg_sentence_time_us,
                target_avg_sentence_time_us
            );

            criterion::black_box((metrics, elapsed));
        });
    });
}

criterion_group!(
    corpus_benches,
    benchmark_corpus_performance,
    benchmark_memory_efficiency,
    benchmark_accuracy_evaluation,
    benchmark_performance_regression
);

criterion_main!(corpus_benches);

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::CorpusLoader;

    #[test]
    fn test_corpus_loader() {
        let synthetic_corpus = CorpusLoader::create_synthetic_corpus(5);
        assert_eq!(synthetic_corpus.len(), 5);
        assert!(synthetic_corpus[0].contains("cat"));
        assert!(synthetic_corpus[1].contains("gave"));
    }

    #[test]
    fn test_corpus_evaluator() {
        let dummy_engine = UDPipeEngine::for_testing();
        let parser = UDPipeParser::new_with_engine(dummy_engine);
        let evaluator = CorpusEvaluator::new(parser);

        let test_corpus = vec!["The cat sat.".to_string(), "Invalid input.".to_string()];

        let metrics = evaluator.evaluate(&test_corpus);
        assert_eq!(metrics.sentences_processed, 2);
        assert!(metrics.success_rate >= 0.0 && metrics.success_rate <= 1.0);
    }

    #[test]
    fn test_memory_config_integration() {
        let config = MemoryConfig {
            max_memory_per_sentence: 1024,
            max_words_per_sentence: 50,
            enable_pooling: true,
            enable_tracking: true,
        };

        let dummy_engine = UDPipeEngine::for_testing();
        let parser = UDPipeParser::new_with_engine(dummy_engine);
        let evaluator = CorpusEvaluator::new(parser).with_memory_config(config);

        // Should not panic with memory constraints
        let corpus = CorpusLoader::create_synthetic_corpus(10);
        let metrics = evaluator.evaluate(&corpus);
        assert!(metrics.sentences_processed > 0);
    }
}
