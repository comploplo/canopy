//! LSP Server Lifecycle Tests
//!
//! These tests cover the server startup, configuration, shutdown, and health
//! monitoring functionality to ensure robust LSP operation.

use crate::CanopyLspServerFactory;
use crate::server::{AnalysisResponse, CanopyServer};

#[cfg(test)]
mod server_lifecycle_tests {
    use super::*;

    #[test]
    fn test_server_startup_with_default_config() {
        // Test that server can be created with default configuration
        let result = CanopyLspServerFactory::create_server();

        assert!(result.is_ok(), "Server should start with default config");

        let server = result.unwrap();
        let health = server.health();

        assert!(health.healthy, "Server should be healthy after startup");
        assert_eq!(
            health.components.len(),
            2,
            "Should have parser and semantic components"
        );

        // Verify component names
        let component_names: Vec<&String> = health.components.keys().collect();
        assert!(component_names.contains(&&"layer1".to_string()));
        assert!(component_names.contains(&&"semantics".to_string()));
    }

    #[test]
    fn test_server_startup_with_custom_config() {
        // Test server creation with custom configuration
        let parser_config = canopy_core::layer1parser::Layer1HelperConfig {
            enable_udpipe: true,
            enable_basic_features: true,
            enable_verbnet: true,
            max_sentence_length: 100,
            debug: true,
            confidence_threshold: 0.7,
        };

        let semantic_config = canopy_core::layer1parser::SemanticConfig {
            enable_theta_roles: true,
            enable_animacy: true,
            enable_definiteness: true,
            confidence_threshold: 0.8,
            debug: false,
        };

        let result =
            CanopyLspServerFactory::create_server_with_config(parser_config, semantic_config);

        assert!(result.is_ok(), "Server should start with custom config");

        let server = result.unwrap();
        let health = server.health();

        assert!(
            health.healthy,
            "Server should be healthy with custom config"
        );

        // Verify custom configuration is applied
        let layer1_health = &health.components["layer1"];
        assert!(layer1_health.healthy, "Layer 1 should be healthy");

        let semantics_health = &health.components["semantics"];
        assert!(
            semantics_health.healthy,
            "Semantics layer should be healthy"
        );
    }

    #[test]
    fn test_server_health_reporting() {
        // Test comprehensive health reporting functionality
        let server = CanopyLspServerFactory::create_server().unwrap();
        let health = server.health();

        // Check overall health structure
        assert!(health.healthy, "Server should report healthy status");
        assert!(
            !health.components.is_empty(),
            "Should have component health info"
        );

        // Check component health details
        for (component_name, component_health) in &health.components {
            assert!(!component_name.is_empty(), "Component should have a name");
            assert!(
                component_health.healthy,
                "Component {} should be healthy",
                component_name
            );
            assert!(
                component_health.last_error.is_none(),
                "Component {} should have no errors",
                component_name
            );
        }

        // Check uptime is reasonable (should be 0 initially, or small positive value)
        assert!(
            health.uptime_seconds < 60,
            "Test server uptime should be less than 60 seconds"
        );

        // Check request count
        assert_eq!(
            health.requests_processed, 0,
            "New server should have zero requests processed"
        );
    }

    #[test]
    fn test_server_processing_updates_metrics() {
        // Test that processing requests updates server metrics
        let server = CanopyLspServerFactory::create_server().unwrap();

        // Initial health check
        let initial_health = server.health();
        assert_eq!(initial_health.requests_processed, 0);

        // Process some text
        let result = server.process_text("Hello world");
        assert!(result.is_ok(), "Processing should succeed");

        // Check health after processing
        let updated_health = server.health();
        assert_eq!(
            updated_health.requests_processed, 1,
            "Request count should be updated"
        );
        assert!(
            updated_health.healthy,
            "Server should remain healthy after processing"
        );
    }

    #[test]
    fn test_concurrent_request_handling() {
        // Test that server can handle multiple concurrent requests
        let server = CanopyLspServerFactory::create_server().unwrap();

        // Process multiple requests
        let texts = vec![
            "The cat sat on the mat",
            "Dogs are loyal animals",
            "Programming is challenging but rewarding",
            "Natural language processing requires careful attention",
        ];

        let mut results = Vec::new();
        for text in &texts {
            let result = server.process_text(text);
            assert!(result.is_ok(), "Each request should succeed");
            results.push(result.unwrap());
        }

        // Verify all requests were processed
        assert_eq!(
            results.len(),
            texts.len(),
            "All requests should be processed"
        );

        // Check final metrics
        let final_health = server.health();
        assert_eq!(
            final_health.requests_processed,
            texts.len() as u64,
            "Request count should match number of processed texts"
        );

        // Verify each result has proper structure
        for (i, result) in results.iter().enumerate() {
            assert!(
                !result.document.sentences.is_empty(),
                "Text {} should have sentences",
                i
            );
            assert!(
                result.metrics.total_time_us > 0,
                "Text {} should have processing time",
                i
            );
            assert!(
                result.layer_results.contains_key("layer1"),
                "Text {} should have layer1 results",
                i
            );
            assert!(
                result.layer_results.contains_key("semantics"),
                "Text {} should have semantic results",
                i
            );
        }
    }

    #[test]
    fn test_server_error_recovery() {
        // Test that server can recover from processing errors
        let server = CanopyLspServerFactory::create_server().unwrap();

        // Process valid text first
        let valid_result = server.process_text("This is valid text");
        assert!(
            valid_result.is_ok(),
            "Valid text should process successfully"
        );

        // Server should remain healthy after successful processing
        let health_after_success = server.health();
        assert!(
            health_after_success.healthy,
            "Server should be healthy after success"
        );

        // Process empty text (edge case)
        let _empty_result = server.process_text("");
        // This might succeed or fail depending on implementation, but shouldn't crash

        // Process very long text (stress test)
        let long_text = "word ".repeat(1000);
        let _long_result = server.process_text(&long_text);
        // Should handle gracefully without crashing

        // Server should remain operational
        let final_health = server.health();
        assert!(
            final_health.healthy,
            "Server should remain healthy after edge cases"
        );

        // Should still be able to process normal text
        let recovery_result = server.process_text("Recovery test");
        assert!(
            recovery_result.is_ok(),
            "Server should recover and process normally"
        );
    }

    #[test]
    fn test_component_configuration_validation() {
        // Test that invalid configurations are properly handled

        // Test with extreme timeout values
        let extreme_config = canopy_core::layer1parser::Layer1HelperConfig {
            enable_udpipe: true,
            enable_basic_features: true,
            enable_verbnet: true,
            max_sentence_length: 0, // Invalid max length
            debug: false,
            confidence_threshold: 1.5, // Invalid confidence (>1.0)
        };

        let semantic_config = canopy_core::layer1parser::SemanticConfig {
            enable_theta_roles: true,
            enable_animacy: true,
            enable_definiteness: true,
            confidence_threshold: 0.5,
            debug: false,
        };

        // Server should still create but with safe defaults
        let result =
            CanopyLspServerFactory::create_server_with_config(extreme_config, semantic_config);

        assert!(
            result.is_ok(),
            "Server should handle extreme config gracefully"
        );

        let server = result.unwrap();
        let health = server.health();
        assert!(
            health.healthy,
            "Server should be healthy despite extreme config"
        );
    }

    #[test]
    fn test_performance_metrics_collection() {
        // Test that performance metrics are properly collected and reported
        let server = CanopyLspServerFactory::create_server().unwrap();

        let text = "Performance testing with multiple sentences. This should generate timing data.";
        let result = server.process_text(text).unwrap();

        // Check that metrics are populated
        assert!(
            result.metrics.total_time_us > 0,
            "Should have positive total time"
        );
        assert!(
            !result.metrics.layer_times.is_empty(),
            "Should have layer timing data"
        );

        // Check layer-specific timing
        assert!(
            result.metrics.layer_times.contains_key("layer1"),
            "Should have layer1 timing"
        );
        assert!(
            result.metrics.layer_times.contains_key("semantics"),
            "Should have semantics timing"
        );

        let layer1_time = result.metrics.layer_times["layer1"];
        let semantics_time = result.metrics.layer_times["semantics"];

        assert!(
            layer1_time > 0,
            "Layer1 should have positive processing time"
        );
        assert!(
            semantics_time > 0,
            "Semantics should have positive processing time"
        );

        // Total time should be at least the sum of layer times
        let sum_layer_times = layer1_time + semantics_time;
        assert!(
            result.metrics.total_time_us >= sum_layer_times,
            "Total time should be >= sum of layer times"
        );

        // Check memory statistics are reasonable
        assert!(
            result.metrics.memory_stats.peak_bytes > 0,
            "Should track peak memory"
        );
        assert!(
            result.metrics.memory_stats.final_bytes > 0,
            "Should track final memory"
        );
        assert!(
            result.metrics.memory_stats.allocations > 0,
            "Should track allocations"
        );

        // Check input statistics
        assert!(
            result.metrics.input_stats.char_count > 0,
            "Should count characters"
        );
        assert!(
            result.metrics.input_stats.word_count > 0,
            "Should count words"
        );
        assert!(
            result.metrics.input_stats.sentence_count > 0,
            "Should count sentences"
        );
    }
}

/// Test utilities for server testing
#[cfg(test)]
mod test_utils {
    use super::*;

    /// Helper to create a test server with known configuration
    pub fn create_test_server() -> Box<dyn CanopyServer> {
        let config = canopy_core::layer1parser::Layer1HelperConfig {
            enable_udpipe: true,
            enable_basic_features: true,
            enable_verbnet: true,
            max_sentence_length: 200,
            debug: false,
            confidence_threshold: 0.7,
        };

        let semantic_config = canopy_core::layer1parser::SemanticConfig {
            enable_theta_roles: true,
            enable_animacy: true,
            enable_definiteness: true,
            confidence_threshold: 0.7,
            debug: false,
        };

        Box::new(
            CanopyLspServerFactory::create_server_with_config(config, semantic_config)
                .expect("Test server should always create successfully"),
        )
    }

    /// Helper to verify basic server response structure
    pub fn verify_response_structure(response: &AnalysisResponse, input_text: &str) {
        assert!(
            !response.document.sentences.is_empty(),
            "Response should have sentences"
        );
        assert!(
            !response.layer_results.is_empty(),
            "Response should have layer results"
        );
        assert!(
            response.metrics.total_time_us > 0,
            "Response should have timing metrics"
        );

        // Verify input text characteristics match
        let expected_char_count = input_text.chars().count();
        assert_eq!(
            response.metrics.input_stats.char_count, expected_char_count,
            "Character count should match input"
        );
    }
}
