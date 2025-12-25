//! Main treebank engine implementation
//!
//! This module implements the TreebankEngine that integrates with the
//! canopy-engine trait system to provide dependency pattern analysis.

use crate::cache::{AdaptiveCache, CacheConfig};
use crate::indexer::{PatternIndexer, TreebankIndex};
use crate::signature::SignatureBuilder;
use crate::synthesizer::PatternSynthesizer;
use crate::types::TreebankAnalysis;
use crate::TreebankResult;
use canopy_core::paths::data_path;
use canopy_engine::{
    traits::{CachedEngine, DataInfo, DataLoader, SemanticEngine, StatisticsProvider},
    BaseEngine, CacheKeyFormat, CacheStats, EngineConfig, EngineCore, EngineError, EngineResult,
    EngineStats, PerformanceMetrics, SemanticResult,
};
use canopy_tokenizer::TreebankProvider;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tracing::{debug, info, warn};

/// Input type for Treebank analysis
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreebankInput {
    pub word: String,
    /// Optional semantic context for enhanced analysis
    pub verbnet_analysis: Option<String>, // Simplified for hashing
    pub framenet_analysis: Option<String>, // Simplified for hashing
}

impl Hash for TreebankInput {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.word.hash(state);
        self.verbnet_analysis.hash(state);
        self.framenet_analysis.hash(state);
    }
}

impl TreebankInput {
    /// Create simple input from word only
    pub fn simple(word: String) -> Self {
        Self {
            word,
            verbnet_analysis: None,
            framenet_analysis: None,
        }
    }

    /// Create enhanced input with semantic context
    pub fn with_context(word: String, verbnet: Option<String>, framenet: Option<String>) -> Self {
        Self {
            word,
            verbnet_analysis: verbnet,
            framenet_analysis: framenet,
        }
    }
}

/// Configuration for the treebank engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreebankConfig {
    /// Path to treebank data directory
    pub data_path: PathBuf,
    /// Path to prebuilt index (optional)
    pub index_path: Option<PathBuf>,
    /// Cache configuration
    pub cache: CacheConfig,
    /// Minimum pattern frequency for indexing
    pub min_frequency: u32,
    /// Enable pattern synthesis fallback
    pub enable_synthesis: bool,
    /// Enable detailed logging
    pub verbose: bool,
    /// Export learned lemma mappings from validation
    pub export_lemma_mappings: bool,
    /// Validate lemmatization against gold standard
    pub validate_lemmatization: bool,
    /// Shared lemma cache configuration
    pub lemma_cache_config: Option<canopy_engine::LemmaCacheConfig>,
    /// Enable BaseEngine integration
    pub enable_base_engine_cache: bool,
    /// BaseEngine cache capacity
    pub base_engine_cache_capacity: usize,
}

impl Default for TreebankConfig {
    fn default() -> Self {
        Self {
            data_path: data_path("data/ud_english-ewt/UD_English-EWT"),
            index_path: Some(data_path("data/cache/treebank_index.bin")),
            cache: CacheConfig::default(),
            min_frequency: 2,
            enable_synthesis: true,
            verbose: false,
            export_lemma_mappings: true, // Enable by default to learn from UD data
            validate_lemmatization: true, // Enable validation by default
            lemma_cache_config: Some(canopy_engine::LemmaCacheConfig::default()),
            enable_base_engine_cache: true,
            base_engine_cache_capacity: 5000,
        }
    }
}

/// Statistics for the treebank engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreebankStats {
    /// Base engine statistics
    pub base: EngineStats,
    /// Cache performance statistics
    pub cache: CacheStats,
    /// Total patterns in index
    pub total_indexed_patterns: usize,
    /// Patterns found via synthesis
    pub synthesized_patterns: u64,
    /// Patterns not found at all
    pub pattern_misses: u64,
    /// Average lookup time in microseconds
    pub avg_lookup_time_us: f64,
}

impl Default for TreebankStats {
    fn default() -> Self {
        Self {
            base: EngineStats::new("TreebankEngine".to_string()),
            cache: canopy_engine::CacheStats::empty(),
            total_indexed_patterns: 0,
            synthesized_patterns: 0,
            pattern_misses: 0,
            avg_lookup_time_us: 0.0,
        }
    }
}

/// Main treebank engine
#[derive(Debug)]
pub struct TreebankEngine {
    /// Base engine handling standard workflow and caching
    base_engine: BaseEngine<TreebankInput, TreebankAnalysis>,
    /// Treebank-specific configuration
    treebank_config: TreebankConfig,
    /// Adaptive pattern cache
    adaptive_cache: Arc<Mutex<AdaptiveCache>>,
    /// Pattern synthesizer for fallback
    synthesizer: PatternSynthesizer,
    /// Signature builder
    signature_builder: SignatureBuilder,
    /// Extended treebank statistics
    treebank_stats: Arc<Mutex<TreebankStats>>,
    /// Is engine initialized flag
    is_initialized: bool,
}

impl TreebankEngine {
    /// Create a new treebank engine
    pub fn new() -> TreebankResult<Self> {
        Self::with_config(TreebankConfig::default())
    }

    /// Create a new treebank engine with custom configuration
    pub fn with_config(treebank_config: TreebankConfig) -> TreebankResult<Self> {
        info!("Initializing treebank engine");

        // Convert TreebankConfig to EngineConfig for BaseEngine
        let engine_config = EngineConfig {
            enable_cache: treebank_config.enable_base_engine_cache,
            cache_capacity: treebank_config.base_engine_cache_capacity,
            enable_metrics: true,
            enable_parallel: false,
            max_threads: 4,
            confidence_threshold: 0.5,
        };

        let base_engine = BaseEngine::new(engine_config, "TreebankEngine".to_string());
        let mut adaptive_cache = AdaptiveCache::new(treebank_config.cache.clone());
        let synthesizer = PatternSynthesizer::new(treebank_config.verbose);
        let signature_builder = SignatureBuilder::new(treebank_config.verbose);

        // Load or build index
        let index = Self::load_or_build_index(&treebank_config)?;
        adaptive_cache.initialize_with_index(index)?;

        let treebank_stats = TreebankStats {
            total_indexed_patterns: adaptive_cache.get_stats().estimated_memory_bytes / 1024,
            ..Default::default()
        };

        info!("Treebank engine initialized successfully");

        Ok(Self {
            base_engine,
            treebank_config,
            adaptive_cache: Arc::new(Mutex::new(adaptive_cache)),
            synthesizer,
            signature_builder,
            treebank_stats: Arc::new(Mutex::new(treebank_stats)),
            is_initialized: true,
        })
    }

    /// Load existing index or build from treebank files
    fn load_or_build_index(config: &TreebankConfig) -> TreebankResult<TreebankIndex> {
        // Try to load existing index first
        if let Some(index_path) = &config.index_path {
            if index_path.exists() {
                info!("Loading existing index from {}", index_path.display());
                match TreebankIndex::load(index_path) {
                    Ok(index) => {
                        info!(
                            "Successfully loaded index with {} patterns",
                            index.get_stats().total_patterns
                        );
                        return Ok(index);
                    }
                    Err(e) => {
                        warn!("Failed to load index: {}, will rebuild", e);
                    }
                }
            }
        }

        // Build new index from treebank files
        info!("Building new index from treebank data");
        let indexer = PatternIndexer::new(config.verbose, config.min_frequency);

        // Find CoNLL-U files in data directory
        let conllu_files = Self::find_conllu_files(&config.data_path)?;
        if conllu_files.is_empty() {
            return Err(canopy_engine::EngineError::config(format!(
                "No CoNLL-U files found in {}",
                config.data_path.display()
            )));
        }

        info!("Found {} CoNLL-U files to process", conllu_files.len());
        let index = indexer.build_index(&conllu_files)?;

        // Save index if path is configured
        if let Some(index_path) = &config.index_path {
            if let Err(e) = index.save(index_path) {
                warn!("Failed to save index to {}: {}", index_path.display(), e);
            } else {
                info!("Saved index to {}", index_path.display());
            }
        }

        Ok(index)
    }

    /// Find CoNLL-U files in directory
    fn find_conllu_files(data_path: &Path) -> TreebankResult<Vec<PathBuf>> {
        let mut files = Vec::new();

        if !data_path.exists() {
            return Err(canopy_engine::EngineError::config(format!(
                "Data path does not exist: {}",
                data_path.display()
            )));
        }

        for entry in std::fs::read_dir(data_path).map_err(|e| {
            canopy_engine::EngineError::io(format!("read directory {}", data_path.display()), e)
        })? {
            let entry = entry.map_err(|e| {
                canopy_engine::EngineError::io("read directory entry".to_string(), e)
            })?;

            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("conllu") {
                files.push(path);
            }
        }

        files.sort(); // Ensure consistent ordering
        Ok(files)
    }

    /// Analyze a word using BaseEngine workflow
    pub fn analyze_word(&self, word: &str) -> EngineResult<SemanticResult<TreebankAnalysis>> {
        let input = TreebankInput::simple(word.to_string());
        self.base_engine.analyze(&input, self)
    }

    /// Legacy method for direct analysis (preserved for compatibility)
    pub fn analyze_word_direct(&self, word: &str) -> TreebankResult<TreebankAnalysis> {
        let start_time = Instant::now();

        // Create basic signature (we don't have full Layer 1 data here)
        let signature = self.signature_builder.build_simplified(word, None);

        // Try to get pattern from cache
        let pattern = if let Ok(mut cache) = self.adaptive_cache.lock() {
            cache.get_pattern(&signature)
        } else {
            None
        };

        let result = if let Some(pattern) = pattern {
            // Found in cache
            TreebankAnalysis::new(
                word.to_string(),
                Some(pattern),
                0.8,
                start_time.elapsed().as_micros() as u64,
                true,
            )
        } else if self.config().enable_synthesis {
            // Try synthesis
            match self.synthesizer.synthesize_pattern(&signature) {
                Ok(synthesized_pattern) => {
                    // Cache synthesized pattern
                    if let Ok(mut cache) = self.adaptive_cache.lock() {
                        cache.cache_pattern(signature, synthesized_pattern.clone());
                    }

                    // Update stats
                    if let Ok(mut stats) = self.treebank_stats.lock() {
                        stats.synthesized_patterns += 1;
                    }

                    // Calculate dynamic confidence based on pattern quality
                    let confidence =
                        self.calculate_synthesis_confidence(&synthesized_pattern, false);
                    TreebankAnalysis::new(
                        word.to_string(),
                        Some(synthesized_pattern),
                        confidence,
                        start_time.elapsed().as_micros() as u64,
                        false,
                    )
                }
                Err(_) => {
                    // No pattern available
                    if let Ok(mut stats) = self.treebank_stats.lock() {
                        stats.pattern_misses += 1;
                    }

                    TreebankAnalysis::no_pattern(
                        word.to_string(),
                        start_time.elapsed().as_micros() as u64,
                    )
                }
            }
        } else {
            // Synthesis disabled, no pattern
            if let Ok(mut stats) = self.treebank_stats.lock() {
                stats.pattern_misses += 1;
            }

            TreebankAnalysis::no_pattern(word.to_string(), start_time.elapsed().as_micros() as u64)
        };

        // Update timing statistics
        let processing_time = start_time.elapsed().as_micros() as f64;
        if let Ok(mut stats) = self.treebank_stats.lock() {
            let total_queries = stats.base.performance.total_queries + 1;
            stats.avg_lookup_time_us = (stats.avg_lookup_time_us
                * stats.base.performance.total_queries as f64
                + processing_time)
                / total_queries as f64;
            stats.base.performance.total_queries = total_queries;
        }

        Ok(result)
    }

    /// Analyze with full Layer 1 semantic signature using BaseEngine
    pub fn analyze_with_signature(
        &self,
        lemma: &str,
        verbnet_analysis: Option<&canopy_verbnet::VerbNetAnalysis>,
        framenet_analysis: Option<&canopy_framenet::FrameNetAnalysis>,
    ) -> EngineResult<SemanticResult<TreebankAnalysis>> {
        let verbnet_str = verbnet_analysis.map(|v| format!("{:?}", v)); // Simplified for now
        let framenet_str = framenet_analysis.map(|f| format!("{:?}", f)); // Simplified for now

        let input = TreebankInput::with_context(lemma.to_string(), verbnet_str, framenet_str);

        self.base_engine.analyze(&input, self)
    }

    /// Legacy method for direct semantic analysis (preserved for compatibility)
    pub fn analyze_with_signature_direct(
        &self,
        lemma: &str,
        verbnet_analysis: Option<&canopy_verbnet::VerbNetAnalysis>,
        framenet_analysis: Option<&canopy_framenet::FrameNetAnalysis>,
    ) -> TreebankResult<TreebankAnalysis> {
        let start_time = Instant::now();

        // Create full semantic signature
        let signature = self.signature_builder.build_from_layer1(
            lemma,
            verbnet_analysis,
            framenet_analysis,
            None,
            canopy_engine::LemmaSource::SimpleLemmatizer, // Default for now
            0.8,                                          // Default confidence
        );

        // Try exact match first
        let mut pattern = if let Ok(mut cache) = self.adaptive_cache.lock() {
            cache.get_pattern(&signature)
        } else {
            None
        };

        // If no exact match, try variants
        if pattern.is_none() {
            let variants = self.signature_builder.build_variants(&signature);
            if let Ok(mut cache) = self.adaptive_cache.lock() {
                pattern = cache.get_pattern_with_fallback(&signature, &variants);
            }
        }

        let result = if let Some(pattern) = pattern {
            TreebankAnalysis::new(
                lemma.to_string(),
                Some(pattern),
                0.9, // High confidence for semantic-guided lookup
                start_time.elapsed().as_micros() as u64,
                true,
            )
        } else if self.config().enable_synthesis {
            // Try synthesis with semantic information
            match self.synthesizer.synthesize_pattern(&signature) {
                Ok(synthesized_pattern) => {
                    if let Ok(mut cache) = self.adaptive_cache.lock() {
                        cache.cache_pattern(signature, synthesized_pattern.clone());
                    }

                    if let Ok(mut stats) = self.treebank_stats.lock() {
                        stats.synthesized_patterns += 1;
                    }

                    TreebankAnalysis::new(
                        lemma.to_string(),
                        Some(synthesized_pattern),
                        0.7, // Higher confidence with semantic info
                        start_time.elapsed().as_micros() as u64,
                        false,
                    )
                }
                Err(_) => {
                    if let Ok(mut stats) = self.treebank_stats.lock() {
                        stats.pattern_misses += 1;
                    }

                    TreebankAnalysis::no_pattern(
                        lemma.to_string(),
                        start_time.elapsed().as_micros() as u64,
                    )
                }
            }
        } else {
            if let Ok(mut stats) = self.treebank_stats.lock() {
                stats.pattern_misses += 1;
            }

            TreebankAnalysis::no_pattern(lemma.to_string(), start_time.elapsed().as_micros() as u64)
        };

        Ok(result)
    }

    /// Get engine statistics
    pub fn get_statistics(&self) -> TreebankStats {
        let mut stats = self.treebank_stats.lock().unwrap().clone();

        // Update cache stats
        if let Ok(cache) = self.adaptive_cache.lock() {
            // Convert our cache stats to engine cache stats
            let our_cache_stats = cache.get_stats();
            stats.cache = canopy_engine::CacheStats {
                hits: our_cache_stats.core_hits + our_cache_stats.lru_hits,
                misses: our_cache_stats.index_lookups,
                total_lookups: our_cache_stats.total_lookups,
                hit_rate: our_cache_stats.hit_rate(),
                evictions: 0,    // Not tracked in our cache
                current_size: 0, // Estimated separately
                has_ttl: false,
            };
        }

        stats
    }

    /// Clear caches and reset statistics
    pub fn reset(&self) -> TreebankResult<()> {
        if let Ok(mut cache) = self.adaptive_cache.lock() {
            cache.clear_caches();
        }

        if let Ok(mut stats) = self.treebank_stats.lock() {
            *stats = TreebankStats::default();
        }

        // Also clear BaseEngine cache
        self.base_engine.clear_cache();

        info!("Treebank engine reset successfully");
        Ok(())
    }

    /// Force memory cleanup
    pub fn cleanup_memory(&self) -> TreebankResult<bool> {
        if let Ok(mut cache) = self.adaptive_cache.lock() {
            Ok(cache.cleanup_if_needed())
        } else {
            Ok(false)
        }
    }

    /// Get memory usage information
    pub fn memory_usage(&self) -> std::collections::HashMap<String, usize> {
        if let Ok(cache) = self.adaptive_cache.lock() {
            cache.memory_breakdown()
        } else {
            std::collections::HashMap::new()
        }
    }

    // Backward compatibility methods for BaseEngine integration
    pub fn config(&self) -> &TreebankConfig {
        &self.treebank_config
    }

    pub fn performance_metrics(&self) -> PerformanceMetrics {
        self.base_engine.get_performance_metrics()
    }

    pub fn cache_stats(&self) -> canopy_engine::CacheStats {
        self.base_engine.cache_stats()
    }

    pub fn clear_cache(&mut self) -> EngineResult<()> {
        self.base_engine.clear_cache();
        if let Ok(mut cache) = self.adaptive_cache.lock() {
            cache.clear_caches();
        }
        Ok(())
    }
}

// EngineCore trait implementation for BaseEngine integration
impl EngineCore<TreebankInput, TreebankAnalysis> for TreebankEngine {
    fn perform_analysis(&self, input: &TreebankInput) -> EngineResult<TreebankAnalysis> {
        if !self.is_initialized {
            debug!("Treebank engine not initialized for analysis");
            return Ok(TreebankAnalysis::no_pattern(input.word.clone(), 0));
        }

        // Use the appropriate analysis method based on input complexity
        let result = if input.verbnet_analysis.is_some() || input.framenet_analysis.is_some() {
            // Enhanced analysis with semantic context
            self.analyze_with_enhanced_context(input)
        } else {
            // Simple word analysis
            self.analyze_simple_word(&input.word)
        };

        match result {
            Ok(analysis) => Ok(analysis),
            Err(_) => Ok(TreebankAnalysis::no_pattern(input.word.clone(), 0)),
        }
    }

    fn calculate_confidence(&self, _input: &TreebankInput, output: &TreebankAnalysis) -> f32 {
        output.confidence
    }

    fn generate_cache_key(&self, input: &TreebankInput) -> String {
        if input.verbnet_analysis.is_some() || input.framenet_analysis.is_some() {
            CacheKeyFormat::Typed(
                "treebank".to_string(),
                format!(
                    "{}:{}:{}",
                    input.word,
                    input.verbnet_analysis.as_deref().unwrap_or(""),
                    input.framenet_analysis.as_deref().unwrap_or("")
                ),
            )
            .to_string()
        } else {
            CacheKeyFormat::Typed("treebank".to_string(), input.word.clone()).to_string()
        }
    }

    fn engine_name(&self) -> &'static str {
        "TreebankEngine"
    }

    fn engine_version(&self) -> &'static str {
        "0.1.0"
    }

    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl TreebankEngine {
    /// Helper method for simple word analysis
    fn analyze_simple_word(&self, word: &str) -> TreebankResult<TreebankAnalysis> {
        // This replicates the logic from the original analyze_word_direct method
        let start_time = std::time::Instant::now();

        let signature = self.signature_builder.build_simplified(word, None);

        let pattern = if let Ok(mut cache) = self.adaptive_cache.lock() {
            cache.get_pattern(&signature)
        } else {
            None
        };

        let result = if let Some(pattern) = pattern {
            TreebankAnalysis::new(
                word.to_string(),
                Some(pattern),
                0.8,
                start_time.elapsed().as_micros() as u64,
                true,
            )
        } else if self.treebank_config.enable_synthesis {
            match self.synthesizer.synthesize_pattern(&signature) {
                Ok(synthesized_pattern) => {
                    if let Ok(mut cache) = self.adaptive_cache.lock() {
                        cache.cache_pattern(signature, synthesized_pattern.clone());
                    }
                    if let Ok(mut stats) = self.treebank_stats.lock() {
                        stats.synthesized_patterns += 1;
                    }
                    TreebankAnalysis::new(
                        word.to_string(),
                        Some(synthesized_pattern),
                        0.6,
                        start_time.elapsed().as_micros() as u64,
                        false,
                    )
                }
                Err(_) => {
                    if let Ok(mut stats) = self.treebank_stats.lock() {
                        stats.pattern_misses += 1;
                    }
                    TreebankAnalysis::no_pattern(
                        word.to_string(),
                        start_time.elapsed().as_micros() as u64,
                    )
                }
            }
        } else {
            if let Ok(mut stats) = self.treebank_stats.lock() {
                stats.pattern_misses += 1;
            }
            TreebankAnalysis::no_pattern(word.to_string(), start_time.elapsed().as_micros() as u64)
        };

        Ok(result)
    }

    /// Helper method for enhanced context analysis
    fn analyze_with_enhanced_context(
        &self,
        input: &TreebankInput,
    ) -> TreebankResult<TreebankAnalysis> {
        let start_time = std::time::Instant::now();

        // Build signature with context (simplified for now)
        let signature = self.signature_builder.build_simplified(&input.word, None);

        // Try exact match first
        let mut pattern = if let Ok(mut cache) = self.adaptive_cache.lock() {
            cache.get_pattern(&signature)
        } else {
            None
        };

        // Try variants if no exact match
        if pattern.is_none() {
            let variants = self.signature_builder.build_variants(&signature);
            if let Ok(mut cache) = self.adaptive_cache.lock() {
                pattern = cache.get_pattern_with_fallback(&signature, &variants);
            }
        }

        let result = if let Some(pattern) = pattern {
            TreebankAnalysis::new(
                input.word.clone(),
                Some(pattern),
                0.9, // Higher confidence for semantic-guided lookup
                start_time.elapsed().as_micros() as u64,
                true,
            )
        } else if self.treebank_config.enable_synthesis {
            match self.synthesizer.synthesize_pattern(&signature) {
                Ok(synthesized_pattern) => {
                    if let Ok(mut cache) = self.adaptive_cache.lock() {
                        cache.cache_pattern(signature, synthesized_pattern.clone());
                    }
                    if let Ok(mut stats) = self.treebank_stats.lock() {
                        stats.synthesized_patterns += 1;
                    }
                    // Calculate dynamic confidence based on pattern quality and semantic info
                    let confidence =
                        self.calculate_synthesis_confidence(&synthesized_pattern, true);
                    TreebankAnalysis::new(
                        input.word.clone(),
                        Some(synthesized_pattern),
                        confidence,
                        start_time.elapsed().as_micros() as u64,
                        false,
                    )
                }
                Err(_) => {
                    if let Ok(mut stats) = self.treebank_stats.lock() {
                        stats.pattern_misses += 1;
                    }
                    TreebankAnalysis::no_pattern(
                        input.word.clone(),
                        start_time.elapsed().as_micros() as u64,
                    )
                }
            }
        } else {
            if let Ok(mut stats) = self.treebank_stats.lock() {
                stats.pattern_misses += 1;
            }
            TreebankAnalysis::no_pattern(
                input.word.clone(),
                start_time.elapsed().as_micros() as u64,
            )
        };

        Ok(result)
    }

    /// Calculate dynamic confidence for synthesized patterns
    fn calculate_synthesis_confidence(
        &self,
        pattern: &crate::types::DependencyPattern,
        has_semantic_info: bool,
    ) -> f32 {
        let mut confidence: f32 = 0.4; // Base confidence for synthesis

        // Increase confidence based on pattern characteristics
        if pattern.frequency > 10 {
            confidence += 0.2; // High frequency pattern
        } else if pattern.frequency > 5 {
            confidence += 0.1; // Medium frequency pattern
        }

        // Increase confidence based on dependency complexity
        if pattern.dependencies.len() >= 3 {
            confidence += 0.15; // Complex dependency structure
        } else if pattern.dependencies.len() >= 2 {
            confidence += 0.1; // Simple dependency structure
        }

        // Bonus for semantic context
        if has_semantic_info {
            confidence += 0.15;
        }

        // Cap at reasonable maximum
        confidence.min(0.95)
    }
}

// Implement the SemanticEngine trait
impl SemanticEngine for TreebankEngine {
    type Input = String;
    type Output = TreebankAnalysis;
    type Config = TreebankConfig;

    fn analyze(&self, input: &Self::Input) -> EngineResult<SemanticResult<Self::Output>> {
        let treebank_input = TreebankInput::simple(input.clone());
        self.base_engine.analyze(&treebank_input, self)
    }

    fn name(&self) -> &'static str {
        "TreebankEngine"
    }

    fn version(&self) -> &'static str {
        "0.1.0"
    }

    fn is_initialized(&self) -> bool {
        self.is_initialized
    }

    fn config(&self) -> &Self::Config {
        &self.treebank_config
    }
}

impl CachedEngine for TreebankEngine {
    fn cache_stats(&self) -> canopy_engine::CacheStats {
        // Return BaseEngine cache stats combined with adaptive cache stats
        let base_stats = self.base_engine.cache_stats();

        if let Ok(cache) = self.adaptive_cache.lock() {
            let treebank_stats = cache.get_stats();
            // Combine both cache statistics
            canopy_engine::CacheStats {
                hits: base_stats.hits + treebank_stats.core_hits + treebank_stats.lru_hits,
                misses: base_stats.misses + treebank_stats.index_lookups,
                total_lookups: base_stats.total_lookups + treebank_stats.total_lookups,
                hit_rate: {
                    let total_hits =
                        base_stats.hits + treebank_stats.core_hits + treebank_stats.lru_hits;
                    let total_lookups = base_stats.total_lookups + treebank_stats.total_lookups;
                    if total_lookups > 0 {
                        (total_hits as f32 / total_lookups as f32) as f64
                    } else {
                        0.0
                    }
                },
                evictions: base_stats.evictions,
                current_size: base_stats.current_size,
                has_ttl: base_stats.has_ttl,
            }
        } else {
            base_stats
        }
    }

    fn clear_cache(&self) {
        self.base_engine.clear_cache();
        if let Ok(mut cache) = self.adaptive_cache.lock() {
            cache.clear_caches();
        }
    }

    fn set_cache_capacity(&mut self, capacity: usize) {
        // Update configuration for future cache rebuilding
        self.treebank_config.base_engine_cache_capacity = capacity;
        // Note: BaseEngine cache capacity would need to be rebuilt to change capacity
        // Adaptive cache capacity is managed separately via TreebankConfig.cache
    }
}

impl StatisticsProvider for TreebankEngine {
    fn statistics(&self) -> EngineStats {
        // Get base engine stats and enhance with treebank-specific data
        let mut base_stats = self.base_engine.get_stats();
        let treebank_stats = self.get_statistics();

        // Update quality stats with treebank-specific metrics
        base_stats.quality.successful_analyses = base_stats
            .quality
            .successful_analyses
            .max(treebank_stats.base.quality.successful_analyses);
        base_stats.quality.failed_analyses += treebank_stats.base.quality.failed_analyses;
        base_stats.quality.avg_confidence =
            (base_stats.quality.avg_confidence + treebank_stats.base.quality.avg_confidence) / 2.0;

        base_stats
    }

    fn performance_metrics(&self) -> PerformanceMetrics {
        self.base_engine.get_performance_metrics()
    }
}

impl DataLoader for TreebankEngine {
    fn load_from_directory<P: std::convert::AsRef<std::path::Path>>(
        &mut self,
        path: P,
    ) -> EngineResult<()> {
        let path = path.as_ref();
        info!("Loading Treebank data from: {}", path.display());

        self.treebank_config.data_path = path.to_path_buf();

        // Rebuild index with new data path
        match Self::load_or_build_index(&self.treebank_config) {
            Ok(index) => {
                if let Ok(mut cache) = self.adaptive_cache.lock() {
                    if let Err(e) = cache.initialize_with_index(index) {
                        return Err(canopy_engine::EngineError::not_initialized(format!(
                            "Failed to initialize cache with new index: {}",
                            e
                        )));
                    }
                }
                info!(
                    "Successfully reloaded Treebank data from: {}",
                    path.display()
                );
                Ok(())
            }
            Err(e) => Err(canopy_engine::EngineError::data_load(format!(
                "Failed to load Treebank data: {}",
                e
            ))),
        }
    }

    fn load_test_data(&mut self) -> EngineResult<()> {
        // Create minimal test setup - just initialize with empty cache
        if let Ok(mut cache) = self.adaptive_cache.lock() {
            // Initialize empty cache for testing
            cache.clear_caches();
        }
        self.is_initialized = true;
        info!("Loaded minimal test data for Treebank engine");
        Ok(())
    }

    fn reload(&mut self) -> EngineResult<()> {
        info!("Reloading Treebank engine");
        let _ = self.reset();

        match Self::load_or_build_index(&self.treebank_config) {
            Ok(index) => {
                if let Ok(mut cache) = self.adaptive_cache.lock() {
                    if let Err(e) = cache.initialize_with_index(index) {
                        return Err(canopy_engine::EngineError::not_initialized(format!(
                            "Failed to reinitialize cache: {}",
                            e
                        )));
                    }
                }
                info!("Successfully reloaded Treebank engine");
                Ok(())
            }
            Err(e) => Err(canopy_engine::EngineError::data_load(format!(
                "Failed to reload Treebank data: {}",
                e
            ))),
        }
    }

    fn data_info(&self) -> DataInfo {
        let stats = self.get_statistics();
        DataInfo::new(
            format!("treebank: {}", self.treebank_config.data_path.display()),
            stats.total_indexed_patterns,
        )
    }
}

impl Default for TreebankEngine {
    fn default() -> Self {
        Self::new().expect("Failed to create default TreebankEngine")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_treebank_config_default() {
        let config = TreebankConfig::default();
        assert_eq!(config.min_frequency, 2);
        assert!(config.enable_synthesis);
    }

    #[test]
    fn test_find_conllu_files() {
        let temp_dir = TempDir::new().unwrap();
        let data_path = temp_dir.path();

        // Create test files
        std::fs::write(data_path.join("test1.conllu"), "# test file 1").unwrap();
        std::fs::write(data_path.join("test2.conllu"), "# test file 2").unwrap();
        std::fs::write(data_path.join("readme.txt"), "not a conllu file").unwrap();

        let files = TreebankEngine::find_conllu_files(data_path).unwrap();
        assert_eq!(files.len(), 2);
        assert!(files.iter().all(|f| f.extension().unwrap() == "conllu"));
    }

    #[test]
    fn test_treebank_stats_default() {
        let stats = TreebankStats::default();
        assert_eq!(stats.synthesized_patterns, 0);
        assert_eq!(stats.pattern_misses, 0);
        assert_eq!(stats.avg_lookup_time_us, 0.0);
    }

    #[test]
    fn test_semantic_engine_trait() {
        // This would need actual treebank data to test fully
        // For now, just verify the trait bounds compile
        fn _test_trait_bounds<T: SemanticEngine>() {}
        _test_trait_bounds::<TreebankEngine>();
    }

    #[test]
    fn test_cached_engine_trait() {
        // Verify CachedEngine trait bounds compile
        fn _test_trait_bounds<T: CachedEngine>() {}
        _test_trait_bounds::<TreebankEngine>();
    }

    #[test]
    fn test_statistics_provider_trait() {
        // Verify StatisticsProvider trait bounds compile
        fn _test_trait_bounds<T: StatisticsProvider>() {}
        _test_trait_bounds::<TreebankEngine>();
    }
}

// Implement TreebankProvider trait for integration with semantic layer
impl TreebankProvider for TreebankEngine {
    fn analyze_word(&self, word: &str) -> Result<canopy_tokenizer::TreebankAnalysis, EngineError> {
        // Use the existing analyze_word method from TreebankEngine
        match self.analyze_word(word) {
            Ok(treebank_result) => {
                // Convert from treebank::TreebankAnalysis to semantic_layer::TreebankAnalysis
                let mut semantic_result = canopy_tokenizer::TreebankAnalysis::new(
                    treebank_result.data.word.clone(),
                    treebank_result.confidence,
                );

                semantic_result.processing_time_us = treebank_result.processing_time_us;
                semantic_result.from_cache = treebank_result.from_cache;

                // Extract dependency relation from pattern if available
                if let Some(ref pattern) = treebank_result.data.pattern {
                    semantic_result.dependency_relation = Some(pattern.verb_lemma.clone());

                    // Convert dependency features to voice and semantic features
                    for (dep_rel, _role) in &pattern.dependencies {
                        let dep_str = format!("{:?}", dep_rel).to_lowercase();
                        if dep_str.contains("pass") {
                            semantic_result.voice_features.push("passive".to_string());
                        }
                        if dep_str.contains("agent") {
                            semantic_result.semantic_features.push("agent".to_string());
                        }
                        if dep_str.contains("subj") {
                            semantic_result
                                .semantic_features
                                .push("subject".to_string());
                        }
                        if dep_str.contains("obj") {
                            semantic_result.semantic_features.push("object".to_string());
                        }
                    }
                }

                Ok(semantic_result)
            }
            Err(e) => Err(e),
        }
    }
}
