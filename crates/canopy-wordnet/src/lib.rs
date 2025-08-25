//! WordNet semantic engine for canopy.rs
//!
//! This crate provides comprehensive WordNet 3.1 integration for semantic analysis,
//! including lexical lookup, semantic relations, and morphological processing.
//!
//! # Features
//!
//! - **Complete WordNet Integration**: Full support for synsets, semantic relations, and word senses
//! - **High-Performance Parsing**: Optimized parsers for WordNet's binary data format
//! - **Semantic Analysis**: Hypernym/hyponym relationships, similarity calculations, and more
//! - **Morphological Processing**: Exception handling and word form normalization
//! - **Engine Integration**: Implements all canopy-engine traits for seamless integration
//!
//! # Example
//!
//! ```rust
//! use canopy_wordnet::{WordNetEngine, WordNetConfig, PartOfSpeech};
//! use canopy_engine::SemanticEngine;
//!
//! // Create and configure engine
//! let config = WordNetConfig::default();
//! let mut engine = WordNetEngine::new(config);
//!
//! // Load WordNet data
//! engine.load_data().expect("Failed to load WordNet data");
//!
//! // Analyze a word
//! let result = engine.analyze_word("dog", PartOfSpeech::Noun)
//!     .expect("Analysis failed");
//!
//! println!("Definitions: {:?}", result.definitions);
//! println!("Synsets: {}", result.synsets.len());
//! ```

pub mod engine;
pub mod loader;
pub mod parser;
pub mod types;

// Re-export main types for convenience
pub use engine::{WordNetConfig, WordNetEngine};
pub use loader::WordNetLoader;
pub use parser::{WordNetParser, WordNetParserConfig};
pub use types::{
    DatabaseStats, ExceptionEntry, IndexEntry, PartOfSpeech, SemanticPointer, SemanticRelation,
    Synset, SynsetWord, VerbFrame, WordNetAnalysis, WordNetDatabase,
};

// Re-export engine traits
pub use canopy_engine::{
    CachedEngine, DataLoader, EngineError, EngineResult, SemanticEngine, SemanticResult,
    StatisticsProvider,
};
