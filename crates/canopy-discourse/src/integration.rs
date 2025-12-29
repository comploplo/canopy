//! Multi-Sentence Semantic Integration
//!
//! Builds richer meaning representations that integrate information
//! across multiple sentences, tracking:
//! - Entity profiles (accumulated properties and roles)
//! - Event chains (causal, temporal, thematic sequences)
//! - Discourse structure (coherence relations between segments)

use crate::coherence::{CoherenceAnalyzer, CoherenceRelation, DrsSegment};
use crate::referent::{PropertyValue, ReferentId};
use canopy_core::ThetaRole;
use canopy_events::ComposedEvent;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Accumulated knowledge about an entity across discourse
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityProfile {
    /// Primary referent ID
    pub id: ReferentId,

    /// Canonical name (first definite description or proper name)
    pub canonical_name: String,

    /// All surface forms used to refer to this entity
    pub aliases: Vec<String>,

    /// Accumulated properties from discourse
    pub properties: IndexMap<String, PropertyValue>,

    /// Events this entity participated in, with roles
    pub event_roles: Vec<(ReferentId, ThetaRole)>,

    /// First mention (sentence index)
    pub first_mention: usize,

    /// Most recent mention (sentence index)
    pub last_mention: usize,

    /// Mention count
    pub mention_count: usize,
}

impl EntityProfile {
    /// Create a new entity profile
    #[must_use]
    pub fn new(id: ReferentId, name: String, first_mention: usize) -> Self {
        Self {
            id,
            canonical_name: name.clone(),
            aliases: vec![name],
            properties: IndexMap::new(),
            event_roles: Vec::new(),
            first_mention,
            last_mention: first_mention,
            mention_count: 1,
        }
    }

    /// Add an alias (another way this entity was referred to)
    pub fn add_alias(&mut self, alias: String) {
        if !self.aliases.contains(&alias) {
            self.aliases.push(alias);
        }
    }

    /// Record event participation
    pub fn add_event_role(&mut self, event_id: ReferentId, role: ThetaRole) {
        self.event_roles.push((event_id, role));
    }

    /// Add or update a property
    pub fn set_property(&mut self, key: impl Into<String>, value: PropertyValue) {
        self.properties.insert(key.into(), value);
    }

    /// Update mention tracking
    pub fn record_mention(&mut self, sentence_idx: usize) {
        self.last_mention = sentence_idx;
        self.mention_count += 1;
    }

    /// Calculate entity prominence in discourse
    #[must_use]
    pub fn prominence_score(&self) -> f32 {
        let mention_factor = (self.mention_count as f32).sqrt();
        let role_factor = self.event_roles.len() as f32 * 0.5;
        let span_factor = (self.last_mention - self.first_mention + 1) as f32 * 0.1;

        mention_factor + role_factor + span_factor
    }

    /// Get all events where entity was an agent
    #[must_use]
    pub fn agent_events(&self) -> Vec<ReferentId> {
        self.event_roles
            .iter()
            .filter_map(|(e, r)| {
                if *r == ThetaRole::Agent {
                    Some(*e)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get all events where entity was a patient/theme
    #[must_use]
    pub fn patient_events(&self) -> Vec<ReferentId> {
        self.event_roles
            .iter()
            .filter_map(|(e, r)| {
                if *r == ThetaRole::Patient || *r == ThetaRole::Theme {
                    Some(*e)
                } else {
                    None
                }
            })
            .collect()
    }
}

/// Type of event chain
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChainType {
    /// Events connected by causation
    Causal,
    /// Events in temporal sequence
    Temporal,
    /// Events sharing a thematic connection
    Thematic,
    /// Events sharing a protagonist
    Protagonist,
}

/// A connected sequence of events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventChain {
    /// Events in the chain (ordered)
    pub events: Vec<ReferentId>,

    /// Type of connection
    pub chain_type: ChainType,

    /// Main participant (protagonist) if any
    pub protagonist: Option<ReferentId>,

    /// Coherence relations between consecutive events
    pub relations: Vec<CoherenceRelation>,
}

impl EventChain {
    /// Create a new event chain
    #[must_use]
    pub fn new(chain_type: ChainType) -> Self {
        Self {
            events: Vec::new(),
            chain_type,
            protagonist: None,
            relations: Vec::new(),
        }
    }

    /// Add an event to the chain
    pub fn add_event(&mut self, event: ReferentId, relation: Option<CoherenceRelation>) {
        self.events.push(event);
        if let Some(rel) = relation {
            self.relations.push(rel);
        }
    }

    /// Get chain length
    #[must_use]
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Check if chain is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

/// Summary of discourse so far
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscourseSummary {
    /// Total sentences processed
    pub sentence_count: usize,

    /// Total events
    pub event_count: usize,

    /// Total entities
    pub entity_count: usize,

    /// Most prominent entities (by score)
    pub prominent_entities: Vec<(ReferentId, f32)>,

    /// Longest event chain
    pub longest_chain_length: usize,

    /// Average coherence score
    pub avg_coherence: f32,
}

/// Multi-sentence semantic integrator
///
/// Tracks entities and events across discourse, building rich profiles
/// and detecting meaningful chains.
#[derive(Debug, Clone)]
pub struct SemanticIntegrator {
    /// Entity profiles indexed by referent ID
    entity_profiles: HashMap<ReferentId, EntityProfile>,

    /// Event chains
    event_chains: Vec<EventChain>,

    /// Current event chain being built
    current_chain: Option<EventChain>,

    /// Coherence analyzer for relation detection
    coherence_analyzer: CoherenceAnalyzer,

    /// Sentence count
    sentence_count: usize,

    /// Event count
    event_count: usize,

    /// Coherence scores for averaging
    coherence_scores: Vec<f32>,

    /// Previous segment for coherence tracking
    prev_segment: Option<DrsSegment>,
}

impl SemanticIntegrator {
    /// Create a new semantic integrator
    #[must_use]
    pub fn new() -> Self {
        Self {
            entity_profiles: HashMap::new(),
            event_chains: Vec::new(),
            current_chain: Some(EventChain::new(ChainType::Temporal)),
            coherence_analyzer: CoherenceAnalyzer::new(),
            sentence_count: 0,
            event_count: 0,
            coherence_scores: Vec::new(),
            prev_segment: None,
        }
    }

    /// Integrate a new sentence (set of composed events)
    pub fn integrate_sentence(&mut self, events: &[ComposedEvent], sentence_idx: usize) {
        self.sentence_count += 1;

        for composed in events {
            self.event_count += 1;
            let event = &composed.event;

            // Create segment for coherence tracking
            let mut segment = DrsSegment::new(sentence_idx);
            segment.predicate = Some(event.predicate.clone());

            // Process participants
            for (role, entity) in &event.participants {
                // Get or create entity profile
                let entity_id = self.get_or_create_entity(&entity.text, sentence_idx);
                segment.add_entity(entity_id);

                // Update entity profile
                if let Some(profile) = self.entity_profiles.get_mut(&entity_id) {
                    // Create a synthetic event referent ID
                    let event_ref = ReferentId(1000 + self.event_count);
                    profile.add_event_role(event_ref, *role);
                    profile.record_mention(sentence_idx);

                    // Set animacy if available
                    if let Some(animacy) = entity.animacy {
                        profile.set_property("animacy", PropertyValue::Animacy(animacy));
                    }
                }

                // Track protagonist if agent
                if *role == ThetaRole::Agent {
                    if let Some(chain) = &mut self.current_chain {
                        chain.protagonist = Some(entity_id);
                    }
                }
            }

            // Add event to current chain
            if let Some(chain) = &mut self.current_chain {
                let event_ref = ReferentId(1000 + self.event_count);
                segment.main_event = Some(event_ref);

                // Compute coherence with previous segment
                let relation = if let Some(prev) = &self.prev_segment {
                    let (rel, conf) = self.coherence_analyzer.infer_relation(prev, &segment, None);
                    self.coherence_scores.push(rel.strength() * conf);
                    Some(rel)
                } else {
                    None
                };

                chain.add_event(event_ref, relation);

                // Check if chain should be broken (topic shift, contrast, etc.)
                if let Some(rel) = relation {
                    if matches!(rel, CoherenceRelation::Contrast) {
                        // Start new chain on contrast
                        if chain.len() > 1 {
                            let finished_chain =
                                std::mem::replace(chain, EventChain::new(ChainType::Temporal));
                            self.event_chains.push(finished_chain);
                        }
                    }
                }
            }

            self.prev_segment = Some(segment);
        }
    }

    /// Get or create an entity ID for a name
    fn get_or_create_entity(&mut self, name: &str, sentence_idx: usize) -> ReferentId {
        // Check existing profiles by canonical name or alias
        for (id, profile) in &self.entity_profiles {
            if profile.canonical_name.eq_ignore_ascii_case(name)
                || profile.aliases.iter().any(|a| a.eq_ignore_ascii_case(name))
            {
                return *id;
            }
        }

        // Create new profile
        let id = ReferentId(self.entity_profiles.len() + 1);
        let profile = EntityProfile::new(id, name.to_string(), sentence_idx);
        self.entity_profiles.insert(id, profile);
        id
    }

    /// Get entity profile by ID
    #[must_use]
    pub fn entity_profile(&self, id: ReferentId) -> Option<&EntityProfile> {
        self.entity_profiles.get(&id)
    }

    /// Get mutable entity profile by ID
    pub fn entity_profile_mut(&mut self, id: ReferentId) -> Option<&mut EntityProfile> {
        self.entity_profiles.get_mut(&id)
    }

    /// Find entity by name
    #[must_use]
    pub fn find_entity(&self, name: &str) -> Option<&EntityProfile> {
        self.entity_profiles.values().find(|p| {
            p.canonical_name.eq_ignore_ascii_case(name)
                || p.aliases.iter().any(|a| a.eq_ignore_ascii_case(name))
        })
    }

    /// Get all event chains involving an entity
    #[must_use]
    pub fn event_chains_for(&self, entity: ReferentId) -> Vec<&EventChain> {
        self.event_chains
            .iter()
            .filter(|chain| chain.protagonist == Some(entity))
            .collect()
    }

    /// Get all event chains
    #[must_use]
    pub fn all_chains(&self) -> &[EventChain] {
        &self.event_chains
    }

    /// Get prominent entities (sorted by prominence score)
    #[must_use]
    pub fn prominent_entities(&self, limit: usize) -> Vec<(ReferentId, f32)> {
        let mut entities: Vec<_> = self
            .entity_profiles
            .iter()
            .map(|(id, profile)| (*id, profile.prominence_score()))
            .collect();

        entities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        entities.truncate(limit);
        entities
    }

    /// Generate discourse summary
    #[must_use]
    pub fn discourse_summary(&self) -> DiscourseSummary {
        let longest_chain = self.event_chains.iter().map(|c| c.len()).max().unwrap_or(0);

        let avg_coherence = if self.coherence_scores.is_empty() {
            1.0
        } else {
            self.coherence_scores.iter().sum::<f32>() / self.coherence_scores.len() as f32
        };

        DiscourseSummary {
            sentence_count: self.sentence_count,
            event_count: self.event_count,
            entity_count: self.entity_profiles.len(),
            prominent_entities: self.prominent_entities(5),
            longest_chain_length: longest_chain,
            avg_coherence,
        }
    }

    /// Finalize current chain and add to completed chains
    pub fn finalize_chain(&mut self) {
        if let Some(chain) = self.current_chain.take() {
            if !chain.is_empty() {
                self.event_chains.push(chain);
            }
        }
        self.current_chain = Some(EventChain::new(ChainType::Temporal));
    }

    /// Reset the integrator
    pub fn reset(&mut self) {
        self.entity_profiles.clear();
        self.event_chains.clear();
        self.current_chain = Some(EventChain::new(ChainType::Temporal));
        self.sentence_count = 0;
        self.event_count = 0;
        self.coherence_scores.clear();
        self.prev_segment = None;
    }

    /// Get entity count
    #[must_use]
    pub fn entity_count(&self) -> usize {
        self.entity_profiles.len()
    }

    /// Get sentence count
    #[must_use]
    pub fn sentence_count(&self) -> usize {
        self.sentence_count
    }
}

impl Default for SemanticIntegrator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use canopy_core::{Action, Animacy, AspectualClass, Entity, Event, LittleV, Voice};

    #[test]
    fn test_entity_profile_creation() {
        let profile = EntityProfile::new(ReferentId(1), "John".to_string(), 0);
        assert_eq!(profile.canonical_name, "John");
        assert_eq!(profile.first_mention, 0);
        assert_eq!(profile.mention_count, 1);
    }

    #[test]
    fn test_entity_alias_tracking() {
        let mut profile = EntityProfile::new(ReferentId(1), "John".to_string(), 0);
        profile.add_alias("he".to_string());
        profile.add_alias("him".to_string());

        assert!(profile.aliases.contains(&"John".to_string()));
        assert!(profile.aliases.contains(&"he".to_string()));
        assert!(profile.aliases.contains(&"him".to_string()));
    }

    #[test]
    fn test_entity_alias_deduplication() {
        let mut profile = EntityProfile::new(ReferentId(1), "John".to_string(), 0);
        profile.add_alias("he".to_string());
        profile.add_alias("he".to_string()); // Duplicate

        // Should only have 2 aliases: "John" and "he"
        assert_eq!(profile.aliases.len(), 2);
    }

    #[test]
    fn test_entity_set_property() {
        let mut profile = EntityProfile::new(ReferentId(1), "John".to_string(), 0);
        profile.set_property("animacy", PropertyValue::Animacy(Animacy::Human));

        assert!(profile.properties.contains_key("animacy"));
        assert_eq!(
            profile.properties.get("animacy"),
            Some(&PropertyValue::Animacy(Animacy::Human))
        );
    }

    #[test]
    fn test_entity_event_roles() {
        let mut profile = EntityProfile::new(ReferentId(1), "John".to_string(), 0);
        profile.add_event_role(ReferentId(100), ThetaRole::Agent);
        profile.add_event_role(ReferentId(101), ThetaRole::Patient);

        let agent_events = profile.agent_events();
        assert_eq!(agent_events.len(), 1);
        assert_eq!(agent_events[0], ReferentId(100));

        let patient_events = profile.patient_events();
        assert_eq!(patient_events.len(), 1);
        assert_eq!(patient_events[0], ReferentId(101));
    }

    #[test]
    fn test_entity_theme_role_in_patient_events() {
        let mut profile = EntityProfile::new(ReferentId(1), "book".to_string(), 0);
        profile.add_event_role(ReferentId(100), ThetaRole::Theme);
        profile.add_event_role(ReferentId(101), ThetaRole::Location); // Not patient/theme

        let patient_events = profile.patient_events();
        assert_eq!(patient_events.len(), 1);
        assert_eq!(patient_events[0], ReferentId(100));
    }

    #[test]
    fn test_prominence_score() {
        let mut profile = EntityProfile::new(ReferentId(1), "John".to_string(), 0);
        profile.record_mention(1);
        profile.record_mention(2);
        profile.add_event_role(ReferentId(100), ThetaRole::Agent);

        let score = profile.prominence_score();
        assert!(score > 0.0);

        // Verify score components:
        // mention_factor = sqrt(3) ≈ 1.73
        // role_factor = 1 * 0.5 = 0.5
        // span_factor = (2 - 0 + 1) * 0.1 = 0.3
        // Total ≈ 2.53
        assert!(score > 2.0 && score < 3.0);
    }

    #[test]
    fn test_event_chain() {
        let mut chain = EventChain::new(ChainType::Temporal);
        chain.add_event(ReferentId(1), None);
        chain.add_event(ReferentId(2), Some(CoherenceRelation::Narration));

        assert_eq!(chain.len(), 2);
        assert_eq!(chain.chain_type, ChainType::Temporal);
    }

    #[test]
    fn test_event_chain_is_empty() {
        let chain = EventChain::new(ChainType::Causal);
        assert!(chain.is_empty());

        let mut chain2 = EventChain::new(ChainType::Thematic);
        chain2.add_event(ReferentId(1), None);
        assert!(!chain2.is_empty());
    }

    #[test]
    fn test_chain_types() {
        // Test all chain types can be created
        let causal = EventChain::new(ChainType::Causal);
        let temporal = EventChain::new(ChainType::Temporal);
        let thematic = EventChain::new(ChainType::Thematic);
        let protagonist = EventChain::new(ChainType::Protagonist);

        assert_eq!(causal.chain_type, ChainType::Causal);
        assert_eq!(temporal.chain_type, ChainType::Temporal);
        assert_eq!(thematic.chain_type, ChainType::Thematic);
        assert_eq!(protagonist.chain_type, ChainType::Protagonist);
    }

    #[test]
    fn test_semantic_integrator() {
        let mut integrator = SemanticIntegrator::new();

        // Simulate finding entity
        let id = integrator.get_or_create_entity("John", 0);
        assert_eq!(id, ReferentId(1));

        // Same name should return same ID
        let id2 = integrator.get_or_create_entity("John", 1);
        assert_eq!(id, id2);

        // Different name gets different ID
        let id3 = integrator.get_or_create_entity("Mary", 1);
        assert_ne!(id, id3);
    }

    #[test]
    fn test_integrator_entity_profile_access() {
        let mut integrator = SemanticIntegrator::new();
        let id = integrator.get_or_create_entity("John", 0);

        // Test entity_profile
        let profile = integrator.entity_profile(id);
        assert!(profile.is_some());
        assert_eq!(profile.unwrap().canonical_name, "John");

        // Test entity_profile_mut
        let profile_mut = integrator.entity_profile_mut(id);
        assert!(profile_mut.is_some());
        profile_mut.unwrap().add_alias("Johnny".to_string());

        // Verify mutation
        let profile = integrator.entity_profile(id).unwrap();
        assert!(profile.aliases.contains(&"Johnny".to_string()));

        // Test non-existent ID
        assert!(integrator.entity_profile(ReferentId(999)).is_none());
    }

    #[test]
    fn test_integrator_find_entity() {
        let mut integrator = SemanticIntegrator::new();
        integrator.get_or_create_entity("John", 0);
        integrator.get_or_create_entity("Mary", 1);

        // Find by canonical name
        let john = integrator.find_entity("John");
        assert!(john.is_some());
        assert_eq!(john.unwrap().canonical_name, "John");

        // Case-insensitive
        let john_lower = integrator.find_entity("john");
        assert!(john_lower.is_some());

        // Add alias and find by alias
        let id = integrator.get_or_create_entity("John", 0);
        integrator
            .entity_profile_mut(id)
            .unwrap()
            .add_alias("he".to_string());
        let by_alias = integrator.find_entity("he");
        assert!(by_alias.is_some());

        // Non-existent
        assert!(integrator.find_entity("Bob").is_none());
    }

    #[test]
    fn test_integrator_entity_count() {
        let mut integrator = SemanticIntegrator::new();
        assert_eq!(integrator.entity_count(), 0);

        integrator.get_or_create_entity("John", 0);
        assert_eq!(integrator.entity_count(), 1);

        integrator.get_or_create_entity("Mary", 1);
        assert_eq!(integrator.entity_count(), 2);

        // Same entity doesn't increase count
        integrator.get_or_create_entity("John", 2);
        assert_eq!(integrator.entity_count(), 2);
    }

    #[test]
    fn test_integrator_sentence_count() {
        let integrator = SemanticIntegrator::new();
        assert_eq!(integrator.sentence_count(), 0);
    }

    #[test]
    fn test_integrator_all_chains() {
        let mut integrator = SemanticIntegrator::new();
        assert!(integrator.all_chains().is_empty());

        // Finalize a chain
        integrator.get_or_create_entity("John", 0);
        if let Some(chain) = &mut integrator.current_chain {
            chain.add_event(ReferentId(100), None);
        }
        integrator.finalize_chain();

        assert_eq!(integrator.all_chains().len(), 1);
    }

    #[test]
    fn test_integrator_event_chains_for() {
        let mut integrator = SemanticIntegrator::new();
        let john_id = integrator.get_or_create_entity("John", 0);

        // Create chain with John as protagonist
        if let Some(chain) = &mut integrator.current_chain {
            chain.protagonist = Some(john_id);
            chain.add_event(ReferentId(100), None);
        }
        integrator.finalize_chain();

        let john_chains = integrator.event_chains_for(john_id);
        assert_eq!(john_chains.len(), 1);

        // Different entity has no chains
        let mary_id = integrator.get_or_create_entity("Mary", 1);
        let mary_chains = integrator.event_chains_for(mary_id);
        assert!(mary_chains.is_empty());
    }

    #[test]
    fn test_integrator_finalize_chain() {
        let mut integrator = SemanticIntegrator::new();

        // Add event to current chain
        if let Some(chain) = &mut integrator.current_chain {
            chain.add_event(ReferentId(100), None);
        }

        // Finalize
        integrator.finalize_chain();
        assert_eq!(integrator.all_chains().len(), 1);

        // New current chain should be created
        assert!(integrator.current_chain.is_some());
        assert!(integrator.current_chain.as_ref().unwrap().is_empty());
    }

    #[test]
    fn test_integrator_finalize_empty_chain() {
        let mut integrator = SemanticIntegrator::new();

        // Finalize empty chain - should not add to event_chains
        integrator.finalize_chain();
        assert!(integrator.all_chains().is_empty());
    }

    #[test]
    fn test_integrator_reset() {
        let mut integrator = SemanticIntegrator::new();

        // Add some data
        integrator.get_or_create_entity("John", 0);
        integrator.get_or_create_entity("Mary", 1);
        if let Some(chain) = &mut integrator.current_chain {
            chain.add_event(ReferentId(100), None);
        }
        integrator.finalize_chain();
        integrator.coherence_scores.push(0.8);

        // Verify data exists
        assert_eq!(integrator.entity_count(), 2);
        assert_eq!(integrator.all_chains().len(), 1);

        // Reset
        integrator.reset();

        // Verify everything cleared
        assert_eq!(integrator.entity_count(), 0);
        assert!(integrator.all_chains().is_empty());
        assert_eq!(integrator.sentence_count(), 0);
        assert!(integrator.current_chain.is_some());
        assert!(integrator.coherence_scores.is_empty());
        assert!(integrator.prev_segment.is_none());
    }

    #[test]
    fn test_integrator_prominent_entities() {
        let mut integrator = SemanticIntegrator::new();

        // Create entities with different prominence
        let john_id = integrator.get_or_create_entity("John", 0);
        let _mary_id = integrator.get_or_create_entity("Mary", 0);

        // John gets more mentions and roles
        if let Some(profile) = integrator.entity_profile_mut(john_id) {
            profile.record_mention(1);
            profile.record_mention(2);
            profile.add_event_role(ReferentId(100), ThetaRole::Agent);
            profile.add_event_role(ReferentId(101), ThetaRole::Agent);
        }

        let prominent = integrator.prominent_entities(5);
        assert_eq!(prominent.len(), 2);

        // John should be first (higher prominence)
        assert_eq!(prominent[0].0, john_id);
        assert!(prominent[0].1 > prominent[1].1);
    }

    #[test]
    fn test_discourse_summary() {
        let integrator = SemanticIntegrator::new();
        let summary = integrator.discourse_summary();

        assert_eq!(summary.sentence_count, 0);
        assert_eq!(summary.entity_count, 0);
        assert_eq!(summary.avg_coherence, 1.0);
    }

    #[test]
    fn test_discourse_summary_with_data() {
        let mut integrator = SemanticIntegrator::new();

        // Add entities
        integrator.get_or_create_entity("John", 0);

        // Add chain with events
        if let Some(chain) = &mut integrator.current_chain {
            chain.add_event(ReferentId(100), None);
            chain.add_event(ReferentId(101), Some(CoherenceRelation::Narration));
        }
        integrator.finalize_chain();

        // Add coherence scores
        integrator.coherence_scores.push(0.8);
        integrator.coherence_scores.push(0.6);

        let summary = integrator.discourse_summary();

        assert_eq!(summary.entity_count, 1);
        assert_eq!(summary.longest_chain_length, 2);
        assert!((summary.avg_coherence - 0.7).abs() < 0.01);
    }

    #[test]
    fn test_integrator_default() {
        let integrator: SemanticIntegrator = Default::default();
        assert_eq!(integrator.entity_count(), 0);
        assert_eq!(integrator.sentence_count(), 0);
    }

    fn create_test_event(predicate: &str, agent_name: &str) -> ComposedEvent {
        let agent = Entity {
            id: 0,
            text: agent_name.to_string(),
            animacy: Some(Animacy::Human),
            definiteness: None,
        };

        let mut participants = std::collections::HashMap::new();
        participants.insert(ThetaRole::Agent, agent.clone());

        let event = Event {
            id: 0,
            predicate: predicate.to_string(),
            little_v: LittleV::Do {
                agent,
                action: Action {
                    predicate: predicate.to_string(),
                    manner: None,
                    instrument: None,
                },
            },
            participants,
            aspect: AspectualClass::Activity,
            voice: Voice::Active,
        };

        ComposedEvent {
            id: 0,
            event,
            token_span: (0, 1),
            verbnet_source: None,
            framenet_source: None,
            decomposition_confidence: 1.0,
            binding_confidence: 1.0,
        }
    }

    #[test]
    fn test_integrate_sentence_basic() {
        let mut integrator = SemanticIntegrator::new();

        let event = create_test_event("walk", "John");
        integrator.integrate_sentence(&[event], 0);

        assert_eq!(integrator.sentence_count(), 1);
        assert_eq!(integrator.entity_count(), 1);

        // John should have agent role
        let john = integrator.find_entity("John").unwrap();
        assert_eq!(john.agent_events().len(), 1);
    }

    #[test]
    fn test_integrate_multiple_sentences() {
        let mut integrator = SemanticIntegrator::new();

        // Sentence 1: John walks
        let event1 = create_test_event("walk", "John");
        integrator.integrate_sentence(&[event1], 0);

        // Sentence 2: John runs
        let event2 = create_test_event("run", "John");
        integrator.integrate_sentence(&[event2], 1);

        assert_eq!(integrator.sentence_count(), 2);
        assert_eq!(integrator.entity_count(), 1); // Same John

        let john = integrator.find_entity("John").unwrap();
        assert_eq!(john.mention_count, 3); // Initial + 2 integrations
        assert_eq!(john.agent_events().len(), 2);
    }

    #[test]
    fn test_integrate_sentence_with_animacy() {
        let mut integrator = SemanticIntegrator::new();

        let event = create_test_event("walk", "John");
        integrator.integrate_sentence(&[event], 0);

        let john = integrator.find_entity("John").unwrap();
        assert!(john.properties.contains_key("animacy"));
    }

    #[test]
    fn test_integrate_sentence_protagonist_tracking() {
        let mut integrator = SemanticIntegrator::new();

        let event = create_test_event("walk", "John");
        integrator.integrate_sentence(&[event], 0);

        // Current chain should have John as protagonist
        let chain = integrator.current_chain.as_ref().unwrap();
        assert!(chain.protagonist.is_some());
    }

    #[test]
    fn test_get_or_create_entity_by_alias() {
        let mut integrator = SemanticIntegrator::new();

        // Create John and add alias
        let john_id = integrator.get_or_create_entity("John", 0);
        integrator
            .entity_profile_mut(john_id)
            .unwrap()
            .add_alias("Johnny".to_string());

        // Should find same entity by alias
        let id_by_alias = integrator.get_or_create_entity("Johnny", 1);
        assert_eq!(john_id, id_by_alias);
    }
}
