//! Discourse Representation Structures (DRS)
//!
//! Implementation of Kamp's Discourse Representation Theory (1981).
//! A DRS consists of:
//! - A set of discourse referents (entities introduced in the discourse)
//! - A set of conditions (predicates over those referents)
//!
//! ## Example
//!
//! For "A man walks. He whistles.", the DRS would be:
//!
//! ```text
//! [ x |
//!   man(x),
//!   walk(x),
//!   whistle(x)
//! ]
//! ```
//!
//! Where `x` is a discourse referent introduced by "a man" and
//! "he" is resolved to the same referent.

use crate::referent::{DiscourseReferent, ReferentId};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// A Discourse Representation Structure
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Drs {
    /// Unique identifier for this DRS
    pub id: DrsId,

    /// Universe: the set of discourse referents
    pub universe: IndexMap<ReferentId, DiscourseReferent>,

    /// Conditions: predicates and relations over referents
    pub conditions: Vec<DrsCondition>,

    /// Subordinate DRS structures (for conditionals, quantification, etc.)
    pub subordinates: Vec<SubordinateDrs>,
}

/// Unique identifier for a DRS
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DrsId(pub usize);

/// Types of DRS conditions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DrsCondition {
    /// Unary predicate: P(x)
    Predicate { name: String, referent: ReferentId },

    /// Binary relation: R(x, y)
    Relation {
        name: String,
        arg1: ReferentId,
        arg2: ReferentId,
    },

    /// Event predicate: verb(e) with participants
    EventPredicate {
        event_id: ReferentId,
        predicate: String,
        participants: IndexMap<String, ReferentId>,
    },

    /// Theta role assignment: role(e, x)
    ThetaRole {
        event_id: ReferentId,
        role: canopy_core::ThetaRole,
        filler: ReferentId,
    },

    /// Equality: x = y
    Equality { ref1: ReferentId, ref2: ReferentId },

    /// Negation: NOT(DRS)
    Negation(Box<Drs>),

    /// Disjunction: DRS1 OR DRS2
    Disjunction(Box<Drs>, Box<Drs>),

    /// Implication: DRS1 => DRS2 (for conditionals and universals)
    Implication {
        antecedent: Box<Drs>,
        consequent: Box<Drs>,
    },

    /// Temporal relation between events: before(e1, e2), after(e1, e2), etc.
    TemporalRelation {
        relation: TemporalRelationType,
        event1: ReferentId,
        event2: ReferentId,
    },

    /// Propositional attitude: believe(x, DRS), know(x, DRS), etc.
    PropositionalAttitude {
        attitude: AttitudeType,
        holder: ReferentId,
        content: Box<Drs>,
    },
}

/// Types of temporal relations between events
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TemporalRelationType {
    /// e1 happens before e2
    Before,
    /// e1 happens after e2
    After,
    /// e1 and e2 overlap
    Overlaps,
    /// e1 contains e2
    Contains,
    /// e1 is contained in e2
    During,
    /// e1 and e2 are simultaneous
    Simultaneous,
    /// e1 immediately precedes e2
    Meets,
}

/// Types of propositional attitudes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttitudeType {
    Believe,
    Know,
    Want,
    Fear,
    Hope,
    Say,
    Think,
}

/// Subordinate DRS for embedded structures
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubordinateDrs {
    /// The type of subordination
    pub relation: SubordinationRelation,
    /// The embedded DRS
    pub drs: Drs,
}

/// Types of DRS subordination
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubordinationRelation {
    /// Conditional antecedent
    Antecedent,
    /// Conditional consequent
    Consequent,
    /// Scope of negation
    NegationScope,
    /// Scope of modal
    ModalScope,
    /// Relative clause
    RelativeClause,
    /// Complement clause
    ComplementClause,
}

impl Drs {
    /// Create a new empty DRS
    #[must_use]
    pub fn new(id: DrsId) -> Self {
        Self {
            id,
            universe: IndexMap::new(),
            conditions: Vec::new(),
            subordinates: Vec::new(),
        }
    }

    /// Add a discourse referent to the universe
    pub fn add_referent(&mut self, referent: DiscourseReferent) {
        self.universe.insert(referent.id, referent);
    }

    /// Add a condition to the DRS
    pub fn add_condition(&mut self, condition: DrsCondition) {
        self.conditions.push(condition);
    }

    /// Add a subordinate DRS
    pub fn add_subordinate(&mut self, relation: SubordinationRelation, drs: Drs) {
        self.subordinates.push(SubordinateDrs { relation, drs });
    }

    /// Get a referent by ID
    #[must_use]
    pub fn get_referent(&self, id: ReferentId) -> Option<&DiscourseReferent> {
        self.universe.get(&id)
    }

    /// Find referents matching a predicate
    #[must_use]
    pub fn find_referents_by_predicate(&self, predicate: &str) -> Vec<ReferentId> {
        self.conditions
            .iter()
            .filter_map(|c| match c {
                DrsCondition::Predicate { name, referent } if name == predicate => Some(*referent),
                _ => None,
            })
            .collect()
    }

    /// Get all event referents
    #[must_use]
    pub fn event_referents(&self) -> Vec<ReferentId> {
        self.universe
            .values()
            .filter(|r| r.is_event)
            .map(|r| r.id)
            .collect()
    }

    /// Get all entity (non-event) referents
    #[must_use]
    pub fn entity_referents(&self) -> Vec<ReferentId> {
        self.universe
            .values()
            .filter(|r| !r.is_event)
            .map(|r| r.id)
            .collect()
    }

    /// Check if a referent is accessible from this DRS
    /// (In the universe or in accessible subordinate DRS)
    #[must_use]
    pub fn is_accessible(&self, id: ReferentId) -> bool {
        if self.universe.contains_key(&id) {
            return true;
        }

        // Check subordinates (simplified accessibility check)
        for sub in &self.subordinates {
            if sub.drs.is_accessible(id) {
                return true;
            }
        }

        false
    }

    /// Merge another DRS into this one
    pub fn merge(&mut self, other: Drs) {
        for (id, referent) in other.universe {
            self.universe.insert(id, referent);
        }
        self.conditions.extend(other.conditions);
        self.subordinates.extend(other.subordinates);
    }

    /// Get the number of referents in the universe
    #[must_use]
    pub fn referent_count(&self) -> usize {
        self.universe.len()
    }

    /// Get the number of conditions
    #[must_use]
    pub fn condition_count(&self) -> usize {
        self.conditions.len()
    }
}

impl Default for Drs {
    fn default() -> Self {
        Self::new(DrsId(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::referent::ReferentType;

    #[test]
    fn test_drs_creation() {
        let drs = Drs::new(DrsId(1));
        assert_eq!(drs.id, DrsId(1));
        assert!(drs.universe.is_empty());
        assert!(drs.conditions.is_empty());
    }

    #[test]
    fn test_add_referent() {
        let mut drs = Drs::new(DrsId(1));
        let referent = DiscourseReferent {
            id: ReferentId(1),
            name: Some("man".to_string()),
            referent_type: ReferentType::Individual,
            is_event: false,
            introduced_at: 0,
            properties: IndexMap::new(),
        };

        drs.add_referent(referent);
        assert_eq!(drs.referent_count(), 1);
        assert!(drs.get_referent(ReferentId(1)).is_some());
    }

    #[test]
    fn test_add_condition() {
        let mut drs = Drs::new(DrsId(1));
        drs.add_condition(DrsCondition::Predicate {
            name: "man".to_string(),
            referent: ReferentId(1),
        });
        assert_eq!(drs.condition_count(), 1);
    }

    #[test]
    fn test_find_referents_by_predicate() {
        let mut drs = Drs::new(DrsId(1));
        drs.add_condition(DrsCondition::Predicate {
            name: "man".to_string(),
            referent: ReferentId(1),
        });
        drs.add_condition(DrsCondition::Predicate {
            name: "walk".to_string(),
            referent: ReferentId(1),
        });
        drs.add_condition(DrsCondition::Predicate {
            name: "man".to_string(),
            referent: ReferentId(2),
        });

        let men = drs.find_referents_by_predicate("man");
        assert_eq!(men.len(), 2);
        assert!(men.contains(&ReferentId(1)));
        assert!(men.contains(&ReferentId(2)));
    }

    #[test]
    fn test_drs_merge() {
        let mut drs1 = Drs::new(DrsId(1));
        let referent1 = DiscourseReferent {
            id: ReferentId(1),
            name: Some("x".to_string()),
            referent_type: ReferentType::Individual,
            is_event: false,
            introduced_at: 0,
            properties: IndexMap::new(),
        };
        drs1.add_referent(referent1);
        drs1.add_condition(DrsCondition::Predicate {
            name: "man".to_string(),
            referent: ReferentId(1),
        });

        let mut drs2 = Drs::new(DrsId(2));
        let referent2 = DiscourseReferent {
            id: ReferentId(2),
            name: Some("y".to_string()),
            referent_type: ReferentType::Individual,
            is_event: false,
            introduced_at: 1,
            properties: IndexMap::new(),
        };
        drs2.add_referent(referent2);
        drs2.add_condition(DrsCondition::Predicate {
            name: "woman".to_string(),
            referent: ReferentId(2),
        });

        drs1.merge(drs2);
        assert_eq!(drs1.referent_count(), 2);
        assert_eq!(drs1.condition_count(), 2);
    }
}
