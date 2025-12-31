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

/// Convert DiscourseError to the unified CanopyError type
impl From<DiscourseError> for canopy_core::CanopyError {
    fn from(error: DiscourseError) -> Self {
        match error {
            DiscourseError::DrsConstructionError(msg) => Self::DrsConstruction(msg),
            DiscourseError::ReferentNotFound(msg) => Self::ReferentNotFound(msg),
            DiscourseError::AnaphoraResolutionFailed { pronoun, reason } => {
                Self::AnaphoraResolutionFailed { pronoun, reason }
            }
            DiscourseError::ContextCapacityExceeded { max, current } => {
                Self::ContextCapacityExceeded { max, current }
            }
            DiscourseError::InvalidOperation(msg) => Self::internal(msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drs_construction_error_display() {
        let err = DiscourseError::DrsConstructionError("missing referent".into());
        assert_eq!(err.to_string(), "DRS construction failed: missing referent");
    }

    #[test]
    fn test_referent_not_found_display() {
        let err = DiscourseError::ReferentNotFound("x1".into());
        assert_eq!(err.to_string(), "referent not found: x1");
    }

    #[test]
    fn test_anaphora_resolution_failed_display() {
        let err = DiscourseError::AnaphoraResolutionFailed {
            pronoun: "he".into(),
            reason: "no antecedent".into(),
        };
        assert_eq!(
            err.to_string(),
            "anaphora resolution failed for 'he': no antecedent"
        );
    }

    #[test]
    fn test_context_capacity_exceeded_display() {
        let err = DiscourseError::ContextCapacityExceeded {
            max: 100,
            current: 150,
        };
        assert_eq!(
            err.to_string(),
            "discourse context capacity exceeded (max: 100, current: 150)"
        );
    }

    #[test]
    fn test_invalid_operation_display() {
        let err = DiscourseError::InvalidOperation("cannot merge".into());
        assert_eq!(err.to_string(), "invalid DRS operation: cannot merge");
    }

    #[test]
    fn test_conversion_to_canopy_error() {
        use canopy_core::CanopyError;

        // Test DrsConstructionError conversion
        let err: CanopyError = DiscourseError::DrsConstructionError("test".into()).into();
        assert!(matches!(err, CanopyError::DrsConstruction(_)));

        // Test ReferentNotFound conversion
        let err: CanopyError = DiscourseError::ReferentNotFound("x".into()).into();
        assert!(matches!(err, CanopyError::ReferentNotFound(_)));

        // Test AnaphoraResolutionFailed conversion
        let err: CanopyError = DiscourseError::AnaphoraResolutionFailed {
            pronoun: "it".into(),
            reason: "ambiguous".into(),
        }
        .into();
        assert!(matches!(err, CanopyError::AnaphoraResolutionFailed { .. }));

        // Test ContextCapacityExceeded conversion
        let err: CanopyError = DiscourseError::ContextCapacityExceeded {
            max: 50,
            current: 60,
        }
        .into();
        assert!(matches!(err, CanopyError::ContextCapacityExceeded { .. }));

        // Test InvalidOperation conversion (maps to internal error)
        let err: CanopyError = DiscourseError::InvalidOperation("bad op".into()).into();
        assert!(matches!(err, CanopyError::Internal { .. }));
    }

    #[test]
    fn test_discourse_result_type() {
        fn returns_ok() -> DiscourseResult<i32> {
            Ok(42)
        }

        fn returns_err() -> DiscourseResult<i32> {
            Err(DiscourseError::InvalidOperation("test".into()))
        }

        assert!(returns_ok().is_ok());
        assert!(returns_err().is_err());
    }
}
