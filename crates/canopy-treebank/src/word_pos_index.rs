//! Word to POS tag index built from UD treebank data
//!
//! This module provides a frequency-based lookup from word forms to their
//! most likely POS tags, derived from the 16,600+ sentences in the
//! UD English-EWT treebank.

use crate::conllu_types::UniversalPos;
use crate::parser::ParsedSentence;
use crate::TreebankResult;
use canopy_engine::{CacheableData, EngineError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;
use tracing::info;

/// Index mapping word forms to POS tags with frequency counts
///
/// Built from UD treebank data, this provides frequency-weighted POS tagging
/// for all words seen in the corpus. Particularly useful for closed-class
/// words (pronouns, determiners) that aren't covered by VerbNet/WordNet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordPosIndex {
    /// form (lowercased) → (POS → frequency)
    index: HashMap<String, HashMap<UniversalPos, u32>>,
    /// Total token count used to build this index
    total_tokens: u32,
    /// Number of unique word forms
    unique_forms: u32,
}

impl WordPosIndex {
    /// Create an empty index
    pub fn new() -> Self {
        Self {
            index: HashMap::new(),
            total_tokens: 0,
            unique_forms: 0,
        }
    }

    /// Build index from parsed treebank sentences
    ///
    /// Collects all (form, POS) pairs and counts frequencies.
    /// This allows frequency-based disambiguation for ambiguous words.
    pub fn from_sentences(sentences: &[ParsedSentence]) -> Self {
        let mut index: HashMap<String, HashMap<UniversalPos, u32>> = HashMap::new();
        let mut total_tokens = 0u32;

        for sentence in sentences {
            for token in &sentence.tokens {
                // Lowercase the form for case-insensitive lookup
                let form = token.form.to_lowercase();

                // Convert string POS to enum
                let pos = UniversalPos::from(token.upos.as_str());

                // Skip punctuation in the index (not useful for POS inference)
                if matches!(pos, UniversalPos::PUNCT | UniversalPos::SYM) {
                    continue;
                }

                // Increment frequency
                *index.entry(form).or_default().entry(pos).or_insert(0) += 1;

                total_tokens += 1;
            }
        }

        let unique_forms = index.len() as u32;

        info!(
            "Built word→POS index: {} unique forms, {} total tokens",
            unique_forms, total_tokens
        );

        Self {
            index,
            total_tokens,
            unique_forms,
        }
    }

    /// Get the most frequent POS tag for a word form
    ///
    /// Returns the POS tag that appears most frequently for this word
    /// in the treebank corpus.
    pub fn get_pos(&self, word: &str) -> Option<UniversalPos> {
        self.index
            .get(&word.to_lowercase())
            .and_then(|poses| poses.iter().max_by_key(|(_, &freq)| freq))
            .map(|(pos, _)| *pos)
    }

    /// Get all POS tags with their frequencies for a word
    ///
    /// Useful for understanding ambiguity. For example, "that" can be
    /// DET, PRON, or SCONJ depending on context.
    pub fn get_all_pos(&self, word: &str) -> Option<&HashMap<UniversalPos, u32>> {
        self.index.get(&word.to_lowercase())
    }

    /// Get POS with frequency threshold
    ///
    /// Only returns POS if it appears at least `min_freq` times.
    /// Helps filter out noise from rare usages.
    pub fn get_pos_with_threshold(&self, word: &str, min_freq: u32) -> Option<UniversalPos> {
        self.index
            .get(&word.to_lowercase())
            .and_then(|poses| {
                poses
                    .iter()
                    .filter(|(_, &freq)| freq >= min_freq)
                    .max_by_key(|(_, &freq)| freq)
            })
            .map(|(pos, _)| *pos)
    }

    /// Check if a word is in the index
    pub fn contains(&self, word: &str) -> bool {
        self.index.contains_key(&word.to_lowercase())
    }

    /// Get total frequency of a word across all POS tags
    pub fn word_frequency(&self, word: &str) -> u32 {
        self.index
            .get(&word.to_lowercase())
            .map(|poses| poses.values().sum())
            .unwrap_or(0)
    }

    /// Get statistics about the index
    pub fn stats(&self) -> WordPosIndexStats {
        WordPosIndexStats {
            unique_forms: self.unique_forms,
            total_tokens: self.total_tokens,
            index_entries: self.index.len(),
        }
    }

    /// Serialize index to disk for fast loading
    pub fn save<P: AsRef<Path>>(&self, path: P) -> TreebankResult<()> {
        let path = path.as_ref();
        let file = File::create(path)
            .map_err(|e| EngineError::io(format!("create {}", path.display()), e))?;
        let writer = BufWriter::new(file);

        bincode::serialize_into(writer, self)
            .map_err(|e| EngineError::cache(format!("serialize word_pos_index: {}", e)))?;

        info!("Saved word→POS index to {}", path.display());
        Ok(())
    }

    /// Load index from disk
    pub fn load<P: AsRef<Path>>(path: P) -> TreebankResult<Self> {
        let path = path.as_ref();
        let file =
            File::open(path).map_err(|e| EngineError::io(format!("open {}", path.display()), e))?;
        let reader = BufReader::new(file);

        let index: Self = bincode::deserialize_from(reader)
            .map_err(|e| EngineError::cache(format!("deserialize word_pos_index: {}", e)))?;

        info!(
            "Loaded word→POS index from {}: {} forms",
            path.display(),
            index.unique_forms
        );
        Ok(index)
    }
}

impl Default for WordPosIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl CacheableData for WordPosIndex {
    fn cache_filename() -> &'static str {
        "word_pos_index.bin"
    }

    fn engine_name() -> &'static str {
        "WordPosIndex"
    }
}

/// Statistics about the word→POS index
#[derive(Debug, Clone)]
pub struct WordPosIndexStats {
    /// Number of unique word forms
    pub unique_forms: u32,
    /// Total tokens processed
    pub total_tokens: u32,
    /// Size of the index HashMap
    pub index_entries: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ParsedToken;
    use crate::types::{DependencyFeatures, DependencyRelation};

    fn make_token(form: &str, upos: &str) -> ParsedToken {
        ParsedToken {
            id: 1,
            form: form.to_string(),
            lemma: form.to_lowercase(),
            upos: upos.to_string(),
            xpos: None,
            features: HashMap::new(),
            head: 0,
            deprel: DependencyRelation::Root,
            dependency_features: DependencyFeatures::default(),
            deps: vec![],
        }
    }

    fn make_sentence(tokens: Vec<ParsedToken>) -> ParsedSentence {
        ParsedSentence {
            sent_id: "test".to_string(),
            text: "test".to_string(),
            tokens,
            root_verb: None,
        }
    }

    #[test]
    fn test_build_from_sentences() {
        let sentences = vec![
            make_sentence(vec![make_token("She", "PRON"), make_token("runs", "VERB")]),
            make_sentence(vec![make_token("He", "PRON"), make_token("walks", "VERB")]),
            make_sentence(vec![
                make_token("she", "PRON"), // lowercase duplicate
                make_token("talks", "VERB"),
            ]),
        ];

        let index = WordPosIndex::from_sentences(&sentences);

        // "she" should appear twice as PRON
        assert_eq!(index.get_pos("she"), Some(UniversalPos::PRON));
        assert_eq!(index.get_pos("She"), Some(UniversalPos::PRON)); // case insensitive
        assert_eq!(index.word_frequency("she"), 2);

        // "he" should appear once as PRON
        assert_eq!(index.get_pos("he"), Some(UniversalPos::PRON));
        assert_eq!(index.word_frequency("he"), 1);

        // verbs should be tagged
        assert_eq!(index.get_pos("runs"), Some(UniversalPos::VERB));
    }

    #[test]
    fn test_ambiguous_words() {
        // "that" can be DET, PRON, or SCONJ
        let sentences = vec![
            make_sentence(vec![make_token("that", "DET")]),
            make_sentence(vec![make_token("that", "DET")]),
            make_sentence(vec![make_token("that", "DET")]),
            make_sentence(vec![make_token("that", "PRON")]),
            make_sentence(vec![make_token("that", "SCONJ")]),
        ];

        let index = WordPosIndex::from_sentences(&sentences);

        // Most frequent should be DET (3 occurrences)
        assert_eq!(index.get_pos("that"), Some(UniversalPos::DET));

        // Should have all three in the map
        let all_pos = index.get_all_pos("that").unwrap();
        assert_eq!(all_pos.get(&UniversalPos::DET), Some(&3));
        assert_eq!(all_pos.get(&UniversalPos::PRON), Some(&1));
        assert_eq!(all_pos.get(&UniversalPos::SCONJ), Some(&1));
    }

    #[test]
    fn test_threshold_filtering() {
        let sentences = vec![
            make_sentence(vec![make_token("word", "NOUN")]),
            make_sentence(vec![make_token("word", "NOUN")]),
            make_sentence(vec![make_token("word", "NOUN")]),
            make_sentence(vec![make_token("word", "VERB")]), // rare usage
        ];

        let index = WordPosIndex::from_sentences(&sentences);

        // With threshold 2, should get NOUN (3 occurrences)
        assert_eq!(
            index.get_pos_with_threshold("word", 2),
            Some(UniversalPos::NOUN)
        );

        // VERB only has 1 occurrence, filtered out with threshold 2
        // But NOUN still qualifies
    }

    #[test]
    fn test_punctuation_excluded() {
        let sentences = vec![make_sentence(vec![
            make_token("Hello", "INTJ"),
            make_token("!", "PUNCT"),
            make_token(".", "PUNCT"),
        ])];

        let index = WordPosIndex::from_sentences(&sentences);

        // Punctuation should not be in the index
        assert!(!index.contains("!"));
        assert!(!index.contains("."));

        // But interjection should be
        assert!(index.contains("hello"));
    }

    #[test]
    fn test_pronouns_and_determiners() {
        // Simulate common pronouns and determiners
        let sentences = vec![
            make_sentence(vec![
                make_token("I", "PRON"),
                make_token("see", "VERB"),
                make_token("the", "DET"),
                make_token("cat", "NOUN"),
            ]),
            make_sentence(vec![
                make_token("You", "PRON"),
                make_token("have", "VERB"),
                make_token("a", "DET"),
                make_token("dog", "NOUN"),
            ]),
            make_sentence(vec![
                make_token("They", "PRON"),
                make_token("want", "VERB"),
                make_token("some", "DET"),
                make_token("food", "NOUN"),
            ]),
        ];

        let index = WordPosIndex::from_sentences(&sentences);

        // Pronouns
        assert_eq!(index.get_pos("i"), Some(UniversalPos::PRON));
        assert_eq!(index.get_pos("you"), Some(UniversalPos::PRON));
        assert_eq!(index.get_pos("they"), Some(UniversalPos::PRON));

        // Determiners
        assert_eq!(index.get_pos("the"), Some(UniversalPos::DET));
        assert_eq!(index.get_pos("a"), Some(UniversalPos::DET));
        assert_eq!(index.get_pos("some"), Some(UniversalPos::DET));
    }
}
