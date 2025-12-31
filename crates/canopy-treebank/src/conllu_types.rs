//! Comprehensive CoNLL-U data types following UD v2 specification
//!
//! This module provides complete type coverage for Universal Dependencies
//! CoNLL-U format parsing with all features, morphology, and enhanced dependencies.

use crate::types::{DependencyFeatures, DependencyRelation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Universal POS tags (UPOS) as defined in UD v2
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
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

    // ======== UniversalPos Tests ========

    #[test]
    fn test_universal_pos_from_str_all_variants() {
        assert_eq!(UniversalPos::from("ADJ"), UniversalPos::ADJ);
        assert_eq!(UniversalPos::from("ADP"), UniversalPos::ADP);
        assert_eq!(UniversalPos::from("ADV"), UniversalPos::ADV);
        assert_eq!(UniversalPos::from("AUX"), UniversalPos::AUX);
        assert_eq!(UniversalPos::from("CCONJ"), UniversalPos::CCONJ);
        assert_eq!(UniversalPos::from("DET"), UniversalPos::DET);
        assert_eq!(UniversalPos::from("INTJ"), UniversalPos::INTJ);
        assert_eq!(UniversalPos::from("NOUN"), UniversalPos::NOUN);
        assert_eq!(UniversalPos::from("NUM"), UniversalPos::NUM);
        assert_eq!(UniversalPos::from("PART"), UniversalPos::PART);
        assert_eq!(UniversalPos::from("PRON"), UniversalPos::PRON);
        assert_eq!(UniversalPos::from("PROPN"), UniversalPos::PROPN);
        assert_eq!(UniversalPos::from("PUNCT"), UniversalPos::PUNCT);
        assert_eq!(UniversalPos::from("SCONJ"), UniversalPos::SCONJ);
        assert_eq!(UniversalPos::from("SYM"), UniversalPos::SYM);
        assert_eq!(UniversalPos::from("VERB"), UniversalPos::VERB);
        assert_eq!(UniversalPos::from("X"), UniversalPos::X);
    }

    #[test]
    fn test_universal_pos_unknown_falls_back_to_x() {
        assert_eq!(UniversalPos::from("UNKNOWN"), UniversalPos::X);
        assert_eq!(UniversalPos::from(""), UniversalPos::X);
        assert_eq!(UniversalPos::from("adj"), UniversalPos::X); // lowercase
    }

    #[test]
    fn test_universal_pos_clone_copy_hash() {
        let pos = UniversalPos::VERB;
        let cloned = pos; // Copy (implements Copy trait)
        let copied = pos;
        assert_eq!(cloned, copied);
        assert_eq!(pos, UniversalPos::VERB);
        // Test Hash by using in HashMap
        let mut map = HashMap::new();
        map.insert(UniversalPos::NOUN, "noun");
        assert_eq!(map.get(&UniversalPos::NOUN), Some(&"noun"));
    }

    // ======== MorphologicalFeatures Tests ========

    #[test]
    fn test_morphological_features_parse_empty() {
        let feats = MorphologicalFeatures::parse("_");
        assert!(feats.animacy.is_none());
        assert!(feats.other.is_empty());

        let feats2 = MorphologicalFeatures::parse("");
        assert!(feats2.tense.is_none());
    }

    #[test]
    fn test_morphological_features_parse_all_known_features() {
        let feats = MorphologicalFeatures::parse(
            "Animacy=Anim|Aspect=Perf|Case=Nom|Definite=Def|Degree=Pos|Gender=Masc|Mood=Ind|Number=Sing|Person=3|Polarity=Pos|PronType=Prs|Tense=Past|VerbForm=Fin|Voice=Act",
        );
        assert_eq!(feats.animacy, Some("Anim".to_string()));
        assert_eq!(feats.aspect, Some("Perf".to_string()));
        assert_eq!(feats.case, Some("Nom".to_string()));
        assert_eq!(feats.definite, Some("Def".to_string()));
        assert_eq!(feats.degree, Some("Pos".to_string()));
        assert_eq!(feats.gender, Some("Masc".to_string()));
        assert_eq!(feats.mood, Some("Ind".to_string()));
        assert_eq!(feats.number, Some("Sing".to_string()));
        assert_eq!(feats.person, Some("3".to_string()));
        assert_eq!(feats.polarity, Some("Pos".to_string()));
        assert_eq!(feats.pron_type, Some("Prs".to_string()));
        assert_eq!(feats.tense, Some("Past".to_string()));
        assert_eq!(feats.verb_form, Some("Fin".to_string()));
        assert_eq!(feats.voice, Some("Act".to_string()));
    }

    #[test]
    fn test_morphological_features_parse_with_other() {
        let feats = MorphologicalFeatures::parse("Number=Plur|CustomFeat=Val");
        assert_eq!(feats.number, Some("Plur".to_string()));
        assert_eq!(feats.other.get("CustomFeat"), Some(&"Val".to_string()));
    }

    #[test]
    fn test_morphological_features_from_hashmap() {
        let mut map = HashMap::new();
        map.insert("Animacy".to_string(), "Inan".to_string());
        map.insert("Tense".to_string(), "Pres".to_string());
        map.insert("Unknown".to_string(), "Value".to_string());

        let feats = MorphologicalFeatures::from_hashmap(&map);
        assert_eq!(feats.animacy, Some("Inan".to_string()));
        assert_eq!(feats.tense, Some("Pres".to_string()));
        assert_eq!(feats.other.get("Unknown"), Some(&"Value".to_string()));
    }

    #[test]
    fn test_morphological_features_from_hashmap_all_known() {
        let mut map = HashMap::new();
        map.insert("Aspect".to_string(), "Imp".to_string());
        map.insert("Case".to_string(), "Acc".to_string());
        map.insert("Definite".to_string(), "Ind".to_string());
        map.insert("Degree".to_string(), "Cmp".to_string());
        map.insert("Gender".to_string(), "Fem".to_string());
        map.insert("Mood".to_string(), "Sub".to_string());
        map.insert("Person".to_string(), "1".to_string());
        map.insert("Polarity".to_string(), "Neg".to_string());
        map.insert("PronType".to_string(), "Dem".to_string());
        map.insert("VerbForm".to_string(), "Inf".to_string());
        map.insert("Voice".to_string(), "Pass".to_string());

        let feats = MorphologicalFeatures::from_hashmap(&map);
        assert_eq!(feats.aspect, Some("Imp".to_string()));
        assert_eq!(feats.case, Some("Acc".to_string()));
        assert_eq!(feats.definite, Some("Ind".to_string()));
        assert_eq!(feats.degree, Some("Cmp".to_string()));
        assert_eq!(feats.gender, Some("Fem".to_string()));
        assert_eq!(feats.mood, Some("Sub".to_string()));
        assert_eq!(feats.person, Some("1".to_string()));
        assert_eq!(feats.polarity, Some("Neg".to_string()));
        assert_eq!(feats.pron_type, Some("Dem".to_string()));
        assert_eq!(feats.verb_form, Some("Inf".to_string()));
        assert_eq!(feats.voice, Some("Pass".to_string()));
    }

    #[test]
    fn test_morphological_features_default() {
        let feats = MorphologicalFeatures::default();
        assert!(feats.animacy.is_none());
        assert!(feats.tense.is_none());
        assert!(feats.other.is_empty());
    }

    // ======== MiscAttributes Tests ========

    #[test]
    fn test_misc_attributes_parse_empty() {
        let misc = MiscAttributes::parse("_");
        assert!(misc.space_after.is_none());
        assert!(misc.other.is_empty());

        let misc2 = MiscAttributes::parse("");
        assert!(misc2.start_char.is_none());
    }

    #[test]
    fn test_misc_attributes_parse_space_after() {
        let misc = MiscAttributes::parse("SpaceAfter=No");
        assert_eq!(misc.space_after, Some(true)); // No space after = true

        let misc2 = MiscAttributes::parse("SpaceAfter=Yes");
        assert_eq!(misc2.space_after, Some(false)); // space_after tracks No
    }

    #[test]
    fn test_misc_attributes_parse_char_positions() {
        let misc = MiscAttributes::parse("StartChar=10|EndChar=15");
        assert_eq!(misc.start_char, Some(10));
        assert_eq!(misc.end_char, Some(15));
    }

    #[test]
    fn test_misc_attributes_parse_token_id() {
        let misc = MiscAttributes::parse("TokenId=1-2");
        assert_eq!(misc.token_id, Some("1-2".to_string()));
    }

    #[test]
    fn test_misc_attributes_parse_other() {
        let misc = MiscAttributes::parse("CustomAttr=Value|AnotherAttr=X");
        assert_eq!(misc.other.get("CustomAttr"), Some(&"Value".to_string()));
        assert_eq!(misc.other.get("AnotherAttr"), Some(&"X".to_string()));
    }

    #[test]
    fn test_misc_attributes_parse_key_only() {
        let misc = MiscAttributes::parse("SomeFlag");
        assert_eq!(misc.other.get("SomeFlag"), Some(&"true".to_string()));
    }

    // ======== EnhancedDependency Tests ========

    #[test]
    fn test_enhanced_dependency_construction() {
        let dep = EnhancedDependency {
            head: 3,
            relation: DependencyRelation::NominalSubject,
        };
        assert_eq!(dep.head, 3);
        assert_eq!(dep.relation, DependencyRelation::NominalSubject);
    }

    #[test]
    fn test_enhanced_dependency_clone_eq() {
        let dep = EnhancedDependency {
            head: 2,
            relation: DependencyRelation::Object,
        };
        let cloned = dep.clone();
        assert_eq!(dep, cloned);
    }

    // ======== ConlluCorpusStats Tests ========

    #[test]
    fn test_conllu_corpus_stats_default() {
        let stats = ConlluCorpusStats::default();
        assert_eq!(stats.sentences, 0);
        assert_eq!(stats.tokens, 0);
        assert!(stats.upos_freq.is_empty());
        assert!(stats.deprel_freq.is_empty());
        assert!(stats.lemma_freq.is_empty());
        assert!(stats.pattern_freq.is_empty());
    }

    #[test]
    fn test_conllu_corpus_stats_add_sentence() {
        let mut stats = ConlluCorpusStats::default();
        let sentence = ConlluSentence {
            sent_id: "test".to_string(),
            newdoc_id: None,
            newpar_id: None,
            text: "John runs.".to_string(),
            tokens: vec![
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
            ],
            metadata: HashMap::new(),
        };

        stats.add_sentence(&sentence);

        assert_eq!(stats.sentences, 1);
        assert_eq!(stats.tokens, 2);
        assert_eq!(stats.upos_freq.get("PROPN"), Some(&1));
        assert_eq!(stats.upos_freq.get("VERB"), Some(&1));
        assert_eq!(stats.deprel_freq.get("NominalSubject"), Some(&1));
        assert_eq!(stats.deprel_freq.get("Root"), Some(&1));
        assert_eq!(stats.lemma_freq.get("John"), Some(&1));
        assert_eq!(stats.lemma_freq.get("run"), Some(&1));
    }

    #[test]
    fn test_conllu_corpus_stats_top_patterns() {
        let mut stats = ConlluCorpusStats::default();
        stats.pattern_freq.insert("pattern1".to_string(), 10);
        stats.pattern_freq.insert("pattern2".to_string(), 5);
        stats.pattern_freq.insert("pattern3".to_string(), 15);

        let top2 = stats.top_patterns(2);
        assert_eq!(top2.len(), 2);
        assert_eq!(top2[0].0, "pattern3");
        assert_eq!(top2[0].1, 15);
        assert_eq!(top2[1].0, "pattern1");
        assert_eq!(top2[1].1, 10);
    }

    // ======== ConlluSentence Tests ========

    #[test]
    fn test_conllu_sentence_root_token() {
        let sentence = ConlluSentence {
            sent_id: "test".to_string(),
            newdoc_id: None,
            newpar_id: None,
            text: "He runs.".to_string(),
            tokens: vec![
                create_test_token(
                    1,
                    "He",
                    "he",
                    UniversalPos::PRON,
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
            ],
            metadata: HashMap::new(),
        };

        let root = sentence.root_token().unwrap();
        assert_eq!(root.lemma, "run");
        assert_eq!(root.head, 0);
    }

    #[test]
    fn test_conllu_sentence_tokens_with_relation() {
        let sentence = ConlluSentence {
            sent_id: "test".to_string(),
            newdoc_id: None,
            newpar_id: None,
            text: "John and Mary run.".to_string(),
            tokens: vec![
                create_test_token(
                    1,
                    "John",
                    "John",
                    UniversalPos::PROPN,
                    4,
                    DependencyRelation::NominalSubject,
                ),
                create_test_token(
                    2,
                    "and",
                    "and",
                    UniversalPos::CCONJ,
                    3,
                    DependencyRelation::CoordinatingConjunction,
                ),
                create_test_token(
                    3,
                    "Mary",
                    "Mary",
                    UniversalPos::PROPN,
                    1,
                    DependencyRelation::Conjunction,
                ),
                create_test_token(
                    4,
                    "run",
                    "run",
                    UniversalPos::VERB,
                    0,
                    DependencyRelation::Root,
                ),
            ],
            metadata: HashMap::new(),
        };

        let nsubj = sentence.tokens_with_relation(&DependencyRelation::NominalSubject);
        assert_eq!(nsubj.len(), 1);
        assert_eq!(nsubj[0].lemma, "John");
    }

    #[test]
    fn test_conllu_sentence_main_verb() {
        let sentence = ConlluSentence {
            sent_id: "test".to_string(),
            newdoc_id: None,
            newpar_id: None,
            text: "He is running.".to_string(),
            tokens: vec![
                create_test_token(
                    1,
                    "He",
                    "he",
                    UniversalPos::PRON,
                    3,
                    DependencyRelation::NominalSubject,
                ),
                create_test_token(
                    2,
                    "is",
                    "be",
                    UniversalPos::AUX,
                    0,
                    DependencyRelation::Root,
                ),
                create_test_token(
                    3,
                    "running",
                    "run",
                    UniversalPos::VERB,
                    2,
                    DependencyRelation::XClausalComplement,
                ),
            ],
            metadata: HashMap::new(),
        };

        let verb = sentence.main_verb().unwrap();
        assert_eq!(verb.lemma, "be"); // AUX at root
        assert_eq!(verb.upos, UniversalPos::AUX);
    }

    #[test]
    fn test_conllu_sentence_verbs() {
        let sentence = ConlluSentence {
            sent_id: "test".to_string(),
            newdoc_id: None,
            newpar_id: None,
            text: "He wants to run.".to_string(),
            tokens: vec![
                create_test_token(
                    1,
                    "He",
                    "he",
                    UniversalPos::PRON,
                    2,
                    DependencyRelation::NominalSubject,
                ),
                create_test_token(
                    2,
                    "wants",
                    "want",
                    UniversalPos::VERB,
                    0,
                    DependencyRelation::Root,
                ),
                create_test_token(
                    3,
                    "to",
                    "to",
                    UniversalPos::PART,
                    4,
                    DependencyRelation::Mark,
                ),
                create_test_token(
                    4,
                    "run",
                    "run",
                    UniversalPos::VERB,
                    2,
                    DependencyRelation::XClausalComplement,
                ),
            ],
            metadata: HashMap::new(),
        };

        let verbs = sentence.verbs();
        assert_eq!(verbs.len(), 2);
        let lemmas: Vec<&str> = verbs.iter().map(|v| v.lemma.as_str()).collect();
        assert!(lemmas.contains(&"want"));
        assert!(lemmas.contains(&"run"));
    }

    #[test]
    fn test_conllu_sentence_get_dependents() {
        let sentence = ConlluSentence {
            sent_id: "test".to_string(),
            newdoc_id: None,
            newpar_id: None,
            text: "The cat runs.".to_string(),
            tokens: vec![
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
            ],
            metadata: HashMap::new(),
        };

        let dependents_of_cat = sentence.get_dependents(2);
        assert_eq!(dependents_of_cat.len(), 1);
        assert_eq!(dependents_of_cat[0].lemma, "the");

        let dependents_of_runs = sentence.get_dependents(3);
        assert_eq!(dependents_of_runs.len(), 1);
        assert_eq!(dependents_of_runs[0].lemma, "cat");
    }

    #[test]
    fn test_conllu_sentence_create_pattern_key_no_verb() {
        let sentence = ConlluSentence {
            sent_id: "test".to_string(),
            newdoc_id: None,
            newpar_id: None,
            text: "Hello!".to_string(),
            tokens: vec![create_test_token(
                1,
                "Hello",
                "hello",
                UniversalPos::INTJ,
                0,
                DependencyRelation::Root,
            )],
            metadata: HashMap::new(),
        };

        // No main verb, so pattern key should be None
        assert!(sentence.create_pattern_key().is_none());
    }

    // ======== DependencyTree Tests ========

    #[test]
    fn test_dependency_tree_depth_single_node() {
        let tree = DependencyTree {
            token: create_test_token(
                1,
                "Hello",
                "hello",
                UniversalPos::INTJ,
                0,
                DependencyRelation::Root,
            ),
            children: vec![],
        };
        assert_eq!(tree.depth(), 1);
    }

    #[test]
    fn test_dependency_tree_node_count() {
        let tree = DependencyTree {
            token: create_test_token(
                1,
                "runs",
                "run",
                UniversalPos::VERB,
                0,
                DependencyRelation::Root,
            ),
            children: vec![
                DependencyTree {
                    token: create_test_token(
                        2,
                        "John",
                        "John",
                        UniversalPos::PROPN,
                        1,
                        DependencyRelation::NominalSubject,
                    ),
                    children: vec![],
                },
                DependencyTree {
                    token: create_test_token(
                        3,
                        "fast",
                        "fast",
                        UniversalPos::ADV,
                        1,
                        DependencyRelation::AdverbialModifier,
                    ),
                    children: vec![],
                },
            ],
        };
        assert_eq!(tree.node_count(), 3);
        assert_eq!(tree.depth(), 2);
    }

    #[test]
    fn test_dependency_tree_find_by_pos_none() {
        let tree = DependencyTree {
            token: create_test_token(
                1,
                "runs",
                "run",
                UniversalPos::VERB,
                0,
                DependencyRelation::Root,
            ),
            children: vec![],
        };
        let nouns = tree.find_by_pos(&UniversalPos::NOUN);
        assert!(nouns.is_empty());
    }

    #[test]
    fn test_dependency_tree_find_by_relation_none() {
        let tree = DependencyTree {
            token: create_test_token(
                1,
                "runs",
                "run",
                UniversalPos::VERB,
                0,
                DependencyRelation::Root,
            ),
            children: vec![],
        };
        let objects = tree.find_by_relation(&DependencyRelation::Object);
        assert!(objects.is_empty());
    }

    // ======== ConlluToken Tests ========

    #[test]
    fn test_conllu_token_construction() {
        let token = ConlluToken {
            id: 1,
            form: "running".to_string(),
            lemma: "run".to_string(),
            upos: UniversalPos::VERB,
            xpos: Some("VBG".to_string()),
            features: MorphologicalFeatures::default(),
            head: 0,
            deprel: DependencyRelation::Root,
            enhanced_deps: vec![EnhancedDependency {
                head: 0,
                relation: DependencyRelation::Root,
            }],
            misc: MiscAttributes::default(),
            dependency_features: DependencyFeatures::default(),
        };
        assert_eq!(token.id, 1);
        assert_eq!(token.form, "running");
        assert_eq!(token.lemma, "run");
        assert_eq!(token.upos, UniversalPos::VERB);
        assert_eq!(token.xpos, Some("VBG".to_string()));
    }

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
