//! Layer 3 Discourse Processing
//!
//! This module integrates canopy-discourse with the pipeline, providing
//! a unified interface for multi-sentence discourse analysis.
//!
//! ## Pipeline Flow
//!
//! ```text
//! Text → Layer 1 (Semantic Analysis)
//!      → Layer 2 (Event Composition)
//!      → Layer 3 (Discourse/DRT)
//!      → DRS (Discourse Representation Structure)
//! ```

use canopy_discourse::{DiscourseConfig, DiscourseContext, DiscourseResult, Drs, ReferentId};
use canopy_events::{ComposedEvent, ComposedEvents};

/// Processor for Layer 3 discourse analysis
///
/// Manages discourse context and builds Discourse Representation Structures (DRS)
/// from Layer 2 event compositions.
#[derive(Debug)]
pub struct DiscourseProcessor {
    context: DiscourseContext,
}

impl DiscourseProcessor {
    /// Create a new discourse processor with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self {
            context: DiscourseContext::with_defaults(),
        }
    }

    /// Create a discourse processor with custom configuration
    #[must_use]
    pub fn with_config(config: DiscourseConfig) -> Self {
        Self {
            context: DiscourseContext::new(config),
        }
    }

    /// Process a single sentence's events and add them to discourse context
    ///
    /// Returns the event referent IDs created for this sentence.
    pub fn process_sentence(
        &mut self,
        text: &str,
        events: &ComposedEvents,
    ) -> DiscourseResult<Vec<ReferentId>> {
        self.context.begin_sentence(text.to_string());

        let mut event_ids = Vec::new();
        for event in &events.events {
            let event_id = self.context.process_event(event)?;
            event_ids.push(event_id);
        }

        self.context.end_sentence();
        Ok(event_ids)
    }

    /// Process multiple sentences and build a complete DRS
    ///
    /// Takes pairs of (sentence_text, composed_events) and processes them
    /// in order, building up the discourse context.
    pub fn process_document(
        &mut self,
        sentences: &[(String, ComposedEvents)],
    ) -> DiscourseResult<&Drs> {
        for (text, events) in sentences {
            self.process_sentence(text, events)?;
        }
        Ok(self.context.drs())
    }

    /// Process a single event directly (for fine-grained control)
    pub fn process_event(&mut self, event: &ComposedEvent) -> DiscourseResult<ReferentId> {
        self.context.process_event(event)
    }

    /// Resolve a pronoun to its antecedent in the current discourse
    ///
    /// Uses recency, animacy, and gender/number agreement to find
    /// the most likely antecedent.
    pub fn resolve_pronoun(&mut self, pronoun: &str) -> DiscourseResult<ReferentId> {
        self.context.resolve_pronoun(pronoun)
    }

    /// Get the current Discourse Representation Structure
    #[must_use]
    pub fn drs(&self) -> &Drs {
        self.context.drs()
    }

    /// Get the underlying discourse context for advanced operations
    #[must_use]
    pub fn context(&self) -> &DiscourseContext {
        &self.context
    }

    /// Get mutable access to the discourse context
    pub fn context_mut(&mut self) -> &mut DiscourseContext {
        &mut self.context
    }

    /// Clear all discourse state and start fresh
    pub fn reset(&mut self) {
        self.context.clear();
    }

    /// Get statistics about the current discourse state
    #[must_use]
    pub fn statistics(&self) -> DiscourseStatistics {
        let ctx_stats = self.context.statistics();
        DiscourseStatistics {
            sentence_count: ctx_stats.sentence_count,
            referent_count: ctx_stats.referent_count,
            condition_count: ctx_stats.condition_count,
            resolution_count: ctx_stats.resolution_count,
        }
    }
}

impl Default for DiscourseProcessor {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about discourse processing
#[derive(Debug, Clone)]
pub struct DiscourseStatistics {
    /// Number of sentences processed
    pub sentence_count: usize,
    /// Number of discourse referents (entities + events)
    pub referent_count: usize,
    /// Number of DRS conditions
    pub condition_count: usize,
    /// Number of anaphora resolutions performed
    pub resolution_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use canopy_core::{
        Action, Animacy, AspectualClass, Definiteness, Entity, Event, LittleV, ThetaRole, Voice,
    };
    use std::collections::HashMap;

    #[test]
    fn test_discourse_processor_creation() {
        let processor = DiscourseProcessor::new();
        assert_eq!(processor.statistics().sentence_count, 0);
        assert_eq!(processor.statistics().referent_count, 0);
    }

    #[test]
    fn test_discourse_processor_reset() {
        let mut processor = DiscourseProcessor::new();
        processor.context_mut().begin_sentence("Test.".to_string());
        processor.context_mut().end_sentence();
        assert_eq!(processor.statistics().sentence_count, 1);

        processor.reset();
        assert_eq!(processor.statistics().sentence_count, 0);
    }

    /// Helper to create a simple intransitive event
    fn create_event(predicate: &str, agent_name: &str) -> ComposedEvent {
        let agent = Entity {
            id: 1,
            text: agent_name.to_string(),
            animacy: Some(Animacy::Human),
            definiteness: Some(Definiteness::Definite),
            number: None,
            distributivity: None,
        };

        let mut participants = HashMap::new();
        participants.insert(ThetaRole::Agent, agent.clone());

        let event = Event {
            id: 1,
            predicate: predicate.to_string(),
            little_v: LittleV::Do {
                agent: agent.clone(),
                action: Action {
                    predicate: predicate.to_string(),
                    manner: None,
                    instrument: None,
                },
            },
            participants,
            aspect: AspectualClass::Activity,
            voice: Voice::Active,
            modality: None,
        };

        ComposedEvent {
            id: 0,
            event,
            token_span: (0, 1),
            verbnet_source: Some("run-51.3".to_string()),
            framenet_source: None,
            decomposition_confidence: 0.9,
            binding_confidence: 0.85,
            presuppositions: Vec::new(),
            polarity: true,
        }
    }

    /// Helper to create a transitive event with patient
    fn create_transitive_event(
        predicate: &str,
        agent_name: &str,
        patient_name: &str,
    ) -> ComposedEvent {
        let agent = Entity {
            id: 1,
            text: agent_name.to_string(),
            animacy: Some(Animacy::Human),
            definiteness: Some(Definiteness::Definite),
            number: None,
            distributivity: None,
        };

        let patient = Entity {
            id: 2,
            text: patient_name.to_string(),
            animacy: Some(Animacy::Inanimate),
            definiteness: Some(Definiteness::Indefinite),
            number: None,
            distributivity: None,
        };

        let mut participants = HashMap::new();
        participants.insert(ThetaRole::Agent, agent.clone());
        participants.insert(ThetaRole::Patient, patient.clone());

        let event = Event {
            id: 1,
            predicate: predicate.to_string(),
            little_v: LittleV::Do {
                agent: agent.clone(),
                action: Action {
                    predicate: predicate.to_string(),
                    manner: None,
                    instrument: None,
                },
            },
            participants,
            aspect: AspectualClass::Activity,
            voice: Voice::Active,
            modality: None,
        };

        ComposedEvent {
            id: 0,
            event,
            token_span: (0, 2),
            verbnet_source: Some("get-13.5".to_string()),
            framenet_source: None,
            decomposition_confidence: 0.9,
            binding_confidence: 0.85,
            presuppositions: Vec::new(),
            polarity: true,
        }
    }

    /// Helper to create a ComposedEvents with extra fields
    fn make_events(events: Vec<ComposedEvent>) -> ComposedEvents {
        ComposedEvents {
            events,
            unbound_entities: Vec::new(),
            confidence: 0.9,
            processing_time_us: 100,
            sources: vec!["test".to_string()],
        }
    }

    #[test]
    fn test_process_document_multiple_sentences() {
        let mut processor = DiscourseProcessor::new();

        // Create a 3-sentence document
        let sentences = vec![
            (
                "John entered the room.".to_string(),
                make_events(vec![create_transitive_event("enter", "John", "room")]),
            ),
            (
                "He sat down.".to_string(),
                make_events(vec![create_event("sit", "John")]),
            ),
            (
                "Then he read a book.".to_string(),
                make_events(vec![create_transitive_event("read", "John", "book")]),
            ),
        ];

        let drs = processor.process_document(&sentences);
        assert!(drs.is_ok(), "process_document should succeed");

        // Verify statistics
        let stats = processor.statistics();
        assert_eq!(stats.sentence_count, 3, "Should process 3 sentences");
        assert!(
            stats.referent_count >= 3,
            "Should have at least 3 referents"
        );
    }

    #[test]
    fn test_process_sentence_returns_event_ids() {
        let mut processor = DiscourseProcessor::new();

        let events = make_events(vec![
            create_event("run", "Mary"),
            create_event("jump", "Mary"),
        ]);

        let result = processor.process_sentence("Mary ran and jumped.", &events);
        assert!(result.is_ok());

        let event_ids = result.unwrap();
        assert_eq!(event_ids.len(), 2, "Should return 2 event IDs");
        // Event IDs should be distinct
        assert_ne!(event_ids[0], event_ids[1]);
    }

    #[test]
    fn test_multi_sentence_anaphora_through_pipeline() {
        let mut processor = DiscourseProcessor::new();

        // Sentence 1: Introduce Mary
        let events1 = make_events(vec![create_event("run", "Mary")]);
        processor
            .process_sentence("Mary runs.", &events1)
            .expect("Should process first sentence");

        // Sentence 2: Process another event for Mary
        let events2 = make_events(vec![create_event("smile", "Mary")]);
        processor
            .process_sentence("She smiles.", &events2)
            .expect("Should process second sentence");

        // Try to resolve "she" - Mary should be the antecedent
        let resolved = processor.resolve_pronoun("she");
        assert!(
            resolved.is_ok(),
            "Should resolve 'she' to Mary: {:?}",
            resolved
        );
    }

    #[test]
    fn test_drs_grows_with_document() {
        let mut processor = DiscourseProcessor::new();

        // Check initial DRS is empty
        let initial_refs =
            processor.drs().entity_referents().len() + processor.drs().event_referents().len();
        assert_eq!(initial_refs, 0);

        // Process first sentence
        let events1 = make_events(vec![create_event("walk", "John")]);
        processor.process_sentence("John walks.", &events1).unwrap();

        let after_first =
            processor.drs().entity_referents().len() + processor.drs().event_referents().len();
        assert!(
            after_first > initial_refs,
            "DRS should have more referents after first sentence"
        );

        // Process second sentence
        let events2 = make_events(vec![create_transitive_event("talk", "John", "Mary")]);
        processor
            .process_sentence("He talks to Mary.", &events2)
            .unwrap();

        let after_second =
            processor.drs().entity_referents().len() + processor.drs().event_referents().len();
        assert!(
            after_second > after_first,
            "DRS should grow with each sentence"
        );
    }

    #[test]
    fn test_discourse_processor_with_custom_config() {
        let config = DiscourseConfig::default();
        let processor = DiscourseProcessor::with_config(config);
        assert_eq!(processor.statistics().sentence_count, 0);
    }

    #[test]
    fn test_process_event_directly() {
        let mut processor = DiscourseProcessor::new();

        // Begin a sentence context manually
        processor
            .context_mut()
            .begin_sentence("Test sentence.".to_string());

        let event = create_event("dance", "Alice");
        let result = processor.process_event(&event);

        assert!(result.is_ok(), "Direct event processing should succeed");
        let event_id = result.unwrap();

        // Verify the event was registered (it's in event_referents)
        assert!(
            processor.drs().event_referents().contains(&event_id),
            "Event should be in DRS event referents"
        );

        processor.context_mut().end_sentence();
    }
}
