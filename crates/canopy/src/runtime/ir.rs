//! Intermediate Representation types for the Canopy semantic kernel.
//!
//! `AnnotatedSyntax` is the contract type between the parser (`SyntaxProvider`)
//! and the kernel. It contains all morphosyntactic information needed for
//! semantic analysis without coupling to any specific parser implementation.

use super::ids::{NodeId, TokenId};
use crate::core::{DepRel, MorphFeatures, UPos};
use serde::{Deserialize, Serialize};

/// Phrasal verb annotation (verb + particle construction).
///
/// Represents lexicalized verb-particle combinations like "give up", "turn off".
/// Based on Construction Grammar (Goldberg 1995) - phrasal verbs are stored
/// whole in the lexicon and should be queried as single units.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhrasalVerb {
    /// The main verb token ID.
    pub verb_id: TokenId,
    /// Particle token IDs (usually one, but can be multiple: "look forward to").
    pub particle_ids: Vec<TokenId>,
    /// Combined lemma for lexical lookup (e.g., `give_up`).
    pub combined_lemma: String,
    /// Byte span covering verb and all particles.
    pub span: (usize, usize),
}

/// Multi-word expression type.
///
/// Based on Universal Dependencies annotation guidelines.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MweType {
    /// Compound noun: "ice cream", "health care" (UD: compound)
    CompoundNoun,
    /// Flat name: "New York", "John Smith" (UD: flat)
    FlatName,
    /// Fixed expression: "in spite of", "as well as" (UD: fixed)
    FixedExpression,
}

/// Multi-word expression annotation.
///
/// Represents token sequences that function as single lexical units.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MweInfo {
    /// Type of multi-word expression.
    pub mwe_type: MweType,
    /// Head token of the MWE.
    pub head_id: TokenId,
    /// All token IDs in the MWE, in sentence order.
    pub token_ids: Vec<TokenId>,
    /// Combined lemma with underscores (e.g., `new_york`, `ice_cream`).
    pub combined_lemma: String,
    /// Byte span covering the entire MWE.
    pub span: (usize, usize),
}

/// Annotated syntax tree - the input IR for semantic analysis.
///
/// This is the contract between the `SyntaxProvider` and the kernel.
/// It contains parsed tokens with morphosyntactic annotations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnnotatedSyntax {
    /// Original input text.
    pub text: String,

    /// Annotated tokens in sentence order.
    pub tokens: Vec<AnnotatedToken>,

    /// Optional syntax tree structure (for phrase structure).
    pub tree: Option<SyntaxTree>,

    /// Detected phrasal verbs (verb + particle constructions).
    #[serde(default)]
    pub phrasal_verbs: Vec<PhrasalVerb>,

    /// Detected multi-word expressions (compounds, names, fixed expressions).
    #[serde(default)]
    pub mwes: Vec<MweInfo>,
}

impl AnnotatedSyntax {
    /// Create a new annotated syntax from text and tokens.
    #[must_use]
    pub fn new(text: String, tokens: Vec<AnnotatedToken>) -> Self {
        Self {
            text,
            tokens,
            tree: None,
            phrasal_verbs: Vec::new(),
            mwes: Vec::new(),
        }
    }

    /// Create with a syntax tree.
    #[must_use]
    pub fn with_tree(text: String, tokens: Vec<AnnotatedToken>, tree: SyntaxTree) -> Self {
        Self {
            text,
            tokens,
            tree: Some(tree),
            phrasal_verbs: Vec::new(),
            mwes: Vec::new(),
        }
    }

    /// Get the predicate lemma for a verb token.
    ///
    /// Returns the phrasal lemma (e.g., `give_up`) if the verb has particles,
    /// otherwise returns the regular token lemma.
    #[must_use]
    pub fn get_predicate_lemma(&self, verb_id: TokenId) -> Option<&str> {
        // Check if this verb is part of a phrasal verb
        if let Some(pv) = self.phrasal_verbs.iter().find(|pv| pv.verb_id == verb_id) {
            return Some(&pv.combined_lemma);
        }
        // Fall back to regular token lemma
        self.get_token(verb_id).map(|t| t.lemma.as_str())
    }

    /// Get the MWE containing a token, if any.
    #[must_use]
    pub fn get_mwe_for_token(&self, token_id: TokenId) -> Option<&MweInfo> {
        self.mwes.iter().find(|m| m.token_ids.contains(&token_id))
    }

    /// Check if a token is part of a phrasal verb.
    #[must_use]
    pub fn is_phrasal_verb(&self, verb_id: TokenId) -> bool {
        self.phrasal_verbs.iter().any(|pv| pv.verb_id == verb_id)
    }

    /// Get a token by ID.
    #[must_use]
    pub fn get_token(&self, id: TokenId) -> Option<&AnnotatedToken> {
        self.tokens.get(id.index())
    }

    /// Find the root token (the one with no head or head pointing to itself).
    #[must_use]
    pub fn root(&self) -> Option<&AnnotatedToken> {
        self.tokens
            .iter()
            .find(|t| t.head.is_none_or(|h| h.index() == t.id.index()))
    }

    /// Get all predicates (verbs and predicate adjectives).
    pub fn predicates(&self) -> impl Iterator<Item = &AnnotatedToken> {
        self.tokens
            .iter()
            .filter(|t| matches!(t.upos, UPos::Verb) || t.is_predicate_adj())
    }

    /// Get all nominal tokens.
    pub fn nominals(&self) -> impl Iterator<Item = &AnnotatedToken> {
        self.tokens
            .iter()
            .filter(|t| matches!(t.upos, UPos::Noun | UPos::Propn | UPos::Pron))
    }

    /// Get children of a token (tokens whose head is this token).
    pub fn children(&self, parent: TokenId) -> impl Iterator<Item = &AnnotatedToken> {
        self.tokens
            .iter()
            .filter(move |t| t.head.is_some_and(|h| h.index() == parent.index()))
    }

    /// Get the subject of a predicate (if any).
    #[must_use]
    pub fn subject_of(&self, pred: TokenId) -> Option<&AnnotatedToken> {
        self.children(pred)
            .find(|t| matches!(t.deprel, DepRel::Nsubj | DepRel::NsubjPass | DepRel::Csubj))
    }

    /// Get the object of a predicate (if any).
    #[must_use]
    pub fn object_of(&self, pred: TokenId) -> Option<&AnnotatedToken> {
        self.children(pred)
            .find(|t| matches!(t.deprel, DepRel::Obj))
    }

    /// Get the indirect object of a predicate (if any).
    #[must_use]
    pub fn iobject_of(&self, pred: TokenId) -> Option<&AnnotatedToken> {
        self.children(pred)
            .find(|t| matches!(t.deprel, DepRel::Iobj))
    }

    /// Check if this is a passive construction.
    #[must_use]
    pub fn is_passive(&self) -> bool {
        self.tokens.iter().any(|t| {
            matches!(
                t.deprel,
                DepRel::NsubjPass | DepRel::CsubjPass | DepRel::AuxPass
            )
        })
    }
}

/// A single annotated token with full morphosyntactic information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnnotatedToken {
    /// Token position in sentence (0-indexed).
    pub id: TokenId,

    /// Surface form.
    pub form: String,

    /// Lemma (base form).
    pub lemma: String,

    /// Universal POS tag.
    pub upos: UPos,

    /// Language-specific POS tag.
    pub xpos: Option<String>,

    /// Morphological features.
    pub feats: MorphFeatures,

    /// Head token ID (for dependency structure).
    pub head: Option<TokenId>,

    /// Dependency relation to head.
    pub deprel: DepRel,

    /// Byte offsets in original text: `text[span.0..span.1]` gives the token's form.
    /// For tokens split from contractions, span points to the parent form.
    pub span: (usize, usize),
}

impl AnnotatedToken {
    /// Create a new annotated token.
    #[must_use]
    pub fn new(
        id: TokenId,
        form: String,
        lemma: String,
        upos: UPos,
        deprel: DepRel,
        span: (usize, usize),
    ) -> Self {
        Self {
            id,
            form,
            lemma,
            upos,
            xpos: None,
            feats: MorphFeatures::default(),
            head: None,
            deprel,
            span,
        }
    }

    /// Create with a head reference.
    #[must_use]
    pub fn with_head(mut self, head: TokenId) -> Self {
        self.head = Some(head);
        self
    }

    /// Create with morphological features.
    #[must_use]
    pub fn with_feats(mut self, feats: MorphFeatures) -> Self {
        self.feats = feats;
        self
    }

    /// Check if this is a content word (noun, verb, adj, adv).
    #[must_use]
    pub fn is_content_word(&self) -> bool {
        matches!(
            self.upos,
            UPos::Noun | UPos::Verb | UPos::Adj | UPos::Adv | UPos::Propn
        )
    }

    /// Check if this token is a predicate adjective (copula complement).
    #[must_use]
    pub fn is_predicate_adj(&self) -> bool {
        matches!(self.upos, UPos::Adj) && self.has_copula_head()
    }

    /// Check if this token's head is a copula.
    fn has_copula_head(&self) -> bool {
        // Would need access to the full syntax to check this properly
        // For now, use deprel as a heuristic
        matches!(self.deprel, DepRel::Root)
    }

    /// Check if this is a finite verb.
    #[must_use]
    pub fn is_finite(&self) -> bool {
        use crate::core::VerbForm;
        matches!(self.upos, UPos::Verb)
            && self
                .feats
                .verb_form
                .as_ref()
                .is_none_or(|vf| matches!(vf, VerbForm::Finite))
    }
}

/// Phrase structure tree (optional, for constituency parsing).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SyntaxTree {
    /// Root node of the tree.
    pub root: SyntaxNode,
}

/// A node in the phrase structure tree.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SyntaxNode {
    /// Node identifier.
    pub id: NodeId,

    /// Category label (NP, VP, S, etc.).
    pub label: String,

    /// Child nodes (phrase structure) or terminal token.
    pub children: SyntaxChildren,

    /// Span in original text.
    pub span: (usize, usize),
}

/// Children of a syntax node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SyntaxChildren {
    /// Non-terminal: has phrase children.
    Phrases(Vec<SyntaxNode>),

    /// Terminal: points to a token.
    Terminal(TokenId),
}

impl SyntaxTree {
    /// Create a new syntax tree with a root node.
    #[must_use]
    pub fn new(root: SyntaxNode) -> Self {
        Self { root }
    }

    /// Find a node by ID (depth-first search).
    #[must_use]
    pub fn find_node(&self, id: NodeId) -> Option<&SyntaxNode> {
        Self::find_in_node(&self.root, id)
    }

    fn find_in_node(node: &SyntaxNode, id: NodeId) -> Option<&SyntaxNode> {
        if node.id == id {
            return Some(node);
        }
        if let SyntaxChildren::Phrases(children) = &node.children {
            for child in children {
                if let Some(found) = Self::find_in_node(child, id) {
                    return Some(found);
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_syntax() -> AnnotatedSyntax {
        // "John gives Mary a book"
        let tokens = vec![
            AnnotatedToken::new(
                TokenId::new(0),
                "John".to_string(),
                "john".to_string(),
                UPos::Propn,
                DepRel::Nsubj,
                (0, 4),
            )
            .with_head(TokenId::new(1)),
            AnnotatedToken::new(
                TokenId::new(1),
                "gives".to_string(),
                "give".to_string(),
                UPos::Verb,
                DepRel::Root,
                (5, 10),
            ),
            AnnotatedToken::new(
                TokenId::new(2),
                "Mary".to_string(),
                "mary".to_string(),
                UPos::Propn,
                DepRel::Iobj,
                (11, 15),
            )
            .with_head(TokenId::new(1)),
            AnnotatedToken::new(
                TokenId::new(3),
                "a".to_string(),
                "a".to_string(),
                UPos::Det,
                DepRel::Det,
                (16, 17),
            )
            .with_head(TokenId::new(4)),
            AnnotatedToken::new(
                TokenId::new(4),
                "book".to_string(),
                "book".to_string(),
                UPos::Noun,
                DepRel::Obj,
                (18, 22),
            )
            .with_head(TokenId::new(1)),
        ];

        AnnotatedSyntax::new("John gives Mary a book".to_string(), tokens)
    }

    #[test]
    fn test_annotated_syntax_creation() {
        let syn = make_test_syntax();
        assert_eq!(syn.tokens.len(), 5);
        assert_eq!(syn.text, "John gives Mary a book");
    }

    #[test]
    fn test_get_token() {
        let syn = make_test_syntax();
        let token = syn.get_token(TokenId::new(1)).unwrap();
        assert_eq!(token.lemma, "give");
    }

    #[test]
    fn test_root() {
        let syn = make_test_syntax();
        let root = syn.root().unwrap();
        assert_eq!(root.lemma, "give");
    }

    #[test]
    fn test_predicates() {
        let syn = make_test_syntax();
        let preds: Vec<_> = syn.predicates().collect();
        assert_eq!(preds.len(), 1);
        assert_eq!(preds[0].lemma, "give");
    }

    #[test]
    fn test_nominals() {
        let syn = make_test_syntax();
        let noms: Vec<_> = syn.nominals().collect();
        assert_eq!(noms.len(), 3); // John, Mary, book
    }

    #[test]
    fn test_subject_of() {
        let syn = make_test_syntax();
        let subj = syn.subject_of(TokenId::new(1)).unwrap();
        assert_eq!(subj.form, "John");
    }

    #[test]
    fn test_object_of() {
        let syn = make_test_syntax();
        let obj = syn.object_of(TokenId::new(1)).unwrap();
        assert_eq!(obj.form, "book");
    }

    #[test]
    fn test_iobject_of() {
        let syn = make_test_syntax();
        let iobj = syn.iobject_of(TokenId::new(1)).unwrap();
        assert_eq!(iobj.form, "Mary");
    }

    #[test]
    fn test_children() {
        let syn = make_test_syntax();
        let children: Vec<_> = syn.children(TokenId::new(1)).collect();
        // John (subj), Mary (iobj), book (obj)
        assert_eq!(children.len(), 3);
    }

    #[test]
    fn test_is_passive() {
        let syn = make_test_syntax();
        assert!(!syn.is_passive());

        // Create a passive sentence
        let passive_tokens = vec![
            AnnotatedToken::new(
                TokenId::new(0),
                "The".to_string(),
                "the".to_string(),
                UPos::Det,
                DepRel::Det,
                (0, 3),
            )
            .with_head(TokenId::new(1)),
            AnnotatedToken::new(
                TokenId::new(1),
                "book".to_string(),
                "book".to_string(),
                UPos::Noun,
                DepRel::NsubjPass,
                (4, 8),
            )
            .with_head(TokenId::new(3)),
            AnnotatedToken::new(
                TokenId::new(2),
                "was".to_string(),
                "be".to_string(),
                UPos::Aux,
                DepRel::AuxPass,
                (9, 12),
            )
            .with_head(TokenId::new(3)),
            AnnotatedToken::new(
                TokenId::new(3),
                "given".to_string(),
                "give".to_string(),
                UPos::Verb,
                DepRel::Root,
                (13, 18),
            ),
        ];
        let passive_syn = AnnotatedSyntax::new("The book was given".to_string(), passive_tokens);
        assert!(passive_syn.is_passive());
    }

    #[test]
    fn test_annotated_token_is_content_word() {
        let noun = AnnotatedToken::new(
            TokenId::new(0),
            "book".to_string(),
            "book".to_string(),
            UPos::Noun,
            DepRel::Obj,
            (0, 4),
        );
        assert!(noun.is_content_word());

        let det = AnnotatedToken::new(
            TokenId::new(0),
            "the".to_string(),
            "the".to_string(),
            UPos::Det,
            DepRel::Det,
            (0, 3),
        );
        assert!(!det.is_content_word());
    }

    #[test]
    fn test_syntax_tree() {
        let root = SyntaxNode {
            id: NodeId::new(0),
            label: "S".to_string(),
            children: SyntaxChildren::Phrases(vec![SyntaxNode {
                id: NodeId::new(1),
                label: "NP".to_string(),
                children: SyntaxChildren::Terminal(TokenId::new(0)),
                span: (0, 4),
            }]),
            span: (0, 22),
        };

        let tree = SyntaxTree::new(root);
        assert!(tree.find_node(NodeId::new(0)).is_some());
        assert!(tree.find_node(NodeId::new(1)).is_some());
        assert!(tree.find_node(NodeId::new(99)).is_none());
    }
}
