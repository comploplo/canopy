//! Reasoner trait for logical inference over DRS.
//!
//! Defines the interface for reasoning engines that can check consistency,
//! entailment, and answer queries.

use super::answer::QueryResult;
use super::proof::{ConditionRef, Explanation};
use super::query::{Proposition, Query};
use crate::kernel::discourse::{Drs, DrsCondition};
use serde::{Deserialize, Serialize};

/// Trait for logical reasoning over discourse representations.
pub trait Reasoner {
    /// Check if the DRS is internally consistent (no contradictions).
    fn check_consistent(&self, drs: &Drs) -> ConsistencyResult;

    /// Check if the DRS entails a proposition.
    fn entails(&self, drs: &Drs, proposition: &Proposition) -> EntailmentResult;

    /// Answer a query against the DRS, returning bindings.
    fn answer(&self, drs: &Drs, query: &Query) -> QueryResult;

    /// Check if adding new conditions would create a contradiction.
    fn would_contradict(&self, drs: &Drs, new_conditions: &[DrsCondition]) -> bool;
}

/// Result of consistency checking.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConsistencyResult {
    /// Whether the DRS is consistent.
    pub consistent: bool,
    /// Conflicts found (if inconsistent).
    pub conflicts: Vec<Conflict>,
    /// Explanation of the result.
    pub explanation: Option<Explanation>,
}

impl ConsistencyResult {
    /// Create a consistent result.
    #[must_use]
    pub fn consistent() -> Self {
        Self {
            consistent: true,
            conflicts: Vec::new(),
            explanation: None,
        }
    }

    /// Create an inconsistent result.
    #[must_use]
    pub fn inconsistent(conflicts: Vec<Conflict>) -> Self {
        Self {
            consistent: false,
            conflicts,
            explanation: None,
        }
    }

    /// Add an explanation.
    #[must_use]
    pub fn with_explanation(mut self, explanation: Explanation) -> Self {
        self.explanation = Some(explanation);
        self
    }
}

impl Default for ConsistencyResult {
    fn default() -> Self {
        Self::consistent()
    }
}

/// A conflict between conditions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Conflict {
    /// First conflicting condition.
    pub condition1: ConditionRef,
    /// Second conflicting condition.
    pub condition2: ConditionRef,
    /// Type of conflict.
    pub conflict_type: ConflictType,
    /// Description of the conflict.
    pub description: String,
}

impl Conflict {
    /// Create a new conflict.
    #[must_use]
    pub fn new(
        condition1: ConditionRef,
        condition2: ConditionRef,
        conflict_type: ConflictType,
        description: impl Into<String>,
    ) -> Self {
        Self {
            condition1,
            condition2,
            conflict_type,
            description: description.into(),
        }
    }

    /// Create a polarity conflict (P and ¬P).
    #[must_use]
    pub fn polarity(
        condition1: ConditionRef,
        condition2: ConditionRef,
        predicate: impl Into<String>,
    ) -> Self {
        let pred = predicate.into();
        Self::new(
            condition1,
            condition2,
            ConflictType::Polarity,
            format!("Conflicting polarity: {pred} and ¬{pred}"),
        )
    }

    /// Create a temporal conflict.
    #[must_use]
    pub fn temporal(condition1: ConditionRef, condition2: ConditionRef) -> Self {
        Self::new(
            condition1,
            condition2,
            ConflictType::Temporal,
            "Temporal ordering cycle detected",
        )
    }

    /// Create a temporal cycle conflict from Allen interval algebra.
    ///
    /// Used when the `TemporalReasoner` detects a cycle in temporal constraints.
    #[must_use]
    pub fn temporal_cycle(cycle: &[crate::kernel::discourse::ReferentId]) -> Self {
        let cycle_str: Vec<String> = cycle.iter().map(|r| format!("e{}", r.0)).collect();
        Self::new(
            ConditionRef::main(0, 0), // Placeholder - cycle spans multiple conditions
            ConditionRef::main(0, 0),
            ConflictType::Temporal,
            format!(
                "Temporal cycle detected: {} → {}",
                cycle_str.join(" → "),
                cycle_str.first().unwrap_or(&"?".to_string())
            ),
        )
    }

    /// Create a modal necessity failure conflict.
    ///
    /// Used when the `ModalReasoner` finds that a necessity (must/should) doesn't
    /// hold in all accessible worlds.
    #[must_use]
    pub fn modal_necessity_failure(
        flavor: crate::core::ModalFlavor,
        predicate: &str,
        failing_worlds: &[crate::kernel::discourse::WorldId],
    ) -> Self {
        let worlds_str: Vec<String> = failing_worlds.iter().map(|w| format!("w{}", w.0)).collect();
        Self::new(
            ConditionRef::main(0, 0), // Placeholder
            ConditionRef::main(0, 0),
            ConflictType::Modal,
            format!(
                "{:?} necessity fails for '{}': not true in worlds [{}]",
                flavor,
                predicate,
                worlds_str.join(", ")
            ),
        )
    }
}

/// Type of conflict between conditions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConflictType {
    /// Polarity conflict: P and ¬P.
    Polarity,
    /// Temporal ordering cycle.
    Temporal,
    /// Equality conflict: x = y and x ≠ y.
    Equality,
    /// Type conflict: incompatible predicates.
    Type,
    /// Modal conflict: necessity fails to hold in some accessible world.
    Modal,
}

/// Result of entailment checking.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntailmentResult {
    /// The entailment status.
    pub entailed: Entailment,
    /// Conditions that support this entailment.
    pub supporting_conditions: Vec<ConditionRef>,
    /// Explanation of the result.
    pub explanation: Option<Explanation>,
}

impl EntailmentResult {
    /// Create a "yes" entailment.
    #[must_use]
    pub fn yes(supporting: Vec<ConditionRef>) -> Self {
        Self {
            entailed: Entailment::Yes,
            supporting_conditions: supporting,
            explanation: None,
        }
    }

    /// Create a "no" entailment (contradicted).
    #[must_use]
    pub fn no(supporting: Vec<ConditionRef>) -> Self {
        Self {
            entailed: Entailment::No,
            supporting_conditions: supporting,
            explanation: None,
        }
    }

    /// Create an "unknown" entailment.
    #[must_use]
    pub fn unknown() -> Self {
        Self {
            entailed: Entailment::Unknown,
            supporting_conditions: Vec::new(),
            explanation: None,
        }
    }

    /// Create an "ambiguous" entailment (scope-dependent).
    #[must_use]
    pub fn ambiguous(results: Vec<bool>) -> Self {
        Self {
            entailed: Entailment::Ambiguous(results),
            supporting_conditions: Vec::new(),
            explanation: None,
        }
    }

    /// Add an explanation.
    #[must_use]
    pub fn with_explanation(mut self, explanation: Explanation) -> Self {
        self.explanation = Some(explanation);
        self
    }

    /// Check if definitely entailed.
    #[must_use]
    pub fn is_yes(&self) -> bool {
        matches!(self.entailed, Entailment::Yes)
    }

    /// Check if definitely not entailed.
    #[must_use]
    pub fn is_no(&self) -> bool {
        matches!(self.entailed, Entailment::No)
    }

    /// Check if unknown.
    #[must_use]
    pub fn is_unknown(&self) -> bool {
        matches!(self.entailed, Entailment::Unknown)
    }

    /// Check if ambiguous (scope-dependent).
    #[must_use]
    pub fn is_ambiguous(&self) -> bool {
        matches!(self.entailed, Entailment::Ambiguous(_))
    }
}

impl Default for EntailmentResult {
    fn default() -> Self {
        Self::unknown()
    }
}

/// Entailment status.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Entailment {
    /// Entailed in all scope readings.
    Yes,
    /// Contradicted (not entailed).
    No,
    /// Not enough information to determine.
    Unknown,
    /// Different answers per scope reading.
    Ambiguous(Vec<bool>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consistency_result() {
        let result = ConsistencyResult::consistent();
        assert!(result.consistent);
        assert!(result.conflicts.is_empty());

        let conflict =
            Conflict::polarity(ConditionRef::main(0, 0), ConditionRef::main(1, 1), "leave");
        let result = ConsistencyResult::inconsistent(vec![conflict]);
        assert!(!result.consistent);
        assert_eq!(result.conflicts.len(), 1);
    }

    #[test]
    fn test_entailment_result() {
        let result = EntailmentResult::yes(vec![ConditionRef::main(0, 0)]);
        assert!(result.is_yes());
        assert!(!result.is_no());
        assert!(!result.is_unknown());

        let result = EntailmentResult::no(vec![]);
        assert!(result.is_no());

        let result = EntailmentResult::unknown();
        assert!(result.is_unknown());

        let result = EntailmentResult::ambiguous(vec![true, false]);
        assert!(result.is_ambiguous());
    }

    #[test]
    fn test_conflict_creation() {
        let conflict =
            Conflict::polarity(ConditionRef::main(0, 0), ConditionRef::main(1, 1), "leave");
        assert_eq!(conflict.conflict_type, ConflictType::Polarity);
        assert!(conflict.description.contains("leave"));
    }
}
