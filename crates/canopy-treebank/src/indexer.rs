//! Pattern indexing for treebank dependency patterns
//!
//! This module builds and manages an index of dependency patterns extracted
//! from treebank data, keyed by semantic signatures for efficient lookup.

use crate::parser::{ConlluParser, ParsedSentence};
use crate::signature::{SemanticSignature, SignatureBuilder};
use crate::types::{DependencyPattern, DependencyRelation, PatternSource};
use crate::TreebankResult;
use canopy_engine::EngineError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;
use tracing::{debug, info};

/// Index of dependency patterns keyed by semantic signatures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreebankIndex {
    /// Pattern lookup by semantic signature
    patterns: HashMap<SemanticSignature, DependencyPattern>,
    /// Frequency statistics
    pattern_frequencies: HashMap<String, u32>,
    /// Index metadata
    metadata: IndexMetadata,
}

/// Metadata about the treebank index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexMetadata {
    /// Total sentences processed
    pub total_sentences: usize,
    /// Total patterns extracted
    pub total_patterns: usize,
    /// Number of unique verbs
    pub unique_verbs: usize,
    /// Creation timestamp
    pub created_at: String,
    /// Source files used
    pub source_files: Vec<String>,
}

impl TreebankIndex {
    /// Create a new empty index
    pub fn new() -> Self {
        Self {
            patterns: HashMap::new(),
            pattern_frequencies: HashMap::new(),
            metadata: IndexMetadata {
                total_sentences: 0,
                total_patterns: 0,
                unique_verbs: 0,
                created_at: chrono::Utc::now().to_rfc3339(),
                source_files: Vec::new(),
            },
        }
    }

    /// Load index from file
    pub fn load<P: AsRef<Path>>(path: P) -> TreebankResult<Self> {
        let path = path.as_ref();
        info!("Loading treebank index from {}", path.display());

        let file = File::open(path)
            .map_err(|e| EngineError::io(format!("open index file {}", path.display()), e))?;

        let reader = BufReader::new(file);
        let index: TreebankIndex = bincode::deserialize_from(reader)
            .map_err(|e| EngineError::internal(format!("Failed to deserialize index: {}", e)))?;

        info!(
            "Loaded index with {} patterns from {} sentences",
            index.metadata.total_patterns, index.metadata.total_sentences
        );

        Ok(index)
    }

    /// Save index to file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> TreebankResult<()> {
        let path = path.as_ref();
        info!("Saving treebank index to {}", path.display());

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                EngineError::io(format!("create directory {}", parent.display()), e)
            })?;
        }

        let file = File::create(path)
            .map_err(|e| EngineError::io(format!("create index file {}", path.display()), e))?;

        let writer = BufWriter::new(file);
        bincode::serialize_into(writer, self)
            .map_err(|e| EngineError::internal(format!("Failed to serialize index: {}", e)))?;

        info!("Saved index with {} patterns", self.metadata.total_patterns);
        Ok(())
    }

    /// Get pattern by semantic signature
    pub fn get_pattern(&self, signature: &SemanticSignature) -> Option<&DependencyPattern> {
        self.patterns.get(signature)
    }

    /// Get all patterns for a lemma
    pub fn get_patterns_for_lemma(&self, lemma: &str) -> Vec<&DependencyPattern> {
        self.patterns
            .iter()
            .filter(|(sig, _)| sig.lemma == lemma)
            .map(|(_, pattern)| pattern)
            .collect()
    }

    /// Get most frequent patterns
    pub fn get_top_patterns(&self, limit: usize) -> Vec<&DependencyPattern> {
        let mut patterns: Vec<_> = self.patterns.values().collect();
        patterns.sort_by(|a, b| b.frequency.cmp(&a.frequency));
        patterns.into_iter().take(limit).collect()
    }

    /// Get index statistics
    pub fn get_stats(&self) -> &IndexMetadata {
        &self.metadata
    }

    /// Add pattern to index
    fn add_pattern(&mut self, signature: SemanticSignature, pattern: DependencyPattern) {
        // Update frequency statistics
        *self
            .pattern_frequencies
            .entry(pattern.verb_lemma.clone())
            .or_insert(0) += pattern.frequency;

        // Store pattern
        self.patterns.insert(signature, pattern);
    }

    /// Update metadata after indexing
    fn update_metadata(&mut self, sentences_processed: usize, source_files: Vec<String>) {
        self.metadata.total_sentences = sentences_processed;
        self.metadata.total_patterns = self.patterns.len();
        self.metadata.unique_verbs = self.pattern_frequencies.len();
        self.metadata.source_files = source_files;
    }
}

impl Default for TreebankIndex {
    fn default() -> Self {
        Self::new()
    }
}

/// Pattern indexer for building treebank indices
pub struct PatternIndexer {
    /// CoNLL-U parser
    parser: ConlluParser,
    /// Signature builder
    signature_builder: SignatureBuilder,
    /// Enable verbose logging
    verbose: bool,
    /// Minimum frequency threshold for patterns
    min_frequency: u32,
}

impl PatternIndexer {
    /// Create a new pattern indexer
    pub fn new(verbose: bool, min_frequency: u32) -> Self {
        Self {
            parser: ConlluParser::new(verbose),
            signature_builder: SignatureBuilder::new(verbose),
            verbose,
            min_frequency,
        }
    }

    /// Build index from treebank files
    pub fn build_index<P: AsRef<Path>>(
        &self,
        treebank_files: &[P],
    ) -> TreebankResult<TreebankIndex> {
        let mut index = TreebankIndex::new();
        let mut all_sentences = Vec::new();
        let mut source_files = Vec::new();

        // Parse all treebank files
        for file_path in treebank_files {
            let path = file_path.as_ref();
            info!("Processing treebank file: {}", path.display());

            let sentences = self.parser.parse_file(path)?;
            all_sentences.extend(sentences);
            source_files.push(path.display().to_string());
        }

        info!("Parsed {} total sentences", all_sentences.len());

        // Extract patterns and build signatures
        let patterns = self.extract_and_index_patterns(&all_sentences)?;

        // Add patterns to index
        for (signature, pattern) in patterns {
            // Only include patterns above frequency threshold
            if pattern.frequency >= self.min_frequency {
                index.add_pattern(signature, pattern);
            } else if self.verbose {
                debug!(
                    "Skipping low-frequency pattern for '{}' (freq={})",
                    pattern.verb_lemma, pattern.frequency
                );
            }
        }

        // Update metadata
        index.update_metadata(all_sentences.len(), source_files);

        info!(
            "Built index with {} patterns from {} sentences",
            index.metadata.total_patterns, index.metadata.total_sentences
        );

        Ok(index)
    }

    /// Extract patterns and create semantic signatures
    fn extract_and_index_patterns(
        &self,
        sentences: &[ParsedSentence],
    ) -> TreebankResult<Vec<(SemanticSignature, DependencyPattern)>> {
        let mut pattern_counts: HashMap<String, HashMap<Vec<(DependencyRelation, String)>, u32>> =
            HashMap::new();

        // First pass: count pattern frequencies
        for sentence in sentences {
            if let Some(root_verb) = &sentence.root_verb {
                let dependencies = self.extract_dependencies_from_sentence(sentence);
                if !dependencies.is_empty() {
                    *pattern_counts
                        .entry(root_verb.clone())
                        .or_default()
                        .entry(dependencies)
                        .or_insert(0) += 1;
                }
            }
        }

        let mut indexed_patterns = Vec::new();

        // Second pass: create patterns and signatures
        for (verb_lemma, verb_patterns) in pattern_counts {
            for (dependencies, frequency) in verb_patterns {
                if frequency >= self.min_frequency {
                    // Create dependency pattern
                    let pattern = DependencyPattern::new(
                        verb_lemma.clone(),
                        dependencies,
                        self.calculate_pattern_confidence(frequency),
                        frequency,
                        PatternSource::Indexed,
                    );

                    // Create semantic signature (simplified for treebank-only data)
                    let signature = self.signature_builder.build_simplified(
                        &verb_lemma,
                        Some("VERB"), // Assume verbs for root tokens
                    );

                    if self.verbose {
                        debug!(
                            "Indexed pattern for '{}' with {} dependencies (freq={})",
                            verb_lemma,
                            pattern.dependencies.len(),
                            frequency
                        );
                    }

                    indexed_patterns.push((signature, pattern));
                }
            }
        }

        info!("Created {} indexed patterns", indexed_patterns.len());
        Ok(indexed_patterns)
    }

    /// Extract dependencies from a sentence
    fn extract_dependencies_from_sentence(
        &self,
        sentence: &ParsedSentence,
    ) -> Vec<(DependencyRelation, String)> {
        let mut dependencies = Vec::new();

        // Find root verb token
        let root_verb_token = sentence
            .tokens
            .iter()
            .find(|token| token.is_root() && token.is_verb());

        if let Some(root_token) = root_verb_token {
            // Find all tokens that depend on the root verb
            for token in &sentence.tokens {
                if token.head == root_token.id {
                    // Skip punctuation and certain function words
                    if matches!(token.upos.as_str(), "PUNCT" | "DET" | "AUX") {
                        continue;
                    }

                    // Map dependency relation to argument type
                    let arg_type = match &token.deprel {
                        DependencyRelation::NominalSubject => "agent",
                        DependencyRelation::Object => "patient",
                        DependencyRelation::IndirectObject => "recipient",
                        DependencyRelation::Oblique => "oblique",
                        DependencyRelation::AdverbialModifier => "modifier",
                        DependencyRelation::ClausalComplement => "complement",
                        DependencyRelation::XClausalComplement => "xcomplement",
                        _ => continue, // Skip other relations for now
                    };

                    dependencies.push((token.deprel.clone(), arg_type.to_string()));
                }
            }
        }

        dependencies
    }

    /// Calculate confidence score based on frequency
    fn calculate_pattern_confidence(&self, frequency: u32) -> f32 {
        // Simple confidence calculation based on frequency
        // More frequent patterns get higher confidence
        match frequency {
            1 => 0.3,
            2..=5 => 0.5,
            6..=20 => 0.7,
            21..=100 => 0.8,
            _ => 0.9,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ParsedToken;
    use crate::types::DependencyFeatures;
    use tempfile::TempDir;

    fn create_test_sentence() -> ParsedSentence {
        ParsedSentence {
            sent_id: "test-001".to_string(),
            text: "John runs quickly.".to_string(),
            root_verb: Some("run".to_string()),
            tokens: vec![
                ParsedToken {
                    id: 1,
                    form: "John".to_string(),
                    lemma: "John".to_string(),
                    upos: "PROPN".to_string(),
                    xpos: None,
                    features: HashMap::new(),
                    head: 2,
                    deprel: DependencyRelation::NominalSubject,
                    dependency_features: DependencyFeatures::default(),
                    deps: vec![],
                },
                ParsedToken {
                    id: 2,
                    form: "runs".to_string(),
                    lemma: "run".to_string(),
                    upos: "VERB".to_string(),
                    xpos: None,
                    features: HashMap::new(),
                    head: 0,
                    deprel: DependencyRelation::Other("root".to_string()),
                    dependency_features: DependencyFeatures::default(),
                    deps: vec![],
                },
                ParsedToken {
                    id: 3,
                    form: "quickly".to_string(),
                    lemma: "quickly".to_string(),
                    upos: "ADV".to_string(),
                    xpos: None,
                    features: HashMap::new(),
                    head: 2,
                    deprel: DependencyRelation::AdverbialModifier,
                    dependency_features: DependencyFeatures::default(),
                    deps: vec![],
                },
            ],
        }
    }

    #[test]
    fn test_treebank_index_creation() {
        let index = TreebankIndex::new();
        assert_eq!(index.patterns.len(), 0);
        assert_eq!(index.pattern_frequencies.len(), 0);
        assert_eq!(index.metadata.total_patterns, 0);
    }

    #[test]
    fn test_pattern_indexer_creation() {
        let indexer = PatternIndexer::new(false, 1);
        assert!(!indexer.verbose);
        assert_eq!(indexer.min_frequency, 1);
    }

    #[test]
    fn test_dependency_extraction() {
        let indexer = PatternIndexer::new(false, 1);
        let sentence = create_test_sentence();

        let dependencies = indexer.extract_dependencies_from_sentence(&sentence);

        // Should extract subject and adverbial modifier
        assert!(!dependencies.is_empty());
        assert!(dependencies
            .iter()
            .any(|(rel, _)| matches!(rel, DependencyRelation::NominalSubject)));
        assert!(dependencies
            .iter()
            .any(|(rel, _)| matches!(rel, DependencyRelation::AdverbialModifier)));
    }

    #[test]
    fn test_pattern_confidence_calculation() {
        let indexer = PatternIndexer::new(false, 1);

        assert_eq!(indexer.calculate_pattern_confidence(1), 0.3);
        assert_eq!(indexer.calculate_pattern_confidence(5), 0.5);
        assert_eq!(indexer.calculate_pattern_confidence(20), 0.7);
        assert_eq!(indexer.calculate_pattern_confidence(50), 0.8);
        assert_eq!(indexer.calculate_pattern_confidence(200), 0.9);
    }

    #[test]
    fn test_index_serialization() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = temp_dir.path().join("test_index.bin");

        // Create and populate index
        let mut index = TreebankIndex::new();
        let signature =
            SemanticSignature::simple("test".to_string(), crate::signature::PosCategory::Verb);
        let pattern = DependencyPattern::new(
            "test".to_string(),
            vec![(DependencyRelation::NominalSubject, "agent".to_string())],
            0.8,
            10,
            PatternSource::Indexed,
        );
        index.add_pattern(signature, pattern);

        // Save and reload
        index.save(&index_path).unwrap();
        let loaded_index = TreebankIndex::load(&index_path).unwrap();

        assert_eq!(loaded_index.patterns.len(), 1);
        assert!(loaded_index.get_patterns_for_lemma("test").len() == 1);
    }

    #[test]
    fn test_top_patterns() {
        let mut index = TreebankIndex::new();

        // Add patterns with different frequencies
        for i in 1..=5 {
            let signature = SemanticSignature::simple(
                format!("verb{}", i),
                crate::signature::PosCategory::Verb,
            );
            let pattern = DependencyPattern::new(
                format!("verb{}", i),
                vec![(DependencyRelation::NominalSubject, "agent".to_string())],
                0.8,
                i * 10, // Different frequencies
                PatternSource::Indexed,
            );
            index.add_pattern(signature, pattern);
        }

        let top_patterns = index.get_top_patterns(3);
        assert_eq!(top_patterns.len(), 3);
        // Should be sorted by frequency (highest first)
        assert!(top_patterns[0].frequency >= top_patterns[1].frequency);
        assert!(top_patterns[1].frequency >= top_patterns[2].frequency);
    }

    #[test]
    fn test_patterns_for_lemma() {
        let mut index = TreebankIndex::new();

        // Add multiple patterns for the same lemma
        for i in 1..=3 {
            let signature = SemanticSignature::new(
                "run".to_string(),
                Some(format!("run-class-{}", i)),
                None,
                crate::signature::PosCategory::Verb,
                canopy_engine::LemmaSource::SimpleLemmatizer,
                0.5,
            );
            let pattern = DependencyPattern::new(
                "run".to_string(),
                vec![(DependencyRelation::NominalSubject, "agent".to_string())],
                0.8,
                10,
                PatternSource::Indexed,
            );
            index.add_pattern(signature, pattern);
        }

        let patterns = index.get_patterns_for_lemma("run");
        assert_eq!(patterns.len(), 3);

        let patterns_none = index.get_patterns_for_lemma("nonexistent");
        assert_eq!(patterns_none.len(), 0);
    }
}
