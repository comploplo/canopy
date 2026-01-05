//! Incremental processing state and processor.
//!
//! Tracks the state of incremental left-to-right processing,
//! including the beam of active readings and surprisal trace.

use crate::runtime::TokenId;

use super::beam::{BeamEntry, BeamSearch, BeamSearchConfig, ChoiceId};
use super::lm::SurprisalModel;
use super::surprisal::{GardenPathDetector, GardenPathEvent, Surprisal};

/// State of incremental processing after each word.
#[derive(Debug, Clone)]
pub struct IncrementalState {
    /// Tokens processed so far.
    pub prefix: Vec<TokenId>,

    /// Forms of tokens processed (for LM queries).
    pub forms: Vec<String>,

    /// Beam of active readings with probabilities.
    pub beam: BeamSearch,

    /// Surprisal at each word position.
    pub surprisal_trace: Vec<Surprisal>,

    /// Garden-path events detected.
    pub garden_paths: Vec<GardenPathEvent>,
}

impl IncrementalState {
    /// Create a new incremental state.
    #[must_use]
    pub fn new() -> Self {
        Self {
            prefix: Vec::new(),
            forms: Vec::new(),
            beam: BeamSearch::new(),
            surprisal_trace: Vec::new(),
            garden_paths: Vec::new(),
        }
    }

    /// Create with custom beam configuration.
    #[must_use]
    pub fn with_beam_config(config: BeamSearchConfig) -> Self {
        Self {
            prefix: Vec::new(),
            forms: Vec::new(),
            beam: BeamSearch::with_config(config),
            surprisal_trace: Vec::new(),
            garden_paths: Vec::new(),
        }
    }

    /// Get the number of words processed.
    #[must_use]
    pub fn word_count(&self) -> usize {
        self.prefix.len()
    }

    /// Get total surprisal so far.
    #[must_use]
    pub fn total_surprisal(&self) -> Surprisal {
        self.surprisal_trace.iter().copied().sum()
    }

    /// Get current entropy of reading distribution.
    #[must_use]
    pub fn entropy(&self) -> f64 {
        self.beam.entropy()
    }

    /// Check if processing is ambiguous.
    #[must_use]
    pub fn is_ambiguous(&self) -> bool {
        self.beam.is_ambiguous()
    }

    /// Get the number of active readings.
    #[must_use]
    pub fn reading_count(&self) -> usize {
        self.beam.reading_count()
    }

    /// Get the best reading so far.
    #[must_use]
    pub fn best_reading(&self) -> Option<&BeamEntry> {
        self.beam.best_reading()
    }

    /// Check if a garden-path was detected.
    #[must_use]
    pub fn has_garden_path(&self) -> bool {
        !self.garden_paths.is_empty()
    }
}

impl Default for IncrementalState {
    fn default() -> Self {
        Self::new()
    }
}

/// A partial reading prefix (before sentence is complete).
///
/// Tracks choices made so far and probability.
pub type ReadingPrefix = BeamEntry;

/// Configuration for the incremental processor.
#[derive(Debug, Clone)]
pub struct IncrementalProcessorConfig {
    /// Beam search configuration.
    pub beam_config: BeamSearchConfig,

    /// Garden-path detector configuration.
    pub garden_path_threshold: f64,

    /// Minimum prefix length before garden-path detection.
    pub garden_path_min_prefix: usize,
}

impl Default for IncrementalProcessorConfig {
    fn default() -> Self {
        Self {
            beam_config: BeamSearchConfig::default(),
            garden_path_threshold: 10.0,
            garden_path_min_prefix: 3,
        }
    }
}

/// Incremental processor for left-to-right semantic analysis.
///
/// Processes tokens one at a time, maintaining a beam of readings
/// and tracking surprisal at each position.
#[derive(Debug)]
pub struct IncrementalProcessor {
    /// Configuration.
    config: IncrementalProcessorConfig,

    /// Garden-path detector.
    garden_path_detector: GardenPathDetector,
}

impl IncrementalProcessor {
    /// Create a new processor with default configuration.
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(IncrementalProcessorConfig::default())
    }

    /// Create a processor with custom configuration.
    #[must_use]
    pub fn with_config(config: IncrementalProcessorConfig) -> Self {
        let garden_path_detector = GardenPathDetector {
            threshold: config.garden_path_threshold,
            min_prefix_length: config.garden_path_min_prefix,
        };

        Self {
            config,
            garden_path_detector,
        }
    }

    /// Create a new incremental state for processing.
    #[must_use]
    pub fn new_state(&self) -> IncrementalState {
        IncrementalState::with_beam_config(self.config.beam_config.clone())
    }

    /// Process the next word, updating state and computing surprisal.
    ///
    /// Returns the surprisal at this word position.
    pub fn process_word<LM: SurprisalModel>(
        &self,
        state: &mut IncrementalState,
        token_id: TokenId,
        form: &str,
        lm: &LM,
    ) -> Surprisal {
        // Compute surprisal for this word
        let prefix_forms: Vec<&str> = state.forms.iter().map(String::as_str).collect();
        let surprisal = lm.word_surprisal(form, &prefix_forms);

        // Update state
        state.prefix.push(token_id);
        state.forms.push(form.to_string());
        state.surprisal_trace.push(surprisal);

        // Update beam with word observation
        let log_prob = if surprisal.bits() >= 20.0 {
            -20.0 * std::f64::consts::LN_2 // Cap at very low probability
        } else {
            -surprisal.bits() * std::f64::consts::LN_2
        };
        state.beam.observe_word(log_prob, surprisal);

        // Check for garden-path
        if let Some(event) = self.garden_path_detector.detect(&state.surprisal_trace) {
            // Only add if it's a new event (at current position)
            if event.word_index == state.prefix.len() - 1 {
                state.garden_paths.push(event);
            }
        }

        surprisal
    }

    /// Add a sense choice point to the beam.
    ///
    /// Call this when the current word has multiple possible senses.
    /// `sense_probs` contains the probability of each sense.
    pub fn add_sense_choice(&self, state: &mut IncrementalState, sense_probs: &[f64]) -> ChoiceId {
        let choice_id = state.beam.new_choice_id();
        state.beam.add_choice(choice_id, sense_probs);
        choice_id
    }

    /// Get entropy reduction from the previous word.
    ///
    /// Positive value means uncertainty was reduced.
    #[must_use]
    pub fn entropy_reduction(&self, prev_entropy: f64, state: &IncrementalState) -> f64 {
        prev_entropy - state.entropy()
    }

    /// Check if the current position is a garden-path.
    #[must_use]
    pub fn is_garden_path(&self, state: &IncrementalState) -> bool {
        if let Some(last_event) = state.garden_paths.last() {
            last_event.word_index == state.prefix.len() - 1
        } else {
            false
        }
    }

    /// Get the garden-path detector for custom analysis.
    #[must_use]
    pub const fn garden_path_detector(&self) -> &GardenPathDetector {
        &self.garden_path_detector
    }
}

impl Default for IncrementalProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::incremental::lm::UniformSurprisalModel;

    #[test]
    fn test_incremental_state_creation() {
        let state = IncrementalState::new();
        assert_eq!(state.word_count(), 0);
        assert!(!state.is_ambiguous());
        assert!(!state.has_garden_path());
    }

    #[test]
    fn test_process_word() {
        let processor = IncrementalProcessor::new();
        let mut state = processor.new_state();
        let lm = UniformSurprisalModel::default();

        let surprisal = processor.process_word(&mut state, TokenId::new(0), "hello", &lm);

        assert!(surprisal.bits() > 0.0);
        assert_eq!(state.word_count(), 1);
        assert_eq!(state.forms[0], "hello");
    }

    #[test]
    fn test_process_multiple_words() {
        let processor = IncrementalProcessor::new();
        let mut state = processor.new_state();
        let lm = UniformSurprisalModel::default();

        processor.process_word(&mut state, TokenId::new(0), "the", &lm);
        processor.process_word(&mut state, TokenId::new(1), "cat", &lm);
        processor.process_word(&mut state, TokenId::new(2), "sat", &lm);

        assert_eq!(state.word_count(), 3);
        assert_eq!(state.surprisal_trace.len(), 3);
    }

    #[test]
    fn test_add_sense_choice() {
        let processor = IncrementalProcessor::new();
        let mut state = processor.new_state();
        let lm = UniformSurprisalModel::default();

        processor.process_word(&mut state, TokenId::new(0), "bank", &lm);

        // Add sense choice: 60% financial, 40% river
        let _choice_id = processor.add_sense_choice(&mut state, &[0.6, 0.4]);

        assert!(state.is_ambiguous());
        assert_eq!(state.reading_count(), 2);
    }

    #[test]
    fn test_entropy_computation() {
        let processor = IncrementalProcessor::new();
        let mut state = processor.new_state();
        let lm = UniformSurprisalModel::default();

        processor.process_word(&mut state, TokenId::new(0), "every", &lm);
        let entropy_before = state.entropy();

        // Equal probability alternatives -> higher entropy
        processor.add_sense_choice(&mut state, &[0.5, 0.5]);
        let entropy_after = state.entropy();

        assert!(entropy_after > entropy_before);
    }
}
