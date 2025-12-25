//! Adaptive caching for treebank pattern lookup
//!
//! This module provides a multi-tier caching system for dependency patterns:
//! 1. Core patterns cache (most frequent patterns, ~500KB)
//! 2. Adaptive LRU cache (runtime patterns, ~1MB)
//! 3. Fallback to disk index (rare patterns)

use crate::signature::SemanticSignature;
use crate::types::DependencyPattern;
use crate::{TreebankIndex, TreebankResult};
use lru::LruCache;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::num::NonZeroUsize;
use tracing::{debug, info, warn};

/// Adaptive cache for dependency patterns
#[derive(Debug)]
pub struct AdaptiveCache {
    /// Core patterns (most frequent, always in memory)
    core_patterns: HashMap<SemanticSignature, DependencyPattern>,
    /// LRU cache for runtime patterns
    lru_cache: LruCache<SemanticSignature, DependencyPattern>,
    /// Usage tracking for cache promotion
    usage_counts: HashMap<SemanticSignature, u32>,
    /// Treebank index for fallback lookups
    index: Option<TreebankIndex>,
    /// Cache statistics
    stats: CacheStats,
    /// Configuration
    config: CacheConfig,
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Number of core patterns to keep in memory
    pub core_capacity: usize,
    /// LRU cache capacity
    pub lru_capacity: usize,
    /// Usage threshold for promoting to LRU cache
    pub promotion_threshold: u32,
    /// Memory budget in bytes
    pub memory_budget_bytes: usize,
    /// Enable detailed logging
    pub verbose: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            core_capacity: 500,             // ~500KB for core patterns
            lru_capacity: 1000,             // ~1MB for LRU cache
            promotion_threshold: 3,         // Promote after 3 uses
            memory_budget_bytes: 2_000_000, // 2MB total
            verbose: false,
        }
    }
}

/// Cache performance statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CacheStats {
    /// Total lookups performed
    pub total_lookups: u64,
    /// Core cache hits
    pub core_hits: u64,
    /// LRU cache hits
    pub lru_hits: u64,
    /// Index lookups (cache misses)
    pub index_lookups: u64,
    /// Patterns promoted to LRU cache
    pub promotions: u64,
    /// Estimated memory usage in bytes
    pub estimated_memory_bytes: usize,
}

impl CacheStats {
    /// Calculate cache hit rate
    pub fn hit_rate(&self) -> f64 {
        if self.total_lookups == 0 {
            0.0
        } else {
            (self.core_hits + self.lru_hits) as f64 / self.total_lookups as f64
        }
    }

    /// Calculate core cache hit rate
    pub fn core_hit_rate(&self) -> f64 {
        if self.total_lookups == 0 {
            0.0
        } else {
            self.core_hits as f64 / self.total_lookups as f64
        }
    }

    /// Calculate memory utilization
    pub fn memory_utilization(&self, budget: usize) -> f64 {
        if budget == 0 {
            0.0
        } else {
            self.estimated_memory_bytes as f64 / budget as f64
        }
    }
}

impl AdaptiveCache {
    /// Create a new adaptive cache
    pub fn new(config: CacheConfig) -> Self {
        let lru_capacity =
            NonZeroUsize::new(config.lru_capacity).unwrap_or(NonZeroUsize::new(1000).unwrap());

        Self {
            core_patterns: HashMap::with_capacity(config.core_capacity),
            lru_cache: LruCache::new(lru_capacity),
            usage_counts: HashMap::new(),
            index: None,
            stats: CacheStats::default(),
            config,
        }
    }

    /// Initialize cache with core patterns from index
    pub fn initialize_with_index(&mut self, index: TreebankIndex) -> TreebankResult<()> {
        info!("Initializing cache with treebank index");

        // Extract top N most frequent patterns for core cache
        let top_patterns = index.get_top_patterns(self.config.core_capacity);

        for pattern in top_patterns {
            // Create signature for the pattern (simplified since we don't have full semantic data)
            let signature = crate::signature::SemanticSignature::simple(
                pattern.verb_lemma.clone(),
                crate::signature::PosCategory::Verb,
            );

            self.core_patterns.insert(signature, pattern.clone());
        }

        // Store index for fallback lookups
        self.index = Some(index);

        // Update memory estimate
        self.update_memory_estimate();

        info!(
            "Initialized cache with {} core patterns, estimated memory: {} KB",
            self.core_patterns.len(),
            self.stats.estimated_memory_bytes / 1024
        );

        Ok(())
    }

    /// Look up a pattern by semantic signature
    pub fn get_pattern(&mut self, signature: &SemanticSignature) -> Option<DependencyPattern> {
        self.stats.total_lookups += 1;

        // 1. Check core patterns first (fastest)
        if let Some(pattern) = self.core_patterns.get(signature) {
            self.stats.core_hits += 1;
            if self.config.verbose {
                debug!("Core cache hit for '{}'", signature.lemma);
            }
            return Some(pattern.clone());
        }

        // 2. Check LRU cache
        if let Some(pattern) = self.lru_cache.get(signature) {
            self.stats.lru_hits += 1;
            if self.config.verbose {
                debug!("LRU cache hit for '{}'", signature.lemma);
            }
            return Some(pattern.clone());
        }

        // 3. Fallback to index lookup
        if let Some(ref index) = self.index {
            if let Some(pattern) = index.get_pattern(signature) {
                self.stats.index_lookups += 1;
                let pattern_clone = pattern.clone();

                // Track usage for potential promotion
                *self.usage_counts.entry(signature.clone()).or_insert(0) += 1;

                // Check if pattern should be promoted to LRU cache
                if self.should_promote(signature) {
                    self.promote_to_lru_cache(signature.clone(), pattern_clone.clone());
                }

                if self.config.verbose {
                    debug!(
                        "Index lookup for '{}' (usage={})",
                        signature.lemma, self.usage_counts[signature]
                    );
                }

                return Some(pattern_clone);
            }
        }

        // Pattern not found anywhere
        if self.config.verbose {
            debug!("Pattern not found for '{}'", signature.lemma);
        }
        None
    }

    /// Look up patterns with fallback variants
    pub fn get_pattern_with_fallback(
        &mut self,
        signature: &SemanticSignature,
        variants: &[SemanticSignature],
    ) -> Option<DependencyPattern> {
        // Try primary signature first
        if let Some(pattern) = self.get_pattern(signature) {
            return Some(pattern);
        }

        // Try variants in order of priority
        for variant in variants {
            if let Some(pattern) = self.get_pattern(variant) {
                // Mark as fallback pattern
                let mut fallback_pattern = pattern;
                fallback_pattern.confidence *= 0.8; // Reduce confidence for fallback
                return Some(fallback_pattern);
            }
        }

        None
    }

    /// Check if pattern should be promoted to LRU cache
    fn should_promote(&self, signature: &SemanticSignature) -> bool {
        self.usage_counts
            .get(signature)
            .is_some_and(|&count| count >= self.config.promotion_threshold)
    }

    /// Promote pattern to LRU cache
    fn promote_to_lru_cache(&mut self, signature: SemanticSignature, pattern: DependencyPattern) {
        self.lru_cache.put(signature.clone(), pattern);
        self.stats.promotions += 1;
        self.update_memory_estimate();

        if self.config.verbose {
            debug!(
                "Promoted '{}' to LRU cache (usage={})",
                signature.lemma,
                self.usage_counts.get(&signature).unwrap_or(&0)
            );
        }
    }

    /// Force add pattern to LRU cache
    pub fn cache_pattern(&mut self, signature: SemanticSignature, pattern: DependencyPattern) {
        self.lru_cache.put(signature, pattern);
        self.update_memory_estimate();
    }

    /// Get cache statistics
    pub fn get_stats(&self) -> &CacheStats {
        &self.stats
    }

    /// Clear all caches (keep core patterns)
    pub fn clear_caches(&mut self) {
        self.lru_cache.clear();
        self.usage_counts.clear();
        self.stats.lru_hits = 0;
        self.stats.index_lookups = 0;
        self.stats.promotions = 0;
        self.update_memory_estimate();

        info!("Cleared LRU cache and usage tracking");
    }

    /// Check if cache is over memory budget
    pub fn is_over_budget(&self) -> bool {
        self.stats.estimated_memory_bytes > self.config.memory_budget_bytes
    }

    /// Get memory pressure level (0.0 to 1.0+)
    pub fn memory_pressure(&self) -> f64 {
        self.stats.estimated_memory_bytes as f64 / self.config.memory_budget_bytes as f64
    }

    /// Perform memory cleanup if over budget
    pub fn cleanup_if_needed(&mut self) -> bool {
        if self.is_over_budget() {
            let before_memory = self.stats.estimated_memory_bytes;

            // Clear half of LRU cache
            let current_len = self.lru_cache.len();
            let target_len = current_len / 2;

            while self.lru_cache.len() > target_len {
                self.lru_cache.pop_lru();
            }

            // Clear old usage counts
            self.usage_counts.clear();

            self.update_memory_estimate();

            warn!(
                "Memory cleanup: reduced from {} KB to {} KB",
                before_memory / 1024,
                self.stats.estimated_memory_bytes / 1024
            );

            true
        } else {
            false
        }
    }

    /// Update estimated memory usage
    fn update_memory_estimate(&mut self) {
        // Rough estimates:
        // - Core pattern: ~1KB each (signature + pattern data)
        // - LRU pattern: ~1KB each
        // - Usage count entry: ~100 bytes each

        let core_memory = self.core_patterns.len() * 1024;
        let lru_memory = self.lru_cache.len() * 1024;
        let usage_memory = self.usage_counts.len() * 100;

        self.stats.estimated_memory_bytes = core_memory + lru_memory + usage_memory;
    }

    /// Get detailed memory breakdown
    pub fn memory_breakdown(&self) -> HashMap<String, usize> {
        let mut breakdown = HashMap::new();
        breakdown.insert("core_patterns".to_string(), self.core_patterns.len() * 1024);
        breakdown.insert("lru_cache".to_string(), self.lru_cache.len() * 1024);
        breakdown.insert("usage_counts".to_string(), self.usage_counts.len() * 100);
        breakdown.insert("total".to_string(), self.stats.estimated_memory_bytes);
        breakdown
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signature::{PosCategory, SemanticSignature};
    use crate::types::{DependencyPattern, DependencyRelation, PatternSource};

    fn create_test_pattern(lemma: &str, frequency: u32) -> DependencyPattern {
        DependencyPattern::new(
            lemma.to_string(),
            vec![(DependencyRelation::NominalSubject, "agent".to_string())],
            0.8,
            frequency,
            PatternSource::Indexed,
        )
    }

    fn create_test_signature(lemma: &str) -> SemanticSignature {
        SemanticSignature::simple(lemma.to_string(), PosCategory::Verb)
    }

    #[test]
    fn test_cache_creation() {
        let config = CacheConfig::default();
        let cache = AdaptiveCache::new(config);

        assert_eq!(cache.core_patterns.len(), 0);
        assert_eq!(cache.lru_cache.len(), 0);
        assert_eq!(cache.stats.total_lookups, 0);
    }

    #[test]
    fn test_cache_stats() {
        let stats = CacheStats {
            total_lookups: 100,
            core_hits: 60,
            lru_hits: 20,
            index_lookups: 20,
            ..Default::default()
        };

        assert_eq!(stats.hit_rate(), 0.8); // (60 + 20) / 100
        assert_eq!(stats.core_hit_rate(), 0.6); // 60 / 100
    }

    #[test]
    fn test_memory_utilization() {
        let stats = CacheStats {
            estimated_memory_bytes: 1_000_000, // 1MB
            ..Default::default()
        };

        assert_eq!(stats.memory_utilization(2_000_000), 0.5); // 1MB / 2MB
    }

    #[test]
    fn test_pattern_lookup() {
        let mut cache = AdaptiveCache::new(CacheConfig::default());

        let signature = create_test_signature("run");
        let pattern = create_test_pattern("run", 10);

        // Cache pattern
        cache.cache_pattern(signature.clone(), pattern.clone());

        // Look up pattern
        let result = cache.get_pattern(&signature);
        assert!(result.is_some());
        assert_eq!(result.unwrap().verb_lemma, "run");
        assert_eq!(cache.stats.lru_hits, 1);
    }

    #[test]
    fn test_promotion_logic() {
        let config = CacheConfig {
            promotion_threshold: 2,
            ..Default::default()
        };
        let cache = AdaptiveCache::new(config);

        let signature = create_test_signature("test");

        // Below threshold
        assert!(!cache.should_promote(&signature));

        // Create cache with usage tracking
        let mut cache = AdaptiveCache::new(CacheConfig {
            promotion_threshold: 2,
            ..Default::default()
        });
        cache.usage_counts.insert(signature.clone(), 1);
        assert!(!cache.should_promote(&signature));

        cache.usage_counts.insert(signature.clone(), 2);
        assert!(cache.should_promote(&signature));
    }

    #[test]
    fn test_memory_cleanup() {
        let mut cache = AdaptiveCache::new(CacheConfig {
            memory_budget_bytes: 1000, // Very small budget
            ..Default::default()
        });

        // Add patterns to exceed budget
        for i in 0..10 {
            let signature = create_test_signature(&format!("verb{}", i));
            let pattern = create_test_pattern(&format!("verb{}", i), 1);
            cache.cache_pattern(signature, pattern);
        }

        assert!(cache.is_over_budget());
        assert!(cache.memory_pressure() > 1.0);

        let cleaned = cache.cleanup_if_needed();
        assert!(cleaned);
        assert!(cache.lru_cache.len() < 10);
    }

    #[test]
    fn test_fallback_pattern_lookup() {
        let mut cache = AdaptiveCache::new(CacheConfig::default());

        let primary_sig = create_test_signature("run");
        let fallback_sig = create_test_signature("jog");
        let pattern = create_test_pattern("jog", 10);

        // Cache fallback pattern only
        cache.cache_pattern(fallback_sig.clone(), pattern.clone());

        // Look up with fallback
        let result = cache.get_pattern_with_fallback(&primary_sig, &[fallback_sig]);
        assert!(result.is_some());

        let found_pattern = result.unwrap();
        assert_eq!(found_pattern.verb_lemma, "jog");
        // Should have reduced confidence for fallback
        assert!(found_pattern.confidence < pattern.confidence);
    }

    #[test]
    fn test_memory_breakdown() {
        let mut cache = AdaptiveCache::new(CacheConfig::default());

        // Add some patterns
        for i in 0..5 {
            let signature = create_test_signature(&format!("verb{}", i));
            let pattern = create_test_pattern(&format!("verb{}", i), 1);
            cache.cache_pattern(signature, pattern);
        }

        let breakdown = cache.memory_breakdown();
        assert!(breakdown.contains_key("core_patterns"));
        assert!(breakdown.contains_key("lru_cache"));
        assert!(breakdown.contains_key("usage_counts"));
        assert!(breakdown.contains_key("total"));

        assert_eq!(breakdown["lru_cache"], 5 * 1024); // 5 patterns * 1KB each
    }

    #[test]
    fn test_cache_clearing() {
        let mut cache = AdaptiveCache::new(CacheConfig::default());

        // Add patterns and usage
        let signature = create_test_signature("test");
        let pattern = create_test_pattern("test", 1);
        cache.cache_pattern(signature.clone(), pattern);
        cache.usage_counts.insert(signature, 5);

        // Clear caches
        cache.clear_caches();

        assert_eq!(cache.lru_cache.len(), 0);
        assert_eq!(cache.usage_counts.len(), 0);
        assert_eq!(cache.stats.lru_hits, 0);
    }
}
