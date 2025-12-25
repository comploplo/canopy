//! PropBank data parser implementation
//!
//! This module handles parsing of PropBank data files in both .prop and .gold_skel formats.
//! It uses the common CoNLL-U parser from canopy-engine for structured format parsing.

use crate::config::PropBankConfig;
use crate::types::{
    ArgumentModifier, PropBankArgument, PropBankFrameset, PropBankPredicate, SemanticRole,
};
use canopy_engine::{ConlluParser, ConlluSentence, EngineError, EngineResult};
use indexmap::IndexMap;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tracing::{debug, info, warn};

/// PropBank data loader and parser
pub struct PropBankParser {
    config: PropBankConfig,
    conllu_parser: ConlluParser,
    prop_file_regex: Regex,
    _frameset_cache: HashMap<String, PropBankFrameset>,
}

impl PropBankParser {
    /// Create a new PropBank parser
    pub fn new(config: PropBankConfig) -> EngineResult<Self> {
        let conllu_parser = ConlluParser::new();

        // Regex for parsing .prop file predicates (e.g., "give.01")
        let prop_file_regex = Regex::new(r"(\w+)\.(\d+)")
            .map_err(|e| EngineError::data_load(format!("Failed to compile regex: {e}")))?;

        Ok(Self {
            config,
            conllu_parser,
            prop_file_regex,
            _frameset_cache: HashMap::new(),
        })
    }

    /// Load PropBank data from configured data sources
    pub fn load_data(&mut self) -> EngineResult<PropBankData> {
        let mut data = PropBankData::new();

        let data_sources = self.config.data_sources.clone();
        for source in &data_sources {
            let source_path = self.config.get_data_source_path(source);
            info!("Loading PropBank data from: {}", source_path.display());

            if self.config.enable_prop_files {
                self.load_prop_files(&source_path, &mut data)?;
            }

            if self.config.enable_gold_skel_files {
                self.load_gold_skel_files(&source_path, &mut data)?;
            }
        }

        info!(
            "Loaded {} framesets with {} total predicates",
            data.framesets.len(),
            data.predicates.len()
        );

        Ok(data)
    }

    /// Load .prop files (structured PropBank annotations)
    fn load_prop_files(&mut self, source_path: &Path, data: &mut PropBankData) -> EngineResult<()> {
        let prop_files = self.find_files_with_extension(source_path, "prop")?;

        for (i, prop_file) in prop_files.iter().enumerate() {
            if let Some(max_files) = self.config.max_files_to_process {
                if i >= max_files {
                    break;
                }
            }

            if self.config.verbose {
                debug!("Processing .prop file: {}", prop_file.display());
            }

            self.parse_prop_file(prop_file, data)?;
        }

        Ok(())
    }

    /// Load .gold_skel files (CoNLL-style format)
    fn load_gold_skel_files(
        &mut self,
        source_path: &Path,
        data: &mut PropBankData,
    ) -> EngineResult<()> {
        let gold_files = self.find_files_with_extension(source_path, "gold_skel")?;

        for (i, gold_file) in gold_files.iter().enumerate() {
            if let Some(max_files) = self.config.max_files_to_process {
                if i >= max_files {
                    break;
                }
            }

            if self.config.verbose {
                debug!("Processing .gold_skel file: {}", gold_file.display());
            }

            self.parse_gold_skel_file(gold_file, data)?;
        }

        Ok(())
    }

    /// Parse a .prop file containing structured PropBank annotations
    fn parse_prop_file(&mut self, file_path: &Path, data: &mut PropBankData) -> EngineResult<()> {
        let content = fs::read_to_string(file_path)?;

        for line in content.lines() {
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }

            if let Ok(predicate) = self.parse_prop_line(line) {
                let frameset_id = predicate.lemma.clone();

                // Add to frameset or create new one
                let frameset = data
                    .framesets
                    .entry(frameset_id.clone())
                    .or_insert_with(|| PropBankFrameset::new(frameset_id.clone(), String::new()));

                frameset.add_roleset(predicate.clone());
                data.predicates.insert(predicate.roleset.clone(), predicate);
            } else if self.config.verbose {
                warn!("Failed to parse .prop line: {}", line);
            }
        }

        Ok(())
    }

    /// Parse a single line from a .prop file
    fn parse_prop_line(&self, line: &str) -> EngineResult<PropBankPredicate> {
        // Simple .prop format: "give.01: ARG0:agent ARG1:theme ARG2:recipient"
        let parts: Vec<&str> = line.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(EngineError::data_load(format!(
                "Invalid .prop line format: {line}"
            )));
        }

        let roleset = parts[0].trim();
        let args_str = parts[1].trim();

        // Extract lemma and sense
        if let Some(captures) = self.prop_file_regex.captures(roleset) {
            let lemma = captures.get(1).unwrap().as_str().to_string();
            let sense = captures.get(2).unwrap().as_str().to_string();

            let mut predicate = PropBankPredicate::new(lemma, sense, String::new());

            // Parse arguments
            for arg_def in args_str.split_whitespace() {
                if let Some((role_str, desc)) = arg_def.split_once(':') {
                    let role = SemanticRole::from_propbank_label(role_str);
                    let argument = PropBankArgument::new(role, desc.to_string(), 0.9);
                    predicate.add_argument(argument);
                }
            }

            Ok(predicate)
        } else {
            Err(EngineError::data_load(format!(
                "Invalid roleset format: {roleset}"
            )))
        }
    }

    /// Parse a .gold_skel file using CoNLL-U format
    fn parse_gold_skel_file(
        &mut self,
        file_path: &Path,
        data: &mut PropBankData,
    ) -> EngineResult<()> {
        let sentences = self.conllu_parser.parse_file(file_path)?;

        for sentence in sentences {
            self.extract_predicate_from_sentence(&sentence, data)?;
        }

        Ok(())
    }

    /// Extract PropBank predicates from a CoNLL-U sentence
    fn extract_predicate_from_sentence(
        &self,
        sentence: &ConlluSentence,
        data: &mut PropBankData,
    ) -> EngineResult<()> {
        // Look for predicate markers in the MISC field or specific columns
        for (token_idx, token) in sentence.tokens.iter().enumerate() {
            // Check if this token is marked as a predicate
            if self.is_predicate_token(token) {
                let lemma = token.lemma.clone();

                // Try to infer sense from context or use default "01"
                let sense = self
                    .infer_predicate_sense(&token.lemma)
                    .unwrap_or_else(|| "01".to_string());

                let mut predicate = PropBankPredicate::new(lemma, sense, String::new());
                predicate.predicate_span = Some(token_idx);

                // Find arguments for this predicate
                self.extract_arguments_for_predicate(sentence, token_idx, &mut predicate)?;

                // Store predicate
                let frameset_id = predicate.lemma.clone();
                let frameset = data
                    .framesets
                    .entry(frameset_id.clone())
                    .or_insert_with(|| PropBankFrameset::new(frameset_id.clone(), String::new()));

                frameset.add_roleset(predicate.clone());
                data.predicates.insert(predicate.roleset.clone(), predicate);
            }
        }

        Ok(())
    }

    /// Check if a token represents a predicate
    fn is_predicate_token(&self, token: &canopy_engine::ConlluToken) -> bool {
        // In PropBank annotations, predicates are often marked in MISC field
        // or can be identified by POS tags (verbs) and specific annotations
        token.upos.starts_with('V') || // Verb POS tag
        token.misc.contains_key("PropBank") ||
        token.misc.contains_key("pred")
    }

    /// Infer predicate sense from lemma (basic heuristic)
    fn infer_predicate_sense(&self, lemma: &str) -> Option<String> {
        // This could be enhanced with a proper sense disambiguation model
        // For now, use "01" as default sense for most verbs
        match lemma {
            "be" | "have" | "do" => Some("01".to_string()),
            _ => None,
        }
    }

    /// Extract arguments for a given predicate from the sentence
    fn extract_arguments_for_predicate(
        &self,
        sentence: &ConlluSentence,
        predicate_idx: usize,
        predicate: &mut PropBankPredicate,
    ) -> EngineResult<()> {
        // Look for dependency relations that indicate PropBank arguments
        for (token_idx, token) in sentence.tokens.iter().enumerate() {
            if token_idx == predicate_idx {
                continue; // Skip the predicate itself
            }

            // Check if this token is an argument of our predicate
            if let Some(role) = self.infer_semantic_role_from_dependency(token, predicate_idx) {
                let argument = PropBankArgument::with_span(
                    role,
                    token.form.clone(),
                    (token_idx, token_idx + 1),
                    0.8, // Confidence based on dependency parsing
                );
                predicate.add_argument(argument);
            }
        }

        Ok(())
    }

    /// Infer semantic role from dependency relation to predicate
    fn infer_semantic_role_from_dependency(
        &self,
        token: &canopy_engine::ConlluToken,
        predicate_idx: usize,
    ) -> Option<SemanticRole> {
        // Check if this token depends on our predicate
        if token.head as usize == predicate_idx + 1 {
            // CoNLL-U uses 1-based indexing
            match token.deprel.as_str() {
                "nsubj" | "nsubj:pass" => Some(SemanticRole::Agent),
                "obj" | "dobj" => Some(SemanticRole::Patient),
                "iobj" => Some(SemanticRole::IndirectObject),
                "obl" => Some(SemanticRole::Modifier(ArgumentModifier::Location)),
                "advmod" => Some(SemanticRole::Modifier(ArgumentModifier::Manner)),
                "nmod:tmod" => Some(SemanticRole::Modifier(ArgumentModifier::Time)),
                _ => None,
            }
        } else {
            None
        }
    }

    /// Find files with a specific extension in a directory
    fn find_files_with_extension(
        &self,
        dir: &Path,
        extension: &str,
    ) -> EngineResult<Vec<std::path::PathBuf>> {
        let mut files = Vec::new();

        if !dir.exists() {
            return Err(EngineError::data_load(format!(
                "Directory does not exist: {}",
                dir.display()
            )));
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == extension {
                        files.push(path);
                    }
                }
            }
        }

        files.sort();
        Ok(files)
    }

    /// Get predicate by roleset (e.g., "give.01")
    pub fn get_predicate<'a>(
        &self,
        data: &'a PropBankData,
        roleset: &str,
    ) -> Option<&'a PropBankPredicate> {
        data.predicates.get(roleset)
    }

    /// Find predicates by lemma (returns all senses)
    pub fn find_predicates_by_lemma<'a>(
        &self,
        data: &'a PropBankData,
        lemma: &str,
    ) -> Vec<&'a PropBankPredicate> {
        data.predicates
            .values()
            .filter(|pred| pred.lemma == lemma)
            .collect()
    }

    /// Perform fuzzy matching for predicate lookup
    pub fn fuzzy_match_predicate<'a>(
        &self,
        data: &'a PropBankData,
        query: &str,
    ) -> Vec<&'a PropBankPredicate> {
        let query_lower = query.to_lowercase();
        let mut matches = Vec::new();

        for predicate in data.predicates.values() {
            let lemma_lower = predicate.lemma.to_lowercase();

            // Exact match gets highest priority
            if lemma_lower == query_lower {
                matches.insert(0, predicate);
            }
            // Prefix or contains match
            else if lemma_lower.starts_with(&query_lower)
                || query_lower.starts_with(&lemma_lower)
                || lemma_lower.contains(&query_lower)
                || query_lower.contains(&lemma_lower)
            {
                matches.push(predicate);
            }
        }

        matches
    }
}

/// Container for all loaded PropBank data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropBankData {
    /// All framesets indexed by lemma
    pub framesets: IndexMap<String, PropBankFrameset>,
    /// All predicates indexed by roleset (lemma.sense)
    pub predicates: IndexMap<String, PropBankPredicate>,
    /// Statistics about loaded data
    pub stats: PropBankStats,
}

impl PropBankData {
    /// Create new empty PropBank data container
    pub fn new() -> Self {
        Self {
            framesets: IndexMap::new(),
            predicates: IndexMap::new(),
            stats: PropBankStats::new(),
        }
    }

    /// Update statistics after loading
    pub fn update_stats(&mut self) {
        self.stats.total_framesets = self.framesets.len();
        self.stats.total_predicates = self.predicates.len();

        // Calculate average arguments per predicate
        if !self.predicates.is_empty() {
            let total_args: usize = self
                .predicates
                .values()
                .map(|pred| pred.arguments.len())
                .sum();
            self.stats.avg_arguments_per_predicate =
                total_args as f32 / self.predicates.len() as f32;
        }
    }
}

impl Default for PropBankData {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about loaded PropBank data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropBankStats {
    pub total_framesets: usize,
    pub total_predicates: usize,
    pub avg_arguments_per_predicate: f32,
    pub prop_files_processed: usize,
    pub gold_skel_files_processed: usize,
}

impl PropBankStats {
    pub fn new() -> Self {
        Self {
            total_framesets: 0,
            total_predicates: 0,
            avg_arguments_per_predicate: 0.0,
            prop_files_processed: 0,
            gold_skel_files_processed: 0,
        }
    }
}

impl Default for PropBankStats {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> PropBankConfig {
        PropBankConfig {
            data_path: std::path::PathBuf::from("test_data"),
            enable_prop_files: true,
            enable_gold_skel_files: false,
            max_files_to_process: Some(10),
            min_confidence: 0.1,
            enable_cache: false,
            cache_capacity: 100,
            enable_fuzzy_matching: true,
            include_modifiers: true,
            verbose: true,
            data_sources: vec!["test_source".to_string()],
        }
    }

    #[test]
    fn test_parser_creation() {
        let config = create_test_config();
        let parser = PropBankParser::new(config);
        assert!(parser.is_ok());
    }

    #[test]
    fn test_prop_line_parsing() {
        let config = create_test_config();
        let parser = PropBankParser::new(config).unwrap();

        let line = "give.01: ARG0:agent ARG1:theme ARG2:recipient";
        let predicate = parser.parse_prop_line(line).unwrap();

        assert_eq!(predicate.lemma, "give");
        assert_eq!(predicate.sense, "01");
        assert_eq!(predicate.arguments.len(), 3);

        let arg0 = &predicate.arguments[0];
        assert_eq!(arg0.role, SemanticRole::Agent);
        assert_eq!(arg0.description, "agent");
    }

    #[test]
    fn test_semantic_role_from_propbank_label() {
        assert_eq!(
            SemanticRole::from_propbank_label("ARG0"),
            SemanticRole::Agent
        );
        assert_eq!(
            SemanticRole::from_propbank_label("ARG1"),
            SemanticRole::Patient
        );
        assert_eq!(
            SemanticRole::from_propbank_label("ARGM-LOC"),
            SemanticRole::Modifier(ArgumentModifier::Location)
        );
    }

    #[test]
    fn test_propbank_data_container() {
        let mut data = PropBankData::new();

        let predicate = PropBankPredicate::new(
            "test".to_string(),
            "01".to_string(),
            "test definition".to_string(),
        );
        data.predicates.insert("test.01".to_string(), predicate);

        data.update_stats();
        assert_eq!(data.stats.total_predicates, 1);
    }
}
