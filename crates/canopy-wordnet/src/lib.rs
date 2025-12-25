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
//! use canopy_wordnet::{WordNetEngine, PartOfSpeech};
//!
//! let engine = WordNetEngine::new();
//! // engine.load_from_directory("path/to/wordnet/data")?;
//!
//! // With loaded data, you can analyze words:
//! // let result = engine.analyze_word("dog", PartOfSpeech::Noun)?;
//! // println!("Definitions: {:?}", result.data.definitions);
//! // println!("Synsets: {}", result.data.synsets.len());
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
