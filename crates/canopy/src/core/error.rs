//! Unified error types for Canopy.

use super::ThetaRole;
use thiserror::Error;

/// Core error types for Canopy analysis.
///
/// This is the unified error type used across the kernel.
#[derive(Error, Debug)]
pub enum CanopyError {
    // === Data Loading & Resources ===
    #[error("Data loading failed: {context}")]
    DataLoad {
        context: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Resource not found: {resource_type} '{identifier}'")]
    ResourceNotFound {
        resource_type: String,
        identifier: String,
    },

    // === Analysis & Processing ===
    #[error("Analysis failed for '{input}': {reason}")]
    Analysis {
        input: String,
        reason: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Parsing failed: {context}")]
    Parse { context: String },

    #[error("Invalid input: expected {expected}, got {actual}")]
    InvalidInput { expected: String, actual: String },

    // === Configuration ===
    #[error("Configuration error: {message}")]
    Config { message: String },

    #[error("Not initialized: {component}")]
    NotInitialized { component: String },

    // === Event Composition ===
    #[error("No predicate found in sentence")]
    NoPredicateFound,

    #[error("Event decomposition failed for '{predicate}': {reason}")]
    DecompositionFailed { predicate: String, reason: String },

    #[error("Missing required role {role} for predicate '{predicate}'")]
    MissingRole { role: ThetaRole, predicate: String },

    // === Discourse ===
    #[error("DRS construction failed: {0}")]
    DrsConstruction(String),

    #[error("Referent not found: {0}")]
    ReferentNotFound(String),

    #[error("Anaphora resolution failed for '{pronoun}': {reason}")]
    AnaphoraResolutionFailed { pronoun: String, reason: String },

    // === IO ===
    #[error("IO error: {operation}")]
    Io {
        operation: String,
        #[source]
        source: std::io::Error,
    },

    // === Mapping ===
    #[error("Unknown role: {0}")]
    UnknownRole(String),

    // === Internal ===
    #[error("Internal error: {message}")]
    Internal { message: String },
}

impl CanopyError {
    pub fn data_load<S: Into<String>>(context: S) -> Self {
        Self::DataLoad {
            context: context.into(),
            source: None,
        }
    }

    pub fn analysis<S: Into<String>, R: Into<String>>(input: S, reason: R) -> Self {
        Self::Analysis {
            input: input.into(),
            reason: reason.into(),
            source: None,
        }
    }

    pub fn config<S: Into<String>>(message: S) -> Self {
        Self::Config {
            message: message.into(),
        }
    }

    pub fn not_initialized<S: Into<String>>(component: S) -> Self {
        Self::NotInitialized {
            component: component.into(),
        }
    }

    pub fn parse<S: Into<String>>(context: S) -> Self {
        Self::Parse {
            context: context.into(),
        }
    }

    pub fn internal<S: Into<String>>(message: S) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }
}

impl From<std::io::Error> for CanopyError {
    fn from(error: std::io::Error) -> Self {
        Self::Io {
            operation: "unknown".to_string(),
            source: error,
        }
    }
}

/// Result type alias for Canopy operations.
pub type CanopyResult<T> = Result<T, CanopyError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = CanopyError::analysis("test input", "something went wrong");
        assert!(err.to_string().contains("test input"));

        let err = CanopyError::config("bad config");
        assert!(err.to_string().contains("bad config"));
    }

    #[test]
    fn test_error_constructors() {
        let _ = CanopyError::data_load("loading VerbNet");
        let _ = CanopyError::not_initialized("parser");
        let _ = CanopyError::parse("unexpected token");
        let _ = CanopyError::internal("invariant violated");
    }
}
