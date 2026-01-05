//! Disambiguation strategies for selecting among multiple readings.
//!
//! Provides various approaches to resolving ambiguity:
//! - Surprisal-based (minimum surprisal = highest probability)
//! - Confidence-based (use provider confidence scores)
//! - Entropy-based (maximize uncertainty reduction)
//! - Hybrid (weighted combination)
//! - Interactive (return all readings to caller)

use super::types::{PackedSemantics, Reading};
use crate::kernel::discourse::Drs;
use crate::kernel::incremental::{IncrementalState, LanguageModel};

/// Context for disambiguation decisions.
pub struct DisambiguationContext<'a> {
    /// Language model for probability estimates.
    pub language_model: Option<&'a dyn LanguageModel>,
    /// Incremental processing state (if available).
    pub incremental_state: Option<&'a IncrementalState>,
    /// Discourse history for context-based disambiguation.
    pub discourse_history: &'a [Drs],
}

impl<'a> DisambiguationContext<'a> {
    /// Create a minimal context with no history.
    #[must_use]
    pub fn minimal() -> Self {
        Self {
            language_model: None,
            incremental_state: None,
            discourse_history: &[],
        }
    }

    /// Create context with a language model.
    #[must_use]
    pub fn with_lm(lm: &'a dyn LanguageModel) -> Self {
        Self {
            language_model: Some(lm),
            incremental_state: None,
            discourse_history: &[],
        }
    }

    /// Add incremental state.
    #[must_use]
    pub fn with_incremental(mut self, state: &'a IncrementalState) -> Self {
        self.incremental_state = Some(state);
        self
    }

    /// Add discourse history.
    #[must_use]
    pub fn with_history(mut self, history: &'a [Drs]) -> Self {
        self.discourse_history = history;
        self
    }
}

/// Trait for disambiguation strategies.
pub trait Disambiguator: Send + Sync {
    /// Select the best reading from a packed representation.
    fn select_reading(
        &self,
        packed: &PackedSemantics,
        ctx: &DisambiguationContext,
    ) -> Option<Reading>;

    /// Rank all readings by preference.
    ///
    /// Returns readings with scores (higher = better).
    fn rank_readings(
        &self,
        packed: &PackedSemantics,
        ctx: &DisambiguationContext,
    ) -> Vec<(Reading, f64)>;

    /// Get the name of this disambiguation strategy.
    fn name(&self) -> &'static str;
}

/// Minimum surprisal disambiguator.
///
/// Selects the reading with lowest total surprisal (= highest probability).
/// This is the primary surprisal-based approach from Hale (2001), Levy (2008).
#[derive(Debug, Clone, Default)]
pub struct MinSurprisalDisambiguator;

impl Disambiguator for MinSurprisalDisambiguator {
    fn select_reading(
        &self,
        packed: &PackedSemantics,
        _ctx: &DisambiguationContext,
    ) -> Option<Reading> {
        // Select reading with lowest surprisal (highest probability)
        packed.readings().min_by(|a, b| {
            a.total_surprisal
                .bits()
                .partial_cmp(&b.total_surprisal.bits())
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    fn rank_readings(
        &self,
        packed: &PackedSemantics,
        _ctx: &DisambiguationContext,
    ) -> Vec<(Reading, f64)> {
        let mut readings: Vec<_> = packed
            .readings()
            .map(|r| {
                // Score = negative surprisal (so higher is better)
                let score = -r.total_surprisal.bits();
                (r, score)
            })
            .collect();

        readings.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        readings
    }

    fn name(&self) -> &'static str {
        "min-surprisal"
    }
}

/// Confidence-based disambiguator.
///
/// Uses confidence scores from semantic resource providers.
/// This is the legacy approach, useful when no LM is available.
#[derive(Debug, Clone, Default)]
pub struct ConfidenceDisambiguator;

impl Disambiguator for ConfidenceDisambiguator {
    fn select_reading(
        &self,
        packed: &PackedSemantics,
        _ctx: &DisambiguationContext,
    ) -> Option<Reading> {
        packed.readings().max_by(|a, b| {
            a.confidence
                .partial_cmp(&b.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    fn rank_readings(
        &self,
        packed: &PackedSemantics,
        _ctx: &DisambiguationContext,
    ) -> Vec<(Reading, f64)> {
        let mut readings: Vec<_> = packed
            .readings()
            .map(|r| (r.clone(), f64::from(r.confidence)))
            .collect();

        readings.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        readings
    }

    fn name(&self) -> &'static str {
        "confidence"
    }
}

/// Hybrid disambiguator combining surprisal and confidence.
///
/// `score = surprisal_weight * (-surprisal) + confidence_weight * confidence`
#[derive(Debug, Clone)]
pub struct HybridDisambiguator {
    /// Weight for surprisal component (negative surprisal).
    pub surprisal_weight: f64,
    /// Weight for confidence component.
    pub confidence_weight: f64,
}

impl Default for HybridDisambiguator {
    fn default() -> Self {
        Self {
            surprisal_weight: 0.7,
            confidence_weight: 0.3,
        }
    }
}

impl HybridDisambiguator {
    /// Create a new hybrid disambiguator with custom weights.
    #[must_use]
    pub const fn new(surprisal_weight: f64, confidence_weight: f64) -> Self {
        Self {
            surprisal_weight,
            confidence_weight,
        }
    }

    fn score(&self, reading: &Reading) -> f64 {
        let surprisal_score = -reading.total_surprisal.bits();
        let confidence_score = f64::from(reading.confidence);

        // Normalize surprisal to roughly 0-1 range (assuming max ~20 bits)
        let normalized_surprisal = (20.0 + surprisal_score) / 20.0;

        self.surprisal_weight * normalized_surprisal + self.confidence_weight * confidence_score
    }
}

impl Disambiguator for HybridDisambiguator {
    fn select_reading(
        &self,
        packed: &PackedSemantics,
        _ctx: &DisambiguationContext,
    ) -> Option<Reading> {
        packed.readings().max_by(|a, b| {
            self.score(a)
                .partial_cmp(&self.score(b))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    fn rank_readings(
        &self,
        packed: &PackedSemantics,
        _ctx: &DisambiguationContext,
    ) -> Vec<(Reading, f64)> {
        let mut readings: Vec<_> = packed
            .readings()
            .map(|r| {
                let score = self.score(&r);
                (r, score)
            })
            .collect();

        readings.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        readings
    }

    fn name(&self) -> &'static str {
        "hybrid"
    }
}

/// Entropy reduction disambiguator.
///
/// Prefers readings that maximize reduction in uncertainty.
/// Based on Roark et al. (2009) entropy reduction hypothesis.
#[derive(Debug, Clone, Default)]
pub struct EntropyReductionDisambiguator;

impl Disambiguator for EntropyReductionDisambiguator {
    fn select_reading(
        &self,
        packed: &PackedSemantics,
        ctx: &DisambiguationContext,
    ) -> Option<Reading> {
        // If we have incremental state, use it for entropy calculation
        if let Some(state) = ctx.incremental_state {
            // Prefer reading from beam that reduced entropy most
            if let Some(_best) = state.best_reading() {
                // Convert beam entry to Reading
                // For now, fall back to probability-based selection
                return packed.readings().max_by(|a, b| {
                    a.probability
                        .partial_cmp(&b.probability)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
            }
        }

        // Fall back to probability
        packed.readings().max_by(|a, b| {
            a.probability
                .partial_cmp(&b.probability)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    fn rank_readings(
        &self,
        packed: &PackedSemantics,
        _ctx: &DisambiguationContext,
    ) -> Vec<(Reading, f64)> {
        // Score by probability (proxy for entropy reduction)
        let mut readings: Vec<_> = packed
            .readings()
            .map(|r| (r.clone(), r.probability))
            .collect();

        readings.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        readings
    }

    fn name(&self) -> &'static str {
        "entropy-reduction"
    }
}

/// Interactive disambiguator.
///
/// Returns all readings rather than selecting one.
/// Useful for systems that want to present options to users
/// or defer disambiguation to a later stage.
#[derive(Debug, Clone, Default)]
pub struct InteractiveDisambiguator;

impl Disambiguator for InteractiveDisambiguator {
    fn select_reading(
        &self,
        packed: &PackedSemantics,
        _ctx: &DisambiguationContext,
    ) -> Option<Reading> {
        // Don't select - return the first reading but caller should check rank_readings
        packed.readings().next()
    }

    fn rank_readings(
        &self,
        packed: &PackedSemantics,
        _ctx: &DisambiguationContext,
    ) -> Vec<(Reading, f64)> {
        // Return all readings with equal scores
        packed.readings().map(|r| (r, 1.0)).collect()
    }

    fn name(&self) -> &'static str {
        "interactive"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::underspec::types::{
        Alternative, ChoiceId, ChoicePoint, ChoiceType, SharedStructure,
    };
    use crate::runtime::{SenseId, TokenId};

    fn create_test_packed() -> PackedSemantics {
        let mut packed = PackedSemantics::new(SharedStructure::default());

        packed.add_choice(ChoicePoint::new(
            ChoiceId::new(0),
            ChoiceType::LexicalSense {
                token_id: TokenId::new(0),
                senses: vec![SenseId::new("bank.01"), SenseId::new("bank.02")],
            },
            vec![
                Alternative::new(0, 0.7, "financial"),
                Alternative::new(1, 0.3, "river"),
            ],
        ));

        packed
    }

    #[test]
    fn test_min_surprisal_disambiguator() {
        let packed = create_test_packed();
        let ctx = DisambiguationContext::minimal();
        let disamb = MinSurprisalDisambiguator;

        // Without surprisal set, should work based on probability
        let best = disamb.select_reading(&packed, &ctx);
        assert!(best.is_some());
    }

    #[test]
    fn test_confidence_disambiguator() {
        let packed = create_test_packed();
        let ctx = DisambiguationContext::minimal();
        let disamb = ConfidenceDisambiguator;

        let best = disamb.select_reading(&packed, &ctx);
        assert!(best.is_some());
    }

    #[test]
    fn test_hybrid_disambiguator() {
        let packed = create_test_packed();
        let ctx = DisambiguationContext::minimal();
        let disamb = HybridDisambiguator::new(0.5, 0.5);

        let ranked = disamb.rank_readings(&packed, &ctx);
        assert_eq!(ranked.len(), 2);
    }

    #[test]
    fn test_interactive_disambiguator() {
        let packed = create_test_packed();
        let ctx = DisambiguationContext::minimal();
        let disamb = InteractiveDisambiguator;

        let ranked = disamb.rank_readings(&packed, &ctx);
        assert_eq!(ranked.len(), 2);

        // All should have equal scores
        assert!((ranked[0].1 - ranked[1].1).abs() < f64::EPSILON);
    }

    #[test]
    fn test_disambiguator_names() {
        assert_eq!(MinSurprisalDisambiguator.name(), "min-surprisal");
        assert_eq!(ConfidenceDisambiguator.name(), "confidence");
        assert_eq!(HybridDisambiguator::default().name(), "hybrid");
        assert_eq!(EntropyReductionDisambiguator.name(), "entropy-reduction");
        assert_eq!(InteractiveDisambiguator.name(), "interactive");
    }
}
