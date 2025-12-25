//! Confidence propagation for event composition
//!
//! Combines confidence scores from multiple sources to produce
//! a final event composition confidence.

use crate::types::{ComposedEvent, ComposedEvents, DecomposedEvent};
use canopy_tokenizer::coordinator::Layer1SemanticResult;

/// Weights for different confidence sources
#[derive(Debug, Clone)]
pub struct ConfidenceWeights {
    /// Weight for VerbNet-derived confidence
    pub verbnet: f32,
    /// Weight for treebank dependency confidence
    pub treebank: f32,
    /// Weight for decomposition confidence
    pub decomposition: f32,
    /// Weight for binding confidence
    pub binding: f32,
}

impl Default for ConfidenceWeights {
    fn default() -> Self {
        Self {
            verbnet: 0.30,
            treebank: 0.20,
            decomposition: 0.25,
            binding: 0.25,
        }
    }
}

/// Calculate confidence for composed events
pub struct ConfidenceCalculator {
    weights: ConfidenceWeights,
}

impl ConfidenceCalculator {
    /// Create a new calculator with default weights
    pub fn new() -> Self {
        Self {
            weights: ConfidenceWeights::default(),
        }
    }

    /// Create with custom weights
    pub fn with_weights(weights: ConfidenceWeights) -> Self {
        Self { weights }
    }

    /// Calculate confidence for a single event
    pub fn calculate_event_confidence(
        &self,
        layer1_tokens: &[Layer1SemanticResult],
        decomposition: &DecomposedEvent,
        composed: &ComposedEvent,
    ) -> f32 {
        let verbnet_conf = decomposition.verbnet_confidence.unwrap_or(0.5);
        let treebank_conf = self.average_treebank_confidence(layer1_tokens);
        let decomp_conf = decomposition.confidence;
        let binding_conf = composed.binding_confidence;

        self.weights.verbnet * verbnet_conf
            + self.weights.treebank * treebank_conf
            + self.weights.decomposition * decomp_conf
            + self.weights.binding * binding_conf
    }

    /// Calculate overall confidence for composed events
    pub fn calculate_overall_confidence(&self, events: &[ComposedEvent]) -> f32 {
        if events.is_empty() {
            return 0.0;
        }

        // Use geometric mean for overall confidence
        let product: f32 = events.iter().map(|e| e.overall_confidence()).product();

        product.powf(1.0 / events.len() as f32)
    }

    /// Average treebank confidence from tokens
    fn average_treebank_confidence(&self, tokens: &[Layer1SemanticResult]) -> f32 {
        let confs: Vec<f32> = tokens
            .iter()
            .filter_map(|t| t.treebank.as_ref())
            .map(|tb| tb.confidence)
            .collect();

        if confs.is_empty() {
            0.5 // Default confidence when no treebank data
        } else {
            confs.iter().sum::<f32>() / confs.len() as f32
        }
    }

    /// Penalize confidence based on unbound entities
    pub fn apply_unbound_penalty(&self, base_confidence: f32, unbound_count: usize) -> f32 {
        // Each unbound entity reduces confidence by 10%
        let penalty = 0.1 * unbound_count as f32;
        (base_confidence - penalty).max(0.0)
    }

    /// Boost confidence for multi-source agreement
    pub fn apply_source_agreement_boost(&self, base_confidence: f32, sources: &[String]) -> f32 {
        // Boost if multiple sources agree
        let source_count = sources.len();
        if source_count >= 3 {
            (base_confidence * 1.1).min(1.0)
        } else if source_count >= 2 {
            (base_confidence * 1.05).min(1.0)
        } else {
            base_confidence
        }
    }
}

impl Default for ConfidenceCalculator {
    fn default() -> Self {
        Self::new()
    }
}

/// Update composed events with calculated confidence
pub fn update_confidence(events: &mut ComposedEvents, calculator: &ConfidenceCalculator) {
    // Apply penalties and boosts
    let mut total_confidence = calculator.calculate_overall_confidence(&events.events);

    // Penalty for unbound entities
    total_confidence =
        calculator.apply_unbound_penalty(total_confidence, events.unbound_entities.len());

    // Boost for source agreement
    total_confidence = calculator.apply_source_agreement_boost(total_confidence, &events.sources);

    events.confidence = total_confidence;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_weights_sum_to_one() {
        let weights = ConfidenceWeights::default();
        let sum = weights.verbnet + weights.treebank + weights.decomposition + weights.binding;
        assert!((sum - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_unbound_penalty() {
        let calc = ConfidenceCalculator::new();
        assert_eq!(calc.apply_unbound_penalty(0.8, 0), 0.8);
        assert!((calc.apply_unbound_penalty(0.8, 2) - 0.6).abs() < 0.001);
        assert_eq!(calc.apply_unbound_penalty(0.2, 5), 0.0);
    }

    #[test]
    fn test_source_agreement_boost() {
        let calc = ConfidenceCalculator::new();
        let sources_1 = vec!["VerbNet".to_string()];
        let sources_2 = vec!["VerbNet".to_string(), "FrameNet".to_string()];
        let sources_3 = vec![
            "VerbNet".to_string(),
            "FrameNet".to_string(),
            "WordNet".to_string(),
        ];

        assert_eq!(calc.apply_source_agreement_boost(0.8, &sources_1), 0.8);
        assert!((calc.apply_source_agreement_boost(0.8, &sources_2) - 0.84).abs() < 0.001);
        assert!((calc.apply_source_agreement_boost(0.8, &sources_3) - 0.88).abs() < 0.001);
    }
}
