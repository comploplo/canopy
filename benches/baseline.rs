use criterion::{Criterion, black_box, criterion_group, criterion_main};
use std::time::Duration;

/// Baseline benchmarks for canopy semantic analysis performance
///
/// These benchmarks measure real semantic analysis operations.
/// Run with: `cargo bench` or `cargo bench --release`
fn semantic_coordinator_benchmark(c: &mut Criterion) {
    use canopy_tokenizer::{SemanticCoordinator, coordinator::CoordinatorConfig};

    let mut group = c.benchmark_group("semantic_analysis");
    group.sample_size(20);
    group.measurement_time(Duration::from_secs(10));

    // Skip if semantic data not available
    let coordinator = match SemanticCoordinator::new(CoordinatorConfig::default()) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Skipping benchmark: semantic data not available: {}", e);
            return;
        }
    };

    let test_words = ["run", "give", "think", "eat", "walk"];

    // Warm up the cache
    for word in test_words.iter() {
        let _ = coordinator.analyze(*word);
    }

    group.bench_function("analyze_word_cached", |b| {
        b.iter(|| {
            for word in test_words.iter() {
                let _ = coordinator.analyze(black_box(*word));
            }
        });
    });

    group.finish();
}

fn engine_lookup_benchmark(c: &mut Criterion) {
    use canopy_verbnet::VerbNetEngine;
    use canopy_wordnet::WordNetEngine;

    let mut group = c.benchmark_group("engine_lookups");
    group.sample_size(50);
    group.measurement_time(Duration::from_secs(5));

    // VerbNet lookup benchmark
    if let Ok(verbnet) = VerbNetEngine::new() {
        group.bench_function("verbnet_analyze", |b| {
            b.iter(|| verbnet.analyze_verb(black_box("give")).ok());
        });
    }

    // WordNet lookup benchmark
    if let Ok(wordnet) = WordNetEngine::new() {
        use canopy_wordnet::types::PartOfSpeech;
        group.bench_function("wordnet_analyze", |b| {
            b.iter(|| {
                wordnet
                    .analyze_word(black_box("run"), PartOfSpeech::Verb)
                    .ok()
            });
        });
    }

    group.finish();
}

fn lemmatization_benchmark(c: &mut Criterion) {
    use canopy_tokenizer::lemmatizer::{Lemmatizer, SimpleLemmatizer};

    let mut group = c.benchmark_group("lemmatization");
    group.sample_size(100);

    let lemmatizer = SimpleLemmatizer::default();

    let test_words = ["running", "gave", "walked", "thinking", "eaten"];

    group.bench_function("simple_lemmatize", |b| {
        b.iter(|| {
            for word in test_words.iter() {
                black_box(lemmatizer.lemmatize(word));
            }
        });
    });

    group.bench_function("lemmatize_with_confidence", |b| {
        b.iter(|| {
            for word in test_words.iter() {
                black_box(lemmatizer.lemmatize_with_confidence(word));
            }
        });
    });

    group.finish();
}

fn treebank_loading_benchmark(c: &mut Criterion) {
    use canopy_core::treebank_loader::TreebankSentenceLoader;

    let mut group = c.benchmark_group("treebank");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(15));

    // Only run if treebank data exists
    let treebank_path = std::path::Path::new("data/ud_english-ewt");
    if !treebank_path.exists() {
        eprintln!("Skipping treebank benchmark: data not available");
        return;
    }

    group.bench_function("load_treebank", |b| {
        b.iter(|| TreebankSentenceLoader::new().ok());
    });

    group.finish();
}

criterion_group!(
    benches,
    semantic_coordinator_benchmark,
    engine_lookup_benchmark,
    lemmatization_benchmark,
    treebank_loading_benchmark,
);

criterion_main!(benches);
