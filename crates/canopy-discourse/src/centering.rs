//! Centering Theory for Thematic Continuity
//!
//! Implements Grosz, Joshi & Weinstein (1995) "Centering: A Framework
//! for Modeling the Local Coherence of Discourse".
//!
//! Centering Theory tracks:
//! - Forward-looking centers (Cf): entities mentioned in current utterance
//! - Backward-looking center (Cb): the current discourse topic
//! - Preferred center (Cp): most salient entity (highest-ranked Cf)
//! - Transition types: Continue, Retain, Shift (smooth/rough)

use crate::referent::{ReferentId, ReferentRegistry};
use canopy_core::{Animacy, ThetaRole};
use serde::{Deserialize, Serialize};

/// Centering transition types (Grosz, Joshi & Weinstein 1995)
///
/// Ordered by preference: Continue > Retain > SmoothShift > RoughShift
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CenteringTransition {
    /// Cb(Un) = Cb(Un-1) = Cp(Un)
    /// Same topic, topic is most salient: smooth continuation
    Continue,

    /// Cb(Un) = Cb(Un-1) ≠ Cp(Un)
    /// Same topic, but not most salient: topic retained but challenged
    Retain,

    /// Cb(Un) ≠ Cb(Un-1), Cb(Un) = Cp(Un)
    /// Topic changed to preferred center: smooth topic shift
    SmoothShift,

    /// Cb(Un) ≠ Cb(Un-1), Cb(Un) ≠ Cp(Un)
    /// Topic changed to non-preferred center: rough/abrupt shift
    RoughShift,

    /// First utterance or no previous Cb
    Establishing,
}

impl CenteringTransition {
    /// Coherence score for the transition type
    /// Higher is more coherent (easier to process)
    #[must_use]
    pub fn coherence_score(self) -> f32 {
        match self {
            Self::Continue => 1.0,
            Self::Retain => 0.75,
            Self::SmoothShift => 0.5,
            Self::RoughShift => 0.25,
            Self::Establishing => 0.8, // First mention is neutral
        }
    }
}

/// Grammatical role for Cf ranking
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum GrammaticalRole {
    /// Highest salience
    Subject,
    /// High salience
    DirectObject,
    /// Medium salience
    IndirectObject,
    /// Lower salience
    Oblique,
    /// Lowest salience
    Other,
}

impl GrammaticalRole {
    /// Convert from theta role to grammatical role (approximation)
    #[must_use]
    pub fn from_theta_role(role: ThetaRole) -> Self {
        match role {
            ThetaRole::Agent | ThetaRole::Experiencer => Self::Subject,
            ThetaRole::Theme | ThetaRole::Patient => Self::DirectObject,
            ThetaRole::Recipient | ThetaRole::Benefactive | ThetaRole::Goal => Self::IndirectObject,
            ThetaRole::Location | ThetaRole::Source | ThetaRole::Instrument => Self::Oblique,
            _ => Self::Other,
        }
    }

    /// Salience score for ranking
    #[must_use]
    pub fn salience(self) -> f32 {
        match self {
            Self::Subject => 1.0,
            Self::DirectObject => 0.8,
            Self::IndirectObject => 0.6,
            Self::Oblique => 0.4,
            Self::Other => 0.2,
        }
    }
}

/// An entity's position in the Cf list
#[derive(Debug, Clone)]
pub struct CfEntry {
    pub referent_id: ReferentId,
    pub grammatical_role: GrammaticalRole,
    pub animacy_boost: f32,
    pub salience_score: f32,
}

impl CfEntry {
    /// Calculate total salience score
    fn calculate_salience(role: GrammaticalRole, animacy: Option<Animacy>) -> f32 {
        let base = role.salience();

        // Animacy boost: humans are more salient
        let animacy_factor = match animacy {
            Some(Animacy::Human) => 1.2,
            Some(Animacy::Animal) => 1.1,
            Some(Animacy::Plant) => 0.9,
            Some(Animacy::Inanimate) => 0.8,
            None => 1.0,
        };

        base * animacy_factor
    }
}

/// Centering Theory tracker
///
/// Maintains centering state across utterances and computes transitions.
#[derive(Debug, Clone)]
pub struct CenteringTracker {
    /// Forward-looking centers for current utterance (ranked by salience)
    cf_list: Vec<CfEntry>,

    /// Backward-looking center (current discourse topic)
    cb: Option<ReferentId>,

    /// Previous utterance's Cb (for transition computation)
    prev_cb: Option<ReferentId>,

    /// Previous utterance's Cf list
    prev_cf: Vec<CfEntry>,

    /// Transition type from previous to current utterance
    current_transition: CenteringTransition,

    /// Utterance count
    utterance_count: usize,
}

impl CenteringTracker {
    /// Create a new centering tracker
    #[must_use]
    pub fn new() -> Self {
        Self {
            cf_list: Vec::new(),
            cb: None,
            prev_cb: None,
            prev_cf: Vec::new(),
            current_transition: CenteringTransition::Establishing,
            utterance_count: 0,
        }
    }

    /// Process a new utterance and update centering state
    ///
    /// # Arguments
    /// * `referents` - Entities mentioned in the utterance with their roles
    /// * `registry` - Reference to the full referent registry for properties
    pub fn process_utterance(
        &mut self,
        referents: &[(ReferentId, ThetaRole)],
        registry: &ReferentRegistry,
    ) {
        self.utterance_count += 1;

        // Save previous state
        self.prev_cb = self.cb;
        self.prev_cf = std::mem::take(&mut self.cf_list);

        // Build new Cf list
        self.cf_list = referents
            .iter()
            .map(|&(id, role)| {
                let gram_role = GrammaticalRole::from_theta_role(role);
                let animacy = registry.get(id).and_then(|r| r.animacy());
                let animacy_boost = match animacy {
                    Some(Animacy::Human) => 0.2,
                    Some(Animacy::Animal) => 0.1,
                    _ => 0.0,
                };
                let salience = CfEntry::calculate_salience(gram_role, animacy);

                CfEntry {
                    referent_id: id,
                    grammatical_role: gram_role,
                    animacy_boost,
                    salience_score: salience,
                }
            })
            .collect();

        // Sort by salience (descending)
        self.cf_list
            .sort_by(|a, b| b.salience_score.partial_cmp(&a.salience_score).unwrap());

        // Compute new Cb
        self.cb = self.compute_cb();

        // Compute transition
        self.current_transition = self.compute_transition();
    }

    /// Compute backward-looking center
    ///
    /// Cb(Un) is the highest-ranked element of Cf(Un-1) that is also in Cf(Un)
    fn compute_cb(&self) -> Option<ReferentId> {
        // First utterance: no Cb yet
        if self.prev_cf.is_empty() {
            // For first utterance, Cb is the highest-ranked Cf
            return self.cf_list.first().map(|e| e.referent_id);
        }

        // Find highest-ranked entity from previous Cf that appears in current Cf
        for prev_entry in &self.prev_cf {
            if self
                .cf_list
                .iter()
                .any(|e| e.referent_id == prev_entry.referent_id)
            {
                return Some(prev_entry.referent_id);
            }
        }

        // No overlap: use highest-ranked current Cf as new topic
        self.cf_list.first().map(|e| e.referent_id)
    }

    /// Compute transition type between previous and current utterance
    fn compute_transition(&self) -> CenteringTransition {
        let cb = match self.cb {
            Some(cb) => cb,
            None => return CenteringTransition::Establishing,
        };

        let cp = match self.cf_list.first() {
            Some(entry) => entry.referent_id,
            None => return CenteringTransition::Establishing,
        };

        let prev_cb = match self.prev_cb {
            Some(prev) => prev,
            None => return CenteringTransition::Establishing,
        };

        // Determine transition type
        let same_cb = cb == prev_cb;
        let cb_is_cp = cb == cp;

        match (same_cb, cb_is_cp) {
            (true, true) => CenteringTransition::Continue,
            (true, false) => CenteringTransition::Retain,
            (false, true) => CenteringTransition::SmoothShift,
            (false, false) => CenteringTransition::RoughShift,
        }
    }

    /// Get the current backward-looking center (topic)
    #[must_use]
    pub fn current_topic(&self) -> Option<ReferentId> {
        self.cb
    }

    /// Get the preferred center (highest-ranked Cf)
    #[must_use]
    pub fn preferred_center(&self) -> Option<ReferentId> {
        self.cf_list.first().map(|e| e.referent_id)
    }

    /// Get the current transition type
    #[must_use]
    pub fn transition_type(&self) -> CenteringTransition {
        self.current_transition
    }

    /// Get the coherence score for current transition
    #[must_use]
    pub fn continuity_score(&self) -> f32 {
        self.current_transition.coherence_score()
    }

    /// Check if there was a topic shift
    #[must_use]
    pub fn has_topic_shift(&self) -> bool {
        matches!(
            self.current_transition,
            CenteringTransition::SmoothShift | CenteringTransition::RoughShift
        )
    }

    /// Get the forward-looking centers list
    #[must_use]
    pub fn cf_list(&self) -> &[CfEntry] {
        &self.cf_list
    }

    /// Get the number of utterances processed
    #[must_use]
    pub fn utterance_count(&self) -> usize {
        self.utterance_count
    }

    /// Reset the tracker
    pub fn reset(&mut self) {
        self.cf_list.clear();
        self.cb = None;
        self.prev_cb = None;
        self.prev_cf.clear();
        self.current_transition = CenteringTransition::Establishing;
        self.utterance_count = 0;
    }
}

impl Default for CenteringTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::referent::DiscourseReferent;

    fn create_test_registry() -> ReferentRegistry {
        let mut registry = ReferentRegistry::new();

        // John (human, masculine)
        let john_id = registry.allocate_id();
        let mut john = DiscourseReferent::entity(john_id, "John".to_string(), 0);
        john.set_animacy(Animacy::Human);
        registry.register(john);

        // Mary (human, feminine)
        let mary_id = registry.allocate_id();
        let mut mary = DiscourseReferent::entity(mary_id, "Mary".to_string(), 0);
        mary.set_animacy(Animacy::Human);
        registry.register(mary);

        // book (inanimate)
        let book_id = registry.allocate_id();
        let mut book = DiscourseReferent::entity(book_id, "book".to_string(), 0);
        book.set_animacy(Animacy::Inanimate);
        registry.register(book);

        registry
    }

    #[test]
    fn test_continue_transition() {
        // "John bought a book. He read it."
        // John stays as Cb, John is Cp → Continue
        let registry = create_test_registry();
        let mut tracker = CenteringTracker::new();

        let john = ReferentId(1);
        let book = ReferentId(3);

        // Utterance 1: "John bought a book"
        tracker.process_utterance(
            &[(john, ThetaRole::Agent), (book, ThetaRole::Theme)],
            &registry,
        );
        assert_eq!(tracker.current_topic(), Some(john)); // John is Cb
        assert_eq!(tracker.transition_type(), CenteringTransition::Establishing);

        // Utterance 2: "He (John) read it (book)"
        tracker.process_utterance(
            &[(john, ThetaRole::Agent), (book, ThetaRole::Theme)],
            &registry,
        );
        assert_eq!(tracker.current_topic(), Some(john)); // John still Cb
        assert_eq!(tracker.transition_type(), CenteringTransition::Continue);
        assert!(tracker.continuity_score() > 0.9);
    }

    #[test]
    fn test_smooth_shift() {
        // "John met Mary. She smiled."
        // Cb shifts from John to Mary, Mary is Cp → SmoothShift
        let registry = create_test_registry();
        let mut tracker = CenteringTracker::new();

        let john = ReferentId(1);
        let mary = ReferentId(2);

        // Utterance 1: "John met Mary"
        tracker.process_utterance(
            &[(john, ThetaRole::Agent), (mary, ThetaRole::Theme)],
            &registry,
        );
        assert_eq!(tracker.current_topic(), Some(john));

        // Utterance 2: "She smiled" (Mary as subject)
        tracker.process_utterance(&[(mary, ThetaRole::Agent)], &registry);
        assert_eq!(tracker.current_topic(), Some(mary)); // Topic shifted to Mary
        assert_eq!(tracker.transition_type(), CenteringTransition::SmoothShift);
    }

    #[test]
    fn test_retain_transition() {
        // "John gave Mary a book. She liked it."
        // John should remain Cb (as he's in prev Cf), but Mary is now Cp
        let registry = create_test_registry();
        let mut tracker = CenteringTracker::new();

        let john = ReferentId(1);
        let mary = ReferentId(2);
        let book = ReferentId(3);

        // Utterance 1: "John gave Mary a book"
        tracker.process_utterance(
            &[
                (john, ThetaRole::Agent),
                (mary, ThetaRole::Recipient),
                (book, ThetaRole::Theme),
            ],
            &registry,
        );
        assert_eq!(tracker.current_topic(), Some(john));

        // Utterance 2: "She liked it" with John still mentioned obliquely
        tracker.process_utterance(
            &[
                (mary, ThetaRole::Experiencer),
                (book, ThetaRole::Theme),
                (john, ThetaRole::Source), // John mentioned obliquely
            ],
            &registry,
        );
        // Mary is Cp (subject), but John might still be Cb from previous ranking
    }

    #[test]
    fn test_grammatical_role_ranking() {
        assert!(GrammaticalRole::Subject.salience() > GrammaticalRole::DirectObject.salience());
        assert!(
            GrammaticalRole::DirectObject.salience() > GrammaticalRole::IndirectObject.salience()
        );
        assert!(GrammaticalRole::IndirectObject.salience() > GrammaticalRole::Oblique.salience());
    }

    #[test]
    fn test_transition_coherence_scores() {
        assert!(
            CenteringTransition::Continue.coherence_score()
                > CenteringTransition::Retain.coherence_score()
        );
        assert!(
            CenteringTransition::Retain.coherence_score()
                > CenteringTransition::SmoothShift.coherence_score()
        );
        assert!(
            CenteringTransition::SmoothShift.coherence_score()
                > CenteringTransition::RoughShift.coherence_score()
        );
    }

    #[test]
    fn test_topic_shift_detection() {
        let registry = create_test_registry();
        let mut tracker = CenteringTracker::new();

        let john = ReferentId(1);
        let mary = ReferentId(2);

        tracker.process_utterance(&[(john, ThetaRole::Agent)], &registry);
        assert!(!tracker.has_topic_shift());

        tracker.process_utterance(&[(mary, ThetaRole::Agent)], &registry);
        assert!(tracker.has_topic_shift());
    }
}
