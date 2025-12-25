//! Error types for event composition

use thiserror::Error;

/// Errors that can occur during event composition
#[derive(Error, Debug)]
pub enum EventError {
    /// No predicate found in sentence
    #[error("no predicate found in sentence")]
    NoPredicateFound,

    /// Failed to decompose predicate into LittleV
    #[error("decomposition failed for predicate '{predicate}': {reason}")]
    DecompositionFailed { predicate: String, reason: String },

    /// Failed to bind participant to theta role
    #[error("binding failed for token '{token}': {reason}")]
    BindingFailed { token: String, reason: String },

    /// Missing required theta role
    #[error("missing required role {role:?} for predicate '{predicate}'")]
    MissingRole {
        role: canopy_core::ThetaRole,
        predicate: String,
    },

    /// VerbNet data not available
    #[error("VerbNet analysis not available for predicate")]
    NoVerbNetData,

    /// Configuration error
    #[error("configuration error: {0}")]
    ConfigError(String),

    /// Internal error
    #[error("internal error: {0}")]
    Internal(String),
}

/// Result type for event composition operations
pub type EventResult<T> = Result<T, EventError>;
