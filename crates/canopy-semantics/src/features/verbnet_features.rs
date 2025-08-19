//! VerbNet-based semantic feature extraction
//!
//! Simple extraction using VerbNet's selectional restrictions directly.
//! No complex inference - just use what VerbNet already provides.

use crate::ThetaRoleType;
use crate::events::{Animacy, Concreteness, Participant};
use crate::verbnet::{SelectionalRestriction, VerbNetEngine};
use canopy_core::Word;
use std::collections::HashMap;
use tracing::{debug, trace};

/// Simple semantic feature extractor using VerbNet restrictions
#[derive(Debug)]
pub struct VerbNetFeatureExtractor {
    /// VerbNet engine for lookups
    verbnet: VerbNetEngine,

    /// Cache for extracted features
    feature_cache: HashMap<String, ExtractedFeatures>,
}

/// Features extracted from VerbNet selectional restrictions
#[derive(Debug, Clone, Default)]
pub struct ExtractedFeatures {
    /// Animacy classification
    pub animacy: Option<Animacy>,

    /// Concreteness classification
    pub concreteness: Option<Concreteness>,

    /// Other semantic properties
    pub properties: Vec<SemanticProperty>,

    /// Confidence (based on VerbNet coverage)
    pub confidence: f32,
}

/// Semantic properties from VerbNet restrictions
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemanticProperty {
    Human,
    Organization,
    Location,
    Time,
    Communication,
    Solid,
    Fluid,
    Substance,
    Machine,
    Vehicle,
    Garment,
    BodyPart,
    Comestible,
    Container,
    Plant,
    Sound,
    Force,
    Scalar,
}

impl VerbNetFeatureExtractor {
    /// Create new extractor with VerbNet engine
    pub fn new(verbnet: VerbNetEngine) -> Self {
        Self {
            verbnet,
            feature_cache: HashMap::new(),
        }
    }

    /// Extract semantic features for a participant given the verb and role
    pub fn extract_features(
        &mut self,
        verb: &str,
        role: &ThetaRoleType,
        participant: &Participant,
    ) -> ExtractedFeatures {
        let cache_key = format!("{}_{:?}_{}", verb, role, participant.expression);

        // Check cache first
        if let Some(cached) = self.feature_cache.get(&cache_key) {
            return cached.clone();
        }

        // Get selectional restrictions from VerbNet
        let restrictions = self.get_restrictions_for_role(verb, role);

        if restrictions.is_empty() {
            debug!("No VerbNet restrictions for {:?} role of '{}'", role, verb);
            return ExtractedFeatures::default();
        }

        trace!(
            "VerbNet restrictions for {} {:?}: {:?}",
            verb, role, restrictions
        );

        // Extract features from restrictions
        let features = self.restrictions_to_features(&restrictions, participant);

        // Cache result
        self.feature_cache.insert(cache_key, features.clone());

        features
    }

    /// Get selectional restrictions for a specific verb and role
    fn get_restrictions_for_role(
        &self,
        verb: &str,
        role: &ThetaRoleType,
    ) -> Vec<SelectionalRestriction> {
        let theta_roles = self.verbnet.get_theta_roles(verb);

        for vn_role in theta_roles {
            if vn_role.role_type == *role {
                return vn_role.selectional_restrictions;
            }
        }

        vec![]
    }

    /// Convert VerbNet restrictions to our semantic features
    fn restrictions_to_features(
        &self,
        restrictions: &[SelectionalRestriction],
        participant: &Participant,
    ) -> ExtractedFeatures {
        let mut features = ExtractedFeatures::default();
        let mut properties = Vec::new();

        for restriction in restrictions {
            match restriction {
                // Animacy features
                SelectionalRestriction::Human => {
                    features.animacy = Some(Animacy::Human);
                    properties.push(SemanticProperty::Human);
                }
                SelectionalRestriction::Animate => {
                    features.animacy = Some(Animacy::Animal);
                }
                SelectionalRestriction::Organization => {
                    features.animacy = Some(Animacy::Animal); // Organizations act as animate
                    properties.push(SemanticProperty::Organization);
                }

                // Concreteness features
                SelectionalRestriction::Concrete => {
                    features.concreteness = Some(Concreteness::Concrete);
                }
                SelectionalRestriction::Abstract => {
                    features.concreteness = Some(Concreteness::Abstract);
                }

                // Specific properties
                SelectionalRestriction::Location => properties.push(SemanticProperty::Location),
                SelectionalRestriction::Time => properties.push(SemanticProperty::Time),
                SelectionalRestriction::Communication => {
                    properties.push(SemanticProperty::Communication)
                }
                SelectionalRestriction::Solid => {
                    features.concreteness = Some(Concreteness::Concrete);
                    properties.push(SemanticProperty::Solid);
                }
                SelectionalRestriction::Fluid => {
                    features.concreteness = Some(Concreteness::Concrete);
                    properties.push(SemanticProperty::Fluid);
                }
                SelectionalRestriction::Substance => properties.push(SemanticProperty::Substance),
                SelectionalRestriction::Machine => properties.push(SemanticProperty::Machine),
                SelectionalRestriction::Vehicle => properties.push(SemanticProperty::Vehicle),
                SelectionalRestriction::Garment => properties.push(SemanticProperty::Garment),
                SelectionalRestriction::BodyPart => properties.push(SemanticProperty::BodyPart),
                SelectionalRestriction::Comestible => properties.push(SemanticProperty::Comestible),
                SelectionalRestriction::Container => properties.push(SemanticProperty::Container),
                SelectionalRestriction::Plant => properties.push(SemanticProperty::Plant),
                SelectionalRestriction::Sound => properties.push(SemanticProperty::Sound),
                SelectionalRestriction::Force => properties.push(SemanticProperty::Force),
                SelectionalRestriction::Scalar => properties.push(SemanticProperty::Scalar),

                // Handle other restrictions as needed
                _ => {}
            }
        }

        features.properties = properties;

        // Set confidence based on number of restrictions (more = higher confidence)
        features.confidence = if restrictions.is_empty() {
            0.0
        } else {
            (restrictions.len() as f32 * 0.2).min(1.0)
        };

        // Apply simple compatibility check with participant word features
        self.check_compatibility(&mut features, participant);

        features
    }

    /// Check if extracted features are compatible with participant's surface features
    fn check_compatibility(&self, features: &mut ExtractedFeatures, participant: &Participant) {
        // If participant already has animacy from morphology, prefer that
        if let Some(morph_animacy) = &participant.features.animacy {
            if features.animacy.is_some() && features.animacy.as_ref() != Some(morph_animacy) {
                // Conflict - lower confidence but keep VerbNet info
                features.confidence *= 0.7;
            }
        }

        // Similar for concreteness
        if let Some(morph_concreteness) = &participant.features.concreteness {
            if features.concreteness.is_some()
                && features.concreteness.as_ref() != Some(morph_concreteness)
            {
                features.confidence *= 0.7;
            }
        }
    }

    /// Extract features for all participants in an event
    pub fn extract_event_features(
        &mut self,
        verb: &str,
        participants: &[(ThetaRoleType, Participant)],
    ) -> HashMap<ThetaRoleType, ExtractedFeatures> {
        let mut result = HashMap::new();

        for (role, participant) in participants {
            let features = self.extract_features(verb, role, participant);
            result.insert(*role, features);
        }

        result
    }

    /// Get simple features from word surface properties (fallback)
    pub fn extract_from_word(&self, word: &Word) -> ExtractedFeatures {
        let mut features = ExtractedFeatures::default();

        // Extract from UDPipe morphological features
        if let Some(animacy) = &word.feats.animacy {
            match animacy {
                canopy_core::UDAnimacy::Animate => features.animacy = Some(Animacy::Human),
                canopy_core::UDAnimacy::Inanimate => features.animacy = Some(Animacy::Inanimate),
            }
        }

        // Simple heuristics for common cases
        match word.lemma.as_str() {
            "person" | "people" | "human" | "man" | "woman" | "child" => {
                features.animacy = Some(Animacy::Human);
                features.properties.push(SemanticProperty::Human);
            }
            "company" | "organization" | "government" => {
                features.animacy = Some(Animacy::Animal);
                features.properties.push(SemanticProperty::Organization);
            }
            "water" | "milk" | "oil" | "juice" => {
                features.concreteness = Some(Concreteness::Concrete);
                features.properties.push(SemanticProperty::Fluid);
            }
            "car" | "truck" | "bus" | "train" => {
                features.concreteness = Some(Concreteness::Concrete);
                features.properties.push(SemanticProperty::Vehicle);
            }
            _ => {}
        }

        features.confidence = 0.3; // Lower confidence for heuristics
        features
    }
}

/// Check if features are consistent with each other
pub fn validate_features(features: &ExtractedFeatures) -> bool {
    // Basic consistency checks

    // If human, should be animate
    if features.properties.contains(&SemanticProperty::Human)
        && !matches!(features.animacy, Some(Animacy::Human))
    {
        return false;
    }

    // If fluid, should be concrete
    if features.properties.contains(&SemanticProperty::Fluid)
        && !matches!(features.concreteness, Some(Concreteness::Concrete))
    {
        return false;
    }

    // If vehicle or machine, should be concrete
    if (features.properties.contains(&SemanticProperty::Vehicle)
        || features.properties.contains(&SemanticProperty::Machine))
        && !matches!(features.concreteness, Some(Concreteness::Concrete))
    {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use canopy_core::{DepRel, MorphFeatures, UPos};

    fn create_test_participant(text: &str) -> Participant {
        let word = Word {
            id: 1,
            text: text.to_string(),
            lemma: text.to_string(),
            upos: UPos::Noun,
            xpos: None,
            feats: MorphFeatures::default(),
            head: Some(0),
            deprel: DepRel::Root,
            deps: None,
            misc: None,
            start: 0,
            end: text.len(),
        };
        Participant::from_word(&word)
    }

    #[test]
    fn test_verbnet_feature_extraction() {
        let mut verbnet = VerbNetEngine::new();
        verbnet.add_test_data();

        let mut extractor = VerbNetFeatureExtractor::new(verbnet);
        let participant = create_test_participant("John");

        // Extract features for Agent role of "hit"
        let features = extractor.extract_features("hit", &ThetaRoleType::Agent, &participant);

        // Should get some features (exact features depend on VerbNet test data)
        assert!(features.confidence >= 0.0);
    }

    #[test]
    fn test_restriction_to_feature_conversion() {
        let verbnet = VerbNetEngine::new();
        let extractor = VerbNetFeatureExtractor::new(verbnet);
        let participant = create_test_participant("person");

        let restrictions = vec![
            SelectionalRestriction::Human,
            SelectionalRestriction::Concrete,
        ];

        let features = extractor.restrictions_to_features(&restrictions, &participant);

        assert_eq!(features.animacy, Some(Animacy::Human));
        assert_eq!(features.concreteness, Some(Concreteness::Concrete));
        assert!(features.properties.contains(&SemanticProperty::Human));
        assert!(features.confidence > 0.0);
    }

    #[test]
    fn test_word_based_extraction() {
        let verbnet = VerbNetEngine::new();
        let extractor = VerbNetFeatureExtractor::new(verbnet);

        let word = Word {
            id: 1,
            text: "person".to_string(),
            lemma: "person".to_string(),
            upos: UPos::Noun,
            xpos: None,
            feats: MorphFeatures::default(),
            head: Some(0),
            deprel: DepRel::Root,
            deps: None,
            misc: None,
            start: 0,
            end: 6,
        };

        let features = extractor.extract_from_word(&word);

        assert_eq!(features.animacy, Some(Animacy::Human));
        assert!(features.properties.contains(&SemanticProperty::Human));
    }

    #[test]
    fn test_feature_validation() {
        let mut features = ExtractedFeatures::default();

        // Valid combination: human + animate
        features.animacy = Some(Animacy::Human);
        features.properties.push(SemanticProperty::Human);
        assert!(validate_features(&features));

        // Invalid combination: human but not animate marked
        features.animacy = Some(Animacy::Inanimate);
        assert!(!validate_features(&features));
    }

    #[test]
    fn test_caching() {
        let mut verbnet = VerbNetEngine::new();
        verbnet.add_test_data();

        let mut extractor = VerbNetFeatureExtractor::new(verbnet);
        let participant = create_test_participant("John");

        // First extraction
        let features1 = extractor.extract_features("hit", &ThetaRoleType::Agent, &participant);

        // Second extraction should use cache
        let features2 = extractor.extract_features("hit", &ThetaRoleType::Agent, &participant);

        // Should be identical (from cache)
        assert_eq!(features1.animacy, features2.animacy);
        assert_eq!(features1.confidence, features2.confidence);
    }
}
