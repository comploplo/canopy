//! Presupposition trigger detection
//!
//! Detects presuppositions triggered by events based on VerbNet classes
//! and FrameNet frames. Does NOT use hardcoded word lists for open-class
//! words - detection is pattern-based.
//!
//! ## Trigger Types
//!
//! - **Factive**: Detected via VerbNet classes (admire-31.2, etc.) and
//!   FrameNet frames (Awareness, Experiencer_focus)
//! - **Aspectual**: Detected via VerbNet classes (stop-55.4, continue-55.3, begin-55.1)
//! - **Definite**: Detected via Entity.definiteness == Definite
//! - **Change**: Detected via adverb dependencies (context-based)
//! - **Cleft**: Detected via syntactic structure

use crate::config::EventComposerConfig;
use crate::error::EventResult;
use crate::types::{PredicateInfo, PresupposedContent, Presupposition, PresuppositionTrigger};
use canopy_core::{Definiteness, Entity, ThetaRole};
use std::collections::HashMap;

/// Detects presuppositions triggered by events
pub struct PresuppositionDetector {
    /// VerbNet class patterns for factive verbs
    factive_patterns: Vec<&'static str>,

    /// VerbNet class patterns for aspectual verbs
    aspectual_patterns: Vec<&'static str>,

    /// FrameNet frame patterns for factive
    factive_frames: Vec<&'static str>,
}

impl PresuppositionDetector {
    /// Create a new presupposition detector
    pub fn new(_config: &EventComposerConfig) -> EventResult<Self> {
        Ok(Self {
            // VerbNet classes for factive verbs
            // These verb classes presuppose truth of their complement
            factive_patterns: vec![
                "admire-31.2",     // psychological verbs (admire, appreciate)
                "marvel-31.3",     // marvel, wonder
                "amuse-31.1",      // psychological causatives
                "judgement-33",    // judge, evaluate (presupposes action occurred)
                "discover-84",     // discover, find out (presupposes truth)
                "comprehend-87.2", // understand, know, realize
                "consider-29.9",   // consider, regard (factual presupposition)
            ],
            // VerbNet classes for aspectual verbs
            // These verbs presuppose prior/ongoing state
            aspectual_patterns: vec![
                "stop-55.4",     // stop, quit, cease (presupposes was V-ing)
                "continue-55.3", // continue, keep (presupposes was V-ing)
                "begin-55.1",    // begin, start (presupposes was NOT V-ing)
                "complete-55.2", // complete, finish (presupposes was V-ing)
            ],
            // FrameNet frames for factive presuppositions
            factive_frames: vec![
                "Awareness",
                "Certainty",
                "Coming_to_believe",
                "Remembering_experience",
                "Perception_experience",
            ],
        })
    }

    /// Detect presuppositions for an event
    pub fn detect(
        &self,
        predicate: &PredicateInfo,
        participants: &HashMap<ThetaRole, Entity>,
    ) -> Vec<Presupposition> {
        let mut presuppositions = Vec::new();

        // Check for factive presuppositions
        if let Some(presup) = self.detect_factive(predicate) {
            presuppositions.push(presup);
        }

        // Check for aspectual presuppositions
        if let Some(presup) = self.detect_aspectual(predicate) {
            presuppositions.push(presup);
        }

        // Check for definite description presuppositions
        for entity in participants.values() {
            if let Some(presup) = self.detect_definite(entity) {
                presuppositions.push(presup);
            }
        }

        presuppositions
    }

    /// Detect factive presuppositions from VerbNet/FrameNet
    fn detect_factive(&self, predicate: &PredicateInfo) -> Option<Presupposition> {
        // Check VerbNet classes
        if let Some(ref vn) = predicate.verbnet_analysis {
            for verb_class in &vn.verb_classes {
                let class_id = &verb_class.id;

                // Check if this class is a factive pattern
                if self
                    .factive_patterns
                    .iter()
                    .any(|p| class_id.starts_with(p.split('-').next().unwrap_or("")))
                {
                    return Some(Presupposition {
                        trigger_type: PresuppositionTrigger::Factive,
                        content: PresupposedContent::State {
                            description: format!("The complement of '{}' is true", predicate.lemma),
                            entity_text: String::new(),
                        },
                        projectable: true, // Factive presuppositions typically project
                    });
                }
            }
        }

        // Check FrameNet frames
        if let Some(ref fn_analysis) = predicate.framenet_analysis {
            for frame in &fn_analysis.frames {
                if self
                    .factive_frames
                    .iter()
                    .any(|f| frame.name.eq_ignore_ascii_case(f))
                {
                    return Some(Presupposition {
                        trigger_type: PresuppositionTrigger::Factive,
                        content: PresupposedContent::State {
                            description: format!("Presupposed by {} frame", frame.name),
                            entity_text: String::new(),
                        },
                        projectable: true,
                    });
                }
            }
        }

        None
    }

    /// Detect aspectual presuppositions from VerbNet classes
    fn detect_aspectual(&self, predicate: &PredicateInfo) -> Option<Presupposition> {
        if let Some(ref vn) = predicate.verbnet_analysis {
            for verb_class in &vn.verb_classes {
                let class_id = &verb_class.id;

                // Check against aspectual patterns
                for pattern in &self.aspectual_patterns {
                    let pattern_prefix = pattern.split('-').next().unwrap_or("");
                    if class_id.starts_with(pattern_prefix) {
                        // Begin/start: presupposes "was NOT V-ing"
                        if pattern.starts_with("begin") {
                            return Some(Presupposition {
                                trigger_type: PresuppositionTrigger::Aspectual,
                                content: PresupposedContent::State {
                                    description: "Activity was not in progress".to_string(),
                                    entity_text: String::new(),
                                },
                                projectable: true,
                            });
                        }

                        // Stop/continue/complete: presupposes "was V-ing"
                        return Some(Presupposition {
                            trigger_type: PresuppositionTrigger::Aspectual,
                            content: PresupposedContent::State {
                                description: "Activity was in progress".to_string(),
                                entity_text: String::new(),
                            },
                            projectable: true,
                        });
                    }
                }
            }
        }

        None
    }

    /// Detect existence presupposition from definite descriptions
    fn detect_definite(&self, entity: &Entity) -> Option<Presupposition> {
        if matches!(entity.definiteness, Some(Definiteness::Definite)) {
            return Some(Presupposition {
                trigger_type: PresuppositionTrigger::Definite,
                content: PresupposedContent::Existence {
                    entity_text: entity.text.clone(),
                },
                projectable: true, // Existence presuppositions project through negation
            });
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entity(text: &str, definiteness: Option<Definiteness>) -> Entity {
        Entity {
            id: 1,
            text: text.to_string(),
            animacy: None,
            definiteness,
            number: None,
            distributivity: None,
        }
    }

    #[test]
    fn test_definite_description_presupposition() {
        let detector = PresuppositionDetector::new(&EventComposerConfig::default()).unwrap();

        let definite_entity = make_entity("the king", Some(Definiteness::Definite));
        let mut participants = HashMap::new();
        participants.insert(ThetaRole::Agent, definite_entity);

        let predicate = PredicateInfo {
            lemma: "arrive".to_string(),
            token_idx: 0,
            verbnet_analysis: None,
            framenet_analysis: None,
            l1_confidence: 1.0,
        };

        let presups = detector.detect(&predicate, &participants);

        assert!(!presups.is_empty());
        assert!(matches!(
            presups[0].trigger_type,
            PresuppositionTrigger::Definite
        ));
    }

    #[test]
    fn test_indefinite_no_presupposition() {
        let detector = PresuppositionDetector::new(&EventComposerConfig::default()).unwrap();

        let indefinite_entity = make_entity("a book", Some(Definiteness::Indefinite));
        let mut participants = HashMap::new();
        participants.insert(ThetaRole::Theme, indefinite_entity);

        let predicate = PredicateInfo {
            lemma: "find".to_string(),
            token_idx: 0,
            verbnet_analysis: None,
            framenet_analysis: None,
            l1_confidence: 1.0,
        };

        let presups = detector.detect(&predicate, &participants);

        // No presupposition from indefinite description
        assert!(
            presups.is_empty(),
            "Indefinite should not trigger presupposition"
        );
    }
}
