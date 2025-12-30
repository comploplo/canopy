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
