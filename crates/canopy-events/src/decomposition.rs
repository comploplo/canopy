//! VerbNet to LittleV decomposition
//!
//! Maps VerbNet semantic predicates and classes to LittleV event primitives.

use crate::config::EventComposerConfig;
use crate::error::EventResult;
use crate::types::{DecomposedEvent, LittleVType, PredicateInfo};
use canopy_core::ThetaRole;
use std::collections::HashMap;

/// Maps VerbNet semantic predicates to LittleV primitives
pub struct EventDecomposer {
    /// VerbNet predicate name -> LittleV template mapping
    predicate_map: HashMap<String, LittleVTemplate>,

    /// VerbNet class ID prefix -> default LittleV type
    class_defaults: HashMap<String, LittleVType>,

    /// Configuration
    #[allow(dead_code)]
    config: EventComposerConfig,
}

/// Template for decomposing a predicate
#[derive(Debug, Clone)]
pub struct LittleVTemplate {
    /// Primary LittleV type
    pub primary_type: LittleVType,

    /// Expected theta roles for this template
    pub expected_roles: Vec<ThetaRole>,

    /// Optional sub-event template
    pub sub_event: Option<Box<LittleVTemplate>>,

    /// Base confidence for this template
    pub base_confidence: f32,
}

impl EventDecomposer {
    /// Create a new decomposer with default mappings
    pub fn new(config: &EventComposerConfig) -> EventResult<Self> {
        let mut decomposer = Self {
            predicate_map: HashMap::new(),
            class_defaults: HashMap::new(),
            config: config.clone(),
        };
        decomposer.initialize_predicate_mappings();
        decomposer.initialize_class_defaults();
        Ok(decomposer)
    }

    /// Initialize VerbNet predicate -> LittleV mappings
    fn initialize_predicate_mappings(&mut self) {
        // === CAUSATION PREDICATES ===
        self.predicate_map.insert(
            "cause".to_string(),
            LittleVTemplate {
                primary_type: LittleVType::Cause,
                expected_roles: vec![ThetaRole::Agent, ThetaRole::Patient],
                sub_event: Some(Box::new(LittleVTemplate {
                    primary_type: LittleVType::Become,
                    expected_roles: vec![ThetaRole::Theme],
                    sub_event: None,
                    base_confidence: 0.9,
                })),
                base_confidence: 0.95,
            },
        );

        // === MOTION PREDICATES ===
        self.predicate_map.insert(
            "motion".to_string(),
            LittleVTemplate {
                primary_type: LittleVType::Go,
                expected_roles: vec![ThetaRole::Theme, ThetaRole::Source, ThetaRole::Goal],
                sub_event: None,
                base_confidence: 0.9,
            },
        );

        self.predicate_map.insert(
            "path_rel".to_string(),
            LittleVTemplate {
                primary_type: LittleVType::Go,
                expected_roles: vec![ThetaRole::Theme, ThetaRole::Goal],
                sub_event: None,
                base_confidence: 0.85,
            },
        );

        // === TRANSFER PREDICATES ===
        self.predicate_map.insert(
            "transfer".to_string(),
            LittleVTemplate {
                primary_type: LittleVType::Cause,
                expected_roles: vec![ThetaRole::Agent, ThetaRole::Theme, ThetaRole::Recipient],
                sub_event: Some(Box::new(LittleVTemplate {
                    primary_type: LittleVType::Have,
                    expected_roles: vec![ThetaRole::Recipient, ThetaRole::Theme],
                    sub_event: None,
                    base_confidence: 0.9,
                })),
                base_confidence: 0.95,
            },
        );

        self.predicate_map.insert(
            "has_possession".to_string(),
            LittleVTemplate {
                primary_type: LittleVType::Have,
                expected_roles: vec![ThetaRole::Agent, ThetaRole::Theme],
                sub_event: None,
                base_confidence: 0.9,
            },
        );

        // === STATE PREDICATES ===
        self.predicate_map.insert(
            "state".to_string(),
            LittleVTemplate {
                primary_type: LittleVType::Be,
                expected_roles: vec![ThetaRole::Theme],
                sub_event: None,
                base_confidence: 0.85,
            },
        );

        self.predicate_map.insert(
            "property".to_string(),
            LittleVTemplate {
                primary_type: LittleVType::Be,
                expected_roles: vec![ThetaRole::Theme],
                sub_event: None,
                base_confidence: 0.85,
            },
        );

        // === CHANGE OF STATE ===
        self.predicate_map.insert(
            "become".to_string(),
            LittleVTemplate {
                primary_type: LittleVType::Become,
                expected_roles: vec![ThetaRole::Theme],
                sub_event: None,
                base_confidence: 0.9,
            },
        );

        self.predicate_map.insert(
            "degradation_material_integrity".to_string(),
            LittleVTemplate {
                primary_type: LittleVType::Become,
                expected_roles: vec![ThetaRole::Patient],
                sub_event: None,
                base_confidence: 0.85,
            },
        );

        // === ACTIVITY PREDICATES ===
        self.predicate_map.insert(
            "do".to_string(),
            LittleVTemplate {
                primary_type: LittleVType::Do,
                expected_roles: vec![ThetaRole::Agent],
                sub_event: None,
                base_confidence: 0.85,
            },
        );

        self.predicate_map.insert(
            "manner".to_string(),
            LittleVTemplate {
                primary_type: LittleVType::Do,
                expected_roles: vec![ThetaRole::Agent],
                sub_event: None,
                base_confidence: 0.8,
            },
        );

        // === PSYCHOLOGICAL PREDICATES ===
        self.predicate_map.insert(
            "experience".to_string(),
            LittleVTemplate {
                primary_type: LittleVType::Experience,
                expected_roles: vec![ThetaRole::Experiencer, ThetaRole::Stimulus],
                sub_event: None,
                base_confidence: 0.9,
            },
        );

        self.predicate_map.insert(
            "emotional_state".to_string(),
            LittleVTemplate {
                primary_type: LittleVType::Experience,
                expected_roles: vec![ThetaRole::Experiencer, ThetaRole::Stimulus],
                sub_event: None,
                base_confidence: 0.85,
            },
        );

        self.predicate_map.insert(
            "perceive".to_string(),
            LittleVTemplate {
                primary_type: LittleVType::Experience,
                expected_roles: vec![ThetaRole::Experiencer, ThetaRole::Stimulus],
                sub_event: None,
                base_confidence: 0.85,
            },
        );

        // === COMMUNICATION PREDICATES ===
        self.predicate_map.insert(
            "say".to_string(),
            LittleVTemplate {
                primary_type: LittleVType::Say,
                expected_roles: vec![ThetaRole::Agent, ThetaRole::Recipient],
                sub_event: None,
                base_confidence: 0.9,
            },
        );

        self.predicate_map.insert(
            "communicate".to_string(),
            LittleVTemplate {
                primary_type: LittleVType::Say,
                expected_roles: vec![ThetaRole::Agent, ThetaRole::Recipient],
                sub_event: None,
                base_confidence: 0.85,
            },
        );

        self.predicate_map.insert(
            "transfer_info".to_string(),
            LittleVTemplate {
                primary_type: LittleVType::Say,
                expected_roles: vec![ThetaRole::Agent, ThetaRole::Recipient, ThetaRole::Theme],
                sub_event: None,
                base_confidence: 0.85,
            },
        );

        // === EXISTENCE PREDICATES ===
        self.predicate_map.insert(
            "exist".to_string(),
            LittleVTemplate {
                primary_type: LittleVType::Exist,
                expected_roles: vec![ThetaRole::Theme, ThetaRole::Location],
                sub_event: None,
                base_confidence: 0.9,
            },
        );

        self.predicate_map.insert(
            "location".to_string(),
            LittleVTemplate {
                primary_type: LittleVType::Exist,
                expected_roles: vec![ThetaRole::Theme, ThetaRole::Location],
                sub_event: None,
                base_confidence: 0.85,
            },
        );

        // === CREATION/DESTRUCTION ===
        self.predicate_map.insert(
            "created".to_string(),
            LittleVTemplate {
                primary_type: LittleVType::Cause,
                expected_roles: vec![ThetaRole::Agent, ThetaRole::Theme],
                sub_event: Some(Box::new(LittleVTemplate {
                    primary_type: LittleVType::Exist,
                    expected_roles: vec![ThetaRole::Theme],
                    sub_event: None,
                    base_confidence: 0.9,
                })),
                base_confidence: 0.9,
            },
        );

        self.predicate_map.insert(
            "destroyed".to_string(),
            LittleVTemplate {
                primary_type: LittleVType::Cause,
                expected_roles: vec![ThetaRole::Agent, ThetaRole::Patient],
                sub_event: Some(Box::new(LittleVTemplate {
                    primary_type: LittleVType::Become,
                    expected_roles: vec![ThetaRole::Patient],
                    sub_event: None,
                    base_confidence: 0.9,
                })),
                base_confidence: 0.9,
            },
        );
    }

    /// Initialize VerbNet class -> default LittleV type mappings
    fn initialize_class_defaults(&mut self) {
        // Motion verbs
        self.class_defaults
            .insert("run-51.3".to_string(), LittleVType::Do);
        self.class_defaults
            .insert("slide-11.2".to_string(), LittleVType::Go);
        self.class_defaults
            .insert("roll-51.3.1".to_string(), LittleVType::Go);
        self.class_defaults
            .insert("escape-51.1".to_string(), LittleVType::Go);
        self.class_defaults
            .insert("arrive-48.1.1".to_string(), LittleVType::Go);

        // Transfer verbs
        self.class_defaults
            .insert("give-13.1".to_string(), LittleVType::Cause);
        self.class_defaults
            .insert("send-11.1".to_string(), LittleVType::Cause);
        self.class_defaults
            .insert("obtain-13.5.2".to_string(), LittleVType::Cause);
        self.class_defaults
            .insert("get-13.5.1".to_string(), LittleVType::Cause);

        // Change of state
        self.class_defaults
            .insert("break-45.1".to_string(), LittleVType::Cause);
        self.class_defaults
            .insert("destroy-44".to_string(), LittleVType::Cause);
        self.class_defaults
            .insert("build-26.1".to_string(), LittleVType::Cause);
        self.class_defaults
            .insert("create-26.4".to_string(), LittleVType::Cause);
        self.class_defaults
            .insert("cut-21.1".to_string(), LittleVType::Cause);

        // Psychological verbs
        self.class_defaults
            .insert("admire-31.2".to_string(), LittleVType::Experience);
        self.class_defaults
            .insert("amuse-31.1".to_string(), LittleVType::Cause); // Causative psych
        self.class_defaults
            .insert("fear-31.3".to_string(), LittleVType::Experience);
        self.class_defaults
            .insert("marvel-31.3".to_string(), LittleVType::Experience);

        // Communication verbs
        self.class_defaults
            .insert("say-37.7".to_string(), LittleVType::Say);
        self.class_defaults
            .insert("tell-37.2".to_string(), LittleVType::Say);
        self.class_defaults
            .insert("complain-37.8".to_string(), LittleVType::Say);

        // Existence verbs
        self.class_defaults
            .insert("exist-47.1".to_string(), LittleVType::Exist);
        self.class_defaults
            .insert("appear-48.1".to_string(), LittleVType::Become);

        // Possession verbs
        self.class_defaults
            .insert("own-100.1".to_string(), LittleVType::Have);
        self.class_defaults
            .insert("contain-47.8".to_string(), LittleVType::Have);

        // Consumption verbs
        self.class_defaults
            .insert("eat-39.1".to_string(), LittleVType::Do);
        self.class_defaults
            .insert("devour-39.4".to_string(), LittleVType::Cause);

        // Putting/placing verbs
        self.class_defaults
            .insert("put-9.1".to_string(), LittleVType::Cause);
        self.class_defaults
            .insert("spray-9.7".to_string(), LittleVType::Cause);
    }

    /// Decompose a predicate into LittleV structure
    pub fn decompose(&self, predicate: &PredicateInfo) -> EventResult<DecomposedEvent> {
        // Priority 1: Check semantic predicates from VerbNet
        if let Some(ref verbnet) = predicate.verbnet_analysis {
            for class in &verbnet.verb_classes {
                // Check semantic predicates in frames
                for frame in &class.frames {
                    for sem_pred in &frame.semantics {
                        let pred_name = sem_pred.value.to_lowercase();
                        if let Some(template) = self.predicate_map.get(&pred_name) {
                            return self.apply_template(
                                template,
                                predicate,
                                Some(class.id.clone()),
                            );
                        }
                    }
                }
            }
        }

        // Priority 2: Check VerbNet class defaults
        if let Some(ref verbnet) = predicate.verbnet_analysis {
            for class in &verbnet.verb_classes {
                // Try exact match first
                if let Some(lv_type) = self.class_defaults.get(&class.id) {
                    return self.apply_default_type(*lv_type, predicate, Some(class.id.clone()));
                }

                // Try class prefix match (e.g., "give-13.1-1" matches "give-13.1")
                for (pattern, lv_type) in &self.class_defaults {
                    if class.id.starts_with(pattern) {
                        return self.apply_default_type(
                            *lv_type,
                            predicate,
                            Some(class.id.clone()),
                        );
                    }
                }
            }
        }

        // Priority 3: Fallback to heuristic based on FrameNet
        if let Some(framenet) = predicate
            .framenet_analysis
            .as_ref()
            .filter(|f| !f.frames.is_empty())
        {
            // Use FrameNet frame elements to guess LittleV type
            return self.decompose_from_framenet(predicate, framenet);
        }

        // Priority 4: Last resort - guess from lemma/POS
        self.decompose_by_heuristic(predicate)
    }

    /// Apply a template to create a decomposed event
    fn apply_template(
        &self,
        template: &LittleVTemplate,
        predicate: &PredicateInfo,
        verbnet_class: Option<String>,
    ) -> EventResult<DecomposedEvent> {
        let sub_event = template.sub_event.as_ref().map(|sub| {
            Box::new(DecomposedEvent {
                primary_type: sub.primary_type,
                expected_roles: sub.expected_roles.clone(),
                sub_event: None,
                confidence: sub.base_confidence,
                verbnet_confidence: None,
                sources: vec!["VerbNet-sub".to_string()],
            })
        });

        Ok(DecomposedEvent {
            primary_type: template.primary_type,
            expected_roles: template.expected_roles.clone(),
            sub_event,
            confidence: template.base_confidence * predicate.l1_confidence,
            verbnet_confidence: Some(predicate.l1_confidence),
            sources: vec![format!(
                "VerbNet:{}",
                verbnet_class.unwrap_or_else(|| "unknown".to_string())
            )],
        })
    }

    /// Apply a default LittleV type
    fn apply_default_type(
        &self,
        lv_type: LittleVType,
        predicate: &PredicateInfo,
        verbnet_class: Option<String>,
    ) -> EventResult<DecomposedEvent> {
        Ok(DecomposedEvent {
            primary_type: lv_type,
            expected_roles: lv_type.default_roles(),
            sub_event: None,
            confidence: 0.75 * predicate.l1_confidence,
            verbnet_confidence: Some(predicate.l1_confidence),
            sources: vec![format!(
                "VerbNet-class:{}",
                verbnet_class.unwrap_or_else(|| "unknown".to_string())
            )],
        })
    }

    /// Decompose using FrameNet information
    fn decompose_from_framenet(
        &self,
        predicate: &PredicateInfo,
        framenet: &canopy_framenet::FrameNetAnalysis,
    ) -> EventResult<DecomposedEvent> {
        // Simple heuristic based on frame name patterns
        let frame_name = framenet
            .frames
            .first()
            .map(|f| f.name.to_lowercase())
            .unwrap_or_default();

        let (lv_type, expected_roles) = if frame_name.contains("caus")
            || frame_name.contains("destroy")
            || frame_name.contains("break")
        {
            (
                LittleVType::Cause,
                vec![ThetaRole::Agent, ThetaRole::Patient],
            )
        } else if frame_name.contains("motion") || frame_name.contains("travel") {
            (
                LittleVType::Go,
                vec![ThetaRole::Theme, ThetaRole::Source, ThetaRole::Goal],
            )
        } else if frame_name.contains("statement")
            || frame_name.contains("communication")
            || frame_name.contains("tell")
        {
            (
                LittleVType::Say,
                vec![ThetaRole::Agent, ThetaRole::Recipient],
            )
        } else if frame_name.contains("experiencer") || frame_name.contains("emotion") {
            (
                LittleVType::Experience,
                vec![ThetaRole::Experiencer, ThetaRole::Stimulus],
            )
        } else if frame_name.contains("possess") || frame_name.contains("have") {
            (LittleVType::Have, vec![ThetaRole::Agent, ThetaRole::Theme])
        } else if frame_name.contains("exist") || frame_name.contains("presence") {
            (
                LittleVType::Exist,
                vec![ThetaRole::Theme, ThetaRole::Location],
            )
        } else {
            // Default to Do for unrecognized frames
            (LittleVType::Do, vec![ThetaRole::Agent])
        };

        Ok(DecomposedEvent {
            primary_type: lv_type,
            expected_roles,
            sub_event: None,
            confidence: 0.6 * predicate.l1_confidence,
            verbnet_confidence: None,
            sources: vec![format!("FrameNet:{}", frame_name)],
        })
    }

    /// Last-resort heuristic decomposition
    fn decompose_by_heuristic(&self, predicate: &PredicateInfo) -> EventResult<DecomposedEvent> {
        // Very simple heuristic based on common verb patterns
        let lemma = predicate.lemma.to_lowercase();

        let (lv_type, expected_roles) = if lemma == "be" || lemma == "seem" || lemma == "appear" {
            (LittleVType::Be, vec![ThetaRole::Theme])
        } else if lemma == "have" || lemma == "own" || lemma == "possess" {
            (LittleVType::Have, vec![ThetaRole::Agent, ThetaRole::Theme])
        } else if lemma == "go" || lemma == "come" || lemma == "move" || lemma == "travel" {
            (LittleVType::Go, vec![ThetaRole::Theme, ThetaRole::Goal])
        } else if lemma == "say" || lemma == "tell" || lemma == "speak" || lemma == "ask" {
            (
                LittleVType::Say,
                vec![ThetaRole::Agent, ThetaRole::Recipient],
            )
        } else if lemma == "feel" || lemma == "think" || lemma == "know" || lemma == "believe" {
            (
                LittleVType::Experience,
                vec![ThetaRole::Experiencer, ThetaRole::Stimulus],
            )
        } else if lemma == "exist" || lemma == "there" {
            (
                LittleVType::Exist,
                vec![ThetaRole::Theme, ThetaRole::Location],
            )
        } else {
            // Default: assume activity (Do) with agent
            (LittleVType::Do, vec![ThetaRole::Agent])
        };

        Ok(DecomposedEvent {
            primary_type: lv_type,
            expected_roles,
            sub_event: None,
            confidence: 0.4, // Low confidence for heuristic
            verbnet_confidence: None,
            sources: vec!["Heuristic".to_string()],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_predicate(lemma: &str) -> PredicateInfo {
        PredicateInfo {
            lemma: lemma.to_string(),
            token_idx: 0,
            verbnet_analysis: None,
            framenet_analysis: None,
            l1_confidence: 0.8,
        }
    }

    #[test]
    fn test_heuristic_be() {
        let decomposer = EventDecomposer::new(&EventComposerConfig::default()).unwrap();
        let predicate = make_predicate("be");
        let result = decomposer.decompose(&predicate).unwrap();
        assert_eq!(result.primary_type, LittleVType::Be);
    }

    #[test]
    fn test_heuristic_have() {
        let decomposer = EventDecomposer::new(&EventComposerConfig::default()).unwrap();
        let predicate = make_predicate("have");
        let result = decomposer.decompose(&predicate).unwrap();
        assert_eq!(result.primary_type, LittleVType::Have);
    }

    #[test]
    fn test_heuristic_go() {
        let decomposer = EventDecomposer::new(&EventComposerConfig::default()).unwrap();
        let predicate = make_predicate("go");
        let result = decomposer.decompose(&predicate).unwrap();
        assert_eq!(result.primary_type, LittleVType::Go);
    }

    #[test]
    fn test_heuristic_unknown_defaults_to_do() {
        let decomposer = EventDecomposer::new(&EventComposerConfig::default()).unwrap();
        let predicate = make_predicate("flibbertigibbet");
        let result = decomposer.decompose(&predicate).unwrap();
        assert_eq!(result.primary_type, LittleVType::Do);
        assert!(result.confidence < 0.5);
    }
}
