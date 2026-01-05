//! `TreebankSyntaxProvider` implementation.
//!
//! A `SyntaxProvider` that uses patterns from UD English-EWT treebank
//! with resource-backed fallback for unknown patterns.

use super::resource_tagger::ResourceBackedTagger;
use super::shared::{parse_deprel, parse_upos};
use crate::engine::{ConlluParser, ConlluSentence, SharedEngines};
use crate::tokenizer::{SimpleTokenizer, Tokenizer};
use canopy::runtime::{AnnotatedSyntax, AnnotatedToken, SyntaxProvider, TokenId};
use canopy::{CanopyError, MorphFeatures};
use std::collections::HashMap;

/// A `SyntaxProvider` that matches patterns from UD treebank.
#[derive(Debug)]
pub struct TreebankSyntaxProvider {
    /// Tokenizer for splitting text.
    tokenizer: SimpleTokenizer,
    /// Resource-backed tagger for fallback (queries treebank, `VerbNet`, `WordNet`).
    tagger: ResourceBackedTagger,
    /// Pattern index: normalized text → `AnnotatedSyntax`.
    pattern_index: HashMap<String, AnnotatedSyntax>,
    /// Configuration (for future use in more sophisticated matching).
    #[allow(dead_code)]
    config: TreebankConfig,
}

/// Configuration for the treebank provider.
#[derive(Debug, Clone)]
pub struct TreebankConfig {
    /// Maximum number of patterns to index.
    pub max_patterns: usize,
    /// Whether to use exact matching only.
    pub exact_match_only: bool,
}

impl Default for TreebankConfig {
    fn default() -> Self {
        Self {
            max_patterns: 20000,
            exact_match_only: false,
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

        Ok(Self {
            tokenizer,
            tagger,
            pattern_index,
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

        Ok(Self {
            tokenizer,
            tagger,
            pattern_index,
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
        if let Some(syntax) = self.pattern_index.get(&normalized) {
            // Clone and update spans for the actual text
            let mut syntax = syntax.clone();
            syntax.text = text.to_string();
            return Ok(syntax);
        }

        // Fall back to resource-backed tagging (queries treebank, VerbNet, WordNet)
        let tokens = self.tokenizer.tokenize(text);
        Ok(self.tagger.parse(text, &tokens))
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
}
