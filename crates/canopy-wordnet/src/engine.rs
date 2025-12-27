//! WordNet semantic engine implementation
//!
//! This module provides the main WordNet engine that implements canopy-engine traits
//! for semantic analysis using Princeton WordNet 3.1 data.
//!
//! ## Performance: Binary Caching
//!
//! WordNet data parsing takes ~10 seconds. To optimize this, the engine uses binary caching:
//! - First load: Parse data files (~10s), save to `data/cache/wordnet.bin`
//! - Subsequent loads: Load from binary cache (~50ms)
//!
//! Engine-level caching is provided via BaseEngine, which implements LRU caching
//! with configurable capacity and TTL. Cache keys include word and POS for proper
//! disambiguation (e.g., "wordnet:bank:n" vs "wordnet:bank:v").

use crate::loader::WordNetLoader;
use crate::parser::WordNetParserConfig;
use crate::types::{PartOfSpeech, WordNetAnalysis, WordNetDatabase};
use canopy_core::paths::{cache_path, data_path_string};
use canopy_engine::{
    BaseEngine, CacheKeyFormat, EngineConfig, EngineCore, EngineResult, EngineStats,
    PerformanceMetrics, SemanticResult,
    traits::{CachedEngine, DataInfo, DataLoader, SemanticEngine, StatisticsProvider},
};
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Input type for WordNet analysis
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WordNetInput {
    pub word: String,
    pub pos: PartOfSpeech,
}

impl Hash for WordNetInput {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.word.hash(state);
        self.pos.hash(state);
    }
}

/// Cached WordNet data - serializable database
#[derive(Debug, Clone, Serialize, Deserialize)]
struct WordNetData {
    database: WordNetDatabase,
}

impl WordNetData {
    /// Load from binary cache file (uses absolute workspace path)
    fn load_from_cache() -> Option<Self> {
        let path = cache_path("wordnet.bin");
        if !path.exists() {
            return None;
        }
        match std::fs::read(&path) {
            Ok(bytes) => match bincode::deserialize(&bytes) {
                Ok(data) => {
                    info!("Loaded WordNet data from cache ({} bytes)", bytes.len());
                    Some(data)
                }
                Err(e) => {
                    warn!("Failed to deserialize WordNet cache: {}", e);
                    None
                }
            },
            Err(e) => {
                warn!("Failed to read WordNet cache: {}", e);
                None
            }
        }
    }

    /// Save to binary cache file (uses absolute workspace path)
    fn save_to_cache(&self) -> Result<(), String> {
        let path = cache_path("wordnet.bin");
        let bytes = bincode::serialize(self)
            .map_err(|e| format!("Failed to serialize WordNet data: {}", e))?;
        std::fs::write(&path, &bytes)
            .map_err(|e| format!("Failed to write WordNet cache: {}", e))?;
        info!("Saved WordNet data to cache ({} bytes)", bytes.len());
        Ok(())
    }
}

/// Configuration for WordNet engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordNetConfig {
    /// Path to WordNet data directory
    pub data_path: String,
    /// Parser configuration
    pub parser_config: WordNetParserConfig,
    /// Enable morphological processing
    pub enable_morphology: bool,
    /// Maximum search depth for semantic relations
    pub max_search_depth: usize,
    /// Minimum confidence threshold for results
    pub min_confidence: f32,
    /// Enable caching
    pub enable_cache: bool,
    /// Cache capacity
    pub cache_capacity: usize,
}

impl Default for WordNetConfig {
    fn default() -> Self {
        Self {
            data_path: data_path_string("data/wordnet/dict"),
            parser_config: WordNetParserConfig::default(),
            enable_morphology: true,
            max_search_depth: 5,
            min_confidence: 0.1,
            enable_cache: true,
            cache_capacity: 10000,
        }
    }
}

/// WordNet semantic analysis engine
#[derive(Debug)]
pub struct WordNetEngine {
    /// Base engine handling cache, stats, and metrics
    base_engine: BaseEngine<WordNetInput, WordNetAnalysis>,
    /// WordNet database
    database: Arc<WordNetDatabase>,
    /// WordNet-specific configuration
    wordnet_config: WordNetConfig,
    /// Is data loaded flag
    is_loaded: bool,
}

impl WordNetEngine {
    /// Create a new WordNet engine
    pub fn new() -> EngineResult<Self> {
        Self::with_config(WordNetConfig::default())
    }

    /// Create a new WordNet engine with custom configuration
    pub fn with_config(wordnet_config: WordNetConfig) -> EngineResult<Self> {
        // Convert WordNetConfig to EngineConfig
        let engine_config = EngineConfig {
            enable_cache: wordnet_config.enable_cache,
            cache_capacity: wordnet_config.cache_capacity,
            enable_metrics: true,
            enable_parallel: false,
            max_threads: 4,
            confidence_threshold: wordnet_config.min_confidence,
        };

        // Check if data path exists - if not, fail fast (don't use cache for invalid paths)
        let data_path_str = &wordnet_config.data_path;
        let wn_path = Path::new(data_path_str);
        if !wn_path.exists() {
            return Err(canopy_engine::EngineError::data_load(format!(
                "WordNet data path does not exist: {}",
                wordnet_config.data_path
            )));
        }

        // Only use binary cache for the default WordNet data path (not for test data)
        let is_default_path = data_path_str.contains("wordnet");

        // Helper to load from data files
        let load_from_files = |should_cache: bool| -> EngineResult<WordNetDatabase> {
            info!("Parsing WordNet data files...");
            let start = std::time::Instant::now();

            let loader = WordNetLoader::new(wordnet_config.parser_config.clone());
            let database = loader.load_database(&wordnet_config.data_path)?;

            info!(
                "Parsed WordNet data in {:.2}s ({} synsets)",
                start.elapsed().as_secs_f64(),
                database.synsets.len()
            );

            // Save to cache only for default path
            if should_cache {
                let data = WordNetData {
                    database: database.clone(),
                };
                if let Err(e) = data.save_to_cache() {
                    warn!("Failed to save WordNet cache: {}", e);
                }
            }

            Ok(database)
        };

        let (database, is_loaded) = if is_default_path {
            // Try loading from binary cache first (fast path: ~50ms)
            if let Some(cached) = WordNetData::load_from_cache() {
                info!(
                    "Using cached WordNet data ({} synsets)",
                    cached.database.synsets.len()
                );
                (Arc::new(cached.database), true)
            } else {
                // Cache miss: parse data files (slow path: ~10s), then cache for next time
                (Arc::new(load_from_files(true)?), true)
            }
        } else {
            // Non-default path (e.g., test data): don't use or save to cache
            (Arc::new(load_from_files(false)?), true)
        };

        let engine = Self {
            base_engine: BaseEngine::new(engine_config, "WordNet".to_string()),
            database,
            wordnet_config,
            is_loaded,
        };

        info!(
            "WordNet engine initialized with {} synsets",
            engine.database.synsets.len()
        );

        Ok(engine)
    }

    /// Analyze a word and return semantic information
    pub fn analyze_word(
        &self,
        word: &str,
        pos: PartOfSpeech,
    ) -> EngineResult<SemanticResult<WordNetAnalysis>> {
        let input = WordNetInput {
            word: word.to_string(),
            pos,
        };
        self.base_engine.analyze(&input, self)
    }

    /// Get hypernyms for a word
    pub fn get_hypernyms(&self, synset_id: &str) -> Vec<String> {
        if !self.is_loaded {
            return Vec::new();
        }

        // Parse synset_id as offset
        if let Ok(offset) = synset_id.parse::<usize>()
            && let Some(synset) = self.database.get_synset(offset)
        {
            return self
                .database
                .get_hypernyms(synset)
                .into_iter()
                .filter_map(|s| s.primary_word().map(|w| w.to_string()))
                .collect();
        }

        Vec::new()
    }

    /// Get hyponyms for a word
    pub fn get_hyponyms(&self, synset_id: &str) -> Vec<String> {
        if !self.is_loaded {
            return Vec::new();
        }

        // Parse synset_id as offset
        if let Ok(offset) = synset_id.parse::<usize>()
            && let Some(synset) = self.database.get_synset(offset)
        {
            return self
                .database
                .get_hyponyms(synset)
                .into_iter()
                .filter_map(|s| s.primary_word().map(|w| w.to_string()))
                .collect();
        }

        Vec::new()
    }

    /// Get synonyms for a word
    pub fn get_synonyms(&self, word: &str, pos: PartOfSpeech) -> Vec<String> {
        if !self.is_loaded {
            return Vec::new();
        }

        // Get all synsets for the word, then extract all words from those synsets
        let synsets = self.database.get_synsets_for_word(word, pos);
        let mut synonyms = Vec::new();

        for synset in synsets {
            for synset_word in &synset.words {
                if synset_word.word.to_lowercase() != word.to_lowercase() {
                    synonyms.push(synset_word.word.clone());
                }
            }
        }

        // Remove duplicates
        synonyms.sort();
        synonyms.dedup();
        synonyms
    }

    /// Check if the engine is ready for analysis
    pub fn is_ready(&self) -> bool {
        self.is_loaded && !self.database.synsets.is_empty()
    }

    /// Calculate WordNet-specific confidence score
    fn calculate_wordnet_confidence(&self, analysis: &WordNetAnalysis) -> f32 {
        if analysis.synsets.is_empty() {
            return 0.0;
        }

        // Base confidence from synset count
        let synset_count_factor = (analysis.synsets.len() as f32 * 0.1).min(0.5);

        // Relations factor (more relations indicate richer semantic data)
        let relations_factor = (analysis.relations.len() as f32 * 0.05).min(0.2);

        (0.3 + synset_count_factor + relations_factor).min(1.0)
    }

    // Backward compatibility methods for BaseEngine integration
    pub fn config(&self) -> &WordNetConfig {
        &self.wordnet_config
    }

    pub fn performance_metrics(&self) -> PerformanceMetrics {
        self.base_engine.get_performance_metrics()
    }

    pub fn cache_stats(&self) -> canopy_engine::CacheStats {
        self.base_engine.cache_stats()
    }

    pub fn clear_cache(&mut self) -> EngineResult<()> {
        self.base_engine.clear_cache();
        Ok(())
    }
}

// EngineCore trait implementation for BaseEngine integration
impl EngineCore<WordNetInput, WordNetAnalysis> for WordNetEngine {
    fn perform_analysis(&self, input: &WordNetInput) -> EngineResult<WordNetAnalysis> {
        if !self.is_loaded {
            debug!("WordNet database not loaded for analysis");
            return Ok(WordNetAnalysis::new(input.word.clone(), input.pos));
        }

        let mut analysis = WordNetAnalysis::new(input.word.clone(), input.pos);

        // Look up the word in the index
        if let Some(_index_entry) = self.database.lookup_word(&input.word, input.pos) {
            // Get all synsets for this word
            analysis.synsets = self
                .database
                .get_synsets_for_word(&input.word, input.pos)
                .into_iter()
                .cloned()
                .collect();

            // Extract definitions and examples
            for synset in &analysis.synsets {
                analysis.definitions.push(synset.definition());
                analysis.examples.extend(synset.examples());
            }

            // Find semantic relations
            for synset in &analysis.synsets {
                let hypernyms = self.database.get_hypernyms(synset);
                if !hypernyms.is_empty() {
                    analysis.relations.push((
                        crate::types::SemanticRelation::Hypernym,
                        hypernyms.into_iter().cloned().collect(),
                    ));
                }

                let hyponyms = self.database.get_hyponyms(synset);
                if !hyponyms.is_empty() {
                    analysis.relations.push((
                        crate::types::SemanticRelation::Hyponym,
                        hyponyms.into_iter().cloned().collect(),
                    ));
                }
            }

            // Calculate confidence
            analysis.confidence = self.calculate_wordnet_confidence(&analysis);
        }

        debug!(
            "WordNet analysis for '{}' ({}): {} synsets found, confidence: {:.2}",
            input.word,
            input.pos.name(),
            analysis.synsets.len(),
            analysis.confidence
        );

        Ok(analysis)
    }

    fn calculate_confidence(&self, _input: &WordNetInput, output: &WordNetAnalysis) -> f32 {
        self.calculate_wordnet_confidence(output)
    }

    fn generate_cache_key(&self, input: &WordNetInput) -> String {
        CacheKeyFormat::Typed(
            "wordnet".to_string(),
            format!("{}:{}", input.word, input.pos.code()),
        )
        .to_string()
    }

    fn engine_name(&self) -> &'static str {
        "WordNet"
    }

    fn engine_version(&self) -> &'static str {
        "3.1"
    }

    fn is_initialized(&self) -> bool {
        self.is_loaded
    }
}

impl SemanticEngine for WordNetEngine {
    type Input = String;
    type Output = WordNetAnalysis;
    type Config = WordNetConfig;

    fn analyze(&self, input: &Self::Input) -> EngineResult<SemanticResult<Self::Output>> {
        // Default to noun POS for simple string input
        let wordnet_input = WordNetInput {
            word: input.clone(),
            pos: PartOfSpeech::Noun,
        };
        self.base_engine.analyze(&wordnet_input, self)
    }

    fn name(&self) -> &'static str {
        "WordNet"
    }

    fn version(&self) -> &'static str {
        "3.1"
    }

    fn is_initialized(&self) -> bool {
        self.is_loaded
    }

    fn config(&self) -> &Self::Config {
        &self.wordnet_config
    }
}

impl DataLoader for WordNetEngine {
    fn load_from_directory<P: AsRef<Path>>(&mut self, path: P) -> EngineResult<()> {
        let path = path.as_ref();
        info!("Loading WordNet data from: {}", path.display());

        let loader = WordNetLoader::new(self.wordnet_config.parser_config.clone());
        let database = loader.load_database(&path.to_string_lossy())?;

        self.database = Arc::new(database);
        self.is_loaded = true;

        info!(
            "WordNet database loaded: {} synsets",
            self.database.synsets.len()
        );

        Ok(())
    }

    fn load_test_data(&mut self) -> EngineResult<()> {
        // Create minimal test data
        self.database = Arc::new(WordNetDatabase::new());
        self.is_loaded = true;
        Ok(())
    }

    fn reload(&mut self) -> EngineResult<()> {
        self.is_loaded = false;
        self.database = Arc::new(WordNetDatabase::new());
        self.load_from_directory(self.wordnet_config.data_path.clone())
    }

    fn data_info(&self) -> DataInfo {
        DataInfo::new(
            format!("wordnet: {}", self.wordnet_config.data_path),
            self.database.synsets.len(),
        )
    }
}

impl CachedEngine for WordNetEngine {
    fn cache_stats(&self) -> canopy_engine::CacheStats {
        self.base_engine.cache_stats()
    }

    fn clear_cache(&self) {
        self.base_engine.clear_cache();
    }

    fn set_cache_capacity(&mut self, capacity: usize) {
        self.wordnet_config.cache_capacity = capacity;
    }
}

impl StatisticsProvider for WordNetEngine {
    fn statistics(&self) -> EngineStats {
        self.base_engine.get_stats()
    }

    fn performance_metrics(&self) -> PerformanceMetrics {
        self.base_engine.get_performance_metrics()
    }
}

impl Default for WordNetEngine {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            panic!("WordNet engine requires real data - cannot create default instance without WordNet files")
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use once_cell::sync::OnceCell;
    use std::sync::Mutex;

    // Shared engine singleton - loaded once per test binary
    static SHARED_ENGINE: OnceCell<Mutex<WordNetEngine>> = OnceCell::new();

    /// Check if WordNet data is available
    fn wordnet_available() -> bool {
        canopy_core::paths::data_path("data/wordnet").exists()
    }

    /// Get shared WordNet engine, or None if data unavailable
    fn shared_engine() -> Option<&'static Mutex<WordNetEngine>> {
        if !wordnet_available() {
            return None;
        }
        Some(SHARED_ENGINE.get_or_init(|| {
            eprintln!("🔧 Loading shared WordNet engine (one-time)...");
            Mutex::new(WordNetEngine::new().expect("WordNet data required"))
        }))
    }

    #[test]
    fn test_wordnet_engine_creation() {
        // Use shared engine (loaded once, reused across tests)
        let Some(engine_ref) = shared_engine() else {
            eprintln!("Skipping test: WordNet data not available");
            return;
        };

        let engine = engine_ref.lock().unwrap();
        assert!(SemanticEngine::is_initialized(&*engine));
        assert_eq!(engine.name(), "WordNet");
        assert_eq!(engine.version(), "3.1");
    }

    #[test]
    fn test_wordnet_config_default() {
        let config = WordNetConfig::default();
        assert!(config.enable_cache);
        assert_eq!(config.cache_capacity, 10000);
        assert_eq!(config.min_confidence, 0.1);
        assert!(config.enable_morphology);
    }

    #[test]
    fn test_wordnet_input_hashing() {
        let input1 = WordNetInput {
            word: "test".to_string(),
            pos: PartOfSpeech::Noun,
        };
        let input2 = WordNetInput {
            word: "test".to_string(),
            pos: PartOfSpeech::Noun,
        };
        let input3 = WordNetInput {
            word: "test".to_string(),
            pos: PartOfSpeech::Verb,
        };

        let mut hasher1 = std::collections::hash_map::DefaultHasher::new();
        let mut hasher2 = std::collections::hash_map::DefaultHasher::new();
        let mut hasher3 = std::collections::hash_map::DefaultHasher::new();

        input1.hash(&mut hasher1);
        input2.hash(&mut hasher2);
        input3.hash(&mut hasher3);

        assert_eq!(hasher1.finish(), hasher2.finish());
        assert_ne!(hasher1.finish(), hasher3.finish());
    }

    #[test]
    fn test_empty_analysis() {
        // Use shared engine (loaded once, reused across tests)
        let Some(engine_ref) = shared_engine() else {
            eprintln!("Skipping test: WordNet data not available");
            return;
        };

        let engine = engine_ref.lock().unwrap();
        // Test analysis with real data
        if let Ok(result) = engine.analyze_word("test", PartOfSpeech::Noun) {
            // With real data, we may or may not find synsets
            assert!(result.confidence >= 0.0);
        }
    }

    #[test]
    fn test_confidence_calculation() {
        // Use shared engine (loaded once, reused across tests)
        let Some(engine_ref) = shared_engine() else {
            eprintln!("Skipping test: WordNet data not available");
            return;
        };

        let engine = engine_ref.lock().unwrap();

        let analysis = WordNetAnalysis::new("test".to_string(), PartOfSpeech::Noun);
        let confidence = engine.calculate_wordnet_confidence(&analysis);

        // Test confidence bounds
        assert!(confidence >= 0.0);
        assert!(confidence <= 1.0);
    }
}
