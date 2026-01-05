//! Resource-backed POS tagger using validated semantic datasets.
//!
//! Provides accurate POS tagging by querying multiple semantic resources
//! in a layered approach, falling back to heuristics only as a last resort.

use super::shared::{
    guess_dependency, lemmatize_by_suffix, suffix_heuristics, SUBORDINATING_CONJUNCTIONS,
};
use super::word_pos_index::WordPosIndex;
use crate::engine::SharedEngines;
use crate::lexicon::LexiconEngine;
use crate::tokenizer::RawToken;
use crate::verbnet::VerbNetEngine;
use crate::wordnet::WordNetEngine;
use canopy::runtime::{AnnotatedSyntax, AnnotatedToken, TokenId};
use canopy::{CanopyError, DepRel, UPos};
use std::sync::Arc;

/// A POS tagger backed by validated semantic resources.
///
/// Uses a layered lookup strategy:
/// 1. Treebank word→POS index (23K+ words, most reliable)
/// 2. `VerbNet` (all English verbs with class assignments)
/// 3. `WordNet` (150K+ words across all POS)
/// 4. Lexicon (closed-class words)
/// 5. Suffix heuristics (last resort)
#[derive(Debug)]
pub struct ResourceBackedTagger {
    /// Word→POS index from treebank statistics
    word_pos_index: WordPosIndex,
    /// `VerbNet` engine for verb detection
    verbnet: Option<Arc<VerbNetEngine>>,
    /// `WordNet` engine for general POS lookup
    wordnet: Option<Arc<WordNetEngine>>,
    /// Lexicon for closed-class words
    lexicon: Arc<LexiconEngine>,
}

impl ResourceBackedTagger {
    /// Create a new resource-backed tagger.
    ///
    /// Loads the word→POS index from treebank data and initializes
    /// semantic engines for fallback lookup.
    ///
    /// # Errors
    /// Returns an error if the word→POS index cannot be loaded from treebank.
    pub fn new() -> Result<Self, CanopyError> {
        let word_pos_index = WordPosIndex::from_treebank()?;

        // Create and load lexicon
        let mut lexicon = LexiconEngine::new();
        let _ = lexicon.load_data();
        let lexicon = Arc::new(lexicon);

        // Try to load VerbNet (optional, may fail if data not available)
        let verbnet = VerbNetEngine::new().ok().map(Arc::new);

        // Try to load WordNet (optional, may fail if data not available)
        let wordnet = WordNetEngine::new().ok().map(Arc::new);

        tracing::info!(
            "ResourceBackedTagger initialized: {} words in index, VerbNet={}, WordNet={}",
            word_pos_index.len(),
            verbnet.is_some(),
            wordnet.is_some()
        );

        Ok(Self {
            word_pos_index,
            verbnet,
            wordnet,
            lexicon,
        })
    }

    /// Create with explicit dependencies (for testing or custom setups).
    #[must_use]
    pub fn with_deps(
        word_pos_index: WordPosIndex,
        verbnet: Option<Arc<VerbNetEngine>>,
        wordnet: Option<Arc<WordNetEngine>>,
        lexicon: Arc<LexiconEngine>,
    ) -> Self {
        Self {
            word_pos_index,
            verbnet,
            wordnet,
            lexicon,
        }
    }

    /// Create with shared engines (for pipeline efficiency).
    ///
    /// Uses engines from a `SharedEngines` instance to avoid duplicate
    /// initialization when multiple components need the same engines.
    ///
    /// # Errors
    /// Returns an error if the word→POS index cannot be loaded from treebank.
    pub fn with_shared_engines(engines: &SharedEngines) -> Result<Self, CanopyError> {
        let word_pos_index = WordPosIndex::from_treebank()?;

        tracing::info!(
            "ResourceBackedTagger initialized with shared engines: {} words in index",
            word_pos_index.len()
        );

        Ok(Self {
            word_pos_index,
            verbnet: engines.verbnet.clone(),
            wordnet: engines.wordnet.clone(),
            lexicon: engines.lexicon.clone(),
        })
    }

    /// Parse tokens into annotated syntax using resource-backed tagging.
    #[must_use]
    pub fn parse(&self, text: &str, tokens: &[RawToken]) -> AnnotatedSyntax {
        let mut annotated = Vec::with_capacity(tokens.len());
        let mut verb_idx: Option<usize> = None;

        // First pass: assign POS tags using layered lookup
        for (idx, token) in tokens.iter().enumerate() {
            let form = &token.form;
            let upos = self.tag_pos(form, idx, tokens.len());

            // Track the first verb as potential root
            if matches!(upos, UPos::Verb) && verb_idx.is_none() {
                verb_idx = Some(idx);
            }

            let lemma = self.lemmatize(form);

            annotated.push(AnnotatedToken::new(
                TokenId::new(idx),
                form.clone(),
                lemma,
                upos,
                DepRel::Dep,
                token.byte_span,
            ));
        }

        // Second pass: assign dependency relations and heads
        let root_idx = verb_idx.unwrap_or(0);
        for (idx, token) in annotated.iter_mut().enumerate() {
            if idx == root_idx {
                token.deprel = DepRel::Root;
            } else {
                let (head, deprel) =
                    guess_dependency(idx, root_idx, token.upos, tokens, &self.lexicon);
                token.head = Some(TokenId::new(head));
                token.deprel = deprel;
            }
        }

        AnnotatedSyntax::new(text.to_string(), annotated)
    }

    /// Tag a word's POS using the layered lookup strategy.
    fn tag_pos(&self, word: &str, position: usize, _total_tokens: usize) -> UPos {
        let lower = word.to_lowercase();

        // Layer 1: Treebank statistics (most reliable, 23K+ words)
        if let Some(pos) = self.word_pos_index.get_pos(&lower) {
            return pos;
        }

        // Layer 2: VerbNet (all English verbs) - early return on match
        if let Some(ref verbnet) = self.verbnet {
            if let Ok(result) = verbnet.analyze_verb(&lower) {
                if !result.data.verb_classes.is_empty() {
                    return UPos::Verb;
                }
            }
        }

        // Layer 3: WordNet (150K+ words) - early return on first match
        if let Some(ref wordnet) = self.wordnet {
            use crate::wordnet::PartOfSpeech;

            // Check verb first (most important for irregular verb detection)
            if let Ok(result) = wordnet.analyze_word(&lower, PartOfSpeech::Verb) {
                if !result.data.synsets.is_empty() {
                    return UPos::Verb;
                }
            }
            // Only check other POS if verb didn't match
            if let Ok(result) = wordnet.analyze_word(&lower, PartOfSpeech::Noun) {
                if !result.data.synsets.is_empty() {
                    return UPos::Noun;
                }
            }
            if let Ok(result) = wordnet.analyze_word(&lower, PartOfSpeech::Adjective) {
                if !result.data.synsets.is_empty() {
                    return UPos::Adj;
                }
            }
            if let Ok(result) = wordnet.analyze_word(&lower, PartOfSpeech::Adverb) {
                if !result.data.synsets.is_empty() {
                    return UPos::Adv;
                }
            }
        }

        // Layer 4: Lexicon (closed-class words)
        if let Some(pos) = self.check_lexicon(&lower) {
            return pos;
        }

        // Check for articles/determiners
        if lower == "a" || lower == "an" || lower == "the" {
            return UPos::Det;
        }

        // Layer 5: Suffix heuristics (last resort)
        suffix_heuristics(word, position)
    }

    /// Check lexicon for closed-class words.
    fn check_lexicon(&self, lower: &str) -> Option<UPos> {
        if self.lexicon.is_pronoun(lower).unwrap_or(false) {
            return Some(UPos::Pron);
        }
        if self.lexicon.is_auxiliary(lower).unwrap_or(false)
            || self.lexicon.is_modal(lower).unwrap_or(false)
        {
            return Some(UPos::Aux);
        }
        if self.lexicon.is_preposition(lower).unwrap_or(false) {
            return Some(UPos::Adp);
        }
        if self.lexicon.is_conjunction(lower).unwrap_or(false) {
            let is_subord = SUBORDINATING_CONJUNCTIONS.contains(&lower);
            return Some(if is_subord { UPos::Sconj } else { UPos::Cconj });
        }
        if self.lexicon.is_wh_word(lower).unwrap_or(false) {
            if lower == "where" || lower == "when" || lower == "why" || lower == "how" {
                return Some(UPos::Adv);
            }
            return Some(UPos::Pron);
        }
        if self.lexicon.is_quantifier(lower).unwrap_or(false) {
            return Some(UPos::Det);
        }
        None
    }

    /// Simple lemmatization using shared suffix rules.
    fn lemmatize(&self, form: &str) -> String {
        let lower = form.to_lowercase();

        // Keep auxiliaries/modals as-is
        if self.lexicon.is_auxiliary(&lower).unwrap_or(false)
            || self.lexicon.is_modal(&lower).unwrap_or(false)
        {
            return lower;
        }

        // Use shared suffix-based lemmatization
        lemmatize_by_suffix(form)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer::{SimpleTokenizer, Tokenizer};

    fn data_available() -> bool {
        // Need both lexicon and treebank for full tagger functionality
        let lexicon = crate::paths::data_path("data/lexicon").exists();
        let treebank = crate::paths::data_path("data/ud_english-ewt/UD_English-EWT").exists()
            || crate::paths::data_path("data/ud_english-ewt")
                .join("en_ewt-ud-train.conllu")
                .exists();
        lexicon && treebank
    }

    #[test]
    fn test_resource_tagger_creation() {
        if !data_available() {
            eprintln!("Skipping: Data not available");
            return;
        }

        let tagger = ResourceBackedTagger::new();
        assert!(
            tagger.is_ok(),
            "Failed to create tagger: {:?}",
            tagger.err()
        );
    }

    #[test]
    fn test_irregular_verb_detection() {
        if !data_available() {
            eprintln!("Skipping: Data not available");
            return;
        }

        let tagger = ResourceBackedTagger::new().unwrap();
        let tokenizer = SimpleTokenizer::new();

        let tokens = tokenizer.tokenize("Mary gave John a book.");
        let syntax = tagger.parse("Mary gave John a book.", &tokens);

        let gave_token = syntax.tokens.iter().find(|t| t.form == "gave");
        assert!(gave_token.is_some(), "Should have 'gave' token");
        assert_eq!(
            gave_token.unwrap().upos,
            UPos::Verb,
            "'gave' should be tagged as VERB, not {:?}",
            gave_token.unwrap().upos
        );
    }

    #[test]
    fn test_common_irregular_verbs() {
        if !data_available() {
            eprintln!("Skipping: Data not available");
            return;
        }

        let tagger = ResourceBackedTagger::new().unwrap();
        let irregular_verbs = [
            "gave", "went", "broke", "took", "saw", "made", "came", "knew",
        ];

        for verb in irregular_verbs {
            let pos = tagger.tag_pos(verb, 1, 3);
            assert_eq!(
                pos,
                UPos::Verb,
                "'{verb}' should be tagged as VERB, got {pos:?}"
            );
        }
    }

    #[test]
    fn test_closed_class_words() {
        if !data_available() {
            eprintln!("Skipping: Data not available");
            return;
        }

        let tagger = ResourceBackedTagger::new().unwrap();

        assert_eq!(tagger.tag_pos("the", 0, 3), UPos::Det);
        assert_eq!(tagger.tag_pos("a", 0, 3), UPos::Det);

        let he_pos = tagger.tag_pos("he", 0, 3);
        assert!(
            matches!(he_pos, UPos::Pron),
            "'he' should be PRON, got {he_pos:?}"
        );
    }

    #[test]
    fn test_suffix_fallback() {
        if !data_available() {
            eprintln!("Skipping: Data not available");
            return;
        }

        let tagger = ResourceBackedTagger::new().unwrap();

        assert_eq!(tagger.tag_pos("glorping", 1, 3), UPos::Verb);
        assert_eq!(tagger.tag_pos("glorped", 1, 3), UPos::Verb);
        assert_eq!(tagger.tag_pos("glorpily", 1, 3), UPos::Adv);
    }
}
