//! Public API interface for the pipeline

use crate::error::PipelineError;
use canopy_semantic_layer::SemanticLayer1Output as SemanticAnalysis;
use serde::{Deserialize, Serialize};

/// Main analyzer interface
pub struct CanopyAnalyzer {
    // Implementation will be added later
}

impl CanopyAnalyzer {
    /// Create new analyzer
    pub fn new(_model_path: Option<&str>) -> Result<Self, PipelineError> {
        todo!("Implementation pending")
    }

    /// Create new async analyzer
    #[cfg(feature = "async")]
    pub async fn new_async(model_path: Option<&str>) -> Result<Self, PipelineError> {
        todo!("Implementation pending")
    }

    /// Analyze text synchronously
    pub fn analyze_sync(&self, _text: &str) -> Result<AnalysisResponse, PipelineError> {
        todo!("Implementation pending")
    }

    /// Analyze text asynchronously
    #[cfg(feature = "async")]
    pub async fn analyze(&self, text: &str) -> Result<AnalysisResponse, PipelineError> {
        todo!("Implementation pending")
    }
}

/// Analysis request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisRequest {
    pub text: String,
    pub config: Option<AnalysisConfig>,
}

/// Analysis response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResponse {
    pub analysis: SemanticAnalysis,
    pub metadata: ResponseMetadata,
}

/// Analysis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisConfig {
    pub enable_caching: bool,
    pub performance_mode: String,
}

/// Response metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMetadata {
    pub processing_time_ms: u64,
    pub model_used: String,
    pub cache_hit: bool,
}

/// Batch analysis request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchAnalysisRequest {
    pub texts: Vec<String>,
    pub config: Option<AnalysisConfig>,
}

/// Batch analysis response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchAnalysisResponse {
    pub results: Vec<AnalysisResponse>,
    pub summary: BatchSummary,
}

/// Batch processing summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchSummary {
    pub total_texts: usize,
    pub successful: usize,
    pub failed: usize,
    pub total_time_ms: u64,
}
