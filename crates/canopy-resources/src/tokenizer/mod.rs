//! Tokenizer module for text segmentation and preprocessing.
//!
//! Provides tokenization utilities using UAX #29 Unicode segmentation with
//! UD treebank-learned contraction patterns (e.g., `don't` → `["do", "n't"]`).
//!
//! ## Span Invariants
//!
//! All spans are **byte offsets** into the original text. For non-split tokens,
//! `text[byte_span.0..byte_span.1] == form`. For split tokens (from contractions),
//! the span points to the parent form and all split parts share the same span.

mod patterns;
mod simple;
mod unicode_tokenizer;

pub use patterns::{extract_patterns_from_treebank, load_ewt_patterns, ContractionPattern};
pub use simple::SimpleTokenizer;
pub use unicode_tokenizer::UnicodeTokenizer;

use canopy::CanopyError;

/// A raw token from initial tokenization (before POS tagging).
///
/// ## Span Semantics
///
/// - `byte_span` is always a valid byte range into the original text
/// - For regular tokens: `text[byte_span.0..byte_span.1] == form`
/// - For split tokens (contractions): span points to parent, e.g., "don't" → both
///   "do" and "n't" have `byte_span` pointing to the full "don't" substring
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawToken {
    /// The surface form of the token.
    /// For split tokens, this is the expanded form (e.g., "do" or "n't").
    pub form: String,

    /// Byte span in original text: `text[byte_span.0..byte_span.1]` is always valid.
    /// For split tokens, this points to the parent form (e.g., "don't").
    pub byte_span: (usize, usize),

    /// For split tokens: index within the parent contraction (0, 1, ...).
    /// `None` for regular (non-split) tokens.
    pub split_index: Option<usize>,

    /// For split tokens: total number of parts the parent was split into.
    /// `None` for regular (non-split) tokens.
    pub split_count: Option<usize>,
}

impl RawToken {
    /// Create a new regular (non-split) token with a byte span.
    ///
    /// # Invariant
    /// For non-split tokens, `text[byte_span.0..byte_span.1]` should equal `form`.
    #[must_use]
    pub fn new(form: String, byte_span: (usize, usize)) -> Self {
        Self {
            form,
            byte_span,
            split_index: None,
            split_count: None,
        }
    }

    /// Create a split token from a contraction expansion.
    ///
    /// All parts of a split contraction share the same `parent_span` pointing
    /// to the original contracted form in the text.
    ///
    /// # Arguments
    /// - `form`: The expanded form (e.g., "do" or "n't")
    /// - `parent_span`: Byte span of the original contraction (e.g., "don't")
    /// - `index`: Position in split sequence (0-based)
    /// - `count`: Total number of parts
    #[must_use]
    pub fn split(form: String, parent_span: (usize, usize), index: usize, count: usize) -> Self {
        Self {
            form,
            byte_span: parent_span,
            split_index: Some(index),
            split_count: Some(count),
        }
    }

    /// Returns true if this token was split from a contraction.
    #[must_use]
    pub fn is_split(&self) -> bool {
        self.split_index.is_some()
    }

    /// Verify the span invariant against the source text.
    ///
    /// For non-split tokens, checks that `text[byte_span] == form`.
    /// For split tokens, checks that the span is valid (but form differs).
    #[must_use]
    pub fn verify_span(&self, text: &str) -> bool {
        let span_text = text.get(self.byte_span.0..self.byte_span.1);
        if self.is_split() {
            // Split tokens: span must be valid, but form differs from span text
            span_text.is_some()
        } else {
            // Regular tokens: span text must equal form
            span_text.is_some_and(|s| s == self.form)
        }
    }
}

/// A sentence boundary with byte offsets into the original text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SentenceBoundary {
    /// The sentence text (trimmed).
    pub text: String,
    /// Byte span in original text.
    pub byte_span: (usize, usize),
}

impl SentenceBoundary {
    /// Create a new sentence boundary.
    #[must_use]
    pub fn new(text: String, byte_span: (usize, usize)) -> Self {
        Self { text, byte_span }
    }
}

/// Trait for tokenizers that segment text into tokens.
pub trait Tokenizer: Send + Sync {
    /// Tokenize text into raw tokens with byte spans.
    fn tokenize(&self, text: &str) -> Vec<RawToken>;

    /// Split text into sentences.
    fn split_sentences(&self, text: &str) -> Vec<String>;
}

/// Result type for tokenizer operations.
pub type TokenizerResult<T> = Result<T, CanopyError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raw_token_creation() {
        let token = RawToken::new("hello".to_string(), (0, 5));
        assert_eq!(token.form, "hello");
        assert_eq!(token.byte_span, (0, 5));
        assert!(!token.is_split());
    }

    #[test]
    fn test_raw_token_equality() {
        let t1 = RawToken::new("test".to_string(), (0, 4));
        let t2 = RawToken::new("test".to_string(), (0, 4));
        let t3 = RawToken::new("test".to_string(), (5, 9));
        assert_eq!(t1, t2);
        assert_ne!(t1, t3);
    }

    #[test]
    fn test_split_token() {
        // "don't" at bytes 0-5 splits into "do" and "n't"
        let t1 = RawToken::split("do".to_string(), (0, 5), 0, 2);
        let t2 = RawToken::split("n't".to_string(), (0, 5), 1, 2);

        assert!(t1.is_split());
        assert!(t2.is_split());
        assert_eq!(t1.byte_span, t2.byte_span); // Same parent span
        assert_eq!(t1.split_index, Some(0));
        assert_eq!(t2.split_index, Some(1));
        assert_eq!(t1.split_count, Some(2));
    }

    #[test]
    fn test_verify_span_regular() {
        let text = "hello world";
        let token = RawToken::new("hello".to_string(), (0, 5));
        assert!(token.verify_span(text));

        let bad_token = RawToken::new("wrong".to_string(), (0, 5));
        assert!(!bad_token.verify_span(text));
    }

    #[test]
    fn test_verify_span_split() {
        let text = "don't";
        let t1 = RawToken::split("do".to_string(), (0, 5), 0, 2);
        let t2 = RawToken::split("n't".to_string(), (0, 5), 1, 2);

        // Both should verify (span is valid, form differs)
        assert!(t1.verify_span(text));
        assert!(t2.verify_span(text));
    }

    #[test]
    fn test_sentence_boundary() {
        let boundary = SentenceBoundary::new("Hello world.".to_string(), (0, 12));
        assert_eq!(boundary.text, "Hello world.");
        assert_eq!(boundary.byte_span, (0, 12));
    }
}
