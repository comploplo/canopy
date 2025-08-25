//! VerbNet semantic engine implementation
//!
//! This module provides the main VerbNet engine that implements canopy-engine traits
//! for semantic analysis using VerbNet verb classes, roles, and frames.

use crate::types::{VerbClass, VerbNetAnalysis, VerbNetConfig, VerbNetStats};
use canopy_engine::{
    traits::DataInfo, CachedEngine, DataLoader, EngineCache, EngineError, EngineResult,
    EngineStats, PerformanceMetrics, SemanticEngine, SemanticResult, StatisticsProvider, XmlParser,
};
use indexmap::IndexMap;
use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;
use tracing::{debug, info};

/// VerbNet semantic analysis engine
#[derive(Debug)]
pub struct VerbNetEngine {
    /// VerbNet verb classes loaded from XML
    verb_classes: IndexMap<String, VerbClass>,
    /// Verb-to-classes mapping for quick lookup
    verb_index: HashMap<String, Vec<String>>,
    /// Engine configuration
    config: VerbNetConfig,
    /// Result cache
    cache: EngineCache<String, VerbNetAnalysis>,
    /// Performance statistics
    stats: VerbNetStats,
    /// Performance metrics
    performance_metrics: PerformanceMetrics,
    /// Engine statistics
    engine_stats: EngineStats,
}

impl VerbNetEngine {
    /// Create a new VerbNet engine with default configuration
    pub fn new() -> Self {
        Self::with_config(VerbNetConfig::default())
    }

    /// Create a new VerbNet engine with custom configuration
    pub fn with_config(config: VerbNetConfig) -> Self {
        let cache = EngineCache::new(config.cache_capacity);

        Self {
            verb_classes: IndexMap::new(),
            verb_index: HashMap::new(),
            config,
            cache,
            stats: VerbNetStats {
                total_classes: 0,
                total_verbs: 0,
                total_queries: 0,
                cache_hits: 0,
                cache_misses: 0,
                avg_query_time_us: 0.0,
            },
            performance_metrics: PerformanceMetrics::new(),
            engine_stats: EngineStats::new("VerbNet".to_string()),
        }
    }

    /// Analyze a verb and return matching classes and semantic information
    pub fn analyze_verb(&mut self, verb: &str) -> EngineResult<SemanticResult<VerbNetAnalysis>> {
        let start_time = Instant::now();
        self.stats.total_queries += 1;

        // Check cache first
        if self.config.enable_cache {
            let cache_key = format!("verb:{verb}");
            if let Some(cached_result) = self.cache.get(&cache_key) {
                self.stats.cache_hits += 1;
                let _processing_time = start_time.elapsed().as_micros() as u64;
                return Ok(SemanticResult::cached(
                    cached_result.clone(),
                    cached_result.confidence,
                ));
            }
        }

        self.stats.cache_misses += 1;

        // Find matching verb classes
        let matching_classes = self.find_verb_classes(verb)?;

        if matching_classes.is_empty() {
            debug!("No VerbNet classes found for verb: {}", verb);
            let analysis = VerbNetAnalysis::new(verb.to_string(), Vec::new(), 0.1);
            let processing_time = start_time.elapsed().as_micros() as u64;
            self.update_performance_metrics(processing_time);
            return Ok(SemanticResult::new(analysis, 0.1, false, processing_time));
        }

        // Calculate confidence based on number of matches and class specificity
        let confidence = self.calculate_confidence(&matching_classes);

        // Create analysis result
        let analysis = VerbNetAnalysis::new(verb.to_string(), matching_classes, confidence);

        // Cache the result
        if self.config.enable_cache {
            let cache_key = format!("verb:{verb}");
            self.cache.insert(cache_key, analysis.clone());
        }

        let processing_time = start_time.elapsed().as_micros() as u64;
        self.update_performance_metrics(processing_time);

        debug!(
            "VerbNet analysis for '{}': {} classes found, confidence: {:.2}",
            verb,
            analysis.verb_classes.len(),
            confidence
        );

        Ok(SemanticResult::new(
            analysis,
            confidence,
            false,
            processing_time,
        ))
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
    fn calculate_confidence(&self, classes: &[VerbClass]) -> f32 {
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

    /// Update performance metrics
    fn update_performance_metrics(&mut self, processing_time_us: u64) {
        // Update running average of query time
        let query_count = self.stats.total_queries as f64;
        self.stats.avg_query_time_us = ((self.stats.avg_query_time_us * (query_count - 1.0))
            + processing_time_us as f64)
            / query_count;

        self.performance_metrics.record_query(processing_time_us);
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

impl DataLoader for VerbNetEngine {
    fn load_from_directory<P: AsRef<Path>>(&mut self, path: P) -> EngineResult<()> {
        let path = path.as_ref();
        info!("Loading VerbNet data from: {}", path.display());

        let parser = XmlParser::new();
        let verb_classes = parser.parse_directory::<VerbClass>(path)?;

        self.verb_classes.clear();
        for verb_class in verb_classes {
            info!(
                "Loaded VerbNet class: {} ({})",
                verb_class.id, verb_class.class_name
            );
            self.verb_classes.insert(verb_class.id.clone(), verb_class);
        }

        self.stats.total_classes = self.verb_classes.len();
        self.build_verb_index();

        info!(
            "VerbNet data loading complete: {} classes, {} verbs",
            self.stats.total_classes, self.stats.total_verbs
        );

        Ok(())
    }

    fn load_test_data(&mut self) -> EngineResult<()> {
        // For now, just indicate that test data isn't implemented
        Err(EngineError::data_load(
            "Test data loading not implemented".to_string(),
        ))
    }

    fn reload(&mut self) -> EngineResult<()> {
        // For now, just clear and indicate reload needs a path
        self.verb_classes.clear();
        self.verb_index.clear();
        Err(EngineError::data_load(
            "Reload requires a data path".to_string(),
        ))
    }

    fn data_info(&self) -> DataInfo {
        DataInfo::new(self.config.data_path.clone(), self.verb_classes.len())
    }
}

impl SemanticEngine for VerbNetEngine {
    type Input = String;
    type Output = VerbNetAnalysis;
    type Config = VerbNetConfig;

    fn analyze(&self, input: &Self::Input) -> EngineResult<SemanticResult<Self::Output>> {
        // Since analyze_verb requires &mut self, we need to work around this
        // For now, we'll create a minimal implementation
        let verb = input;
        let matching_classes = if let Some(class_ids) = self.verb_index.get(verb) {
            class_ids
                .iter()
                .filter_map(|id| self.verb_classes.get(id))
                .cloned()
                .collect()
        } else {
            Vec::new()
        };

        let confidence = if matching_classes.is_empty() {
            0.1
        } else {
            0.8
        };
        let analysis = VerbNetAnalysis::new(verb.clone(), matching_classes, confidence);

        Ok(SemanticResult::new(analysis, confidence, false, 0))
    }

    fn name(&self) -> &'static str {
        "VerbNet"
    }

    fn version(&self) -> &'static str {
        "3.4"
    }

    fn is_initialized(&self) -> bool {
        !self.verb_classes.is_empty()
    }

    fn config(&self) -> &Self::Config {
        &self.config
    }
}

impl CachedEngine for VerbNetEngine {
    fn cache_stats(&self) -> canopy_engine::CacheStats {
        self.cache.stats()
    }

    fn clear_cache(&self) {
        // Note: The trait requires &self, not &mut self, so we can't actually clear
        // This is a limitation of the current trait design
    }

    fn set_cache_capacity(&mut self, capacity: usize) {
        self.config.cache_capacity = capacity;
        // Note: EngineCache doesn't have set_capacity method, would need to recreate
    }
}

impl StatisticsProvider for VerbNetEngine {
    fn statistics(&self) -> EngineStats {
        self.engine_stats.clone()
    }

    fn performance_metrics(&self) -> PerformanceMetrics {
        self.performance_metrics.clone()
    }
}

impl Default for VerbNetEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

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
    fn test_verbnet_engine_creation() {
        let engine = VerbNetEngine::new();
        assert_eq!(engine.stats.total_classes, 0);
        assert_eq!(engine.stats.total_verbs, 0);
        assert!(!engine.is_loaded());
    }

    #[test]
    fn test_load_verbnet_data() {
        let temp_dir = tempdir().unwrap();
        let xml_path = temp_dir.path().join("give-13.1.xml");
        fs::write(&xml_path, create_test_verbnet_xml()).unwrap();

        let mut engine = VerbNetEngine::new();
        engine.load_from_directory(temp_dir.path()).unwrap();

        assert!(engine.is_loaded());
        assert_eq!(engine.stats.total_classes, 1);
        assert_eq!(engine.stats.total_verbs, 2); // "give" and "hand"
    }

    #[test]
    fn test_verb_analysis() {
        let temp_dir = tempdir().unwrap();
        let xml_path = temp_dir.path().join("give-13.1.xml");
        fs::write(&xml_path, create_test_verbnet_xml()).unwrap();

        let mut engine = VerbNetEngine::new();
        engine.load_from_directory(temp_dir.path()).unwrap();

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

        let mut engine = VerbNetEngine::new();
        engine.load_from_directory(temp_dir.path()).unwrap();

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

        let mut engine = VerbNetEngine::new();
        engine.load_from_directory(temp_dir.path()).unwrap();

        // First query - should miss cache
        let result1 = engine.analyze_verb("give").unwrap();
        assert!(!result1.from_cache);
        assert_eq!(engine.stats.cache_misses, 1);

        // Second query - should hit cache
        let result2 = engine.analyze_verb("give").unwrap();
        assert!(result2.from_cache);
        assert_eq!(engine.stats.cache_hits, 1);
    }

    #[test]
    fn test_confidence_calculation() {
        let engine = VerbNetEngine::new();

        // Test empty classes
        assert_eq!(engine.calculate_confidence(&[]), 0.0);

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
        let confidence = engine.calculate_confidence(&single_class);
        assert!(confidence > 0.8);
    }
}
