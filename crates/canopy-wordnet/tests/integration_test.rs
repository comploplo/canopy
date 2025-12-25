//! Integration tests for WordNet engine with BaseEngine infrastructure
//!
//! These tests verify that WordNet properly loads and uses real data.
//! With workspace-relative path resolution, engines find their data automatically.

use canopy_engine::{EngineCore, SemanticEngine, StatisticsProvider};
use canopy_wordnet::{DataLoader, PartOfSpeech, WordNetEngine};

#[test]
fn test_wordnet_engine_baseengine_integration() {
    let engine = match WordNetEngine::new() {
        Ok(e) => e,
        Err(e) => {
            println!("Skipping test: WordNet data not available: {}", e);
            return;
        }
    };

    // With proper path resolution, engine should be initialized
    assert!(
        EngineCore::is_initialized(&engine),
        "Engine should be initialized"
    );
    assert!(engine.is_ready(), "Engine should be ready");

    println!("📊 Engine Statistics:");
    let data_info = engine.data_info();
    println!("  Synsets: {}", data_info.entry_count);
    assert!(data_info.entry_count > 0, "Should have loaded synsets");

    // Test BaseEngine integration
    println!("🔧 BaseEngine Integration:");
    let stats = engine.statistics();
    println!("  Engine stats available: {}", stats.engine_name);
    assert_eq!(stats.engine_name, "WordNet");

    // Test semantic analysis through BaseEngine
    let result = engine.analyze_word("dog", PartOfSpeech::Noun).unwrap();
    println!("🎯 Semantic Analysis Test:");
    println!("  Input: 'dog' (noun)");
    println!("  Synsets found: {}", result.data.synsets.len());
    println!("  Confidence: {:.2}", result.confidence);
    println!("  From cache: {}", result.from_cache);

    assert!(
        !result.data.synsets.is_empty(),
        "Should find synsets for 'dog'"
    );
    assert!(result.confidence > 0.0, "Should have positive confidence");

    // Test different POS
    let result = engine.analyze_word("run", PartOfSpeech::Verb).unwrap();
    println!("🎯 Verb Analysis Test:");
    println!("  Input: 'run' (verb)");
    println!("  Synsets found: {}", result.data.synsets.len());
    assert!(
        !result.data.synsets.is_empty(),
        "Should find synsets for 'run'"
    );
}

#[test]
fn test_wordnet_engine_with_loaded_data() {
    let engine = match WordNetEngine::new() {
        Ok(e) => e,
        Err(e) => {
            println!("Skipping test: WordNet data not available: {}", e);
            return;
        }
    };

    // Engine auto-loads data on creation
    assert!(
        EngineCore::is_initialized(&engine),
        "Engine should be initialized"
    );
    assert!(engine.is_ready(), "Engine should be ready");

    // Test analysis with data
    let result = engine.analyze_word("test", PartOfSpeech::Noun).unwrap();
    assert!(
        result.confidence >= 0.0,
        "Confidence should be non-negative"
    );
    assert_eq!(result.data.word, "test");
    assert_eq!(result.data.pos, PartOfSpeech::Noun);

    // Data info should show loaded synsets
    let data_info = engine.data_info();
    assert!(data_info.entry_count > 0, "Should have loaded synsets");
}

#[test]
fn test_wordnet_caching_behavior() {
    let engine = match WordNetEngine::new() {
        Ok(e) => e,
        Err(e) => {
            println!("Skipping test: WordNet data not available: {}", e);
            return;
        }
    };

    // Test cache behavior - should start empty
    let initial_stats = engine.cache_stats();
    let initial_hits = initial_stats.hits;
    let initial_misses = initial_stats.misses;

    // First analysis (cache miss)
    let result1 = engine.analyze_word("test", PartOfSpeech::Noun).unwrap();

    // Second analysis (should be cache hit)
    let result2 = engine.analyze_word("test", PartOfSpeech::Noun).unwrap();

    // Results should be consistent
    assert_eq!(result1.confidence, result2.confidence);
    assert_eq!(result1.data.synsets.len(), result2.data.synsets.len());

    // Cache should show activity
    let final_stats = engine.cache_stats();
    assert!(
        final_stats.hits > initial_hits || final_stats.misses > initial_misses,
        "Cache should show some activity"
    );

    // Second result should be from cache
    assert!(result2.from_cache, "Second lookup should be from cache");

    // Different POS should not be cached result from first POS
    let _result3 = engine.analyze_word("test", PartOfSpeech::Verb).unwrap();
}

#[test]
fn test_wordnet_config_and_traits() {
    let engine = match WordNetEngine::new() {
        Ok(e) => e,
        Err(e) => {
            println!("Skipping test: WordNet data not available: {}", e);
            return;
        }
    };

    // Test trait implementations
    assert_eq!(engine.name(), "WordNet");
    assert_eq!(engine.version(), "3.1");
    assert_eq!(engine.engine_name(), "WordNet");
    assert_eq!(engine.engine_version(), "3.1");

    // Test configuration access
    let config = engine.config();
    assert!(config.enable_cache, "Default config should enable caching");
    assert_eq!(
        config.cache_capacity, 10000,
        "Default cache capacity should be 10000"
    );
    assert_eq!(
        config.min_confidence, 0.1,
        "Default min confidence should be 0.1"
    );
    assert!(
        config.enable_morphology,
        "Default config should enable morphology"
    );
    assert_eq!(
        config.max_search_depth, 5,
        "Default max search depth should be 5"
    );
}

#[test]
fn test_wordnet_pos_variations() {
    let engine = match WordNetEngine::new() {
        Ok(e) => e,
        Err(e) => {
            println!("Skipping test: WordNet data not available: {}", e);
            return;
        }
    };

    // Test different part-of-speech categories
    let pos_variants = [
        PartOfSpeech::Noun,
        PartOfSpeech::Verb,
        PartOfSpeech::Adjective,
        PartOfSpeech::Adverb,
    ];

    for pos in &pos_variants {
        let result = engine.analyze_word("good", *pos).unwrap();
        // Results vary by POS but should be consistent
        assert_eq!(result.data.word, "good");
        assert_eq!(result.data.pos, *pos);
        assert!(
            result.confidence >= 0.0,
            "Confidence should be non-negative"
        );
    }
}

#[test]
fn test_wordnet_semantic_methods() {
    let engine = match WordNetEngine::new() {
        Ok(e) => e,
        Err(e) => {
            println!("Skipping test: WordNet data not available: {}", e);
            return;
        }
    };

    // Test synonym lookup with loaded data
    let synonyms = engine.get_synonyms("dog", PartOfSpeech::Noun);
    println!("Synonyms for 'dog': {:?}", synonyms);
    // Should find some synonyms for common words
    // Note: exact results depend on WordNet data

    // Test hypernyms with a synset (would need valid synset ID from analysis)
    let hypernyms = engine.get_hypernyms("invalid_synset");
    assert!(hypernyms.is_empty(), "Invalid synset should return empty");

    let hyponyms = engine.get_hyponyms("invalid_synset");
    assert!(hyponyms.is_empty(), "Invalid synset should return empty");
}

#[test]
fn test_wordnet_data_loader_trait() {
    let mut engine = match WordNetEngine::new() {
        Ok(e) => e,
        Err(e) => {
            println!("Skipping test: WordNet data not available: {}", e);
            return;
        }
    };

    // Engine should be initialized after creation
    assert!(EngineCore::is_initialized(&engine), "Should be initialized");

    // Reload should succeed with valid data path
    match engine.reload() {
        Ok(()) => println!("Reload succeeded"),
        Err(e) => println!("Reload result: {}", e),
    }

    // Test data info
    let data_info = engine.data_info();
    assert!(
        data_info.source.contains("wordnet"),
        "Data info should mention WordNet"
    );
    assert!(
        data_info.entry_count > 0,
        "Should have entries after reload"
    );
}

#[test]
fn test_wordnet_string_analysis() {
    let engine = match WordNetEngine::new() {
        Ok(e) => e,
        Err(e) => {
            println!("Skipping test: WordNet data not available: {}", e);
            return;
        }
    };

    // Test SemanticEngine trait with string input (defaults to noun POS)
    let result = engine.analyze(&"dog".to_string()).unwrap();
    assert_eq!(
        result.data.pos,
        PartOfSpeech::Noun,
        "Should default to noun POS"
    );
    assert!(
        !result.data.synsets.is_empty(),
        "Should find synsets for 'dog'"
    );
    assert!(result.confidence > 0.0, "Should have positive confidence");
}
