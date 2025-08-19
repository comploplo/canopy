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
