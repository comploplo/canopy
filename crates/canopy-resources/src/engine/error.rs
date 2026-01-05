//! Error handling for semantic engines

use std::fmt;
use thiserror::Error;

/// Common result type for all engine operations
pub type EngineResult<T> = Result<T, EngineError>;

/// Unified error type for all semantic engines
#[derive(Error, Debug)]
pub enum EngineError {
    #[error("Data loading failed: {context}")]
    DataLoadError {
        context: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Analysis failed for input '{input}': {reason}")]
    AnalysisError {
        input: String,
        reason: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Cache operation failed: {operation}")]
    CacheError {
        operation: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Configuration error: {message}")]
    ConfigError { message: String },

    #[error("Resource not found: {resource_type} '{identifier}'")]
    ResourceNotFound {
        resource_type: String,
        identifier: String,
    },

    #[error("Invalid input format: {expected} expected, got {actual}")]
    InvalidInput { expected: String, actual: String },

    #[error("Engine not initialized: {engine_name}")]
    NotInitialized { engine_name: String },

    #[error("Timeout occurred during {operation} after {timeout_ms}ms")]
    Timeout { operation: String, timeout_ms: u64 },

    #[error("Parallel processing error: {message}")]
    ParallelError {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Data corruption detected: {details}")]
    DataCorruption { details: String },

    #[error("Version mismatch: expected {expected}, found {found}")]
    VersionMismatch { expected: String, found: String },

    #[error("IO error: {operation}")]
    IoError {
        operation: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Serialization error: {context}")]
    SerializationError {
        context: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Internal engine error: {message}")]
    Internal {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

impl EngineError {
    /// Create a data loading error
    pub fn data_load<S: Into<String>>(context: S) -> Self {
        Self::DataLoadError {
            context: context.into(),
            source: None,
        }
    }

    /// Create a data loading error with source
    pub fn data_load_with_source<S: Into<String>, E: std::error::Error + Send + Sync + 'static>(
        context: S,
        source: E,
    ) -> Self {
        Self::DataLoadError {
            context: context.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Create an analysis error
    pub fn analysis<S: Into<String>, R: Into<String>>(input: S, reason: R) -> Self {
        Self::AnalysisError {
            input: input.into(),
            reason: reason.into(),
            source: None,
        }
    }

    /// Create a cache error
    pub fn cache<S: Into<String>>(operation: S) -> Self {
        Self::CacheError {
            operation: operation.into(),
            source: None,
        }
    }

    /// Create a configuration error
    pub fn config<S: Into<String>>(message: S) -> Self {
        Self::ConfigError {
            message: message.into(),
        }
    }

    /// Create a resource not found error
    pub fn resource_not_found<T: Into<String>, I: Into<String>>(
        resource_type: T,
        identifier: I,
    ) -> Self {
        Self::ResourceNotFound {
            resource_type: resource_type.into(),
            identifier: identifier.into(),
        }
    }

    /// Create an invalid input error
    pub fn invalid_input<E: Into<String>, A: Into<String>>(expected: E, actual: A) -> Self {
        Self::InvalidInput {
            expected: expected.into(),
            actual: actual.into(),
        }
    }

    /// Create a not initialized error
    pub fn not_initialized<S: Into<String>>(engine_name: S) -> Self {
        Self::NotInitialized {
            engine_name: engine_name.into(),
        }
    }

    /// Create a timeout error
    pub fn timeout<S: Into<String>>(operation: S, timeout_ms: u64) -> Self {
        Self::Timeout {
            operation: operation.into(),
            timeout_ms,
        }
    }

    /// Create a parallel processing error
    pub fn parallel<S: Into<String>>(message: S) -> Self {
        Self::ParallelError {
            message: message.into(),
            source: None,
        }
    }

    /// Create a data corruption error
    pub fn data_corruption<S: Into<String>>(details: S) -> Self {
        Self::DataCorruption {
            details: details.into(),
        }
    }

    /// Create a version mismatch error
    pub fn version_mismatch<E: Into<String>, F: Into<String>>(expected: E, found: F) -> Self {
        Self::VersionMismatch {
            expected: expected.into(),
            found: found.into(),
        }
    }

    /// Create an IO error
    pub fn io<S: Into<String>>(operation: S, source: std::io::Error) -> Self {
        Self::IoError {
            operation: operation.into(),
            source,
        }
    }

    /// Create an internal error
    pub fn internal<S: Into<String>>(message: S) -> Self {
        Self::Internal {
            message: message.into(),
            source: None,
        }
    }

    /// Check if this is a recoverable error
    #[must_use]
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::Timeout { .. }
                | Self::CacheError { .. }
                | Self::ParallelError { .. }
                | Self::IoError { .. }
        )
    }

    /// Get error category for metrics
    #[must_use]
    pub fn category(&self) -> ErrorCategory {
        match self {
            Self::DataLoadError { .. } => ErrorCategory::DataLoad,
            Self::AnalysisError { .. } => ErrorCategory::Analysis,
            Self::CacheError { .. } => ErrorCategory::Cache,
            Self::ConfigError { .. } => ErrorCategory::Configuration,
            Self::ResourceNotFound { .. } => ErrorCategory::Resource,
            Self::InvalidInput { .. } => ErrorCategory::Input,
            Self::NotInitialized { .. } => ErrorCategory::Initialization,
            Self::Timeout { .. } => ErrorCategory::Performance,
            Self::ParallelError { .. } => ErrorCategory::Concurrency,
            Self::DataCorruption { .. } => ErrorCategory::DataIntegrity,
            Self::VersionMismatch { .. } => ErrorCategory::Compatibility,
            Self::IoError { .. } => ErrorCategory::IO,
            Self::SerializationError { .. } => ErrorCategory::Serialization,
            Self::Internal { .. } => ErrorCategory::Internal,
        }
    }
}

/// Error categories for metrics
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCategory {
    DataLoad,
    Analysis,
    Cache,
    Configuration,
    Resource,
    Input,
    Initialization,
    Performance,
    Concurrency,
    DataIntegrity,
    Compatibility,
    IO,
    Serialization,
    Internal,
}

impl fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DataLoad => write!(f, "data_load"),
            Self::Analysis => write!(f, "analysis"),
            Self::Cache => write!(f, "cache"),
            Self::Configuration => write!(f, "configuration"),
            Self::Resource => write!(f, "resource"),
            Self::Input => write!(f, "input"),
            Self::Initialization => write!(f, "initialization"),
            Self::Performance => write!(f, "performance"),
            Self::Concurrency => write!(f, "concurrency"),
            Self::DataIntegrity => write!(f, "data_integrity"),
            Self::Compatibility => write!(f, "compatibility"),
            Self::IO => write!(f, "io"),
            Self::Serialization => write!(f, "serialization"),
            Self::Internal => write!(f, "internal"),
        }
    }
}

impl From<std::io::Error> for EngineError {
    fn from(error: std::io::Error) -> Self {
        Self::IoError {
            operation: "unknown".to_string(),
            source: error,
        }
    }
}

impl From<serde_json::Error> for EngineError {
    fn from(error: serde_json::Error) -> Self {
        Self::SerializationError {
            context: "JSON serialization".to_string(),
            source: Some(Box::new(error)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = EngineError::analysis("test input", "invalid format");
        assert!(error.to_string().contains("test input"));
        assert!(error.to_string().contains("invalid format"));
    }

    #[test]
    fn test_error_categories() {
        let error = EngineError::cache("lookup failed");
        assert_eq!(error.category(), ErrorCategory::Cache);

        let error = EngineError::timeout("query", 5000);
        assert_eq!(error.category(), ErrorCategory::Performance);
    }

    #[test]
    fn test_error_recoverability() {
        let timeout_error = EngineError::timeout("query", 5000);
        assert!(timeout_error.is_recoverable());

        let config_error = EngineError::config("invalid setting");
        assert!(!config_error.is_recoverable());
    }

    #[test]
    fn test_error_constructors() {
        let _ = EngineError::data_load("failed to load");
        let _ = EngineError::resource_not_found("file", "data.xml");
        let _ = EngineError::invalid_input("positive number", "-5");
        let _ = EngineError::not_initialized("TestEngine");
        let _ = EngineError::parallel("thread panic");
        let _ = EngineError::data_corruption("checksum mismatch");
        let _ = EngineError::version_mismatch("1.0", "2.0");
        let _ = EngineError::internal("unexpected state");
        let _ = EngineError::io(
            "read",
            std::io::Error::new(std::io::ErrorKind::NotFound, "not found"),
        );
    }

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
    fn test_error_category_all_variants() {
        // Test that each error type maps to the correct category
        assert_eq!(
            EngineError::data_load("test").category(),
            ErrorCategory::DataLoad
        );
        assert_eq!(
            EngineError::analysis("i", "r").category(),
            ErrorCategory::Analysis
        );
        assert_eq!(EngineError::cache("test").category(), ErrorCategory::Cache);
        assert_eq!(
            EngineError::config("test").category(),
            ErrorCategory::Configuration
        );
        assert_eq!(
            EngineError::resource_not_found("type", "id").category(),
            ErrorCategory::Resource
        );
        assert_eq!(
            EngineError::invalid_input("i", "r").category(),
            ErrorCategory::Input
        );
        assert_eq!(
            EngineError::not_initialized("test").category(),
            ErrorCategory::Initialization
        );
        assert_eq!(
            EngineError::timeout("op", 100).category(),
            ErrorCategory::Performance
        );
        assert_eq!(
            EngineError::parallel("test").category(),
            ErrorCategory::Concurrency
        );
        assert_eq!(
            EngineError::data_corruption("test").category(),
            ErrorCategory::DataIntegrity
        );
        assert_eq!(
            EngineError::version_mismatch("1", "2").category(),
            ErrorCategory::Compatibility
        );
        assert_eq!(
            EngineError::internal("test").category(),
            ErrorCategory::Internal
        );
    }

    #[test]
    fn test_io_error_from() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let engine_error: EngineError = io_error.into();
        assert_eq!(engine_error.category(), ErrorCategory::IO);
        assert!(engine_error.is_recoverable());
    }
}
