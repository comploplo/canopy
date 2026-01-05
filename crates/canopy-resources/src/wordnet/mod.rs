//! `WordNet` semantic engine
//!
//! Complete `WordNet` 3.1 integration for lexical lookup and semantic analysis.

pub mod engine;
pub mod loader;
pub mod parser;
pub mod types;

// Re-export main types
pub use engine::{WordNetConfig, WordNetEngine};
pub use loader::WordNetLoader;
pub use parser::{WordNetParser, WordNetParserConfig};
pub use types::{
    DatabaseStats, ExceptionEntry, IndexEntry, PartOfSpeech, SemanticPointer, SemanticRelation,
    Synset, SynsetWord, VerbFrame, WordNetAnalysis, WordNetDatabase,
};
