//! Modal semantics and counterfactual reasoning.
//!
//! Implements possible world semantics following Kratzer (1981, 1991) and
//! counterfactual conditionals following Lewis (1973) and Stalnaker (1968).
//!
//! ## Modal Semantics
//!
//! Modality is analyzed in terms of quantification over possible worlds:
//! - Necessity (□): true in ALL accessible worlds
//! - Possibility (◇): true in SOME accessible world
//!
//! The accessibility relation determines the modal flavor:
//! - Epistemic: worlds compatible with what is known
//! - Deontic: worlds where obligations are fulfilled
//! - Circumstantial: worlds compatible with circumstances
//!
//! ## Counterfactual Semantics
//!
//! "If φ had been the case, ψ would have been the case"
//!
//! Evaluated by finding the closest worlds where φ holds and checking
//! whether ψ holds in those worlds (Lewis's similarity semantics).

use super::drs::Drs;
use crate::core::{ModalFlavor, ModalForce};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

// ============================================================================
// World Identifiers
// ============================================================================

/// Identifier for a possible world.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct WorldId(pub u32);

impl WorldId {
    /// The actual world (w₀).
    pub const ACTUAL: WorldId = WorldId(0);

    /// Create a new world ID.
    #[must_use]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    /// Check if this is the actual world.
    #[must_use]
    pub const fn is_actual(&self) -> bool {
        self.0 == 0
    }
}

impl std::fmt::Display for WorldId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_actual() {
            write!(f, "w₀")
        } else {
            write!(f, "w{}", self.0)
        }
    }
}

// ============================================================================
// Accessibility Relations
// ============================================================================

/// Accessibility relation between worlds.
///
/// An accessibility relation R(w, w') means w' is accessible from w
/// relative to some modal base (epistemic, deontic, etc.).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AccessibilityRelation {
    /// Source world.
    pub from: WorldId,
    /// Target world.
    pub to: WorldId,
    /// Type of accessibility.
    pub relation_type: AccessibilityType,
}

impl AccessibilityRelation {
    /// Create a new accessibility relation.
    #[must_use]
    pub const fn new(from: WorldId, to: WorldId, relation_type: AccessibilityType) -> Self {
        Self {
            from,
            to,
            relation_type,
        }
    }
}

/// Types of accessibility between worlds.
///
/// Each type corresponds to a different modal flavor (Kratzer's
/// conversational backgrounds).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AccessibilityType {
    /// Epistemic: worlds compatible with what is known.
    /// "John must be home" (given what we know).
    Epistemic,

    /// Deontic: worlds where obligations are fulfilled.
    /// "John must pay" (given the rules).
    Deontic,

    /// Circumstantial: worlds compatible with circumstances.
    /// "John can swim" (given his abilities).
    Circumstantial,

    /// Bouletic: worlds where desires are satisfied.
    /// "John wants to leave" (his desired worlds).
    Bouletic,

    /// Teleological: worlds where goals are achieved.
    /// "To win, John must train" (given his goals).
    Teleological,

    /// Similarity: ordering for counterfactuals (Lewis).
    /// "If John had left..." (closest worlds).
    Similarity,
}

impl From<ModalFlavor> for AccessibilityType {
    fn from(flavor: ModalFlavor) -> Self {
        match flavor {
            ModalFlavor::Epistemic => Self::Epistemic,
            ModalFlavor::Deontic => Self::Deontic,
            ModalFlavor::Circumstantial => Self::Circumstantial,
            ModalFlavor::Bouletic => Self::Bouletic,
            ModalFlavor::Teleological => Self::Teleological,
        }
    }
}

// ============================================================================
// World Sets
// ============================================================================

/// A set of possible worlds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorldSet {
    /// The worlds in this set.
    pub worlds: HashSet<WorldId>,
    /// Description of how this set was selected.
    pub description: WorldDescription,
}

impl WorldSet {
    /// Create an empty world set.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            worlds: HashSet::new(),
            description: WorldDescription::Enumerated,
        }
    }

    /// Create a singleton world set.
    #[must_use]
    pub fn singleton(world: WorldId) -> Self {
        let mut worlds = HashSet::new();
        worlds.insert(world);
        Self {
            worlds,
            description: WorldDescription::Enumerated,
        }
    }

    /// Create the set of all accessible worlds.
    #[must_use]
    pub fn all_accessible() -> Self {
        Self {
            worlds: HashSet::new(),
            description: WorldDescription::AllAccessible,
        }
    }

    /// Create the set of some accessible worlds.
    #[must_use]
    pub fn some_accessible() -> Self {
        Self {
            worlds: HashSet::new(),
            description: WorldDescription::SomeAccessible,
        }
    }

    /// Check if this set is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.worlds.is_empty()
    }

    /// Check if this set contains a world.
    #[must_use]
    pub fn contains(&self, world: &WorldId) -> bool {
        self.worlds.contains(world)
    }

    /// Add a world to this set.
    pub fn insert(&mut self, world: WorldId) {
        self.worlds.insert(world);
    }
}

impl Default for WorldSet {
    fn default() -> Self {
        Self::singleton(WorldId::ACTUAL)
    }
}

/// Description of how a world set was selected.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WorldDescription {
    /// All accessible worlds (universal quantification).
    AllAccessible,
    /// Some accessible worlds (existential quantification).
    SomeAccessible,
    /// Closest worlds satisfying a condition (counterfactual).
    Closest {
        /// The condition that must hold in these worlds.
        condition_description: String,
    },
    /// Explicitly enumerated worlds.
    Enumerated,
}

// ============================================================================
// Modal Frame
// ============================================================================

/// Modal frame for evaluating modal statements.
///
/// A modal frame specifies:
/// - The evaluation world (typically the actual world)
/// - Accessibility relations to other worlds
/// - Conversational backgrounds (Kratzer's modal base and ordering source)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModalFrame {
    /// The world from which we evaluate.
    pub evaluation_world: WorldId,
    /// Accessibility relations from the evaluation world.
    pub accessibility: Vec<AccessibilityRelation>,
    /// Modal base (restricts which worlds are relevant).
    pub modal_base: ConversationalBackground,
    /// Ordering source (orders the relevant worlds).
    pub ordering_source: ConversationalBackground,
}

impl ModalFrame {
    /// Create a new modal frame at the actual world.
    #[must_use]
    pub fn at_actual() -> Self {
        Self {
            evaluation_world: WorldId::ACTUAL,
            accessibility: Vec::new(),
            modal_base: ConversationalBackground::empty(BackgroundKind::Circumstantial),
            ordering_source: ConversationalBackground::empty(BackgroundKind::Stereotypical),
        }
    }

    /// Create an epistemic modal frame.
    #[must_use]
    pub fn epistemic() -> Self {
        Self {
            evaluation_world: WorldId::ACTUAL,
            accessibility: Vec::new(),
            modal_base: ConversationalBackground::empty(BackgroundKind::Epistemic),
            ordering_source: ConversationalBackground::empty(BackgroundKind::Stereotypical),
        }
    }

    /// Create a deontic modal frame.
    #[must_use]
    pub fn deontic() -> Self {
        Self {
            evaluation_world: WorldId::ACTUAL,
            accessibility: Vec::new(),
            modal_base: ConversationalBackground::empty(BackgroundKind::Circumstantial),
            ordering_source: ConversationalBackground::empty(BackgroundKind::Deontic),
        }
    }

    /// Add an accessibility relation.
    pub fn add_accessibility(&mut self, to: WorldId, relation_type: AccessibilityType) {
        self.accessibility.push(AccessibilityRelation::new(
            self.evaluation_world,
            to,
            relation_type,
        ));
    }

    /// Get worlds accessible via a given relation type.
    #[must_use]
    pub fn get_accessible(&self, relation_type: AccessibilityType) -> Vec<WorldId> {
        self.accessibility
            .iter()
            .filter(|r| r.relation_type == relation_type && r.from == self.evaluation_world)
            .map(|r| r.to)
            .collect()
    }
}

impl Default for ModalFrame {
    fn default() -> Self {
        Self::at_actual()
    }
}

// ============================================================================
// Conversational Background (Kratzer)
// ============================================================================

/// Conversational background (Kratzer 1981, 1991).
///
/// A function from worlds to sets of propositions that restricts
/// the domain of quantification for modals.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConversationalBackground {
    /// Kind of background.
    pub kind: BackgroundKind,
    /// Propositions in the background (simplified representation).
    pub propositions: Vec<String>,
}

impl ConversationalBackground {
    /// Create an empty background.
    #[must_use]
    pub fn empty(kind: BackgroundKind) -> Self {
        Self {
            kind,
            propositions: Vec::new(),
        }
    }

    /// Add a proposition to the background.
    pub fn add_proposition(&mut self, prop: impl Into<String>) {
        self.propositions.push(prop.into());
    }
}

/// Kind of conversational background.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BackgroundKind {
    /// Facts about circumstances.
    Circumstantial,
    /// What is known.
    Epistemic,
    /// What is required/permitted.
    Deontic,
    /// What is stereotypically the case.
    Stereotypical,
}

// ============================================================================
// Counterfactual Structures
// ============================================================================

/// Counterfactual conditional.
///
/// "If φ had been the case, ψ would have been the case"
///
/// Evaluated using Lewis's closest-world semantics: ψ is true in the
/// closest worlds where φ holds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Counterfactual {
    /// The antecedent ("if φ had been the case").
    pub antecedent: CounterfactualAntecedent,
    /// The consequent ("ψ would have been the case").
    pub consequent: CounterfactualConsequent,
    /// World selection strategy.
    pub world_selection: ClosestWorldSelection,
}

impl Counterfactual {
    /// Create a new counterfactual.
    #[must_use]
    pub fn new(antecedent: Box<Drs>, consequent: Box<Drs>, modal: CounterfactualModal) -> Self {
        Self {
            antecedent: CounterfactualAntecedent {
                drs: antecedent,
                is_past: true,
            },
            consequent: CounterfactualConsequent {
                drs: consequent,
                modal_marker: modal,
            },
            world_selection: ClosestWorldSelection::default(),
        }
    }
}

/// Antecedent of a counterfactual.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CounterfactualAntecedent {
    /// The DRS representing the counterfactual supposition.
    pub drs: Box<Drs>,
    /// Whether this is a past counterfactual ("had" + participle).
    pub is_past: bool,
}

/// Consequent of a counterfactual.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CounterfactualConsequent {
    /// The DRS representing the consequent.
    pub drs: Box<Drs>,
    /// Modal marker (would, might, could).
    pub modal_marker: CounterfactualModal,
}

/// Modal markers in counterfactual consequents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CounterfactualModal {
    /// "would" - necessity in closest worlds.
    #[default]
    Would,
    /// "might" - possibility in closest worlds.
    Might,
    /// "could" - ability/possibility.
    Could,
}

impl CounterfactualModal {
    /// Convert to modal force.
    #[must_use]
    pub const fn to_force(&self) -> ModalForce {
        match self {
            Self::Would => ModalForce::Necessity,
            Self::Might | Self::Could => ModalForce::Possibility,
        }
    }
}

impl std::fmt::Display for CounterfactualModal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Would => write!(f, "would"),
            Self::Might => write!(f, "might"),
            Self::Could => write!(f, "could"),
        }
    }
}

// ============================================================================
// Closest World Selection (Lewis)
// ============================================================================

/// Strategy for selecting closest worlds in counterfactual evaluation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClosestWorldSelection {
    /// Base world (typically actual world).
    pub base_world: WorldId,
    /// Similarity metric to use.
    pub similarity_metric: SimilarityMetric,
    /// Cached closest worlds (populated during evaluation).
    pub closest_worlds: Vec<ScoredWorld>,
}

impl ClosestWorldSelection {
    /// Create a new selection from the actual world.
    #[must_use]
    pub fn from_actual() -> Self {
        Self {
            base_world: WorldId::ACTUAL,
            similarity_metric: SimilarityMetric::Lewis,
            closest_worlds: Vec::new(),
        }
    }

    /// Get the closest worlds (must be populated first).
    #[must_use]
    pub fn get_closest(&self) -> Vec<WorldId> {
        if self.closest_worlds.is_empty() {
            return Vec::new();
        }

        // Return all worlds with maximum similarity
        let max_sim = self.closest_worlds[0].similarity;
        self.closest_worlds
            .iter()
            .take_while(|sw| (sw.similarity - max_sim).abs() < f32::EPSILON)
            .map(|sw| sw.world)
            .collect()
    }
}

impl Default for ClosestWorldSelection {
    fn default() -> Self {
        Self::from_actual()
    }
}

/// A world with its similarity score.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScoredWorld {
    /// The world.
    pub world: WorldId,
    /// Similarity to base world (higher = more similar).
    pub similarity: f32,
}

impl ScoredWorld {
    /// Create a new scored world.
    #[must_use]
    pub const fn new(world: WorldId, similarity: f32) -> Self {
        Self { world, similarity }
    }
}

/// Similarity metric for counterfactual evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum SimilarityMetric {
    /// Lewis's system: minimize miracles, match particular facts.
    #[default]
    Lewis,
    /// Stalnaker's system: unique closest world (selection function).
    Stalnaker,
    /// Pragmatic: context-dependent similarity.
    Pragmatic,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_id() {
        assert!(WorldId::ACTUAL.is_actual());
        assert!(!WorldId::new(1).is_actual());
        assert_eq!(format!("{}", WorldId::ACTUAL), "w₀");
        assert_eq!(format!("{}", WorldId::new(5)), "w5");
    }

    #[test]
    fn test_accessibility_from_flavor() {
        assert_eq!(
            AccessibilityType::from(ModalFlavor::Epistemic),
            AccessibilityType::Epistemic
        );
        assert_eq!(
            AccessibilityType::from(ModalFlavor::Deontic),
            AccessibilityType::Deontic
        );
    }

    #[test]
    fn test_world_set() {
        let mut set = WorldSet::empty();
        assert!(set.is_empty());

        set.insert(WorldId::ACTUAL);
        assert!(!set.is_empty());
        assert!(set.contains(&WorldId::ACTUAL));
        assert!(!set.contains(&WorldId::new(1)));
    }

    #[test]
    fn test_modal_frame() {
        let mut frame = ModalFrame::at_actual();
        assert!(frame.evaluation_world.is_actual());

        frame.add_accessibility(WorldId::new(1), AccessibilityType::Epistemic);
        let accessible = frame.get_accessible(AccessibilityType::Epistemic);
        assert_eq!(accessible.len(), 1);
        assert_eq!(accessible[0], WorldId::new(1));
    }

    #[test]
    fn test_counterfactual_modal() {
        assert_eq!(CounterfactualModal::Would.to_force(), ModalForce::Necessity);
        assert_eq!(
            CounterfactualModal::Might.to_force(),
            ModalForce::Possibility
        );
        assert_eq!(format!("{}", CounterfactualModal::Would), "would");
    }

    #[test]
    fn test_closest_world_selection() {
        let mut selection = ClosestWorldSelection::from_actual();
        selection.closest_worlds = vec![
            ScoredWorld::new(WorldId::new(1), 0.9),
            ScoredWorld::new(WorldId::new(2), 0.9),
            ScoredWorld::new(WorldId::new(3), 0.7),
        ];

        let closest = selection.get_closest();
        assert_eq!(closest.len(), 2);
        assert!(closest.contains(&WorldId::new(1)));
        assert!(closest.contains(&WorldId::new(2)));
    }

    #[test]
    fn test_conversational_background() {
        let mut bg = ConversationalBackground::empty(BackgroundKind::Epistemic);
        assert!(bg.propositions.is_empty());

        bg.add_proposition("John is at home");
        assert_eq!(bg.propositions.len(), 1);
    }
}
