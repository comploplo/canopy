//! Consolidated Semantic Analysis Engines for Canopy
//!
//! This crate provides unified access to all semantic analysis engines:
//! - **VerbNet**: Verb class analysis with theta roles and frames
//! - **FrameNet**: Frame-based semantic parsing
//! - **WordNet**: Lexical database for synonyms, hypernyms, etc.
//! - **Lexicon**: Closed-class words and function words
//! - **PropBank**: Semantic role labeling with predicate-argument structures
//!
//! # Feature Flags
//!
//! Each engine can be enabled/disabled via feature flags:
//! - `verbnet` - VerbNet 3.4 support
//! - `framenet` - FrameNet 1.7 support
//! - `wordnet` - WordNet 3.1 support
//! - `lexicon` - Lexicon engine support
//! - `propbank` - PropBank support
//! - `all` - Enable all engines (default)
//! - `parallel` - Enable parallel processing
//!
//! # Example
//!
//! ```rust,no_run
//! use canopy_semantic_engines::verbnet::VerbNetEngine;
//! use canopy_semantic_engines::framenet::FrameNetEngine;
//! use canopy_engine::SemanticEngine;
//!
//! // Create engines
//! let verbnet = VerbNetEngine::new().expect("Failed to create VerbNet");
//! let framenet = FrameNetEngine::new().expect("Failed to create FrameNet");
//! ```

#[cfg(feature = "verbnet")]
pub mod verbnet;

#[cfg(feature = "framenet")]
pub mod framenet;

#[cfg(feature = "wordnet")]
pub mod wordnet;

#[cfg(feature = "lexicon")]
pub mod lexicon;

#[cfg(feature = "propbank")]
pub mod propbank;

// Re-export engine traits for convenience
pub use canopy_engine::{
    CachedEngine, DataLoader, EngineError, EngineResult, SemanticEngine, SemanticResult,
    StatisticsProvider,
};

// Convenience re-exports at crate root for common types
#[cfg(feature = "verbnet")]
pub use verbnet::{VerbNetAnalysis, VerbNetConfig, VerbNetEngine};

#[cfg(feature = "framenet")]
pub use framenet::{FrameNetAnalysis, FrameNetConfig, FrameNetEngine};

#[cfg(feature = "wordnet")]
pub use wordnet::{PartOfSpeech, WordNetAnalysis, WordNetConfig, WordNetEngine};

#[cfg(feature = "lexicon")]
pub use lexicon::{LexiconAnalysis, LexiconConfig, LexiconEngine, WordClassType};

#[cfg(feature = "propbank")]
pub use propbank::{PropBankAnalysis, PropBankConfig, PropBankEngine, SemanticRole};
