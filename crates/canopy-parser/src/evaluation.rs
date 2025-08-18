//! Corpus accuracy evaluation framework
//!
//! This module provides tools for evaluating parser accuracy against gold standard
//! annotations from corpora like Penn Treebank and Universal Dependencies.

use crate::udpipe::UDPipeParser;
use canopy_core::{DepRel, UPos};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
// use std::io::BufRead;  // TODO: Remove if not needed for streaming CoNLL-U parsing
use std::path::Path;
use thiserror::Error;

/// Errors that can occur during corpus evaluation
#[derive(Debug, Error)]
pub enum EvaluationError {
    #[error("Failed to load gold standard file: {source}")]
    GoldStandardLoadError {
        #[source]
        source: std::io::Error,
    },

    #[error("Parsing failed for sentence {index}: {source}")]
    ParsingError {
        index: usize,
        #[source]
        source: crate::udpipe::ParseError,
    },

    #[error("Mismatched sentence count: expected {expected}, got {actual}")]
    SentenceCountMismatch { expected: usize, actual: usize },

    #[error("Invalid annotation format in line {line}: {reason}")]
    InvalidAnnotationFormat { line: usize, reason: String },
}

/// A gold standard annotation for evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoldStandardSentence {
    /// Original text
    pub text: String,

    /// Gold standard words with annotations
    pub words: Vec<GoldStandardWord>,
}

/// A gold standard word annotation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoldStandardWord {
    /// Word form
    pub form: String,

    /// Lemma
    pub lemma: String,

    /// Universal POS tag
    pub upos: UPos,

    /// Dependency relation
    pub deprel: DepRel,

    /// Head word index (1-based)
    pub head: usize,

    /// Character start position
    pub start: usize,

    /// Character end position
    pub end: usize,
}

/// Accuracy metrics for evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccuracyMetrics {
    /// Total number of tokens evaluated
    pub total_tokens: usize,

    /// Universal POS tagging accuracy
    pub upos_accuracy: f64,

    /// Lemmatization accuracy
    pub lemma_accuracy: f64,

    /// Unlabeled attachment score (UAS)
    pub uas: f64,

    /// Labeled attachment score (LAS)
    pub las: f64,

    /// Sentence-level exact match accuracy
    pub sentence_accuracy: f64,

    /// Detailed per-tag accuracy
    pub per_tag_accuracy: HashMap<String, f64>,

    /// Processing time in milliseconds
    pub processing_time_ms: f64,
}

impl Default for AccuracyMetrics {
    fn default() -> Self {
        Self {
            total_tokens: 0,
            upos_accuracy: 0.0,
            lemma_accuracy: 0.0,
            uas: 0.0,
            las: 0.0,
            sentence_accuracy: 0.0,
            per_tag_accuracy: HashMap::new(),
            processing_time_ms: 0.0,
        }
    }
}

/// Corpus accuracy evaluator
pub struct CorpusEvaluator {
    parser: UDPipeParser,
}

impl CorpusEvaluator {
    /// Create a new corpus evaluator
    pub fn new(parser: UDPipeParser) -> Self {
        Self { parser }
    }

    /// Evaluate parser accuracy against gold standard annotations
    pub fn evaluate(
        &self,
        gold_sentences: &[GoldStandardSentence],
    ) -> Result<AccuracyMetrics, EvaluationError> {
        let start_time = std::time::Instant::now();

        let mut total_tokens = 0;
        let mut correct_upos = 0;
        let mut correct_lemma = 0;
        let mut correct_head = 0;
        let mut correct_deprel = 0;
        let mut correct_sentences = 0;
        let mut per_tag_counts: HashMap<String, (usize, usize)> = HashMap::new(); // (correct, total)

        for (sent_idx, gold_sentence) in gold_sentences.iter().enumerate() {
            // Parse the sentence
            let parsed_doc = self
                .parser
                .parse_document(&gold_sentence.text)
                .map_err(|source| EvaluationError::ParsingError {
                    index: sent_idx,
                    source,
                })?;

            if parsed_doc.sentences.is_empty() {
                continue; // Skip empty parses
            }

            let parsed_sentence = &parsed_doc.sentences[0]; // Take first sentence
            let mut sentence_perfect = true;

            // Compare words (align by position for now)
            let min_len = std::cmp::min(gold_sentence.words.len(), parsed_sentence.words.len());

            for i in 0..min_len {
                let gold_word = &gold_sentence.words[i];
                let parsed_word = &parsed_sentence.words[i];

                total_tokens += 1;

                // UPOS accuracy
                if gold_word.upos == parsed_word.upos {
                    correct_upos += 1;
                } else {
                    sentence_perfect = false;
                }

                // Lemma accuracy
                if gold_word.lemma == parsed_word.lemma {
                    correct_lemma += 1;
                } else {
                    sentence_perfect = false;
                }

                // Head accuracy (UAS)
                let parsed_head = parsed_word.head.unwrap_or(0);
                if gold_word.head == parsed_head {
                    correct_head += 1;
                } else {
                    sentence_perfect = false;
                }

                // Labeled attachment (LAS) - convert deprel string to enum for comparison
                let parsed_deprel = Self::convert_deprel_from_conllu(&parsed_word.deprel);
                if gold_word.head == parsed_head && gold_word.deprel == parsed_deprel {
                    correct_deprel += 1;
                } else {
                    sentence_perfect = false;
                }

                // Per-tag accuracy
                let tag_str = format!("{:?}", gold_word.upos);
                let (correct, total) = per_tag_counts.entry(tag_str.clone()).or_insert((0, 0));
                *total += 1;
                if gold_word.upos == parsed_word.upos {
                    *correct += 1;
                }
            }

            if sentence_perfect && gold_sentence.words.len() == parsed_sentence.words.len() {
                correct_sentences += 1;
            }
        }

        let processing_time_ms = start_time.elapsed().as_millis() as f64;

        // Calculate final metrics
        let upos_accuracy = if total_tokens > 0 {
            correct_upos as f64 / total_tokens as f64
        } else {
            0.0
        };
        let lemma_accuracy = if total_tokens > 0 {
            correct_lemma as f64 / total_tokens as f64
        } else {
            0.0
        };
        let uas = if total_tokens > 0 {
            correct_head as f64 / total_tokens as f64
        } else {
            0.0
        };
        let las = if total_tokens > 0 {
            correct_deprel as f64 / total_tokens as f64
        } else {
            0.0
        };
        let sentence_accuracy = if !gold_sentences.is_empty() {
            correct_sentences as f64 / gold_sentences.len() as f64
        } else {
            0.0
        };

        let per_tag_accuracy: HashMap<String, f64> = per_tag_counts
            .into_iter()
            .map(|(tag, (correct, total))| {
                let accuracy = if total > 0 {
                    correct as f64 / total as f64
                } else {
                    0.0
                };
                (tag, accuracy)
            })
            .collect();

        Ok(AccuracyMetrics {
            total_tokens,
            upos_accuracy,
            lemma_accuracy,
            uas,
            las,
            sentence_accuracy,
            per_tag_accuracy,
            processing_time_ms,
        })
    }

    /// Load gold standard from CoNLL-U format (simplified implementation)
    pub fn load_conllu<P: AsRef<Path>>(
        path: P,
    ) -> Result<Vec<GoldStandardSentence>, EvaluationError> {
        let content = fs::read_to_string(path)
            .map_err(|source| EvaluationError::GoldStandardLoadError { source })?;

        Self::parse_conllu(&content)
    }

    /// Parse CoNLL-U format content
    pub fn parse_conllu(content: &str) -> Result<Vec<GoldStandardSentence>, EvaluationError> {
        let mut sentences = Vec::new();
        let mut current_words = Vec::new();
        let mut current_text = String::new();

        for (line_no, line) in content.lines().enumerate() {
            let line = line.trim();

            if line.is_empty() {
                // End of sentence
                if !current_words.is_empty() {
                    sentences.push(GoldStandardSentence {
                        text: current_text.clone(),
                        words: current_words,
                    });
                    current_words = Vec::new();
                    current_text.clear();
                }
            } else if line.starts_with('#') {
                // Comment line, might contain sentence text
                if let Some(stripped) = line.strip_prefix("# text = ") {
                    current_text = stripped.to_string();
                }
            } else {
                // Word line: ID FORM LEMMA UPOS XPOS FEATS HEAD DEPREL DEPS MISC
                let fields: Vec<&str> = line.split('\t').collect();
                if fields.len() < 8 {
                    return Err(EvaluationError::InvalidAnnotationFormat {
                        line: line_no + 1,
                        reason: format!("Expected at least 8 fields, got {}", fields.len()),
                    });
                }

                // Skip multiword tokens (ID contains '-')
                if fields[0].contains('-') {
                    continue;
                }

                let form = fields[1].to_string();
                let lemma = fields[2].to_string();

                // Parse UPOS
                let upos = match fields[3] {
                    "NOUN" => UPos::Noun,
                    "VERB" => UPos::Verb,
                    "ADJ" => UPos::Adj,
                    "ADV" => UPos::Adv,
                    "PRON" => UPos::Pron,
                    "DET" => UPos::Det,
                    "ADP" => UPos::Adp,
                    "NUM" => UPos::Num,
                    "CONJ" => UPos::Cconj, // Map old CONJ to CCONJ
                    "CCONJ" => UPos::Cconj,
                    "SCONJ" => UPos::Sconj,
                    "PRT" => UPos::Part, // Map old PRT to PART
                    "PART" => UPos::Part,
                    "INTJ" => UPos::Intj,
                    "PUNCT" => UPos::Punct,
                    "SYM" => UPos::Sym,
                    "X" => UPos::X,
                    _ => UPos::X, // Default for unknown tags
                };

                // Parse head
                let head = fields[6].parse::<usize>().map_err(|_| {
                    EvaluationError::InvalidAnnotationFormat {
                        line: line_no + 1,
                        reason: format!("Invalid head index: {}", fields[6]),
                    }
                })?;

                // Parse DEPREL
                let deprel = match fields[7] {
                    "root" => DepRel::Root,
                    "nsubj" => DepRel::Nsubj,
                    "obj" => DepRel::Obj,
                    "iobj" => DepRel::Iobj,
                    "nmod" => DepRel::Nmod,
                    "amod" => DepRel::Amod,
                    "det" => DepRel::Det,
                    "case" => DepRel::Case,
                    "cc" => DepRel::Cc,
                    "conj" => DepRel::Conj,
                    "punct" => DepRel::Punct,
                    "aux" => DepRel::Aux,
                    "cop" => DepRel::Cop,
                    "mark" => DepRel::Mark,
                    _ => DepRel::Dep, // Default for unknown relations
                };

                current_words.push(GoldStandardWord {
                    form,
                    lemma,
                    upos,
                    deprel,
                    head,
                    start: 0, // TODO: Calculate actual character positions
                    end: 0,
                });
            }
        }

        // Handle last sentence if file doesn't end with blank line
        if !current_words.is_empty() {
            sentences.push(GoldStandardSentence {
                text: current_text,
                words: current_words,
            });
        }

        Ok(sentences)
    }

    /// Convert Universal POS tag from CoNLL-U format to our UPos enum
    /// Handles all 17 Universal Dependencies POS tags
    #[allow(dead_code)]
    fn convert_upos_from_conllu(upos_str: &str) -> UPos {
        match upos_str {
            "ADJ" => UPos::Adj,     // adjective
            "ADP" => UPos::Adp,     // adposition
            "ADV" => UPos::Adv,     // adverb
            "AUX" => UPos::Aux,     // auxiliary
            "CCONJ" => UPos::Cconj, // coordinating conjunction
            "DET" => UPos::Det,     // determiner
            "INTJ" => UPos::Intj,   // interjection
            "NOUN" => UPos::Noun,   // noun
            "NUM" => UPos::Num,     // numeral
            "PART" => UPos::Part,   // particle
            "PRON" => UPos::Pron,   // pronoun
            "PROPN" => UPos::Propn, // proper noun
            "PUNCT" => UPos::Punct, // punctuation
            "SCONJ" => UPos::Sconj, // subordinating conjunction
            "SYM" => UPos::Sym,     // symbol
            "VERB" => UPos::Verb,   // verb
            "X" => UPos::X,         // other
            _ => UPos::X,           // fallback for unknown tags
        }
    }

    /// Convert dependency relation from CoNLL-U format to our DepRel enum
    /// Handles all Universal Dependencies relations with subtypes
    fn convert_deprel_from_conllu(deprel_str: &str) -> DepRel {
        // Handle subtypes by taking the main type (before colon)
        let main_rel = deprel_str.split(':').next().unwrap_or(deprel_str);

        match main_rel {
            // Core relations
            "root" => DepRel::Root,
            "nsubj" => DepRel::Nsubj,
            "obj" => DepRel::Obj,
            "iobj" => DepRel::Iobj,

            // Nominal modifiers
            "nmod" => DepRel::Nmod,
            "amod" => DepRel::Amod,
            "det" => DepRel::Det,
            "nummod" => DepRel::Nummod,
            "appos" => DepRel::Appos,

            // Clausal relations
            "acl" => DepRel::Acl,
            "advcl" => DepRel::Advcl,
            "ccomp" => DepRel::Ccomp,
            "xcomp" => DepRel::Xcomp,
            "csubj" => DepRel::Csubj,

            // Function words
            "aux" => DepRel::Aux,
            "cop" => DepRel::Cop,
            "case" => DepRel::Case,
            "mark" => DepRel::Mark,
            "cc" => DepRel::Cc,

            // Other modifiers
            "advmod" => DepRel::Advmod,
            "neg" => DepRel::Neg,
            "compound" => DepRel::Compound,
            "flat" => DepRel::Flat,
            "fixed" => DepRel::Fixed,

            // Coordination
            "conj" => DepRel::Conj,

            // Oblique
            "obl" => DepRel::Obl,

            // Discourse and special
            "discourse" => DepRel::Discourse,
            "vocative" => DepRel::Vocative,
            "expl" => DepRel::Expl,
            "dislocated" => DepRel::Dislocated,

            // Punctuation and lists
            "punct" => DepRel::Punct,
            "list" => DepRel::List,
            "parataxis" => DepRel::Parataxis,
            "orphan" => DepRel::Orphan,
            "goeswith" => DepRel::Goeswith,
            "reparandum" => DepRel::Reparandum,

            // Classifiers
            "clf" => DepRel::Clf,

            // Handle subtypes explicitly for important ones
            "nsubj:pass" => DepRel::NsubjPass,
            "csubj:pass" => DepRel::CsubjPass,
            "aux:pass" => DepRel::AuxPass,

            // Fallback
            "dep" => DepRel::Dep,
            _ => DepRel::Other(deprel_str.to_string()),
        }
    }

    /// Create synthetic gold standard for testing
    pub fn create_synthetic_gold_standard() -> Vec<GoldStandardSentence> {
        vec![
            GoldStandardSentence {
                text: "The cat sat.".to_string(),
                words: vec![
                    GoldStandardWord {
                        form: "The".to_string(),
                        lemma: "the".to_string(),
                        upos: UPos::Det,
                        deprel: DepRel::Det,
                        head: 2,
                        start: 0,
                        end: 3,
                    },
                    GoldStandardWord {
                        form: "cat".to_string(),
                        lemma: "cat".to_string(),
                        upos: UPos::Noun,
                        deprel: DepRel::Nsubj,
                        head: 3,
                        start: 4,
                        end: 7,
                    },
                    GoldStandardWord {
                        form: "sat".to_string(),
                        lemma: "sit".to_string(),
                        upos: UPos::Verb,
                        deprel: DepRel::Root,
                        head: 0,
                        start: 8,
                        end: 11,
                    },
                ],
            },
            GoldStandardSentence {
                text: "She gave him a book.".to_string(),
                words: vec![
                    GoldStandardWord {
                        form: "She".to_string(),
                        lemma: "she".to_string(),
                        upos: UPos::Pron,
                        deprel: DepRel::Nsubj,
                        head: 2,
                        start: 0,
                        end: 3,
                    },
                    GoldStandardWord {
                        form: "gave".to_string(),
                        lemma: "give".to_string(),
                        upos: UPos::Verb,
                        deprel: DepRel::Root,
                        head: 0,
                        start: 4,
                        end: 8,
                    },
                    GoldStandardWord {
                        form: "him".to_string(),
                        lemma: "he".to_string(),
                        upos: UPos::Pron,
                        deprel: DepRel::Iobj,
                        head: 2,
                        start: 9,
                        end: 12,
                    },
                    GoldStandardWord {
                        form: "a".to_string(),
                        lemma: "a".to_string(),
                        upos: UPos::Det,
                        deprel: DepRel::Det,
                        head: 5,
                        start: 13,
                        end: 14,
                    },
                    GoldStandardWord {
                        form: "book".to_string(),
                        lemma: "book".to_string(),
                        upos: UPos::Noun,
                        deprel: DepRel::Obj,
                        head: 2,
                        start: 15,
                        end: 19,
                    },
                ],
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::udpipe::UDPipeEngine;

    #[test]
    fn test_synthetic_gold_standard() {
        let gold_sentences = CorpusEvaluator::create_synthetic_gold_standard();
        assert_eq!(gold_sentences.len(), 2);
        assert_eq!(gold_sentences[0].text, "The cat sat.");
        assert_eq!(gold_sentences[0].words.len(), 3);
        assert_eq!(gold_sentences[1].words.len(), 5);
    }

    #[test]
    fn test_accuracy_evaluation() {
        let test_engine = UDPipeEngine::for_testing();
        let parser = UDPipeParser::new_with_engine(test_engine);
        let evaluator = CorpusEvaluator::new(parser);

        let gold_sentences = CorpusEvaluator::create_synthetic_gold_standard();
        let metrics = evaluator
            .evaluate(&gold_sentences)
            .expect("Evaluation should succeed");

        assert!(metrics.total_tokens > 0);
        assert!(metrics.processing_time_ms >= 0.0);
        assert!(metrics.upos_accuracy >= 0.0 && metrics.upos_accuracy <= 1.0);
        assert!(metrics.lemma_accuracy >= 0.0 && metrics.lemma_accuracy <= 1.0);
        assert!(metrics.uas >= 0.0 && metrics.uas <= 1.0);
        assert!(metrics.las >= 0.0 && metrics.las <= 1.0);
    }

    #[test]
    fn test_conllu_parsing() {
        let conllu_content = r#"# text = The cat sat.
1	The	the	DET	DT	_	2	det	_	_
2	cat	cat	NOUN	NN	_	3	nsubj	_	_
3	sat	sit	VERB	VBD	_	0	root	_	SpaceAfter=No
4	.	.	PUNCT	.	_	3	punct	_	_

# text = She loves reading.
1	She	she	PRON	PRP	_	2	nsubj	_	_
2	loves	love	VERB	VBZ	_	0	root	_	_
3	reading	read	VERB	VBG	_	2	obj	_	SpaceAfter=No
4	.	.	PUNCT	.	_	2	punct	_	_

"#;

        let sentences =
            CorpusEvaluator::parse_conllu(conllu_content).expect("Should parse CoNLL-U");
        assert_eq!(sentences.len(), 2);
        assert_eq!(sentences[0].text, "The cat sat.");
        assert_eq!(sentences[0].words.len(), 4); // Including punctuation
        assert_eq!(sentences[1].text, "She loves reading.");
        assert_eq!(sentences[1].words.len(), 4);
    }

    #[test]
    fn test_per_tag_accuracy() {
        let test_engine = UDPipeEngine::for_testing();
        let parser = UDPipeParser::new_with_engine(test_engine);
        let evaluator = CorpusEvaluator::new(parser);

        let gold_sentences = CorpusEvaluator::create_synthetic_gold_standard();
        let metrics = evaluator
            .evaluate(&gold_sentences)
            .expect("Evaluation should succeed");

        // Should have per-tag accuracy for at least some tags
        assert!(!metrics.per_tag_accuracy.is_empty());

        // All accuracy values should be valid percentages
        for (tag, accuracy) in &metrics.per_tag_accuracy {
            assert!(
                *accuracy >= 0.0 && *accuracy <= 1.0,
                "Tag {tag} has invalid accuracy: {accuracy}"
            );
        }
    }
}
