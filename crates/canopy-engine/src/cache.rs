//! High-performance caching infrastructure for semantic engines
//!
//! This module provides LRU caching with performance metrics and thread-safety,
//! designed to be used across all semantic engines.

use lru::LruCache;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::hash::Hash;
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Trait for cache keys used in semantic engines
pub trait CacheKey: Clone + Debug + Hash + Eq + Send + Sync {}

/// Blanket implementation for types that satisfy the requirements
impl<T> CacheKey for T where T: Clone + Debug + Hash + Eq + Send + Sync {}

/// High-performance cache with metrics and TTL support
#[derive(Debug)]
pub struct EngineCache<K, V>
where
    K: CacheKey,
    V: Clone + Debug,
{
    /// LRU cache for storing results
    cache: Mutex<LruCache<K, CacheEntry<V>>>,
    /// Cache hit counter
    hits: AtomicU64,
    /// Cache miss counter
    misses: AtomicU64,
    /// Total lookup counter
    total_lookups: AtomicU64,
    /// Cache eviction counter
    evictions: AtomicU64,
    /// TTL for cache entries (optional)
    ttl: Option<Duration>,
}

/// Cache entry with timestamp for TTL support
#[derive(Debug, Clone)]
struct CacheEntry<V> {
    value: V,
    created_at: Instant,
}

impl<V> CacheEntry<V> {
    fn new(value: V) -> Self {
        Self {
            value,
            created_at: Instant::now(),
        }
    }

    fn is_expired(&self, ttl: Duration) -> bool {
        self.created_at.elapsed() > ttl
    }
}

impl<K, V> EngineCache<K, V>
where
    K: CacheKey,
    V: Clone + Debug,
{
    /// Create a new cache with specified capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: Mutex::new(LruCache::new(
                NonZeroUsize::new(capacity).unwrap_or_else(|| NonZeroUsize::new(1000).unwrap()),
            )),
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            total_lookups: AtomicU64::new(0),
            evictions: AtomicU64::new(0),
            ttl: None,
        }
    }

    /// Create a new cache with TTL support
    pub fn with_ttl(capacity: usize, ttl: Duration) -> Self {
        let mut cache = Self::new(capacity);
        cache.ttl = Some(ttl);
        cache
    }

    /// Get an item from the cache
    pub fn get(&self, key: &K) -> Option<V> {
        self.total_lookups.fetch_add(1, Ordering::Relaxed);

        if let Ok(mut cache) = self.cache.lock() {
            if let Some(entry) = cache.get(key) {
                // Check TTL if enabled
                if let Some(ttl) = self.ttl {
                    if entry.is_expired(ttl) {
                        cache.pop(key);
                        self.misses.fetch_add(1, Ordering::Relaxed);
                        return None;
                    }
                }

                self.hits.fetch_add(1, Ordering::Relaxed);
                return Some(entry.value.clone());
            }
        }

        self.misses.fetch_add(1, Ordering::Relaxed);
        None
    }

    /// Insert an item into the cache
    pub fn insert(&self, key: K, value: V) -> Option<V> {
        if let Ok(mut cache) = self.cache.lock() {
            let entry = CacheEntry::new(value);
            let evicted = cache.put(key, entry);

            if evicted.is_some() {
                self.evictions.fetch_add(1, Ordering::Relaxed);
            }

            evicted.map(|e| e.value)
        } else {
            None
        }
    }

    /// Remove an item from the cache
    pub fn remove(&self, key: &K) -> Option<V> {
        if let Ok(mut cache) = self.cache.lock() {
            cache.pop(key).map(|e| e.value)
        } else {
            None
        }
    }

    /// Clear all items from the cache
    pub fn clear(&self) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.clear();
        }

        // Reset counters
        self.hits.store(0, Ordering::Relaxed);
        self.misses.store(0, Ordering::Relaxed);
        self.total_lookups.store(0, Ordering::Relaxed);
        self.evictions.store(0, Ordering::Relaxed);
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let total = self.total_lookups.load(Ordering::Relaxed);
        let evictions = self.evictions.load(Ordering::Relaxed);

        let hit_rate = if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        };

        let size = if let Ok(cache) = self.cache.lock() {
            cache.len()
        } else {
            0
        };

        CacheStats {
            hits,
            misses,
            total_lookups: total,
            hit_rate,
            evictions,
            current_size: size,
            has_ttl: self.ttl.is_some(),
        }
    }

    /// Get current cache size
    pub fn len(&self) -> usize {
        if let Ok(cache) = self.cache.lock() {
            cache.len()
        } else {
            0
        }
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Cleanup expired entries (if TTL is enabled)
    pub fn cleanup_expired(&self) {
        if let Some(ttl) = self.ttl {
            if let Ok(mut cache) = self.cache.lock() {
                let mut expired_keys = Vec::new();

                // Find expired keys
                for (key, entry) in cache.iter() {
                    if entry.is_expired(ttl) {
                        expired_keys.push(key.clone());
                    }
                }

                // Remove expired entries
                for key in expired_keys {
                    cache.pop(&key);
                    self.evictions.fetch_add(1, Ordering::Relaxed);
                }
            }
        }
    }
}

/// Cache performance statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    /// Number of cache hits
    pub hits: u64,
    /// Number of cache misses
    pub misses: u64,
    /// Total number of lookups
    pub total_lookups: u64,
    /// Hit rate (0.0 - 1.0)
    pub hit_rate: f64,
    /// Number of evictions
    pub evictions: u64,
    /// Current cache size
    pub current_size: usize,
    /// Whether TTL is enabled
    pub has_ttl: bool,
}

impl CacheStats {
    /// Create empty cache stats
    pub fn empty() -> Self {
        Self {
            hits: 0,
            misses: 0,
            total_lookups: 0,
            hit_rate: 0.0,
            evictions: 0,
            current_size: 0,
            has_ttl: false,
        }
    }

    /// Miss rate (1.0 - hit_rate)
    pub fn miss_rate(&self) -> f64 {
        1.0 - self.hit_rate
    }

    /// Check if cache is performing well (>= 70% hit rate)
    pub fn is_performing_well(&self) -> bool {
        self.hit_rate >= 0.7
    }
}

/// Multi-level cache for hierarchical caching
#[derive(Debug)]
pub struct MultiLevelCache<K, V>
where
    K: CacheKey,
    V: Clone + Debug,
{
    /// L1 cache (small, fast)
    l1_cache: EngineCache<K, V>,
    /// L2 cache (larger, slower)
    l2_cache: EngineCache<K, V>,
}

impl<K, V> MultiLevelCache<K, V>
where
    K: CacheKey,
    V: Clone + Debug,
{
    /// Create a new multi-level cache
    pub fn new(l1_capacity: usize, l2_capacity: usize) -> Self {
        Self {
            l1_cache: EngineCache::new(l1_capacity),
            l2_cache: EngineCache::new(l2_capacity),
        }
    }

    /// Get an item from the cache (checks L1 first, then L2)
    pub fn get(&self, key: &K) -> Option<V> {
        // Check L1 first
        if let Some(value) = self.l1_cache.get(key) {
            return Some(value);
        }

        // Check L2 and promote to L1 if found
        if let Some(value) = self.l2_cache.get(key) {
            self.l1_cache.insert(key.clone(), value.clone());
            return Some(value);
        }

        None
    }

    /// Insert an item into the cache
    pub fn insert(&self, key: K, value: V) {
        // Insert into both levels
        self.l1_cache.insert(key.clone(), value.clone());
        self.l2_cache.insert(key, value);
    }

    /// Get combined cache statistics
    pub fn stats(&self) -> MultiLevelCacheStats {
        MultiLevelCacheStats {
            l1_stats: self.l1_cache.stats(),
            l2_stats: self.l2_cache.stats(),
        }
    }
}

/// Statistics for multi-level cache
#[derive(Debug, Clone)]
pub struct MultiLevelCacheStats {
    /// L1 cache statistics
    pub l1_stats: CacheStats,
    /// L2 cache statistics
    pub l2_stats: CacheStats,
}

impl MultiLevelCacheStats {
    /// Overall hit rate across both levels
    pub fn overall_hit_rate(&self) -> f64 {
        let total_hits = self.l1_stats.hits + self.l2_stats.hits;
        let total_lookups = self.l1_stats.total_lookups + self.l2_stats.total_lookups;

        if total_lookups == 0 {
            0.0
        } else {
            total_hits as f64 / total_lookups as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_cache_basic_operations() {
        let cache: EngineCache<String, i32> = EngineCache::new(3);

        // Test insertion and retrieval
        cache.insert("key1".to_string(), 100);
        assert_eq!(cache.get(&"key1".to_string()), Some(100));

        // Test miss
        assert_eq!(cache.get(&"key2".to_string()), None);

        // Test statistics
        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.total_lookups, 2);
        assert_eq!(stats.hit_rate, 0.5);
    }

    #[test]
    fn test_cache_ttl() {
        let cache: EngineCache<String, i32> = EngineCache::with_ttl(3, Duration::from_millis(100));

        cache.insert("key1".to_string(), 100);
        assert_eq!(cache.get(&"key1".to_string()), Some(100));

        // Wait for TTL to expire
        thread::sleep(Duration::from_millis(150));
        assert_eq!(cache.get(&"key1".to_string()), None);
    }

    #[test]
    fn test_multi_level_cache() {
        let cache: MultiLevelCache<String, i32> = MultiLevelCache::new(2, 5);

        cache.insert("key1".to_string(), 100);
        assert_eq!(cache.get(&"key1".to_string()), Some(100));

        let stats = cache.stats();
        assert!(stats.overall_hit_rate() > 0.0);
    }
}
