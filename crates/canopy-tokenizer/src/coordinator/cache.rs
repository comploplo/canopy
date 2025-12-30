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
