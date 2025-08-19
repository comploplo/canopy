//! Comprehensive tests for server.rs to improve coverage from 38% to 70%+
//!
//! These tests target uncovered lines to significantly boost coverage

use crate::server::*;
use canopy_core::layer1parser::{ComponentHealth, LayerConfig, LayerHandler};
use canopy_core::{AnalysisResult, CanopyError, Word};
use std::collections::HashMap;

// Enhanced Mock implementations to cover more code paths

struct AdvancedMockParser {
    config: AdvancedMockConfig,
    should_fail: bool,
    health_status: bool,
}

struct AdvancedMockSemantics {
    config: AdvancedMockConfig,
    should_fail: bool,
    health_status: bool,
}

struct AdvancedMockConfig {
    layer_name: String,
    debug: bool,
    should_validate: bool,
}

impl LayerConfig for AdvancedMockConfig {
    fn to_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("layer".to_string(), self.layer_name.clone());
        map.insert("debug".to_string(), self.debug.to_string());
        map.insert("validate".to_string(), self.should_validate.to_string());
        map
    }

    fn validate(&self) -> Result<(), String> {
        if self.should_validate {
            Ok(())
        } else {
            Err("Validation failed".to_string())
        }
    }

    fn layer_name(&self) -> &'static str {
        "advanced_mock_config"
    }
}

impl LayerHandler<String, Vec<Word>> for AdvancedMockParser {
    fn process(&self, input: String) -> AnalysisResult<Vec<Word>> {
        if self.should_fail {
            return Err(CanopyError::ParseError {
                context: "Mock parser failure".to_string(),
            });
        }

        // Create more realistic words with different properties
        let words: Vec<Word> = input
            .split_whitespace()
            .enumerate()
            .map(|(i, word)| {
                let start = input.find(word).unwrap_or(i * 5);
                let end = start + word.len();
                let mut w = Word::new(i + 1, word.to_string(), start, end);
                // Add some variety to trigger different code paths
                if word.len() > 4 {
                    w.lemma = format!("{}_lemma", word.to_lowercase());
                }
                w
            })
            .collect();

        Ok(words)
    }

    fn config(&self) -> &dyn LayerConfig {
        &self.config
    }

    fn health(&self) -> ComponentHealth {
        let mut metrics = HashMap::new();
        metrics.insert("processed_count".to_string(), 100.0);
        metrics.insert("error_rate".to_string(), 0.05);

        ComponentHealth {
            name: "advanced_mock_parser".to_string(),
            healthy: self.health_status,
            last_error: if self.health_status {
                None
            } else {
                Some("Mock parser error".to_string())
            },
            metrics,
        }
    }
}

impl LayerHandler<Vec<Word>, Vec<Word>> for AdvancedMockSemantics {
    fn process(&self, mut input: Vec<Word>) -> AnalysisResult<Vec<Word>> {
        if self.should_fail {
            return Err(CanopyError::SemanticError(
                "Mock semantics failure".to_string(),
            ));
        }

        // Add semantic enhancements to words
        for word in &mut input {
            if word.text.to_lowercase().contains("cat") {
                word.lemma = "feline".to_string();
            }
            if word.text.to_lowercase().contains("run") {
                word.lemma = "motion_verb".to_string();
            }
        }

        Ok(input)
    }

    fn config(&self) -> &dyn LayerConfig {
        &self.config
    }

    fn health(&self) -> ComponentHealth {
        let mut metrics = HashMap::new();
        metrics.insert("semantic_coverage".to_string(), 0.85);
        metrics.insert("confidence_avg".to_string(), 0.92);

        ComponentHealth {
            name: "advanced_mock_semantics".to_string(),
            healthy: self.health_status,
            last_error: if self.health_status {
                None
            } else {
                Some("Mock semantics error".to_string())
            },
            metrics,
        }
    }
}

fn create_healthy_server() -> DefaultCanopyServer<AdvancedMockParser, AdvancedMockSemantics> {
    let parser = AdvancedMockParser {
        config: AdvancedMockConfig {
            layer_name: "parser".to_string(),
            debug: false,
            should_validate: true,
        },
        should_fail: false,
        health_status: true,
    };

    let semantics = AdvancedMockSemantics {
        config: AdvancedMockConfig {
            layer_name: "semantics".to_string(),
            debug: true,
            should_validate: true,
        },
        should_fail: false,
        health_status: true,
    };

    DefaultCanopyServer::new(parser, semantics)
}

fn create_failing_server() -> DefaultCanopyServer<AdvancedMockParser, AdvancedMockSemantics> {
    let parser = AdvancedMockParser {
        config: AdvancedMockConfig {
            layer_name: "parser".to_string(),
            debug: true,
            should_validate: false,
        },
        should_fail: true,
        health_status: false,
    };

    let semantics = AdvancedMockSemantics {
        config: AdvancedMockConfig {
            layer_name: "semantics".to_string(),
            debug: false,
            should_validate: false,
        },
        should_fail: true,
        health_status: false,
    };

    DefaultCanopyServer::new(parser, semantics)
}

#[test]
fn test_server_config_default() {
    let config = ServerConfig::default();
    assert!(config.enable_metrics);
    assert_eq!(config.timeout_ms, 5000);
    assert!(!config.debug);
    assert!(config.layer_configs.is_empty());
}

#[test]
fn test_server_config_custom() {
    let mut layer_configs = HashMap::new();
    let mut parser_config = HashMap::new();
    parser_config.insert("model_path".to_string(), "/tmp/model.bin".to_string());
    layer_configs.insert("parser".to_string(), parser_config);

    let config = ServerConfig {
        enable_metrics: false,
        timeout_ms: 10000,
        debug: true,
        layer_configs,
    };

    assert!(!config.enable_metrics);
    assert_eq!(config.timeout_ms, 10000);
    assert!(config.debug);
    assert_eq!(config.layer_configs.len(), 1);
}

#[test]
fn test_server_stats_default() {
    let stats = ServerStats::default();
    assert_eq!(stats.requests, 0);
    assert_eq!(stats.total_time_us, 0);
    assert_eq!(stats.errors, 0);
    // start_time should be recent
    assert!(stats.start_time.elapsed().as_secs() < 1);
}

#[test]
fn test_server_with_config() {
    let parser = AdvancedMockParser {
        config: AdvancedMockConfig {
            layer_name: "custom_parser".to_string(),
            debug: true,
            should_validate: true,
        },
        should_fail: false,
        health_status: true,
    };

    let semantics = AdvancedMockSemantics {
        config: AdvancedMockConfig {
            layer_name: "custom_semantics".to_string(),
            debug: false,
            should_validate: true,
        },
        should_fail: false,
        health_status: true,
    };

    let custom_config = ServerConfig {
        enable_metrics: true,
        timeout_ms: 15000,
        debug: true,
        layer_configs: HashMap::new(),
    };

    let server = DefaultCanopyServer::with_config(parser, semantics, custom_config);

    // Test that server works with custom config
    let response = server.process_text("Custom test input").unwrap();
    assert!(!response.document.text.is_empty());
    assert!(response.metrics.total_time_us > 0);
}

#[test]
fn test_analysis_response_structure() {
    let server = create_healthy_server();
    let response = server.process_text("The quick brown fox jumps").unwrap();

    // Test AnalysisResponse structure
    assert_eq!(response.document.text, "The quick brown fox jumps");
    assert_eq!(response.document.sentences.len(), 1);
    assert_eq!(response.document.sentences[0].words.len(), 5);

    // Test layer results
    assert_eq!(response.layer_results.len(), 2);
    assert!(response.layer_results.contains_key("layer1"));
    assert!(response.layer_results.contains_key("semantics"));

    let layer1_result = &response.layer_results["layer1"];
    assert_eq!(layer1_result.layer, "layer1");
    assert_eq!(layer1_result.items_processed, 5);
    assert_eq!(layer1_result.confidence, 0.85);
    assert!(layer1_result.processing_time_us > 0);

    let semantics_result = &response.layer_results["semantics"];
    assert_eq!(semantics_result.layer, "semantics");
    assert_eq!(semantics_result.items_processed, 5);
    assert_eq!(semantics_result.confidence, 0.75);
    assert!(semantics_result.processing_time_us > 0);
}

#[test]
fn test_analysis_metrics_structure() {
    let server = create_healthy_server();
    let response = server.process_text("Test metrics analysis").unwrap();

    // Test AnalysisMetrics structure
    let metrics = &response.metrics;
    assert!(metrics.total_time_us > 0);
    assert_eq!(metrics.layer_times.len(), 2);
    assert!(metrics.layer_times.contains_key("layer1"));
    assert!(metrics.layer_times.contains_key("semantics"));

    // Test MemoryStats
    let memory = &metrics.memory_stats;
    assert!(memory.peak_bytes > memory.final_bytes);
    assert!(memory.allocations >= 3); // Should be at least words + overhead

    // Test InputStats
    let input_stats = &metrics.input_stats;
    assert_eq!(input_stats.char_count, "Test metrics analysis".len());
    assert_eq!(input_stats.word_count, 3);
    assert_eq!(input_stats.sentence_count, 1);
}

#[test]
fn test_server_health_comprehensive() {
    let server = create_healthy_server();

    // Process some requests to build up stats
    for i in 0..5 {
        let _ = server.process_text(&format!("Test request {}", i));
    }

    let health = server.health();

    // Test ServerHealth structure
    assert!(health.healthy);
    assert_eq!(health.components.len(), 2);
    assert!(health.uptime_seconds >= 0);
    assert_eq!(health.requests_processed, 5);
    assert!(health.avg_response_time_us > 0);

    // Test component health
    let parser_health = &health.components["layer1"];
    assert_eq!(parser_health.name, "advanced_mock_parser");
    assert!(parser_health.healthy);
    assert!(parser_health.last_error.is_none());
    assert!(!parser_health.metrics.is_empty());

    let semantics_health = &health.components["semantics"];
    assert_eq!(semantics_health.name, "advanced_mock_semantics");
    assert!(semantics_health.healthy);
    assert!(semantics_health.last_error.is_none());
    assert!(!semantics_health.metrics.is_empty());
}

#[test]
fn test_server_health_unhealthy() {
    let server = create_failing_server();
    let health = server.health();

    // Overall health should be false if any component is unhealthy
    assert!(!health.healthy);

    // Both components should be unhealthy
    let parser_health = &health.components["layer1"];
    assert!(!parser_health.healthy);
    assert!(parser_health.last_error.is_some());

    let semantics_health = &health.components["semantics"];
    assert!(!semantics_health.healthy);
    assert!(semantics_health.last_error.is_some());
}

#[test]
fn test_parser_failure_handling() {
    let server = create_failing_server();
    let result = server.process_text("This should fail");

    assert!(result.is_err());
    match result {
        Err(CanopyError::ParseError { context }) => {
            assert_eq!(context, "Mock parser failure");
        }
        _ => panic!("Expected ParseError"),
    }

    // Check that error stats are updated
    let health = server.health();
    assert!(health.requests_processed >= 1);
}

#[test]
fn test_semantics_failure_handling() {
    // Create server where parser succeeds but semantics fails
    let parser = AdvancedMockParser {
        config: AdvancedMockConfig {
            layer_name: "parser".to_string(),
            debug: false,
            should_validate: true,
        },
        should_fail: false,
        health_status: true,
    };

    let semantics = AdvancedMockSemantics {
        config: AdvancedMockConfig {
            layer_name: "semantics".to_string(),
            debug: false,
            should_validate: true,
        },
        should_fail: true,
        health_status: true,
    };

    let server = DefaultCanopyServer::new(parser, semantics);
    let result = server.process_text("This should fail at semantics");

    assert!(result.is_err());
    match result {
        Err(CanopyError::SemanticError(context)) => {
            assert_eq!(context, "Mock semantics failure");
        }
        _ => panic!("Expected SemanticError"),
    }
}

#[test]
fn test_empty_input_variations() {
    let server = create_healthy_server();

    // Test various empty input patterns
    let empty_inputs = ["", "   ", "\t", "\n", "  \n  \t  "];

    for input in &empty_inputs {
        let result = server.process_text(input);
        assert!(result.is_err());

        if let Err(CanopyError::ParseError { context }) = result {
            assert_eq!(context, "Empty input text");
        } else {
            panic!("Expected ParseError for input: '{}'", input);
        }
    }
}

#[test]
fn test_stats_tracking() {
    let server = create_healthy_server();

    // Process multiple requests
    for i in 0..10 {
        let text = format!("Request number {}", i);
        let _ = server.process_text(&text);
    }

    let health = server.health();
    assert_eq!(health.requests_processed, 10);
    assert!(health.avg_response_time_us > 0);
    assert!(health.uptime_seconds >= 0);
}

#[test]
fn test_stats_tracking_with_errors() {
    let server = create_failing_server();

    // Try multiple requests that will fail
    for i in 0..5 {
        let text = format!("Failing request {}", i);
        let _ = server.process_text(&text);
    }

    let health = server.health();
    assert_eq!(health.requests_processed, 5);
    // All requests should have failed, so stats should reflect that
}

#[test]
fn test_layer_config_validation() {
    let valid_config = AdvancedMockConfig {
        layer_name: "test".to_string(),
        debug: true,
        should_validate: true,
    };
    assert!(valid_config.validate().is_ok());

    let invalid_config = AdvancedMockConfig {
        layer_name: "test".to_string(),
        debug: false,
        should_validate: false,
    };
    assert!(invalid_config.validate().is_err());
}

#[test]
fn test_layer_config_to_map() {
    let config = AdvancedMockConfig {
        layer_name: "test_layer".to_string(),
        debug: true,
        should_validate: false,
    };

    let map = config.to_map();
    assert_eq!(map.get("layer").unwrap(), "test_layer");
    assert_eq!(map.get("debug").unwrap(), "true");
    assert_eq!(map.get("validate").unwrap(), "false");
}

#[test]
fn test_memory_stats_calculations() {
    let server = create_healthy_server();
    let long_text = "This is a much longer text input that should result in higher memory usage estimates and more detailed statistics tracking.";

    let response = server.process_text(long_text).unwrap();
    let memory = &response.metrics.memory_stats;

    // Memory estimates should scale with input size
    assert!(memory.peak_bytes > 100); // Should be substantial for long text
    assert!(memory.final_bytes < memory.peak_bytes); // Final should be less than peak
    assert!(memory.allocations > 5); // Should have multiple allocations
}

#[test]
fn test_concurrent_access_simulation() {
    use std::sync::Arc;
    use std::thread;

    let server = Arc::new(create_healthy_server());
    let mut handles = vec![];

    // Simulate concurrent requests
    for i in 0..5 {
        let server_clone = Arc::clone(&server);
        let handle = thread::spawn(move || {
            let text = format!("Concurrent request {}", i);
            server_clone.process_text(&text)
        });
        handles.push(handle);
    }

    // Wait for all threads and collect results
    let mut success_count = 0;
    for handle in handles {
        if handle.join().unwrap().is_ok() {
            success_count += 1;
        }
    }

    assert_eq!(success_count, 5);

    // Check final stats
    let health = server.health();
    assert_eq!(health.requests_processed, 5);
}

#[test]
fn test_process_pipeline_edge_cases() {
    let server = create_healthy_server();

    // Test various input patterns that might trigger different code paths
    let test_cases = [
        "Single",
        "Two words",
        "Multiple words in a longer sentence",
        "Numbers 123 and symbols !@#",
        "Mixed CASE and punctuation.",
        "Unicode: café naïve résumé",
    ];

    for test_case in &test_cases {
        let response = server.process_text(test_case).unwrap();

        // Verify basic structure
        assert_eq!(response.document.text, *test_case);
        assert!(!response.document.sentences.is_empty());
        assert!(response.metrics.total_time_us > 0);

        // Verify layer results
        assert!(response.layer_results.contains_key("layer1"));
        assert!(response.layer_results.contains_key("semantics"));

        // Verify input stats match
        assert_eq!(response.metrics.input_stats.char_count, test_case.len());
    }
}
