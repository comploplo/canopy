//! WordNet type definitions
//!
//! This module contains comprehensive type definitions for WordNet 3.1 data structures,
//! including synsets, word senses, semantic relations, and lexical entries.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Part-of-speech categories in WordNet
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

/// Semantic relations between synsets in WordNet
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
    /// Get the symbolic representation used in WordNet data files
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
    /// Usage count/frequency (from TagCount if available)
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

/// A WordNet synset (synonym set)
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
    pub fn primary_word(&self) -> Option<&str> {
        self.words.first().map(|w| w.word.as_str())
    }

    /// Check if this synset contains a specific word
    pub fn contains_word(&self, word: &str) -> bool {
        self.words.iter().any(|w| w.word == word)
    }

    /// Get all words as a vector of strings
    pub fn word_list(&self) -> Vec<String> {
        self.words.iter().map(|w| w.word.clone()).collect()
    }

    /// Get pointers of a specific relation type
    pub fn get_relations(&self, relation: &SemanticRelation) -> Vec<&SemanticPointer> {
        self.pointers
            .iter()
            .filter(|p| &p.relation == relation)
            .collect()
    }

    /// Extract definition from gloss (text before first semicolon or quote)
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
    pub fn primary_synset_offset(&self) -> Option<usize> {
        self.synset_offsets.first().copied()
    }

    /// Check if word participates in a specific semantic relation
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

/// Complete WordNet lexical database
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
    /// Create a new empty WordNet database
    pub fn new() -> Self {
        Self {
            synsets: HashMap::new(),
            index: HashMap::new(),
            exceptions: HashMap::new(),
            synset_words: HashMap::new(),
        }
    }

    /// Look up synsets for a word
    pub fn lookup_word(&self, word: &str, pos: PartOfSpeech) -> Option<&IndexEntry> {
        self.index.get(&(word.to_lowercase(), pos))
    }

    /// Get synset by offset
    pub fn get_synset(&self, offset: usize) -> Option<&Synset> {
        self.synsets.get(&offset)
    }

    /// Get all synsets for a word
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
    pub fn get_hypernyms(&self, synset: &Synset) -> Vec<&Synset> {
        synset
            .get_relations(&SemanticRelation::Hypernym)
            .iter()
            .filter_map(|ptr| self.synsets.get(&ptr.target_offset))
            .collect()
    }

    /// Get hyponyms (more specific terms) for a synset
    pub fn get_hyponyms(&self, synset: &Synset) -> Vec<&Synset> {
        synset
            .get_relations(&SemanticRelation::Hyponym)
            .iter()
            .filter_map(|ptr| self.synsets.get(&ptr.target_offset))
            .collect()
    }

    /// Get instance hypernyms (classes of this instance) for a synset
    pub fn get_instance_hypernyms(&self, synset: &Synset) -> Vec<&Synset> {
        synset
            .get_relations(&SemanticRelation::InstanceHypernym)
            .iter()
            .filter_map(|ptr| self.synsets.get(&ptr.target_offset))
            .collect()
    }

    /// Get instance hyponyms (instances of this class) for a synset
    pub fn get_instance_hyponyms(&self, synset: &Synset) -> Vec<&Synset> {
        synset
            .get_relations(&SemanticRelation::InstanceHyponym)
            .iter()
            .filter_map(|ptr| self.synsets.get(&ptr.target_offset))
            .collect()
    }

    /// Find the lowest common hypernym of two synsets
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
    pub fn path_similarity(&self, synset1: &Synset, synset2: &Synset) -> f32 {
        if synset1.offset == synset2.offset {
            return 1.0;
        }

        if let Some(_lch) = self.lowest_common_hypernym(synset1, synset2) {
            // Simplified path similarity calculation
            // In a full implementation, this would calculate the actual path distance
            0.5 // Placeholder value
        } else {
            0.0
        }
    }

    /// Get database statistics
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
        let adj_synsets = self
            .synsets
            .values()
            .filter(|s| {
                matches!(
                    s.pos,
                    PartOfSpeech::Adjective | PartOfSpeech::AdjectiveSatellite
                )
            })
            .count();
        let adv_synsets = self
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
            adjective_synsets: adj_synsets,
            adverb_synsets: adv_synsets,
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

/// Analysis result from WordNet engine
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
    pub fn has_results(&self) -> bool {
        !self.synsets.is_empty()
    }

    /// Get the primary definition (from first synset)
    pub fn primary_definition(&self) -> Option<&String> {
        self.definitions.first()
    }
}
