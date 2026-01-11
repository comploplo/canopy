//! Syntax provider implementations.
//!
//! Provides implementations of the `SyntaxProvider` trait that produce
//! `AnnotatedSyntax` from raw text. The primary implementation uses
//! patterns learned from the UD English-EWT treebank.
//!
//! ## Architecture
//!
//! - `TreebankSyntaxProvider`: Main provider, tries exact pattern match first
//! - `ResourceBackedTagger`: Fallback tagger using validated datasets
//! - `WordPosIndex`: Fast word→POS lookup from treebank statistics
//! - `shared`: Common utilities for POS parsing, lemmatization, dependency guessing

mod gerund;
mod mwe;
mod pattern_matcher;
mod pattern_types;
mod phrasal_verb;
mod resource_tagger;
mod shared;
mod treebank_provider;
mod verbnet_patterns;
mod word_pos_index;

pub use gerund::GerundClassifier;
pub use mwe::{Mwe, MweDetector, MweType};
pub use pattern_matcher::{extract_patterns_from_syntax, MatcherStats, PatternMatcher};
pub use pattern_types::{ArgumentPattern, ArgumentPosition, DependencyPattern, SemanticSignature};
pub use phrasal_verb::PhrasalVerbDetector;
pub use resource_tagger::ResourceBackedTagger;
pub use shared::{parse_deprel, parse_upos};
pub use treebank_provider::{TreebankConfig, TreebankSyntaxProvider};
pub use verbnet_patterns::{
    pattern_count as verbnet_pattern_count, synthesize_pattern, VERBNET_PATTERNS,
};
pub use word_pos_index::{WordLemmaIndex, WordPosIndex};

use canopy::CanopyError;

/// Result type for syntax operations.
pub type SyntaxResult<T> = Result<T, CanopyError>;
