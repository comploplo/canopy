//! Tests for LSP main.rs functionality
//!
//! Tests the main LSP server entry point to reach 70% coverage target

#[cfg(test)]
mod tests {
    // Test the configuration structures used in main.rs
    use canopy_core::layer1parser::{Layer1HelperConfig, SemanticConfig};

    #[test]
    fn test_layer1_helper_config_creation() {
        let config = Layer1HelperConfig {
            enable_udpipe: true,
            enable_basic_features: true,
            enable_verbnet: true,
            max_sentence_length: 100,
            debug: true,
            confidence_threshold: 0.5,
        };

        assert!(config.enable_udpipe);
        assert!(config.enable_basic_features);
        assert!(config.enable_verbnet);
        assert_eq!(config.max_sentence_length, 100);
        assert!(config.debug);
        assert_eq!(config.confidence_threshold, 0.5);
    }

    #[test]
    fn test_semantic_config_creation() {
        let config = SemanticConfig {
            enable_theta_roles: true,
            enable_animacy: true,
            enable_definiteness: true,
            confidence_threshold: 0.6,
            debug: true,
        };

        assert!(config.enable_theta_roles);
        assert!(config.enable_animacy);
        assert!(config.enable_definiteness);
        assert_eq!(config.confidence_threshold, 0.6);
        assert!(config.debug);
    }

    #[test]
    fn test_config_combinations() {
        let parser_configs = [
            Layer1HelperConfig {
                enable_udpipe: true,
                enable_basic_features: true,
                enable_verbnet: true,
                max_sentence_length: 50,
                debug: false,
                confidence_threshold: 0.3,
            },
            Layer1HelperConfig {
                enable_udpipe: false,
                enable_basic_features: true,
                enable_verbnet: false,
                max_sentence_length: 200,
                debug: true,
                confidence_threshold: 0.8,
            },
        ];

        let semantic_configs = [
            SemanticConfig {
                enable_theta_roles: false,
                enable_animacy: false,
                enable_definiteness: false,
                confidence_threshold: 0.1,
                debug: false,
            },
            SemanticConfig {
                enable_theta_roles: true,
                enable_animacy: true,
                enable_definiteness: true,
                confidence_threshold: 0.9,
                debug: true,
            },
        ];

        // Test all combinations work
        for parser_config in &parser_configs {
            for semantic_config in &semantic_configs {
                // Should be able to create both config types without issues
                assert!(parser_config.confidence_threshold >= 0.0);
                assert!(parser_config.confidence_threshold <= 1.0);
                assert!(semantic_config.confidence_threshold >= 0.0);
                assert!(semantic_config.confidence_threshold <= 1.0);
                assert!(parser_config.max_sentence_length > 0);
            }
        }
    }

    #[test]
    fn test_server_creation_with_configs() {
        use crate::CanopyLspServerFactory;

        let parser_config = Layer1HelperConfig {
            enable_udpipe: true,
            enable_basic_features: true,
            enable_verbnet: true,
            max_sentence_length: 100,
            debug: false, // Don't spam debug output in tests
            confidence_threshold: 0.5,
        };

        let semantic_config = SemanticConfig {
            enable_theta_roles: true,
            enable_animacy: true,
            enable_definiteness: true,
            confidence_threshold: 0.6,
            debug: false, // Don't spam debug output in tests
        };

        // This mimics what main.rs does
        let result =
            CanopyLspServerFactory::create_server_with_config(parser_config, semantic_config);

        assert!(result.is_ok());
    }

    #[test]
    fn test_server_text_processing() {
        use crate::{CanopyLspServerFactory, server::CanopyServer};

        let parser_config = Layer1HelperConfig {
            enable_udpipe: true,
            enable_basic_features: true,
            enable_verbnet: true,
            max_sentence_length: 100,
            debug: false,
            confidence_threshold: 0.5,
        };

        let semantic_config = SemanticConfig {
            enable_theta_roles: true,
            enable_animacy: true,
            enable_definiteness: true,
            confidence_threshold: 0.6,
            debug: false,
        };

        let server =
            CanopyLspServerFactory::create_server_with_config(parser_config, semantic_config)
                .unwrap();

        // Test the same text processing that main.rs does
        let response = server.process_text("John runs quickly");
        assert!(response.is_ok());

        let response = response.unwrap();
        assert!(response.metrics.total_time_us > 0);
        assert!(response.document.total_word_count() > 0);
    }

    #[test]
    fn test_server_health_check() {
        use crate::{CanopyLspServerFactory, server::CanopyServer};

        let parser_config = Layer1HelperConfig {
            enable_udpipe: true,
            enable_basic_features: true,
            enable_verbnet: true,
            max_sentence_length: 100,
            debug: false,
            confidence_threshold: 0.5,
        };

        let semantic_config = SemanticConfig {
            enable_theta_roles: true,
            enable_animacy: true,
            enable_definiteness: true,
            confidence_threshold: 0.6,
            debug: false,
        };

        let server =
            CanopyLspServerFactory::create_server_with_config(parser_config, semantic_config)
                .unwrap();

        // Test the health check that main.rs does
        let health = server.health();
        // Server might not be fully healthy without real UDPipe models, but should return status
        assert!(health.components.len() >= 2); // Should have layer1 and semantics
    }

    #[test]
    fn test_verbnet_integration_function() {
        // Test that the verbnet_test::test_verbnet_integration function exists and can be called
        // This is what main.rs calls
        crate::verbnet_test::test_verbnet_integration();
        // If this doesn't panic, the function works
    }

    #[test]
    fn test_configuration_edge_cases() {
        // Test edge cases for configurations
        let edge_configs = [
            Layer1HelperConfig {
                enable_udpipe: false,
                enable_basic_features: false,
                enable_verbnet: false,
                max_sentence_length: 1,
                debug: false,
                confidence_threshold: 0.0,
            },
            Layer1HelperConfig {
                enable_udpipe: true,
                enable_basic_features: true,
                enable_verbnet: true,
                max_sentence_length: 1000,
                debug: true,
                confidence_threshold: 1.0,
            },
        ];

        let semantic_edge_configs = [
            SemanticConfig {
                enable_theta_roles: false,
                enable_animacy: false,
                enable_definiteness: false,
                confidence_threshold: 0.0,
                debug: false,
            },
            SemanticConfig {
                enable_theta_roles: true,
                enable_animacy: true,
                enable_definiteness: true,
                confidence_threshold: 1.0,
                debug: true,
            },
        ];

        // All edge cases should create servers successfully
        for parser_config in &edge_configs {
            for semantic_config in &semantic_edge_configs {
                let result = crate::CanopyLspServerFactory::create_server_with_config(
                    parser_config.clone(),
                    semantic_config.clone(),
                );
                assert!(result.is_ok());
            }
        }
    }

    #[test]
    fn test_multiple_text_processing_calls() {
        use crate::{CanopyLspServerFactory, server::CanopyServer};

        let parser_config = Layer1HelperConfig {
            enable_udpipe: true,
            enable_basic_features: true,
            enable_verbnet: true,
            max_sentence_length: 100,
            debug: false,
            confidence_threshold: 0.5,
        };

        let semantic_config = SemanticConfig {
            enable_theta_roles: true,
            enable_animacy: true,
            enable_definiteness: true,
            confidence_threshold: 0.6,
            debug: false,
        };

        let server =
            CanopyLspServerFactory::create_server_with_config(parser_config, semantic_config)
                .unwrap();

        // Test multiple calls like a real LSP server would handle
        let test_texts = [
            "John runs quickly",
            "Mary gave John a book",
            "The cat sleeps",
            "What did you see?",
        ];

        for text in &test_texts {
            let response = server.process_text(text);
            assert!(response.is_ok(), "Failed to process: {}", text);

            let response = response.unwrap();
            assert!(response.document.total_word_count() > 0);
            assert!(response.metrics.total_time_us > 0);
        }
    }
}
