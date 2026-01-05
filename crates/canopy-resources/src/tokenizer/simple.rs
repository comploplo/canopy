//! Simple tokenizer implementation with contraction handling.
//!
//! Tokenizes text using whitespace and punctuation rules,
//! with contraction splitting learned from UD treebank.

use super::{ContractionPattern, RawToken, Tokenizer, TokenizerResult};
use std::collections::{HashMap, HashSet};

/// A simple rule-based tokenizer with contraction support.
#[derive(Debug)]
pub struct SimpleTokenizer {
    /// Contraction patterns: lowercase form → expanded tokens.
    contractions: HashMap<String, Vec<String>>,
    /// Punctuation characters to separate.
    punctuation: HashSet<char>,
    /// Sentence-ending punctuation.
    sentence_enders: HashSet<char>,
    /// Common abbreviations (to avoid false sentence splits).
    abbreviations: HashSet<String>,
}

impl Default for SimpleTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

impl SimpleTokenizer {
    /// Create a new tokenizer with default rules.
    #[must_use]
    pub fn new() -> Self {
        Self {
            contractions: HashMap::new(),
            punctuation: Self::default_punctuation(),
            sentence_enders: Self::default_sentence_enders(),
            abbreviations: Self::default_abbreviations(),
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

    fn default_sentence_enders() -> HashSet<char> {
        ['.', '!', '?'].into_iter().collect()
    }

    fn default_abbreviations() -> HashSet<String> {
        [
            "Mr.", "Mrs.", "Ms.", "Dr.", "Prof.", "Sr.", "Jr.", "vs.", "etc.", "i.e.", "e.g.",
            "Inc.", "Ltd.", "Corp.", "Co.", "St.", "Ave.", "Blvd.", "Rd.", "Jan.", "Feb.", "Mar.",
            "Apr.", "Jun.", "Jul.", "Aug.", "Sep.", "Sept.", "Oct.", "Nov.", "Dec.", "Fig.", "No.",
            "Vol.", "p.", "pp.", "ed.", "eds.", "trans.", "Rev.", "Gen.", "Col.", "Lt.", "Capt.",
            "Sgt.", "Rep.", "Sen.", "Gov.", "Pres.", "Mt.", "Ft.",
        ]
        .into_iter()
        .map(str::to_lowercase)
        .collect()
    }

    /// Split a word if it matches a contraction pattern.
    fn split_contraction(&self, word: &str) -> Vec<String> {
        let lower = word.to_lowercase();
        if let Some(tokens) = self.contractions.get(&lower) {
            // Preserve original case for first token if word started with uppercase
            if word.chars().next().is_some_and(char::is_uppercase) && !tokens.is_empty() {
                let mut result = tokens.clone();
                if let Some(first) = result.first_mut() {
                    if let Some(c) = first.chars().next() {
                        *first = format!("{}{}", c.to_uppercase(), &first[c.len_utf8()..]);
                    }
                }
                result
            } else {
                tokens.clone()
            }
        } else {
            vec![word.to_string()]
        }
    }

    /// Check if a token is an abbreviation.
    fn is_abbreviation(&self, word: &str) -> bool {
        self.abbreviations.contains(&word.to_lowercase())
    }

    /// Tokenize a single "word" (whitespace-delimited chunk).
    fn tokenize_word(&self, word: &str, start_offset: usize) -> Vec<RawToken> {
        let mut tokens = Vec::new();
        let current_start = start_offset;

        // First, handle leading punctuation
        let mut chars = word.char_indices().peekable();
        let mut word_start = 0;

        while let Some(&(i, c)) = chars.peek() {
            if self.punctuation.contains(&c) && c != '\'' {
                tokens.push(RawToken::new(
                    c.to_string(),
                    (current_start + i, current_start + i + c.len_utf8()),
                ));
                chars.next();
                word_start = i + c.len_utf8();
            } else {
                break;
            }
        }

        // Find the core word (without trailing punctuation)
        let remaining = &word[word_start..];
        let mut word_end = remaining.len();
        let mut trailing_punct = Vec::new();

        for (i, c) in remaining.char_indices().rev() {
            if self.punctuation.contains(&c) && c != '\'' {
                trailing_punct.push((current_start + word_start + i, c));
                word_end = i;
            } else {
                break;
            }
        }

        // Process the core word
        let core_word = &remaining[..word_end];
        if !core_word.is_empty() {
            let core_start = current_start + word_start;
            let expanded = self.split_contraction(core_word);

            if expanded.len() == 1 {
                // No contraction split
                tokens.push(RawToken::new(
                    core_word.to_string(),
                    (core_start, core_start + core_word.len()),
                ));
            } else {
                // Contraction was split - all parts share the parent span
                let parent_span = (core_start, core_start + core_word.len());
                let count = expanded.len();
                for (i, form) in expanded.into_iter().enumerate() {
                    tokens.push(RawToken::split(form, parent_span, i, count));
                }
            }
        }

        // Add trailing punctuation (in original order)
        trailing_punct.reverse();
        for (offset, c) in trailing_punct {
            tokens.push(RawToken::new(
                c.to_string(),
                (offset, offset + c.len_utf8()),
            ));
        }

        tokens
    }

    /// Check if position is likely a sentence boundary.
    fn is_sentence_boundary(&self, text: &str, pos: usize) -> bool {
        // Must have a sentence-ending character
        let chars: Vec<char> = text.chars().collect();
        if pos >= chars.len() {
            return false;
        }

        let c = chars[pos];
        if !self.sentence_enders.contains(&c) {
            return false;
        }

        // Check for abbreviation (look back for word)
        let mut word_start = pos;
        while word_start > 0 && chars[word_start - 1].is_alphabetic() {
            word_start -= 1;
        }
        let potential_abbrev: String = chars[word_start..=pos].iter().collect();
        if self.is_abbreviation(&potential_abbrev) {
            return false;
        }

        // Check if followed by whitespace + capital letter or end of text
        let mut next_pos = pos + 1;

        // Skip any closing quotes/brackets
        while next_pos < chars.len() && matches!(chars[next_pos], '"' | '\'' | ')' | ']') {
            next_pos += 1;
        }

        // Must have whitespace after
        if next_pos >= chars.len() {
            return true; // End of text
        }

        if !chars[next_pos].is_whitespace() {
            return false;
        }

        // Skip whitespace
        while next_pos < chars.len() && chars[next_pos].is_whitespace() {
            next_pos += 1;
        }

        // Must have capital letter or end of text
        if next_pos >= chars.len() {
            return true;
        }

        chars[next_pos].is_uppercase() || chars[next_pos] == '"' || chars[next_pos] == '\''
    }
}

impl Tokenizer for SimpleTokenizer {
    fn tokenize(&self, text: &str) -> Vec<RawToken> {
        let mut tokens = Vec::new();
        let mut offset = 0;

        for segment in text.split_whitespace() {
            // Find the actual position of this segment in the original text
            if let Some(pos) = text[offset..].find(segment) {
                let start = offset + pos;
                let word_tokens = self.tokenize_word(segment, start);
                tokens.extend(word_tokens);
                offset = start + segment.len();
            }
        }

        tokens
    }

    fn split_sentences(&self, text: &str) -> Vec<String> {
        let mut sentences = Vec::new();
        let mut current_start = 0;
        let chars: Vec<char> = text.chars().collect();

        for (i, &c) in chars.iter().enumerate() {
            if self.sentence_enders.contains(&c) && self.is_sentence_boundary(text, i) {
                // Find the end of this sentence (include closing quotes, etc.)
                let mut end = i + 1;
                while end < chars.len() && matches!(chars[end], '"' | '\'' | ')' | ']') {
                    end += 1;
                }

                // Extract sentence
                let byte_start = chars[..current_start]
                    .iter()
                    .map(|c| c.len_utf8())
                    .sum::<usize>();
                let byte_end = chars[..end].iter().map(|c| c.len_utf8()).sum::<usize>();
                let sentence = text[byte_start..byte_end].trim().to_string();

                if !sentence.is_empty() {
                    sentences.push(sentence);
                }

                // Skip whitespace for next sentence
                current_start = end;
                while current_start < chars.len() && chars[current_start].is_whitespace() {
                    current_start += 1;
                }
            }
        }

        // Add remaining text as final sentence
        if current_start < chars.len() {
            let byte_start = chars[..current_start]
                .iter()
                .map(|c| c.len_utf8())
                .sum::<usize>();
            let sentence = text[byte_start..].trim().to_string();
            if !sentence.is_empty() {
                sentences.push(sentence);
            }
        }

        // Handle case where no sentence boundaries were found
        if sentences.is_empty() && !text.trim().is_empty() {
            sentences.push(text.trim().to_string());
        }

        sentences
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_tokenizer_creation() {
        let tokenizer = SimpleTokenizer::new();
        assert!(tokenizer.contractions.is_empty());
        assert!(!tokenizer.punctuation.is_empty());
    }

    #[test]
    fn test_tokenize_simple_sentence() {
        let tokenizer = SimpleTokenizer::new();
        let tokens = tokenizer.tokenize("Hello world");

        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].form, "Hello");
        assert_eq!(tokens[1].form, "world");
    }

    #[test]
    fn test_tokenize_with_punctuation() {
        let tokenizer = SimpleTokenizer::new();
        let tokens = tokenizer.tokenize("Hello, world!");

        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0].form, "Hello");
        assert_eq!(tokens[1].form, ",");
        assert_eq!(tokens[2].form, "world");
        assert_eq!(tokens[3].form, "!");
    }

    #[test]
    fn test_tokenize_with_contractions() {
        let patterns = vec![ContractionPattern::new(
            "don't".to_string(),
            vec!["do".to_string(), "n't".to_string()],
        )];
        let tokenizer = SimpleTokenizer::with_treebank_patterns(&patterns);

        let tokens = tokenizer.tokenize("I don't know");

        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0].form, "I");
        assert_eq!(tokens[1].form, "do");
        assert_eq!(tokens[2].form, "n't");
        assert_eq!(tokens[3].form, "know");
    }

    #[test]
    fn test_contraction_case_preservation() {
        let patterns = vec![ContractionPattern::new(
            "don't".to_string(),
            vec!["do".to_string(), "n't".to_string()],
        )];
        let tokenizer = SimpleTokenizer::with_treebank_patterns(&patterns);

        let tokens = tokenizer.tokenize("Don't go");

        assert_eq!(tokens[0].form, "Do"); // Capital preserved
        assert_eq!(tokens[1].form, "n't");
    }

    #[test]
    fn test_split_sentences_simple() {
        let tokenizer = SimpleTokenizer::new();
        let sentences = tokenizer.split_sentences("Hello world. How are you?");

        assert_eq!(sentences.len(), 2);
        assert_eq!(sentences[0], "Hello world.");
        assert_eq!(sentences[1], "How are you?");
    }

    #[test]
    fn test_split_sentences_abbreviations() {
        let tokenizer = SimpleTokenizer::new();
        let sentences = tokenizer.split_sentences("Dr. Smith went home. He was tired.");

        assert_eq!(sentences.len(), 2);
        assert_eq!(sentences[0], "Dr. Smith went home.");
        assert_eq!(sentences[1], "He was tired.");
    }

    #[test]
    fn test_split_sentences_no_boundary() {
        let tokenizer = SimpleTokenizer::new();
        let sentences = tokenizer.split_sentences("Hello world");

        assert_eq!(sentences.len(), 1);
        assert_eq!(sentences[0], "Hello world");
    }

    #[test]
    fn test_tokenize_quotes() {
        let tokenizer = SimpleTokenizer::new();
        let tokens = tokenizer.tokenize("\"Hello,\" she said.");

        assert!(tokens.len() >= 5);
        assert_eq!(tokens[0].form, "\"");
        assert_eq!(tokens[1].form, "Hello");
    }

    #[test]
    fn test_from_ewt() {
        // This test requires the treebank data
        let ud_path = crate::paths::data_path("data/ud_english-ewt");
        if !ud_path.exists() {
            eprintln!("Skipping: UD English-EWT data not available");
            return;
        }

        let tokenizer = SimpleTokenizer::from_ewt().expect("Failed to load EWT patterns");

        // Should be able to split contractions
        let tokens = tokenizer.tokenize("I don't know");
        assert!(
            tokens.len() >= 3,
            "Should tokenize with contractions: {tokens:?}"
        );
    }

    #[test]
    fn test_token_spans() {
        let tokenizer = SimpleTokenizer::new();
        let text = "Hello world";
        let tokens = tokenizer.tokenize(text);

        assert_eq!(tokens[0].byte_span, (0, 5));
        assert_eq!(tokens[1].byte_span, (6, 11));

        // Verify spans match original text
        assert_eq!(&text[tokens[0].byte_span.0..tokens[0].byte_span.1], "Hello");
        assert_eq!(&text[tokens[1].byte_span.0..tokens[1].byte_span.1], "world");
    }

    #[test]
    fn test_contraction_shared_spans() {
        let patterns = vec![ContractionPattern::new(
            "don't".to_string(),
            vec!["do".to_string(), "n't".to_string()],
        )];
        let tokenizer = SimpleTokenizer::with_treebank_patterns(&patterns);

        let text = "I don't know";
        let tokens = tokenizer.tokenize(text);

        // Find the split tokens
        let split_tokens: Vec<_> = tokens.iter().filter(|t| t.is_split()).collect();
        assert_eq!(split_tokens.len(), 2);

        // Both should have the same span pointing to "don't"
        assert_eq!(split_tokens[0].byte_span, split_tokens[1].byte_span);
        assert_eq!(split_tokens[0].byte_span, (2, 7)); // "don't" starts at byte 2

        // Verify span points to original contraction
        let span = split_tokens[0].byte_span;
        assert_eq!(&text[span.0..span.1], "don't");

        // Verify split metadata
        assert_eq!(split_tokens[0].split_index, Some(0));
        assert_eq!(split_tokens[1].split_index, Some(1));
        assert_eq!(split_tokens[0].split_count, Some(2));
    }
}
