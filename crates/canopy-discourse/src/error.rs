//! Error types for discourse processing

use thiserror::Error;

/// Errors that can occur during discourse processing
#[derive(Error, Debug)]
pub enum DiscourseError {
    /// Failed to build DRS from events
    #[error("DRS construction failed: {0}")]
    DrsConstructionError(String),

    /// Referent not found in context
    #[error("referent not found: {0}")]
    ReferentNotFound(String),

    /// Anaphora resolution failed
    #[error("anaphora resolution failed for '{pronoun}': {reason}")]
    AnaphoraResolutionFailed { pronoun: String, reason: String },

    /// Context capacity exceeded
    #[error("discourse context capacity exceeded (max: {max}, current: {current})")]
    ContextCapacityExceeded { max: usize, current: usize },

    /// Invalid DRS operation
    #[error("invalid DRS operation: {0}")]
    InvalidOperation(String),
}

/// Result type for discourse operations
pub type DiscourseResult<T> = Result<T, DiscourseError>;
