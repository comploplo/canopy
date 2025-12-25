//! Pattern matching benchmarks for dependency pattern lookup
//!
//! Measures performance of pattern matching operations to validate
//! M6 targets for cache hit rates and lookup latencies.

use canopy_treebank::{DependencyPattern, DependencyRelation, PatternSource};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use lru::LruCache;
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::time::Instant;

/// Create sample patterns for benchmarking
fn create_sample_patterns(count: usize) -> HashMap<String, DependencyPattern> {
    let mut patterns = HashMap::new();

    let common_verbs = [
        "be", "have", "do", "say", "get", "make", "go", "know", "take", "see", "come", "think",
        "look", "want", "give", "use", "find", "tell", "ask", "work", "seem", "feel", "try",
        "leave", "call", "good", "new", "first", "last", "long",
    ];

    let common_rels = ["nsubj", "obj", "iobj", "obl", "ccomp", "xcomp", "advmod"];
    let common_pos = ["NOUN", "PRON", "ADJ", "ADV", "ADP"];

    for i in 0..count {
        let verb = common_verbs[i % common_verbs.len()];
        let rel1 = common_rels[i % common_rels.len()];
        let pos1 = common_pos[i % common_pos.len()];
        let rel2 = common_rels[(i + 1) % common_rels.len()];
        let pos2 = common_pos[(i + 1) % common_pos.len()];

        let pattern = DependencyPattern {
            verb_lemma: verb.to_string(),
            dependencies: vec![
                (DependencyRelation::from(rel1), pos1.to_string()),
                (DependencyRelation::from(rel2), pos2.to_string()),
            ],
            confidence: 0.8 + (i as f32 * 0.01) % 0.2,
            frequency: (i * 7 + 13) as u32 % 100 + 1, // Varied frequencies
            source: PatternSource::Indexed,
        };

        let key = format!("{}:{}:{}", verb, rel1, rel2);
        patterns.insert(key, pattern);
    }

    patterns
}

/// Create sample signature keys for lookup
fn create_sample_signature_keys(count: usize) -> Vec<String> {
    let mut keys = Vec::new();

    let common_verbs = [
        "be", "have", "do", "say", "get", "make", "go", "know", "take", "see", "run", "walk",
        "eat", "drink", "sleep", "read", "write", "play", "work", "study",
    ];

    for i in 0..count {
        let verb = common_verbs[i % common_verbs.len()];
        keys.push(format!("{}:nsubj:obj", verb));
    }

    keys
}

fn bench_core_cache_lookup(c: &mut Criterion) {
    // Simulate core cache with 2,000 most frequent patterns
    let patterns = create_sample_patterns(2000);
    let signature_keys = create_sample_signature_keys(1000);

    c.bench_function("core_cache_lookup", |b| {
        b.iter(|| {
            for key in &signature_keys {
                if let Some(pattern) = patterns.get(key) {
                    black_box(pattern);
                }
            }
        });
    });
}

fn bench_lru_cache_performance(c: &mut Criterion) {
    let patterns = create_sample_patterns(8000);
    let signatures = create_sample_signature_keys(1000);

    let mut group = c.benchmark_group("lru_cache");

    // Test different LRU cache sizes
    for cache_size in [1000, 2000, 3000, 5000].iter() {
        group.bench_with_input(
            BenchmarkId::new("cache_size", cache_size),
            cache_size,
            |b, &cache_size| {
                b.iter_custom(|iters| {
                    let mut total_time = std::time::Duration::new(0, 0);

                    for _ in 0..iters {
                        let mut cache = LruCache::<String, DependencyPattern>::new(
                            NonZeroUsize::new(cache_size).unwrap(),
                        );

                        // Pre-populate cache with some patterns
                        for (key, pattern) in patterns.iter().take(cache_size / 2) {
                            cache.put(key.clone(), pattern.clone());
                        }

                        let start = Instant::now();

                        for signature in &signatures {
                            let key = signature.clone();

                            if let Some(pattern) = cache.get(&key) {
                                black_box(pattern);
                            } else {
                                // Cache miss - simulate loading from index
                                if let Some(pattern) = patterns.get(&key) {
                                    cache.put(key, pattern.clone());
                                }
                            }
                        }

                        total_time += start.elapsed();
                    }

                    total_time
                });
            },
        );
    }

    group.finish();
}

fn bench_pattern_key_generation(c: &mut Criterion) {
    let signatures = create_sample_signature_keys(1000);

    c.bench_function("pattern_key_generation", |b| {
        b.iter(|| {
            for signature in &signatures {
                // Simulate different key generation strategies
                let simple_key = format!("{}:basic", signature);
                let complex_key = format!("{}:complex", signature);

                black_box((simple_key, complex_key));
            }
        });
    });
}

fn bench_cache_hit_rate_simulation(c: &mut Criterion) {
    let patterns = create_sample_patterns(8000);
    let signatures = create_sample_signature_keys(2000);

    c.bench_function("cache_hit_rate_simulation", |b| {
        b.iter_custom(|iters| {
            let mut total_time = std::time::Duration::new(0, 0);

            for _ in 0..iters {
                let start = Instant::now();

                // Simulate core cache (top 2000 patterns)
                let core_patterns: HashMap<String, &DependencyPattern> = patterns
                    .iter()
                    .take(2000)
                    .map(|(k, v)| (k.clone(), v))
                    .collect();

                // Simulate LRU cache
                let mut lru_cache =
                    LruCache::<String, DependencyPattern>::new(NonZeroUsize::new(3000).unwrap());

                let mut core_hits = 0;
                let mut lru_hits = 0;
                let mut misses = 0;

                for signature in &signatures {
                    let key = signature.clone();

                    if core_patterns.contains_key(&key) {
                        core_hits += 1;
                    } else if lru_cache.get(&key).is_some() {
                        lru_hits += 1;
                    } else {
                        misses += 1;

                        // Simulate loading from full patterns
                        if let Some(pattern) = patterns.get(&key) {
                            lru_cache.put(key, pattern.clone());
                        }
                    }
                }

                let total_requests = signatures.len();
                let core_hit_rate = core_hits as f32 / total_requests as f32;
                let lru_hit_rate = lru_hits as f32 / total_requests as f32;
                let miss_rate = misses as f32 / total_requests as f32;

                black_box((core_hit_rate, lru_hit_rate, miss_rate));

                total_time += start.elapsed();
            }

            total_time
        });
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(50)
        .measurement_time(std::time::Duration::from_secs(15));
    targets = bench_core_cache_lookup,
              bench_lru_cache_performance,
              bench_pattern_key_generation,
              bench_cache_hit_rate_simulation
}

criterion_main!(benches);
