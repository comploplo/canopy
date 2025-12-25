//! Comprehensive CoNLL-U data types following UD v2 specification
//!
//! This module provides complete type coverage for Universal Dependencies
//! CoNLL-U format parsing with all features, morphology, and enhanced dependencies.

use crate::types::{DependencyFeatures, DependencyRelation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Universal POS tags (UPOS) as defined in UD v2
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum UniversalPos {
    /// Adjective
    ADJ,
    /// Adposition
    ADP,
    /// Adverb
    ADV,
    /// Auxiliary
    AUX,
    /// Coordinating conjunction
    CCONJ,
    /// Determiner
    DET,
    /// Interjection
    INTJ,
    /// Noun
    NOUN,
    /// Numeral
    NUM,
    /// Particle
    PART,
    /// Pronoun
    PRON,
    /// Proper noun
    PROPN,
    /// Punctuation
    PUNCT,
    /// Subordinating conjunction
    SCONJ,
    /// Symbol
    SYM,
    /// Verb
    VERB,
    /// Other (for unknown/non-standard tags)
    X,
}

impl From<&str> for UniversalPos {
    fn from(s: &str) -> Self {
        match s {
            "ADJ" => Self::ADJ,
            "ADP" => Self::ADP,
            "ADV" => Self::ADV,
            "AUX" => Self::AUX,
            "CCONJ" => Self::CCONJ,
            "DET" => Self::DET,
            "INTJ" => Self::INTJ,
            "NOUN" => Self::NOUN,
            "NUM" => Self::NUM,
            "PART" => Self::PART,
            "PRON" => Self::PRON,
            "PROPN" => Self::PROPN,
            "PUNCT" => Self::PUNCT,
            "SCONJ" => Self::SCONJ,
            "SYM" => Self::SYM,
            "VERB" => Self::VERB,
            _ => Self::X,
        }
    }
}

/// Morphological features following UD guidelines
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct MorphologicalFeatures {
    /// Animacy
    pub animacy: Option<String>,
    /// Aspect
    pub aspect: Option<String>,
    /// Case
    pub case: Option<String>,
    /// Definiteness
    pub definite: Option<String>,
    /// Degree
    pub degree: Option<String>,
    /// Gender
    pub gender: Option<String>,
    /// Mood
    pub mood: Option<String>,
    /// Number
    pub number: Option<String>,
    /// Person
    pub person: Option<String>,
    /// Polarity
    pub polarity: Option<String>,
    /// Pronoun type
    pub pron_type: Option<String>,
    /// Tense
    pub tense: Option<String>,
    /// Verb form
    pub verb_form: Option<String>,
    /// Voice
    pub voice: Option<String>,
    /// Other features not covered above
    pub other: HashMap<String, String>,
}

impl MorphologicalFeatures {
    /// Parse features from CoNLL-U FEATS field
    pub fn parse(feats_str: &str) -> Self {
        let mut features = Self::default();

        if feats_str == "_" || feats_str.is_empty() {
            return features;
        }

        for feat in feats_str.split('|') {
            if let Some((key, value)) = feat.split_once('=') {
                match key {
                    "Animacy" => features.animacy = Some(value.to_string()),
                    "Aspect" => features.aspect = Some(value.to_string()),
                    "Case" => features.case = Some(value.to_string()),
                    "Definite" => features.definite = Some(value.to_string()),
                    "Degree" => features.degree = Some(value.to_string()),
                    "Gender" => features.gender = Some(value.to_string()),
                    "Mood" => features.mood = Some(value.to_string()),
                    "Number" => features.number = Some(value.to_string()),
                    "Person" => features.person = Some(value.to_string()),
                    "Polarity" => features.polarity = Some(value.to_string()),
                    "PronType" => features.pron_type = Some(value.to_string()),
                    "Tense" => features.tense = Some(value.to_string()),
                    "VerbForm" => features.verb_form = Some(value.to_string()),
                    "Voice" => features.voice = Some(value.to_string()),
                    _ => {
                        features.other.insert(key.to_string(), value.to_string());
                    }
                }
            }
        }

        features
    }

    /// Create from HashMap (for conversion from ParsedToken)
    pub fn from_hashmap(map: &HashMap<String, String>) -> Self {
        let mut features = Self::default();

        for (key, value) in map {
            match key.as_str() {
                "Animacy" => features.animacy = Some(value.clone()),
                "Aspect" => features.aspect = Some(value.clone()),
                "Case" => features.case = Some(value.clone()),
                "Definite" => features.definite = Some(value.clone()),
                "Degree" => features.degree = Some(value.clone()),
                "Gender" => features.gender = Some(value.clone()),
                "Mood" => features.mood = Some(value.clone()),
                "Number" => features.number = Some(value.clone()),
                "Person" => features.person = Some(value.clone()),
                "Polarity" => features.polarity = Some(value.clone()),
                "PronType" => features.pron_type = Some(value.clone()),
                "Tense" => features.tense = Some(value.clone()),
                "VerbForm" => features.verb_form = Some(value.clone()),
                "Voice" => features.voice = Some(value.clone()),
                _ => {
                    features.other.insert(key.clone(), value.clone());
                }
            }
        }

        features
    }
}

/// Enhanced dependency relation (for DEPS field)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EnhancedDependency {
    /// Head token ID
    pub head: u32,
    /// Dependency relation
    pub relation: DependencyRelation,
}

/// Miscellaneous attributes (MISC field)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct MiscAttributes {
    /// SpaceAfter=No indicates no space after token
    pub space_after: Option<bool>,
    /// Start and end character positions
    pub start_char: Option<u32>,
    pub end_char: Option<u32>,
    /// Token ID in original text
    pub token_id: Option<String>,
    /// Other miscellaneous attributes
    pub other: HashMap<String, String>,
}

impl MiscAttributes {
    /// Parse MISC field
    pub fn parse(misc_str: &str) -> Self {
        let mut misc = Self::default();

        if misc_str == "_" || misc_str.is_empty() {
            return misc;
        }

        for attr in misc_str.split('|') {
            if let Some((key, value)) = attr.split_once('=') {
                match key {
                    "SpaceAfter" => misc.space_after = Some(value == "No"),
                    "StartChar" => misc.start_char = value.parse().ok(),
                    "EndChar" => misc.end_char = value.parse().ok(),
                    "TokenId" => misc.token_id = Some(value.to_string()),
                    _ => {
                        misc.other.insert(key.to_string(), value.to_string());
                    }
                }
            } else {
                // Handle key-only attributes
                match attr {
                    "SpaceAfter=No" => misc.space_after = Some(false),
                    _ => {
                        misc.other.insert(attr.to_string(), "true".to_string());
                    }
                }
            }
        }

        misc
    }
}

/// Complete CoNLL-U token with all 10 fields plus extracted features
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConlluToken {
    /// Token ID (1-based indexing)
    pub id: u32,
    /// Word form or punctuation symbol
    pub form: String,
    /// Lemma
    pub lemma: String,
    /// Universal part-of-speech tag
    pub upos: UniversalPos,
    /// Language-specific part-of-speech tag
    pub xpos: Option<String>,
    /// Morphological features
    pub features: MorphologicalFeatures,
    /// Head of the current word (0 for root)
    pub head: u32,
    /// Universal dependency relation to the head
    pub deprel: DependencyRelation,
    /// Enhanced dependency graph
    pub enhanced_deps: Vec<EnhancedDependency>,
    /// Miscellaneous information
    pub misc: MiscAttributes,
    /// Extracted features from dependency relation subtypes
    pub dependency_features: DependencyFeatures,
}

/// CoNLL-U sentence with complete metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConlluSentence {
    /// Sentence ID
    pub sent_id: String,
    /// Document ID
    pub newdoc_id: Option<String>,
    /// Paragraph ID
    pub newpar_id: Option<String>,
    /// Original text
    pub text: String,
    /// Tokens in the sentence
    pub tokens: Vec<ConlluToken>,
    /// Additional sentence-level metadata
    pub metadata: HashMap<String, String>,
}

impl ConlluSentence {
    /// Get the root token of the sentence
    pub fn root_token(&self) -> Option<&ConlluToken> {
        self.tokens.iter().find(|t| t.head == 0)
    }

    /// Get all tokens with a specific dependency relation
    pub fn tokens_with_relation(&self, rel: &DependencyRelation) -> Vec<&ConlluToken> {
        self.tokens.iter().filter(|t| &t.deprel == rel).collect()
    }

    /// Get main verb (root VERB token)
    pub fn main_verb(&self) -> Option<&ConlluToken> {
        self.tokens
            .iter()
            .find(|t| t.head == 0 && matches!(t.upos, UniversalPos::VERB | UniversalPos::AUX))
    }

    /// Get all verbs in the sentence
    pub fn verbs(&self) -> Vec<&ConlluToken> {
        self.tokens
            .iter()
            .filter(|t| matches!(t.upos, UniversalPos::VERB | UniversalPos::AUX))
            .collect()
    }

    /// Create dependency pattern key for this sentence (flat structure)
    pub fn create_pattern_key(&self) -> Option<String> {
        if let Some(main_verb) = self.main_verb() {
            let mut deps: Vec<String> = self
                .tokens
                .iter()
                .filter(|t| t.head == main_verb.id)
                .map(|t| format!("{:?}:{:?}", t.deprel, t.upos))
                .collect();
            deps.sort();
            Some(format!("{}|{}", main_verb.lemma, deps.join(",")))
        } else {
            None
        }
    }

    /// Build complete dependency tree from root
    pub fn build_dependency_tree(&self) -> Option<DependencyTree> {
        self.root_token()
            .map(|root| self.build_tree_recursive(root))
    }

    /// Build dependency tree recursively from a given token
    fn build_tree_recursive(&self, token: &ConlluToken) -> DependencyTree {
        let children: Vec<DependencyTree> = self
            .tokens
            .iter()
            .filter(|t| t.head == token.id)
            .map(|t| self.build_tree_recursive(t))
            .collect();

        DependencyTree {
            token: token.clone(),
            children,
        }
    }

    /// Get all tokens that depend on a specific token
    pub fn get_dependents(&self, token_id: u32) -> Vec<&ConlluToken> {
        self.tokens.iter().filter(|t| t.head == token_id).collect()
    }

    /// Create enhanced pattern key that includes nested structure
    pub fn create_hierarchical_pattern_key(&self) -> Option<String> {
        self.build_dependency_tree()
            .map(|tree| self.tree_to_pattern(&tree))
    }

    /// Convert dependency tree to pattern string
    #[allow(clippy::only_used_in_recursion)]
    fn tree_to_pattern(&self, tree: &DependencyTree) -> String {
        let token = &tree.token;

        if tree.children.is_empty() {
            format!("{}:{:?}", token.lemma, token.upos)
        } else {
            let mut child_patterns: Vec<String> = tree
                .children
                .iter()
                .map(|child| format!("{:?}({})", child.token.deprel, self.tree_to_pattern(child)))
                .collect();
            child_patterns.sort();
            format!("{}[{}]", token.lemma, child_patterns.join(","))
        }
    }
}

/// Recursive dependency tree structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyTree {
    /// The token at this node
    pub token: ConlluToken,
    /// Child dependencies
    pub children: Vec<DependencyTree>,
}

impl DependencyTree {
    /// Get the depth of this tree
    pub fn depth(&self) -> usize {
        if self.children.is_empty() {
            1
        } else {
            1 + self.children.iter().map(|c| c.depth()).max().unwrap_or(0)
        }
    }

    /// Count total nodes in tree
    pub fn node_count(&self) -> usize {
        1 + self.children.iter().map(|c| c.node_count()).sum::<usize>()
    }

    /// Find all subtrees with a specific POS tag
    pub fn find_by_pos(&self, pos: &UniversalPos) -> Vec<&DependencyTree> {
        let mut results = Vec::new();
        if self.token.upos == *pos {
            results.push(self);
        }
        for child in &self.children {
            results.extend(child.find_by_pos(pos));
        }
        results
    }

    /// Find all subtrees with a specific dependency relation
    pub fn find_by_relation(&self, relation: &DependencyRelation) -> Vec<&DependencyTree> {
        let mut results = Vec::new();
        if self.token.deprel == *relation {
            results.push(self);
        }
        for child in &self.children {
            results.extend(child.find_by_relation(relation));
        }
        results
    }
}

/// Statistics for CoNLL-U corpus analysis
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConlluCorpusStats {
    /// Total sentences
    pub sentences: u32,
    /// Total tokens
    pub tokens: u32,
    /// UPOS tag frequency
    pub upos_freq: HashMap<String, u32>,
    /// Dependency relation frequency
    pub deprel_freq: HashMap<String, u32>,
    /// Lemma frequency
    pub lemma_freq: HashMap<String, u32>,
    /// Pattern frequency
    pub pattern_freq: HashMap<String, u32>,
}

impl ConlluCorpusStats {
    /// Add sentence to statistics
    pub fn add_sentence(&mut self, sentence: &ConlluSentence) {
        self.sentences += 1;
        self.tokens += sentence.tokens.len() as u32;

        for token in &sentence.tokens {
            // Count UPOS tags
            *self
                .upos_freq
                .entry(format!("{:?}", token.upos))
                .or_insert(0) += 1;

            // Count dependency relations
            *self
                .deprel_freq
                .entry(format!("{:?}", token.deprel))
                .or_insert(0) += 1;

            // Count lemmas
            *self.lemma_freq.entry(token.lemma.clone()).or_insert(0) += 1;
        }

        // Count sentence pattern
        if let Some(pattern_key) = sentence.create_pattern_key() {
            *self.pattern_freq.entry(pattern_key).or_insert(0) += 1;
        }
    }

    /// Get most frequent items
    pub fn top_patterns(&self, n: usize) -> Vec<(String, u32)> {
        let mut patterns: Vec<_> = self.pattern_freq.iter().collect();
        patterns.sort_by(|a, b| b.1.cmp(a.1));
        patterns
            .into_iter()
            .take(n)
            .map(|(k, v)| (k.clone(), *v))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{DependencyFeatures, DependencyRelation};
    use std::collections::HashMap;

    fn create_test_token(
        id: u32,
        form: &str,
        lemma: &str,
        upos: UniversalPos,
        head: u32,
        deprel: DependencyRelation,
    ) -> ConlluToken {
        ConlluToken {
            id,
            form: form.to_string(),
            lemma: lemma.to_string(),
            upos,
            xpos: None,
            features: MorphologicalFeatures::default(),
            head,
            deprel,
            enhanced_deps: vec![],
            misc: MiscAttributes::default(),
            dependency_features: DependencyFeatures::default(),
        }
    }

    #[test]
    fn test_dependency_tree_building() {
        // Create a simple sentence: "John runs quickly"
        let tokens = vec![
            create_test_token(
                1,
                "John",
                "John",
                UniversalPos::PROPN,
                2,
                DependencyRelation::NominalSubject,
            ),
            create_test_token(
                2,
                "runs",
                "run",
                UniversalPos::VERB,
                0,
                DependencyRelation::Root,
            ),
            create_test_token(
                3,
                "quickly",
                "quickly",
                UniversalPos::ADV,
                2,
                DependencyRelation::AdverbialModifier,
            ),
        ];

        let sentence = ConlluSentence {
            sent_id: "test-tree".to_string(),
            newdoc_id: None,
            newpar_id: None,
            text: "John runs quickly.".to_string(),
            tokens,
            metadata: HashMap::new(),
        };

        // Build dependency tree
        let tree = sentence.build_dependency_tree().unwrap();

        // Root should be "runs"
        assert_eq!(tree.token.lemma, "run");
        assert_eq!(tree.token.upos, UniversalPos::VERB);

        // Should have 2 children
        assert_eq!(tree.children.len(), 2);

        // Find John and quickly as children
        let john = tree
            .children
            .iter()
            .find(|c| c.token.lemma == "John")
            .unwrap();
        let quickly = tree
            .children
            .iter()
            .find(|c| c.token.lemma == "quickly")
            .unwrap();

        assert_eq!(john.token.deprel, DependencyRelation::NominalSubject);
        assert_eq!(quickly.token.deprel, DependencyRelation::AdverbialModifier);

        // Check tree depth and node count
        assert_eq!(tree.depth(), 2);
        assert_eq!(tree.node_count(), 3);
    }

    #[test]
    fn test_hierarchical_pattern_key() {
        // Create a nested sentence structure
        let tokens = vec![
            create_test_token(
                1,
                "John",
                "John",
                UniversalPos::PROPN,
                2,
                DependencyRelation::NominalSubject,
            ),
            create_test_token(
                2,
                "runs",
                "run",
                UniversalPos::VERB,
                0,
                DependencyRelation::Root,
            ),
            create_test_token(
                3,
                "quickly",
                "quickly",
                UniversalPos::ADV,
                2,
                DependencyRelation::AdverbialModifier,
            ),
        ];

        let sentence = ConlluSentence {
            sent_id: "test-hierarchical".to_string(),
            newdoc_id: None,
            newpar_id: None,
            text: "John runs quickly.".to_string(),
            tokens,
            metadata: HashMap::new(),
        };

        // Get hierarchical pattern key
        let pattern = sentence.create_hierarchical_pattern_key().unwrap();

        // Should include nested structure
        assert!(pattern.contains("run["));
        assert!(pattern.contains("NominalSubject"));
        assert!(pattern.contains("AdverbialModifier"));
    }

    #[test]
    fn test_dependency_tree_search() {
        let tokens = vec![
            create_test_token(
                1,
                "The",
                "the",
                UniversalPos::DET,
                2,
                DependencyRelation::Determiner,
            ),
            create_test_token(
                2,
                "cat",
                "cat",
                UniversalPos::NOUN,
                3,
                DependencyRelation::NominalSubject,
            ),
            create_test_token(
                3,
                "runs",
                "run",
                UniversalPos::VERB,
                0,
                DependencyRelation::Root,
            ),
            create_test_token(
                4,
                "quickly",
                "quickly",
                UniversalPos::ADV,
                3,
                DependencyRelation::AdverbialModifier,
            ),
        ];

        let sentence = ConlluSentence {
            sent_id: "test-search".to_string(),
            newdoc_id: None,
            newpar_id: None,
            text: "The cat runs quickly.".to_string(),
            tokens,
            metadata: HashMap::new(),
        };

        let tree = sentence.build_dependency_tree().unwrap();

        // Find all nouns
        let nouns = tree.find_by_pos(&UniversalPos::NOUN);
        assert_eq!(nouns.len(), 1);
        assert_eq!(nouns[0].token.lemma, "cat");

        // Find all determiners
        let dets = tree.find_by_pos(&UniversalPos::DET);
        assert_eq!(dets.len(), 1);
        assert_eq!(dets[0].token.lemma, "the");

        // Find all adverbial modifiers
        let advmods = tree.find_by_relation(&DependencyRelation::AdverbialModifier);
        assert_eq!(advmods.len(), 1);
        assert_eq!(advmods[0].token.lemma, "quickly");
    }

    #[test]
    fn test_enhanced_vs_flat_patterns() {
        // Create a complex sentence with nested structure
        let tokens = vec![
            create_test_token(
                1,
                "The",
                "the",
                UniversalPos::DET,
                2,
                DependencyRelation::Determiner,
            ),
            create_test_token(
                2,
                "president",
                "president",
                UniversalPos::NOUN,
                3,
                DependencyRelation::NominalSubject,
            ),
            create_test_token(
                3,
                "announced",
                "announce",
                UniversalPos::VERB,
                0,
                DependencyRelation::Root,
            ),
            create_test_token(
                4,
                "new",
                "new",
                UniversalPos::ADJ,
                5,
                DependencyRelation::AdjectivalModifier,
            ),
            create_test_token(
                5,
                "policies",
                "policy",
                UniversalPos::NOUN,
                3,
                DependencyRelation::Object,
            ),
            create_test_token(
                6,
                "yesterday",
                "yesterday",
                UniversalPos::ADV,
                3,
                DependencyRelation::AdverbialModifier,
            ),
        ];

        let sentence = ConlluSentence {
            sent_id: "test-enhanced".to_string(),
            newdoc_id: None,
            newpar_id: None,
            text: "The president announced new policies yesterday.".to_string(),
            tokens,
            metadata: HashMap::new(),
        };

        // Get flat pattern (old way)
        let flat_pattern = sentence.create_pattern_key().unwrap();
        assert!(flat_pattern.contains("announce"));
        assert!(flat_pattern.contains("NominalSubject"));
        assert!(flat_pattern.contains("Object"));

        // Get hierarchical pattern (new way)
        let hierarchical_pattern = sentence.create_hierarchical_pattern_key().unwrap();
        assert!(hierarchical_pattern.contains("announce["));
        assert!(hierarchical_pattern.contains("NominalSubject"));
        assert!(hierarchical_pattern.contains("AdjectivalModifier"));

        // Hierarchical should capture nested structure that flat misses
        // The determiner "The" and adjective "new" are lost in flat but preserved in hierarchical
        println!("Flat pattern: {}", flat_pattern);
        println!("Hierarchical pattern: {}", hierarchical_pattern);
    }
}
