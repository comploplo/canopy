//! CoNLL-U parser for Universal Dependencies treebank data
//!
//! This module provides parsing functionality for the CoNLL-U format used
//! by Universal Dependencies treebanks.

use crate::types::{DependencyFeatures, DependencyRelation};
use crate::TreebankResult;
use canopy_engine::EngineError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use tracing::{debug, info};

/// A parsed sentence with dependency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedSentence {
    /// Sentence ID from treebank
    pub sent_id: String,
    /// Original text
    pub text: String,
    /// Parsed tokens with dependencies
    pub tokens: Vec<ParsedToken>,
    /// Root verb lemma (if exists)
    pub root_verb: Option<String>,
}

impl ParsedSentence {
    /// Convert to ConlluSentence for advanced tree operations
    pub fn to_conllu_sentence(&self) -> crate::conllu_types::ConlluSentence {
        use crate::conllu_types::*;

        let tokens = self
            .tokens
            .iter()
            .map(|t| ConlluToken {
                id: t.id,
                form: t.form.clone(),
                lemma: t.lemma.clone(),
                upos: UniversalPos::from(t.upos.as_str()),
                xpos: t.xpos.clone(),
                features: MorphologicalFeatures::from_hashmap(&t.features),
                head: t.head,
                deprel: t.deprel.clone(),
                enhanced_deps: t
                    .deps
                    .iter()
                    .map(|(head, rel)| EnhancedDependency {
                        head: *head,
                        relation: rel.clone(),
                    })
                    .collect(),
                misc: MiscAttributes::default(),
                dependency_features: t.dependency_features.clone(),
            })
            .collect();

        ConlluSentence {
            sent_id: self.sent_id.clone(),
            newdoc_id: None,
            newpar_id: None,
            text: self.text.clone(),
            tokens,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Build complete dependency tree
    pub fn build_dependency_tree(&self) -> Option<crate::conllu_types::DependencyTree> {
        self.to_conllu_sentence().build_dependency_tree()
    }

    /// Create hierarchical pattern key
    pub fn create_hierarchical_pattern_key(&self) -> Option<String> {
        self.to_conllu_sentence().create_hierarchical_pattern_key()
    }

    /// Create flat pattern key
    pub fn create_pattern_key(&self) -> Option<String> {
        self.to_conllu_sentence().create_pattern_key()
    }
}

/// A parsed token with dependency information and extracted features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedToken {
    /// Token ID
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
    pub deprel: DependencyRelation,
    /// Extracted features from dependency relation subtypes
    pub dependency_features: DependencyFeatures,
    /// Additional dependencies
    pub deps: Vec<(u32, DependencyRelation)>,
}

impl ParsedToken {
    /// Check if token is a verb
    pub fn is_verb(&self) -> bool {
        matches!(self.upos.as_str(), "VERB" | "AUX")
    }

    /// Check if token is root
    pub fn is_root(&self) -> bool {
        self.head == 0
    }
}

/// CoNLL-U parser for treebank data
pub struct ConlluParser {
    /// Enable detailed logging
    verbose: bool,
}

impl ConlluParser {
    /// Create a new CoNLL-U parser
    pub fn new(verbose: bool) -> Self {
        Self { verbose }
    }

    /// Parse a CoNLL-U file and return sentences
    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> TreebankResult<Vec<ParsedSentence>> {
        let path = path.as_ref();
        info!("Parsing CoNLL-U file: {}", path.display());

        let file =
            File::open(path).map_err(|e| EngineError::io(format!("open {}", path.display()), e))?;

        let reader = BufReader::new(file);
        let mut parsed_sentences = Vec::new();

        // Parse CoNLL-U format manually
        let mut current_comments = Vec::new();
        let mut current_tokens = Vec::new();

        for line_result in reader.lines() {
            let line =
                line_result.map_err(|e| EngineError::io(format!("read {}", path.display()), e))?;

            let line = line.trim();

            if line.is_empty() {
                // End of sentence
                if !current_tokens.is_empty() {
                    if let Some(parsed) =
                        self.parse_sentence_from_parts(&current_comments, &current_tokens)?
                    {
                        parsed_sentences.push(parsed);
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
            if let Some(parsed) =
                self.parse_sentence_from_parts(&current_comments, &current_tokens)?
            {
                parsed_sentences.push(parsed);
            }
        }

        if self.verbose {
            info!(
                "Parsed {} sentences from {}",
                parsed_sentences.len(),
                path.display()
            );
        }

        info!(
            "Parsed {} sentences from {}",
            parsed_sentences.len(),
            path.display()
        );
        Ok(parsed_sentences)
    }

    /// Parse a sentence from comment and token lines
    fn parse_sentence_from_parts(
        &self,
        comments: &[String],
        token_lines: &[String],
    ) -> TreebankResult<Option<ParsedSentence>> {
        // Extract sentence ID from comments
        let sent_id = comments
            .iter()
            .find(|comment| comment.starts_with("# sent_id = "))
            .map(|comment| comment.trim_start_matches("# sent_id = ").to_string())
            .unwrap_or_else(|| format!("sent-{:04}", token_lines.len()));

        // Extract text from comments
        let text = comments
            .iter()
            .find(|comment| comment.starts_with("# text = "))
            .map(|comment| comment.trim_start_matches("# text = ").to_string())
            .unwrap_or_else(|| "".to_string());

        // Skip sentences without text
        if text.is_empty() {
            if self.verbose {
                debug!("Skipping sentence {} - no text", sent_id);
            }
            return Ok(None);
        }

        let mut tokens = Vec::new();
        let mut root_verb = None;

        for token_line in token_lines {
            match self.parse_token_line(token_line) {
                Ok(parsed_token) => {
                    // Track root verb
                    if parsed_token.is_root() && parsed_token.is_verb() {
                        root_verb = Some(parsed_token.lemma.clone());
                    }
                    tokens.push(parsed_token);
                }
                Err(e) => {
                    // Skip problematic tokens (like multi-word tokens) but log them
                    if self.verbose {
                        debug!("Skipping token in sentence {}: {}", sent_id, e);
                    }
                    continue;
                }
            }
        }

        // Skip sentences without tokens
        if tokens.is_empty() {
            if self.verbose {
                debug!("Skipping sentence {} - no tokens", sent_id);
            }
            return Ok(None);
        }

        Ok(Some(ParsedSentence {
            sent_id,
            text,
            tokens,
            root_verb,
        }))
    }

    /// Parse a single CoNLL-U token line
    fn parse_token_line(&self, line: &str) -> TreebankResult<ParsedToken> {
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
        if id_str.contains('-') {
            return Err(EngineError::data_load(format!(
                "Skipping multi-word token: {}",
                id_str
            )));
        }

        let id = if id_str.contains('.') {
            // For empty nodes like "1.1", take the integer part
            id_str.split('.').next().unwrap_or("1").parse::<u32>()
        } else {
            id_str.parse::<u32>()
        }
        .map_err(|_| EngineError::data_load(format!("Invalid token ID: {}", id_str)))?;

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
        let deprel_str = fields[7];
        let (deprel, dependency_features) = DependencyFeatures::parse_subtypes(deprel_str);

        // Parse additional dependencies
        let mut deps = Vec::new();
        if fields[8] != "_" {
            for dep in fields[8].split('|') {
                if let Some((head_str, rel_str)) = dep.split_once(':') {
                    if let Ok(dep_head) = head_str.parse::<u32>() {
                        let dep_rel = DependencyRelation::from(rel_str);
                        deps.push((dep_head, dep_rel));
                    }
                }
            }
        }

        Ok(ParsedToken {
            id,
            form,
            lemma,
            upos,
            xpos,
            features,
            head,
            deprel,
            dependency_features,
            deps,
        })
    }

    /// Extract dependency patterns from parsed sentences
    pub fn extract_patterns(
        &self,
        sentences: &[ParsedSentence],
    ) -> TreebankResult<HashMap<String, Vec<(DependencyRelation, String)>>> {
        let mut patterns = HashMap::new();

        for sentence in sentences {
            if let Some(root_verb) = &sentence.root_verb {
                let mut dependencies = Vec::new();

                for token in &sentence.tokens {
                    // Find dependencies of the root verb
                    if let Some(_head_token) = sentence
                        .tokens
                        .iter()
                        .find(|t| t.id == token.head && t.lemma == *root_verb)
                    {
                        // Skip punctuation and determiners
                        if matches!(token.upos.as_str(), "PUNCT" | "DET") {
                            continue;
                        }

                        let arg_type = match token.deprel {
                            DependencyRelation::NominalSubject => "agent",
                            DependencyRelation::Object => "patient",
                            DependencyRelation::IndirectObject => "recipient",
                            DependencyRelation::Oblique => "location",
                            _ => continue,
                        };

                        dependencies.push((token.deprel.clone(), arg_type.to_string()));
                    }
                }

                if !dependencies.is_empty() {
                    patterns
                        .entry(root_verb.clone())
                        .or_insert_with(Vec::new)
                        .extend(dependencies);
                }
            }
        }

        debug!("Extracted patterns for {} verbs", patterns.len());
        Ok(patterns)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use canopy_core::ThetaRole;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_conllu_parser_creation() {
        let parser = ConlluParser::new(false);
        assert!(!parser.verbose);

        let parser = ConlluParser::new(true);
        assert!(parser.verbose);
    }

    #[test]
    fn test_parse_empty_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file).unwrap();

        let parser = ConlluParser::new(false);
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
        write!(temp_file, "{}", conllu_data).unwrap();

        let parser = ConlluParser::new(false);
        let result = parser.parse_file(temp_file.path()).unwrap();

        // Real implementation should parse the sentence
        assert_eq!(result.len(), 1);

        let sentence = &result[0];
        assert_eq!(sentence.sent_id, "test-001");
        assert_eq!(sentence.text, "John runs.");
        assert_eq!(sentence.tokens.len(), 3);

        // Check that we identified the root verb
        assert_eq!(sentence.root_verb, Some("run".to_string()));
    }

    #[test]
    fn test_dependency_relation_conversion() {
        assert_eq!(
            DependencyRelation::from("nsubj"),
            DependencyRelation::NominalSubject
        );
        assert_eq!(DependencyRelation::from("obj"), DependencyRelation::Object);
        assert_eq!(
            DependencyRelation::from("unknown"),
            DependencyRelation::Other("unknown".to_string())
        );
    }

    #[test]
    fn test_dependency_to_theta_role() {
        assert_eq!(
            DependencyRelation::NominalSubject.to_theta_role(),
            Some(ThetaRole::Agent)
        );
        assert_eq!(
            DependencyRelation::Object.to_theta_role(),
            Some(ThetaRole::Patient)
        );
        assert_eq!(
            DependencyRelation::IndirectObject.to_theta_role(),
            Some(ThetaRole::Recipient)
        );
        assert_eq!(DependencyRelation::AdverbialModifier.to_theta_role(), None);
    }

    #[test]
    fn test_pattern_extraction() {
        let sentence = ParsedSentence {
            sent_id: "test-001".to_string(),
            text: "John gives Mary a book".to_string(),
            root_verb: Some("give".to_string()),
            tokens: vec![
                ParsedToken {
                    id: 1,
                    form: "John".to_string(),
                    lemma: "John".to_string(),
                    upos: "PROPN".to_string(),
                    xpos: None,
                    features: HashMap::new(),
                    head: 2,
                    deprel: DependencyRelation::NominalSubject,
                    dependency_features: DependencyFeatures::default(),
                    deps: vec![],
                },
                ParsedToken {
                    id: 2,
                    form: "gives".to_string(),
                    lemma: "give".to_string(),
                    upos: "VERB".to_string(),
                    xpos: None,
                    features: HashMap::new(),
                    head: 0,
                    deprel: DependencyRelation::Other("root".to_string()),
                    dependency_features: DependencyFeatures::default(),
                    deps: vec![],
                },
            ],
        };

        let parser = ConlluParser::new(false);
        let patterns = parser.extract_patterns(&[sentence]).unwrap();

        assert!(patterns.contains_key("give"));
        let give_patterns = &patterns["give"];
        assert!(give_patterns
            .iter()
            .any(|(rel, _)| matches!(rel, DependencyRelation::NominalSubject)));
    }

    #[test]
    fn test_dependency_feature_extraction() {
        use crate::types::{
            DependencyFeatureType, DependencyFeatures, SemanticRoleFeature, VoiceFeature,
        };

        // Test passive subject parsing
        let (relation, features) = DependencyFeatures::parse_subtypes("nsubj:pass");
        assert_eq!(relation, DependencyRelation::NominalSubject);
        assert!(features.is_passive());
        assert_eq!(features.features.len(), 1);
        assert!(matches!(
            features.features[0],
            DependencyFeatureType::Voice(VoiceFeature::Pass)
        ));

        // Test agent marker parsing
        let (relation, features) = DependencyFeatures::parse_subtypes("obl:agent");
        assert_eq!(relation, DependencyRelation::Oblique);
        assert!(features.is_agent());
        assert_eq!(features.features.len(), 1);
        assert!(matches!(
            features.features[0],
            DependencyFeatureType::SemanticRole(SemanticRoleFeature::Agent)
        ));

        // Test auxiliary passive parsing
        let (relation, features) = DependencyFeatures::parse_subtypes("aux:pass");
        assert_eq!(relation, DependencyRelation::Auxiliary);
        assert!(features.is_passive());

        // Test base relation without subtypes
        let (relation, features) = DependencyFeatures::parse_subtypes("obj");
        assert_eq!(relation, DependencyRelation::Object);
        assert!(!features.is_passive());
        assert!(!features.is_agent());
        assert!(features.features.is_empty());
    }

    #[test]
    fn test_parse_passive_sentence() {
        let conllu_data = r#"# sent_id = test-passive-001
# text = John was attacked by Mary.
1	John	John	PROPN	NNP	Number=Sing	3	nsubj:pass	3:nsubj:pass	_
2	was	be	AUX	VBD	Mood=Ind|Number=Sing|Person=3|Tense=Past|VerbForm=Fin	3	aux:pass	3:aux:pass	_
3	attacked	attack	VERB	VBN	Tense=Past|VerbForm=Part|Voice=Pass	0	root	0:root	_
4	by	by	ADP	IN	_	5	case	5:case	_
5	Mary	Mary	PROPN	NNP	Number=Sing	3	obl:agent	3:obl:agent	SpaceAfter=No
6	.	.	PUNCT	.	_	3	punct	3:punct	_

"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", conllu_data).unwrap();

        let parser = ConlluParser::new(false);
        let result = parser.parse_file(temp_file.path()).unwrap();

        assert_eq!(result.len(), 1);
        let sentence = &result[0];
        assert_eq!(sentence.sent_id, "test-passive-001");
        assert_eq!(sentence.text, "John was attacked by Mary.");

        // Find passive subject
        let passive_subj = sentence.tokens.iter().find(|t| t.form == "John").unwrap();
        assert!(passive_subj.dependency_features.is_passive());
        assert_eq!(passive_subj.deprel, DependencyRelation::NominalSubject);

        // Find passive auxiliary
        let passive_aux = sentence.tokens.iter().find(|t| t.form == "was").unwrap();
        assert!(passive_aux.dependency_features.is_passive());
        assert_eq!(passive_aux.deprel, DependencyRelation::Auxiliary);

        // Find agent
        let agent = sentence.tokens.iter().find(|t| t.form == "Mary").unwrap();
        assert!(agent.dependency_features.is_agent());
        assert_eq!(agent.deprel, DependencyRelation::Oblique);
    }

    #[test]
    #[ignore] // Only run when treebank data is available
    fn test_real_treebank_feature_extraction() {
        let dev_path =
            std::path::Path::new("data/ud_english-ewt/UD_English-EWT/en_ewt-ud-dev.conllu");

        if !dev_path.exists() {
            println!("Skipping real treebank test - data not available");
            return;
        }

        let parser = ConlluParser::new(false);
        let sentences = parser.parse_file(dev_path).unwrap();

        assert!(
            !sentences.is_empty(),
            "Should parse some sentences from treebank"
        );

        // Count feature occurrences
        let mut passive_count = 0;
        let mut agent_count = 0;
        let mut total_tokens = 0;

        for sentence in sentences.iter().take(100) {
            // Test first 100 sentences
            for token in &sentence.tokens {
                total_tokens += 1;
                if token.dependency_features.is_passive() {
                    passive_count += 1;
                }
                if token.dependency_features.is_agent() {
                    agent_count += 1;
                }
            }
        }

        println!("Real treebank feature extraction results:");
        println!("  Total tokens analyzed: {}", total_tokens);
        println!("  Passive markers found: {}", passive_count);
        println!("  Agent markers found: {}", agent_count);

        // Verify we found some features (passive constructions exist in English)
        assert!(total_tokens > 0, "Should have analyzed some tokens");
        // We expect some passive constructions in English treebank data
        // (though we don't assert exact counts since they vary)
    }
}
