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

    // === Subordination Tests ===

    #[test]
    fn test_add_subordinate_drs() {
        let mut main_drs = Drs::new(DrsId(1));

        // Add main referent (John)
        let john = DiscourseReferent {
            id: ReferentId(1),
            name: Some("John".to_string()),
            referent_type: ReferentType::Individual,
            is_event: false,
            introduced_at: 0,
            properties: IndexMap::new(),
        };
        main_drs.add_referent(john);

        // Create subordinate DRS for embedded clause "Mary left"
        let mut sub_drs = Drs::new(DrsId(2));
        let mary = DiscourseReferent {
            id: ReferentId(2),
            name: Some("Mary".to_string()),
            referent_type: ReferentType::Individual,
            is_event: false,
            introduced_at: 0,
            properties: IndexMap::new(),
        };
        sub_drs.add_referent(mary);
        sub_drs.add_condition(DrsCondition::Predicate {
            name: "leave".to_string(),
            referent: ReferentId(2),
        });

        // Add as complement clause
        main_drs.add_subordinate(SubordinationRelation::ComplementClause, sub_drs);

        assert_eq!(main_drs.subordinates.len(), 1);
        assert_eq!(
            main_drs.subordinates[0].relation,
            SubordinationRelation::ComplementClause
        );
        assert_eq!(main_drs.subordinates[0].drs.referent_count(), 1);
    }

    #[test]
    fn test_propositional_attitude_believe() {
        // "John believes that Mary left"
        let mut main_drs = Drs::new(DrsId(1));

        // John
        let john = DiscourseReferent {
            id: ReferentId(1),
            name: Some("John".to_string()),
            referent_type: ReferentType::Individual,
            is_event: false,
            introduced_at: 0,
            properties: IndexMap::new(),
        };
        main_drs.add_referent(john);

        // Create embedded DRS for belief content
        let mut belief_content = Drs::new(DrsId(2));
        let mary = DiscourseReferent {
            id: ReferentId(2),
            name: Some("Mary".to_string()),
            referent_type: ReferentType::Individual,
            is_event: false,
            introduced_at: 0,
            properties: IndexMap::new(),
        };
        belief_content.add_referent(mary);
        belief_content.add_condition(DrsCondition::Predicate {
            name: "leave".to_string(),
            referent: ReferentId(2),
        });

        // Add propositional attitude condition
        main_drs.add_condition(DrsCondition::PropositionalAttitude {
            attitude: AttitudeType::Believe,
            holder: ReferentId(1),
            content: Box::new(belief_content),
        });

        assert_eq!(main_drs.condition_count(), 1);
        match &main_drs.conditions[0] {
            DrsCondition::PropositionalAttitude {
                attitude,
                holder,
                content,
            } => {
                assert_eq!(*attitude, AttitudeType::Believe);
                assert_eq!(*holder, ReferentId(1));
                assert_eq!(content.referent_count(), 1);
            }
            _ => panic!("Expected PropositionalAttitude condition"),
        }
    }

    #[test]
    fn test_propositional_attitude_want() {
        // "Mary wants John to win"
        let mut main_drs = Drs::new(DrsId(1));

        let mary = DiscourseReferent {
            id: ReferentId(1),
            name: Some("Mary".to_string()),
            referent_type: ReferentType::Individual,
            is_event: false,
            introduced_at: 0,
            properties: IndexMap::new(),
        };
        main_drs.add_referent(mary);

        // Desire content: John wins
        let mut want_content = Drs::new(DrsId(2));
        let john = DiscourseReferent {
            id: ReferentId(2),
            name: Some("John".to_string()),
            referent_type: ReferentType::Individual,
            is_event: false,
            introduced_at: 0,
            properties: IndexMap::new(),
        };
        want_content.add_referent(john);
        want_content.add_condition(DrsCondition::Predicate {
            name: "win".to_string(),
            referent: ReferentId(2),
        });

        main_drs.add_condition(DrsCondition::PropositionalAttitude {
            attitude: AttitudeType::Want,
            holder: ReferentId(1),
            content: Box::new(want_content),
        });

        match &main_drs.conditions[0] {
            DrsCondition::PropositionalAttitude { attitude, .. } => {
                assert_eq!(*attitude, AttitudeType::Want);
            }
            _ => panic!("Expected PropositionalAttitude condition"),
        }
    }

    #[test]
    fn test_accessibility_in_subordinate_drs() {
        let mut main_drs = Drs::new(DrsId(1));

        // Add referent to main DRS
        let john = DiscourseReferent {
            id: ReferentId(1),
            name: Some("John".to_string()),
            referent_type: ReferentType::Individual,
            is_event: false,
            introduced_at: 0,
            properties: IndexMap::new(),
        };
        main_drs.add_referent(john);

        // Create subordinate with another referent
        let mut sub_drs = Drs::new(DrsId(2));
        let mary = DiscourseReferent {
            id: ReferentId(2),
            name: Some("Mary".to_string()),
            referent_type: ReferentType::Individual,
            is_event: false,
            introduced_at: 0,
            properties: IndexMap::new(),
        };
        sub_drs.add_referent(mary);

        main_drs.add_subordinate(SubordinationRelation::ComplementClause, sub_drs);

        // Main DRS referent is accessible
        assert!(main_drs.is_accessible(ReferentId(1)));
        // Subordinate referent is also accessible from main
        assert!(main_drs.is_accessible(ReferentId(2)));
        // Non-existent referent is not accessible
        assert!(!main_drs.is_accessible(ReferentId(99)));
    }

    #[test]
    fn test_nested_subordination() {
        // "John said that Mary thinks that Bill left"
        let mut main_drs = Drs::new(DrsId(1));

        // John
        let john = DiscourseReferent {
            id: ReferentId(1),
            name: Some("John".to_string()),
            referent_type: ReferentType::Individual,
            is_event: false,
            introduced_at: 0,
            properties: IndexMap::new(),
        };
        main_drs.add_referent(john);

        // Level 1: Mary's thinking
        let mut mary_drs = Drs::new(DrsId(2));
        let mary = DiscourseReferent {
            id: ReferentId(2),
            name: Some("Mary".to_string()),
            referent_type: ReferentType::Individual,
            is_event: false,
            introduced_at: 0,
            properties: IndexMap::new(),
        };
        mary_drs.add_referent(mary);

        // Level 2: Bill's leaving (nested inside Mary's thinking)
        let mut bill_drs = Drs::new(DrsId(3));
        let bill = DiscourseReferent {
            id: ReferentId(3),
            name: Some("Bill".to_string()),
            referent_type: ReferentType::Individual,
            is_event: false,
            introduced_at: 0,
            properties: IndexMap::new(),
        };
        bill_drs.add_referent(bill);
        bill_drs.add_condition(DrsCondition::Predicate {
            name: "leave".to_string(),
            referent: ReferentId(3),
        });

        // Mary thinks [Bill left]
        mary_drs.add_condition(DrsCondition::PropositionalAttitude {
            attitude: AttitudeType::Think,
            holder: ReferentId(2),
            content: Box::new(bill_drs),
        });

        // John said [Mary thinks [Bill left]]
        main_drs.add_condition(DrsCondition::PropositionalAttitude {
            attitude: AttitudeType::Say,
            holder: ReferentId(1),
            content: Box::new(mary_drs),
        });

        // All referents accessible from main
        assert!(main_drs.is_accessible(ReferentId(1))); // John in main
                                                        // Note: referents inside PropositionalAttitude content aren't in main universe
                                                        // They're in the embedded DRS structure within the condition
    }

    #[test]
    fn test_implication_drs_conditional() {
        // "If a man owns a donkey, he beats it"
        let mut main_drs = Drs::new(DrsId(1));

        // Antecedent: a man owns a donkey
        let mut antecedent = Drs::new(DrsId(2));
        let man = DiscourseReferent {
            id: ReferentId(1),
            name: Some("x".to_string()),
            referent_type: ReferentType::Individual,
            is_event: false,
            introduced_at: 0,
            properties: IndexMap::new(),
        };
        let donkey = DiscourseReferent {
            id: ReferentId(2),
            name: Some("y".to_string()),
            referent_type: ReferentType::Individual,
            is_event: false,
            introduced_at: 0,
            properties: IndexMap::new(),
        };
        antecedent.add_referent(man);
        antecedent.add_referent(donkey);
        antecedent.add_condition(DrsCondition::Predicate {
            name: "man".to_string(),
            referent: ReferentId(1),
        });
        antecedent.add_condition(DrsCondition::Predicate {
            name: "donkey".to_string(),
            referent: ReferentId(2),
        });
        antecedent.add_condition(DrsCondition::Relation {
            name: "own".to_string(),
            arg1: ReferentId(1),
            arg2: ReferentId(2),
        });

        // Consequent: he beats it
        let mut consequent = Drs::new(DrsId(3));
        consequent.add_condition(DrsCondition::Relation {
            name: "beat".to_string(),
            arg1: ReferentId(1), // he -> man
            arg2: ReferentId(2), // it -> donkey
        });

        // Add implication
        main_drs.add_condition(DrsCondition::Implication {
            antecedent: Box::new(antecedent),
            consequent: Box::new(consequent),
        });

        assert_eq!(main_drs.condition_count(), 1);
        match &main_drs.conditions[0] {
            DrsCondition::Implication {
                antecedent,
                consequent,
            } => {
                assert_eq!(antecedent.referent_count(), 2);
                assert_eq!(antecedent.condition_count(), 3);
                assert_eq!(consequent.condition_count(), 1);
            }
            _ => panic!("Expected Implication condition"),
        }
    }

    #[test]
    fn test_negation_drs() {
        // "John did not leave"
        let mut main_drs = Drs::new(DrsId(1));

        let john = DiscourseReferent {
            id: ReferentId(1),
            name: Some("John".to_string()),
            referent_type: ReferentType::Individual,
            is_event: false,
            introduced_at: 0,
            properties: IndexMap::new(),
        };
        main_drs.add_referent(john);

        // Negated content
        let mut negated = Drs::new(DrsId(2));
        let event = DiscourseReferent {
            id: ReferentId(2),
            name: None,
            referent_type: ReferentType::Event,
            is_event: true,
            introduced_at: 0,
            properties: IndexMap::new(),
        };
        negated.add_referent(event);
        negated.add_condition(DrsCondition::Predicate {
            name: "leave".to_string(),
            referent: ReferentId(2),
        });

        main_drs.add_condition(DrsCondition::Negation(Box::new(negated)));

        match &main_drs.conditions[0] {
            DrsCondition::Negation(neg_drs) => {
                assert_eq!(neg_drs.condition_count(), 1);
            }
            _ => panic!("Expected Negation condition"),
        }
    }

    #[test]
    fn test_drs_merge_with_subordinates() {
        let mut drs1 = Drs::new(DrsId(1));
        let ref1 = DiscourseReferent {
            id: ReferentId(1),
            name: Some("x".to_string()),
            referent_type: ReferentType::Individual,
            is_event: false,
            introduced_at: 0,
            properties: IndexMap::new(),
        };
        drs1.add_referent(ref1);

        // Add subordinate to drs1
        let mut sub1 = Drs::new(DrsId(10));
        sub1.add_condition(DrsCondition::Predicate {
            name: "test".to_string(),
            referent: ReferentId(1),
        });
        drs1.add_subordinate(SubordinationRelation::RelativeClause, sub1);

        let mut drs2 = Drs::new(DrsId(2));
        let ref2 = DiscourseReferent {
            id: ReferentId(2),
            name: Some("y".to_string()),
            referent_type: ReferentType::Individual,
            is_event: false,
            introduced_at: 1,
            properties: IndexMap::new(),
        };
        drs2.add_referent(ref2);

        // Add subordinate to drs2
        let mut sub2 = Drs::new(DrsId(11));
        sub2.add_condition(DrsCondition::Predicate {
            name: "test2".to_string(),
            referent: ReferentId(2),
        });
        drs2.add_subordinate(SubordinationRelation::ModalScope, sub2);

        drs1.merge(drs2);

        // Both referents merged
        assert_eq!(drs1.referent_count(), 2);
        // Both subordinates merged
        assert_eq!(drs1.subordinates.len(), 2);
        assert_eq!(
            drs1.subordinates[0].relation,
            SubordinationRelation::RelativeClause
        );
        assert_eq!(
            drs1.subordinates[1].relation,
            SubordinationRelation::ModalScope
        );
    }
}
