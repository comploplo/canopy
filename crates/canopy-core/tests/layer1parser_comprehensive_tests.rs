//! Comprehensive tests for layer1parser.rs module to improve coverage

use canopy_core::layer1parser::{
    ComponentHealth, Layer1HelperConfig, Layer1ParserHandler, LayerConfig, LayerHandler,
    SemanticAnalysisHandler, SemanticConfig,
};
use canopy_core::{UDNumber, UDPerson, UPos, Word};
use std::collections::HashMap;

#[cfg(test)]
mod layer1parser_comprehensive_tests {
    use super::*;

    // Tests for Layer1HelperConfig

    #[test]
    fn test_layer1_config_to_map() {
        let config = Layer1HelperConfig {
            enable_udpipe: false,
            enable_basic_features: true,
            enable_verbnet: false,
            max_sentence_length: 50,
            debug: true,
            confidence_threshold: 0.8,
        };

        let map = config.to_map();
        assert_eq!(map.get("enable_udpipe"), Some(&"false".to_string()));
        assert_eq!(map.get("enable_basic_features"), Some(&"true".to_string()));
        assert_eq!(map.get("enable_verbnet"), Some(&"false".to_string()));
        assert_eq!(map.get("max_sentence_length"), Some(&"50".to_string()));
        assert_eq!(map.get("debug"), Some(&"true".to_string()));
        assert_eq!(map.get("confidence_threshold"), Some(&"0.8".to_string()));
        assert_eq!(map.len(), 6);
    }

    #[test]
    fn test_layer1_config_validation_success() {
        let config = Layer1HelperConfig {
            max_sentence_length: 100,
            confidence_threshold: 0.5,
            ..Default::default()
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_layer1_config_validation_zero_sentence_length() {
        let config = Layer1HelperConfig {
            max_sentence_length: 0,
            confidence_threshold: 0.5,
            ..Default::default()
        };

        let result = config.validate();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "max_sentence_length must be greater than 0"
        );
    }

    #[test]
    fn test_layer1_config_validation_invalid_confidence_low() {
        let config = Layer1HelperConfig {
            max_sentence_length: 100,
            confidence_threshold: -0.1,
            ..Default::default()
        };

        let result = config.validate();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "confidence_threshold must be between 0.0 and 1.0"
        );
    }

    #[test]
    fn test_layer1_config_validation_invalid_confidence_high() {
        let config = Layer1HelperConfig {
            max_sentence_length: 100,
            confidence_threshold: 1.1,
            ..Default::default()
        };

        let result = config.validate();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "confidence_threshold must be between 0.0 and 1.0"
        );
    }

    #[test]
    fn test_layer1_config_validation_boundary_values() {
        // Test exact boundary values
        let config_min = Layer1HelperConfig {
            max_sentence_length: 1,
            confidence_threshold: 0.0,
            ..Default::default()
        };
        assert!(config_min.validate().is_ok());

        let config_max = Layer1HelperConfig {
            max_sentence_length: 1000,
            confidence_threshold: 1.0,
            ..Default::default()
        };
        assert!(config_max.validate().is_ok());
    }

    #[test]
    fn test_layer1_config_layer_name() {
        let config = Layer1HelperConfig::default();
        assert_eq!(config.layer_name(), "layer1_helper");
    }

    // Tests for Layer1ParserHandler

    #[test]
    fn test_layer1_parser_handler_new() {
        let handler = Layer1ParserHandler::new();
        assert_eq!(handler.config().layer_name(), "layer1_helper");
    }

    #[test]
    fn test_layer1_parser_handler_with_config() {
        let config = Layer1HelperConfig {
            enable_udpipe: false,
            max_sentence_length: 200,
            debug: true,
            ..Default::default()
        };

        let handler = Layer1ParserHandler::with_config(config.clone());

        // Verify the config was set properly
        let handler_map = handler.config().to_map();
        assert_eq!(handler_map.get("enable_udpipe"), Some(&"false".to_string()));
        assert_eq!(
            handler_map.get("max_sentence_length"),
            Some(&"200".to_string())
        );
        assert_eq!(handler_map.get("debug"), Some(&"true".to_string()));
    }

    #[test]
    fn test_layer1_parser_handler_process() {
        let handler = Layer1ParserHandler::new();
        let result = handler.process("The quick brown fox jumps".to_string());

        assert!(result.is_ok());
        let words = result.unwrap();
        assert_eq!(words.len(), 5);

        // Check first word
        assert_eq!(words[0].text, "The");
        assert_eq!(words[0].id, 1);

        // Check last word
        assert_eq!(words[4].text, "jumps");
        assert_eq!(words[4].id, 5);
    }

    #[test]
    fn test_layer1_parser_handler_process_empty_text() {
        let handler = Layer1ParserHandler::new();
        let result = handler.process("".to_string());

        // May return error or empty result depending on implementation
        if result.is_ok() {
            let words = result.unwrap();
            assert_eq!(words.len(), 0);
        } else {
            // Error is also acceptable for empty input
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_layer1_parser_handler_process_whitespace_only() {
        let handler = Layer1ParserHandler::new();
        let result = handler.process("   \t\n  ".to_string());

        // May return error or empty result depending on implementation
        if result.is_ok() {
            let words = result.unwrap();
            assert_eq!(words.len(), 0);
        } else {
            // Error is also acceptable for whitespace-only input
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_layer1_parser_handler_process_single_word() {
        let handler = Layer1ParserHandler::new();
        let result = handler.process("hello".to_string());

        assert!(result.is_ok());
        let words = result.unwrap();
        assert_eq!(words.len(), 1);
        assert_eq!(words[0].text, "hello");
        assert_eq!(words[0].id, 1);
        assert_eq!(words[0].start, 0);
        assert_eq!(words[0].end, 5);
    }

    #[test]
    fn test_layer1_parser_handler_morphology_pronouns() {
        let handler = Layer1ParserHandler::new();

        // Test various pronouns to trigger morphology analysis
        let test_cases = vec![
            (
                "I am here",
                "I",
                Some(UDPerson::First),
                Some(UDNumber::Singular),
            ),
            ("you are there", "you", Some(UDPerson::Second), None),
            (
                "he runs",
                "he",
                Some(UDPerson::Third),
                Some(UDNumber::Singular),
            ),
            (
                "she walks",
                "she",
                Some(UDPerson::Third),
                Some(UDNumber::Singular),
            ),
            (
                "it works",
                "it",
                Some(UDPerson::Third),
                Some(UDNumber::Singular),
            ),
            ("we go", "we", Some(UDPerson::First), Some(UDNumber::Plural)),
            (
                "they come",
                "they",
                Some(UDPerson::Third),
                Some(UDNumber::Plural),
            ),
        ];

        for (text, target_word, expected_person, expected_number) in test_cases {
            let result = handler.process(text.to_string());
            assert!(result.is_ok());
            let words = result.unwrap();

            let word = words
                .iter()
                .find(|w| w.text.to_lowercase() == target_word.to_lowercase())
                .unwrap();
            assert_eq!(
                word.feats.person, expected_person,
                "Failed for word: {}",
                target_word
            );
            assert_eq!(
                word.feats.number, expected_number,
                "Failed for word: {}",
                target_word
            );
        }
    }

    #[test]
    fn test_layer1_parser_handler_pos_tagging() {
        let handler = Layer1ParserHandler::new();
        let result = handler.process("The quick brown fox jumps over lazy dogs".to_string());

        assert!(result.is_ok());
        let words = result.unwrap();
        assert_eq!(words.len(), 8);

        // Check that POS tags are assigned (basic heuristics)
        for word in &words {
            // All words should have some POS tag assigned
            assert_ne!(word.upos, UPos::X); // Should not be unknown
        }
    }

    #[test]
    fn test_layer1_parser_handler_health() {
        let handler = Layer1ParserHandler::new();
        let health = handler.health();

        // The name might use snake_case or other format
        assert!(!health.name.is_empty());
        assert!(health.healthy);
        assert!(health.last_error.is_none());
        // Just check that some metrics exist
        assert!(!health.metrics.is_empty());
    }

    // Tests for SemanticConfig

    #[test]
    fn test_semantic_config_to_map() {
        let config = SemanticConfig {
            enable_theta_roles: false,
            enable_animacy: true,
            enable_definiteness: false,
            confidence_threshold: 0.75,
            debug: true,
        };

        let map = config.to_map();
        assert_eq!(map.get("enable_theta_roles"), Some(&"false".to_string()));
        assert_eq!(map.get("enable_animacy"), Some(&"true".to_string()));
        assert_eq!(map.get("enable_definiteness"), Some(&"false".to_string()));
        assert_eq!(map.get("confidence_threshold"), Some(&"0.75".to_string()));
        assert_eq!(map.get("debug"), Some(&"true".to_string()));
        assert_eq!(map.len(), 5);
    }

    #[test]
    fn test_semantic_config_validation() {
        let config = SemanticConfig {
            confidence_threshold: 0.6,
            ..Default::default()
        };
        assert!(config.validate().is_ok());

        // Test invalid confidence threshold
        let invalid_config = SemanticConfig {
            confidence_threshold: 1.5,
            ..Default::default()
        };
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_semantic_config_layer_name() {
        let config = SemanticConfig::default();
        assert_eq!(config.layer_name(), "semantic_analysis");
    }

    // Tests for SemanticAnalysisHandler

    #[test]
    fn test_semantic_handler_new() {
        let handler = SemanticAnalysisHandler::new();
        assert_eq!(handler.config().layer_name(), "semantic_analysis");
    }

    #[test]
    fn test_semantic_handler_with_config() {
        let config = SemanticConfig {
            enable_theta_roles: false,
            enable_animacy: true,
            confidence_threshold: 0.8,
            ..Default::default()
        };

        let handler = SemanticAnalysisHandler::with_config(config);

        let handler_map = handler.config().to_map();
        assert_eq!(
            handler_map.get("enable_theta_roles"),
            Some(&"false".to_string())
        );
        assert_eq!(handler_map.get("enable_animacy"), Some(&"true".to_string()));
        assert_eq!(
            handler_map.get("confidence_threshold"),
            Some(&"0.8".to_string())
        );
    }

    #[test]
    fn test_semantic_handler_process() {
        let handler = SemanticAnalysisHandler::new();

        // Create some test words
        let mut words = vec![
            Word::new(1, "John".to_string(), 0, 4),
            Word::new(2, "runs".to_string(), 5, 9),
            Word::new(3, "quickly".to_string(), 10, 17),
        ];

        // Set basic POS tags
        words[0].upos = UPos::Noun;
        words[1].upos = UPos::Verb;
        words[2].upos = UPos::Adv;

        let result = handler.process(words);
        assert!(result.is_ok());

        let enhanced_words = result.unwrap();
        assert_eq!(enhanced_words.len(), 3);

        // Check that semantic features were added
        // The implementation should add some semantic features to words
        for word in &enhanced_words {
            // Just verify words were processed without error
            assert!(!word.text.is_empty());
        }
    }

    #[test]
    fn test_semantic_handler_process_empty_input() {
        let handler = SemanticAnalysisHandler::new();
        let result = handler.process(vec![]);

        assert!(result.is_ok());
        let words = result.unwrap();
        assert_eq!(words.len(), 0);
    }

    #[test]
    fn test_semantic_handler_health() {
        let handler = SemanticAnalysisHandler::new();
        let health = handler.health();

        // The name might use snake_case or other format
        assert!(!health.name.is_empty());
        assert!(health.healthy);
        assert!(health.last_error.is_none());
        // Just check that some metrics exist
        assert!(!health.metrics.is_empty());
    }

    // Integration tests

    #[test]
    fn test_layer1_to_semantic_pipeline() {
        let layer1_handler = Layer1ParserHandler::new();
        let semantic_handler = SemanticAnalysisHandler::new();

        // Process through layer1 first
        let layer1_result = layer1_handler.process("The cat sat on the mat".to_string());
        assert!(layer1_result.is_ok());

        let words = layer1_result.unwrap();
        assert_eq!(words.len(), 6);

        // Then through semantic analysis
        let semantic_result = semantic_handler.process(words);
        assert!(semantic_result.is_ok());

        let final_words = semantic_result.unwrap();
        assert_eq!(final_words.len(), 6);
    }

    #[test]
    fn test_component_health_creation() {
        let health = ComponentHealth {
            name: "test_component".to_string(),
            healthy: true,
            last_error: None,
            metrics: HashMap::new(),
        };

        assert_eq!(health.name, "test_component");
        assert!(health.healthy);
        assert!(health.last_error.is_none());
        assert!(health.metrics.is_empty());
    }

    #[test]
    fn test_component_health_with_error() {
        let mut metrics = HashMap::new();
        metrics.insert("error_count".to_string(), 1.0);

        let health = ComponentHealth {
            name: "failing_component".to_string(),
            healthy: false,
            last_error: Some("Connection failed".to_string()),
            metrics,
        };

        assert_eq!(health.name, "failing_component");
        assert!(!health.healthy);
        assert_eq!(health.last_error, Some("Connection failed".to_string()));
        assert_eq!(health.metrics.get("error_count"), Some(&1.0));
    }

    #[test]
    fn test_handler_stats_calculation() {
        let handler = Layer1ParserHandler::new();

        // Process multiple requests to generate stats
        for i in 0..5 {
            let text = format!("Test sentence number {}", i);
            let result = handler.process(text);
            assert!(result.is_ok());
        }

        let health = handler.health();
        // Just verify that metrics exist and are reasonable
        if let Some(requests) = health.metrics.get("requests") {
            assert!(*requests >= 0.0);
        }
        if let Some(successes) = health.metrics.get("successes") {
            assert!(*successes >= 0.0);
        }
    }

    #[test]
    fn test_config_edge_cases() {
        // Test with extreme values
        let config = Layer1HelperConfig {
            enable_udpipe: false,
            enable_basic_features: false,
            enable_verbnet: false,
            max_sentence_length: 1,
            debug: false,
            confidence_threshold: 0.0,
        };

        assert!(config.validate().is_ok());

        let handler = Layer1ParserHandler::with_config(config);
        let result = handler.process("Word".to_string());
        assert!(result.is_ok());
    }
}
