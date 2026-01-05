//! Discourse Representation Structures (DRS).
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
//! [ x, e1, e2 |
//!   man(x),
//!   walk(e1),
//!   agent(e1, x),
//!   whistle(e2),
//!   agent(e2, x)
//! ]
//! ```

use super::referent::{DiscourseReferent, ReferentId};
use crate::core::ThetaRole;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique identifier for a DRS.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct DrsId(pub usize);

impl DrsId {
    /// Create a new DRS ID.
    #[must_use]
    pub const fn new(id: usize) -> Self {
        Self(id)
    }
}

impl std::fmt::Display for DrsId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "drs{}", self.0)
    }
}

/// A Discourse Representation Structure.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Drs {
    /// Unique identifier for this DRS.
    pub id: DrsId,

    /// Universe: the set of discourse referents.
    pub universe: HashMap<ReferentId, DiscourseReferent>,

    /// Conditions: predicates and relations over referents.
    pub conditions: Vec<DrsCondition>,

    /// Subordinate DRS structures (for conditionals, quantification, etc.).
    pub subordinates: Vec<SubordinateDrs>,
}

impl Default for Drs {
    fn default() -> Self {
        Self::new(DrsId::default())
    }
}

impl Drs {
    /// Create a new empty DRS.
    #[must_use]
    pub fn new(id: DrsId) -> Self {
        Self {
            id,
            universe: HashMap::new(),
            conditions: Vec::new(),
            subordinates: Vec::new(),
        }
    }

    /// Add a discourse referent to the universe.
    pub fn add_referent(&mut self, referent: DiscourseReferent) {
        self.universe.insert(referent.id, referent);
    }

    /// Add a condition to the DRS.
    pub fn add_condition(&mut self, condition: DrsCondition) {
        self.conditions.push(condition);
    }

    /// Add a unary predicate condition.
    pub fn add_predicate(&mut self, name: impl Into<String>, referent: ReferentId) {
        self.conditions.push(DrsCondition::Predicate {
            name: name.into(),
            referent,
        });
    }

    /// Add a binary relation condition.
    pub fn add_relation(&mut self, name: impl Into<String>, arg1: ReferentId, arg2: ReferentId) {
        self.conditions.push(DrsCondition::Relation {
            name: name.into(),
            arg1,
            arg2,
        });
    }

    /// Add a theta role condition.
    pub fn add_theta_role(&mut self, event_id: ReferentId, role: ThetaRole, filler: ReferentId) {
        self.conditions.push(DrsCondition::ThetaRole {
            event_id,
            role,
            filler,
        });
    }

    /// Add an event predicate condition.
    pub fn add_event_predicate(
        &mut self,
        event_id: ReferentId,
        predicate: impl Into<String>,
        participants: HashMap<ThetaRole, ReferentId>,
    ) {
        self.conditions.push(DrsCondition::EventPredicate {
            event_id,
            predicate: predicate.into(),
            participants,
        });
    }

    /// Add a subordinate DRS.
    pub fn add_subordinate(&mut self, relation: SubordinationRelation, drs: Drs) {
        self.subordinates.push(SubordinateDrs { relation, drs });
    }

    /// Get a referent by ID.
    #[must_use]
    pub fn get_referent(&self, id: ReferentId) -> Option<&DiscourseReferent> {
        self.universe.get(&id)
    }

    /// Find referents matching a predicate.
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

    /// Get all event referents.
    #[must_use]
    pub fn event_referents(&self) -> Vec<ReferentId> {
        self.universe
            .values()
            .filter(|r| r.is_event)
            .map(|r| r.id)
            .collect()
    }

    /// Get all entity (non-event) referents.
    #[must_use]
    pub fn entity_referents(&self) -> Vec<ReferentId> {
        self.universe
            .values()
            .filter(|r| !r.is_event)
            .map(|r| r.id)
            .collect()
    }

    /// Check if a referent is accessible from this DRS.
    #[must_use]
    pub fn is_accessible(&self, id: ReferentId) -> bool {
        if self.universe.contains_key(&id) {
            return true;
        }

        // Check subordinates (simplified accessibility)
        for sub in &self.subordinates {
            if sub.drs.is_accessible(id) {
                return true;
            }
        }

        false
    }

    /// Merge another DRS into this one.
    pub fn merge(&mut self, other: Drs) {
        for (id, referent) in other.universe {
            self.universe.insert(id, referent);
        }
        self.conditions.extend(other.conditions);
        self.subordinates.extend(other.subordinates);
    }

    /// Get the number of referents in the universe.
    #[must_use]
    pub fn referent_count(&self) -> usize {
        self.universe.len()
    }

    /// Get the number of conditions.
    #[must_use]
    pub fn condition_count(&self) -> usize {
        self.conditions.len()
    }

    /// Pretty print the DRS in box notation.
    #[must_use]
    pub fn to_box_notation(&self) -> String {
        use std::fmt::Write;
        let mut output = String::new();

        // Universe
        let refs: Vec<_> = self.universe.keys().map(|id| format!("{id}")).collect();
        let _ = writeln!(output, "[ {} |", refs.join(", "));

        // Conditions
        for cond in &self.conditions {
            let _ = writeln!(output, "  {cond},");
        }

        output.push(']');
        output
    }
}

/// Types of DRS conditions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DrsCondition {
    /// Unary predicate: P(x).
    Predicate { name: String, referent: ReferentId },

    /// Binary relation: R(x, y).
    Relation {
        name: String,
        arg1: ReferentId,
        arg2: ReferentId,
    },

    /// Event predicate with participants.
    EventPredicate {
        event_id: ReferentId,
        predicate: String,
        participants: HashMap<ThetaRole, ReferentId>,
    },

    /// Theta role assignment: role(e, x).
    ThetaRole {
        event_id: ReferentId,
        role: ThetaRole,
        filler: ReferentId,
    },

    /// Equality: x = y.
    Equality { ref1: ReferentId, ref2: ReferentId },

    /// Negation: NOT(DRS).
    Negation(Box<Drs>),

    /// Disjunction: DRS1 OR DRS2.
    Disjunction(Box<Drs>, Box<Drs>),

    /// Implication: DRS1 => DRS2 (for conditionals and universals).
    Implication {
        antecedent: Box<Drs>,
        consequent: Box<Drs>,
    },

    /// Temporal relation between events.
    TemporalRelation {
        relation: TemporalRelationType,
        event1: ReferentId,
        event2: ReferentId,
    },
}

impl std::fmt::Display for DrsCondition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DrsCondition::Predicate { name, referent } => {
                write!(f, "{name}({referent})")
            }
            DrsCondition::Relation { name, arg1, arg2 } => {
                write!(f, "{name}({arg1}, {arg2})")
            }
            DrsCondition::EventPredicate {
                event_id,
                predicate,
                participants,
            } => {
                let parts: Vec<_> = participants
                    .iter()
                    .map(|(role, id)| format!("{role:?}={id}"))
                    .collect();
                write!(f, "{predicate}({event_id})[{}]", parts.join(", "))
            }
            DrsCondition::ThetaRole {
                event_id,
                role,
                filler,
            } => {
                write!(f, "{role:?}({event_id}, {filler})")
            }
            DrsCondition::Equality { ref1, ref2 } => {
                write!(f, "{ref1} = {ref2}")
            }
            DrsCondition::Negation(_) => write!(f, "NOT(...)"),
            DrsCondition::Disjunction(_, _) => write!(f, "OR(...)"),
            DrsCondition::Implication { .. } => write!(f, "IF(...) THEN (...)"),
            DrsCondition::TemporalRelation {
                relation,
                event1,
                event2,
            } => {
                write!(f, "{relation:?}({event1}, {event2})")
            }
        }
    }
}

/// Types of temporal relations between events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TemporalRelationType {
    /// e1 happens before e2.
    Before,
    /// e1 happens after e2.
    After,
    /// e1 and e2 overlap.
    Overlaps,
    /// e1 contains e2.
    Contains,
    /// e1 is contained in e2.
    During,
    /// e1 and e2 are simultaneous.
    Simultaneous,
    /// e1 immediately precedes e2.
    Meets,
}

/// Subordinate DRS for embedded structures.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubordinateDrs {
    /// The type of subordination.
    pub relation: SubordinationRelation,
    /// The embedded DRS.
    pub drs: Drs,
}

/// Types of DRS subordination.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SubordinationRelation {
    /// Conditional antecedent.
    Antecedent,
    /// Conditional consequent.
    Consequent,
    /// Scope of negation.
    NegationScope,
    /// Scope of modal.
    ModalScope,
    /// Relative clause.
    RelativeClause,
    /// Complement clause.
    ComplementClause,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drs_creation() {
        let drs = Drs::new(DrsId::new(1));
        assert_eq!(drs.id, DrsId::new(1));
        assert!(drs.universe.is_empty());
        assert!(drs.conditions.is_empty());
    }

    #[test]
    fn test_add_referent() {
        let mut drs = Drs::new(DrsId::new(1));
        let referent = DiscourseReferent::entity(ReferentId::new(1), "man", 0);

        drs.add_referent(referent);
        assert_eq!(drs.referent_count(), 1);
        assert!(drs.get_referent(ReferentId::new(1)).is_some());
    }

    #[test]
    fn test_add_predicate() {
        let mut drs = Drs::new(DrsId::new(1));
        drs.add_predicate("man", ReferentId::new(1));
        assert_eq!(drs.condition_count(), 1);
    }

    #[test]
    fn test_find_referents_by_predicate() {
        let mut drs = Drs::new(DrsId::new(1));
        drs.add_predicate("man", ReferentId::new(1));
        drs.add_predicate("walk", ReferentId::new(1));
        drs.add_predicate("man", ReferentId::new(2));

        let men = drs.find_referents_by_predicate("man");
        assert_eq!(men.len(), 2);
    }

    #[test]
    fn test_drs_merge() {
        let mut drs1 = Drs::new(DrsId::new(1));
        drs1.add_referent(DiscourseReferent::entity(ReferentId::new(1), "x", 0));
        drs1.add_predicate("man", ReferentId::new(1));

        let mut drs2 = Drs::new(DrsId::new(2));
        drs2.add_referent(DiscourseReferent::entity(ReferentId::new(2), "y", 1));
        drs2.add_predicate("woman", ReferentId::new(2));

        drs1.merge(drs2);
        assert_eq!(drs1.referent_count(), 2);
        assert_eq!(drs1.condition_count(), 2);
    }

    #[test]
    fn test_accessibility() {
        let mut main_drs = Drs::new(DrsId::new(1));
        main_drs.add_referent(DiscourseReferent::entity(ReferentId::new(1), "John", 0));

        let mut sub_drs = Drs::new(DrsId::new(2));
        sub_drs.add_referent(DiscourseReferent::entity(ReferentId::new(2), "Mary", 0));

        main_drs.add_subordinate(SubordinationRelation::ComplementClause, sub_drs);

        assert!(main_drs.is_accessible(ReferentId::new(1)));
        assert!(main_drs.is_accessible(ReferentId::new(2)));
        assert!(!main_drs.is_accessible(ReferentId::new(99)));
    }

    #[test]
    fn test_theta_role_condition() {
        let mut drs = Drs::new(DrsId::new(1));
        drs.add_theta_role(
            ReferentId::new(0), // event
            ThetaRole::Agent,
            ReferentId::new(1), // filler
        );

        assert_eq!(drs.condition_count(), 1);
        match &drs.conditions[0] {
            DrsCondition::ThetaRole {
                event_id,
                role,
                filler,
            } => {
                assert_eq!(*event_id, ReferentId::new(0));
                assert_eq!(*role, ThetaRole::Agent);
                assert_eq!(*filler, ReferentId::new(1));
            }
            _ => panic!("Expected ThetaRole condition"),
        }
    }

    #[test]
    fn test_box_notation() {
        let mut drs = Drs::new(DrsId::new(1));
        drs.add_referent(DiscourseReferent::entity(ReferentId::new(0), "x", 0));
        drs.add_predicate("man", ReferentId::new(0));

        let notation = drs.to_box_notation();
        assert!(notation.contains("r0"));
        assert!(notation.contains("man(r0)"));
    }

    #[test]
    fn test_implication_drs() {
        let mut main_drs = Drs::new(DrsId::new(1));

        let mut antecedent = Drs::new(DrsId::new(2));
        antecedent.add_referent(DiscourseReferent::entity(ReferentId::new(1), "x", 0));
        antecedent.add_predicate("man", ReferentId::new(1));

        let mut consequent = Drs::new(DrsId::new(3));
        consequent.add_predicate("mortal", ReferentId::new(1));

        main_drs.add_condition(DrsCondition::Implication {
            antecedent: Box::new(antecedent),
            consequent: Box::new(consequent),
        });

        assert_eq!(main_drs.condition_count(), 1);
    }
}
