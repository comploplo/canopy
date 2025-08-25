//! FrameNet semantic engine implementation
//!
//! This module provides the main FrameNet engine that implements canopy-engine traits
//! for semantic analysis using FrameNet frames, frame elements, and lexical units.

use crate::types::{Frame, FrameNetAnalysis, FrameNetConfig, FrameNetStats, LexicalUnit};
use canopy_engine::{
    traits::DataInfo, CachedEngine, DataLoader, EngineCache, EngineError, EngineResult,
    EngineStats, PerformanceMetrics, SemanticEngine, SemanticResult, StatisticsProvider, XmlParser,
};
use indexmap::IndexMap;
use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;
use tracing::{debug, info};

/// FrameNet semantic analysis engine
#[derive(Debug)]
pub struct FrameNetEngine {
    /// FrameNet frames loaded from XML
    frames: IndexMap<String, Frame>,
    /// Lexical units loaded from XML
    lexical_units: IndexMap<String, LexicalUnit>,
    /// Frame name to frame ID mapping
    frame_name_index: HashMap<String, String>,
    /// Lexical unit name to LU mapping
    lu_name_index: HashMap<String, Vec<String>>,
    /// Engine configuration
    config: FrameNetConfig,
    /// Result cache
    cache: EngineCache<String, FrameNetAnalysis>,
    /// Performance statistics
    stats: FrameNetStats,
    /// Performance metrics
    performance_metrics: PerformanceMetrics,
    /// Engine statistics
    engine_stats: EngineStats,
}

impl FrameNetEngine {
    /// Create a new FrameNet engine with default configuration
    pub fn new() -> Self {
        Self::with_config(FrameNetConfig::default())
    }

    /// Create a new FrameNet engine with custom configuration
    pub fn with_config(config: FrameNetConfig) -> Self {
        let cache = EngineCache::new(config.cache_capacity);

        Self {
            frames: IndexMap::new(),
            lexical_units: IndexMap::new(),
            frame_name_index: HashMap::new(),
            lu_name_index: HashMap::new(),
            config,
            cache,
            stats: FrameNetStats {
                total_frames: 0,
                total_lexical_units: 0,
                total_frame_elements: 0,
                total_queries: 0,
                cache_hits: 0,
                cache_misses: 0,
                avg_query_time_us: 0.0,
            },
            performance_metrics: PerformanceMetrics::new(),
            engine_stats: EngineStats::new("FrameNet".to_string()),
        }
    }

    /// Analyze input text and return matching frames and lexical units
    pub fn analyze_text(&mut self, text: &str) -> EngineResult<SemanticResult<FrameNetAnalysis>> {
        let start_time = Instant::now();
        self.stats.total_queries += 1;

        // Check cache first
        if self.config.enable_cache {
            let cache_key = format!("text:{text}");
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

        // Find matching lexical units and frames
        let matching_lus = self.find_lexical_units(text)?;
        let matching_frames = self.get_frames_for_lexical_units(&matching_lus)?;

        // Calculate confidence based on matches
        let confidence = self.calculate_confidence(&matching_frames, &matching_lus);

        // Create analysis result
        let analysis = FrameNetAnalysis::new(text.to_string(), matching_frames, confidence);

        // Cache the result
        if self.config.enable_cache {
            let cache_key = format!("text:{text}");
            self.cache.insert(cache_key, analysis.clone());
        }

        let processing_time = start_time.elapsed().as_micros() as u64;
        self.update_performance_metrics(processing_time);

        debug!(
            "FrameNet analysis for '{}': {} frames found, confidence: {:.2}",
            text,
            analysis.frames.len(),
            confidence
        );

        Ok(SemanticResult::new(
            analysis,
            confidence,
            false,
            processing_time,
        ))
    }

    /// Find lexical units that match the input text
    fn find_lexical_units(&self, text: &str) -> EngineResult<Vec<LexicalUnit>> {
        let mut matching_lus = Vec::new();
        let text_lower = text.to_lowercase();

        // Direct lexical unit lookup
        for (lu_name, lu_ids) in &self.lu_name_index {
            if text_lower.contains(&lu_name.to_lowercase()) {
                for lu_id in lu_ids {
                    if let Some(lu) = self.lexical_units.get(lu_id) {
                        matching_lus.push(lu.clone());
                    }
                }
            }
        }

        // Word-based matching for multi-word expressions
        let words: Vec<&str> = text.split_whitespace().collect();
        for word in words {
            let word_lower = word.to_lowercase();
            for (lu_name, lu_ids) in &self.lu_name_index {
                // Extract base word from lexical unit name (e.g., "give.v" -> "give")
                let base_word = lu_name.split('.').next().unwrap_or(lu_name);
                if word_lower == base_word.to_lowercase() {
                    for lu_id in lu_ids {
                        if let Some(lu) = self.lexical_units.get(lu_id) {
                            // Avoid duplicates
                            if !matching_lus.iter().any(|existing| existing.id == lu.id) {
                                matching_lus.push(lu.clone());
                            }
                        }
                    }
                }
            }
        }

        Ok(matching_lus)
    }

    /// Get frames for a list of lexical units
    fn get_frames_for_lexical_units(
        &self,
        lexical_units: &[LexicalUnit],
    ) -> EngineResult<Vec<Frame>> {
        let mut frames = Vec::new();
        let mut frame_ids = std::collections::HashSet::new();

        for lu in lexical_units {
            if !frame_ids.contains(&lu.frame_id) {
                if let Some(frame) = self.frames.get(&lu.frame_id) {
                    frames.push(frame.clone());
                    frame_ids.insert(lu.frame_id.clone());
                }
            }
        }

        Ok(frames)
    }

    /// Calculate confidence score based on matching frames and lexical units
    fn calculate_confidence(&self, frames: &[Frame], lexical_units: &[LexicalUnit]) -> f32 {
        if frames.is_empty() && lexical_units.is_empty() {
            return 0.0;
        }

        let base_confidence = match (frames.len(), lexical_units.len()) {
            (0, 0) => 0.0,
            (1, 1) => 0.95,          // Perfect single match
            (1, n) if n > 1 => 0.85, // One frame, multiple LUs
            (n, 1) if n > 1 => 0.80, // Multiple frames, one LU (unusual)
            (_, _) => 0.75,          // Multiple matches
        };

        // Boost confidence for high-quality lexical units
        let lu_quality_bonus = lexical_units
            .iter()
            .map(|lu| {
                // Higher total annotations suggest higher quality
                let annotation_score = (lu.total_annotated as f32 * 0.01).min(0.1);
                // Finished status is better than others
                let status_score = if lu.status.contains("Finished") {
                    0.05
                } else {
                    0.0
                };
                annotation_score + status_score
            })
            .sum::<f32>()
            / lexical_units.len().max(1) as f32;

        (base_confidence + lu_quality_bonus).min(0.98)
    }

    /// Build indices for fast lookup
    fn build_indices(&mut self) {
        // Build frame name index
        self.frame_name_index.clear();
        for (frame_id, frame) in &self.frames {
            self.frame_name_index
                .insert(frame.name.clone(), frame_id.clone());
        }

        // Build lexical unit name index
        self.lu_name_index.clear();
        for (lu_id, lu) in &self.lexical_units {
            self.lu_name_index
                .entry(lu.name.clone())
                .or_default()
                .push(lu_id.clone());

            // Also index by the base word (without POS tag)
            if let Some(base_name) = lu.name.split('.').next() {
                if base_name != lu.name {
                    self.lu_name_index
                        .entry(base_name.to_string())
                        .or_default()
                        .push(lu_id.clone());
                }
            }
        }

        // Update statistics
        self.stats.total_frames = self.frames.len();
        self.stats.total_lexical_units = self.lexical_units.len();
        self.stats.total_frame_elements =
            self.frames.values().map(|f| f.frame_elements.len()).sum();

        info!(
            "Built FrameNet indices: {} frames, {} lexical units, {} frame elements",
            self.stats.total_frames,
            self.stats.total_lexical_units,
            self.stats.total_frame_elements
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

    /// Get frame by ID
    pub fn get_frame(&self, frame_id: &str) -> Option<&Frame> {
        self.frames.get(frame_id)
    }

    /// Get frame by name
    pub fn get_frame_by_name(&self, frame_name: &str) -> Option<&Frame> {
        self.frame_name_index
            .get(frame_name)
            .and_then(|id| self.frames.get(id))
    }

    /// Get lexical unit by ID
    pub fn get_lexical_unit(&self, lu_id: &str) -> Option<&LexicalUnit> {
        self.lexical_units.get(lu_id)
    }

    /// Get all loaded frames
    pub fn get_all_frames(&self) -> Vec<&Frame> {
        self.frames.values().collect()
    }

    /// Get all loaded lexical units
    pub fn get_all_lexical_units(&self) -> Vec<&LexicalUnit> {
        self.lexical_units.values().collect()
    }

    /// Search frames by pattern
    pub fn search_frames(&self, pattern: &str) -> Vec<&Frame> {
        let pattern_lower = pattern.to_lowercase();
        self.frames
            .values()
            .filter(|f| {
                f.name.to_lowercase().contains(&pattern_lower)
                    || f.definition.to_lowercase().contains(&pattern_lower)
            })
            .collect()
    }

    /// Search lexical units by pattern
    pub fn search_lexical_units(&self, pattern: &str) -> Vec<&LexicalUnit> {
        let pattern_lower = pattern.to_lowercase();
        self.lexical_units
            .values()
            .filter(|lu| {
                lu.name.to_lowercase().contains(&pattern_lower)
                    || lu.frame_name.to_lowercase().contains(&pattern_lower)
            })
            .collect()
    }

    /// Check if the engine has loaded data
    pub fn is_loaded(&self) -> bool {
        !self.frames.is_empty() || !self.lexical_units.is_empty()
    }
}

impl DataLoader for FrameNetEngine {
    fn load_from_directory<P: AsRef<Path>>(&mut self, path: P) -> EngineResult<()> {
        let path = path.as_ref();
        info!("Loading FrameNet data from: {}", path.display());

        // Load frames
        let frames_path = path.join("frame");
        if frames_path.exists() {
            self.load_frames(&frames_path)?;
        }

        // Load lexical units
        let lu_path = path.join("lu");
        if lu_path.exists() {
            self.load_lexical_units(&lu_path)?;
        }

        // If neither subdirectory exists, assume path contains mixed frame/LU files
        if !frames_path.exists() && !lu_path.exists() {
            self.load_mixed_directory(path)?;
        }

        self.build_indices();

        info!(
            "FrameNet data loading complete: {} frames, {} lexical units",
            self.stats.total_frames, self.stats.total_lexical_units
        );

        Ok(())
    }

    fn load_test_data(&mut self) -> EngineResult<()> {
        Err(EngineError::data_load(
            "Test data loading not implemented".to_string(),
        ))
    }

    fn reload(&mut self) -> EngineResult<()> {
        self.frames.clear();
        self.lexical_units.clear();
        self.frame_name_index.clear();
        self.lu_name_index.clear();
        Err(EngineError::data_load(
            "Reload requires a data path".to_string(),
        ))
    }

    fn data_info(&self) -> DataInfo {
        DataInfo::new(
            format!(
                "frames: {}, lu: {}",
                self.config.frames_path, self.config.lexical_units_path
            ),
            self.frames.len() + self.lexical_units.len(),
        )
    }
}

impl FrameNetEngine {
    /// Load frames from directory (using parallel processing)
    fn load_frames<P: AsRef<Path>>(&mut self, path: P) -> EngineResult<()> {
        let parser = XmlParser::new();

        // Use parallel parsing if available, fallback to sequential
        #[cfg(feature = "parallel")]
        let frames = parser.parse_directory_parallel::<Frame>(path.as_ref())?;

        #[cfg(not(feature = "parallel"))]
        let frames = parser.parse_directory::<Frame>(path.as_ref())?;

        info!("Loaded {} FrameNet frames", frames.len());

        for frame in frames {
            debug!("Loaded FrameNet frame: {} ({})", frame.name, frame.id);
            self.frames.insert(frame.id.clone(), frame);
        }

        Ok(())
    }

    /// Load lexical units from directory (using parallel processing)
    fn load_lexical_units<P: AsRef<Path>>(&mut self, path: P) -> EngineResult<()> {
        let parser = XmlParser::new();

        // Use parallel parsing if available, fallback to sequential
        #[cfg(feature = "parallel")]
        let lexical_units = parser.parse_directory_parallel::<LexicalUnit>(path.as_ref())?;

        #[cfg(not(feature = "parallel"))]
        let lexical_units = parser.parse_directory::<LexicalUnit>(path.as_ref())?;

        info!("Loaded {} FrameNet lexical units", lexical_units.len());

        for lu in lexical_units {
            debug!("Loaded FrameNet lexical unit: {} ({})", lu.name, lu.id);
            self.lexical_units.insert(lu.id.clone(), lu);
        }

        Ok(())
    }

    /// Load from directory containing mixed frame and LU files
    fn load_mixed_directory<P: AsRef<Path>>(&mut self, path: P) -> EngineResult<()> {
        let path = path.as_ref();

        // Try to load as frames first, then as lexical units
        let entries = std::fs::read_dir(path).map_err(|e| {
            EngineError::data_load(format!(
                "Failed to read directory {}: {}",
                path.display(),
                e
            ))
        })?;

        let parser = XmlParser::new();

        for entry in entries {
            let entry = entry.map_err(|e| {
                EngineError::data_load(format!("Failed to read directory entry: {e}"))
            })?;

            let file_path = entry.path();

            if file_path.extension().and_then(|s| s.to_str()) == Some("xml") {
                // Try frame first
                if let Ok(frame) = parser.parse_file::<Frame>(&file_path) {
                    info!("Loaded FrameNet frame: {} ({})", frame.name, frame.id);
                    self.frames.insert(frame.id.clone(), frame);
                    continue;
                }
                // Fall through to try LU

                // Try lexical unit
                match parser.parse_file::<LexicalUnit>(&file_path) {
                    Ok(lu) => {
                        info!("Loaded FrameNet lexical unit: {} ({})", lu.name, lu.id);
                        self.lexical_units.insert(lu.id.clone(), lu);
                    }
                    Err(e) => {
                        debug!(
                            "Failed to parse {} as frame or LU: {}",
                            file_path.display(),
                            e
                        );
                    }
                }
            }
        }

        Ok(())
    }
    /// Find lexical units that match the input text (trait-compatible version)
    fn find_lexical_units_for_trait(&self, text: &str) -> EngineResult<Vec<LexicalUnit>> {
        let mut matching_lus = Vec::new();
        let text_lower = text.to_lowercase();

        // Direct lexical unit lookup
        for (lu_name, lu_ids) in &self.lu_name_index {
            if text_lower.contains(&lu_name.to_lowercase()) {
                for lu_id in lu_ids {
                    if let Some(lu) = self.lexical_units.get(lu_id) {
                        matching_lus.push(lu.clone());
                    }
                }
            }
        }

        // Word-based matching for multi-word expressions
        let words: Vec<&str> = text.split_whitespace().collect();
        for word in words {
            let word_lower = word.to_lowercase();
            for (lu_name, lu_ids) in &self.lu_name_index {
                // Extract base word from lexical unit name (e.g., "give.v" -> "give")
                let base_word = lu_name.split('.').next().unwrap_or(lu_name);
                if word_lower == base_word.to_lowercase() {
                    for lu_id in lu_ids {
                        if let Some(lu) = self.lexical_units.get(lu_id) {
                            // Avoid duplicates
                            if !matching_lus.iter().any(|existing| existing.id == lu.id) {
                                matching_lus.push(lu.clone());
                            }
                        }
                    }
                }
            }
        }

        Ok(matching_lus)
    }

    /// Get frames for lexical units (trait-compatible version)
    fn get_frames_for_lexical_units_for_trait(
        &self,
        lexical_units: &[LexicalUnit],
    ) -> EngineResult<Vec<Frame>> {
        let mut frames = Vec::new();
        let mut frame_ids = std::collections::HashSet::new();

        for lu in lexical_units {
            if !frame_ids.contains(&lu.frame_id) {
                if let Some(frame) = self.frames.get(&lu.frame_id) {
                    frames.push(frame.clone());
                    frame_ids.insert(lu.frame_id.clone());
                }
            }
        }

        Ok(frames)
    }
}

impl SemanticEngine for FrameNetEngine {
    type Input = String;
    type Output = FrameNetAnalysis;
    type Config = FrameNetConfig;

    fn analyze(&self, input: &Self::Input) -> EngineResult<SemanticResult<Self::Output>> {
        let start_time = Instant::now();

        // Use sophisticated matching logic (same as analyze_text)
        let matching_lus = self.find_lexical_units_for_trait(input)?;
        let matching_frames = self.get_frames_for_lexical_units_for_trait(&matching_lus)?;

        // Calculate confidence based on matches
        let confidence = self.calculate_confidence(&matching_frames, &matching_lus);

        // Create analysis result
        let analysis = FrameNetAnalysis::new(input.clone(), matching_frames, confidence);

        let processing_time = start_time.elapsed().as_micros() as u64;

        Ok(SemanticResult::new(
            analysis,
            confidence,
            false,
            processing_time,
        ))
    }

    fn name(&self) -> &'static str {
        "FrameNet"
    }

    fn version(&self) -> &'static str {
        "1.7"
    }

    fn is_initialized(&self) -> bool {
        !self.frames.is_empty() || !self.lexical_units.is_empty()
    }

    fn config(&self) -> &Self::Config {
        &self.config
    }
}

impl CachedEngine for FrameNetEngine {
    fn cache_stats(&self) -> canopy_engine::CacheStats {
        self.cache.stats()
    }

    fn clear_cache(&self) {
        // Note: trait requires &self, not &mut self
    }

    fn set_cache_capacity(&mut self, capacity: usize) {
        self.config.cache_capacity = capacity;
    }
}

impl StatisticsProvider for FrameNetEngine {
    fn statistics(&self) -> EngineStats {
        self.engine_stats.clone()
    }

    fn performance_metrics(&self) -> PerformanceMetrics {
        self.performance_metrics.clone()
    }
}

impl Default for FrameNetEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn create_test_frame_xml() -> &'static str {
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
        <frame ID="139" name="Giving">
            <definition>&lt;def-root&gt;A frame about giving&lt;/def-root&gt;</definition>
            <FE ID="1052" name="Donor" abbrev="Donor" coreType="Core">
                <definition>&lt;def-root&gt;The giver&lt;/def-root&gt;</definition>
            </FE>
            <FE ID="1053" name="Recipient" abbrev="Rec" coreType="Core">
                <definition>&lt;def-root&gt;The receiver&lt;/def-root&gt;</definition>
            </FE>
        </frame>"#
    }

    fn create_test_lu_xml() -> &'static str {
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
        <lexUnit ID="2477" name="give.v" POS="V" status="Finished_Initial" frame="Giving" frameID="139" totalAnnotated="25">
            <definition>To transfer possession</definition>
            <lexeme POS="V" name="give"/>
        </lexUnit>"#
    }

    #[test]
    fn test_framenet_engine_creation() {
        let engine = FrameNetEngine::new();
        assert_eq!(engine.stats.total_frames, 0);
        assert_eq!(engine.stats.total_lexical_units, 0);
        assert!(!engine.is_loaded());
    }

    #[test]
    fn test_load_framenet_data() {
        let temp_dir = tempdir().unwrap();
        let frame_dir = temp_dir.path().join("frame");
        let lu_dir = temp_dir.path().join("lu");
        fs::create_dir(&frame_dir).unwrap();
        fs::create_dir(&lu_dir).unwrap();

        fs::write(frame_dir.join("Giving.xml"), create_test_frame_xml()).unwrap();
        fs::write(lu_dir.join("give.xml"), create_test_lu_xml()).unwrap();

        let mut engine = FrameNetEngine::new();
        engine.load_from_directory(temp_dir.path()).unwrap();

        assert!(engine.is_loaded());
        assert_eq!(engine.stats.total_frames, 1);
        assert_eq!(engine.stats.total_lexical_units, 1);
        assert_eq!(engine.stats.total_frame_elements, 2);
    }

    #[test]
    fn test_frame_analysis() {
        let temp_dir = tempdir().unwrap();
        let frame_dir = temp_dir.path().join("frame");
        let lu_dir = temp_dir.path().join("lu");
        fs::create_dir(&frame_dir).unwrap();
        fs::create_dir(&lu_dir).unwrap();

        fs::write(frame_dir.join("Giving.xml"), create_test_frame_xml()).unwrap();
        fs::write(lu_dir.join("give.xml"), create_test_lu_xml()).unwrap();

        let mut engine = FrameNetEngine::new();
        engine.load_from_directory(temp_dir.path()).unwrap();

        let result = engine.analyze_text("give").unwrap();
        assert!(result.confidence > 0.5);
        assert_eq!(result.data.input, "give");
        assert_eq!(result.data.frames.len(), 1);
        assert_eq!(result.data.frames[0].name, "Giving");
    }

    #[test]
    fn test_frame_search() {
        let temp_dir = tempdir().unwrap();
        let frame_dir = temp_dir.path().join("frame");
        fs::create_dir(&frame_dir).unwrap();
        fs::write(frame_dir.join("Giving.xml"), create_test_frame_xml()).unwrap();

        let mut engine = FrameNetEngine::new();
        engine.load_from_directory(temp_dir.path()).unwrap();

        let frames = engine.search_frames("giving");
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].name, "Giving");
    }

    #[test]
    fn test_confidence_calculation() {
        let engine = FrameNetEngine::new();

        // Test empty inputs
        assert_eq!(engine.calculate_confidence(&[], &[]), 0.0);

        // Test single matches
        let frames = vec![Frame {
            id: "139".to_string(),
            name: "Giving".to_string(),
            created_by: None,
            created_date: None,
            definition: "test".to_string(),
            frame_elements: vec![],
            frame_relations: vec![],
            lexical_units: vec![],
        }];

        let lus = vec![LexicalUnit {
            id: "2477".to_string(),
            name: "give.v".to_string(),
            pos: "V".to_string(),
            status: "Finished_Initial".to_string(),
            frame_id: "139".to_string(),
            frame_name: "Giving".to_string(),
            total_annotated: 25,
            definition: "test".to_string(),
            lexemes: vec![],
            valences: vec![],
            subcategorization: vec![],
        }];

        let confidence = engine.calculate_confidence(&frames, &lus);
        assert!(confidence > 0.9);
    }
}
