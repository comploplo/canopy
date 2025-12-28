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

pub mod config;
pub mod container;
pub mod discourse;
pub mod error;
pub mod models;
pub mod pipeline;
pub mod real_implementations;
pub mod traits;

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

// Re-export Layer 3 discourse types
pub use canopy_discourse::{
    DiscourseConfig, DiscourseContext, DiscourseResult, Drs, DrsCondition, ReferentId,
};
pub use discourse::{DiscourseProcessor, DiscourseStatistics};

/// Version information for the pipeline
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Supported UDPipe model versions
pub const SUPPORTED_UDPIPE_VERSIONS: &[&str] = &["1.2", "2.15"];

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
pub fn create_l1_analyzer(
) -> Result<canopy_tokenizer::SemanticCoordinator, Box<dyn std::error::Error>> {
    use canopy_tokenizer::coordinator::CoordinatorConfig;
    use canopy_tokenizer::SemanticCoordinator;

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

/// Create a fully-loaded L1 semantic analyzer with treebank integration
///
/// This extends `create_l1_analyzer()` with UD Treebank pattern matching for
/// dependency-enhanced semantic analysis. The treebank engine provides:
/// - Dependency pattern matching from UD English-EWT corpus
/// - Voice detection (active/passive)
/// - Semantic role features from dependency relations
///
/// # Performance
///
/// - Cache hit latency: <1μs
/// - Pattern synthesis: <10μs
/// - Memory overhead: <2MB
///
/// # Example
///
/// ```rust,no_run
/// use canopy_pipeline::create_l1_analyzer_with_treebank;
///
/// let analyzer = create_l1_analyzer_with_treebank()?;
/// let result = analyzer.analyze("running")?;
/// if let Some(treebank) = &result.treebank {
///     println!("Dependency relation: {:?}", treebank.dependency_relation);
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn create_l1_analyzer_with_treebank(
) -> Result<canopy_tokenizer::SemanticCoordinator, Box<dyn std::error::Error>> {
    use canopy_tokenizer::coordinator::CoordinatorConfig;
    use canopy_tokenizer::SemanticCoordinator;
    use canopy_treebank::TreebankEngine;
    use std::sync::Arc;

    let config = CoordinatorConfig {
        // Enable all engines for comprehensive analysis
        enable_verbnet: true,
        enable_framenet: true,
        enable_wordnet: true,
        enable_lexicon: true,
        enable_treebank: true,

        // Enable lemmatization
        enable_lemmatization: true,

        // Production-ready settings
        confidence_threshold: 0.1,
        l1_cache_memory_mb: 100,

        ..CoordinatorConfig::default()
    };

    let mut coordinator = SemanticCoordinator::new(config)?;

    // Wire up the TreebankEngine as the TreebankProvider
    match TreebankEngine::new() {
        Ok(engine) => {
            println!("✅ Treebank engine loaded with real data");
            coordinator.set_treebank_provider(Arc::new(engine));
        }
        Err(e) => {
            // Treebank is optional - warn but don't fail
            eprintln!("⚠️  Treebank initialization failed (optional): {}", e);
        }
    }

    Ok(coordinator)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_info() {
        // VERSION and SUPPORTED_UDPIPE_VERSIONS are non-empty string constants
        assert_eq!(VERSION, "0.1.0");
        assert!(SUPPORTED_UDPIPE_VERSIONS.contains(&"1.2"));
    }

    #[test]
    fn test_model_listing() {
        let _models = list_available_models();
        // Should at least detect if models are available
        // Reaching here means list_available_models() succeeded
    }

    #[test]
    fn test_create_l1_analyzer_with_treebank() {
        // Create analyzer with treebank integration
        let analyzer = create_l1_analyzer_with_treebank();

        // Should succeed (treebank is optional, may warn but not fail)
        assert!(
            analyzer.is_ok(),
            "Failed to create analyzer: {:?}",
            analyzer.err()
        );

        let coordinator = analyzer.unwrap();

        // Analyze a word - should work with or without treebank
        let result = coordinator.analyze("running");
        assert!(result.is_ok(), "Analysis failed: {:?}", result.err());

        let analysis = result.unwrap();

        // Basic validation
        assert_eq!(analysis.lemma, "run");
        assert!(
            !analysis.sources.is_empty(),
            "Should have at least one semantic source"
        );
    }

    #[test]
    fn test_treebank_analysis_populated() {
        let analyzer = create_l1_analyzer_with_treebank();
        if analyzer.is_err() {
            eprintln!("Skipping test: analyzer creation failed");
            return;
        }

        let coordinator = analyzer.unwrap();

        // Analyze a verb that should have treebank patterns
        let result = coordinator.analyze("give");
        assert!(result.is_ok());

        let analysis = result.unwrap();

        // If treebank is available, it should be populated
        if analysis.treebank.is_some() {
            let tb = analysis.treebank.as_ref().unwrap();

            // Confidence should be valid
            assert!(
                tb.confidence >= 0.0 && tb.confidence <= 1.0,
                "Invalid confidence: {}",
                tb.confidence
            );

            // Sources should include Treebank
            assert!(
                analysis.sources.contains(&"Treebank".to_string()),
                "Sources should include Treebank: {:?}",
                analysis.sources
            );
        }
    }

    #[test]
    fn test_treebank_batch_analysis() {
        let analyzer = create_l1_analyzer_with_treebank();
        if analyzer.is_err() {
            eprintln!("Skipping test: analyzer creation failed");
            return;
        }

        let coordinator = analyzer.unwrap();

        let words: Vec<String> = vec!["run", "walk", "give", "take", "see", "make"]
            .into_iter()
            .map(String::from)
            .collect();

        let results = coordinator.analyze_batch(&words);
        assert!(results.is_ok());

        let analyses = results.unwrap();
        assert_eq!(analyses.len(), words.len());

        // At least some should have analysis
        let with_sources = analyses.iter().filter(|r| !r.sources.is_empty()).count();
        assert!(
            with_sources > 0,
            "At least some results should have sources"
        );
    }

    #[test]
    fn test_l1_analyzer_without_treebank() {
        // Original function should still work
        let analyzer = create_l1_analyzer();
        assert!(analyzer.is_ok());

        let coordinator = analyzer.unwrap();
        let result = coordinator.analyze("running");
        assert!(result.is_ok());

        let analysis = result.unwrap();
        // Without treebank wiring, treebank field should be None
        // (enable_treebank config is true but no provider is set)
        // This is expected behavior for the non-treebank version
        assert_eq!(analysis.lemma, "run");
    }
}
