//! Core types for underspecified semantic representations.

use std::collections::HashMap;

use crate::runtime::{SenseId, TokenId};

use super::scope::ScopeUnderspec;
use crate::kernel::discourse::{ReferentId, UnderspecBinding};
use crate::kernel::incremental::Surprisal;

/// Unique identifier for a choice point.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChoiceId(pub u32);

impl ChoiceId {
    /// Create a new choice ID.
    #[must_use]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }
}

/// Unique identifier for a reading.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReadingId(pub u32);

impl ReadingId {
    /// Create a new reading ID.
    #[must_use]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }
}

/// An alternative at a choice point.
#[derive(Debug, Clone)]
pub struct Alternative {
    /// Index of this alternative within the choice point.
    pub index: usize,

    /// Probability of this alternative (from LM or provider).
    pub probability: f64,

    /// Description for debugging/display.
    pub description: String,
}

impl Alternative {
    /// Create a new alternative.
    #[must_use]
    pub fn new(index: usize, probability: f64, description: impl Into<String>) -> Self {
        Self {
            index,
            probability,
            description: description.into(),
        }
    }
}

/// Type of ambiguity at a choice point.
#[derive(Debug, Clone)]
pub enum ChoiceType {
    /// Lexical ambiguity: multiple word senses.
    ///
    /// Example: "bank" (financial institution vs. river bank)
    LexicalSense {
        /// Token with the ambiguous word.
        token_id: TokenId,
        /// Possible senses.
        senses: Vec<SenseId>,
    },

    /// Structural ambiguity: attachment options.
    ///
    /// Example: "I saw the man with the telescope"
    /// - PP "with the telescope" attaches to "saw" (instrument)
    /// - PP "with the telescope" attaches to "man" (possession)
    Attachment {
        /// The modifier/PP being attached.
        modifier: TokenId,
        /// Possible attachment heads.
        heads: Vec<TokenId>,
    },

    /// Scope ambiguity: quantifier/operator scope.
    ///
    /// Example: "Every student read a book"
    /// - ∀x.∃y. (∀ > ∃): each student read possibly different books
    /// - ∃y.∀x. (∃ > ∀): there's one book all students read
    Scope {
        /// Scope-bearing elements involved.
        operators: Vec<ScopeBearingElement>,
    },

    /// Referential ambiguity: pronoun antecedent candidates.
    ///
    /// Example: "John told Bill he was tired"
    /// - "he" = John
    /// - "he" = Bill
    Reference {
        /// The anaphor (pronoun).
        anaphor: ReferentId,
        /// Possible antecedents.
        candidates: Vec<ReferentId>,
    },
}

/// A scope-bearing element (quantifier, negation, modal, etc.).
#[derive(Debug, Clone)]
pub struct ScopeBearingElement {
    /// Token introducing this operator.
    pub token_id: TokenId,
    /// Type of scope-bearing element.
    pub operator_type: ScopeOperatorType,
    /// Restriction (if quantifier).
    pub restriction: Option<String>,
}

/// Types of scope-bearing operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeOperatorType {
    /// Universal quantifier (every, all, each).
    Universal,
    /// Existential quantifier (a, some, several).
    Existential,
    /// Negation (not, never, no).
    Negation,
    /// Modal (can, must, might).
    Modal,
    /// Other scope-taking element.
    Other,
}

/// A choice point where readings diverge.
#[derive(Debug, Clone)]
pub struct ChoicePoint {
    /// Unique identifier.
    pub id: ChoiceId,
    /// Type of choice/ambiguity.
    pub choice_type: ChoiceType,
    /// Available alternatives.
    pub alternatives: Vec<Alternative>,
    /// Default alternative (if any preference).
    pub default_idx: Option<usize>,
}

impl ChoicePoint {
    /// Create a new choice point.
    #[must_use]
    pub fn new(id: ChoiceId, choice_type: ChoiceType, alternatives: Vec<Alternative>) -> Self {
        Self {
            id,
            choice_type,
            alternatives,
            default_idx: None,
        }
    }

    /// Set the default alternative.
    #[must_use]
    pub fn with_default(mut self, idx: usize) -> Self {
        self.default_idx = Some(idx);
        self
    }

    /// Get the number of alternatives.
    #[must_use]
    pub fn alternative_count(&self) -> usize {
        self.alternatives.len()
    }

    /// Check if this choice point is trivial (single alternative).
    #[must_use]
    pub fn is_trivial(&self) -> bool {
        self.alternatives.len() <= 1
    }
}

/// Constraint between choices.
#[derive(Debug, Clone)]
pub struct SemanticConstraint {
    /// Choice points involved in this constraint.
    pub choice_ids: Vec<ChoiceId>,
    /// Constraint type.
    pub constraint_type: ConstraintType,
}

/// Types of semantic constraints.
#[derive(Debug, Clone)]
pub enum ConstraintType {
    /// Choices must have the same value.
    SameChoice,
    /// Choices must have different values.
    DifferentChoice,
    /// Custom constraint function.
    Custom(String),
}

/// Shared structure common to all readings.
#[derive(Debug, Clone, Default)]
pub struct SharedStructure {
    /// Text being analyzed.
    pub text: String,
    /// Token count.
    pub token_count: usize,
    /// Predicate positions.
    pub predicate_positions: Vec<TokenId>,
}

/// Packed semantic representation.
///
/// Shares structure across readings using choice points,
/// achieving O(n) memory instead of O(2^n) for explicit enumeration.
#[derive(Debug, Clone)]
pub struct PackedSemantics {
    /// Shared structure across all readings.
    pub shared: SharedStructure,

    /// Choice points where readings diverge.
    pub choices: Vec<ChoicePoint>,

    /// Constraints between choices.
    pub constraints: Vec<SemanticConstraint>,

    /// Scope underspecification (MRS-style).
    pub scope_underspec: Option<ScopeUnderspec>,
}

impl PackedSemantics {
    /// Create a new packed semantics.
    #[must_use]
    pub fn new(shared: SharedStructure) -> Self {
        Self {
            shared,
            choices: Vec::new(),
            constraints: Vec::new(),
            scope_underspec: None,
        }
    }

    /// Add a choice point.
    pub fn add_choice(&mut self, choice: ChoicePoint) {
        self.choices.push(choice);
    }

    /// Add a semantic constraint.
    pub fn add_constraint(&mut self, constraint: SemanticConstraint) {
        self.constraints.push(constraint);
    }

    /// Set scope underspecification.
    pub fn set_scope_underspec(&mut self, scope: ScopeUnderspec) {
        self.scope_underspec = Some(scope);
    }

    /// Add a referential ambiguity from an underspecified binding.
    ///
    /// Creates a choice point representing pronoun-antecedent ambiguity.
    /// Each candidate antecedent becomes an alternative with its score
    /// converted to probability.
    ///
    /// # Arguments
    /// * `anaphor_id` - Referent ID for the anaphor (pronoun)
    /// * `binding` - The underspecified binding with candidates
    ///
    /// # Returns
    /// The choice ID for the newly created choice point, or None if no ambiguity.
    ///
    /// # Panics
    /// Panics if the number of choices exceeds `u32::MAX`.
    pub fn add_referential_ambiguity(
        &mut self,
        anaphor_id: ReferentId,
        binding: &UnderspecBinding,
    ) -> Option<ChoiceId> {
        // No ambiguity if single candidate or no candidates
        if binding.candidate_count() <= 1 {
            return None;
        }

        let choice_id = ChoiceId::new(
            u32::try_from(self.choices.len()).expect("choice count exceeds u32::MAX"),
        );

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
                anaphor: anaphor_id,
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

        self.choices.push(choice_point);
        Some(choice_id)
    }

    /// Compute the total number of readings (product of alternatives).
    ///
    /// Warning: This can be exponential! Use with caution.
    #[must_use]
    pub fn reading_count(&self) -> usize {
        if self.choices.is_empty() {
            return 1;
        }

        self.choices
            .iter()
            .filter(|c| !c.is_trivial())
            .map(ChoicePoint::alternative_count)
            .product()
    }

    /// Check if there's any ambiguity.
    #[must_use]
    pub fn is_ambiguous(&self) -> bool {
        self.choices.iter().any(|c| c.alternative_count() > 1)
    }

    /// Get summary of ambiguity types.
    #[must_use]
    pub fn ambiguity_summary(&self) -> AmbiguitySummary {
        let mut summary = AmbiguitySummary::default();

        for choice in &self.choices {
            if choice.alternative_count() <= 1 {
                continue;
            }

            match &choice.choice_type {
                ChoiceType::LexicalSense { .. } => summary.lexical += 1,
                ChoiceType::Attachment { .. } => summary.structural += 1,
                ChoiceType::Scope { .. } => summary.scope += 1,
                ChoiceType::Reference { .. } => summary.referential += 1,
            }
        }

        summary.total_readings = self.reading_count();
        summary
    }

    /// Create an iterator over all readings.
    #[must_use]
    pub fn readings(&self) -> ReadingsIterator<'_> {
        ReadingsIterator::new(self)
    }

    /// Get the best reading (highest probability).
    ///
    /// # Panics
    /// Panics if probability contains NaN values.
    #[must_use]
    pub fn best_reading(&self) -> Option<Reading> {
        self.readings()
            .max_by(|a, b| a.probability.partial_cmp(&b.probability).unwrap())
    }
}

impl Default for PackedSemantics {
    fn default() -> Self {
        Self::new(SharedStructure::default())
    }
}

/// A single fully-resolved interpretation.
#[derive(Debug, Clone)]
pub struct Reading {
    /// Unique identifier.
    pub id: ReadingId,

    /// Choices made at each choice point.
    pub choices: HashMap<ChoiceId, usize>,

    /// Probability of this reading P(reading | sentence).
    pub probability: f64,

    /// Total surprisal (sum across words).
    pub total_surprisal: Surprisal,

    /// Legacy confidence from providers.
    pub confidence: f32,
}

impl Reading {
    /// Create a new reading.
    #[must_use]
    pub fn new(id: ReadingId, choices: HashMap<ChoiceId, usize>, probability: f64) -> Self {
        Self {
            id,
            choices,
            probability,
            total_surprisal: Surprisal::ZERO,
            confidence: 1.0,
        }
    }

    /// Set total surprisal.
    #[must_use]
    pub fn with_surprisal(mut self, surprisal: Surprisal) -> Self {
        self.total_surprisal = surprisal;
        self
    }

    /// Set confidence.
    #[must_use]
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence;
        self
    }

    /// Get the choice made at a specific choice point.
    #[must_use]
    pub fn choice_at(&self, choice_id: ChoiceId) -> Option<usize> {
        self.choices.get(&choice_id).copied()
    }
}

/// Summary of ambiguity in a packed representation.
#[derive(Debug, Clone, Default)]
pub struct AmbiguitySummary {
    /// Number of lexical ambiguity points.
    pub lexical: usize,
    /// Number of structural ambiguity points.
    pub structural: usize,
    /// Number of scope ambiguity points.
    pub scope: usize,
    /// Number of referential ambiguity points.
    pub referential: usize,
    /// Total number of readings.
    pub total_readings: usize,
}

impl AmbiguitySummary {
    /// Check if there's any ambiguity.
    #[must_use]
    pub fn is_ambiguous(&self) -> bool {
        self.total_readings > 1
    }

    /// Get total number of ambiguity points.
    #[must_use]
    pub fn total_points(&self) -> usize {
        self.lexical + self.structural + self.scope + self.referential
    }
}

/// Iterator over readings in a packed representation.
pub struct ReadingsIterator<'a> {
    packed: &'a PackedSemantics,
    /// Current state: index at each choice point.
    indices: Vec<usize>,
    /// Whether we've exhausted all readings.
    done: bool,
    /// Next reading ID.
    next_id: u32,
}

impl<'a> ReadingsIterator<'a> {
    fn new(packed: &'a PackedSemantics) -> Self {
        let indices = vec![0; packed.choices.len()];
        let done = packed.choices.iter().any(|c| c.alternatives.is_empty());

        Self {
            packed,
            indices,
            done,
            next_id: 0,
        }
    }

    fn current_reading(&self) -> Reading {
        let id = ReadingId::new(self.next_id);

        let mut choices = HashMap::new();
        let mut probability = 1.0;

        for (i, choice) in self.packed.choices.iter().enumerate() {
            let alt_idx = self.indices[i];
            choices.insert(choice.id, alt_idx);

            if let Some(alt) = choice.alternatives.get(alt_idx) {
                probability *= alt.probability;
            }
        }

        Reading::new(id, choices, probability)
    }

    fn advance(&mut self) {
        // Increment indices like a multi-digit counter
        for i in (0..self.indices.len()).rev() {
            self.indices[i] += 1;
            if self.indices[i] < self.packed.choices[i].alternatives.len() {
                return; // Successfully incremented
            }
            self.indices[i] = 0; // Carry to next position
        }
        // If we get here, we've exhausted all combinations
        self.done = true;
    }
}

impl Iterator for ReadingsIterator<'_> {
    type Item = Reading;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        let reading = self.current_reading();
        self.next_id += 1;
        self.advance();

        Some(reading)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.done {
            return (0, Some(0));
        }

        let remaining = self.packed.reading_count();
        (remaining, Some(remaining))
    }
}

/// Trait for accessing readings from a packed representation.
pub trait ReadingsAccess {
    /// Check if there are multiple readings.
    fn is_ambiguous(&self) -> bool;

    /// Get the number of readings.
    fn reading_count(&self) -> usize;

    /// Get the best reading by probability.
    fn best_reading(&self) -> Option<Reading>;

    /// Get summary of ambiguity types.
    fn ambiguity_summary(&self) -> AmbiguitySummary;
}

impl ReadingsAccess for PackedSemantics {
    fn is_ambiguous(&self) -> bool {
        PackedSemantics::is_ambiguous(self)
    }

    fn reading_count(&self) -> usize {
        PackedSemantics::reading_count(self)
    }

    fn best_reading(&self) -> Option<Reading> {
        PackedSemantics::best_reading(self)
    }

    fn ambiguity_summary(&self) -> AmbiguitySummary {
        PackedSemantics::ambiguity_summary(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_choice_id_creation() {
        let id = ChoiceId::new(42);
        assert_eq!(id.0, 42);
    }

    #[test]
    fn test_alternative_creation() {
        let alt = Alternative::new(0, 0.6, "financial institution");
        assert_eq!(alt.index, 0);
        assert!((alt.probability - 0.6).abs() < f64::EPSILON);
        assert_eq!(alt.description, "financial institution");
    }

    #[test]
    fn test_choice_point_creation() {
        let choice = ChoicePoint::new(
            ChoiceId::new(0),
            ChoiceType::LexicalSense {
                token_id: TokenId::new(1),
                senses: vec![SenseId::new("bank.01"), SenseId::new("bank.02")],
            },
            vec![
                Alternative::new(0, 0.6, "financial"),
                Alternative::new(1, 0.4, "river"),
            ],
        );

        assert_eq!(choice.alternative_count(), 2);
        assert!(!choice.is_trivial());
    }

    #[test]
    fn test_packed_semantics_reading_count() {
        let mut packed = PackedSemantics::default();

        // Add choice with 2 alternatives
        packed.add_choice(ChoicePoint::new(
            ChoiceId::new(0),
            ChoiceType::LexicalSense {
                token_id: TokenId::new(0),
                senses: vec![],
            },
            vec![Alternative::new(0, 0.5, "a"), Alternative::new(1, 0.5, "b")],
        ));

        // Add choice with 3 alternatives
        packed.add_choice(ChoicePoint::new(
            ChoiceId::new(1),
            ChoiceType::LexicalSense {
                token_id: TokenId::new(1),
                senses: vec![],
            },
            vec![
                Alternative::new(0, 0.33, "x"),
                Alternative::new(1, 0.33, "y"),
                Alternative::new(2, 0.34, "z"),
            ],
        ));

        // 2 * 3 = 6 readings
        assert_eq!(packed.reading_count(), 6);
        assert!(packed.is_ambiguous());
    }

    #[test]
    fn test_readings_iterator() {
        let mut packed = PackedSemantics::default();

        packed.add_choice(ChoicePoint::new(
            ChoiceId::new(0),
            ChoiceType::LexicalSense {
                token_id: TokenId::new(0),
                senses: vec![],
            },
            vec![Alternative::new(0, 0.6, "a"), Alternative::new(1, 0.4, "b")],
        ));

        packed.add_choice(ChoicePoint::new(
            ChoiceId::new(1),
            ChoiceType::LexicalSense {
                token_id: TokenId::new(1),
                senses: vec![],
            },
            vec![Alternative::new(0, 0.7, "x"), Alternative::new(1, 0.3, "y")],
        ));

        let readings: Vec<_> = packed.readings().collect();
        assert_eq!(readings.len(), 4);

        // Check first reading (0, 0) has probability 0.6 * 0.7 = 0.42
        let first = &readings[0];
        assert!((first.probability - 0.42).abs() < 0.001);
    }

    #[test]
    fn test_ambiguity_summary() {
        let mut packed = PackedSemantics::default();

        // Lexical ambiguity
        packed.add_choice(ChoicePoint::new(
            ChoiceId::new(0),
            ChoiceType::LexicalSense {
                token_id: TokenId::new(0),
                senses: vec![],
            },
            vec![Alternative::new(0, 0.5, "a"), Alternative::new(1, 0.5, "b")],
        ));

        // Referential ambiguity
        packed.add_choice(ChoicePoint::new(
            ChoiceId::new(1),
            ChoiceType::Reference {
                anaphor: ReferentId::new(0),
                candidates: vec![ReferentId::new(1), ReferentId::new(2)],
            },
            vec![Alternative::new(0, 0.5, "x"), Alternative::new(1, 0.5, "y")],
        ));

        let summary = packed.ambiguity_summary();
        assert_eq!(summary.lexical, 1);
        assert_eq!(summary.referential, 1);
        assert_eq!(summary.structural, 0);
        assert_eq!(summary.scope, 0);
        assert_eq!(summary.total_readings, 4);
    }

    #[test]
    fn test_best_reading() {
        let mut packed = PackedSemantics::default();

        packed.add_choice(ChoicePoint::new(
            ChoiceId::new(0),
            ChoiceType::LexicalSense {
                token_id: TokenId::new(0),
                senses: vec![],
            },
            vec![
                Alternative::new(0, 0.8, "high"),
                Alternative::new(1, 0.2, "low"),
            ],
        ));

        let best = packed.best_reading().expect("Should have a reading");
        assert!((best.probability - 0.8).abs() < 0.001);
        assert_eq!(best.choice_at(ChoiceId::new(0)), Some(0));
    }

    #[test]
    fn test_empty_packed_semantics() {
        let packed = PackedSemantics::default();
        assert_eq!(packed.reading_count(), 1);
        assert!(!packed.is_ambiguous());

        let readings: Vec<_> = packed.readings().collect();
        assert_eq!(readings.len(), 1);
    }

    #[test]
    fn test_add_referential_ambiguity() {
        use crate::kernel::discourse::AnaphorType;

        let mut packed = PackedSemantics::default();

        // Create an underspec binding with multiple candidates
        let binding = UnderspecBinding::new(
            vec![(ReferentId::new(1), 0.8), (ReferentId::new(2), 0.6)],
            AnaphorType::Personal,
            false,
        );

        let choice_id = packed.add_referential_ambiguity(ReferentId::new(0), &binding);
        assert!(choice_id.is_some());

        assert_eq!(packed.reading_count(), 2);
        assert!(packed.is_ambiguous());

        let summary = packed.ambiguity_summary();
        assert_eq!(summary.referential, 1);
    }

    #[test]
    fn test_add_referential_ambiguity_single_candidate() {
        use crate::kernel::discourse::AnaphorType;

        let mut packed = PackedSemantics::default();

        // Single candidate - no ambiguity
        let binding = UnderspecBinding::new(
            vec![(ReferentId::new(1), 0.9)],
            AnaphorType::Personal,
            false,
        );

        let choice_id = packed.add_referential_ambiguity(ReferentId::new(0), &binding);
        assert!(choice_id.is_none()); // No ambiguity added

        assert_eq!(packed.reading_count(), 1);
        assert!(!packed.is_ambiguous());
    }

    #[test]
    fn test_packed_memory_efficiency() {
        // Verify packed representation is O(n) not O(2^n)
        // 10 binary choices = 1024 readings, but only 10 choice points stored
        let mut packed = PackedSemantics::default();

        for i in 0u32..10 {
            packed.add_choice(ChoicePoint::new(
                ChoiceId::new(i),
                ChoiceType::LexicalSense {
                    token_id: TokenId::new(i as usize),
                    senses: vec![],
                },
                vec![
                    Alternative::new(0, 0.6, format!("sense_{i}_a")),
                    Alternative::new(1, 0.4, format!("sense_{i}_b")),
                ],
            ));
        }

        // 1024 readings (2^10)
        assert_eq!(packed.reading_count(), 1024);

        // But only 10 choice points stored (O(n) memory)
        assert_eq!(packed.choices.len(), 10);
    }

    #[test]
    fn test_lazy_enumeration_efficiency() {
        // Create packed with 8 binary choices (256 readings)
        let mut packed = PackedSemantics::default();

        for i in 0u32..8 {
            packed.add_choice(ChoicePoint::new(
                ChoiceId::new(i),
                ChoiceType::LexicalSense {
                    token_id: TokenId::new(i as usize),
                    senses: vec![],
                },
                vec![
                    Alternative::new(0, 0.6, format!("a{i}")),
                    Alternative::new(1, 0.4, format!("b{i}")),
                ],
            ));
        }

        assert_eq!(packed.reading_count(), 256);

        // Enumerate only first 10 readings (lazy)
        let first_10: Vec<_> = packed.readings().take(10).collect();
        assert_eq!(first_10.len(), 10);

        // All readings are unique
        let ids: std::collections::HashSet<_> = first_10.iter().map(|r| r.id).collect();
        assert_eq!(ids.len(), 10);
    }

    #[test]
    fn test_unambiguous_no_overhead() {
        // Unambiguous sentence should have minimal overhead
        let packed = PackedSemantics::default();

        assert_eq!(packed.reading_count(), 1);
        assert!(!packed.is_ambiguous());

        // Single reading enumeration
        let readings: Vec<_> = packed.readings().collect();
        assert_eq!(readings.len(), 1);
        assert_eq!(readings[0].id, ReadingId::new(0));
    }

    #[test]
    fn test_high_ambiguity_count() {
        // Test with many readings (up to 1000)
        let mut packed = PackedSemantics::default();

        // 3 choices with 10 alternatives each = 1000 readings
        for i in 0u32..3 {
            let alts: Vec<_> = (0..10)
                .map(|j| Alternative::new(j, 0.1, format!("alt_{i}_{j}")))
                .collect();

            packed.add_choice(ChoicePoint::new(
                ChoiceId::new(i),
                ChoiceType::LexicalSense {
                    token_id: TokenId::new(i as usize),
                    senses: vec![],
                },
                alts,
            ));
        }

        assert_eq!(packed.reading_count(), 1000);

        // Can enumerate all without memory explosion
        let count = packed.readings().count();
        assert_eq!(count, 1000);
    }

    #[test]
    fn test_add_referential_ambiguity_with_preferred() {
        use crate::kernel::discourse::AnaphorType;

        let mut packed = PackedSemantics::default();

        // Create binding with preferred candidate
        let mut binding = UnderspecBinding::new(
            vec![(ReferentId::new(1), 0.9), (ReferentId::new(2), 0.7)],
            AnaphorType::Personal,
            false,
        );
        binding.set_preferred(ReferentId::new(2)); // Prefer second candidate

        let choice_id = packed.add_referential_ambiguity(ReferentId::new(0), &binding);
        assert!(choice_id.is_some());

        // Check that default was set correctly
        let choice = &packed.choices[0];
        assert_eq!(choice.default_idx, Some(1)); // Index of ReferentId(2)
    }
}
