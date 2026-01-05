//! Pronoun resolution and binding theory.
//!
//! Implements anaphora resolution based on:
//! - Salience (recency, grammatical role, etc.)
//! - Agreement (gender, number, person)
//! - Binding Theory constraints (Conditions A, B, C)
//!
//! # Binding Theory
//!
//! Per Chomsky (1981) and Reinhart & Reuland (1993):
//! - **Condition A**: Reflexives must be bound in their local domain
//! - **Condition B**: Pronouns must be free in their local domain
//! - **Condition C**: R-expressions must be free everywhere
//!
//! This module implements structural constraints; word-level knowledge
//! (gender lookup, animacy, etc.) comes from providers.

use super::referent::{DiscourseReferent, Gender, NumberFeature, ReferentId, ReferentRegistry};
use serde::{Deserialize, Serialize};

/// Result of pronoun resolution.
#[derive(Debug, Clone)]
pub struct BindingResult {
    /// The resolved antecedent, if found.
    pub antecedent: Option<ReferentId>,

    /// Confidence in the resolution (0.0 to 1.0).
    pub confidence: f32,

    /// All candidate antecedents considered.
    pub candidates: Vec<ReferentId>,

    /// Any binding constraints that were violated.
    pub violations: Vec<BindingConstraint>,
}

impl BindingResult {
    /// Create a successful binding result.
    #[must_use]
    pub fn resolved(antecedent: ReferentId, confidence: f32) -> Self {
        Self {
            antecedent: Some(antecedent),
            confidence,
            candidates: vec![antecedent],
            violations: Vec::new(),
        }
    }

    /// Create a failed binding result.
    #[must_use]
    pub fn unresolved(candidates: Vec<ReferentId>) -> Self {
        Self {
            antecedent: None,
            confidence: 0.0,
            candidates,
            violations: Vec::new(),
        }
    }

    /// Check if resolution was successful.
    #[must_use]
    pub fn is_resolved(&self) -> bool {
        self.antecedent.is_some()
    }
}

/// Binding theory constraints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BindingConstraint {
    /// Condition A: Reflexives must be locally bound.
    ConditionA,

    /// Condition B: Pronouns must be locally free.
    ConditionB,

    /// Condition C: R-expressions must be free.
    ConditionC,

    /// Gender agreement violation.
    GenderMismatch,

    /// Number agreement violation.
    NumberMismatch,

    /// Person agreement violation.
    PersonMismatch,

    /// No accessible antecedent.
    NoAccessibleAntecedent,
}

/// Type of anaphoric expression.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AnaphorType {
    /// Reflexive pronoun: himself, herself, themselves.
    Reflexive,

    /// Personal pronoun: he, she, it, they.
    Personal,

    /// Possessive pronoun: his, her, its, their.
    Possessive,

    /// Demonstrative: this, that, these, those.
    Demonstrative,

    /// Relative pronoun: who, which, that.
    Relative,
}

/// Pronoun resolver for anaphora resolution.
#[derive(Debug, Clone, Default)]
pub struct PronounResolver {
    /// Minimum confidence threshold for resolution.
    pub min_confidence: f32,

    /// Weight for recency in salience calculation.
    pub recency_weight: f32,

    /// Weight for grammatical role in salience calculation.
    pub role_weight: f32,
}

impl PronounResolver {
    /// Create a new resolver with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self {
            min_confidence: 0.3,
            recency_weight: 0.5,
            role_weight: 0.3,
        }
    }

    /// Resolve a pronoun to an antecedent.
    ///
    /// # Arguments
    /// * `registry` - The referent registry
    /// * `anaphor_type` - Type of anaphoric expression
    /// * `gender` - Required gender (if known)
    /// * `number` - Required number (if known)
    /// * `current_sentence` - Current sentence index
    ///
    /// # Returns
    /// Binding result with resolved antecedent or candidates.
    #[must_use]
    pub fn resolve(
        &self,
        registry: &ReferentRegistry,
        anaphor_type: AnaphorType,
        gender: Option<Gender>,
        number: Option<NumberFeature>,
        current_sentence: usize,
    ) -> BindingResult {
        // Find candidates that agree in gender/number
        let candidates = registry.find_candidates(gender, number);

        if candidates.is_empty() {
            return BindingResult {
                antecedent: None,
                confidence: 0.0,
                candidates: Vec::new(),
                violations: vec![BindingConstraint::NoAccessibleAntecedent],
            };
        }

        // Score candidates
        let mut scored: Vec<_> = candidates
            .iter()
            .map(|r| {
                let score = self.score_candidate(r, anaphor_type, current_sentence);
                (r.id, score)
            })
            .collect();

        // Sort by score descending
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let candidate_ids: Vec<_> = scored.iter().map(|(id, _)| *id).collect();

        // Select best candidate if above threshold
        if let Some((best_id, best_score)) = scored.first() {
            if *best_score >= self.min_confidence {
                return BindingResult {
                    antecedent: Some(*best_id),
                    confidence: *best_score,
                    candidates: candidate_ids,
                    violations: Vec::new(),
                };
            }
        }

        BindingResult::unresolved(candidate_ids)
    }

    /// Score a candidate antecedent.
    fn score_candidate(
        &self,
        referent: &DiscourseReferent,
        anaphor_type: AnaphorType,
        current_sentence: usize,
    ) -> f32 {
        let mut score = referent.salience;

        // Recency bonus
        let sentence_distance = current_sentence.saturating_sub(referent.introduced_at);
        let recency_bonus = match sentence_distance {
            0 => 0.3, // Same sentence
            1 => 0.2, // Previous sentence
            2 => 0.1, // Two sentences back
            _ => 0.0, // Older
        };
        score += recency_bonus * self.recency_weight;

        // Anaphor type constraints
        match anaphor_type {
            AnaphorType::Reflexive => {
                // Reflexives prefer local antecedents (same sentence)
                if sentence_distance == 0 {
                    score += 0.3;
                } else {
                    score -= 0.5; // Penalty for non-local antecedent
                }
            }
            AnaphorType::Personal => {
                // Personal pronouns slightly prefer non-local antecedents
                // (Condition B: must be free in local domain)
                if sentence_distance > 0 {
                    score += 0.1;
                }
            }
            AnaphorType::Demonstrative => {
                // Demonstratives prefer recently mentioned entities
                if sentence_distance <= 1 {
                    score += 0.2;
                }
            }
            _ => {}
        }

        // Cap at 1.0
        score.min(1.0)
    }

    /// Check binding constraints for a specific antecedent.
    #[must_use]
    pub fn check_constraints(
        &self,
        anaphor_type: AnaphorType,
        _antecedent: &DiscourseReferent,
        same_clause: bool,
    ) -> Vec<BindingConstraint> {
        let mut violations = Vec::new();

        match anaphor_type {
            AnaphorType::Reflexive => {
                // Condition A: reflexives must be bound in local domain
                if !same_clause {
                    violations.push(BindingConstraint::ConditionA);
                }
            }
            AnaphorType::Personal => {
                // Condition B: pronouns must be free in local domain
                // (This is a simplification - full implementation needs c-command)
                // For now, we just note that same-clause binding is suspicious
                if same_clause {
                    // Not necessarily a violation, depends on structural position
                }
            }
            _ => {}
        }

        violations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolver_creation() {
        let resolver = PronounResolver::new();
        assert!((resolver.min_confidence - 0.3).abs() < f32::EPSILON);
    }

    #[test]
    fn test_resolve_with_single_candidate() {
        let mut registry = ReferentRegistry::new();
        let id = registry.introduce_entity("John");
        if let Some(r) = registry.get_mut(id) {
            r.gender = Gender::Masculine;
            r.salience = 0.9;
        }

        let resolver = PronounResolver::new();
        let result = resolver.resolve(
            &registry,
            AnaphorType::Personal,
            Some(Gender::Masculine),
            None,
            0,
        );

        assert!(result.is_resolved());
        assert_eq!(result.antecedent, Some(id));
    }

    #[test]
    fn test_resolve_gender_mismatch() {
        let mut registry = ReferentRegistry::new();
        let id = registry.introduce_entity("John");
        if let Some(r) = registry.get_mut(id) {
            r.gender = Gender::Masculine;
        }

        let resolver = PronounResolver::new();
        let result = resolver.resolve(
            &registry,
            AnaphorType::Personal,
            Some(Gender::Feminine), // Looking for feminine
            None,
            0,
        );

        assert!(!result.is_resolved());
        assert!(result
            .violations
            .contains(&BindingConstraint::NoAccessibleAntecedent));
    }

    #[test]
    fn test_resolve_multiple_candidates() {
        let mut registry = ReferentRegistry::new();

        // John (masculine, high salience)
        let john_id = registry.introduce_entity("John");
        if let Some(r) = registry.get_mut(john_id) {
            r.gender = Gender::Masculine;
            r.salience = 0.9;
        }

        // Bill (masculine, lower salience)
        let bill_id = registry.introduce_entity("Bill");
        if let Some(r) = registry.get_mut(bill_id) {
            r.gender = Gender::Masculine;
            r.salience = 0.5;
        }

        let resolver = PronounResolver::new();
        let result = resolver.resolve(
            &registry,
            AnaphorType::Personal,
            Some(Gender::Masculine),
            None,
            0,
        );

        assert!(result.is_resolved());
        assert_eq!(result.antecedent, Some(john_id)); // Higher salience wins
        assert_eq!(result.candidates.len(), 2);
    }

    #[test]
    fn test_binding_result_helpers() {
        let resolved = BindingResult::resolved(ReferentId::new(0), 0.9);
        assert!(resolved.is_resolved());

        let unresolved = BindingResult::unresolved(vec![ReferentId::new(0)]);
        assert!(!unresolved.is_resolved());
    }

    #[test]
    fn test_reflexive_prefers_local() {
        let mut registry = ReferentRegistry::new();

        // John (introduced earlier)
        let john_id = registry.introduce_entity("John");
        if let Some(r) = registry.get_mut(john_id) {
            r.gender = Gender::Masculine;
            r.salience = 0.9;
            r.introduced_at = 0;
        }

        registry.next_sentence();

        // Bill (introduced in current sentence)
        let bill_id = registry.introduce_entity("Bill");
        if let Some(r) = registry.get_mut(bill_id) {
            r.gender = Gender::Masculine;
            r.salience = 0.7;
            r.introduced_at = 1;
        }

        let resolver = PronounResolver::new();
        let result = resolver.resolve(
            &registry,
            AnaphorType::Reflexive,
            Some(Gender::Masculine),
            None,
            1, // Current sentence
        );

        // Reflexive should prefer Bill (same sentence) despite lower base salience
        assert!(result.is_resolved());
        assert_eq!(result.antecedent, Some(bill_id));
    }
}
