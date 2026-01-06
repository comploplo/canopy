//! Modal reasoning over possible worlds.
//!
//! Implements evaluation of modal operators (necessity/possibility) using
//! Kripke semantics with accessibility relations.
//!
//! # Modal Flavors
//!
//! - **Epistemic**: What is known (John knows that p)
//! - **Deontic**: What is obligatory/permitted (John must leave)
//! - **Circumstantial**: What is possible given circumstances (It can rain)
//! - **Bouletic**: What is desired (John wants p)

use crate::core::{ModalFlavor, ModalForce};
use crate::kernel::discourse::{AccessibilityRelation, AccessibilityType, ModalFrame, WorldId};
use std::collections::{HashMap, HashSet};

/// A possible world with its properties.
#[derive(Debug, Clone)]
pub struct World {
    /// Unique identifier.
    pub id: WorldId,
    /// Facts true in this world (DRS conditions as strings for simplicity).
    pub facts: HashSet<String>,
    /// Whether this is the actual world.
    pub is_actual: bool,
}

impl World {
    /// Create a new world.
    #[must_use]
    pub fn new(id: WorldId) -> Self {
        Self {
            id,
            facts: HashSet::new(),
            is_actual: id == WorldId::ACTUAL,
        }
    }

    /// Create the actual world.
    #[must_use]
    pub fn actual() -> Self {
        Self::new(WorldId::ACTUAL)
    }

    /// Add a fact to this world.
    pub fn add_fact(&mut self, fact: impl Into<String>) {
        self.facts.insert(fact.into());
    }

    /// Check if a fact holds in this world.
    #[must_use]
    pub fn has_fact(&self, fact: &str) -> bool {
        self.facts.contains(fact)
    }
}

/// Result of modal evaluation.
#[derive(Debug, Clone)]
pub struct ModalEvaluation {
    /// Whether the modal formula holds.
    pub holds: bool,
    /// The modal force (necessity/possibility).
    pub force: ModalForce,
    /// The modal flavor (epistemic/deontic/etc.).
    pub flavor: ModalFlavor,
    /// Worlds where the scope held (for possibility) or failed (for necessity).
    pub witness_worlds: Vec<WorldId>,
    /// Explanation of why it holds/fails.
    pub explanation: String,
}

/// Modal reasoner using Kripke semantics.
#[derive(Debug, Default)]
pub struct ModalReasoner {
    /// All worlds, indexed by ID.
    worlds: HashMap<WorldId, World>,
    /// Accessibility relations between worlds.
    accessibility: Vec<AccessibilityRelation>,
    /// Counter for generating new world IDs.
    next_world_id: u32,
}

impl ModalReasoner {
    /// Create a new modal reasoner with the actual world.
    #[must_use]
    pub fn new() -> Self {
        let mut reasoner = Self::default();
        reasoner.add_world(World::actual());
        reasoner.next_world_id = 1;
        reasoner
    }

    /// Add a world to the model.
    pub fn add_world(&mut self, world: World) {
        let id = world.id;
        self.worlds.insert(id, world);
    }

    /// Create a new world and return its ID.
    pub fn create_world(&mut self) -> WorldId {
        let id = WorldId(self.next_world_id);
        self.next_world_id += 1;
        self.add_world(World::new(id));
        id
    }

    /// Get a world by ID.
    #[must_use]
    pub fn get_world(&self, id: &WorldId) -> Option<&World> {
        self.worlds.get(id)
    }

    /// Get a mutable reference to a world.
    pub fn get_world_mut(&mut self, id: &WorldId) -> Option<&mut World> {
        self.worlds.get_mut(id)
    }

    /// Add an accessibility relation.
    pub fn add_accessibility(&mut self, relation: AccessibilityRelation) {
        self.accessibility.push(relation);
    }

    /// Make a world accessible from another with given relation type.
    pub fn make_accessible(
        &mut self,
        from: WorldId,
        to: WorldId,
        relation_type: AccessibilityType,
    ) {
        self.add_accessibility(AccessibilityRelation::new(from, to, relation_type));
    }

    /// Get all worlds accessible from a given world with a specific relation type.
    #[must_use]
    pub fn accessible_worlds(
        &self,
        from: &WorldId,
        relation_type: AccessibilityType,
    ) -> Vec<&World> {
        self.accessibility
            .iter()
            .filter(|r| r.from == *from && r.relation_type == relation_type)
            .filter_map(|r| self.worlds.get(&r.to))
            .collect()
    }

    /// Get all worlds accessible from a given world (any relation type).
    #[must_use]
    pub fn all_accessible_from(&self, from: &WorldId) -> Vec<&World> {
        self.accessibility
            .iter()
            .filter(|r| r.from == *from)
            .filter_map(|r| self.worlds.get(&r.to))
            .collect()
    }

    /// Evaluate a modal formula.
    ///
    /// # Arguments
    /// * `force` - Necessity (box) or Possibility (diamond)
    /// * `flavor` - The modal flavor (epistemic, deontic, etc.)
    /// * `scope_check` - Function that checks if the scope holds in a world
    /// * `eval_world` - World to evaluate from (defaults to actual)
    #[must_use]
    pub fn evaluate_modal<F>(
        &self,
        force: ModalForce,
        flavor: ModalFlavor,
        scope_check: F,
        eval_world: Option<&WorldId>,
    ) -> ModalEvaluation
    where
        F: Fn(&World) -> bool,
    {
        let world_id = eval_world.unwrap_or(&WorldId::ACTUAL);
        let relation_type = Self::flavor_to_accessibility(flavor);

        let accessible = self.accessible_worlds(world_id, relation_type);

        // Handle case with no accessible worlds
        if accessible.is_empty() {
            return ModalEvaluation {
                holds: force == ModalForce::Necessity,
                force,
                flavor,
                witness_worlds: vec![],
                explanation: format!("No {relation_type:?} accessible worlds from {world_id}"),
            };
        }

        let mut witness_worlds = Vec::new();

        match force {
            ModalForce::Necessity => {
                // Box: scope must hold in ALL accessible worlds
                for world in &accessible {
                    if !scope_check(world) {
                        witness_worlds.push(world.id);
                    }
                }

                let holds = witness_worlds.is_empty();
                let witness_count = witness_worlds.len();
                let explanation = if holds {
                    format!(
                        "Necessarily holds: scope true in all {} accessible worlds",
                        accessible.len()
                    )
                } else {
                    format!("Necessity fails: scope false in {witness_count} worlds")
                };

                ModalEvaluation {
                    holds,
                    force,
                    flavor,
                    witness_worlds,
                    explanation,
                }
            }
            ModalForce::Possibility => {
                // Diamond: scope must hold in SOME accessible world
                for world in &accessible {
                    if scope_check(world) {
                        witness_worlds.push(world.id);
                    }
                }

                let holds = !witness_worlds.is_empty();
                let witness_count = witness_worlds.len();
                let explanation = if holds {
                    format!("Possibly holds: scope true in {witness_count} worlds")
                } else {
                    format!(
                        "Possibility fails: scope false in all {} accessible worlds",
                        accessible.len()
                    )
                };

                ModalEvaluation {
                    holds,
                    force,
                    flavor,
                    witness_worlds,
                    explanation,
                }
            }
        }
    }

    /// Evaluate a simple modal with a fact check.
    #[must_use]
    pub fn evaluate_modal_fact(
        &self,
        force: ModalForce,
        flavor: ModalFlavor,
        fact: &str,
    ) -> ModalEvaluation {
        self.evaluate_modal(force, flavor, |world| world.has_fact(fact), None)
    }

    /// Map modal flavor to accessibility type.
    #[must_use]
    pub fn flavor_to_accessibility(flavor: ModalFlavor) -> AccessibilityType {
        match flavor {
            ModalFlavor::Epistemic => AccessibilityType::Epistemic,
            ModalFlavor::Deontic => AccessibilityType::Deontic,
            // Teleological uses circumstantial accessibility (goal-based reasoning)
            ModalFlavor::Circumstantial | ModalFlavor::Teleological => {
                AccessibilityType::Circumstantial
            }
            ModalFlavor::Bouletic => AccessibilityType::Bouletic,
        }
    }

    /// Get the count of worlds.
    #[must_use]
    pub fn world_count(&self) -> usize {
        self.worlds.len()
    }

    /// Get the count of accessibility relations.
    #[must_use]
    pub fn accessibility_count(&self) -> usize {
        self.accessibility.len()
    }

    /// Build a modal frame for the current state.
    #[must_use]
    pub fn to_modal_frame(&self, eval_world: WorldId) -> ModalFrame {
        let accessible: Vec<_> = self
            .accessibility
            .iter()
            .filter(|r| r.from == eval_world)
            .cloned()
            .collect();

        let mut frame = ModalFrame::at_actual();
        frame.evaluation_world = eval_world;
        frame.accessibility = accessible;
        frame
    }

    // ====================================================================
    // Counterfactual Reasoning (Lewis/Stalnaker semantics)
    // ====================================================================

    /// Evaluate a counterfactual conditional: "If A had been true, then B would be true"
    ///
    /// Uses Lewis's closest-world semantics:
    /// - Find worlds where antecedent A is true
    /// - Select the closest such worlds to the actual world
    /// - Check if consequent B holds in all/some of those closest worlds
    ///
    /// # Arguments
    /// * `antecedent_check` - Function checking if antecedent holds in a world
    /// * `consequent_check` - Function checking if consequent holds in a world
    /// * `modal` - The counterfactual modal (would/might/could)
    #[must_use]
    pub fn evaluate_counterfactual<A, C>(
        &self,
        antecedent_check: A,
        consequent_check: C,
        modal: CounterfactualModal,
    ) -> CounterfactualEvaluation
    where
        A: Fn(&World) -> bool,
        C: Fn(&World) -> bool,
    {
        // Find all worlds where antecedent holds
        let antecedent_worlds: Vec<&World> = self
            .worlds
            .values()
            .filter(|w| antecedent_check(w))
            .collect();

        if antecedent_worlds.is_empty() {
            // Vacuously true if no worlds satisfy antecedent
            return CounterfactualEvaluation {
                holds: true,
                modal,
                closest_worlds: vec![],
                explanation: "Vacuously true: no worlds satisfy antecedent".to_string(),
            };
        }

        // Select closest worlds (using similarity metric)
        let closest = self.select_closest_worlds(&antecedent_worlds);

        // Check consequent in closest worlds
        let result = match modal {
            CounterfactualModal::Would => {
                // "Would" requires consequent in ALL closest worlds
                let all_hold = closest.iter().all(|w| consequent_check(w));
                CounterfactualEvaluation {
                    holds: all_hold,
                    modal,
                    closest_worlds: closest.iter().map(|w| w.id).collect(),
                    explanation: if all_hold {
                        format!(
                            "Counterfactual holds: consequent true in all {} closest worlds",
                            closest.len()
                        )
                    } else {
                        "Counterfactual fails: consequent false in some closest worlds".to_string()
                    },
                }
            }
            CounterfactualModal::Might | CounterfactualModal::Could => {
                // "Might/Could" requires consequent in SOME closest world
                let any_holds = closest.iter().any(|w| consequent_check(w));
                CounterfactualEvaluation {
                    holds: any_holds,
                    modal,
                    closest_worlds: closest.iter().map(|w| w.id).collect(),
                    explanation: if any_holds {
                        "Counterfactual holds: consequent true in some closest world".to_string()
                    } else {
                        format!(
                            "Counterfactual fails: consequent false in all {} closest worlds",
                            closest.len()
                        )
                    },
                }
            }
        };

        result
    }

    /// Evaluate a counterfactual with simple fact checks.
    #[must_use]
    pub fn evaluate_counterfactual_facts(
        &self,
        antecedent_fact: &str,
        consequent_fact: &str,
        modal: CounterfactualModal,
    ) -> CounterfactualEvaluation {
        self.evaluate_counterfactual(
            |w| w.has_fact(antecedent_fact),
            |w| w.has_fact(consequent_fact),
            modal,
        )
    }

    /// Select the closest worlds from a set of candidate worlds.
    ///
    /// Uses a simple similarity metric based on shared facts with the actual world.
    fn select_closest_worlds<'a>(&'a self, candidates: &[&'a World]) -> Vec<&'a World> {
        let actual = self.worlds.get(&WorldId::ACTUAL);

        // If no actual world, return all candidates
        let Some(actual) = actual else {
            return candidates.to_vec();
        };

        // Calculate similarity scores
        let mut scored: Vec<_> = candidates
            .iter()
            .map(|w| {
                let similarity = Self::world_similarity(actual, w);
                (*w, similarity)
            })
            .collect();

        // Sort by similarity (highest first)
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Return all worlds with highest similarity (ties allowed)
        if scored.is_empty() {
            return vec![];
        }

        let max_similarity = scored[0].1;
        scored
            .into_iter()
            .take_while(|(_, sim)| (*sim - max_similarity).abs() < f64::EPSILON)
            .map(|(w, _)| w)
            .collect()
    }

    /// Calculate similarity between two worlds based on shared facts.
    ///
    /// Uses Jaccard similarity: |A ∩ B| / |A ∪ B|
    fn world_similarity(w1: &World, w2: &World) -> f64 {
        if w1.facts.is_empty() && w2.facts.is_empty() {
            return 1.0;
        }

        // Convert to u32 first for lossless f64 conversion (practical fact counts fit in u32)
        let shared = u32::try_from(w1.facts.intersection(&w2.facts).count()).unwrap_or(u32::MAX);
        let total = u32::try_from(w1.facts.union(&w2.facts).count()).unwrap_or(u32::MAX);

        if total == 0 {
            1.0
        } else {
            f64::from(shared) / f64::from(total)
        }
    }
}

/// Counterfactual modal operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CounterfactualModal {
    /// "would" - requires consequent in ALL closest worlds
    Would,
    /// "might" - requires consequent in SOME closest world
    Might,
    /// "could" - like might, some closest world
    Could,
}

/// Result of counterfactual evaluation.
#[derive(Debug, Clone)]
pub struct CounterfactualEvaluation {
    /// Whether the counterfactual holds.
    pub holds: bool,
    /// The modal used.
    pub modal: CounterfactualModal,
    /// The closest worlds selected.
    pub closest_worlds: Vec<WorldId>,
    /// Explanation of the evaluation.
    pub explanation: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_creation() {
        let world = World::actual();
        assert!(world.is_actual);
        assert_eq!(world.id, WorldId::ACTUAL);
    }

    #[test]
    fn test_world_facts() {
        let mut world = World::new(WorldId(1));
        world.add_fact("raining");
        world.add_fact("cold");

        assert!(world.has_fact("raining"));
        assert!(world.has_fact("cold"));
        assert!(!world.has_fact("sunny"));
    }

    #[test]
    fn test_reasoner_initialization() {
        let reasoner = ModalReasoner::new();
        assert_eq!(reasoner.world_count(), 1);
        assert!(reasoner.get_world(&WorldId::ACTUAL).is_some());
    }

    #[test]
    fn test_create_world() {
        let mut reasoner = ModalReasoner::new();
        let w1 = reasoner.create_world();
        let w2 = reasoner.create_world();

        assert_ne!(w1, w2);
        assert_eq!(reasoner.world_count(), 3); // actual + 2 new
    }

    #[test]
    fn test_accessibility() {
        let mut reasoner = ModalReasoner::new();
        let w1 = reasoner.create_world();
        let w2 = reasoner.create_world();

        reasoner.make_accessible(WorldId::ACTUAL, w1, AccessibilityType::Epistemic);
        reasoner.make_accessible(WorldId::ACTUAL, w2, AccessibilityType::Epistemic);
        reasoner.make_accessible(WorldId::ACTUAL, w1, AccessibilityType::Deontic);

        let epistemic_worlds =
            reasoner.accessible_worlds(&WorldId::ACTUAL, AccessibilityType::Epistemic);
        assert_eq!(epistemic_worlds.len(), 2);

        let deontic_worlds =
            reasoner.accessible_worlds(&WorldId::ACTUAL, AccessibilityType::Deontic);
        assert_eq!(deontic_worlds.len(), 1);
    }

    #[test]
    fn test_necessity_holds() {
        let mut reasoner = ModalReasoner::new();
        let w1 = reasoner.create_world();
        let w2 = reasoner.create_world();

        // Make both worlds accessible
        reasoner.make_accessible(WorldId::ACTUAL, w1, AccessibilityType::Epistemic);
        reasoner.make_accessible(WorldId::ACTUAL, w2, AccessibilityType::Epistemic);

        // Add same fact to both worlds
        reasoner.get_world_mut(&w1).unwrap().add_fact("p");
        reasoner.get_world_mut(&w2).unwrap().add_fact("p");

        // Necessity should hold
        let result =
            reasoner.evaluate_modal_fact(ModalForce::Necessity, ModalFlavor::Epistemic, "p");

        assert!(result.holds);
    }

    #[test]
    fn test_necessity_fails() {
        let mut reasoner = ModalReasoner::new();
        let w1 = reasoner.create_world();
        let w2 = reasoner.create_world();

        reasoner.make_accessible(WorldId::ACTUAL, w1, AccessibilityType::Epistemic);
        reasoner.make_accessible(WorldId::ACTUAL, w2, AccessibilityType::Epistemic);

        // Only add fact to one world
        reasoner.get_world_mut(&w1).unwrap().add_fact("p");
        // w2 doesn't have "p"

        // Necessity should fail
        let result =
            reasoner.evaluate_modal_fact(ModalForce::Necessity, ModalFlavor::Epistemic, "p");

        assert!(!result.holds);
        assert!(!result.witness_worlds.is_empty());
    }

    #[test]
    fn test_possibility_holds() {
        let mut reasoner = ModalReasoner::new();
        let w1 = reasoner.create_world();
        let w2 = reasoner.create_world();

        reasoner.make_accessible(WorldId::ACTUAL, w1, AccessibilityType::Circumstantial);
        reasoner.make_accessible(WorldId::ACTUAL, w2, AccessibilityType::Circumstantial);

        // Only add fact to one world
        reasoner.get_world_mut(&w1).unwrap().add_fact("raining");
        // w2 doesn't have "raining"

        // Possibility should hold (at least one world has the fact)
        let result = reasoner.evaluate_modal_fact(
            ModalForce::Possibility,
            ModalFlavor::Circumstantial,
            "raining",
        );

        assert!(result.holds);
    }

    #[test]
    fn test_possibility_fails() {
        let mut reasoner = ModalReasoner::new();
        let w1 = reasoner.create_world();
        let w2 = reasoner.create_world();

        reasoner.make_accessible(WorldId::ACTUAL, w1, AccessibilityType::Circumstantial);
        reasoner.make_accessible(WorldId::ACTUAL, w2, AccessibilityType::Circumstantial);

        // Neither world has the fact

        // Possibility should fail
        let result = reasoner.evaluate_modal_fact(
            ModalForce::Possibility,
            ModalFlavor::Circumstantial,
            "flying",
        );

        assert!(!result.holds);
    }

    #[test]
    fn test_no_accessible_worlds() {
        let reasoner = ModalReasoner::new();
        // No accessibility relations added

        // Necessity vacuously holds with no accessible worlds
        let result =
            reasoner.evaluate_modal_fact(ModalForce::Necessity, ModalFlavor::Epistemic, "p");
        assert!(result.holds);

        // Possibility fails with no accessible worlds
        let result =
            reasoner.evaluate_modal_fact(ModalForce::Possibility, ModalFlavor::Epistemic, "p");
        assert!(!result.holds);
    }

    #[test]
    fn test_deontic_obligation() {
        let mut reasoner = ModalReasoner::new();

        // Create obligation-fulfilling worlds
        let w1 = reasoner.create_world();
        let w2 = reasoner.create_world();

        reasoner.make_accessible(WorldId::ACTUAL, w1, AccessibilityType::Deontic);
        reasoner.make_accessible(WorldId::ACTUAL, w2, AccessibilityType::Deontic);

        // In both worlds, John leaves (fulfilling obligation)
        reasoner.get_world_mut(&w1).unwrap().add_fact("leave(john)");
        reasoner.get_world_mut(&w2).unwrap().add_fact("leave(john)");

        // "John must leave" - deontic necessity
        let result = reasoner.evaluate_modal_fact(
            ModalForce::Necessity,
            ModalFlavor::Deontic,
            "leave(john)",
        );

        assert!(result.holds);
    }

    #[test]
    fn test_flavor_to_accessibility() {
        assert_eq!(
            ModalReasoner::flavor_to_accessibility(ModalFlavor::Epistemic),
            AccessibilityType::Epistemic
        );
        assert_eq!(
            ModalReasoner::flavor_to_accessibility(ModalFlavor::Deontic),
            AccessibilityType::Deontic
        );
    }

    #[test]
    fn test_to_modal_frame() {
        let mut reasoner = ModalReasoner::new();
        let w1 = reasoner.create_world();

        reasoner.make_accessible(WorldId::ACTUAL, w1, AccessibilityType::Epistemic);

        let frame = reasoner.to_modal_frame(WorldId::ACTUAL);
        assert_eq!(frame.evaluation_world, WorldId::ACTUAL);
        assert_eq!(frame.accessibility.len(), 1);
    }

    // Counterfactual tests

    #[test]
    fn test_counterfactual_would_holds() {
        let mut reasoner = ModalReasoner::new();

        // Set up actual world
        reasoner
            .get_world_mut(&WorldId::ACTUAL)
            .unwrap()
            .add_fact("not_raining");
        reasoner
            .get_world_mut(&WorldId::ACTUAL)
            .unwrap()
            .add_fact("dry");

        // Create counterfactual world where it rains
        let w1 = reasoner.create_world();
        reasoner.get_world_mut(&w1).unwrap().add_fact("raining");
        reasoner.get_world_mut(&w1).unwrap().add_fact("wet"); // consequent

        // "If it had rained, it would be wet"
        let result =
            reasoner.evaluate_counterfactual_facts("raining", "wet", CounterfactualModal::Would);

        assert!(result.holds);
    }

    #[test]
    fn test_counterfactual_would_fails() {
        let mut reasoner = ModalReasoner::new();

        // Set up actual world
        reasoner
            .get_world_mut(&WorldId::ACTUAL)
            .unwrap()
            .add_fact("not_raining");

        // Create two counterfactual worlds
        let w1 = reasoner.create_world();
        let w2 = reasoner.create_world();

        // Both have antecedent, but only one has consequent
        reasoner.get_world_mut(&w1).unwrap().add_fact("raining");
        reasoner.get_world_mut(&w1).unwrap().add_fact("wet");

        reasoner.get_world_mut(&w2).unwrap().add_fact("raining");
        // w2 doesn't have "wet" - maybe it has good drainage

        // "If it had rained, it would be wet" - fails because w2 doesn't have consequent
        let result =
            reasoner.evaluate_counterfactual_facts("raining", "wet", CounterfactualModal::Would);

        // With equal similarity, both are closest, so "would" fails
        assert!(!result.holds);
    }

    #[test]
    fn test_counterfactual_might_holds() {
        let mut reasoner = ModalReasoner::new();

        // Set up actual world
        reasoner
            .get_world_mut(&WorldId::ACTUAL)
            .unwrap()
            .add_fact("not_raining");

        // Create two counterfactual worlds
        let w1 = reasoner.create_world();
        let w2 = reasoner.create_world();

        // Both have antecedent, but only one has consequent
        reasoner.get_world_mut(&w1).unwrap().add_fact("raining");
        reasoner.get_world_mut(&w1).unwrap().add_fact("wet");

        reasoner.get_world_mut(&w2).unwrap().add_fact("raining");
        // w2 doesn't have "wet"

        // "If it had rained, it might be wet" - holds because at least one world has consequent
        let result =
            reasoner.evaluate_counterfactual_facts("raining", "wet", CounterfactualModal::Might);

        assert!(result.holds);
    }

    #[test]
    fn test_counterfactual_vacuous_truth() {
        let reasoner = ModalReasoner::new();

        // No worlds have the antecedent
        // "If unicorns existed, they would be horses" - vacuously true
        let result =
            reasoner.evaluate_counterfactual_facts("unicorn", "horse", CounterfactualModal::Would);

        assert!(result.holds);
        assert!(result.closest_worlds.is_empty());
    }

    #[test]
    fn test_closest_world_selection() {
        let mut reasoner = ModalReasoner::new();

        // Set up actual world with some facts
        reasoner
            .get_world_mut(&WorldId::ACTUAL)
            .unwrap()
            .add_fact("a");
        reasoner
            .get_world_mut(&WorldId::ACTUAL)
            .unwrap()
            .add_fact("b");
        reasoner
            .get_world_mut(&WorldId::ACTUAL)
            .unwrap()
            .add_fact("c");

        // Create worlds with varying similarity
        let w1 = reasoner.create_world();
        let w2 = reasoner.create_world();

        // w1 shares more facts with actual (more similar)
        reasoner.get_world_mut(&w1).unwrap().add_fact("a");
        reasoner.get_world_mut(&w1).unwrap().add_fact("b");
        reasoner.get_world_mut(&w1).unwrap().add_fact("antecedent");
        reasoner.get_world_mut(&w1).unwrap().add_fact("consequent");

        // w2 shares fewer facts (less similar)
        reasoner.get_world_mut(&w2).unwrap().add_fact("antecedent");
        // w2 doesn't have consequent

        // "would" should select closest (w1) and find consequent true
        let result = reasoner.evaluate_counterfactual_facts(
            "antecedent",
            "consequent",
            CounterfactualModal::Would,
        );

        // w1 is closest and has consequent
        assert!(result.holds);
        assert_eq!(result.closest_worlds.len(), 1);
    }
}
