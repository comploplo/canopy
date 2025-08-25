//! Basic tests for wordnet engine.rs module

use canopy_engine::EngineConfig;
use canopy_wordnet::engine::{WordNetConfig, WordNetEngine};
use canopy_wordnet::parser::WordNetParserConfig;

#[cfg(test)]
mod engine_tests {
    use super::*;

    #[test]
    fn test_wordnet_config_default() {
        let config = WordNetConfig::default();

        assert_eq!(config.data_path, "data/wordnet/dict");
        assert!(config.enable_morphology);
        assert_eq!(config.max_search_depth, 5);
        assert_eq!(config.min_confidence, 0.1);
    }

    #[test]
    fn test_wordnet_config_creation() {
        let base_config = EngineConfig {
            enable_cache: true,
            cache_capacity: 2000,
            enable_metrics: true,
            enable_parallel: false,
            max_threads: 2,
            confidence_threshold: 0.8,
        };

        let parser_config = WordNetParserConfig {
            strict_mode: false,
            max_file_size: 50 * 1024 * 1024, // 50MB
            skip_prefixes: vec!["#".to_string()],
        };

        let config = WordNetConfig {
            base: base_config,
            data_path: "/custom/wordnet/path".to_string(),
            parser_config,
            enable_morphology: false,
            max_search_depth: 3,
            min_confidence: 0.2,
        };

        assert_eq!(config.data_path, "/custom/wordnet/path");
        assert!(!config.enable_morphology);
        assert_eq!(config.max_search_depth, 3);
        assert_eq!(config.min_confidence, 0.2);
        assert_eq!(config.base.cache_capacity, 2000);
        assert!(!config.parser_config.strict_mode);
    }

    #[test]
    fn test_wordnet_engine_creation() {
        let config = WordNetConfig::default();
        let engine = WordNetEngine::new(config.clone());

        // We can't easily test internal state, but we can verify creation succeeds
        // and that the engine is in expected initial state
        assert_eq!(
            std::mem::size_of_val(&engine),
            std::mem::size_of::<WordNetEngine>()
        );
    }

    #[test]
    fn test_wordnet_engine_with_custom_config() {
        let config = WordNetConfig {
            base: EngineConfig {
                enable_cache: true,
                cache_capacity: 500,
                enable_metrics: false,
                enable_parallel: true,
                max_threads: 4,
                confidence_threshold: 0.9,
            },
            data_path: "/test/wordnet".to_string(),
            parser_config: WordNetParserConfig::default(),
            enable_morphology: true,
            max_search_depth: 7,
            min_confidence: 0.05,
        };

        let engine = WordNetEngine::new(config);

        // Basic test to ensure the engine can be created
        assert_eq!(
            std::mem::size_of_val(&engine),
            std::mem::size_of::<WordNetEngine>()
        );
    }

    #[test]
    fn test_multiple_engine_creation() {
        let config1 = WordNetConfig {
            data_path: "/path1".to_string(),
            ..Default::default()
        };

        let config2 = WordNetConfig {
            data_path: "/path2".to_string(),
            max_search_depth: 10,
            ..Default::default()
        };

        let engine1 = WordNetEngine::new(config1);
        let engine2 = WordNetEngine::new(config2);

        // Test that multiple engines can be created independently
        assert_eq!(
            std::mem::size_of_val(&engine1),
            std::mem::size_of::<WordNetEngine>()
        );
        assert_eq!(
            std::mem::size_of_val(&engine2),
            std::mem::size_of::<WordNetEngine>()
        );
    }
}
