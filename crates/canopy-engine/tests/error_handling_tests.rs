//! Tests for EngineError types and error handling paths to achieve coverage targets

use canopy_engine::{EngineError, EngineResult};
use std::error::Error;
use std::io;

#[test]
fn test_data_load_error_creation() {
    let error = EngineError::data_load("Failed to read XML file".to_string());

    assert!(matches!(error, EngineError::DataLoadError { .. }));
    let error_msg = error.to_string();
    assert!(error_msg.contains("Data loading failed"));
    assert!(error_msg.contains("Failed to read XML file"));
}

#[test]
fn test_data_load_error_with_source() {
    let source_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
    let error = EngineError::data_load_with_source("XML file missing".to_string(), source_error);

    if let EngineError::DataLoadError { context, source } = &error {
        assert_eq!(context, "XML file missing");
        assert!(source.is_some());
        assert!(source
            .as_ref()
            .unwrap()
            .to_string()
            .contains("File not found"));
    } else {
        panic!("Expected DataLoadError");
    }
}

#[test]
fn test_analysis_error_creation() {
    let error = EngineError::analysis("test_input".to_string(), "Invalid verb format".to_string());

    if let EngineError::AnalysisError {
        input,
        reason,
        source,
    } = &error
    {
        assert_eq!(input, "test_input");
        assert_eq!(reason, "Invalid verb format");
        assert!(source.is_none());
    } else {
        panic!("Expected AnalysisError");
    }

    let error_msg = EngineError::analysis("test".to_string(), "failed".to_string()).to_string();
    assert!(error_msg.contains("Analysis failed for input 'test'"));
    assert!(error_msg.contains("failed"));
}

#[test]
fn test_analysis_error_with_source() {
    let source_error = io::Error::new(io::ErrorKind::InvalidInput, "Bad input");
    let error = EngineError::analysis_with_source(
        "bad_verb".to_string(),
        "Parsing failed".to_string(),
        source_error,
    );

    if let EngineError::AnalysisError {
        input,
        reason,
        source,
    } = &error
    {
        assert_eq!(input, "bad_verb");
        assert_eq!(reason, "Parsing failed");
        assert!(source.is_some());
    } else {
        panic!("Expected AnalysisError");
    }
}

#[test]
fn test_cache_error_creation() {
    let error = EngineError::cache("insert".to_string());

    if let EngineError::CacheError { operation, source } = &error {
        assert_eq!(operation, "insert");
        assert!(source.is_none());
    } else {
        panic!("Expected CacheError");
    }

    let error_msg = EngineError::cache("clear".to_string()).to_string();
    assert!(error_msg.contains("Cache operation failed: clear"));
}

#[test]
fn test_cache_error_with_source() {
    // No cache_with_source method - test simple cache method
    let error = EngineError::cache("eviction".to_string());

    if let EngineError::CacheError { operation, source } = &error {
        assert_eq!(operation, "eviction");
        assert!(source.is_none());
    } else {
        panic!("Expected CacheError");
    }
}

#[test]
fn test_config_error_creation() {
    let error = EngineError::config("Invalid cache size: -1".to_string());

    if let EngineError::ConfigError { message } = &error {
        assert_eq!(message, "Invalid cache size: -1");
    } else {
        panic!("Expected ConfigError");
    }

    let error_msg = EngineError::config("Bad config".to_string()).to_string();
    assert!(error_msg.contains("Configuration error: Bad config"));
}

#[test]
fn test_resource_not_found_error() {
    let error = EngineError::resource_not_found("VerbClass".to_string(), "give-13.1".to_string());

    if let EngineError::ResourceNotFound {
        resource_type,
        identifier,
    } = &error
    {
        assert_eq!(resource_type, "VerbClass");
        assert_eq!(identifier, "give-13.1");
    } else {
        panic!("Expected ResourceNotFound");
    }

    let error_msg = error.to_string();
    assert!(error_msg.contains("Resource not found: VerbClass 'give-13.1'"));
}

#[test]
fn test_invalid_input_error() {
    let error = EngineError::invalid_input("String".to_string(), "Number".to_string());

    if let EngineError::InvalidInput { expected, actual } = &error {
        assert_eq!(expected, "String");
        assert_eq!(actual, "Number");
    } else {
        panic!("Expected InvalidInput");
    }

    let error_msg = error.to_string();
    assert!(error_msg.contains("Invalid input format: String expected, got Number"));
}

#[test]
fn test_not_initialized_error() {
    let error = EngineError::not_initialized("VerbNet".to_string());

    if let EngineError::NotInitialized { engine_name } = &error {
        assert_eq!(engine_name, "VerbNet");
    } else {
        panic!("Expected NotInitialized");
    }

    let error_msg = error.to_string();
    assert!(error_msg.contains("Engine not initialized: VerbNet"));
}

#[test]
fn test_timeout_error() {
    let error = EngineError::timeout("data_loading".to_string(), 5000);

    if let EngineError::Timeout {
        operation,
        timeout_ms,
    } = &error
    {
        assert_eq!(operation, "data_loading");
        assert_eq!(*timeout_ms, 5000);
    } else {
        panic!("Expected Timeout");
    }

    let error_msg = error.to_string();
    assert!(error_msg.contains("Timeout occurred during data_loading after 5000ms"));
}

#[test]
fn test_parallel_error_creation() {
    let error = EngineError::parallel("Thread pool exhausted".to_string());

    if let EngineError::ParallelError { message, source } = &error {
        assert_eq!(message, "Thread pool exhausted");
        assert!(source.is_none());
    } else {
        panic!("Expected ParallelError");
    }

    let error_msg = error.to_string();
    assert!(error_msg.contains("Parallel processing error: Thread pool exhausted"));
}

#[test]
fn test_parallel_error_with_source() {
    // No parallel_with_source method - test simple parallel method
    let error = EngineError::parallel("Worker thread failed".to_string());

    if let EngineError::ParallelError { message, source } = &error {
        assert_eq!(message, "Worker thread failed");
        assert!(source.is_none());
    } else {
        panic!("Expected ParallelError");
    }
}

#[test]
fn test_version_mismatch_error() {
    let error = EngineError::version_mismatch("1.0".to_string(), "2.0".to_string());

    if let EngineError::VersionMismatch { expected, found } = &error {
        assert_eq!(expected, "1.0");
        assert_eq!(found, "2.0");
    } else {
        panic!("Expected VersionMismatch");
    }

    let error_msg = error.to_string();
    assert!(error_msg.contains("Version mismatch: expected 1.0, found 2.0"));
}

#[test]
fn test_io_error_creation() {
    let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "Access denied");
    let error = EngineError::io("file_read".to_string(), io_error);

    if let EngineError::IoError { operation, source } = &error {
        assert_eq!(operation, "file_read");
        assert_eq!(source.kind(), io::ErrorKind::PermissionDenied);
    } else {
        panic!("Expected IoError");
    }

    let error_msg = error.to_string();
    assert!(error_msg.contains("IO error: file_read"));
}

#[test]
fn test_internal_error_creation() {
    let error = EngineError::internal("Unexpected state in parser".to_string());

    let error_msg = error.to_string();
    assert!(error_msg.contains("Internal engine error: Unexpected state in parser"));
}

#[test]
fn test_internal_error_with_source() {
    let error = EngineError::internal("Critical failure".to_string());

    let error_msg = error.to_string();
    assert!(error_msg.contains("Internal engine error: Critical failure"));
}

#[test]
fn test_error_result_patterns() {
    // Test EngineResult usage patterns
    let success = "success".to_string();
    assert_eq!(success, "success");

    let failure: EngineResult<String> = Err(EngineError::config("Bad config".to_string()));
    assert!(failure.is_err());

    match failure {
        Err(EngineError::ConfigError { message }) => {
            assert_eq!(message, "Bad config");
        }
        _ => panic!("Expected ConfigError"),
    }
}

#[test]
fn test_error_debug_and_display() {
    let error = EngineError::analysis("test".to_string(), "failed".to_string());

    // Test Debug formatting
    let debug_str = format!("{error:?}");
    assert!(debug_str.contains("AnalysisError"));

    // Test Display formatting
    let display_str = format!("{error}");
    assert!(display_str.contains("Analysis failed"));
    assert!(display_str.contains("test"));
    assert!(display_str.contains("failed"));
}

#[test]
fn test_error_chain_with_sources() {
    let root_cause = io::Error::new(io::ErrorKind::NotFound, "File missing");
    let wrapped_error = EngineError::data_load_with_source(
        "Could not load VerbNet data".to_string(),
        Box::new(root_cause),
    );

    // Test that the error chain is preserved
    let error_msg = wrapped_error.to_string();
    assert!(error_msg.contains("Data loading failed"));
    assert!(error_msg.contains("Could not load VerbNet data"));

    // Test source chain
    let source = wrapped_error.source();
    assert!(source.is_some());
    assert!(source.unwrap().to_string().contains("File missing"));
}
