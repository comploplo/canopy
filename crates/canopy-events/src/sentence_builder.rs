//! Sentence analysis builder for Layer 2 event composition
//!
//! This module provides utilities to construct `SentenceAnalysis` from various
//! input sources including Layer 1 semantic results and treebank parsed sentences.

use crate::types::{DependencyArc, SentenceAnalysis, SentenceMetadata};
use canopy_tokenizer::coordinator::Layer1SemanticResult;
use canopy_treebank::parser::ParsedSentence;
use canopy_treebank::types::DependencyRelation;

/// Builder for constructing `SentenceAnalysis` from various sources
#[derive(Debug, Default)]
pub struct SentenceAnalysisBuilder {
    text: String,
    tokens: Vec<Layer1SemanticResult>,
    dependencies: Vec<DependencyArc>,
    metadata: SentenceMetadata,
}

impl SentenceAnalysisBuilder {
    /// Create a new builder with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the sentence text
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }

    /// Set the Layer 1 tokens
    pub fn tokens(mut self, tokens: Vec<Layer1SemanticResult>) -> Self {
        self.tokens = tokens;
        self
    }

    /// Set the dependency arcs
    pub fn dependencies(mut self, deps: Vec<DependencyArc>) -> Self {
        self.dependencies = deps;
        self
    }

    /// Set the sentence metadata
    pub fn metadata(mut self, metadata: SentenceMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// Build the SentenceAnalysis
    pub fn build(self) -> SentenceAnalysis {
        SentenceAnalysis {
            text: self.text,
            tokens: self.tokens,
            dependencies: self.dependencies,
            metadata: self.metadata,
        }
    }

    /// Create a builder from Layer 1 semantic results
    pub fn from_layer1_results(text: String, tokens: Vec<Layer1SemanticResult>) -> Self {
        Self {
            text,
            tokens,
            dependencies: Vec::new(),
            metadata: SentenceMetadata::default(),
        }
    }

    /// Create a builder from a parsed treebank sentence
    ///
    /// This extracts dependency arcs and metadata from the parsed sentence.
    /// Note: Layer 1 tokens must be added separately as they require semantic analysis.
    pub fn from_parsed_sentence(sentence: &ParsedSentence) -> Self {
        let dependencies = extract_dependency_arcs(sentence);
        let metadata = extract_metadata(sentence);

        Self {
            text: sentence.text.clone(),
            tokens: Vec::new(), // Must be added separately via with_tokens()
            dependencies,
            metadata,
        }
    }

    /// Add Layer 1 tokens to a builder created from parsed sentence
    pub fn with_tokens(mut self, tokens: Vec<Layer1SemanticResult>) -> Self {
        self.tokens = tokens;
        self
    }

    /// Create a complete SentenceAnalysis from both Layer 1 results and parsed sentence
    ///
    /// This is the recommended way to create a SentenceAnalysis when you have both
    /// semantic analysis results and treebank dependency structure.
    pub fn from_layer1_and_treebank(
        tokens: Vec<Layer1SemanticResult>,
        sentence: &ParsedSentence,
    ) -> SentenceAnalysis {
        let dependencies = extract_dependency_arcs(sentence);
        let metadata = extract_metadata(sentence);

        SentenceAnalysis {
            text: sentence.text.clone(),
            tokens,
            dependencies,
            metadata,
        }
    }
}

/// Extract dependency arcs from a parsed treebank sentence
///
/// Converts the CoNLL-U token-level dependency information into
/// `DependencyArc` structures suitable for event composition.
pub fn extract_dependency_arcs(sentence: &ParsedSentence) -> Vec<DependencyArc> {
    sentence
        .tokens
        .iter()
        .filter_map(|token| {
            // Skip root tokens (head = 0) and punctuation
            if token.head == 0 || token.deprel == DependencyRelation::Punctuation {
                return None;
            }

            // Convert 1-indexed CoNLL-U IDs to 0-indexed
            let dependent_idx = (token.id - 1) as usize;
            let head_idx = (token.head - 1) as usize;

            Some(DependencyArc::new(
                head_idx,
                dependent_idx,
                token.deprel.clone(),
            ))
        })
        .collect()
}

/// Extract sentence metadata from a parsed treebank sentence
///
/// Analyzes the dependency features to determine sentence properties
/// like passive voice, interrogative mood, etc.
pub fn extract_metadata(sentence: &ParsedSentence) -> SentenceMetadata {
    let is_passive = sentence
        .tokens
        .iter()
        .any(|t| t.dependency_features.is_passive());

    let is_interrogative = sentence.tokens.iter().any(|t| {
        t.features
            .get("Mood")
            .map(|v| v == "Inter")
            .unwrap_or(false)
    });

    let is_negated = sentence.tokens.iter().any(|t| {
        t.deprel == DependencyRelation::AdverbialModifier
            && (t.lemma == "not" || t.lemma == "n't" || t.lemma == "never")
    });

    let is_imperative = sentence
        .tokens
        .iter()
        .any(|t| t.features.get("Mood").map(|v| v == "Imp").unwrap_or(false));

    SentenceMetadata {
        sentence_id: Some(sentence.sent_id.clone()),
        is_passive,
        is_interrogative,
        is_negated,
        is_imperative,
    }
}

/// Create minimal Layer1SemanticResult tokens from parsed tokens
///
/// This is useful for testing when you don't have real semantic analysis.
/// The resulting tokens will have lemmas and POS tags but no VerbNet/FrameNet data.
pub fn layer1_tokens_from_parsed(sentence: &ParsedSentence) -> Vec<Layer1SemanticResult> {
    use canopy_core::UPos;

    sentence
        .tokens
        .iter()
        .map(|token| {
            let pos = match token.upos.as_str() {
                "NOUN" => Some(UPos::Noun),
                "VERB" => Some(UPos::Verb),
                "ADJ" => Some(UPos::Adj),
                "ADV" => Some(UPos::Adv),
                "ADP" => Some(UPos::Adp),
                "AUX" => Some(UPos::Aux),
                "CCONJ" => Some(UPos::Cconj),
                "DET" => Some(UPos::Det),
                "INTJ" => Some(UPos::Intj),
                "NUM" => Some(UPos::Num),
                "PART" => Some(UPos::Part),
                "PRON" => Some(UPos::Pron),
                "PROPN" => Some(UPos::Propn),
                "PUNCT" => Some(UPos::Punct),
                "SCONJ" => Some(UPos::Sconj),
                "SYM" => Some(UPos::Sym),
                "X" => Some(UPos::X),
                _ => None,
            };

            Layer1SemanticResult {
                original_word: token.form.clone(),
                lemma: token.lemma.clone(),
                pos,
                lemmatization_confidence: Some(1.0), // Treebank has gold lemmas
                verbnet: None,
                framenet: None,
                wordnet: None,
                lexicon: None,
                treebank: None,
                confidence: 0.5, // Low confidence without semantic engines
                sources: vec!["treebank".to_string()],
                errors: Vec::new(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use canopy_treebank::parser::ParsedToken;
    use canopy_treebank::types::DependencyFeatures;
    use std::collections::HashMap;

    fn create_test_sentence() -> ParsedSentence {
        ParsedSentence {
            sent_id: "test-001".to_string(),
            text: "John gave Mary a book.".to_string(),
            root_verb: Some("give".to_string()),
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
                    form: "gave".to_string(),
                    lemma: "give".to_string(),
                    upos: "VERB".to_string(),
                    xpos: None,
                    features: HashMap::new(),
                    head: 0,
                    deprel: DependencyRelation::Root,
                    dependency_features: DependencyFeatures::default(),
                    deps: vec![],
                },
                ParsedToken {
                    id: 3,
                    form: "Mary".to_string(),
                    lemma: "Mary".to_string(),
                    upos: "PROPN".to_string(),
                    xpos: None,
                    features: HashMap::new(),
                    head: 2,
                    deprel: DependencyRelation::IndirectObject,
                    dependency_features: DependencyFeatures::default(),
                    deps: vec![],
                },
                ParsedToken {
                    id: 4,
                    form: "a".to_string(),
                    lemma: "a".to_string(),
                    upos: "DET".to_string(),
                    xpos: None,
                    features: HashMap::new(),
                    head: 5,
                    deprel: DependencyRelation::Determiner,
                    dependency_features: DependencyFeatures::default(),
                    deps: vec![],
                },
                ParsedToken {
                    id: 5,
                    form: "book".to_string(),
                    lemma: "book".to_string(),
                    upos: "NOUN".to_string(),
                    xpos: None,
                    features: HashMap::new(),
                    head: 2,
                    deprel: DependencyRelation::Object,
                    dependency_features: DependencyFeatures::default(),
                    deps: vec![],
                },
                ParsedToken {
                    id: 6,
                    form: ".".to_string(),
                    lemma: ".".to_string(),
                    upos: "PUNCT".to_string(),
                    xpos: None,
                    features: HashMap::new(),
                    head: 2,
                    deprel: DependencyRelation::Punctuation,
                    dependency_features: DependencyFeatures::default(),
                    deps: vec![],
                },
            ],
        }
    }

    #[test]
    fn test_extract_dependency_arcs() {
        let sentence = create_test_sentence();
        let arcs = extract_dependency_arcs(&sentence);

        // Should have 4 arcs (excluding root and punctuation)
        assert_eq!(arcs.len(), 4);

        // Check nsubj: John -> gave (head=1, dependent=0)
        let nsubj_arc = arcs.iter().find(|a| a.dependent_idx == 0).unwrap();
        assert_eq!(nsubj_arc.head_idx, 1);
        assert_eq!(nsubj_arc.relation, DependencyRelation::NominalSubject);

        // Check iobj: Mary -> gave (head=1, dependent=2)
        let iobj_arc = arcs.iter().find(|a| a.dependent_idx == 2).unwrap();
        assert_eq!(iobj_arc.head_idx, 1);
        assert_eq!(iobj_arc.relation, DependencyRelation::IndirectObject);

        // Check obj: book -> gave (head=1, dependent=4)
        let obj_arc = arcs.iter().find(|a| a.dependent_idx == 4).unwrap();
        assert_eq!(obj_arc.head_idx, 1);
        assert_eq!(obj_arc.relation, DependencyRelation::Object);
    }

    #[test]
    fn test_extract_metadata() {
        let sentence = create_test_sentence();
        let metadata = extract_metadata(&sentence);

        assert_eq!(metadata.sentence_id, Some("test-001".to_string()));
        assert!(!metadata.is_passive);
        assert!(!metadata.is_interrogative);
        assert!(!metadata.is_negated);
        assert!(!metadata.is_imperative);
    }

    #[test]
    fn test_passive_metadata_extraction() {
        use canopy_treebank::types::{DependencyFeatureType, VoiceFeature};

        let mut sentence = create_test_sentence();
        sentence.text = "The book was given.".to_string();

        // Mark token as passive
        sentence.tokens[0].dependency_features = DependencyFeatures {
            features: vec![DependencyFeatureType::Voice(VoiceFeature::Pass)],
        };

        let metadata = extract_metadata(&sentence);
        assert!(metadata.is_passive);
    }

    #[test]
    fn test_layer1_tokens_from_parsed() {
        let sentence = create_test_sentence();
        let tokens = layer1_tokens_from_parsed(&sentence);

        assert_eq!(tokens.len(), 6);
        assert_eq!(tokens[0].original_word, "John");
        assert_eq!(tokens[0].lemma, "John");
        assert_eq!(tokens[0].pos, Some(canopy_core::UPos::Propn));

        assert_eq!(tokens[1].original_word, "gave");
        assert_eq!(tokens[1].lemma, "give");
        assert_eq!(tokens[1].pos, Some(canopy_core::UPos::Verb));
    }

    #[test]
    fn test_sentence_analysis_builder() {
        let sentence = create_test_sentence();
        let tokens = layer1_tokens_from_parsed(&sentence);

        let analysis = SentenceAnalysisBuilder::from_layer1_and_treebank(tokens, &sentence);

        assert_eq!(analysis.text, "John gave Mary a book.");
        assert_eq!(analysis.tokens.len(), 6);
        assert_eq!(analysis.dependencies.len(), 4);
        assert_eq!(analysis.metadata.sentence_id, Some("test-001".to_string()));
    }

    #[test]
    fn test_builder_fluent_api() {
        let analysis = SentenceAnalysisBuilder::new()
            .text("Test sentence")
            .tokens(vec![])
            .dependencies(vec![DependencyArc::new(
                0,
                1,
                DependencyRelation::NominalSubject,
            )])
            .metadata(SentenceMetadata {
                sentence_id: Some("test".to_string()),
                is_passive: true,
                ..Default::default()
            })
            .build();

        assert_eq!(analysis.text, "Test sentence");
        assert_eq!(analysis.dependencies.len(), 1);
        assert!(analysis.metadata.is_passive);
    }
}
