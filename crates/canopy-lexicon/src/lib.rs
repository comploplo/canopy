//! Canopy Lexicon Engine
//!
//! This crate provides comprehensive analysis of closed-class words and functional
//! lexical items in English, including stop words, negation markers, discourse
//! markers, quantifiers, temporal expressions, and more.
//!
//! # Features
//!
//! - **Comprehensive Word Classification**: Stop words, negation, discourse markers, quantifiers, etc.
//! - **Pattern-based Analysis**: Morphological patterns for prefixes, suffixes, and phrases
//! - **XML Data Format**: Structured lexicon data with validation and extensibility
//! - **High-Performance Lookup**: Fast classification with caching and indexing
//! - **Discourse Analysis**: Negation scope, discourse structure, and semantic weighting
//! - **Engine Integration**: Full canopy-engine trait implementation
//!
//! # Example
//!
//! ```rust,no_run
//! use canopy_lexicon::{LexiconEngine, LexiconConfig, WordClassType};
//! use canopy_engine::SemanticEngine;
//!
//! // Create and configure engine
//! let config = LexiconConfig::default();
//! let mut engine = LexiconEngine::with_config(config);
//!
//! // Load lexicon data
//! engine.load_data().expect("Failed to load lexicon data");
//!
//! // Classify words
//! let is_stop = engine.is_stop_word("the").expect("Classification failed");
//! let is_negation = engine.is_negation("not").expect("Classification failed");
//!
//! // Analyze patterns
//! let analysis = engine.analyze_word("unhappy").expect("Analysis failed");
//! println!("Pattern matches: {}", analysis.data.pattern_matches.len());
//! ```
//!
//! # Word Classes
//!
//! The lexicon includes these major word classes:
//!
//! - **Stop Words**: Function words with low semantic content
//! - **Negation**: Words and patterns indicating negation or denial
//! - **Discourse Markers**: Words organizing discourse relationships
//! - **Quantifiers**: Words indicating quantity, amount, or degree
//! - **Temporal**: Time-related expressions and temporal markers
//! - **Intensifiers**: Words that strengthen or weaken other words
//! - **Hedge Words**: Uncertainty and approximation markers

pub mod engine;
pub mod parser;
pub mod types;

// Re-export main types for convenience
pub use engine::{LexiconConfig, LexiconEngine};
pub use parser::LexiconXmlResource;
pub use types::{
    ClassificationType, LexiconAnalysis, LexiconDatabase, LexiconPattern, LexiconStats,
    LexiconWord, PatternMatch, PatternType, PropertyValue, WordClass, WordClassType,
    WordClassification,
};

// Re-export engine traits
pub use canopy_engine::{
    CachedEngine, DataLoader, EngineError, EngineResult, SemanticEngine, SemanticResult,
    StatisticsProvider,
};
