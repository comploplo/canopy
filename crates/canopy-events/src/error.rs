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

/// Convert EventError to the unified CanopyError type
impl From<EventError> for canopy_core::CanopyError {
    fn from(error: EventError) -> Self {
        match error {
            EventError::NoPredicateFound => Self::NoPredicateFound,
            EventError::DecompositionFailed { predicate, reason } => {
                Self::DecompositionFailed { predicate, reason }
            }
            EventError::BindingFailed { token, reason } => Self::BindingFailed { token, reason },
            EventError::MissingRole { role, predicate } => Self::MissingRole { role, predicate },
            EventError::NoVerbNetData => Self::resource_not_found("VerbNet", "predicate analysis"),
            EventError::ConfigError(msg) => Self::config(msg),
            EventError::Internal(msg) => Self::internal(msg),
        }
    }
}
