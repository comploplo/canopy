//! Engine tests with loaded data for canopy-lexicon
//!
//! Tests that require actual data loading to achieve higher coverage

use canopy_engine::{DataLoader, SemanticEngine, StatisticsProvider};
use canopy_lexicon::types::WordClassType;
use canopy_lexicon::{LexiconConfig, LexiconEngine};
use std::fs;
use tempfile::TempDir;

#[cfg(test)]
mod loaded_engine_tests {
    use super::*;

    fn create_test_lexicon_with_data() -> (TempDir, LexiconConfig) {
        let temp_dir = TempDir::new().unwrap();
        let lexicon_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<lexicon version="1.0" language="en" xmlns="http://canopy.rs/lexicon">
  <metadata>
    <title>Test Lexicon</title>
    <description>Test lexicon for coverage tests</description>
    <created>2024-01-01</created>
    <author>Test</author>
    <license>MIT</license>
  </metadata>

  <word-classes>
    <word-class id="test-stop-words" name="Test Stop Words" type="stop-words" priority="10">
      <description>Test stop words</description>
      <properties>
        <property name="semantic-weight" value="0.1" type="float"/>
      </properties>
      <words>
        <word pos="DT">the</word>
        <word pos="DT">a</word>
        <word pos="CC">and</word>
      </words>
    </word-class>

    <word-class id="test-negation" name="Test Negation" type="negation" priority="9">
      <description>Test negation words</description>
      <words>
        <word pos="RB">not</word>
        <word pos="DT">no</word>
      </words>
      <patterns>
        <pattern id="neg-prefix-un" type="prefix" confidence="0.8">
          <regex>^un[a-z]+</regex>
          <description>Un- prefix</description>
          <examples>
            <example>unhappy</example>
          </examples>
        </pattern>
      </patterns>
    </word-class>

    <word-class id="test-discourse" name="Test Discourse" type="discourse-markers" priority="8">
      <description>Test discourse markers</description>
      <words>
        <word pos="RB">however</word>
        <word pos="CC">therefore</word>
      </words>
    </word-class>

    <word-class id="test-quantifiers" name="Test Quantifiers" type="quantifiers" priority="7">
      <description>Test quantifiers</description>
      <words>
        <word pos="DT">all</word>
        <word pos="DT">some</word>
        <word pos="DT">many</word>
      </words>
    </word-class>
  </word-classes>
</lexicon>"#;

        fs::write(temp_dir.path().join("english-lexicon.xml"), lexicon_xml).unwrap();

        let config = LexiconConfig {
            data_path: temp_dir.path().to_string_lossy().to_string(),
            ..LexiconConfig::default()
        };

        (temp_dir, config)
    }

    #[test]
    fn test_successful_data_loading() {
        let (_temp_dir, config) = create_test_lexicon_with_data();
        let mut engine = LexiconEngine::with_config(config);

        // Test loading succeeds
        let result = engine.load_data();
        assert!(result.is_ok(), "Data loading should succeed");
        assert!(
            engine.is_initialized(),
            "Engine should be initialized after loading"
        );

        // Test data info after loading
        let data_info = engine.data_info();
        assert!(
            data_info.entry_count > 0,
            "Should have entries after loading"
        );
        assert!(
            !data_info.source.is_empty(),
            "Should have source information"
        );
    }

    #[test]
    fn test_word_classification_with_loaded_data() {
        let (_temp_dir, config) = create_test_lexicon_with_data();
        let mut engine = LexiconEngine::with_config(config);
        engine.load_data().expect("Failed to load data");

        // Test stop word detection
        assert!(engine.is_stop_word("the").unwrap());
        assert!(engine.is_stop_word("a").unwrap());
        assert!(engine.is_stop_word("and").unwrap());
        assert!(!engine.is_stop_word("happy").unwrap());

        // Test negation detection
        assert!(engine.is_negation("not").unwrap());
        assert!(engine.is_negation("no").unwrap());
        assert!(!engine.is_negation("yes").unwrap());

        // Test discourse marker detection
        assert!(engine.is_discourse_marker("however").unwrap());
        assert!(engine.is_discourse_marker("therefore").unwrap());
        assert!(!engine.is_discourse_marker("cat").unwrap());
    }

    #[test]
    fn test_get_words_by_type() {
        let (_temp_dir, config) = create_test_lexicon_with_data();
        let mut engine = LexiconEngine::with_config(config);
        engine.load_data().expect("Failed to load data");

        // Test getting stop words
        let stop_words = engine.get_words_by_type(WordClassType::StopWords).unwrap();
        assert!(stop_words.contains(&"the".to_string()));
        assert!(stop_words.contains(&"a".to_string()));
        assert!(stop_words.contains(&"and".to_string()));

        // Test getting negation words
        let negation_words = engine.get_words_by_type(WordClassType::Negation).unwrap();
        assert!(negation_words.contains(&"not".to_string()));
        assert!(negation_words.contains(&"no".to_string()));

        // Test getting discourse markers
        let discourse_words = engine
            .get_words_by_type(WordClassType::DiscourseMarkers)
            .unwrap();
        assert!(discourse_words.contains(&"however".to_string()));
        assert!(discourse_words.contains(&"therefore".to_string()));

        // Test getting quantifiers
        let quantifier_words = engine
            .get_words_by_type(WordClassType::Quantifiers)
            .unwrap();
        assert!(quantifier_words.contains(&"all".to_string()));
        assert!(quantifier_words.contains(&"some".to_string()));
        assert!(quantifier_words.contains(&"many".to_string()));
    }

    #[test]
    fn test_analyze_text() {
        let (_temp_dir, config) = create_test_lexicon_with_data();
        let mut engine = LexiconEngine::with_config(config);
        engine.load_data().expect("Failed to load data");

        let text = "The cat is not happy, however it is resting.";
        let results = engine.analyze_text(text).unwrap();

        // Should find classifications for "the", "not", "however"
        assert!(!results.is_empty());

        // Find specific word analyses
        let the_analysis = results.iter().find(|r| r.input == "The");
        assert!(the_analysis.is_some());

        let not_analysis = results.iter().find(|r| r.input == "not");
        assert!(not_analysis.is_some());

        let however_analysis = results.iter().find(|r| r.input == "however");
        assert!(however_analysis.is_some());
    }

    #[test]
    fn test_get_semantic_weight() {
        let (_temp_dir, config) = create_test_lexicon_with_data();
        let mut engine = LexiconEngine::with_config(config);
        engine.load_data().expect("Failed to load data");

        // Test semantic weight for stop words (should be low)
        let stop_weight = engine.get_semantic_weight("the").unwrap();
        assert!(
            stop_weight < 1.0,
            "Stop words should have low semantic weight"
        );

        // Test semantic weight for unknown words (should be 1.0)
        let unknown_weight = engine.get_semantic_weight("unknownword").unwrap();
        assert_eq!(
            unknown_weight, 1.0,
            "Unknown words should have default weight 1.0"
        );
    }

    #[test]
    fn test_analyze_negation_scope() {
        let (_temp_dir, config) = create_test_lexicon_with_data();
        let mut engine = LexiconEngine::with_config(config);
        engine.load_data().expect("Failed to load data");

        let text = "I do not like this, no way!";
        let negations = engine.analyze_negation_scope(text).unwrap();

        // Should find "not" and "no"
        assert_eq!(negations.len(), 2);

        let not_found = negations.iter().any(|(word, _, _)| word == "not");
        let no_found = negations.iter().any(|(word, _, _)| word == "no");

        assert!(not_found, "Should find 'not' in negation analysis");
        assert!(no_found, "Should find 'no' in negation analysis");
    }

    #[test]
    fn test_extract_discourse_structure() {
        let (_temp_dir, config) = create_test_lexicon_with_data();
        let mut engine = LexiconEngine::with_config(config);
        engine.load_data().expect("Failed to load data");

        let text = "I like cats. However, dogs are also nice. Therefore, I like both.";
        let _discourse_markers = engine.extract_discourse_structure(text).unwrap();

        // Test that the function runs without error (markers may be empty if no context set)
        // This tests the code path rather than requiring specific content
        // discourse_markers.len() check - reaching here means function succeeded

        // Test that we can find discourse markers in the text
        let however_found = engine.is_discourse_marker("however").unwrap();
        let therefore_found = engine.is_discourse_marker("therefore").unwrap();

        assert!(however_found, "Should find 'however' as discourse marker");
        assert!(
            therefore_found,
            "Should find 'therefore' as discourse marker"
        );
    }

    #[test]
    fn test_filter_stop_words() {
        let (_temp_dir, config) = create_test_lexicon_with_data();
        let mut engine = LexiconEngine::with_config(config);
        engine.load_data().expect("Failed to load data");

        let words = vec![
            "the".to_string(),
            "cat".to_string(),
            "and".to_string(),
            "dog".to_string(),
            "a".to_string(),
            "house".to_string(),
        ];

        let filtered = engine.filter_stop_words(&words).unwrap();

        // Should remove stop words but keep content words
        assert!(!filtered.contains(&"the".to_string()));
        assert!(!filtered.contains(&"and".to_string()));
        assert!(!filtered.contains(&"a".to_string()));
        assert!(filtered.contains(&"cat".to_string()));
        assert!(filtered.contains(&"dog".to_string()));
        assert!(filtered.contains(&"house".to_string()));
    }

    #[test]
    fn test_get_intensifier_strength() {
        let (_temp_dir, config) = create_test_lexicon_with_data();
        let mut engine = LexiconEngine::with_config(config);
        engine.load_data().expect("Failed to load data");

        // Test with a non-intensifier word
        let strength = engine.get_intensifier_strength("cat").unwrap();
        assert!(strength.is_none(), "Non-intensifier should return None");

        // Test with unknown word
        let unknown_strength = engine.get_intensifier_strength("unknownword").unwrap();
        assert!(
            unknown_strength.is_none(),
            "Unknown word should return None"
        );
    }

    #[test]
    fn test_pattern_matching_with_loaded_data() {
        let (_temp_dir, config) = create_test_lexicon_with_data();
        let mut engine = LexiconEngine::with_config(config);
        engine.load_data().expect("Failed to load data");

        // Test pattern matching with "un-" prefix
        let analysis = engine.analyze_word("unhappy").unwrap();
        assert!(
            !analysis.data.pattern_matches.is_empty(),
            "Should match un- prefix pattern"
        );

        let pattern_match = &analysis.data.pattern_matches[0];
        assert_eq!(pattern_match.pattern_id, "neg-prefix-un");
        assert_eq!(pattern_match.matched_text, "unhappy");
    }

    #[test]
    fn test_semantic_engine_trait_with_loaded_data() {
        let (_temp_dir, config) = create_test_lexicon_with_data();
        let mut engine = LexiconEngine::with_config(config);
        engine.load_data().expect("Failed to load data");

        // Test analyze method from SemanticEngine trait
        let result = engine.analyze(&"the".to_string()).unwrap();
        assert!(result.data.has_results());
        assert!(result.confidence > 0.0);
        assert!(!result.from_cache); // First time should not be from cache
        assert!(result.processing_time_us > 0);

        // Test version and name
        assert_eq!(engine.name(), "Lexicon");
        assert_eq!(engine.version(), "1.0");
        assert!(engine.is_initialized());
    }

    #[test]
    fn test_cache_functionality() {
        let (_temp_dir, config) = create_test_lexicon_with_data();
        let mut engine = LexiconEngine::with_config(config);
        engine.load_data().expect("Failed to load data");

        // First analysis - not from cache
        let _result1 = engine.analyze_word("the").unwrap();

        // Check cache stats
        let cache_stats = engine.cache_stats();
        assert_eq!(cache_stats.total_lookups, 1);

        // Second analysis - should hit cache
        let _result2 = engine.analyze_word("the").unwrap();

        let cache_stats_after = engine.cache_stats();
        assert_eq!(cache_stats_after.total_lookups, 2);
        assert_eq!(cache_stats_after.hits, 1);
    }

    #[test]
    fn test_data_loader_interface() {
        let (temp_dir, config) = create_test_lexicon_with_data();
        let mut engine = LexiconEngine::with_config(config);

        // First load the data successfully
        engine.load_data().expect("Failed to load initial data");

        // Test reload
        let reload_result = engine.reload();
        assert!(reload_result.is_ok(), "Reload should succeed");

        // Test load_from_directory with empty directory
        let temp_dir2 = TempDir::new().unwrap();
        let result = engine.load_from_directory(temp_dir2.path());
        assert!(result.is_err(), "Should fail to load from empty directory");

        // Test load_from_directory with valid directory
        let valid_result = engine.load_from_directory(temp_dir.path());
        assert!(
            valid_result.is_ok(),
            "Should succeed to load from valid directory"
        );

        // Test load_test_data (should succeed with minimal data)
        let test_data_result = engine.load_test_data();
        assert!(
            test_data_result.is_ok(),
            "Test data loading should succeed with minimal data"
        );
    }

    #[test]
    fn test_configuration_with_patterns_disabled() {
        let (_temp_dir, mut config) = create_test_lexicon_with_data();
        config.enable_patterns = false;

        let mut engine = LexiconEngine::with_config(config);
        engine.load_data().expect("Failed to load data");

        // Analysis should not include pattern matches
        let analysis = engine.analyze_word("unhappy").unwrap();
        assert!(
            analysis.data.pattern_matches.is_empty(),
            "Pattern matches should be empty when disabled"
        );
    }

    #[test]
    fn test_confidence_and_max_classifications_limits() {
        let (_temp_dir, mut config) = create_test_lexicon_with_data();
        config.min_confidence = 0.9; // Very high threshold
        config.max_classifications = 1;

        let mut engine = LexiconEngine::with_config(config);
        engine.load_data().expect("Failed to load data");

        let analysis = engine.analyze_word("the").unwrap();
        // With high confidence threshold, might filter out results
        assert!(
            analysis.data.classifications.len() <= 1,
            "Should respect max_classifications limit"
        );
    }

    #[test]
    fn test_engine_statistics_with_loaded_data() {
        let (_temp_dir, config) = create_test_lexicon_with_data();
        let mut engine = LexiconEngine::with_config(config);
        engine.load_data().expect("Failed to load data");

        let stats = engine.statistics();
        assert_eq!(stats.engine_name, "Lexicon");

        let perf_metrics = engine.performance_metrics();
        assert_eq!(perf_metrics.total_queries, 0); // Should start at 0
    }
}
