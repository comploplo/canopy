//! Common data loading utilities for semantic engines
//!
//! This module provides standardized data loading patterns, progress tracking,
//! and validation utilities that can be shared across all engines.

use crate::{EngineError, EngineResult, XmlParser, XmlResource};
use indexmap::IndexMap;
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs;
use std::hash::Hash;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tracing::{debug, info, warn};

/// Progress callback type for data loading operations
pub type ProgressCallback = Box<dyn Fn(usize, usize, &str) + Send + Sync>;

/// Configuration for data loading operations
#[derive(Debug, Clone)]
pub struct DataLoaderConfig {
    /// Enable progress reporting
    pub enable_progress: bool,
    /// Progress report interval (every N items)
    pub progress_interval: usize,
    /// Enable detailed logging
    pub verbose: bool,
    /// Maximum number of items to load (0 = no limit)
    pub max_items: usize,
    /// Skip validation of loaded data
    pub skip_validation: bool,
}

impl Default for DataLoaderConfig {
    fn default() -> Self {
        Self {
            enable_progress: true,
            progress_interval: 100,
            verbose: false,
            max_items: 0,
            skip_validation: false,
        }
    }
}

/// Statistics about a completed data loading operation
#[derive(Debug, Clone)]
pub struct LoadingStats {
    /// Total number of files processed
    pub files_processed: usize,
    /// Total number of items loaded
    pub items_loaded: usize,
    /// Number of items that failed validation
    pub validation_failures: usize,
    /// Total loading time
    pub loading_time_ms: u64,
    /// Average time per item in microseconds
    pub avg_time_per_item_us: f64,
    /// Size of loaded data in memory (estimated)
    pub memory_size_bytes: usize,
}

/// Standard data loader providing common loading patterns
pub struct DataLoader {
    config: DataLoaderConfig,
}

impl DataLoader {
    /// Create a new data loader with default configuration
    pub fn new() -> Self {
        Self {
            config: DataLoaderConfig::default(),
        }
    }

    /// Create a new data loader with custom configuration
    pub fn with_config(config: DataLoaderConfig) -> Self {
        Self { config }
    }

    /// Load XML files from a directory and parse them into the specified type
    pub fn load_xml_directory<T>(&self, directory: &Path) -> EngineResult<(Vec<T>, LoadingStats)>
    where
        T: XmlResource + Debug,
    {
        let start_time = Instant::now();
        let mut stats = LoadingStats {
            files_processed: 0,
            items_loaded: 0,
            validation_failures: 0,
            loading_time_ms: 0,
            avg_time_per_item_us: 0.0,
            memory_size_bytes: 0,
        };

        if !directory.exists() {
            return Err(EngineError::data_load(format!(
                "Directory not found: {}",
                directory.display()
            )));
        }

        info!("Loading XML files from {}", directory.display());

        // Find all XML files
        let xml_files = self.find_xml_files(directory)?;

        if xml_files.is_empty() {
            warn!("No XML files found in {}", directory.display());
            return Ok((Vec::new(), stats));
        }

        let mut results = Vec::new();
        let parser = XmlParser::new();

        for (i, file_path) in xml_files.iter().enumerate() {
            if self.config.max_items > 0 && i >= self.config.max_items {
                debug!(
                    "Reached maximum file limit ({}), stopping",
                    self.config.max_items
                );
                break;
            }

            // Progress reporting
            if self.config.enable_progress && (i + 1) % self.config.progress_interval == 0 {
                info!(
                    "Processing file {} of {}: {}",
                    i + 1,
                    xml_files.len(),
                    file_path.display()
                );
            }

            // Parse the XML file
            match parser.parse_file::<T>(file_path) {
                Ok(parsed_item) => {
                    // Validate if not skipped
                    if !self.config.skip_validation {
                        if let Err(e) = parsed_item.validate() {
                            warn!("Validation failed for {}: {}", file_path.display(), e);
                            stats.validation_failures += 1;
                            continue;
                        }
                    }

                    if self.config.verbose {
                        debug!("Successfully loaded: {}", file_path.display());
                    }

                    results.push(parsed_item);
                    stats.items_loaded += 1;
                }
                Err(e) => {
                    warn!("Failed to parse {}: {}", file_path.display(), e);
                }
            }

            stats.files_processed += 1;
        }

        // Calculate final statistics
        stats.loading_time_ms = start_time.elapsed().as_millis() as u64;
        if stats.items_loaded > 0 {
            stats.avg_time_per_item_us =
                (stats.loading_time_ms * 1000) as f64 / stats.items_loaded as f64;
        }
        stats.memory_size_bytes = std::mem::size_of_val(&results);

        info!(
            "Loaded {} items from {} files in {}ms (avg: {:.2}μs per item)",
            stats.items_loaded,
            stats.files_processed,
            stats.loading_time_ms,
            stats.avg_time_per_item_us
        );

        Ok((results, stats))
    }

    /// Load data with a progress callback
    pub fn load_with_progress<T, F>(&self, data_source: &Path, mut loader_fn: F) -> EngineResult<T>
    where
        F: FnMut(&Path, Option<&ProgressCallback>) -> EngineResult<T>,
    {
        let progress_callback = if self.config.enable_progress {
            Some(Box::new(|current: usize, total: usize, message: &str| {
                if current % 50 == 0 || current == total {
                    info!("Progress: {}/{} - {}", current, total, message);
                }
            }) as ProgressCallback)
        } else {
            None
        };

        loader_fn(data_source, progress_callback.as_ref())
    }

    /// Validate and index loaded data
    pub fn validate_and_index<T, K>(
        &self,
        data: Vec<T>,
        key_fn: impl Fn(&T) -> K,
    ) -> EngineResult<IndexMap<K, T>>
    where
        K: Hash + Eq + Clone + Debug,
        T: Debug,
    {
        let start_time = Instant::now();
        let mut indexed_data = IndexMap::new();
        let mut duplicates = 0;

        for item in data {
            let key = key_fn(&item);

            if indexed_data.contains_key(&key) {
                duplicates += 1;
                if self.config.verbose {
                    warn!("Duplicate key found: {:?}", key);
                }
            }

            indexed_data.insert(key, item);
        }

        let indexing_time = start_time.elapsed();
        info!(
            "Indexed {} items in {:.2}ms ({} duplicates)",
            indexed_data.len(),
            indexing_time.as_secs_f64() * 1000.0,
            duplicates
        );

        Ok(indexed_data)
    }

    /// Create a reverse index (e.g., word -> list of items containing that word)
    pub fn create_reverse_index<T, K, V>(
        &self,
        data: &IndexMap<K, T>,
        extract_fn: impl Fn(&T) -> Vec<V>,
    ) -> HashMap<V, Vec<K>>
    where
        K: Clone + Debug,
        V: Hash + Eq + Clone + Debug,
    {
        let start_time = Instant::now();
        let mut reverse_index: HashMap<V, Vec<K>> = HashMap::new();

        for (key, item) in data {
            let values = extract_fn(item);
            for value in values {
                reverse_index.entry(value).or_default().push(key.clone());
            }
        }

        let indexing_time = start_time.elapsed();
        debug!(
            "Created reverse index with {} entries in {:.2}ms",
            reverse_index.len(),
            indexing_time.as_secs_f64() * 1000.0
        );

        reverse_index
    }

    /// Find all XML files in a directory (recursively)
    fn find_xml_files(&self, directory: &Path) -> EngineResult<Vec<PathBuf>> {
        let mut xml_files = Vec::new();

        fn collect_xml_files(dir: &Path, files: &mut Vec<PathBuf>) -> EngineResult<()> {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir() {
                    collect_xml_files(&path, files)?;
                } else if path.extension().and_then(|s| s.to_str()) == Some("xml") {
                    files.push(path);
                }
            }
            Ok(())
        }

        collect_xml_files(directory, &mut xml_files)?;
        xml_files.sort(); // Ensure consistent ordering

        Ok(xml_files)
    }
}

impl Default for DataLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder pattern for creating data loaders with specific configurations
pub struct DataLoaderBuilder {
    config: DataLoaderConfig,
}

impl DataLoaderBuilder {
    /// Create a new data loader builder
    pub fn new() -> Self {
        Self {
            config: DataLoaderConfig::default(),
        }
    }

    /// Enable or disable progress reporting
    pub fn with_progress(mut self, enable: bool) -> Self {
        self.config.enable_progress = enable;
        self
    }

    /// Set progress reporting interval
    pub fn progress_interval(mut self, interval: usize) -> Self {
        self.config.progress_interval = interval;
        self
    }

    /// Enable verbose logging
    pub fn verbose(mut self) -> Self {
        self.config.verbose = true;
        self
    }

    /// Set maximum number of items to load
    pub fn max_items(mut self, max: usize) -> Self {
        self.config.max_items = max;
        self
    }

    /// Skip validation of loaded data
    pub fn skip_validation(mut self) -> Self {
        self.config.skip_validation = true;
        self
    }

    /// Build the data loader
    pub fn build(self) -> DataLoader {
        DataLoader::with_config(self.config)
    }
}

impl Default for DataLoaderBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct TestXmlResource {
        id: String,
        value: String,
    }

    impl XmlResource for TestXmlResource {
        fn parse_xml<R: std::io::BufRead>(reader: &mut quick_xml::Reader<R>) -> EngineResult<Self> {
            // Simple test implementation
            Ok(TestXmlResource {
                id: "test".to_string(),
                value: "test_value".to_string(),
            })
        }

        fn root_element() -> &'static str {
            "test"
        }

        fn validate(&self) -> EngineResult<()> {
            if self.id.is_empty() {
                Err(EngineError::config("ID cannot be empty"))
            } else {
                Ok(())
            }
        }
    }

    #[test]
    fn test_data_loader_creation() {
        let loader = DataLoader::new();
        assert!(loader.config.enable_progress);
        assert_eq!(loader.config.progress_interval, 100);
    }

    #[test]
    fn test_data_loader_builder() {
        let loader = DataLoaderBuilder::new()
            .with_progress(false)
            .progress_interval(50)
            .verbose()
            .max_items(1000)
            .skip_validation()
            .build();

        assert!(!loader.config.enable_progress);
        assert_eq!(loader.config.progress_interval, 50);
        assert!(loader.config.verbose);
        assert_eq!(loader.config.max_items, 1000);
        assert!(loader.config.skip_validation);
    }

    #[test]
    fn test_validate_and_index() {
        let loader = DataLoader::new();
        let test_data = vec![
            TestXmlResource {
                id: "1".to_string(),
                value: "one".to_string(),
            },
            TestXmlResource {
                id: "2".to_string(),
                value: "two".to_string(),
            },
        ];

        let indexed = loader
            .validate_and_index(test_data, |item| item.id.clone())
            .unwrap();
        assert_eq!(indexed.len(), 2);
        assert!(indexed.contains_key("1"));
        assert!(indexed.contains_key("2"));
    }
}
