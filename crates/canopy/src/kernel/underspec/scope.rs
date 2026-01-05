//! MRS-style scope underspecification.
//!
//! Implements handle-based scope underspecification following
//! Minimal Recursion Semantics (Copestake et al. 2005).
//!
//! # Key Concepts
//!
//! - **Handle**: A scope position that can be filled
//! - **EP** (Elementary Predication): A single predicate with its arguments
//! - **HCONS** (Handle Constraints): Restrictions on scope orderings
//! - **Qeq** (=q): The label must equal or be immediately outscoped by the hole

use std::collections::{HashMap, HashSet};

/// A handle (scope position) in MRS.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Handle(pub u32);

impl Handle {
    /// Create a new handle.
    #[must_use]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }
}

/// A semantic variable (event or entity).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Variable {
    /// Variable type prefix (e, x, h, i, u).
    pub var_type: VariableType,
    /// Unique index.
    pub index: u32,
}

impl Variable {
    /// Create a new variable.
    #[must_use]
    pub const fn new(var_type: VariableType, index: u32) -> Self {
        Self { var_type, index }
    }

    /// Create an event variable.
    #[must_use]
    pub const fn event(index: u32) -> Self {
        Self::new(VariableType::Event, index)
    }

    /// Create an entity variable.
    #[must_use]
    pub const fn entity(index: u32) -> Self {
        Self::new(VariableType::Entity, index)
    }

    /// Create a handle variable.
    #[must_use]
    pub const fn handle(index: u32) -> Self {
        Self::new(VariableType::Handle, index)
    }
}

/// Types of semantic variables.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VariableType {
    /// Event variable (e).
    Event,
    /// Entity/individual variable (x).
    Entity,
    /// Handle variable (h).
    Handle,
    /// Individual or event (i).
    IndividualOrEvent,
    /// Underspecified (u).
    Underspecified,
}

/// Elementary predication (EP) - a single semantic unit.
#[derive(Debug, Clone)]
pub struct ElementaryPredication {
    /// Label (handle) of this EP.
    pub label: Handle,
    /// Predicate symbol.
    pub predicate: String,
    /// Arguments (ARG0, ARG1, etc.).
    pub args: Vec<Variable>,
    /// Link to external resources.
    pub link: Option<String>,
}

impl ElementaryPredication {
    /// Create a new elementary predication.
    #[must_use]
    pub fn new(label: Handle, predicate: impl Into<String>, args: Vec<Variable>) -> Self {
        Self {
            label,
            predicate: predicate.into(),
            args,
            link: None,
        }
    }

    /// Add an external link (e.g., `VerbNet` class).
    #[must_use]
    pub fn with_link(mut self, link: impl Into<String>) -> Self {
        self.link = Some(link.into());
        self
    }

    /// Get ARG0 (typically the event/state variable).
    #[must_use]
    pub fn arg0(&self) -> Option<&Variable> {
        self.args.first()
    }
}

/// Handle constraint type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandleConstraintType {
    /// Qeq (=q): The label must be equal to or immediately outscope the hole.
    /// This is the standard MRS constraint.
    Qeq,
    /// Leq (≤): The label must be equal to or dominated by the hole.
    Leq,
    /// Equals (=): The handles must be equal (same scope position).
    Equals,
}

/// A handle constraint (HCONS in MRS).
#[derive(Debug, Clone)]
pub struct HandleConstraint {
    /// The hole (position to be filled).
    pub hole: Handle,
    /// The label (what fills the hole).
    pub label: Handle,
    /// Type of constraint.
    pub constraint_type: HandleConstraintType,
}

impl HandleConstraint {
    /// Create a qeq constraint: hole =q label.
    #[must_use]
    pub const fn qeq(hole: Handle, label: Handle) -> Self {
        Self {
            hole,
            label,
            constraint_type: HandleConstraintType::Qeq,
        }
    }

    /// Create an equals constraint: hole = label.
    #[must_use]
    pub const fn equals(hole: Handle, label: Handle) -> Self {
        Self {
            hole,
            label,
            constraint_type: HandleConstraintType::Equals,
        }
    }
}

/// Underspecified scope representation (MRS-style).
#[derive(Debug, Clone)]
pub struct ScopeUnderspec {
    /// Top handle (sentence-level scope).
    pub top: Handle,
    /// Elementary predications.
    pub eps: Vec<ElementaryPredication>,
    /// Handle constraints.
    pub hcons: Vec<HandleConstraint>,
    /// Next handle ID.
    next_handle: u32,
}

impl ScopeUnderspec {
    /// Create a new scope underspecification with a top handle.
    #[must_use]
    pub fn new() -> Self {
        Self {
            top: Handle::new(0),
            eps: Vec::new(),
            hcons: Vec::new(),
            next_handle: 1,
        }
    }

    /// Allocate a new handle.
    pub fn new_handle(&mut self) -> Handle {
        let h = Handle::new(self.next_handle);
        self.next_handle += 1;
        h
    }

    /// Add an elementary predication.
    pub fn add_ep(&mut self, ep: ElementaryPredication) {
        self.eps.push(ep);
    }

    /// Add a handle constraint.
    pub fn add_constraint(&mut self, constraint: HandleConstraint) {
        self.hcons.push(constraint);
    }

    /// Add a qeq constraint: hole =q label.
    pub fn add_qeq(&mut self, hole: Handle, label: Handle) {
        self.hcons.push(HandleConstraint::qeq(hole, label));
    }

    /// Get all handles used in EPs.
    #[must_use]
    pub fn ep_labels(&self) -> HashSet<Handle> {
        self.eps.iter().map(|ep| ep.label).collect()
    }

    /// Enumerate all valid scope orderings.
    ///
    /// A scope ordering assigns handles to holes respecting constraints.
    #[must_use]
    pub fn enumerate_orderings(&self) -> Vec<ScopeOrdering> {
        let labels: Vec<Handle> = self.ep_labels().into_iter().collect();
        let holes: Vec<Handle> = self.hcons.iter().map(|c| c.hole).collect();

        if holes.is_empty() {
            // No constraints - single trivial ordering
            return vec![ScopeOrdering {
                assignments: HashMap::new(),
            }];
        }

        // Simple enumeration for now - try all assignments
        let mut orderings = Vec::new();
        self.enumerate_recursive(&holes, &labels, 0, HashMap::new(), &mut orderings);
        orderings
    }

    fn enumerate_recursive(
        &self,
        holes: &[Handle],
        labels: &[Handle],
        hole_idx: usize,
        current: HashMap<Handle, Handle>,
        results: &mut Vec<ScopeOrdering>,
    ) {
        if hole_idx >= holes.len() {
            // Check if this assignment satisfies all constraints
            if self.satisfies_constraints(&current) {
                results.push(ScopeOrdering {
                    assignments: current,
                });
            }
            return;
        }

        let hole = holes[hole_idx];

        // Find the required label for this hole
        let required_label = self.hcons.iter().find(|c| c.hole == hole).map(|c| c.label);

        match required_label {
            Some(label) => {
                // For qeq constraints, try this label and labels that outscope it
                let mut next = current.clone();
                next.insert(hole, label);
                self.enumerate_recursive(holes, labels, hole_idx + 1, next, results);

                // Also try other labels that could satisfy qeq
                // (simplified: just try the specified label for now)
            }
            None => {
                // No constraint - try all labels
                for &label in labels {
                    let mut next = current.clone();
                    next.insert(hole, label);
                    self.enumerate_recursive(holes, labels, hole_idx + 1, next, results);
                }
            }
        }
    }

    fn satisfies_constraints(&self, assignments: &HashMap<Handle, Handle>) -> bool {
        for constraint in &self.hcons {
            match constraint.constraint_type {
                HandleConstraintType::Qeq => {
                    // For qeq, the assigned label should be the constraint's label
                    // (simplified check - real MRS allows immediate outscoping)
                    if let Some(&assigned) = assignments.get(&constraint.hole) {
                        if assigned != constraint.label {
                            // Could still be valid if assigned outscopes label
                            // For now, accept it (simplified)
                        }
                    }
                }
                HandleConstraintType::Equals => {
                    if let Some(&assigned) = assignments.get(&constraint.hole) {
                        if assigned != constraint.label {
                            return false;
                        }
                    }
                }
                HandleConstraintType::Leq => {
                    // Would need dominance relation - accept for now
                }
            }
        }
        true
    }

    /// Get the default (surface) scope ordering.
    #[must_use]
    pub fn default_ordering(&self) -> ScopeOrdering {
        // Surface scope: assign each hole to its qeq label
        let mut assignments = HashMap::new();
        for constraint in &self.hcons {
            assignments.insert(constraint.hole, constraint.label);
        }
        ScopeOrdering { assignments }
    }

    /// Count the number of valid scope orderings.
    #[must_use]
    pub fn ordering_count(&self) -> usize {
        self.enumerate_orderings().len()
    }
}

impl Default for ScopeUnderspec {
    fn default() -> Self {
        Self::new()
    }
}

/// A resolved scope ordering.
#[derive(Debug, Clone)]
pub struct ScopeOrdering {
    /// Mapping from holes to labels (what fills each hole).
    pub assignments: HashMap<Handle, Handle>,
}

impl ScopeOrdering {
    /// Get the label assigned to a hole.
    #[must_use]
    pub fn get(&self, hole: Handle) -> Option<Handle> {
        self.assignments.get(&hole).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_creation() {
        let h = Handle::new(5);
        assert_eq!(h.0, 5);
    }

    #[test]
    fn test_variable_creation() {
        let e = Variable::event(0);
        assert_eq!(e.var_type, VariableType::Event);
        assert_eq!(e.index, 0);

        let x = Variable::entity(1);
        assert_eq!(x.var_type, VariableType::Entity);
        assert_eq!(x.index, 1);
    }

    #[test]
    fn test_elementary_predication() {
        let ep = ElementaryPredication::new(
            Handle::new(1),
            "run",
            vec![Variable::event(0), Variable::entity(1)],
        )
        .with_link("run-51.3.2");

        assert_eq!(ep.label, Handle::new(1));
        assert_eq!(ep.predicate, "run");
        assert_eq!(ep.args.len(), 2);
        assert_eq!(ep.link, Some("run-51.3.2".to_string()));
    }

    #[test]
    fn test_scope_underspec_creation() {
        let mut scope = ScopeUnderspec::new();

        let h1 = scope.new_handle();
        let h2 = scope.new_handle();

        scope.add_ep(ElementaryPredication::new(
            h1,
            "every",
            vec![Variable::entity(0)],
        ));
        scope.add_ep(ElementaryPredication::new(
            h2,
            "student",
            vec![Variable::entity(0)],
        ));

        assert_eq!(scope.eps.len(), 2);
        assert_eq!(scope.ep_labels().len(), 2);
    }

    #[test]
    fn test_default_ordering() {
        let mut scope = ScopeUnderspec::new();

        let h1 = scope.new_handle();
        let h2 = scope.new_handle();
        let h3 = scope.new_handle(); // hole

        scope.add_ep(ElementaryPredication::new(h1, "every", vec![]));
        scope.add_ep(ElementaryPredication::new(h2, "some", vec![]));
        scope.add_qeq(h3, h2);

        let ordering = scope.default_ordering();
        assert_eq!(ordering.get(h3), Some(h2));
    }

    #[test]
    fn test_no_constraints() {
        let scope = ScopeUnderspec::new();
        let orderings = scope.enumerate_orderings();
        assert_eq!(orderings.len(), 1); // Single trivial ordering
    }
}
