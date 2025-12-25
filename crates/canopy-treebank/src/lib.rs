//! UD Treebank Engine for Dependency Pattern Analysis
//!
//! This crate provides a semantic engine that extracts dependency patterns
//! from Universal Dependencies (UD) treebank data and matches them to semantic
//! signatures generated from Layer 1 semantic analysis.
//!
//! # Features
//!
//! - **CoNLL-U Parsing**: Parse UD English-EWT treebank format
//! - **Pattern Indexing**: Extract and index dependency patterns by frequency
//! - **Semantic Signatures**: Create hash keys from VerbNet + FrameNet + lemmas
//! - **Adaptive Caching**: Core patterns + LRU cache for performance
//! - **Pattern Synthesis**: Fallback generation from VerbNet/FrameNet data
//!
//! # Usage
//!
//! ```rust,no_run
//! use canopy_treebank::{TreebankEngine, engine::TreebankConfig};
//! use canopy_engine::SemanticEngine;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let config = TreebankConfig::default();
//! let engine = TreebankEngine::with_config(config)?;
//! let analysis = engine.analyze_word("running")?;
//! # Ok(())
//! # }
//! ```

pub mod cache;
pub mod conllu_types;
pub mod engine;
pub mod indexer;
pub mod lemma_validator;
pub mod parser;
pub mod pattern_cache;
pub mod pattern_indexer;
pub mod semantic_integration;
pub mod signature;
pub mod synthesizer;
pub mod types;

// Re-export main types for convenience
pub use cache::AdaptiveCache;
pub use conllu_types::{
    ConlluCorpusStats, ConlluSentence, ConlluToken, DependencyTree, MorphologicalFeatures,
    UniversalPos,
};
pub use engine::TreebankEngine;
pub use indexer::{PatternIndexer as OldPatternIndexer, TreebankIndex};
pub use lemma_validator::{LemmaValidationResult, LemmaValidator};
pub use parser::{ConlluParser, ParsedSentence, ParsedToken};
pub use pattern_cache::{CacheStatistics, PatternCache, PatternCacheFactory};
pub use pattern_indexer::PatternIndexer;
pub use semantic_integration::{
    ExtendedSemanticResult, TreebankSemanticConfig, TreebankSemanticCoordinator,
};
pub use signature::{SemanticSignature, SignatureBuilder};
pub use synthesizer::PatternSynthesizer;
pub use types::{
    DependencyFeatureType, DependencyFeatures, DependencyPattern, DependencyRelation,
    PatternSource, SemanticRoleFeature, SyntacticFeature, TemporalFeature, TreebankAnalysis,
    VoiceFeature,
};

/// Result type for treebank operations
pub type TreebankResult<T> = Result<T, canopy_engine::EngineError>;
