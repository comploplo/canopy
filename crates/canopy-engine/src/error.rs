//! Error handling for semantic engines
//!
//! This module provides unified error types and handling patterns
//! for all semantic engines.

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

    /// Create an analysis error with source
    pub fn analysis_with_source<
        S: Into<String>,
        R: Into<String>,
        E: std::error::Error + Send + Sync + 'static,
    >(
        input: S,
        reason: R,
        source: E,
    ) -> Self {
        Self::AnalysisError {
            input: input.into(),
            reason: reason.into(),
            source: Some(Box::new(source)),
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
    pub fn is_recoverable(&self) -> bool {
        match self {
            Self::Timeout { .. } => true,
            Self::CacheError { .. } => true,
            Self::ParallelError { .. } => true,
            Self::IoError { .. } => true, // Might be temporary
            _ => false,
        }
    }

    /// Check if this error suggests retrying with different input
    pub fn should_retry_with_different_input(&self) -> bool {
        matches!(self, Self::InvalidInput { .. } | Self::AnalysisError { .. })
    }

    /// Get error category for metrics
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

/// Error categories for metrics and analysis
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

/// Error statistics for monitoring
#[derive(Debug, Clone, Default)]
pub struct ErrorStats {
    /// Error counts by category
    pub error_counts: std::collections::HashMap<ErrorCategory, u64>,
    /// Total error count
    pub total_errors: u64,
    /// Recent error descriptions (last 100)
    pub recent_error_descriptions: Vec<String>,
}

impl ErrorStats {
    /// Record a new error
    pub fn record_error(&mut self, error: EngineError) {
        let category = error.category();
        *self.error_counts.entry(category).or_insert(0) += 1;
        self.total_errors += 1;

        // Keep only last 100 error descriptions
        self.recent_error_descriptions.push(error.to_string());
        if self.recent_error_descriptions.len() > 100 {
            self.recent_error_descriptions.remove(0);
        }
    }

    /// Get error rate for a specific category
    pub fn error_rate(&self, category: ErrorCategory) -> f64 {
        if self.total_errors == 0 {
            0.0
        } else {
            *self.error_counts.get(&category).unwrap_or(&0) as f64 / self.total_errors as f64
        }
    }

    /// Get most common error category
    pub fn most_common_error(&self) -> Option<ErrorCategory> {
        self.error_counts
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(category, _)| *category)
    }

    /// Check if error rate is concerning (>5% total errors)
    pub fn has_concerning_error_rate(&self) -> bool {
        // This would need total request count to calculate properly
        // For now, just check if we have many recent errors
        self.recent_error_descriptions.len() > 50
    }
}

/// Convert from IO errors
impl From<std::io::Error> for EngineError {
    fn from(error: std::io::Error) -> Self {
        Self::IoError {
            operation: "unknown".to_string(),
            source: error,
        }
    }
}

/// Convert from serde JSON errors
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
    fn test_error_stats() {
        let mut stats = ErrorStats::default();

        stats.record_error(EngineError::cache("test"));
        stats.record_error(EngineError::analysis("input", "reason"));

        assert_eq!(stats.total_errors, 2);
        assert_eq!(stats.error_rate(ErrorCategory::Cache), 0.5);
        assert_eq!(stats.recent_error_descriptions.len(), 2);
    }
}
