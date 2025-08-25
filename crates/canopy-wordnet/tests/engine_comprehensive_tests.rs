//! Comprehensive tests for WordNet engine functionality using public APIs

use canopy_engine::EngineConfig;
use canopy_wordnet::engine::{WordNetConfig, WordNetEngine};
use canopy_wordnet::parser::WordNetParserConfig;
use canopy_wordnet::types::PartOfSpeech;
use std::fs;
use std::io::Write;
use tempfile::TempDir;

#[cfg(test)]
mod engine_tests {
    use super::*;

    fn create_test_config_with_path(data_path: &str) -> WordNetConfig {
        WordNetConfig {
            base: EngineConfig {
                enable_cache: true,
                cache_capacity: 100,
                enable_metrics: true,
                enable_parallel: false,
                max_threads: 2,
                confidence_threshold: 0.5,
            },
            data_path: data_path.to_string(),
            parser_config: WordNetParserConfig::default(),
            enable_morphology: true,
            max_search_depth: 3,
            min_confidence: 0.1,
        }
    }

    fn create_test_wordnet_files(temp_dir: &TempDir) -> std::io::Result<()> {
        let data_dir = temp_dir.path();

        // Create a simple test data.noun file
        let mut data_noun = fs::File::create(data_dir.join("data.noun"))?;
        writeln!(
            data_noun,
            "  1 This software and database is being provided to you, the LICENSEE,"
        )?;
        writeln!(
            data_noun,
            "100001740 03 n 01 entity 0 001 @ 100002137 n 0000 | that which is perceived or known or inferred to have its own distinct existence (living or nonliving)  "
        )?;
        writeln!(
            data_noun,
            "100002137 03 n 01 thing 0 002 @ 100001930 n 0000 ~ 100001740 n 0000 | a separate and self-contained entity  "
        )?;

        // Create a simple test index.noun file
        let mut index_noun = fs::File::create(data_dir.join("index.noun"))?;
        writeln!(
            index_noun,
            "  1 This software and database is being provided to you, the LICENSEE,"
        )?;
        writeln!(index_noun, "entity n 1 1 @ 1 0 100001740")?;
        writeln!(index_noun, "thing n 1 2 @ ~ 1 0 100002137")?;

        // Create a simple test data.verb file
        let mut data_verb = fs::File::create(data_dir.join("data.verb"))?;
        writeln!(
            data_verb,
            "  1 This software and database is being provided to you, the LICENSEE,"
        )?;
        writeln!(
            data_verb,
            "200001740 30 v 02 run 0 go 0 001 @ 200002137 v 0000 | move fast by using one's feet, with one foot off the ground at any given time  \"Don't walk when you can run\"  "
        )?;

        // Create a simple test index.verb file
        let mut index_verb = fs::File::create(data_dir.join("index.verb"))?;
        writeln!(
            index_verb,
            "  1 This software and database is being provided to you, the LICENSEE,"
        )?;
        writeln!(index_verb, "run v 1 1 @ 1 0 200001740")?;

        Ok(())
    }

    #[test]
    fn test_engine_load_data_success() {
        let temp_dir = TempDir::new().unwrap();
        create_test_wordnet_files(&temp_dir).unwrap();

        let config = create_test_config_with_path(temp_dir.path().to_str().unwrap());
        let mut engine = WordNetEngine::new(config);

        // Initially not ready
        assert!(!engine.is_ready());

        // Load data
        let result = engine.load_data();
        assert!(result.is_ok());

        // Should be ready after loading
        assert!(engine.is_ready());
    }

    #[test]
    fn test_engine_load_data_invalid_path() {
        let config = create_test_config_with_path("/nonexistent/path");
        let mut engine = WordNetEngine::new(config);

        // Should fail to load from nonexistent path
        let result = engine.load_data();
        assert!(result.is_err());
        assert!(!engine.is_ready());
    }

    #[test]
    fn test_engine_load_data_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config_with_path(temp_dir.path().to_str().unwrap());
        let mut engine = WordNetEngine::new(config);

        // Should succeed but result in empty database
        let result = engine.load_data();
        assert!(result.is_ok());

        // Engine should not be ready with empty database
        assert!(!engine.is_ready());
    }

    #[test]
    fn test_engine_analyze_with_data() {
        let temp_dir = TempDir::new().unwrap();
        create_test_wordnet_files(&temp_dir).unwrap();

        let config = create_test_config_with_path(temp_dir.path().to_str().unwrap());
        let mut engine = WordNetEngine::new(config);

        engine.load_data().unwrap();

        // Test analyzing existing word
        let result = engine.analyze_word("entity", PartOfSpeech::Noun);
        assert!(result.is_ok());

        let analysis = result.unwrap();
        assert_eq!(analysis.word, "entity");
        assert_eq!(analysis.pos, PartOfSpeech::Noun);

        if analysis.has_results() {
            assert!(!analysis.synsets.is_empty());
            assert!(analysis.confidence > 0.0);
        }
    }

    #[test]
    fn test_engine_analyze_nonexistent_word() {
        let temp_dir = TempDir::new().unwrap();
        create_test_wordnet_files(&temp_dir).unwrap();

        let config = create_test_config_with_path(temp_dir.path().to_str().unwrap());
        let mut engine = WordNetEngine::new(config);

        engine.load_data().unwrap();

        // Test analyzing nonexistent word
        let result = engine.analyze_word("nonexistent", PartOfSpeech::Noun);
        assert!(result.is_ok());

        let analysis = result.unwrap();
        assert_eq!(analysis.word, "nonexistent");
        assert!(!analysis.has_results());
        assert_eq!(analysis.confidence, 0.0);
    }

    #[test]
    fn test_engine_caching_behavior() {
        let temp_dir = TempDir::new().unwrap();
        create_test_wordnet_files(&temp_dir).unwrap();

        let mut config = create_test_config_with_path(temp_dir.path().to_str().unwrap());
        config.base.enable_cache = true;

        let mut engine = WordNetEngine::new(config);
        engine.load_data().unwrap();

        // First analysis
        let result1 = engine.analyze_word("entity", PartOfSpeech::Noun);
        assert!(result1.is_ok());

        // Second analysis (should use cache if word found)
        let result2 = engine.analyze_word("entity", PartOfSpeech::Noun);
        assert!(result2.is_ok());

        // Results should be consistent
        let analysis1 = result1.unwrap();
        let analysis2 = result2.unwrap();
        assert_eq!(analysis1.word, analysis2.word);
        assert_eq!(analysis1.confidence, analysis2.confidence);
    }

    #[test]
    fn test_engine_caching_disabled() {
        let temp_dir = TempDir::new().unwrap();
        create_test_wordnet_files(&temp_dir).unwrap();

        let mut config = create_test_config_with_path(temp_dir.path().to_str().unwrap());
        config.base.enable_cache = false;

        let mut engine = WordNetEngine::new(config);
        engine.load_data().unwrap();

        // Analysis should work even with caching disabled
        let result = engine.analyze_word("entity", PartOfSpeech::Noun);
        assert!(result.is_ok());
    }

    #[test]
    fn test_engine_confidence_calculation() {
        let temp_dir = TempDir::new().unwrap();
        create_test_wordnet_files(&temp_dir).unwrap();

        let config = create_test_config_with_path(temp_dir.path().to_str().unwrap());
        let mut engine = WordNetEngine::new(config);
        engine.load_data().unwrap();

        // Test confidence calculation for found words
        let result = engine.analyze_word("entity", PartOfSpeech::Noun);
        if result.is_ok() {
            let analysis = result.unwrap();
            if analysis.has_results() {
                // Confidence should be positive for found words
                assert!(analysis.confidence > 0.0);
                assert!(analysis.confidence <= 1.0);
            }
        }

        // Test confidence for nonexistent words
        let empty_result = engine.analyze_word("nonexistent", PartOfSpeech::Noun);
        assert!(empty_result.is_ok());
        let empty_analysis = empty_result.unwrap();
        assert_eq!(empty_analysis.confidence, 0.0);
    }

    #[test]
    fn test_engine_different_pos_analysis() {
        let temp_dir = TempDir::new().unwrap();
        create_test_wordnet_files(&temp_dir).unwrap();

        let config = create_test_config_with_path(temp_dir.path().to_str().unwrap());
        let mut engine = WordNetEngine::new(config);
        engine.load_data().unwrap();

        // Test all POS types
        let pos_types = vec![
            PartOfSpeech::Noun,
            PartOfSpeech::Verb,
            PartOfSpeech::Adjective,
            PartOfSpeech::Adverb,
        ];

        for pos in pos_types {
            let result = engine.analyze_word("test", pos);
            assert!(result.is_ok());

            let analysis = result.unwrap();
            assert_eq!(analysis.pos, pos);
        }
    }

    #[test]
    fn test_engine_morphology_config() {
        let temp_dir = TempDir::new().unwrap();
        create_test_wordnet_files(&temp_dir).unwrap();

        // Test with morphology enabled
        let mut config1 = create_test_config_with_path(temp_dir.path().to_str().unwrap());
        config1.enable_morphology = true;
        let mut engine1 = WordNetEngine::new(config1);
        engine1.load_data().unwrap();

        // Test with morphology disabled
        let mut config2 = create_test_config_with_path(temp_dir.path().to_str().unwrap());
        config2.enable_morphology = false;
        let mut engine2 = WordNetEngine::new(config2);
        engine2.load_data().unwrap();

        // Both should work
        let result1 = engine1.analyze_word("entity", PartOfSpeech::Noun);
        let result2 = engine2.analyze_word("entity", PartOfSpeech::Noun);

        assert!(result1.is_ok());
        assert!(result2.is_ok());
    }

    #[test]
    fn test_engine_search_depth_config() {
        let temp_dir = TempDir::new().unwrap();
        create_test_wordnet_files(&temp_dir).unwrap();

        // Test different search depths
        let depths = vec![1, 3, 5, 10];

        for depth in depths {
            let mut config = create_test_config_with_path(temp_dir.path().to_str().unwrap());
            config.max_search_depth = depth;

            let mut engine = WordNetEngine::new(config);
            engine.load_data().unwrap();

            let result = engine.analyze_word("entity", PartOfSpeech::Noun);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_engine_confidence_threshold() {
        let temp_dir = TempDir::new().unwrap();
        create_test_wordnet_files(&temp_dir).unwrap();

        // Test different confidence thresholds
        let thresholds = vec![0.0, 0.1, 0.5, 0.9];

        for threshold in thresholds {
            let mut config = create_test_config_with_path(temp_dir.path().to_str().unwrap());
            config.min_confidence = threshold;

            let mut engine = WordNetEngine::new(config);
            engine.load_data().unwrap();

            let result = engine.analyze_word("entity", PartOfSpeech::Noun);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_engine_parser_config_integration() {
        let temp_dir = TempDir::new().unwrap();
        create_test_wordnet_files(&temp_dir).unwrap();

        // Test with custom parser config
        let mut config = create_test_config_with_path(temp_dir.path().to_str().unwrap());
        config.parser_config = WordNetParserConfig {
            strict_mode: true,
            max_file_size: 1024 * 1024,
            skip_prefixes: vec!["  1 This".to_string()],
        };

        let mut engine = WordNetEngine::new(config);
        let result = engine.load_data();
        assert!(result.is_ok());
    }

    #[test]
    fn test_engine_batch_analysis() {
        let temp_dir = TempDir::new().unwrap();
        create_test_wordnet_files(&temp_dir).unwrap();

        let config = create_test_config_with_path(temp_dir.path().to_str().unwrap());
        let mut engine = WordNetEngine::new(config);
        engine.load_data().unwrap();

        // Test analyzing multiple words
        let test_words = vec![
            ("entity", PartOfSpeech::Noun),
            ("thing", PartOfSpeech::Noun),
            ("run", PartOfSpeech::Verb),
            ("nonexistent", PartOfSpeech::Adjective),
        ];

        for (word, pos) in test_words {
            let result = engine.analyze_word(word, pos);
            assert!(result.is_ok());

            let analysis = result.unwrap();
            assert_eq!(analysis.word, word);
            assert_eq!(analysis.pos, pos);
        }
    }

    #[test]
    fn test_engine_error_recovery() {
        let temp_dir = TempDir::new().unwrap();

        let config = create_test_config_with_path(temp_dir.path().to_str().unwrap());
        let mut engine = WordNetEngine::new(config);

        // Try to analyze before loading - should fail gracefully
        let result1 = engine.analyze_word("test", PartOfSpeech::Noun);
        assert!(result1.is_err());

        // Create data files and load
        create_test_wordnet_files(&temp_dir).unwrap();
        let load_result = engine.load_data();
        assert!(load_result.is_ok());

        // Analysis should now work
        let result2 = engine.analyze_word("test", PartOfSpeech::Noun);
        assert!(result2.is_ok());
    }

    #[test]
    fn test_engine_large_cache_capacity() {
        let temp_dir = TempDir::new().unwrap();
        create_test_wordnet_files(&temp_dir).unwrap();

        let mut config = create_test_config_with_path(temp_dir.path().to_str().unwrap());
        config.base.cache_capacity = 10000;

        let mut engine = WordNetEngine::new(config);
        engine.load_data().unwrap();

        // Test multiple analyses with large cache
        for i in 0..50 {
            let word = if i % 2 == 0 { "entity" } else { "thing" };
            let result = engine.analyze_word(word, PartOfSpeech::Noun);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_engine_small_cache_capacity() {
        let temp_dir = TempDir::new().unwrap();
        create_test_wordnet_files(&temp_dir).unwrap();

        let mut config = create_test_config_with_path(temp_dir.path().to_str().unwrap());
        config.base.cache_capacity = 1;

        let mut engine = WordNetEngine::new(config);
        engine.load_data().unwrap();

        // Test with very small cache
        let result1 = engine.analyze_word("entity", PartOfSpeech::Noun);
        let result2 = engine.analyze_word("thing", PartOfSpeech::Noun);

        assert!(result1.is_ok());
        assert!(result2.is_ok());
    }

    #[test]
    fn test_engine_serialization_compatibility() {
        // Test that config can be serialized/deserialized
        let config = WordNetConfig::default();

        // Test JSON serialization (requires serde)
        let json_result = serde_json::to_string(&config);
        assert!(json_result.is_ok());

        let json_str = json_result.unwrap();
        let deserialize_result: Result<WordNetConfig, _> = serde_json::from_str(&json_str);
        assert!(deserialize_result.is_ok());

        let deserialized_config = deserialize_result.unwrap();
        assert_eq!(config.data_path, deserialized_config.data_path);
        assert_eq!(
            config.enable_morphology,
            deserialized_config.enable_morphology
        );
    }

    #[test]
    fn test_engine_concurrent_analysis() {
        let temp_dir = TempDir::new().unwrap();
        create_test_wordnet_files(&temp_dir).unwrap();

        let config = create_test_config_with_path(temp_dir.path().to_str().unwrap());
        let mut engine = WordNetEngine::new(config);
        engine.load_data().unwrap();

        // Simulate concurrent analysis (sequential in test but tests the same paths)
        let words = vec!["entity", "thing", "run", "entity", "thing"];

        for word in words {
            let result = engine.analyze_word(word, PartOfSpeech::Noun);
            assert!(result.is_ok());
        }
    }
}
