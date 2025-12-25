//! Simplified coordinator for coverage testing
//! This file is temporarily simplified to allow coverage testing of other modules

use crate::lemmatizer::{Lemmatizer, SimpleLemmatizer};
use canopy_core::UPos;
use canopy_engine::EngineResult;
use canopy_framenet::FrameNetEngine;
use canopy_verbnet::VerbNetEngine;
use canopy_wordnet::{PartOfSpeech, WordNetEngine};
#[cfg(feature = "parallel")]
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

/// Convert Universal Dependencies POS to WordNet PartOfSpeech
/// Returns None for function words that don't have WordNet entries
fn upos_to_wordnet_pos(upos: UPos) -> Option<PartOfSpeech> {
    match upos {
        UPos::Adj => Some(PartOfSpeech::Adjective),
        UPos::Adv => Some(PartOfSpeech::Adverb),
        UPos::Verb | UPos::Aux => Some(PartOfSpeech::Verb),
        UPos::Noun | UPos::Propn => Some(PartOfSpeech::Noun),
        // Function words and others don't have WordNet entries
        _ => None,
    }
}

/// Check if this POS should query VerbNet (verbs/auxiliaries only)
fn should_query_verbnet(upos: Option<UPos>) -> bool {
    matches!(upos, Some(UPos::Verb) | Some(UPos::Aux) | None)
}

/// Check if this POS should query FrameNet (content words)
fn should_query_framenet(upos: Option<UPos>) -> bool {
    matches!(
        upos,
        Some(UPos::Verb)
            | Some(UPos::Aux)
            | Some(UPos::Noun)
            | Some(UPos::Propn)
            | Some(UPos::Adj)
            | None
    )
}

/// Guess likely POS from word suffix (for WordNet optimization)
/// Returns the most likely POS based on common English morphological patterns
pub fn guess_pos_from_suffix(word: &str) -> Option<PartOfSpeech> {
    let w = word.to_lowercase();
    let len = w.len();

    if len < 3 {
        return None;
    }

    // Adverb suffixes (check first - most distinctive)
    if w.ends_with("ly") && len > 4 {
        return Some(PartOfSpeech::Adverb);
    }

    // Verb suffixes
    if w.ends_with("ing") && len > 5 {
        return Some(PartOfSpeech::Verb);
    }
    if w.ends_with("ed") && len > 4 && !w.ends_with("eed") {
        return Some(PartOfSpeech::Verb);
    }
    if w.ends_with("ize") || w.ends_with("ise") || w.ends_with("ate") {
        return Some(PartOfSpeech::Verb);
    }

    // Noun suffixes
    if w.ends_with("tion") || w.ends_with("sion") || w.ends_with("ment") {
        return Some(PartOfSpeech::Noun);
    }
    if w.ends_with("ness") || w.ends_with("ity") || w.ends_with("ance") || w.ends_with("ence") {
        return Some(PartOfSpeech::Noun);
    }
    if (w.ends_with("er") || w.ends_with("or")) && len > 4 {
        return Some(PartOfSpeech::Noun); // agent nouns
    }

    // Adjective suffixes
    if w.ends_with("ful") || w.ends_with("less") || w.ends_with("ous") || w.ends_with("ive") {
        return Some(PartOfSpeech::Adjective);
    }
    if w.ends_with("able") || w.ends_with("ible") || w.ends_with("ical") {
        return Some(PartOfSpeech::Adjective);
    }

    None
}

/// All WordNet POS types for parallel querying
const WORDNET_ALL_POS: [PartOfSpeech; 4] = [
    PartOfSpeech::Noun,
    PartOfSpeech::Verb,
    PartOfSpeech::Adjective,
    PartOfSpeech::Adverb,
];

/// Confidence threshold for early exit when suffix heuristics match
const WORDNET_EARLY_EXIT_CONFIDENCE: f32 = 0.7;

/// Lightweight treebank analysis result for Layer 1 integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreebankAnalysis {
    /// Original word analyzed
    pub word: String,
    /// Found dependency relation (if any)
    pub dependency_relation: Option<String>,
    /// Analysis confidence
    pub confidence: f32,
    /// Processing time in microseconds
    pub processing_time_us: u64,
    /// Whether result came from cache
    pub from_cache: bool,
    /// Voice features extracted (passive, active, etc.)
    pub voice_features: Vec<String>,
    /// Semantic role features (:agent, :pass, etc.)
    pub semantic_features: Vec<String>,
}

impl TreebankAnalysis {
    pub fn new(word: String, confidence: f32) -> Self {
        Self {
            word,
            dependency_relation: None,
            confidence,
            processing_time_us: 0,
            from_cache: false,
            voice_features: Vec::new(),
            semantic_features: Vec::new(),
        }
    }
}

/// Memory usage statistics
#[derive(Debug, Clone)]
pub struct MemoryUsage {
    pub estimated_usage_mb: f32,
    pub budget_mb: usize,
    pub utilization_percent: f32,
}

impl Default for MemoryUsage {
    fn default() -> Self {
        Self {
            estimated_usage_mb: 0.0,
            budget_mb: 100,
            utilization_percent: 0.0,
        }
    }
}

/// Memory pressure alert
#[derive(Debug, Clone)]
pub struct MemoryPressureAlert {
    pub message: String,
    pub severity: String,
    pub usage_mb: f32,
    pub budget_mb: usize,
    pub current_usage_mb: f32,
    pub current_utilization: f32,
    pub recommendation: String,
}

/// Statistics for semantic analysis
#[derive(Debug, Clone)]
pub struct CoordinatorStatistics {
    pub total_analyses: usize,
    pub cache_hits: usize,
    pub cache_misses: usize,
    pub successful_analyses: usize,
    pub failed_analyses: usize,
    pub average_confidence: f32,
    pub total_queries: usize,
    pub cache_hit_rate: f32,
    pub parallel_queries: usize,
    pub parallel_query_rate: f32,
    pub warmed_queries: usize,
    pub memory_usage: MemoryUsage,
    pub active_engines: Vec<String>,
}

impl Default for CoordinatorStatistics {
    fn default() -> Self {
        Self {
            total_analyses: 0,
            cache_hits: 0,
            cache_misses: 0,
            successful_analyses: 0,
            failed_analyses: 0,
            average_confidence: 0.0,
            total_queries: 0,
            cache_hit_rate: 0.0,
            parallel_queries: 0,
            parallel_query_rate: 0.0,
            warmed_queries: 0,
            memory_usage: MemoryUsage::default(),
            active_engines: Vec::new(),
        }
    }
}

/// Configuration for the semantic coordinator
#[derive(Debug, Clone)]
pub struct CoordinatorConfig {
    pub enable_verbnet: bool,
    pub enable_framenet: bool,
    pub enable_wordnet: bool,
    pub enable_lexicon: bool,
    pub enable_treebank: bool,
    pub enable_lemmatization: bool,
    pub use_advanced_lemmatization: bool,
    pub confidence_threshold: f32,
    pub l1_cache_memory_mb: usize,
    /// Use treebank gold-standard lemmas when available
    pub use_treebank_lemmas: bool,
    /// Minimum confidence threshold for lemma caching
    pub lemma_confidence_threshold: f32,
    /// Enable integration with shared lemma cache
    pub enable_shared_lemma_cache: bool,
    /// Cache configuration
    pub cache_capacity: usize,
    pub enable_cache_warmup: bool,
    pub cache_warmup_common_words: bool,
}

impl Default for CoordinatorConfig {
    fn default() -> Self {
        Self {
            enable_verbnet: true,
            enable_framenet: true,
            enable_wordnet: true,
            enable_lexicon: true,
            enable_treebank: true,
            enable_lemmatization: true,
            use_advanced_lemmatization: false,
            confidence_threshold: 0.1,
            l1_cache_memory_mb: 50,
            use_treebank_lemmas: true, // Prefer gold-standard lemmas
            lemma_confidence_threshold: 0.3,
            enable_shared_lemma_cache: true,
            cache_capacity: 10000,            // Much larger cache
            enable_cache_warmup: false,       // Disabled by default for faster startup
            cache_warmup_common_words: false, // Enable explicitly when needed
        }
    }
}

/// Layer 1 semantic analysis result
#[derive(Debug, Clone)]
pub struct Layer1SemanticResult {
    pub original_word: String,
    pub lemma: String,
    pub pos: Option<UPos>,
    pub lemmatization_confidence: Option<f32>,
    pub verbnet: Option<canopy_verbnet::VerbNetAnalysis>,
    pub framenet: Option<canopy_framenet::FrameNetAnalysis>,
    pub wordnet: Option<canopy_wordnet::WordNetAnalysis>,
    pub lexicon: Option<canopy_lexicon::LexiconAnalysis>,
    pub treebank: Option<TreebankAnalysis>,
    pub confidence: f32,
    pub sources: Vec<String>,
    pub errors: Vec<String>,
}

impl Layer1SemanticResult {
    pub fn new(original_word: String, lemma: String) -> Self {
        Self {
            original_word,
            lemma,
            pos: None,
            lemmatization_confidence: None,
            verbnet: None,
            framenet: None,
            wordnet: None,
            lexicon: None,
            treebank: None,
            confidence: 0.0,
            sources: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// Check if the result has any semantic analysis data
    pub fn has_results(&self) -> bool {
        self.verbnet.is_some()
            || self.framenet.is_some()
            || self.wordnet.is_some()
            || self.lexicon.is_some()
            || self.treebank.is_some()
            || !self.sources.is_empty()
    }

    /// Check if the result has coverage from multiple engines
    pub fn has_multi_engine_coverage(&self) -> bool {
        let engine_count = [
            self.verbnet.is_some(),
            self.framenet.is_some(),
            self.wordnet.is_some(),
            self.lexicon.is_some(),
            self.treebank.is_some(),
        ]
        .iter()
        .filter(|&&has| has)
        .count();

        engine_count >= 2
    }
}

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

        println!(
            "🔥 Warming cache with {} common words...",
            common_words.len()
        );

        // Use parallel batch analysis for faster warmup
        if let Ok(results) = coordinator.analyze_batch_parallel(&common_words) {
            for (word, result) in common_words.iter().zip(results) {
                let key = self.generate_key(word, true);
                if !self.cache.contains_key(&key) {
                    self.insert(key, result);
                }
            }
        }

        println!("✅ Cache warmed with {} entries", self.cache.len());
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

/// Trait for treebank analysis integration
pub trait TreebankProvider: Send + Sync {
    fn analyze_word(&self, word: &str) -> Result<TreebankAnalysis, canopy_engine::EngineError>;
}

/// Semantic coordinator for Layer 1 analysis
pub struct SemanticCoordinator {
    config: CoordinatorConfig,
    lemmatizer: Arc<dyn Lemmatizer>,
    treebank_provider: Option<Arc<dyn TreebankProvider>>,
    verbnet_engine: Option<VerbNetEngine>,
    framenet_engine: Option<FrameNetEngine>,
    wordnet_engine: Option<WordNetEngine>,
    stats: Arc<Mutex<CoordinatorStatistics>>,
    cache: Arc<Mutex<SemanticCache>>,
}

impl SemanticCoordinator {
    pub fn new(config: CoordinatorConfig) -> EngineResult<Self> {
        let lemmatizer: Arc<dyn Lemmatizer> = Arc::new(SimpleLemmatizer::new()?);

        // Create engines in parallel for faster initialization
        let enable_verbnet = config.enable_verbnet;
        let enable_framenet = config.enable_framenet;
        let enable_wordnet = config.enable_wordnet;

        // Launch parallel engine initialization
        let verbnet_handle = if enable_verbnet {
            Some(thread::spawn(
                move || -> Result<VerbNetEngine, canopy_engine::EngineError> {
                    VerbNetEngine::new()
                },
            ))
        } else {
            None
        };

        let framenet_handle = if enable_framenet {
            Some(thread::spawn(
                move || -> Result<FrameNetEngine, canopy_engine::EngineError> {
                    FrameNetEngine::new()
                },
            ))
        } else {
            None
        };

        let wordnet_handle = if enable_wordnet {
            Some(thread::spawn(
                move || -> Result<WordNetEngine, canopy_engine::EngineError> {
                    WordNetEngine::new()
                },
            ))
        } else {
            None
        };

        // Collect results from parallel initialization - fail fast if data unavailable
        let verbnet_engine = if let Some(handle) = verbnet_handle {
            match handle.join() {
                Ok(Ok(engine)) => {
                    println!("✅ VerbNet engine loaded with real data");
                    Some(engine)
                }
                Ok(Err(e)) => {
                    return Err(e);
                }
                Err(_) => {
                    return Err(canopy_engine::EngineError::data_load(
                        "VerbNet thread panicked",
                    ));
                }
            }
        } else {
            None
        };

        let framenet_engine = if let Some(handle) = framenet_handle {
            match handle.join() {
                Ok(Ok(engine)) => {
                    println!("✅ FrameNet engine loaded with real data");
                    Some(engine)
                }
                Ok(Err(e)) => {
                    return Err(e);
                }
                Err(_) => {
                    return Err(canopy_engine::EngineError::data_load(
                        "FrameNet thread panicked",
                    ));
                }
            }
        } else {
            None
        };

        let wordnet_engine = if let Some(handle) = wordnet_handle {
            match handle.join() {
                Ok(Ok(engine)) => {
                    println!("✅ WordNet engine loaded with real data");
                    Some(engine)
                }
                Ok(Err(e)) => {
                    return Err(e);
                }
                Err(_) => {
                    return Err(canopy_engine::EngineError::data_load(
                        "WordNet thread panicked",
                    ));
                }
            }
        } else {
            None
        };

        let cache = Arc::new(Mutex::new(SemanticCache::new(config.cache_capacity)));

        let coordinator = Self {
            config: config.clone(),
            lemmatizer,
            treebank_provider: None,
            verbnet_engine,
            framenet_engine,
            wordnet_engine,
            stats: Arc::new(Mutex::new(CoordinatorStatistics::default())),
            cache: cache.clone(),
        };

        // Warmup cache if enabled
        // NOTE: Don't hold cache lock while warming up - warmup calls analyze() which needs the lock
        if config.enable_cache_warmup && config.cache_warmup_common_words {
            coordinator.warmup_cache();
        }

        Ok(coordinator)
    }

    /// Set a treebank provider for dependency analysis
    pub fn set_treebank_provider(&mut self, provider: Arc<dyn TreebankProvider>) {
        self.treebank_provider = Some(provider);
    }

    pub fn analyze(&self, word: &str) -> EngineResult<Layer1SemanticResult> {
        // Lemmatize FIRST so cache key is based on lemma (not surface form)
        // This ensures "running", "runs", "ran" all share the same cache entry
        let (lemma, confidence) = if self.config.enable_lemmatization {
            let (lemma, conf) = self.lemmatizer.lemmatize_with_confidence(word);
            (lemma, Some(conf))
        } else {
            (word.to_string(), None)
        };

        // Generate cache key from LEMMA (not original word)
        let cache_key = lemma.to_lowercase();

        // Check cache
        if let Ok(mut cache) = self.cache.lock() {
            if let Some(cached_result) = cache.get(&cache_key) {
                // Cache hit - update stats and return result with original word
                if let Ok(mut stats) = self.stats.lock() {
                    stats.total_queries += 1;
                    stats.cache_hits += 1;
                    stats.cache_hit_rate = stats.cache_hits as f32 / stats.total_queries as f32;
                }
                let mut result = cached_result.clone();
                result.original_word = word.to_string(); // Update to current word
                result.lemmatization_confidence = confidence; // Update confidence for this word
                return Ok(result);
            }
        }

        let mut result = Layer1SemanticResult::new(word.to_string(), lemma.clone());
        result.lemmatization_confidence = confidence;

        // Perform parallel engine queries for VerbNet, FrameNet, and WordNet
        let verbnet_engine = self.verbnet_engine.as_ref();
        let framenet_engine = self.framenet_engine.as_ref();
        let wordnet_engine = self.wordnet_engine.as_ref();

        // Use thread::scope to parallelize engine queries (~3x speedup for cache misses)
        let (verbnet_result, framenet_result, wordnet_result) = thread::scope(|s| {
            // VerbNet analysis
            let verbnet_handle =
                s.spawn(|| verbnet_engine.and_then(|engine| engine.analyze_verb(&lemma).ok()));

            // FrameNet analysis
            let framenet_handle =
                s.spawn(|| framenet_engine.and_then(|engine| engine.analyze_text(&lemma).ok()));

            // WordNet analysis - optimized with suffix heuristics + early exit
            let wordnet_handle = s.spawn(|| {
                wordnet_engine.and_then(|engine| {
                    // Strategy 1: Try suffix-based POS guess first (can skip all 4 queries)
                    if let Some(guessed_pos) = guess_pos_from_suffix(&lemma) {
                        if let Ok(result) = engine.analyze_word(&lemma, guessed_pos) {
                            if result.confidence >= WORDNET_EARLY_EXIT_CONFIDENCE {
                                return Some(result);
                            }
                        }
                    }

                    // Strategy 2: Sequential queries with early exit on high confidence
                    // Already parallel to VerbNet/FrameNet, so no nested threading needed
                    let mut best_result: Option<
                        canopy_engine::SemanticResult<canopy_wordnet::WordNetAnalysis>,
                    > = None;

                    for pos in WORDNET_ALL_POS {
                        if let Ok(result) = engine.analyze_word(&lemma, pos) {
                            // Early exit on high confidence
                            if result.confidence >= WORDNET_EARLY_EXIT_CONFIDENCE {
                                return Some(result);
                            }
                            // Track best result so far
                            if best_result
                                .as_ref()
                                .is_none_or(|b| result.confidence > b.confidence)
                            {
                                best_result = Some(result);
                            }
                        }
                    }

                    best_result
                })
            });

            (
                verbnet_handle.join().unwrap_or(None),
                framenet_handle.join().unwrap_or(None),
                wordnet_handle.join().unwrap_or(None),
            )
        });

        // Process VerbNet result
        if let Some(verbnet_res) = verbnet_result {
            result.verbnet = Some(verbnet_res.data);
            result.sources.push("VerbNet".to_string());
            if verbnet_res.confidence > result.confidence {
                result.confidence = verbnet_res.confidence;
            }
        }

        // Process FrameNet result
        if let Some(framenet_res) = framenet_result {
            result.framenet = Some(framenet_res.data);
            result.sources.push("FrameNet".to_string());
            if framenet_res.confidence > result.confidence {
                result.confidence = framenet_res.confidence;
            }
        }

        // Process WordNet result
        if let Some(wordnet_res) = wordnet_result {
            result.wordnet = Some(wordnet_res.data);
            result.sources.push("WordNet".to_string());
            if wordnet_res.confidence > result.confidence {
                result.confidence = wordnet_res.confidence;
            }
        }

        // Perform treebank analysis if enabled and provider is available
        if self.config.enable_treebank {
            if let Some(ref provider) = self.treebank_provider {
                if let Ok(treebank_analysis) = provider.analyze_word(&lemma) {
                    result.treebank = Some(treebank_analysis);
                    result.sources.push("Treebank".to_string());

                    // Update confidence based on treebank analysis
                    if let Some(ref tb) = result.treebank {
                        if tb.confidence > result.confidence {
                            result.confidence = tb.confidence;
                        }
                    }
                }
            }
        }

        // Store in cache using lemmatized form as key
        if let Ok(mut cache) = self.cache.lock() {
            cache.insert(cache_key, result.clone());
        }

        // Update statistics
        if let Ok(mut stats) = self.stats.lock() {
            stats.total_queries += 1;
            stats.total_analyses += 1;
            stats.successful_analyses += 1;
            stats.cache_misses += 1;

            // Update cache hit rate
            if stats.total_queries > 0 {
                stats.cache_hit_rate = stats.cache_hits as f32 / stats.total_queries as f32;
            }
        }

        Ok(result)
    }

    /// Analyze a word with known POS for better cache hits and accurate sense selection
    ///
    /// When POS is provided:
    /// - Cache key includes POS for differentiation (e.g., "bank:Noun" vs "bank:Verb")
    /// - VerbNet only queried for Verb/Aux
    /// - WordNet uses correct POS directly instead of trying all
    /// - FrameNet queries filtered by content word POS
    pub fn analyze_with_pos(
        &self,
        word: &str,
        pos: Option<UPos>,
    ) -> EngineResult<Layer1SemanticResult> {
        // Lemmatize FIRST so cache key is based on lemma (not surface form)
        // This ensures "running", "runs", "ran" all share the same cache entry
        let (lemma, confidence) = if self.config.enable_lemmatization {
            let (lemma, conf) = self.lemmatizer.lemmatize_with_confidence(word);
            (lemma, Some(conf))
        } else {
            (word.to_string(), None)
        };

        // Generate POS-aware cache key from LEMMA (not original word)
        let cache_key = if let Ok(cache) = self.cache.lock() {
            cache.generate_key_with_pos(&lemma, pos)
        } else {
            lemma.to_lowercase()
        };

        // Check cache
        if let Ok(mut cache) = self.cache.lock() {
            if let Some(cached_result) = cache.get(&cache_key) {
                if let Ok(mut stats) = self.stats.lock() {
                    stats.total_queries += 1;
                    stats.cache_hits += 1;
                    stats.cache_hit_rate = stats.cache_hits as f32 / stats.total_queries as f32;
                }
                let mut result = cached_result.clone();
                result.original_word = word.to_string();
                result.lemmatization_confidence = confidence; // Update for this word
                return Ok(result);
            }
        }

        let mut result = Layer1SemanticResult::new(word.to_string(), lemma.clone());
        result.pos = pos;
        result.lemmatization_confidence = confidence;

        // Get engine references
        let verbnet_engine = self.verbnet_engine.as_ref();
        let framenet_engine = self.framenet_engine.as_ref();
        let wordnet_engine = self.wordnet_engine.as_ref();

        // Parallel engine queries with POS filtering
        let (verbnet_result, framenet_result, wordnet_result) = thread::scope(|s| {
            // VerbNet: only for verbs/aux (skip for nouns, adjectives, etc.)
            let verbnet_handle = s.spawn(|| {
                if should_query_verbnet(pos) {
                    verbnet_engine.and_then(|engine| engine.analyze_verb(&lemma).ok())
                } else {
                    None
                }
            });

            // FrameNet: for content words only
            let framenet_handle = s.spawn(|| {
                if should_query_framenet(pos) {
                    framenet_engine.and_then(|engine| engine.analyze_text(&lemma).ok())
                } else {
                    None
                }
            });

            // WordNet: use specific POS if known, otherwise try all
            let wordnet_handle = s.spawn(|| {
                wordnet_engine.and_then(|engine| {
                    if let Some(upos) = pos {
                        // Known POS - query directly (1 query instead of 4)
                        if let Some(wordnet_pos) = upos_to_wordnet_pos(upos) {
                            engine.analyze_word(&lemma, wordnet_pos).ok()
                        } else {
                            None // Function words don't have WordNet entries
                        }
                    } else {
                        // Unknown POS - use suffix heuristics + early exit
                        if let Some(guessed_pos) = guess_pos_from_suffix(&lemma) {
                            if let Ok(result) = engine.analyze_word(&lemma, guessed_pos) {
                                if result.confidence >= WORDNET_EARLY_EXIT_CONFIDENCE {
                                    return Some(result);
                                }
                            }
                        }

                        // Sequential queries with early exit
                        let mut best_result: Option<
                            canopy_engine::SemanticResult<canopy_wordnet::WordNetAnalysis>,
                        > = None;
                        for pos in WORDNET_ALL_POS {
                            if let Ok(result) = engine.analyze_word(&lemma, pos) {
                                if result.confidence >= WORDNET_EARLY_EXIT_CONFIDENCE {
                                    return Some(result);
                                }
                                if best_result
                                    .as_ref()
                                    .is_none_or(|b| result.confidence > b.confidence)
                                {
                                    best_result = Some(result);
                                }
                            }
                        }
                        best_result
                    }
                })
            });

            (
                verbnet_handle.join().unwrap_or(None),
                framenet_handle.join().unwrap_or(None),
                wordnet_handle.join().unwrap_or(None),
            )
        });

        // Process VerbNet result
        if let Some(verbnet_res) = verbnet_result {
            result.verbnet = Some(verbnet_res.data);
            result.sources.push("VerbNet".to_string());
            if verbnet_res.confidence > result.confidence {
                result.confidence = verbnet_res.confidence;
            }
        }

        // Process FrameNet result
        if let Some(framenet_res) = framenet_result {
            result.framenet = Some(framenet_res.data);
            result.sources.push("FrameNet".to_string());
            if framenet_res.confidence > result.confidence {
                result.confidence = framenet_res.confidence;
            }
        }

        // Process WordNet result
        if let Some(wordnet_res) = wordnet_result {
            result.wordnet = Some(wordnet_res.data);
            result.sources.push("WordNet".to_string());
            if wordnet_res.confidence > result.confidence {
                result.confidence = wordnet_res.confidence;
            }
        }

        // Treebank analysis (if enabled)
        if self.config.enable_treebank {
            if let Some(ref provider) = self.treebank_provider {
                if let Ok(treebank_analysis) = provider.analyze_word(&lemma) {
                    result.treebank = Some(treebank_analysis);
                    result.sources.push("Treebank".to_string());
                    if let Some(ref tb) = result.treebank {
                        if tb.confidence > result.confidence {
                            result.confidence = tb.confidence;
                        }
                    }
                }
            }
        }

        // Cache result
        if let Ok(mut cache) = self.cache.lock() {
            cache.insert(cache_key, result.clone());
        }

        // Update statistics
        if let Ok(mut stats) = self.stats.lock() {
            stats.total_queries += 1;
            stats.total_analyses += 1;
            stats.successful_analyses += 1;
            stats.cache_misses += 1;
            if stats.total_queries > 0 {
                stats.cache_hit_rate = stats.cache_hits as f32 / stats.total_queries as f32;
            }
        }

        Ok(result)
    }

    /// Internal method for cache warmup (bypasses some optimizations)
    pub fn analyze_word_internal(&self, word: &str) -> EngineResult<Layer1SemanticResult> {
        self.analyze(word)
    }

    pub fn analyze_batch(&self, words: &[String]) -> EngineResult<Vec<Layer1SemanticResult>> {
        words.iter().map(|word| self.analyze(word)).collect()
    }

    /// Get current statistics
    pub fn get_statistics(&self) -> CoordinatorStatistics {
        let mut stats = self.stats.lock().unwrap().clone();

        // Update active engines list
        stats.active_engines.clear();
        if self.verbnet_engine.is_some() {
            stats.active_engines.push("VerbNet".to_string());
        }
        if self.framenet_engine.is_some() {
            stats.active_engines.push("FrameNet".to_string());
        }
        if self.wordnet_engine.is_some() {
            stats.active_engines.push("WordNet".to_string());
        }
        if self.treebank_provider.is_some() {
            stats.active_engines.push("Treebank".to_string());
        }

        stats
    }

    /// Warm up cache with common words
    pub fn warm_cache(&self, words: &[String]) -> EngineResult<Vec<Layer1SemanticResult>> {
        // For now, just analyze the words without special cache warming
        self.analyze_batch(words)
    }

    /// Warm up cache with built-in common words
    pub fn warmup_cache(&self) {
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

        println!(
            "🔥 Warming cache with {} common words...",
            common_words.len()
        );

        // Analyze all common words - this populates the cache via normal analyze() calls
        let _ = self.analyze_batch(&common_words);

        if let Ok(cache) = self.cache.lock() {
            println!("✅ Cache warmed with {} entries", cache.len());
        }
    }

    /// Analyze words with parallel execution (uses rayon when parallel feature enabled)
    #[cfg(feature = "parallel")]
    pub fn analyze_batch_parallel(
        &self,
        words: &[String],
    ) -> EngineResult<Vec<Layer1SemanticResult>> {
        // Use rayon for parallel iteration
        words.par_iter().map(|word| self.analyze(word)).collect()
    }

    /// Analyze words with parallel execution (sequential fallback when parallel disabled)
    #[cfg(not(feature = "parallel"))]
    pub fn analyze_batch_parallel(
        &self,
        words: &[String],
    ) -> EngineResult<Vec<Layer1SemanticResult>> {
        self.analyze_batch(words)
    }

    /// Analyze batch of words with POS, deduplicating by (lemma, pos) key
    ///
    /// Deduplicates to avoid redundant analysis within a single batch.
    /// Returns results in same order as input.
    pub fn analyze_batch_deduped(
        &self,
        words: &[(String, Option<UPos>)],
    ) -> EngineResult<Vec<Layer1SemanticResult>> {
        if words.is_empty() {
            return Ok(Vec::new());
        }

        // Build deduplication map: key -> first occurrence index
        let mut seen: HashMap<String, usize> = HashMap::new();
        let mut key_order: Vec<String> = Vec::with_capacity(words.len());

        for (idx, (word, pos)) in words.iter().enumerate() {
            let key = if let Ok(cache) = self.cache.lock() {
                cache.generate_key_with_pos(word, *pos)
            } else {
                format!("{}:{:?}", word.to_lowercase(), pos)
            };
            key_order.push(key.clone());
            seen.entry(key).or_insert(idx);
        }

        // Analyze unique keys only, storing results
        let mut results_map: HashMap<String, Layer1SemanticResult> = HashMap::new();
        for (key, &idx) in &seen {
            let (word, pos) = &words[idx];
            let result = self.analyze_with_pos(word, *pos)?;
            results_map.insert(key.clone(), result);
        }

        // Map results back to original order
        let results: Vec<Layer1SemanticResult> = key_order
            .iter()
            .enumerate()
            .map(|(idx, key)| {
                let mut result = results_map.get(key).cloned().unwrap_or_else(|| {
                    Layer1SemanticResult::new(words[idx].0.clone(), words[idx].0.clone())
                });
                result.original_word = words[idx].0.clone();
                result
            })
            .collect();

        Ok(results)
    }

    /// Parallel version of analyze_batch_deduped
    #[cfg(feature = "parallel")]
    pub fn analyze_batch_deduped_parallel(
        &self,
        words: &[(String, Option<UPos>)],
    ) -> EngineResult<Vec<Layer1SemanticResult>> {
        if words.is_empty() {
            return Ok(Vec::new());
        }

        // Build deduplication map
        let mut seen: HashMap<String, usize> = HashMap::new();
        let mut key_order: Vec<String> = Vec::with_capacity(words.len());

        for (idx, (word, pos)) in words.iter().enumerate() {
            let key = if let Ok(cache) = self.cache.lock() {
                cache.generate_key_with_pos(word, *pos)
            } else {
                format!("{}:{:?}", word.to_lowercase(), pos)
            };
            key_order.push(key.clone());
            seen.entry(key).or_insert(idx);
        }

        // Collect unique entries for parallel processing
        let unique_entries: Vec<(String, String, Option<UPos>)> = seen
            .iter()
            .map(|(key, &idx)| (key.clone(), words[idx].0.clone(), words[idx].1))
            .collect();

        // Analyze unique keys in parallel
        let unique_results: Vec<(String, Layer1SemanticResult)> = unique_entries
            .par_iter()
            .filter_map(|(key, word, pos)| {
                self.analyze_with_pos(word, *pos)
                    .ok()
                    .map(|r| (key.clone(), r))
            })
            .collect();

        let results_map: HashMap<String, Layer1SemanticResult> =
            unique_results.into_iter().collect();

        // Map results back to original order
        let results: Vec<Layer1SemanticResult> = key_order
            .iter()
            .enumerate()
            .map(|(idx, key)| {
                let mut result = results_map.get(key).cloned().unwrap_or_else(|| {
                    Layer1SemanticResult::new(words[idx].0.clone(), words[idx].0.clone())
                });
                result.original_word = words[idx].0.clone();
                result
            })
            .collect();

        Ok(results)
    }

    /// Parallel version of analyze_batch_deduped (sequential fallback)
    #[cfg(not(feature = "parallel"))]
    pub fn analyze_batch_deduped_parallel(
        &self,
        words: &[(String, Option<UPos>)],
    ) -> EngineResult<Vec<Layer1SemanticResult>> {
        self.analyze_batch_deduped(words)
    }

    /// Check for memory pressure
    pub fn check_memory_pressure(&self) -> Option<MemoryPressureAlert> {
        let stats = self.stats.lock().unwrap();
        let usage = &stats.memory_usage;
        if usage.utilization_percent > 90.0 {
            Some(MemoryPressureAlert {
                message: "High memory usage detected".to_string(),
                severity: "high".to_string(),
                usage_mb: usage.estimated_usage_mb,
                budget_mb: usage.budget_mb,
                current_usage_mb: usage.estimated_usage_mb,
                current_utilization: usage.utilization_percent,
                recommendation: "Consider clearing cache or reducing batch sizes".to_string(),
            })
        } else {
            None
        }
    }

    /// Force cleanup of resources
    pub fn force_cleanup(&self) -> EngineResult<()> {
        // Placeholder for cleanup logic
        Ok(())
    }

    /// Get cache analytics
    pub fn get_cache_analytics(&self) -> CoordinatorStatistics {
        self.stats.lock().unwrap().clone()
    }
}

/// Create a Layer 1 analyzer with default configuration
pub fn create_l1_analyzer() -> EngineResult<SemanticCoordinator> {
    SemanticCoordinator::new(CoordinatorConfig::default())
}
