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

#[cfg(test)]
mod tests {
    use super::*;
    use canopy_core::ThetaRole;

    #[test]
    fn test_no_predicate_found_display() {
        let err = EventError::NoPredicateFound;
        assert_eq!(err.to_string(), "no predicate found in sentence");
    }

    #[test]
    fn test_decomposition_failed_display() {
        let err = EventError::DecompositionFailed {
            predicate: "run".into(),
            reason: "unknown class".into(),
        };
        assert_eq!(
            err.to_string(),
            "decomposition failed for predicate 'run': unknown class"
        );
    }

    #[test]
    fn test_binding_failed_display() {
        let err = EventError::BindingFailed {
            token: "Mary".into(),
            reason: "no available role".into(),
        };
        assert_eq!(
            err.to_string(),
            "binding failed for token 'Mary': no available role"
        );
    }

    #[test]
    fn test_missing_role_display() {
        let err = EventError::MissingRole {
            role: ThetaRole::Agent,
            predicate: "give".into(),
        };
        assert!(err.to_string().contains("Agent"));
        assert!(err.to_string().contains("give"));
    }

    #[test]
    fn test_no_verbnet_data_display() {
        let err = EventError::NoVerbNetData;
        assert_eq!(
            err.to_string(),
            "VerbNet analysis not available for predicate"
        );
    }

    #[test]
    fn test_config_error_display() {
        let err = EventError::ConfigError("invalid setting".into());
        assert_eq!(err.to_string(), "configuration error: invalid setting");
    }

    #[test]
    fn test_internal_error_display() {
        let err = EventError::Internal("unexpected state".into());
        assert_eq!(err.to_string(), "internal error: unexpected state");
    }

    #[test]
    fn test_conversion_to_canopy_error() {
        use canopy_core::CanopyError;

        // Test NoPredicateFound conversion
        let err: CanopyError = EventError::NoPredicateFound.into();
        assert!(matches!(err, CanopyError::NoPredicateFound));

        // Test DecompositionFailed conversion
        let err: CanopyError = EventError::DecompositionFailed {
            predicate: "test".into(),
            reason: "fail".into(),
        }
        .into();
        assert!(matches!(err, CanopyError::DecompositionFailed { .. }));

        // Test BindingFailed conversion
        let err: CanopyError = EventError::BindingFailed {
            token: "tok".into(),
            reason: "err".into(),
        }
        .into();
        assert!(matches!(err, CanopyError::BindingFailed { .. }));

        // Test MissingRole conversion
        let err: CanopyError = EventError::MissingRole {
            role: ThetaRole::Theme,
            predicate: "give".into(),
        }
        .into();
        assert!(matches!(err, CanopyError::MissingRole { .. }));

        // Test NoVerbNetData conversion
        let err: CanopyError = EventError::NoVerbNetData.into();
        assert!(matches!(err, CanopyError::ResourceNotFound { .. }));

        // Test ConfigError conversion
        let err: CanopyError = EventError::ConfigError("bad".into()).into();
        assert!(matches!(err, CanopyError::Config { .. }));

        // Test Internal conversion
        let err: CanopyError = EventError::Internal("oops".into()).into();
        assert!(matches!(err, CanopyError::Internal { .. }));
    }

    #[test]
    fn test_event_result_type() {
        fn returns_ok() -> EventResult<i32> {
            Ok(42)
        }

        fn returns_err() -> EventResult<i32> {
            Err(EventError::NoPredicateFound)
        }

        assert!(returns_ok().is_ok());
        assert!(returns_err().is_err());
    }
}
