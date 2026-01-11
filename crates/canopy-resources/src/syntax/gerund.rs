//! Gerund usage classification.
//!
//! Classifies gerunds (-ing forms) based on their syntactic position
//! to determine whether they function as nominal, verbal, or adjectival.

use canopy::core::{DepRel, GerundUsage, VerbForm};
use canopy::runtime::{AnnotatedSyntax, TokenId};

/// Classifies gerund usage based on syntactic position.
///
/// Distinguishes:
/// - **Nominal**: "Swimming is fun" - gerund as subject/object
/// - **Verbal**: "He is swimming" - progressive aspect with auxiliary
/// - **Adjectival**: "The boring lecture" - modifying a noun
#[derive(Debug, Default)]
pub struct GerundClassifier;

impl GerundClassifier {
    /// Create a new gerund classifier.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Classify gerund usage for a token.
    ///
    /// Returns `None` if the token is not a gerund.
    #[must_use]
    pub fn classify(&self, syntax: &AnnotatedSyntax, token_id: TokenId) -> Option<GerundUsage> {
        Self::classify_token(syntax, token_id)
    }

    /// Internal classification logic (static to allow recursion without self).
    fn classify_token(syntax: &AnnotatedSyntax, token_id: TokenId) -> Option<GerundUsage> {
        let token = syntax.get_token(token_id)?;

        // Must be a gerund form
        if token.feats.verb_form != Some(VerbForm::Gerund) {
            return None;
        }

        // Classify based on dependency relation
        match token.deprel {
            // Subject/object/oblique positions -> nominal gerund
            // "Swimming is fun", "I enjoy swimming", "I'm interested in swimming"
            DepRel::Nsubj
            | DepRel::NsubjPass
            | DepRel::Csubj
            | DepRel::CsubjPass
            | DepRel::Obj
            | DepRel::Iobj
            | DepRel::Obl => Some(GerundUsage::Nominal),

            // Modifier of noun -> adjectival
            // "The boring lecture", "The man swimming in the pool"
            DepRel::Amod | DepRel::Acl => Some(GerundUsage::Adjectival),

            // Root with auxiliary -> progressive (verbal)
            // "He is swimming"
            DepRel::Root => {
                if Self::has_auxiliary(syntax, token_id) {
                    Some(GerundUsage::Verbal)
                } else {
                    // Standalone gerund root is usually nominal
                    // "Swimming!" as exclamation
                    Some(GerundUsage::Nominal)
                }
            }

            // Conjunction - inherit from context
            DepRel::Conj => {
                // Try to get the head's classification
                if let Some(head_id) = token.head {
                    // Recursive call on head
                    Self::classify_token(syntax, head_id)
                } else {
                    Some(GerundUsage::Verbal)
                }
            }

            // Default to verbal for other cases (xcomp, advcl, etc.)
            _ => Some(GerundUsage::Verbal),
        }
    }

    /// Check if a token has an auxiliary dependent.
    fn has_auxiliary(syntax: &AnnotatedSyntax, token_id: TokenId) -> bool {
        syntax
            .tokens
            .iter()
            .any(|t| t.head == Some(token_id) && matches!(t.deprel, DepRel::Aux | DepRel::AuxPass))
    }

    /// Classify all gerunds in the syntax.
    ///
    /// Returns a list of (`TokenId`, `GerundUsage`) pairs.
    #[must_use]
    pub fn classify_all(&self, syntax: &AnnotatedSyntax) -> Vec<(TokenId, GerundUsage)> {
        syntax
            .tokens
            .iter()
            .filter_map(|t| self.classify(syntax, t.id).map(|usage| (t.id, usage)))
            .collect()
    }

    /// Check if a gerund is being used nominally.
    #[must_use]
    pub fn is_nominal(&self, syntax: &AnnotatedSyntax, token_id: TokenId) -> bool {
        self.classify(syntax, token_id) == Some(GerundUsage::Nominal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use canopy::core::{MorphFeatures, UPos};
    use canopy::runtime::AnnotatedToken;

    fn make_token(
        id: usize,
        form: &str,
        lemma: &str,
        upos: UPos,
        head: Option<usize>,
        deprel: DepRel,
        verb_form: Option<VerbForm>,
    ) -> AnnotatedToken {
        AnnotatedToken {
            id: TokenId::new(id),
            form: form.to_string(),
            lemma: lemma.to_string(),
            upos,
            xpos: None,
            feats: MorphFeatures {
                verb_form,
                ..Default::default()
            },
            head: head.map(TokenId::new),
            deprel,
            span: (0, form.len()),
        }
    }

    #[test]
    fn test_nominal_gerund_subject() {
        // "Swimming is fun"
        let syntax = AnnotatedSyntax::new(
            "Swimming is fun".to_string(),
            vec![
                make_token(
                    0,
                    "Swimming",
                    "swim",
                    UPos::Verb,
                    Some(2),
                    DepRel::Nsubj,
                    Some(VerbForm::Gerund),
                ),
                make_token(1, "is", "be", UPos::Aux, Some(2), DepRel::Cop, None),
                make_token(2, "fun", "fun", UPos::Adj, None, DepRel::Root, None),
            ],
        );

        let classifier = GerundClassifier::new();
        let usage = classifier.classify(&syntax, TokenId::new(0));

        assert_eq!(usage, Some(GerundUsage::Nominal));
    }

    #[test]
    fn test_nominal_gerund_object() {
        // "I enjoy swimming"
        let syntax = AnnotatedSyntax::new(
            "I enjoy swimming".to_string(),
            vec![
                make_token(0, "I", "I", UPos::Pron, Some(1), DepRel::Nsubj, None),
                make_token(1, "enjoy", "enjoy", UPos::Verb, None, DepRel::Root, None),
                make_token(
                    2,
                    "swimming",
                    "swim",
                    UPos::Verb,
                    Some(1),
                    DepRel::Obj,
                    Some(VerbForm::Gerund),
                ),
            ],
        );

        let classifier = GerundClassifier::new();
        let usage = classifier.classify(&syntax, TokenId::new(2));

        assert_eq!(usage, Some(GerundUsage::Nominal));
    }

    #[test]
    fn test_verbal_gerund_progressive() {
        // "He is swimming"
        let syntax = AnnotatedSyntax::new(
            "He is swimming".to_string(),
            vec![
                make_token(0, "He", "he", UPos::Pron, Some(2), DepRel::Nsubj, None),
                make_token(1, "is", "be", UPos::Aux, Some(2), DepRel::Aux, None),
                make_token(
                    2,
                    "swimming",
                    "swim",
                    UPos::Verb,
                    None,
                    DepRel::Root,
                    Some(VerbForm::Gerund),
                ),
            ],
        );

        let classifier = GerundClassifier::new();
        let usage = classifier.classify(&syntax, TokenId::new(2));

        assert_eq!(usage, Some(GerundUsage::Verbal));
    }

    #[test]
    fn test_adjectival_gerund() {
        // "The boring lecture"
        let syntax = AnnotatedSyntax::new(
            "The boring lecture".to_string(),
            vec![
                make_token(0, "The", "the", UPos::Det, Some(2), DepRel::Det, None),
                make_token(
                    1,
                    "boring",
                    "bore",
                    UPos::Verb,
                    Some(2),
                    DepRel::Amod,
                    Some(VerbForm::Gerund),
                ),
                make_token(
                    2,
                    "lecture",
                    "lecture",
                    UPos::Noun,
                    None,
                    DepRel::Root,
                    None,
                ),
            ],
        );

        let classifier = GerundClassifier::new();
        let usage = classifier.classify(&syntax, TokenId::new(1));

        assert_eq!(usage, Some(GerundUsage::Adjectival));
    }

    #[test]
    fn test_non_gerund_returns_none() {
        // "He runs" - not a gerund
        let syntax = AnnotatedSyntax::new(
            "He runs".to_string(),
            vec![
                make_token(0, "He", "he", UPos::Pron, Some(1), DepRel::Nsubj, None),
                make_token(
                    1,
                    "runs",
                    "run",
                    UPos::Verb,
                    None,
                    DepRel::Root,
                    Some(VerbForm::Finite),
                ),
            ],
        );

        let classifier = GerundClassifier::new();
        let usage = classifier.classify(&syntax, TokenId::new(1));

        assert_eq!(usage, None);
    }

    #[test]
    fn test_classify_all() {
        // "Swimming and running are fun"
        let syntax = AnnotatedSyntax::new(
            "Swimming and running are fun".to_string(),
            vec![
                make_token(
                    0,
                    "Swimming",
                    "swim",
                    UPos::Verb,
                    Some(4),
                    DepRel::Nsubj,
                    Some(VerbForm::Gerund),
                ),
                make_token(1, "and", "and", UPos::Cconj, Some(2), DepRel::Cc, None),
                make_token(
                    2,
                    "running",
                    "run",
                    UPos::Verb,
                    Some(0),
                    DepRel::Conj,
                    Some(VerbForm::Gerund),
                ),
                make_token(3, "are", "be", UPos::Aux, Some(4), DepRel::Cop, None),
                make_token(4, "fun", "fun", UPos::Adj, None, DepRel::Root, None),
            ],
        );

        let classifier = GerundClassifier::new();
        let all = classifier.classify_all(&syntax);

        assert_eq!(all.len(), 2);
        // Both should be nominal since they're in subject position
        assert!(all.iter().all(|(_, usage)| *usage == GerundUsage::Nominal));
    }
}
