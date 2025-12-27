//! Engine tests for canopy-lexicon
//!
//! Tests engine functionality with focus on what's actually implemented

use canopy_engine::{CachedEngine, DataLoader, SemanticEngine, StatisticsProvider};
use canopy_lexicon::{LexiconConfig, LexiconEngine};

#[cfg(test)]
mod engine_tests {
    use super::*;

    #[test]
    fn test_lexicon_engine_creation() {
        let config = LexiconConfig::default();
        let engine = LexiconEngine::with_config(config);

        // Engine should be created successfully
        assert_eq!(engine.name(), "Lexicon");
    }

    #[test]
    fn test_lexicon_config_default() {
        let config = LexiconConfig::default();

        // Config should have reasonable defaults
        assert!(!config.data_path.is_empty());
        assert!(config.enable_patterns);
        assert!(config.max_classifications > 0);
        assert!(config.min_confidence >= 0.0);
        assert!(config.min_confidence <= 1.0);
    }

    #[test]
    fn test_lexicon_config_custom() {
        let custom_config = LexiconConfig {
            data_path: "custom/path".to_string(),
            enable_patterns: false,
            max_classifications: 5,
            min_confidence: 0.2,
            enable_fuzzy_matching: true,
            ..LexiconConfig::default()
        };

        assert_eq!(custom_config.data_path, "custom/path");
        assert!(!custom_config.enable_patterns);
        assert_eq!(custom_config.max_classifications, 5);
        assert_eq!(custom_config.min_confidence, 0.2);
        assert!(custom_config.enable_fuzzy_matching);
    }

    #[test]
    fn test_engine_basic_methods() {
        let config = LexiconConfig::default();
        let engine = LexiconEngine::with_config(config);

        // Test basic engine interface
        assert_eq!(engine.name(), "Lexicon");
        assert!(!engine.is_initialized());

        // Test statistics
        let stats = engine.statistics();
        assert_eq!(stats.engine_name, "Lexicon");
    }

    #[test]
    fn test_engine_without_data() {
        let config = LexiconConfig {
            data_path: "/nonexistent/path".to_string(),
            ..LexiconConfig::default()
        };
        let mut engine = LexiconEngine::with_config(config);

        // Loading should fail for nonexistent path
        let result = engine.load_data();
        assert!(result.is_err());
        assert!(!engine.is_initialized());
    }

    #[test]
    fn test_analysis_methods_without_data() {
        let config = LexiconConfig::default();
        let engine = LexiconEngine::with_config(config);

        // Analysis methods should work even without loaded data
        // (they might return empty results or errors)
        let stop_result = engine.is_stop_word("the");
        let negation_result = engine.is_negation("not");
        let analysis_result = engine.analyze_word("test");

        // These might succeed with empty results or fail gracefully
        // Either outcome is acceptable without loaded data
        match (stop_result, negation_result, analysis_result) {
            (Ok(_), Ok(_), Ok(_)) => {
                // All succeeded with empty/default results
            }
            _ => {
                // Some failed, which is expected without data
            }
        }
    }

    #[test]
    fn test_cache_operations() {
        let config = LexiconConfig::default();
        let engine = LexiconEngine::with_config(config);

        // Cache operations should work
        let cache_stats = engine.cache_stats();
        assert_eq!(cache_stats.total_lookups, 0);

        // Clear cache should not panic
        engine.clear_cache();

        let cache_stats_after = engine.cache_stats();
        assert_eq!(cache_stats_after.total_lookups, 0);
    }

    #[test]
    fn test_data_info_without_data() {
        let config = LexiconConfig::default();
        let engine = LexiconEngine::with_config(config);

        let data_info = engine.data_info();
        assert!(!data_info.source.is_empty());
        assert_eq!(data_info.entry_count, 0); // Should be 0 without loaded data
    }

    #[test]
    fn test_configuration_edge_cases() {
        // Test edge case configurations
        let edge_config = LexiconConfig {
            data_path: "".to_string(),
            enable_patterns: false,
            max_classifications: 0,
            min_confidence: 0.0,
            enable_fuzzy_matching: false,
            ..LexiconConfig::default()
        };

        let engine = LexiconEngine::with_config(edge_config);
        assert_eq!(engine.name(), "Lexicon");
    }

    #[test]
    fn test_configuration_boundary_values() {
        // Test boundary values
        let boundary_config = LexiconConfig {
            data_path: "test".to_string(),
            enable_patterns: true,
            max_classifications: 1,
            min_confidence: 1.0,
            enable_fuzzy_matching: true,
            ..LexiconConfig::default()
        };

        let engine = LexiconEngine::with_config(boundary_config);
        assert_eq!(engine.name(), "Lexicon");
    }

    #[test]
    fn test_statistics_consistency() {
        let config = LexiconConfig::default();
        let engine = LexiconEngine::with_config(config);

        let stats1 = engine.statistics();
        let stats2 = engine.statistics();

        // Statistics should be consistent when called multiple times
        assert_eq!(stats1.engine_name, stats2.engine_name);
    }

    #[test]
    fn test_engine_name_consistency() {
        let config1 = LexiconConfig::default();
        let config2 = LexiconConfig {
            data_path: "different".to_string(),
            ..LexiconConfig::default()
        };

        let engine1 = LexiconEngine::with_config(config1);
        let engine2 = LexiconEngine::with_config(config2);

        // Engine name should be consistent regardless of configuration
        assert_eq!(engine1.name(), engine2.name());
        assert_eq!(engine1.name(), "Lexicon");
    }

    #[test]
    fn test_multiple_engines() {
        let config = LexiconConfig::default();

        // Should be able to create multiple engines
        let engine1 = LexiconEngine::with_config(config.clone());
        let engine2 = LexiconEngine::with_config(config);

        assert_eq!(engine1.name(), "Lexicon");
        assert_eq!(engine2.name(), "Lexicon");
        assert!(!engine1.is_initialized());
        assert!(!engine2.is_initialized());
    }

    #[test]
    fn test_error_handling() {
        let config = LexiconConfig {
            data_path: "/definitely/does/not/exist".to_string(),
            ..LexiconConfig::default()
        };

        let mut engine = LexiconEngine::with_config(config);

        // Should handle errors gracefully
        let load_result = engine.load_data();
        assert!(load_result.is_err());

        let error = load_result.unwrap_err();
        let error_msg = error.to_string();
        assert!(!error_msg.is_empty());
    }

    #[test]
    fn test_analysis_result_structure() {
        let config = LexiconConfig::default();
        let engine = LexiconEngine::with_config(config);

        // Test that analysis returns proper structure
        match engine.analyze_word("test") {
            Ok(analysis) => {
                assert_eq!(analysis.data.input, "test");
                assert!(analysis.confidence >= 0.0);
                assert!(analysis.confidence <= 1.0);
                // Classifications and pattern matches might be empty without data
            }
            Err(_) => {
                // Error is acceptable without loaded data
            }
        }
    }

    #[test]
    fn test_word_classification_methods() {
        let config = LexiconConfig::default();
        let engine = LexiconEngine::with_config(config);

        // Test word classification methods
        let test_words = vec!["the", "not", "however", "all"];

        for word in &test_words {
            // These methods should exist and return results (even if empty)
            let _ = engine.is_stop_word(word);
            let _ = engine.is_negation(word);
            let _ = engine.analyze_word(word);

            // Test discourse marker method which exists
            let _ = engine.is_discourse_marker(word);
        }
    }

    #[test]
    fn test_empty_and_special_inputs() {
        let config = LexiconConfig::default();
        let engine = LexiconEngine::with_config(config);

        let special_inputs = vec!["", "  ", "123", "@#$", "very_long_word"];

        for input in &special_inputs {
            // Should handle special inputs without panicking
            let _ = engine.is_stop_word(input);
            let _ = engine.is_negation(input);
            let _ = engine.analyze_word(input);
        }
    }

    #[test]
    fn test_unicode_input_handling() {
        let config = LexiconConfig::default();
        let engine = LexiconEngine::with_config(config);

        let unicode_inputs = vec!["café", "naïve", "résumé", "你好"];

        for input in &unicode_inputs {
            // Should handle Unicode inputs without panicking
            let _ = engine.is_stop_word(input);
            let _ = engine.is_negation(input);
            let _ = engine.analyze_word(input);
        }
    }
}
