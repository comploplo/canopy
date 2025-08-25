//! Comprehensive tests for EngineError and error handling

use canopy_engine::error::{EngineError, ErrorCategory, ErrorStats};
use std::collections::HashMap;

#[cfg(test)]
mod tests {
    use super::*;

    // EngineError Construction Tests

    #[test]
    fn test_data_load_error() {
        let error = EngineError::data_load("Failed to load model file");

        match &error {
            EngineError::DataLoadError { context, source } => {
                assert_eq!(context, "Failed to load model file");
                assert!(source.is_none());
            }
            _ => panic!("Expected DataLoadError"),
        }

        assert!(error.to_string().contains("Data loading failed"));
        assert!(error.to_string().contains("Failed to load model file"));
        assert_eq!(error.category(), ErrorCategory::DataLoad);
        assert!(!error.is_recoverable());
    }

    #[test]
    fn test_data_load_error_with_source() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let error = EngineError::data_load_with_source("Cannot read config", io_error);

        match &error {
            EngineError::DataLoadError { context, source } => {
                assert_eq!(context, "Cannot read config");
                assert!(source.is_some());
            }
            _ => panic!("Expected DataLoadError"),
        }

        assert!(error.to_string().contains("Cannot read config"));
        assert_eq!(error.category(), ErrorCategory::DataLoad);
    }

    #[test]
    fn test_analysis_error() {
        let error = EngineError::analysis("hello world", "unsupported language");

        match &error {
            EngineError::AnalysisError {
                input,
                reason,
                source,
            } => {
                assert_eq!(input, "hello world");
                assert_eq!(reason, "unsupported language");
                assert!(source.is_none());
            }
            _ => panic!("Expected AnalysisError"),
        }

        assert!(error.to_string().contains("Analysis failed"));
        assert!(error.to_string().contains("hello world"));
        assert!(error.to_string().contains("unsupported language"));
        assert_eq!(error.category(), ErrorCategory::Analysis);
        assert!(!error.is_recoverable());
        assert!(error.should_retry_with_different_input());
    }

    #[test]
    fn test_analysis_error_with_source() {
        let inner_error = std::fmt::Error;
        let error = EngineError::analysis_with_source("input text", "parsing failed", inner_error);

        match &error {
            EngineError::AnalysisError {
                input,
                reason,
                source,
            } => {
                assert_eq!(input, "input text");
                assert_eq!(reason, "parsing failed");
                assert!(source.is_some());
            }
            _ => panic!("Expected AnalysisError"),
        }

        assert!(error.should_retry_with_different_input());
    }

    #[test]
    fn test_cache_error() {
        let error = EngineError::cache("Redis connection timeout");

        match &error {
            EngineError::CacheError { operation, source } => {
                assert_eq!(operation, "Redis connection timeout");
                assert!(source.is_none());
            }
            _ => panic!("Expected CacheError"),
        }

        assert!(error.to_string().contains("Cache operation failed"));
        assert_eq!(error.category(), ErrorCategory::Cache);
        assert!(error.is_recoverable());
        assert!(!error.should_retry_with_different_input());
    }

    #[test]
    fn test_config_error() {
        let error = EngineError::config("Invalid timeout value: -1");

        match &error {
            EngineError::ConfigError { message } => {
                assert_eq!(message, "Invalid timeout value: -1");
            }
            _ => panic!("Expected ConfigError"),
        }

        assert!(error.to_string().contains("Configuration error"));
        assert_eq!(error.category(), ErrorCategory::Configuration);
        assert!(!error.is_recoverable());
    }

    #[test]
    fn test_resource_not_found_error() {
        let error = EngineError::resource_not_found("VerbNet class", "run-51.3.2");

        match &error {
            EngineError::ResourceNotFound {
                resource_type,
                identifier,
            } => {
                assert_eq!(resource_type, "VerbNet class");
                assert_eq!(identifier, "run-51.3.2");
            }
            _ => panic!("Expected ResourceNotFound"),
        }

        assert!(error.to_string().contains("Resource not found"));
        assert!(error.to_string().contains("VerbNet class"));
        assert!(error.to_string().contains("run-51.3.2"));
        assert_eq!(error.category(), ErrorCategory::Resource);
    }

    #[test]
    fn test_invalid_input_error() {
        let error = EngineError::invalid_input("UTF-8 string", "binary data");

        match &error {
            EngineError::InvalidInput { expected, actual } => {
                assert_eq!(expected, "UTF-8 string");
                assert_eq!(actual, "binary data");
            }
            _ => panic!("Expected InvalidInput"),
        }

        assert!(error.to_string().contains("Invalid input format"));
        assert!(error.to_string().contains("UTF-8 string expected"));
        assert!(error.to_string().contains("got binary data"));
        assert_eq!(error.category(), ErrorCategory::Input);
        assert!(error.should_retry_with_different_input());
    }

    #[test]
    fn test_not_initialized_error() {
        let error = EngineError::not_initialized("VerbNetEngine");

        match &error {
            EngineError::NotInitialized { engine_name } => {
                assert_eq!(engine_name, "VerbNetEngine");
            }
            _ => panic!("Expected NotInitialized"),
        }

        assert!(error.to_string().contains("Engine not initialized"));
        assert!(error.to_string().contains("VerbNetEngine"));
        assert_eq!(error.category(), ErrorCategory::Initialization);
    }

    #[test]
    fn test_timeout_error() {
        let error = EngineError::timeout("semantic analysis", 5000);

        match &error {
            EngineError::Timeout {
                operation,
                timeout_ms,
            } => {
                assert_eq!(operation, "semantic analysis");
                assert_eq!(*timeout_ms, 5000);
            }
            _ => panic!("Expected Timeout"),
        }

        assert!(error.to_string().contains("Timeout occurred"));
        assert!(error.to_string().contains("semantic analysis"));
        assert!(error.to_string().contains("5000ms"));
        assert_eq!(error.category(), ErrorCategory::Performance);
        assert!(error.is_recoverable());
    }

    #[test]
    fn test_parallel_error() {
        let error = EngineError::parallel("Thread pool exhausted");

        match &error {
            EngineError::ParallelError { message, source } => {
                assert_eq!(message, "Thread pool exhausted");
                assert!(source.is_none());
            }
            _ => panic!("Expected ParallelError"),
        }

        assert!(error.to_string().contains("Parallel processing error"));
        assert_eq!(error.category(), ErrorCategory::Concurrency);
        assert!(error.is_recoverable());
    }

    #[test]
    fn test_data_corruption_error() {
        let error = EngineError::data_corruption("Checksum mismatch in index file");

        match &error {
            EngineError::DataCorruption { details } => {
                assert_eq!(details, "Checksum mismatch in index file");
            }
            _ => panic!("Expected DataCorruption"),
        }

        assert!(error.to_string().contains("Data corruption detected"));
        assert_eq!(error.category(), ErrorCategory::DataIntegrity);
    }

    #[test]
    fn test_version_mismatch_error() {
        let error = EngineError::version_mismatch("v2.0", "v1.5");

        match &error {
            EngineError::VersionMismatch { expected, found } => {
                assert_eq!(expected, "v2.0");
                assert_eq!(found, "v1.5");
            }
            _ => panic!("Expected VersionMismatch"),
        }

        assert!(error.to_string().contains("Version mismatch"));
        assert!(error.to_string().contains("expected v2.0"));
        assert!(error.to_string().contains("found v1.5"));
        assert_eq!(error.category(), ErrorCategory::Compatibility);
    }

    #[test]
    fn test_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let error = EngineError::io("file write", io_err);

        match &error {
            EngineError::IoError {
                operation,
                source: _,
            } => {
                assert_eq!(operation, "file write");
            }
            _ => panic!("Expected IoError"),
        }

        assert!(error.to_string().contains("IO error"));
        assert_eq!(error.category(), ErrorCategory::IO);
        assert!(error.is_recoverable());
    }

    #[test]
    fn test_internal_error() {
        let error = EngineError::internal("Unexpected null pointer");

        match &error {
            EngineError::Internal { message, source } => {
                assert_eq!(message, "Unexpected null pointer");
                assert!(source.is_none());
            }
            _ => panic!("Expected Internal"),
        }

        assert!(error.to_string().contains("Internal engine error"));
        assert_eq!(error.category(), ErrorCategory::Internal);
    }

    // ErrorCategory Tests

    #[test]
    fn test_error_category_display() {
        assert_eq!(ErrorCategory::DataLoad.to_string(), "data_load");
        assert_eq!(ErrorCategory::Analysis.to_string(), "analysis");
        assert_eq!(ErrorCategory::Cache.to_string(), "cache");
        assert_eq!(ErrorCategory::Configuration.to_string(), "configuration");
        assert_eq!(ErrorCategory::Resource.to_string(), "resource");
        assert_eq!(ErrorCategory::Input.to_string(), "input");
        assert_eq!(ErrorCategory::Initialization.to_string(), "initialization");
        assert_eq!(ErrorCategory::Performance.to_string(), "performance");
        assert_eq!(ErrorCategory::Concurrency.to_string(), "concurrency");
        assert_eq!(ErrorCategory::DataIntegrity.to_string(), "data_integrity");
        assert_eq!(ErrorCategory::Compatibility.to_string(), "compatibility");
        assert_eq!(ErrorCategory::IO.to_string(), "io");
        assert_eq!(ErrorCategory::Serialization.to_string(), "serialization");
        assert_eq!(ErrorCategory::Internal.to_string(), "internal");
    }

    #[test]
    fn test_error_category_equality() {
        assert_eq!(ErrorCategory::Cache, ErrorCategory::Cache);
        assert_ne!(ErrorCategory::Cache, ErrorCategory::Analysis);

        let categories = [
            ErrorCategory::DataLoad,
            ErrorCategory::Analysis,
            ErrorCategory::Cache,
            ErrorCategory::Configuration,
            ErrorCategory::Resource,
            ErrorCategory::Input,
            ErrorCategory::Initialization,
            ErrorCategory::Performance,
            ErrorCategory::Concurrency,
            ErrorCategory::DataIntegrity,
            ErrorCategory::Compatibility,
            ErrorCategory::IO,
            ErrorCategory::Serialization,
            ErrorCategory::Internal,
        ];

        // Each category should be equal to itself and different from others
        for (i, cat1) in categories.iter().enumerate() {
            for (j, cat2) in categories.iter().enumerate() {
                if i == j {
                    assert_eq!(cat1, cat2);
                } else {
                    assert_ne!(cat1, cat2);
                }
            }
        }
    }

    #[test]
    fn test_error_category_hash() {
        let mut map = HashMap::new();
        map.insert(ErrorCategory::Cache, "cache_errors");
        map.insert(ErrorCategory::Analysis, "analysis_errors");

        assert_eq!(map.get(&ErrorCategory::Cache), Some(&"cache_errors"));
        assert_eq!(map.get(&ErrorCategory::Analysis), Some(&"analysis_errors"));
        assert_eq!(map.get(&ErrorCategory::Internal), None);
    }

    // ErrorStats Tests

    #[test]
    fn test_error_stats_default() {
        let stats = ErrorStats::default();

        assert_eq!(stats.total_errors, 0);
        assert!(stats.error_counts.is_empty());
        assert!(stats.recent_error_descriptions.is_empty());
        assert_eq!(stats.error_rate(ErrorCategory::Cache), 0.0);
        assert_eq!(stats.most_common_error(), None);
        assert!(!stats.has_concerning_error_rate());
    }

    #[test]
    fn test_error_stats_record_single_error() {
        let mut stats = ErrorStats::default();
        let error = EngineError::cache("connection failed");

        stats.record_error(error);

        assert_eq!(stats.total_errors, 1);
        assert_eq!(stats.error_counts.len(), 1);
        assert_eq!(stats.error_counts[&ErrorCategory::Cache], 1);
        assert_eq!(stats.recent_error_descriptions.len(), 1);
        assert!(stats.recent_error_descriptions[0].contains("Cache operation failed"));
        assert_eq!(stats.error_rate(ErrorCategory::Cache), 1.0);
        assert_eq!(stats.most_common_error(), Some(ErrorCategory::Cache));
    }

    #[test]
    fn test_error_stats_record_multiple_errors() {
        let mut stats = ErrorStats::default();

        // Record different types of errors
        stats.record_error(EngineError::cache("cache miss"));
        stats.record_error(EngineError::cache("cache timeout"));
        stats.record_error(EngineError::analysis("input", "failed"));
        stats.record_error(EngineError::timeout("query", 1000));

        assert_eq!(stats.total_errors, 4);
        assert_eq!(stats.error_counts.len(), 3);
        assert_eq!(stats.error_counts[&ErrorCategory::Cache], 2);
        assert_eq!(stats.error_counts[&ErrorCategory::Analysis], 1);
        assert_eq!(stats.error_counts[&ErrorCategory::Performance], 1);

        // Cache should be most common (2/4 = 0.5)
        assert_eq!(stats.error_rate(ErrorCategory::Cache), 0.5);
        assert_eq!(stats.error_rate(ErrorCategory::Analysis), 0.25);
        assert_eq!(stats.error_rate(ErrorCategory::Performance), 0.25);
        assert_eq!(stats.error_rate(ErrorCategory::Configuration), 0.0);

        assert_eq!(stats.most_common_error(), Some(ErrorCategory::Cache));
        assert_eq!(stats.recent_error_descriptions.len(), 4);
    }

    #[test]
    fn test_error_stats_recent_descriptions_limit() {
        let mut stats = ErrorStats::default();

        // Record 120 errors (more than the 100 limit)
        for i in 0..120 {
            stats.record_error(EngineError::cache(format!("error {}", i)));
        }

        assert_eq!(stats.total_errors, 120);
        assert_eq!(stats.recent_error_descriptions.len(), 100);

        // Should contain the last 100 errors
        assert!(stats.recent_error_descriptions[0].contains("error 20"));
        assert!(stats.recent_error_descriptions[99].contains("error 119"));
    }

    #[test]
    fn test_error_stats_most_common_error() {
        let mut stats = ErrorStats::default();

        // Record many analysis errors and few cache errors
        for _ in 0..10 {
            stats.record_error(EngineError::analysis("input", "reason"));
        }
        for _ in 0..3 {
            stats.record_error(EngineError::cache("operation"));
        }

        assert_eq!(stats.most_common_error(), Some(ErrorCategory::Analysis));
        assert_eq!(stats.error_rate(ErrorCategory::Analysis), 10.0 / 13.0);
        assert_eq!(stats.error_rate(ErrorCategory::Cache), 3.0 / 13.0);
    }

    #[test]
    fn test_error_stats_concerning_rate() {
        let mut stats = ErrorStats::default();

        // Record 30 errors (should not be concerning)
        for i in 0..30 {
            stats.record_error(EngineError::cache(format!("error {}", i)));
        }
        assert!(!stats.has_concerning_error_rate());

        // Record 51 errors (should be concerning)
        for i in 30..51 {
            stats.record_error(EngineError::cache(format!("error {}", i)));
        }
        assert!(stats.has_concerning_error_rate());
    }

    // Error Conversion Tests

    #[test]
    fn test_io_error_conversion() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let engine_error: EngineError = io_error.into();

        match &engine_error {
            EngineError::IoError {
                operation,
                source: _,
            } => {
                assert_eq!(operation, "unknown");
            }
            _ => panic!("Expected IoError"),
        }

        assert_eq!(engine_error.category(), ErrorCategory::IO);
        assert!(engine_error.is_recoverable());
    }

    #[test]
    fn test_serde_json_error_conversion() {
        // Create a JSON parsing error
        let json_error = serde_json::from_str::<serde_json::Value>("invalid json");
        let engine_error: EngineError = json_error.unwrap_err().into();

        match &engine_error {
            EngineError::SerializationError { context, source } => {
                assert_eq!(context, "JSON serialization");
                assert!(source.is_some());
            }
            _ => panic!("Expected SerializationError"),
        }

        assert_eq!(engine_error.category(), ErrorCategory::Serialization);
    }

    // Recoverability Tests

    #[test]
    fn test_recoverable_errors() {
        let recoverable_errors = vec![
            EngineError::timeout("operation", 1000),
            EngineError::cache("miss"),
            EngineError::parallel("thread error"),
            EngineError::io(
                "write",
                std::io::Error::new(std::io::ErrorKind::Interrupted, "interrupted"),
            ),
        ];

        for error in recoverable_errors {
            assert!(
                error.is_recoverable(),
                "Error should be recoverable: {:?}",
                error
            );
        }
    }

    #[test]
    fn test_non_recoverable_errors() {
        let non_recoverable_errors = vec![
            EngineError::data_load("load failed"),
            EngineError::analysis("input", "reason"),
            EngineError::config("invalid config"),
            EngineError::resource_not_found("type", "id"),
            EngineError::invalid_input("expected", "actual"),
            EngineError::not_initialized("engine"),
            EngineError::data_corruption("corruption"),
            EngineError::version_mismatch("v1", "v2"),
            EngineError::internal("internal error"),
        ];

        for error in non_recoverable_errors {
            assert!(
                !error.is_recoverable(),
                "Error should not be recoverable: {:?}",
                error
            );
        }
    }

    // Retry with Different Input Tests

    #[test]
    fn test_should_retry_with_different_input() {
        let retry_errors = vec![
            EngineError::invalid_input("UTF-8", "binary"),
            EngineError::analysis("input", "parsing failed"),
        ];

        for error in retry_errors {
            assert!(
                error.should_retry_with_different_input(),
                "Error should suggest retry: {:?}",
                error
            );
        }
    }

    #[test]
    fn test_should_not_retry_with_different_input() {
        let no_retry_errors = vec![
            EngineError::cache("miss"),
            EngineError::config("invalid"),
            EngineError::timeout("op", 1000),
            EngineError::internal("internal"),
        ];

        for error in no_retry_errors {
            assert!(
                !error.should_retry_with_different_input(),
                "Error should not suggest retry: {:?}",
                error
            );
        }
    }

    // Complex Scenario Tests

    #[test]
    fn test_error_chain_analysis() {
        let mut stats = ErrorStats::default();

        // Simulate a system under stress with various error patterns
        let error_sequence = vec![
            EngineError::cache("connection timeout"), // Infrastructure issue
            EngineError::cache("connection timeout"), // Repeated
            EngineError::analysis("", "empty input"), // Input validation
            EngineError::timeout("query", 5000),      // Performance issue
            EngineError::cache("redis down"),         // Infrastructure
            EngineError::analysis("invalid chars", "encoding error"), // Input issue
            EngineError::parallel("thread panic"),    // Concurrency issue
            EngineError::resource_not_found("memory", "limit"), // Resource issue
        ];

        for error in error_sequence {
            stats.record_error(error);
        }

        // Analysis
        assert_eq!(stats.total_errors, 8);
        // Now we have: Cache (3), Analysis (2), Performance (1), Concurrency (1), Resource (1)
        assert_eq!(stats.most_common_error(), Some(ErrorCategory::Cache)); // 3 cache errors
        assert_eq!(stats.error_rate(ErrorCategory::Cache), 3.0 / 8.0);
        assert_eq!(stats.error_rate(ErrorCategory::Analysis), 2.0 / 8.0);
        assert_eq!(stats.error_rate(ErrorCategory::Performance), 1.0 / 8.0);
        assert_eq!(stats.error_rate(ErrorCategory::Concurrency), 1.0 / 8.0);
        assert_eq!(stats.error_rate(ErrorCategory::Resource), 1.0 / 8.0);

        // All cache and timeout errors should be recoverable
        let cache_error = EngineError::cache("test");
        let timeout_error = EngineError::timeout("test", 1000);
        let analysis_error = EngineError::analysis("test", "reason");

        assert!(cache_error.is_recoverable());
        assert!(timeout_error.is_recoverable());
        assert!(!analysis_error.is_recoverable());
    }

    #[test]
    fn test_error_message_format_consistency() {
        let errors = vec![
            EngineError::data_load("test context"),
            EngineError::analysis("test input", "test reason"),
            EngineError::cache("test operation"),
            EngineError::config("test message"),
            EngineError::resource_not_found("test type", "test id"),
            EngineError::invalid_input("expected", "actual"),
            EngineError::not_initialized("test engine"),
            EngineError::timeout("test op", 1000),
            EngineError::parallel("test message"),
            EngineError::data_corruption("test details"),
            EngineError::version_mismatch("v1", "v2"),
            EngineError::internal("test message"),
        ];

        for error in errors {
            let message = error.to_string();
            assert!(!message.is_empty(), "Error message should not be empty");
            assert!(
                !message.contains("{}"),
                "Error message should not contain unfilled placeholders"
            );
            println!("Error message: {}", message); // For manual verification
        }
    }
}
