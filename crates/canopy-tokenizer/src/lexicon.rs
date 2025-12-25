//! Closed-class lexicon for function words and grammatical particles
//!
//! This module provides a comprehensive database of closed-class words
//! (determiners, prepositions, conjunctions, etc.) that don't typically
//! appear in semantic databases like FrameNet or VerbNet.

use crate::SemanticResult;
use std::collections::HashSet;
use tracing::info;

/// Closed-class lexicon database
pub struct ClosedClassLexicon {
    determiners: HashSet<String>,
    prepositions: HashSet<String>,
    conjunctions: HashSet<String>,
    auxiliaries: HashSet<String>,
    pronouns: HashSet<String>,
    particles: HashSet<String>,
    quantifiers: HashSet<String>,
    wh_words: HashSet<String>,
}

impl ClosedClassLexicon {
    /// Create a new closed-class lexicon with standard English function words
    pub fn new() -> SemanticResult<Self> {
        info!("Initializing closed-class lexicon");

        // Determiners
        let determiners: HashSet<String> = vec![
            "the", "a", "an", "this", "that", "these", "those", "my", "your", "his", "her", "its",
            "our", "their", "some", "any", "no", "every", "each", "all", "both", "many", "much",
            "few", "little", "several", "most", "enough", "such", "what", "which", "whose",
        ]
        .into_iter()
        .map(String::from)
        .collect();

        // Prepositions
        let prepositions: HashSet<String> = vec![
            "in",
            "on",
            "at",
            "by",
            "for",
            "with",
            "without",
            "to",
            "from",
            "into",
            "onto",
            "upon",
            "under",
            "over",
            "above",
            "below",
            "through",
            "across",
            "around",
            "between",
            "among",
            "within",
            "during",
            "before",
            "after",
            "since",
            "until",
            "about",
            "against",
            "toward",
            "towards",
            "beside",
            "behind",
            "beyond",
            "beneath",
            "inside",
            "outside",
            "throughout",
            "underneath",
            "alongside",
            "amid",
            "amidst",
            "concerning",
            "regarding",
            "despite",
            "except",
            "excluding",
            "including",
            "plus",
            "minus",
            "via",
            "per",
            "pro",
            "anti",
            "off",
            "up",
            "down",
            "out",
        ]
        .into_iter()
        .map(String::from)
        .collect();

        // Conjunctions
        let conjunctions: HashSet<String> = vec![
            // Coordinating conjunctions
            "and",
            "or",
            "but",
            "nor",
            "for",
            "so",
            "yet",
            // Subordinating conjunctions
            "if",
            "when",
            "while",
            "although",
            "though",
            "because",
            "since",
            "as",
            "unless",
            "until",
            "wherever",
            "whereas",
            "whether",
            "before",
            "after",
            "once",
            "provided",
            "assuming",
            "given",
            "considering",
            "seeing",
            "granted",
            "supposing",
            // Correlative conjunctions
            "either",
            "neither",
            "both",
            "not",
            "only",
        ]
        .into_iter()
        .map(String::from)
        .collect();

        // Auxiliary verbs
        let auxiliaries: HashSet<String> = vec![
            "be", "am", "is", "are", "was", "were", "been", "being", "have", "has", "had",
            "having", "do", "does", "did", "done", "doing", "will", "would", "shall", "should",
            "can", "could", "may", "might", "must", "ought", "used", "dare", "need",
        ]
        .into_iter()
        .map(String::from)
        .collect();

        // Pronouns
        let pronouns: HashSet<String> = vec![
            // Personal pronouns
            "i",
            "me",
            "my",
            "mine",
            "myself",
            "you",
            "your",
            "yours",
            "yourself",
            "yourselves",
            "he",
            "him",
            "his",
            "himself",
            "she",
            "her",
            "hers",
            "herself",
            "it",
            "its",
            "itself",
            "we",
            "us",
            "our",
            "ours",
            "ourselves",
            "they",
            "them",
            "their",
            "theirs",
            "themselves",
            // Demonstrative pronouns
            "this",
            "that",
            "these",
            "those",
            // Relative pronouns
            "who",
            "whom",
            "whose",
            "which",
            "that",
            // Interrogative pronouns
            "what",
            "where",
            "when",
            "why",
            "how",
            // Indefinite pronouns
            "someone",
            "somebody",
            "something",
            "somewhere",
            "anyone",
            "anybody",
            "anything",
            "anywhere",
            "everyone",
            "everybody",
            "everything",
            "everywhere",
            "no one",
            "nobody",
            "nothing",
            "nowhere",
            "one",
            "ones",
            "another",
            "other",
            "others",
            "each",
            "either",
            "neither",
            "both",
            "all",
            "some",
            "any",
            "none",
            "most",
            "many",
            "few",
            "several",
            "such",
        ]
        .into_iter()
        .map(String::from)
        .collect();

        // Particles (often used with phrasal verbs)
        let particles: HashSet<String> = vec![
            "up", "down", "in", "out", "on", "off", "over", "under", "through", "across", "around",
            "about", "away", "back", "along", "apart", "aside", "forth", "forward", "ahead",
            "behind", "beyond", "below", "above", "within", "without",
        ]
        .into_iter()
        .map(String::from)
        .collect();

        // Quantifiers
        let quantifiers: HashSet<String> = vec![
            "all",
            "some",
            "any",
            "no",
            "none",
            "every",
            "each",
            "many",
            "much",
            "few",
            "little",
            "several",
            "most",
            "both",
            "either",
            "neither",
            "enough",
            "plenty",
            "lots",
            "loads",
            "tons",
            "heaps",
            "masses",
            "dozens",
            "hundreds",
            "thousands",
            "millions",
            "billions",
            "half",
            "quarter",
            "third",
            "double",
            "triple",
        ]
        .into_iter()
        .map(String::from)
        .collect();

        // Wh-words (interrogative and relative)
        let wh_words: HashSet<String> = vec![
            "what",
            "when",
            "where",
            "who",
            "whom",
            "whose",
            "which",
            "why",
            "how",
            "whatever",
            "whenever",
            "wherever",
            "whoever",
            "whomever",
            "whichever",
            "however",
        ]
        .into_iter()
        .map(String::from)
        .collect();

        Ok(Self {
            determiners,
            prepositions,
            conjunctions,
            auxiliaries,
            pronouns,
            particles,
            quantifiers,
            wh_words,
        })
    }

    /// Check if a word is a function word (any closed-class category)
    pub fn is_function_word(&self, word: &str) -> bool {
        let lowercase = word.to_lowercase();
        self.determiners.contains(&lowercase)
            || self.prepositions.contains(&lowercase)
            || self.conjunctions.contains(&lowercase)
            || self.auxiliaries.contains(&lowercase)
            || self.pronouns.contains(&lowercase)
            || self.particles.contains(&lowercase)
            || self.quantifiers.contains(&lowercase)
            || self.wh_words.contains(&lowercase)
    }

    /// Check if a word is a determiner
    pub fn is_determiner(&self, word: &str) -> bool {
        self.determiners.contains(&word.to_lowercase())
    }

    /// Check if a word is a preposition
    pub fn is_preposition(&self, word: &str) -> bool {
        self.prepositions.contains(&word.to_lowercase())
    }

    /// Check if a word is a conjunction
    pub fn is_conjunction(&self, word: &str) -> bool {
        self.conjunctions.contains(&word.to_lowercase())
    }

    /// Check if a word is an auxiliary verb
    pub fn is_auxiliary(&self, word: &str) -> bool {
        self.auxiliaries.contains(&word.to_lowercase())
    }

    /// Check if a word is a pronoun
    pub fn is_pronoun(&self, word: &str) -> bool {
        self.pronouns.contains(&word.to_lowercase())
    }

    /// Check if a word is a particle
    pub fn is_particle(&self, word: &str) -> bool {
        self.particles.contains(&word.to_lowercase())
    }

    /// Check if a word is a quantifier
    pub fn is_quantifier(&self, word: &str) -> bool {
        self.quantifiers.contains(&word.to_lowercase())
    }

    /// Check if a word is a wh-word
    pub fn is_wh_word(&self, word: &str) -> bool {
        self.wh_words.contains(&word.to_lowercase())
    }

    /// Get the functional category of a word
    pub fn get_category(&self, word: &str) -> Vec<String> {
        let mut categories = Vec::new();
        let lowercase = word.to_lowercase();

        if self.determiners.contains(&lowercase) {
            categories.push("determiner".to_string());
        }
        if self.prepositions.contains(&lowercase) {
            categories.push("preposition".to_string());
        }
        if self.conjunctions.contains(&lowercase) {
            categories.push("conjunction".to_string());
        }
        if self.auxiliaries.contains(&lowercase) {
            categories.push("auxiliary".to_string());
        }
        if self.pronouns.contains(&lowercase) {
            categories.push("pronoun".to_string());
        }
        if self.particles.contains(&lowercase) {
            categories.push("particle".to_string());
        }
        if self.quantifiers.contains(&lowercase) {
            categories.push("quantifier".to_string());
        }
        if self.wh_words.contains(&lowercase) {
            categories.push("wh_word".to_string());
        }

        categories
    }

    /// Get all words in a specific category
    pub fn get_words_in_category(&self, category: &str) -> Vec<String> {
        let set = match category {
            "determiner" => &self.determiners,
            "preposition" => &self.prepositions,
            "conjunction" => &self.conjunctions,
            "auxiliary" => &self.auxiliaries,
            "pronoun" => &self.pronouns,
            "particle" => &self.particles,
            "quantifier" => &self.quantifiers,
            "wh_word" => &self.wh_words,
            _ => return Vec::new(),
        };

        set.iter().cloned().collect()
    }

    /// Check if a word could be ambiguous between function word and content word
    pub fn is_potentially_ambiguous(&self, word: &str) -> bool {
        let categories = self.get_category(word);
        // Words that appear in multiple categories or are particles/prepositions
        // are often ambiguous (e.g., "up" can be particle, preposition, or adverb)
        categories.len() > 1
            || categories.contains(&"particle".to_string())
            || (categories.contains(&"preposition".to_string())
                && self.particles.contains(&word.to_lowercase()))
    }

    /// Get statistics about the lexicon
    pub fn get_stats(&self) -> ClosedClassStats {
        ClosedClassStats {
            determiners: self.determiners.len(),
            prepositions: self.prepositions.len(),
            conjunctions: self.conjunctions.len(),
            auxiliaries: self.auxiliaries.len(),
            pronouns: self.pronouns.len(),
            particles: self.particles.len(),
            quantifiers: self.quantifiers.len(),
            wh_words: self.wh_words.len(),
            total_words: self.determiners.len()
                + self.prepositions.len()
                + self.conjunctions.len()
                + self.auxiliaries.len()
                + self.pronouns.len()
                + self.particles.len()
                + self.quantifiers.len()
                + self.wh_words.len(),
        }
    }
}

/// Statistics about the closed-class lexicon
#[derive(Debug, Clone)]
pub struct ClosedClassStats {
    pub determiners: usize,
    pub prepositions: usize,
    pub conjunctions: usize,
    pub auxiliaries: usize,
    pub pronouns: usize,
    pub particles: usize,
    pub quantifiers: usize,
    pub wh_words: usize,
    pub total_words: usize,
}

impl Default for ClosedClassLexicon {
    fn default() -> Self {
        Self::new().expect("Failed to initialize closed-class lexicon")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexicon_creation() {
        let lexicon = ClosedClassLexicon::new().unwrap();
        let stats = lexicon.get_stats();
        assert!(stats.total_words > 100); // Should have plenty of function words
    }

    #[test]
    fn test_determiner_detection() {
        let lexicon = ClosedClassLexicon::new().unwrap();

        assert!(lexicon.is_determiner("the"));
        assert!(lexicon.is_determiner("a"));
        assert!(lexicon.is_determiner("this"));
        assert!(lexicon.is_determiner("my"));
        assert!(!lexicon.is_determiner("book"));
    }

    #[test]
    fn test_preposition_detection() {
        let lexicon = ClosedClassLexicon::new().unwrap();

        assert!(lexicon.is_preposition("in"));
        assert!(lexicon.is_preposition("on"));
        assert!(lexicon.is_preposition("with"));
        assert!(lexicon.is_preposition("through"));
        assert!(!lexicon.is_preposition("run"));
    }

    #[test]
    fn test_auxiliary_detection() {
        let lexicon = ClosedClassLexicon::new().unwrap();

        assert!(lexicon.is_auxiliary("is"));
        assert!(lexicon.is_auxiliary("have"));
        assert!(lexicon.is_auxiliary("will"));
        assert!(lexicon.is_auxiliary("can"));
        assert!(!lexicon.is_auxiliary("give"));
    }

    #[test]
    fn test_function_word_detection() {
        let lexicon = ClosedClassLexicon::new().unwrap();

        assert!(lexicon.is_function_word("the"));
        assert!(lexicon.is_function_word("in"));
        assert!(lexicon.is_function_word("and"));
        assert!(lexicon.is_function_word("he"));
        assert!(lexicon.is_function_word("will"));
        assert!(!lexicon.is_function_word("book"));
        assert!(!lexicon.is_function_word("give"));
    }

    #[test]
    fn test_category_identification() {
        let lexicon = ClosedClassLexicon::new().unwrap();

        let the_categories = lexicon.get_category("the");
        assert!(the_categories.contains(&"determiner".to_string()));

        let and_categories = lexicon.get_category("and");
        assert!(and_categories.contains(&"conjunction".to_string()));

        let he_categories = lexicon.get_category("he");
        assert!(he_categories.contains(&"pronoun".to_string()));
    }

    #[test]
    fn test_ambiguous_words() {
        let lexicon = ClosedClassLexicon::new().unwrap();

        // "up" can be particle or preposition
        assert!(lexicon.is_potentially_ambiguous("up"));

        // "that" can be determiner, pronoun, or conjunction
        assert!(lexicon.is_potentially_ambiguous("that"));

        // "book" should not be ambiguous in our function word lexicon
        assert!(!lexicon.is_potentially_ambiguous("book"));
    }

    #[test]
    fn test_wh_word_detection() {
        let lexicon = ClosedClassLexicon::new().unwrap();

        assert!(lexicon.is_wh_word("what"));
        assert!(lexicon.is_wh_word("who"));
        assert!(lexicon.is_wh_word("where"));
        assert!(lexicon.is_wh_word("when"));
        assert!(lexicon.is_wh_word("why"));
        assert!(lexicon.is_wh_word("how"));
        assert!(!lexicon.is_wh_word("book"));
    }

    #[test]
    fn test_case_insensitivity() {
        let lexicon = ClosedClassLexicon::new().unwrap();

        assert!(lexicon.is_determiner("THE"));
        assert!(lexicon.is_preposition("IN"));
        assert!(lexicon.is_auxiliary("IS"));
        assert!(lexicon.is_function_word("And"));
    }

    #[test]
    fn test_category_word_retrieval() {
        let lexicon = ClosedClassLexicon::new().unwrap();

        let determiners = lexicon.get_words_in_category("determiner");
        assert!(determiners.contains(&"the".to_string()));
        assert!(determiners.contains(&"a".to_string()));

        let prepositions = lexicon.get_words_in_category("preposition");
        assert!(prepositions.contains(&"in".to_string()));
        assert!(prepositions.contains(&"on".to_string()));

        let empty = lexicon.get_words_in_category("nonexistent");
        assert!(empty.is_empty());
    }
}
