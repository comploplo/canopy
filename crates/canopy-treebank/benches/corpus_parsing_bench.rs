//! Corpus parsing benchmarks for UD English-EWT data
//!
//! Measures CoNLL-U parsing performance on the full training corpus
//! to establish baseline metrics for M6 implementation.

use canopy_treebank::{ConlluParser, ParsedSentence};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::path::Path;
use std::time::{Duration, Instant};

fn bench_corpus_parsing(c: &mut Criterion) {
    let training_path = Path::new("data/ud_english-ewt/UD_English-EWT/en_ewt-ud-train.conllu");

    if !training_path.exists() {
        println!("Skipping corpus parsing benchmark - training data not found");
        return;
    }

    let mut group = c.benchmark_group("corpus_parsing");

    // Benchmark full corpus parsing
    group.bench_function("parse_full_training_corpus", |b| {
        b.iter(|| {
            let parser = ConlluParser::new(false);
            let sentences = parser
                .parse_file(black_box(training_path))
                .expect("Failed to parse training data");
            black_box(sentences)
        })
    });

    // Benchmark parsing speed per sentence
    group.bench_function("parse_sentences_throughput", |b| {
        let parser = ConlluParser::new(false);
        let sentences = parser
            .parse_file(training_path)
            .expect("Failed to parse training data");

        b.iter(|| {
            for sentence in sentences.iter().take(100) {
                black_box(sentence);
            }
        })
    });

    group.finish();
}

fn bench_memory_usage(c: &mut Criterion) {
    let training_path = Path::new("data/ud_english-ewt/UD_English-EWT/en_ewt-ud-train.conllu");

    if !training_path.exists() {
        return;
    }

    c.bench_function("memory_usage_full_corpus", |b| {
        b.iter_custom(|iters| {
            let mut total_duration = Duration::new(0, 0);

            for _ in 0..iters {
                let start = Instant::now();

                let parser = ConlluParser::new(false);
                let sentences = parser
                    .parse_file(black_box(training_path))
                    .expect("Failed to parse training data");

                // Force evaluation of all sentences
                let sentence_count = sentences.len();
                let word_count: usize = sentences.iter().map(|s| s.tokens.len()).sum();

                black_box((sentence_count, word_count));

                total_duration += start.elapsed();
            }

            total_duration
        });
    });
}

/// Benchmark parsing different corpus sizes to understand scaling
fn bench_corpus_scaling(c: &mut Criterion) {
    let training_path = Path::new("data/ud_english-ewt/UD_English-EWT/en_ewt-ud-train.conllu");

    if !training_path.exists() {
        return;
    }

    let parser = ConlluParser::new(false);
    let all_sentences = parser
        .parse_file(training_path)
        .expect("Failed to parse training data");

    let mut group = c.benchmark_group("corpus_scaling");

    // Test different corpus sizes
    for size in [100, 500, 1000, 5000, 12544].iter() {
        let size = *size.min(&all_sentences.len());

        group.bench_with_input(BenchmarkId::new("sentences", size), &size, |b, &size| {
            b.iter(|| {
                let sentences: Vec<&ParsedSentence> = all_sentences.iter().take(size).collect();

                // Simulate pattern extraction work
                let word_count: usize = sentences.iter().map(|s| s.tokens.len()).sum();

                black_box(word_count);
            });
        });
    }

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(10) // Fewer samples for large corpus
        .measurement_time(Duration::from_secs(30));
    targets = bench_corpus_parsing, bench_memory_usage, bench_corpus_scaling
}

criterion_main!(benches);
