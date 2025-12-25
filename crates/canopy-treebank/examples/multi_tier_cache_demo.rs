//! Multi-tier caching system demonstration
//!
//! Shows how to build and use the complete 3-tier caching system:
//! 1. Build pattern index from UD corpus data
//! 2. Populate multi-tier cache (core + LRU + disk)
//! 3. Test cache performance with realistic lookups

use canopy_engine::LemmaSource;
use canopy_treebank::signature::PosCategory;
use canopy_treebank::{DependencyPattern, PatternCacheFactory, PatternIndexer, SemanticSignature};
use std::path::Path;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Multi-Tier Cache System Demo");
    println!("==============================");

    // Step 1: Index patterns from corpus data
    println!("\n📊 Step 1: Building pattern index from corpus...");

    let corpus_path = Path::new("data/ud_english-ewt/UD_English-EWT/en_ewt-ud-train.conllu");

    if !corpus_path.exists() {
        println!("⚠️  Corpus file not found at {:?}", corpus_path);
        println!("   This demo shows the architecture with synthetic data instead.");
        demonstrate_with_synthetic_data()?;
        return Ok(());
    }

    let start_time = Instant::now();
    let mut indexer = PatternIndexer::new();
    indexer.index_from_corpus(corpus_path)?;

    let indexing_time = start_time.elapsed();
    println!(
        "✅ Indexed {} patterns in {:.2}s",
        indexer.pattern_count(),
        indexing_time.as_secs_f32()
    );

    // Step 2: Get frequency-sorted patterns for cache population
    let sorted_patterns = indexer.get_patterns_by_frequency();

    // Calculate coverage statistics
    let coverage_2k = indexer.calculate_coverage(2000) * 100.0;
    let coverage_5k = indexer.calculate_coverage(5000) * 100.0;
    let coverage_10k = indexer.calculate_coverage(10000) * 100.0;

    println!("   Coverage with 2,000 patterns: {:.1}%", coverage_2k);
    println!("   Coverage with 5,000 patterns: {:.1}%", coverage_5k);
    println!("   Coverage with 10,000 patterns: {:.1}%", coverage_10k);

    // Step 3: Create and populate multi-tier cache
    println!("\n🔄 Step 2: Setting up multi-tier cache system...");

    let mut cache = PatternCacheFactory::create_m6_optimized(None)?;

    // Populate core cache with top 2,000 patterns
    cache.populate_core_cache(&sorted_patterns[..2000.min(sorted_patterns.len())]);

    println!("✅ Core cache populated with top 2,000 patterns");
    println!(
        "   Memory usage: ~{:.1} KB",
        cache.estimate_memory_usage() as f64 / 1024.0
    );

    // Step 4: Test cache performance
    println!("\n⚡ Step 3: Testing cache performance...");

    let test_signatures = create_test_signatures();
    let lookup_start = Instant::now();

    for signature in &test_signatures {
        let _pattern = cache.get_pattern(signature);
        // In real usage, we'd process the pattern here
    }

    let lookup_time = lookup_start.elapsed();
    let avg_lookup_time = lookup_time.as_micros() as f64 / test_signatures.len() as f64;

    println!(
        "✅ Processed {} lookups in {:.2}ms",
        test_signatures.len(),
        lookup_time.as_millis()
    );
    println!("   Average lookup time: {:.1}μs", avg_lookup_time);

    // Display cache statistics
    println!("\n📈 Cache Performance Statistics:");
    cache.print_statistics();

    // Show top patterns
    println!("\n🔝 Top 10 Most Frequent Patterns:");
    for (i, (_key, pattern)) in sorted_patterns.iter().take(10).enumerate() {
        println!(
            "   {:2}. {} (freq: {}, deps: {})",
            i + 1,
            pattern.verb_lemma,
            pattern.frequency,
            pattern.dependencies.len()
        );
    }

    println!("\n✨ Multi-tier cache demo complete!");

    Ok(())
}

fn demonstrate_with_synthetic_data() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🧪 Demonstrating with synthetic test data...");

    // Create synthetic patterns
    let synthetic_patterns = create_synthetic_patterns(1000);

    // Create and populate cache
    let mut cache = PatternCacheFactory::create_test_cache()?;
    cache.populate_core_cache(&synthetic_patterns[..100.min(synthetic_patterns.len())]);

    // Test lookups
    let test_signatures = create_test_signatures();

    for signature in &test_signatures {
        let _pattern = cache.get_pattern(signature);
    }

    println!("✅ Synthetic test completed");
    cache.print_statistics();

    Ok(())
}

fn create_synthetic_patterns(count: usize) -> Vec<(String, DependencyPattern)> {
    use canopy_treebank::{DependencyRelation, PatternSource};

    let mut patterns = Vec::new();

    let verbs = [
        "run", "walk", "eat", "see", "make", "have", "be", "do", "say", "think",
    ];
    let relations = [
        DependencyRelation::NominalSubject,
        DependencyRelation::Object,
        DependencyRelation::IndirectObject,
        DependencyRelation::Oblique,
    ];
    let pos_tags = ["NOUN", "PRON", "PROPN"];

    for i in 0..count {
        let verb = verbs[i % verbs.len()];
        let rel = &relations[i % relations.len()];
        let pos = pos_tags[i % pos_tags.len()];

        let pattern = DependencyPattern {
            verb_lemma: verb.to_string(),
            dependencies: vec![(rel.clone(), pos.to_string())],
            confidence: 0.7 + (i as f32 * 0.001) % 0.3,
            frequency: (1000 - i) as u32, // Decreasing frequency
            source: PatternSource::Indexed,
        };

        let key = format!("{}|{:?}:{}", verb, rel, pos);
        patterns.push((key, pattern));
    }

    patterns
}

fn create_test_signatures() -> Vec<SemanticSignature> {
    let verbs = [
        "run", "walk", "eat", "see", "make", "have", "be", "do", "say", "think",
    ];

    verbs
        .iter()
        .map(|&verb| {
            SemanticSignature {
                lemma: verb.to_string(),
                verbnet_class: None,
                framenet_frame: None,
                pos_category: PosCategory::Verb,
                lemma_source: LemmaSource::UDGold,
                lemma_confidence: 0.95,
                hash_code: 0, // Would be calculated properly
            }
        })
        .collect()
}
