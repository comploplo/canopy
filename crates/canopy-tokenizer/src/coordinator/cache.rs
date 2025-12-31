//! Semantic analysis cache
//!
//! LRU cache for Layer 1 semantic analysis results with Arc-based sharing.

use super::types::Layer1SemanticResult;
use super::SemanticCoordinator;
use canopy_core::UPos;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::Instant;

/// Intelligent cache for semantic analysis results using Arc for efficient sharing
#[derive(Debug)]
pub struct SemanticCache {
    /// Cache stores Arc-wrapped results for zero-copy sharing when possible
    cache: HashMap<String, (Arc<Layer1SemanticResult>, Instant)>,
    access_order: VecDeque<String>,
    capacity: usize,
    hits: usize,
    misses: usize,
    evictions: usize,
}

impl SemanticCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: HashMap::new(),
            access_order: VecDeque::new(),
            capacity,
            hits: 0,
            misses: 0,
            evictions: 0,
        }
    }

    /// Generate optimized cache key for better hit rates
    pub fn generate_key(&self, word: &str, use_lemma_only: bool) -> String {
        if use_lemma_only {
            // Use only lemmatized form for better cache hits on inflected words
            word.to_lowercase()
        } else {
            // Include minimal context for precision
            format!("word:{}", word.to_lowercase())
        }
    }

    /// Generate cache key with optional POS for better cache differentiation
    /// Format: "lemma:Verb" when POS provided, "lemma" otherwise
    pub fn generate_key_with_pos(&self, word: &str, pos: Option<UPos>) -> String {
        let base = word.to_lowercase();
        match pos {
            Some(p) => format!("{}:{:?}", base, p),
            None => base,
        }
    }

    /// Get cached result with LRU updating (returns Arc for zero-copy when possible)
    pub fn get_arc(&mut self, key: &str) -> Option<Arc<Layer1SemanticResult>> {
        if let Some((result, _time)) = self.cache.get(key) {
            // Update access order
            self.access_order.retain(|k| k != key);
            self.access_order.push_back(key.to_string());
            self.hits += 1;
            Some(Arc::clone(result)) // Cheap pointer copy
        } else {
            self.misses += 1;
            None
        }
    }

    /// Get cached result (clones for compatibility, prefer get_arc for efficiency)
    pub fn get(&mut self, key: &str) -> Option<Layer1SemanticResult> {
        self.get_arc(key).map(|arc| (*arc).clone())
    }

    /// Insert result with LRU eviction (wraps in Arc automatically)
    pub fn insert(&mut self, key: String, result: Layer1SemanticResult) {
        self.insert_arc(key, Arc::new(result));
    }

    /// Insert Arc-wrapped result with LRU eviction
    pub fn insert_arc(&mut self, key: String, result: Arc<Layer1SemanticResult>) {
        // Check if already exists
        if self.cache.contains_key(&key) {
            // Update existing entry
            self.access_order.retain(|k| k != &key);
            self.access_order.push_back(key.clone());
            self.cache.insert(key, (result, Instant::now()));
            return;
        }

        // Check capacity and evict oldest if needed
        if self.cache.len() >= self.capacity {
            if let Some(evicted_key) = self.access_order.pop_front() {
                self.cache.remove(&evicted_key);
                self.evictions += 1;
            }
        }

        // Insert new entry
        self.cache.insert(key.clone(), (result, Instant::now()));
        self.access_order.push_back(key);
    }

    /// Preload common words for warmup (uses parallel batch when available)
    pub fn warmup_common_words(&mut self, coordinator: &SemanticCoordinator) {
        let common_words: Vec<String> = [
            "the", "be", "to", "of", "and", "a", "in", "that", "have", "it", "for", "not", "on",
            "with", "he", "as", "you", "do", "at", "this", "but", "his", "by", "from", "they",
            "she", "or", "an", "will", "my", "one", "all", "would", "there", "their", "what", "so",
            "up", "out", "if", "about", "who", "get", "which", "go", "me", "when", "make", "can",
            "like", "time", "no", "just", "him", "know", "take", "people", "into", "year", "your",
            // Common verbs that benefit from semantic analysis
            "run", "walk", "give", "take", "make", "see", "come", "go", "think", "say", "get",
            "want", "use", "find", "work", "call", "try", "ask", "turn", "move",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();

        println!("Warming cache with {} common words...", common_words.len());

        // Use parallel batch analysis for faster warmup
        if let Ok(results) = coordinator.analyze_batch_parallel(&common_words) {
            for (word, result) in common_words.iter().zip(results) {
                let key = self.generate_key(word, true);
                if !self.cache.contains_key(&key) {
                    self.insert(key, result);
                }
            }
        }

        println!("Cache warmed with {} entries", self.cache.len());
    }

    /// Get cache statistics
    pub fn stats(&self) -> (f32, usize, usize, usize) {
        let total = self.hits + self.misses;
        let hit_rate = if total > 0 {
            self.hits as f32 / total as f32
        } else {
            0.0
        };
        (hit_rate, self.hits, self.misses, self.evictions)
    }

    pub fn len(&self) -> usize {
        self.cache.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_result(word: &str) -> Layer1SemanticResult {
        Layer1SemanticResult::new(word.to_string(), word.to_string())
    }

    #[test]
    fn test_semantic_cache_new() {
        let cache = SemanticCache::new(100);
        assert_eq!(cache.capacity, 100);
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_semantic_cache_generate_key_lemma_only() {
        let cache = SemanticCache::new(100);
        let key = cache.generate_key("Running", true);
        assert_eq!(key, "running");
    }

    #[test]
    fn test_semantic_cache_generate_key_with_prefix() {
        let cache = SemanticCache::new(100);
        let key = cache.generate_key("Running", false);
        assert_eq!(key, "word:running");
    }

    #[test]
    fn test_semantic_cache_generate_key_with_pos() {
        let cache = SemanticCache::new(100);
        let key_with_pos = cache.generate_key_with_pos("run", Some(UPos::Verb));
        assert!(key_with_pos.contains("run"));
        assert!(key_with_pos.contains("Verb"));

        let key_without_pos = cache.generate_key_with_pos("run", None);
        assert_eq!(key_without_pos, "run");
    }

    #[test]
    fn test_semantic_cache_insert_and_get() {
        let mut cache = SemanticCache::new(100);
        let result = make_result("test");
        cache.insert("test".to_string(), result.clone());

        assert!(!cache.is_empty());
        assert_eq!(cache.len(), 1);

        let retrieved = cache.get("test");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().original_word, "test");
    }

    #[test]
    fn test_semantic_cache_get_arc() {
        let mut cache = SemanticCache::new(100);
        let result = make_result("arc_test");
        cache.insert("arc".to_string(), result);

        let arc1 = cache.get_arc("arc").unwrap();
        let arc2 = cache.get_arc("arc").unwrap();

        // Both Arcs should point to the same data
        assert_eq!(arc1.original_word, arc2.original_word);
    }

    #[test]
    fn test_semantic_cache_miss() {
        let mut cache = SemanticCache::new(100);
        let result = cache.get("nonexistent");
        assert!(result.is_none());
    }

    #[test]
    fn test_semantic_cache_lru_eviction() {
        let mut cache = SemanticCache::new(3);

        // Insert 3 items
        cache.insert("a".to_string(), make_result("a"));
        cache.insert("b".to_string(), make_result("b"));
        cache.insert("c".to_string(), make_result("c"));

        assert_eq!(cache.len(), 3);

        // Insert 4th item - should evict "a" (oldest)
        cache.insert("d".to_string(), make_result("d"));

        assert_eq!(cache.len(), 3);
        assert!(cache.get("a").is_none()); // "a" was evicted
        assert!(cache.get("b").is_some());
        assert!(cache.get("c").is_some());
        assert!(cache.get("d").is_some());
    }

    #[test]
    fn test_semantic_cache_lru_access_updates_order() {
        let mut cache = SemanticCache::new(3);

        cache.insert("a".to_string(), make_result("a"));
        cache.insert("b".to_string(), make_result("b"));
        cache.insert("c".to_string(), make_result("c"));

        // Access "a" to make it most recently used
        let _ = cache.get("a");

        // Insert new item - should evict "b" (now oldest)
        cache.insert("d".to_string(), make_result("d"));

        assert!(cache.get("a").is_some()); // "a" was accessed, not evicted
        assert!(cache.get("b").is_none()); // "b" was evicted
        assert!(cache.get("c").is_some());
        assert!(cache.get("d").is_some());
    }

    #[test]
    fn test_semantic_cache_update_existing() {
        let mut cache = SemanticCache::new(100);

        let result1 = make_result("v1");
        cache.insert("key".to_string(), result1);

        let mut result2 = make_result("v2");
        result2.confidence = 0.9;
        cache.insert("key".to_string(), result2);

        // Should still have only 1 entry
        assert_eq!(cache.len(), 1);

        // Should have the updated value
        let retrieved = cache.get("key").unwrap();
        assert_eq!(retrieved.original_word, "v2");
        assert_eq!(retrieved.confidence, 0.9);
    }

    #[test]
    fn test_semantic_cache_stats() {
        let mut cache = SemanticCache::new(100);
        cache.insert("a".to_string(), make_result("a"));

        // Initial stats
        let (rate, hits, misses, evictions) = cache.stats();
        assert_eq!(rate, 0.0);
        assert_eq!(hits, 0);
        assert_eq!(misses, 0);
        assert_eq!(evictions, 0);

        // Hit
        let _ = cache.get("a");
        let (rate, hits, misses, _) = cache.stats();
        assert_eq!(hits, 1);
        assert_eq!(misses, 0);
        assert_eq!(rate, 1.0);

        // Miss
        let _ = cache.get("nonexistent");
        let (rate, hits, misses, _) = cache.stats();
        assert_eq!(hits, 1);
        assert_eq!(misses, 1);
        assert_eq!(rate, 0.5);
    }

    #[test]
    fn test_semantic_cache_stats_with_eviction() {
        let mut cache = SemanticCache::new(2);
        cache.insert("a".to_string(), make_result("a"));
        cache.insert("b".to_string(), make_result("b"));
        cache.insert("c".to_string(), make_result("c")); // Evicts "a"

        let (_, _, _, evictions) = cache.stats();
        assert_eq!(evictions, 1);
    }

    #[test]
    fn test_semantic_cache_insert_arc() {
        let mut cache = SemanticCache::new(100);
        let result = Arc::new(make_result("arc_insert"));
        cache.insert_arc("key".to_string(), Arc::clone(&result));

        assert_eq!(cache.len(), 1);
        let retrieved = cache.get_arc("key").unwrap();
        assert_eq!(retrieved.original_word, "arc_insert");
    }

    #[test]
    fn test_semantic_cache_debug() {
        let cache = SemanticCache::new(100);
        let debug = format!("{:?}", cache);
        assert!(debug.contains("SemanticCache"));
    }
}
