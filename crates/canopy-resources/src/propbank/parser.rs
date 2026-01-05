//! `PropBank` data parser implementation
//!
//! This module handles parsing of `PropBank` data files in both .prop and .`gold_skel` formats.
//! It uses the common CoNLL-U parser from canopy-engine for structured format parsing.

use super::config::PropBankConfig;
use super::types::{
    ArgumentModifier, PropBankArgument, PropBankFrameset, PropBankPredicate, SemanticRole,
};
use crate::engine::{count_to_f32, ConlluParser, ConlluSentence, EngineError, EngineResult};
use indexmap::IndexMap;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tracing::{debug, info, warn};

/// `PropBank` data loader and parser
pub struct PropBankParser {
    config: PropBankConfig,
    conllu_parser: ConlluParser,
    prop_file_regex: Regex,
    _frameset_cache: HashMap<String, PropBankFrameset>,
}

impl PropBankParser {
    /// Create a new `PropBank` parser
    ///
    /// # Errors
    /// Returns an error if the parser cannot be initialized.
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

    /// Load `PropBank` data from configured data sources
    ///
    /// # Errors
    /// Returns an error if data files cannot be loaded or parsed.
    pub fn load_data(&mut self) -> EngineResult<PropBankData> {
        let mut data = PropBankData::new();

        let data_sources = self.config.data_sources.clone();
        for source in &data_sources {
            let source_path = self.config.get_data_source_path(source);
            info!("Loading PropBank data from: {}", source_path.display());

            if self.config.enable_prop_files() {
                self.load_prop_files(&source_path, &mut data)?;
            }

            if self.config.enable_gold_skel_files() {
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

    /// Load .prop files (structured `PropBank` annotations)
    fn load_prop_files(&mut self, source_path: &Path, data: &mut PropBankData) -> EngineResult<()> {
        let prop_files = Self::find_files_with_extension(source_path, "prop")?;

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

    /// Load .`gold_skel` files (CoNLL-style format)
    fn load_gold_skel_files(
        &mut self,
        source_path: &Path,
        data: &mut PropBankData,
    ) -> EngineResult<()> {
        let gold_files = Self::find_files_with_extension(source_path, "gold_skel")?;

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

    /// Parse a .prop file containing structured `PropBank` annotations
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

    /// Parse a single line from a .prop file (CoNLL-style `PropBank` format)
    ///
    /// Format: `file_path sent_idx token_idx tagger lemma roleset ----- arg1 arg2 ...`
    /// Example: `ewt/newsgroup/00/file.xml.parse 0 19 gold build build.01 ----- 14:1-ARG0 19:0-rel 20:1-ARG1`
    ///
    /// Each argument is in format `position:height-ROLE` where:
    /// - position: token position in sentence
    /// - height: tree height indicator
    /// - ROLE: semantic role (ARG0, ARG1, ARGM-LOC, rel, etc.)
    fn parse_prop_line(&self, line: &str) -> EngineResult<PropBankPredicate> {
        let fields: Vec<&str> = line.split_whitespace().collect();

        // Minimum fields: file sent_idx token_idx tagger lemma roleset separator
        if fields.len() < 7 {
            return Err(EngineError::data_load(format!(
                "Invalid .prop line format (too few fields): {line}"
            )));
        }

        // Fields: 0=file, 1=sent_idx, 2=token_idx, 3=tagger, 4=lemma, 5=roleset, 6=separator
        let lemma = fields[4].to_string();
        let roleset_full = fields[5]; // e.g., "build.01" or "shut_down.05"

        // Extract sense from roleset (e.g., "build.01" -> "01")
        let sense = if let Some(captures) = self.prop_file_regex.captures(roleset_full) {
            captures.get(2).unwrap().as_str().to_string()
        } else {
            // Handle multi-word predicates like "shut_down.05"
            if let Some(dot_pos) = roleset_full.rfind('.') {
                roleset_full[dot_pos + 1..].to_string()
            } else {
                "01".to_string() // Default sense
            }
        };

        // Get predicate token position
        let predicate_pos: usize = fields[2].parse().unwrap_or(0);

        let mut predicate = PropBankPredicate::new(lemma, sense, String::new());
        predicate.predicate_span = Some(predicate_pos);

        // Parse arguments (fields after the "-----" separator)
        // Find separator position
        let sep_pos = fields.iter().position(|&f| f == "-----");
        if let Some(sep_idx) = sep_pos {
            for arg_field in &fields[sep_idx + 1..] {
                if let Some(parsed_arg) = Self::parse_conll_argument(arg_field) {
                    predicate.add_argument(parsed_arg);
                }
            }
        }

        Ok(predicate)
    }

    /// Parse a CoNLL-style argument field
    ///
    /// Format: `position:height-ROLE` or `pos1:h1*pos2:h2-ROLE` for split arguments
    /// Examples: `14:1-ARG0`, `19:0-rel`, `0:1*11:1-ARGM-TMP`
    fn parse_conll_argument(arg_field: &str) -> Option<PropBankArgument> {
        // Split on last hyphen to separate position info from role
        // Handle roles like "ARGM-LOC" which contain hyphens
        let (positions_part, role_part) = Self::split_arg_field(arg_field)?;

        // Skip "rel" markers (they indicate the predicate itself)
        if role_part == "rel" {
            return None;
        }

        // Parse the role
        let role = SemanticRole::from_propbank_label(role_part);

        // Parse position(s) - may be split arguments like "0:1*11:1"
        let positions: Vec<usize> = positions_part
            .split('*')
            .filter_map(|pos_height| {
                pos_height
                    .split(':')
                    .next()
                    .and_then(|p| p.split(';').next()) // Handle "9:1;12:2" format
                    .and_then(|p| p.parse().ok())
            })
            .collect();

        if positions.is_empty() {
            return None;
        }

        let start_pos = *positions.iter().min().unwrap_or(&0);
        let end_pos = *positions.iter().max().unwrap_or(&0) + 1;

        Some(PropBankArgument::with_span(
            role,
            String::new(), // Description not available in this format
            (start_pos, end_pos),
            0.95, // High confidence for gold annotations
        ))
    }

    /// Split argument field into positions and role parts
    /// Handles complex role names like "ARGM-LOC", "ARGM-TMP", "LINK-SLC"
    fn split_arg_field(field: &str) -> Option<(&str, &str)> {
        // Known role prefixes that may contain hyphens
        let role_patterns = [
            "ARGM-", "LINK-", "ARG0", "ARG1", "ARG2", "ARG3", "ARG4", "ARG5", "rel",
        ];

        for pattern in &role_patterns {
            if let Some(pos) = field.find(pattern) {
                if pos > 0 {
                    // Position part is before the role
                    let positions = &field[..pos - 1]; // -1 to skip the hyphen
                    let role = &field[pos..];
                    return Some((positions, role));
                }
            }
        }

        // Fallback: split on last hyphen
        field
            .rfind('-')
            .map(|pos| (&field[..pos], &field[pos + 1..]))
    }

    /// Parse a .`gold_skel` file using CoNLL-U format
    fn parse_gold_skel_file(
        &mut self,
        file_path: &Path,
        data: &mut PropBankData,
    ) -> EngineResult<()> {
        let sentences = self.conllu_parser.parse_file(file_path)?;

        for sentence in sentences {
            Self::extract_predicate_from_sentence(&sentence, data);
        }

        Ok(())
    }

    /// Extract `PropBank` predicates from a CoNLL-U sentence
    fn extract_predicate_from_sentence(sentence: &ConlluSentence, data: &mut PropBankData) {
        // Look for predicate markers in the MISC field or specific columns
        for (token_idx, token) in sentence.tokens.iter().enumerate() {
            // Check if this token is marked as a predicate
            if Self::is_predicate_token(token) {
                let lemma = token.lemma.clone();

                // Try to infer sense from context or use default "01"
                let sense =
                    Self::infer_predicate_sense(&token.lemma).unwrap_or_else(|| "01".to_string());

                let mut predicate = PropBankPredicate::new(lemma, sense, String::new());
                predicate.predicate_span = Some(token_idx);

                // Find arguments for this predicate
                Self::extract_arguments_for_predicate(sentence, token_idx, &mut predicate);

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
    }

    /// Check if a token represents a predicate
    fn is_predicate_token(token: &crate::engine::ConlluToken) -> bool {
        // In PropBank annotations, predicates are often marked in MISC field
        // or can be identified by POS tags (verbs) and specific annotations
        token.upos.starts_with('V') || // Verb POS tag
        token.misc.contains_key("PropBank") ||
        token.misc.contains_key("pred")
    }

    /// Infer predicate sense from lemma (basic heuristic)
    fn infer_predicate_sense(lemma: &str) -> Option<String> {
        // This could be enhanced with a proper sense disambiguation model
        // For now, use "01" as default sense for most verbs
        match lemma {
            "be" | "have" | "do" => Some("01".to_string()),
            _ => None,
        }
    }

    /// Extract arguments for a given predicate from the sentence
    fn extract_arguments_for_predicate(
        sentence: &ConlluSentence,
        predicate_idx: usize,
        predicate: &mut PropBankPredicate,
    ) {
        // Look for dependency relations that indicate PropBank arguments
        for (token_idx, token) in sentence.tokens.iter().enumerate() {
            if token_idx == predicate_idx {
                continue; // Skip the predicate itself
            }

            // Check if this token is an argument of our predicate
            if let Some(role) = Self::infer_semantic_role_from_dependency(token, predicate_idx) {
                let argument = PropBankArgument::with_span(
                    role,
                    token.form.clone(),
                    (token_idx, token_idx + 1),
                    0.8, // Confidence based on dependency parsing
                );
                predicate.add_argument(argument);
            }
        }
    }

    /// Infer semantic role from dependency relation to predicate
    fn infer_semantic_role_from_dependency(
        token: &crate::engine::ConlluToken,
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

    /// Find files with a specific extension in a directory (recursive)
    fn find_files_with_extension(
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

        Self::find_files_recursive(dir, extension, &mut files)?;

        files.sort();
        Ok(files)
    }

    /// Recursively find files with a specific extension
    fn find_files_recursive(
        dir: &Path,
        extension: &str,
        files: &mut Vec<std::path::PathBuf>,
    ) -> EngineResult<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // Recurse into subdirectories
                Self::find_files_recursive(&path, extension, files)?;
            } else if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == extension {
                        files.push(path);
                    }
                }
            }
        }
        Ok(())
    }

    /// Get predicate by roleset (e.g., "give.01")
    #[must_use]
    pub fn get_predicate<'a>(
        &self,
        data: &'a PropBankData,
        roleset: &str,
    ) -> Option<&'a PropBankPredicate> {
        data.predicates.get(roleset)
    }

    /// Find predicates by lemma (returns all senses)
    #[must_use]
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
    #[must_use]
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

/// Container for all loaded `PropBank` data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropBankData {
    /// All framesets indexed by lemma
    pub framesets: IndexMap<String, PropBankFrameset>,
    /// All predicates indexed by roleset (lemma.sense)
    pub predicates: IndexMap<String, PropBankPredicate>,
    /// Lemma to roleset IDs index for O(1) lookup
    pub lemma_index: std::collections::HashMap<String, Vec<String>>,
    /// Statistics about loaded data
    pub stats: PropBankStats,
}

impl PropBankData {
    /// Create new empty `PropBank` data container
    #[must_use]
    pub fn new() -> Self {
        Self {
            framesets: IndexMap::new(),
            predicates: IndexMap::new(),
            lemma_index: std::collections::HashMap::new(),
            stats: PropBankStats::new(),
        }
    }

    /// Update statistics after loading
    pub fn update_stats(&mut self) {
        self.stats.total_framesets = self.framesets.len();
        self.stats.total_predicates = self.predicates.len();

        // Build lemma index for O(1) lookups
        self.lemma_index.clear();
        for (roleset_id, predicate) in &self.predicates {
            self.lemma_index
                .entry(predicate.lemma.clone())
                .or_default()
                .push(roleset_id.clone());
        }

        // Calculate average arguments per predicate
        if !self.predicates.is_empty() {
            let total_args: usize = self
                .predicates
                .values()
                .map(|pred| pred.arguments.len())
                .sum();
            self.stats.avg_arguments_per_predicate =
                count_to_f32(total_args) / count_to_f32(self.predicates.len());
        }
    }
}

impl Default for PropBankData {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about loaded `PropBank` data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropBankStats {
    pub total_framesets: usize,
    pub total_predicates: usize,
    pub avg_arguments_per_predicate: f32,
    pub prop_files_processed: usize,
    pub gold_skel_files_processed: usize,
}

impl PropBankStats {
    #[must_use]
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
    use crate::propbank::config::{PropBankFeatureFlags, PropBankLoadingFlags};

    fn create_test_config() -> PropBankConfig {
        PropBankConfig {
            data_path: std::path::PathBuf::from("test_data"),
            loading: PropBankLoadingFlags {
                prop_files: true,
                gold_skel_files: false,
            },
            features: PropBankFeatureFlags {
                cache: false,
                fuzzy_matching: true,
                modifiers: true,
            },
            max_files_to_process: Some(10),
            min_confidence: 0.1,
            cache_capacity: 100,
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

        // Test CoNLL-style PropBank format
        let line = "ewt/newsgroup/00/file.xml.parse 0 19 gold give give.01 ----- 14:1-ARG0 19:0-rel 20:1-ARG1 23:1-ARG2";
        let predicate = parser.parse_prop_line(line).unwrap();

        assert_eq!(predicate.lemma, "give");
        assert_eq!(predicate.sense, "01");
        // 3 arguments (rel is skipped)
        assert_eq!(predicate.arguments.len(), 3);

        let arg0 = &predicate.arguments[0];
        assert_eq!(arg0.role, SemanticRole::Agent);
    }

    #[test]
    fn test_prop_line_with_modifiers() {
        let config = create_test_config();
        let parser = PropBankParser::new(config).unwrap();

        // Test line with ARGM modifiers
        let line = "ewt/file.xml.parse 0 19 gold build build.01 ----- 0:2-ARGM-TMP 14:1-ARG0 19:0-rel 20:1-ARG1 23:1-ARGM-LOC";
        let predicate = parser.parse_prop_line(line).unwrap();

        assert_eq!(predicate.lemma, "build");
        assert_eq!(predicate.sense, "01");
        assert_eq!(predicate.arguments.len(), 4); // TMP, ARG0, ARG1, LOC (rel skipped)
    }

    #[test]
    fn test_prop_line_multiword_predicate() {
        let config = create_test_config();
        let parser = PropBankParser::new(config).unwrap();

        // Test multi-word predicate like shut_down.05
        let line = "ewt/file.xml.parse 5 23 gold shut shut_down.05 ----- 20:1-ARG1 23:0,25:0-rel";
        let predicate = parser.parse_prop_line(line).unwrap();

        assert_eq!(predicate.lemma, "shut");
        assert_eq!(predicate.sense, "05");
    }

    #[test]
    fn test_split_arg_field() {
        // Test basic argument
        let result = PropBankParser::split_arg_field("14:1-ARG0");
        assert_eq!(result, Some(("14:1", "ARG0")));

        // Test modifier with hyphen
        let result = PropBankParser::split_arg_field("23:1-ARGM-LOC");
        assert_eq!(result, Some(("23:1", "ARGM-LOC")));

        // Test split argument
        let result = PropBankParser::split_arg_field("0:1*11:1-ARGM-TMP");
        assert_eq!(result, Some(("0:1*11:1", "ARGM-TMP")));

        // Test rel marker
        let result = PropBankParser::split_arg_field("19:0-rel");
        assert_eq!(result, Some(("19:0", "rel")));
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
