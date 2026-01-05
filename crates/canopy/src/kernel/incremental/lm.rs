//! Language model trait for surprisal computation.
//!
//! The language model provides probability estimates P(word|context) needed
//! for surprisal-based disambiguation and garden-path detection.

use super::Surprisal;

/// Language model trait for computing word/sense probabilities.
///
/// Implementations provide P(word|context) for surprisal computation
/// and P(sense|word, context) for sense disambiguation.
pub trait LanguageModel: Send + Sync {
    /// Compute P(token | prefix).
    ///
    /// Returns probability of the token given the preceding context.
    /// Used for surprisal computation: S(token) = -log₂ P(token|prefix)
    fn word_probability(&self, token: &str, prefix: &[&str]) -> f64;

    /// Compute surprisal for a token given prefix.
    ///
    /// Default implementation uses `word_probability`.
    fn word_surprisal(&self, token: &str, prefix: &[&str]) -> Surprisal {
        let p = self.word_probability(token, prefix);
        if p <= 0.0 {
            // Very low probability (unseen word) - cap at ~20 bits
            Surprisal::from_bits(20.0)
        } else {
            Surprisal::from_probability(p)
        }
    }

    /// Compute P(sense | word, context).
    ///
    /// Returns probability of a particular word sense given the word
    /// and its surrounding context. Used for sense disambiguation.
    fn sense_probability(&self, sense_id: &str, word: &str, context: &[&str]) -> f64;

    /// Compute joint probability P(reading | sentence).
    ///
    /// Returns the overall probability of a complete reading,
    /// which is the product of individual word probabilities
    /// (or sum of surprisals in log space).
    fn reading_probability(&self, words: &[&str]) -> f64 {
        if words.is_empty() {
            return 1.0;
        }

        // P(w1, w2, ..., wn) = P(w1) * P(w2|w1) * P(w3|w1,w2) * ...
        let mut log_prob = 0.0;
        for i in 0..words.len() {
            let prefix = &words[..i];
            let p = self.word_probability(words[i], prefix);
            if p <= 0.0 {
                return 0.0; // Zero probability
            }
            log_prob += p.ln();
        }
        log_prob.exp()
    }

    /// Compute total surprisal for a sentence.
    ///
    /// Sum of surprisals: S(sentence) = Σ S(wi|w1..wi-1)
    fn sentence_surprisal(&self, words: &[&str]) -> Surprisal {
        let mut total = Surprisal::ZERO;
        for i in 0..words.len() {
            let prefix = &words[..i];
            total += self.word_surprisal(words[i], prefix);
        }
        total
    }
}

/// Uniform language model - assigns equal probability to all words.
///
/// Useful as a baseline or when no better model is available.
/// All words have P = `1/vocabulary_size`.
#[derive(Debug, Clone)]
pub struct UniformLanguageModel {
    /// Size of the vocabulary (determines probability).
    pub vocabulary_size: usize,
    /// Number of word senses (for sense probability).
    pub sense_count: usize,
}

impl Default for UniformLanguageModel {
    fn default() -> Self {
        Self {
            vocabulary_size: 50_000, // Typical English vocabulary
            sense_count: 5,          // Average senses per word
        }
    }
}

impl LanguageModel for UniformLanguageModel {
    #[allow(clippy::cast_precision_loss)]
    fn word_probability(&self, _token: &str, _prefix: &[&str]) -> f64 {
        1.0 / self.vocabulary_size as f64
    }

    #[allow(clippy::cast_precision_loss)]
    fn sense_probability(&self, _sense_id: &str, _word: &str, _context: &[&str]) -> f64 {
        1.0 / self.sense_count as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uniform_word_probability() {
        let lm = UniformLanguageModel {
            vocabulary_size: 1000,
            sense_count: 5,
        };

        let p = lm.word_probability("hello", &[]);
        assert!((p - 0.001).abs() < f64::EPSILON);
    }

    #[test]
    fn test_uniform_surprisal() {
        let lm = UniformLanguageModel {
            vocabulary_size: 1024, // 2^10
            sense_count: 5,
        };

        let s = lm.word_surprisal("word", &[]);
        // 1/1024 = 2^-10, so surprisal = 10 bits
        assert!((s.bits() - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_sentence_surprisal() {
        let lm = UniformLanguageModel {
            vocabulary_size: 1024,
            sense_count: 5,
        };

        let words = ["the", "cat", "sat"];
        let s = lm.sentence_surprisal(&words);
        // 3 words * 10 bits each = 30 bits
        assert!((s.bits() - 30.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_reading_probability() {
        let lm = UniformLanguageModel {
            vocabulary_size: 100,
            sense_count: 5,
        };

        let words = ["a", "b"];
        let p = lm.reading_probability(&words);
        // P = (1/100) * (1/100) = 1/10000
        assert!((p - 0.0001).abs() < 1e-10);
    }
}
