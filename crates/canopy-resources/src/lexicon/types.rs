//! Type definitions for the Canopy Lexicon
//!
//! This module contains comprehensive type definitions for lexical classification,
//! pattern matching, and discourse analysis of closed-class words and functional items.

use crate::engine::count_to_f32;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Types of word classes in the lexicon
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WordClassType {
    /// Stop words and basic function words
    StopWords,
    /// Negation words and patterns
    Negation,
    /// Discourse markers and connectives
    DiscourseMarkers,
    /// Quantifiers and determiners
    Quantifiers,
    /// Temporal expressions
    Temporal,
    /// Modal verbs (can, could, will, would, shall, should, may, might, must)
    Modal,
    /// Auxiliary verbs (be, have, do and their forms)
    Auxiliary,
    /// Pronouns (personal, indefinite, reflexive, etc.)
    Pronouns,
    /// Prepositions
    Prepositions,
    /// Conjunctions
    Conjunctions,
    /// Intensifiers and degree modifiers
    Intensifiers,
    /// Hedge words and uncertainty markers
    HedgeWords,
    /// Sentiment indicators
    Sentiment,
    /// Articles and determiners
    Determiners,
    /// Wh-words (who, what, where, etc.)
    WhWords,
    /// Adverbs (frequency, manner, etc.)
    Adverbs,
    /// Interjections
    Interjections,
    /// Particles
    Particles,
    /// Comparison markers
    ComparisonMarkers,
    /// Expletives (existential "there", etc.)
    Expletives,
    /// Politeness markers
    PolitenessMarkers,
    /// Other functional words
    Functional,
}

/// Grammatical person for pronouns
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Person {
    First,
    Second,
    Third,
}

/// Grammatical case for pronouns
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PronounCase {
    Nominative, // I, he, she, we, they
    Accusative, // me, him, her, us, them
    Genitive,   // my, his, her, our, their (possessive)
    Reflexive,  // myself, himself, herself
}

/// Features extracted from pronoun classification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PronounFeatures {
    /// Grammatical person (first, second, third)
    pub person: Option<Person>,
    /// Number (singular, plural)
    pub number: Option<PronounNumber>,
    /// Gender (masculine, feminine, neuter, unknown)
    pub gender: Option<PronounGender>,
    /// Case (nominative, accusative, genitive, reflexive)
    pub case: Option<PronounCase>,
}

/// Number feature for pronouns
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PronounNumber {
    Singular,
    Plural,
}

/// Gender feature for pronouns
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PronounGender {
    Masculine,
    Feminine,
    Neuter,
    Unknown,
}

impl WordClassType {
    /// Get string representation
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            WordClassType::StopWords => "stop-words",
            WordClassType::Negation => "negation",
            WordClassType::DiscourseMarkers => "discourse-markers",
            WordClassType::Quantifiers => "quantifiers",
            WordClassType::Temporal => "temporal",
            WordClassType::Modal => "modal",
            WordClassType::Auxiliary => "auxiliary",
            WordClassType::Pronouns => "pronouns",
            WordClassType::Prepositions => "prepositions",
            WordClassType::Conjunctions => "conjunctions",
            WordClassType::Intensifiers => "intensifiers",
            WordClassType::HedgeWords => "hedge-words",
            WordClassType::Sentiment => "sentiment",
            WordClassType::Determiners => "determiners",
            WordClassType::WhWords => "wh-words",
            WordClassType::Adverbs => "adverbs",
            WordClassType::Interjections => "interjections",
            WordClassType::Particles => "particles",
            WordClassType::ComparisonMarkers => "comparison-markers",
            WordClassType::Expletives => "expletives",
            WordClassType::PolitenessMarkers => "politeness-markers",
            WordClassType::Functional => "functional",
        }
    }

    /// Parse from string representation
    #[must_use]
    pub fn parse_str(s: &str) -> Option<Self> {
        match s {
            "stop-words" => Some(WordClassType::StopWords),
            "negation" => Some(WordClassType::Negation),
            "discourse-markers" => Some(WordClassType::DiscourseMarkers),
            "quantifiers" => Some(WordClassType::Quantifiers),
            "temporal" => Some(WordClassType::Temporal),
            "modal" | "modal-verbs" => Some(WordClassType::Modal),
            "auxiliary" | "auxiliary-verbs" => Some(WordClassType::Auxiliary),
            "pronouns"
            | "personal-pronouns"
            | "indefinite-pronouns"
            | "reflexive-reciprocal"
            | "demonstratives"
            | "possessive" => Some(WordClassType::Pronouns),
            "prepositions" => Some(WordClassType::Prepositions),
            "conjunctions" => Some(WordClassType::Conjunctions),
            "intensifiers" => Some(WordClassType::Intensifiers),
            "hedge-words" => Some(WordClassType::HedgeWords),
            "sentiment" => Some(WordClassType::Sentiment),
            "articles-determiners" | "determiners" => Some(WordClassType::Determiners),
            "wh-words" => Some(WordClassType::WhWords),
            "frequency-adverbs" | "adverbs" => Some(WordClassType::Adverbs),
            "interjections" => Some(WordClassType::Interjections),
            "particles" => Some(WordClassType::Particles),
            "comparison-markers" => Some(WordClassType::ComparisonMarkers),
            "existential-expletive" | "expletives" => Some(WordClassType::Expletives),
            "politeness-markers" => Some(WordClassType::PolitenessMarkers),
            "functional" => Some(WordClassType::Functional),
            _ => None,
        }
    }
}

/// Pattern types for morphological analysis
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PatternType {
    /// Prefix pattern (e.g., un-, dis-)
    Prefix,
    /// Suffix pattern (e.g., -less, -ness)
    Suffix,
    /// Infix pattern (rare in English)
    Infix,
    /// Whole word pattern
    WholeWord,
    /// Multi-word phrase pattern
    Phrase,
}

impl PatternType {
    /// Get string representation
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            PatternType::Prefix => "prefix",
            PatternType::Suffix => "suffix",
            PatternType::Infix => "infix",
            PatternType::WholeWord => "whole-word",
            PatternType::Phrase => "phrase",
        }
    }

    /// Parse from string representation
    #[must_use]
    pub fn parse_str(s: &str) -> Option<Self> {
        match s {
            "prefix" => Some(PatternType::Prefix),
            "suffix" => Some(PatternType::Suffix),
            "infix" => Some(PatternType::Infix),
            "whole-word" => Some(PatternType::WholeWord),
            "phrase" => Some(PatternType::Phrase),
            _ => None,
        }
    }
}

/// Property value types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PropertyValue {
    String(String),
    Boolean(bool),
    Integer(i64),
    Float(f64),
}

impl PropertyValue {
    /// Get as string if possible
    #[must_use]
    pub fn as_string(&self) -> Option<&str> {
        match self {
            PropertyValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// Get as boolean if possible
    #[must_use]
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            PropertyValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    /// Get as integer if possible
    #[must_use]
    pub fn as_int(&self) -> Option<i64> {
        match self {
            PropertyValue::Integer(i) => Some(*i),
            _ => None,
        }
    }

    /// Get as float if possible
    #[must_use]
    pub fn as_float(&self) -> Option<f64> {
        match self {
            PropertyValue::Float(f) => Some(*f),
            _ => None,
        }
    }
}

/// Individual word entry in a word class
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LexiconWord {
    /// The word form
    pub word: String,
    /// Alternative forms or variants
    pub variants: Vec<String>,
    /// Part-of-speech tag
    pub pos: Option<String>,
    /// Confidence score (0.0-1.0)
    pub confidence: f32,
    /// Usage frequency (if available)
    pub frequency: Option<u32>,
    /// Semantic or pragmatic context
    pub context: Option<String>,
}

impl LexiconWord {
    /// Create a new lexicon word
    #[must_use]
    pub fn new(word: String) -> Self {
        Self {
            word,
            variants: Vec::new(),
            pos: None,
            confidence: 1.0,
            frequency: None,
            context: None,
        }
    }

    /// Check if this word matches a given string (including variants)
    #[must_use]
    pub fn matches(&self, input: &str) -> bool {
        let input_lower = input.to_lowercase();
        let word_lower = self.word.to_lowercase();

        if word_lower == input_lower {
            return true;
        }

        self.variants
            .iter()
            .any(|variant| variant.to_lowercase() == input_lower)
    }
}

/// Pattern for morphological analysis
#[derive(Debug, Clone)]
pub struct LexiconPattern {
    /// Pattern identifier
    pub id: String,
    /// Pattern type
    pub pattern_type: PatternType,
    /// Regular expression pattern
    pub regex: Regex,
    /// Raw regex string (for serialization)
    pub regex_str: String,
    /// Description of the pattern
    pub description: String,
    /// Confidence score for matches
    pub confidence: f32,
    /// Example words that match this pattern
    pub examples: Vec<String>,
}

impl Serialize for LexiconPattern {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("LexiconPattern", 7)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("pattern_type", &self.pattern_type)?;
        state.serialize_field("regex_str", &self.regex_str)?;
        state.serialize_field("description", &self.description)?;
        state.serialize_field("confidence", &self.confidence)?;
        state.serialize_field("examples", &self.examples)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for LexiconPattern {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};
        use std::fmt;

        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            Id,
            PatternType,
            RegexStr,
            Description,
            Confidence,
            Examples,
        }

        struct LexiconPatternVisitor;

        impl<'de> Visitor<'de> for LexiconPatternVisitor {
            type Value = LexiconPattern;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct LexiconPattern")
            }

            fn visit_map<V>(self, mut map: V) -> Result<LexiconPattern, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut id = None;
                let mut pattern_type = None;
                let mut regex_str: Option<String> = None;
                let mut description = None;
                let mut confidence = None;
                let mut examples = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Id => {
                            if id.is_some() {
                                return Err(de::Error::duplicate_field("id"));
                            }
                            id = Some(map.next_value()?);
                        }
                        Field::PatternType => {
                            if pattern_type.is_some() {
                                return Err(de::Error::duplicate_field("pattern_type"));
                            }
                            pattern_type = Some(map.next_value()?);
                        }
                        Field::RegexStr => {
                            if regex_str.is_some() {
                                return Err(de::Error::duplicate_field("regex_str"));
                            }
                            regex_str = Some(map.next_value::<String>()?);
                        }
                        Field::Description => {
                            if description.is_some() {
                                return Err(de::Error::duplicate_field("description"));
                            }
                            description = Some(map.next_value()?);
                        }
                        Field::Confidence => {
                            if confidence.is_some() {
                                return Err(de::Error::duplicate_field("confidence"));
                            }
                            confidence = Some(map.next_value()?);
                        }
                        Field::Examples => {
                            if examples.is_some() {
                                return Err(de::Error::duplicate_field("examples"));
                            }
                            examples = Some(map.next_value()?);
                        }
                    }
                }

                let id = id.ok_or_else(|| de::Error::missing_field("id"))?;
                let pattern_type =
                    pattern_type.ok_or_else(|| de::Error::missing_field("pattern_type"))?;
                let regex_str = regex_str.ok_or_else(|| de::Error::missing_field("regex_str"))?;
                let description =
                    description.ok_or_else(|| de::Error::missing_field("description"))?;
                let confidence = confidence.unwrap_or(0.8);
                let examples = examples.unwrap_or_default();

                let regex = Regex::new(&regex_str)
                    .map_err(|e| de::Error::custom(format!("Invalid regex: {e}")))?;

                Ok(LexiconPattern {
                    id,
                    pattern_type,
                    regex,
                    regex_str,
                    description,
                    confidence,
                    examples,
                })
            }
        }

        const FIELDS: &[&str] = &[
            "id",
            "pattern_type",
            "regex_str",
            "description",
            "confidence",
            "examples",
        ];
        deserializer.deserialize_struct("LexiconPattern", FIELDS, LexiconPatternVisitor)
    }
}

impl LexiconPattern {
    /// Create a new pattern
    ///
    /// # Errors
    /// Returns an error if the regex pattern is invalid.
    pub fn new(
        id: String,
        pattern_type: PatternType,
        regex_str: String,
        description: String,
    ) -> Result<Self, regex::Error> {
        let regex = Regex::new(&regex_str)?;

        Ok(Self {
            id,
            pattern_type,
            regex,
            regex_str,
            description,
            confidence: 0.8,
            examples: Vec::new(),
        })
    }

    /// Check if this pattern matches a word
    #[must_use]
    pub fn matches(&self, word: &str) -> bool {
        self.regex.is_match(word)
    }

    /// Extract the matched portion of the word
    #[must_use]
    pub fn extract_match(&self, word: &str) -> Option<String> {
        self.regex.find(word).map(|m| m.as_str().to_string())
    }
}

/// Word class definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordClass {
    /// Unique identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Type of word class
    pub word_class_type: WordClassType,
    /// Description of the word class
    pub description: String,
    /// Priority for classification (higher = more important)
    pub priority: u8,
    /// Properties for this word class
    pub properties: HashMap<String, PropertyValue>,
    /// Words in this class
    pub words: Vec<LexiconWord>,
    /// Patterns for morphological matching
    pub patterns: Vec<LexiconPattern>,
}

impl WordClass {
    /// Create a new word class
    #[must_use]
    pub fn new(
        id: String,
        name: String,
        word_class_type: WordClassType,
        description: String,
    ) -> Self {
        Self {
            id,
            name,
            word_class_type,
            description,
            priority: 1,
            properties: HashMap::new(),
            words: Vec::new(),
            patterns: Vec::new(),
        }
    }

    /// Check if a word belongs to this class
    #[must_use]
    pub fn contains_word(&self, word: &str) -> Option<&LexiconWord> {
        self.words.iter().find(|w| w.matches(word))
    }

    /// Check if a word matches any patterns in this class
    #[must_use]
    pub fn matches_pattern(&self, word: &str) -> Vec<&LexiconPattern> {
        self.patterns.iter().filter(|p| p.matches(word)).collect()
    }

    /// Get property value by name
    #[must_use]
    pub fn get_property(&self, name: &str) -> Option<&PropertyValue> {
        self.properties.get(name)
    }

    /// Check if this is a stop word class
    #[must_use]
    pub fn is_stop_words(&self) -> bool {
        matches!(self.word_class_type, WordClassType::StopWords)
    }

    /// Check if this class modifies polarity
    #[must_use]
    pub fn modifies_polarity(&self) -> bool {
        matches!(self.word_class_type, WordClassType::Negation)
    }

    /// Check if this class provides discourse structure
    #[must_use]
    pub fn provides_discourse_structure(&self) -> bool {
        matches!(self.word_class_type, WordClassType::DiscourseMarkers)
    }
}

/// Complete lexicon database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LexiconDatabase {
    /// Metadata about the lexicon
    pub title: String,
    pub description: String,
    pub version: String,
    pub language: String,
    pub created: String,
    pub author: String,
    pub license: String,

    /// Word classes in the lexicon
    pub word_classes: Vec<WordClass>,

    /// Fast lookup by word class type
    pub type_index: HashMap<WordClassType, Vec<usize>>,

    /// Fast lookup by word
    pub word_index: HashMap<String, Vec<(usize, usize)>>, // (class_index, word_index)
}

impl LexiconDatabase {
    /// Create a new empty lexicon database
    #[must_use]
    pub fn new() -> Self {
        Self {
            title: String::new(),
            description: String::new(),
            version: "1.0".to_string(),
            language: "en".to_string(),
            created: String::new(),
            author: String::new(),
            license: String::new(),
            word_classes: Vec::new(),
            type_index: HashMap::new(),
            word_index: HashMap::new(),
        }
    }

    /// Build indices for fast lookup
    pub fn build_indices(&mut self) {
        self.type_index.clear();
        self.word_index.clear();

        for (class_idx, word_class) in self.word_classes.iter().enumerate() {
            // Build type index
            self.type_index
                .entry(word_class.word_class_type)
                .or_default()
                .push(class_idx);

            // Build word index
            for (word_idx, word) in word_class.words.iter().enumerate() {
                // Index main word
                self.word_index
                    .entry(word.word.to_lowercase())
                    .or_default()
                    .push((class_idx, word_idx));

                // Index variants
                for variant in &word.variants {
                    self.word_index
                        .entry(variant.to_lowercase())
                        .or_default()
                        .push((class_idx, word_idx));
                }
            }
        }
    }

    /// Classify a word by looking up exact matches
    #[must_use]
    pub fn classify_word(&self, word: &str) -> Vec<WordClassification> {
        let word_lower = word.to_lowercase();
        let mut classifications = Vec::new();

        if let Some(indices) = self.word_index.get(&word_lower) {
            for &(class_idx, word_idx) in indices {
                if let Some(word_class) = self.word_classes.get(class_idx) {
                    if let Some(lexicon_word) = word_class.words.get(word_idx) {
                        classifications.push(WordClassification {
                            word_class_type: word_class.word_class_type,
                            word_class_id: word_class.id.clone(),
                            word_class_name: word_class.name.clone(),
                            matched_word: lexicon_word.word.clone(),
                            input_word: word.to_string(),
                            confidence: lexicon_word.confidence,
                            classification_type: ClassificationType::ExactMatch,
                            context: lexicon_word.context.clone(),
                            properties: word_class.properties.clone(),
                        });
                    }
                }
            }
        }

        // Sort by priority (higher priority first)
        classifications.sort_by(|a, b| {
            let a_priority = self.get_class_priority(&a.word_class_id);
            let b_priority = self.get_class_priority(&b.word_class_id);
            b_priority.cmp(&a_priority)
        });

        classifications
    }

    /// Analyze patterns in a word
    #[must_use]
    pub fn analyze_patterns(&self, word: &str) -> Vec<PatternMatch> {
        let mut matches = Vec::new();

        for word_class in &self.word_classes {
            for pattern in &word_class.patterns {
                if pattern.matches(word) {
                    if let Some(matched_text) = pattern.extract_match(word) {
                        matches.push(PatternMatch {
                            word_class_type: word_class.word_class_type,
                            word_class_id: word_class.id.clone(),
                            pattern_id: pattern.id.clone(),
                            pattern_type: pattern.pattern_type.clone(),
                            input_word: word.to_string(),
                            matched_text,
                            confidence: pattern.confidence,
                            description: pattern.description.clone(),
                        });
                    }
                }
            }
        }

        // Sort by confidence (higher confidence first)
        matches.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        matches
    }

    /// Get word classes by type
    #[must_use]
    pub fn get_classes_by_type(&self, class_type: &WordClassType) -> Vec<&WordClass> {
        if let Some(indices) = self.type_index.get(class_type) {
            indices
                .iter()
                .filter_map(|&idx| self.word_classes.get(idx))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get class priority by ID
    fn get_class_priority(&self, class_id: &str) -> u8 {
        self.word_classes
            .iter()
            .find(|wc| wc.id == class_id)
            .map_or(0, |wc| wc.priority)
    }

    /// Get database statistics
    #[must_use]
    pub fn stats(&self) -> LexiconStats {
        let total_words: usize = self.word_classes.iter().map(|wc| wc.words.len()).sum();
        let total_patterns: usize = self.word_classes.iter().map(|wc| wc.patterns.len()).sum();

        let mut by_type = HashMap::new();
        for word_class in &self.word_classes {
            *by_type.entry(word_class.word_class_type).or_insert(0) += word_class.words.len();
        }

        LexiconStats {
            total_word_classes: self.word_classes.len(),
            total_words,
            total_patterns,
            words_by_type: by_type,
        }
    }
}

impl Default for LexiconDatabase {
    fn default() -> Self {
        Self::new()
    }
}

/// Classification types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ClassificationType {
    /// Exact word match
    ExactMatch,
    /// Pattern-based match
    PatternMatch,
    /// Fuzzy/probabilistic match
    FuzzyMatch,
}

/// Result of word classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordClassification {
    /// Type of word class
    pub word_class_type: WordClassType,
    /// Word class identifier
    pub word_class_id: String,
    /// Word class name
    pub word_class_name: String,
    /// The word that matched from the lexicon
    pub matched_word: String,
    /// The input word that was classified
    pub input_word: String,
    /// Confidence score
    pub confidence: f32,
    /// How the classification was made
    pub classification_type: ClassificationType,
    /// Semantic or pragmatic context
    pub context: Option<String>,
    /// Properties from the word class
    pub properties: HashMap<String, PropertyValue>,
}

impl WordClassification {
    /// Check if this is a negation word
    #[must_use]
    pub fn is_negation(&self) -> bool {
        matches!(self.word_class_type, WordClassType::Negation)
    }

    /// Check if this is a stop word
    #[must_use]
    pub fn is_stop_word(&self) -> bool {
        matches!(self.word_class_type, WordClassType::StopWords)
    }

    /// Check if this is a discourse marker
    #[must_use]
    pub fn is_discourse_marker(&self) -> bool {
        matches!(self.word_class_type, WordClassType::DiscourseMarkers)
    }

    /// Check if this is a quantifier
    #[must_use]
    pub fn is_quantifier(&self) -> bool {
        matches!(self.word_class_type, WordClassType::Quantifiers)
    }

    /// Get semantic weight (for stop words)
    #[must_use]
    pub fn semantic_weight(&self) -> f64 {
        if let Some(PropertyValue::Float(weight)) = self.properties.get("semantic-weight") {
            *weight
        } else {
            1.0 // Default weight
        }
    }
}

/// Result of pattern matching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternMatch {
    /// Type of word class
    pub word_class_type: WordClassType,
    /// Word class identifier
    pub word_class_id: String,
    /// Pattern identifier
    pub pattern_id: String,
    /// Type of pattern
    pub pattern_type: PatternType,
    /// The input word
    pub input_word: String,
    /// The part of the word that matched
    pub matched_text: String,
    /// Confidence score
    pub confidence: f32,
    /// Pattern description
    pub description: String,
}

/// Analysis result from lexicon engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LexiconAnalysis {
    /// Input text analyzed
    pub input: String,
    /// Word classifications found
    pub classifications: Vec<WordClassification>,
    /// Pattern matches found
    pub pattern_matches: Vec<PatternMatch>,
    /// Overall confidence score
    pub confidence: f32,
}

impl LexiconAnalysis {
    /// Create a new analysis result
    #[must_use]
    pub fn new(input: String) -> Self {
        Self {
            input,
            classifications: Vec::new(),
            pattern_matches: Vec::new(),
            confidence: 0.0,
        }
    }

    /// Check if any results were found
    #[must_use]
    pub fn has_results(&self) -> bool {
        !self.classifications.is_empty() || !self.pattern_matches.is_empty()
    }

    /// Get all negation indicators
    #[must_use]
    pub fn get_negations(&self) -> Vec<&WordClassification> {
        self.classifications
            .iter()
            .filter(|c| c.is_negation())
            .collect()
    }

    /// Get all stop words
    #[must_use]
    pub fn get_stop_words(&self) -> Vec<&WordClassification> {
        self.classifications
            .iter()
            .filter(|c| c.is_stop_word())
            .collect()
    }

    /// Get all discourse markers
    #[must_use]
    pub fn get_discourse_markers(&self) -> Vec<&WordClassification> {
        self.classifications
            .iter()
            .filter(|c| c.is_discourse_marker())
            .collect()
    }

    /// Calculate combined confidence
    pub fn calculate_confidence(&mut self) {
        if self.classifications.is_empty() && self.pattern_matches.is_empty() {
            self.confidence = 0.0;
            return;
        }

        let classification_conf: f32 = self.classifications.iter().map(|c| c.confidence).sum();
        let pattern_conf: f32 = self.pattern_matches.iter().map(|p| p.confidence).sum();
        let total_items = count_to_f32(self.classifications.len() + self.pattern_matches.len());

        self.confidence = (classification_conf + pattern_conf) / total_items;
    }
}

/// Lexicon database statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LexiconStats {
    pub total_word_classes: usize,
    pub total_words: usize,
    pub total_patterns: usize,
    pub words_by_type: HashMap<WordClassType, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // === WordClassType Tests ===

    #[test]
    fn test_word_class_type_as_str() {
        assert_eq!(WordClassType::StopWords.as_str(), "stop-words");
        assert_eq!(WordClassType::Negation.as_str(), "negation");
        assert_eq!(
            WordClassType::DiscourseMarkers.as_str(),
            "discourse-markers"
        );
        assert_eq!(WordClassType::Quantifiers.as_str(), "quantifiers");
        assert_eq!(WordClassType::Temporal.as_str(), "temporal");
        assert_eq!(WordClassType::Modal.as_str(), "modal");
        assert_eq!(WordClassType::Pronouns.as_str(), "pronouns");
        assert_eq!(WordClassType::Prepositions.as_str(), "prepositions");
        assert_eq!(WordClassType::Conjunctions.as_str(), "conjunctions");
        assert_eq!(WordClassType::Intensifiers.as_str(), "intensifiers");
        assert_eq!(WordClassType::HedgeWords.as_str(), "hedge-words");
        assert_eq!(WordClassType::Sentiment.as_str(), "sentiment");
        assert_eq!(WordClassType::Functional.as_str(), "functional");
    }

    #[test]
    fn test_word_class_type_parse_str() {
        assert_eq!(
            WordClassType::parse_str("stop-words"),
            Some(WordClassType::StopWords)
        );
        assert_eq!(
            WordClassType::parse_str("negation"),
            Some(WordClassType::Negation)
        );
        assert_eq!(
            WordClassType::parse_str("modal"),
            Some(WordClassType::Modal)
        );
        // Test alias
        assert_eq!(
            WordClassType::parse_str("modal-verbs"),
            Some(WordClassType::Modal)
        );
        // Test invalid
        assert_eq!(WordClassType::parse_str("invalid"), None);
        assert_eq!(WordClassType::parse_str(""), None);
    }

    // === PatternType Tests ===

    #[test]
    fn test_pattern_type_as_str() {
        assert_eq!(PatternType::Prefix.as_str(), "prefix");
        assert_eq!(PatternType::Suffix.as_str(), "suffix");
        assert_eq!(PatternType::Infix.as_str(), "infix");
        assert_eq!(PatternType::WholeWord.as_str(), "whole-word");
        assert_eq!(PatternType::Phrase.as_str(), "phrase");
    }

    #[test]
    fn test_pattern_type_parse_str() {
        assert_eq!(PatternType::parse_str("prefix"), Some(PatternType::Prefix));
        assert_eq!(PatternType::parse_str("suffix"), Some(PatternType::Suffix));
        assert_eq!(PatternType::parse_str("infix"), Some(PatternType::Infix));
        assert_eq!(
            PatternType::parse_str("whole-word"),
            Some(PatternType::WholeWord)
        );
        assert_eq!(PatternType::parse_str("phrase"), Some(PatternType::Phrase));
        assert_eq!(PatternType::parse_str("invalid"), None);
    }

    // === PropertyValue Tests ===

    #[test]
    fn test_property_value_as_string() {
        let pv = PropertyValue::String("test".to_string());
        assert_eq!(pv.as_string(), Some("test"));

        let pv = PropertyValue::Boolean(true);
        assert_eq!(pv.as_string(), None);
    }

    #[test]
    fn test_property_value_as_bool() {
        let pv = PropertyValue::Boolean(true);
        assert_eq!(pv.as_bool(), Some(true));

        let pv = PropertyValue::Boolean(false);
        assert_eq!(pv.as_bool(), Some(false));

        let pv = PropertyValue::String("true".to_string());
        assert_eq!(pv.as_bool(), None);
    }

    #[test]
    fn test_property_value_as_int() {
        let pv = PropertyValue::Integer(42);
        assert_eq!(pv.as_int(), Some(42));

        let pv = PropertyValue::Integer(-100);
        assert_eq!(pv.as_int(), Some(-100));

        let pv = PropertyValue::Float(42.0);
        assert_eq!(pv.as_int(), None);
    }

    #[test]
    fn test_property_value_as_float() {
        let pv = PropertyValue::Float(2.5);
        assert_eq!(pv.as_float(), Some(2.5));

        let pv = PropertyValue::Integer(42);
        assert_eq!(pv.as_float(), None);
    }

    // === LexiconWord Tests ===

    #[test]
    fn test_lexicon_word_new() {
        let word = LexiconWord::new("test".to_string());
        assert_eq!(word.word, "test");
        assert!(word.variants.is_empty());
        assert!(word.pos.is_none());
        assert!((word.confidence - 1.0).abs() < f32::EPSILON);
        assert!(word.frequency.is_none());
        assert!(word.context.is_none());
    }

    #[test]
    fn test_lexicon_word_matches() {
        let mut word = LexiconWord::new("run".to_string());
        word.variants = vec!["running".to_string(), "runs".to_string()];

        // Exact match
        assert!(word.matches("run"));
        // Case insensitive
        assert!(word.matches("RUN"));
        // Variant match
        assert!(word.matches("running"));
        assert!(word.matches("RUNS"));
        // No match
        assert!(!word.matches("ran"));
    }

    // === LexiconPattern Tests ===

    #[test]
    fn test_lexicon_pattern_new() {
        let pattern = LexiconPattern::new(
            "neg-prefix".to_string(),
            PatternType::Prefix,
            "^un".to_string(),
            "Negative prefix un-".to_string(),
        )
        .unwrap();

        assert_eq!(pattern.id, "neg-prefix");
        assert_eq!(pattern.pattern_type, PatternType::Prefix);
        assert_eq!(pattern.regex_str, "^un");
        assert_eq!(pattern.description, "Negative prefix un-");
        assert!((pattern.confidence - 0.8).abs() < f32::EPSILON);
        assert!(pattern.examples.is_empty());
    }

    #[test]
    fn test_lexicon_pattern_new_invalid_regex() {
        let result = LexiconPattern::new(
            "bad".to_string(),
            PatternType::Prefix,
            "[invalid".to_string(),
            "Bad regex".to_string(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_lexicon_pattern_matches() {
        let pattern = LexiconPattern::new(
            "ing-suffix".to_string(),
            PatternType::Suffix,
            "ing$".to_string(),
            "Gerund suffix".to_string(),
        )
        .unwrap();

        assert!(pattern.matches("running"));
        assert!(pattern.matches("walking"));
        assert!(!pattern.matches("run"));
        assert!(!pattern.matches("ingredient")); // ing not at end
    }

    #[test]
    fn test_lexicon_pattern_extract_match() {
        let pattern = LexiconPattern::new(
            "un-prefix".to_string(),
            PatternType::Prefix,
            "^un".to_string(),
            "Un- prefix".to_string(),
        )
        .unwrap();

        assert_eq!(pattern.extract_match("unhappy"), Some("un".to_string()));
        assert_eq!(pattern.extract_match("happy"), None);
    }

    // === WordClass Tests ===

    #[test]
    fn test_word_class_new() {
        let wc = WordClass::new(
            "neg-001".to_string(),
            "Negation Words".to_string(),
            WordClassType::Negation,
            "Words that negate".to_string(),
        );

        assert_eq!(wc.id, "neg-001");
        assert_eq!(wc.name, "Negation Words");
        assert_eq!(wc.word_class_type, WordClassType::Negation);
        assert_eq!(wc.description, "Words that negate");
        assert_eq!(wc.priority, 1);
        assert!(wc.properties.is_empty());
        assert!(wc.words.is_empty());
        assert!(wc.patterns.is_empty());
    }

    #[test]
    fn test_word_class_contains_word() {
        let mut wc = WordClass::new(
            "test".to_string(),
            "Test".to_string(),
            WordClassType::Negation,
            "Test class".to_string(),
        );
        wc.words.push(LexiconWord::new("not".to_string()));
        wc.words.push(LexiconWord::new("never".to_string()));

        assert!(wc.contains_word("not").is_some());
        assert!(wc.contains_word("NOT").is_some()); // case insensitive
        assert!(wc.contains_word("always").is_none());
    }

    #[test]
    fn test_word_class_is_stop_words() {
        let stop_class = WordClass::new(
            "stop".to_string(),
            "Stop Words".to_string(),
            WordClassType::StopWords,
            "Stop words".to_string(),
        );
        assert!(stop_class.is_stop_words());

        let neg_class = WordClass::new(
            "neg".to_string(),
            "Negation".to_string(),
            WordClassType::Negation,
            "Negation".to_string(),
        );
        assert!(!neg_class.is_stop_words());
    }

    #[test]
    fn test_word_class_modifies_polarity() {
        let neg_class = WordClass::new(
            "neg".to_string(),
            "Negation".to_string(),
            WordClassType::Negation,
            "Negation".to_string(),
        );
        assert!(neg_class.modifies_polarity());

        let stop_class = WordClass::new(
            "stop".to_string(),
            "Stop Words".to_string(),
            WordClassType::StopWords,
            "Stop words".to_string(),
        );
        assert!(!stop_class.modifies_polarity());
    }

    #[test]
    fn test_word_class_provides_discourse_structure() {
        let dm_class = WordClass::new(
            "dm".to_string(),
            "Discourse Markers".to_string(),
            WordClassType::DiscourseMarkers,
            "Markers".to_string(),
        );
        assert!(dm_class.provides_discourse_structure());

        let neg_class = WordClass::new(
            "neg".to_string(),
            "Negation".to_string(),
            WordClassType::Negation,
            "Negation".to_string(),
        );
        assert!(!neg_class.provides_discourse_structure());
    }

    // === LexiconDatabase Tests ===

    #[test]
    fn test_lexicon_database_new() {
        let db = LexiconDatabase::new();
        assert!(db.title.is_empty());
        assert_eq!(db.version, "1.0");
        assert_eq!(db.language, "en");
        assert!(db.word_classes.is_empty());
        assert!(db.type_index.is_empty());
        assert!(db.word_index.is_empty());
    }

    #[test]
    fn test_lexicon_database_default() {
        let db = LexiconDatabase::default();
        assert!(db.title.is_empty());
        assert_eq!(db.version, "1.0");
    }

    #[test]
    fn test_lexicon_database_build_indices() {
        let mut db = LexiconDatabase::new();

        let mut wc = WordClass::new(
            "neg".to_string(),
            "Negation".to_string(),
            WordClassType::Negation,
            "Negation words".to_string(),
        );
        wc.words.push(LexiconWord::new("not".to_string()));
        wc.words.push(LexiconWord::new("never".to_string()));
        db.word_classes.push(wc);

        db.build_indices();

        // Check type index
        assert!(db.type_index.contains_key(&WordClassType::Negation));
        assert_eq!(
            db.type_index.get(&WordClassType::Negation).unwrap().len(),
            1
        );

        // Check word index
        assert!(db.word_index.contains_key("not"));
        assert!(db.word_index.contains_key("never"));
    }

    #[test]
    fn test_lexicon_database_classify_word() {
        let mut db = LexiconDatabase::new();

        let mut wc = WordClass::new(
            "neg".to_string(),
            "Negation".to_string(),
            WordClassType::Negation,
            "Negation words".to_string(),
        );
        wc.words.push(LexiconWord::new("not".to_string()));
        db.word_classes.push(wc);
        db.build_indices();

        let classifications = db.classify_word("not");
        assert_eq!(classifications.len(), 1);
        assert_eq!(classifications[0].word_class_type, WordClassType::Negation);
        assert_eq!(classifications[0].matched_word, "not");

        let no_match = db.classify_word("happy");
        assert!(no_match.is_empty());
    }

    #[test]
    fn test_lexicon_database_stats() {
        let mut db = LexiconDatabase::new();

        let mut wc1 = WordClass::new(
            "neg".to_string(),
            "Negation".to_string(),
            WordClassType::Negation,
            "Negation".to_string(),
        );
        wc1.words.push(LexiconWord::new("not".to_string()));
        wc1.words.push(LexiconWord::new("never".to_string()));

        let mut wc2 = WordClass::new(
            "stop".to_string(),
            "Stop Words".to_string(),
            WordClassType::StopWords,
            "Stop words".to_string(),
        );
        wc2.words.push(LexiconWord::new("the".to_string()));

        db.word_classes.push(wc1);
        db.word_classes.push(wc2);

        let stats = db.stats();
        assert_eq!(stats.total_word_classes, 2);
        assert_eq!(stats.total_words, 3);
        assert_eq!(stats.total_patterns, 0);
        assert_eq!(stats.words_by_type.get(&WordClassType::Negation), Some(&2));
        assert_eq!(stats.words_by_type.get(&WordClassType::StopWords), Some(&1));
    }

    // === ClassificationType Tests ===

    #[test]
    fn test_classification_type_equality() {
        assert_eq!(
            ClassificationType::ExactMatch,
            ClassificationType::ExactMatch
        );
        assert_ne!(
            ClassificationType::ExactMatch,
            ClassificationType::PatternMatch
        );
        assert_ne!(
            ClassificationType::PatternMatch,
            ClassificationType::FuzzyMatch
        );
    }

    // === WordClassification Tests ===

    #[test]
    fn test_word_classification_is_negation() {
        let wc = WordClassification {
            word_class_type: WordClassType::Negation,
            word_class_id: "neg".to_string(),
            word_class_name: "Negation".to_string(),
            matched_word: "not".to_string(),
            input_word: "not".to_string(),
            confidence: 1.0,
            classification_type: ClassificationType::ExactMatch,
            context: None,
            properties: HashMap::new(),
        };
        assert!(wc.is_negation());
        assert!(!wc.is_stop_word());
    }

    #[test]
    fn test_word_classification_is_stop_word() {
        let wc = WordClassification {
            word_class_type: WordClassType::StopWords,
            word_class_id: "stop".to_string(),
            word_class_name: "Stop Words".to_string(),
            matched_word: "the".to_string(),
            input_word: "the".to_string(),
            confidence: 1.0,
            classification_type: ClassificationType::ExactMatch,
            context: None,
            properties: HashMap::new(),
        };
        assert!(wc.is_stop_word());
        assert!(!wc.is_negation());
    }

    #[test]
    fn test_word_classification_is_discourse_marker() {
        let wc = WordClassification {
            word_class_type: WordClassType::DiscourseMarkers,
            word_class_id: "dm".to_string(),
            word_class_name: "Discourse Markers".to_string(),
            matched_word: "however".to_string(),
            input_word: "however".to_string(),
            confidence: 1.0,
            classification_type: ClassificationType::ExactMatch,
            context: None,
            properties: HashMap::new(),
        };
        assert!(wc.is_discourse_marker());
    }

    #[test]
    fn test_word_classification_is_quantifier() {
        let wc = WordClassification {
            word_class_type: WordClassType::Quantifiers,
            word_class_id: "quant".to_string(),
            word_class_name: "Quantifiers".to_string(),
            matched_word: "all".to_string(),
            input_word: "all".to_string(),
            confidence: 1.0,
            classification_type: ClassificationType::ExactMatch,
            context: None,
            properties: HashMap::new(),
        };
        assert!(wc.is_quantifier());
    }

    #[test]
    fn test_word_classification_semantic_weight() {
        let mut properties = HashMap::new();
        properties.insert("semantic-weight".to_string(), PropertyValue::Float(0.5));

        let wc = WordClassification {
            word_class_type: WordClassType::StopWords,
            word_class_id: "stop".to_string(),
            word_class_name: "Stop Words".to_string(),
            matched_word: "the".to_string(),
            input_word: "the".to_string(),
            confidence: 1.0,
            classification_type: ClassificationType::ExactMatch,
            context: None,
            properties,
        };
        assert!((wc.semantic_weight() - 0.5).abs() < f64::EPSILON);

        // Default weight
        let wc_default = WordClassification {
            word_class_type: WordClassType::StopWords,
            word_class_id: "stop".to_string(),
            word_class_name: "Stop Words".to_string(),
            matched_word: "the".to_string(),
            input_word: "the".to_string(),
            confidence: 1.0,
            classification_type: ClassificationType::ExactMatch,
            context: None,
            properties: HashMap::new(),
        };
        assert!((wc_default.semantic_weight() - 1.0).abs() < f64::EPSILON);
    }

    // === LexiconAnalysis Tests ===

    #[test]
    fn test_lexicon_analysis_new() {
        let analysis = LexiconAnalysis::new("test input".to_string());
        assert_eq!(analysis.input, "test input");
        assert!(analysis.classifications.is_empty());
        assert!(analysis.pattern_matches.is_empty());
        assert!(analysis.confidence.abs() < f32::EPSILON);
    }

    #[test]
    fn test_lexicon_analysis_has_results() {
        let empty = LexiconAnalysis::new("test".to_string());
        assert!(!empty.has_results());

        let mut with_classification = LexiconAnalysis::new("not".to_string());
        with_classification
            .classifications
            .push(WordClassification {
                word_class_type: WordClassType::Negation,
                word_class_id: "neg".to_string(),
                word_class_name: "Negation".to_string(),
                matched_word: "not".to_string(),
                input_word: "not".to_string(),
                confidence: 1.0,
                classification_type: ClassificationType::ExactMatch,
                context: None,
                properties: HashMap::new(),
            });
        assert!(with_classification.has_results());
    }

    #[test]
    fn test_lexicon_analysis_get_negations() {
        let mut analysis = LexiconAnalysis::new("not good".to_string());
        analysis.classifications.push(WordClassification {
            word_class_type: WordClassType::Negation,
            word_class_id: "neg".to_string(),
            word_class_name: "Negation".to_string(),
            matched_word: "not".to_string(),
            input_word: "not".to_string(),
            confidence: 1.0,
            classification_type: ClassificationType::ExactMatch,
            context: None,
            properties: HashMap::new(),
        });
        analysis.classifications.push(WordClassification {
            word_class_type: WordClassType::Sentiment,
            word_class_id: "sent".to_string(),
            word_class_name: "Sentiment".to_string(),
            matched_word: "good".to_string(),
            input_word: "good".to_string(),
            confidence: 1.0,
            classification_type: ClassificationType::ExactMatch,
            context: None,
            properties: HashMap::new(),
        });

        let negations = analysis.get_negations();
        assert_eq!(negations.len(), 1);
        assert_eq!(negations[0].matched_word, "not");
    }

    #[test]
    fn test_lexicon_analysis_calculate_confidence() {
        let mut analysis = LexiconAnalysis::new("test".to_string());
        analysis.calculate_confidence();
        assert!(analysis.confidence.abs() < f32::EPSILON);

        analysis.classifications.push(WordClassification {
            word_class_type: WordClassType::Negation,
            word_class_id: "neg".to_string(),
            word_class_name: "Negation".to_string(),
            matched_word: "not".to_string(),
            input_word: "not".to_string(),
            confidence: 0.8,
            classification_type: ClassificationType::ExactMatch,
            context: None,
            properties: HashMap::new(),
        });
        analysis.classifications.push(WordClassification {
            word_class_type: WordClassType::StopWords,
            word_class_id: "stop".to_string(),
            word_class_name: "Stop".to_string(),
            matched_word: "the".to_string(),
            input_word: "the".to_string(),
            confidence: 1.0,
            classification_type: ClassificationType::ExactMatch,
            context: None,
            properties: HashMap::new(),
        });

        analysis.calculate_confidence();
        // (0.8 + 1.0) / 2 = 0.9
        assert!((analysis.confidence - 0.9).abs() < 0.001);
    }

    // === Serialization Tests ===

    #[test]
    fn test_word_class_type_serialization() {
        let wct = WordClassType::Modal;
        let json = serde_json::to_string(&wct).unwrap();
        let deserialized: WordClassType = serde_json::from_str(&json).unwrap();
        assert_eq!(wct, deserialized);
    }

    #[test]
    fn test_pattern_type_serialization() {
        let pt = PatternType::Suffix;
        let json = serde_json::to_string(&pt).unwrap();
        let deserialized: PatternType = serde_json::from_str(&json).unwrap();
        assert_eq!(pt, deserialized);
    }

    #[test]
    fn test_lexicon_word_serialization() {
        let word = LexiconWord::new("test".to_string());
        let json = serde_json::to_string(&word).unwrap();
        let deserialized: LexiconWord = serde_json::from_str(&json).unwrap();
        assert_eq!(word.word, deserialized.word);
    }
}
