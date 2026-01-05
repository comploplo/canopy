//! `DiscourseCueProvider` implementation using Lexicon.
//!
//! Identifies discourse connectives and their relations based on
//! the closed-class lexicon data in `data/lexicon/english-lexicon.xml`.

use crate::lexicon::LexiconEngine;
use canopy::runtime::{AnnotatedSyntax, DiscourseCueProvider, DiscourseRelation, TokenId};
use canopy::CanopyError;
use std::sync::Arc;

/// `DiscourseCueProvider` implementation using a closed-class lexicon.
///
/// Identifies discourse connectives (subordinating conjunctions,
/// coordinating conjunctions, discourse adverbs) and maps them
/// to discourse relations. All word lists are stored in the lexicon
/// XML file, not hardcoded in code.
#[derive(Debug, Clone)]
pub struct LexiconDiscourseCueProvider {
    /// Reference to the lexicon engine for word lookups
    engine: Arc<LexiconEngine>,
}

impl Default for LexiconDiscourseCueProvider {
    fn default() -> Self {
        // Create and load the engine
        let mut engine = LexiconEngine::new();
        if let Err(e) = engine.load_data() {
            tracing::warn!("Failed to load lexicon data for discourse cues: {e}");
        }
        Self {
            engine: Arc::new(engine),
        }
    }
}

impl LexiconDiscourseCueProvider {
    /// Create a new provider.
    ///
    /// # Errors
    /// Returns an error if the lexicon data cannot be loaded.
    pub fn new() -> Result<Self, CanopyError> {
        let mut engine = LexiconEngine::new();
        engine.load_data().map_err(|e| {
            CanopyError::data_load(format!("Failed to load lexicon for discourse cues: {e}"))
        })?;
        Ok(Self {
            engine: Arc::new(engine),
        })
    }

    /// Create a provider with a shared lexicon engine.
    #[must_use]
    pub fn with_engine(engine: Arc<LexiconEngine>) -> Self {
        Self { engine }
    }

    /// Look up a word's discourse relation from the lexicon.
    fn lookup(&self, word: &str) -> Option<DiscourseRelation> {
        self.engine.get_discourse_relation(word).ok().flatten()
    }
}

impl DiscourseCueProvider for LexiconDiscourseCueProvider {
    fn is_discourse_connective(&self, syntax: &AnnotatedSyntax, token_id: TokenId) -> bool {
        if let Some(token) = syntax.tokens.get(token_id.index()) {
            let word = &token.lemma;
            self.lookup(word).is_some()
        } else {
            false
        }
    }

    fn discourse_relation(
        &self,
        syntax: &AnnotatedSyntax,
        token_id: TokenId,
    ) -> Option<DiscourseRelation> {
        let token = syntax.tokens.get(token_id.index())?;
        let word = &token.lemma;
        self.lookup(word)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use canopy::runtime::AnnotatedToken;

    #[test]
    fn test_provider_creation() {
        let provider = LexiconDiscourseCueProvider::new();
        assert!(provider.is_ok());
    }

    #[test]
    fn test_is_discourse_connective() {
        use canopy::{DepRel, UPos};

        let provider = LexiconDiscourseCueProvider::new().unwrap();

        let syntax = AnnotatedSyntax::new(
            "However it failed".to_string(),
            vec![
                AnnotatedToken::new(
                    TokenId::new(0),
                    "However".to_string(),
                    "however".to_string(),
                    UPos::Adv,
                    DepRel::Advmod,
                    (0, 7),
                ),
                AnnotatedToken::new(
                    TokenId::new(1),
                    "it".to_string(),
                    "it".to_string(),
                    UPos::Pron,
                    DepRel::Nsubj,
                    (8, 10),
                ),
                AnnotatedToken::new(
                    TokenId::new(2),
                    "failed".to_string(),
                    "fail".to_string(),
                    UPos::Verb,
                    DepRel::Root,
                    (11, 17),
                ),
            ],
        );

        assert!(provider.is_discourse_connective(&syntax, TokenId::new(0)));
        assert!(!provider.is_discourse_connective(&syntax, TokenId::new(1)));
        assert!(!provider.is_discourse_connective(&syntax, TokenId::new(2)));
    }

    #[test]
    fn test_discourse_relation() {
        use canopy::{DepRel, UPos};

        let provider = LexiconDiscourseCueProvider::new().unwrap();

        let syntax = AnnotatedSyntax::new(
            "because".to_string(),
            vec![AnnotatedToken::new(
                TokenId::new(0),
                "because".to_string(),
                "because".to_string(),
                UPos::Sconj,
                DepRel::Mark,
                (0, 7),
            )],
        );

        let relation = provider.discourse_relation(&syntax, TokenId::new(0));
        assert_eq!(relation, Some(DiscourseRelation::Cause));
    }

    #[test]
    fn test_connective_mappings() {
        let provider = LexiconDiscourseCueProvider::new().unwrap();

        // Test each category
        assert_eq!(provider.lookup("because"), Some(DiscourseRelation::Cause));
        assert_eq!(
            provider.lookup("however"),
            Some(DiscourseRelation::Contrast)
        );
        assert_eq!(
            provider.lookup("although"),
            Some(DiscourseRelation::Concession)
        );
        assert_eq!(provider.lookup("if"), Some(DiscourseRelation::Condition));
        assert_eq!(provider.lookup("then"), Some(DiscourseRelation::Temporal));
        assert_eq!(provider.lookup("also"), Some(DiscourseRelation::Addition));
        assert_eq!(
            provider.lookup("specifically"),
            Some(DiscourseRelation::Elaboration)
        );

        // Test case insensitivity
        assert_eq!(provider.lookup("BECAUSE"), Some(DiscourseRelation::Cause));
        assert_eq!(
            provider.lookup("However"),
            Some(DiscourseRelation::Contrast)
        );
    }
}
