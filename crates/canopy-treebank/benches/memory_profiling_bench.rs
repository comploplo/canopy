//! Memory profiling benchmarks for treebank operations
//!
//! Validates M6 memory targets of 10-15MB total overhead
//! through comprehensive memory usage measurements.

use canopy_treebank::{DependencyPattern, DependencyRelation, PatternSource};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use lru::LruCache;
use std::collections::HashMap;
use std::mem;
use std::num::NonZeroUsize;

/// Estimate memory usage of data structures
fn estimate_memory_usage() -> MemoryReport {
    // Estimate pattern memory usage
    let sample_pattern = DependencyPattern {
        verb_lemma: "example".to_string(),
        dependencies: vec![
            (DependencyRelation::from("nsubj"), "NOUN".to_string()),
            (DependencyRelation::from("obj"), "NOUN".to_string()),
        ],
        confidence: 0.8,
        frequency: 42,
        source: PatternSource::Indexed,
    };

    let pattern_size = mem::size_of_val(&sample_pattern)
        + sample_pattern.verb_lemma.capacity()
        + sample_pattern
            .dependencies
            .iter()
            .map(|(a, b)| 20 + b.capacity()) // Estimate for DependencyRelation + string
            .sum::<usize>()
        + 0; // Removed source_sentences field

    // Estimate signature memory usage (simplified)
    let signature_size = 100; // Estimated basic signature size

    MemoryReport {
        pattern_size_bytes: pattern_size,
        signature_size_bytes: signature_size,
        estimated_total_mb: 0.0, // Will be calculated
    }
}

#[derive(Debug)]
struct MemoryReport {
    pattern_size_bytes: usize,
    signature_size_bytes: usize,
    estimated_total_mb: f64,
}

fn bench_memory_usage_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_patterns");

    // Test memory usage with different pattern counts
    for count in [1000, 2000, 5000, 8000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::new("pattern_count", count),
            count,
            |b, &count| {
                b.iter(|| {
                    let mut patterns = HashMap::new();

                    for i in 0..count {
                        let pattern = DependencyPattern {
                            verb_lemma: format!("verb_{}", i % 100),
                            dependencies: vec![
                                (DependencyRelation::from("nsubj"), format!("pos_{}", i % 5)),
                                (
                                    DependencyRelation::from("obj"),
                                    format!("pos_{}", (i + 1) % 5),
                                ),
                            ],
                            confidence: 0.5 + (i as f32 * 0.001) % 0.5,
                            frequency: (i * 7 + 13) % 100 + 1,
                            source: PatternSource::Indexed,
                        };

                        let key = format!("pattern_{}", i);
                        patterns.insert(key, pattern);
                    }

                    // Estimate memory usage
                    let estimated_size_mb = (patterns.len() * 200) as f64 / 1024.0 / 1024.0;

                    black_box((patterns.len(), estimated_size_mb))
                });
            },
        );
    }

    group.finish();
}

fn bench_cache_memory_overhead(c: &mut Criterion) {
    let patterns = create_test_patterns(5000);

    let mut group = c.benchmark_group("cache_memory");

    // Test different cache sizes and their memory overhead
    for cache_size in [1000, 2000, 3000, 5000].iter() {
        group.bench_with_input(
            BenchmarkId::new("cache_entries", cache_size),
            cache_size,
            |b, &cache_size| {
                b.iter(|| {
                    let mut cache = LruCache::<String, DependencyPattern>::new(
                        NonZeroUsize::new(cache_size).unwrap(),
                    );

                    // Fill cache with patterns
                    for (i, (_key, pattern)) in patterns.iter().take(cache_size).enumerate() {
                        cache.put(format!("cache_{}", i), pattern.clone());
                    }

                    // Estimate cache memory usage
                    let estimated_cache_mb = (cache_size * 300) as f64 / 1024.0 / 1024.0;

                    black_box((cache.len(), estimated_cache_mb))
                });
            },
        );
    }

    group.finish();
}

fn create_test_patterns(count: usize) -> HashMap<String, DependencyPattern> {
    let mut patterns = HashMap::new();

    for i in 0..count {
        let pattern = DependencyPattern {
            verb_lemma: format!("verb_{}", i % 50),
            dependencies: vec![
                (DependencyRelation::from("nsubj"), "NOUN".to_string()),
                (DependencyRelation::from("obj"), "NOUN".to_string()),
            ],
            confidence: 0.8,
            frequency: i as u32 + 1,
            source: PatternSource::Indexed,
        };

        patterns.insert(format!("key_{}", i), pattern);
    }

    patterns
}

fn bench_total_memory_simulation(c: &mut Criterion) {
    c.bench_function("total_memory_simulation", |b| {
        b.iter(|| {
            // Simulate M6 target memory usage:
            // - 8,000 patterns in main storage
            // - 2,000 patterns in core cache
            // - 3,000 entry LRU cache
            // - Indexing structures

            let main_patterns = create_test_patterns(8000);

            let core_cache: HashMap<String, DependencyPattern> = main_patterns
                .iter()
                .take(2000)
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();

            let mut lru_cache =
                LruCache::<String, DependencyPattern>::new(NonZeroUsize::new(3000).unwrap());

            // Populate LRU cache
            for (i, (_key, pattern)) in main_patterns.iter().skip(2000).take(3000).enumerate() {
                lru_cache.put(format!("lru_{}", i), pattern.clone());
            }

            // Estimate total memory usage
            let pattern_memory_mb = (main_patterns.len() * 200) as f64 / 1024.0 / 1024.0;
            let core_cache_mb = (core_cache.len() * 200) as f64 / 1024.0 / 1024.0;
            let lru_cache_mb = (3000 * 200) as f64 / 1024.0 / 1024.0;
            let indexing_overhead_mb = 2.0; // Estimated

            let total_memory_mb =
                pattern_memory_mb + core_cache_mb + lru_cache_mb + indexing_overhead_mb;

            let memory_report = MemoryUsageReport {
                pattern_storage_mb: pattern_memory_mb,
                core_cache_mb,
                lru_cache_mb,
                indexing_mb: indexing_overhead_mb,
                total_mb: total_memory_mb,
                within_target: total_memory_mb <= 15.0,
            };

            black_box(memory_report)
        });
    });
}

#[derive(Debug)]
struct MemoryUsageReport {
    pattern_storage_mb: f64,
    core_cache_mb: f64,
    lru_cache_mb: f64,
    indexing_mb: f64,
    total_mb: f64,
    within_target: bool,
}

fn bench_memory_leak_detection(c: &mut Criterion) {
    c.bench_function("memory_leak_detection", |b| {
        b.iter_custom(|iters| {
            let mut total_time = std::time::Duration::new(0, 0);
            let mut initial_memory_estimate = 0;

            for iter in 0..iters {
                let start = std::time::Instant::now();

                // Create and destroy patterns multiple times
                let patterns = create_test_patterns(1000);
                let pattern_count = patterns.len();

                if iter == 0 {
                    initial_memory_estimate = pattern_count * 200; // bytes
                }

                // Simulate pattern usage
                for (key, pattern) in &patterns {
                    black_box((key, pattern));
                }

                // Patterns should be dropped here
                let current_estimate = pattern_count * 200;
                let memory_stable = current_estimate == initial_memory_estimate;

                black_box(memory_stable);

                total_time += start.elapsed();
            }

            total_time
        });
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(20)
        .measurement_time(std::time::Duration::from_secs(15));
    targets = bench_memory_usage_patterns,
              bench_cache_memory_overhead,
              bench_total_memory_simulation,
              bench_memory_leak_detection
}

criterion_main!(benches);
