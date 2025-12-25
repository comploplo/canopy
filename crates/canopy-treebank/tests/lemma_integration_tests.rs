//! Integration tests for lemma validation and shared caching
//!
//! These tests validate the integration between the treebank's gold-standard lemmas
//! and the semantic layer's lemmatization system.

use canopy_engine::{LemmaCache, LemmaCacheConfig, LemmaSource, SemanticEngine};
use canopy_tokenizer::lemmatizer::SimpleLemmatizer;
use canopy_treebank::engine::TreebankConfig;
use canopy_treebank::{ConlluParser, LemmaValidator, ParsedSentence, TreebankEngine};
use std::collections::HashMap;

/// Load real dev data sample for testing (first 50 sentences)
fn load_dev_sample() -> Vec<ParsedSentence> {
    let dev_path = std::path::Path::new("data/ud_english-ewt/UD_English-EWT/en_ewt-ud-dev.conllu");

    if !dev_path.exists() {
        // Skip test if real data not available
        println!("Skipping test - dev data not found at {:?}", dev_path);
        return Vec::new();
    }

    let parser = ConlluParser::new(false);
    let sentences = parser
        .parse_file(dev_path)
        .expect("Failed to parse dev data");

    // Take first 50 sentences for reasonable test performance
    sentences.into_iter().take(50).collect()
}

/// Load full dev data for comprehensive testing
fn load_full_dev() -> Vec<ParsedSentence> {
    let dev_path = std::path::Path::new("data/ud_english-ewt/UD_English-EWT/en_ewt-ud-dev.conllu");

    if !dev_path.exists() {
        println!("Skipping test - dev data not found at {:?}", dev_path);
        return Vec::new();
    }

    let parser = ConlluParser::new(false);
    parser
        .parse_file(dev_path)
        .expect("Failed to parse dev data")
}

#[test]
fn test_lemma_cache_basic_operations() {
    let cache = LemmaCache::default();

    // Test empty cache
    assert!(cache.get("running").is_none());
    assert_eq!(cache.len(), 0);

    // Insert lemma with different sources
    cache.insert_simple("running", "run", LemmaSource::SimpleLemmatizer);
    cache.insert_simple("children", "child", LemmaSource::UDGold);

    // Test retrieval
    let entry1 = cache.get("running").unwrap();
    assert_eq!(entry1.lemma, "run");
    assert_eq!(entry1.source, LemmaSource::SimpleLemmatizer);

    let entry2 = cache.get("children").unwrap();
    assert_eq!(entry2.lemma, "child");
    assert_eq!(entry2.source, LemmaSource::UDGold);

    // Test stats
    let stats = cache.stats();
    assert_eq!(stats.hits, 2);
    assert_eq!(stats.misses, 1);
    assert_eq!(stats.insertions, 2);
    assert!(stats.hit_rate() > 0.0);
}

#[test]
fn test_lemma_cache_priority_system() {
    let cache = LemmaCache::default();

    // Insert lower priority first
    cache.insert_simple("running", "run_simple", LemmaSource::SimpleLemmatizer);
    assert_eq!(cache.get("running").unwrap().lemma, "run_simple");

    // Higher priority should override
    cache.insert_simple("running", "run_gold", LemmaSource::UDGold);
    assert_eq!(cache.get("running").unwrap().lemma, "run_gold");

    // Lower priority should not override
    cache.insert_simple("running", "run_synth", LemmaSource::Synthesized);
    assert_eq!(cache.get("running").unwrap().lemma, "run_gold"); // Still gold
}

#[test]
fn test_lemma_validator_against_simple_lemmatizer() {
    let sentences = load_dev_sample();
    if sentences.is_empty() {
        println!("Skipping test - no dev data available");
        return;
    }

    assert!(
        sentences.len() >= 10,
        "Should have substantial dev data sample"
    );

    // Create lemmatizer and validator
    let lemmatizer = SimpleLemmatizer::new().unwrap();
    let validator = LemmaValidator::new(true);

    // Validate lemmatizer against gold standard
    let result = validator
        .validate_lemmatizer(&lemmatizer, &sentences)
        .unwrap();

    // Check validation results with real data
    assert!(
        result.total_words > 500,
        "Should validate many words from real data"
    );
    println!(
        "Real data validation results ({} sentences):",
        sentences.len()
    );
    println!("  Total words: {}", result.total_words);
    println!("  Exact matches: {}", result.exact_matches);
    println!("  Accuracy: {:.2}%", result.accuracy * 100.0);
    println!("  Learned irregulars: {}", result.learned_irregulars.len());

    // Should have reasonable accuracy on real data
    assert!(
        result.accuracy > 0.4,
        "Should have >40% accuracy on real English data"
    );
    assert!(
        result.accuracy < 1.0,
        "Shouldn't have perfect accuracy (would indicate stub data)"
    );

    // Export learned mappings
    let learned = validator.export_learned_irregulars(&result);
    println!("  Exported mappings: {}", learned.len());
    assert!(
        learned.len() > 10,
        "Should learn many mappings from real data"
    );

    for (word, lemma) in learned.iter().take(10) {
        println!("    {} -> {}", word, lemma);
    }
}

#[test]
fn test_lemma_cache_bulk_operations() {
    let cache = LemmaCache::default();

    // Create bulk mappings
    let mut mappings = HashMap::new();
    mappings.insert("running".to_string(), "run".to_string());
    mappings.insert("children".to_string(), "child".to_string());
    mappings.insert("gave".to_string(), "give".to_string());
    mappings.insert("quickly".to_string(), "quickly".to_string());

    // Bulk insert
    cache.insert_bulk(mappings, LemmaSource::UDGold, 0.95);

    // Test all entries
    assert_eq!(cache.get("running").unwrap().lemma, "run");
    assert_eq!(cache.get("children").unwrap().lemma, "child");
    assert_eq!(cache.get("gave").unwrap().lemma, "give");
    assert_eq!(cache.get("quickly").unwrap().lemma, "quickly");

    // Check stats
    let stats = cache.stats();
    assert_eq!(stats.insertions, 4);
    assert_eq!(stats.hits, 4);
}

#[test]
fn test_lemma_validation_accuracy_metrics() {
    let sentences = load_dev_sample();
    if sentences.is_empty() {
        println!("Skipping test - no dev data available");
        return;
    }

    assert!(
        sentences.len() >= 10,
        "Should have substantial dev data sample"
    );

    let lemmatizer = SimpleLemmatizer::new().unwrap();
    let validator = LemmaValidator::new(false);

    // Validate each sentence individually
    for (i, sentence) in sentences.iter().enumerate().take(10) {
        // Test first 10 sentences for speed
        let result = validator.validate_sentence(&lemmatizer, sentence).unwrap();

        if result.total_words > 0 {
            println!(
                "Sentence {}: {}/{} accuracy ({:.1}%)",
                i + 1,
                result.exact_matches,
                result.total_words,
                result.accuracy * 100.0
            );

            // Check that accuracy is calculated correctly
            let expected_accuracy = result.exact_matches as f32 / result.total_words as f32;
            assert!((result.accuracy - expected_accuracy).abs() < 0.001);
        }
    }
}

#[test]
fn test_shared_cache_integration() {
    let cache_config = LemmaCacheConfig {
        max_entries: 1000,
        min_confidence: 0.1,
        enable_metrics: true,
        ..Default::default()
    };

    let cache = LemmaCache::new(cache_config);

    // Simulate different engines adding to cache
    cache.insert(
        "running".to_string(),
        "run".to_string(),
        LemmaSource::SimpleLemmatizer,
        0.7,
    );
    cache.insert(
        "children".to_string(),
        "child".to_string(),
        LemmaSource::UDGold,
        0.99,
    );
    cache.insert(
        "walking".to_string(),
        "walk".to_string(),
        LemmaSource::Synthesized,
        0.5,
    );

    // Test retrieval and priority
    let entry = cache.get("children").unwrap();
    assert_eq!(entry.source, LemmaSource::UDGold);
    assert_eq!(entry.confidence, 0.99);

    // Test memory usage tracking
    let memory_usage = cache.memory_usage();
    assert!(memory_usage > 0);
    println!("Cache memory usage: {} bytes", memory_usage);

    // Test cache statistics by source
    let stats = cache.stats();
    println!("Cache statistics:");
    println!("  Total requests: {}", stats.total_requests());
    println!("  Hit rate: {:.2}%", stats.hit_rate() * 100.0);

    for (source, count) in &stats.hits_by_source {
        println!("  Hits from {}: {}", source, count);
    }
}

#[test]
fn test_lemma_confidence_scoring() {
    let _cache = LemmaCache::default();

    // Test confidence-based filtering
    let low_conf_config = LemmaCacheConfig {
        min_confidence: 0.8, // High threshold
        ..Default::default()
    };
    let filtered_cache = LemmaCache::new(low_conf_config);

    // Low confidence should not be cached
    filtered_cache.insert(
        "test1".to_string(),
        "test1".to_string(),
        LemmaSource::Synthesized,
        0.3,
    );
    assert!(filtered_cache.get("test1").is_none());

    // High confidence should be cached
    filtered_cache.insert(
        "test2".to_string(),
        "test2".to_string(),
        LemmaSource::UDGold,
        0.95,
    );
    assert!(filtered_cache.get("test2").is_some());

    // Check confidence values
    let entry = filtered_cache.get("test2").unwrap();
    assert_eq!(entry.confidence, 0.95);
}

#[test]
fn test_treebank_engine_basic_integration() {
    // Test with actual data path if available, otherwise skip
    let data_path = std::path::Path::new("data/ud_english-ewt/UD_English-EWT");
    if !data_path.exists() {
        println!(
            "Skipping test - UD English-EWT data not found at {:?}",
            data_path
        );
        return;
    }

    let config = TreebankConfig {
        data_path: data_path.to_path_buf(),
        index_path: None,
        min_frequency: 1,
        enable_synthesis: true,
        verbose: false,
        export_lemma_mappings: true,
        validate_lemmatization: true,
        ..TreebankConfig::default()
    };

    let engine = TreebankEngine::with_config(config);
    assert!(
        engine.is_ok(),
        "TreebankEngine should initialize successfully"
    );

    let engine = engine.unwrap();
    assert!(engine.is_initialized());

    // Test basic analysis with real words likely to be in the training data
    let test_words = ["run", "child", "give", "walk", "book"];
    for word in &test_words {
        let result = engine.analyze_word(word);
        assert!(result.is_ok(), "Should analyze '{}' successfully", word);

        let analysis = result.unwrap();
        assert!(
            analysis.confidence >= 0.0 && analysis.confidence <= 1.0,
            "Confidence for '{}' should be between 0 and 1, got {}",
            word,
            analysis.confidence
        );
    }
}

#[test]
#[ignore] // This test requires the actual dev data file
fn test_real_dev_data_validation() {
    use std::path::Path;

    let dev_data_path = Path::new("data/ud_english-ewt/UD_English-EWT/en_ewt-ud-dev.conllu");

    if !dev_data_path.exists() {
        println!("Skipping real dev data test - file not found");
        return;
    }

    let parser = ConlluParser::new(false);
    let sentences = parser.parse_file(dev_data_path).unwrap();

    assert!(
        sentences.len() > 1000,
        "Should have substantial number of sentences"
    );

    let lemmatizer = SimpleLemmatizer::new().unwrap();
    let validator = LemmaValidator::new(false);

    // Test on first 100 sentences for speed
    let sample_sentences: Vec<_> = sentences.into_iter().take(100).collect();
    let result = validator
        .validate_lemmatizer(&lemmatizer, &sample_sentences)
        .unwrap();

    println!("Real dev data validation results (first 100 sentences):");
    println!("  Total words: {}", result.total_words);
    println!("  Accuracy: {:.2}%", result.accuracy * 100.0);
    println!("  Learned patterns: {}", result.learned_irregulars.len());

    // Should have reasonable accuracy
    assert!(
        result.accuracy > 0.4,
        "Should have at least 40% accuracy on real data"
    );
    assert!(
        result.learned_irregulars.len() > 0,
        "Should learn some irregular patterns"
    );
}
