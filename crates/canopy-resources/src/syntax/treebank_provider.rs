//! `TreebankSyntaxProvider` implementation.
//!
//! A `SyntaxProvider` that uses patterns from UD English-EWT treebank
//! with resource-backed fallback for unknown patterns.
//!
//! Pattern matching is always-on: verbs are automatically matched against
//! VerbNet-derived dependency patterns for enhanced theta role hints.

use super::gerund::GerundClassifier;
use super::mwe::MweDetector;
use super::pattern_matcher::{extract_patterns_from_syntax, PatternMatcher};
use super::pattern_types::SemanticSignature;
use super::phrasal_verb::PhrasalVerbDetector;
use super::resource_tagger::ResourceBackedTagger;
use super::shared::{parse_deprel, parse_upos};
use crate::engine::{ConlluParser, ConlluSentence, SharedEngines};
use crate::tokenizer::{SimpleTokenizer, Tokenizer};
use canopy::runtime::{
    AnnotatedSyntax, AnnotatedToken, MweInfo, MweType, PhrasalVerb, SyntaxProvider, TokenId,
};
use canopy::{CanopyError, MorphFeatures, UPos};
use std::collections::HashMap;
use std::sync::Mutex;

/// A `SyntaxProvider` that matches patterns from UD treebank.
pub struct TreebankSyntaxProvider {
    /// Tokenizer for splitting text.
    tokenizer: SimpleTokenizer,
    /// Resource-backed tagger for fallback (queries treebank, `VerbNet`, `WordNet`).
    tagger: ResourceBackedTagger,
    /// Pattern index: normalized text → `AnnotatedSyntax`.
    pattern_index: HashMap<String, AnnotatedSyntax>,
    /// Pattern matcher for semantic-aware dependency matching (always-on).
    pattern_matcher: Mutex<PatternMatcher>,
    /// Configuration (for future use in more sophisticated matching).
    #[allow(dead_code)]
    config: TreebankConfig,
}

impl std::fmt::Debug for TreebankSyntaxProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TreebankSyntaxProvider")
            .field("tokenizer", &self.tokenizer)
            .field("tagger", &self.tagger)
            .field("pattern_count", &self.pattern_index.len())
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

/// Configuration for the treebank provider.
#[derive(Debug, Clone)]
pub struct TreebankConfig {
    /// Maximum number of patterns to index.
    pub max_patterns: usize,
    /// Cache size for pattern matcher.
    pub pattern_cache_size: usize,
}

impl Default for TreebankConfig {
    fn default() -> Self {
        Self {
            max_patterns: 20000,
            pattern_cache_size: 1000,
        }
    }
}

impl TreebankSyntaxProvider {
    /// Create a new `TreebankSyntaxProvider` with default configuration.
    ///
    /// # Errors
    /// Returns an error if the resource-backed tagger cannot be initialized.
    pub fn new() -> Result<Self, CanopyError> {
        Self::with_config(TreebankConfig::default())
    }

    /// Create with custom configuration.
    ///
    /// # Errors
    /// Returns an error if the resource-backed tagger cannot be initialized.
    pub fn with_config(config: TreebankConfig) -> Result<Self, CanopyError> {
        let tokenizer = SimpleTokenizer::from_ewt().unwrap_or_else(|_| SimpleTokenizer::new());

        // Create resource-backed tagger (loads treebank index, VerbNet, WordNet, lexicon)
        let tagger = ResourceBackedTagger::new()?;

        let pattern_index = Self::load_treebank_patterns(&config);

        // Create pattern matcher with configured cache size
        let pattern_matcher = PatternMatcher::with_cache_size(config.pattern_cache_size);

        Ok(Self {
            tokenizer,
            tagger,
            pattern_index,
            pattern_matcher: Mutex::new(pattern_matcher),
            config,
        })
    }

    /// Create with shared engines (for pipeline efficiency).
    ///
    /// Uses engines from a `SharedEngines` instance to avoid duplicate
    /// initialization when multiple components need the same engines.
    ///
    /// # Errors
    /// Returns an error if the resource-backed tagger cannot be initialized.
    pub fn with_shared_engines(
        config: TreebankConfig,
        engines: &SharedEngines,
    ) -> Result<Self, CanopyError> {
        let tokenizer = SimpleTokenizer::from_ewt().unwrap_or_else(|_| SimpleTokenizer::new());
        let tagger = ResourceBackedTagger::with_shared_engines(engines)?;
        let pattern_index = Self::load_treebank_patterns(&config);
        let pattern_matcher = PatternMatcher::with_cache_size(config.pattern_cache_size);

        Ok(Self {
            tokenizer,
            tagger,
            pattern_index,
            pattern_matcher: Mutex::new(pattern_matcher),
            config,
        })
    }

    /// Load patterns from treebank files.
    fn load_treebank_patterns(config: &TreebankConfig) -> HashMap<String, AnnotatedSyntax> {
        use crate::paths::data_path;

        let mut patterns = HashMap::new();

        // Check both possible locations
        let ud_dir = data_path("data/ud_english-ewt/UD_English-EWT");
        let ud_dir = if ud_dir.exists() {
            ud_dir
        } else {
            let alt = data_path("data/ud_english-ewt");
            if alt.exists() {
                alt
            } else {
                // No treebank available - return empty patterns
                tracing::warn!("UD English-EWT treebank not found, using heuristic parsing only");
                return patterns;
            }
        };

        let parser = ConlluParser::new();

        // Load patterns from each split
        for split in &["train", "dev", "test"] {
            let file_path = ud_dir.join(format!("en_ewt-ud-{split}.conllu"));
            if file_path.exists() {
                match parser.parse_file(&file_path) {
                    Ok(sentences) => {
                        for sentence in sentences {
                            if patterns.len() >= config.max_patterns {
                                break;
                            }

                            let normalized = Self::normalize_text(&sentence.text);
                            if let std::collections::hash_map::Entry::Vacant(e) =
                                patterns.entry(normalized)
                            {
                                if let Some(syntax) = Self::conllu_to_annotated(&sentence) {
                                    e.insert(syntax);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to parse {}: {}", file_path.display(), e);
                    }
                }
            }

            if patterns.len() >= config.max_patterns {
                break;
            }
        }

        tracing::info!("Loaded {} treebank patterns", patterns.len());
        patterns
    }

    /// Enhance syntax with pattern-based theta role hints.
    ///
    /// For each verb in the syntax, looks up expected argument patterns
    /// and attaches them as metadata for downstream semantic processing.
    fn enhance_with_patterns(&self, syntax: &mut AnnotatedSyntax) {
        let mut matcher = match self.pattern_matcher.lock() {
            Ok(guard) => guard,
            Err(e) => {
                tracing::warn!(
                    "Pattern matcher mutex poisoned, skipping enhancement: {}",
                    e
                );
                return;
            }
        };

        // Extract patterns from this syntax and add to matcher for future use
        let extracted = extract_patterns_from_syntax(syntax);
        matcher.add_treebank_patterns(extracted);

        // For each verb, get the matching pattern
        for token in &syntax.tokens {
            if token.upos == UPos::Verb {
                let signature = SemanticSignature::from_lemma(&token.lemma);
                // Match pattern - this caches for future lookups
                let _pattern = matcher.match_pattern(&signature);
                // Pattern information is stored in the matcher cache;
                // downstream semantic processing can query it via get_pattern()
            }
        }
    }

    /// Detect and annotate multi-word expressions in syntax.
    ///
    /// Populates `phrasal_verbs`, `mwes`, and `gerund_usage` fields based on
    /// dependency relations (compound:prt, compound, flat, fixed).
    #[allow(clippy::unused_self)] // Method pattern for future configuration
    fn annotate_mwes(&self, syntax: &mut AnnotatedSyntax) {
        // Detect phrasal verbs (verb + particle constructions)
        let pv_detector = PhrasalVerbDetector::new();
        for pred in syntax.predicates().map(|t| t.id).collect::<Vec<_>>() {
            if pv_detector.has_particle(syntax, pred) {
                let particles = pv_detector.find_particles(syntax, pred);
                let lemma = pv_detector.phrasal_lemma(syntax, pred);
                let span = pv_detector.phrasal_span(syntax, pred).unwrap_or((0, 0));

                syntax.phrasal_verbs.push(PhrasalVerb {
                    verb_id: pred,
                    particle_ids: particles,
                    combined_lemma: lemma,
                    span,
                });
            }
        }

        // Detect MWEs (compounds, flat names, fixed expressions)
        let mwe_detector = MweDetector::new();
        for mwe in mwe_detector.find_mwes(syntax) {
            let mwe_type = match mwe.mwe_type {
                super::mwe::MweType::CompoundNoun => MweType::CompoundNoun,
                super::mwe::MweType::FlatName => MweType::FlatName,
                super::mwe::MweType::FixedExpression => MweType::FixedExpression,
            };

            syntax.mwes.push(MweInfo {
                mwe_type,
                head_id: mwe.head_token,
                token_ids: mwe.tokens,
                combined_lemma: mwe.combined_lemma,
                span: mwe.span,
            });
        }

        // Classify gerund usage
        let classifier = GerundClassifier::new();
        for (token_id, usage) in classifier.classify_all(syntax) {
            if let Some(token) = syntax.tokens.get_mut(token_id.index()) {
                token.feats.gerund_usage = Some(usage);
            }
        }
    }

    /// Get pattern matcher statistics.
    #[must_use]
    pub fn pattern_stats(&self) -> Option<super::pattern_matcher::MatcherStats> {
        self.pattern_matcher.lock().ok().map(|m| m.stats().clone())
    }

    /// Get a pattern for a semantic signature (read-only).
    #[must_use]
    pub fn get_pattern(
        &self,
        signature: &SemanticSignature,
    ) -> Option<super::pattern_types::DependencyPattern> {
        self.pattern_matcher.lock().ok()?.get_pattern(signature)
    }

    /// Get patterns for all verbs in the given syntax.
    ///
    /// Returns a map from token ID to matched dependency pattern.
    /// Only includes tokens that are verbs and have a matched pattern.
    #[must_use]
    pub fn get_patterns_for_syntax(
        &self,
        syntax: &AnnotatedSyntax,
    ) -> HashMap<TokenId, super::pattern_types::DependencyPattern> {
        let mut patterns = HashMap::new();

        let Ok(matcher) = self.pattern_matcher.lock() else {
            return patterns;
        };

        for token in &syntax.tokens {
            if token.upos == UPos::Verb {
                let signature = SemanticSignature::from_lemma(&token.lemma);
                if let Some(pattern) = matcher.get_pattern(&signature) {
                    patterns.insert(token.id, pattern);
                }
            }
        }

        patterns
    }

    /// Normalize text for pattern matching.
    fn normalize_text(text: &str) -> String {
        text.to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Convert a CoNLL-U sentence to `AnnotatedSyntax`.
    fn conllu_to_annotated(sentence: &ConlluSentence) -> Option<AnnotatedSyntax> {
        let text = &sentence.text;
        let mut tokens = Vec::with_capacity(sentence.tokens.len());
        let mut byte_offset = 0;

        for (idx, token) in sentence.tokens.iter().enumerate() {
            let upos = parse_upos(&token.upos);
            let deprel = parse_deprel(&token.deprel);

            // Calculate byte span (Rust strings are byte-indexed)
            let form = &token.form;
            let start = text[byte_offset..]
                .find(form.as_str())
                .map_or(byte_offset, |p| byte_offset + p);
            let end = start + form.len(); // .len() returns byte length
            byte_offset = end;

            let mut annotated = AnnotatedToken::new(
                TokenId::new(idx),
                token.form.clone(),
                token.lemma.clone(),
                upos,
                deprel,
                (start, end),
            );

            // Set head (CoNLL-U uses 1-indexed, we use 0-indexed)
            if token.head > 0 {
                annotated.head = Some(TokenId::new((token.head - 1) as usize));
            }

            // Set xpos if present
            annotated.xpos.clone_from(&token.xpos);

            // Parse morphological features
            annotated.feats = Self::parse_features(&token.features);

            tokens.push(annotated);
        }

        if tokens.is_empty() {
            return None;
        }

        Some(AnnotatedSyntax::new(text.clone(), tokens))
    }

    /// Parse morphological features from CoNLL-U format.
    fn parse_features(features: &HashMap<String, String>) -> MorphFeatures {
        use canopy::core::{
            Case, Definiteness, Gender, Mood, MorphVoice, Number, Person, Tense, VerbForm,
        };

        let mut morph = MorphFeatures::default();

        for (key, value) in features {
            match key.as_str() {
                "Number" => {
                    morph.number = match value.as_str() {
                        "Sing" => Some(Number::Singular),
                        "Plur" => Some(Number::Plural),
                        _ => None,
                    };
                }
                "Person" => {
                    morph.person = match value.as_str() {
                        "1" => Some(Person::First),
                        "2" => Some(Person::Second),
                        "3" => Some(Person::Third),
                        _ => None,
                    };
                }
                "Tense" => {
                    morph.tense = match value.as_str() {
                        "Past" => Some(Tense::Past),
                        "Pres" => Some(Tense::Present),
                        "Fut" => Some(Tense::Future),
                        _ => None,
                    };
                }
                "VerbForm" => {
                    morph.verb_form = match value.as_str() {
                        "Fin" => Some(VerbForm::Finite),
                        "Inf" => Some(VerbForm::Infinitive),
                        "Part" => Some(VerbForm::Participle),
                        "Ger" => Some(VerbForm::Gerund),
                        _ => None,
                    };
                }
                "Mood" => {
                    morph.mood = match value.as_str() {
                        "Ind" => Some(Mood::Indicative),
                        "Imp" => Some(Mood::Imperative),
                        "Sub" => Some(Mood::Subjunctive),
                        "Cnd" => Some(Mood::Conditional),
                        _ => None,
                    };
                }
                "Voice" => {
                    morph.voice = match value.as_str() {
                        "Act" => Some(MorphVoice::Active),
                        "Pass" => Some(MorphVoice::Passive),
                        _ => None,
                    };
                }
                "Case" => {
                    morph.case = match value.as_str() {
                        "Nom" => Some(Case::Nominative),
                        "Acc" => Some(Case::Accusative),
                        "Dat" => Some(Case::Dative),
                        "Gen" => Some(Case::Genitive),
                        _ => None,
                    };
                }
                "Definite" => {
                    morph.definiteness = match value.as_str() {
                        "Def" => Some(Definiteness::Definite),
                        "Ind" => Some(Definiteness::Indefinite),
                        _ => None,
                    };
                }
                "Gender" => {
                    morph.gender = match value.as_str() {
                        "Masc" => Some(Gender::Masculine),
                        "Fem" => Some(Gender::Feminine),
                        "Neut" => Some(Gender::Neuter),
                        _ => None,
                    };
                }
                _ => {}
            }
        }

        morph
    }
}

impl SyntaxProvider for TreebankSyntaxProvider {
    fn parse(&self, text: &str) -> Result<AnnotatedSyntax, CanopyError> {
        // Try exact match first
        let normalized = Self::normalize_text(text);
        let mut syntax = if let Some(cached) = self.pattern_index.get(&normalized) {
            // Clone and update spans for the actual text
            let mut syntax = cached.clone();
            syntax.text = text.to_string();
            syntax
        } else {
            // Fall back to resource-backed tagging (queries treebank, VerbNet, WordNet)
            let tokens = self.tokenizer.tokenize(text);
            self.tagger.parse(text, &tokens)
        };

        // Enhance with pattern matching for verbs
        self.enhance_with_patterns(&mut syntax);

        // Detect and annotate multi-word expressions
        self.annotate_mwes(&mut syntax);

        Ok(syntax)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use canopy::UPos;

    fn data_available() -> bool {
        crate::paths::data_path("data/lexicon").exists()
    }

    #[test]
    fn test_normalize_text() {
        assert_eq!(
            TreebankSyntaxProvider::normalize_text("Hello, World!"),
            "hello world"
        );
        assert_eq!(
            TreebankSyntaxProvider::normalize_text("  The  cat  runs.  "),
            "the cat runs"
        );
    }

    #[test]
    fn test_provider_creation() {
        if !data_available() {
            eprintln!("Skipping: Lexicon data not available");
            return;
        }

        let provider = TreebankSyntaxProvider::new();
        assert!(
            provider.is_ok(),
            "Failed to create provider: {:?}",
            provider.err()
        );
    }

    #[test]
    fn test_parse_simple_sentence() {
        if !data_available() {
            eprintln!("Skipping: Lexicon data not available");
            return;
        }

        let provider = TreebankSyntaxProvider::new().unwrap();
        let syntax = provider.parse("The cat runs.").unwrap();

        assert!(!syntax.tokens.is_empty(), "Should have tokens");
        assert_eq!(syntax.text, "The cat runs.");

        // Should have at least one verb
        let has_verb = syntax.tokens.iter().any(|t| matches!(t.upos, UPos::Verb));
        assert!(has_verb, "Should detect verb");
    }

    #[test]
    fn test_pattern_matching() {
        if !data_available() {
            eprintln!("Skipping: Lexicon data not available");
            return;
        }

        // This test checks that pattern matching works
        // The exact sentence may or may not be in the treebank
        let provider = TreebankSyntaxProvider::new().unwrap();

        // Parse a sentence and check structure
        let syntax = provider.parse("John gave Mary a book.").unwrap();
        assert!(!syntax.tokens.is_empty());

        // Find the root
        let root = syntax.root();
        assert!(root.is_some(), "Should have a root token");
    }

    #[test]
    fn test_morph_features() {
        let mut features = HashMap::new();
        features.insert("Number".to_string(), "Sing".to_string());
        features.insert("Person".to_string(), "3".to_string());
        features.insert("Tense".to_string(), "Pres".to_string());

        let morph = TreebankSyntaxProvider::parse_features(&features);

        assert_eq!(morph.number, Some(canopy::core::Number::Singular));
        assert_eq!(morph.person, Some(canopy::core::Person::Third));
        assert_eq!(morph.tense, Some(canopy::core::Tense::Present));
    }

    #[test]
    fn test_phrasal_verb_annotation() {
        // Test that annotate_mwes correctly detects phrasal verbs
        // when compound:prt relation is present
        use canopy::core::{DepRel, MorphFeatures};

        let mut syntax = AnnotatedSyntax::new(
            "He gave up.".to_string(),
            vec![
                AnnotatedToken {
                    id: TokenId::new(0),
                    form: "He".to_string(),
                    lemma: "he".to_string(),
                    upos: UPos::Pron,
                    xpos: None,
                    feats: MorphFeatures::default(),
                    head: Some(TokenId::new(1)),
                    deprel: DepRel::Nsubj,
                    span: (0, 2),
                },
                AnnotatedToken {
                    id: TokenId::new(1),
                    form: "gave".to_string(),
                    lemma: "give".to_string(),
                    upos: UPos::Verb,
                    xpos: None,
                    feats: MorphFeatures::default(),
                    head: None,
                    deprel: DepRel::Root,
                    span: (3, 7),
                },
                AnnotatedToken {
                    id: TokenId::new(2),
                    form: "up".to_string(),
                    lemma: "up".to_string(),
                    upos: UPos::Part,
                    xpos: None,
                    feats: MorphFeatures::default(),
                    head: Some(TokenId::new(1)),
                    deprel: DepRel::CompoundPrt, // Key: particle relation
                    span: (8, 10),
                },
            ],
        );

        // Manually run annotate_mwes (simulating what parse() does)
        let pv_detector = super::super::phrasal_verb::PhrasalVerbDetector::new();
        for pred in syntax.predicates().map(|t| t.id).collect::<Vec<_>>() {
            if pv_detector.has_particle(&syntax, pred) {
                let particles = pv_detector.find_particles(&syntax, pred);
                let lemma = pv_detector.phrasal_lemma(&syntax, pred);
                let span = pv_detector.phrasal_span(&syntax, pred).unwrap_or((0, 0));

                syntax.phrasal_verbs.push(PhrasalVerb {
                    verb_id: pred,
                    particle_ids: particles,
                    combined_lemma: lemma,
                    span,
                });
            }
        }

        // Verify phrasal verb was detected
        assert_eq!(syntax.phrasal_verbs.len(), 1);
        assert_eq!(syntax.phrasal_verbs[0].combined_lemma, "give_up");
        assert_eq!(syntax.phrasal_verbs[0].verb_id, TokenId::new(1));

        // Verify get_predicate_lemma returns phrasal form
        assert_eq!(syntax.get_predicate_lemma(TokenId::new(1)), Some("give_up"));
    }
}
