//! VerbNet semantic engine implementation
//!
//! This module provides the main VerbNet engine that implements canopy-engine traits
//! for semantic analysis using VerbNet verb classes, roles, and frames.
//!
//! ## Performance: Binary Caching
//!
//! VerbNet XML parsing takes ~10-15 seconds. To optimize this, the engine uses binary caching:
//! - First load: Parse XML (~10-15s), save to `data/cache/verbnet.bin`
//! - Subsequent loads: Load from binary cache (~50ms)

use crate::types::{VerbClass, VerbNetAnalysis, VerbNetConfig, VerbNetStats};
use canopy_core::paths::cache_path;
use canopy_engine::{
    traits::DataInfo, BaseEngine, CacheKeyFormat, CommonDataLoader, EngineConfig, EngineCore,
    EngineResult, SemanticResult,
};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::Path;
use tracing::{debug, info, warn};

/// Input type for VerbNet analysis
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerbNetInput {
    pub verb: String,
}

impl Hash for VerbNetInput {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.verb.hash(state);
    }
}

/// Cached VerbNet data - serializable subset of engine state
#[derive(Debug, Clone, Serialize, Deserialize)]
struct VerbNetData {
    verb_classes: IndexMap<String, VerbClass>,
}

impl VerbNetData {
    /// Load from binary cache file (uses absolute workspace path)
    fn load_from_cache() -> Option<Self> {
        let path = cache_path("verbnet.bin");
        if !path.exists() {
            return None;
        }
        match std::fs::read(&path) {
            Ok(bytes) => match bincode::deserialize(&bytes) {
                Ok(data) => {
                    info!("Loaded VerbNet data from cache ({} bytes)", bytes.len());
                    Some(data)
                }
                Err(e) => {
                    warn!("Failed to deserialize VerbNet cache: {}", e);
                    None
                }
            },
            Err(e) => {
                warn!("Failed to read VerbNet cache: {}", e);
                None
            }
        }
    }

    /// Save to binary cache file (uses absolute workspace path)
    fn save_to_cache(&self) -> Result<(), String> {
        let path = cache_path("verbnet.bin");
        let bytes = bincode::serialize(self)
            .map_err(|e| format!("Failed to serialize VerbNet data: {}", e))?;
        std::fs::write(&path, &bytes)
            .map_err(|e| format!("Failed to write VerbNet cache: {}", e))?;
        info!("Saved VerbNet data to cache ({} bytes)", bytes.len());
        Ok(())
    }
}

/// VerbNet semantic analysis engine
#[derive(Debug)]
pub struct VerbNetEngine {
    /// Base engine handling cache, stats, and metrics
    base_engine: BaseEngine<VerbNetInput, VerbNetAnalysis>,
    /// VerbNet verb classes loaded from XML
    verb_classes: IndexMap<String, VerbClass>,
    /// Verb-to-classes mapping for quick lookup
    verb_index: HashMap<String, Vec<String>>,
    /// VerbNet-specific configuration
    verbnet_config: VerbNetConfig,
    /// VerbNet-specific statistics
    stats: VerbNetStats,
}

impl VerbNetEngine {
    /// Create a new VerbNet engine with default configuration and REQUIRED data loading
    /// This constructor will FAIL if VerbNet XML files cannot be loaded - NO STUBS!
    pub fn new() -> EngineResult<Self> {
        Self::with_config(VerbNetConfig::default())
    }

    /// Create a new VerbNet engine with custom configuration and REQUIRED data loading
    /// This constructor will FAIL if VerbNet XML files cannot be loaded - NO STUBS!
    pub fn with_config(verbnet_config: VerbNetConfig) -> EngineResult<Self> {
        // Convert VerbNetConfig to EngineConfig
        let engine_config = EngineConfig {
            enable_cache: verbnet_config.enable_cache,
            cache_capacity: verbnet_config.cache_capacity,
            enable_metrics: true,
            enable_parallel: false,
            max_threads: 4,
            confidence_threshold: verbnet_config.confidence_threshold,
        };

        // Check if data path exists - if not, fail fast (don't use cache for invalid paths)
        let data_path_str = &verbnet_config.data_path;
        let data_path = Path::new(data_path_str);
        if !data_path.exists() {
            return Err(canopy_engine::EngineError::data_load(format!(
                "VerbNet data path does not exist: {}",
                verbnet_config.data_path
            )));
        }

        // Only use binary cache for the default VerbNet data path (not for test data)
        let is_default_path = data_path_str.contains("data/verbnet/vn-gl");

        // Helper to load from XML
        let load_from_xml = |should_cache: bool| -> EngineResult<IndexMap<String, VerbClass>> {
            info!("Parsing VerbNet XML files...");
            let start = std::time::Instant::now();

            let data_loader = CommonDataLoader::new();
            let (raw_classes, loading_stats) = data_loader
                .load_xml_directory::<VerbClass>(Path::new(&verbnet_config.data_path))?;

            if raw_classes.is_empty() {
                return Err(canopy_engine::EngineError::data_load(format!(
                    "No VerbNet XML files found in {}",
                    verbnet_config.data_path
                )));
            }

            let indexed_classes =
                data_loader.validate_and_index(raw_classes, |vc| vc.id.clone())?;

            let mut classes = IndexMap::new();
            for (class_id, verb_class) in indexed_classes {
                debug!(
                    "Loaded VerbNet class: {} ({})",
                    verb_class.id, verb_class.class_name
                );
                classes.insert(class_id, verb_class);
            }

            info!(
                "Parsed VerbNet XML in {:.2}s ({} classes)",
                start.elapsed().as_secs_f64(),
                loading_stats.items_loaded
            );

            // Save to cache only for default path
            if should_cache {
                let data = VerbNetData {
                    verb_classes: classes.clone(),
                };
                if let Err(e) = data.save_to_cache() {
                    warn!("Failed to save VerbNet cache: {}", e);
                }
            }

            Ok(classes)
        };

        let verb_classes = if is_default_path {
            // Try loading from binary cache first (fast path: ~50ms)
            if let Some(cached) = VerbNetData::load_from_cache() {
                info!(
                    "Using cached VerbNet data ({} classes)",
                    cached.verb_classes.len()
                );
                cached.verb_classes
            } else {
                // Cache miss: parse XML (slow path: ~10-15s), then cache for next time
                load_from_xml(true)?
            }
        } else {
            // Non-default path (e.g., test data): don't use or save to cache
            load_from_xml(false)?
        };

        let mut engine = Self {
            base_engine: BaseEngine::new(engine_config, "VerbNet".to_string()),
            verb_classes,
            verb_index: HashMap::new(),
            verbnet_config,
            stats: VerbNetStats {
                total_classes: 0,
                total_verbs: 0,
                total_queries: 0,
                cache_hits: 0,
                cache_misses: 0,
                avg_query_time_us: 0.0,
            },
        };

        // Build index and update stats
        engine.stats.total_classes = engine.verb_classes.len();
        engine.build_verb_index();

        info!(
            "VerbNet engine initialized with {} classes and {} verbs",
            engine.stats.total_classes, engine.stats.total_verbs
        );

        Ok(engine)
    }

    /// Analyze a verb and return matching classes and semantic information
    pub fn analyze_verb(&self, verb: &str) -> EngineResult<SemanticResult<VerbNetAnalysis>> {
        let input = VerbNetInput {
            verb: verb.to_string(),
        };
        self.base_engine.analyze(&input, self)
    }

    /// Find verb classes that contain the given verb
    fn find_verb_classes(&self, verb: &str) -> EngineResult<Vec<VerbClass>> {
        let mut matching_classes = Vec::new();

        // Direct lookup in verb index
        if let Some(class_ids) = self.verb_index.get(verb) {
            for class_id in class_ids {
                if let Some(verb_class) = self.verb_classes.get(class_id) {
                    matching_classes.push(verb_class.clone());
                }
            }
        }

        // If no direct matches, try lemmatization and partial matching
        if matching_classes.is_empty() {
            matching_classes = self.fuzzy_verb_search(verb)?;
        }

        Ok(matching_classes)
    }

    /// Perform fuzzy search for verbs (basic lemmatization and partial matching)
    fn fuzzy_verb_search(&self, verb: &str) -> EngineResult<Vec<VerbClass>> {
        let mut matching_classes = Vec::new();
        let verb_lower = verb.to_lowercase();

        // Try basic lemmatization patterns
        let mut patterns = vec![
            verb_lower.clone(),
            // Remove common suffixes
            verb_lower.trim_end_matches("ing").to_string(),
            verb_lower.trim_end_matches("ed").to_string(),
            verb_lower.trim_end_matches("s").to_string(),
            // Add common suffixes if not present
            format!("{}e", verb_lower.trim_end_matches("e")),
            format!("{}d", verb_lower),
            format!("{}ing", verb_lower),
        ];

        // Handle special -ing cases
        if verb_lower.ends_with("ing") {
            let stem = verb_lower.trim_end_matches("ing");
            // Check if the consonant was doubled (like "running" -> "run")
            if stem.len() >= 2 {
                let chars: Vec<char> = stem.chars().collect();
                if chars.len() >= 2 && chars[chars.len() - 1] == chars[chars.len() - 2] {
                    // Remove the doubled consonant
                    let single_consonant_stem = &stem[0..stem.len() - 1];
                    patterns.push(single_consonant_stem.to_string());
                    patterns.push(format!("{single_consonant_stem}e"));
                }
            }
            // Also try adding 'e' to the stem (like "giv" -> "give")
            patterns.push(format!("{stem}e"));
        }

        for pattern in patterns {
            if let Some(class_ids) = self.verb_index.get(&pattern) {
                for class_id in class_ids {
                    if let Some(verb_class) = self.verb_classes.get(class_id) {
                        // Avoid duplicates
                        if !matching_classes
                            .iter()
                            .any(|c: &VerbClass| c.id == verb_class.id)
                        {
                            matching_classes.push(verb_class.clone());
                        }
                    }
                }
            }
        }

        Ok(matching_classes)
    }

    /// Calculate confidence score based on matching classes
    fn calculate_verb_confidence(&self, classes: &[VerbClass]) -> f32 {
        if classes.is_empty() {
            return 0.0;
        }

        // Base confidence on number of classes and their specificity
        let base_confidence = match classes.len() {
            1 => 0.9,     // Single exact match is highly confident
            2..=3 => 0.8, // Few matches are still confident
            4..=6 => 0.7, // Several matches are moderately confident
            _ => 0.6,     // Many matches suggest ambiguity
        };

        // Adjust based on class depth and specificity
        let avg_specificity = classes
            .iter()
            .map(|c| {
                // More specific classes (longer IDs) get higher scores
                let specificity = c.id.matches('-').count() as f32 * 0.1;
                // Classes with more frames are more informative
                let frame_bonus = (c.frames.len() as f32 * 0.05).min(0.2);
                specificity + frame_bonus
            })
            .sum::<f32>()
            / classes.len() as f32;

        (base_confidence + avg_specificity).min(0.95)
    }

    /// Build verb-to-classes index for fast lookup
    fn build_verb_index(&mut self) {
        self.verb_index.clear();

        for (class_id, verb_class) in &self.verb_classes {
            for member in &verb_class.members {
                self.verb_index
                    .entry(member.name.clone())
                    .or_default()
                    .push(class_id.clone());
            }
        }

        self.stats.total_verbs = self.verb_index.len();
        info!(
            "Built VerbNet verb index: {} unique verbs across {} classes",
            self.stats.total_verbs, self.stats.total_classes
        );
    }

    /// Get verb class by ID
    pub fn get_verb_class(&self, class_id: &str) -> Option<&VerbClass> {
        self.verb_classes.get(class_id)
    }

    /// Get all loaded verb classes
    pub fn get_all_classes(&self) -> Vec<&VerbClass> {
        self.verb_classes.values().collect()
    }

    /// Check if the engine has loaded data
    pub fn is_loaded(&self) -> bool {
        !self.verb_classes.is_empty()
    }

    /// Get verbs in a specific class
    pub fn get_class_verbs(&self, class_id: &str) -> Option<Vec<&str>> {
        self.verb_classes.get(class_id).map(|c| c.get_members())
    }

    /// Search for classes by pattern
    pub fn search_classes(&self, pattern: &str) -> Vec<&VerbClass> {
        let pattern_lower = pattern.to_lowercase();
        self.verb_classes
            .values()
            .filter(|c| {
                c.id.to_lowercase().contains(&pattern_lower)
                    || c.class_name.to_lowercase().contains(&pattern_lower)
            })
            .collect()
    }
}

/// Implementation of EngineCore trait for BaseEngine integration
impl EngineCore<VerbNetInput, VerbNetAnalysis> for VerbNetEngine {
    fn perform_analysis(&self, input: &VerbNetInput) -> EngineResult<VerbNetAnalysis> {
        // Find matching verb classes
        let matching_classes = self.find_verb_classes(&input.verb)?;

        if matching_classes.is_empty() {
            debug!("No VerbNet classes found for verb: {}", input.verb);
            return Ok(VerbNetAnalysis::new(input.verb.clone(), Vec::new(), 0.1));
        }

        // Calculate confidence based on number of matches and class specificity
        let confidence = self.calculate_verb_confidence(&matching_classes);

        // Create analysis result
        let analysis = VerbNetAnalysis::new(input.verb.clone(), matching_classes, confidence);

        debug!(
            "VerbNet analysis for '{}': {} classes found, confidence: {:.2}",
            input.verb,
            analysis.verb_classes.len(),
            confidence
        );

        Ok(analysis)
    }

    fn calculate_confidence(&self, _input: &VerbNetInput, output: &VerbNetAnalysis) -> f32 {
        output.confidence
    }

    fn generate_cache_key(&self, input: &VerbNetInput) -> String {
        CacheKeyFormat::Typed("verbnet".to_string(), input.verb.clone()).to_string()
    }

    fn engine_name(&self) -> &'static str {
        "VerbNet"
    }

    fn engine_version(&self) -> &'static str {
        "3.4"
    }

    fn is_initialized(&self) -> bool {
        !self.verb_classes.is_empty()
    }
}

impl VerbNetEngine {
    /// Create VerbNet engine with test data for testing only
    #[cfg(test)]
    pub fn new_with_test_data<P: AsRef<Path>>(test_data_path: P) -> EngineResult<Self> {
        let config = VerbNetConfig {
            data_path: test_data_path.as_ref().to_string_lossy().to_string(),
            ..Default::default()
        };
        Self::with_config(config)
    }

    /// Load VerbNet data from directory using CommonDataLoader
    pub fn load_from_directory<P: AsRef<Path>>(&mut self, path: P) -> EngineResult<()> {
        let path = path.as_ref();
        info!("Loading VerbNet data from: {}", path.display());

        let data_loader = CommonDataLoader::new();
        let (verb_classes, loading_stats) = data_loader.load_xml_directory::<VerbClass>(path)?;

        // Check if no classes were loaded and return error for compatibility
        if verb_classes.is_empty() {
            return Err(canopy_engine::EngineError::data_load(format!(
                "No VerbNet XML files found in {}",
                path.display()
            )));
        }

        self.verb_classes.clear();
        let indexed_classes = data_loader.validate_and_index(verb_classes, |vc| vc.id.clone())?;

        for (class_id, verb_class) in indexed_classes {
            info!(
                "Loaded VerbNet class: {} ({})",
                verb_class.id, verb_class.class_name
            );
            self.verb_classes.insert(class_id, verb_class);
        }

        self.stats.total_classes = self.verb_classes.len();
        self.build_verb_index();

        info!(
            "VerbNet data loading complete: {} classes, {} verbs in {}ms",
            loading_stats.items_loaded, self.stats.total_verbs, loading_stats.loading_time_ms
        );

        Ok(())
    }

    /// Reload VerbNet data (clears current data)
    pub fn reload(&mut self) -> EngineResult<()> {
        self.verb_classes.clear();
        self.verb_index.clear();
        self.base_engine.clear_cache();
        // Return error to match expected behavior
        Err(canopy_engine::EngineError::data_load(
            "Reload requires a data path".to_string(),
        ))
    }

    /// Get engine statistics from BaseEngine
    pub fn get_stats(&self) -> canopy_engine::EngineStats {
        self.base_engine.get_stats()
    }

    /// Get performance metrics from BaseEngine
    pub fn get_performance_metrics(&self) -> canopy_engine::PerformanceMetrics {
        self.base_engine.get_performance_metrics()
    }

    /// Get quality metrics from BaseEngine
    pub fn get_quality_metrics(&self) -> canopy_engine::QualityMetrics {
        self.base_engine.get_quality_metrics()
    }

    /// Get cache statistics from BaseEngine
    pub fn cache_stats(&self) -> canopy_engine::CacheStats {
        self.base_engine.cache_stats()
    }

    /// Clear cache via BaseEngine
    pub fn clear_cache(&self) {
        self.base_engine.clear_cache();
    }

    /// Get VerbNet configuration (for backward compatibility)
    pub fn config(&self) -> &VerbNetConfig {
        &self.verbnet_config
    }

    /// Get engine statistics (alias for get_stats)
    pub fn statistics(&self) -> canopy_engine::EngineStats {
        self.get_stats()
    }

    /// Get performance metrics (alias for get_performance_metrics)
    pub fn performance_metrics(&self) -> canopy_engine::PerformanceMetrics {
        self.get_performance_metrics()
    }

    /// Engine name (for backward compatibility)
    pub fn name(&self) -> &'static str {
        "VerbNet"
    }

    /// Engine version (for backward compatibility)
    pub fn version(&self) -> &'static str {
        "3.4"
    }

    /// Analyze using string input (for backward compatibility)
    pub fn analyze(&self, input: &str) -> EngineResult<SemanticResult<VerbNetAnalysis>> {
        self.analyze_verb(input)
    }

    /// Load test data (placeholder for compatibility)
    pub fn load_test_data(&mut self) -> EngineResult<()> {
        Err(canopy_engine::EngineError::data_load(
            "Test data loading not implemented".to_string(),
        ))
    }

    /// Get data info (for compatibility)
    pub fn data_info(&self) -> DataInfo {
        DataInfo::new(
            self.verbnet_config.data_path.clone(),
            self.verb_classes.len(),
        )
    }

    /// Set cache capacity (for compatibility)
    pub fn set_cache_capacity(&mut self, capacity: usize) {
        // Update the verbnet config to reflect the new capacity
        self.verbnet_config.cache_capacity = capacity;
        // Note: BaseEngine doesn't support changing capacity after creation
        // This is a limitation of the current design
    }
}

// No Default implementation - engines must explicitly load data to prevent stubs

#[cfg(test)]
mod tests {
    use super::*;
    use once_cell::sync::OnceCell;
    use std::fs;
    use std::path::Path;
    use std::sync::Mutex;
    use tempfile::tempdir;

    // Shared engine singleton - loaded once per test binary
    static SHARED_ENGINE: OnceCell<Mutex<VerbNetEngine>> = OnceCell::new();

    /// Check if VerbNet data is available
    fn verbnet_available() -> bool {
        canopy_core::paths::data_path("data/verbnet").exists()
    }

    /// Get shared VerbNet engine, or None if data unavailable
    fn shared_engine() -> Option<&'static Mutex<VerbNetEngine>> {
        if !verbnet_available() {
            return None;
        }
        Some(SHARED_ENGINE.get_or_init(|| {
            eprintln!("🔧 Loading shared VerbNet engine (one-time)...");
            Mutex::new(VerbNetEngine::new().expect("VerbNet data required"))
        }))
    }

    fn create_test_verbnet_xml() -> &'static str {
        r#"<?xml version="1.0" encoding="UTF-8"?>
        <VNCLASS ID="give-13.1" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
            <MEMBERS>
                <MEMBER name="give" wn="give%2:40:00" grouping="give.01"/>
                <MEMBER name="hand" wn="hand%2:35:00" grouping="hand.01"/>
            </MEMBERS>
            <THEMROLES>
                <THEMROLE type="Agent">
                    <SELRESTRS logic="or">
                        <SELRESTR Value="+" type="animate"/>
                    </SELRESTRS>
                </THEMROLE>
                <THEMROLE type="Theme">
                    <SELRESTRS/>
                </THEMROLE>
                <THEMROLE type="Recipient">
                    <SELRESTRS logic="or">
                        <SELRESTR Value="+" type="animate"/>
                    </SELRESTRS>
                </THEMROLE>
            </THEMROLES>
            <FRAMES>
                <FRAME>
                    <DESCRIPTION descriptionNumber="0.1" primary="Basic Transitive" secondary="NP V NP" xtag="0.1"/>
                    <EXAMPLES>
                        <EXAMPLE>I gave the book to Mary.</EXAMPLE>
                    </EXAMPLES>
                    <SYNTAX>
                        <NP value="Agent"><SYNRESTRS/></NP>
                        <VERB/>
                        <NP value="Theme"><SYNRESTRS/></NP>
                        <PREP value="to"><SYNRESTRS/></PREP>
                        <NP value="Recipient"><SYNRESTRS/></NP>
                    </SYNTAX>
                    <SEMANTICS>
                        <PRED value="cause">
                            <ARGS>
                                <ARG type="ThemRole" value="Agent"/>
                                <ARG type="Event" value="E"/>
                            </ARGS>
                        </PRED>
                        <PRED value="transfer">
                            <ARGS>
                                <ARG type="Event" value="during(E)"/>
                                <ARG type="ThemRole" value="Agent"/>
                                <ARG type="ThemRole" value="Theme"/>
                                <ARG type="ThemRole" value="Recipient"/>
                            </ARGS>
                        </PRED>
                    </SEMANTICS>
                </FRAME>
            </FRAMES>
        </VNCLASS>"#
    }

    #[test]
    fn test_verbnet_engine_creation_with_path_resolution() {
        // Use shared engine (loaded once, reused across tests)
        let Some(engine_ref) = shared_engine() else {
            eprintln!("Skipping test: VerbNet data not available");
            return;
        };

        let engine = engine_ref.lock().unwrap();
        // Engine should be loaded with real data
        assert!(engine.is_loaded(), "Engine should be loaded with real data");
        assert!(
            engine.stats.total_classes > 0,
            "Should have loaded verb classes"
        );
    }

    #[test]
    fn test_load_verbnet_data() {
        let temp_dir = tempdir().unwrap();
        let xml_path = temp_dir.path().join("give-13.1.xml");
        fs::write(&xml_path, create_test_verbnet_xml()).unwrap();

        let engine = VerbNetEngine::new_with_test_data(temp_dir.path()).unwrap();

        assert!(engine.is_loaded());
        assert_eq!(engine.stats.total_classes, 1);
        assert_eq!(engine.stats.total_verbs, 2); // "give" and "hand"
    }

    #[test]
    fn test_verb_analysis() {
        let temp_dir = tempdir().unwrap();
        let xml_path = temp_dir.path().join("give-13.1.xml");
        fs::write(&xml_path, create_test_verbnet_xml()).unwrap();

        let engine = VerbNetEngine::new_with_test_data(temp_dir.path()).unwrap();

        let result = engine.analyze_verb("give").unwrap();
        assert!(result.confidence > 0.5);
        assert_eq!(result.data.verb, "give");
        assert_eq!(result.data.verb_classes.len(), 1);
        assert_eq!(result.data.verb_classes[0].id, "give-13.1");
    }

    #[test]
    fn test_fuzzy_verb_search() {
        let temp_dir = tempdir().unwrap();
        let xml_path = temp_dir.path().join("give-13.1.xml");
        fs::write(&xml_path, create_test_verbnet_xml()).unwrap();

        let engine = VerbNetEngine::new_with_test_data(temp_dir.path()).unwrap();

        // Test basic lemmatization
        let result = engine.analyze_verb("giving").unwrap();
        assert!(result.confidence > 0.0);
        assert_eq!(result.data.verb_classes.len(), 1);
    }

    #[test]
    fn test_cache_functionality() {
        let temp_dir = tempdir().unwrap();
        let xml_path = temp_dir.path().join("give-13.1.xml");
        fs::write(&xml_path, create_test_verbnet_xml()).unwrap();

        let engine = VerbNetEngine::new_with_test_data(temp_dir.path()).unwrap();

        // First query - should miss cache
        let result1 = engine.analyze_verb("give").unwrap();
        assert!(!result1.from_cache);

        // Second query - should hit cache
        let result2 = engine.analyze_verb("give").unwrap();
        assert!(result2.from_cache);

        // Check cache stats from BaseEngine
        let cache_stats = engine.cache_stats();
        assert!(cache_stats.hits > 0);
        assert!(cache_stats.total_lookups >= 2);
    }

    #[test]
    fn test_confidence_calculation() {
        // Use shared engine (loaded once, reused across tests)
        let Some(engine_ref) = shared_engine() else {
            eprintln!("Skipping test: VerbNet data not available");
            return;
        };

        let engine = engine_ref.lock().unwrap();

        // Test empty classes
        assert_eq!(engine.calculate_verb_confidence(&[]), 0.0);

        // Test single class
        let single_class = vec![VerbClass {
            id: "test-1.0".to_string(),
            class_name: "Test".to_string(),
            parent_class: None,
            members: vec![],
            themroles: vec![],
            frames: vec![],
            subclasses: vec![],
        }];
        let confidence = engine.calculate_verb_confidence(&single_class);
        assert!(confidence > 0.8);
    }
}
