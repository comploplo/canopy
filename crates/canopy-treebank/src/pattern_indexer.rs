//! Pattern indexer for building cache data from corpus
//!
//! This module handles building pattern indexes from parsed corpus data,
//! creating frequency-sorted pattern lists for cache population.

use crate::parser::{ConlluParser, ParsedSentence};
use crate::types::{DependencyPattern, DependencyRelation, PatternSource};
use crate::TreebankResult;
use std::collections::HashMap;
use std::path::Path;
use tracing::{debug, info};

/// Pattern indexer for corpus analysis
pub struct PatternIndexer {
    /// Extracted patterns with frequency counts
    patterns: HashMap<String, DependencyPattern>,
    /// Total pattern instances found
    total_instances: u32,
}

impl PatternIndexer {
    /// Create a new pattern indexer
    pub fn new() -> Self {
        Self {
            patterns: HashMap::new(),
            total_instances: 0,
        }
    }

    /// Index patterns from a CoNLL-U corpus file
    pub fn index_from_corpus(&mut self, corpus_path: &Path) -> TreebankResult<()> {
        info!("Indexing patterns from corpus: {:?}", corpus_path);

        let parser = ConlluParser::new(false);
        let sentences = parser.parse_file(corpus_path)?;

        info!(
            "Processing {} sentences for pattern extraction",
            sentences.len()
        );

        for sentence in &sentences {
            self.extract_patterns_from_sentence(sentence);
        }

        info!(
            "Indexed {} unique patterns ({} total instances)",
            self.patterns.len(),
            self.total_instances
        );

        Ok(())
    }

    /// Extract dependency patterns from a single sentence
    fn extract_patterns_from_sentence(&mut self, sentence: &ParsedSentence) {
        for token in &sentence.tokens {
            // Focus on verbs for dependency pattern extraction
            if token.upos == "VERB" {
                // Find all dependents of this verb
                let dependents: Vec<_> = sentence
                    .tokens
                    .iter()
                    .filter(|t| t.head == token.id && t.deprel != DependencyRelation::from("punct"))
                    .map(|t| (t.deprel.clone(), t.upos.clone()))
                    .collect();

                if !dependents.is_empty() {
                    // Create normalized pattern key
                    let pattern_key = self.create_pattern_key(&token.lemma, &dependents);

                    // Update or create pattern
                    self.total_instances += 1;

                    if let Some(existing_pattern) = self.patterns.get_mut(&pattern_key) {
                        existing_pattern.frequency += 1;
                    } else {
                        let pattern = DependencyPattern {
                            verb_lemma: token.lemma.clone(),
                            dependencies: dependents,
                            confidence: 0.8, // Default confidence
                            frequency: 1,
                            source: PatternSource::Indexed,
                        };

                        self.patterns.insert(pattern_key, pattern);
                    }
                }
            }
        }
    }

    /// Create a normalized pattern key
    fn create_pattern_key(
        &self,
        verb_lemma: &str,
        dependencies: &[(DependencyRelation, String)],
    ) -> String {
        // Sort dependencies to normalize pattern key
        let mut dep_strings: Vec<String> = dependencies
            .iter()
            .map(|(rel, pos)| format!("{:?}:{}", rel, pos))
            .collect();
        dep_strings.sort();

        format!("{}|{}", verb_lemma, dep_strings.join(","))
    }

    /// Get patterns sorted by frequency (descending)
    pub fn get_patterns_by_frequency(&self) -> Vec<(String, DependencyPattern)> {
        let mut pattern_vec: Vec<(String, DependencyPattern)> = self
            .patterns
            .iter()
            .map(|(key, pattern)| (key.clone(), pattern.clone()))
            .collect();

        // Sort by frequency (descending)
        pattern_vec.sort_by(|a, b| b.1.frequency.cmp(&a.1.frequency));

        debug!("Sorted {} patterns by frequency", pattern_vec.len());
        pattern_vec
    }

    /// Calculate coverage statistics
    pub fn calculate_coverage(&self, pattern_count: usize) -> f64 {
        let sorted_patterns = self.get_patterns_by_frequency();
        let covered_frequency: u32 = sorted_patterns
            .iter()
            .take(pattern_count)
            .map(|(_, pattern)| pattern.frequency)
            .sum();

        covered_frequency as f64 / self.total_instances as f64
    }

    /// Get total number of unique patterns
    pub fn pattern_count(&self) -> usize {
        self.patterns.len()
    }

    /// Get total pattern instances
    pub fn total_instances(&self) -> u32 {
        self.total_instances
    }
}

impl Default for PatternIndexer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{ParsedSentence, ParsedToken};
    use crate::types::DependencyFeatures;
    use std::collections::HashMap;

    fn create_test_sentence() -> ParsedSentence {
        ParsedSentence {
            sent_id: "test-1".to_string(),
            text: "John saw Mary".to_string(),
            tokens: vec![
                ParsedToken {
                    id: 1,
                    form: "John".to_string(),
                    lemma: "John".to_string(),
                    upos: "NOUN".to_string(),
                    xpos: Some("NNP".to_string()),
                    features: HashMap::new(),
                    head: 2,
                    deprel: DependencyRelation::from("nsubj"),
                    dependency_features: DependencyFeatures::default(),
                    deps: vec![],
                },
                ParsedToken {
                    id: 2,
                    form: "saw".to_string(),
                    lemma: "see".to_string(),
                    upos: "VERB".to_string(),
                    xpos: Some("VBD".to_string()),
                    features: HashMap::new(),
                    head: 0,
                    deprel: DependencyRelation::from("root"),
                    dependency_features: DependencyFeatures::default(),
                    deps: vec![],
                },
                ParsedToken {
                    id: 3,
                    form: "Mary".to_string(),
                    lemma: "Mary".to_string(),
                    upos: "NOUN".to_string(),
                    xpos: Some("NNP".to_string()),
                    features: HashMap::new(),
                    head: 2,
                    deprel: DependencyRelation::from("obj"),
                    dependency_features: DependencyFeatures::default(),
                    deps: vec![],
                },
            ],
            root_verb: Some("see".to_string()),
        }
    }

    #[test]
    fn test_pattern_indexer_creation() {
        let indexer = PatternIndexer::new();
        assert_eq!(indexer.pattern_count(), 0);
        assert_eq!(indexer.total_instances(), 0);
    }

    #[test]
    fn test_pattern_extraction() {
        let mut indexer = PatternIndexer::new();
        let sentence = create_test_sentence();

        indexer.extract_patterns_from_sentence(&sentence);

        assert_eq!(indexer.pattern_count(), 1);
        assert_eq!(indexer.total_instances(), 1);

        let patterns = indexer.get_patterns_by_frequency();
        assert_eq!(patterns.len(), 1);

        let (key, pattern) = &patterns[0];
        assert!(key.starts_with("see|"));
        assert_eq!(pattern.verb_lemma, "see");
        assert_eq!(pattern.frequency, 1);
        assert_eq!(pattern.dependencies.len(), 2); // nsubj and obj
    }

    #[test]
    fn test_coverage_calculation() {
        let mut indexer = PatternIndexer::new();
        let sentence = create_test_sentence();

        indexer.extract_patterns_from_sentence(&sentence);

        // With 1 pattern covering 1 instance, should be 100% coverage
        let coverage = indexer.calculate_coverage(1);
        assert_eq!(coverage, 1.0);

        // With 0 patterns, should be 0% coverage
        let coverage = indexer.calculate_coverage(0);
        assert_eq!(coverage, 0.0);
    }
}
