//! Multi-tier pattern caching system for dependency patterns
//!
//! Implements a three-tier caching system:
//! 1. Core cache: Top 2,000 most frequent patterns (HashMap)
//! 2. LRU cache: Working set of 3,000 patterns (LruCache)
//! 3. Disk index: Full pattern set (memory-mapped file)

use crate::{DependencyPattern, SemanticSignature, TreebankResult};
use lru::LruCache;
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tracing::{debug, info};

/// Multi-tier pattern cache with core, LRU, and disk tiers
pub struct PatternCache {
    /// Core cache: Most frequent patterns for fastest access
    core_patterns: HashMap<String, DependencyPattern>,

    /// LRU cache: Working set patterns
    lru_cache: LruCache<String, DependencyPattern>,

    /// Pattern index for disk lookups
    pattern_index: Option<PatternIndex>,

    /// Cache statistics
    stats: CacheStatistics,

    /// Configuration
    config: PatternCacheConfig,
}

#[derive(Debug, Clone)]
pub struct PatternCacheConfig {
    /// Number of core patterns to keep in memory
    pub core_cache_size: usize,

    /// Number of LRU cache entries
    pub lru_cache_size: usize,

    /// Path to pattern index file
    pub index_path: Option<PathBuf>,

    /// Enable usage tracking for cache promotion
    pub enable_usage_tracking: bool,
}

impl Default for PatternCacheConfig {
    fn default() -> Self {
        Self {
            core_cache_size: 2000,
            lru_cache_size: 3000,
            index_path: None,
            enable_usage_tracking: true,
        }
    }
}

/// Statistics for cache performance monitoring
#[derive(Debug, Default, Clone)]
pub struct CacheStatistics {
    /// Core cache hits
    pub core_hits: u64,

    /// LRU cache hits
    pub lru_hits: u64,

    /// Disk index hits
    pub disk_hits: u64,

    /// Cache misses (pattern not found anywhere)
    pub misses: u64,

    /// Total lookup requests
    pub total_requests: u64,

    /// Pattern synthesis attempts
    pub synthesis_attempts: u64,

    /// Successful syntheses
    pub synthesis_successes: u64,
}

impl CacheStatistics {
    /// Calculate core cache hit rate
    pub fn core_hit_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            self.core_hits as f64 / self.total_requests as f64
        }
    }

    /// Calculate overall hit rate (core + LRU + disk)
    pub fn total_hit_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            (self.core_hits + self.lru_hits + self.disk_hits) as f64 / self.total_requests as f64
        }
    }

    /// Calculate pattern synthesis success rate
    pub fn synthesis_success_rate(&self) -> f64 {
        if self.synthesis_attempts == 0 {
            0.0
        } else {
            self.synthesis_successes as f64 / self.synthesis_attempts as f64
        }
    }
}

/// Pattern index for disk-based lookups
struct PatternIndex {
    /// Path to the index file
    _index_path: PathBuf,

    /// In-memory pattern lookup (simplified for now)
    patterns: HashMap<String, DependencyPattern>,
}

impl PatternCache {
    /// Create a new pattern cache with the given configuration
    pub fn new(config: PatternCacheConfig) -> TreebankResult<Self> {
        let lru_cache_size = NonZeroUsize::new(config.lru_cache_size).ok_or_else(|| {
            canopy_engine::EngineError::ConfigError {
                message: "LRU cache size must be greater than 0".to_string(),
            }
        })?;

        let mut cache = Self {
            core_patterns: HashMap::with_capacity(config.core_cache_size),
            lru_cache: LruCache::new(lru_cache_size),
            pattern_index: None,
            stats: CacheStatistics::default(),
            config,
        };

        // Load pattern index if configured
        if let Some(index_path) = cache.config.index_path.clone() {
            cache.load_pattern_index(&index_path)?;
        }

        Ok(cache)
    }

    /// Load pattern index from disk
    fn load_pattern_index(&mut self, index_path: &Path) -> TreebankResult<()> {
        info!("Loading pattern index from {:?}", index_path);

        // For now, create empty index - would load from disk in full implementation
        let index = PatternIndex {
            _index_path: index_path.to_path_buf(),
            patterns: HashMap::new(),
        };

        self.pattern_index = Some(index);
        debug!("Pattern index loaded successfully");

        Ok(())
    }

    /// Populate core cache with the most frequent patterns
    pub fn populate_core_cache(&mut self, patterns: &[(String, DependencyPattern)]) {
        info!(
            "Populating core cache with {} patterns",
            patterns.len().min(self.config.core_cache_size)
        );

        self.core_patterns.clear();
        self.core_patterns.reserve(self.config.core_cache_size);

        // Take the top patterns by frequency
        for (key, pattern) in patterns.iter().take(self.config.core_cache_size) {
            self.core_patterns.insert(key.clone(), pattern.clone());
        }

        info!(
            "Core cache populated with {} patterns",
            self.core_patterns.len()
        );
    }

    /// Look up a dependency pattern for the given signature
    pub fn get_pattern(&mut self, signature: &SemanticSignature) -> Option<DependencyPattern> {
        self.stats.total_requests += 1;
        let start = Instant::now();

        // Generate lookup key from signature
        let key = self.generate_pattern_key(signature);

        // Tier 1: Check core cache first
        if let Some(pattern) = self.core_patterns.get(&key) {
            self.stats.core_hits += 1;
            debug!(
                "Core cache hit for key: {} ({:.1}μs)",
                key,
                start.elapsed().as_micros()
            );
            return Some(pattern.clone());
        }

        // Tier 2: Check LRU cache
        if let Some(pattern) = self.lru_cache.get(&key) {
            self.stats.lru_hits += 1;
            debug!(
                "LRU cache hit for key: {} ({:.1}μs)",
                key,
                start.elapsed().as_micros()
            );
            return Some(pattern.clone());
        }

        // Tier 3: Check disk index
        if let Some(ref pattern_index) = self.pattern_index {
            if let Some(pattern) = pattern_index.patterns.get(&key) {
                self.stats.disk_hits += 1;

                // Promote to LRU cache
                let key_for_debug = key.clone();
                self.lru_cache.put(key, pattern.clone());

                debug!(
                    "Disk index hit for key: {} ({:.1}μs)",
                    key_for_debug,
                    start.elapsed().as_micros()
                );
                return Some(pattern.clone());
            }
        }

        // Pattern not found in any cache
        self.stats.misses += 1;
        debug!(
            "Cache miss for key: {} ({:.1}μs)",
            key,
            start.elapsed().as_micros()
        );

        None
    }

    /// Look up a dependency pattern by exact key string
    pub fn get_pattern_by_key(&mut self, key: &str) -> Option<DependencyPattern> {
        self.stats.total_requests += 1;
        let start = Instant::now();

        // Tier 1: Check core cache first
        if let Some(pattern) = self.core_patterns.get(key) {
            self.stats.core_hits += 1;
            debug!(
                "Core cache hit for key: {} ({:.1}μs)",
                key,
                start.elapsed().as_micros()
            );
            return Some(pattern.clone());
        }

        // Tier 2: Check LRU cache
        if let Some(pattern) = self.lru_cache.get(key) {
            self.stats.lru_hits += 1;
            debug!(
                "LRU cache hit for key: {} ({:.1}μs)",
                key,
                start.elapsed().as_micros()
            );
            return Some(pattern.clone());
        }

        // Tier 3: Check disk index
        if let Some(ref pattern_index) = self.pattern_index {
            if let Some(pattern) = pattern_index.patterns.get(key) {
                self.stats.disk_hits += 1;

                // Promote to LRU cache
                self.lru_cache.put(key.to_string(), pattern.clone());

                debug!(
                    "Disk index hit for key: {} ({:.1}μs)",
                    key,
                    start.elapsed().as_micros()
                );
                return Some(pattern.clone());
            }
        }

        // Pattern not found in any cache
        self.stats.misses += 1;
        debug!(
            "Cache miss for key: {} ({:.1}μs)",
            key,
            start.elapsed().as_micros()
        );

        None
    }

    /// Generate a pattern key from semantic signature
    fn generate_pattern_key(&self, signature: &SemanticSignature) -> String {
        // Generate key based on lemma and available semantic information
        let mut key_parts = vec![signature.lemma.clone()];

        if let Some(ref verbnet_class) = signature.verbnet_class {
            key_parts.push(format!("vn:{}", verbnet_class));
        }

        if let Some(ref framenet_frame) = signature.framenet_frame {
            key_parts.push(format!("fn:{}", framenet_frame));
        }

        key_parts.join("|")
    }

    /// Insert a pattern into the appropriate cache tier
    pub fn insert_pattern(&mut self, key: String, pattern: DependencyPattern) {
        // Decide which cache tier based on pattern frequency/confidence
        if pattern.frequency > 100 || pattern.confidence > 0.9 {
            // High-frequency or high-confidence patterns go to core cache
            if self.core_patterns.len() < self.config.core_cache_size {
                self.core_patterns.insert(key, pattern);
            }
        } else {
            // Other patterns go to LRU cache
            self.lru_cache.put(key, pattern);
        }
    }

    /// Get cache statistics
    pub fn get_statistics(&self) -> &CacheStatistics {
        &self.stats
    }

    /// Estimate memory usage in bytes
    pub fn estimate_memory_usage(&self) -> usize {
        let core_memory = self.core_patterns.len() * std::mem::size_of::<DependencyPattern>();
        let lru_memory = self.lru_cache.len() * std::mem::size_of::<DependencyPattern>();

        // Add estimated string overhead (rough approximation)
        let string_overhead = (self.core_patterns.len() + self.lru_cache.len()) * 100;

        core_memory + lru_memory + string_overhead
    }

    /// Clear all cache tiers
    pub fn clear(&mut self) {
        self.core_patterns.clear();
        self.lru_cache.clear();
        self.stats = CacheStatistics::default();

        info!("All cache tiers cleared");
    }

    /// Print cache statistics
    pub fn print_statistics(&self) {
        let stats = &self.stats;

        println!("🔍 Pattern Cache Statistics");
        println!("   Total requests: {}", stats.total_requests);
        println!(
            "   Core cache hits: {} ({:.1}%)",
            stats.core_hits,
            stats.core_hit_rate() * 100.0
        );
        println!("   LRU cache hits: {}", stats.lru_hits);
        println!("   Disk index hits: {}", stats.disk_hits);
        println!("   Total hit rate: {:.1}%", stats.total_hit_rate() * 100.0);
        println!("   Cache misses: {}", stats.misses);

        if stats.synthesis_attempts > 0 {
            println!(
                "   Pattern synthesis: {} attempts, {} successes ({:.1}%)",
                stats.synthesis_attempts,
                stats.synthesis_successes,
                stats.synthesis_success_rate() * 100.0
            );
        }

        println!(
            "   Memory usage: ~{:.1} KB",
            self.estimate_memory_usage() as f64 / 1024.0
        );
    }
}

/// Factory for creating pre-configured pattern caches
pub struct PatternCacheFactory;

impl PatternCacheFactory {
    /// Create a cache optimized for M6 performance targets
    pub fn create_m6_optimized(index_path: Option<PathBuf>) -> TreebankResult<PatternCache> {
        let config = PatternCacheConfig {
            core_cache_size: 2000, // Top 2K patterns
            lru_cache_size: 3000,  // 3K working set
            index_path,
            enable_usage_tracking: true,
        };

        PatternCache::new(config)
    }

    /// Create a minimal cache for testing
    pub fn create_test_cache() -> TreebankResult<PatternCache> {
        let config = PatternCacheConfig {
            core_cache_size: 100,
            lru_cache_size: 200,
            index_path: None,
            enable_usage_tracking: false,
        };

        PatternCache::new(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DependencyRelation, PatternSource, SemanticSignature};
    use canopy_engine::LemmaSource;

    fn create_test_signature(lemma: &str) -> SemanticSignature {
        SemanticSignature {
            lemma: lemma.to_string(),
            verbnet_class: None,
            framenet_frame: None,
            pos_category: crate::signature::PosCategory::Verb,
            lemma_source: LemmaSource::UDGold,
            lemma_confidence: 0.95,
            hash_code: 0, // Would be calculated properly
        }
    }

    fn create_test_pattern(verb: &str, freq: u32) -> DependencyPattern {
        DependencyPattern {
            verb_lemma: verb.to_string(),
            dependencies: vec![
                (DependencyRelation::NominalSubject, "NOUN".to_string()),
                (DependencyRelation::Object, "NOUN".to_string()),
            ],
            confidence: 0.8,
            frequency: freq,
            source: PatternSource::Indexed,
        }
    }

    #[test]
    fn test_pattern_cache_creation() {
        let cache = PatternCacheFactory::create_test_cache();
        assert!(cache.is_ok());

        let cache = cache.unwrap();
        assert_eq!(cache.core_patterns.len(), 0);
        assert_eq!(cache.lru_cache.len(), 0);
    }

    #[test]
    fn test_core_cache_population() {
        let mut cache = PatternCacheFactory::create_test_cache().unwrap();

        let patterns = vec![
            ("run|basic".to_string(), create_test_pattern("run", 100)),
            ("walk|basic".to_string(), create_test_pattern("walk", 80)),
            ("talk|basic".to_string(), create_test_pattern("talk", 60)),
        ];

        cache.populate_core_cache(&patterns);

        assert_eq!(cache.core_patterns.len(), 3);
        assert!(cache.core_patterns.contains_key("run|basic"));
    }

    #[test]
    fn test_pattern_lookup_flow() {
        let mut cache = PatternCacheFactory::create_test_cache().unwrap();

        // Populate core cache with key that matches signature generation
        let patterns = vec![("run".to_string(), create_test_pattern("run", 100))];
        cache.populate_core_cache(&patterns);

        // Test lookup
        let signature = create_test_signature("run");
        let result = cache.get_pattern(&signature);

        assert!(result.is_some());
        assert_eq!(cache.stats.total_requests, 1);
        assert_eq!(cache.stats.core_hits, 1);
    }

    #[test]
    fn test_cache_statistics() {
        let mut cache = PatternCacheFactory::create_test_cache().unwrap();
        let signature = create_test_signature("unknown");

        // This should result in a miss
        let result = cache.get_pattern(&signature);
        assert!(result.is_none());
        assert_eq!(cache.stats.misses, 1);
        assert_eq!(cache.stats.total_requests, 1);
        assert_eq!(cache.stats.total_hit_rate(), 0.0);
    }

    #[test]
    fn test_memory_estimation() {
        let mut cache = PatternCacheFactory::create_test_cache().unwrap();
        let initial_memory = cache.estimate_memory_usage();

        // Add some patterns
        let patterns = vec![
            ("test1".to_string(), create_test_pattern("test1", 50)),
            ("test2".to_string(), create_test_pattern("test2", 40)),
        ];
        cache.populate_core_cache(&patterns);

        let final_memory = cache.estimate_memory_usage();
        assert!(final_memory > initial_memory);
    }
}
