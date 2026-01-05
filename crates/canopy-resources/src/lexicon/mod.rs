//! Canopy Lexicon Engine
//!
//! Analysis of closed-class words and functional lexical items.

pub mod engine;
pub mod parser;
pub mod types;

// Re-export main types
pub use engine::{LexiconConfig, LexiconEngine};
pub use parser::LexiconXmlResource;
pub use types::{
    ClassificationType, LexiconAnalysis, LexiconDatabase, LexiconPattern, LexiconStats,
    LexiconWord, PatternMatch, PatternType, Person, PronounCase, PronounFeatures, PronounGender,
    PronounNumber, PropertyValue, WordClass, WordClassType, WordClassification,
};
