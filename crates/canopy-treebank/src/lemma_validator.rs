//! Lemma validation against Universal Dependencies gold standard
//!
//! This module compares lemmatization results from external lemmatizers (like SimpleLemmatizer)
//! against the gold-standard lemmas in UD treebanks to validate accuracy and learn new patterns.

use crate::parser::{ParsedSentence, ParsedToken};
use crate::TreebankResult;
use canopy_tokenizer::lemmatizer::Lemmatizer;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

/// Results of lemma validation against gold standard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LemmaValidationResult {
    /// Total words validated
    pub total_words: usize,
    /// Number of exact matches
    pub exact_matches: usize,
    /// Number of mismatches
    pub mismatches: usize,
    /// Accuracy percentage (0.0-1.0)
    pub accuracy: f32,
    /// Learned irregular mappings from mismatches
    pub learned_irregulars: HashMap<String, String>,
    /// Confidence scores per word form
    pub confidence_scores: HashMap<String, f32>,
}

impl Default for LemmaValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

impl LemmaValidationResult {
    /// Create a new empty validation result
    pub fn new() -> Self {
        Self {
            total_words: 0,
            exact_matches: 0,
            mismatches: 0,
            accuracy: 0.0,
            learned_irregulars: HashMap::new(),
            confidence_scores: HashMap::new(),
        }
    }

    /// Update accuracy calculation
    pub fn calculate_accuracy(&mut self) {
        if self.total_words > 0 {
            self.accuracy = self.exact_matches as f32 / self.total_words as f32;
        }
    }

    /// Add a validation result for a single word
    pub fn add_validation(&mut self, word: &str, predicted: &str, gold: &str) {
        self.total_words += 1;

        if predicted == gold {
            self.exact_matches += 1;
            // High confidence for exact matches
            self.confidence_scores.insert(word.to_string(), 0.95);
        } else {
            self.mismatches += 1;
            // Learn the gold mapping for future use
            self.learned_irregulars
                .insert(word.to_string(), gold.to_string());
            // Lower confidence for mismatches
            self.confidence_scores.insert(word.to_string(), 0.3);

            debug!(
                "Lemma mismatch: '{}' -> predicted: '{}', gold: '{}'",
                word, predicted, gold
            );
        }

        self.calculate_accuracy();
    }
}

/// Validates lemmatization against UD gold standard
pub struct LemmaValidator {
    /// Enable verbose logging
    verbose: bool,
}

impl LemmaValidator {
    /// Create a new lemma validator
    pub fn new(verbose: bool) -> Self {
        Self { verbose }
    }

    /// Validate a lemmatizer against parsed sentences with gold lemmas
    pub fn validate_lemmatizer(
        &self,
        lemmatizer: &dyn Lemmatizer,
        sentences: &[ParsedSentence],
    ) -> TreebankResult<LemmaValidationResult> {
        let mut result = LemmaValidationResult::new();

        info!(
            "Starting lemma validation against {} sentences",
            sentences.len()
        );

        for sentence in sentences {
            for token in &sentence.tokens {
                // Skip punctuation and function words for validation
                if self.should_skip_token(token) {
                    continue;
                }

                let word = &token.form;
                let gold_lemma = &token.lemma;
                let predicted_lemma = lemmatizer.lemmatize(word);

                result.add_validation(word, &predicted_lemma, gold_lemma);

                if self.verbose && predicted_lemma != *gold_lemma {
                    debug!(
                        "Validation: '{}' -> predicted: '{}', gold: '{}', POS: {}",
                        word, predicted_lemma, gold_lemma, token.upos
                    );
                }
            }
        }

        info!(
            "Lemma validation completed: {}/{} accuracy ({:.2}%), {} learned irregulars",
            result.exact_matches,
            result.total_words,
            result.accuracy * 100.0,
            result.learned_irregulars.len()
        );

        if self.verbose {
            self.log_common_mismatches(&result);
        }

        Ok(result)
    }

    /// Validate against a single sentence
    pub fn validate_sentence(
        &self,
        lemmatizer: &dyn Lemmatizer,
        sentence: &ParsedSentence,
    ) -> TreebankResult<LemmaValidationResult> {
        self.validate_lemmatizer(lemmatizer, std::slice::from_ref(sentence))
    }

    /// Check if token should be skipped during validation
    fn should_skip_token(&self, token: &ParsedToken) -> bool {
        // Skip punctuation
        if token.upos == "PUNCT" {
            return true;
        }

        // Skip function words that are typically unchanged
        if matches!(
            token.upos.as_str(),
            "DET" | "ADP" | "CONJ" | "CCONJ" | "SCONJ" | "PART"
        ) {
            return true;
        }

        // Skip single character words
        if token.form.len() <= 1 {
            return true;
        }

        // Skip if form equals lemma (no change needed)
        if token.form == token.lemma {
            return true;
        }

        false
    }

    /// Log the most common types of mismatches for analysis
    fn log_common_mismatches(&self, result: &LemmaValidationResult) {
        let mut mismatch_patterns = HashMap::new();

        for (word, gold_lemma) in &result.learned_irregulars {
            let pattern = self.categorize_mismatch(word, gold_lemma);
            *mismatch_patterns.entry(pattern).or_insert(0) += 1;
        }

        info!("Common mismatch patterns:");
        let mut patterns: Vec<_> = mismatch_patterns.into_iter().collect();
        patterns.sort_by_key(|(_, count)| -*count);

        for (pattern, count) in patterns.into_iter().take(10) {
            info!("  {}: {} occurrences", pattern, count);
        }
    }

    /// Categorize the type of mismatch for analysis
    fn categorize_mismatch(&self, word: &str, gold_lemma: &str) -> String {
        if word.ends_with("ing") && !gold_lemma.ends_with("ing") {
            "present_participle".to_string()
        } else if word.ends_with("ed") && !gold_lemma.ends_with("ed") {
            "past_tense".to_string()
        } else if word.ends_with("s") && word.len() > gold_lemma.len() {
            "plural_or_3rd_person".to_string()
        } else if word.ends_with("er") && !gold_lemma.ends_with("er") {
            "comparative".to_string()
        } else if word.ends_with("est") && !gold_lemma.ends_with("est") {
            "superlative".to_string()
        } else if word.len() > gold_lemma.len() {
            "suffix_removal".to_string()
        } else if word.len() < gold_lemma.len() {
            "irregular_expansion".to_string()
        } else {
            "stem_change".to_string()
        }
    }

    /// Export learned irregular mappings in a format suitable for SimpleLemmatizer
    pub fn export_learned_irregulars(
        &self,
        result: &LemmaValidationResult,
    ) -> HashMap<String, String> {
        let mut filtered_irregulars = HashMap::new();

        for (word, lemma) in &result.learned_irregulars {
            // Only export if we have some confidence and it's not a trivial mapping
            if word != lemma && word.len() > 2 && lemma.len() > 1 {
                filtered_irregulars.insert(word.clone(), lemma.clone());
            }
        }

        info!(
            "Exported {} learned irregular mappings",
            filtered_irregulars.len()
        );
        filtered_irregulars
    }
}

impl Default for LemmaValidator {
    fn default() -> Self {
        Self::new(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{DependencyFeatures, DependencyRelation};
    use canopy_tokenizer::lemmatizer::SimpleLemmatizer;
    use std::collections::HashMap;

    fn create_test_token(form: &str, lemma: &str, upos: &str) -> ParsedToken {
        ParsedToken {
            id: 1,
            form: form.to_string(),
            lemma: lemma.to_string(),
            upos: upos.to_string(),
            xpos: None,
            features: HashMap::new(),
            head: 0,
            deprel: DependencyRelation::Other("root".to_string()),
            dependency_features: DependencyFeatures::default(),
            deps: vec![],
        }
    }

    fn create_test_sentence(tokens: Vec<ParsedToken>) -> ParsedSentence {
        ParsedSentence {
            sent_id: "test-001".to_string(),
            text: "Test sentence".to_string(),
            tokens,
            root_verb: None,
        }
    }

    #[test]
    fn test_lemma_validator_creation() {
        let validator = LemmaValidator::new(true);
        assert!(validator.verbose);

        let validator = LemmaValidator::default();
        assert!(!validator.verbose);
    }

    #[test]
    fn test_validation_result() {
        let mut result = LemmaValidationResult::new();
        assert_eq!(result.accuracy, 0.0);
        assert_eq!(result.total_words, 0);

        // Add exact match
        result.add_validation("run", "run", "run");
        assert_eq!(result.exact_matches, 1);
        assert_eq!(result.accuracy, 1.0);

        // Add mismatch
        result.add_validation("running", "ran", "run");
        assert_eq!(result.mismatches, 1);
        assert_eq!(result.accuracy, 0.5);
        assert!(result.learned_irregulars.contains_key("running"));
    }

    #[test]
    fn test_should_skip_token() {
        let validator = LemmaValidator::default();

        // Should skip punctuation
        let punct = create_test_token(".", ".", "PUNCT");
        assert!(validator.should_skip_token(&punct));

        // Should skip determiners
        let det = create_test_token("the", "the", "DET");
        assert!(validator.should_skip_token(&det));

        // Should not skip verbs
        let verb = create_test_token("running", "run", "VERB");
        assert!(!validator.should_skip_token(&verb));

        // Should skip if form equals lemma
        let unchanged = create_test_token("run", "run", "VERB");
        assert!(validator.should_skip_token(&unchanged));
    }

    #[test]
    fn test_validate_sentence() {
        let validator = LemmaValidator::new(false);
        let lemmatizer = SimpleLemmatizer::new().unwrap();

        let tokens = vec![
            create_test_token("running", "run", "VERB"),
            create_test_token("quickly", "quickly", "ADV"),
            create_test_token(".", ".", "PUNCT"), // Should be skipped
        ];

        let sentence = create_test_sentence(tokens);
        let result = validator.validate_sentence(&lemmatizer, &sentence).unwrap();

        // Only "running" should be validated (not punct or unchanged words)
        assert!(result.total_words >= 1);
        assert!(result.accuracy >= 0.0 && result.accuracy <= 1.0);
    }

    #[test]
    fn test_categorize_mismatch() {
        let validator = LemmaValidator::default();

        assert_eq!(
            validator.categorize_mismatch("running", "run"),
            "present_participle"
        );
        assert_eq!(
            validator.categorize_mismatch("walked", "walk"),
            "past_tense"
        );
        assert_eq!(
            validator.categorize_mismatch("cats", "cat"),
            "plural_or_3rd_person"
        );
        assert_eq!(
            validator.categorize_mismatch("bigger", "big"),
            "comparative"
        );
        assert_eq!(
            validator.categorize_mismatch("biggest", "big"),
            "superlative"
        );
    }

    #[test]
    fn test_export_learned_irregulars() {
        let validator = LemmaValidator::default();
        let mut result = LemmaValidationResult::new();

        // Add some learned mappings
        result
            .learned_irregulars
            .insert("went".to_string(), "go".to_string());
        result
            .learned_irregulars
            .insert("a".to_string(), "a".to_string()); // Too short
        result
            .learned_irregulars
            .insert("same".to_string(), "same".to_string()); // No change

        let exported = validator.export_learned_irregulars(&result);

        assert!(exported.contains_key("went"));
        assert!(!exported.contains_key("a"));
        assert!(!exported.contains_key("same"));
    }
}
