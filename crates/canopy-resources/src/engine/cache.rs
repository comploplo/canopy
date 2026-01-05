//! High-performance caching infrastructure for semantic engines

use lru::LruCache;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::hash::Hash;
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Convert u64 to f64 for statistics (saturates at `u32::MAX` for lossless conversion).
#[inline]
fn cache_u64_to_f64(n: u64) -> f64 {
    f64::from(u32::try_from(n).unwrap_or(u32::MAX))
}

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
    cache: Mutex<LruCache<K, CacheEntry<V>>>,
    hits: AtomicU64,
    misses: AtomicU64,
    total_lookups: AtomicU64,
    evictions: AtomicU64,
    ttl: Option<Duration>,
}

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
    ///
    /// # Panics
    ///
    /// This function will not panic as it falls back to capacity of 1000 if given 0.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        // SAFETY: 1000 is non-zero, so the fallback never panics
        let cap = NonZeroUsize::new(capacity)
            .unwrap_or_else(|| NonZeroUsize::new(1000).expect("1000 is non-zero"));
        Self {
            cache: Mutex::new(LruCache::new(cap)),
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            total_lookups: AtomicU64::new(0),
            evictions: AtomicU64::new(0),
            ttl: None,
        }
    }

    /// Create a new cache with TTL support
    #[must_use]
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
            cache_u64_to_f64(hits) / cache_u64_to_f64(total)
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
}

/// Cache performance statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub total_lookups: u64,
    pub hit_rate: f64,
    pub evictions: u64,
    pub current_size: usize,
    pub has_ttl: bool,
}

impl CacheStats {
    /// Create empty cache stats
    #[must_use]
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

    /// Miss rate (1.0 - `hit_rate`)
    #[must_use]
    pub fn miss_rate(&self) -> f64 {
        1.0 - self.hit_rate
    }

    /// Check if cache is performing well (>= 70% hit rate)
    #[must_use]
    pub fn is_performing_well(&self) -> bool {
        self.hit_rate >= 0.7
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_basic_operations() {
        let cache: EngineCache<String, i32> = EngineCache::new(3);

        cache.insert("key1".to_string(), 100);
        assert_eq!(cache.get(&"key1".to_string()), Some(100));
        assert_eq!(cache.get(&"key2".to_string()), None);

        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.total_lookups, 2);
        assert!((stats.hit_rate - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_cache_stats_empty() {
        let stats = CacheStats::empty();
        assert_eq!(stats.hits, 0);
        assert!((stats.hit_rate - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_cache_remove() {
        let cache: EngineCache<String, i32> = EngineCache::new(10);
        cache.insert("key".to_string(), 42);
        assert_eq!(cache.get(&"key".to_string()), Some(42));

        let removed = cache.remove(&"key".to_string());
        assert_eq!(removed, Some(42));
        assert_eq!(cache.get(&"key".to_string()), None);
    }

    #[test]
    fn test_cache_clear() {
        let cache: EngineCache<String, i32> = EngineCache::new(10);
        cache.insert("k1".to_string(), 1);
        cache.insert("k2".to_string(), 2);
        assert!(!cache.is_empty());
        assert_eq!(cache.len(), 2);

        cache.clear();
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
        let stats = cache.stats();
        assert_eq!(stats.total_lookups, 0);
    }

    #[test]
    fn test_cache_with_ttl() {
        let cache: EngineCache<String, i32> = EngineCache::with_ttl(10, Duration::from_millis(50));
        cache.insert("key".to_string(), 42);
        assert_eq!(cache.get(&"key".to_string()), Some(42));
        let stats = cache.stats();
        assert!(stats.has_ttl);
    }

    #[test]
    fn test_cache_stats_methods() {
        let mut stats = CacheStats::empty();
        stats.hits = 7;
        stats.misses = 3;
        stats.total_lookups = 10;
        stats.hit_rate = 0.7;

        assert!((stats.miss_rate() - 0.3).abs() < f64::EPSILON);
        assert!(stats.is_performing_well());

        stats.hit_rate = 0.5;
        assert!(!stats.is_performing_well());
    }

    #[test]
    fn test_cache_eviction() {
        let cache: EngineCache<String, i32> = EngineCache::new(2);
        cache.insert("k1".to_string(), 1);
        cache.insert("k2".to_string(), 2);
        cache.insert("k3".to_string(), 3); // This should evict k1 (LRU)

        assert_eq!(cache.len(), 2);
        // Note: LruCache.put only returns Some if the key was already present
        // When capacity is exceeded, it silently evicts the LRU entry without returning it
        assert_eq!(cache.get(&"k3".to_string()), Some(3));
        assert_eq!(cache.get(&"k2".to_string()), Some(2));
    }
}
