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
//! # Underspecified Binding
//!
//! For ambiguity handling, this module also provides `UnderspecBinding`
//! which preserves all candidate antecedents with their scores,
//! allowing disambiguation to be deferred.
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

/// Underspecified binding result for ambiguity preservation.
///
/// Unlike `BindingResult` which selects a single best antecedent,
/// `UnderspecBinding` preserves all candidate antecedents with their
/// scores, allowing disambiguation to be deferred.
///
/// This supports the packed representation approach where multiple
/// readings are tracked simultaneously.
#[derive(Debug, Clone)]
pub struct UnderspecBinding {
    /// All candidate antecedents with their salience scores (0.0 to 1.0).
    /// Sorted by score descending.
    pub candidates: Vec<(ReferentId, f32)>,

    /// Context-preferred candidate, if one is clearly favored.
    /// This may be set based on centering theory or discourse salience.
    pub preferred: Option<ReferentId>,

    /// Whether this binding must be resolved for interpretation.
    /// - `true`: Anaphor requires antecedent (e.g., reflexive "himself")
    /// - `false`: Can remain ambiguous (e.g., "it" with multiple referents)
    pub requires_resolution: bool,

    /// Type of anaphoric expression.
    pub anaphor_type: AnaphorType,

    /// Any binding constraints that apply.
    pub constraints: Vec<BindingConstraint>,
}

impl UnderspecBinding {
    /// Create a new underspecified binding.
    #[must_use]
    pub fn new(
        candidates: Vec<(ReferentId, f32)>,
        anaphor_type: AnaphorType,
        requires_resolution: bool,
    ) -> Self {
        let preferred = if candidates.len() == 1 {
            Some(candidates[0].0)
        } else {
            None
        };

        Self {
            candidates,
            preferred,
            requires_resolution,
            anaphor_type,
            constraints: Vec::new(),
        }
    }

    /// Check if binding is ambiguous (multiple viable candidates).
    #[must_use]
    pub fn is_ambiguous(&self) -> bool {
        self.candidates.len() > 1 && self.preferred.is_none()
    }

    /// Get the number of candidate antecedents.
    #[must_use]
    pub fn candidate_count(&self) -> usize {
        self.candidates.len()
    }

    /// Get the best candidate (highest score).
    #[must_use]
    pub fn best_candidate(&self) -> Option<(ReferentId, f32)> {
        self.candidates.first().copied()
    }

    /// Get candidates above a confidence threshold.
    #[must_use]
    pub fn candidates_above(&self, threshold: f32) -> Vec<(ReferentId, f32)> {
        self.candidates
            .iter()
            .filter(|(_, score)| *score >= threshold)
            .copied()
            .collect()
    }

    /// Convert to a resolved `BindingResult` by selecting the best candidate.
    #[must_use]
    pub fn to_resolved(&self) -> BindingResult {
        if let Some((id, score)) = self.best_candidate() {
            BindingResult {
                antecedent: Some(id),
                confidence: score,
                candidates: self.candidates.iter().map(|(id, _)| *id).collect(),
                violations: self.constraints.clone(),
            }
        } else {
            BindingResult {
                antecedent: None,
                confidence: 0.0,
                candidates: Vec::new(),
                violations: vec![BindingConstraint::NoAccessibleAntecedent],
            }
        }
    }

    /// Set the preferred candidate based on context.
    pub fn set_preferred(&mut self, preferred: ReferentId) {
        if self.candidates.iter().any(|(id, _)| *id == preferred) {
            self.preferred = Some(preferred);
        }
    }

    /// Check if a specific referent is a candidate.
    #[must_use]
    pub fn has_candidate(&self, referent: ReferentId) -> bool {
        self.candidates.iter().any(|(id, _)| *id == referent)
    }

    /// Get the score for a specific candidate.
    #[must_use]
    pub fn score_for(&self, referent: ReferentId) -> Option<f32> {
        self.candidates
            .iter()
            .find(|(id, _)| *id == referent)
            .map(|(_, score)| *score)
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

    /// Resolve a pronoun to all candidate antecedents (underspecified).
    ///
    /// Unlike `resolve()` which selects the best antecedent, this method
    /// returns all viable candidates with their scores, preserving ambiguity.
    ///
    /// # Arguments
    /// * `registry` - The referent registry
    /// * `anaphor_type` - Type of anaphoric expression
    /// * `gender` - Required gender (if known)
    /// * `number` - Required number (if known)
    /// * `current_sentence` - Current sentence index
    ///
    /// # Returns
    /// An `UnderspecBinding` with all candidates and their scores.
    #[must_use]
    pub fn resolve_underspec(
        &self,
        registry: &ReferentRegistry,
        anaphor_type: AnaphorType,
        gender: Option<Gender>,
        number: Option<NumberFeature>,
        current_sentence: usize,
    ) -> UnderspecBinding {
        // Find candidates that agree in gender/number
        let candidates = registry.find_candidates(gender, number);

        if candidates.is_empty() {
            let mut binding = UnderspecBinding::new(
                Vec::new(),
                anaphor_type,
                Self::requires_resolution(anaphor_type),
            );
            binding
                .constraints
                .push(BindingConstraint::NoAccessibleAntecedent);
            return binding;
        }

        // Score all candidates
        let mut scored: Vec<_> = candidates
            .iter()
            .map(|r| {
                let score = self.score_candidate(r, anaphor_type, current_sentence);
                (r.id, score)
            })
            .collect();

        // Sort by score descending
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Filter out very low scores (below half of min_confidence)
        let min_threshold = self.min_confidence * 0.5;
        scored.retain(|(_, score)| *score >= min_threshold);

        let requires_resolution = Self::requires_resolution(anaphor_type);
        let mut binding = UnderspecBinding::new(scored, anaphor_type, requires_resolution);

        // Set preferred if the best candidate is significantly better
        if let Some((best_id, best_score)) = binding.candidates.first().copied() {
            if best_score >= self.min_confidence {
                // Check if best is significantly better than second-best
                if let Some((_, second_score)) = binding.candidates.get(1) {
                    // If best is at least 50% more confident, prefer it
                    if best_score >= second_score * 1.5 {
                        binding.preferred = Some(best_id);
                    }
                } else {
                    // Only one candidate, it's preferred
                    binding.preferred = Some(best_id);
                }
            }
        }

        binding
    }

    /// Determine if an anaphor type requires resolution.
    ///
    /// Reflexives must be resolved (Condition A).
    /// Personal pronouns can often remain ambiguous.
    #[must_use]
    const fn requires_resolution(anaphor_type: AnaphorType) -> bool {
        match anaphor_type {
            // Must be bound in local domain / must have antecedent
            AnaphorType::Reflexive | AnaphorType::Relative => true,
            // Can remain ambiguous
            AnaphorType::Personal | AnaphorType::Possessive | AnaphorType::Demonstrative => false,
        }
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

    // === UnderspecBinding Tests ===

    #[test]
    fn test_underspec_binding_creation() {
        let candidates = vec![(ReferentId::new(0), 0.9), (ReferentId::new(1), 0.7)];
        let binding = UnderspecBinding::new(candidates, AnaphorType::Personal, false);

        assert_eq!(binding.candidate_count(), 2);
        assert!(!binding.requires_resolution);
        assert!(binding.is_ambiguous()); // Multiple candidates, no preferred
    }

    #[test]
    fn test_underspec_binding_single_candidate_preferred() {
        let candidates = vec![(ReferentId::new(0), 0.9)];
        let binding = UnderspecBinding::new(candidates, AnaphorType::Personal, false);

        assert_eq!(binding.candidate_count(), 1);
        assert!(!binding.is_ambiguous()); // Single candidate is preferred
        assert_eq!(binding.preferred, Some(ReferentId::new(0)));
    }

    #[test]
    fn test_underspec_binding_best_candidate() {
        let candidates = vec![
            (ReferentId::new(0), 0.9),
            (ReferentId::new(1), 0.7),
            (ReferentId::new(2), 0.5),
        ];
        let binding = UnderspecBinding::new(candidates, AnaphorType::Personal, false);

        let best = binding.best_candidate();
        assert!(best.is_some());
        let (id, score) = best.unwrap();
        assert_eq!(id, ReferentId::new(0));
        assert!((score - 0.9).abs() < f32::EPSILON);
    }

    #[test]
    fn test_underspec_binding_candidates_above_threshold() {
        let candidates = vec![
            (ReferentId::new(0), 0.9),
            (ReferentId::new(1), 0.7),
            (ReferentId::new(2), 0.3),
        ];
        let binding = UnderspecBinding::new(candidates, AnaphorType::Personal, false);

        let above = binding.candidates_above(0.5);
        assert_eq!(above.len(), 2);
    }

    #[test]
    fn test_underspec_binding_to_resolved() {
        let candidates = vec![(ReferentId::new(0), 0.9), (ReferentId::new(1), 0.7)];
        let binding = UnderspecBinding::new(candidates, AnaphorType::Personal, false);

        let resolved = binding.to_resolved();
        assert!(resolved.is_resolved());
        assert_eq!(resolved.antecedent, Some(ReferentId::new(0)));
        assert!((resolved.confidence - 0.9).abs() < f32::EPSILON);
        assert_eq!(resolved.candidates.len(), 2);
    }

    #[test]
    fn test_underspec_binding_set_preferred() {
        let candidates = vec![(ReferentId::new(0), 0.9), (ReferentId::new(1), 0.7)];
        let mut binding = UnderspecBinding::new(candidates, AnaphorType::Personal, false);

        assert!(binding.is_ambiguous());
        binding.set_preferred(ReferentId::new(1));
        assert!(!binding.is_ambiguous());
        assert_eq!(binding.preferred, Some(ReferentId::new(1)));
    }

    #[test]
    fn test_underspec_binding_has_candidate() {
        let candidates = vec![(ReferentId::new(0), 0.9), (ReferentId::new(1), 0.7)];
        let binding = UnderspecBinding::new(candidates, AnaphorType::Personal, false);

        assert!(binding.has_candidate(ReferentId::new(0)));
        assert!(binding.has_candidate(ReferentId::new(1)));
        assert!(!binding.has_candidate(ReferentId::new(2)));
    }

    #[test]
    fn test_underspec_binding_score_for() {
        let candidates = vec![(ReferentId::new(0), 0.9), (ReferentId::new(1), 0.7)];
        let binding = UnderspecBinding::new(candidates, AnaphorType::Personal, false);

        assert_eq!(binding.score_for(ReferentId::new(0)), Some(0.9));
        assert_eq!(binding.score_for(ReferentId::new(1)), Some(0.7));
        assert_eq!(binding.score_for(ReferentId::new(2)), None);
    }

    // === resolve_underspec Tests ===

    #[test]
    fn test_resolve_underspec_single_candidate() {
        let mut registry = ReferentRegistry::new();
        let id = registry.introduce_entity("John");
        if let Some(r) = registry.get_mut(id) {
            r.gender = Gender::Masculine;
            r.salience = 0.9;
        }

        let resolver = PronounResolver::new();
        let binding = resolver.resolve_underspec(
            &registry,
            AnaphorType::Personal,
            Some(Gender::Masculine),
            None,
            0,
        );

        assert_eq!(binding.candidate_count(), 1);
        assert!(!binding.is_ambiguous());
        assert_eq!(binding.preferred, Some(id));
        assert!(!binding.requires_resolution); // Personal pronouns don't require
    }

    #[test]
    fn test_resolve_underspec_multiple_candidates() {
        let mut registry = ReferentRegistry::new();

        // John (masculine, high salience)
        let john_id = registry.introduce_entity("John");
        if let Some(r) = registry.get_mut(john_id) {
            r.gender = Gender::Masculine;
            r.salience = 0.9;
        }

        // Bill (masculine, similar salience - creates ambiguity)
        let bill_id = registry.introduce_entity("Bill");
        if let Some(r) = registry.get_mut(bill_id) {
            r.gender = Gender::Masculine;
            r.salience = 0.85;
        }

        let resolver = PronounResolver::new();
        let binding = resolver.resolve_underspec(
            &registry,
            AnaphorType::Personal,
            Some(Gender::Masculine),
            None,
            0,
        );

        assert_eq!(binding.candidate_count(), 2);
        // With similar salience, no clear preferred
        assert!(binding.is_ambiguous());
    }

    #[test]
    fn test_resolve_underspec_reflexive_requires_resolution() {
        let mut registry = ReferentRegistry::new();
        let id = registry.introduce_entity("John");
        if let Some(r) = registry.get_mut(id) {
            r.gender = Gender::Masculine;
            r.salience = 0.9;
        }

        let resolver = PronounResolver::new();
        let binding = resolver.resolve_underspec(
            &registry,
            AnaphorType::Reflexive,
            Some(Gender::Masculine),
            None,
            0,
        );

        assert!(binding.requires_resolution); // Reflexives must be bound
    }

    #[test]
    fn test_resolve_underspec_no_candidates() {
        let registry = ReferentRegistry::new(); // Empty registry

        let resolver = PronounResolver::new();
        let binding = resolver.resolve_underspec(
            &registry,
            AnaphorType::Personal,
            Some(Gender::Masculine),
            None,
            0,
        );

        assert_eq!(binding.candidate_count(), 0);
        assert!(binding
            .constraints
            .contains(&BindingConstraint::NoAccessibleAntecedent));
    }

    #[test]
    fn test_resolve_underspec_preferred_when_dominant() {
        let mut registry = ReferentRegistry::new();

        // John (masculine, very high salience)
        let john_id = registry.introduce_entity("John");
        if let Some(r) = registry.get_mut(john_id) {
            r.gender = Gender::Masculine;
            r.salience = 0.95;
        }

        // Bill (masculine, much lower salience)
        let bill_id = registry.introduce_entity("Bill");
        if let Some(r) = registry.get_mut(bill_id) {
            r.gender = Gender::Masculine;
            r.salience = 0.3;
        }

        let resolver = PronounResolver::new();
        let binding = resolver.resolve_underspec(
            &registry,
            AnaphorType::Personal,
            Some(Gender::Masculine),
            None,
            0,
        );

        // John should be preferred as he's significantly more salient
        assert!(!binding.is_ambiguous());
        assert_eq!(binding.preferred, Some(john_id));
    }
}
