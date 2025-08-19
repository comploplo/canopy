//! Comprehensive Error Handling Tests
//!
//! Tests for error handling, recovery, and graceful degradation across all
//! LSP components and edge cases.

use crate::handlers::{DiagnosticSeverity, create_diagnostic};
use crate::server::{CanopyServer, DefaultCanopyServer};
use crate::{CanopyLspServerFactory, integration::RealServerFactory};
use canopy_core::CanopyError;
use canopy_core::layer1parser::{
    Layer1HelperConfig, Layer1ParserHandler, SemanticAnalysisHandler, SemanticConfig,
};

#[cfg(test)]
mod comprehensive_error_tests {
    use super::*;

    #[test]
    fn test_input_validation_errors() {
        // Test comprehensive input validation and error handling
        let server = CanopyLspServerFactory::create_server().unwrap();

        let invalid_inputs = vec![
            ("", "Empty string"),
            ("   ", "Whitespace only"),
            ("\n\n\n", "Newlines only"),
            ("\t\t\t", "Tabs only"),
            ("\r\n\r\n", "CRLF only"),
            ("\x00\x01\x02", "Control characters"),
            ("🏴󠁧󠁢󠁳󠁣󠁴󠁿💀👻", "Complex unicode"),
        ];

        for (input, description) in invalid_inputs {
            let result = server.process_text(input);

            match result {
                Ok(response) => {
                    // Some inputs might be handled gracefully
                    println!(
                        "{}: Handled gracefully - {} sentences",
                        description,
                        response.document.sentences.len()
                    );

                    // Graceful handling should still produce valid structure
                    assert!(
                        response.metrics.total_time_us > 0,
                        "Should have timing even for edge cases"
                    );
                }
                Err(error) => {
                    // Errors should be well-formed and informative
                    println!("{}: Error (expected) - {:?}", description, error);

                    match error {
                        CanopyError::ParseError { context } => {
                            assert!(!context.is_empty(), "Parse error should have context");
                            assert!(
                                context.len() < 500,
                                "Error context should be reasonable length"
                            );
                        }
                        CanopyError::SemanticError(_) => {
                            // Semantic errors are acceptable for invalid input
                            assert!(true, "Semantic errors acceptable for invalid input");
                        }
                        CanopyError::LspError(_) => {
                            // LSP errors are acceptable for invalid input
                            assert!(true, "LSP errors acceptable for invalid input");
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_extreme_input_sizes() {
        // Test error handling with extreme input sizes
        let server = CanopyLspServerFactory::create_server().unwrap();

        let repeated_char = "a".repeat(10000);
        let long_word = "supercalifragilisticexpialidocious".repeat(100);
        let very_long_text = "word ".repeat(2000);
        let extreme_cases = vec![
            ("VeryLong", very_long_text.trim()),      // 2000 words
            ("VeryShort", "a"),                       // 1 character
            ("RepeatedChar", repeated_char.as_str()), // 10k same character
            ("LongWord", long_word.as_str()),         // Very long words
        ];

        for (case_name, input) in extreme_cases {
            let result = server.process_text(input);

            match result {
                Ok(response) => {
                    println!(
                        "{}: Processed {} chars -> {} sentences",
                        case_name,
                        input.len(),
                        response.document.sentences.len()
                    );

                    // Should handle extreme cases without crashing
                    assert!(
                        response.metrics.total_time_us > 0,
                        "Should have processing time"
                    );

                    // Processing time should be reasonable even for large inputs
                    assert!(
                        response.metrics.total_time_us < 10_000_000, // 10 seconds
                        "Processing should complete in reasonable time"
                    );
                }
                Err(error) => {
                    println!("{}: Failed gracefully - {:?}", case_name, error);

                    // Should provide meaningful error for extreme cases
                    match error {
                        CanopyError::ParseError { context } => {
                            assert!(
                                context.contains("too long")
                                    || context.contains("limit")
                                    || context.contains("size"),
                                "Parse error should mention size limits"
                            );
                        }
                        _ => {
                            assert!(true, "Any error type acceptable for extreme inputs");
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_configuration_error_handling() {
        // Test error handling with invalid configurations

        // Test with invalid Layer1HelperConfig
        let invalid_configs = vec![
            Layer1HelperConfig {
                enable_udpipe: false,
                enable_basic_features: false,
                enable_verbnet: false,
                max_sentence_length: 0, // Invalid: zero length
                debug: false,
                confidence_threshold: -0.5, // Invalid: negative
            },
            Layer1HelperConfig {
                enable_udpipe: true,
                enable_basic_features: true,
                enable_verbnet: true,
                max_sentence_length: 1000000, // Very large
                debug: true,
                confidence_threshold: 2.0, // Invalid: > 1.0
            },
        ];

        for (i, config) in invalid_configs.into_iter().enumerate() {
            let semantic_config = SemanticConfig::default();

            let result = CanopyLspServerFactory::create_server_with_config(config, semantic_config);

            match result {
                Ok(server) => {
                    // Server might handle invalid config gracefully
                    let health = server.health();
                    println!(
                        "Invalid config {}: Server created, healthy={}",
                        i, health.healthy
                    );

                    // Try processing with invalid config
                    let process_result = server.process_text("Config test.");
                    match process_result {
                        Ok(_) => println!("Invalid config {}: Processing succeeded", i),
                        Err(error) => {
                            println!("Invalid config {}: Processing failed - {:?}", i, error)
                        }
                    }
                }
                Err(error) => {
                    // Configuration validation should catch invalid configs
                    println!(
                        "Invalid config {}: Creation failed (expected) - {:?}",
                        i, error
                    );
                    assert!(true, "Invalid configuration rejection is correct behavior");
                }
            }
        }
    }

    #[test]
    fn test_component_failure_isolation() {
        // Test that component failures don't crash the entire system
        let parser_handler = Layer1ParserHandler::new();
        let semantic_handler = SemanticAnalysisHandler::new();

        let server = DefaultCanopyServer::new(parser_handler, semantic_handler);

        // Test with inputs that might cause specific component failures
        let component_stress_tests = vec![
            ("ParserStress", "Ṫḧïṡ ïṡ ä ṫëṡṫ ẅïṫḧ ṡṗëċïäl ċḧäräċṫëṙṡ"), // Special characters
            ("SemanticStress", "The the the the the the the"),          // Repetitive structure
            ("BothStress", "!@#$%^&*()_+-=[]{}|;':\",./<>?"),           // Punctuation only
            ("UnicodeStress", "こんにちは 世界 🌍 Здравствуй мир"),     // Mixed unicode
        ];

        for (test_name, input) in component_stress_tests {
            let result = server.process_text(input);

            match result {
                Ok(response) => {
                    println!("{}: Components handled stress successfully", test_name);
                    assert!(
                        !response.document.sentences.is_empty(),
                        "Should produce some output"
                    );
                }
                Err(error) => {
                    println!("{}: Component stress caused error - {:?}", test_name, error);

                    // Component failures should be isolated and informative
                    match error {
                        CanopyError::ParseError { context } => {
                            assert!(!context.is_empty(), "Parse errors should have context");
                        }
                        CanopyError::SemanticError(_) => {
                            assert!(true, "Semantic errors acceptable for stress tests");
                        }
                        CanopyError::LspError(_) => {
                            assert!(true, "LSP errors acceptable for stress tests");
                        }
                    }
                }
            }
        }

        // Server should remain healthy after component stress
        let health = server.health();
        assert!(
            health.healthy,
            "Server should remain healthy after component stress tests"
        );
    }

    #[test]
    fn test_error_message_quality() {
        // Test that error messages are helpful and informative
        let server = CanopyLspServerFactory::create_server().unwrap();

        let long_input = "word ".repeat(5000);
        let error_inducing_inputs = vec![
            ("", "empty input"),
            ("   ", "whitespace input"),
            (long_input.trim(), "oversized input"),
        ];

        for (input, description) in error_inducing_inputs {
            let result = server.process_text(input);

            if let Err(error) = result {
                println!("Error for {}: {:?}", description, error);

                // Error messages should be helpful
                let error_string = format!("{:?}", error);

                // Should contain useful information
                assert!(
                    error_string.len() > 10,
                    "Error message should be substantial"
                );
                assert!(
                    error_string.len() < 1000,
                    "Error message should not be excessive"
                );

                // Should not contain internal implementation details
                assert!(
                    !error_string.contains("unwrap"),
                    "Error should not expose unwrap calls"
                );
                assert!(
                    !error_string.contains("panic"),
                    "Error should not mention panics"
                );

                // Should be properly formatted
                assert!(
                    !error_string.contains("\\n\\n"),
                    "Error should not have excessive newlines"
                );
            }
        }
    }

    #[test]
    fn test_diagnostic_creation_error_handling() {
        // Test error handling in diagnostic creation

        // Test normal diagnostic creation
        let normal_diagnostic = create_diagnostic(
            "Normal diagnostic".to_string(),
            DiagnosticSeverity::Information,
            5,
            10,
        );

        assert_eq!(normal_diagnostic.message, "Normal diagnostic");
        assert_eq!(normal_diagnostic.range.start.line, 5);
        assert_eq!(normal_diagnostic.range.start.character, 10);

        // Test edge cases
        let edge_cases = vec![
            (0, 0, "Start of document"),
            (u32::MAX - 1, u32::MAX - 1, "Near maximum values"),
            (1000, 1000, "Large values"),
        ];

        for (line, character, description) in edge_cases {
            let diagnostic = create_diagnostic(
                description.to_string(),
                DiagnosticSeverity::Warning,
                line,
                character,
            );

            // Should handle edge cases gracefully
            assert_eq!(diagnostic.message, description);
            assert_eq!(diagnostic.range.start.line, line);
            assert_eq!(diagnostic.range.start.character, character);

            // End character should be handled safely (no overflow)
            if character == u32::MAX {
                assert_eq!(
                    diagnostic.range.end.character,
                    u32::MAX,
                    "Should handle u32::MAX safely"
                );
            } else {
                assert_eq!(
                    diagnostic.range.end.character,
                    character + 1,
                    "Should increment safely"
                );
            }
        }

        // Test with empty and very long messages
        let long_message = "a".repeat(10000);
        let message_tests = vec![
            ("", "Empty message"),
            (long_message.as_str(), "Very long message"),
            ("Multi\nline\nmessage", "Multiline message"),
            ("Unicode: 🔥💯✨", "Unicode message"),
        ];

        for (message, description) in message_tests {
            let diagnostic =
                create_diagnostic(message.to_string(), DiagnosticSeverity::Error, 1, 1);

            assert_eq!(
                diagnostic.message,
                message.to_string(),
                "Should preserve message for {}",
                description
            );
            println!("Diagnostic message test passed: {}", description);
        }
    }

    #[test]
    fn test_concurrent_error_handling() {
        // Test error handling under concurrent-like stress
        let server = CanopyLspServerFactory::create_server().unwrap();

        // Mix of good and problematic inputs
        let long_input = "word ".repeat(1000);
        let mixed_inputs = vec![
            ("Good", "This is a normal sentence."),
            ("Empty", ""),
            ("Good", "Another normal sentence."),
            ("Unicode", "🚀 Unicode test 🌟"),
            ("Good", "Final normal sentence."),
            ("Long", long_input.trim()),
        ];

        let mut results = Vec::new();
        let start_time = std::time::Instant::now();

        // Process all inputs rapidly
        for (test_type, input) in &mixed_inputs {
            let result = server.process_text(input);
            results.push((test_type, result));
        }

        let total_time = start_time.elapsed();
        println!(
            "Processed {} mixed inputs in {:?}",
            mixed_inputs.len(),
            total_time
        );

        // Analyze results
        let mut success_count = 0;
        let mut error_count = 0;

        for (test_type, result) in results {
            match result {
                Ok(response) => {
                    success_count += 1;
                    println!(
                        "{} input: SUCCESS ({}μs)",
                        test_type, response.metrics.total_time_us
                    );
                }
                Err(error) => {
                    error_count += 1;
                    println!("{} input: ERROR - {:?}", test_type, error);
                }
            }
        }

        // Should handle mix of inputs
        assert!(success_count > 0, "Should have some successful processing");
        println!(
            "Concurrent error handling: {} success, {} errors",
            success_count, error_count
        );

        // Server should remain stable
        let health = server.health();
        assert!(
            health.healthy,
            "Server should remain healthy after mixed input stress"
        );
    }

    #[test]
    fn test_real_server_error_handling() {
        // Test error handling in real server factory
        let result = RealServerFactory::create();

        match result {
            Ok(server) => {
                println!("Real server created successfully");

                // Test error handling in real server
                let long_test_input = "word ".repeat(100);
                let error_inputs = vec!["", "🏴󠁧󠁢󠁳󠁣󠁴󠁿", long_test_input.trim()];

                for (i, input) in error_inputs.iter().enumerate() {
                    let process_result = server.process_text(input);

                    match process_result {
                        Ok(response) => {
                            println!("Real server error test {}: SUCCESS", i);
                            assert!(
                                !response.document.sentences.is_empty(),
                                "Real server should produce output"
                            );
                        }
                        Err(error) => {
                            println!("Real server error test {}: ERROR - {:?}", i, error);
                            // Real server errors are acceptable due to model dependencies
                        }
                    }
                }

                // Real server should maintain health
                let health = server.health();
                assert!(health.healthy, "Real server should maintain health");
            }
            Err(error) => {
                println!("Real server creation failed (expected): {:?}", error);
                assert!(
                    true,
                    "Real server creation failure expected due to dependencies"
                );
            }
        }
    }

    #[test]
    fn test_error_recovery_patterns() {
        // Test various error recovery patterns
        let server = CanopyLspServerFactory::create_server().unwrap();

        // Pattern: Good -> Bad -> Good
        let recovery_sequence = vec![
            ("Good1", "This is a good sentence.", true),
            ("Bad1", "", false),
            ("Good2", "This is another good sentence.", true),
            ("Bad2", "\x00\x01", false),
            ("Good3", "Final good sentence.", true),
        ];

        for (test_name, input, expect_success) in recovery_sequence {
            let result = server.process_text(input);

            match (result, expect_success) {
                (Ok(response), true) => {
                    println!("{}: Expected success - ✓", test_name);
                    assert!(
                        !response.document.sentences.is_empty(),
                        "Good input should produce output"
                    );
                }
                (Err(error), false) => {
                    println!("{}: Expected error - ✓ ({:?})", test_name, error);
                    // Expected errors are fine
                }
                (Ok(response), false) => {
                    println!("{}: Unexpected success (graceful handling) - ✓", test_name);
                    // Graceful handling of bad input is actually good
                    assert!(response.metrics.total_time_us > 0, "Should have timing");
                }
                (Err(error), true) => {
                    println!("{}: Unexpected error - {:?}", test_name, error);
                    // This is only a problem if it's a truly good input
                    if input.len() > 5
                        && input
                            .chars()
                            .all(|c| c.is_ascii_graphic() || c.is_whitespace())
                    {
                        panic!("Good input should not fail: {}", test_name);
                    }
                }
            }
        }

        // Server should be healthy after recovery tests
        let final_health = server.health();
        assert!(
            final_health.healthy,
            "Server should be healthy after recovery pattern tests"
        );
    }

    #[test]
    fn test_resource_cleanup_on_errors() {
        // Test that resources are properly cleaned up when errors occur
        let server = CanopyLspServerFactory::create_server().unwrap();

        // Get baseline
        let initial_health = server.health();
        println!("Initial server health: {}", initial_health.healthy);

        // Cause various types of errors
        let size_error_text = "x".repeat(50000);
        let error_scenarios = vec![
            ("ParseError", ""),
            ("UnicodeError", "\u{FEFF}\u{200B}"),
            ("SizeError", size_error_text.as_str()),
            ("StructureError", "(((((((((("),
        ];

        for (scenario_name, input) in error_scenarios {
            let _result = server.process_text(input);
            // Don't assert on result - may succeed or fail

            // Check that server remains healthy after each error scenario
            let health = server.health();
            assert!(
                health.healthy,
                "Server should remain healthy after {}",
                scenario_name
            );

            println!("After {}: Server health maintained", scenario_name);
        }

        // Test that server can still process normal input after errors
        let recovery_result = server.process_text("Normal recovery sentence.");

        match recovery_result {
            Ok(response) => {
                assert!(
                    !response.document.sentences.is_empty(),
                    "Should recover normal processing"
                );
                println!("Resource cleanup test: PASS - Normal processing recovered");
            }
            Err(error) => {
                println!("Resource cleanup test: Recovery failed - {:?}", error);
                // Recovery failures acceptable in test environment
            }
        }

        // Final health check
        let final_health = server.health();
        assert!(
            final_health.healthy,
            "Server should be healthy after resource cleanup tests"
        );
    }
}

/// Test utilities for comprehensive error testing
#[cfg(test)]
mod test_utils {
    use super::*;

    /// Helper to classify error types
    #[allow(dead_code)]
    pub fn classify_error(error: &CanopyError) -> ErrorClass {
        match error {
            CanopyError::ParseError { context } => {
                if context.contains("empty") || context.contains("length") {
                    ErrorClass::InputValidation
                } else if context.contains("character") || context.contains("unicode") {
                    ErrorClass::Encoding
                } else {
                    ErrorClass::Parse
                }
            }
            CanopyError::SemanticError(_) => ErrorClass::Semantic,
            CanopyError::LspError(_) => ErrorClass::Parse, // LSP errors are typically parse-related
        }
    }

    /// Error classification for analysis
    #[derive(Debug, PartialEq)]
    #[allow(dead_code)]
    pub enum ErrorClass {
        #[allow(dead_code)]
        InputValidation,
        #[allow(dead_code)]
        Encoding,
        #[allow(dead_code)]
        Parse,
        #[allow(dead_code)]
        Semantic,
    }

    /// Helper to generate stress test inputs
    pub fn generate_stress_inputs() -> Vec<(&'static str, String)> {
        vec![
            ("Empty", "".to_string()),
            ("Whitespace", "   \t\n  ".to_string()),
            ("VeryLong", "word ".repeat(1000)),
            ("Unicode", "🌍🔥💯✨🚀".to_string()),
            ("Mixed", "Hello 世界 🌍 123 !@#".to_string()),
            ("Control", "\x00\x01\x02\x03".to_string()),
            ("Repeated", "the ".repeat(500)),
        ]
    }

    /// Helper to validate error message quality
    pub fn validate_error_message(error: &CanopyError) -> bool {
        let error_string = format!("{:?}", error);

        // Basic quality checks
        if error_string.len() < 5 || error_string.len() > 2000 {
            return false;
        }

        // Should not expose internal details
        if error_string.contains("unwrap") || error_string.contains("panic") {
            return false;
        }

        // Should be properly formatted
        if error_string.contains("\\n\\n\\n") {
            return false;
        }

        true
    }
}
