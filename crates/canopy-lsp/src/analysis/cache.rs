//! Analysis caching for performance
//!
//! Caches per-sentence analysis results to avoid redundant computation.

use canopy::CanopyError;
use canopy_resources::CanopyPipeline;
use canopy_resources::SemanticAnalysis;
use lru::LruCache;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::num::NonZeroUsize;
use std::sync::Mutex;
use std::time::Instant;

/// Hash of a sentence for cache lookup.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct SentenceHash(u64);

impl SentenceHash {
    fn from_text(text: &str) -> Self {
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        Self(hasher.finish())
    }
}

/// Cached analysis result.
struct CachedAnalysis {
    analysis: SemanticAnalysis,
    #[allow(dead_code)]
    computed_at: Instant,
}

/// LRU cache for sentence analysis results.
pub struct AnalysisCache {
    /// Per-sentence analysis cache.
    cache: Mutex<LruCache<SentenceHash, CachedAnalysis>>,
    /// Cache hit counter.
    hits: Mutex<u64>,
    /// Cache miss counter.
    misses: Mutex<u64>,
}

impl Default for AnalysisCache {
    fn default() -> Self {
        Self::new(1000)
    }
}

impl AnalysisCache {
    /// Create a new analysis cache with the given capacity.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: Mutex::new(LruCache::new(
                NonZeroUsize::new(capacity).unwrap_or(NonZeroUsize::new(1).unwrap()),
            )),
            hits: Mutex::new(0),
            misses: Mutex::new(0),
        }
    }

    /// Get or compute analysis for a sentence.
    pub fn get_or_analyze(
        &self,
        sentence: &str,
        pipeline: &CanopyPipeline,
    ) -> Result<SemanticAnalysis, CanopyError> {
        let hash = SentenceHash::from_text(sentence);

        // Check cache first
        {
            let mut cache = self.cache.lock().unwrap();
            if let Some(cached) = cache.get(&hash) {
                *self.hits.lock().unwrap() += 1;
                return Ok(cached.analysis.clone());
            }
        }

        // Cache miss - compute analysis
        *self.misses.lock().unwrap() += 1;
        let analysis = pipeline.analyze(sentence)?;

        // Store in cache
        {
            let mut cache = self.cache.lock().unwrap();
            cache.put(
                hash,
                CachedAnalysis {
                    analysis: analysis.clone(),
                    computed_at: Instant::now(),
                },
            );
        }

        Ok(analysis)
    }

    /// Get cache statistics.
    #[must_use]
    pub fn stats(&self) -> CacheStats {
        let hits = *self.hits.lock().unwrap();
        let misses = *self.misses.lock().unwrap();
        let size = self.cache.lock().unwrap().len();

        CacheStats { hits, misses, size }
    }

    /// Clear the cache.
    pub fn clear(&self) {
        self.cache.lock().unwrap().clear();
        *self.hits.lock().unwrap() = 0;
        *self.misses.lock().unwrap() = 0;
    }
}

/// Cache statistics.
#[derive(Debug, Clone, Copy)]
pub struct CacheStats {
    /// Number of cache hits.
    pub hits: u64,
    /// Number of cache misses.
    pub misses: u64,
    /// Current cache size.
    pub size: usize,
}

impl CacheStats {
    /// Calculate hit rate as a percentage.
    #[must_use]
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            (self.hits as f64 / total as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sentence_hash() {
        let h1 = SentenceHash::from_text("Hello world.");
        let h2 = SentenceHash::from_text("Hello world.");
        let h3 = SentenceHash::from_text("Different text.");

        assert_eq!(h1, h2);
        assert_ne!(h1, h3);
    }

    #[test]
    fn test_sentence_hash_empty() {
        let h1 = SentenceHash::from_text("");
        let h2 = SentenceHash::from_text("");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_sentence_hash_whitespace_matters() {
        let h1 = SentenceHash::from_text("Hello world.");
        let h2 = SentenceHash::from_text("Hello  world.");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_cache_stats() {
        let stats = CacheStats {
            hits: 80,
            misses: 20,
            size: 100,
        };

        assert!((stats.hit_rate() - 80.0).abs() < 0.001);
    }

    #[test]
    fn test_cache_stats_empty() {
        let stats = CacheStats {
            hits: 0,
            misses: 0,
            size: 0,
        };

        assert!((stats.hit_rate() - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_cache_stats_all_hits() {
        let stats = CacheStats {
            hits: 100,
            misses: 0,
            size: 50,
        };

        assert!((stats.hit_rate() - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_cache_stats_all_misses() {
        let stats = CacheStats {
            hits: 0,
            misses: 100,
            size: 100,
        };

        assert!((stats.hit_rate() - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_cache_creation() {
        let cache = AnalysisCache::new(100);
        let stats = cache.stats();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.size, 0);
    }

    #[test]
    fn test_cache_default() {
        let cache = AnalysisCache::default();
        let stats = cache.stats();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.size, 0);
    }

    #[test]
    fn test_cache_clear() {
        let cache = AnalysisCache::new(100);
        // Manually increment counters for testing
        *cache.hits.lock().unwrap() = 50;
        *cache.misses.lock().unwrap() = 25;

        let stats_before = cache.stats();
        assert_eq!(stats_before.hits, 50);
        assert_eq!(stats_before.misses, 25);

        cache.clear();

        let stats_after = cache.stats();
        assert_eq!(stats_after.hits, 0);
        assert_eq!(stats_after.misses, 0);
        assert_eq!(stats_after.size, 0);
    }

    #[test]
    fn test_cache_zero_capacity_fallback() {
        // Zero capacity should fallback to 1
        let cache = AnalysisCache::new(0);
        let stats = cache.stats();
        assert_eq!(stats.size, 0);
    }

    #[test]
    fn test_cache_stats_debug() {
        let stats = CacheStats {
            hits: 10,
            misses: 5,
            size: 15,
        };
        // Test Debug impl
        let debug_str = format!("{stats:?}");
        assert!(debug_str.contains("hits"));
        assert!(debug_str.contains("10"));
    }

    #[test]
    fn test_cache_stats_clone() {
        let stats = CacheStats {
            hits: 10,
            misses: 5,
            size: 15,
        };
        let cloned = stats;
        assert_eq!(cloned.hits, stats.hits);
        assert_eq!(cloned.misses, stats.misses);
        assert_eq!(cloned.size, stats.size);
    }

    #[test]
    fn test_cache_get_or_analyze() {
        let pipeline = match CanopyPipeline::new() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Skipping test: {e}");
                return;
            }
        };

        let cache = AnalysisCache::new(100);

        // First call should miss
        let result1 = cache.get_or_analyze("The cat runs.", &pipeline);
        assert!(result1.is_ok());

        let stats1 = cache.stats();
        assert_eq!(stats1.misses, 1);
        assert_eq!(stats1.hits, 0);
        assert_eq!(stats1.size, 1);

        // Second call with same sentence should hit
        let result2 = cache.get_or_analyze("The cat runs.", &pipeline);
        assert!(result2.is_ok());

        let stats2 = cache.stats();
        assert_eq!(stats2.misses, 1);
        assert_eq!(stats2.hits, 1);
        assert_eq!(stats2.size, 1);

        // Different sentence should miss
        let result3 = cache.get_or_analyze("The dog barks.", &pipeline);
        assert!(result3.is_ok());

        let stats3 = cache.stats();
        assert_eq!(stats3.misses, 2);
        assert_eq!(stats3.hits, 1);
        assert_eq!(stats3.size, 2);
    }

    #[test]
    fn test_cache_lru_eviction() {
        let pipeline = match CanopyPipeline::new() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Skipping test: {e}");
                return;
            }
        };

        // Small cache that will require eviction
        let cache = AnalysisCache::new(2);

        // Fill cache with 2 entries
        let _ = cache.get_or_analyze("Sentence one.", &pipeline);
        let _ = cache.get_or_analyze("Sentence two.", &pipeline);

        assert_eq!(cache.stats().size, 2);

        // Add third entry - should evict LRU
        let _ = cache.get_or_analyze("Sentence three.", &pipeline);

        assert_eq!(cache.stats().size, 2);
    }

    #[test]
    fn test_cache_analysis_correctness() {
        let pipeline = match CanopyPipeline::new() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Skipping test: {e}");
                return;
            }
        };

        let cache = AnalysisCache::new(100);
        let sentence = "John gives Mary a book.";

        // Get analysis from cache (miss)
        let result1 = cache.get_or_analyze(sentence, &pipeline).unwrap();

        // Get analysis from cache (hit)
        let result2 = cache.get_or_analyze(sentence, &pipeline).unwrap();

        // Results should be equivalent
        assert_eq!(result1.syntax.tokens.len(), result2.syntax.tokens.len());
        assert_eq!(result1.decompositions.len(), result2.decompositions.len());
    }
}
