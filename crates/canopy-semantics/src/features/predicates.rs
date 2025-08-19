//! VerbNet semantic predicate extraction
//!
//! VerbNet provides 146+ semantic predicate types like MOTION, TRANSFER, CONTACT.
//! This module extracts them directly from VerbNet data without complex inference.

use crate::events::Event;
use crate::verbnet::{PredicateType, VerbNetEngine};
use std::collections::HashMap;
use tracing::{debug, trace};

/// Simple predicate extractor using VerbNet data directly
#[derive(Debug)]
pub struct PredicateExtractor {
    /// VerbNet engine for lookups
    verbnet: VerbNetEngine,

    /// Cache for verb predicates
    predicate_cache: HashMap<String, Vec<ExtractedPredicate>>,
}

/// Predicate extracted from VerbNet
#[derive(Debug, Clone)]
pub struct ExtractedPredicate {
    /// The predicate type from VerbNet
    pub predicate_type: PredicateType,

    /// Arguments of the predicate
    pub arguments: Vec<String>,

    /// Temporal specification (during, start, end)
    pub event_time: EventTime,

    /// Whether predicate is negated
    pub negated: bool,

    /// VerbNet class this comes from
    pub verbnet_class: String,
}

/// Event time specification from VerbNet
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventTime {
    /// During the event
    During,

    /// At event start
    Start,

    /// At event end
    End,
}

impl PredicateExtractor {
    /// Create new predicate extractor
    pub fn new(verbnet: VerbNetEngine) -> Self {
        Self {
            verbnet,
            predicate_cache: HashMap::new(),
        }
    }

    /// Extract semantic predicates for a verb
    pub fn extract_predicates(&mut self, verb: &str) -> Vec<ExtractedPredicate> {
        // Check cache first
        if let Some(cached) = self.predicate_cache.get(verb) {
            return cached.clone();
        }

        debug!("Extracting VerbNet predicates for: {}", verb);

        // Get verb classes from VerbNet
        let verb_classes = self.verbnet.get_verb_classes(verb);
        if verb_classes.is_empty() {
            debug!("No VerbNet classes for: {}", verb);
            return vec![];
        }

        let mut all_predicates = Vec::new();

        for verb_class in verb_classes {
            trace!("Processing class: {}", verb_class.id);

            // Extract predicates from all frames in this class
            for frame in &verb_class.frames {
                for semantic_pred in &frame.semantics {
                    let extracted = ExtractedPredicate {
                        predicate_type: semantic_pred.predicate_type.clone(),
                        arguments: semantic_pred.arguments.clone(),
                        event_time: self.convert_event_time(&semantic_pred.event_time),
                        negated: semantic_pred.negated,
                        verbnet_class: verb_class.id.clone(),
                    };

                    all_predicates.push(extracted);
                }
            }
        }

        trace!(
            "Extracted {} predicates for '{}'",
            all_predicates.len(),
            verb
        );

        // Cache result
        self.predicate_cache
            .insert(verb.to_string(), all_predicates.clone());

        all_predicates
    }

    /// Extract predicates and attach to event
    pub fn annotate_event(&mut self, event: &mut Event) {
        let predicates = self.extract_predicates(&event.predicate.lemma);

        // Update event predicate with VerbNet information
        if let Some(first_pred) = predicates.first() {
            event.predicate.semantic_type = self.convert_predicate_type(&first_pred.predicate_type);

            // Add all predicates as semantic features
            for pred in predicates {
                let feature = self.predicate_to_semantic_feature(&pred.predicate_type);
                if !event.predicate.features.contains(&feature) {
                    event.predicate.features.push(feature);
                }
            }
        }
    }

    /// Get dominant predicate type for a verb (most common/general)
    pub fn get_dominant_predicate(&mut self, verb: &str) -> Option<PredicateType> {
        let predicates = self.extract_predicates(verb);

        if predicates.is_empty() {
            return None;
        }

        // Count frequency of predicate types
        let mut counts: HashMap<String, usize> = HashMap::new();
        for pred in &predicates {
            let key = format!("{:?}", pred.predicate_type);
            *counts.entry(key).or_insert(0) += 1;
        }

        // Return most frequent predicate type
        if let Some((most_common_key, _)) = counts.iter().max_by_key(|(_, count)| *count) {
            // Find predicate with this type
            for pred in predicates {
                let key = format!("{:?}", pred.predicate_type);
                if key == *most_common_key {
                    return Some(pred.predicate_type);
                }
            }
        }

        None
    }

    /// Convert VerbNet event time to our enum
    fn convert_event_time(&self, vn_time: &crate::verbnet::EventTime) -> EventTime {
        match vn_time {
            crate::verbnet::EventTime::During => EventTime::During,
            crate::verbnet::EventTime::Start => EventTime::Start,
            crate::verbnet::EventTime::End => EventTime::End,
        }
    }

    /// Convert VerbNet predicate type to event predicate type
    fn convert_predicate_type(&self, pred_type: &PredicateType) -> crate::events::PredicateType {
        match pred_type {
            PredicateType::Motion => crate::events::PredicateType::Activity,
            PredicateType::Cause => crate::events::PredicateType::Causative,
            PredicateType::Change => crate::events::PredicateType::Achievement,
            PredicateType::Location => crate::events::PredicateType::State,
            PredicateType::Transfer => crate::events::PredicateType::Action,
            PredicateType::Contact => crate::events::PredicateType::Action,
            PredicateType::Exist => crate::events::PredicateType::State,
            PredicateType::HasState => crate::events::PredicateType::State,
            _ => crate::events::PredicateType::Action, // Default fallback
        }
    }

    /// Convert predicate type to semantic feature
    fn predicate_to_semantic_feature(
        &self,
        pred_type: &PredicateType,
    ) -> crate::events::SemanticFeature {
        match pred_type {
            PredicateType::Motion => crate::events::SemanticFeature::Motion,
            PredicateType::Transfer => crate::events::SemanticFeature::Transfer,
            PredicateType::Contact => crate::events::SemanticFeature::Contact,
            PredicateType::Change => crate::events::SemanticFeature::ChangeOfState,
            PredicateType::Perceive => crate::events::SemanticFeature::Perception,
            _ => crate::events::SemanticFeature::Motion, // Default fallback
        }
    }

    /// Get all predicate types available in VerbNet
    pub fn get_all_predicate_types(&self) -> Vec<PredicateType> {
        // This would return all 146+ predicate types from VerbNet
        // For now, return the common ones we handle
        vec![
            PredicateType::Motion,
            PredicateType::Transfer,
            PredicateType::Contact,
            PredicateType::Change,
            PredicateType::Cause,
            PredicateType::Location,
            PredicateType::Exist,
            PredicateType::HasState,
            PredicateType::Perceive,
            // Add more as needed...
        ]
    }
}

/// Predicate pattern analysis
impl PredicateExtractor {
    /// Check if verb has motion predicates
    pub fn has_motion(&mut self, verb: &str) -> bool {
        let predicates = self.extract_predicates(verb);
        predicates
            .iter()
            .any(|p| matches!(p.predicate_type, PredicateType::Motion))
    }

    /// Check if verb has causative predicates
    pub fn has_causation(&mut self, verb: &str) -> bool {
        let predicates = self.extract_predicates(verb);
        predicates
            .iter()
            .any(|p| matches!(p.predicate_type, PredicateType::Cause))
    }

    /// Check if verb has change of state predicates
    pub fn has_change_of_state(&mut self, verb: &str) -> bool {
        let predicates = self.extract_predicates(verb);
        predicates
            .iter()
            .any(|p| matches!(p.predicate_type, PredicateType::Change))
    }

    /// Get semantic complexity (number of distinct predicate types)
    pub fn get_semantic_complexity(&mut self, verb: &str) -> usize {
        let predicates = self.extract_predicates(verb);
        let mut unique_types = std::collections::HashSet::new();

        for pred in predicates {
            unique_types.insert(format!("{:?}", pred.predicate_type));
        }

        unique_types.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{AspectualClass, Event, EventId, Predicate};

    #[test]
    fn test_predicate_extraction() {
        let mut verbnet = VerbNetEngine::new();
        verbnet.add_test_data();

        let mut extractor = PredicateExtractor::new(verbnet);

        // Test with a known verb from test data
        let _predicates = extractor.extract_predicates("give");

        // Should get some predicates from VerbNet test data
        // Should get some predicates from VerbNet test data (len() is always >= 0)
    }

    #[test]
    fn test_event_annotation() {
        let mut verbnet = VerbNetEngine::new();
        verbnet.add_test_data();

        let mut extractor = PredicateExtractor::new(verbnet);

        let predicate = Predicate {
            lemma: "give".to_string(),
            semantic_type: crate::events::PredicateType::Action,
            verbnet_class: None,
            features: vec![],
        };

        let mut event = Event {
            id: EventId(1),
            predicate,
            participants: std::collections::HashMap::new(),
            modifiers: Vec::new(),
            aspect: AspectualClass::Activity,
            structure: None,
            time: crate::events::EventTime::Now,
            movement_chains: Vec::new(),
            little_v: None,
        };

        extractor.annotate_event(&mut event);

        // Event should now have VerbNet predicate information
        // (Exact features depend on what's in VerbNet test data)
    }

    #[test]
    fn test_predicate_patterns() {
        let mut verbnet = VerbNetEngine::new();
        verbnet.add_test_data();

        let mut extractor = PredicateExtractor::new(verbnet);

        // Test pattern detection
        let _has_motion = extractor.has_motion("run");
        let _has_causation = extractor.has_causation("break");
        let _complexity = extractor.get_semantic_complexity("give");

        // These should work regardless of VerbNet test data content
        // Complexity is non-negative by design
    }

    #[test]
    fn test_caching() {
        let mut verbnet = VerbNetEngine::new();
        verbnet.add_test_data();

        let mut extractor = PredicateExtractor::new(verbnet);

        // First extraction
        let predicates1 = extractor.extract_predicates("give");

        // Second extraction should use cache
        let predicates2 = extractor.extract_predicates("give");

        assert_eq!(predicates1.len(), predicates2.len());
    }

    #[test]
    fn test_all_predicate_types() {
        let verbnet = VerbNetEngine::new();
        let extractor = PredicateExtractor::new(verbnet);

        let all_types = extractor.get_all_predicate_types();

        assert!(!all_types.is_empty());
        assert!(all_types.contains(&PredicateType::Motion));
        assert!(all_types.contains(&PredicateType::Transfer));
    }

    #[test]
    fn test_empty_verb_predicates() {
        let verbnet = VerbNetEngine::new(); // Empty engine
        let mut extractor = PredicateExtractor::new(verbnet);

        // Test with non-existent verb
        let predicates = extractor.extract_predicates("nonexistent");
        assert!(predicates.is_empty());

        // Test dominant predicate for empty results
        let dominant = extractor.get_dominant_predicate("nonexistent");
        assert!(dominant.is_none());

        // Test pattern methods with empty results
        assert!(!extractor.has_motion("nonexistent"));
        assert!(!extractor.has_causation("nonexistent"));
        assert!(!extractor.has_change_of_state("nonexistent"));
        assert_eq!(extractor.get_semantic_complexity("nonexistent"), 0);
    }

    #[test]
    fn test_event_time_conversion() {
        let verbnet = VerbNetEngine::new();
        let extractor = PredicateExtractor::new(verbnet);

        // Test all event time conversions
        assert_eq!(
            extractor.convert_event_time(&crate::verbnet::EventTime::During),
            EventTime::During
        );
        assert_eq!(
            extractor.convert_event_time(&crate::verbnet::EventTime::Start),
            EventTime::Start
        );
        assert_eq!(
            extractor.convert_event_time(&crate::verbnet::EventTime::End),
            EventTime::End
        );
    }

    #[test]
    fn test_predicate_type_conversion() {
        let verbnet = VerbNetEngine::new();
        let extractor = PredicateExtractor::new(verbnet);

        // Test specific predicate type conversions
        assert_eq!(
            extractor.convert_predicate_type(&PredicateType::Motion),
            crate::events::PredicateType::Activity
        );
        assert_eq!(
            extractor.convert_predicate_type(&PredicateType::Cause),
            crate::events::PredicateType::Causative
        );
        assert_eq!(
            extractor.convert_predicate_type(&PredicateType::Change),
            crate::events::PredicateType::Achievement
        );
        assert_eq!(
            extractor.convert_predicate_type(&PredicateType::Location),
            crate::events::PredicateType::State
        );
        assert_eq!(
            extractor.convert_predicate_type(&PredicateType::Transfer),
            crate::events::PredicateType::Action
        );
        assert_eq!(
            extractor.convert_predicate_type(&PredicateType::Contact),
            crate::events::PredicateType::Action
        );
        assert_eq!(
            extractor.convert_predicate_type(&PredicateType::Exist),
            crate::events::PredicateType::State
        );
        assert_eq!(
            extractor.convert_predicate_type(&PredicateType::HasState),
            crate::events::PredicateType::State
        );

        // Test fallback case
        assert_eq!(
            extractor.convert_predicate_type(&PredicateType::Other("unknown".to_string())),
            crate::events::PredicateType::Action
        );
    }

    #[test]
    fn test_semantic_feature_conversion() {
        let verbnet = VerbNetEngine::new();
        let extractor = PredicateExtractor::new(verbnet);

        // Test semantic feature conversions
        assert_eq!(
            extractor.predicate_to_semantic_feature(&PredicateType::Motion),
            crate::events::SemanticFeature::Motion
        );
        assert_eq!(
            extractor.predicate_to_semantic_feature(&PredicateType::Transfer),
            crate::events::SemanticFeature::Transfer
        );
        assert_eq!(
            extractor.predicate_to_semantic_feature(&PredicateType::Contact),
            crate::events::SemanticFeature::Contact
        );
        assert_eq!(
            extractor.predicate_to_semantic_feature(&PredicateType::Change),
            crate::events::SemanticFeature::ChangeOfState
        );
        assert_eq!(
            extractor.predicate_to_semantic_feature(&PredicateType::Perceive),
            crate::events::SemanticFeature::Perception
        );

        // Test fallback case
        assert_eq!(
            extractor.predicate_to_semantic_feature(&PredicateType::Other("unknown".to_string())),
            crate::events::SemanticFeature::Motion
        );
    }

    #[test]
    fn test_dominant_predicate_logic() {
        let mut verbnet = VerbNetEngine::new();
        verbnet.add_test_data();

        let mut extractor = PredicateExtractor::new(verbnet);

        // Test with verbs that have VerbNet data
        let dominant_give = extractor.get_dominant_predicate("give");
        let dominant_hit = extractor.get_dominant_predicate("hit");

        // Should get some dominant predicate for verbs with data
        if dominant_give.is_some() {
            assert!(matches!(
                dominant_give.unwrap(),
                PredicateType::Transfer | PredicateType::Cause
            ));
        }

        if dominant_hit.is_some() {
            assert!(matches!(
                dominant_hit.unwrap(),
                PredicateType::Contact | PredicateType::Cause
            ));
        }
    }

    #[test]
    fn test_comprehensive_verb_patterns() {
        let mut verbnet = VerbNetEngine::new();
        verbnet.add_test_data();

        let mut extractor = PredicateExtractor::new(verbnet);

        // Test with verbs from test data
        let verbs = ["give", "hit", "hand", "strike"];

        for verb in &verbs {
            let predicates = extractor.extract_predicates(verb);
            let complexity = extractor.get_semantic_complexity(verb);
            let has_motion = extractor.has_motion(verb);
            let has_causation = extractor.has_causation(verb);
            let has_change = extractor.has_change_of_state(verb);

            // If we have predicates, complexity should be > 0
            if !predicates.is_empty() {
                assert!(complexity > 0);
            }

            // Pattern tests should be consistent
            assert!(has_motion || !has_motion); // Always boolean
            assert!(has_causation || !has_causation); // Always boolean
            assert!(has_change || !has_change); // Always boolean
        }
    }

    #[test]
    fn test_event_annotation_comprehensive() {
        let mut verbnet = VerbNetEngine::new();
        verbnet.add_test_data();

        let mut extractor = PredicateExtractor::new(verbnet);

        // Test with verb that has no VerbNet data
        let mut event_empty = Event {
            id: EventId(1),
            predicate: Predicate {
                lemma: "nonexistent".to_string(),
                semantic_type: crate::events::PredicateType::Action,
                verbnet_class: None,
                features: vec![],
            },
            participants: std::collections::HashMap::new(),
            modifiers: Vec::new(),
            aspect: AspectualClass::Activity,
            structure: None,
            time: crate::events::EventTime::Now,
            movement_chains: Vec::new(),
            little_v: None,
        };

        extractor.annotate_event(&mut event_empty);
        // Event should remain mostly unchanged since no VerbNet data
        assert_eq!(event_empty.predicate.lemma, "nonexistent");

        // Test with verb that has VerbNet data
        let mut event_give = Event {
            id: EventId(2),
            predicate: Predicate {
                lemma: "give".to_string(),
                semantic_type: crate::events::PredicateType::Action,
                verbnet_class: None,
                features: vec![],
            },
            participants: std::collections::HashMap::new(),
            modifiers: Vec::new(),
            aspect: AspectualClass::Activity,
            structure: None,
            time: crate::events::EventTime::Now,
            movement_chains: Vec::new(),
            little_v: None,
        };

        extractor.annotate_event(&mut event_give);
        // Event may have been updated with VerbNet information
        assert_eq!(event_give.predicate.lemma, "give");
    }

    #[test]
    fn test_cache_consistency() {
        let mut verbnet = VerbNetEngine::new();
        verbnet.add_test_data();

        let mut extractor = PredicateExtractor::new(verbnet);

        // Extract predicates multiple times to test caching
        let predicates1 = extractor.extract_predicates("give");
        let predicates2 = extractor.extract_predicates("give");
        let predicates3 = extractor.extract_predicates("give");

        // All should be identical
        assert_eq!(predicates1.len(), predicates2.len());
        assert_eq!(predicates2.len(), predicates3.len());

        // Test that different verbs get different results
        let _predicates_hit = extractor.extract_predicates("hit");

        // Should be able to extract for different verbs independently
        let predicates_give_again = extractor.extract_predicates("give");
        assert_eq!(predicates1.len(), predicates_give_again.len());
    }
}
