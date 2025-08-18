//! Feature extraction using VerbNet selectional restrictions
//!
//! This module leverages VerbNet's selectional restrictions to extract semantic
//! features for words. VerbNet restrictions provide rich information about:
//! - Animacy (animate, human, organization)
//! - Concreteness (concrete, abstract, solid, fluid)
//! - Semantic categories (location, time, communication, etc.)

use crate::verbnet::types::*;
use crate::verbnet::engine::VerbNetEngine;
use canopy_core::{Word, UPos};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, trace};

/// Semantic features extracted from VerbNet restrictions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerbNetFeatures {
    /// Animacy classification
    pub animacy: Option<Animacy>,
    
    /// Concreteness classification  
    pub concreteness: Option<Concreteness>,
    
    /// Temporal classification
    pub temporality: Option<Temporality>,
    
    /// Communication classification
    pub communication: Option<Communication>,
    
    /// Physical properties
    pub physical_properties: Vec<PhysicalProperty>,
    
    /// Spatial properties
    pub spatial_properties: Vec<SpatialProperty>,
    
    /// Confidence scores for each feature
    pub confidence: FeatureConfidence,
}

/// Animacy levels derived from VerbNet restrictions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Animacy {
    /// Human beings (from Human restriction)
    Human,
    
    /// Living beings including humans (from Animate restriction)
    Animate,
    
    /// Organizations as animate entities (from Organization restriction)
    OrganizationAnimate,
    
    /// Abstract entities that can act as agents (from AnimateAbstract)
    AbstractAnimate,
    
    /// Inanimate entities
    Inanimate,
}

/// Concreteness levels from VerbNet restrictions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Concreteness {
    /// Physical, tangible objects (from Concrete restriction)
    Concrete,
    
    /// Solid matter (from Solid restriction)
    Solid,
    
    /// Liquid matter (from Fluid restriction)
    Fluid,
    
    /// Chemical substances (from Substance restriction)
    Substance,
    
    /// Non-physical concepts (from Abstract restriction)
    Abstract,
    
    /// Ideas and mental constructs (from Idea restriction)
    Idea,
}

/// Temporal properties from VerbNet
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Temporality {
    /// Time concepts (from Time restriction)
    Temporal,
    
    /// Duration concepts (mapped from Duration theta role)
    Durative,
    
    /// Non-temporal
    NonTemporal,
}

/// Communication properties
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Communication {
    /// Communication events/entities (from Communication restriction)
    Communicative,
    
    /// Sound-related (from Sound restriction)
    Sound,
    
    /// Non-communicative
    NonCommunicative,
}

/// Physical properties from VerbNet restrictions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PhysicalProperty {
    /// Long, thin objects (from Elongated restriction)
    Elongated,
    
    /// Sharp, pointed objects (from Pointy restriction)
    Pointy,
    
    /// Inflexible objects (from Rigid restriction)
    Rigid,
    
    /// Flexible objects (from NonRigid restriction)
    Flexible,
    
    /// Clothing items (from Garment restriction)
    Garment,
    
    /// Body parts (from BodyPart restriction)
    BodyPart,
    
    /// Consumable items (from Comestible restriction)
    Comestible,
}

/// Spatial properties
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpatialProperty {
    /// Spatial locations (from Location restriction)
    Location,
    
    /// Geographic regions (from Region restriction)
    Region,
    
    /// Specific places (from Place restriction)
    Place,
    
    /// Routes or paths (from Path restriction)
    Path,
    
    /// Containers (from Container restriction)
    Container,
}

/// Confidence scores for extracted features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureConfidence {
    /// How confident we are in animacy classification (0.0-1.0)
    pub animacy: f32,
    
    /// How confident we are in concreteness classification (0.0-1.0)
    pub concreteness: f32,
    
    /// How confident we are in temporal classification (0.0-1.0)
    pub temporality: f32,
    
    /// How confident we are in communication classification (0.0-1.0)
    pub communication: f32,
}

impl Default for FeatureConfidence {
    fn default() -> Self {
        Self {
            animacy: 0.0,
            concreteness: 0.0,
            temporality: 0.0,
            communication: 0.0,
        }
    }
}

/// VerbNet-based feature extractor
#[derive(Debug)]
pub struct VerbNetFeatureExtractor {
    engine: VerbNetEngine,
    
    /// Cache for noun feature extraction
    noun_features_cache: HashMap<String, VerbNetFeatures>,
}

impl VerbNetFeatureExtractor {
    /// Create a new feature extractor with a VerbNet engine
    pub fn new(engine: VerbNetEngine) -> Self {
        Self {
            engine,
            noun_features_cache: HashMap::new(),
        }
    }

    /// Extract semantic features for a word using VerbNet restrictions
    pub fn extract_features(&mut self, word: &Word) -> VerbNetFeatures {
        match word.upos {
            UPos::Verb => self.extract_verb_features(word),
            UPos::Noun => self.extract_noun_features(word),
            UPos::Adj => self.extract_adjective_features(word),
            _ => VerbNetFeatures::default(),
        }
    }

    /// Extract features for verbs using their selectional restrictions
    fn extract_verb_features(&self, word: &Word) -> VerbNetFeatures {
        let mut features = VerbNetFeatures::default();
        
        // Get all theta roles for this verb
        let theta_roles = self.engine.get_theta_roles(&word.lemma);
        
        if theta_roles.is_empty() {
            debug!("No VerbNet data found for verb: {}", word.lemma);
            return features;
        }
        
        trace!("Extracting features for verb '{}' with {} theta roles", 
               word.lemma, theta_roles.len());

        // Analyze selectional restrictions across all theta roles
        let mut all_restrictions = Vec::new();
        for role in &theta_roles {
            all_restrictions.extend(&role.selectional_restrictions);
        }

        // Extract animacy from Agent/Experiencer restrictions
        features.animacy = self.infer_animacy_from_restrictions(&all_restrictions);
        features.concreteness = self.infer_concreteness_from_restrictions(&all_restrictions);
        features.temporality = self.infer_temporality_from_restrictions(&all_restrictions);
        features.communication = self.infer_communication_from_restrictions(&all_restrictions);
        features.physical_properties = self.infer_physical_properties(&all_restrictions);
        features.spatial_properties = self.infer_spatial_properties(&all_restrictions);

        // Set confidence based on data quality
        features.confidence = self.calculate_confidence(&theta_roles, &all_restrictions);
        
        features
    }

    /// Extract features for nouns by looking at how they're used as arguments
    fn extract_noun_features(&mut self, word: &Word) -> VerbNetFeatures {
        // Check cache first
        if let Some(cached) = self.noun_features_cache.get(&word.lemma) {
            return cached.clone();
        }

        let mut features = VerbNetFeatures::default();
        
        // For nouns, we look at what restrictions apply when they appear as arguments
        // This is more complex and might require corpus analysis or semantic similarity
        
        // For now, use simple heuristics based on word properties
        features.animacy = self.heuristic_animacy(&word.lemma);
        features.concreteness = self.heuristic_concreteness(&word.lemma);
        
        // Cache the result
        self.noun_features_cache.insert(word.lemma.clone(), features.clone());
        
        features
    }

    /// Extract features for adjectives (limited)
    fn extract_adjective_features(&self, _word: &Word) -> VerbNetFeatures {
        // Adjectives have limited VerbNet coverage
        // Most feature extraction would need other resources
        VerbNetFeatures::default()
    }

    /// Infer animacy from selectional restrictions
    fn infer_animacy_from_restrictions(&self, restrictions: &[SelectionalRestriction]) -> Option<Animacy> {
        for restriction in restrictions {
            match restriction {
                SelectionalRestriction::Human => return Some(Animacy::Human),
                SelectionalRestriction::Animate => return Some(Animacy::Animate),
                SelectionalRestriction::Organization => return Some(Animacy::OrganizationAnimate),
                SelectionalRestriction::AnimateAbstract => return Some(Animacy::AbstractAnimate),
                _ => {}
            }
        }
        
        // If we have concrete restrictions, likely inanimate
        if restrictions.iter().any(|r| r.implies_concreteness()) {
            Some(Animacy::Inanimate)
        } else {
            None
        }
    }

    /// Infer concreteness from selectional restrictions
    fn infer_concreteness_from_restrictions(&self, restrictions: &[SelectionalRestriction]) -> Option<Concreteness> {
        for restriction in restrictions {
            match restriction {
                SelectionalRestriction::Concrete => return Some(Concreteness::Concrete),
                SelectionalRestriction::Solid => return Some(Concreteness::Solid),
                SelectionalRestriction::Fluid => return Some(Concreteness::Fluid),
                SelectionalRestriction::Substance => return Some(Concreteness::Substance),
                SelectionalRestriction::Abstract => return Some(Concreteness::Abstract),
                SelectionalRestriction::Idea => return Some(Concreteness::Idea),
                _ => {}
            }
        }
        None
    }

    /// Infer temporal properties from restrictions
    fn infer_temporality_from_restrictions(&self, restrictions: &[SelectionalRestriction]) -> Option<Temporality> {
        if restrictions.contains(&SelectionalRestriction::Time) {
            Some(Temporality::Temporal)
        } else {
            Some(Temporality::NonTemporal)
        }
    }

    /// Infer communication properties from restrictions
    fn infer_communication_from_restrictions(&self, restrictions: &[SelectionalRestriction]) -> Option<Communication> {
        for restriction in restrictions {
            match restriction {
                SelectionalRestriction::Communication => return Some(Communication::Communicative),
                SelectionalRestriction::Sound => return Some(Communication::Sound),
                _ => {}
            }
        }
        Some(Communication::NonCommunicative)
    }

    /// Extract physical properties from restrictions
    fn infer_physical_properties(&self, restrictions: &[SelectionalRestriction]) -> Vec<PhysicalProperty> {
        let mut properties = Vec::new();
        
        for restriction in restrictions {
            match restriction {
                SelectionalRestriction::Elongated => properties.push(PhysicalProperty::Elongated),
                SelectionalRestriction::Pointy => properties.push(PhysicalProperty::Pointy),
                SelectionalRestriction::Rigid => properties.push(PhysicalProperty::Rigid),
                SelectionalRestriction::NonRigid => properties.push(PhysicalProperty::Flexible),
                SelectionalRestriction::Garment => properties.push(PhysicalProperty::Garment),
                SelectionalRestriction::BodyPart => properties.push(PhysicalProperty::BodyPart),
                SelectionalRestriction::Comestible => properties.push(PhysicalProperty::Comestible),
                _ => {}
            }
        }
        
        properties
    }

    /// Extract spatial properties from restrictions
    fn infer_spatial_properties(&self, restrictions: &[SelectionalRestriction]) -> Vec<SpatialProperty> {
        let mut properties = Vec::new();
        
        for restriction in restrictions {
            match restriction {
                SelectionalRestriction::Location => properties.push(SpatialProperty::Location),
                SelectionalRestriction::Region => properties.push(SpatialProperty::Region),
                SelectionalRestriction::Place => properties.push(SpatialProperty::Place),
                SelectionalRestriction::Path => properties.push(SpatialProperty::Path),
                SelectionalRestriction::Container => properties.push(SpatialProperty::Container),
                _ => {}
            }
        }
        
        properties
    }

    /// Calculate confidence scores based on data quality
    fn calculate_confidence(&self, theta_roles: &[ThetaRole], restrictions: &[SelectionalRestriction]) -> FeatureConfidence {
        let base_confidence = if theta_roles.is_empty() { 0.0 } else { 0.7 };
        let restriction_bonus = (restrictions.len() as f32 * 0.1).min(0.3);
        
        FeatureConfidence {
            animacy: base_confidence + restriction_bonus,
            concreteness: base_confidence + restriction_bonus,
            temporality: base_confidence,
            communication: base_confidence,
        }
    }

    /// Simple heuristic animacy for nouns (fallback)
    fn heuristic_animacy(&self, lemma: &str) -> Option<Animacy> {
        // Simple word list heuristics
        match lemma {
            "person" | "people" | "human" | "man" | "woman" | "child" => Some(Animacy::Human),
            "animal" | "dog" | "cat" | "bird" => Some(Animacy::Animate),
            "company" | "organization" | "government" => Some(Animacy::OrganizationAnimate),
            _ => Some(Animacy::Inanimate),
        }
    }

    /// Simple heuristic concreteness for nouns (fallback)
    fn heuristic_concreteness(&self, lemma: &str) -> Option<Concreteness> {
        match lemma {
            "idea" | "concept" | "thought" => Some(Concreteness::Idea),
            "love" | "happiness" | "freedom" => Some(Concreteness::Abstract),
            "water" | "milk" | "oil" => Some(Concreteness::Fluid),
            _ => Some(Concreteness::Concrete),
        }
    }
}

impl Default for VerbNetFeatures {
    fn default() -> Self {
        Self {
            animacy: None,
            concreteness: None,
            temporality: None,
            communication: None,
            physical_properties: Vec::new(),
            spatial_properties: Vec::new(),
            confidence: FeatureConfidence::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::verbnet::engine::VerbNetEngine;

    #[test]
    fn test_restriction_to_animacy() {
        let mut engine = VerbNetEngine::new();
        engine.add_test_data();
        let extractor = VerbNetFeatureExtractor::new(engine);
        
        let restrictions = vec![SelectionalRestriction::Human];
        let animacy = extractor.infer_animacy_from_restrictions(&restrictions);
        assert_eq!(animacy, Some(Animacy::Human));
        
        let restrictions = vec![SelectionalRestriction::Animate];
        let animacy = extractor.infer_animacy_from_restrictions(&restrictions);
        assert_eq!(animacy, Some(Animacy::Animate));
    }

    #[test]
    fn test_restriction_to_concreteness() {
        let mut engine = VerbNetEngine::new();
        engine.add_test_data();
        let extractor = VerbNetFeatureExtractor::new(engine);
        
        let restrictions = vec![SelectionalRestriction::Concrete];
        let concreteness = extractor.infer_concreteness_from_restrictions(&restrictions);
        assert_eq!(concreteness, Some(Concreteness::Concrete));
        
        let restrictions = vec![SelectionalRestriction::Abstract];
        let concreteness = extractor.infer_concreteness_from_restrictions(&restrictions);
        assert_eq!(concreteness, Some(Concreteness::Abstract));
    }

    #[test]
    fn test_physical_properties() {
        let mut engine = VerbNetEngine::new();
        engine.add_test_data();
        let extractor = VerbNetFeatureExtractor::new(engine);
        
        let restrictions = vec![
            SelectionalRestriction::Elongated,
            SelectionalRestriction::Rigid,
        ];
        let properties = extractor.infer_physical_properties(&restrictions);
        assert!(properties.contains(&PhysicalProperty::Elongated));
        assert!(properties.contains(&PhysicalProperty::Rigid));
    }
}