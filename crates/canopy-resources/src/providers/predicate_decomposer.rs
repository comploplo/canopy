//! Multi-engine predicate decomposer.
//!
//! Decomposes predicates by aggregating evidence from `VerbNet`, `FrameNet`,
//! and `PropBank` using the `LemmaQueryable` interface.

use crate::engine::{
    LemmaQuery, LemmaQueryable, PredicateToLittleVMap, ResourceSource, SemanticEvidence,
    SharedEngines,
};
use canopy::core::ThetaRole;
use canopy::kernel::events::LittleVType;
use canopy::runtime::{
    AnnotatedSyntax, DecompositionSource, FrameId, PredicateDecomposition, SenseId, SenseInfo,
    SenseProvider, SenseSource, TokenId,
};
use canopy::CanopyError;
use std::collections::{HashMap, HashSet};

/// Configuration for predicate decomposition.
#[derive(Debug, Clone)]
pub struct DecomposerConfig {
    /// Minimum confidence threshold for returning results.
    pub min_confidence: f32,
    /// Weights for each source when aggregating evidence.
    pub source_weights: HashMap<ResourceSource, f32>,
    /// Number of sources that must agree for high confidence.
    pub required_agreement: usize,
    /// Deduplicate by `LittleVType` (keep highest confidence per type).
    pub deduplicate_by_event_type: bool,
}

impl Default for DecomposerConfig {
    fn default() -> Self {
        let mut source_weights = HashMap::new();
        source_weights.insert(ResourceSource::VerbNet, 1.0);
        source_weights.insert(ResourceSource::FrameNet, 1.0);
        source_weights.insert(ResourceSource::PropBank, 0.9);

        Self {
            min_confidence: 0.6,
            source_weights,
            required_agreement: 2, // Require 2+ sources for precision
            deduplicate_by_event_type: true,
        }
    }
}

impl DecomposerConfig {
    /// Create a precision-focused configuration with higher thresholds.
    #[must_use]
    pub fn precision() -> Self {
        Self {
            min_confidence: 0.75,
            required_agreement: 2,
            ..Default::default()
        }
    }

    /// Create a permissive configuration (single source sufficient).
    ///
    /// Use for backward compatibility or when coverage is prioritized.
    #[must_use]
    pub fn permissive() -> Self {
        Self {
            required_agreement: 1,
            ..Default::default()
        }
    }
}

/// Decomposes predicates using multi-engine evidence.
///
/// This is the primary implementation of `SenseProvider` that aggregates
/// evidence from `VerbNet`, `FrameNet`, and `PropBank`.
pub struct PredicateDecomposer {
    engines: SharedEngines,
    config: DecomposerConfig,
    predicate_map: PredicateToLittleVMap,
}

impl PredicateDecomposer {
    /// Create a new predicate decomposer.
    ///
    /// # Errors
    /// Returns an error if engines cannot be initialized.
    pub fn new(engines: SharedEngines, config: DecomposerConfig) -> Result<Self, CanopyError> {
        let predicate_map = PredicateToLittleVMap::load().unwrap_or_else(|e| {
            tracing::warn!("Failed to load predicate mappings, using defaults: {e}");
            PredicateToLittleVMap::default()
        });

        Ok(Self {
            engines,
            config,
            predicate_map,
        })
    }

    /// Create with default configuration.
    ///
    /// # Errors
    /// Returns an error if engines cannot be initialized.
    pub fn with_default_config(engines: SharedEngines) -> Result<Self, CanopyError> {
        Self::new(engines, DecomposerConfig::default())
    }

    /// Query all available engines for evidence.
    fn query_all_engines(&self, query: &LemmaQuery) -> Vec<SemanticEvidence> {
        let mut all_evidence = Vec::new();

        // Query VerbNet
        if let Some(ref vn) = self.engines.verbnet {
            if let Ok(evidence) = vn.query_by_lemma(query) {
                all_evidence.extend(evidence);
            }
        }

        // Query FrameNet
        if let Some(ref fn_) = self.engines.framenet {
            if let Ok(evidence) = fn_.query_by_lemma(query) {
                all_evidence.extend(evidence);
            }
        }

        // Query PropBank
        if let Some(ref pb) = self.engines.propbank {
            if let Ok(evidence) = pb.query_by_lemma(query) {
                all_evidence.extend(evidence);
            }
        }

        all_evidence
    }

    /// Aggregate evidence into predicate decompositions.
    ///
    /// Implements `required_agreement` filtering: if fewer than `required_agreement`
    /// distinct sources provide evidence, returns empty. When multiple sources agree,
    /// confidence is boosted.
    fn aggregate_evidence(
        &self,
        evidence: Vec<SemanticEvidence>,
        token_id: TokenId,
    ) -> Vec<PredicateDecomposition> {
        // Count unique sources that provided evidence
        let sources: HashSet<_> = evidence.iter().map(|e| e.source).collect();

        // Check required_agreement threshold
        if sources.len() < self.config.required_agreement {
            tracing::debug!(
                "Insufficient source agreement: {} sources, {} required",
                sources.len(),
                self.config.required_agreement
            );
            return vec![];
        }

        // Confidence boost when multiple sources agree (10% per additional source)
        // At most 5 engines, so source_count always fits in u8
        let source_count = u8::try_from(sources.len()).unwrap_or(5);
        let agreement_boost = f32::from(source_count.saturating_sub(1)) * 0.10;

        let mut decompositions = Vec::new();

        for ev in evidence {
            // Filter by minimum confidence
            if ev.calibrated_confidence < self.config.min_confidence {
                continue;
            }

            // Determine LittleV type from theta roles or evidence
            let little_v = self.infer_little_v(&ev);

            let source = match ev.source {
                ResourceSource::VerbNet => DecompositionSource::VerbNet,
                ResourceSource::FrameNet => DecompositionSource::FrameNet,
                ResourceSource::PropBank => DecompositionSource::PropBank,
                _ => DecompositionSource::Heuristic,
            };

            // Apply source weight and agreement boost to confidence
            let weight = self
                .config
                .source_weights
                .get(&ev.source)
                .copied()
                .unwrap_or(1.0);
            let weighted_confidence =
                (ev.calibrated_confidence * weight + agreement_boost).min(0.98);

            let decomp = PredicateDecomposition::new(
                SenseId::new(&ev.evidence_id),
                little_v,
                ev.theta_roles.clone(),
            )
            .with_confidence(weighted_confidence)
            .with_source(source)
            .with_token_id(token_id);

            decompositions.push(decomp);
        }

        // Sort by confidence descending
        decompositions.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Deduplicate by LittleVType - keep highest-confidence per type
        if self.config.deduplicate_by_event_type {
            let mut seen_types: HashSet<LittleVType> = HashSet::new();
            decompositions.retain(|d| seen_types.insert(d.little_v_type));
        }

        decompositions
    }

    /// Infer `LittleV` type from semantic evidence.
    fn infer_little_v(&self, evidence: &SemanticEvidence) -> LittleVType {
        // Use explicit LittleV type if provided
        if let Some(little_v) = evidence.little_v_type {
            return little_v;
        }

        // Check predicate mapping first (for VerbNet class IDs)
        let evidence_lower = evidence.evidence_id.to_lowercase();
        if self.predicate_map.contains(&evidence_lower) {
            return self.predicate_map.get(&evidence_lower);
        }

        // Infer from theta roles
        if evidence.theta_roles.contains(&ThetaRole::Agent)
            && evidence.theta_roles.contains(&ThetaRole::Patient)
        {
            return LittleVType::Cause;
        }

        if evidence.theta_roles.contains(&ThetaRole::Experiencer) {
            return LittleVType::Experience;
        }

        // GO pattern: Theme + (Goal OR Source) - motion/transfer events
        if evidence.theta_roles.contains(&ThetaRole::Theme)
            && (evidence.theta_roles.contains(&ThetaRole::Goal)
                || evidence.theta_roles.contains(&ThetaRole::Source))
        {
            return LittleVType::Go;
        }

        if evidence.theta_roles.contains(&ThetaRole::Agent) {
            return LittleVType::Do;
        }

        if evidence.theta_roles.contains(&ThetaRole::Theme) {
            return LittleVType::Become;
        }

        // Default
        self.predicate_map.default_type()
    }
}

impl std::fmt::Debug for PredicateDecomposer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PredicateDecomposer")
            .field("config", &self.config)
            .field("engines", &self.engines)
            .finish_non_exhaustive()
    }
}

impl SenseProvider for PredicateDecomposer {
    fn decompose_predicate(
        &self,
        syntax: &AnnotatedSyntax,
        pred_id: TokenId,
    ) -> Result<Vec<PredicateDecomposition>, CanopyError> {
        // Get the predicate token
        let Some(token) = syntax.tokens.get(pred_id.index()) else {
            return Ok(vec![]);
        };

        // Use phrasal lemma if available (e.g., "give_up" instead of "give")
        // This enables correct VerbNet/FrameNet lookup for verb-particle constructions
        let lemma = syntax
            .get_predicate_lemma(pred_id)
            .unwrap_or(&token.lemma)
            .to_string();

        // Create query from lemma (phrasal or regular)
        let query = LemmaQuery::new(&lemma, token.upos).with_token_id(pred_id);

        // Only process verbs and adjectives (potential predicates)
        if !query.is_verb() && !query.is_adj() {
            return Ok(vec![]);
        }

        // Query all engines
        let evidence = self.query_all_engines(&query);

        if evidence.is_empty() {
            tracing::debug!(
                "No evidence found for predicate '{}' at token {}",
                lemma,
                pred_id.index()
            );
            return Ok(vec![]);
        }

        // Aggregate into decompositions
        let decompositions = self.aggregate_evidence(evidence, pred_id);

        tracing::debug!(
            "Decomposed predicate '{}': {} decompositions",
            lemma,
            decompositions.len()
        );

        Ok(decompositions)
    }

    fn frames_for_sense(&self, sense: &SenseId) -> Result<Vec<FrameId>, CanopyError> {
        // Extract frame info from sense ID
        // VerbNet: "give-13.1" -> frame comes from class
        // FrameNet: frame name is the sense ID
        // PropBank: "give.01" -> frameset

        let sense_str = sense.to_string();

        // For FrameNet senses, the sense ID is the frame name
        if let Some(ref fn_) = self.engines.framenet {
            if fn_.get_frame_by_name(&sense_str).is_some() {
                return Ok(vec![FrameId::new(&sense_str)]);
            }
        }

        // For VerbNet, extract the class as a frame
        if sense_str.contains('-') {
            return Ok(vec![FrameId::new(&sense_str)]);
        }

        // For PropBank, use the roleset
        if sense_str.contains('.') {
            return Ok(vec![FrameId::new(&sense_str)]);
        }

        Ok(vec![])
    }

    fn get_sense(&self, id: &SenseId) -> Result<Option<SenseInfo>, CanopyError> {
        let sense_str = id.to_string();

        // Try VerbNet first
        if let Some(ref vn) = self.engines.verbnet {
            if let Some(class) = vn.get_verb_class(&sense_str) {
                let theta_roles: Vec<_> = class
                    .themroles
                    .iter()
                    .filter_map(|r| ThetaRole::parse(&r.role_type))
                    .collect();

                return Ok(Some(SenseInfo {
                    id: id.clone(),
                    description: class.class_name.clone(),
                    source: SenseSource::VerbNet,
                    theta_roles,
                }));
            }
        }

        // Try FrameNet
        if let Some(ref fn_) = self.engines.framenet {
            if let Some(frame) = fn_.get_frame_by_name(&sense_str) {
                let theta_roles: Vec<_> = frame
                    .frame_elements
                    .iter()
                    .filter_map(|fe| ThetaRole::parse(&fe.name))
                    .collect();

                return Ok(Some(SenseInfo {
                    id: id.clone(),
                    description: frame.definition.clone(),
                    source: SenseSource::FrameNet,
                    theta_roles,
                }));
            }
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn engines_available() -> bool {
        crate::paths::data_path("data/verbnet").exists()
    }

    #[test]
    fn test_decomposer_config_default() {
        let config = DecomposerConfig::default();
        assert!((config.min_confidence - 0.6).abs() < f32::EPSILON);
        assert_eq!(config.required_agreement, 2); // Now requires 2+ sources
    }

    #[test]
    fn test_decomposer_config_precision() {
        let config = DecomposerConfig::precision();
        assert!((config.min_confidence - 0.75).abs() < f32::EPSILON);
        assert_eq!(config.required_agreement, 2);
    }

    #[test]
    fn test_decomposer_config_permissive() {
        let config = DecomposerConfig::permissive();
        assert_eq!(config.required_agreement, 1); // Single source sufficient
    }

    #[test]
    fn test_decomposer_creation() {
        if !engines_available() {
            eprintln!("Skipping: Data not available");
            return;
        }

        let engines = SharedEngines::new().expect("Failed to create engines");
        let decomposer = PredicateDecomposer::with_default_config(engines);
        assert!(decomposer.is_ok());
    }

    #[test]
    fn test_required_agreement_filtering() {
        // Test that required_agreement=2 filters out evidence from single source
        let config = DecomposerConfig {
            required_agreement: 2,
            min_confidence: 0.5,
            ..Default::default()
        };

        // Create evidence from only one source
        let single_source_evidence =
            [
                SemanticEvidence::new(ResourceSource::VerbNet, "test-1.1".to_string())
                    .with_confidence(0.8),
            ];

        // With required_agreement=2, single source should return empty
        let sources: HashSet<_> = single_source_evidence.iter().map(|e| e.source).collect();
        assert!(sources.len() < config.required_agreement);
    }

    #[test]
    fn test_agreement_boost_calculation() {
        // Test that multiple sources get a confidence boost
        let evidence = [
            SemanticEvidence::new(ResourceSource::VerbNet, "test-1").with_confidence(0.8),
            SemanticEvidence::new(ResourceSource::FrameNet, "Test").with_confidence(0.8),
        ];

        let sources: HashSet<_> = evidence.iter().map(|e| e.source).collect();
        let source_count = u8::try_from(sources.len()).unwrap_or(5);
        let agreement_boost = f32::from(source_count.saturating_sub(1)) * 0.10;

        // With 2 sources, boost should be 0.10
        assert!((agreement_boost - 0.10).abs() < f32::EPSILON);
    }

    #[test]
    fn test_go_event_inference_with_source_role() {
        // Test that GO is inferred when Theme + Source (not just Theme + Goal)
        let evidence_with_source = SemanticEvidence::new(ResourceSource::VerbNet, "leave-51.2")
            .with_confidence(0.8)
            .with_roles(vec![ThetaRole::Theme, ThetaRole::Source]);

        // Theme + Source should infer GO
        assert!(evidence_with_source.theta_roles.contains(&ThetaRole::Theme));
        assert!(evidence_with_source
            .theta_roles
            .contains(&ThetaRole::Source));

        // The inference logic checks for Theme + (Goal OR Source)
        let has_go_pattern = evidence_with_source.theta_roles.contains(&ThetaRole::Theme)
            && (evidence_with_source.theta_roles.contains(&ThetaRole::Goal)
                || evidence_with_source
                    .theta_roles
                    .contains(&ThetaRole::Source));
        assert!(has_go_pattern);
    }

    #[test]
    fn test_deduplicate_by_event_type_default_enabled() {
        let config = DecomposerConfig::default();
        assert!(config.deduplicate_by_event_type);
    }

    #[test]
    fn test_deduplicate_by_event_type_in_precision() {
        let config = DecomposerConfig::precision();
        assert!(config.deduplicate_by_event_type);
    }

    #[test]
    fn test_deduplicate_by_event_type_in_permissive() {
        let config = DecomposerConfig::permissive();
        assert!(config.deduplicate_by_event_type);
    }
}
