//! Modality resolution for events
//!
//! Resolves Kratzerian modal force and flavor from modal auxiliaries
//! and contextual cues (VerbNet classes, FrameNet frames).
//!
//! ## Modal Auxiliaries (Closed Class)
//!
//! Modal verbs in English form a closed class:
//! - Necessity: must, have to, need to, ought to, shall
//! - Possibility: can, could, may, might, will, would
//!
//! ## Modal Flavor Detection
//!
//! Flavor is inferred from context:
//! - Epistemic: knowledge/belief verbs, certainty frames
//! - Deontic: obligation verbs, permission frames
//! - Circumstantial: ability verbs, circumstance frames
//! - Bouletic: desire verbs (VerbNet want-32.1)
//! - Teleological: purpose clauses

use crate::config::EventComposerConfig;
use crate::error::EventResult;
use crate::types::{PredicateInfo, SentenceAnalysis, SentenceMetadata};
use canopy_core::{EventModality, ModalFlavor, ModalForce, UPos};
use canopy_tokenizer::coordinator::Layer1SemanticResult;

/// Resolves modality from sentence context
pub struct ModalityResolver {
    _config: EventComposerConfig,
}

impl ModalityResolver {
    /// Create a new modality resolver
    pub fn new(config: &EventComposerConfig) -> EventResult<Self> {
        Ok(Self {
            _config: config.clone(),
        })
    }

    /// Resolve modality for an event
    ///
    /// Examines auxiliary tokens and context to determine modal force and flavor.
    pub fn resolve(
        &self,
        predicate: &PredicateInfo,
        analysis: &SentenceAnalysis,
    ) -> Option<EventModality> {
        // Find auxiliary tokens for this predicate
        let aux_tokens = self.find_auxiliaries(predicate.token_idx, analysis);

        if aux_tokens.is_empty() {
            return None;
        }

        // Get the primary modal auxiliary
        let primary_modal = aux_tokens
            .iter()
            .find(|t| self.is_modal_auxiliary(&t.lemma))?;

        // Determine force from lemma
        let force = self.determine_force(&primary_modal.lemma);

        // Determine flavor from context
        let flavor = self.determine_flavor(predicate, analysis, &analysis.metadata);

        Some(EventModality {
            force,
            flavor,
            auxiliary: Some(primary_modal.lemma.clone()),
        })
    }

    /// Find auxiliary tokens for a predicate
    fn find_auxiliaries<'a>(
        &self,
        predicate_idx: usize,
        analysis: &'a SentenceAnalysis,
    ) -> Vec<&'a Layer1SemanticResult> {
        // Look for AUX tokens that are dependents of the predicate
        let mut auxiliaries = Vec::new();

        for arc in &analysis.dependencies {
            if arc.head_idx == predicate_idx {
                if let Some(token) = analysis.get_token(arc.dependent_idx) {
                    if matches!(token.pos, Some(UPos::Aux)) {
                        auxiliaries.push(token);
                    }
                }
            }
        }

        // Also check tokens immediately before the predicate
        if predicate_idx > 0 {
            if let Some(prev_token) = analysis.get_token(predicate_idx - 1) {
                if matches!(prev_token.pos, Some(UPos::Aux))
                    && self.is_modal_auxiliary(&prev_token.lemma)
                {
                    auxiliaries.push(prev_token);
                }
            }
        }

        auxiliaries
    }

    /// Check if a lemma is a modal auxiliary
    ///
    /// Modal auxiliaries form a closed class in English.
    fn is_modal_auxiliary(&self, lemma: &str) -> bool {
        matches!(
            lemma.to_lowercase().as_str(),
            "can" | "could" | "may" | "might" | "must" | "shall" | "should" | "will" | "would"
                | "need" // in "need to"
                | "ought" // in "ought to"
                | "have" // in "have to" (but context-dependent)
        )
    }

    /// Determine modal force from lemma
    fn determine_force(&self, lemma: &str) -> ModalForce {
        match lemma.to_lowercase().as_str() {
            // Necessity modals
            "must" | "shall" | "should" | "need" | "ought" | "have" => ModalForce::Necessity,
            // Possibility modals
            "can" | "could" | "may" | "might" | "will" | "would" => ModalForce::Possibility,
            // Default to possibility
            _ => ModalForce::Possibility,
        }
    }

    /// Determine modal flavor from context
    fn determine_flavor(
        &self,
        predicate: &PredicateInfo,
        _analysis: &SentenceAnalysis,
        _metadata: &SentenceMetadata,
    ) -> ModalFlavor {
        // Priority 1: Check VerbNet class for bouletic/teleological
        if let Some(ref vn) = predicate.verbnet_analysis {
            for verb_class in &vn.verb_classes {
                let class_id = &verb_class.id;

                // Bouletic: desire/want verbs
                if class_id.starts_with("want-32") || class_id.starts_with("wish-62") {
                    return ModalFlavor::Bouletic;
                }

                // Teleological: purpose verbs
                if class_id.starts_with("try-61") || class_id.starts_with("attempt-61") {
                    return ModalFlavor::Teleological;
                }
            }
        }

        // Priority 2: Check FrameNet frames
        if let Some(ref fn_analysis) = predicate.framenet_analysis {
            for frame in &fn_analysis.frames {
                let frame_name = frame.name.to_lowercase();

                // Epistemic: knowledge/belief frames
                if frame_name.contains("awareness")
                    || frame_name.contains("certainty")
                    || frame_name.contains("likelihood")
                    || frame_name.contains("opinion")
                    || frame_name.contains("belief")
                {
                    return ModalFlavor::Epistemic;
                }

                // Deontic: obligation/permission frames
                if frame_name.contains("obligation")
                    || frame_name.contains("permission")
                    || frame_name.contains("prohibiting")
                    || frame_name.contains("required")
                {
                    return ModalFlavor::Deontic;
                }

                // Circumstantial: ability frames
                if frame_name.contains("capability") || frame_name.contains("ability") {
                    return ModalFlavor::Circumstantial;
                }
            }
        }

        // Priority 3: Default based on lemma patterns
        // "must" with a stative verb → epistemic ("He must be tired")
        // "can" with an action verb → circumstantial ("She can swim")
        // Otherwise default to epistemic
        ModalFlavor::Epistemic
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modal_auxiliary_detection() {
        let resolver = ModalityResolver::new(&EventComposerConfig::default()).unwrap();

        assert!(resolver.is_modal_auxiliary("can"));
        assert!(resolver.is_modal_auxiliary("must"));
        assert!(resolver.is_modal_auxiliary("will"));
        assert!(!resolver.is_modal_auxiliary("run"));
        assert!(!resolver.is_modal_auxiliary("go"));
    }

    #[test]
    fn test_force_determination() {
        let resolver = ModalityResolver::new(&EventComposerConfig::default()).unwrap();

        assert!(matches!(
            resolver.determine_force("must"),
            ModalForce::Necessity
        ));
        assert!(matches!(
            resolver.determine_force("can"),
            ModalForce::Possibility
        ));
        assert!(matches!(
            resolver.determine_force("might"),
            ModalForce::Possibility
        ));
        assert!(matches!(
            resolver.determine_force("should"),
            ModalForce::Necessity
        ));
    }
}
