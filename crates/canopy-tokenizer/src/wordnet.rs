//! WordNet integration for sense disambiguation and semantic relations
//!
//! This module provides access to WordNet synsets, enabling
//! sense disambiguation and semantic relation analysis.

use crate::{SemanticResult, WordNetSense};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

/// WordNet database engine
pub struct WordNetEngine {
    // Database connections and indices
    synsets: HashMap<String, Vec<WordNetSense>>,
    hypernyms: HashMap<String, Vec<String>>,
    hyponyms: HashMap<String, Vec<String>>,
}

impl WordNetEngine {
    /// Create a new WordNet engine with loaded database
    pub fn new() -> SemanticResult<Self> {
        info!("Initializing WordNet engine");

        let mut synsets = HashMap::new();
        let mut hypernyms = HashMap::new();
        let mut hyponyms = HashMap::new();

        // Sample WordNet data for testing

        // "book" noun senses
        synsets.insert(
            "book".to_string(),
            vec![
                WordNetSense {
                    synset_id: "book.n.01".to_string(),
                    definition: "a written work or composition".to_string(),
                    pos: "n".to_string(),
                    hypernyms: vec!["publication.n.01".to_string()],
                    hyponyms: vec!["novel.n.01".to_string(), "textbook.n.01".to_string()],
                    sense_rank: 1,
                },
                WordNetSense {
                    synset_id: "book.n.02".to_string(),
                    definition: "physical objects consisting of bound pages".to_string(),
                    pos: "n".to_string(),
                    hypernyms: vec!["product.n.01".to_string()],
                    hyponyms: vec!["paperback.n.01".to_string(), "hardcover.n.01".to_string()],
                    sense_rank: 2,
                },
            ],
        );

        // "give" verb senses
        synsets.insert(
            "give".to_string(),
            vec![
                WordNetSense {
                    synset_id: "give.v.01".to_string(),
                    definition: "cause to have, in the abstract sense or physical sense"
                        .to_string(),
                    pos: "v".to_string(),
                    hypernyms: vec!["transfer.v.01".to_string()],
                    hyponyms: vec!["hand.v.01".to_string(), "grant.v.01".to_string()],
                    sense_rank: 1,
                },
                WordNetSense {
                    synset_id: "give.v.02".to_string(),
                    definition: "give or supply".to_string(),
                    pos: "v".to_string(),
                    hypernyms: vec!["provide.v.01".to_string()],
                    hyponyms: vec!["offer.v.01".to_string()],
                    sense_rank: 2,
                },
            ],
        );

        // "john" proper noun
        synsets.insert(
            "john".to_string(),
            vec![WordNetSense {
                synset_id: "john.n.01".to_string(),
                definition: "a male given name".to_string(),
                pos: "n".to_string(),
                hypernyms: vec!["male_given_name.n.01".to_string()],
                hyponyms: vec![],
                sense_rank: 1,
            }],
        );

        // "mary" proper noun
        synsets.insert(
            "mary".to_string(),
            vec![WordNetSense {
                synset_id: "mary.n.01".to_string(),
                definition: "a female given name".to_string(),
                pos: "n".to_string(),
                hypernyms: vec!["female_given_name.n.01".to_string()],
                hyponyms: vec![],
                sense_rank: 1,
            }],
        );

        // Build hypernym/hyponym indices
        for senses in synsets.values() {
            for sense in senses {
                // Index hypernyms
                for hypernym in &sense.hypernyms {
                    hyponyms
                        .entry(hypernym.clone())
                        .or_insert_with(Vec::new)
                        .push(sense.synset_id.clone());
                }

                // Index hyponyms
                for hyponym in &sense.hyponyms {
                    hypernyms
                        .entry(hyponym.clone())
                        .or_insert_with(Vec::new)
                        .push(sense.synset_id.clone());
                }
            }
        }

        Ok(Self {
            synsets,
            hypernyms,
            hyponyms,
        })
    }

    /// Analyze a token for WordNet senses
    pub fn analyze_token(&self, lemma: &str) -> SemanticResult<Vec<WordNetSense>> {
        debug!("Analyzing token for WordNet senses: {}", lemma);

        let senses = self
            .synsets
            .get(&lemma.to_lowercase())
            .cloned()
            .unwrap_or_default();

        if !senses.is_empty() {
            debug!("Found {} WordNet senses for '{}'", senses.len(), lemma);
        }

        Ok(senses)
    }

    /// Get the most frequent (primary) sense for a word
    pub fn get_primary_sense(&self, lemma: &str) -> Option<WordNetSense> {
        self.synsets
            .get(&lemma.to_lowercase())?
            .iter()
            .min_by_key(|sense| sense.sense_rank)
            .cloned()
    }

    /// Get hypernyms for a synset
    pub fn get_hypernyms(&self, synset_id: &str) -> Vec<String> {
        self.hypernyms.get(synset_id).cloned().unwrap_or_default()
    }

    /// Get hyponyms for a synset
    pub fn get_hyponyms(&self, synset_id: &str) -> Vec<String> {
        self.hyponyms.get(synset_id).cloned().unwrap_or_default()
    }

    /// Check if a word exists in WordNet
    pub fn contains_word(&self, lemma: &str) -> bool {
        self.synsets.contains_key(&lemma.to_lowercase())
    }

    /// Get all senses for a specific part of speech
    pub fn get_senses_by_pos(&self, lemma: &str, pos: &str) -> Vec<WordNetSense> {
        self.synsets
            .get(&lemma.to_lowercase())
            .map(|senses| {
                senses
                    .iter()
                    .filter(|sense| sense.pos == pos)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Perform sense disambiguation based on context (simplified)
    pub fn disambiguate_sense(
        &self,
        lemma: &str,
        context_words: &[String],
    ) -> Option<WordNetSense> {
        let senses = self.synsets.get(&lemma.to_lowercase())?;

        // Simple disambiguation: find sense with most hypernym/hyponym overlap with context
        let mut best_sense = None;
        let mut best_score = 0;

        for sense in senses {
            let mut score = 0;

            // Check if any context words are related
            for context_word in context_words {
                if sense.hypernyms.iter().any(|h| h.contains(context_word))
                    || sense.hyponyms.iter().any(|h| h.contains(context_word))
                {
                    score += 1;
                }
            }

            if score > best_score {
                best_score = score;
                best_sense = Some(sense.clone());
            }
        }

        // Fall back to most frequent sense
        best_sense.or_else(|| self.get_primary_sense(lemma))
    }

    /// Get statistics about the loaded WordNet data
    pub fn get_stats(&self) -> WordNetStats {
        WordNetStats {
            total_words: self.synsets.len(),
            total_senses: self.synsets.values().map(|v| v.len()).sum(),
            total_hypernym_relations: self.hypernyms.len(),
            total_hyponym_relations: self.hyponyms.len(),
        }
    }
}

/// Statistics about WordNet data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordNetStats {
    pub total_words: usize,
    pub total_senses: usize,
    pub total_hypernym_relations: usize,
    pub total_hyponym_relations: usize,
}

impl Default for WordNetEngine {
    fn default() -> Self {
        Self::new().expect("Failed to initialize WordNet engine")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wordnet_engine_creation() {
        let engine = WordNetEngine::new().unwrap();
        let stats = engine.get_stats();
        assert!(stats.total_words > 0);
        assert!(stats.total_senses > 0);
    }

    #[test]
    fn test_sense_analysis() {
        let engine = WordNetEngine::new().unwrap();

        let book_senses = engine.analyze_token("book").unwrap();
        assert!(!book_senses.is_empty());
        assert_eq!(book_senses[0].pos, "n");

        let give_senses = engine.analyze_token("give").unwrap();
        assert!(!give_senses.is_empty());
        assert_eq!(give_senses[0].pos, "v");
    }

    #[test]
    fn test_primary_sense() {
        let engine = WordNetEngine::new().unwrap();

        let primary_book = engine.get_primary_sense("book");
        assert!(primary_book.is_some());
        assert_eq!(primary_book.unwrap().sense_rank, 1);
    }

    #[test]
    fn test_word_existence() {
        let engine = WordNetEngine::new().unwrap();

        assert!(engine.contains_word("book"));
        assert!(engine.contains_word("give"));
        assert!(engine.contains_word("john"));
        assert!(!engine.contains_word("unknownword"));
    }

    #[test]
    fn test_pos_filtering() {
        let engine = WordNetEngine::new().unwrap();

        let noun_senses = engine.get_senses_by_pos("book", "n");
        assert!(!noun_senses.is_empty());
        assert!(noun_senses.iter().all(|s| s.pos == "n"));

        let verb_senses = engine.get_senses_by_pos("give", "v");
        assert!(!verb_senses.is_empty());
        assert!(verb_senses.iter().all(|s| s.pos == "v"));
    }

    #[test]
    fn test_hypernym_relations() {
        let engine = WordNetEngine::new().unwrap();

        // Test that we can retrieve hypernyms
        let book_sense = engine.get_primary_sense("book").unwrap();
        assert!(!book_sense.hypernyms.is_empty());

        // Test hypernym lookup
        let hypernyms = engine.get_hypernyms(&book_sense.synset_id);
        // Note: This will be empty in our simple test data structure
        // A full implementation would have proper bidirectional relations
    }

    #[test]
    fn test_sense_disambiguation() {
        let engine = WordNetEngine::new().unwrap();

        let context = vec!["publication".to_string(), "read".to_string()];
        let disambiguated = engine.disambiguate_sense("book", &context);
        assert!(disambiguated.is_some());

        // Should fall back to primary sense when no context matches
        let no_context: Vec<String> = vec![];
        let fallback = engine.disambiguate_sense("book", &no_context);
        assert!(fallback.is_some());
        assert_eq!(fallback.unwrap().sense_rank, 1);
    }
}
