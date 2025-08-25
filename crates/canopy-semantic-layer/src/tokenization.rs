//! Advanced tokenization for semantic analysis
//!
//! This module provides sophisticated tokenization that preserves semantic units
//! and handles linguistic phenomena important for semantic analysis.

use crate::{SemanticError, SemanticResult};
use std::collections::HashMap;

/// Token with position and metadata
#[derive(Debug, Clone)]
pub struct Token {
    /// Original text form
    pub text: String,
    /// Start position in original text
    pub start: usize,
    /// End position in original text
    pub end: usize,
    /// Whether this token is likely a content word
    pub is_content_word: bool,
    /// Whether this token is punctuation
    pub is_punctuation: bool,
}

/// Advanced tokenizer for semantic analysis
pub struct Tokenizer {
    /// Common function words to identify
    function_words: HashMap<String, bool>,
    /// Contractions mappings
    contractions: HashMap<String, Vec<String>>,
}

impl Tokenizer {
    /// Create a new tokenizer
    pub fn new() -> Self {
        let mut function_words = HashMap::new();

        // Common English function words
        let function_word_list = [
            "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with",
            "by", "from", "up", "about", "into", "through", "during", "before", "after", "above",
            "below", "between", "among", "under", "over", "is", "are", "was", "were", "be", "been",
            "being", "have", "has", "had", "do", "does", "did", "will", "would", "could", "should",
            "may", "might", "can", "must", "i", "you", "he", "she", "it", "we", "they", "me",
            "him", "her", "us", "them", "my", "your", "his", "her", "its", "our", "their", "this",
            "that", "these", "those", "some", "any", "all", "each", "every", "no", "not",
        ];

        for word in &function_word_list {
            function_words.insert(word.to_string(), true);
        }

        let mut contractions = HashMap::new();
        // Common contractions
        contractions.insert(
            "don't".to_string(),
            vec!["do".to_string(), "not".to_string()],
        );
        contractions.insert(
            "won't".to_string(),
            vec!["will".to_string(), "not".to_string()],
        );
        contractions.insert(
            "can't".to_string(),
            vec!["can".to_string(), "not".to_string()],
        );
        contractions.insert(
            "isn't".to_string(),
            vec!["is".to_string(), "not".to_string()],
        );
        contractions.insert(
            "aren't".to_string(),
            vec!["are".to_string(), "not".to_string()],
        );
        contractions.insert(
            "wasn't".to_string(),
            vec!["was".to_string(), "not".to_string()],
        );
        contractions.insert(
            "weren't".to_string(),
            vec!["were".to_string(), "not".to_string()],
        );
        contractions.insert(
            "haven't".to_string(),
            vec!["have".to_string(), "not".to_string()],
        );
        contractions.insert(
            "hasn't".to_string(),
            vec!["has".to_string(), "not".to_string()],
        );
        contractions.insert(
            "hadn't".to_string(),
            vec!["had".to_string(), "not".to_string()],
        );
        contractions.insert(
            "shouldn't".to_string(),
            vec!["should".to_string(), "not".to_string()],
        );
        contractions.insert(
            "wouldn't".to_string(),
            vec!["would".to_string(), "not".to_string()],
        );
        contractions.insert(
            "couldn't".to_string(),
            vec!["could".to_string(), "not".to_string()],
        );
        contractions.insert("i'm".to_string(), vec!["i".to_string(), "am".to_string()]);
        contractions.insert(
            "you're".to_string(),
            vec!["you".to_string(), "are".to_string()],
        );
        contractions.insert(
            "we're".to_string(),
            vec!["we".to_string(), "are".to_string()],
        );
        contractions.insert(
            "they're".to_string(),
            vec!["they".to_string(), "are".to_string()],
        );
        contractions.insert(
            "i've".to_string(),
            vec!["i".to_string(), "have".to_string()],
        );
        contractions.insert(
            "you've".to_string(),
            vec!["you".to_string(), "have".to_string()],
        );
        contractions.insert(
            "we've".to_string(),
            vec!["we".to_string(), "have".to_string()],
        );
        contractions.insert(
            "they've".to_string(),
            vec!["they".to_string(), "have".to_string()],
        );

        Self {
            function_words,
            contractions,
        }
    }

    /// Tokenize text into individual tokens with metadata
    pub fn tokenize(&self, text: &str) -> SemanticResult<Vec<Token>> {
        let mut tokens = Vec::new();
        let mut current_pos = 0;

        // Split on whitespace but preserve positions
        for word in text.split_whitespace() {
            let word_start = text[current_pos..].find(word).unwrap() + current_pos;
            let word_end = word_start + word.len();
            current_pos = word_end;

            // Handle contractions first
            let word_lower = word.to_lowercase();
            if let Some(expanded) = self.contractions.get(&word_lower) {
                // Add expanded tokens
                for (i, token_text) in expanded.iter().enumerate() {
                    tokens.push(Token {
                        text: token_text.clone(),
                        start: word_start,
                        end: if i == expanded.len() - 1 {
                            word_end
                        } else {
                            word_start
                        },
                        is_content_word: !self.function_words.contains_key(token_text),
                        is_punctuation: false,
                    });
                }
            } else {
                // Handle punctuation
                let clean_word = word.trim_matches(|c: char| c.is_ascii_punctuation());

                if !clean_word.is_empty() {
                    tokens.push(Token {
                        text: clean_word.to_string(),
                        start: word_start,
                        end: word_end,
                        is_content_word: !self
                            .function_words
                            .contains_key(&clean_word.to_lowercase()),
                        is_punctuation: false,
                    });
                }

                // Add punctuation tokens if present
                for ch in word.chars() {
                    if ch.is_ascii_punctuation() {
                        tokens.push(Token {
                            text: ch.to_string(),
                            start: word_start, // Simplified position
                            end: word_start + 1,
                            is_content_word: false,
                            is_punctuation: true,
                        });
                    }
                }
            }
        }

        if tokens.is_empty() {
            return Err(SemanticError::TokenizationError {
                context: "No tokens found in input text".to_string(),
            });
        }

        Ok(tokens)
    }

    /// Simple tokenize method that returns just strings for compatibility
    pub fn tokenize_simple(&self, text: &str) -> SemanticResult<Vec<String>> {
        let tokens = self.tokenize(text)?;
        Ok(tokens.into_iter().map(|t| t.text).collect())
    }

    /// Segment text into sentences
    pub fn segment_sentences(&self, text: &str) -> SemanticResult<Vec<String>> {
        // Simple sentence segmentation
        let sentences: Vec<String> = text
            .split(&['.', '!', '?'][..])
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if sentences.is_empty() {
            return Err(SemanticError::TokenizationError {
                context: "No sentences found in input text".to_string(),
            });
        }

        Ok(sentences)
    }
}

impl Default for Tokenizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokenization() {
        let tokenizer = Tokenizer::new();
        let tokens = tokenizer.tokenize("John gave Mary a book").unwrap();
        let text_tokens: Vec<&str> = tokens.iter().map(|t| t.text.as_str()).collect();
        assert_eq!(text_tokens, vec!["John", "gave", "Mary", "a", "book"]);
    }

    #[test]
    fn test_punctuation_handling() {
        let tokenizer = Tokenizer::new();
        let tokens = tokenizer.tokenize("Hello, world!").unwrap();
        let content_tokens: Vec<&str> = tokens
            .iter()
            .filter(|t| !t.is_punctuation)
            .map(|t| t.text.as_str())
            .collect();
        assert_eq!(content_tokens, vec!["Hello", "world"]);
    }

    #[test]
    fn test_sentence_segmentation() {
        let tokenizer = Tokenizer::new();
        let sentences = tokenizer
            .segment_sentences("First sentence. Second sentence! Third?")
            .unwrap();
        assert_eq!(
            sentences,
            vec!["First sentence", "Second sentence", "Third"]
        );
    }

    #[test]
    fn test_empty_input() {
        let tokenizer = Tokenizer::new();
        assert!(tokenizer.tokenize("").is_err());
        assert!(tokenizer.tokenize("   ").is_err());
    }

    #[test]
    fn test_tokenize_simple_method() {
        let tokenizer = Tokenizer::new();
        let simple_tokens = tokenizer.tokenize_simple("John loves Mary").unwrap();
        assert_eq!(simple_tokens, vec!["John", "loves", "Mary"]);

        // Test error case for lines 145-146
        assert!(tokenizer.tokenize_simple("   \t\n  ").is_err());
    }

    #[test]
    fn test_default_implementation() {
        let tokenizer = Tokenizer::default();
        let tokens = tokenizer.tokenize("test").unwrap();
        assert_eq!(tokens[0].text, "test");
    }

    #[test]
    fn test_empty_sentence_segmentation() {
        let tokenizer = Tokenizer::new();
        // Test empty input (lines 160-161)
        let result = tokenizer.segment_sentences("");
        assert!(result.is_err());
        if let Err(SemanticError::TokenizationError { context }) = result {
            assert_eq!(context, "No sentences found in input text");
        }

        // Test whitespace only
        assert!(tokenizer.segment_sentences("   ").is_err());

        // Test punctuation only (should also trigger empty sentences)
        assert!(tokenizer.segment_sentences("...").is_err());
    }

    #[test]
    fn test_whitespace_only_tokenization() {
        let tokenizer = Tokenizer::new();
        // This should trigger the empty tokens error path (lines 136-137)
        let result = tokenizer.tokenize("   \t\n  ");
        assert!(result.is_err());
        if let Err(SemanticError::TokenizationError { context }) = result {
            assert_eq!(context, "No tokens found in input text");
        } else {
            panic!("Expected TokenizationError");
        }
    }
}
