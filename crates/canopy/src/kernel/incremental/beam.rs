//! Beam search for tracking multiple readings during incremental processing.
//!
//! Maintains a beam of partial readings, pruning low-probability candidates
//! to keep computation tractable while preserving ambiguity.

use std::collections::HashMap;

use super::Surprisal;

/// Identifier for a choice point (where ambiguity occurs).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChoiceId(pub u32);

impl ChoiceId {
    /// Create a new choice ID.
    #[must_use]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }
}

/// A partial reading being tracked during incremental processing.
///
/// Represents a set of choices made so far, with associated probability.
#[derive(Debug, Clone)]
pub struct BeamEntry {
    /// Choices made at each choice point (`choice_id` -> alternative index).
    pub choices: HashMap<ChoiceId, usize>,

    /// Log probability of this reading (sum of log probs).
    pub log_probability: f64,

    /// Total surprisal accumulated.
    pub total_surprisal: Surprisal,

    /// Number of words processed.
    pub words_processed: usize,
}

impl BeamEntry {
    /// Create a new empty beam entry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            choices: HashMap::new(),
            log_probability: 0.0,
            total_surprisal: Surprisal::ZERO,
            words_processed: 0,
        }
    }

    /// Get probability from log probability.
    #[must_use]
    pub fn probability(&self) -> f64 {
        self.log_probability.exp()
    }

    /// Create a copy with an additional choice made.
    #[must_use]
    pub fn with_choice(&self, choice_id: ChoiceId, alternative: usize) -> Self {
        let mut new = self.clone();
        new.choices.insert(choice_id, alternative);
        new
    }

    /// Update with new word probability.
    pub fn update_probability(&mut self, word_log_prob: f64, word_surprisal: Surprisal) {
        self.log_probability += word_log_prob;
        self.total_surprisal += word_surprisal;
        self.words_processed += 1;
    }
}

impl Default for BeamEntry {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for beam search.
#[derive(Debug, Clone)]
pub struct BeamSearchConfig {
    /// Maximum number of readings to track.
    pub beam_width: usize,

    /// Minimum probability (relative to best) to keep a reading.
    /// E.g., 0.001 means prune readings with P < 0.001 * `P_best`.
    pub pruning_threshold: f64,

    /// Maximum log probability difference from best before pruning.
    /// Derived from `pruning_threshold` for efficiency.
    pub max_log_prob_diff: f64,
}

impl Default for BeamSearchConfig {
    fn default() -> Self {
        let pruning_threshold = 0.001;
        Self {
            beam_width: 100,
            pruning_threshold,
            max_log_prob_diff: pruning_threshold.ln().abs(),
        }
    }
}

impl BeamSearchConfig {
    /// Create config with custom beam width.
    #[must_use]
    pub fn with_beam_width(mut self, width: usize) -> Self {
        self.beam_width = width;
        self
    }

    /// Create config with custom pruning threshold.
    #[must_use]
    pub fn with_pruning_threshold(mut self, threshold: f64) -> Self {
        self.pruning_threshold = threshold;
        self.max_log_prob_diff = threshold.ln().abs();
        self
    }
}

/// Beam search state for incremental processing.
#[derive(Debug, Clone)]
pub struct BeamSearch {
    /// Configuration.
    config: BeamSearchConfig,

    /// Current beam of readings.
    beam: Vec<BeamEntry>,

    /// Next choice ID to assign.
    next_choice_id: u32,
}

impl BeamSearch {
    /// Create a new beam search with default configuration.
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(BeamSearchConfig::default())
    }

    /// Create a new beam search with custom configuration.
    #[must_use]
    pub fn with_config(config: BeamSearchConfig) -> Self {
        Self {
            config,
            beam: vec![BeamEntry::new()],
            next_choice_id: 0,
        }
    }

    /// Get the current beam of readings.
    #[must_use]
    pub fn beam(&self) -> &[BeamEntry] {
        &self.beam
    }

    /// Get the number of readings currently tracked.
    #[must_use]
    pub fn reading_count(&self) -> usize {
        self.beam.len()
    }

    /// Check if there are multiple readings (ambiguity).
    #[must_use]
    pub fn is_ambiguous(&self) -> bool {
        self.beam.len() > 1
    }

    /// Get the best (highest probability) reading.
    ///
    /// # Panics
    /// Panics if `log_probability` contains NaN values.
    #[must_use]
    pub fn best_reading(&self) -> Option<&BeamEntry> {
        self.beam
            .iter()
            .max_by(|a, b| a.log_probability.partial_cmp(&b.log_probability).unwrap())
    }

    /// Allocate a new choice ID.
    pub fn new_choice_id(&mut self) -> ChoiceId {
        let id = ChoiceId::new(self.next_choice_id);
        self.next_choice_id += 1;
        id
    }

    /// Add a choice point with multiple alternatives.
    ///
    /// Expands the beam by creating entries for each combination
    /// of existing entries × new alternatives.
    pub fn add_choice(&mut self, choice_id: ChoiceId, alternative_probs: &[f64]) {
        if alternative_probs.is_empty() {
            return;
        }

        if alternative_probs.len() == 1 {
            // No ambiguity - just update all entries
            let log_prob = alternative_probs[0].ln();
            for entry in &mut self.beam {
                entry.choices.insert(choice_id, 0);
                entry.log_probability += log_prob;
            }
            return;
        }

        // Expand beam with all alternatives
        let mut new_beam = Vec::with_capacity(self.beam.len() * alternative_probs.len());

        for entry in &self.beam {
            for (alt_idx, &prob) in alternative_probs.iter().enumerate() {
                if prob <= 0.0 {
                    continue;
                }
                let mut new_entry = entry.with_choice(choice_id, alt_idx);
                new_entry.log_probability += prob.ln();
                new_beam.push(new_entry);
            }
        }

        self.beam = new_beam;
        self.prune();
    }

    /// Update all readings with a new word observation.
    pub fn observe_word(&mut self, word_log_prob: f64, word_surprisal: Surprisal) {
        for entry in &mut self.beam {
            entry.update_probability(word_log_prob, word_surprisal);
        }
    }

    /// Prune low-probability readings.
    ///
    /// # Panics
    /// Panics if `log_probability` contains NaN values.
    pub fn prune(&mut self) {
        if self.beam.is_empty() {
            return;
        }

        // Find best log probability
        let best_log_prob = self
            .beam
            .iter()
            .map(|e| e.log_probability)
            .fold(f64::NEG_INFINITY, f64::max);

        // Remove entries too far below best
        let threshold = best_log_prob - self.config.max_log_prob_diff;
        self.beam.retain(|e| e.log_probability >= threshold);

        // Trim to beam width if still too large
        if self.beam.len() > self.config.beam_width {
            // Sort by probability (descending)
            self.beam
                .sort_by(|a, b| b.log_probability.partial_cmp(&a.log_probability).unwrap());
            self.beam.truncate(self.config.beam_width);
        }
    }

    /// Normalize probabilities to sum to 1.
    pub fn normalize(&mut self) {
        if self.beam.is_empty() {
            return;
        }

        // Use log-sum-exp for numerical stability
        let max_log_prob = self
            .beam
            .iter()
            .map(|e| e.log_probability)
            .fold(f64::NEG_INFINITY, f64::max);

        let log_sum: f64 = self
            .beam
            .iter()
            .map(|e| (e.log_probability - max_log_prob).exp())
            .sum::<f64>()
            .ln()
            + max_log_prob;

        for entry in &mut self.beam {
            entry.log_probability -= log_sum;
        }
    }

    /// Compute entropy of current probability distribution (in bits).
    #[must_use]
    pub fn entropy(&self) -> f64 {
        if self.beam.is_empty() {
            return 0.0;
        }

        // Normalize probabilities first
        let max_log_prob = self
            .beam
            .iter()
            .map(|e| e.log_probability)
            .fold(f64::NEG_INFINITY, f64::max);

        let sum: f64 = self
            .beam
            .iter()
            .map(|e| (e.log_probability - max_log_prob).exp())
            .sum();

        // H = -Σ p log₂ p
        let mut entropy = 0.0;
        for entry in &self.beam {
            let p = (entry.log_probability - max_log_prob).exp() / sum;
            if p > 0.0 {
                entropy -= p * p.log2();
            }
        }

        entropy
    }

    /// Compute entropy reduction from adding new information.
    #[must_use]
    pub fn entropy_reduction(&self, previous_entropy: f64) -> f64 {
        previous_entropy - self.entropy()
    }
}

impl Default for BeamSearch {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_beam_entry_creation() {
        let entry = BeamEntry::new();
        assert!(entry.choices.is_empty());
        assert!((entry.log_probability - 0.0).abs() < f64::EPSILON);
        assert_eq!(entry.words_processed, 0);
    }

    #[test]
    fn test_beam_entry_with_choice() {
        let entry = BeamEntry::new();
        let choice_id = ChoiceId::new(0);
        let new_entry = entry.with_choice(choice_id, 2);

        assert_eq!(new_entry.choices.get(&choice_id), Some(&2));
        assert!(entry.choices.is_empty()); // Original unchanged
    }

    #[test]
    fn test_beam_search_initial_state() {
        let beam = BeamSearch::new();
        assert_eq!(beam.reading_count(), 1);
        assert!(!beam.is_ambiguous());
    }

    #[test]
    fn test_beam_search_add_choice() {
        let mut beam = BeamSearch::new();
        let choice_id = beam.new_choice_id();

        // Add 3 alternatives with equal probability
        beam.add_choice(choice_id, &[0.33, 0.33, 0.34]);

        assert_eq!(beam.reading_count(), 3);
        assert!(beam.is_ambiguous());
    }

    #[test]
    fn test_beam_search_pruning() {
        let config = BeamSearchConfig::default()
            .with_beam_width(2)
            .with_pruning_threshold(0.1);
        let mut beam = BeamSearch::with_config(config);
        let choice_id = beam.new_choice_id();

        // Add 5 alternatives - should prune to beam width
        beam.add_choice(choice_id, &[0.4, 0.3, 0.15, 0.1, 0.05]);

        assert!(beam.reading_count() <= 2);
    }

    #[test]
    fn test_beam_search_entropy() {
        let mut beam = BeamSearch::new();
        let choice_id = beam.new_choice_id();

        // Two equally likely alternatives -> 1 bit of entropy
        beam.add_choice(choice_id, &[0.5, 0.5]);
        beam.normalize();

        let entropy = beam.entropy();
        assert!((entropy - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_beam_search_single_alternative() {
        let mut beam = BeamSearch::new();
        let choice_id = beam.new_choice_id();

        // Single alternative - no expansion
        beam.add_choice(choice_id, &[1.0]);

        assert_eq!(beam.reading_count(), 1);
        assert!(!beam.is_ambiguous());
    }
}
