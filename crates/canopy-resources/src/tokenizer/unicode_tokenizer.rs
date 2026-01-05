//! Unicode-aware tokenizer using UAX #29 segmentation.
//!
//! Provides standards-compliant word and sentence segmentation with
//! UD treebank-learned contraction splitting as a post-pass.

use super::{ContractionPattern, RawToken, SentenceBoundary, Tokenizer, TokenizerResult};
use std::collections::{HashMap, HashSet};
use unicode_segmentation::UnicodeSegmentation;

/// A Unicode-aware tokenizer using UAX #29 word boundaries.
///
/// This tokenizer uses the Unicode Text Segmentation standard (UAX #29) for
/// word and sentence boundaries, then applies UD treebank-learned contraction
/// patterns as a post-processing step.
///
/// ## Span Invariants
///
/// All byte spans are valid slices into the original text:
/// - For regular tokens: `text[byte_span.0..byte_span.1] == form`
/// - For split tokens: all parts share the parent's span
#[derive(Debug)]
pub struct UnicodeTokenizer {
    /// Contraction patterns: lowercase form → expanded tokens.
    contractions: HashMap<String, Vec<String>>,
    /// Punctuation characters to separate (post-UAX processing).
    punctuation: HashSet<char>,
}

impl Default for UnicodeTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

impl UnicodeTokenizer {
    /// Create a new tokenizer with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self {
            contractions: HashMap::new(),
            punctuation: Self::default_punctuation(),
        }
    }

    /// Create a tokenizer with patterns learned from treebank.
    #[must_use]
    pub fn with_treebank_patterns(patterns: &[ContractionPattern]) -> Self {
        let mut tokenizer = Self::new();
        for pattern in patterns {
            tokenizer
                .contractions
                .insert(pattern.form.to_lowercase(), pattern.tokens.clone());
        }
        tokenizer
    }

    /// Load patterns from UD English-EWT and create tokenizer.
    ///
    /// # Errors
    /// Returns an error if the treebank patterns cannot be loaded.
    pub fn from_ewt() -> TokenizerResult<Self> {
        let patterns = super::load_ewt_patterns()?;
        Ok(Self::with_treebank_patterns(&patterns))
    }

    fn default_punctuation() -> HashSet<char> {
        [
            '.', ',', '!', '?', ';', ':', '"', '\'', '(', ')', '[', ']', '{', '}', '-', '–', '—',
            '/', '\\', '@', '#', '$', '%', '^', '&', '*', '+', '=', '<', '>', '|', '~', '`',
        ]
        .into_iter()
        .collect()
    }

    /// Tokenize using UAX #29 word boundaries, then separate punctuation.
    fn tokenize_base(&self, text: &str) -> Vec<RawToken> {
        let mut tokens = Vec::new();

        // UAX #29 split_word_bound_indices gives (byte_offset, segment)
        for (byte_offset, segment) in text.split_word_bound_indices() {
            // Skip pure whitespace segments
            if segment.chars().all(char::is_whitespace) {
                continue;
            }

            // Separate leading punctuation
            let current_offset = byte_offset;
            let mut chars = segment.char_indices().peekable();

            while let Some(&(i, c)) = chars.peek() {
                if self.punctuation.contains(&c) && c != '\'' {
                    let c_len = c.len_utf8();
                    tokens.push(RawToken::new(
                        c.to_string(),
                        (current_offset + i, current_offset + i + c_len),
                    ));
                    chars.next();
                } else {
                    break;
                }
            }

            // Find where core word starts
            let core_start_in_segment = chars.peek().map_or(segment.len(), |(i, _)| *i);
            let core_start = current_offset + core_start_in_segment;

            // Collect remaining characters to find trailing punctuation
            let remaining: Vec<(usize, char)> = chars.collect();
            if remaining.is_empty() {
                continue;
            }

            // Find trailing punctuation
            let mut core_end_idx = remaining.len();
            let mut trailing_punct = Vec::new();

            for i in (0..remaining.len()).rev() {
                let (idx_in_seg, c) = remaining[i];
                if self.punctuation.contains(&c) && c != '\'' {
                    trailing_punct.push((current_offset + idx_in_seg, c));
                    core_end_idx = i;
                } else {
                    break;
                }
            }

            // Extract core word
            if core_end_idx > 0 {
                let core_chars: String = remaining[..core_end_idx].iter().map(|(_, c)| c).collect();
                let core_end = if core_end_idx < remaining.len() {
                    current_offset + remaining[core_end_idx].0
                } else {
                    byte_offset + segment.len()
                };

                if !core_chars.is_empty() {
                    tokens.push(RawToken::new(core_chars, (core_start, core_end)));
                }
            }

            // Add trailing punctuation in original order
            trailing_punct.reverse();
            for (offset, c) in trailing_punct {
                tokens.push(RawToken::new(
                    c.to_string(),
                    (offset, offset + c.len_utf8()),
                ));
            }
        }

        tokens
    }

    /// Apply contraction splitting with shared parent spans.
    fn apply_contractions(&self, tokens: Vec<RawToken>) -> Vec<RawToken> {
        let mut result = Vec::with_capacity(tokens.len() + 10);

        for token in tokens {
            if token.is_split() {
                // Already split, pass through
                result.push(token);
                continue;
            }

            let lower = token.form.to_lowercase();
            if let Some(expansions) = self.contractions.get(&lower) {
                // Split token: all parts share the SAME byte_span
                let parent_span = token.byte_span;
                let count = expansions.len();

                for (i, expanded_form) in expansions.iter().enumerate() {
                    let mut form = expanded_form.clone();

                    // Preserve case of first letter if original was capitalized
                    if i == 0 && token.form.chars().next().is_some_and(char::is_uppercase) {
                        if let Some(first) = form.chars().next() {
                            form = format!("{}{}", first.to_uppercase(), &form[first.len_utf8()..]);
                        }
                    }

                    result.push(RawToken::split(form, parent_span, i, count));
                }
            } else {
                result.push(token);
            }
        }

        result
    }

    /// Split text into sentences using UAX #29 sentence boundaries.
    #[must_use]
    pub fn split_sentences_with_spans(&self, text: &str) -> Vec<SentenceBoundary> {
        let mut sentences = Vec::new();
        let mut byte_offset = 0;

        for sentence in text.split_sentence_bounds() {
            let trimmed = sentence.trim();
            if !trimmed.is_empty() {
                // Find byte position in original text
                let start = byte_offset;
                let end = byte_offset + sentence.len();

                sentences.push(SentenceBoundary::new(trimmed.to_string(), (start, end)));
            }
            byte_offset += sentence.len();
        }

        sentences
    }
}

impl Tokenizer for UnicodeTokenizer {
    fn tokenize(&self, text: &str) -> Vec<RawToken> {
        let base_tokens = self.tokenize_base(text);
        self.apply_contractions(base_tokens)
    }

    fn split_sentences(&self, text: &str) -> Vec<String> {
        self.split_sentences_with_spans(text)
            .into_iter()
            .map(|s| s.text)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unicode_tokenizer_creation() {
        let tokenizer = UnicodeTokenizer::new();
        assert!(tokenizer.contractions.is_empty());
        assert!(!tokenizer.punctuation.is_empty());
    }

    #[test]
    fn test_tokenize_simple() {
        let tokenizer = UnicodeTokenizer::new();
        let text = "Hello world";
        let tokens = tokenizer.tokenize(text);

        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].form, "Hello");
        assert_eq!(tokens[1].form, "world");

        // Verify span invariant
        for token in &tokens {
            assert!(
                token.verify_span(text),
                "Span invariant failed for {token:?}"
            );
        }
    }

    #[test]
    fn test_tokenize_with_punctuation() {
        let tokenizer = UnicodeTokenizer::new();
        let text = "Hello, world!";
        let tokens = tokenizer.tokenize(text);

        assert!(tokens.len() >= 4);
        assert_eq!(tokens[0].form, "Hello");

        // Verify all spans are valid
        for token in &tokens {
            assert!(token.verify_span(text));
        }
    }

    #[test]
    fn test_byte_span_invariant_ascii() {
        let tokenizer = UnicodeTokenizer::new();
        let text = "The quick brown fox";
        let tokens = tokenizer.tokenize(text);

        for token in &tokens {
            if !token.is_split() {
                assert_eq!(
                    &text[token.byte_span.0..token.byte_span.1],
                    token.form,
                    "Span invariant failed for: {token:?}"
                );
            }
        }
    }

    #[test]
    fn test_byte_span_invariant_unicode() {
        let tokenizer = UnicodeTokenizer::new();
        let text = "Café naïve";
        let tokens = tokenizer.tokenize(text);

        // Should have tokens for multi-byte UTF-8 characters
        for token in &tokens {
            if !token.is_split() {
                let extracted = text.get(token.byte_span.0..token.byte_span.1);
                assert!(extracted.is_some(), "Invalid byte span for {token:?}");
                assert_eq!(
                    extracted.unwrap(),
                    token.form,
                    "Span mismatch for: {token:?}"
                );
            }
        }
    }

    #[test]
    fn test_contraction_shared_spans() {
        let patterns = vec![ContractionPattern::new(
            "don't".to_string(),
            vec!["do".to_string(), "n't".to_string()],
        )];
        let tokenizer = UnicodeTokenizer::with_treebank_patterns(&patterns);

        let text = "I don't know";
        let tokens = tokenizer.tokenize(text);

        // Find the split tokens
        let split_tokens: Vec<_> = tokens.iter().filter(|t| t.is_split()).collect();
        assert_eq!(split_tokens.len(), 2);

        // Both should have the same span pointing to "don't"
        assert_eq!(split_tokens[0].byte_span, split_tokens[1].byte_span);

        // Verify span points to original contraction
        let span = split_tokens[0].byte_span;
        assert_eq!(&text[span.0..span.1], "don't");

        // Verify split metadata
        assert_eq!(split_tokens[0].split_index, Some(0));
        assert_eq!(split_tokens[1].split_index, Some(1));
    }

    #[test]
    fn test_contraction_case_preservation() {
        let patterns = vec![ContractionPattern::new(
            "don't".to_string(),
            vec!["do".to_string(), "n't".to_string()],
        )];
        let tokenizer = UnicodeTokenizer::with_treebank_patterns(&patterns);

        let tokens = tokenizer.tokenize("Don't go");

        let split_tokens: Vec<_> = tokens.iter().filter(|t| t.is_split()).collect();
        assert_eq!(split_tokens[0].form, "Do"); // Capital preserved
        assert_eq!(split_tokens[1].form, "n't");
    }

    #[test]
    fn test_sentence_segmentation() {
        let tokenizer = UnicodeTokenizer::new();
        let text = "Hello world. How are you?";
        let sentences = tokenizer.split_sentences(text);

        assert_eq!(sentences.len(), 2);
    }

    #[test]
    fn test_sentence_spans() {
        let tokenizer = UnicodeTokenizer::new();
        let text = "Hello world. How are you?";
        let sentences = tokenizer.split_sentences_with_spans(text);

        // Verify byte spans are valid
        for sent in &sentences {
            let extracted = text.get(sent.byte_span.0..sent.byte_span.1);
            assert!(extracted.is_some(), "Invalid sentence span: {sent:?}");
        }
    }

    #[test]
    fn test_emoji_handling() {
        let tokenizer = UnicodeTokenizer::new();
        let text = "Hello 😀 world";
        let tokens = tokenizer.tokenize(text);

        // Should not panic
        for token in &tokens {
            assert!(token.byte_span.1 <= text.len());
            assert!(token.byte_span.0 <= token.byte_span.1);
        }
    }

    #[test]
    fn test_from_ewt() {
        let ud_path = crate::paths::data_path("data/ud_english-ewt");
        if !ud_path.exists() {
            eprintln!("Skipping: UD English-EWT data not available");
            return;
        }

        let tokenizer = UnicodeTokenizer::from_ewt().expect("Failed to load EWT patterns");

        let tokens = tokenizer.tokenize("I don't know");
        assert!(tokens.len() >= 3);
    }
}
