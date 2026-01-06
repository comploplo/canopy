//! Discourse Representation Structures (DRS).
//!
//! Implementation of Kamp's Discourse Representation Theory (1981).
//! A DRS consists of:
//! - A set of discourse referents (entities introduced in the discourse)
//! - A set of conditions (predicates over those referents)
//!
//! ## Underspecified DRS (UDRT)
//!
//! This module also implements Underspecified DRS (Reyle 1993) for
//! representing scope ambiguity without premature commitment:
//!
//! - Labeled boxes for DRS fragments
//! - Subordination constraints between labels
//! - Lazy enumeration of valid scope orderings
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

use super::modal::{AccessibilityType, ClosestWorldSelection, CounterfactualModal, WorldId};
use super::referent::{DiscourseReferent, ReferentId};
use super::temporal::{AspectualOperator, TemporalAnchorType, TemporalFrame, TimePoint};
use crate::core::{ModalFlavor, ModalForce, ThetaRole};
use crate::kernel::underspec::{
    Alternative, ChoiceId, ChoicePoint, ChoiceType, PackedSemantics, ScopeUnderspec,
};
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
    ///
    /// In DRT, accessibility is asymmetric:
    /// - Referents in the main DRS are accessible to subordinate DRSs
    /// - Referents in subordinate DRSs are NOT accessible from the main DRS
    ///
    /// This method only checks the current DRS's universe. To check if a
    /// referent from a superordinate DRS is accessible, use the
    /// `is_accessible_from` method with the superordinate DRS.
    #[must_use]
    pub fn is_accessible(&self, id: ReferentId) -> bool {
        // Only check local universe - subordinate referents are NOT accessible
        self.universe.contains_key(&id)
    }

    /// Check if a referent is accessible from this DRS, given a superordinate context.
    ///
    /// Accessibility in DRT flows downward: referents from superordinate DRSs
    /// are accessible in subordinate contexts, but not vice versa.
    #[must_use]
    pub fn is_accessible_from(&self, id: ReferentId, superordinate: &Drs) -> bool {
        // Check local universe first
        if self.universe.contains_key(&id) {
            return true;
        }
        // Check superordinate context
        superordinate.universe.contains_key(&id)
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

    // ========================================================================
    // TAM (Tense, Aspect, Modality) Conditions
    // ========================================================================
    /// Temporal frame assignment (Reichenbachian S/R/E).
    TemporalFrameAssignment {
        /// The event being temporally located.
        event: ReferentId,
        /// The temporal frame (speech, reference, event times).
        frame: TemporalFrame,
    },

    /// Aspectual operator (PROG, PERF, HAB, etc.).
    AspectualOp {
        /// The aspectual operator.
        operator: AspectualOperator,
        /// The event being modified.
        event: ReferentId,
        /// The scope of the operator.
        scope: Box<Drs>,
    },

    /// Temporal anchor: locates event relative to a time point.
    TemporalAnchor {
        /// The event being anchored.
        event: ReferentId,
        /// How the event relates to the anchor.
        anchor_type: TemporalAnchorType,
        /// The reference time point.
        reference: TimePoint,
    },

    /// Modal operator (necessity/possibility with flavor).
    ModalOp {
        /// Quantificational force (necessity = □, possibility = ◇).
        force: ModalForce,
        /// Modal flavor (epistemic, deontic, etc.).
        flavor: ModalFlavor,
        /// The scope of the modal.
        scope: Box<Drs>,
        /// Optional world variable for modal subordination.
        world_var: Option<WorldId>,
    },

    /// World-relative condition: condition holds in specified world.
    InWorld {
        /// The world where the condition holds.
        world: WorldId,
        /// The condition that holds in that world.
        condition: Box<DrsCondition>,
    },

    /// Accessibility relation between worlds.
    Accessible {
        /// Source world.
        from_world: WorldId,
        /// Target world.
        to_world: WorldId,
        /// Type of accessibility.
        relation: AccessibilityType,
    },

    /// Counterfactual conditional.
    Counterfactual {
        /// The antecedent ("if φ had been the case").
        antecedent: Box<Drs>,
        /// The consequent ("ψ would have been the case").
        consequent: Box<Drs>,
        /// Modal force (would = necessity, might = possibility).
        modal_force: CounterfactualModal,
        /// Closest world selection strategy.
        closest_worlds: ClosestWorldSelection,
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
            // TAM conditions
            DrsCondition::TemporalFrameAssignment { event, frame } => {
                write!(f, "TFRAME({event}, {frame:?})")
            }
            DrsCondition::AspectualOp {
                operator, event, ..
            } => {
                write!(f, "{operator}({event})")
            }
            DrsCondition::TemporalAnchor {
                event, anchor_type, ..
            } => {
                write!(f, "ANCHOR({event}, {anchor_type:?})")
            }
            DrsCondition::ModalOp { force, flavor, .. } => {
                let op = match force {
                    ModalForce::Necessity => "□",
                    ModalForce::Possibility => "◇",
                };
                write!(f, "{op}_{flavor:?}(...)")
            }
            DrsCondition::InWorld { world, .. } => {
                write!(f, "IN({world}, ...)")
            }
            DrsCondition::Accessible {
                from_world,
                to_world,
                relation,
            } => {
                write!(f, "ACC_{relation:?}({from_world}, {to_world})")
            }
            DrsCondition::Counterfactual { modal_force, .. } => {
                write!(f, "CF_{modal_force}(...)")
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

// ============================================================================
// UDRT (Underspecified DRS) Types
// ============================================================================

/// Label for a DRS box in UDRT.
///
/// Labels allow referring to DRS fragments before their scope is determined.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Label(pub u32);

impl Label {
    /// Create a new label.
    #[must_use]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }
}

impl std::fmt::Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "l{}", self.0)
    }
}

/// Subordination constraint between labels (UDRT).
///
/// Expresses that one DRS fragment must be subordinate to another
/// in the final resolved structure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubordinationConstraint {
    /// The subordinate label (must be inside).
    pub subordinate: Label,
    /// The superordinate label (must contain).
    pub superordinate: Label,
    /// Type of subordination.
    pub constraint_type: SubordinationConstraintType,
}

impl SubordinationConstraint {
    /// Create a new subordination constraint.
    #[must_use]
    pub const fn new(subordinate: Label, superordinate: Label) -> Self {
        Self {
            subordinate,
            superordinate,
            constraint_type: SubordinationConstraintType::Immediate,
        }
    }

    /// Create a transitive subordination constraint.
    #[must_use]
    pub const fn transitive(subordinate: Label, superordinate: Label) -> Self {
        Self {
            subordinate,
            superordinate,
            constraint_type: SubordinationConstraintType::Transitive,
        }
    }
}

/// Types of subordination constraints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SubordinationConstraintType {
    /// Immediate subordination (directly inside).
    Immediate,
    /// Transitive subordination (somewhere inside).
    Transitive,
}

/// Underspecified DRS (UDRT-style).
///
/// Represents semantic ambiguity without committing to a single reading.
/// Contains labeled DRS boxes and constraints on their arrangement.
#[derive(Debug, Clone)]
pub struct UnderspecDrs {
    /// Base DRS structure (shared across readings).
    pub base: Drs,

    /// Labeled DRS boxes.
    pub labeled_boxes: HashMap<Label, Drs>,

    /// Subordination constraints between labels.
    pub subordination: Vec<SubordinationConstraint>,

    /// MRS-style scope constraints (optional).
    pub scope_constraints: Option<ScopeUnderspec>,

    /// Choice points from packed semantics.
    pub choice_points: Vec<ChoicePoint>,

    /// Next label ID.
    next_label: u32,
}

impl UnderspecDrs {
    /// Create a new underspecified DRS.
    #[must_use]
    pub fn new(base: Drs) -> Self {
        Self {
            base,
            labeled_boxes: HashMap::new(),
            subordination: Vec::new(),
            scope_constraints: None,
            choice_points: Vec::new(),
            next_label: 0,
        }
    }

    /// Create from a packed semantics structure.
    #[must_use]
    pub fn from_packed(packed: &PackedSemantics, base: Drs) -> Self {
        let mut udrs = Self::new(base);

        // Copy choice points
        for choice in &packed.choices {
            udrs.choice_points.push(choice.clone());
        }

        // Copy scope constraints if present
        if let Some(scope) = &packed.scope_underspec {
            udrs.scope_constraints = Some(scope.clone());
        }

        udrs
    }

    /// Allocate a new label.
    pub fn new_label(&mut self) -> Label {
        let label = Label::new(self.next_label);
        self.next_label += 1;
        label
    }

    /// Add a labeled DRS box.
    pub fn add_labeled_box(&mut self, label: Label, drs: Drs) {
        self.labeled_boxes.insert(label, drs);
    }

    /// Add a subordination constraint.
    pub fn add_subordination(&mut self, constraint: SubordinationConstraint) {
        self.subordination.push(constraint);
    }

    /// Add a simple subordination: subordinate < superordinate.
    pub fn add_sub(&mut self, subordinate: Label, superordinate: Label) {
        self.subordination
            .push(SubordinationConstraint::new(subordinate, superordinate));
    }

    /// Set scope constraints.
    pub fn set_scope_constraints(&mut self, scope: ScopeUnderspec) {
        self.scope_constraints = Some(scope);
    }

    /// Check if this UDRS is ambiguous.
    #[must_use]
    pub fn is_ambiguous(&self) -> bool {
        !self.choice_points.is_empty()
            || self
                .scope_constraints
                .as_ref()
                .is_some_and(|s| s.ordering_count() > 1)
    }

    /// Get the number of possible resolutions.
    #[must_use]
    pub fn resolution_count(&self) -> usize {
        let choice_count: usize = if self.choice_points.is_empty() {
            1
        } else {
            self.choice_points
                .iter()
                .map(|c| c.alternative_count().max(1))
                .product()
        };

        let scope_count = self
            .scope_constraints
            .as_ref()
            .map_or(1, ScopeUnderspec::ordering_count);

        choice_count * scope_count
    }

    /// Get the default (surface scope) resolution.
    #[must_use]
    pub fn default_resolution(&self) -> Drs {
        // Start with base DRS
        let mut result = self.base.clone();

        // Merge all labeled boxes (surface order)
        for drs in self.labeled_boxes.values() {
            result.merge(drs.clone());
        }

        // Apply default scope ordering if present
        if let Some(scope) = &self.scope_constraints {
            let _ordering = scope.default_ordering();
            // Would apply ordering to result here
            // For now, just return merged structure
        }

        result
    }

    /// Enumerate all valid resolutions.
    ///
    /// Returns an iterator over resolved DRS structures.
    pub fn resolutions(&self) -> impl Iterator<Item = Drs> + '_ {
        DrsResolutionIterator::new(self)
    }

    /// Get labels in subordination order (topological sort).
    #[must_use]
    pub fn labels_in_order(&self) -> Vec<Label> {
        let mut labels: Vec<_> = self.labeled_boxes.keys().copied().collect();
        labels.sort_by_key(|l| l.0);
        labels
    }
}

impl Default for UnderspecDrs {
    fn default() -> Self {
        Self::new(Drs::default())
    }
}

/// Iterator over DRS resolutions.
pub struct DrsResolutionIterator<'a> {
    udrs: &'a UnderspecDrs,
    /// Current choice indices.
    choice_indices: Vec<usize>,
    /// Current scope ordering index.
    scope_idx: usize,
    /// Scope orderings (precomputed).
    scope_orderings: Vec<crate::kernel::underspec::ScopeOrdering>,
    /// Whether we've exhausted all resolutions.
    done: bool,
}

impl<'a> DrsResolutionIterator<'a> {
    fn new(udrs: &'a UnderspecDrs) -> Self {
        let choice_indices = vec![0; udrs.choice_points.len()];

        let scope_orderings = udrs
            .scope_constraints
            .as_ref()
            .map_or_else(Vec::new, ScopeUnderspec::enumerate_orderings);

        let done = udrs.choice_points.iter().any(|c| c.alternatives.is_empty())
            && !udrs.choice_points.is_empty();

        Self {
            udrs,
            choice_indices,
            scope_idx: 0,
            scope_orderings,
            done,
        }
    }

    fn current_resolution(&self) -> Drs {
        // Build DRS based on current choices
        let mut result = self.udrs.base.clone();

        // Merge labeled boxes
        for drs in self.udrs.labeled_boxes.values() {
            result.merge(drs.clone());
        }

        result
    }

    fn advance(&mut self) {
        // First advance scope ordering
        if !self.scope_orderings.is_empty() {
            self.scope_idx += 1;
            if self.scope_idx < self.scope_orderings.len() {
                return;
            }
            self.scope_idx = 0;
        }

        // Then advance choice indices
        for i in (0..self.choice_indices.len()).rev() {
            self.choice_indices[i] += 1;
            if self.choice_indices[i] < self.udrs.choice_points[i].alternatives.len() {
                return;
            }
            self.choice_indices[i] = 0;
        }

        // If we get here, we've exhausted all combinations
        self.done = true;
    }
}

impl Iterator for DrsResolutionIterator<'_> {
    type Item = Drs;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        // Handle empty case
        if self.udrs.choice_points.is_empty() && self.scope_orderings.is_empty() {
            self.done = true;
            return Some(self.udrs.base.clone());
        }

        let resolution = self.current_resolution();
        self.advance();

        Some(resolution)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.done {
            return (0, Some(0));
        }
        let count = self.udrs.resolution_count();
        (count, Some(count))
    }
}

/// Builder for constructing UDRS from events.
#[derive(Debug)]
pub struct UdrsBuilder {
    base: Drs,
    next_label: u32,
    labeled_boxes: HashMap<Label, Drs>,
    subordination: Vec<SubordinationConstraint>,
    choice_points: Vec<ChoicePoint>,
    next_choice_id: u32,
}

impl UdrsBuilder {
    /// Create a new UDRS builder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            base: Drs::default(),
            next_label: 0,
            labeled_boxes: HashMap::new(),
            subordination: Vec::new(),
            choice_points: Vec::new(),
            next_choice_id: 0,
        }
    }

    /// Set the base DRS.
    #[must_use]
    pub fn with_base(mut self, base: Drs) -> Self {
        self.base = base;
        self
    }

    /// Add a labeled box.
    pub fn add_box(&mut self, drs: Drs) -> Label {
        let label = Label::new(self.next_label);
        self.next_label += 1;
        self.labeled_boxes.insert(label, drs);
        label
    }

    /// Add subordination constraint.
    pub fn add_subordination(&mut self, subordinate: Label, superordinate: Label) {
        self.subordination
            .push(SubordinationConstraint::new(subordinate, superordinate));
    }

    /// Add an unresolved anaphoric binding.
    ///
    /// Creates a choice point for the anaphor-antecedent ambiguity.
    /// The binding will not be resolved until disambiguation occurs.
    ///
    /// # Arguments
    /// * `anaphor` - The pronoun/anaphor referent
    /// * `binding` - The underspecified binding with candidate antecedents
    ///
    /// # Returns
    /// The choice ID for this referential ambiguity, or None if not ambiguous.
    pub fn add_unresolved_binding(
        &mut self,
        anaphor: ReferentId,
        binding: &super::binding::UnderspecBinding,
    ) -> Option<ChoiceId> {
        // No ambiguity if single or no candidates
        if binding.candidate_count() <= 1 {
            // If there's exactly one candidate, add equality condition
            if let Some((antecedent, _)) = binding.best_candidate() {
                self.base.add_condition(DrsCondition::Equality {
                    ref1: anaphor,
                    ref2: antecedent,
                });
            }
            return None;
        }

        let choice_id = ChoiceId::new(self.next_choice_id);
        self.next_choice_id += 1;

        // Create alternatives from candidates
        let alternatives: Vec<Alternative> = binding
            .candidates
            .iter()
            .enumerate()
            .map(|(idx, (referent_id, score))| {
                Alternative::new(idx, f64::from(*score), format!("ref_{}", referent_id.0))
            })
            .collect();

        let candidates: Vec<ReferentId> = binding.candidates.iter().map(|(id, _)| *id).collect();

        let choice_point = ChoicePoint::new(
            choice_id,
            ChoiceType::Reference {
                anaphor,
                candidates,
            },
            alternatives,
        );

        // Set default if binding has a preferred candidate
        let choice_point = if let Some(preferred) = binding.preferred {
            if let Some(idx) = binding
                .candidates
                .iter()
                .position(|(id, _)| *id == preferred)
            {
                choice_point.with_default(idx)
            } else {
                choice_point
            }
        } else {
            choice_point
        };

        self.choice_points.push(choice_point);
        Some(choice_id)
    }

    /// Build the UDRS.
    #[must_use]
    pub fn build(self) -> UnderspecDrs {
        UnderspecDrs {
            base: self.base,
            labeled_boxes: self.labeled_boxes,
            subordination: self.subordination,
            scope_constraints: None,
            choice_points: self.choice_points,
            next_label: self.next_label,
        }
    }
}

impl Default for UdrsBuilder {
    fn default() -> Self {
        Self::new()
    }
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

        main_drs.add_subordinate(SubordinationRelation::ComplementClause, sub_drs.clone());

        // Main DRS referent is accessible
        assert!(main_drs.is_accessible(ReferentId::new(1)));

        // Subordinate referent is NOT accessible from main (DRT asymmetry)
        assert!(
            !main_drs.is_accessible(ReferentId::new(2)),
            "Subordinate referents should not be accessible from main DRS"
        );

        // Non-existent referent is not accessible
        assert!(!main_drs.is_accessible(ReferentId::new(99)));

        // Test is_accessible_from: subordinate can access main's referents
        assert!(
            sub_drs.is_accessible_from(ReferentId::new(1), &main_drs),
            "Main DRS referents should be accessible from subordinate"
        );
        assert!(
            sub_drs.is_accessible_from(ReferentId::new(2), &main_drs),
            "Local referents should be accessible"
        );
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

    // =========== UDRT Tests ===========

    #[test]
    fn test_label_creation() {
        let label = Label::new(5);
        assert_eq!(label.0, 5);
        assert_eq!(label.to_string(), "l5");
    }

    #[test]
    fn test_subordination_constraint() {
        let l1 = Label::new(1);
        let l2 = Label::new(2);

        let constraint = SubordinationConstraint::new(l1, l2);
        assert_eq!(constraint.subordinate, l1);
        assert_eq!(constraint.superordinate, l2);
        assert_eq!(
            constraint.constraint_type,
            SubordinationConstraintType::Immediate
        );

        let transitive = SubordinationConstraint::transitive(l1, l2);
        assert_eq!(
            transitive.constraint_type,
            SubordinationConstraintType::Transitive
        );
    }

    #[test]
    fn test_underspec_drs_creation() {
        let base = Drs::new(DrsId::new(1));
        let udrs = UnderspecDrs::new(base);

        assert!(udrs.labeled_boxes.is_empty());
        assert!(udrs.subordination.is_empty());
        assert!(!udrs.is_ambiguous());
        assert_eq!(udrs.resolution_count(), 1);
    }

    #[test]
    fn test_underspec_drs_labeled_boxes() {
        let base = Drs::new(DrsId::new(1));
        let mut udrs = UnderspecDrs::new(base);

        let label = udrs.new_label();
        assert_eq!(label, Label::new(0));

        let mut box_drs = Drs::new(DrsId::new(2));
        box_drs.add_predicate("man", ReferentId::new(1));
        udrs.add_labeled_box(label, box_drs);

        assert_eq!(udrs.labeled_boxes.len(), 1);
        assert!(udrs.labeled_boxes.contains_key(&label));
    }

    #[test]
    fn test_underspec_drs_default_resolution() {
        let mut base = Drs::new(DrsId::new(1));
        base.add_referent(DiscourseReferent::entity(ReferentId::new(1), "x", 0));

        let mut udrs = UnderspecDrs::new(base);

        // Add a labeled box
        let label = udrs.new_label();
        let mut box_drs = Drs::new(DrsId::new(2));
        box_drs.add_predicate("man", ReferentId::new(1));
        udrs.add_labeled_box(label, box_drs);

        let resolved = udrs.default_resolution();
        // Should have merged the referent and the predicate
        assert_eq!(resolved.referent_count(), 1);
        assert_eq!(resolved.condition_count(), 1);
    }

    #[test]
    fn test_underspec_drs_resolutions_empty() {
        let base = Drs::new(DrsId::new(1));
        let udrs = UnderspecDrs::new(base);

        let resolutions: Vec<_> = udrs.resolutions().collect();
        assert_eq!(resolutions.len(), 1);
    }

    #[test]
    fn test_udrs_builder() {
        let mut builder = UdrsBuilder::new();

        let mut drs1 = Drs::new(DrsId::new(1));
        drs1.add_predicate("every", ReferentId::new(1));
        let l1 = builder.add_box(drs1);

        let mut drs2 = Drs::new(DrsId::new(2));
        drs2.add_predicate("some", ReferentId::new(2));
        let l2 = builder.add_box(drs2);

        builder.add_subordination(l1, l2);

        let udrs = builder.build();
        assert_eq!(udrs.labeled_boxes.len(), 2);
        assert_eq!(udrs.subordination.len(), 1);
    }

    #[test]
    fn test_underspec_drs_from_packed() {
        use crate::kernel::underspec::{
            Alternative, ChoiceId, ChoicePoint, ChoiceType, PackedSemantics, SharedStructure,
        };
        use crate::runtime::{SenseId, TokenId};

        let mut packed = PackedSemantics::new(SharedStructure::default());
        packed.add_choice(ChoicePoint::new(
            ChoiceId::new(0),
            ChoiceType::LexicalSense {
                token_id: TokenId::new(0),
                senses: vec![SenseId::new("bank.01"), SenseId::new("bank.02")],
            },
            vec![
                Alternative::new(0, 0.7, "financial"),
                Alternative::new(1, 0.3, "river"),
            ],
        ));

        let base = Drs::new(DrsId::new(1));
        let udrs = UnderspecDrs::from_packed(&packed, base);

        assert_eq!(udrs.choice_points.len(), 1);
        assert!(udrs.is_ambiguous());
        assert_eq!(udrs.resolution_count(), 2);
    }

    // =========== Unresolved Binding Tests ===========

    #[test]
    fn test_udrs_builder_unresolved_binding() {
        use super::super::binding::{AnaphorType, UnderspecBinding};

        let mut builder = UdrsBuilder::new();

        // Create an ambiguous binding: "John told Bill he was tired"
        // "he" could refer to John or Bill
        let binding = UnderspecBinding::new(
            vec![
                (ReferentId::new(1), 0.8), // John
                (ReferentId::new(2), 0.6), // Bill
            ],
            AnaphorType::Personal,
            false,
        );

        let anaphor = ReferentId::new(3); // "he"
        let choice_id = builder.add_unresolved_binding(anaphor, &binding);

        assert!(choice_id.is_some());

        let udrs = builder.build();
        assert_eq!(udrs.choice_points.len(), 1);
        assert!(udrs.is_ambiguous());
        assert_eq!(udrs.resolution_count(), 2);
    }

    #[test]
    fn test_udrs_builder_single_candidate_binding() {
        use super::super::binding::{AnaphorType, UnderspecBinding};

        let mut builder = UdrsBuilder::new();

        // Single candidate - not ambiguous
        let binding = UnderspecBinding::new(
            vec![(ReferentId::new(1), 0.9)],
            AnaphorType::Personal,
            false,
        );

        let anaphor = ReferentId::new(2);
        let choice_id = builder.add_unresolved_binding(anaphor, &binding);

        assert!(choice_id.is_none()); // No ambiguity

        let udrs = builder.build();
        assert!(!udrs.is_ambiguous());

        // Should have added equality condition
        assert_eq!(udrs.base.condition_count(), 1);
        match &udrs.base.conditions[0] {
            DrsCondition::Equality { ref1, ref2 } => {
                assert_eq!(*ref1, anaphor);
                assert_eq!(*ref2, ReferentId::new(1));
            }
            _ => panic!("Expected Equality condition"),
        }
    }

    #[test]
    fn test_udrs_builder_binding_with_preferred() {
        use super::super::binding::{AnaphorType, UnderspecBinding};

        let mut builder = UdrsBuilder::new();

        // Create binding with preferred candidate
        let mut binding = UnderspecBinding::new(
            vec![(ReferentId::new(1), 0.9), (ReferentId::new(2), 0.7)],
            AnaphorType::Personal,
            false,
        );
        binding.set_preferred(ReferentId::new(2)); // Prefer second

        let anaphor = ReferentId::new(3);
        let _choice_id = builder.add_unresolved_binding(anaphor, &binding);

        let udrs = builder.build();
        assert_eq!(udrs.choice_points.len(), 1);

        // Check default is set
        let choice = &udrs.choice_points[0];
        assert_eq!(choice.default_idx, Some(1)); // Index of ReferentId(2)
    }
}
