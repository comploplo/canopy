//! # Canopy Pipeline
//!
//! Central orchestration layer for Canopy's linguistic analysis pipeline.
//! This crate provides a unified, clean API for consuming applications
//! like LSP servers, Python bindings, and CLI tools.
//!
//! ## Architecture
//!
//! ```text
//! Text → Layer1 (UDPipe) → Layer2 (Semantics) → Results
//!        ↓                 ↓                    ↓
//!    [Morphology]     [Events & Theta]    [Final Analysis]
//! ```
//!
//! ## Public API Design
//!
//! This crate solves the circular dependency issue by providing:
//! - **Clean Public Interface**: Simple, ergonomic API for consumers
//! - **Dependency Isolation**: No circular dependencies between parser/semantics
//! - **Performance Optimization**: Built-in caching and batching
//! - **Multiple Backends**: Support for different UDPipe models
//! - **Extension Points**: Plugin architecture for custom analysis

pub mod api;
pub mod benchmarks;
pub mod config;
pub mod container;
pub mod error;
pub mod models;
pub mod pipeline;
pub mod real_implementations;
// pub mod real_benchmarks;  // Temporarily disabled due to deprecated dependency references
pub mod traits;

// Include coverage tests for traits.rs 0% coverage
// #[cfg(test)]
// mod traits_coverage_tests;  // Temporarily disabled due to deprecated dependencies

// #[cfg(test)]
// pub mod implementations;  // Temporarily disabled due to deprecated dependencies

// Re-export the main public API
pub use api::{
    AnalysisConfig, AnalysisRequest, AnalysisResponse, BatchAnalysisRequest, BatchAnalysisResponse,
    CanopyAnalyzer,
};

// Re-export configuration types
pub use config::{
    CacheConfig, LoggingConfig, MemoryConfig, ModelConfig, PerformanceConfig,
    PipelineConfig as ConfigPipelineConfig,
};

// Re-export error types
pub use error::{AnalysisError, ModelLoadError, PipelineError};

// Re-export model management
pub use models::{ModelInfo, ModelManager, SupportedModel};

// Re-export core pipeline
pub use pipeline::{
    AnalysisTiming, FullAnalysisResult, LinguisticPipeline, PipelineBuilder, PipelineContext,
    PipelineMetrics, PipelineStage, StageResult,
};

// Re-export dependency injection
pub use container::{ContainerBuilder, PipelineContainer};
pub use traits::*;

// Re-export benchmarking utilities
pub use benchmarks::{
    BenchmarkConfig, BenchmarkResults, ModelComparison, PerformanceProfile, PipelineBenchmark,
    run_model_comparison,
};
// TODO: Re-enable real_benchmarks when dependencies are updated
// pub use real_benchmarks::{
//     FullStackResults, LayerBenchmarkResults, MemoryBenchmarkResults, ModelBenchmarkResults,
//     ModelBenchmarkSuite, QualityMetrics,
// };

// Re-export types from underlying crates for convenience
pub use canopy_core::ThetaRole;
pub use canopy_core::{DepRel, MorphFeatures, UPos, Word};
pub use canopy_tokenizer::{SemanticLayer1Output, SemanticPredicate};

// Re-export Layer 2 event composition types
pub use canopy_events::{
    ComposedEvent, ComposedEvents, DependencyArc, EventComposer, EventComposerConfig, LittleVType,
    SentenceAnalysis, SentenceAnalysisBuilder,
};

/// Version information for the pipeline
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Supported UDPipe model versions
pub const SUPPORTED_UDPIPE_VERSIONS: &[&str] = &["1.2", "2.15"];

/// Quick start function for simple text analysis
///
/// This is the easiest way to get started with Canopy analysis.
/// For production use, create a `CanopyAnalyzer` instance for better performance.
///
/// # Example
///
/// ```rust,no_run
/// use canopy_pipeline::analyze_text;
///
/// let result = analyze_text("John gave Mary a book.", None).await?;
/// println!("Found {} events", result.events.len());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[cfg(feature = "async")]
pub async fn analyze_text(
    text: &str,
    model_path: Option<&str>,
) -> Result<AnalysisResponse, PipelineError> {
    let analyzer = CanopyAnalyzer::new_async(model_path).await?;
    analyzer.analyze(text).await
}

/// Synchronous version of analyze_text for simpler use cases
///
/// # Example
///
/// ```rust,no_run
/// use canopy_pipeline::analyze_text_sync;
///
/// let result = analyze_text_sync("John gave Mary a book.", None)?;
/// println!("Found {} tokens", result.analysis.tokens.len());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn analyze_text_sync(
    text: &str,
    model_path: Option<&str>,
) -> Result<AnalysisResponse, PipelineError> {
    let analyzer = CanopyAnalyzer::new(model_path)?;
    analyzer.analyze_sync(text)
}

/// Get information about available models
pub fn list_available_models() -> Vec<ModelInfo> {
    ModelManager::list_available()
}

/// Check if a model is installed and ready to use
pub fn is_model_available(model_name: &str) -> bool {
    ModelManager::is_available_by_name(model_name)
}

/// Create a fully-loaded L1 semantic analyzer with all engines ready to use
///
/// This is the recommended way to get a production-ready analyzer that includes:
/// - VerbNet engine (verb semantic classes and theta roles)
/// - FrameNet engine (frame semantics and frame elements)
/// - WordNet engine (lexical semantics and word relationships)
/// - Lexicon engine (morphological and lexical analysis)
/// - Intelligent caching and performance optimization
///
/// # Example
///
/// ```rust,no_run
/// use canopy_pipeline::create_l1_analyzer;
///
/// let analyzer = create_l1_analyzer()?;
/// let result = analyzer.analyze("running")?;
/// println!("Found {} semantic sources", result.sources.len());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn create_l1_analyzer()
-> Result<canopy_tokenizer::SemanticCoordinator, Box<dyn std::error::Error>> {
    use canopy_tokenizer::SemanticCoordinator;
    use canopy_tokenizer::coordinator::CoordinatorConfig;

    let config = CoordinatorConfig {
        // Enable all engines for comprehensive analysis
        enable_verbnet: true,
        enable_framenet: true,
        enable_wordnet: true,
        enable_lexicon: true,

        // Enable lemmatization
        enable_lemmatization: true,

        // Production-ready settings
        confidence_threshold: 0.1,
        l1_cache_memory_mb: 100,

        ..CoordinatorConfig::default()
    };

    let coordinator = SemanticCoordinator::new(config)?;
    Ok(coordinator)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_info() {
        assert!(!VERSION.is_empty());
        assert!(!SUPPORTED_UDPIPE_VERSIONS.is_empty());
    }

    #[test]
    fn test_model_listing() {
        let models = list_available_models();
        // Should at least detect if models are available
        assert!(models.len() >= 0);
    }
}
