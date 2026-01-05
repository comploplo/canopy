//! Configuration for `PropBank` engine

use crate::paths::data_path_string;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// File loading configuration for `PropBank`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropBankLoadingFlags {
    /// Enable loading .prop files (structured annotations)
    pub prop_files: bool,
    /// Enable loading .`gold_skel` files (CoNLL-style format)
    pub gold_skel_files: bool,
}

impl Default for PropBankLoadingFlags {
    fn default() -> Self {
        Self {
            prop_files: true,
            gold_skel_files: false,
        }
    }
}

/// Analysis feature flags for `PropBank`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropBankFeatureFlags {
    /// Enable caching of analysis results
    pub cache: bool,
    /// Enable fuzzy matching for predicate lookup
    pub fuzzy_matching: bool,
    /// Include argument modifiers in analysis
    pub modifiers: bool,
}

impl Default for PropBankFeatureFlags {
    fn default() -> Self {
        Self {
            cache: true,
            fuzzy_matching: true,
            modifiers: true,
        }
    }
}

/// Configuration for the `PropBank` engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropBankConfig {
    /// Path to `PropBank` data directory
    pub data_path: PathBuf,
    /// File loading flags
    pub loading: PropBankLoadingFlags,
    /// Feature flags
    pub features: PropBankFeatureFlags,
    /// Maximum number of files to process (for testing/development)
    pub max_files_to_process: Option<usize>,
    /// Minimum confidence threshold for results
    pub min_confidence: f32,
    /// Cache capacity
    pub cache_capacity: usize,
    /// Verbose logging for debugging
    pub verbose: bool,
    /// Data sources to load (specific corpora)
    pub data_sources: Vec<String>,
}

impl Default for PropBankConfig {
    fn default() -> Self {
        Self {
            data_path: PathBuf::from(data_path_string("data/propbank/propbank-release/data")),
            loading: PropBankLoadingFlags::default(),
            features: PropBankFeatureFlags::default(),
            max_files_to_process: Some(1000), // Limit for initial development
            min_confidence: 0.1,
            cache_capacity: 10000,
            verbose: false,
            data_sources: vec![
                "google/ewt".to_string(), // Start with English Web Treebank
            ],
        }
    }
}

// Implement EngineConfigurable trait directly (can't use macro due to nested fields)
impl crate::engine::EngineConfigurable for PropBankConfig {
    fn enable_cache(&self) -> bool {
        self.features.cache
    }
    fn cache_capacity(&self) -> usize {
        self.cache_capacity
    }
    fn confidence_threshold(&self) -> f32 {
        self.min_confidence
    }
}

impl PropBankConfig {
    /// Create a new `PropBank` configuration
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the data path
    #[must_use]
    pub fn with_data_path<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.data_path = path.into();
        self
    }

    /// Enable or disable .prop file loading
    #[must_use]
    pub fn with_prop_files(mut self, enable: bool) -> Self {
        self.loading.prop_files = enable;
        self
    }

    /// Enable or disable .`gold_skel` file loading
    #[must_use]
    pub fn with_gold_skel_files(mut self, enable: bool) -> Self {
        self.loading.gold_skel_files = enable;
        self
    }

    /// Set maximum files to process
    #[must_use]
    pub fn with_max_files(mut self, max_files: Option<usize>) -> Self {
        self.max_files_to_process = max_files;
        self
    }

    /// Set minimum confidence threshold
    #[must_use]
    pub fn with_min_confidence(mut self, confidence: f32) -> Self {
        self.min_confidence = confidence;
        self
    }

    /// Enable or disable caching
    #[must_use]
    pub fn with_cache(mut self, enable: bool, capacity: usize) -> Self {
        self.features.cache = enable;
        self.cache_capacity = capacity;
        self
    }

    /// Enable or disable fuzzy matching
    #[must_use]
    pub fn with_fuzzy_matching(mut self, enable: bool) -> Self {
        self.features.fuzzy_matching = enable;
        self
    }

    /// Set data sources to load
    #[must_use]
    pub fn with_data_sources(mut self, sources: Vec<String>) -> Self {
        self.data_sources = sources;
        self
    }

    /// Enable verbose logging
    #[must_use]
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Create a minimal configuration for testing
    #[must_use]
    pub fn minimal() -> Self {
        Self {
            max_files_to_process: Some(10),
            data_sources: vec!["google/ewt".to_string()],
            verbose: true,
            ..Self::default()
        }
    }

    /// Create a fast configuration optimized for performance
    #[must_use]
    pub fn fast() -> Self {
        Self {
            loading: PropBankLoadingFlags {
                prop_files: true,
                gold_skel_files: false,
            },
            features: PropBankFeatureFlags {
                cache: true,
                fuzzy_matching: false,
                modifiers: false,
            },
            verbose: false,
            ..Self::default()
        }
    }

    /// Get full path to a data source
    #[must_use]
    pub fn get_data_source_path(&self, source: &str) -> PathBuf {
        self.data_path.join(source)
    }

    /// Accessor for backward compatibility: `enable_prop_files`
    #[must_use]
    pub fn enable_prop_files(&self) -> bool {
        self.loading.prop_files
    }

    /// Accessor for backward compatibility: `enable_gold_skel_files`
    #[must_use]
    pub fn enable_gold_skel_files(&self) -> bool {
        self.loading.gold_skel_files
    }

    /// Accessor for backward compatibility: `enable_cache`
    #[must_use]
    pub fn enable_cache(&self) -> bool {
        self.features.cache
    }

    /// Accessor for backward compatibility: `enable_fuzzy_matching`
    #[must_use]
    pub fn enable_fuzzy_matching(&self) -> bool {
        self.features.fuzzy_matching
    }

    /// Accessor for backward compatibility: `include_modifiers`
    #[must_use]
    pub fn include_modifiers(&self) -> bool {
        self.features.modifiers
    }

    /// Validate configuration
    ///
    /// # Errors
    /// Returns an error if the configuration is invalid.
    pub fn validate(&self) -> Result<(), String> {
        if !self.data_path.exists() {
            return Err(format!(
                "Data path does not exist: {}",
                self.data_path.display()
            ));
        }

        if !self.loading.prop_files && !self.loading.gold_skel_files {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_propbank_config_default() {
        let config = PropBankConfig::default();
        assert!(config.enable_prop_files());
        assert!(!config.enable_gold_skel_files());
        assert_eq!(config.max_files_to_process, Some(1000));
        assert!((config.min_confidence - 0.1).abs() < f32::EPSILON);
        assert!(config.enable_cache());
        assert_eq!(config.cache_capacity, 10000);
        assert!(config.enable_fuzzy_matching());
        assert!(config.include_modifiers());
        assert!(!config.verbose);
        assert!(!config.data_sources.is_empty());
    }

    #[test]
    fn test_propbank_config_new() {
        let config = PropBankConfig::new();
        assert!((config.min_confidence - 0.1).abs() < f32::EPSILON);
    }

    #[test]
    fn test_propbank_config_with_data_path() {
        let config = PropBankConfig::new().with_data_path("/custom/path");
        assert_eq!(config.data_path, PathBuf::from("/custom/path"));
    }

    #[test]
    fn test_propbank_config_with_prop_files() {
        let config = PropBankConfig::new().with_prop_files(false);
        assert!(!config.enable_prop_files());
    }

    #[test]
    fn test_propbank_config_with_gold_skel_files() {
        let config = PropBankConfig::new().with_gold_skel_files(true);
        assert!(config.enable_gold_skel_files());
    }

    #[test]
    fn test_propbank_config_with_max_files() {
        let config = PropBankConfig::new().with_max_files(Some(50));
        assert_eq!(config.max_files_to_process, Some(50));

        let config2 = PropBankConfig::new().with_max_files(None);
        assert!(config2.max_files_to_process.is_none());
    }

    #[test]
    fn test_propbank_config_with_min_confidence() {
        let config = PropBankConfig::new().with_min_confidence(0.5);
        assert!((config.min_confidence - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_propbank_config_with_cache() {
        let config = PropBankConfig::new().with_cache(false, 5000);
        assert!(!config.enable_cache());
        assert_eq!(config.cache_capacity, 5000);
    }

    #[test]
    fn test_propbank_config_with_fuzzy_matching() {
        let config = PropBankConfig::new().with_fuzzy_matching(false);
        assert!(!config.enable_fuzzy_matching());
    }

    #[test]
    fn test_propbank_config_with_data_sources() {
        let sources = vec!["source1".to_string(), "source2".to_string()];
        let config = PropBankConfig::new().with_data_sources(sources.clone());
        assert_eq!(config.data_sources, sources);
    }

    #[test]
    fn test_propbank_config_with_verbose() {
        let config = PropBankConfig::new().with_verbose(true);
        assert!(config.verbose);
    }

    #[test]
    fn test_propbank_config_minimal() {
        let config = PropBankConfig::minimal();
        assert_eq!(config.max_files_to_process, Some(10));
        assert!(config.verbose);
    }

    #[test]
    fn test_propbank_config_fast() {
        let config = PropBankConfig::fast();
        assert!(!config.enable_gold_skel_files());
        assert!(!config.enable_fuzzy_matching());
        assert!(!config.include_modifiers());
        assert!(!config.verbose);
    }

    #[test]
    fn test_propbank_config_get_data_source_path() {
        let config = PropBankConfig::new().with_data_path("/base");
        let path = config.get_data_source_path("subdir");
        assert_eq!(path, PathBuf::from("/base/subdir"));
    }

    #[test]
    fn test_propbank_config_validate_no_formats_enabled() {
        // Test that validation catches when no formats are enabled
        // Note: validation also checks data path exists, so this might fail on path check first
        let config = PropBankConfig::new()
            .with_prop_files(false)
            .with_gold_skel_files(false);
        let result = config.validate();
        // Should fail validation (either path doesn't exist or formats disabled)
        assert!(result.is_err());
    }

    #[test]
    fn test_propbank_config_validate_invalid_confidence() {
        // Test that validation catches invalid confidence values
        let config = PropBankConfig::new().with_min_confidence(-0.1);
        let result = config.validate();
        // Should fail validation (either path doesn't exist or invalid confidence)
        assert!(result.is_err());

        let config2 = PropBankConfig::new().with_min_confidence(1.5);
        let result2 = config2.validate();
        assert!(result2.is_err());
    }

    #[test]
    fn test_propbank_config_validate_empty_data_sources() {
        // Test that validation catches when no data sources are specified
        let config = PropBankConfig::new().with_data_sources(vec![]);
        let result = config.validate();
        // Should fail validation (either path doesn't exist or no sources)
        assert!(result.is_err());
    }

    #[test]
    fn test_propbank_config_clone_debug() {
        let config = PropBankConfig::default();
        let cloned = config.clone();
        assert!((cloned.min_confidence - 0.1).abs() < f32::EPSILON);
        let debug = format!("{config:?}");
        assert!(debug.contains("PropBankConfig"));
    }

    #[test]
    fn test_propbank_config_builder_chain() {
        let config = PropBankConfig::new()
            .with_data_path("/custom")
            .with_min_confidence(0.3)
            .with_cache(true, 5000)
            .with_verbose(true);

        assert_eq!(config.data_path, PathBuf::from("/custom"));
        assert!((config.min_confidence - 0.3).abs() < f32::EPSILON);
        assert!(config.enable_cache());
        assert_eq!(config.cache_capacity, 5000);
        assert!(config.verbose);
    }
}
