//! `WordNet` type definitions
//!
//! This module contains comprehensive type definitions for `WordNet` 3.1 data structures,
//! including synsets, word senses, semantic relations, and lexical entries.

use crate::engine::count_to_f32;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Part-of-speech categories in `WordNet`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PartOfSpeech {
    /// Noun
    Noun,
    /// Verb
    Verb,
    /// Adjective
    Adjective,
    /// Adjective satellite (similar to adjective)
    AdjectiveSatellite,
    /// Adverb
    Adverb,
}

impl PartOfSpeech {
    /// Get the single character code for this part of speech
    #[must_use]
    pub fn code(&self) -> char {
        match self {
            PartOfSpeech::Noun => 'n',
            PartOfSpeech::Verb => 'v',
            PartOfSpeech::Adjective => 'a',
            PartOfSpeech::AdjectiveSatellite => 's',
            PartOfSpeech::Adverb => 'r',
        }
    }

    /// Get the human-readable name
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            PartOfSpeech::Noun => "noun",
            PartOfSpeech::Verb => "verb",
            PartOfSpeech::Adjective => "adjective",
            PartOfSpeech::AdjectiveSatellite => "adjective satellite",
            PartOfSpeech::Adverb => "adverb",
        }
    }
}

/// Semantic relations between synsets in `WordNet`
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SemanticRelation {
    /// Antonym (opposite meaning)
    Antonym,
    /// Hypernym (superordinate, "is-a" relation)
    Hypernym,
    /// Hyponym (subordinate, reverse of hypernym)
    Hyponym,
    /// Instance hypernym (instance-to-class relation, e.g., "Einstein" -> "physicist")
    InstanceHypernym,
    /// Instance hyponym (class-to-instance relation, e.g., "physicist" -> "Einstein")
    InstanceHyponym,
    /// Member holonym (whole of which synset is member)
    MemberHolonym,
    /// Substance holonym (whole of which synset is substance)
    SubstanceHolonym,
    /// Part holonym (whole of which synset is part)
    PartHolonym,
    /// Member meronym (has member)
    MemberMeronym,
    /// Substance meronym (has substance)
    SubstanceMeronym,
    /// Part meronym (has part)
    PartMeronym,
    /// Attribute (adjective-noun pairs)
    Attribute,
    /// Derivationally related form
    Derivation,
    /// Domain of synset (topic)
    DomainTopic,
    /// Domain of synset (region)
    DomainRegion,
    /// Domain of synset (usage)
    DomainUsage,
    /// Member of domain (topic)
    MemberTopic,
    /// Member of domain (region)
    MemberRegion,
    /// Member of domain (usage)
    MemberUsage,
    /// Entailment (verbs)
    Entailment,
    /// Cause (verbs)
    Cause,
    /// Also see (additional information)
    AlsoSee,
    /// Verb group
    VerbGroup,
    /// Similar to (adjectives)
    SimilarTo,
    /// Participle of verb
    Participle,
    /// Pertainym (adjectives pertaining to nouns)
    Pertainym,
}

impl SemanticRelation {
    /// Get the symbolic representation used in `WordNet` data files
    #[must_use]
    pub fn symbol(&self) -> &'static str {
        match self {
            SemanticRelation::Antonym => "!",
            SemanticRelation::Hypernym => "@",
            SemanticRelation::Hyponym => "~",
            SemanticRelation::InstanceHypernym => "@i",
            SemanticRelation::InstanceHyponym => "~i",
            SemanticRelation::MemberHolonym => "#m",
            SemanticRelation::SubstanceHolonym => "#s",
            SemanticRelation::PartHolonym => "#p",
            SemanticRelation::MemberMeronym => "%m",
            SemanticRelation::SubstanceMeronym => "%s",
            SemanticRelation::PartMeronym => "%p",
            SemanticRelation::Attribute => "=",
            SemanticRelation::Derivation => "+",
            SemanticRelation::DomainTopic => ";c",
            SemanticRelation::DomainRegion => ";r",
            SemanticRelation::DomainUsage => ";u",
            SemanticRelation::MemberTopic => "-c",
            SemanticRelation::MemberRegion => "-r",
            SemanticRelation::MemberUsage => "-u",
            SemanticRelation::Entailment => "*",
            SemanticRelation::Cause => ">",
            SemanticRelation::AlsoSee => "^",
            SemanticRelation::VerbGroup => "$",
            SemanticRelation::SimilarTo => "&",
            SemanticRelation::Participle => "<",
            SemanticRelation::Pertainym => "\\",
        }
    }

    /// Get human-readable description
    #[must_use]
    pub fn description(&self) -> &'static str {
        match self {
            SemanticRelation::Antonym => "opposite meaning",
            SemanticRelation::Hypernym => "more general term",
            SemanticRelation::Hyponym => "more specific term",
            SemanticRelation::InstanceHypernym => "class of this instance",
            SemanticRelation::InstanceHyponym => "instance of this class",
            SemanticRelation::MemberHolonym => "whole that has this as member",
            SemanticRelation::SubstanceHolonym => "whole that has this as substance",
            SemanticRelation::PartHolonym => "whole that has this as part",
            SemanticRelation::MemberMeronym => "has member",
            SemanticRelation::SubstanceMeronym => "has substance",
            SemanticRelation::PartMeronym => "has part",
            SemanticRelation::Attribute => "attribute relationship",
            SemanticRelation::Derivation => "derivationally related",
            SemanticRelation::DomainTopic => "topic domain",
            SemanticRelation::DomainRegion => "region domain",
            SemanticRelation::DomainUsage => "usage domain",
            SemanticRelation::MemberTopic => "member of topic",
            SemanticRelation::MemberRegion => "member of region",
            SemanticRelation::MemberUsage => "member of usage",
            SemanticRelation::Entailment => "entails",
            SemanticRelation::Cause => "causes",
            SemanticRelation::AlsoSee => "see also",
            SemanticRelation::VerbGroup => "verb group",
            SemanticRelation::SimilarTo => "similar to",
            SemanticRelation::Participle => "participle form",
            SemanticRelation::Pertainym => "pertains to",
        }
    }
}

/// A semantic pointer linking synsets
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SemanticPointer {
    /// Type of semantic relation
    pub relation: SemanticRelation,
    /// Target synset offset
    pub target_offset: usize,
    /// Target part of speech
    pub target_pos: PartOfSpeech,
    /// Source word number (0 if whole synset)
    pub source_word: u8,
    /// Target word number (0 if whole synset)
    pub target_word: u8,
}

/// A word in a synset with its lexical ID and usage count
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SynsetWord {
    /// The word form
    pub word: String,
    /// Lexical ID for disambiguation
    pub lex_id: u8,
    /// Usage count/frequency (from `TagCount` if available)
    pub tag_count: Option<u32>,
}

/// Verb frame information for verb synsets
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerbFrame {
    /// Frame number
    pub frame_number: u8,
    /// Word number this frame applies to (0 for all words)
    pub word_number: u8,
    /// Frame template
    pub template: String,
}

/// A `WordNet` synset (synonym set)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Synset {
    /// Synset offset (unique identifier)
    pub offset: usize,
    /// Lexicographer file number
    pub lex_filenum: u8,
    /// Part of speech
    pub pos: PartOfSpeech,
    /// Words in this synset
    pub words: Vec<SynsetWord>,
    /// Semantic pointers to other synsets
    pub pointers: Vec<SemanticPointer>,
    /// Verb frames (only for verb synsets)
    pub frames: Vec<VerbFrame>,
    /// Gloss (definition and examples)
    pub gloss: String,
}

impl Synset {
    /// Get the primary word (first word in the synset)
    #[must_use]
    pub fn primary_word(&self) -> Option<&str> {
        self.words.first().map(|w| w.word.as_str())
    }

    /// Check if this synset contains a specific word
    #[must_use]
    pub fn contains_word(&self, word: &str) -> bool {
        self.words.iter().any(|w| w.word == word)
    }

    /// Get all words as a vector of strings
    #[must_use]
    pub fn word_list(&self) -> Vec<String> {
        self.words.iter().map(|w| w.word.clone()).collect()
    }

    /// Get pointers of a specific relation type
    #[must_use]
    pub fn get_relations(&self, relation: &SemanticRelation) -> Vec<&SemanticPointer> {
        self.pointers
            .iter()
            .filter(|p| &p.relation == relation)
            .collect()
    }

    /// Extract definition from gloss (text before first semicolon or quote)
    #[must_use]
    pub fn definition(&self) -> String {
        if let Some(pos) = self.gloss.find(';') {
            self.gloss[..pos].trim().to_string()
        } else if let Some(pos) = self.gloss.find('"') {
            self.gloss[..pos].trim().to_string()
        } else {
            self.gloss.trim().to_string()
        }
    }

    /// Extract examples from gloss (text in quotes)
    #[must_use]
    pub fn examples(&self) -> Vec<String> {
        let mut examples = Vec::new();
        let mut in_quote = false;
        let mut current_example = String::new();

        for ch in self.gloss.chars() {
            match ch {
                '"' => {
                    if in_quote {
                        if !current_example.trim().is_empty() {
                            examples.push(current_example.trim().to_string());
                        }
                        current_example.clear();
                    }
                    in_quote = !in_quote;
                }
                _ if in_quote => {
                    current_example.push(ch);
                }
                _ => {}
            }
        }

        examples
    }
}

/// An index entry mapping a word to its synsets
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IndexEntry {
    /// The word (lemma)
    pub lemma: String,
    /// Part of speech
    pub pos: PartOfSpeech,
    /// Number of synsets containing this word
    pub synset_count: u32,
    /// Number of different semantic relations
    pub pointer_count: u32,
    /// Semantic relation types this word participates in
    pub relations: Vec<SemanticRelation>,
    /// Number of times word is tagged in semantic concordance
    pub tag_sense_count: u32,
    /// Offsets of synsets containing this word
    pub synset_offsets: Vec<usize>,
}

impl IndexEntry {
    /// Get the primary synset (first one, usually most common)
    #[must_use]
    pub fn primary_synset_offset(&self) -> Option<usize> {
        self.synset_offsets.first().copied()
    }

    /// Check if word participates in a specific semantic relation
    #[must_use]
    pub fn has_relation(&self, relation: &SemanticRelation) -> bool {
        self.relations.contains(relation)
    }
}

/// Exception list entry for morphological processing
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExceptionEntry {
    /// Inflected form
    pub inflected: String,
    /// Base forms
    pub base_forms: Vec<String>,
}

/// Complete `WordNet` lexical database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordNetDatabase {
    /// Synsets indexed by offset
    pub synsets: HashMap<usize, Synset>,
    /// Index entries by word and part of speech
    pub index: HashMap<(String, PartOfSpeech), IndexEntry>,
    /// Exception lists for morphological processing
    pub exceptions: HashMap<PartOfSpeech, HashMap<String, ExceptionEntry>>,
    /// Reverse lookup: synset offset to words
    pub synset_words: HashMap<usize, Vec<String>>,
}

impl WordNetDatabase {
    /// Create a new empty `WordNet` database
    #[must_use]
    pub fn new() -> Self {
        Self {
            synsets: HashMap::new(),
            index: HashMap::new(),
            exceptions: HashMap::new(),
            synset_words: HashMap::new(),
        }
    }

    /// Look up synsets for a word
    #[must_use]
    pub fn lookup_word(&self, word: &str, pos: PartOfSpeech) -> Option<&IndexEntry> {
        self.index.get(&(word.to_lowercase(), pos))
    }

    /// Get synset by offset
    #[must_use]
    pub fn get_synset(&self, offset: usize) -> Option<&Synset> {
        self.synsets.get(&offset)
    }

    /// Get all synsets for a word
    #[must_use]
    pub fn get_synsets_for_word(&self, word: &str, pos: PartOfSpeech) -> Vec<&Synset> {
        if let Some(entry) = self.lookup_word(word, pos) {
            entry
                .synset_offsets
                .iter()
                .filter_map(|&offset| self.synsets.get(&offset))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get hypernyms (more general terms) for a synset
    #[must_use]
    pub fn get_hypernyms(&self, synset: &Synset) -> Vec<&Synset> {
        synset
            .get_relations(&SemanticRelation::Hypernym)
            .iter()
            .filter_map(|ptr| self.synsets.get(&ptr.target_offset))
            .collect()
    }

    /// Get hyponyms (more specific terms) for a synset
    #[must_use]
    pub fn get_hyponyms(&self, synset: &Synset) -> Vec<&Synset> {
        synset
            .get_relations(&SemanticRelation::Hyponym)
            .iter()
            .filter_map(|ptr| self.synsets.get(&ptr.target_offset))
            .collect()
    }

    /// Get instance hypernyms (classes of this instance) for a synset
    #[must_use]
    pub fn get_instance_hypernyms(&self, synset: &Synset) -> Vec<&Synset> {
        synset
            .get_relations(&SemanticRelation::InstanceHypernym)
            .iter()
            .filter_map(|ptr| self.synsets.get(&ptr.target_offset))
            .collect()
    }

    /// Get instance hyponyms (instances of this class) for a synset
    #[must_use]
    pub fn get_instance_hyponyms(&self, synset: &Synset) -> Vec<&Synset> {
        synset
            .get_relations(&SemanticRelation::InstanceHyponym)
            .iter()
            .filter_map(|ptr| self.synsets.get(&ptr.target_offset))
            .collect()
    }

    /// Find the lowest common hypernym of two synsets
    #[must_use]
    pub fn lowest_common_hypernym<'a>(
        &'a self,
        synset1: &'a Synset,
        synset2: &'a Synset,
    ) -> Option<&'a Synset> {
        let mut hypernyms1 = vec![synset1];
        let mut current = synset1;

        // Collect all hypernyms of synset1
        while let Some(hypernym) = self.get_hypernyms(current).first() {
            hypernyms1.push(hypernym);
            current = hypernym;
        }

        // Check hypernyms of synset2 against synset1's hypernyms
        let mut current = synset2;
        loop {
            if hypernyms1.contains(&current) {
                return Some(current);
            }

            if let Some(hypernym) = self.get_hypernyms(current).first() {
                current = hypernym;
            } else {
                break;
            }
        }

        None
    }

    /// Calculate semantic similarity between two synsets using path distance
    /// Uses the formula: similarity = 1 / (`path_length` + 1)
    /// where `path_length` is the shortest path through the hypernym hierarchy
    #[must_use]
    pub fn path_similarity(&self, synset1: &Synset, synset2: &Synset) -> f32 {
        if synset1.offset == synset2.offset {
            return 1.0;
        }

        if let Some(lch) = self.lowest_common_hypernym(synset1, synset2) {
            // Calculate depth from synset1 to LCH
            let depth1 = self.depth_to_ancestor(synset1, lch);
            // Calculate depth from synset2 to LCH
            let depth2 = self.depth_to_ancestor(synset2, lch);

            let path_length = depth1 + depth2;
            // Use inverse path distance: similarity = 1 / (path_length + 1)
            1.0 / (count_to_f32(path_length) + 1.0)
        } else {
            0.0
        }
    }

    /// Calculate depth from a synset to an ancestor in the hypernym hierarchy
    fn depth_to_ancestor(&self, synset: &Synset, ancestor: &Synset) -> usize {
        if synset.offset == ancestor.offset {
            return 0;
        }

        let mut depth = 0;
        let mut current = synset;

        while current.offset != ancestor.offset {
            depth += 1;
            if let Some(hypernym) = self.get_hypernyms(current).first() {
                current = hypernym;
            } else {
                // Should not happen if ancestor is valid, but return max depth as fallback
                return depth;
            }
        }

        depth
    }

    /// Get database statistics
    #[must_use]
    pub fn stats(&self) -> DatabaseStats {
        let noun_synsets = self
            .synsets
            .values()
            .filter(|s| s.pos == PartOfSpeech::Noun)
            .count();
        let verb_synsets = self
            .synsets
            .values()
            .filter(|s| s.pos == PartOfSpeech::Verb)
            .count();
        let adjective_count = self
            .synsets
            .values()
            .filter(|s| {
                matches!(
                    s.pos,
                    PartOfSpeech::Adjective | PartOfSpeech::AdjectiveSatellite
                )
            })
            .count();
        let adverb_count = self
            .synsets
            .values()
            .filter(|s| s.pos == PartOfSpeech::Adverb)
            .count();

        let total_words: usize = self.synsets.values().map(|s| s.words.len()).sum();
        let total_relations: usize = self.synsets.values().map(|s| s.pointers.len()).sum();

        DatabaseStats {
            total_synsets: self.synsets.len(),
            noun_synsets,
            verb_synsets,
            adjective_synsets: adjective_count,
            adverb_synsets: adverb_count,
            total_words,
            total_index_entries: self.index.len(),
            total_relations,
        }
    }
}

impl Default for WordNetDatabase {
    fn default() -> Self {
        Self::new()
    }
}

/// Database statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseStats {
    pub total_synsets: usize,
    pub noun_synsets: usize,
    pub verb_synsets: usize,
    pub adjective_synsets: usize,
    pub adverb_synsets: usize,
    pub total_words: usize,
    pub total_index_entries: usize,
    pub total_relations: usize,
}

/// Analysis result from `WordNet` engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordNetAnalysis {
    /// Input word being analyzed
    pub word: String,
    /// Part of speech
    pub pos: PartOfSpeech,
    /// Synsets containing this word
    pub synsets: Vec<Synset>,
    /// Semantic relations found
    pub relations: Vec<(SemanticRelation, Vec<Synset>)>,
    /// Word definitions
    pub definitions: Vec<String>,
    /// Usage examples
    pub examples: Vec<String>,
    /// Confidence score
    pub confidence: f32,
}

impl WordNetAnalysis {
    /// Create a new analysis result
    #[must_use]
    pub fn new(word: String, pos: PartOfSpeech) -> Self {
        Self {
            word,
            pos,
            synsets: Vec::new(),
            relations: Vec::new(),
            definitions: Vec::new(),
            examples: Vec::new(),
            confidence: 0.0,
        }
    }

    /// Check if any synsets were found
    #[must_use]
    pub fn has_results(&self) -> bool {
        !self.synsets.is_empty()
    }

    /// Get the primary definition (from first synset)
    #[must_use]
    pub fn primary_definition(&self) -> Option<&String> {
        self.definitions.first()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === PartOfSpeech Tests ===

    #[test]
    fn test_part_of_speech_code() {
        assert_eq!(PartOfSpeech::Noun.code(), 'n');
        assert_eq!(PartOfSpeech::Verb.code(), 'v');
        assert_eq!(PartOfSpeech::Adjective.code(), 'a');
        assert_eq!(PartOfSpeech::AdjectiveSatellite.code(), 's');
        assert_eq!(PartOfSpeech::Adverb.code(), 'r');
    }

    #[test]
    fn test_part_of_speech_name() {
        assert_eq!(PartOfSpeech::Noun.name(), "noun");
        assert_eq!(PartOfSpeech::Verb.name(), "verb");
        assert_eq!(PartOfSpeech::Adjective.name(), "adjective");
        assert_eq!(
            PartOfSpeech::AdjectiveSatellite.name(),
            "adjective satellite"
        );
        assert_eq!(PartOfSpeech::Adverb.name(), "adverb");
    }

    // === SemanticRelation Tests ===

    #[test]
    fn test_semantic_relation_symbol() {
        assert_eq!(SemanticRelation::Antonym.symbol(), "!");
        assert_eq!(SemanticRelation::Hypernym.symbol(), "@");
        assert_eq!(SemanticRelation::Hyponym.symbol(), "~");
        assert_eq!(SemanticRelation::InstanceHypernym.symbol(), "@i");
        assert_eq!(SemanticRelation::InstanceHyponym.symbol(), "~i");
        assert_eq!(SemanticRelation::MemberHolonym.symbol(), "#m");
        assert_eq!(SemanticRelation::SubstanceHolonym.symbol(), "#s");
        assert_eq!(SemanticRelation::PartHolonym.symbol(), "#p");
        assert_eq!(SemanticRelation::MemberMeronym.symbol(), "%m");
        assert_eq!(SemanticRelation::SubstanceMeronym.symbol(), "%s");
        assert_eq!(SemanticRelation::PartMeronym.symbol(), "%p");
        assert_eq!(SemanticRelation::Attribute.symbol(), "=");
        assert_eq!(SemanticRelation::Derivation.symbol(), "+");
        assert_eq!(SemanticRelation::Entailment.symbol(), "*");
        assert_eq!(SemanticRelation::Cause.symbol(), ">");
        assert_eq!(SemanticRelation::AlsoSee.symbol(), "^");
        assert_eq!(SemanticRelation::VerbGroup.symbol(), "$");
        assert_eq!(SemanticRelation::SimilarTo.symbol(), "&");
        assert_eq!(SemanticRelation::Participle.symbol(), "<");
        assert_eq!(SemanticRelation::Pertainym.symbol(), "\\");
    }

    #[test]
    fn test_semantic_relation_description() {
        assert_eq!(SemanticRelation::Antonym.description(), "opposite meaning");
        assert_eq!(
            SemanticRelation::Hypernym.description(),
            "more general term"
        );
        assert_eq!(
            SemanticRelation::Hyponym.description(),
            "more specific term"
        );
        assert_eq!(
            SemanticRelation::InstanceHypernym.description(),
            "class of this instance"
        );
        assert_eq!(
            SemanticRelation::InstanceHyponym.description(),
            "instance of this class"
        );
        assert!(!SemanticRelation::MemberHolonym.description().is_empty());
        assert!(!SemanticRelation::Derivation.description().is_empty());
        assert!(!SemanticRelation::Entailment.description().is_empty());
        assert!(!SemanticRelation::Cause.description().is_empty());
    }

    // === Synset Tests ===

    fn create_test_synset() -> Synset {
        Synset {
            offset: 100_001,
            lex_filenum: 3,
            pos: PartOfSpeech::Noun,
            words: vec![
                SynsetWord {
                    word: "dog".to_string(),
                    lex_id: 0,
                    tag_count: Some(100),
                },
                SynsetWord {
                    word: "canine".to_string(),
                    lex_id: 1,
                    tag_count: Some(50),
                },
            ],
            pointers: vec![SemanticPointer {
                relation: SemanticRelation::Hypernym,
                target_offset: 200_001,
                target_pos: PartOfSpeech::Noun,
                source_word: 0,
                target_word: 0,
            }],
            frames: vec![],
            gloss: "a domesticated carnivorous mammal; \"the dog barked all night\"".to_string(),
        }
    }

    #[test]
    fn test_synset_primary_word() {
        let synset = create_test_synset();
        assert_eq!(synset.primary_word(), Some("dog"));

        let empty_synset = Synset {
            offset: 0,
            lex_filenum: 0,
            pos: PartOfSpeech::Noun,
            words: vec![],
            pointers: vec![],
            frames: vec![],
            gloss: String::new(),
        };
        assert_eq!(empty_synset.primary_word(), None);
    }

    #[test]
    fn test_synset_contains_word() {
        let synset = create_test_synset();
        assert!(synset.contains_word("dog"));
        assert!(synset.contains_word("canine"));
        assert!(!synset.contains_word("cat"));
    }

    #[test]
    fn test_synset_word_list() {
        let synset = create_test_synset();
        let words = synset.word_list();
        assert_eq!(words.len(), 2);
        assert!(words.contains(&"dog".to_string()));
        assert!(words.contains(&"canine".to_string()));
    }

    #[test]
    fn test_synset_get_relations() {
        let synset = create_test_synset();
        let hypernyms = synset.get_relations(&SemanticRelation::Hypernym);
        assert_eq!(hypernyms.len(), 1);
        assert_eq!(hypernyms[0].target_offset, 200_001);

        let antonyms = synset.get_relations(&SemanticRelation::Antonym);
        assert!(antonyms.is_empty());
    }

    #[test]
    fn test_synset_definition() {
        let synset = create_test_synset();
        let def = synset.definition();
        assert_eq!(def, "a domesticated carnivorous mammal");

        // Test with semicolon separator
        let synset_semi = Synset {
            offset: 0,
            lex_filenum: 0,
            pos: PartOfSpeech::Noun,
            words: vec![],
            pointers: vec![],
            frames: vec![],
            gloss: "definition text; example sentence".to_string(),
        };
        assert_eq!(synset_semi.definition(), "definition text");

        // Test with no separator
        let synset_plain = Synset {
            offset: 0,
            lex_filenum: 0,
            pos: PartOfSpeech::Noun,
            words: vec![],
            pointers: vec![],
            frames: vec![],
            gloss: "just a definition".to_string(),
        };
        assert_eq!(synset_plain.definition(), "just a definition");
    }

    #[test]
    fn test_synset_examples() {
        let synset = create_test_synset();
        let examples = synset.examples();
        assert_eq!(examples.len(), 1);
        assert_eq!(examples[0], "the dog barked all night");

        // Test with multiple examples
        let synset_multi = Synset {
            offset: 0,
            lex_filenum: 0,
            pos: PartOfSpeech::Noun,
            words: vec![],
            pointers: vec![],
            frames: vec![],
            gloss: "definition; \"first example\" \"second example\"".to_string(),
        };
        let examples = synset_multi.examples();
        assert_eq!(examples.len(), 2);

        // Test with no examples
        let synset_no_ex = Synset {
            offset: 0,
            lex_filenum: 0,
            pos: PartOfSpeech::Noun,
            words: vec![],
            pointers: vec![],
            frames: vec![],
            gloss: "just a definition".to_string(),
        };
        assert!(synset_no_ex.examples().is_empty());
    }

    // === IndexEntry Tests ===

    #[test]
    fn test_index_entry_primary_synset_offset() {
        let entry = IndexEntry {
            lemma: "dog".to_string(),
            pos: PartOfSpeech::Noun,
            synset_count: 2,
            pointer_count: 5,
            relations: vec![SemanticRelation::Hypernym, SemanticRelation::Hyponym],
            tag_sense_count: 10,
            synset_offsets: vec![100_001, 100_002],
        };
        assert_eq!(entry.primary_synset_offset(), Some(100_001));

        let empty_entry = IndexEntry {
            lemma: "test".to_string(),
            pos: PartOfSpeech::Noun,
            synset_count: 0,
            pointer_count: 0,
            relations: vec![],
            tag_sense_count: 0,
            synset_offsets: vec![],
        };
        assert_eq!(empty_entry.primary_synset_offset(), None);
    }

    #[test]
    fn test_index_entry_has_relation() {
        let entry = IndexEntry {
            lemma: "dog".to_string(),
            pos: PartOfSpeech::Noun,
            synset_count: 1,
            pointer_count: 2,
            relations: vec![SemanticRelation::Hypernym, SemanticRelation::Hyponym],
            tag_sense_count: 5,
            synset_offsets: vec![100_001],
        };
        assert!(entry.has_relation(&SemanticRelation::Hypernym));
        assert!(entry.has_relation(&SemanticRelation::Hyponym));
        assert!(!entry.has_relation(&SemanticRelation::Antonym));
    }

    // === ExceptionEntry Tests ===

    #[test]
    fn test_exception_entry() {
        let entry = ExceptionEntry {
            inflected: "dogs".to_string(),
            base_forms: vec!["dog".to_string()],
        };
        assert_eq!(entry.inflected, "dogs");
        assert_eq!(entry.base_forms.len(), 1);
        assert_eq!(entry.base_forms[0], "dog");
    }

    // === WordNetDatabase Tests ===

    #[test]
    fn test_wordnet_database_new() {
        let db = WordNetDatabase::new();
        assert!(db.synsets.is_empty());
        assert!(db.index.is_empty());
        assert!(db.exceptions.is_empty());
        assert!(db.synset_words.is_empty());
    }

    #[test]
    fn test_wordnet_database_default() {
        let db = WordNetDatabase::default();
        assert!(db.synsets.is_empty());
    }

    #[test]
    fn test_wordnet_database_lookup_word() {
        let mut db = WordNetDatabase::new();
        let entry = IndexEntry {
            lemma: "dog".to_string(),
            pos: PartOfSpeech::Noun,
            synset_count: 1,
            pointer_count: 1,
            relations: vec![SemanticRelation::Hypernym],
            tag_sense_count: 5,
            synset_offsets: vec![100_001],
        };
        db.index
            .insert(("dog".to_string(), PartOfSpeech::Noun), entry);

        assert!(db.lookup_word("dog", PartOfSpeech::Noun).is_some());
        assert!(db.lookup_word("DOG", PartOfSpeech::Noun).is_some()); // case insensitive
        assert!(db.lookup_word("cat", PartOfSpeech::Noun).is_none());
        assert!(db.lookup_word("dog", PartOfSpeech::Verb).is_none());
    }

    #[test]
    fn test_wordnet_database_get_synset() {
        let mut db = WordNetDatabase::new();
        let synset = create_test_synset();
        db.synsets.insert(100_001, synset);

        assert!(db.get_synset(100_001).is_some());
        assert_eq!(db.get_synset(100_001).unwrap().offset, 100_001);
        assert!(db.get_synset(999_999).is_none());
    }

    #[test]
    fn test_wordnet_database_get_synsets_for_word() {
        let mut db = WordNetDatabase::new();
        let synset = create_test_synset();
        db.synsets.insert(100_001, synset);
        db.index.insert(
            ("dog".to_string(), PartOfSpeech::Noun),
            IndexEntry {
                lemma: "dog".to_string(),
                pos: PartOfSpeech::Noun,
                synset_count: 1,
                pointer_count: 1,
                relations: vec![],
                tag_sense_count: 0,
                synset_offsets: vec![100_001],
            },
        );

        let synsets = db.get_synsets_for_word("dog", PartOfSpeech::Noun);
        assert_eq!(synsets.len(), 1);
        assert_eq!(synsets[0].offset, 100_001);

        let no_synsets = db.get_synsets_for_word("cat", PartOfSpeech::Noun);
        assert!(no_synsets.is_empty());
    }

    #[test]
    fn test_wordnet_database_stats() {
        let mut db = WordNetDatabase::new();
        let synset = create_test_synset();
        db.synsets.insert(100_001, synset);

        let verb_synset = Synset {
            offset: 200_001,
            lex_filenum: 1,
            pos: PartOfSpeech::Verb,
            words: vec![SynsetWord {
                word: "run".to_string(),
                lex_id: 0,
                tag_count: None,
            }],
            pointers: vec![],
            frames: vec![],
            gloss: "move fast".to_string(),
        };
        db.synsets.insert(200_001, verb_synset);

        let stats = db.stats();
        assert_eq!(stats.total_synsets, 2);
        assert_eq!(stats.noun_synsets, 1);
        assert_eq!(stats.verb_synsets, 1);
        assert_eq!(stats.adjective_synsets, 0);
        assert_eq!(stats.adverb_synsets, 0);
        assert_eq!(stats.total_words, 3); // dog, canine, run
        assert_eq!(stats.total_relations, 1); // one hypernym pointer
    }

    #[test]
    fn test_wordnet_database_path_similarity_same_synset() {
        let db = WordNetDatabase::new();
        let synset = create_test_synset();
        let similarity = db.path_similarity(&synset, &synset);
        assert!((similarity - 1.0).abs() < f32::EPSILON);
    }

    // === WordNetAnalysis Tests ===

    #[test]
    fn test_wordnet_analysis_new() {
        let analysis = WordNetAnalysis::new("dog".to_string(), PartOfSpeech::Noun);
        assert_eq!(analysis.word, "dog");
        assert_eq!(analysis.pos, PartOfSpeech::Noun);
        assert!(analysis.synsets.is_empty());
        assert!(analysis.relations.is_empty());
        assert!(analysis.definitions.is_empty());
        assert!(analysis.examples.is_empty());
        assert!(analysis.confidence.abs() < f32::EPSILON);
    }

    #[test]
    fn test_wordnet_analysis_has_results() {
        let empty_analysis = WordNetAnalysis::new("test".to_string(), PartOfSpeech::Noun);
        assert!(!empty_analysis.has_results());

        let mut with_synset = WordNetAnalysis::new("dog".to_string(), PartOfSpeech::Noun);
        with_synset.synsets.push(create_test_synset());
        assert!(with_synset.has_results());
    }

    #[test]
    fn test_wordnet_analysis_primary_definition() {
        let empty_analysis = WordNetAnalysis::new("test".to_string(), PartOfSpeech::Noun);
        assert!(empty_analysis.primary_definition().is_none());

        let mut with_defs = WordNetAnalysis::new("dog".to_string(), PartOfSpeech::Noun);
        with_defs.definitions.push("definition one".to_string());
        with_defs.definitions.push("definition two".to_string());
        assert_eq!(
            with_defs.primary_definition(),
            Some(&"definition one".to_string())
        );
    }

    // === Serialization Tests ===

    #[test]
    fn test_part_of_speech_serialization() {
        let pos = PartOfSpeech::Verb;
        let json = serde_json::to_string(&pos).unwrap();
        let deserialized: PartOfSpeech = serde_json::from_str(&json).unwrap();
        assert_eq!(pos, deserialized);
    }

    #[test]
    fn test_semantic_relation_serialization() {
        let rel = SemanticRelation::Hypernym;
        let json = serde_json::to_string(&rel).unwrap();
        let deserialized: SemanticRelation = serde_json::from_str(&json).unwrap();
        assert_eq!(rel, deserialized);
    }

    #[test]
    fn test_synset_serialization() {
        let synset = create_test_synset();
        let json = serde_json::to_string(&synset).unwrap();
        let deserialized: Synset = serde_json::from_str(&json).unwrap();
        assert_eq!(synset.offset, deserialized.offset);
        assert_eq!(synset.words.len(), deserialized.words.len());
    }
}
