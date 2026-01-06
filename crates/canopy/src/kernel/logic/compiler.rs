//! DRS to constraint compilation.
//!
//! Compiles DRS structures into a normalized form suitable for logical reasoning.

use super::proof::ConditionRef;
use crate::core::ThetaRole;
use crate::kernel::discourse::{Drs, DrsCondition, ReferentId, TemporalRelationType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A compiled DRS in normalized form for reasoning.
#[derive(Debug, Clone, Default)]
pub struct CompiledDrs {
    /// Atomic facts (predicates and relations).
    pub facts: Vec<Fact>,
    /// Implications (conditionals and universals).
    pub implications: Vec<CompiledImplication>,
    /// Negated formulas.
    pub negations: Vec<NegatedFormula>,
    /// Disjunctions.
    pub disjunctions: Vec<CompiledDisjunction>,
    /// Equality constraints.
    pub equalities: Vec<(ReferentId, ReferentId)>,
    /// Temporal constraints.
    pub temporal_constraints: Vec<TemporalConstraint>,
    /// Event predicates with participants.
    pub event_predicates: Vec<EventFact>,
    /// Theta role assignments.
    pub theta_roles: Vec<ThetaRoleFact>,
}

impl CompiledDrs {
    /// Create a new empty compiled DRS.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Find facts matching a predicate.
    #[must_use]
    pub fn find_predicate(&self, predicate: &str) -> Vec<&Fact> {
        self.facts
            .iter()
            .filter(|f| f.predicate == predicate)
            .collect()
    }

    /// Find event facts matching a predicate.
    #[must_use]
    pub fn find_events(&self, predicate: &str) -> Vec<&EventFact> {
        self.event_predicates
            .iter()
            .filter(|e| e.predicate == predicate)
            .collect()
    }

    /// Find theta roles for an event.
    #[must_use]
    pub fn roles_for_event(&self, event_id: ReferentId) -> Vec<&ThetaRoleFact> {
        self.theta_roles
            .iter()
            .filter(|r| r.event_id == event_id)
            .collect()
    }

    /// Find the filler for a specific role in an event.
    #[must_use]
    pub fn filler_for_role(&self, event_id: ReferentId, role: ThetaRole) -> Option<ReferentId> {
        self.theta_roles
            .iter()
            .find(|r| r.event_id == event_id && r.role == role)
            .map(|r| r.filler)
    }

    /// Check if there's a negated predicate.
    #[must_use]
    pub fn has_negated(&self, predicate: &str) -> bool {
        self.negations
            .iter()
            .any(|n| n.inner.facts.iter().any(|f| f.predicate == predicate))
    }

    /// Get all predicates for a referent.
    #[must_use]
    pub fn predicates_for(&self, referent: ReferentId) -> Vec<&str> {
        self.facts
            .iter()
            .filter(|f| f.args.contains(&referent))
            .map(|f| f.predicate.as_str())
            .collect()
    }
}

/// An atomic fact.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Fact {
    /// The predicate name.
    pub predicate: String,
    /// Arguments (referent IDs).
    pub args: Vec<ReferentId>,
    /// Source reference for provenance.
    pub source: ConditionRef,
}

impl Fact {
    /// Create a unary fact.
    #[must_use]
    pub fn unary(predicate: impl Into<String>, arg: ReferentId, source: ConditionRef) -> Self {
        Self {
            predicate: predicate.into(),
            args: vec![arg],
            source,
        }
    }

    /// Create a binary fact.
    #[must_use]
    pub fn binary(
        predicate: impl Into<String>,
        arg1: ReferentId,
        arg2: ReferentId,
        source: ConditionRef,
    ) -> Self {
        Self {
            predicate: predicate.into(),
            args: vec![arg1, arg2],
            source,
        }
    }
}

/// An event fact with participants.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventFact {
    /// The event referent ID.
    pub event_id: ReferentId,
    /// The event predicate.
    pub predicate: String,
    /// Participants by role.
    pub participants: HashMap<ThetaRole, ReferentId>,
    /// Source reference.
    pub source: ConditionRef,
}

/// A theta role assignment fact.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThetaRoleFact {
    /// The event this role belongs to.
    pub event_id: ReferentId,
    /// The theta role.
    pub role: ThetaRole,
    /// The filler referent.
    pub filler: ReferentId,
    /// Source reference.
    pub source: ConditionRef,
}

/// A compiled implication.
#[derive(Debug, Clone)]
pub struct CompiledImplication {
    /// The antecedent.
    pub antecedent: CompiledDrs,
    /// The consequent.
    pub consequent: CompiledDrs,
    /// Source reference.
    pub source: ConditionRef,
}

/// A negated formula.
#[derive(Debug, Clone)]
pub struct NegatedFormula {
    /// The negated content.
    pub inner: CompiledDrs,
    /// Source reference.
    pub source: ConditionRef,
}

/// A compiled disjunction.
#[derive(Debug, Clone)]
pub struct CompiledDisjunction {
    /// Left disjunct.
    pub left: CompiledDrs,
    /// Right disjunct.
    pub right: CompiledDrs,
    /// Source reference.
    pub source: ConditionRef,
}

/// A temporal constraint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemporalConstraint {
    /// The temporal relation.
    pub relation: TemporalRelationType,
    /// First event.
    pub event1: ReferentId,
    /// Second event.
    pub event2: ReferentId,
    /// Source reference.
    pub source: ConditionRef,
}

/// Compile a DRS into normalized form.
#[must_use]
pub fn compile(drs: &Drs) -> CompiledDrs {
    compile_with_path(drs, &[], 0)
}

/// Create a condition reference based on path.
fn make_source(idx: usize, path: &[usize], sentence: usize) -> ConditionRef {
    if path.is_empty() {
        ConditionRef::main(idx, sentence)
    } else {
        ConditionRef::subordinate(idx, path.to_vec(), sentence)
    }
}

/// Make a sub-path by appending an index.
fn extend_path(path: &[usize], idx: usize) -> Vec<usize> {
    let mut sub_path = path.to_vec();
    sub_path.push(idx);
    sub_path
}

/// Merge subordinate DRS into the main compiled DRS.
fn merge_subordinate(compiled: &mut CompiledDrs, sub: CompiledDrs) {
    compiled.facts.extend(sub.facts);
    compiled.event_predicates.extend(sub.event_predicates);
    compiled.theta_roles.extend(sub.theta_roles);
    compiled.equalities.extend(sub.equalities);
    compiled
        .temporal_constraints
        .extend(sub.temporal_constraints);
    compiled.negations.extend(sub.negations);
    compiled.disjunctions.extend(sub.disjunctions);
    compiled.implications.extend(sub.implications);
}

/// Compile a DRS with a path for subordinate tracking.
fn compile_with_path(drs: &Drs, path: &[usize], sentence: usize) -> CompiledDrs {
    let mut compiled = CompiledDrs::new();

    for (idx, condition) in drs.conditions.iter().enumerate() {
        let source = make_source(idx, path, sentence);
        compile_condition(&mut compiled, condition, idx, path, sentence, source);
    }

    // Process subordinate DRSs
    for (sub_idx, subordinate) in drs.subordinates.iter().enumerate() {
        let sub_path = extend_path(path, drs.conditions.len() + sub_idx);
        let sub_compiled = compile_with_path(&subordinate.drs, &sub_path, sentence);
        merge_subordinate(&mut compiled, sub_compiled);
    }

    compiled
}

/// Compile a single condition into the compiled DRS.
fn compile_condition(
    compiled: &mut CompiledDrs,
    condition: &DrsCondition,
    idx: usize,
    path: &[usize],
    sentence: usize,
    source: ConditionRef,
) {
    match condition {
        DrsCondition::Predicate { name, referent } => {
            compiled.facts.push(Fact::unary(name, *referent, source));
        }
        DrsCondition::Relation { name, arg1, arg2 } => {
            compiled
                .facts
                .push(Fact::binary(name, *arg1, *arg2, source));
        }
        DrsCondition::EventPredicate {
            event_id,
            predicate,
            participants,
        } => {
            compiled.event_predicates.push(EventFact {
                event_id: *event_id,
                predicate: predicate.clone(),
                participants: participants.clone(),
                source,
            });
        }
        DrsCondition::ThetaRole {
            event_id,
            role,
            filler,
        } => {
            compiled.theta_roles.push(ThetaRoleFact {
                event_id: *event_id,
                role: *role,
                filler: *filler,
                source,
            });
        }
        DrsCondition::Equality { ref1, ref2 } => {
            compiled.equalities.push((*ref1, *ref2));
        }
        DrsCondition::Negation(inner) => {
            let sub_path = extend_path(path, idx);
            let inner_compiled = compile_with_path(inner, &sub_path, sentence);
            compiled.negations.push(NegatedFormula {
                inner: inner_compiled,
                source,
            });
        }
        DrsCondition::Disjunction(left, right) => {
            let sub_path = extend_path(path, idx);
            let left_compiled = compile_with_path(left, &sub_path, sentence);
            let right_compiled = compile_with_path(right, &sub_path, sentence);
            compiled.disjunctions.push(CompiledDisjunction {
                left: left_compiled,
                right: right_compiled,
                source,
            });
        }
        DrsCondition::Implication {
            antecedent,
            consequent,
        } => {
            let sub_path = extend_path(path, idx);
            let ant_compiled = compile_with_path(antecedent, &sub_path, sentence);
            let cons_compiled = compile_with_path(consequent, &sub_path, sentence);
            compiled.implications.push(CompiledImplication {
                antecedent: ant_compiled,
                consequent: cons_compiled,
                source,
            });
        }
        DrsCondition::TemporalRelation {
            relation,
            event1,
            event2,
        } => {
            compiled.temporal_constraints.push(TemporalConstraint {
                relation: *relation,
                event1: *event1,
                event2: *event2,
                source,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::discourse::DrsId;

    #[test]
    fn test_compile_empty_drs() {
        let drs = Drs::new(DrsId::new(0));
        let compiled = compile(&drs);

        assert!(compiled.facts.is_empty());
        assert!(compiled.event_predicates.is_empty());
        assert!(compiled.theta_roles.is_empty());
    }

    #[test]
    fn test_compile_predicate() {
        let mut drs = Drs::new(DrsId::new(0));
        drs.add_predicate("man", ReferentId::new(1));

        let compiled = compile(&drs);
        assert_eq!(compiled.facts.len(), 1);
        assert_eq!(compiled.facts[0].predicate, "man");
        assert_eq!(compiled.facts[0].args, vec![ReferentId::new(1)]);
    }

    #[test]
    fn test_compile_relation() {
        let mut drs = Drs::new(DrsId::new(0));
        drs.add_relation("owns", ReferentId::new(1), ReferentId::new(2));

        let compiled = compile(&drs);
        assert_eq!(compiled.facts.len(), 1);
        assert_eq!(compiled.facts[0].predicate, "owns");
        assert_eq!(
            compiled.facts[0].args,
            vec![ReferentId::new(1), ReferentId::new(2)]
        );
    }

    #[test]
    fn test_compile_theta_role() {
        let mut drs = Drs::new(DrsId::new(0));
        drs.add_theta_role(ReferentId::new(0), ThetaRole::Agent, ReferentId::new(1));

        let compiled = compile(&drs);
        assert_eq!(compiled.theta_roles.len(), 1);
        assert_eq!(compiled.theta_roles[0].event_id, ReferentId::new(0));
        assert_eq!(compiled.theta_roles[0].role, ThetaRole::Agent);
        assert_eq!(compiled.theta_roles[0].filler, ReferentId::new(1));
    }

    #[test]
    fn test_compile_event_predicate() {
        let mut drs = Drs::new(DrsId::new(0));
        let mut participants = HashMap::new();
        participants.insert(ThetaRole::Agent, ReferentId::new(1));
        drs.add_event_predicate(ReferentId::new(0), "leave", participants);

        let compiled = compile(&drs);
        assert_eq!(compiled.event_predicates.len(), 1);
        assert_eq!(compiled.event_predicates[0].predicate, "leave");
        assert_eq!(compiled.event_predicates[0].event_id, ReferentId::new(0));
    }

    #[test]
    fn test_compile_negation() {
        let mut drs = Drs::new(DrsId::new(0));
        let mut inner = Drs::new(DrsId::new(1));
        inner.add_predicate("leave", ReferentId::new(1));
        drs.add_condition(DrsCondition::Negation(Box::new(inner)));

        let compiled = compile(&drs);
        assert_eq!(compiled.negations.len(), 1);
        assert_eq!(compiled.negations[0].inner.facts.len(), 1);
        assert_eq!(compiled.negations[0].inner.facts[0].predicate, "leave");
    }

    #[test]
    fn test_compile_temporal() {
        let mut drs = Drs::new(DrsId::new(0));
        drs.add_condition(DrsCondition::TemporalRelation {
            relation: TemporalRelationType::Before,
            event1: ReferentId::new(0),
            event2: ReferentId::new(1),
        });

        let compiled = compile(&drs);
        assert_eq!(compiled.temporal_constraints.len(), 1);
        assert_eq!(
            compiled.temporal_constraints[0].relation,
            TemporalRelationType::Before
        );
    }

    #[test]
    fn test_find_predicate() {
        let mut drs = Drs::new(DrsId::new(0));
        drs.add_predicate("man", ReferentId::new(1));
        drs.add_predicate("man", ReferentId::new(2));
        drs.add_predicate("woman", ReferentId::new(3));

        let compiled = compile(&drs);
        let men = compiled.find_predicate("man");
        assert_eq!(men.len(), 2);

        let women = compiled.find_predicate("woman");
        assert_eq!(women.len(), 1);
    }

    #[test]
    fn test_roles_for_event() {
        let mut drs = Drs::new(DrsId::new(0));
        drs.add_theta_role(ReferentId::new(0), ThetaRole::Agent, ReferentId::new(1));
        drs.add_theta_role(ReferentId::new(0), ThetaRole::Theme, ReferentId::new(2));
        drs.add_theta_role(ReferentId::new(1), ThetaRole::Agent, ReferentId::new(3));

        let compiled = compile(&drs);
        let roles = compiled.roles_for_event(ReferentId::new(0));
        assert_eq!(roles.len(), 2);

        let filler = compiled.filler_for_role(ReferentId::new(0), ThetaRole::Agent);
        assert_eq!(filler, Some(ReferentId::new(1)));
    }
}
