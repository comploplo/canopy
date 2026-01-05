//! Pattern types for semantic-aware dependency matching.
//!
//! These types enable matching parsed syntax against patterns from the
//! UD English-EWT treebank, with VerbNet-aware synthesis for unknown verbs.

use canopy::core::{DepRel, ThetaRole, UPos};
use std::hash::{Hash, Hasher};

/// Position of an argument relative to the verb.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArgumentPosition {
    /// Before the verb (e.g., subject in English)
    PreVerbal,
    /// After the verb (e.g., object in English)
    PostVerbal,
    /// Position varies or is unspecified
    Flexible,
}

/// Pattern for a single argument in a verb's argument structure.
#[derive(Debug, Clone, PartialEq)]
pub struct ArgumentPattern {
    /// The dependency relation to the verb
    pub dep_rel: DepRel,
    /// Suggested theta role based on UTAH mapping
    pub role_hint: Option<ThetaRole>,
    /// Expected position relative to verb
    pub position: ArgumentPosition,
    /// Whether this argument is required (vs optional)
    pub required: bool,
}

impl ArgumentPattern {
    /// Create a required argument pattern.
    #[must_use]
    pub fn required(dep_rel: DepRel, role_hint: ThetaRole, position: ArgumentPosition) -> Self {
        Self {
            dep_rel,
            role_hint: Some(role_hint),
            position,
            required: true,
        }
    }

    /// Create an optional argument pattern.
    #[must_use]
    pub fn optional(dep_rel: DepRel, role_hint: ThetaRole, position: ArgumentPosition) -> Self {
        Self {
            dep_rel,
            role_hint: Some(role_hint),
            position,
            required: false,
        }
    }

    /// Create a pattern without a role hint.
    #[must_use]
    pub fn dep_only(dep_rel: DepRel, position: ArgumentPosition) -> Self {
        Self {
            dep_rel,
            role_hint: None,
            position,
            required: false,
        }
    }
}

/// A dependency pattern for a verb, extracted from treebank or synthesized.
#[derive(Debug, Clone, PartialEq)]
pub struct DependencyPattern {
    /// The verb lemma this pattern is for
    pub verb_lemma: String,
    /// `VerbNet` class if known (e.g., "give-13.1")
    pub verbnet_class: Option<String>,
    /// Expected argument patterns
    pub arguments: Vec<ArgumentPattern>,
    /// How often this pattern was seen in treebank
    pub frequency: u32,
    /// Confidence score (0.0-1.0)
    pub confidence: f32,
}

impl DependencyPattern {
    /// Create a new dependency pattern.
    #[must_use]
    pub fn new(verb_lemma: impl Into<String>, arguments: Vec<ArgumentPattern>) -> Self {
        Self {
            verb_lemma: verb_lemma.into(),
            verbnet_class: None,
            arguments,
            frequency: 1,
            confidence: 0.5,
        }
    }

    /// Set the `VerbNet` class.
    #[must_use]
    pub fn with_verbnet_class(mut self, class: impl Into<String>) -> Self {
        self.verbnet_class = Some(class.into());
        self
    }

    /// Set the frequency.
    #[must_use]
    pub fn with_frequency(mut self, freq: u32) -> Self {
        self.frequency = freq;
        self
    }

    /// Set the confidence.
    #[must_use]
    pub fn with_confidence(mut self, conf: f32) -> Self {
        self.confidence = conf;
        self
    }

    /// Get the required arguments.
    pub fn required_arguments(&self) -> impl Iterator<Item = &ArgumentPattern> {
        self.arguments.iter().filter(|a| a.required)
    }

    /// Get the optional arguments.
    pub fn optional_arguments(&self) -> impl Iterator<Item = &ArgumentPattern> {
        self.arguments.iter().filter(|a| !a.required)
    }

    /// Check if this pattern expects a given dependency relation.
    #[must_use]
    pub fn expects_dep(&self, dep_rel: &DepRel) -> bool {
        self.arguments.iter().any(|a| a.dep_rel == *dep_rel)
    }

    /// Get the role hint for a dependency relation.
    #[must_use]
    pub fn role_for_dep(&self, dep_rel: &DepRel) -> Option<ThetaRole> {
        self.arguments
            .iter()
            .find(|a| a.dep_rel == *dep_rel)
            .and_then(|a| a.role_hint)
    }

    /// Create a basic intransitive pattern (NP V).
    #[must_use]
    pub fn intransitive(verb_lemma: impl Into<String>) -> Self {
        Self::new(
            verb_lemma,
            vec![ArgumentPattern::required(
                DepRel::Nsubj,
                ThetaRole::Agent,
                ArgumentPosition::PreVerbal,
            )],
        )
    }

    /// Create a basic transitive pattern (NP V NP).
    #[must_use]
    pub fn transitive(verb_lemma: impl Into<String>) -> Self {
        Self::new(
            verb_lemma,
            vec![
                ArgumentPattern::required(
                    DepRel::Nsubj,
                    ThetaRole::Agent,
                    ArgumentPosition::PreVerbal,
                ),
                ArgumentPattern::required(
                    DepRel::Obj,
                    ThetaRole::Patient,
                    ArgumentPosition::PostVerbal,
                ),
            ],
        )
    }

    /// Create a ditransitive pattern (NP V NP NP/PP).
    #[must_use]
    pub fn ditransitive(verb_lemma: impl Into<String>) -> Self {
        Self::new(
            verb_lemma,
            vec![
                ArgumentPattern::required(
                    DepRel::Nsubj,
                    ThetaRole::Agent,
                    ArgumentPosition::PreVerbal,
                ),
                ArgumentPattern::required(
                    DepRel::Obj,
                    ThetaRole::Theme,
                    ArgumentPosition::PostVerbal,
                ),
                ArgumentPattern::optional(
                    DepRel::Iobj,
                    ThetaRole::Recipient,
                    ArgumentPosition::PostVerbal,
                ),
            ],
        )
    }
}

/// Semantic signature for pattern matching lookup.
///
/// This is used as a cache key and for matching against patterns.
#[derive(Debug, Clone)]
pub struct SemanticSignature {
    /// The verb lemma
    pub lemma: String,
    /// Part of speech (should be Verb)
    pub pos: UPos,
    /// `VerbNet` class if known
    pub verbnet_class: Option<String>,
}

impl SemanticSignature {
    /// Create a new semantic signature.
    #[must_use]
    pub fn new(lemma: impl Into<String>, pos: UPos) -> Self {
        Self {
            lemma: lemma.into(),
            pos,
            verbnet_class: None,
        }
    }

    /// Create a signature with a `VerbNet` class.
    #[must_use]
    pub fn with_verbnet(lemma: impl Into<String>, class: impl Into<String>) -> Self {
        Self {
            lemma: lemma.into(),
            pos: UPos::Verb,
            verbnet_class: Some(class.into()),
        }
    }

    /// Create from a verb lemma only.
    #[must_use]
    pub fn from_lemma(lemma: impl Into<String>) -> Self {
        Self::new(lemma, UPos::Verb)
    }
}

// Implement Hash and Eq for cache keying.
// Note: `pos` is intentionally excluded from both Hash and Eq because:
// 1. SemanticSignature is only used for verbs (pos is always UPos::Verb)
// 2. Cache lookups are based on lemma + verbnet_class only
// 3. This keeps the Hash/Eq contract intact (equal items have equal hashes)
impl Hash for SemanticSignature {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.lemma.hash(state);
        if let Some(class) = &self.verbnet_class {
            class.hash(state);
        }
        // pos intentionally excluded - see comment above
    }
}

impl PartialEq for SemanticSignature {
    fn eq(&self, other: &Self) -> bool {
        // pos intentionally excluded - see comment above
        self.lemma == other.lemma && self.verbnet_class == other.verbnet_class
    }
}

impl Eq for SemanticSignature {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_argument_pattern_required() {
        let pattern =
            ArgumentPattern::required(DepRel::Nsubj, ThetaRole::Agent, ArgumentPosition::PreVerbal);
        assert!(pattern.required);
        assert_eq!(pattern.dep_rel, DepRel::Nsubj);
        assert_eq!(pattern.role_hint, Some(ThetaRole::Agent));
    }

    #[test]
    fn test_argument_pattern_optional() {
        let pattern =
            ArgumentPattern::optional(DepRel::Obl, ThetaRole::Location, ArgumentPosition::Flexible);
        assert!(!pattern.required);
        assert_eq!(pattern.position, ArgumentPosition::Flexible);
    }

    #[test]
    fn test_dependency_pattern_intransitive() {
        let pattern = DependencyPattern::intransitive("run");
        assert_eq!(pattern.verb_lemma, "run");
        assert_eq!(pattern.arguments.len(), 1);
        assert!(pattern.expects_dep(&DepRel::Nsubj));
        assert!(!pattern.expects_dep(&DepRel::Obj));
    }

    #[test]
    fn test_dependency_pattern_transitive() {
        let pattern = DependencyPattern::transitive("hit");
        assert_eq!(pattern.arguments.len(), 2);
        assert!(pattern.expects_dep(&DepRel::Nsubj));
        assert!(pattern.expects_dep(&DepRel::Obj));
    }

    #[test]
    fn test_dependency_pattern_ditransitive() {
        let pattern = DependencyPattern::ditransitive("give");
        assert_eq!(pattern.arguments.len(), 3);
        assert_eq!(pattern.role_for_dep(&DepRel::Obj), Some(ThetaRole::Theme));
        assert_eq!(
            pattern.role_for_dep(&DepRel::Iobj),
            Some(ThetaRole::Recipient)
        );
    }

    #[test]
    fn test_dependency_pattern_builder() {
        let pattern = DependencyPattern::transitive("eat")
            .with_verbnet_class("eat-39.1")
            .with_frequency(100)
            .with_confidence(0.9);

        assert_eq!(pattern.verbnet_class, Some("eat-39.1".to_string()));
        assert_eq!(pattern.frequency, 100);
        assert!((pattern.confidence - 0.9).abs() < f32::EPSILON);
    }

    #[test]
    fn test_semantic_signature_equality() {
        let sig1 = SemanticSignature::from_lemma("give");
        let sig2 = SemanticSignature::from_lemma("give");
        let sig3 = SemanticSignature::from_lemma("take");

        assert_eq!(sig1, sig2);
        assert_ne!(sig1, sig3);
    }

    #[test]
    fn test_semantic_signature_with_verbnet() {
        let sig1 = SemanticSignature::with_verbnet("give", "give-13.1");
        let sig2 = SemanticSignature::with_verbnet("give", "give-13.1");
        let sig3 = SemanticSignature::with_verbnet("give", "give-13.2");

        assert_eq!(sig1, sig2);
        assert_ne!(sig1, sig3); // Different class
    }

    #[test]
    fn test_semantic_signature_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(SemanticSignature::from_lemma("run"));
        set.insert(SemanticSignature::from_lemma("walk"));
        set.insert(SemanticSignature::from_lemma("run")); // Duplicate

        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_required_optional_arguments() {
        let pattern = DependencyPattern::ditransitive("give");
        let required: Vec<_> = pattern.required_arguments().collect();
        let optional: Vec<_> = pattern.optional_arguments().collect();

        assert_eq!(required.len(), 2); // nsubj, obj
        assert_eq!(optional.len(), 1); // iobj
    }
}
