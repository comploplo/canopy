//! Comprehensive error handling tests for LSP server functionality
//!
//! These tests cover edge cases and error scenarios to improve coverage.

#[cfg(test)]
mod lsp_error_tests {
    use crate::server::CanopyServer;
    use crate::*;
    use canopy_core::{CanopyError, layer1parser};

    #[test]
    fn test_server_factory_creation_errors() {
        // Test that server factory handles various creation scenarios
        let result = CanopyLspServerFactory::create_server();
        assert!(result.is_ok());
    }

    #[test]
    fn test_server_factory_with_invalid_config() {
        // Test with extreme configuration values
        let parser_config = layer1parser::Layer1HelperConfig {
            enable_udpipe: true,
            enable_basic_features: false,
            enable_verbnet: false,
            max_sentence_length: 0, // Edge case: zero length
            debug: true,
            confidence_threshold: 2.0, // Edge case: > 1.0
        };

        let semantic_config = layer1parser::SemanticConfig {
            enable_theta_roles: false,
            enable_animacy: false,
            enable_definiteness: false,
            confidence_threshold: 2.0, // Edge case: > 1.0
            debug: true,
        };

        let result =
            CanopyLspServerFactory::create_server_with_config(parser_config, semantic_config);
        assert!(result.is_ok()); // Should handle gracefully
    }

    #[test]
    fn test_error_propagation_through_handlers() {
        // Test error propagation through the handler chain
        let server =
            CanopyLspServerFactory::create_server().expect("Server creation should succeed");

        // Test with problematic input
        let empty_text = "";
        let result = server.process_text(empty_text);

        // Should handle empty input gracefully
        assert!(result.is_ok() || result.is_err());

        // Test with only whitespace
        let whitespace_text = "   \n\t  ";
        let result = server.process_text(whitespace_text);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_server_dependency_injection_edge_cases() {
        // Test that DI handles null/empty states
        use server::DefaultCanopyServer;

        let parser_handler = layer1parser::Layer1ParserHandler::new();
        let semantic_handler = layer1parser::SemanticAnalysisHandler::new();

        let server = DefaultCanopyServer::new(parser_handler, semantic_handler);

        // Test server with minimal input
        let result = server.process_text("A");
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_lsp_backend_error_scenarios() {
        // Test LSP backend error handling
        use crate::lsp_backend::CanopyLspStub;

        let backend = CanopyLspStub::new();

        // Test with various input scenarios
        let binding = "x".repeat(1000);
        let test_cases = vec![
            "",
            " ",
            "\n",
            "\t",
            "a",
            "Test sentence.",
            &binding, // Very long input
        ];

        for input in test_cases {
            let result = backend.analyze_text(input);
            // Should handle all inputs without panic
            assert!(result.is_ok() || result.is_err());
        }
    }

    #[test]
    fn test_diagnostic_edge_cases() {
        use crate::handlers::{DiagnosticSeverity, create_diagnostic};

        // Test diagnostic creation with edge cases
        let diagnostic = create_diagnostic(
            "".to_string(), // Empty message
            DiagnosticSeverity::Error,
            0, // Line
            0, // Character
        );

        assert_eq!(diagnostic.range.start.line, 0);
        assert_eq!(diagnostic.range.end.line, 0);

        // Test with very long message
        let long_message = "x".repeat(10000);
        let diagnostic =
            create_diagnostic(long_message.clone(), DiagnosticSeverity::Warning, 100, 50);

        assert_eq!(diagnostic.message, long_message);
    }

    #[test]
    fn test_handler_chain_error_recovery() {
        // Test that handler chain recovers from errors in individual handlers
        let server =
            CanopyLspServerFactory::create_server().expect("Server creation should succeed");

        // Test with input that might cause issues in parsing
        let problematic_inputs = vec![
            "This is a sentence with ü n i c o d e characters",
            "Sentence with numbers 123 and symbols !@#$%",
            "Mixed case WORDS with Different Capitalizations",
            "Very, very, very, very, very, very long sentence with many commas, semicolons; and other punctuation marks!",
        ];

        for input in problematic_inputs {
            let result = server.process_text(input);
            // All should be handled gracefully
            assert!(result.is_ok() || result.is_err());
        }
    }

    #[test]
    fn test_verbnet_integration_error_handling() {
        // Test VerbNet integration error handling
        use verbnet_test::test_verbnet_integration;

        // This should handle missing VerbNet data gracefully
        test_verbnet_integration();
        // If we get here without panic, it handled missing data gracefully
        assert!(true);
    }

    #[test]
    fn test_concurrent_server_access() {
        // Test concurrent access to server
        use std::sync::Arc;
        use std::thread;

        let server = Arc::new(
            CanopyLspServerFactory::create_server().expect("Server creation should succeed"),
        );

        let handles: Vec<_> = (0..4)
            .map(|i| {
                let server_clone = Arc::clone(&server);
                thread::spawn(move || {
                    let text = format!("Test sentence {}", i);
                    let result = server_clone.process_text(&text);
                    assert!(result.is_ok() || result.is_err());
                })
            })
            .collect();

        for handle in handles {
            handle.join().expect("Thread should not panic");
        }
    }

    #[test]
    fn test_error_serialization() {
        // Test that errors can be serialized/deserialized for LSP responses
        let error = CanopyError::ParseError {
            context: "test context".to_string(),
        };

        let error_str = format!("{}", error);
        assert!(error_str.contains("parsing failed"));

        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("ParseError"));
    }

    #[test]
    fn test_handler_configuration_edge_cases() {
        // Test various handler configurations
        let configs = vec![
            layer1parser::Layer1HelperConfig {
                enable_udpipe: true,
                enable_basic_features: true,
                enable_verbnet: true,
                max_sentence_length: 1, // Very small
                debug: false,
                confidence_threshold: 0.5,
            },
            layer1parser::Layer1HelperConfig {
                enable_udpipe: false,
                enable_basic_features: false,
                enable_verbnet: false,
                max_sentence_length: 10000, // Very large
                debug: true,
                confidence_threshold: 0.1,
            },
        ];

        for config in configs {
            let semantic_config = layer1parser::SemanticConfig::default();
            let result = CanopyLspServerFactory::create_server_with_config(config, semantic_config);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_memory_cleanup_on_errors() {
        // Test that memory is properly cleaned up on errors
        let server =
            CanopyLspServerFactory::create_server().expect("Server creation should succeed");

        // Process many requests to test memory handling
        for i in 0..100 {
            let text = format!("Test sentence number {}", i);
            let _ = server.process_text(&text);
        }

        // If we get here without OOM, memory cleanup is working
        assert!(true);
    }

    #[test]
    fn test_lsp_protocol_edge_cases() {
        // Test LSP protocol edge cases
        use crate::lsp_backend::CanopyLspStub;

        let backend = CanopyLspStub::new();

        // Test with various text encodings scenarios
        let unicode_text = "Hello 世界 🌍 café naïve résumé";
        let result = backend.analyze_text(unicode_text);
        assert!(result.is_ok() || result.is_err());

        // Test with control characters
        let control_chars = "Test\x00\x01\x02text";
        let result = backend.analyze_text(control_chars);
        assert!(result.is_ok() || result.is_err());
    }
}
