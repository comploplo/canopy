//! CoNLL-U parser utilities for universal format parsing
//!
//! This module provides a reusable CoNLL-U parser that can be used by multiple engines
//! that need to parse Universal Dependencies formatted data.

use crate::{EngineError, EngineResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use tracing::{debug, info};

/// A parsed CoNLL-U sentence with metadata and tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConlluSentence {
    /// Sentence ID from treebank
    pub sent_id: String,
    /// Original text
    pub text: String,
    /// Parsed tokens with dependencies
    pub tokens: Vec<ConlluToken>,
    /// Additional metadata from comments
    pub metadata: HashMap<String, String>,
}

impl ConlluSentence {
    /// Create a new empty sentence
    pub fn new(sent_id: String, text: String) -> Self {
        Self {
            sent_id,
            text,
            tokens: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add a token to this sentence
    pub fn add_token(&mut self, token: ConlluToken) {
        self.tokens.push(token);
    }

    /// Get all tokens with a specific POS tag
    pub fn get_tokens_by_pos(&self, pos: &str) -> Vec<&ConlluToken> {
        self.tokens.iter().filter(|t| t.upos == pos).collect()
    }

    /// Find the root token (head = 0)
    pub fn get_root_token(&self) -> Option<&ConlluToken> {
        self.tokens.iter().find(|t| t.head == 0)
    }

    /// Get all tokens that are children of the given token
    pub fn get_children(&self, parent_id: u32) -> Vec<&ConlluToken> {
        self.tokens.iter().filter(|t| t.head == parent_id).collect()
    }

    /// Check if sentence has any verb tokens
    pub fn has_verb(&self) -> bool {
        self.tokens
            .iter()
            .any(|t| matches!(t.upos.as_str(), "VERB" | "AUX"))
    }
}

/// A parsed CoNLL-U token with all standard fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConlluToken {
    /// Token ID (1-based)
    pub id: u32,
    /// Surface form
    pub form: String,
    /// Lemma
    pub lemma: String,
    /// Universal POS tag
    pub upos: String,
    /// Language-specific POS tag
    pub xpos: Option<String>,
    /// Morphological features
    pub features: HashMap<String, String>,
    /// Head token ID (0 for root)
    pub head: u32,
    /// Dependency relation
    pub deprel: String,
    /// Additional dependencies (enhanced representation)
    pub deps: Vec<String>,
    /// Miscellaneous attributes
    pub misc: HashMap<String, String>,
}

impl ConlluToken {
    /// Check if token is a verb
    pub fn is_verb(&self) -> bool {
        matches!(self.upos.as_str(), "VERB" | "AUX")
    }

    /// Check if token is root
    pub fn is_root(&self) -> bool {
        self.head == 0
    }

    /// Check if token is punctuation
    pub fn is_punct(&self) -> bool {
        self.upos == "PUNCT"
    }

    /// Get morphological feature value
    pub fn get_feature(&self, key: &str) -> Option<&String> {
        self.features.get(key)
    }

    /// Check if token has a specific morphological feature
    pub fn has_feature(&self, key: &str, value: &str) -> bool {
        self.features.get(key).is_some_and(|v| v == value)
    }
}

/// Configuration for CoNLL-U parsing
#[derive(Debug, Clone)]
pub struct ConlluParserConfig {
    /// Enable verbose logging
    pub verbose: bool,
    /// Skip multi-word tokens (ranges like "1-2")
    pub skip_multiword_tokens: bool,
    /// Skip empty nodes (like "1.1")
    pub skip_empty_nodes: bool,
    /// Maximum number of sentences to parse (for development/testing)
    pub max_sentences: Option<usize>,
}

impl Default for ConlluParserConfig {
    fn default() -> Self {
        Self {
            verbose: false,
            skip_multiword_tokens: true,
            skip_empty_nodes: true,
            max_sentences: None,
        }
    }
}

/// CoNLL-U parser for Universal Dependencies format
pub struct ConlluParser {
    config: ConlluParserConfig,
}

impl ConlluParser {
    /// Create a new CoNLL-U parser with default configuration
    pub fn new() -> Self {
        Self {
            config: ConlluParserConfig::default(),
        }
    }

    /// Create a new CoNLL-U parser with custom configuration
    pub fn with_config(config: ConlluParserConfig) -> Self {
        Self { config }
    }

    /// Parse a CoNLL-U file and return sentences
    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> EngineResult<Vec<ConlluSentence>> {
        let path = path.as_ref();
        info!("Parsing CoNLL-U file: {}", path.display());

        let file =
            File::open(path).map_err(|e| EngineError::io(format!("open {}", path.display()), e))?;

        let reader = BufReader::new(file);
        let mut sentences = Vec::new();

        // Parse CoNLL-U format manually
        let mut current_comments = Vec::new();
        let mut current_tokens = Vec::new();
        let mut sentences_parsed = 0;

        for line_result in reader.lines() {
            let line =
                line_result.map_err(|e| EngineError::io(format!("read {}", path.display()), e))?;

            let line = line.trim();

            if line.is_empty() {
                // End of sentence
                if !current_tokens.is_empty() {
                    if let Some(sentence) =
                        self.parse_sentence_from_parts(&current_comments, &current_tokens)?
                    {
                        sentences.push(sentence);
                        sentences_parsed += 1;

                        // Check max sentences limit
                        if let Some(max) = self.config.max_sentences {
                            if sentences_parsed >= max {
                                break;
                            }
                        }
                    }
                    current_comments.clear();
                    current_tokens.clear();
                }
            } else if line.starts_with('#') {
                // Comment line
                current_comments.push(line.to_string());
            } else {
                // Token line
                current_tokens.push(line.to_string());
            }
        }

        // Handle final sentence if file doesn't end with empty line
        if !current_tokens.is_empty() {
            // Check if we've already reached max_sentences limit
            let should_add = if let Some(max) = self.config.max_sentences {
                sentences_parsed < max
            } else {
                true
            };

            if should_add {
                if let Some(sentence) =
                    self.parse_sentence_from_parts(&current_comments, &current_tokens)?
                {
                    sentences.push(sentence);
                }
            }
        }

        if self.config.verbose {
            info!(
                "Parsed {} sentences from {}",
                sentences.len(),
                path.display()
            );
        }

        Ok(sentences)
    }

    /// Parse a sentence from comment and token lines
    fn parse_sentence_from_parts(
        &self,
        comments: &[String],
        token_lines: &[String],
    ) -> EngineResult<Option<ConlluSentence>> {
        // Extract metadata from comments
        let mut metadata = HashMap::new();
        let mut sent_id = None;
        let mut text = None;

        for comment in comments {
            if let Some(content) = comment.strip_prefix("# ") {
                if let Some((key, value)) = content.split_once(" = ") {
                    metadata.insert(key.to_string(), value.to_string());
                    match key {
                        "sent_id" => sent_id = Some(value.to_string()),
                        "text" => text = Some(value.to_string()),
                        _ => {}
                    }
                }
            }
        }

        // Use fallback values if not found in comments
        let sent_id = sent_id.unwrap_or_else(|| format!("sent-{:04}", token_lines.len()));
        let text = text.unwrap_or_else(|| "".to_string());

        // Skip sentences without text
        if text.is_empty() {
            if self.config.verbose {
                debug!("Skipping sentence {} - no text", sent_id);
            }
            return Ok(None);
        }

        let mut sentence = ConlluSentence::new(sent_id, text);
        sentence.metadata = metadata;

        for token_line in token_lines {
            match self.parse_token_line(token_line) {
                Ok(token) => sentence.add_token(token),
                Err(e) => {
                    // Skip problematic tokens (like multi-word tokens) but log them
                    if self.config.verbose {
                        debug!("Skipping token in sentence {}: {}", sentence.sent_id, e);
                    }
                    continue;
                }
            }
        }

        // Skip sentences without tokens
        if sentence.tokens.is_empty() {
            if self.config.verbose {
                debug!("Skipping sentence {} - no tokens", sentence.sent_id);
            }
            return Ok(None);
        }

        Ok(Some(sentence))
    }

    /// Parse a single CoNLL-U token line
    fn parse_token_line(&self, line: &str) -> EngineResult<ConlluToken> {
        let fields: Vec<&str> = line.split('\t').collect();

        // CoNLL-U format has 10 fields:
        // ID FORM LEMMA UPOS XPOS FEATS HEAD DEPREL DEPS MISC
        if fields.len() != 10 {
            return Err(EngineError::data_load(format!(
                "Invalid CoNLL-U token line, expected 10 fields, got {}: {}",
                fields.len(),
                line
            )));
        }

        // Parse ID (handle ranges like "1-2" and decimals like "1.1")
        let id_str = fields[0];

        // Skip multi-word tokens (ranges like "1-2")
        if self.config.skip_multiword_tokens && id_str.contains('-') {
            return Err(EngineError::data_load(format!(
                "Skipping multi-word token: {id_str}"
            )));
        }

        // Skip empty nodes (like "1.1")
        if self.config.skip_empty_nodes && id_str.contains('.') {
            return Err(EngineError::data_load(format!(
                "Skipping empty node: {id_str}"
            )));
        }

        let id = if id_str.contains('.') {
            // For empty nodes like "1.1", take the integer part
            id_str.split('.').next().unwrap_or("1").parse::<u32>()
        } else {
            id_str.parse::<u32>()
        }
        .map_err(|_| EngineError::data_load(format!("Invalid token ID: {id_str}")))?;

        let form = fields[1].to_string();
        let lemma = if fields[2] == "_" {
            form.clone()
        } else {
            fields[2].to_string()
        };
        let upos = fields[3].to_string();
        let xpos = if fields[4] == "_" {
            None
        } else {
            Some(fields[4].to_string())
        };

        // Parse morphological features
        let mut features = HashMap::new();
        if fields[5] != "_" {
            for feature in fields[5].split('|') {
                if let Some((key, value)) = feature.split_once('=') {
                    features.insert(key.to_string(), value.to_string());
                }
            }
        }

        // Extract head and dependency relation
        let head = if fields[6] == "_" {
            0 // Use 0 for tokens without a head
        } else {
            fields[6]
                .parse::<u32>()
                .map_err(|_| EngineError::data_load(format!("Invalid head: {}", fields[6])))?
        };
        let deprel = fields[7].to_string();

        // Parse additional dependencies
        let mut deps = Vec::new();
        if fields[8] != "_" {
            for dep in fields[8].split('|') {
                deps.push(dep.to_string());
            }
        }

        // Parse miscellaneous attributes
        let mut misc = HashMap::new();
        if fields[9] != "_" {
            for attr in fields[9].split('|') {
                if let Some((key, value)) = attr.split_once('=') {
                    misc.insert(key.to_string(), value.to_string());
                } else {
                    // Some attributes might not have values
                    misc.insert(attr.to_string(), "true".to_string());
                }
            }
        }

        Ok(ConlluToken {
            id,
            form,
            lemma,
            upos,
            xpos,
            features,
            head,
            deprel,
            deps,
            misc,
        })
    }
}

impl Default for ConlluParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_conllu_parser_creation() {
        let parser = ConlluParser::new();
        assert!(!parser.config.verbose);
        assert!(parser.config.skip_multiword_tokens);
        assert!(parser.config.skip_empty_nodes);

        let config = ConlluParserConfig {
            verbose: true,
            skip_multiword_tokens: false,
            skip_empty_nodes: false,
            max_sentences: Some(100),
        };
        let parser = ConlluParser::with_config(config);
        assert!(parser.config.verbose);
        assert!(!parser.config.skip_multiword_tokens);
        assert!(!parser.config.skip_empty_nodes);
        assert_eq!(parser.config.max_sentences, Some(100));
    }

    #[test]
    fn test_parse_empty_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file).unwrap();

        let parser = ConlluParser::new();
        let result = parser.parse_file(temp_file.path()).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_simple_sentence() {
        let conllu_data = r#"# sent_id = test-001
# text = John runs.
1	John	John	PROPN	NNP	Number=Sing	2	nsubj	2:nsubj	_
2	runs	run	VERB	VBZ	Mood=Ind|Number=Sing|Person=3|Tense=Pres|VerbForm=Fin	0	root	0:root	SpaceAfter=No
3	.	.	PUNCT	.	_	2	punct	2:punct	_

"#;
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{conllu_data}").unwrap();

        let parser = ConlluParser::new();
        let result = parser.parse_file(temp_file.path()).unwrap();

        assert_eq!(result.len(), 1);

        let sentence = &result[0];
        assert_eq!(sentence.sent_id, "test-001");
        assert_eq!(sentence.text, "John runs.");
        assert_eq!(sentence.tokens.len(), 3);

        // Check root verb identification
        let root = sentence.get_root_token().unwrap();
        assert_eq!(root.lemma, "run");
        assert!(root.is_verb());
        assert!(root.is_root());
    }

    #[test]
    fn test_max_sentences_limit() {
        let conllu_data = r#"# sent_id = test-001
# text = First sentence.
1	First	first	ADJ	JJ	_	2	amod	2:amod	_
2	sentence	sentence	NOUN	NN	_	0	root	0:root	SpaceAfter=No
3	.	.	PUNCT	.	_	2	punct	2:punct	_

# sent_id = test-002
# text = Second sentence.
1	Second	second	ADJ	JJ	_	2	amod	2:amod	_
2	sentence	sentence	NOUN	NN	_	0	root	0:root	SpaceAfter=No
3	.	.	PUNCT	.	_	2	punct	2:punct	_

"#;
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{conllu_data}").unwrap();

        let config = ConlluParserConfig {
            max_sentences: Some(1),
            ..Default::default()
        };
        let parser = ConlluParser::with_config(config);
        let result = parser.parse_file(temp_file.path()).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].sent_id, "test-001");
    }

    #[test]
    fn test_token_utility_methods() {
        let token = ConlluToken {
            id: 2,
            form: "runs".to_string(),
            lemma: "run".to_string(),
            upos: "VERB".to_string(),
            xpos: Some("VBZ".to_string()),
            features: {
                let mut features = HashMap::new();
                features.insert("Tense".to_string(), "Pres".to_string());
                features.insert("Number".to_string(), "Sing".to_string());
                features
            },
            head: 0,
            deprel: "root".to_string(),
            deps: vec!["0:root".to_string()],
            misc: HashMap::new(),
        };

        assert!(token.is_verb());
        assert!(token.is_root());
        assert!(!token.is_punct());
        assert_eq!(token.get_feature("Tense"), Some(&"Pres".to_string()));
        assert!(token.has_feature("Number", "Sing"));
        assert!(!token.has_feature("Number", "Plur"));
    }

    #[test]
    fn test_sentence_utility_methods() {
        let sentence = ConlluSentence {
            sent_id: "test-001".to_string(),
            text: "John runs quickly.".to_string(),
            tokens: vec![
                ConlluToken {
                    id: 1,
                    form: "John".to_string(),
                    lemma: "John".to_string(),
                    upos: "PROPN".to_string(),
                    xpos: None,
                    features: HashMap::new(),
                    head: 2,
                    deprel: "nsubj".to_string(),
                    deps: vec![],
                    misc: HashMap::new(),
                },
                ConlluToken {
                    id: 2,
                    form: "runs".to_string(),
                    lemma: "run".to_string(),
                    upos: "VERB".to_string(),
                    xpos: None,
                    features: HashMap::new(),
                    head: 0,
                    deprel: "root".to_string(),
                    deps: vec![],
                    misc: HashMap::new(),
                },
                ConlluToken {
                    id: 3,
                    form: "quickly".to_string(),
                    lemma: "quickly".to_string(),
                    upos: "ADV".to_string(),
                    xpos: None,
                    features: HashMap::new(),
                    head: 2,
                    deprel: "advmod".to_string(),
                    deps: vec![],
                    misc: HashMap::new(),
                },
            ],
            metadata: HashMap::new(),
        };

        assert!(sentence.has_verb());

        let verbs = sentence.get_tokens_by_pos("VERB");
        assert_eq!(verbs.len(), 1);
        assert_eq!(verbs[0].lemma, "run");

        let root = sentence.get_root_token().unwrap();
        assert_eq!(root.lemma, "run");

        let children = sentence.get_children(2);
        assert_eq!(children.len(), 2); // "John" and "quickly"
        assert!(children.iter().any(|t| t.lemma == "John"));
        assert!(children.iter().any(|t| t.lemma == "quickly"));
    }
}
