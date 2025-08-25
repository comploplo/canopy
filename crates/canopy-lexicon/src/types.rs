//! Type definitions for the Canopy Lexicon
//!
//! This module contains comprehensive type definitions for lexical classification,
//! pattern matching, and discourse analysis of closed-class words and functional items.

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Types of word classes in the lexicon
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    /// Modal auxiliaries
    Modal,
    /// Pronouns
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
    /// Other functional words
    Functional,
}

impl WordClassType {
    /// Get string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            WordClassType::StopWords => "stop-words",
            WordClassType::Negation => "negation",
            WordClassType::DiscourseMarkers => "discourse-markers",
            WordClassType::Quantifiers => "quantifiers",
            WordClassType::Temporal => "temporal",
            WordClassType::Modal => "modal",
            WordClassType::Pronouns => "pronouns",
            WordClassType::Prepositions => "prepositions",
            WordClassType::Conjunctions => "conjunctions",
            WordClassType::Intensifiers => "intensifiers",
            WordClassType::HedgeWords => "hedge-words",
            WordClassType::Sentiment => "sentiment",
            WordClassType::Functional => "functional",
        }
    }

    /// Parse from string representation
    pub fn parse_str(s: &str) -> Option<Self> {
        match s {
            "stop-words" => Some(WordClassType::StopWords),
            "negation" => Some(WordClassType::Negation),
            "discourse-markers" => Some(WordClassType::DiscourseMarkers),
            "quantifiers" => Some(WordClassType::Quantifiers),
            "temporal" => Some(WordClassType::Temporal),
            "modal" => Some(WordClassType::Modal),
            "pronouns" => Some(WordClassType::Pronouns),
            "prepositions" => Some(WordClassType::Prepositions),
            "conjunctions" => Some(WordClassType::Conjunctions),
            "intensifiers" => Some(WordClassType::Intensifiers),
            "hedge-words" => Some(WordClassType::HedgeWords),
            "sentiment" => Some(WordClassType::Sentiment),
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
    pub fn as_string(&self) -> Option<&str> {
        match self {
            PropertyValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// Get as boolean if possible
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            PropertyValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    /// Get as integer if possible
    pub fn as_int(&self) -> Option<i64> {
        match self {
            PropertyValue::Integer(i) => Some(*i),
            _ => None,
        }
    }

    /// Get as float if possible
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
    pub fn matches(&self, word: &str) -> bool {
        self.regex.is_match(word)
    }

    /// Extract the matched portion of the word
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
    pub fn contains_word(&self, word: &str) -> Option<&LexiconWord> {
        self.words.iter().find(|w| w.matches(word))
    }

    /// Check if a word matches any patterns in this class
    pub fn matches_pattern(&self, word: &str) -> Vec<&LexiconPattern> {
        self.patterns.iter().filter(|p| p.matches(word)).collect()
    }

    /// Get property value by name
    pub fn get_property(&self, name: &str) -> Option<&PropertyValue> {
        self.properties.get(name)
    }

    /// Check if this is a stop word class
    pub fn is_stop_words(&self) -> bool {
        matches!(self.word_class_type, WordClassType::StopWords)
    }

    /// Check if this class modifies polarity
    pub fn modifies_polarity(&self) -> bool {
        matches!(self.word_class_type, WordClassType::Negation)
    }

    /// Check if this class provides discourse structure
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
                .entry(word_class.word_class_type.clone())
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
    pub fn classify_word(&self, word: &str) -> Vec<WordClassification> {
        let word_lower = word.to_lowercase();
        let mut classifications = Vec::new();

        if let Some(indices) = self.word_index.get(&word_lower) {
            for &(class_idx, word_idx) in indices {
                if let Some(word_class) = self.word_classes.get(class_idx) {
                    if let Some(lexicon_word) = word_class.words.get(word_idx) {
                        classifications.push(WordClassification {
                            word_class_type: word_class.word_class_type.clone(),
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
    pub fn analyze_patterns(&self, word: &str) -> Vec<PatternMatch> {
        let mut matches = Vec::new();

        for word_class in &self.word_classes {
            for pattern in &word_class.patterns {
                if pattern.matches(word) {
                    if let Some(matched_text) = pattern.extract_match(word) {
                        matches.push(PatternMatch {
                            word_class_type: word_class.word_class_type.clone(),
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
            .map(|wc| wc.priority)
            .unwrap_or(0)
    }

    /// Get database statistics
    pub fn stats(&self) -> LexiconStats {
        let total_words: usize = self.word_classes.iter().map(|wc| wc.words.len()).sum();
        let total_patterns: usize = self.word_classes.iter().map(|wc| wc.patterns.len()).sum();

        let mut by_type = HashMap::new();
        for word_class in &self.word_classes {
            *by_type
                .entry(word_class.word_class_type.clone())
                .or_insert(0) += word_class.words.len();
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
    pub fn is_negation(&self) -> bool {
        matches!(self.word_class_type, WordClassType::Negation)
    }

    /// Check if this is a stop word
    pub fn is_stop_word(&self) -> bool {
        matches!(self.word_class_type, WordClassType::StopWords)
    }

    /// Check if this is a discourse marker
    pub fn is_discourse_marker(&self) -> bool {
        matches!(self.word_class_type, WordClassType::DiscourseMarkers)
    }

    /// Check if this is a quantifier
    pub fn is_quantifier(&self) -> bool {
        matches!(self.word_class_type, WordClassType::Quantifiers)
    }

    /// Get semantic weight (for stop words)
    pub fn semantic_weight(&self) -> f32 {
        if let Some(PropertyValue::Float(weight)) = self.properties.get("semantic-weight") {
            *weight as f32
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
    pub fn new(input: String) -> Self {
        Self {
            input,
            classifications: Vec::new(),
            pattern_matches: Vec::new(),
            confidence: 0.0,
        }
    }

    /// Check if any results were found
    pub fn has_results(&self) -> bool {
        !self.classifications.is_empty() || !self.pattern_matches.is_empty()
    }

    /// Get all negation indicators
    pub fn get_negations(&self) -> Vec<&WordClassification> {
        self.classifications
            .iter()
            .filter(|c| c.is_negation())
            .collect()
    }

    /// Get all stop words
    pub fn get_stop_words(&self) -> Vec<&WordClassification> {
        self.classifications
            .iter()
            .filter(|c| c.is_stop_word())
            .collect()
    }

    /// Get all discourse markers
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
        let total_items = (self.classifications.len() + self.pattern_matches.len()) as f32;

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
