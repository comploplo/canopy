//! Word→POS index extracted from UD treebank.
//!
//! Provides fast O(1) lookup for the most likely POS tag of a word
//! based on observed frequencies in the English Web Treebank (EWT).

use super::shared::parse_upos;
use crate::engine::ConlluParser;
use crate::paths::data_path;
use canopy::{CanopyError, UPos};
use std::collections::HashMap;
use std::path::Path;

/// Index mapping words to their POS tag distributions.
///
/// Built from UD English-EWT treebank data (~25K word forms).
#[derive(Debug, Clone)]
pub struct WordPosIndex {
    /// word (lowercase) → {POS → frequency}
    entries: HashMap<String, HashMap<UPos, u32>>,
}

impl WordPosIndex {
    /// Create an empty index.
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Load word→POS statistics from treebank files.
    ///
    /// Scans all CoNLL-U files in the treebank directory and builds
    /// frequency counts for each (word, POS) pair.
    ///
    /// # Errors
    /// This function currently cannot fail but returns `Result` for API consistency.
    pub fn from_treebank() -> Result<Self, CanopyError> {
        let mut index = Self::new();

        // Check both possible treebank locations
        let ud_dir = data_path("data/ud_english-ewt/UD_English-EWT");
        let ud_dir = if ud_dir.exists() {
            ud_dir
        } else {
            let alt = data_path("data/ud_english-ewt");
            if alt.exists() {
                alt
            } else {
                tracing::warn!("UD English-EWT treebank not found, WordPosIndex will be empty");
                return Ok(index);
            }
        };

        let parser = ConlluParser::new();

        // Load from each split
        for split in &["train", "dev", "test"] {
            let file_path = ud_dir.join(format!("en_ewt-ud-{split}.conllu"));
            if file_path.exists() {
                match parser.parse_file(&file_path) {
                    Ok(sentences) => {
                        for sentence in sentences {
                            for token in &sentence.tokens {
                                let word = token.form.to_lowercase();
                                let pos = parse_upos(&token.upos);
                                *index
                                    .entries
                                    .entry(word)
                                    .or_default()
                                    .entry(pos)
                                    .or_insert(0) += 1;
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to parse {}: {}", file_path.display(), e);
                    }
                }
            }
        }

        tracing::info!("WordPosIndex loaded {} unique words", index.entries.len());
        Ok(index)
    }

    /// Load from a specific treebank directory.
    ///
    /// # Errors
    /// This function currently cannot fail but returns `Result` for API consistency.
    pub fn from_treebank_dir(treebank_dir: &Path) -> Result<Self, CanopyError> {
        let mut index = Self::new();

        if !treebank_dir.exists() {
            tracing::warn!("Treebank directory not found: {}", treebank_dir.display());
            return Ok(index);
        }

        let parser = ConlluParser::new();

        for split in &["train", "dev", "test"] {
            let file_path = treebank_dir.join(format!("en_ewt-ud-{split}.conllu"));
            if file_path.exists() {
                if let Ok(sentences) = parser.parse_file(&file_path) {
                    for sentence in sentences {
                        for token in &sentence.tokens {
                            let word = token.form.to_lowercase();
                            let pos = parse_upos(&token.upos);
                            *index
                                .entries
                                .entry(word)
                                .or_default()
                                .entry(pos)
                                .or_insert(0) += 1;
                        }
                    }
                }
            }
        }

        Ok(index)
    }

    /// Get the most likely POS for a word based on treebank frequencies.
    ///
    /// Returns the POS tag that appeared most frequently with this word
    /// in the training data. Returns `None` if word is not in the index.
    #[must_use]
    pub fn get_pos(&self, word: &str) -> Option<UPos> {
        let lower = word.to_lowercase();
        self.entries.get(&lower).and_then(|dist| {
            dist.iter()
                .max_by_key(|(_, count)| *count)
                .map(|(pos, _)| *pos)
        })
    }

    /// Get the POS distribution for a word.
    ///
    /// Returns a map of POS → frequency for all observed tags.
    /// Useful for handling ambiguous words.
    #[must_use]
    pub fn get_pos_distribution(&self, word: &str) -> Option<&HashMap<UPos, u32>> {
        let lower = word.to_lowercase();
        self.entries.get(&lower)
    }

    /// Check if a word exists in the index.
    #[must_use]
    pub fn contains(&self, word: &str) -> bool {
        let lower = word.to_lowercase();
        self.entries.contains_key(&lower)
    }

    /// Get the number of unique words in the index.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the index is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for WordPosIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn treebank_available() -> bool {
        let ud_dir = data_path("data/ud_english-ewt/UD_English-EWT");
        ud_dir.exists() || data_path("data/ud_english-ewt").exists()
    }

    #[test]
    fn test_empty_index() {
        let index = WordPosIndex::new();
        assert!(index.is_empty());
        assert_eq!(index.get_pos("the"), None);
    }

    #[test]
    fn test_from_treebank() {
        if !treebank_available() {
            eprintln!("Skipping: Treebank data not available");
            return;
        }

        let index = WordPosIndex::from_treebank().unwrap();
        assert!(!index.is_empty(), "Index should have entries");

        // Common words should be present
        assert!(index.contains("the"));
        assert!(index.contains("is"));
        assert!(index.contains("gave"));

        // "the" should be DET
        assert_eq!(index.get_pos("the"), Some(UPos::Det));

        // "gave" should be VERB (this is the key irregular verb test!)
        assert_eq!(index.get_pos("gave"), Some(UPos::Verb));
    }

    #[test]
    fn test_case_insensitivity() {
        if !treebank_available() {
            eprintln!("Skipping: Treebank data not available");
            return;
        }

        let index = WordPosIndex::from_treebank().unwrap();

        // Should work regardless of case
        assert_eq!(index.get_pos("The"), index.get_pos("the"));
        assert_eq!(index.get_pos("GAVE"), index.get_pos("gave"));
    }

    #[test]
    fn test_pos_distribution() {
        if !treebank_available() {
            eprintln!("Skipping: Treebank data not available");
            return;
        }

        let index = WordPosIndex::from_treebank().unwrap();

        // Some words are ambiguous (e.g., "run" can be NOUN or VERB)
        if let Some(dist) = index.get_pos_distribution("run") {
            assert!(!dist.is_empty());
        }
    }
}
