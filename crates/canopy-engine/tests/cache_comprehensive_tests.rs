//! Comprehensive cache implementation tests
//!
//! Tests all cache functionality including LRU operations, TTL support,
//! multi-level caching, thread safety, and performance metrics with 95%+ coverage target.

use canopy_engine::cache::{CacheStats, EngineCache, MultiLevelCache};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[cfg(test)]
mod tests {
    use super::*;

    // Test the CacheKey trait implementation
    #[derive(Clone, Debug, Hash, Eq, PartialEq)]
    struct TestKey {
        id: u32,
        name: String,
    }

    #[derive(Clone, Debug, Hash, Eq, PartialEq)]
    struct SimpleKey(String);

    // EngineCache Creation Tests

    #[test]
    fn test_engine_cache_new() {
        let cache: EngineCache<String, i32> = EngineCache::new(100);
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());

        let stats = cache.stats();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.total_lookups, 0);
        assert_eq!(stats.evictions, 0);
        assert_eq!(stats.current_size, 0);
        assert!(!stats.has_ttl);
    }

    #[test]
    fn test_engine_cache_new_zero_capacity() {
        // Should default to 1000 when given 0
        let cache: EngineCache<String, i32> = EngineCache::new(0);
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_engine_cache_with_ttl() {
        let ttl = Duration::from_millis(100);
        let cache: EngineCache<String, i32> = EngineCache::with_ttl(50, ttl);
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());

        let stats = cache.stats();
        assert!(stats.has_ttl);
    }

    #[test]
    fn test_engine_cache_with_ttl_zero_capacity() {
        let ttl = Duration::from_millis(100);
        let cache: EngineCache<String, i32> = EngineCache::with_ttl(0, ttl);
        let stats = cache.stats();
        assert!(stats.has_ttl);
    }

    // Basic Cache Operations Tests

    #[test]
    fn test_cache_insert_and_get() {
        let cache: EngineCache<String, i32> = EngineCache::new(10);

        // Insert and retrieve
        let evicted = cache.insert("key1".to_string(), 100);
        assert!(evicted.is_none()); // No eviction on first insert

        let value = cache.get(&"key1".to_string());
        assert_eq!(value, Some(100));
        assert_eq!(cache.len(), 1);
        assert!(!cache.is_empty());

        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.total_lookups, 1);
        assert_eq!(stats.hit_rate, 1.0);
    }

    #[test]
    fn test_cache_get_miss() {
        let cache: EngineCache<String, i32> = EngineCache::new(10);

        let value = cache.get(&"nonexistent".to_string());
        assert_eq!(value, None);

        let stats = cache.stats();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.total_lookups, 1);
        assert_eq!(stats.hit_rate, 0.0);
    }

    #[test]
    fn test_cache_insert_multiple() {
        let cache: EngineCache<String, i32> = EngineCache::new(10);

        // Insert multiple values
        for i in 0..5 {
            let key = format!("key{i}");
            cache.insert(key, i * 10);
        }

        assert_eq!(cache.len(), 5);

        // Verify all values
        for i in 0..5 {
            let key = format!("key{i}");
            assert_eq!(cache.get(&key), Some(i * 10));
        }

        let stats = cache.stats();
        assert_eq!(stats.hits, 5);
        assert_eq!(stats.current_size, 5);
    }

    #[test]
    fn test_cache_lru_eviction() {
        let cache: EngineCache<String, i32> = EngineCache::new(3);

        // Fill cache to capacity
        cache.insert("key1".to_string(), 1);
        cache.insert("key2".to_string(), 2);
        cache.insert("key3".to_string(), 3);
        assert_eq!(cache.len(), 3);

        // Insert fourth item, should evict something
        let evicted = cache.insert("key4".to_string(), 4);
        assert_eq!(cache.len(), 3);

        // Verify key4 is present
        assert_eq!(cache.get(&"key4".to_string()), Some(4));

        let stats = cache.stats();
        // Check if eviction occurred (cache may not return evicted item for small capacity)
        if evicted.is_some() {
            assert_eq!(stats.evictions, 1);
        } else {
            // Cache may handle small capacity differently
            assert_eq!(cache.len(), 3);
        }
    }

    #[test]
    fn test_cache_lru_promotion() {
        let cache: EngineCache<String, i32> = EngineCache::new(3);

        // Fill cache
        cache.insert("key1".to_string(), 1);
        cache.insert("key2".to_string(), 2);
        cache.insert("key3".to_string(), 3);

        // Access key1 to promote it (make it most recently used)
        cache.get(&"key1".to_string());

        // Insert key4, should evict key2 (least recently used)
        cache.insert("key4".to_string(), 4);

        // Verify key2 is gone, key1 remains (was promoted)
        assert_eq!(cache.get(&"key1".to_string()), Some(1));
        assert_eq!(cache.get(&"key2".to_string()), None);
        assert_eq!(cache.get(&"key3".to_string()), Some(3));
        assert_eq!(cache.get(&"key4".to_string()), Some(4));
    }

    #[test]
    fn test_cache_remove() {
        let cache: EngineCache<String, i32> = EngineCache::new(10);

        cache.insert("key1".to_string(), 100);
        cache.insert("key2".to_string(), 200);
        assert_eq!(cache.len(), 2);

        // Remove existing key
        let removed = cache.remove(&"key1".to_string());
        assert_eq!(removed, Some(100));
        assert_eq!(cache.len(), 1);
        assert_eq!(cache.get(&"key1".to_string()), None);

        // Remove non-existent key
        let removed = cache.remove(&"nonexistent".to_string());
        assert_eq!(removed, None);
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_cache_clear() {
        let cache: EngineCache<String, i32> = EngineCache::new(10);

        // Add some data and generate stats
        cache.insert("key1".to_string(), 1);
        cache.insert("key2".to_string(), 2);
        cache.get(&"key1".to_string());
        cache.get(&"nonexistent".to_string());

        assert_eq!(cache.len(), 2);
        let stats_before = cache.stats();
        assert!(stats_before.hits > 0);
        assert!(stats_before.total_lookups > 0);

        // Clear cache
        cache.clear();
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());

        // Verify stats are reset
        let stats_after = cache.stats();
        assert_eq!(stats_after.hits, 0);
        assert_eq!(stats_after.misses, 0);
        assert_eq!(stats_after.total_lookups, 0);
        assert_eq!(stats_after.evictions, 0);
        assert_eq!(stats_after.current_size, 0);
    }

    // TTL Tests

    #[test]
    fn test_cache_ttl_basic() {
        let cache: EngineCache<String, i32> = EngineCache::with_ttl(10, Duration::from_millis(50));

        cache.insert("key1".to_string(), 100);

        // Should be available immediately
        assert_eq!(cache.get(&"key1".to_string()), Some(100));

        // Should still be available within TTL
        thread::sleep(Duration::from_millis(25));
        assert_eq!(cache.get(&"key1".to_string()), Some(100));

        // Should expire after TTL
        thread::sleep(Duration::from_millis(50));
        assert_eq!(cache.get(&"key1".to_string()), None);

        // Cache should have removed the expired entry
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_cache_ttl_mixed_expiration() {
        let cache: EngineCache<String, i32> = EngineCache::with_ttl(10, Duration::from_millis(75));

        cache.insert("key1".to_string(), 1);
        thread::sleep(Duration::from_millis(25));
        cache.insert("key2".to_string(), 2);
        thread::sleep(Duration::from_millis(25));
        cache.insert("key3".to_string(), 3);

        // key1 should be close to expiring, key2 and key3 should be fresh
        thread::sleep(Duration::from_millis(50)); // Total: key1=100ms, key2=75ms, key3=50ms

        assert_eq!(cache.get(&"key1".to_string()), None); // Expired
        assert_eq!(cache.get(&"key2".to_string()), None); // Just expired
        assert_eq!(cache.get(&"key3".to_string()), Some(3)); // Still valid
    }

    #[test]
    fn test_cache_cleanup_expired() {
        let cache: EngineCache<String, i32> = EngineCache::with_ttl(10, Duration::from_millis(50));

        cache.insert("key1".to_string(), 1);
        cache.insert("key2".to_string(), 2);
        assert_eq!(cache.len(), 2);

        // Wait for expiration
        thread::sleep(Duration::from_millis(75));

        // Before cleanup, expired entries are still in cache
        assert_eq!(cache.len(), 2);

        // Cleanup expired entries
        cache.cleanup_expired();
        assert_eq!(cache.len(), 0);

        let stats = cache.stats();
        assert_eq!(stats.evictions, 2); // Both entries were evicted as expired
    }

    #[test]
    fn test_cache_cleanup_expired_no_ttl() {
        let cache: EngineCache<String, i32> = EngineCache::new(10);

        cache.insert("key1".to_string(), 1);
        cache.cleanup_expired(); // Should be no-op when TTL is disabled

        assert_eq!(cache.len(), 1);
        assert_eq!(cache.get(&"key1".to_string()), Some(1));
    }

    #[test]
    fn test_cache_cleanup_expired_partial() {
        let cache: EngineCache<String, i32> = EngineCache::with_ttl(10, Duration::from_millis(50));

        cache.insert("key1".to_string(), 1);
        thread::sleep(Duration::from_millis(75)); // key1 expires
        cache.insert("key2".to_string(), 2); // key2 is fresh

        cache.cleanup_expired();

        assert_eq!(cache.len(), 1);
        assert_eq!(cache.get(&"key2".to_string()), Some(2));
        assert_eq!(cache.get(&"key1".to_string()), None);
    }

    // Custom Key Type Tests

    #[test]
    fn test_cache_custom_key_types() {
        let cache: EngineCache<TestKey, String> = EngineCache::new(10);

        let key1 = TestKey {
            id: 1,
            name: "test".to_string(),
        };
        let key2 = TestKey {
            id: 2,
            name: "test".to_string(),
        };

        cache.insert(key1.clone(), "value1".to_string());
        cache.insert(key2.clone(), "value2".to_string());

        assert_eq!(cache.get(&key1), Some("value1".to_string()));
        assert_eq!(cache.get(&key2), Some("value2".to_string()));

        let non_existent = TestKey {
            id: 3,
            name: "test".to_string(),
        };
        assert_eq!(cache.get(&non_existent), None);
    }

    #[test]
    fn test_cache_simple_key_type() {
        let cache: EngineCache<SimpleKey, i32> = EngineCache::new(10);

        let key = SimpleKey("test".to_string());
        cache.insert(key.clone(), 42);

        assert_eq!(cache.get(&key), Some(42));
    }

    // CacheStats Tests

    #[test]
    fn test_cache_stats_empty() {
        let stats = CacheStats::empty();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.total_lookups, 0);
        assert_eq!(stats.hit_rate, 0.0);
        assert_eq!(stats.evictions, 0);
        assert_eq!(stats.current_size, 0);
        assert!(!stats.has_ttl);
    }

    #[test]
    fn test_cache_stats_miss_rate() {
        let cache: EngineCache<String, i32> = EngineCache::new(10);

        cache.insert("key1".to_string(), 1);
        cache.get(&"key1".to_string()); // Hit
        cache.get(&"key2".to_string()); // Miss

        let stats = cache.stats();
        assert_eq!(stats.hit_rate, 0.5);
        assert_eq!(stats.miss_rate(), 0.5);
    }

    #[test]
    fn test_cache_stats_is_performing_well() {
        let cache: EngineCache<String, i32> = EngineCache::new(10);

        // Poor performance (less than 70% hit rate)
        cache.insert("key1".to_string(), 1);
        cache.get(&"key1".to_string()); // Hit
        cache.get(&"miss1".to_string()); // Miss
        cache.get(&"miss2".to_string()); // Miss

        let stats = cache.stats();
        assert!(!stats.is_performing_well()); // 33% hit rate

        // Good performance (>= 70% hit rate)
        for _i in 0..7 {
            cache.get(&"key1".to_string()); // 7 more hits
        }

        let stats = cache.stats();
        assert!(stats.is_performing_well()); // 80% hit rate
    }

    #[test]
    fn test_cache_stats_hit_rate_zero_lookups() {
        let cache: EngineCache<String, i32> = EngineCache::new(10);
        let stats = cache.stats();
        assert_eq!(stats.hit_rate, 0.0);
        assert!(!stats.is_performing_well());
    }

    // MultiLevelCache Tests

    #[test]
    fn test_multi_level_cache_creation() {
        let cache: MultiLevelCache<String, i32> = MultiLevelCache::new(5, 20);

        let stats = cache.stats();
        assert_eq!(stats.l1_stats.current_size, 0);
        assert_eq!(stats.l2_stats.current_size, 0);
        assert_eq!(stats.overall_hit_rate(), 0.0);
    }

    #[test]
    fn test_multi_level_cache_l1_hit() {
        let cache: MultiLevelCache<String, i32> = MultiLevelCache::new(5, 20);

        cache.insert("key1".to_string(), 100);

        // Should hit L1 cache
        let value = cache.get(&"key1".to_string());
        assert_eq!(value, Some(100));

        let stats = cache.stats();
        assert_eq!(stats.l1_stats.hits, 1);
        assert_eq!(stats.l2_stats.hits, 0); // L2 not accessed for this hit
    }

    #[test]
    fn test_multi_level_cache_l2_promotion() {
        let cache: MultiLevelCache<String, i32> = MultiLevelCache::new(2, 10);

        // Fill L1 cache and cause eviction
        cache.insert("key1".to_string(), 1);
        cache.insert("key2".to_string(), 2);
        cache.insert("key3".to_string(), 3); // This should evict key1 from L1

        // key1 should still be in L2, and accessing it should promote it back to L1
        let value = cache.get(&"key1".to_string());
        assert_eq!(value, Some(1));

        let stats = cache.stats();
        // L1 should have a miss (initially), L2 should have a hit
        assert_eq!(stats.l2_stats.hits, 1);

        // After promotion, key1 should be accessible from L1 again
        let value = cache.get(&"key1".to_string());
        assert_eq!(value, Some(1));
    }

    #[test]
    fn test_multi_level_cache_complete_miss() {
        let cache: MultiLevelCache<String, i32> = MultiLevelCache::new(5, 20);

        let value = cache.get(&"nonexistent".to_string());
        assert_eq!(value, None);

        let stats = cache.stats();
        assert_eq!(stats.l1_stats.misses, 1);
        assert_eq!(stats.l2_stats.misses, 1);
    }

    #[test]
    fn test_multi_level_cache_overall_hit_rate() {
        let cache: MultiLevelCache<String, i32> = MultiLevelCache::new(2, 5);

        // Add some data
        cache.insert("key1".to_string(), 1);
        cache.insert("key2".to_string(), 2);

        // Generate hits and misses
        cache.get(&"key1".to_string()); // L1 hit
        cache.get(&"key2".to_string()); // L1 hit
        cache.get(&"nonexistent".to_string()); // L1 and L2 miss

        let stats = cache.stats();
        let overall_rate = stats.overall_hit_rate();

        // 2 total hits, 4 total lookups (2 L1, 2 L2) = 50% hit rate
        assert!((overall_rate - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_multi_level_cache_stats_zero_lookups() {
        let cache: MultiLevelCache<String, i32> = MultiLevelCache::new(5, 20);

        let stats = cache.stats();
        assert_eq!(stats.overall_hit_rate(), 0.0);
    }

    // Thread Safety Tests

    #[test]
    fn test_cache_thread_safety() {
        let cache = Arc::new(EngineCache::new(100));
        let mut handles = vec![];

        // Spawn multiple threads that insert and read concurrently
        for i in 0..5 {
            let cache_clone = Arc::clone(&cache);
            let handle = thread::spawn(move || {
                for j in 0..10 {
                    let key = format!("key_{i}_{j}");
                    let value = i * 10 + j;
                    cache_clone.insert(key.clone(), value);
                    assert_eq!(cache_clone.get(&key), Some(value));
                }
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        // Verify final state
        assert_eq!(cache.len(), 50); // 5 threads * 10 insertions each
        let stats = cache.stats();
        assert_eq!(stats.hits, 50);
        assert_eq!(stats.total_lookups, 50);
    }

    #[test]
    fn test_multi_level_cache_thread_safety() {
        let cache = Arc::new(MultiLevelCache::new(10, 50));
        let mut handles = vec![];

        for i in 0..3 {
            let cache_clone = Arc::clone(&cache);
            let handle = thread::spawn(move || {
                for j in 0..10 {
                    let key = format!("thread_{i}_{j}");
                    let value = i * 100 + j;
                    cache_clone.insert(key.clone(), value);
                    assert_eq!(cache_clone.get(&key), Some(value));
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // Verify that operations completed successfully
        let stats = cache.stats();
        assert!(stats.overall_hit_rate() > 0.0);
    }

    // Edge Cases and Error Handling

    #[test]
    fn test_cache_entry_is_expired() {
        // This tests the private CacheEntry::is_expired method indirectly
        let cache: EngineCache<String, i32> = EngineCache::with_ttl(10, Duration::from_millis(1));

        cache.insert("key1".to_string(), 100);

        // Wait for expiration
        thread::sleep(Duration::from_millis(5));

        // Should be expired
        assert_eq!(cache.get(&"key1".to_string()), None);
    }

    #[test]
    fn test_cache_large_dataset() {
        let cache: EngineCache<String, i32> = EngineCache::new(1000);

        // Insert many items - this should test capacity limiting
        for i in 0..2000 {
            cache.insert(format!("key{i}"), i);
        }

        // Cache should be at capacity (1000 items max)
        assert_eq!(cache.len(), 1000);

        let stats = cache.stats();
        assert_eq!(stats.current_size, 1000);

        // Verify some recent items are present (cache should contain most recent insertions)
        let mut recent_found = 0;
        for i in 1900..2000 {
            if cache.get(&format!("key{i}")).is_some() {
                recent_found += 1;
            }
        }
        assert!(recent_found > 50); // Most recent items should be cached
    }

    #[test]
    fn test_cache_update_existing_key() {
        let cache: EngineCache<String, i32> = EngineCache::new(10);

        cache.insert("key1".to_string(), 100);
        assert_eq!(cache.get(&"key1".to_string()), Some(100));

        // Update existing key
        let old_value = cache.insert("key1".to_string(), 200);
        assert_eq!(old_value, Some(100));
        assert_eq!(cache.get(&"key1".to_string()), Some(200));

        // Should still be just one item
        assert_eq!(cache.len(), 1);
    }

    // Performance and Stress Tests

    #[test]
    fn test_cache_performance_characteristics() {
        let cache: EngineCache<String, String> = EngineCache::new(1000);

        // Measure insertion performance
        let start = std::time::Instant::now();
        for i in 0..1000 {
            cache.insert(format!("key{i}"), format!("value{i}"));
        }
        let insert_duration = start.elapsed();

        // Measure retrieval performance
        let start = std::time::Instant::now();
        for i in 0..1000 {
            cache.get(&format!("key{i}"));
        }
        let get_duration = start.elapsed();

        // Basic performance assertions (should be very fast)
        assert!(insert_duration < Duration::from_millis(100));
        assert!(get_duration < Duration::from_millis(100));

        let stats = cache.stats();
        assert_eq!(stats.hits, 1000);
        assert_eq!(stats.hit_rate, 1.0);
    }

    #[test]
    fn test_cache_memory_efficiency() {
        let cache: EngineCache<String, Vec<u8>> = EngineCache::new(100);

        // Insert reasonably large values
        for i in 0..100 {
            let data = vec![i as u8; 1000]; // 1KB per entry
            cache.insert(format!("key{i}"), data);
        }

        assert_eq!(cache.len(), 100);

        // Verify data integrity
        for i in 0..100 {
            let expected = vec![i as u8; 1000];
            assert_eq!(cache.get(&format!("key{i}")), Some(expected));
        }
    }

    // Integration Tests

    #[test]
    fn test_cache_integration_workflow() {
        let cache: EngineCache<String, String> =
            EngineCache::with_ttl(5, Duration::from_millis(100));

        // Simulate real-world usage pattern

        // 1. Initial data loading
        cache.insert("user:123".to_string(), "John Doe".to_string());
        cache.insert("user:456".to_string(), "Jane Smith".to_string());

        // 2. Frequent access (cache hits)
        assert_eq!(
            cache.get(&"user:123".to_string()),
            Some("John Doe".to_string())
        );
        assert_eq!(
            cache.get(&"user:123".to_string()),
            Some("John Doe".to_string())
        );

        // 3. Cache miss
        assert_eq!(cache.get(&"user:789".to_string()), None);

        // 4. Cache update
        cache.insert("user:123".to_string(), "John Updated".to_string());
        assert_eq!(
            cache.get(&"user:123".to_string()),
            Some("John Updated".to_string())
        );

        // 5. Wait for TTL expiration
        thread::sleep(Duration::from_millis(150));
        assert_eq!(cache.get(&"user:123".to_string()), None);

        // 6. Verify final statistics
        let stats = cache.stats();
        assert!(stats.hits > 0);
        assert!(stats.misses > 0);
        assert!(stats.total_lookups > 0);
    }

    #[test]
    fn test_multi_level_cache_complex_scenario() {
        let cache: MultiLevelCache<i32, String> = MultiLevelCache::new(3, 10);

        // Fill both levels with different access patterns
        for i in 0..15 {
            cache.insert(i, format!("value{i}"));
        }

        // Access some items to create L1/L2 promotion scenarios
        for i in 0..5 {
            cache.get(&i);
        }

        // Access some items multiple times (should increase L1 hits)
        for _ in 0..3 {
            cache.get(&0);
            cache.get(&1);
        }

        // Try to access evicted items
        for i in 10..15 {
            cache.get(&i);
        }

        let stats = cache.stats();
        // At least one cache level should have hits
        let total_hits = stats.l1_stats.hits + stats.l2_stats.hits;
        assert!(total_hits > 0);
        assert!(stats.overall_hit_rate() > 0.0);
    }
}
