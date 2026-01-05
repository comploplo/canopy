//! `FrameNet` semantic engine implementation
//!
//! This module provides the main `FrameNet` engine that implements canopy-engine traits
//! for semantic analysis using `FrameNet` frames, frame elements, and lexical units.
//!
//! ## Performance: Binary Caching
//!
//! `FrameNet` XML parsing takes ~50 seconds. To optimize this, the engine uses binary caching:
//! - First load: Parse XML (~50s), save to `data/cache/framenet.bin`
//! - Subsequent loads: Load from binary cache (~50ms)

use super::types::{Frame, FrameNetAnalysis, FrameNetConfig, FrameNetStats, LexicalUnit};
use crate::engine::{
    count_to_f32, i32_count_to_f32, BaseEngine, CacheKeyFormat, CacheableData, CachedEngine,
    DataInfo, DataLoader, EngineConfigurable, EngineCore, EngineError, EngineResult, EngineStats,
    PerformanceMetrics, SemanticEngine, SemanticResult, StatisticsProvider, XmlParser,
};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::Path;
use tracing::{debug, info, warn};

/// Cached `FrameNet` data - serializable subset of engine state
#[derive(Debug, Clone, Serialize, Deserialize)]
struct FrameNetData {
    frames: IndexMap<String, Frame>,
    lexical_units: IndexMap<String, LexicalUnit>,
}

impl CacheableData for FrameNetData {
    fn cache_filename() -> &'static str {
        "framenet.bin"
    }

    fn engine_name() -> &'static str {
        "FrameNet"
    }
}

/// Input type for `FrameNet` analysis
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameNetInput {
    pub text: String,
}

impl Hash for FrameNetInput {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.text.hash(state);
    }
}

/// `FrameNet` semantic analysis engine
#[derive(Debug)]
pub struct FrameNetEngine {
    /// Base engine handling cache, stats, and metrics
    base_engine: BaseEngine<FrameNetInput, FrameNetAnalysis>,
    /// `FrameNet` frames loaded from XML
    frames: IndexMap<String, Frame>,
    /// Lexical units loaded from XML
    lexical_units: IndexMap<String, LexicalUnit>,
    /// Frame name to frame ID mapping
    frame_name_index: HashMap<String, String>,
    /// Lexical unit name to LU mapping
    lu_name_index: HashMap<String, Vec<String>>,
    /// FrameNet-specific configuration
    framenet_config: FrameNetConfig,
    /// FrameNet-specific statistics
    stats: FrameNetStats,
}

impl FrameNetEngine {
    /// Create a new `FrameNet` engine with default configuration
    ///
    /// # Errors
    /// Returns an error if `FrameNet` data path does not exist or data cannot be loaded.
    pub fn new() -> EngineResult<Self> {
        Self::with_config(FrameNetConfig::default())
    }

    /// Create a new `FrameNet` engine with custom configuration
    ///
    /// # Errors
    /// Returns an error if `FrameNet` data path does not exist or data cannot be loaded.
    pub fn with_config(framenet_config: FrameNetConfig) -> EngineResult<Self> {
        // Convert FrameNetConfig to EngineConfig using trait
        let engine_config = framenet_config.to_engine_config();

        // Check if data paths exist - if not, fail fast (don't use cache for invalid paths)
        let frames_path_str = &framenet_config.frames_path;
        let frames_path = Path::new(frames_path_str);
        if !frames_path.exists() {
            return Err(EngineError::data_load(format!(
                "FrameNet frames path does not exist: {}",
                framenet_config.frames_path
            )));
        }

        // Only use binary cache for the default FrameNet data path (not for test data)
        let is_default_path = frames_path_str.contains("framenet");

        // Helper to load from XML
        let load_from_xml = |should_cache: bool| -> EngineResult<(
            IndexMap<String, Frame>,
            IndexMap<String, LexicalUnit>,
        )> {
            info!("Parsing FrameNet XML files...");
            let start = std::time::Instant::now();

            let mut temp_engine = Self {
                base_engine: BaseEngine::new(engine_config.clone(), "FrameNet".to_string()),
                frames: IndexMap::new(),
                lexical_units: IndexMap::new(),
                frame_name_index: HashMap::new(),
                lu_name_index: HashMap::new(),
                framenet_config: framenet_config.clone(),
                stats: FrameNetStats::default(),
            };

            // Parse XML files
            let frames_path = framenet_config.frames_path.clone();
            let lexical_units_path = framenet_config.lexical_units_path.clone();
            temp_engine.load_frames(&frames_path)?;
            temp_engine.load_lexical_units(&lexical_units_path)?;

            info!(
                "Parsed FrameNet XML in {:.2}s",
                start.elapsed().as_secs_f64()
            );

            // Save to cache only for default path
            if should_cache {
                let data = FrameNetData {
                    frames: temp_engine.frames.clone(),
                    lexical_units: temp_engine.lexical_units.clone(),
                };
                if let Err(e) = data.save_to_cache() {
                    warn!("Failed to save FrameNet cache: {}", e);
                }
            }

            Ok((temp_engine.frames, temp_engine.lexical_units))
        };

        let (frames, lexical_units) = if is_default_path {
            // Try loading from binary cache first (fast path: ~50ms)
            if let Some(cached) = FrameNetData::load_from_cache() {
                info!(
                    "Using cached FrameNet data ({} frames, {} LUs)",
                    cached.frames.len(),
                    cached.lexical_units.len()
                );
                (cached.frames, cached.lexical_units)
            } else {
                // Cache miss: parse XML (slow path: ~50s), then cache for next time
                load_from_xml(true)?
            }
        } else {
            // Non-default path (e.g., test data): don't use or save to cache
            load_from_xml(false)?
        };

        let mut engine = Self {
            base_engine: BaseEngine::new(engine_config, "FrameNet".to_string()),
            frames,
            lexical_units,
            frame_name_index: HashMap::new(),
            lu_name_index: HashMap::new(),
            framenet_config,
            stats: FrameNetStats::default(),
        };

        // Build indices for fast lookup
        engine.build_indices();

        // Update stats
        engine.stats.total_frames = engine.frames.len();
        engine.stats.total_lexical_units = engine.lexical_units.len();
        engine.stats.total_frame_elements =
            engine.frames.values().map(|f| f.frame_elements.len()).sum();

        Ok(engine)
    }

    /// Analyze input text and return matching frames and lexical units
    ///
    /// # Errors
    /// Returns an error if analysis fails.
    pub fn analyze_text(&self, text: &str) -> EngineResult<SemanticResult<FrameNetAnalysis>> {
        let input = FrameNetInput {
            text: text.to_string(),
        };
        self.base_engine.analyze(&input, self)
    }

    /// Find lexical units that match the input text
    /// Uses O(1) hash lookups instead of O(n) scanning
    fn find_lexical_units(&self, text: &str) -> Vec<LexicalUnit> {
        let mut matching_lus = Vec::new();
        let mut seen_ids = std::collections::HashSet::new();

        // Helper to add LUs without duplicates
        let mut add_lus = |lu_ids: &[String]| {
            for lu_id in lu_ids {
                if seen_ids.insert(lu_id.clone()) {
                    if let Some(lu) = self.lexical_units.get(lu_id) {
                        matching_lus.push(lu.clone());
                    }
                }
            }
        };

        // Direct lookup by the full text (lowercased) - O(1)
        let text_lower = text.to_lowercase();
        if let Some(lu_ids) = self.lu_name_index.get(&text_lower) {
            add_lus(lu_ids);
        }

        // Word-based lookups - O(words) instead of O(words * lexical_units)
        for word in text.split_whitespace() {
            let word_lower = word.to_lowercase();
            // Direct hash lookup for the word
            if let Some(lu_ids) = self.lu_name_index.get(&word_lower) {
                add_lus(lu_ids);
            }
        }

        matching_lus
    }

    /// Get frames for a list of lexical units
    fn get_frames_for_lexical_units(&self, lexical_units: &[LexicalUnit]) -> Vec<Frame> {
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

        frames
    }

    /// Calculate confidence score based on matching frames and lexical units
    fn calculate_framenet_confidence(frames: &[Frame], lexical_units: &[LexicalUnit]) -> f32 {
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
                let annotation_score = (i32_count_to_f32(lu.total_annotated) * 0.01).min(0.1);
                // Finished status is better than others
                let status_score = if lu.status.contains("Finished") {
                    0.05
                } else {
                    0.0
                };
                annotation_score + status_score
            })
            .sum::<f32>()
            / count_to_f32(lexical_units.len().max(1));

        (base_confidence + lu_quality_bonus).min(0.98)
    }

    /// Build indices for fast lookup
    /// All keys are lowercased for case-insensitive O(1) lookups
    fn build_indices(&mut self) {
        // Build frame name index
        self.frame_name_index.clear();
        for (frame_id, frame) in &self.frames {
            self.frame_name_index
                .insert(frame.name.clone(), frame_id.clone());
        }

        // Build lexical unit name index with lowercase keys for O(1) case-insensitive lookup
        self.lu_name_index.clear();
        for (lu_id, lu) in &self.lexical_units {
            // Index by full name (lowercased), e.g., "give.v"
            self.lu_name_index
                .entry(lu.name.to_lowercase())
                .or_default()
                .push(lu_id.clone());

            // Also index by the base word (without POS tag), e.g., "give"
            if let Some(base_name) = lu.name.split('.').next() {
                let base_lower = base_name.to_lowercase();
                if base_lower != lu.name.to_lowercase() {
                    self.lu_name_index
                        .entry(base_lower)
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
            "Built FrameNet indices: {} frames, {} lexical units, {} frame elements, {} index entries",
            self.stats.total_frames,
            self.stats.total_lexical_units,
            self.stats.total_frame_elements,
            self.lu_name_index.len()
        );
    }

    /// Get frame by ID
    #[must_use]
    pub fn get_frame(&self, frame_id: &str) -> Option<&Frame> {
        self.frames.get(frame_id)
    }

    /// Get frame by name
    #[must_use]
    pub fn get_frame_by_name(&self, frame_name: &str) -> Option<&Frame> {
        self.frame_name_index
            .get(frame_name)
            .and_then(|id| self.frames.get(id))
    }

    /// Get lexical unit by ID
    #[must_use]
    pub fn get_lexical_unit(&self, lu_id: &str) -> Option<&LexicalUnit> {
        self.lexical_units.get(lu_id)
    }

    /// Get all loaded frames
    #[must_use]
    pub fn get_all_frames(&self) -> Vec<&Frame> {
        self.frames.values().collect()
    }

    /// Get all loaded lexical units
    #[must_use]
    pub fn get_all_lexical_units(&self) -> Vec<&LexicalUnit> {
        self.lexical_units.values().collect()
    }

    /// Search frames by pattern
    #[must_use]
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
    #[must_use]
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
    #[must_use]
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
                self.framenet_config.frames_path, self.framenet_config.lexical_units_path
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
    // Backward compatibility methods for BaseEngine integration
    #[must_use]
    pub fn config(&self) -> &FrameNetConfig {
        &self.framenet_config
    }

    #[must_use]
    pub fn statistics(&self) -> &FrameNetStats {
        &self.stats
    }

    #[must_use]
    pub fn performance_metrics(&self) -> PerformanceMetrics {
        self.base_engine.get_performance_metrics()
    }

    #[must_use]
    pub fn cache_stats(&self) -> crate::engine::CacheStats {
        self.base_engine.cache_stats()
    }

    /// Clear the analysis cache
    ///
    /// # Errors
    /// This function currently cannot fail but returns `Result` for API consistency.
    pub fn clear_cache(&mut self) -> EngineResult<()> {
        self.base_engine.clear_cache();
        Ok(())
    }
}

// EngineCore trait implementation for BaseEngine integration
impl EngineCore<FrameNetInput, FrameNetAnalysis> for FrameNetEngine {
    fn perform_analysis(&self, input: &FrameNetInput) -> EngineResult<FrameNetAnalysis> {
        // Find matching lexical units and frames
        let matching_lus = self.find_lexical_units(&input.text);
        let matching_frames = self.get_frames_for_lexical_units(&matching_lus);

        if matching_frames.is_empty() && matching_lus.is_empty() {
            debug!(
                "No FrameNet frames or lexical units found for text: {}",
                input.text
            );
            return Ok(FrameNetAnalysis::new(input.text.clone(), Vec::new(), 0.0));
        }

        // Calculate confidence based on matches
        let confidence = Self::calculate_framenet_confidence(&matching_frames, &matching_lus);

        // Create analysis result
        let mut analysis = FrameNetAnalysis::new(input.text.clone(), matching_frames, confidence);
        analysis.lexical_units = matching_lus;

        debug!(
            "FrameNet analysis for '{}': {} frames found, confidence: {:.2}",
            input.text,
            analysis.frames.len(),
            confidence
        );

        Ok(analysis)
    }

    fn calculate_confidence(&self, _input: &FrameNetInput, output: &FrameNetAnalysis) -> f32 {
        output.confidence
    }

    fn generate_cache_key(&self, input: &FrameNetInput) -> String {
        CacheKeyFormat::Typed("framenet".to_string(), input.text.clone()).to_string()
    }

    fn engine_name(&self) -> &'static str {
        "FrameNet"
    }

    fn engine_version(&self) -> &'static str {
        "1.7"
    }

    fn is_initialized(&self) -> bool {
        !self.frames.is_empty() || !self.lexical_units.is_empty()
    }
}

impl SemanticEngine for FrameNetEngine {
    type Input = String;
    type Output = FrameNetAnalysis;
    type Config = FrameNetConfig;

    fn analyze(&self, input: &Self::Input) -> EngineResult<SemanticResult<Self::Output>> {
        // Use BaseEngine for analysis
        let framenet_input = FrameNetInput {
            text: input.clone(),
        };
        self.base_engine.analyze(&framenet_input, self)
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
        &self.framenet_config
    }
}

impl CachedEngine for FrameNetEngine {
    fn cache_stats(&self) -> crate::engine::CacheStats {
        self.base_engine.cache_stats()
    }

    fn clear_cache(&self) {
        // Note: trait requires &self, not &mut self, but BaseEngine uses Arc<Mutex>
        self.base_engine.clear_cache();
    }

    fn set_cache_capacity(&mut self, capacity: usize) {
        self.framenet_config.cache_capacity = capacity;
    }
}

impl StatisticsProvider for FrameNetEngine {
    fn statistics(&self) -> EngineStats {
        self.base_engine.get_stats()
    }

    fn performance_metrics(&self) -> PerformanceMetrics {
        self.base_engine.get_performance_metrics()
    }
}

impl Default for FrameNetEngine {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            panic!("FrameNet engine requires real data - cannot create default instance without FrameNet files")
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use once_cell::sync::OnceCell;
    use std::fs;
    use std::sync::Mutex;
    use tempfile::tempdir;

    // Shared engine singleton - loaded once per test binary
    static SHARED_ENGINE: OnceCell<Mutex<FrameNetEngine>> = OnceCell::new();

    /// Check if `FrameNet` data is available
    fn framenet_available() -> bool {
        crate::paths::data_path("data/framenet").exists()
    }

    /// Get shared `FrameNet` engine, or None if data unavailable
    fn shared_engine() -> Option<&'static Mutex<FrameNetEngine>> {
        if !framenet_available() {
            return None;
        }
        Some(SHARED_ENGINE.get_or_init(|| {
            eprintln!("🔧 Loading shared FrameNet engine (one-time)...");
            Mutex::new(FrameNetEngine::new().expect("FrameNet data required"))
        }))
    }

    /// Execute a closure with the shared engine
    fn with_engine<F, R>(f: F) -> Option<R>
    where
        F: FnOnce(&mut FrameNetEngine) -> R,
    {
        let engine_mutex = shared_engine()?;
        let mut engine = engine_mutex.lock().unwrap();
        Some(f(&mut engine))
    }

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
        // Use shared engine (loaded once, reused across tests)
        let Some(()) = with_engine(|engine| {
            // With data loaded, stats should reflect loaded content
            assert!(engine.is_loaded());
        }) else {
            eprintln!("Skipping test: FrameNet data not available");
            return;
        };
    }

    #[test]
    fn test_load_framenet_data() {
        let Some(()) = with_engine(|engine| {
            // Load additional test data from temp directory
            let temp_dir = tempdir().unwrap();
            let frame_dir = temp_dir.path().join("frame");
            let lu_dir = temp_dir.path().join("lu");
            fs::create_dir(&frame_dir).unwrap();
            fs::create_dir(&lu_dir).unwrap();

            fs::write(frame_dir.join("Giving.xml"), create_test_frame_xml()).unwrap();
            fs::write(lu_dir.join("give.xml"), create_test_lu_xml()).unwrap();

            engine.load_from_directory(temp_dir.path()).unwrap();

            assert!(engine.is_loaded());
        }) else {
            eprintln!("Skipping test: FrameNet data not available");
            return;
        };
    }

    #[test]
    fn test_frame_analysis() {
        let Some(result) = with_engine(|engine| {
            let temp_dir = tempdir().unwrap();
            let frame_dir = temp_dir.path().join("frame");
            let lu_dir = temp_dir.path().join("lu");
            fs::create_dir(&frame_dir).unwrap();
            fs::create_dir(&lu_dir).unwrap();

            fs::write(frame_dir.join("Giving.xml"), create_test_frame_xml()).unwrap();
            fs::write(lu_dir.join("give.xml"), create_test_lu_xml()).unwrap();

            engine.load_from_directory(temp_dir.path()).unwrap();

            engine.analyze_text("give").unwrap()
        }) else {
            eprintln!("Skipping test: FrameNet data not available");
            return;
        };

        assert!(result.confidence > 0.5);
        assert_eq!(result.data.input, "give");
    }

    #[test]
    fn test_frame_search() {
        let Some(found_frames) = with_engine(|engine| {
            let temp_dir = tempdir().unwrap();
            let frame_dir = temp_dir.path().join("frame");
            fs::create_dir(&frame_dir).unwrap();
            fs::write(frame_dir.join("Giving.xml"), create_test_frame_xml()).unwrap();

            engine.load_from_directory(temp_dir.path()).unwrap();

            // Return count instead of references to avoid lifetime issues
            let frames = engine.search_frames("giving");
            !frames.is_empty()
        }) else {
            eprintln!("Skipping test: FrameNet data not available");
            return;
        };

        assert!(found_frames, "Should find at least one frame for 'giving'");
    }

    #[test]
    fn test_confidence_calculation() {
        let Some(engine_ref) = shared_engine() else {
            eprintln!("Skipping test: FrameNet data not available");
            return;
        };
        let engine = engine_ref.lock().unwrap();

        // Test empty inputs
        let empty_analysis = FrameNetAnalysis::new("test".to_string(), vec![], 0.0);
        let empty_input = FrameNetInput {
            text: "test".to_string(),
        };
        assert!(
            engine
                .calculate_confidence(&empty_input, &empty_analysis)
                .abs()
                < f32::EPSILON
        );

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

        let analysis = FrameNetAnalysis {
            input: "give".to_string(),
            frames: frames.clone(),
            lexical_units: lus.clone(),
            frame_element_assignments: vec![],
            confidence: 0.95,
        };
        let input = FrameNetInput {
            text: "give".to_string(),
        };
        let confidence = engine.calculate_confidence(&input, &analysis);
        assert!(confidence > 0.9);
    }
}
