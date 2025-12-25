//! Treebank sentence loader for gold-standard CoNLL-U data
//!
//! This module provides loading and conversion of pre-parsed UD treebank sentences.
//! Unlike heuristic parsing, this uses gold-standard POS tags, lemmas, and dependencies
//! from the Universal Dependencies English-EWT corpus.

use crate::{AnalysisResult, CanopyError, DepRel, MorphFeatures, UPos, Word};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

/// A parsed sentence from the UD treebank with gold-standard annotations
#[derive(Debug, Clone)]
pub struct TreebankSentence {
    /// Sentence ID (e.g., "en_ewt-ud-dev-0001")
    pub sent_id: String,
    /// Original text
    pub text: String,
    /// Parsed tokens with gold annotations
    pub tokens: Vec<TreebankToken>,
}

/// A token from the treebank with gold-standard linguistic annotations
#[derive(Debug, Clone)]
pub struct TreebankToken {
    /// Token ID (1-indexed)
    pub id: usize,
    /// Surface form
    pub form: String,
    /// Gold lemma
    pub lemma: String,
    /// Universal POS tag
    pub upos: String,
    /// Language-specific POS tag (optional)
    pub xpos: Option<String>,
    /// Morphological features
    pub features: HashMap<String, String>,
    /// Head token ID (0 for root)
    pub head: usize,
    /// Dependency relation
    pub deprel: String,
    /// Character start position
    pub start: usize,
    /// Character end position
    pub end: usize,
}

/// Loads and manages parsed sentences from UD treebank CoNLL-U files
pub struct TreebankSentenceLoader {
    /// Development set sentences
    dev_sentences: Vec<TreebankSentence>,
    /// Training set sentences
    train_sentences: Vec<TreebankSentence>,
    /// Test set sentences (optional)
    test_sentences: Vec<TreebankSentence>,
    /// Index mapping sent_id to (dataset, index) for fast lookup
    sentence_index: HashMap<String, (Dataset, usize)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Dataset {
    Dev,
    Train,
    Test,
}

impl TreebankSentenceLoader {
    /// Create a new loader with default UD English-EWT paths
    ///
    /// # Errors
    ///
    /// Returns an error if the treebank files cannot be loaded
    pub fn new() -> AnalysisResult<Self> {
        let base_path = PathBuf::from("data/ud_english-ewt/UD_English-EWT");
        Self::from_path(&base_path)
    }

    /// Create a loader from a custom path
    ///
    /// Expected structure:
    /// ```text
    /// path/
    ///   en_ewt-ud-dev.conllu
    ///   en_ewt-ud-train.conllu
    ///   en_ewt-ud-test.conllu  (optional)
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the files cannot be read or parsed
    pub fn from_path(path: &Path) -> AnalysisResult<Self> {
        let dev_path = path.join("en_ewt-ud-dev.conllu");
        let train_path = path.join("en_ewt-ud-train.conllu");
        let test_path = path.join("en_ewt-ud-test.conllu");

        let dev_sentences = Self::load_conllu_file(&dev_path)?;
        let train_sentences = Self::load_conllu_file(&train_path)?;
        let test_sentences = if test_path.exists() {
            Self::load_conllu_file(&test_path)?
        } else {
            Vec::new()
        };

        let mut sentence_index = HashMap::new();
        for (i, sentence) in dev_sentences.iter().enumerate() {
            sentence_index.insert(sentence.sent_id.clone(), (Dataset::Dev, i));
        }
        for (i, sentence) in train_sentences.iter().enumerate() {
            sentence_index.insert(sentence.sent_id.clone(), (Dataset::Train, i));
        }
        for (i, sentence) in test_sentences.iter().enumerate() {
            sentence_index.insert(sentence.sent_id.clone(), (Dataset::Test, i));
        }

        Ok(Self {
            dev_sentences,
            train_sentences,
            test_sentences,
            sentence_index,
        })
    }

    /// Load a single CoNLL-U file for testing or custom corpora
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be loaded
    pub fn from_file(path: &Path) -> AnalysisResult<Vec<TreebankSentence>> {
        Self::load_conllu_file(path)
    }

    /// Get a sentence by its ID
    pub fn get_sentence(&self, sent_id: &str) -> Option<&TreebankSentence> {
        self.sentence_index
            .get(sent_id)
            .map(|(dataset, idx)| match dataset {
                Dataset::Dev => &self.dev_sentences[*idx],
                Dataset::Train => &self.train_sentences[*idx],
                Dataset::Test => &self.test_sentences[*idx],
            })
    }

    /// Get a dev sentence by index
    pub fn get_dev_by_index(&self, idx: usize) -> Option<&TreebankSentence> {
        self.dev_sentences.get(idx)
    }

    /// Get a train sentence by index
    pub fn get_train_by_index(&self, idx: usize) -> Option<&TreebankSentence> {
        self.train_sentences.get(idx)
    }

    /// Iterator over dev sentences
    pub fn iter_dev(&self) -> impl Iterator<Item = &TreebankSentence> {
        self.dev_sentences.iter()
    }

    /// Iterator over train sentences
    pub fn iter_train(&self) -> impl Iterator<Item = &TreebankSentence> {
        self.train_sentences.iter()
    }

    /// Number of dev sentences
    pub fn dev_count(&self) -> usize {
        self.dev_sentences.len()
    }

    /// Number of train sentences
    pub fn train_count(&self) -> usize {
        self.train_sentences.len()
    }

    /// Number of test sentences
    pub fn test_count(&self) -> usize {
        self.test_sentences.len()
    }

    /// Total number of sentences across all datasets
    pub fn total_count(&self) -> usize {
        self.dev_count() + self.train_count() + self.test_count()
    }

    /// List available sentence IDs (up to limit)
    pub fn list_available(&self, limit: usize) -> Vec<String> {
        self.sentence_index.keys().take(limit).cloned().collect()
    }

    /// Convert a treebank sentence to Word structures for semantic analysis
    ///
    /// Uses gold-standard POS tags, lemmas, and dependencies from the treebank.
    ///
    /// # Errors
    ///
    /// Returns an error if conversion fails (e.g., invalid POS tag)
    pub fn convert_to_words(&self, sentence: &TreebankSentence) -> AnalysisResult<Vec<Word>> {
        let mut words = Vec::with_capacity(sentence.tokens.len());

        for token in &sentence.tokens {
            // Parse UPos from string
            let upos = Self::parse_upos(&token.upos)?;

            // Parse DepRel from string
            let deprel = Self::parse_deprel(&token.deprel)?;

            // Convert features
            let feats = Self::convert_features(&token.features);

            let word = Word {
                id: token.id,
                text: token.form.clone(),
                lemma: token.lemma.clone(),
                upos,
                xpos: token.xpos.clone(),
                feats,
                head: Some(token.head),
                deprel,
                deps: None, // Enhanced dependencies not needed for now
                misc: None,
                start: token.start,
                end: token.end,
            };

            words.push(word);
        }

        Ok(words)
    }

    /// Load sentences from a CoNLL-U file
    fn load_conllu_file(path: &Path) -> AnalysisResult<Vec<TreebankSentence>> {
        let file = File::open(path).map_err(|e| CanopyError::ParseError {
            context: format!("Failed to open treebank file {}: {}", path.display(), e),
        })?;

        let reader = BufReader::new(file);
        let mut sentences = Vec::new();
        let mut current_tokens = Vec::new();
        let mut current_sent_id = None;
        let mut current_text = None;

        for line in reader.lines() {
            let line = line.map_err(|e| CanopyError::ParseError {
                context: format!("Failed to read line: {}", e),
            })?;

            if line.starts_with('#') {
                // Parse metadata
                if let Some(sent_id) = line.strip_prefix("# sent_id = ") {
                    current_sent_id = Some(sent_id.trim().to_string());
                } else if let Some(text) = line.strip_prefix("# text = ") {
                    current_text = Some(text.trim().to_string());
                }
            } else if line.trim().is_empty() {
                // End of sentence
                if !current_tokens.is_empty() {
                    sentences.push(TreebankSentence {
                        sent_id: current_sent_id
                            .take()
                            .unwrap_or_else(|| format!("sent-{}", sentences.len())),
                        text: current_text.take().unwrap_or_default(),
                        tokens: current_tokens.clone(),
                    });
                    current_tokens.clear();
                }
            } else if !line.contains('-') && !line.contains('.') {
                // Regular token line (skip multiword tokens and empty nodes)
                let fields: Vec<&str> = line.split('\t').collect();
                if fields.len() >= 10
                    && let Ok(id) = fields[0].parse::<usize>()
                {
                    // Parse features
                    let features = Self::parse_features(fields[5]);

                    // Calculate character positions (approximate from tokens)
                    let start = current_tokens
                        .iter()
                        .map(|t: &TreebankToken| t.form.len() + 1)
                        .sum();
                    let end = start + fields[1].len();

                    let token = TreebankToken {
                        id,
                        form: fields[1].to_string(),
                        lemma: fields[2].to_string(),
                        upos: fields[3].to_string(),
                        xpos: if fields[4] == "_" {
                            None
                        } else {
                            Some(fields[4].to_string())
                        },
                        features,
                        head: fields[6].parse().unwrap_or(0),
                        deprel: fields[7].to_string(),
                        start,
                        end,
                    };
                    current_tokens.push(token);
                }
            }
        }

        // Handle last sentence if file doesn't end with blank line
        if !current_tokens.is_empty() {
            sentences.push(TreebankSentence {
                sent_id: current_sent_id.unwrap_or_else(|| format!("sent-{}", sentences.len())),
                text: current_text.unwrap_or_default(),
                tokens: current_tokens,
            });
        }

        Ok(sentences)
    }

    /// Parse features from CoNLL-U format (e.g., "Gender=Masc|Number=Sing")
    fn parse_features(feat_str: &str) -> HashMap<String, String> {
        if feat_str == "_" {
            return HashMap::new();
        }

        feat_str
            .split('|')
            .filter_map(|pair| {
                let parts: Vec<&str> = pair.split('=').collect();
                if parts.len() == 2 {
                    Some((parts[0].to_string(), parts[1].to_string()))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Convert treebank features to MorphFeatures
    fn convert_features(features: &HashMap<String, String>) -> MorphFeatures {
        use crate::{UDNumber, UDPerson, UDTense, UDVerbForm};

        let mut feats = MorphFeatures::default();

        // Convert person
        if let Some(person) = features.get("Person") {
            feats.person = match person.as_str() {
                "1" => Some(UDPerson::First),
                "2" => Some(UDPerson::Second),
                "3" => Some(UDPerson::Third),
                _ => None,
            };
        }

        // Convert number
        if let Some(number) = features.get("Number") {
            feats.number = match number.as_str() {
                "Sing" => Some(UDNumber::Singular),
                "Plur" => Some(UDNumber::Plural),
                _ => None,
            };
        }

        // Convert tense
        if let Some(tense) = features.get("Tense") {
            feats.tense = match tense.as_str() {
                "Past" => Some(UDTense::Past),
                "Pres" => Some(UDTense::Present),
                "Fut" => Some(UDTense::Future),
                _ => None,
            };
        }

        // Convert verb form
        if let Some(verbform) = features.get("VerbForm") {
            feats.verbform = match verbform.as_str() {
                "Fin" => Some(UDVerbForm::Finite),
                "Inf" => Some(UDVerbForm::Infinitive),
                "Part" => Some(UDVerbForm::Participle),
                "Ger" => Some(UDVerbForm::Gerund),
                _ => None,
            };
        }

        feats
    }

    /// Parse UPos from string
    fn parse_upos(upos_str: &str) -> AnalysisResult<UPos> {
        match upos_str {
            "ADJ" => Ok(UPos::Adj),
            "ADP" => Ok(UPos::Adp),
            "ADV" => Ok(UPos::Adv),
            "AUX" => Ok(UPos::Aux),
            "CCONJ" => Ok(UPos::Cconj),
            "DET" => Ok(UPos::Det),
            "INTJ" => Ok(UPos::Intj),
            "NOUN" => Ok(UPos::Noun),
            "NUM" => Ok(UPos::Num),
            "PART" => Ok(UPos::Part),
            "PRON" => Ok(UPos::Pron),
            "PROPN" => Ok(UPos::Propn),
            "PUNCT" => Ok(UPos::Punct),
            "SCONJ" => Ok(UPos::Sconj),
            "SYM" => Ok(UPos::Sym),
            "VERB" => Ok(UPos::Verb),
            "X" => Ok(UPos::X),
            _ => Err(CanopyError::ParseError {
                context: format!("Unknown UPos tag: {}", upos_str),
            }),
        }
    }

    /// Parse DepRel from string
    fn parse_deprel(deprel_str: &str) -> AnalysisResult<DepRel> {
        // Take base relation (before :) for enhanced dependencies
        let base = deprel_str.split(':').next().unwrap_or(deprel_str);

        match base {
            "root" => Ok(DepRel::Root),
            "nsubj" => Ok(DepRel::Nsubj),
            "obj" => Ok(DepRel::Obj),
            "iobj" => Ok(DepRel::Iobj),
            "csubj" => Ok(DepRel::Csubj),
            "ccomp" => Ok(DepRel::Ccomp),
            "xcomp" => Ok(DepRel::Xcomp),
            "obl" => Ok(DepRel::Obl),
            "vocative" => Ok(DepRel::Vocative),
            "expl" => Ok(DepRel::Expl),
            "dislocated" => Ok(DepRel::Dislocated),
            "advcl" => Ok(DepRel::Advcl),
            "advmod" => Ok(DepRel::Advmod),
            "discourse" => Ok(DepRel::Discourse),
            "aux" => Ok(DepRel::Aux),
            "cop" => Ok(DepRel::Cop),
            "mark" => Ok(DepRel::Mark),
            "nmod" => Ok(DepRel::Nmod),
            "appos" => Ok(DepRel::Appos),
            "nummod" => Ok(DepRel::Nummod),
            "acl" => Ok(DepRel::Acl),
            "amod" => Ok(DepRel::Amod),
            "det" => Ok(DepRel::Det),
            "clf" => Ok(DepRel::Clf),
            "case" => Ok(DepRel::Case),
            "conj" => Ok(DepRel::Conj),
            "cc" => Ok(DepRel::Cc),
            "fixed" => Ok(DepRel::Fixed),
            "flat" => Ok(DepRel::Flat),
            "compound" => Ok(DepRel::Compound),
            "list" => Ok(DepRel::List),
            "parataxis" => Ok(DepRel::Parataxis),
            "orphan" => Ok(DepRel::Orphan),
            "goeswith" => Ok(DepRel::Goeswith),
            "reparandum" => Ok(DepRel::Reparandum),
            "punct" => Ok(DepRel::Punct),
            "dep" => Ok(DepRel::Dep),
            _ => Ok(DepRel::Dep), // Fallback for unknown relations
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_parse_features() {
        let features = TreebankSentenceLoader::parse_features("Gender=Masc|Number=Sing");
        assert_eq!(features.get("Gender"), Some(&"Masc".to_string()));
        assert_eq!(features.get("Number"), Some(&"Sing".to_string()));

        let empty = TreebankSentenceLoader::parse_features("_");
        assert!(empty.is_empty());
    }

    #[test]
    fn test_parse_upos() {
        assert_eq!(
            TreebankSentenceLoader::parse_upos("NOUN").unwrap(),
            UPos::Noun
        );
        assert_eq!(
            TreebankSentenceLoader::parse_upos("VERB").unwrap(),
            UPos::Verb
        );
        assert_eq!(
            TreebankSentenceLoader::parse_upos("DET").unwrap(),
            UPos::Det
        );
    }

    #[test]
    fn test_parse_deprel() {
        assert_eq!(
            TreebankSentenceLoader::parse_deprel("root").unwrap(),
            DepRel::Root
        );
        assert_eq!(
            TreebankSentenceLoader::parse_deprel("nsubj").unwrap(),
            DepRel::Nsubj
        );
        assert_eq!(
            TreebankSentenceLoader::parse_deprel("nsubj:pass").unwrap(),
            DepRel::Nsubj
        ); // Base relation
    }

    #[test]
    fn test_load_canonical_fixtures() {
        let fixture_path = Path::new("tests/fixtures/canonical.conllu");
        if !fixture_path.exists() {
            // Skip if fixtures not yet created
            return;
        }

        let sentences = TreebankSentenceLoader::from_file(fixture_path).unwrap();

        assert_eq!(sentences.len(), 20, "Should load 20 canonical sentences");
        assert_eq!(sentences[0].sent_id, "canonical-001");
        assert_eq!(sentences[0].text, "John gave Mary a book.");
        assert_eq!(sentences[0].tokens.len(), 6); // 5 words + punctuation

        // Test passive sentence
        assert_eq!(sentences[1].sent_id, "canonical-002");
        assert_eq!(sentences[1].text, "The book was given to Mary.");

        // Test convert_to_words - need a full loader for this
        // Create a minimal loader just for the conversion test
        let base_path = Path::new("data/ud_english-ewt/UD_English-EWT");
        if base_path.exists() {
            let loader = TreebankSentenceLoader::from_path(base_path).unwrap();
            let words = loader.convert_to_words(&sentences[0]).unwrap();
            assert_eq!(words.len(), 6);
            assert_eq!(words[0].text, "John");
            assert_eq!(words[0].lemma, "John");
            assert_eq!(words[0].upos, UPos::Propn);
            assert_eq!(words[1].text, "gave");
            assert_eq!(words[1].lemma, "give");
            assert_eq!(words[1].upos, UPos::Verb);
        }
    }
}
