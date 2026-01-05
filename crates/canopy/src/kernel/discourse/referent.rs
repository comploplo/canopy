//! Discourse referent tracking.
//!
//! Manages discourse referents - entities and events introduced in discourse
//! that can be referred back to by pronouns and other anaphoric expressions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique identifier for a discourse referent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ReferentId(pub usize);

impl ReferentId {
    /// Create a new referent ID.
    #[must_use]
    pub const fn new(id: usize) -> Self {
        Self(id)
    }

    /// Get the underlying value.
    #[must_use]
    pub const fn value(self) -> usize {
        self.0
    }
}

impl std::fmt::Display for ReferentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "r{}", self.0)
    }
}

/// A discourse referent (entity or event that can be referred to).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiscourseReferent {
    /// Unique identifier.
    pub id: ReferentId,

    /// Optional name/description (from the introducing NP).
    pub name: Option<String>,

    /// Type of referent.
    pub referent_type: ReferentType,

    /// Whether this is an event referent (vs entity).
    pub is_event: bool,

    /// Sentence index where this referent was introduced.
    pub introduced_at: usize,

    /// Grammatical gender (for pronoun resolution).
    pub gender: Gender,

    /// Grammatical number.
    pub number: NumberFeature,

    /// Grammatical person.
    pub person: Person,

    /// Salience score (higher = more salient, more likely antecedent).
    pub salience: f32,
}

impl DiscourseReferent {
    /// Create a new entity referent.
    pub fn entity(id: ReferentId, name: impl Into<String>, introduced_at: usize) -> Self {
        Self {
            id,
            name: Some(name.into()),
            referent_type: ReferentType::Individual,
            is_event: false,
            introduced_at,
            gender: Gender::Unknown,
            number: NumberFeature::Singular,
            person: Person::Third,
            salience: 1.0,
        }
    }

    /// Create a new event referent.
    pub fn event(id: ReferentId, predicate: impl Into<String>, introduced_at: usize) -> Self {
        Self {
            id,
            name: Some(predicate.into()),
            referent_type: ReferentType::Event,
            is_event: true,
            introduced_at,
            gender: Gender::Neuter,
            number: NumberFeature::Singular,
            person: Person::Third,
            salience: 0.5, // Events are less salient than entities
        }
    }

    /// Set gender.
    #[must_use]
    pub fn with_gender(mut self, gender: Gender) -> Self {
        self.gender = gender;
        self
    }

    /// Set number.
    #[must_use]
    pub fn with_number(mut self, number: NumberFeature) -> Self {
        self.number = number;
        self
    }

    /// Set person.
    #[must_use]
    pub fn with_person(mut self, person: Person) -> Self {
        self.person = person;
        self
    }

    /// Set salience.
    #[must_use]
    pub fn with_salience(mut self, salience: f32) -> Self {
        self.salience = salience;
        self
    }

    /// Check if this referent agrees with given features.
    #[must_use]
    pub fn agrees_with(&self, gender: Option<Gender>, number: Option<NumberFeature>) -> bool {
        // Check gender agreement
        if let Some(g) = gender {
            if self.gender != Gender::Unknown && g != Gender::Unknown && self.gender != g {
                return false;
            }
        }

        // Check number agreement
        if let Some(n) = number {
            if self.number != NumberFeature::Unknown
                && n != NumberFeature::Unknown
                && self.number != n
            {
                return false;
            }
        }

        true
    }
}

/// Types of discourse referents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReferentType {
    /// Individual entity: "a man", "John".
    Individual,

    /// Plural/group: "the men", "some books".
    Group,

    /// Mass/uncountable: "water", "information".
    Mass,

    /// Event: "the meeting", "his departure".
    Event,

    /// Proposition: "that he left", "the fact that...".
    Proposition,

    /// Time point or interval.
    Time,

    /// Location/place.
    Location,
}

/// Grammatical gender for anaphora resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Gender {
    Masculine,
    Feminine,
    Neuter,
    #[default]
    Unknown,
}

/// Grammatical number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum NumberFeature {
    Singular,
    Plural,
    #[default]
    Unknown,
}

/// Grammatical person.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Person {
    /// First person: I, we, myself, ourselves.
    First,
    /// Second person: you, yourself, yourselves.
    Second,
    /// Third person: he, she, it, they.
    #[default]
    Third,
}

/// Registry of active discourse referents.
#[derive(Debug, Clone, Default)]
pub struct ReferentRegistry {
    /// All referents, keyed by ID.
    referents: HashMap<ReferentId, DiscourseReferent>,

    /// Next available ID.
    next_id: usize,

    /// Current sentence index.
    current_sentence: usize,
}

impl ReferentRegistry {
    /// Create a new empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Introduce a new entity referent.
    pub fn introduce_entity(&mut self, name: impl Into<String>) -> ReferentId {
        let id = ReferentId::new(self.next_id);
        self.next_id += 1;

        let referent = DiscourseReferent::entity(id, name, self.current_sentence);
        self.referents.insert(id, referent);
        id
    }

    /// Introduce a new event referent.
    pub fn introduce_event(&mut self, predicate: impl Into<String>) -> ReferentId {
        let id = ReferentId::new(self.next_id);
        self.next_id += 1;

        let referent = DiscourseReferent::event(id, predicate, self.current_sentence);
        self.referents.insert(id, referent);
        id
    }

    /// Introduce a referent with full specification.
    pub fn introduce(&mut self, referent: DiscourseReferent) -> ReferentId {
        let id = referent.id;
        self.referents.insert(id, referent);
        if id.0 >= self.next_id {
            self.next_id = id.0 + 1;
        }
        id
    }

    /// Get a referent by ID.
    #[must_use]
    pub fn get(&self, id: ReferentId) -> Option<&DiscourseReferent> {
        self.referents.get(&id)
    }

    /// Get a mutable referent by ID.
    pub fn get_mut(&mut self, id: ReferentId) -> Option<&mut DiscourseReferent> {
        self.referents.get_mut(&id)
    }

    /// Update referent salience (decay older referents).
    pub fn decay_salience(&mut self, decay_factor: f32) {
        for referent in self.referents.values_mut() {
            referent.salience *= decay_factor;
        }
    }

    /// Boost salience of a referent (when mentioned again).
    pub fn boost_salience(&mut self, id: ReferentId, boost: f32) {
        if let Some(referent) = self.referents.get_mut(&id) {
            referent.salience = (referent.salience + boost).min(1.0);
        }
    }

    /// Find candidates for pronoun resolution.
    ///
    /// Returns referents that agree with the given features, sorted by salience.
    #[must_use]
    pub fn find_candidates(
        &self,
        gender: Option<Gender>,
        number: Option<NumberFeature>,
    ) -> Vec<&DiscourseReferent> {
        let mut candidates: Vec<_> = self
            .referents
            .values()
            .filter(|r| !r.is_event && r.agrees_with(gender, number))
            .collect();

        // Sort by salience (descending)
        candidates.sort_by(|a, b| {
            b.salience
                .partial_cmp(&a.salience)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        candidates
    }

    /// Get all entity referents.
    #[must_use]
    pub fn entities(&self) -> Vec<&DiscourseReferent> {
        self.referents.values().filter(|r| !r.is_event).collect()
    }

    /// Get all event referents.
    #[must_use]
    pub fn events(&self) -> Vec<&DiscourseReferent> {
        self.referents.values().filter(|r| r.is_event).collect()
    }

    /// Get count of referents.
    #[must_use]
    pub fn len(&self) -> usize {
        self.referents.len()
    }

    /// Check if registry is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.referents.is_empty()
    }

    /// Advance to next sentence.
    pub fn next_sentence(&mut self) {
        self.current_sentence += 1;
    }

    /// Get current sentence index.
    #[must_use]
    pub fn current_sentence(&self) -> usize {
        self.current_sentence
    }

    /// Allocate next referent ID without introducing.
    pub fn next_id(&mut self) -> ReferentId {
        let id = ReferentId::new(self.next_id);
        self.next_id += 1;
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_referent_id_display() {
        let id = ReferentId::new(5);
        assert_eq!(format!("{id}"), "r5");
    }

    #[test]
    fn test_entity_creation() {
        let id = ReferentId::new(0);
        let entity = DiscourseReferent::entity(id, "man", 0);

        assert_eq!(entity.id, id);
        assert_eq!(entity.name, Some("man".to_string()));
        assert!(!entity.is_event);
        assert_eq!(entity.referent_type, ReferentType::Individual);
    }

    #[test]
    fn test_event_creation() {
        let id = ReferentId::new(0);
        let event = DiscourseReferent::event(id, "walk", 0);

        assert!(event.is_event);
        assert_eq!(event.referent_type, ReferentType::Event);
        assert_eq!(event.gender, Gender::Neuter);
    }

    #[test]
    fn test_agreement() {
        let entity = DiscourseReferent::entity(ReferentId::new(0), "man", 0)
            .with_gender(Gender::Masculine)
            .with_number(NumberFeature::Singular);

        assert!(entity.agrees_with(Some(Gender::Masculine), Some(NumberFeature::Singular)));
        assert!(!entity.agrees_with(Some(Gender::Feminine), None));
        assert!(!entity.agrees_with(None, Some(NumberFeature::Plural)));
        assert!(entity.agrees_with(Some(Gender::Unknown), None)); // Unknown always agrees
    }

    #[test]
    fn test_registry_introduce_entity() {
        let mut registry = ReferentRegistry::new();
        let id = registry.introduce_entity("John");

        assert_eq!(id, ReferentId::new(0));
        assert_eq!(registry.len(), 1);
        assert!(registry.get(id).is_some());
    }

    #[test]
    fn test_registry_find_candidates() {
        let mut registry = ReferentRegistry::new();

        // Introduce some entities
        let john_id = registry.introduce_entity("John");
        if let Some(john) = registry.get_mut(john_id) {
            john.gender = Gender::Masculine;
            john.salience = 0.9;
        }

        let mary_id = registry.introduce_entity("Mary");
        if let Some(mary) = registry.get_mut(mary_id) {
            mary.gender = Gender::Feminine;
            mary.salience = 0.8;
        }

        // Find masculine candidates
        let candidates = registry.find_candidates(Some(Gender::Masculine), None);
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].id, john_id);

        // Find all candidates
        let all = registry.find_candidates(None, None);
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].id, john_id); // Higher salience first
    }

    #[test]
    fn test_salience_decay() {
        let mut registry = ReferentRegistry::new();
        let id = registry.introduce_entity("test");

        if let Some(r) = registry.get_mut(id) {
            r.salience = 1.0;
        }

        registry.decay_salience(0.8);

        let referent = registry.get(id).unwrap();
        assert!((referent.salience - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_salience_boost() {
        let mut registry = ReferentRegistry::new();
        let id = registry.introduce_entity("test");

        if let Some(r) = registry.get_mut(id) {
            r.salience = 0.5;
        }

        registry.boost_salience(id, 0.3);

        let referent = registry.get(id).unwrap();
        assert!((referent.salience - 0.8).abs() < 0.001);
    }
}
