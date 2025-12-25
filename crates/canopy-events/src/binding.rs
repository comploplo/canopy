//! Participant binding from dependencies to theta roles
//!
//! Maps syntactic dependents to semantic participants based on
//! dependency relations and VerbNet theta role expectations.

use crate::config::EventComposerConfig;
use crate::error::EventResult;
use crate::types::{
    ComposedEvent, DecomposedEvent, PredicateInfo, SentenceAnalysis, UnbindingReason, UnboundEntity,
};
use canopy_core::{
    Action, Animacy, AspectualClass, Definiteness, Entity, Event, LittleV, Path, PossessionType,
    Proposition, PsychType, State, ThetaRole, Voice,
};
use canopy_tokenizer::coordinator::Layer1SemanticResult;
use canopy_treebank::types::DependencyRelation;
use std::collections::HashMap;

/// Binds syntactic dependents to semantic participants
pub struct ParticipantBinder {
    /// Dependency relation -> candidate theta roles
    dep_to_theta: HashMap<DependencyRelation, Vec<ThetaRole>>,

    /// Priority order for role selection
    role_priority: Vec<ThetaRole>,

    /// Configuration
    #[allow(dead_code)]
    config: EventComposerConfig,
}

impl ParticipantBinder {
    /// Create a new binder with default mappings
    pub fn new(config: &EventComposerConfig) -> EventResult<Self> {
        let mut binder = Self {
            dep_to_theta: HashMap::new(),
            role_priority: vec![
                ThetaRole::Agent,
                ThetaRole::Patient,
                ThetaRole::Theme,
                ThetaRole::Recipient,
                ThetaRole::Experiencer,
                ThetaRole::Goal,
                ThetaRole::Source,
                ThetaRole::Location,
                ThetaRole::Instrument,
                ThetaRole::Benefactive,
                ThetaRole::Manner,
                ThetaRole::Temporal,
            ],
            config: config.clone(),
        };
        binder.initialize_dep_mappings();
        Ok(binder)
    }

    /// Initialize dependency -> theta role mappings
    fn initialize_dep_mappings(&mut self) {
        use DependencyRelation::*;

        // Core argument mappings
        self.dep_to_theta.insert(
            NominalSubject,
            vec![ThetaRole::Agent, ThetaRole::Experiencer, ThetaRole::Theme],
        );

        self.dep_to_theta
            .insert(Object, vec![ThetaRole::Patient, ThetaRole::Theme]);

        self.dep_to_theta.insert(
            IndirectObject,
            vec![ThetaRole::Recipient, ThetaRole::Benefactive, ThetaRole::Goal],
        );

        // Oblique arguments are more ambiguous
        self.dep_to_theta.insert(
            Oblique,
            vec![
                ThetaRole::Location,
                ThetaRole::Source,
                ThetaRole::Goal,
                ThetaRole::Instrument,
                ThetaRole::Manner,
                ThetaRole::Temporal,
            ],
        );

        // Clausal subject
        self.dep_to_theta.insert(
            ClausalSubject,
            vec![ThetaRole::Theme, ThetaRole::Stimulus],
        );
    }

    /// Bind participants to theta roles
    pub fn bind_participants(
        &self,
        decomposed: DecomposedEvent,
        analysis: &SentenceAnalysis,
        predicate: &PredicateInfo,
    ) -> EventResult<(ComposedEvent, Vec<UnboundEntity>)> {
        let mut participants: HashMap<ThetaRole, Entity> = HashMap::new();
        let mut unbound: Vec<UnboundEntity> = Vec::new();
        let mut binding_confidence = 1.0;

        // Get dependents of the predicate
        let dependents = analysis.get_dependents(predicate.token_idx);

        // Get expected roles from decomposition
        let expected_roles = &decomposed.expected_roles;

        // Bind each dependent
        for arc in dependents {
            let token = match analysis.get_token(arc.dependent_idx) {
                Some(t) => t,
                None => continue,
            };

            // Get candidate roles for this dependency relation
            let candidate_roles = self
                .dep_to_theta
                .get(&arc.relation)
                .cloned()
                .unwrap_or_default();

            // Find best matching role
            let bound_role = self.select_best_role(
                &candidate_roles,
                expected_roles,
                &participants,
                token,
            );

            if let Some(role) = bound_role {
                let entity = self.create_entity(token, arc.dependent_idx);
                participants.insert(role, entity);
                binding_confidence *= arc.confidence;
            } else {
                // Track unbound entity
                unbound.push(UnboundEntity {
                    token_idx: arc.dependent_idx,
                    text: token.original_word.clone(),
                    suggested_role: candidate_roles.first().copied(),
                    reason: if candidate_roles.is_empty() {
                        UnbindingReason::MissingDependency
                    } else if participants.len() >= expected_roles.len() {
                        UnbindingReason::ExtraCoreArgument
                    } else {
                        UnbindingReason::AmbiguousRole
                    },
                });
            }
        }

        // Build the LittleV structure
        let little_v = self.build_little_v(&decomposed, &participants, predicate)?;

        // Determine aspect and voice
        let aspect = self.determine_aspect(&decomposed, predicate);
        let voice = self.determine_voice(analysis, predicate);

        let event = Event {
            id: 0,
            predicate: predicate.lemma.clone(),
            little_v,
            participants: participants.clone(),
            aspect,
            voice,
        };

        let composed = ComposedEvent {
            id: 0,
            event,
            token_span: (predicate.token_idx, predicate.token_idx),
            verbnet_source: predicate.verbnet_class_id().map(|s| s.to_string()),
            framenet_source: predicate
                .framenet_analysis
                .as_ref()
                .and_then(|f| f.frames.first())
                .map(|f| f.name.clone()),
            decomposition_confidence: decomposed.confidence,
            binding_confidence,
        };

        Ok((composed, unbound))
    }

    /// Select the best theta role given constraints
    fn select_best_role(
        &self,
        candidates: &[ThetaRole],
        expected: &[ThetaRole],
        already_bound: &HashMap<ThetaRole, Entity>,
        token: &Layer1SemanticResult,
    ) -> Option<ThetaRole> {
        // Priority 1: Match both candidate and expected, not yet bound
        for role in &self.role_priority {
            if candidates.contains(role)
                && expected.contains(role)
                && !already_bound.contains_key(role)
            {
                // Check animacy constraints
                if self.satisfies_animacy_constraint(*role, token) {
                    return Some(*role);
                }
            }
        }

        // Priority 2: Any expected role not yet bound
        for role in &self.role_priority {
            if expected.contains(role) && !already_bound.contains_key(role) {
                return Some(*role);
            }
        }

        // Priority 3: Any candidate role not yet bound
        for role in candidates {
            if !already_bound.contains_key(role) {
                return Some(*role);
            }
        }

        None
    }

    /// Check if token satisfies animacy constraints for role
    fn satisfies_animacy_constraint(&self, role: ThetaRole, token: &Layer1SemanticResult) -> bool {
        match role {
            ThetaRole::Agent | ThetaRole::Experiencer => {
                // Agent and Experiencer typically require animacy
                // If we have WordNet data, we could check animacy
                // For now, accept proper nouns and pronouns as likely animate
                if let Some(pos) = token.pos {
                    if matches!(pos, canopy_core::UPos::Propn | canopy_core::UPos::Pron) {
                        return true;
                    }
                }
                // Accept by default if no negative evidence
                true
            }
            _ => true,
        }
    }

    /// Create an Entity from a token
    fn create_entity(&self, token: &Layer1SemanticResult, idx: usize) -> Entity {
        Entity {
            id: idx,
            text: token.original_word.clone(),
            animacy: self.infer_animacy(token),
            definiteness: self.infer_definiteness(token),
        }
    }

    /// Infer animacy from token
    fn infer_animacy(&self, token: &Layer1SemanticResult) -> Option<Animacy> {
        // Check POS - proper nouns and pronouns are often animate
        if let Some(pos) = token.pos {
            if matches!(pos, canopy_core::UPos::Propn | canopy_core::UPos::Pron) {
                return Some(Animacy::Human);
            }
        }

        // Could check WordNet hypernyms for animacy here
        // For now, return None (unknown)
        None
    }

    /// Infer definiteness from token
    fn infer_definiteness(&self, token: &Layer1SemanticResult) -> Option<Definiteness> {
        // Could check for determiners in dependents
        // Proper nouns are typically definite
        if let Some(pos) = token.pos {
            if matches!(pos, canopy_core::UPos::Propn) {
                return Some(Definiteness::Definite);
            }
        }
        None
    }

    /// Build LittleV structure from decomposition and participants
    fn build_little_v(
        &self,
        decomposed: &DecomposedEvent,
        participants: &HashMap<ThetaRole, Entity>,
        predicate: &PredicateInfo,
    ) -> EventResult<LittleV> {
        use crate::types::LittleVType;

        // Helper to get entity or create placeholder
        let get_entity = |role: ThetaRole| -> Entity {
            participants.get(&role).cloned().unwrap_or_else(|| Entity {
                id: 0,
                text: format!("[{}]", format!("{:?}", role).to_lowercase()),
                animacy: None,
                definiteness: None,
            })
        };

        match decomposed.primary_type {
            LittleVType::Cause => Ok(LittleV::Cause {
                causer: get_entity(ThetaRole::Agent),
                caused_predicate: predicate.lemma.clone(),
                caused_theme: participants
                    .get(&ThetaRole::Patient)
                    .or_else(|| participants.get(&ThetaRole::Theme))
                    .cloned()
                    .unwrap_or_else(|| get_entity(ThetaRole::Theme)),
            }),

            LittleVType::Become => Ok(LittleV::Become {
                theme: get_entity(ThetaRole::Theme),
                result_state: State {
                    predicate: predicate.lemma.clone(),
                    polarity: true,
                },
            }),

            LittleVType::Be => Ok(LittleV::Be {
                theme: get_entity(ThetaRole::Theme),
                state: State {
                    predicate: predicate.lemma.clone(),
                    polarity: true,
                },
            }),

            LittleVType::Do => Ok(LittleV::Do {
                agent: get_entity(ThetaRole::Agent),
                action: Action {
                    predicate: predicate.lemma.clone(),
                    manner: None,
                    instrument: participants.get(&ThetaRole::Instrument).cloned(),
                },
            }),

            LittleVType::Experience => Ok(LittleV::Experience {
                experiencer: get_entity(ThetaRole::Experiencer),
                stimulus: get_entity(ThetaRole::Stimulus),
                psych_type: PsychType::SubjectExp, // Default, could infer from frame
            }),

            LittleVType::Go => Ok(LittleV::Go {
                theme: get_entity(ThetaRole::Theme),
                path: Path {
                    source: participants.get(&ThetaRole::Source).cloned(),
                    goal: participants.get(&ThetaRole::Goal).cloned(),
                    route: None,
                    direction: None,
                },
            }),

            LittleVType::Have => Ok(LittleV::Have {
                possessor: get_entity(ThetaRole::Agent),
                possessee: get_entity(ThetaRole::Theme),
                possession_type: PossessionType::Temporary, // Default
            }),

            LittleVType::Say => Ok(LittleV::Say {
                speaker: get_entity(ThetaRole::Agent),
                addressee: participants.get(&ThetaRole::Recipient).cloned(),
                content: Proposition {
                    content: String::new(),
                    modality: None,
                    polarity: true,
                },
            }),

            LittleVType::Exist => Ok(LittleV::Exist {
                entity: get_entity(ThetaRole::Theme),
                location: participants.get(&ThetaRole::Location).cloned(),
            }),
        }
    }

    /// Determine aspectual class from decomposition
    fn determine_aspect(&self, decomposed: &DecomposedEvent, _predicate: &PredicateInfo) -> AspectualClass {
        use crate::types::LittleVType;

        match decomposed.primary_type {
            LittleVType::Be | LittleVType::Have | LittleVType::Exist => AspectualClass::State,
            LittleVType::Do => AspectualClass::Activity,
            LittleVType::Become => AspectualClass::Achievement,
            LittleVType::Cause | LittleVType::Go => AspectualClass::Accomplishment,
            LittleVType::Experience | LittleVType::Say => AspectualClass::Activity,
        }
    }

    /// Determine voice from sentence analysis
    fn determine_voice(&self, analysis: &SentenceAnalysis, _predicate: &PredicateInfo) -> Voice {
        // Use metadata for voice detection
        // Note: Enhanced dependency parsing could provide aux:pass markers
        // but the basic DependencyRelation enum doesn't include those features
        if analysis.metadata.is_passive {
            Voice::Passive
        } else {
            Voice::Active
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::LittleVType;

    #[allow(dead_code)]
    fn make_decomposed(lv_type: LittleVType) -> DecomposedEvent {
        DecomposedEvent {
            primary_type: lv_type,
            expected_roles: lv_type.default_roles(),
            sub_event: None,
            confidence: 0.9,
            verbnet_confidence: Some(0.9),
            sources: vec!["test".to_string()],
        }
    }

    #[test]
    fn test_role_priority() {
        let binder = ParticipantBinder::new(&EventComposerConfig::default()).unwrap();
        assert_eq!(binder.role_priority[0], ThetaRole::Agent);
        assert_eq!(binder.role_priority[1], ThetaRole::Patient);
    }

    #[test]
    fn test_dep_to_theta_nsubj() {
        let binder = ParticipantBinder::new(&EventComposerConfig::default()).unwrap();
        let roles = binder.dep_to_theta.get(&DependencyRelation::NominalSubject);
        assert!(roles.is_some());
        assert!(roles.unwrap().contains(&ThetaRole::Agent));
    }
}
