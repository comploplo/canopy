//! Discourse context management.
//!
//! Manages discourse state across sentences, including:
//! - DRS construction
//! - Referent tracking
//! - Pronoun resolution
//! - Temporal ordering

use super::binding::{AnaphorType, BindingResult, PronounResolver};
use super::drs::{Drs, DrsCondition, DrsId, TemporalRelationType};
use super::referent::{Gender, NumberFeature, ReferentId, ReferentRegistry};
use crate::core::ThetaRole;
use crate::kernel::events::{ComposedEvent, ComposedEvents, LittleVType};
use serde::{Deserialize, Serialize};

/// Configuration for discourse processing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscourseConfig {
    /// Salience decay factor between sentences.
    pub salience_decay: f32,

    /// Minimum confidence for pronoun resolution.
    pub min_resolution_confidence: f32,

    /// Whether to track temporal relations.
    pub track_temporal: bool,

    /// Maximum discourse referents to track.
    pub max_referents: usize,
}

impl Default for DiscourseConfig {
    fn default() -> Self {
        Self {
            salience_decay: 0.8,
            min_resolution_confidence: 0.3,
            track_temporal: true,
            max_referents: 100,
        }
    }
}

/// Discourse context - manages state across sentences.
#[derive(Debug, Clone)]
pub struct DiscourseContext {
    /// Configuration.
    config: DiscourseConfig,

    /// The main DRS being built.
    drs: Drs,

    /// Registry of discourse referents.
    registry: ReferentRegistry,

    /// Pronoun resolver.
    resolver: PronounResolver,

    /// Current sentence index.
    current_sentence: usize,

    /// Last event referent (for temporal ordering).
    last_event: Option<ReferentId>,

    /// Next DRS ID.
    next_drs_id: usize,
}

impl Default for DiscourseContext {
    fn default() -> Self {
        Self::new(DiscourseConfig::default())
    }
}

impl DiscourseContext {
    /// Create a new discourse context.
    #[must_use]
    pub fn new(config: DiscourseConfig) -> Self {
        let mut resolver = PronounResolver::new();
        resolver.min_confidence = config.min_resolution_confidence;

        Self {
            config,
            drs: Drs::new(DrsId::new(0)),
            registry: ReferentRegistry::new(),
            resolver,
            current_sentence: 0,
            last_event: None,
            next_drs_id: 1,
        }
    }

    /// Begin processing a new sentence.
    pub fn begin_sentence(&mut self) {
        // Decay salience of existing referents
        self.registry.decay_salience(self.config.salience_decay);
    }

    /// End processing of current sentence.
    pub fn end_sentence(&mut self) {
        self.current_sentence += 1;
        self.registry.next_sentence();
    }

    /// Get the current DRS.
    #[must_use]
    pub fn drs(&self) -> &Drs {
        &self.drs
    }

    /// Get mutable reference to DRS.
    pub fn drs_mut(&mut self) -> &mut Drs {
        &mut self.drs
    }

    /// Get the referent registry.
    #[must_use]
    pub fn registry(&self) -> &ReferentRegistry {
        &self.registry
    }

    /// Introduce a new entity referent.
    pub fn introduce_entity(&mut self, name: impl Into<String>) -> ReferentId {
        let id = self.registry.introduce_entity(name.into());

        // Add to DRS universe
        if let Some(referent) = self.registry.get(id) {
            self.drs.add_referent(referent.clone());
        }

        id
    }

    /// Introduce an entity with specific features.
    pub fn introduce_entity_with_features(
        &mut self,
        name: impl Into<String>,
        gender: Gender,
        number: NumberFeature,
    ) -> ReferentId {
        let id = self.registry.introduce_entity(name);

        if let Some(r) = self.registry.get_mut(id) {
            r.gender = gender;
            r.number = number;
        }

        if let Some(referent) = self.registry.get(id) {
            self.drs.add_referent(referent.clone());
        }

        id
    }

    /// Introduce an event referent.
    pub fn introduce_event(&mut self, predicate: impl Into<String>) -> ReferentId {
        let id = self.registry.introduce_event(predicate);

        if let Some(referent) = self.registry.get(id) {
            self.drs.add_referent(referent.clone());
        }

        // Track temporal ordering
        if self.config.track_temporal {
            if let Some(prev_event) = self.last_event {
                // By default, events in sequence are ordered temporally
                self.drs.add_condition(DrsCondition::TemporalRelation {
                    relation: TemporalRelationType::Before,
                    event1: prev_event,
                    event2: id,
                });
            }
            self.last_event = Some(id);
        }

        id
    }

    /// Add a predicate condition for a referent.
    pub fn add_predicate(&mut self, predicate: impl Into<String>, referent: ReferentId) {
        self.drs.add_predicate(predicate, referent);
    }

    /// Add a theta role binding.
    pub fn add_theta_role(&mut self, event: ReferentId, role: ThetaRole, filler: ReferentId) {
        self.drs.add_theta_role(event, role, filler);

        // Boost salience of filler
        self.registry.boost_salience(filler, 0.2);
    }

    /// Resolve a pronoun.
    pub fn resolve_pronoun(
        &mut self,
        anaphor_type: AnaphorType,
        gender: Option<Gender>,
        number: Option<NumberFeature>,
    ) -> BindingResult {
        let result = self.resolver.resolve(
            &self.registry,
            anaphor_type,
            gender,
            number,
            self.current_sentence,
        );

        // Boost salience of resolved antecedent
        if let Some(antecedent) = result.antecedent {
            self.registry.boost_salience(antecedent, 0.3);
        }

        result
    }

    /// Process composed events from Layer 2.
    pub fn process_events(&mut self, events: &ComposedEvents) {
        for event in &events.events {
            self.process_single_event(event);
        }
    }

    /// Process a single composed event.
    fn process_single_event(&mut self, event: &ComposedEvent) {
        // Create event referent
        let event_id = self.introduce_event(&event.predicate);

        // Add event predicate
        self.drs.add_predicate(&event.predicate, event_id);

        // Process participants
        for (role, participant) in &event.participants {
            // Look up or create referent for participant
            // For now, create new referent (real implementation would resolve)
            let participant_id = self.introduce_entity(&participant.text);

            // Add theta role
            self.add_theta_role(event_id, *role, participant_id);
        }

        // Add aspectual class as property
        let aspect_pred = match event.little_v_type {
            LittleVType::Cause => "causative",
            LittleVType::Become => "inchoative",
            LittleVType::Be => "stative",
            LittleVType::Do => "activity",
            LittleVType::Experience => "psychological",
            LittleVType::Go => "motion",
            LittleVType::Have => "possessive",
            LittleVType::Say => "communication",
            LittleVType::Exist => "existential",
        };
        self.drs.add_predicate(aspect_pred, event_id);

        // Handle polarity
        if !event.polarity {
            // Negated event - would need proper DRS negation
            self.drs.add_predicate("negated", event_id);
        }
    }

    /// Get current sentence index.
    #[must_use]
    pub fn current_sentence(&self) -> usize {
        self.current_sentence
    }

    /// Get referent count.
    #[must_use]
    pub fn referent_count(&self) -> usize {
        self.registry.len()
    }

    /// Allocate a new DRS ID.
    pub fn next_drs_id(&mut self) -> DrsId {
        let id = DrsId::new(self.next_drs_id);
        self.next_drs_id += 1;
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_creation() {
        let ctx = DiscourseContext::default();
        assert_eq!(ctx.current_sentence(), 0);
        assert_eq!(ctx.referent_count(), 0);
    }

    #[test]
    fn test_introduce_entity() {
        let mut ctx = DiscourseContext::default();
        let id = ctx.introduce_entity("John");

        assert_eq!(ctx.referent_count(), 1);
        assert!(ctx.drs().get_referent(id).is_some());
    }

    #[test]
    fn test_introduce_event() {
        let mut ctx = DiscourseContext::default();
        let id = ctx.introduce_event("walk");

        assert_eq!(ctx.referent_count(), 1);
        let referent = ctx.registry().get(id).unwrap();
        assert!(referent.is_event);
    }

    #[test]
    fn test_sentence_progression() {
        let mut ctx = DiscourseContext::default();

        ctx.begin_sentence();
        ctx.introduce_entity("John");
        ctx.end_sentence();

        assert_eq!(ctx.current_sentence(), 1);

        ctx.begin_sentence();
        ctx.introduce_entity("Mary");
        ctx.end_sentence();

        assert_eq!(ctx.current_sentence(), 2);
        assert_eq!(ctx.referent_count(), 2);
    }

    #[test]
    fn test_pronoun_resolution() {
        let mut ctx = DiscourseContext::default();

        // Introduce John
        ctx.begin_sentence();
        let john_id =
            ctx.introduce_entity_with_features("John", Gender::Masculine, NumberFeature::Singular);
        ctx.end_sentence();

        // Resolve "he"
        ctx.begin_sentence();
        let result = ctx.resolve_pronoun(
            AnaphorType::Personal,
            Some(Gender::Masculine),
            Some(NumberFeature::Singular),
        );

        assert!(result.is_resolved());
        assert_eq!(result.antecedent, Some(john_id));
    }

    #[test]
    fn test_temporal_ordering() {
        let config = DiscourseConfig {
            track_temporal: true,
            ..Default::default()
        };
        let mut ctx = DiscourseContext::new(config);

        ctx.begin_sentence();
        let e1 = ctx.introduce_event("walk");
        let e2 = ctx.introduce_event("fall");
        ctx.end_sentence();

        // Check temporal relation was added
        let has_temporal = ctx.drs().conditions.iter().any(|c| {
            matches!(c, DrsCondition::TemporalRelation {
                relation: TemporalRelationType::Before,
                event1,
                event2,
            } if *event1 == e1 && *event2 == e2)
        });

        assert!(has_temporal);
    }

    #[test]
    fn test_theta_role_addition() {
        let mut ctx = DiscourseContext::default();

        ctx.begin_sentence();
        let event_id = ctx.introduce_event("walk");
        let john_id = ctx.introduce_entity("John");
        ctx.add_theta_role(event_id, ThetaRole::Agent, john_id);
        ctx.end_sentence();

        // Check theta role was added
        let has_role = ctx.drs().conditions.iter().any(|c| {
            matches!(c, DrsCondition::ThetaRole {
                event_id: e,
                role: ThetaRole::Agent,
                filler: f,
            } if *e == event_id && *f == john_id)
        });

        assert!(has_role);
    }

    #[test]
    fn test_salience_decay() {
        let mut ctx = DiscourseContext::default();

        // Introduce entity in first sentence
        ctx.begin_sentence();
        let id = ctx.introduce_entity("John");
        if let Some(r) = ctx.registry.get_mut(id) {
            r.salience = 1.0;
        }
        ctx.end_sentence();

        // Begin new sentence (triggers decay)
        ctx.begin_sentence();

        let salience = ctx.registry().get(id).unwrap().salience;
        assert!(salience < 1.0);
        assert!((salience - 0.8).abs() < 0.01); // Default decay is 0.8
    }

    #[test]
    fn test_drs_box_notation() {
        let mut ctx = DiscourseContext::default();

        ctx.begin_sentence();
        let john_id = ctx.introduce_entity("John");
        ctx.add_predicate("man", john_id);
        let event_id = ctx.introduce_event("walk");
        ctx.add_theta_role(event_id, ThetaRole::Agent, john_id);
        ctx.end_sentence();

        let notation = ctx.drs().to_box_notation();
        assert!(notation.contains("man"));
        assert!(notation.contains("Agent"));
    }
}
