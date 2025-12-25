//! Pattern extraction benchmarks for dependency patterns
//!
//! Measures performance of extracting dependency patterns from parsed
//! UD English-EWT data to optimize for M6 targets.

use canopy_treebank::{ConlluParser, DependencyPattern, ParsedSentence, PatternSource};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::collections::HashMap;
use std::path::Path;
use std::time::{Duration, Instant};

/// Extract dependency patterns from parsed sentences
fn extract_patterns_from_sentences(
    sentences: &[ParsedSentence],
) -> HashMap<String, DependencyPattern> {
    let mut patterns = HashMap::new();

    for sentence in sentences {
        for token in &sentence.tokens {
            if token.upos == "VERB" {
                // Create a basic dependency pattern
                let pattern = DependencyPattern {
                    verb_lemma: token.lemma.clone(),
                    dependencies: sentence
                        .tokens
                        .iter()
                        .filter(|t| t.head == token.id)
                        .map(|t| (t.deprel.clone(), t.upos.clone()))
                        .collect(),
                    confidence: 0.8,
                    frequency: 1,
                    source: PatternSource::Indexed,
                };

                let key = format!(
                    "{}:{}",
                    token.lemma,
                    pattern
                        .dependencies
                        .iter()
                        .map(|(rel, pos)| format!("{:?}:{}", rel, pos))
                        .collect::<Vec<_>>()
                        .join(",")
                );

                patterns
                    .entry(key)
                    .and_modify(|p: &mut DependencyPattern| p.frequency += 1)
                    .or_insert(pattern);
            }
        }
    }

    patterns
}

fn bench_pattern_extraction(c: &mut Criterion) {
    let training_path = Path::new("data/ud_english-ewt/UD_English-EWT/en_ewt-ud-train.conllu");

    if !training_path.exists() {
        println!("Skipping pattern extraction benchmark - training data not found");
        return;
    }

    let parser = ConlluParser::new(false);
    let all_sentences = parser
        .parse_file(training_path)
        .expect("Failed to parse training data");

    println!(
        "Loaded {} sentences for pattern extraction",
        all_sentences.len()
    );

    let mut group = c.benchmark_group("pattern_extraction");

    // Benchmark pattern extraction on different corpus sizes
    for size in [100, 500, 1000, 2000].iter() {
        let size = *size.min(&all_sentences.len());
        let sentences = &all_sentences[..size];

        group.bench_with_input(
            BenchmarkId::new("extract_patterns", size),
            &sentences,
            |b, sentences| {
                b.iter(|| {
                    let patterns = extract_patterns_from_sentences(black_box(sentences));
                    black_box(patterns)
                });
            },
        );
    }

    group.finish();
}

fn bench_pattern_frequency_analysis(c: &mut Criterion) {
    let training_path = Path::new("data/ud_english-ewt/UD_English-EWT/en_ewt-ud-train.conllu");

    if !training_path.exists() {
        return;
    }

    let parser = ConlluParser::new(false);
    let sentences = parser
        .parse_file(training_path)
        .expect("Failed to parse training data");

    // Pre-extract patterns for benchmarking
    let patterns = extract_patterns_from_sentences(&sentences[..1000]);

    c.bench_function("frequency_analysis", |b| {
        b.iter(|| {
            // Sort patterns by frequency (top-k analysis)
            let mut pattern_freq: Vec<(&String, &DependencyPattern)> = patterns.iter().collect();
            pattern_freq.sort_by(|a, b| b.1.frequency.cmp(&a.1.frequency));

            // Simulate coverage analysis
            let total_freq: u32 = pattern_freq.iter().map(|(_, p)| p.frequency).sum();
            let mut cumulative_freq = 0;
            let mut coverage_points = Vec::new();

            for (i, (_, pattern)) in pattern_freq.iter().enumerate() {
                cumulative_freq += pattern.frequency;
                let coverage = cumulative_freq as f32 / total_freq as f32;

                if i % 100 == 0 || coverage > 0.8 {
                    coverage_points.push((i + 1, coverage));
                }

                if coverage > 0.95 {
                    break;
                }
            }

            black_box(coverage_points)
        });
    });
}

fn bench_pattern_memory_usage(c: &mut Criterion) {
    let training_path = Path::new("data/ud_english-ewt/UD_English-EWT/en_ewt-ud-train.conllu");

    if !training_path.exists() {
        return;
    }

    c.bench_function("pattern_memory_estimation", |b| {
        b.iter_custom(|iters| {
            let mut total_duration = Duration::new(0, 0);

            for _ in 0..iters {
                let start = Instant::now();

                let parser = ConlluParser::new(false);
                let sentences = parser
                    .parse_file(black_box(training_path))
                    .expect("Failed to parse training data");

                let patterns = extract_patterns_from_sentences(&sentences[..2000]);

                // Estimate memory usage
                let pattern_count = patterns.len();
                let estimated_memory_kb = pattern_count * 200 / 1024; // ~200 bytes per pattern

                black_box((pattern_count, estimated_memory_kb));

                total_duration += start.elapsed();
            }

            total_duration
        });
    });
}

/// Benchmark signature creation for pattern matching
fn bench_signature_creation(c: &mut Criterion) {
    let training_path = Path::new("data/ud_english-ewt/UD_English-EWT/en_ewt-ud-train.conllu");

    if !training_path.exists() {
        return;
    }

    let parser = ConlluParser::new(false);
    let sentences = parser
        .parse_file(training_path)
        .expect("Failed to parse training data");

    let sample_sentences = &sentences[..100];

    c.bench_function("signature_creation", |b| {
        b.iter(|| {
            for sentence in sample_sentences {
                for token in &sentence.tokens {
                    if token.upos == "VERB" {
                        // Create semantic signature
                        // Just create simple signature for benchmarking
                        let _signature_size = token.lemma.len() + 100; // Estimated

                        black_box(_signature_size);
                    }
                }
            }
        });
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(10)
        .measurement_time(Duration::from_secs(20));
    targets = bench_pattern_extraction,
              bench_pattern_frequency_analysis,
              bench_pattern_memory_usage,
              bench_signature_creation
}

criterion_main!(benches);
