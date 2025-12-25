//! Lemmatization for semantic analysis
//!
//! This module provides lemmatization capabilities to reduce words to their base forms
//! before semantic analysis. It supports multiple lemmatization strategies with a
//! trait-based architecture for extensibility.

use std::collections::HashMap;
use thiserror::Error;

/// Errors that can occur during lemmatization
#[derive(Error, Debug)]
pub enum LemmatizerError {
    #[error("Failed to initialize lemmatizer: {0}")]
    InitializationError(String),
    #[error("Lemmatization failed for word '{word}': {reason}")]
    LemmatizationError { word: String, reason: String },
    #[error("Feature not available: {0}")]
    FeatureNotAvailable(String),
}

impl From<LemmatizerError> for canopy_engine::EngineError {
    fn from(error: LemmatizerError) -> Self {
        match error {
            LemmatizerError::InitializationError(msg) => Self::ConfigError { message: msg },
            LemmatizerError::LemmatizationError { word, reason } => Self::AnalysisError {
                input: word,
                reason,
                source: None,
            },
            LemmatizerError::FeatureNotAvailable(feature) => Self::ResourceNotFound {
                resource_type: "feature".to_string(),
                identifier: feature,
            },
        }
    }
}

/// Result type for lemmatization operations
pub type LemmatizerResult<T> = Result<T, LemmatizerError>;

/// Trait for lemmatization to allow multiple implementations
pub trait Lemmatizer: Send + Sync {
    /// Basic lemmatization - returns base form
    fn lemmatize(&self, word: &str) -> String;

    /// Lemmatization with confidence score
    fn lemmatize_with_confidence(&self, word: &str) -> (String, f32) {
        (self.lemmatize(word), 1.0) // Default high confidence
    }

    /// Check if lemmatizer supports batch operations
    fn supports_batch(&self) -> bool {
        false
    }

    /// Batch lemmatization (optional optimization)
    fn lemmatize_batch(&self, words: &[String]) -> Vec<String> {
        words.iter().map(|word| self.lemmatize(word)).collect()
    }
}

/// Simple rule-based lemmatizer for basic cases
pub struct SimpleLemmatizer {
    /// Common irregular forms mapping
    irregulars: HashMap<String, String>,
}

impl SimpleLemmatizer {
    /// Create a new simple lemmatizer with common irregular forms
    pub fn new() -> LemmatizerResult<Self> {
        let mut irregulars = HashMap::new();

        // Common irregular verbs
        irregulars.insert("went".to_string(), "go".to_string());
        irregulars.insert("ran".to_string(), "run".to_string());
        irregulars.insert("was".to_string(), "be".to_string());
        irregulars.insert("were".to_string(), "be".to_string());
        irregulars.insert("had".to_string(), "have".to_string());
        irregulars.insert("did".to_string(), "do".to_string());
        irregulars.insert("said".to_string(), "say".to_string());
        irregulars.insert("gave".to_string(), "give".to_string());
        irregulars.insert("took".to_string(), "take".to_string());
        irregulars.insert("came".to_string(), "come".to_string());
        irregulars.insert("got".to_string(), "get".to_string());
        irregulars.insert("saw".to_string(), "see".to_string());
        irregulars.insert("knew".to_string(), "know".to_string());
        irregulars.insert("thought".to_string(), "think".to_string());
        irregulars.insert("found".to_string(), "find".to_string());
        irregulars.insert("told".to_string(), "tell".to_string());
        irregulars.insert("felt".to_string(), "feel".to_string());
        irregulars.insert("brought".to_string(), "bring".to_string());
        irregulars.insert("bought".to_string(), "buy".to_string());
        irregulars.insert("caught".to_string(), "catch".to_string());
        irregulars.insert("taught".to_string(), "teach".to_string());
        irregulars.insert("fought".to_string(), "fight".to_string());
        irregulars.insert("sought".to_string(), "seek".to_string());

        // Common irregular nouns
        irregulars.insert("children".to_string(), "child".to_string());
        irregulars.insert("feet".to_string(), "foot".to_string());
        irregulars.insert("teeth".to_string(), "tooth".to_string());
        irregulars.insert("geese".to_string(), "goose".to_string());
        irregulars.insert("mice".to_string(), "mouse".to_string());
        irregulars.insert("women".to_string(), "woman".to_string());
        irregulars.insert("men".to_string(), "man".to_string());
        irregulars.insert("people".to_string(), "person".to_string());

        Ok(Self { irregulars })
    }

    /// Apply simple rule-based lemmatization
    fn apply_rules(&self, word: &str) -> String {
        let lower = word.to_lowercase();

        // Check irregulars first
        if let Some(lemma) = self.irregulars.get(&lower) {
            return lemma.clone();
        }

        // Simple suffix rules
        if lower.ends_with("ing") && lower.len() > 5 {
            let stem = &lower[..lower.len() - 3];
            // Handle doubled consonants (running -> run)
            if stem.len() > 2 {
                let chars: Vec<char> = stem.chars().collect();
                if chars[chars.len() - 1] == chars[chars.len() - 2]
                    && chars[chars.len() - 1].is_alphabetic()
                    && !"aeiou".contains(chars[chars.len() - 1])
                {
                    return stem[..stem.len() - 1].to_string();
                }
            }
            return stem.to_string();
        }

        if lower.ends_with("ed") && lower.len() > 4 {
            let stem = &lower[..lower.len() - 2];
            return stem.to_string();
        }

        if lower.ends_with("s")
            && lower.len() > 3
            && !lower.ends_with("ss")
            && !lower.ends_with("us")
        {
            return lower[..lower.len() - 1].to_string();
        }

        if lower.ends_with("ly") && lower.len() > 4 {
            return lower[..lower.len() - 2].to_string();
        }

        // Return original if no rules apply
        lower
    }
}

impl Default for SimpleLemmatizer {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            irregulars: HashMap::new(),
        })
    }
}

impl Lemmatizer for SimpleLemmatizer {
    fn lemmatize(&self, word: &str) -> String {
        self.apply_rules(word)
    }

    fn lemmatize_with_confidence(&self, word: &str) -> (String, f32) {
        let lemma = self.apply_rules(word);
        let confidence = if self.irregulars.contains_key(&word.to_lowercase()) {
            0.95 // High confidence for known irregulars
        } else if lemma != word.to_lowercase() {
            0.80 // Medium confidence for rule-applied
        } else {
            0.60 // Lower confidence for unchanged
        };
        (lemma, confidence)
    }

    fn supports_batch(&self) -> bool {
        true
    }

    fn lemmatize_batch(&self, words: &[String]) -> Vec<String> {
        words.iter().map(|word| self.lemmatize(word)).collect()
    }
}

/// NLP Rule-based lemmatizer using nlprule crate
#[cfg(feature = "lemmatization")]
pub struct NLPRuleLemmatizer {
    rules: nlprule::Rules,
    tokenizer: nlprule::Tokenizer,
}

#[cfg(feature = "lemmatization")]
impl NLPRuleLemmatizer {
    /// Create a new NLPRule lemmatizer
    pub fn new() -> LemmatizerResult<Self> {
        // For now, create a simplified version that focuses on lemmatization
        // The actual NLP Rule integration can be improved later
        let tokenizer = nlprule::Tokenizer::new("en")
            .map_err(|e| LemmatizerError::InitializationError(e.to_string()))?;

        let rules = nlprule::Rules::new("en")
            .map_err(|e| LemmatizerError::InitializationError(e.to_string()))?;

        Ok(Self { rules, tokenizer })
    }

    /// Get English language lemmatizer
    pub fn english() -> LemmatizerResult<Self> {
        Self::new()
    }
}

#[cfg(feature = "lemmatization")]
impl Lemmatizer for NLPRuleLemmatizer {
    fn lemmatize(&self, word: &str) -> String {
        // Simplified implementation for now
        // TODO: Improve with proper NLP Rule integration
        let suggestions = self.rules.suggest(word, &self.tokenizer);

        if let Some(first_suggestion) = suggestions.first() {
            if let Some(replacement) = first_suggestion.replacements().first() {
                replacement.to_string()
            } else {
                word.to_lowercase()
            }
        } else {
            word.to_lowercase()
        }
    }

    fn lemmatize_with_confidence(&self, word: &str) -> (String, f32) {
        let lemma = self.lemmatize(word);
        let confidence = if lemma != word.to_lowercase() {
            0.90 // High confidence for NLP rule-based
        } else {
            0.70 // Medium confidence for unchanged
        };
        (lemma, confidence)
    }

    fn supports_batch(&self) -> bool {
        true
    }
}

/// Factory for creating lemmatizers
pub struct LemmatizerFactory;

impl LemmatizerFactory {
    /// Create the best available lemmatizer
    pub fn create_default() -> LemmatizerResult<Box<dyn Lemmatizer>> {
        #[cfg(feature = "lemmatization")]
        {
            match NLPRuleLemmatizer::new() {
                Ok(lemmatizer) => Ok(Box::new(lemmatizer)),
                Err(_) => {
                    // Fallback to simple lemmatizer
                    Ok(Box::new(SimpleLemmatizer::new()?))
                }
            }
        }

        #[cfg(not(feature = "lemmatization"))]
        {
            Ok(Box::new(SimpleLemmatizer::new()?))
        }
    }

    /// Create simple rule-based lemmatizer
    pub fn create_simple() -> LemmatizerResult<Box<dyn Lemmatizer>> {
        Ok(Box::new(SimpleLemmatizer::new()?))
    }

    /// Create NLP rule-based lemmatizer (requires lemmatization feature)
    #[cfg(feature = "lemmatization")]
    pub fn create_nlprule() -> LemmatizerResult<Box<dyn Lemmatizer>> {
        Ok(Box::new(NLPRuleLemmatizer::new()?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_simple_lemmatizer_basic() {
        let lemmatizer = SimpleLemmatizer::new().unwrap();

        // Test irregular verbs
        assert_eq!(lemmatizer.lemmatize("went"), "go");
        assert_eq!(lemmatizer.lemmatize("gave"), "give");
        assert_eq!(lemmatizer.lemmatize("ran"), "run");

        // Test regular verbs
        assert_eq!(lemmatizer.lemmatize("running"), "run");
        assert_eq!(lemmatizer.lemmatize("walked"), "walk");
        assert_eq!(lemmatizer.lemmatize("jumping"), "jump");

        // Test nouns
        assert_eq!(lemmatizer.lemmatize("books"), "book");
        assert_eq!(lemmatizer.lemmatize("children"), "child");
        assert_eq!(lemmatizer.lemmatize("cats"), "cat");

        // Test unchanged words
        assert_eq!(lemmatizer.lemmatize("book"), "book");
        assert_eq!(lemmatizer.lemmatize("run"), "run");
    }

    #[test]
    fn test_simple_lemmatizer_confidence() {
        let lemmatizer = SimpleLemmatizer::new().unwrap();

        // Irregular forms should have high confidence
        let (lemma, conf) = lemmatizer.lemmatize_with_confidence("gave");
        assert_eq!(lemma, "give");
        assert!(conf > 0.90);

        // Rule-applied should have medium confidence
        let (lemma, conf) = lemmatizer.lemmatize_with_confidence("running");
        assert_eq!(lemma, "run");
        assert!(conf > 0.75 && conf < 0.90);

        // Unchanged should have lower confidence
        let (lemma, conf) = lemmatizer.lemmatize_with_confidence("book");
        assert_eq!(lemma, "book");
        assert!(conf > 0.50 && conf < 0.75);
    }

    #[test]
    fn test_lemmatizer_factory() {
        let lemmatizer = LemmatizerFactory::create_default().unwrap();

        assert_eq!(lemmatizer.lemmatize("running"), "run");
        assert_eq!(lemmatizer.lemmatize("gave"), "give");
        assert_eq!(lemmatizer.lemmatize("books"), "book");
    }

    #[test]
    fn test_lemmatization_performance() {
        let lemmatizer = match SimpleLemmatizer::new() {
            Ok(l) => l,
            Err(e) => {
                if e.to_string().contains("not found") || e.to_string().contains("No such file") {
                    eprintln!("Skipping test: lemmatization data not available");
                    return;
                }
                panic!("Unexpected error: {}", e);
            }
        };
        let words = vec!["running", "jumped", "swimming", "wrote", "thinking"];

        let start = Instant::now();
        for word in &words {
            lemmatizer.lemmatize(word);
        }
        let duration = start.elapsed();

        // Should be reasonably fast - under 100μs per word in debug mode
        // (Release mode is much faster at ~5μs per word)
        let per_word = duration.as_micros() / words.len() as u128;
        assert!(
            per_word < 100,
            "Lemmatization too slow: {}μs per word",
            per_word
        );
    }

    #[test]
    fn test_batch_lemmatization() {
        let lemmatizer = SimpleLemmatizer::new().unwrap();
        let words = vec![
            "running".to_string(),
            "gave".to_string(),
            "books".to_string(),
        ];

        let results = lemmatizer.lemmatize_batch(&words);
        assert_eq!(results, vec!["run", "give", "book"]);
    }

    #[cfg(feature = "lemmatization")]
    #[test]
    fn test_nlprule_lemmatizer() {
        if let Ok(lemmatizer) = NLPRuleLemmatizer::new() {
            assert_eq!(lemmatizer.lemmatize("running"), "run");
            assert_eq!(lemmatizer.lemmatize("gave"), "give");

            let (lemma, confidence) = lemmatizer.lemmatize_with_confidence("jumping");
            assert_eq!(lemma, "jump");
            assert!(confidence > 0.5);
        }
    }

    #[test]
    fn test_unknown_words_graceful() {
        let lemmatizer = SimpleLemmatizer::new().unwrap();

        // Unknown words should return lowercase version
        assert_eq!(lemmatizer.lemmatize("flurble"), "flurble");
        assert_eq!(lemmatizer.lemmatize("XyZzY"), "xyzzy");
    }
}
