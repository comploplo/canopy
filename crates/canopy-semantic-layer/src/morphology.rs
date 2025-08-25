//! Morphological analysis database for lemmatization and inflection analysis
//!
//! This module provides morphological analysis capabilities,
//! including lemmatization and inflection pattern recognition.

use crate::{InflectionType, MorphologicalAnalysis, SemanticResult};
use std::collections::HashMap;
use tracing::{debug, info};

/// Morphological analysis database
pub struct MorphologyDatabase {
    // Inflection mappings: inflected form -> lemma
    verb_inflections: HashMap<String, String>,
    noun_inflections: HashMap<String, String>,
    adjective_inflections: HashMap<String, String>,

    // Feature mappings: word form -> features
    morphological_features: HashMap<String, HashMap<String, String>>,
}

impl MorphologyDatabase {
    /// Create a new morphology database with pre-loaded patterns
    pub fn new() -> SemanticResult<Self> {
        info!("Initializing morphology database");

        let mut verb_inflections = HashMap::new();
        let mut noun_inflections = HashMap::new();
        let mut adjective_inflections = HashMap::new();
        let mut morphological_features = HashMap::new();

        // Common verb inflections
        let verb_patterns = vec![
            // give -> gave, given, giving, gives
            ("gave", "give"),
            ("given", "give"),
            ("giving", "give"),
            ("gives", "give"),
            // walk -> walked, walking, walks
            ("walked", "walk"),
            ("walking", "walk"),
            ("walks", "walk"),
            // run -> ran, running, runs
            ("ran", "run"),
            ("running", "run"),
            ("runs", "run"),
            // love -> loved, loving, loves
            ("loved", "love"),
            ("loving", "love"),
            ("loves", "love"),
            // Regular patterns
            ("played", "play"),
            ("playing", "play"),
            ("plays", "play"),
            // Irregular copula forms that shouldn't be processed by regular rules
            ("is", "be"),
            ("am", "be"),
            ("are", "be"),
            ("was", "be"),
            ("were", "be"),
            ("seems", "seem"),
            ("wants", "want"),
        ];

        for (inflected, lemma) in verb_patterns {
            verb_inflections.insert(inflected.to_string(), lemma.to_string());
        }

        // Common noun inflections
        let noun_patterns = vec![
            // book -> books
            ("books", "book"),
            // Regular plurals
            ("cats", "cat"),
            ("dogs", "dog"),
            ("houses", "house"),
            ("boxes", "box"),
            // Irregular plurals
            ("children", "child"),
            ("mice", "mouse"),
            ("feet", "foot"),
            ("teeth", "tooth"),
            ("men", "man"),
            ("women", "woman"),
        ];

        for (inflected, lemma) in noun_patterns {
            noun_inflections.insert(inflected.to_string(), lemma.to_string());
        }

        // Adjective inflections
        let adjective_patterns = vec![
            // Comparative/superlative
            ("bigger", "big"),
            ("biggest", "big"),
            ("smaller", "small"),
            ("smallest", "small"),
            ("better", "good"),
            ("best", "good"),
            ("worse", "bad"),
            ("worst", "bad"),
            // Common adjectives that might be mistaken for past participles
            ("red", "red"),
            ("blue", "blue"),
            ("green", "green"),
            ("black", "black"),
            ("white", "white"),
        ];

        for (inflected, lemma) in adjective_patterns {
            adjective_inflections.insert(inflected.to_string(), lemma.to_string());
        }

        // Sample morphological features
        let feature_data = vec![
            ("give", vec![("pos", "verb"), ("tense", "present")]),
            ("gave", vec![("pos", "verb"), ("tense", "past")]),
            ("given", vec![("pos", "verb"), ("aspect", "perfect")]),
            ("giving", vec![("pos", "verb"), ("aspect", "progressive")]),
            (
                "gives",
                vec![
                    ("pos", "verb"),
                    ("tense", "present"),
                    ("person", "3"),
                    ("number", "singular"),
                ],
            ),
            ("book", vec![("pos", "noun"), ("number", "singular")]),
            ("books", vec![("pos", "noun"), ("number", "plural")]),
            (
                "john",
                vec![("pos", "noun"), ("proper", "true"), ("gender", "masculine")],
            ),
            (
                "mary",
                vec![("pos", "noun"), ("proper", "true"), ("gender", "feminine")],
            ),
            // Copula forms
            (
                "is",
                vec![
                    ("pos", "verb"),
                    ("tense", "present"),
                    ("person", "3"),
                    ("number", "singular"),
                ],
            ),
            (
                "am",
                vec![
                    ("pos", "verb"),
                    ("tense", "present"),
                    ("person", "1"),
                    ("number", "singular"),
                ],
            ),
            (
                "are",
                vec![("pos", "verb"), ("tense", "present"), ("number", "plural")],
            ),
            (
                "was",
                vec![("pos", "verb"), ("tense", "past"), ("number", "singular")],
            ),
            (
                "were",
                vec![("pos", "verb"), ("tense", "past"), ("number", "plural")],
            ),
            ("be", vec![("pos", "verb"), ("tense", "infinitive")]),
            (
                "seems",
                vec![
                    ("pos", "verb"),
                    ("tense", "present"),
                    ("person", "3"),
                    ("number", "singular"),
                ],
            ),
            (
                "wants",
                vec![
                    ("pos", "verb"),
                    ("tense", "present"),
                    ("person", "3"),
                    ("number", "singular"),
                ],
            ),
            // Common adjectives
            ("red", vec![("pos", "adjective")]),
            ("blue", vec![("pos", "adjective")]),
            ("green", vec![("pos", "adjective")]),
            ("black", vec![("pos", "adjective")]),
            ("white", vec![("pos", "adjective")]),
        ];

        for (word, features) in feature_data {
            let mut feature_map = HashMap::new();
            for (key, value) in features {
                feature_map.insert(key.to_string(), value.to_string());
            }
            morphological_features.insert(word.to_string(), feature_map);
        }

        Ok(Self {
            verb_inflections,
            noun_inflections,
            adjective_inflections,
            morphological_features,
        })
    }

    /// Analyze morphological properties of a word
    pub fn analyze(&self, word: &str) -> SemanticResult<MorphologicalAnalysis> {
        debug!("Analyzing morphological properties of: {}", word);

        let lowercase_word = word.to_lowercase();

        // Try to find lemma through inflection tables
        let (lemma, inflection_type) = self.find_lemma(&lowercase_word);

        // Get morphological features if available
        let features = self
            .morphological_features
            .get(&lowercase_word)
            .cloned()
            .unwrap_or_default();

        let is_recognized = self.is_recognized_word(&lowercase_word);

        Ok(MorphologicalAnalysis {
            lemma,
            features,
            inflection_type,
            is_recognized,
        })
    }

    /// Find lemma for a given word form
    fn find_lemma(&self, word: &str) -> (String, InflectionType) {
        // Check verb inflections
        if let Some(lemma) = self.verb_inflections.get(word) {
            return (lemma.clone(), InflectionType::Verbal);
        }

        // Check noun inflections
        if let Some(lemma) = self.noun_inflections.get(word) {
            return (lemma.clone(), InflectionType::Nominal);
        }

        // Check adjective inflections
        if let Some(lemma) = self.adjective_inflections.get(word) {
            return (lemma.clone(), InflectionType::Adjectival);
        }

        // Apply regular inflection rules
        if let Some((lemma, inflection_type)) = self.apply_regular_rules(word) {
            return (lemma, inflection_type);
        }

        // Default: word is its own lemma
        (word.to_string(), InflectionType::None)
    }

    /// Apply regular morphological rules for common patterns
    fn apply_regular_rules(&self, word: &str) -> Option<(String, InflectionType)> {
        // Regular verb patterns
        if word.ends_with("ed") && word.len() > 2 {
            let stem = &word[..word.len() - 2];
            return Some((stem.to_string(), InflectionType::Verbal));
        }

        if word.ends_with("ing") && word.len() > 3 {
            let stem = &word[..word.len() - 3];
            return Some((stem.to_string(), InflectionType::Verbal));
        }

        if word.ends_with("s") && word.len() > 1 {
            let stem = &word[..word.len() - 1];
            // Could be verb 3rd person singular or noun plural
            // Simple heuristic: if stem ends in consonant + y, it's likely plural
            if stem.ends_with("y") && stem.len() > 1 {
                let prev_char = stem.chars().nth(stem.len() - 2).unwrap_or('a');
                if !"aeiou".contains(prev_char) {
                    // Try "ies" -> "y" pattern (e.g., "flies" -> "fly")
                    return Some((
                        format!("{}y", &stem[..stem.len() - 1]),
                        InflectionType::Nominal,
                    ));
                }
            }
            return Some((stem.to_string(), InflectionType::Nominal));
        }

        // Regular comparative/superlative
        if word.ends_with("er") && word.len() > 2 {
            let stem = &word[..word.len() - 2];
            return Some((stem.to_string(), InflectionType::Adjectival));
        }

        if word.ends_with("est") && word.len() > 3 {
            let stem = &word[..word.len() - 3];
            return Some((stem.to_string(), InflectionType::Adjectival));
        }

        None
    }

    /// Check if a word is recognized in the database
    fn is_recognized_word(&self, word: &str) -> bool {
        self.verb_inflections.contains_key(word)
            || self.noun_inflections.contains_key(word)
            || self.adjective_inflections.contains_key(word)
            || self.morphological_features.contains_key(word)
    }

    /// Get all inflected forms of a lemma
    pub fn get_inflections(&self, lemma: &str) -> Vec<String> {
        let mut inflections = Vec::new();

        // Check verb inflections
        for (inflected, base) in &self.verb_inflections {
            if base == lemma {
                inflections.push(inflected.clone());
            }
        }

        // Check noun inflections
        for (inflected, base) in &self.noun_inflections {
            if base == lemma {
                inflections.push(inflected.clone());
            }
        }

        // Check adjective inflections
        for (inflected, base) in &self.adjective_inflections {
            if base == lemma {
                inflections.push(inflected.clone());
            }
        }

        inflections
    }

    /// Get morphological features for a specific word form
    pub fn get_features(&self, word: &str) -> HashMap<String, String> {
        self.morphological_features
            .get(&word.to_lowercase())
            .cloned()
            .unwrap_or_default()
    }

    /// Check if a word is a verb based on morphological analysis
    pub fn is_verb(&self, word: &str) -> bool {
        let features = self.get_features(word);
        features.get("pos") == Some(&"verb".to_string())
            || self.verb_inflections.contains_key(&word.to_lowercase())
    }

    /// Check if a word is a noun based on morphological analysis
    pub fn is_noun(&self, word: &str) -> bool {
        let features = self.get_features(word);
        features.get("pos") == Some(&"noun".to_string())
            || self.noun_inflections.contains_key(&word.to_lowercase())
    }

    /// Get statistics about the morphology database
    pub fn get_stats(&self) -> MorphologyStats {
        MorphologyStats {
            verb_inflections: self.verb_inflections.len(),
            noun_inflections: self.noun_inflections.len(),
            adjective_inflections: self.adjective_inflections.len(),
            total_features: self.morphological_features.len(),
        }
    }
}

/// Statistics about morphology database
#[derive(Debug, Clone)]
pub struct MorphologyStats {
    pub verb_inflections: usize,
    pub noun_inflections: usize,
    pub adjective_inflections: usize,
    pub total_features: usize,
}

impl Default for MorphologyDatabase {
    fn default() -> Self {
        Self::new().expect("Failed to initialize morphology database")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_morphology_database_creation() {
        let db = MorphologyDatabase::new().unwrap();
        let stats = db.get_stats();
        assert!(stats.verb_inflections > 0);
        assert!(stats.noun_inflections > 0);
    }

    #[test]
    fn test_verb_lemmatization() {
        let db = MorphologyDatabase::new().unwrap();

        let gave_analysis = db.analyze("gave").unwrap();
        assert_eq!(gave_analysis.lemma, "give");
        assert_eq!(gave_analysis.inflection_type, InflectionType::Verbal);

        let giving_analysis = db.analyze("giving").unwrap();
        assert_eq!(giving_analysis.lemma, "give");
        assert_eq!(giving_analysis.inflection_type, InflectionType::Verbal);
    }

    #[test]
    fn test_noun_lemmatization() {
        let db = MorphologyDatabase::new().unwrap();

        let books_analysis = db.analyze("books").unwrap();
        assert_eq!(books_analysis.lemma, "book");
        assert_eq!(books_analysis.inflection_type, InflectionType::Nominal);

        let children_analysis = db.analyze("children").unwrap();
        assert_eq!(children_analysis.lemma, "child");
        assert_eq!(children_analysis.inflection_type, InflectionType::Nominal);
    }

    #[test]
    fn test_regular_patterns() {
        let db = MorphologyDatabase::new().unwrap();

        // Test regular -ed pattern
        let played_analysis = db.analyze("played").unwrap();
        assert_eq!(played_analysis.lemma, "play");
        assert_eq!(played_analysis.inflection_type, InflectionType::Verbal);

        // Test regular -s pattern
        let cats_analysis = db.analyze("cats").unwrap();
        assert_eq!(cats_analysis.lemma, "cat");
        assert_eq!(cats_analysis.inflection_type, InflectionType::Nominal);
    }

    #[test]
    fn test_feature_extraction() {
        let db = MorphologyDatabase::new().unwrap();

        let give_features = db.get_features("give");
        assert_eq!(give_features.get("pos"), Some(&"verb".to_string()));

        let john_features = db.get_features("john");
        assert_eq!(john_features.get("proper"), Some(&"true".to_string()));
    }

    #[test]
    fn test_pos_detection() {
        let db = MorphologyDatabase::new().unwrap();

        assert!(db.is_verb("give"));
        assert!(db.is_verb("gave"));
        assert!(!db.is_verb("book"));

        assert!(db.is_noun("book"));
        assert!(db.is_noun("john"));
        assert!(!db.is_noun("give"));
    }

    #[test]
    fn test_inflection_retrieval() {
        let db = MorphologyDatabase::new().unwrap();

        let give_inflections = db.get_inflections("give");
        assert!(give_inflections.contains(&"gave".to_string()));
        assert!(give_inflections.contains(&"given".to_string()));
        assert!(give_inflections.contains(&"giving".to_string()));
    }

    #[test]
    fn test_unrecognized_words() {
        let db = MorphologyDatabase::new().unwrap();

        let unknown_analysis = db.analyze("unknownword").unwrap();
        assert_eq!(unknown_analysis.lemma, "unknownword");
        assert_eq!(unknown_analysis.inflection_type, InflectionType::None);
        assert!(!unknown_analysis.is_recognized);
    }
}
