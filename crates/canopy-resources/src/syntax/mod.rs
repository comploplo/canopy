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

mod resource_tagger;
mod shared;
mod treebank_provider;
mod word_pos_index;

pub use resource_tagger::ResourceBackedTagger;
pub use shared::{parse_deprel, parse_upos};
pub use treebank_provider::{TreebankConfig, TreebankSyntaxProvider};
pub use word_pos_index::WordPosIndex;

use canopy::CanopyError;

/// Result type for syntax operations.
pub type SyntaxResult<T> = Result<T, CanopyError>;
