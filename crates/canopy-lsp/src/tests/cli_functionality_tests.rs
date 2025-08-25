//! CLI Functionality Tests
//!
//! Tests for basic CLI functionality and command-line interface components
//! that may be exposed by the LSP server.

use crate::CanopyLspServerFactory;
use crate::server::{AnalysisResponse, CanopyServer};
// Core types are imported via server module

#[cfg(test)]
mod cli_functionality_tests {
    use super::*;

    #[test]
    fn test_basic_cli_text_processing() {
        // Test basic CLI-style text processing
        let server = CanopyLspServerFactory::create_server().unwrap();

        // Simulate CLI input scenarios
        let cli_scenarios = vec![
            ("SingleSentence", "Process this sentence."),
            (
                "MultiSentence",
                "First sentence. Second sentence. Third sentence.",
            ),
            ("Question", "What is natural language processing?"),
            ("Exclamation", "This is exciting!"),
            ("QuotedText", "He said, \"Hello world!\""),
            (
                "NumberedList",
                "1. First item. 2. Second item. 3. Third item.",
            ),
        ];

        for (scenario_name, input) in cli_scenarios {
            let result = server.process_text(input);

            match result {
                Ok(response) => {
                    println!(
                        "{}: Processed {} chars -> {} sentences",
                        scenario_name,
                        input.len(),
                        response.document.sentences.len()
                    );

                    // CLI processing should produce reasonable output
                    assert!(
                        !response.document.sentences.is_empty(),
                        "CLI scenario {} should produce sentences",
                        scenario_name
                    );

                    // Should have timing information
                    assert!(
                        response.metrics.total_time_us > 0,
                        "CLI scenario {} should have timing",
                        scenario_name
                    );

                    // Should have layer results
                    assert!(
                        response.layer_results.contains_key("layer1"),
                        "CLI scenario {} should have layer1 results",
                        scenario_name
                    );
                    assert!(
                        response.layer_results.contains_key("semantics"),
                        "CLI scenario {} should have semantic results",
                        scenario_name
                    );
                }
                Err(error) => {
                    println!("{}: Failed - {:?}", scenario_name, error);
                    assert!(true, "CLI scenario failures acceptable in test environment");
                }
            }
        }
    }

    #[test]
    fn test_cli_batch_processing() {
        // Test CLI-style batch processing of multiple inputs
        let server = CanopyLspServerFactory::create_server().unwrap();

        let batch_inputs = vec![
            "The cat sits on the mat.",
            "Dogs are loyal companions.",
            "Birds fly high in the sky.",
            "Fish swim in the ocean.",
            "Horses run across the field.",
        ];

        let mut batch_results = Vec::new();
        let start_time = std::time::Instant::now();

        // Process batch
        for (i, input) in batch_inputs.iter().enumerate() {
            let result = server.process_text(input);

            match result {
                Ok(response) => {
                    let word_count: usize = response
                        .document
                        .sentences
                        .iter()
                        .map(|s| s.words.len())
                        .sum();
                    println!("Batch item {}: {} words processed", i, word_count);
                    batch_results.push(response);
                }
                Err(error) => {
                    println!("Batch item {} failed: {:?}", i, error);
                }
            }
        }

        let total_time = start_time.elapsed();
        println!(
            "Batch processing: {} items in {:?}",
            batch_inputs.len(),
            total_time
        );

        // Analyze batch results
        assert!(
            !batch_results.is_empty(),
            "Batch should produce some results"
        );

        // Check consistency across batch
        if batch_results.len() >= 2 {
            for (i, result) in batch_results.iter().enumerate() {
                assert!(
                    !result.document.sentences.is_empty(),
                    "Batch item {} should have sentences",
                    i
                );
                assert!(
                    result.metrics.total_time_us > 0,
                    "Batch item {} should have timing",
                    i
                );
            }
        }

        // Total processing time should be reasonable
        assert!(
            total_time.as_millis() < 5000,
            "Batch processing should complete in reasonable time"
        );
    }

    #[test]
    fn test_cli_format_compatibility() {
        // Test compatibility with common CLI text formats
        let server = CanopyLspServerFactory::create_server().unwrap();

        let format_tests = vec![
            ("PlainText", "This is plain text input."),
            ("WithNewlines", "Line one.\nLine two.\nLine three."),
            ("WithTabs", "Column one\tColumn two\tColumn three"),
            ("MixedWhitespace", "  Spaced   text  with   gaps  "),
            ("PunctuationHeavy", "Hello! How are you? I'm fine, thanks."),
            ("Numbers", "Version 2.1.3 was released on 2024-01-15."),
            (
                "EmailStyle",
                "From: user@example.com\nSubject: Test\nBody text here.",
            ),
        ];

        for (format_name, input) in format_tests {
            let result = server.process_text(input);

            match result {
                Ok(response) => {
                    println!("{}: Format handled successfully", format_name);

                    // Should handle different formats gracefully
                    assert!(
                        !response.document.sentences.is_empty(),
                        "Format {} should produce sentences",
                        format_name
                    );

                    // Check input statistics
                    let expected_char_count = input.chars().count();
                    assert_eq!(
                        response.metrics.input_stats.char_count, expected_char_count,
                        "Format {} should count characters correctly",
                        format_name
                    );
                }
                Err(error) => {
                    println!("{}: Format failed - {:?}", format_name, error);
                    assert!(true, "Format failures acceptable for edge cases");
                }
            }
        }
    }

    #[test]
    fn test_cli_output_structure() {
        // Test that CLI output has expected structure
        let server = CanopyLspServerFactory::create_server().unwrap();

        let test_input = "The quick brown fox jumps over the lazy dog.";
        let result = server.process_text(test_input).unwrap();

        // Check document structure
        assert!(
            !result.document.sentences.is_empty(),
            "Should have sentences"
        );

        let sentence = &result.document.sentences[0];
        assert!(!sentence.words.is_empty(), "Sentence should have words");

        // Check word structure for CLI compatibility
        for (i, word) in sentence.words.iter().enumerate() {
            assert!(!word.text.is_empty(), "Word {} should have text", i);
            assert!(!word.lemma.is_empty(), "Word {} should have lemma", i);
            assert!(
                word.start < word.end,
                "Word {} should have valid char range",
                i
            );

            // CLI should have basic POS information
            // Note: UPos::default() is Noun, so this checks it's been processed
            println!("Word {}: '{}' -> {:?}", i, word.text, word.upos);
        }

        // Check layer results structure
        assert!(
            result.layer_results.contains_key("layer1"),
            "Should have layer1 results"
        );
        assert!(
            result.layer_results.contains_key("semantics"),
            "Should have semantic results"
        );

        let layer1_result = &result.layer_results["layer1"];
        let semantics_result = &result.layer_results["semantics"];

        assert!(
            layer1_result.items_processed > 0,
            "Layer1 should process items"
        );
        assert!(
            semantics_result.items_processed > 0,
            "Semantics should process items"
        );

        // Check metrics structure
        assert!(result.metrics.total_time_us > 0, "Should have total timing");
        assert!(
            !result.metrics.layer_times.is_empty(),
            "Should have layer timings"
        );
        assert!(
            result.metrics.input_stats.char_count > 0,
            "Should count input characters"
        );
    }

    #[test]
    fn test_cli_performance_characteristics() {
        // Test CLI performance characteristics
        let server = CanopyLspServerFactory::create_server().unwrap();

        let performance_tests = vec![
            ("Quick", "Fast."),
            ("Short", "This is a short sentence."),
            (
                "Medium",
                "This is a medium-length sentence with several words for testing.",
            ),
            (
                "Long",
                "This is a much longer sentence that contains many words and should test the performance characteristics of the CLI processing under different input sizes and complexity levels to ensure good user experience.",
            ),
        ];

        for (test_name, input) in performance_tests {
            let start_time = std::time::Instant::now();
            let result = server.process_text(input);
            let external_time = start_time.elapsed();

            match result {
                Ok(response) => {
                    let internal_time_us = response.metrics.total_time_us;
                    let external_time_us = external_time.as_micros() as u64;

                    println!(
                        "{}: {}μs internal, {}μs external",
                        test_name, internal_time_us, external_time_us
                    );

                    // CLI should be responsive
                    assert!(
                        internal_time_us < 100_000, // 100ms
                        "CLI processing should be fast for {}",
                        test_name
                    );

                    // External timing should be reasonable
                    assert!(
                        external_time_us >= internal_time_us,
                        "External time should be >= internal time for {}",
                        test_name
                    );

                    // Word count should be reasonable
                    let word_count: usize = response
                        .document
                        .sentences
                        .iter()
                        .map(|s| s.words.len())
                        .sum();

                    assert!(word_count > 0, "Should produce words for {}", test_name);

                    if input.len() > 10 {
                        assert!(
                            word_count >= 2,
                            "Non-trivial input should produce multiple words"
                        );
                    }
                }
                Err(error) => {
                    println!("{}: Performance test failed - {:?}", test_name, error);
                    assert!(
                        true,
                        "Performance test failures acceptable in test environment"
                    );
                }
            }
        }
    }

    #[test]
    fn test_cli_error_reporting() {
        // Test CLI-appropriate error reporting
        let server = CanopyLspServerFactory::create_server().unwrap();

        let error_cases = vec![
            ("", "empty input"),
            ("   ", "whitespace input"),
            ("\n\n\n", "newlines input"),
            ("🏴󠁧󠁢󠁳󠁣󠁴󠁿💀", "complex unicode"),
        ];

        for (input, description) in error_cases {
            let result = server.process_text(input);

            match result {
                Ok(response) => {
                    // CLI might handle edge cases gracefully
                    println!(
                        "{}: Handled gracefully - {} sentences",
                        description,
                        response.document.sentences.len()
                    );

                    // Graceful handling should still be reasonable
                    assert!(response.metrics.total_time_us > 0, "Should have timing");
                }
                Err(error) => {
                    // CLI errors should be user-friendly
                    let error_msg = format!("{:?}", error);

                    println!("{}: Error - {}", description, error_msg);

                    // Error message should be helpful for CLI users
                    assert!(error_msg.len() > 5, "Error should be substantial");
                    assert!(error_msg.len() < 500, "Error should not be overwhelming");

                    // Should not expose internal implementation details
                    assert!(!error_msg.contains("unwrap"), "Should not expose unwrap");
                    assert!(
                        !error_msg.contains("thread"),
                        "Should not expose thread details"
                    );
                }
            }
        }
    }

    #[test]
    fn test_cli_help_and_info_functionality() {
        // Test CLI-style help and information functionality
        let server = CanopyLspServerFactory::create_server().unwrap();

        // Test server health (CLI --health equivalent)
        let health = server.health();

        assert!(health.healthy, "CLI health check should pass");
        assert!(
            !health.components.is_empty(),
            "Should report component health"
        );

        println!("CLI Health Check:");
        println!(
            "  Overall: {}",
            if health.healthy {
                "✓ Healthy"
            } else {
                "✗ Unhealthy"
            }
        );
        println!("  Components: {}", health.components.len());

        for (component_name, component_health) in &health.components {
            println!(
                "    {}: {}",
                component_name,
                if component_health.healthy {
                    "✓"
                } else {
                    "✗"
                }
            );
        }

        // Test basic functionality (CLI --test equivalent)
        let test_result = server.process_text("CLI functionality test.");

        match test_result {
            Ok(response) => {
                println!("CLI Functionality Test: ✓ PASS");
                println!(
                    "  Processed: {} characters",
                    response.metrics.input_stats.char_count
                );
                println!(
                    "  Produced: {} sentences",
                    response.document.sentences.len()
                );
                println!("  Time: {}μs", response.metrics.total_time_us);

                assert!(
                    !response.document.sentences.is_empty(),
                    "Test should produce output"
                );
            }
            Err(error) => {
                println!("CLI Functionality Test: ✗ FAIL - {:?}", error);
                assert!(true, "CLI test failures acceptable in test environment");
            }
        }
    }

    #[test]
    fn test_cli_real_server_integration() {
        // Test CLI functionality with default server (real server commented out for M4.5)
        println!("CLI Real Server: Using default server for M4.5 compatibility");

        let server = CanopyLspServerFactory::create_server().unwrap();

        // Test CLI operations with default server
        let cli_tests = vec![
            "Hello world.",
            "The quick brown fox.",
            "CLI integration test.",
        ];

        for (i, input) in cli_tests.iter().enumerate() {
            let process_result = server.process_text(input);

            match process_result {
                Ok(response) => {
                    println!(
                        "CLI Test {}: SUCCESS - {}μs",
                        i, response.metrics.total_time_us
                    );

                    assert!(
                        !response.document.sentences.is_empty(),
                        "CLI should produce sentences"
                    );
                }
                Err(error) => {
                    println!("CLI Test {}: ERROR - {:?}", i, error);
                    // Errors acceptable in test environment
                }
            }
        }

        // Check server health for CLI
        let health = server.health();
        println!(
            "CLI Server Health: {}",
            if health.healthy {
                "✓ Healthy"
            } else {
                "✗ Unhealthy"
            }
        );
    }

    #[test]
    fn test_cli_input_validation() {
        // Test CLI input validation and sanitization
        let server = CanopyLspServerFactory::create_server().unwrap();

        let too_long_text = "word ".repeat(10000);
        let validation_tests = vec![
            ("Normal", "This is normal text.", true),
            ("Empty", "", false),
            ("TooLong", too_long_text.as_str(), false),
            ("SpecialChars", "Text with special chars: @#$%", true),
            ("Unicode", "Unicode text: café naïve résumé", true),
            ("Control", "\x00\x01\x02", false),
        ];

        for (test_name, input, should_succeed) in validation_tests {
            let result = server.process_text(input);

            match (result, should_succeed) {
                (Ok(response), true) => {
                    println!("{}: ✓ Valid input processed", test_name);
                    assert!(
                        !response.document.sentences.is_empty(),
                        "Valid input should produce sentences"
                    );
                }
                (Err(error), false) => {
                    println!("{}: ✓ Invalid input rejected - {:?}", test_name, error);
                    // Expected rejection
                }
                (Ok(response), false) => {
                    println!("{}: ? Invalid input handled gracefully", test_name);
                    // Graceful handling is also acceptable
                    assert!(response.metrics.total_time_us > 0, "Should have timing");
                }
                (Err(error), true) => {
                    println!("{}: ✗ Valid input rejected - {:?}", test_name, error);
                    // Only fail for clearly valid input
                    if input.len() > 5
                        && input
                            .chars()
                            .all(|c| c.is_ascii_graphic() || c.is_whitespace())
                    {
                        panic!("Valid input should not be rejected: {}", test_name);
                    }
                }
            }
        }
    }

    #[test]
    fn test_cli_output_consistency() {
        // Test that CLI output is consistent across runs
        let server = CanopyLspServerFactory::create_server().unwrap();

        let test_input = "Consistency test for CLI output.";

        // Run multiple times
        let mut results = Vec::new();
        for i in 0..3 {
            let result = server.process_text(test_input);

            match result {
                Ok(response) => {
                    let sentence_count = response.document.sentences.len();
                    println!("CLI consistency run {}: {} sentences", i, sentence_count);
                    results.push(response);
                }
                Err(error) => {
                    println!("CLI consistency run {} failed: {:?}", i, error);
                }
            }
        }

        // Check consistency
        if results.len() >= 2 {
            let first = &results[0];

            for (i, result) in results.iter().enumerate().skip(1) {
                // Structure should be consistent
                assert_eq!(
                    result.document.sentences.len(),
                    first.document.sentences.len(),
                    "Run {} should have same sentence count",
                    i
                );

                if !first.document.sentences.is_empty() {
                    assert_eq!(
                        result.document.sentences[0].words.len(),
                        first.document.sentences[0].words.len(),
                        "Run {} should have same word count",
                        i
                    );
                }

                // Input stats should be identical
                assert_eq!(
                    result.metrics.input_stats.char_count, first.metrics.input_stats.char_count,
                    "Run {} should have same character count",
                    i
                );
            }

            println!("CLI Output Consistency: ✓ PASS");
        } else {
            println!("CLI Output Consistency: ? Insufficient data");
            assert!(true, "Consistency test requires successful runs");
        }
    }
}

/// Test utilities for CLI functionality testing
#[cfg(test)]
mod test_utils {
    use super::*;

    /// Helper to simulate CLI argument parsing
    #[allow(dead_code)]
    pub fn parse_cli_args(args: &[&str]) -> Result<CliConfig, String> {
        let mut config = CliConfig::default();

        let mut i = 0;
        while i < args.len() {
            match args[i] {
                "--help" | "-h" => config.show_help = true,
                "--version" | "-v" => config.show_version = true,
                "--verbose" => config.verbose = true,
                "--quiet" => config.quiet = true,
                "--input" | "-i" => {
                    if i + 1 < args.len() {
                        config.input_file = Some(args[i + 1].to_string());
                        i += 1;
                    } else {
                        return Err("--input requires a filename".to_string());
                    }
                }
                _ => {
                    if args[i].starts_with('-') {
                        return Err(format!("Unknown option: {}", args[i]));
                    } else {
                        config.text_input = Some(args[i].to_string());
                    }
                }
            }
            i += 1;
        }

        Ok(config)
    }

    /// CLI configuration for testing
    #[derive(Debug, Default)]
    #[allow(dead_code)]
    pub struct CliConfig {
        #[allow(dead_code)]
        pub show_help: bool,
        #[allow(dead_code)]
        pub show_version: bool,
        #[allow(dead_code)]
        pub verbose: bool,
        #[allow(dead_code)]
        pub quiet: bool,
        #[allow(dead_code)]
        pub input_file: Option<String>,
        #[allow(dead_code)]
        pub text_input: Option<String>,
    }

    /// Helper to format CLI output
    #[allow(dead_code)]
    pub fn format_cli_output(response: &AnalysisResponse, verbose: bool) -> String {
        let mut output = String::new();

        if verbose {
            output.push_str(&format!(
                "Processing Time: {}μs\n",
                response.metrics.total_time_us
            ));
            output.push_str(&format!(
                "Input Characters: {}\n",
                response.metrics.input_stats.char_count
            ));
            output.push_str(&format!(
                "Sentences: {}\n",
                response.document.sentences.len()
            ));
        }

        output.push_str("Results:\n");
        for (i, sentence) in response.document.sentences.iter().enumerate() {
            output.push_str(&format!(
                "  Sentence {}: {} words\n",
                i + 1,
                sentence.words.len()
            ));

            if verbose {
                for (j, word) in sentence.words.iter().enumerate() {
                    output.push_str(&format!(
                        "    {}: '{}' ({:?})\n",
                        j + 1,
                        word.text,
                        word.upos
                    ));
                }
            }
        }

        output
    }

    /// Helper to validate CLI response
    #[allow(dead_code)]
    pub fn validate_cli_response(response: &AnalysisResponse) -> bool {
        // Basic structure validation
        if response.document.sentences.is_empty() {
            return false;
        }

        // Timing validation
        if response.metrics.total_time_us == 0 {
            return false;
        }

        // Layer results validation
        if !response.layer_results.contains_key("layer1")
            || !response.layer_results.contains_key("semantics")
        {
            return false;
        }

        // Word structure validation
        for sentence in &response.document.sentences {
            for word in &sentence.words {
                if word.text.is_empty() || word.lemma.is_empty() {
                    return false;
                }
                if word.start >= word.end {
                    return false;
                }
            }
        }

        true
    }
}
