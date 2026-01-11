//! Multi-word expression (MWE) detection.
//!
//! Detects and groups tokens that form multi-word expressions using
//! UD dependency relations: `compound`, `flat`, and `fixed`.

use canopy::core::DepRel;
use canopy::runtime::{AnnotatedSyntax, TokenId};
use serde::{Deserialize, Serialize};

/// Types of multi-word expressions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MweType {
    /// Compound noun: "ice cream", "health care"
    CompoundNoun,
    /// Flat name: "New York", "John Smith"
    FlatName,
    /// Fixed expression: "in spite of", "as well as"
    FixedExpression,
}

impl MweType {
    /// Get the display name for this MWE type.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            MweType::CompoundNoun => "compound",
            MweType::FlatName => "flat",
            MweType::FixedExpression => "fixed",
        }
    }
}

/// A detected multi-word expression.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Mwe {
    /// Type of MWE.
    pub mwe_type: MweType,
    /// Head token of the MWE (rightmost in UD annotation).
    pub head_token: TokenId,
    /// All tokens in the MWE, in sentence order.
    pub tokens: Vec<TokenId>,
    /// Combined lemma with underscores (e.g., "new\_york", "ice\_cream").
    pub combined_lemma: String,
    /// Byte span covering the entire MWE.
    pub span: (usize, usize),
}

impl Mwe {
    /// Check if this MWE contains the given token.
    #[must_use]
    pub fn contains(&self, token_id: TokenId) -> bool {
        self.tokens.contains(&token_id)
    }

    /// Get the number of tokens in this MWE.
    #[must_use]
    pub fn len(&self) -> usize {
        self.tokens.len()
    }

    /// Check if this MWE is empty (shouldn't happen in practice).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }
}

/// Detects multi-word expressions by grouping dependency relations.
#[derive(Debug, Default)]
pub struct MweDetector;

impl MweDetector {
    /// Create a new MWE detector.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Find all multi-word expressions in the syntax.
    ///
    /// Groups tokens connected by `compound`, `flat`, or `fixed` relations.
    #[must_use]
    pub fn find_mwes(&self, syntax: &AnnotatedSyntax) -> Vec<Mwe> {
        let mut mwes = Vec::new();
        let mut processed_heads: Vec<TokenId> = Vec::new();

        for token in &syntax.tokens {
            // Skip if already processed as part of another MWE
            if processed_heads.contains(&token.id) {
                continue;
            }

            // Find children with MWE relations
            let mwe_children: Vec<_> = syntax
                .tokens
                .iter()
                .filter(|t| {
                    t.head == Some(token.id)
                        && matches!(t.deprel, DepRel::Compound | DepRel::Flat | DepRel::Fixed)
                })
                .collect();

            if mwe_children.is_empty() {
                continue;
            }

            // Determine MWE type from first child's relation
            let mwe_type = match mwe_children[0].deprel {
                DepRel::Compound => MweType::CompoundNoun,
                DepRel::Flat => MweType::FlatName,
                DepRel::Fixed => MweType::FixedExpression,
                _ => continue,
            };

            // Collect all tokens including head
            let mut tokens: Vec<TokenId> = mwe_children.iter().map(|c| c.id).collect();
            tokens.push(token.id);

            // Sort by position for correct word order
            tokens.sort_by_key(|t| t.index());

            // Build combined lemma
            let combined_lemma = tokens
                .iter()
                .filter_map(|&id| syntax.get_token(id))
                .map(|t| t.lemma.to_lowercase())
                .collect::<Vec<_>>()
                .join("_");

            // Calculate span
            let spans: Vec<_> = tokens
                .iter()
                .filter_map(|&id| syntax.get_token(id))
                .map(|t| t.span)
                .collect();

            let span = (
                spans.iter().map(|s| s.0).min().unwrap_or(0),
                spans.iter().map(|s| s.1).max().unwrap_or(0),
            );

            mwes.push(Mwe {
                mwe_type,
                head_token: token.id,
                tokens,
                combined_lemma,
                span,
            });

            processed_heads.push(token.id);
        }

        mwes
    }

    /// Find the MWE containing a specific token, if any.
    #[must_use]
    pub fn find_mwe_for_token<'a>(&self, mwes: &'a [Mwe], token_id: TokenId) -> Option<&'a Mwe> {
        mwes.iter().find(|mwe| mwe.contains(token_id))
    }

    /// Get all compound nouns in the syntax.
    #[must_use]
    pub fn find_compound_nouns(&self, syntax: &AnnotatedSyntax) -> Vec<Mwe> {
        self.find_mwes(syntax)
            .into_iter()
            .filter(|m| m.mwe_type == MweType::CompoundNoun)
            .collect()
    }

    /// Get all flat names in the syntax.
    #[must_use]
    pub fn find_flat_names(&self, syntax: &AnnotatedSyntax) -> Vec<Mwe> {
        self.find_mwes(syntax)
            .into_iter()
            .filter(|m| m.mwe_type == MweType::FlatName)
            .collect()
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
    ) -> AnnotatedToken {
        AnnotatedToken {
            id: TokenId::new(id),
            form: form.to_string(),
            lemma: lemma.to_string(),
            upos,
            xpos: None,
            feats: MorphFeatures::default(),
            head: head.map(TokenId::new),
            deprel,
            span: (0, form.len()),
        }
    }

    #[test]
    fn test_compound_noun_detection() {
        // "ice cream" - compound noun
        let syntax = AnnotatedSyntax::new(
            "ice cream".to_string(),
            vec![
                make_token(0, "ice", "ice", UPos::Noun, Some(1), DepRel::Compound),
                make_token(1, "cream", "cream", UPos::Noun, None, DepRel::Root),
            ],
        );

        let detector = MweDetector::new();
        let mwes = detector.find_mwes(&syntax);

        assert_eq!(mwes.len(), 1);
        assert_eq!(mwes[0].mwe_type, MweType::CompoundNoun);
        assert_eq!(mwes[0].combined_lemma, "ice_cream");
        assert_eq!(mwes[0].tokens.len(), 2);
    }

    #[test]
    fn test_flat_name_detection() {
        // "New York" - flat name
        let syntax = AnnotatedSyntax::new(
            "New York".to_string(),
            vec![
                make_token(0, "New", "New", UPos::Propn, Some(1), DepRel::Flat),
                make_token(1, "York", "York", UPos::Propn, None, DepRel::Root),
            ],
        );

        let detector = MweDetector::new();
        let mwes = detector.find_mwes(&syntax);

        assert_eq!(mwes.len(), 1);
        assert_eq!(mwes[0].mwe_type, MweType::FlatName);
        assert_eq!(mwes[0].combined_lemma, "new_york");
    }

    #[test]
    fn test_no_mwe() {
        // "The cat runs" - no MWE
        let syntax = AnnotatedSyntax::new(
            "The cat runs".to_string(),
            vec![
                make_token(0, "The", "the", UPos::Det, Some(1), DepRel::Det),
                make_token(1, "cat", "cat", UPos::Noun, Some(2), DepRel::Nsubj),
                make_token(2, "runs", "run", UPos::Verb, None, DepRel::Root),
            ],
        );

        let detector = MweDetector::new();
        let mwes = detector.find_mwes(&syntax);

        assert!(mwes.is_empty());
    }

    #[test]
    fn test_find_mwe_for_token() {
        let syntax = AnnotatedSyntax::new(
            "ice cream".to_string(),
            vec![
                make_token(0, "ice", "ice", UPos::Noun, Some(1), DepRel::Compound),
                make_token(1, "cream", "cream", UPos::Noun, None, DepRel::Root),
            ],
        );

        let detector = MweDetector::new();
        let mwes = detector.find_mwes(&syntax);

        // Both tokens should be in the same MWE
        let mwe_for_ice = detector.find_mwe_for_token(&mwes, TokenId::new(0));
        let mwe_for_cream = detector.find_mwe_for_token(&mwes, TokenId::new(1));

        assert!(mwe_for_ice.is_some());
        assert!(mwe_for_cream.is_some());
        assert_eq!(mwe_for_ice.unwrap().combined_lemma, "ice_cream");
    }

    #[test]
    fn test_mwe_len() {
        let mwe = Mwe {
            mwe_type: MweType::CompoundNoun,
            head_token: TokenId::new(1),
            tokens: vec![TokenId::new(0), TokenId::new(1)],
            combined_lemma: "ice_cream".to_string(),
            span: (0, 9),
        };

        assert_eq!(mwe.len(), 2);
        assert!(!mwe.is_empty());
    }
}
