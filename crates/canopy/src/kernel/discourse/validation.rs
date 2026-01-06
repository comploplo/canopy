use crate::core::ThetaRole;
use crate::kernel::events::{ComposedEvent, PresupposedContent, Presupposition};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ValidationStatus {
    Accepted,
    Contradiction,
    PresuppositionFailure,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub sentence_index: usize,
    pub event_id: usize,
    pub predicate: String,
    pub status: ValidationStatus,
    pub message: Option<String>,
}

impl ValidationReport {
    fn accepted(sentence: usize, event: &ComposedEvent) -> Self {
        Self {
            sentence_index: sentence,
            event_id: event.id,
            predicate: event.predicate.clone(),
            status: ValidationStatus::Accepted,
            message: None,
        }
    }

    fn contradiction(sentence: usize, event: &ComposedEvent, prior_sentence: usize) -> Self {
        Self {
            sentence_index: sentence,
            event_id: event.id,
            predicate: event.predicate.clone(),
            status: ValidationStatus::Contradiction,
            message: Some(format!(
                "Contradicts assertion from sentence {}",
                prior_sentence + 1
            )),
        }
    }

    fn presupp_failure(sentence: usize, event: &ComposedEvent, presupp: &Presupposition) -> Self {
        Self {
            sentence_index: sentence,
            event_id: event.id,
            predicate: event.predicate.clone(),
            status: ValidationStatus::PresuppositionFailure,
            message: Some(format!("Unsatisfied presupposition {presupp}")),
        }
    }
}

#[derive(Debug, Clone)]
struct Commitment {
    predicate: String,
    participants: Vec<(ThetaRole, String)>,
    polarity: bool,
    sentence_index: usize,
}

impl Commitment {
    fn signature(event: &ComposedEvent) -> (String, Vec<(ThetaRole, String)>) {
        let mut participants: Vec<_> = event
            .participants
            .iter()
            .map(|(role, participant)| (*role, participant.text.to_lowercase()))
            .collect();
        participants.sort_by_key(|(role, _)| *role as usize);
        (event.predicate.to_lowercase(), participants)
    }

    fn matches(&self, predicate: &str, participants: &[(ThetaRole, String)]) -> bool {
        self.predicate == predicate
            && self.participants.len() == participants.len()
            && self
                .participants
                .iter()
                .zip(participants.iter())
                .all(|((role_a, text_a), (role_b, text_b))| role_a == role_b && text_a == text_b)
    }
}

#[derive(Default, Debug, Clone)]
struct CommitmentStore {
    assertions: Vec<Commitment>,
}

impl CommitmentStore {
    fn add(&mut self, commitment: Commitment) {
        self.assertions.push(commitment);
    }

    fn find(&self, predicate: &str, participants: &[(ThetaRole, String)]) -> Option<&Commitment> {
        self.assertions
            .iter()
            .find(|c| c.matches(predicate, participants))
    }

    fn satisfies(&self, presupposition: &Presupposition) -> bool {
        match &presupposition.content {
            PresupposedContent::Event { predicate, .. } => self
                .assertions
                .iter()
                .any(|c| c.predicate.eq_ignore_ascii_case(predicate) && c.polarity),
            PresupposedContent::State { description, .. } => self
                .assertions
                .iter()
                .any(|c| c.predicate.eq_ignore_ascii_case(description)),
            PresupposedContent::Existence { entity_text } => self.assertions.iter().any(|c| {
                c.participants
                    .iter()
                    .any(|(_, text)| text.eq_ignore_ascii_case(entity_text))
            }),
        }
    }
}

/// Heuristic validation engine for discourse coherence checking.
///
/// # Implementation Note
///
/// This validation engine maintains its own parallel commitment store separate
/// from the DRS (Discourse Representation Structure). It provides fast heuristic
/// contradiction detection and presupposition checking but does NOT validate
/// against the actual DRS structure.
///
/// For full DRS-based validation, the DRS module's own consistency checking
/// should be used. This engine is designed as a lightweight first pass that
/// catches obvious contradictions (e.g., "John left" followed by "John didn't leave")
/// without the complexity of full logical inference.
///
/// # Limitations
///
/// - Does not check DRS conditions directly
/// - Presupposition satisfaction is approximated via string matching
/// - Cannot detect indirect contradictions that require inference
/// - First sentence of a discourse may show presupposition failures for
///   entities that would normally be introduced by the sentence itself
#[derive(Default, Debug, Clone)]
pub struct ValidationEngine {
    store: CommitmentStore,
}

impl ValidationEngine {
    pub fn assess(&mut self, sentence_index: usize, event: &ComposedEvent) -> ValidationReport {
        let (predicate_key, participant_key) = Commitment::signature(event);

        if let Some(existing) = self.store.find(&predicate_key, &participant_key) {
            if existing.polarity != event.polarity {
                return ValidationReport::contradiction(
                    sentence_index,
                    event,
                    existing.sentence_index,
                );
            }
        }

        if let Some(presupposition) = event
            .presuppositions
            .iter()
            .find(|pres| !self.store.satisfies(pres))
        {
            return ValidationReport::presupp_failure(sentence_index, event, presupposition);
        }

        self.store.add(Commitment {
            predicate: predicate_key,
            participants: participant_key,
            polarity: event.polarity,
            sentence_index,
        });

        ValidationReport::accepted(sentence_index, event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{AspectualClass, Voice};
    use crate::kernel::events::{ComposedEvent, LittleVType, Participant};
    use crate::runtime::TokenId;
    use std::collections::HashMap;

    fn make_event(predicate: &str, agent: &str, id: usize, polarity: bool) -> ComposedEvent {
        let mut participants = HashMap::new();
        participants.insert(ThetaRole::Agent, Participant::new(TokenId::new(0), agent));

        ComposedEvent {
            id,
            predicate: predicate.to_string(),
            little_v_type: LittleVType::Do,
            participants,
            aspect: AspectualClass::Activity,
            voice: Voice::Active,
            token_span: (TokenId::new(0), TokenId::new(1)),
            source_sense: None,
            decomposition_confidence: 1.0,
            binding_confidence: 1.0,
            presuppositions: Vec::new(),
            polarity,
            temporal_frame: None,
            aspectual_viewpoint: None,
        }
    }

    #[test]
    fn test_validation_accepts_first_assertion() {
        let mut engine = ValidationEngine::default();
        let event = make_event("run", "John", 0, true);
        let report = engine.assess(0, &event);

        assert_eq!(report.status, ValidationStatus::Accepted);
        assert!(report.message.is_none());
    }

    #[test]
    fn test_validation_accepts_consistent_assertions() {
        let mut engine = ValidationEngine::default();

        // First assertion: John runs
        let event1 = make_event("run", "John", 0, true);
        let report1 = engine.assess(0, &event1);
        assert_eq!(report1.status, ValidationStatus::Accepted);

        // Second assertion: Mary walks (different predicate, no conflict)
        let event2 = make_event("walk", "Mary", 1, true);
        let report2 = engine.assess(1, &event2);
        assert_eq!(report2.status, ValidationStatus::Accepted);
    }

    #[test]
    fn test_validation_detects_contradiction() {
        let mut engine = ValidationEngine::default();

        // First assertion: John runs (positive)
        let event1 = make_event("run", "John", 0, true);
        engine.assess(0, &event1);

        // Contradicting assertion: John doesn't run (negative)
        let event2 = make_event("run", "John", 1, false);
        let report2 = engine.assess(1, &event2);

        assert_eq!(report2.status, ValidationStatus::Contradiction);
        assert!(report2.message.is_some());
    }

    #[test]
    fn test_validation_report_methods() {
        let mut engine = ValidationEngine::default();
        let event = make_event("leave", "Mary", 0, true);
        let report = engine.assess(0, &event);

        assert_eq!(report.sentence_index, 0);
        assert_eq!(report.event_id, 0);
        assert_eq!(report.predicate, "leave");
    }
}
