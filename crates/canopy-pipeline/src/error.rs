//! Error types for the pipeline

use std::time::Duration;
use thiserror::Error;

/// Main pipeline error type
#[derive(Debug, Error)]
pub enum PipelineError {
    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Analysis error: {0}")]
    AnalysisError(#[from] AnalysisError),

    #[error("Model loading error: {0}")]
    ModelLoadError(#[from] ModelLoadError),

    #[error("Pipeline not ready: {0}")]
    NotReady(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Timeout after {0:?}")]
    Timeout(Duration),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Analysis-specific errors
#[derive(Debug, Error)]
pub enum AnalysisError {
    #[error("Parse failed: {0}")]
    ParseFailed(String),

    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("Feature extraction failed: {0}")]
    FeatureExtractionFailed(String),

    #[error("Semantic analysis failed: {0}")]
    SemanticAnalysisFailed(String),

    #[error("Cache error: {0}")]
    CacheError(String),
}

/// Model loading errors
#[derive(Debug, Error)]
pub enum ModelLoadError {
    #[error("Model file not found: {0}")]
    FileNotFound(String),

    #[error("Invalid model format: {0}")]
    InvalidFormat(String),

    #[error("Model validation failed: {0}")]
    ValidationFailed(String),

    #[error("Download failed: {0}")]
    DownloadFailed(String),
}

/// Convert PipelineError to the unified CanopyError type
impl From<PipelineError> for canopy_core::CanopyError {
    fn from(error: PipelineError) -> Self {
        match error {
            PipelineError::ConfigurationError(msg) => Self::config(msg),
            PipelineError::AnalysisError(e) => e.into(),
            PipelineError::ModelLoadError(e) => e.into(),
            PipelineError::NotReady(msg) => Self::not_initialized(msg),
            PipelineError::InvalidInput(msg) => Self::invalid_input("valid input", msg),
            PipelineError::Timeout(duration) => {
                Self::timeout("pipeline", duration.as_millis() as u64)
            }
            PipelineError::IoError(e) => Self::from(e),
        }
    }
}

/// Convert AnalysisError to the unified CanopyError type
impl From<AnalysisError> for canopy_core::CanopyError {
    fn from(error: AnalysisError) -> Self {
        match error {
            AnalysisError::ParseFailed(msg) => Self::parse(msg),
            AnalysisError::ModelNotFound(msg) => Self::resource_not_found("model", msg),
            AnalysisError::FeatureExtractionFailed(msg) => {
                Self::analysis("feature extraction", msg)
            }
            AnalysisError::SemanticAnalysisFailed(msg) => Self::analysis("semantic", msg),
            AnalysisError::CacheError(msg) => Self::cache(msg),
        }
    }
}

/// Convert ModelLoadError to the unified CanopyError type
impl From<ModelLoadError> for canopy_core::CanopyError {
    fn from(error: ModelLoadError) -> Self {
        match error {
            ModelLoadError::FileNotFound(path) => Self::resource_not_found("model file", path),
            ModelLoadError::InvalidFormat(msg) => {
                Self::data_load(format!("invalid model format: {msg}"))
            }
            ModelLoadError::ValidationFailed(msg) => {
                Self::data_load(format!("model validation failed: {msg}"))
            }
            ModelLoadError::DownloadFailed(msg) => {
                Self::data_load(format!("model download failed: {msg}"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    // === PipelineError Display Tests ===

    #[test]
    fn test_configuration_error_display() {
        let err = PipelineError::ConfigurationError("invalid setting".into());
        assert_eq!(err.to_string(), "Configuration error: invalid setting");
    }

    #[test]
    fn test_analysis_error_display() {
        let inner = AnalysisError::ParseFailed("syntax error".into());
        let err = PipelineError::AnalysisError(inner);
        assert_eq!(
            err.to_string(),
            "Analysis error: Parse failed: syntax error"
        );
    }

    #[test]
    fn test_model_load_error_display() {
        let inner = ModelLoadError::FileNotFound("/path/to/model".into());
        let err = PipelineError::ModelLoadError(inner);
        assert_eq!(
            err.to_string(),
            "Model loading error: Model file not found: /path/to/model"
        );
    }

    #[test]
    fn test_not_ready_display() {
        let err = PipelineError::NotReady("models not loaded".into());
        assert_eq!(err.to_string(), "Pipeline not ready: models not loaded");
    }

    #[test]
    fn test_invalid_input_display() {
        let err = PipelineError::InvalidInput("empty string".into());
        assert_eq!(err.to_string(), "Invalid input: empty string");
    }

    #[test]
    fn test_timeout_display() {
        let err = PipelineError::Timeout(Duration::from_secs(30));
        assert_eq!(err.to_string(), "Timeout after 30s");
    }

    #[test]
    fn test_io_error_display() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let err = PipelineError::IoError(io_err);
        assert!(err.to_string().contains("IO error"));
    }

    // === AnalysisError Display Tests ===

    #[test]
    fn test_parse_failed_display() {
        let err = AnalysisError::ParseFailed("unexpected token".into());
        assert_eq!(err.to_string(), "Parse failed: unexpected token");
    }

    #[test]
    fn test_model_not_found_display() {
        let err = AnalysisError::ModelNotFound("bert-base".into());
        assert_eq!(err.to_string(), "Model not found: bert-base");
    }

    #[test]
    fn test_feature_extraction_failed_display() {
        let err = AnalysisError::FeatureExtractionFailed("dimension mismatch".into());
        assert_eq!(
            err.to_string(),
            "Feature extraction failed: dimension mismatch"
        );
    }

    #[test]
    fn test_semantic_analysis_failed_display() {
        let err = AnalysisError::SemanticAnalysisFailed("unknown verb class".into());
        assert_eq!(
            err.to_string(),
            "Semantic analysis failed: unknown verb class"
        );
    }

    #[test]
    fn test_cache_error_display() {
        let err = AnalysisError::CacheError("serialization failed".into());
        assert_eq!(err.to_string(), "Cache error: serialization failed");
    }

    // === ModelLoadError Display Tests ===

    #[test]
    fn test_file_not_found_display() {
        let err = ModelLoadError::FileNotFound("/models/test.bin".into());
        assert_eq!(err.to_string(), "Model file not found: /models/test.bin");
    }

    #[test]
    fn test_invalid_format_display() {
        let err = ModelLoadError::InvalidFormat("expected binary, got text".into());
        assert_eq!(
            err.to_string(),
            "Invalid model format: expected binary, got text"
        );
    }

    #[test]
    fn test_validation_failed_display() {
        let err = ModelLoadError::ValidationFailed("checksum mismatch".into());
        assert_eq!(
            err.to_string(),
            "Model validation failed: checksum mismatch"
        );
    }

    #[test]
    fn test_download_failed_display() {
        let err = ModelLoadError::DownloadFailed("connection timeout".into());
        assert_eq!(err.to_string(), "Download failed: connection timeout");
    }

    // === PipelineError to CanopyError Conversion Tests ===

    #[test]
    fn test_pipeline_error_conversion() {
        use canopy_core::CanopyError;

        // ConfigurationError
        let err: CanopyError = PipelineError::ConfigurationError("bad config".into()).into();
        assert!(matches!(err, CanopyError::Config { .. }));

        // NotReady
        let err: CanopyError = PipelineError::NotReady("not loaded".into()).into();
        assert!(matches!(err, CanopyError::NotInitialized { .. }));

        // InvalidInput
        let err: CanopyError = PipelineError::InvalidInput("empty".into()).into();
        assert!(matches!(err, CanopyError::InvalidInput { .. }));

        // Timeout
        let err: CanopyError = PipelineError::Timeout(Duration::from_millis(5000)).into();
        assert!(matches!(err, CanopyError::Timeout { .. }));

        // IoError
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "missing");
        let err: CanopyError = PipelineError::IoError(io_err).into();
        assert!(matches!(err, CanopyError::Io { .. }));
    }

    #[test]
    fn test_analysis_error_conversion() {
        use canopy_core::CanopyError;

        // ParseFailed
        let err: CanopyError = AnalysisError::ParseFailed("syntax".into()).into();
        assert!(matches!(err, CanopyError::Parse { .. }));

        // ModelNotFound
        let err: CanopyError = AnalysisError::ModelNotFound("test".into()).into();
        assert!(matches!(err, CanopyError::ResourceNotFound { .. }));

        // FeatureExtractionFailed
        let err: CanopyError = AnalysisError::FeatureExtractionFailed("fail".into()).into();
        assert!(matches!(err, CanopyError::Analysis { .. }));

        // SemanticAnalysisFailed
        let err: CanopyError = AnalysisError::SemanticAnalysisFailed("fail".into()).into();
        assert!(matches!(err, CanopyError::Analysis { .. }));

        // CacheError
        let err: CanopyError = AnalysisError::CacheError("fail".into()).into();
        assert!(matches!(err, CanopyError::Cache { .. }));
    }

    #[test]
    fn test_model_load_error_conversion() {
        use canopy_core::CanopyError;

        // FileNotFound
        let err: CanopyError = ModelLoadError::FileNotFound("/path".into()).into();
        assert!(matches!(err, CanopyError::ResourceNotFound { .. }));

        // InvalidFormat
        let err: CanopyError = ModelLoadError::InvalidFormat("bad".into()).into();
        assert!(matches!(err, CanopyError::DataLoad { .. }));

        // ValidationFailed
        let err: CanopyError = ModelLoadError::ValidationFailed("bad".into()).into();
        assert!(matches!(err, CanopyError::DataLoad { .. }));

        // DownloadFailed
        let err: CanopyError = ModelLoadError::DownloadFailed("timeout".into()).into();
        assert!(matches!(err, CanopyError::DataLoad { .. }));
    }

    #[test]
    fn test_nested_analysis_error_conversion() {
        use canopy_core::CanopyError;

        // AnalysisError nested in PipelineError
        let inner = AnalysisError::SemanticAnalysisFailed("verb not found".into());
        let outer = PipelineError::AnalysisError(inner);
        let err: CanopyError = outer.into();
        assert!(matches!(err, CanopyError::Analysis { .. }));
    }

    #[test]
    fn test_nested_model_load_error_conversion() {
        use canopy_core::CanopyError;

        // ModelLoadError nested in PipelineError
        let inner = ModelLoadError::DownloadFailed("network error".into());
        let outer = PipelineError::ModelLoadError(inner);
        let err: CanopyError = outer.into();
        assert!(matches!(err, CanopyError::DataLoad { .. }));
    }
}
