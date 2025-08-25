//! WordNet-specific parsing infrastructure
//!
//! This module provides WordNet-specific parsing capabilities for the binary
//! format data files (index.*, data.*) used by Princeton WordNet 3.1.

use canopy_engine::{EngineError, EngineResult};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

/// Configuration for WordNet parsing
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WordNetParserConfig {
    /// Whether to use strict parsing (fail on any error)
    pub strict_mode: bool,
    /// Maximum file size to parse (bytes)
    pub max_file_size: usize,
    /// Skip lines that start with these prefixes (e.g., license text)
    pub skip_prefixes: Vec<String>,
}

impl Default for WordNetParserConfig {
    fn default() -> Self {
        Self {
            strict_mode: false,
            max_file_size: 100 * 1024 * 1024,      // 100MB
            skip_prefixes: vec!["  ".to_string()], // WordNet license lines start with spaces
        }
    }
}

/// WordNet-specific parser for data and index files
#[derive(Debug)]
pub struct WordNetParser {
    config: WordNetParserConfig,
}

impl WordNetParser {
    /// Create a new WordNet parser with default configuration
    pub fn new() -> Self {
        Self {
            config: WordNetParserConfig::default(),
        }
    }

    /// Create a new WordNet parser with custom configuration
    pub fn with_config(config: WordNetParserConfig) -> Self {
        Self { config }
    }

    /// Parse a single WordNet file
    pub fn parse_file<T, F>(&self, path: &Path, parse_fn: F) -> EngineResult<T>
    where
        F: FnOnce(BufReader<File>) -> EngineResult<T>,
    {
        // Check file size
        let metadata = std::fs::metadata(path).map_err(|e| {
            EngineError::data_load(format!(
                "Failed to read file metadata for {}: {}",
                path.display(),
                e
            ))
        })?;

        if metadata.len() > self.config.max_file_size as u64 {
            return Err(EngineError::data_load(format!(
                "File {} too large: {} bytes (max: {})",
                path.display(),
                metadata.len(),
                self.config.max_file_size
            )));
        }

        // Open and parse the file
        let file = File::open(path).map_err(|e| {
            EngineError::data_load(format!(
                "Failed to open WordNet file {}: {}",
                path.display(),
                e
            ))
        })?;

        let reader = BufReader::new(file);

        parse_fn(reader).map_err(|e| {
            EngineError::data_load(format!(
                "Failed to parse WordNet file {}: {}",
                path.display(),
                e
            ))
        })
    }

    /// Get parser configuration
    pub fn config(&self) -> &WordNetParserConfig {
        &self.config
    }

    /// Update parser configuration
    pub fn set_config(&mut self, config: WordNetParserConfig) {
        self.config = config;
    }
}

impl Default for WordNetParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility functions for WordNet parsing
pub mod utils {
    use super::*;

    /// Split a line into fields using whitespace
    pub fn split_fields(line: &str) -> Vec<String> {
        line.split_whitespace().map(|s| s.to_string()).collect()
    }

    /// Parse a numeric field, returning an error if invalid
    pub fn parse_numeric_field<T: std::str::FromStr>(
        field: &str,
        field_name: &str,
    ) -> EngineResult<T>
    where
        T::Err: std::fmt::Display,
    {
        field.parse().map_err(|e| {
            EngineError::data_load(format!("Invalid {field_name} field '{field}': {e}"))
        })
    }

    /// Skip comments and empty lines (WordNet license text starts with spaces)
    pub fn is_license_or_empty(line: &str) -> bool {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return true;
        }

        // WordNet license lines start with spaces
        line.starts_with("  ") || line.starts_with("\t")
    }

    /// Parse hex offset to usize (WordNet synset offsets)
    pub fn parse_synset_offset(hex_str: &str) -> EngineResult<usize> {
        // WordNet offsets are decimal, not hex
        hex_str
            .parse()
            .map_err(|e| EngineError::data_load(format!("Invalid synset offset '{hex_str}': {e}")))
    }

    /// Extract gloss text from synset data (text after '|' separator)
    pub fn extract_gloss(line: &str) -> Option<String> {
        line.find('|').map(|pos| line[pos + 1..].trim().to_string())
    }

    /// Parse WordNet part-of-speech code
    pub fn parse_pos(pos_char: char) -> EngineResult<crate::types::PartOfSpeech> {
        use crate::types::PartOfSpeech;
        match pos_char {
            'n' => Ok(PartOfSpeech::Noun),
            'v' => Ok(PartOfSpeech::Verb),
            'a' => Ok(PartOfSpeech::Adjective),
            's' => Ok(PartOfSpeech::AdjectiveSatellite),
            'r' => Ok(PartOfSpeech::Adverb),
            _ => Err(EngineError::data_load(format!(
                "Invalid part-of-speech: {pos_char}"
            ))),
        }
    }

    /// Parse WordNet pointer symbol to relation type
    pub fn parse_pointer_symbol(symbol: &str) -> EngineResult<crate::types::SemanticRelation> {
        use crate::types::SemanticRelation;
        match symbol {
            "!" => Ok(SemanticRelation::Antonym),
            "@" => Ok(SemanticRelation::Hypernym),
            "~" => Ok(SemanticRelation::Hyponym),
            "@i" => Ok(SemanticRelation::InstanceHypernym),
            "~i" => Ok(SemanticRelation::InstanceHyponym),
            "#m" => Ok(SemanticRelation::MemberHolonym),
            "#s" => Ok(SemanticRelation::SubstanceHolonym),
            "#p" => Ok(SemanticRelation::PartHolonym),
            "%m" => Ok(SemanticRelation::MemberMeronym),
            "%s" => Ok(SemanticRelation::SubstanceMeronym),
            "%p" => Ok(SemanticRelation::PartMeronym),
            "=" => Ok(SemanticRelation::Attribute),
            "+" => Ok(SemanticRelation::Derivation),
            ";c" => Ok(SemanticRelation::DomainTopic),
            ";r" => Ok(SemanticRelation::DomainRegion),
            ";u" => Ok(SemanticRelation::DomainUsage),
            "-c" => Ok(SemanticRelation::MemberTopic),
            "-r" => Ok(SemanticRelation::MemberRegion),
            "-u" => Ok(SemanticRelation::MemberUsage),
            "*" => Ok(SemanticRelation::Entailment),
            ">" => Ok(SemanticRelation::Cause),
            "^" => Ok(SemanticRelation::AlsoSee),
            "$" => Ok(SemanticRelation::VerbGroup),
            "&" => Ok(SemanticRelation::SimilarTo),
            "<" => Ok(SemanticRelation::Participle),
            "\\" => Ok(SemanticRelation::Pertainym),
            _ => Err(EngineError::data_load(format!(
                "Unknown pointer symbol: {symbol}"
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_fields() {
        let line = "field1 field2 field3";
        let fields = utils::split_fields(line);

        assert_eq!(fields, vec!["field1", "field2", "field3"]);
    }

    #[test]
    fn test_parse_numeric_field() {
        assert_eq!(utils::parse_numeric_field::<i32>("42", "test").unwrap(), 42);
        assert!(utils::parse_numeric_field::<i32>("invalid", "test").is_err());
    }

    #[test]
    fn test_is_license_or_empty() {
        assert!(utils::is_license_or_empty(""));
        assert!(utils::is_license_or_empty("  "));
        assert!(utils::is_license_or_empty("  License text"));
        assert!(!utils::is_license_or_empty("data line"));
    }

    #[test]
    fn test_parse_synset_offset() {
        assert_eq!(utils::parse_synset_offset("01234567").unwrap(), 1234567);
        assert!(utils::parse_synset_offset("invalid").is_err());
    }

    #[test]
    fn test_extract_gloss() {
        assert_eq!(
            utils::extract_gloss("synset data | this is the gloss"),
            Some("this is the gloss".to_string())
        );
        assert_eq!(utils::extract_gloss("no gloss here"), None);
    }
}
