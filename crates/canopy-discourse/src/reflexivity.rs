//! Predicate reflexivity analysis
//!
//! Based on Reinhart & Reuland (1993) "Reflexivity" and Reuland (2011)
//! "Anaphora and Language Design".
//!
//! Key insight: Reflexivity is a property of **predicates**, not anaphors.
//!
//! - **Condition A**: A reflexive-marked syntactic predicate is reflexive
//! - **Condition B**: A reflexive semantic predicate is reflexive-marked
//!
//! A predicate is reflexive when two of its arguments co-refer.
//! A predicate is reflexive-marked when one argument is a SELF-anaphor.

use crate::referent::{classify_anaphor, AnaphorType, ReferentId};
use canopy_core::Entity;
use std::collections::HashSet;

/// Intrinsically reflexive verbs that can omit their object
/// with a reflexive interpretation.
///
/// "John washed" means "John washed himself"
/// "John shaved" means "John shaved himself"
const INTRINSICALLY_REFLEXIVE_VERBS: &[&str] = &[
    // Grooming verbs (classic intrinsically reflexive)
    "wash",
    "bathe",
    "shave",
    "shower",
    "dress",
    "undress",
    "groom",
    // Body positioning
    "stretch",
    "relax",
    "brace",
    "prepare",
    "ready",
    // Social actions with reflexive defaults
    "introduce",
    "present",
    "excuse",
    "defend",
    "protect",
];

/// Analyze predicate reflexivity per Reinhart & Reuland (1993)
#[derive(Debug, Clone)]
pub struct PredicateAnalyzer {
    /// Set of intrinsically reflexive verb lemmas
    intrinsic_reflexives: HashSet<String>,
}

impl PredicateAnalyzer {
    /// Create a new predicate analyzer
    #[must_use]
    pub fn new() -> Self {
        let intrinsic_reflexives = INTRINSICALLY_REFLEXIVE_VERBS
            .iter()
            .map(|s| s.to_string())
            .collect();

        Self {
            intrinsic_reflexives,
        }
    }

    /// Check if a verb is intrinsically reflexive
    ///
    /// Intrinsically reflexive verbs can omit their object and still
    /// receive a reflexive interpretation: "John washed" = "John washed himself"
    #[must_use]
    pub fn is_intrinsically_reflexive(&self, verb_lemma: &str) -> bool {
        self.intrinsic_reflexives
            .contains(&verb_lemma.to_lowercase())
    }

    /// Check if a predicate is reflexive-marked by a SELF-anaphor
    ///
    /// A predicate is reflexive-marked when one of its arguments is
    /// a SELF-anaphor (himself, herself, itself, themselves, etc.)
    #[must_use]
    pub fn is_reflexive_marked(&self, participants: &[&Entity]) -> bool {
        participants.iter().any(|p| {
            matches!(
                classify_anaphor(&p.text).anaphor_type,
                AnaphorType::SelfAnaphor
            )
        })
    }

    /// Check if a predicate is reflexive-marked by text
    ///
    /// Convenience method when you have participant text directly
    #[must_use]
    pub fn is_reflexive_marked_by_text(&self, participant_texts: &[&str]) -> bool {
        participant_texts.iter().any(|text| {
            matches!(
                classify_anaphor(text).anaphor_type,
                AnaphorType::SelfAnaphor
            )
        })
    }

    /// Check if two referent IDs represent a reflexive configuration
    ///
    /// A predicate is semantically reflexive when two of its arguments co-refer.
    #[must_use]
    pub fn is_semantically_reflexive(&self, arg1: ReferentId, arg2: ReferentId) -> bool {
        arg1 == arg2
    }

    /// Validate Condition B: A reflexive semantic predicate must be reflexive-marked
    ///
    /// If two arguments of a predicate co-refer (making it reflexive),
    /// then one of them must be a SELF-anaphor OR the predicate must be
    /// intrinsically reflexive.
    ///
    /// Returns `true` if the configuration is valid, `false` if it violates Condition B.
    ///
    /// # Examples
    ///
    /// - "John washed himself" → valid (reflexive-marked)
    /// - "John washed" → valid (intrinsically reflexive)
    /// - "John criticized himself" → valid (reflexive-marked)
    /// - "John criticized him" where him=John → INVALID (not reflexive-marked)
    #[must_use]
    pub fn check_condition_b(
        &self,
        verb_lemma: &str,
        arg1_id: ReferentId,
        arg2_id: ReferentId,
        arg2_text: &str,
    ) -> ConditionBResult {
        // If arguments don't co-refer, Condition B doesn't apply
        if arg1_id != arg2_id {
            return ConditionBResult::NotApplicable;
        }

        // Predicate is semantically reflexive (two args co-refer)
        // It must be reflexive-marked

        // Check if reflexive-marked by SELF-anaphor
        if matches!(
            classify_anaphor(arg2_text).anaphor_type,
            AnaphorType::SelfAnaphor
        ) {
            return ConditionBResult::Valid {
                reason: "reflexive-marked by SELF-anaphor".to_string(),
            };
        }

        // Check if intrinsically reflexive
        if self.is_intrinsically_reflexive(verb_lemma) {
            return ConditionBResult::Valid {
                reason: "intrinsically reflexive verb".to_string(),
            };
        }

        // Violation: reflexive predicate without reflexive-marking
        ConditionBResult::Violation {
            reason: format!(
                "predicate '{}' is reflexive but not reflexive-marked (expected SELF-anaphor, got '{}')",
                verb_lemma, arg2_text
            ),
        }
    }

    /// Check if a pronoun can co-refer with an entity as co-arguments
    ///
    /// This is the resolution-time check for Condition B:
    /// A personal pronoun CANNOT co-refer with a co-argument unless
    /// the predicate is intrinsically reflexive.
    #[must_use]
    pub fn can_corefer_as_coarguments(&self, verb_lemma: &str, pronoun_text: &str) -> bool {
        let classification = classify_anaphor(pronoun_text);

        match classification.anaphor_type {
            // SELF-anaphors CAN co-refer with co-arguments (that's their job)
            AnaphorType::SelfAnaphor => true,

            // Personal pronouns CANNOT co-refer with co-arguments
            // unless the predicate is intrinsically reflexive
            AnaphorType::Personal => self.is_intrinsically_reflexive(verb_lemma),

            // Possessives have more complex binding (not co-arguments in same sense)
            AnaphorType::Possessive => true,

            // Non-pronouns: no constraint
            AnaphorType::None => true,
        }
    }
}

impl Default for PredicateAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of checking Condition B
#[derive(Debug, Clone, PartialEq)]
pub enum ConditionBResult {
    /// Condition B is satisfied
    Valid { reason: String },
    /// Condition B is violated
    Violation { reason: String },
    /// Condition B doesn't apply (arguments don't co-refer)
    NotApplicable,
}

impl ConditionBResult {
    /// Check if the result indicates a valid configuration
    #[must_use]
    pub fn is_valid(&self) -> bool {
        matches!(
            self,
            ConditionBResult::Valid { .. } | ConditionBResult::NotApplicable
        )
    }

    /// Check if the result indicates a violation
    #[must_use]
    pub fn is_violation(&self) -> bool {
        matches!(self, ConditionBResult::Violation { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intrinsically_reflexive() {
        let analyzer = PredicateAnalyzer::new();

        assert!(analyzer.is_intrinsically_reflexive("wash"));
        assert!(analyzer.is_intrinsically_reflexive("shave"));
        assert!(analyzer.is_intrinsically_reflexive("dress"));
        assert!(!analyzer.is_intrinsically_reflexive("criticize"));
        assert!(!analyzer.is_intrinsically_reflexive("hit"));
    }

    #[test]
    fn test_reflexive_marked_by_text() {
        let analyzer = PredicateAnalyzer::new();

        assert!(analyzer.is_reflexive_marked_by_text(&["John", "himself"]));
        assert!(analyzer.is_reflexive_marked_by_text(&["Mary", "herself"]));
        assert!(!analyzer.is_reflexive_marked_by_text(&["John", "him"]));
        assert!(!analyzer.is_reflexive_marked_by_text(&["Mary", "her"]));
    }

    #[test]
    fn test_condition_b_with_reflexive() {
        let analyzer = PredicateAnalyzer::new();
        let id = ReferentId(1);

        // "John criticized himself" → valid
        let result = analyzer.check_condition_b("criticize", id, id, "himself");
        assert!(result.is_valid());
    }

    #[test]
    fn test_condition_b_violation() {
        let analyzer = PredicateAnalyzer::new();
        let id = ReferentId(1);

        // "John criticized him" where him=John → violation
        let result = analyzer.check_condition_b("criticize", id, id, "him");
        assert!(result.is_violation());
    }

    #[test]
    fn test_condition_b_intrinsic_reflexive() {
        let analyzer = PredicateAnalyzer::new();
        let id = ReferentId(1);

        // "John washed him" where him=John → valid (wash is intrinsically reflexive)
        // Note: This is a marginal case, but the grammar allows it
        let result = analyzer.check_condition_b("wash", id, id, "him");
        assert!(result.is_valid());
    }

    #[test]
    fn test_condition_b_no_coreference() {
        let analyzer = PredicateAnalyzer::new();
        let id1 = ReferentId(1);
        let id2 = ReferentId(2);

        // "John criticized him" where him≠John → not applicable
        let result = analyzer.check_condition_b("criticize", id1, id2, "him");
        assert!(matches!(result, ConditionBResult::NotApplicable));
    }

    #[test]
    fn test_can_corefer() {
        let analyzer = PredicateAnalyzer::new();

        // SELF-anaphors can always co-refer with co-arguments
        assert!(analyzer.can_corefer_as_coarguments("criticize", "himself"));
        assert!(analyzer.can_corefer_as_coarguments("hit", "herself"));

        // Personal pronouns cannot (for regular predicates)
        assert!(!analyzer.can_corefer_as_coarguments("criticize", "him"));
        assert!(!analyzer.can_corefer_as_coarguments("hit", "her"));

        // Personal pronouns CAN for intrinsically reflexive predicates
        assert!(analyzer.can_corefer_as_coarguments("wash", "him"));
        assert!(analyzer.can_corefer_as_coarguments("shave", "her"));
    }
}
