//! Configuration for PropBank engine

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for the PropBank engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropBankConfig {
    /// Path to PropBank data directory
    pub data_path: PathBuf,
    /// Enable loading .prop files (structured annotations)
    pub enable_prop_files: bool,
    /// Enable loading .gold_skel files (CoNLL-style format)
    pub enable_gold_skel_files: bool,
    /// Maximum number of files to process (for testing/development)
    pub max_files_to_process: Option<usize>,
    /// Minimum confidence threshold for results
    pub min_confidence: f32,
    /// Enable caching of analysis results
    pub enable_cache: bool,
    /// Cache capacity
    pub cache_capacity: usize,
    /// Enable fuzzy matching for predicate lookup
    pub enable_fuzzy_matching: bool,
    /// Include argument modifiers in analysis
    pub include_modifiers: bool,
    /// Verbose logging for debugging
    pub verbose: bool,
    /// Data sources to load (specific corpora)
    pub data_sources: Vec<String>,
}

impl Default for PropBankConfig {
    fn default() -> Self {
        Self {
            data_path: PathBuf::from("data/propbank/propbank-release/data"),
            enable_prop_files: true,
            enable_gold_skel_files: false, // Start with structured format
            max_files_to_process: Some(1000), // Limit for initial development
            min_confidence: 0.1,
            enable_cache: true,
            cache_capacity: 10000,
            enable_fuzzy_matching: true,
            include_modifiers: true,
            verbose: false,
            data_sources: vec![
                "google/ewt".to_string(), // Start with English Web Treebank
            ],
        }
    }
}

impl PropBankConfig {
    /// Create a new PropBank configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the data path
    pub fn with_data_path<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.data_path = path.into();
        self
    }

    /// Enable or disable .prop file loading
    pub fn with_prop_files(mut self, enable: bool) -> Self {
        self.enable_prop_files = enable;
        self
    }

    /// Enable or disable .gold_skel file loading
    pub fn with_gold_skel_files(mut self, enable: bool) -> Self {
        self.enable_gold_skel_files = enable;
        self
    }

    /// Set maximum files to process
    pub fn with_max_files(mut self, max_files: Option<usize>) -> Self {
        self.max_files_to_process = max_files;
        self
    }

    /// Set minimum confidence threshold
    pub fn with_min_confidence(mut self, confidence: f32) -> Self {
        self.min_confidence = confidence;
        self
    }

    /// Enable or disable caching
    pub fn with_cache(mut self, enable: bool, capacity: usize) -> Self {
        self.enable_cache = enable;
        self.cache_capacity = capacity;
        self
    }

    /// Enable or disable fuzzy matching
    pub fn with_fuzzy_matching(mut self, enable: bool) -> Self {
        self.enable_fuzzy_matching = enable;
        self
    }

    /// Set data sources to load
    pub fn with_data_sources(mut self, sources: Vec<String>) -> Self {
        self.data_sources = sources;
        self
    }

    /// Enable verbose logging
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Create a minimal configuration for testing
    pub fn minimal() -> Self {
        Self {
            max_files_to_process: Some(10),
            data_sources: vec!["google/ewt".to_string()],
            verbose: true,
            ..Self::default()
        }
    }

    /// Create a fast configuration optimized for performance
    pub fn fast() -> Self {
        Self {
            enable_gold_skel_files: false,
            enable_fuzzy_matching: false,
            include_modifiers: false,
            verbose: false,
            ..Self::default()
        }
    }

    /// Get full path to a data source
    pub fn get_data_source_path(&self, source: &str) -> PathBuf {
        self.data_path.join(source)
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        if !self.data_path.exists() {
            return Err(format!(
                "Data path does not exist: {}",
                self.data_path.display()
            ));
        }

        if !self.enable_prop_files && !self.enable_gold_skel_files {
            return Err("At least one file format must be enabled".to_string());
        }

        if self.min_confidence < 0.0 || self.min_confidence > 1.0 {
            return Err("Min confidence must be between 0.0 and 1.0".to_string());
        }

        if self.data_sources.is_empty() {
            return Err("At least one data source must be specified".to_string());
        }

        // Check if data sources exist
        for source in &self.data_sources {
            let source_path = self.get_data_source_path(source);
            if !source_path.exists() {
                return Err(format!(
                    "Data source does not exist: {}",
                    source_path.display()
                ));
            }
        }

        Ok(())
    }
}
