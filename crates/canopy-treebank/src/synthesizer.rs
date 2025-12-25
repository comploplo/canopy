//! Pattern synthesis for unknown cases
//!
//! This module provides fallback pattern generation when no treebank pattern
//! is found, using VerbNet classes and FrameNet frames to synthesize plausible
//! dependency structures.

use crate::signature::SemanticSignature;
use crate::types::{DependencyPattern, DependencyRelation, PatternSource};
use crate::TreebankResult;
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Pattern synthesizer for generating fallback patterns
#[derive(Debug)]
pub struct PatternSynthesizer {
    /// VerbNet class to dependency mappings
    verbnet_mappings: HashMap<String, Vec<(DependencyRelation, String)>>,
    /// FrameNet frame to dependency mappings
    framenet_mappings: HashMap<String, Vec<(DependencyRelation, String)>>,
    /// Default patterns by POS category
    default_patterns: HashMap<String, Vec<(DependencyRelation, String)>>,
    /// Enable verbose logging
    verbose: bool,
}

impl PatternSynthesizer {
    /// Create a new pattern synthesizer
    pub fn new(verbose: bool) -> Self {
        let mut synthesizer = Self {
            verbnet_mappings: HashMap::new(),
            framenet_mappings: HashMap::new(),
            default_patterns: HashMap::new(),
            verbose,
        };

        synthesizer.initialize_mappings();
        synthesizer
    }

    /// Synthesize a dependency pattern from semantic signature
    pub fn synthesize_pattern(
        &self,
        signature: &SemanticSignature,
    ) -> TreebankResult<DependencyPattern> {
        if self.verbose {
            debug!("Synthesizing pattern for '{}'", signature.lemma);
        }

        // 1. Try VerbNet class mapping
        if let Some(verbnet_class) = &signature.verbnet_class {
            if let Some(dependencies) = self.verbnet_mappings.get(verbnet_class) {
                return Ok(DependencyPattern::new(
                    signature.lemma.clone(),
                    dependencies.clone(),
                    0.7, // Moderate confidence for VerbNet synthesis
                    0,   // No frequency data for synthesized patterns
                    PatternSource::VerbNet(verbnet_class.clone()),
                ));
            }
        }

        // 2. Try FrameNet frame mapping
        if let Some(framenet_frame) = &signature.framenet_frame {
            if let Some(dependencies) = self.framenet_mappings.get(framenet_frame) {
                return Ok(DependencyPattern::new(
                    signature.lemma.clone(),
                    dependencies.clone(),
                    0.6, // Lower confidence for FrameNet synthesis
                    0,
                    PatternSource::FrameNet(framenet_frame.clone()),
                ));
            }
        }

        // 3. Use default pattern for POS category
        let pos_key = format!("{:?}", signature.pos_category);
        if let Some(dependencies) = self.default_patterns.get(&pos_key) {
            return Ok(DependencyPattern::new(
                signature.lemma.clone(),
                dependencies.clone(),
                0.4, // Low confidence for default patterns
                0,
                PatternSource::Default,
            ));
        }

        // 4. Last resort: empty pattern
        warn!(
            "No synthesis possible for '{}', using empty pattern",
            signature.lemma
        );
        Ok(DependencyPattern::new(
            signature.lemma.clone(),
            vec![],
            0.1, // Very low confidence
            0,
            PatternSource::Default,
        ))
    }

    /// Batch synthesize patterns for multiple signatures
    pub fn synthesize_batch(
        &self,
        signatures: &[SemanticSignature],
    ) -> Vec<(SemanticSignature, DependencyPattern)> {
        signatures
            .iter()
            .filter_map(|sig| {
                self.synthesize_pattern(sig)
                    .ok()
                    .map(|pattern| (sig.clone(), pattern))
            })
            .collect()
    }

    /// Initialize VerbNet class to dependency mappings
    fn initialize_verbnet_mappings(&mut self) {
        let mappings = vec![
            // Motion verbs (run, walk, etc.)
            (
                "run-51.3.2",
                vec![
                    (DependencyRelation::NominalSubject, "agent".to_string()),
                    (DependencyRelation::Oblique, "path".to_string()),
                ],
            ),
            (
                "walk-51.3.2",
                vec![
                    (DependencyRelation::NominalSubject, "agent".to_string()),
                    (DependencyRelation::Oblique, "path".to_string()),
                ],
            ),
            (
                "escape-51.1",
                vec![
                    (DependencyRelation::NominalSubject, "agent".to_string()),
                    (DependencyRelation::Oblique, "source".to_string()),
                ],
            ),
            // Transfer verbs (give, send, etc.)
            (
                "give-13.1",
                vec![
                    (DependencyRelation::NominalSubject, "agent".to_string()),
                    (DependencyRelation::Object, "theme".to_string()),
                    (DependencyRelation::IndirectObject, "recipient".to_string()),
                ],
            ),
            (
                "send-11.1",
                vec![
                    (DependencyRelation::NominalSubject, "agent".to_string()),
                    (DependencyRelation::Object, "theme".to_string()),
                    (DependencyRelation::Oblique, "destination".to_string()),
                ],
            ),
            // Creation verbs (make, build, etc.)
            (
                "build-26.1",
                vec![
                    (DependencyRelation::NominalSubject, "agent".to_string()),
                    (DependencyRelation::Object, "product".to_string()),
                    (DependencyRelation::Oblique, "material".to_string()),
                ],
            ),
            (
                "create-26.4",
                vec![
                    (DependencyRelation::NominalSubject, "agent".to_string()),
                    (DependencyRelation::Object, "product".to_string()),
                ],
            ),
            // Perception verbs (see, hear, etc.)
            (
                "see-30.1",
                vec![
                    (
                        DependencyRelation::NominalSubject,
                        "experiencer".to_string(),
                    ),
                    (DependencyRelation::Object, "stimulus".to_string()),
                ],
            ),
            (
                "hear-30.1",
                vec![
                    (
                        DependencyRelation::NominalSubject,
                        "experiencer".to_string(),
                    ),
                    (DependencyRelation::Object, "stimulus".to_string()),
                ],
            ),
            // Communication verbs (tell, say, etc.)
            (
                "tell-37.2",
                vec![
                    (DependencyRelation::NominalSubject, "agent".to_string()),
                    (DependencyRelation::IndirectObject, "recipient".to_string()),
                    (DependencyRelation::Object, "message".to_string()),
                ],
            ),
            (
                "say-37.7",
                vec![
                    (DependencyRelation::NominalSubject, "agent".to_string()),
                    (DependencyRelation::Object, "message".to_string()),
                ],
            ),
            // Change of state verbs
            (
                "break-45.1",
                vec![
                    (DependencyRelation::NominalSubject, "agent".to_string()),
                    (DependencyRelation::Object, "patient".to_string()),
                ],
            ),
            (
                "open-45.4",
                vec![
                    (DependencyRelation::NominalSubject, "agent".to_string()),
                    (DependencyRelation::Object, "patient".to_string()),
                ],
            ),
            // Cooking verbs
            (
                "cooking-45.3",
                vec![
                    (DependencyRelation::NominalSubject, "agent".to_string()),
                    (DependencyRelation::Object, "patient".to_string()),
                    (DependencyRelation::Oblique, "instrument".to_string()),
                ],
            ),
            // Hit verbs
            (
                "hit-18.1",
                vec![
                    (DependencyRelation::NominalSubject, "agent".to_string()),
                    (DependencyRelation::Object, "patient".to_string()),
                    (DependencyRelation::Oblique, "instrument".to_string()),
                ],
            ),
            // Emotion verbs
            (
                "amuse-31.1",
                vec![
                    (DependencyRelation::NominalSubject, "stimulus".to_string()),
                    (DependencyRelation::Object, "experiencer".to_string()),
                ],
            ),
            (
                "admire-31.2",
                vec![
                    (
                        DependencyRelation::NominalSubject,
                        "experiencer".to_string(),
                    ),
                    (DependencyRelation::Object, "stimulus".to_string()),
                ],
            ),
        ];

        for (class, dependencies) in mappings {
            self.verbnet_mappings
                .insert(class.to_string(), dependencies);
        }

        info!(
            "Initialized {} VerbNet class mappings",
            self.verbnet_mappings.len()
        );
    }

    /// Initialize FrameNet frame to dependency mappings
    fn initialize_framenet_mappings(&mut self) {
        let mappings = vec![
            // Motion frames
            (
                "Motion",
                vec![
                    (DependencyRelation::NominalSubject, "theme".to_string()),
                    (DependencyRelation::Oblique, "path".to_string()),
                ],
            ),
            (
                "Self_motion",
                vec![
                    (DependencyRelation::NominalSubject, "self_mover".to_string()),
                    (DependencyRelation::Oblique, "path".to_string()),
                ],
            ),
            // Transfer frames
            (
                "Giving",
                vec![
                    (DependencyRelation::NominalSubject, "donor".to_string()),
                    (DependencyRelation::Object, "theme".to_string()),
                    (DependencyRelation::IndirectObject, "recipient".to_string()),
                ],
            ),
            (
                "Sending",
                vec![
                    (DependencyRelation::NominalSubject, "sender".to_string()),
                    (DependencyRelation::Object, "theme".to_string()),
                    (DependencyRelation::Oblique, "goal".to_string()),
                ],
            ),
            // Creation frames
            (
                "Creating",
                vec![
                    (DependencyRelation::NominalSubject, "creator".to_string()),
                    (DependencyRelation::Object, "created_entity".to_string()),
                ],
            ),
            (
                "Building",
                vec![
                    (DependencyRelation::NominalSubject, "agent".to_string()),
                    (DependencyRelation::Object, "created_entity".to_string()),
                    (DependencyRelation::Oblique, "components".to_string()),
                ],
            ),
            // Perception frames
            (
                "Perception_experience",
                vec![
                    (
                        DependencyRelation::NominalSubject,
                        "perceiver_passive".to_string(),
                    ),
                    (DependencyRelation::Object, "phenomenon".to_string()),
                ],
            ),
            (
                "Becoming_aware",
                vec![
                    (DependencyRelation::NominalSubject, "cognizer".to_string()),
                    (DependencyRelation::Object, "phenomenon".to_string()),
                ],
            ),
            // Communication frames
            (
                "Statement",
                vec![
                    (DependencyRelation::NominalSubject, "speaker".to_string()),
                    (DependencyRelation::Object, "message".to_string()),
                    (DependencyRelation::IndirectObject, "addressee".to_string()),
                ],
            ),
            (
                "Request",
                vec![
                    (DependencyRelation::NominalSubject, "speaker".to_string()),
                    (DependencyRelation::IndirectObject, "addressee".to_string()),
                    (DependencyRelation::Object, "message".to_string()),
                ],
            ),
            // Change frames
            (
                "Cause_change",
                vec![
                    (DependencyRelation::NominalSubject, "agent".to_string()),
                    (DependencyRelation::Object, "entity".to_string()),
                ],
            ),
            (
                "Undergo_change",
                vec![(DependencyRelation::NominalSubject, "entity".to_string())],
            ),
            // Action frames
            (
                "Intentionally_act",
                vec![
                    (DependencyRelation::NominalSubject, "agent".to_string()),
                    (DependencyRelation::Object, "act".to_string()),
                ],
            ),
            (
                "Activity_start",
                vec![
                    (DependencyRelation::NominalSubject, "agent".to_string()),
                    (DependencyRelation::Object, "activity".to_string()),
                ],
            ),
        ];

        for (frame, dependencies) in mappings {
            self.framenet_mappings
                .insert(frame.to_string(), dependencies);
        }

        info!(
            "Initialized {} FrameNet frame mappings",
            self.framenet_mappings.len()
        );
    }

    /// Initialize default patterns by POS category
    fn initialize_default_patterns(&mut self) {
        // Default verb pattern (basic transitive)
        self.default_patterns.insert(
            "Verb".to_string(),
            vec![
                (DependencyRelation::NominalSubject, "agent".to_string()),
                (DependencyRelation::Object, "patient".to_string()),
            ],
        );

        // Default noun pattern (basic nominal)
        self.default_patterns.insert(
            "Noun".to_string(),
            vec![
                (
                    DependencyRelation::AdjectivalModifier,
                    "modifier".to_string(),
                ),
                (DependencyRelation::Determiner, "determiner".to_string()),
            ],
        );

        // Default adjective pattern
        self.default_patterns.insert(
            "Adjective".to_string(),
            vec![(DependencyRelation::AdverbialModifier, "degree".to_string())],
        );

        // Default adverb pattern
        self.default_patterns.insert(
            "Adverb".to_string(),
            vec![], // Adverbs typically don't have dependents
        );

        // Fallback for unknown categories
        self.default_patterns.insert("Other".to_string(), vec![]);

        info!(
            "Initialized {} default POS patterns",
            self.default_patterns.len()
        );
    }

    /// Initialize all mappings
    fn initialize_mappings(&mut self) {
        self.initialize_verbnet_mappings();
        self.initialize_framenet_mappings();
        self.initialize_default_patterns();

        if self.verbose {
            debug!(
                "Initialized pattern synthesizer with {} VerbNet, {} FrameNet, {} default mappings",
                self.verbnet_mappings.len(),
                self.framenet_mappings.len(),
                self.default_patterns.len()
            );
        }
    }

    /// Get available VerbNet classes
    pub fn get_verbnet_classes(&self) -> Vec<&str> {
        self.verbnet_mappings.keys().map(|s| s.as_str()).collect()
    }

    /// Get available FrameNet frames
    pub fn get_framenet_frames(&self) -> Vec<&str> {
        self.framenet_mappings.keys().map(|s| s.as_str()).collect()
    }

    /// Check if a VerbNet class is supported
    pub fn supports_verbnet_class(&self, class: &str) -> bool {
        self.verbnet_mappings.contains_key(class)
    }

    /// Check if a FrameNet frame is supported
    pub fn supports_framenet_frame(&self, frame: &str) -> bool {
        self.framenet_mappings.contains_key(frame)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signature::{PosCategory, SemanticSignature};
    use canopy_engine::LemmaSource;

    #[test]
    fn test_pattern_synthesizer_creation() {
        let synthesizer = PatternSynthesizer::new(false);
        assert!(!synthesizer.verbnet_mappings.is_empty());
        assert!(!synthesizer.framenet_mappings.is_empty());
        assert!(!synthesizer.default_patterns.is_empty());
    }

    #[test]
    fn test_verbnet_pattern_synthesis() {
        let synthesizer = PatternSynthesizer::new(false);
        let signature = SemanticSignature::new(
            "give".to_string(),
            Some("give-13.1".to_string()),
            None,
            PosCategory::Verb,
            LemmaSource::SimpleLemmatizer,
            0.5,
        );

        let pattern = synthesizer.synthesize_pattern(&signature).unwrap();

        assert_eq!(pattern.verb_lemma, "give");
        assert_eq!(pattern.confidence, 0.7);
        assert!(matches!(pattern.source, PatternSource::VerbNet(_)));

        // Should have agent, theme, recipient for give-13.1
        assert!(pattern.has_relation(&DependencyRelation::NominalSubject));
        assert!(pattern.has_relation(&DependencyRelation::Object));
        assert!(pattern.has_relation(&DependencyRelation::IndirectObject));
    }

    #[test]
    fn test_framenet_pattern_synthesis() {
        let synthesizer = PatternSynthesizer::new(false);
        let signature = SemanticSignature::new(
            "motion".to_string(),
            None,
            Some("Motion".to_string()),
            PosCategory::Verb,
            LemmaSource::SimpleLemmatizer,
            0.5,
        );

        let pattern = synthesizer.synthesize_pattern(&signature).unwrap();

        assert_eq!(pattern.verb_lemma, "motion");
        assert_eq!(pattern.confidence, 0.6);
        assert!(matches!(pattern.source, PatternSource::FrameNet(_)));

        // Should have theme and path for Motion frame
        assert!(pattern.has_relation(&DependencyRelation::NominalSubject));
        assert!(pattern.has_relation(&DependencyRelation::Oblique));
    }

    #[test]
    fn test_default_pattern_synthesis() {
        let synthesizer = PatternSynthesizer::new(false);
        let signature = SemanticSignature::simple("unknown".to_string(), PosCategory::Verb);

        let pattern = synthesizer.synthesize_pattern(&signature).unwrap();

        assert_eq!(pattern.verb_lemma, "unknown");
        assert_eq!(pattern.confidence, 0.4);
        assert!(matches!(pattern.source, PatternSource::Default));

        // Should have basic transitive pattern
        assert!(pattern.has_relation(&DependencyRelation::NominalSubject));
        assert!(pattern.has_relation(&DependencyRelation::Object));
    }

    #[test]
    fn test_empty_pattern_fallback() {
        let synthesizer = PatternSynthesizer::new(false);
        let signature = SemanticSignature::simple("unknown".to_string(), PosCategory::Other);

        let pattern = synthesizer.synthesize_pattern(&signature).unwrap();

        assert_eq!(pattern.verb_lemma, "unknown");
        assert_eq!(pattern.confidence, 0.4);
        assert!(matches!(pattern.source, PatternSource::Default));
        assert!(pattern.dependencies.is_empty());
    }

    #[test]
    fn test_batch_synthesis() {
        let synthesizer = PatternSynthesizer::new(false);
        let signatures = vec![
            SemanticSignature::new(
                "run".to_string(),
                Some("run-51.3.2".to_string()),
                None,
                PosCategory::Verb,
                LemmaSource::SimpleLemmatizer,
                0.5,
            ),
            SemanticSignature::simple("walk".to_string(), PosCategory::Verb),
        ];

        let results = synthesizer.synthesize_batch(&signatures);
        assert_eq!(results.len(), 2);

        // First should use VerbNet mapping
        assert!(matches!(results[0].1.source, PatternSource::VerbNet(_)));
        // Second should use default pattern
        assert!(matches!(results[1].1.source, PatternSource::Default));
    }

    #[test]
    fn test_support_queries() {
        let synthesizer = PatternSynthesizer::new(false);

        assert!(synthesizer.supports_verbnet_class("give-13.1"));
        assert!(synthesizer.supports_framenet_frame("Motion"));

        assert!(!synthesizer.supports_verbnet_class("nonexistent-class"));
        assert!(!synthesizer.supports_framenet_frame("NonexistentFrame"));
    }

    #[test]
    fn test_mapping_coverage() {
        let synthesizer = PatternSynthesizer::new(false);

        let verbnet_classes = synthesizer.get_verbnet_classes();
        let framenet_frames = synthesizer.get_framenet_frames();

        // Should have reasonable coverage of common classes and frames
        assert!(verbnet_classes.len() >= 10);
        assert!(framenet_frames.len() >= 10);

        // Check for some expected entries
        assert!(verbnet_classes.contains(&"give-13.1"));
        assert!(framenet_frames.contains(&"Motion"));
    }

    #[test]
    fn test_confidence_levels() {
        let synthesizer = PatternSynthesizer::new(false);

        // VerbNet should have higher confidence than FrameNet
        let verbnet_sig = SemanticSignature::new(
            "give".to_string(),
            Some("give-13.1".to_string()),
            None,
            PosCategory::Verb,
            LemmaSource::SimpleLemmatizer,
            0.5,
        );
        let verbnet_pattern = synthesizer.synthesize_pattern(&verbnet_sig).unwrap();

        let framenet_sig = SemanticSignature::new(
            "give".to_string(),
            None,
            Some("Giving".to_string()),
            PosCategory::Verb,
            LemmaSource::SimpleLemmatizer,
            0.5,
        );
        let framenet_pattern = synthesizer.synthesize_pattern(&framenet_sig).unwrap();

        assert!(verbnet_pattern.confidence > framenet_pattern.confidence);
    }
}
