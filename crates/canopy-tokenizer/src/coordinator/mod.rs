//! Semantic coordinator for Layer 1 analysis
//!
//! Orchestrates all semantic engines (VerbNet, FrameNet, WordNet, PropBank, Lexicon)
//! for unified lexical semantic analysis.

mod cache;
mod pos_utils;
mod types;

// Re-export all public types
pub use cache::SemanticCache;
pub use pos_utils::{
    guess_pos_from_suffix, should_query_framenet, should_query_verbnet, upos_to_wordnet_pos,
    wordnet_pos_to_upos, WORDNET_ALL_POS, WORDNET_EARLY_EXIT_CONFIDENCE,
};
pub use types::{
    CoordinatorConfig, CoordinatorStatistics, DependencyArc, DependencyRelation,
    Layer1SemanticResult, MemoryPressureAlert, MemoryUsage, SentenceAnalysisResult,
    SentenceMetadata, TextAnalysisResult, TextAnalysisStats, TreebankAnalysis,
};

use crate::lemmatizer::{Lemmatizer, SimpleLemmatizer};
use canopy_core::UPos;
use canopy_engine::EngineResult;
use canopy_semantic_engines::framenet::FrameNetEngine;
use canopy_semantic_engines::lexicon::LexiconEngine;
use canopy_semantic_engines::propbank::PropBankEngine;
use canopy_semantic_engines::verbnet::VerbNetEngine;
use canopy_semantic_engines::wordnet::WordNetEngine;
#[cfg(feature = "parallel")]
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

/// Trait for treebank analysis integration
pub trait TreebankProvider: Send + Sync {
    fn analyze_word(&self, word: &str) -> Result<TreebankAnalysis, canopy_engine::EngineError>;

    /// Get POS tag for a word from treebank statistics
    ///
    /// Returns the most frequent POS tag observed for this word form
    /// in the UD English-EWT treebank. Useful for closed-class words
    /// (pronouns, determiners) not covered by VerbNet/WordNet.
    fn get_pos_for_word(&self, word: &str) -> Option<UPos>;
}

/// Helper to spawn an engine loader in a new thread if enabled.
fn spawn_engine<T, F>(enabled: bool, loader: F) -> Option<thread::JoinHandle<T>>
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    if enabled {
        Some(thread::spawn(loader))
    } else {
        None
    }
}

/// Helper to collect a required engine result from a thread handle.
/// Returns an error if the engine fails to load or the thread panics.
fn collect_required_engine<T: Send + 'static>(
    handle: Option<thread::JoinHandle<Result<T, canopy_engine::EngineError>>>,
    name: &str,
) -> EngineResult<Option<T>> {
    match handle {
        Some(h) => match h.join() {
            Ok(Ok(engine)) => {
                println!("  {} engine loaded", name);
                Ok(Some(engine))
            }
            Ok(Err(e)) => Err(e),
            Err(_) => Err(canopy_engine::EngineError::data_load(format!(
                "{} thread panicked",
                name
            ))),
        },
        None => Ok(None),
    }
}

/// Helper to collect an optional engine result from a thread handle.
/// Logs a warning if the engine fails but doesn't return an error.
fn collect_optional_engine<T: Send + 'static>(
    handle: Option<thread::JoinHandle<Result<T, canopy_engine::EngineError>>>,
    name: &str,
) -> Option<T> {
    match handle {
        Some(h) => match h.join() {
            Ok(Ok(engine)) => {
                println!("  {} engine loaded", name);
                Some(engine)
            }
            Ok(Err(e)) => {
                eprintln!("  {} initialization failed (optional): {}", name, e);
                None
            }
            Err(_) => {
                eprintln!("  {} thread panicked (optional)", name);
                None
            }
        },
        None => None,
    }
}

/// Semantic coordinator for Layer 1 analysis
pub struct SemanticCoordinator {
    config: CoordinatorConfig,
    lemmatizer: Arc<dyn Lemmatizer>,
    treebank_provider: Option<Arc<dyn TreebankProvider>>,
    verbnet_engine: Option<VerbNetEngine>,
    framenet_engine: Option<FrameNetEngine>,
    wordnet_engine: Option<WordNetEngine>,
    propbank_engine: Option<PropBankEngine>,
    lexicon_engine: Option<LexiconEngine>,
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
        let enable_propbank = config.enable_propbank;
        let enable_lexicon = config.enable_lexicon;

        // Launch parallel engine initialization
        let verbnet_handle = spawn_engine(enable_verbnet, VerbNetEngine::new);
        let framenet_handle = spawn_engine(enable_framenet, FrameNetEngine::new);
        let wordnet_handle = spawn_engine(enable_wordnet, WordNetEngine::new);
        let propbank_handle = spawn_engine(enable_propbank, PropBankEngine::new);
        let lexicon_handle = spawn_engine(enable_lexicon, LexiconEngine::new);

        // Collect results from parallel initialization
        // Required engines fail fast, optional engines log warnings
        let verbnet_engine = collect_required_engine(verbnet_handle, "VerbNet")?;
        let framenet_engine = collect_required_engine(framenet_handle, "FrameNet")?;
        let wordnet_engine = collect_required_engine(wordnet_handle, "WordNet")?;
        let propbank_engine = collect_optional_engine(propbank_handle, "PropBank");

        // Lexicon returns Self directly (not Result), handle separately
        let lexicon_engine = match lexicon_handle {
            Some(h) => match h.join() {
                Ok(engine) => {
                    println!("  Lexicon engine loaded");
                    Some(engine)
                }
                Err(_) => {
                    eprintln!("  Lexicon thread panicked (optional)");
                    None
                }
            },
            None => None,
        };

        let cache = Arc::new(Mutex::new(SemanticCache::new(config.cache_capacity)));

        let coordinator = Self {
            config: config.clone(),
            lemmatizer,
            treebank_provider: None,
            verbnet_engine,
            framenet_engine,
            wordnet_engine,
            propbank_engine,
            lexicon_engine,
            stats: Arc::new(Mutex::new(CoordinatorStatistics::default())),
            cache: cache.clone(),
        };

        // Warmup cache if enabled
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
                result.original_word = word.to_string();
                result.lemmatization_confidence = confidence;
                return Ok(result);
            }
        }

        let mut result = Layer1SemanticResult::new(word.to_string(), lemma.clone());
        result.lemmatization_confidence = confidence;

        // Perform parallel engine queries
        let verbnet_engine = self.verbnet_engine.as_ref();
        let framenet_engine = self.framenet_engine.as_ref();
        let wordnet_engine = self.wordnet_engine.as_ref();
        let propbank_engine = self.propbank_engine.as_ref();
        let lexicon_engine = self.lexicon_engine.as_ref();

        // Use thread::scope to parallelize engine queries
        let (verbnet_result, framenet_result, wordnet_result, propbank_result, lexicon_result) =
            thread::scope(|s| {
                let verbnet_handle =
                    s.spawn(|| verbnet_engine.and_then(|engine| engine.analyze_verb(&lemma).ok()));

                let framenet_handle =
                    s.spawn(|| framenet_engine.and_then(|engine| engine.analyze_text(&lemma).ok()));

                // WordNet analysis - optimized with suffix heuristics + early exit
                let wordnet_handle = s.spawn(|| {
                    wordnet_engine.and_then(|engine| {
                        // Strategy 1: Try suffix-based POS guess first
                        if let Some(guessed_pos) = guess_pos_from_suffix(&lemma) {
                            if let Ok(result) = engine.analyze_word(&lemma, guessed_pos) {
                                if result.confidence >= WORDNET_EARLY_EXIT_CONFIDENCE {
                                    return Some(result);
                                }
                            }
                        }

                        // Strategy 2: Sequential queries with early exit on high confidence
                        let mut best_result: Option<
                            canopy_engine::SemanticResult<
                                canopy_semantic_engines::wordnet::WordNetAnalysis,
                            >,
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
                    })
                });

                let propbank_handle =
                    s.spawn(|| propbank_engine.and_then(|engine| engine.analyze_word(&lemma).ok()));

                let lexicon_handle =
                    s.spawn(|| lexicon_engine.and_then(|engine| engine.analyze_word(&lemma).ok()));

                (
                    verbnet_handle.join().unwrap_or(None),
                    framenet_handle.join().unwrap_or(None),
                    wordnet_handle.join().unwrap_or(None),
                    propbank_handle.join().unwrap_or(None),
                    lexicon_handle.join().unwrap_or(None),
                )
            });

        // Process results
        if let Some(verbnet_res) = verbnet_result {
            result.verbnet = Some(verbnet_res.data);
            result.sources.push("VerbNet".to_string());
            if verbnet_res.confidence > result.confidence {
                result.confidence = verbnet_res.confidence;
            }
        }

        if let Some(framenet_res) = framenet_result {
            result.framenet = Some(framenet_res.data);
            result.sources.push("FrameNet".to_string());
            if framenet_res.confidence > result.confidence {
                result.confidence = framenet_res.confidence;
            }
        }

        if let Some(wordnet_res) = wordnet_result {
            result.wordnet = Some(wordnet_res.data);
            result.sources.push("WordNet".to_string());
            if wordnet_res.confidence > result.confidence {
                result.confidence = wordnet_res.confidence;
            }
        }

        if let Some(propbank_res) = propbank_result {
            result.propbank = Some(propbank_res.data);
            result.sources.push("PropBank".to_string());
            if propbank_res.confidence > result.confidence {
                result.confidence = propbank_res.confidence;
            }
        }

        if let Some(lexicon_res) = lexicon_result {
            result.lexicon = Some(lexicon_res.data);
            result.sources.push("Lexicon".to_string());
            if lexicon_res.confidence > result.confidence {
                result.confidence = lexicon_res.confidence;
            }
        }

        // Perform treebank analysis if enabled
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

        // Store in cache
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

    /// Analyze a word with known POS for better cache hits and accurate sense selection
    pub fn analyze_with_pos(
        &self,
        word: &str,
        pos: Option<UPos>,
    ) -> EngineResult<Layer1SemanticResult> {
        let (lemma, confidence) = if self.config.enable_lemmatization {
            let (lemma, conf) = self.lemmatizer.lemmatize_with_confidence(word);
            (lemma, Some(conf))
        } else {
            (word.to_string(), None)
        };

        // Generate POS-aware cache key
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
                result.lemmatization_confidence = confidence;
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
        let propbank_engine = self.propbank_engine.as_ref();
        let lexicon_engine = self.lexicon_engine.as_ref();

        // Parallel engine queries with POS filtering
        let (verbnet_result, framenet_result, wordnet_result, propbank_result, lexicon_result) =
            thread::scope(|s| {
                // VerbNet: only for verbs/aux
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

                // WordNet: use specific POS if known
                let wordnet_handle = s.spawn(|| {
                    wordnet_engine.and_then(|engine| {
                        if let Some(upos) = pos {
                            if let Some(wordnet_pos) = upos_to_wordnet_pos(upos) {
                                engine.analyze_word(&lemma, wordnet_pos).ok()
                            } else {
                                None
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

                            let mut best_result: Option<
                                canopy_engine::SemanticResult<
                                    canopy_semantic_engines::wordnet::WordNetAnalysis,
                                >,
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

                // PropBank analysis (for verbs)
                let propbank_handle = s.spawn(|| {
                    if should_query_verbnet(pos) {
                        propbank_engine.and_then(|engine| engine.analyze_word(&lemma).ok())
                    } else {
                        None
                    }
                });

                // Lexicon analysis
                let lexicon_handle =
                    s.spawn(|| lexicon_engine.and_then(|engine| engine.analyze_word(&lemma).ok()));

                (
                    verbnet_handle.join().unwrap_or(None),
                    framenet_handle.join().unwrap_or(None),
                    wordnet_handle.join().unwrap_or(None),
                    propbank_handle.join().unwrap_or(None),
                    lexicon_handle.join().unwrap_or(None),
                )
            });

        // Process results
        if let Some(verbnet_res) = verbnet_result {
            result.verbnet = Some(verbnet_res.data);
            result.sources.push("VerbNet".to_string());
            if verbnet_res.confidence > result.confidence {
                result.confidence = verbnet_res.confidence;
            }
        }

        if let Some(framenet_res) = framenet_result {
            result.framenet = Some(framenet_res.data);
            result.sources.push("FrameNet".to_string());
            if framenet_res.confidence > result.confidence {
                result.confidence = framenet_res.confidence;
            }
        }

        if let Some(wordnet_res) = wordnet_result {
            result.wordnet = Some(wordnet_res.data);
            result.sources.push("WordNet".to_string());
            if wordnet_res.confidence > result.confidence {
                result.confidence = wordnet_res.confidence;
            }
        }

        if let Some(propbank_res) = propbank_result {
            result.propbank = Some(propbank_res.data);
            result.sources.push("PropBank".to_string());
            if propbank_res.confidence > result.confidence {
                result.confidence = propbank_res.confidence;
            }
        }

        if let Some(lexicon_res) = lexicon_result {
            result.lexicon = Some(lexicon_res.data);
            result.sources.push("Lexicon".to_string());
            if lexicon_res.confidence > result.confidence {
                result.confidence = lexicon_res.confidence;
            }
        }

        // Treebank analysis
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

    /// Internal method for cache warmup
    pub fn analyze_word_internal(&self, word: &str) -> EngineResult<Layer1SemanticResult> {
        self.analyze(word)
    }

    pub fn analyze_batch(&self, words: &[String]) -> EngineResult<Vec<Layer1SemanticResult>> {
        words.iter().map(|word| self.analyze(word)).collect()
    }

    /// Get current statistics
    pub fn get_statistics(&self) -> CoordinatorStatistics {
        let mut stats = self.stats.lock().unwrap().clone();

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
        if self.propbank_engine.is_some() {
            stats.active_engines.push("PropBank".to_string());
        }
        if self.lexicon_engine.is_some() {
            stats.active_engines.push("Lexicon".to_string());
        }
        if self.treebank_provider.is_some() {
            stats.active_engines.push("Treebank".to_string());
        }

        stats
    }

    /// Warm up cache with common words
    pub fn warm_cache(&self, words: &[String]) -> EngineResult<Vec<Layer1SemanticResult>> {
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
            "run", "walk", "give", "take", "make", "see", "come", "go", "think", "say", "get",
            "want", "use", "find", "work", "call", "try", "ask", "turn", "move",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();

        println!("Warming cache with {} common words...", common_words.len());

        let _ = self.analyze_batch(&common_words);

        if let Ok(cache) = self.cache.lock() {
            println!("Cache warmed with {} entries", cache.len());
        }
    }

    /// Analyze words with parallel execution
    #[cfg(feature = "parallel")]
    pub fn analyze_batch_parallel(
        &self,
        words: &[String],
    ) -> EngineResult<Vec<Layer1SemanticResult>> {
        words.par_iter().map(|word| self.analyze(word)).collect()
    }

    #[cfg(not(feature = "parallel"))]
    pub fn analyze_batch_parallel(
        &self,
        words: &[String],
    ) -> EngineResult<Vec<Layer1SemanticResult>> {
        self.analyze_batch(words)
    }

    /// Analyze batch of words with POS, deduplicating by (lemma, pos) key
    pub fn analyze_batch_deduped(
        &self,
        words: &[(String, Option<UPos>)],
    ) -> EngineResult<Vec<Layer1SemanticResult>> {
        if words.is_empty() {
            return Ok(Vec::new());
        }

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

        let mut results_map: HashMap<String, Layer1SemanticResult> = HashMap::new();
        for (key, &idx) in &seen {
            let (word, pos) = &words[idx];
            let result = self.analyze_with_pos(word, *pos)?;
            results_map.insert(key.clone(), result);
        }

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

        let unique_entries: Vec<(String, String, Option<UPos>)> = seen
            .iter()
            .map(|(key, &idx)| (key.clone(), words[idx].0.clone(), words[idx].1))
            .collect();

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
        Ok(())
    }

    /// Get cache analytics
    pub fn get_cache_analytics(&self) -> CoordinatorStatistics {
        self.stats.lock().unwrap().clone()
    }

    // ========================================================================
    // Sentence Analysis Methods (Layer 1 → Layer 2 Bridge)
    // ========================================================================

    /// Analyze a complete sentence, returning tokens with POS and dependencies
    pub fn analyze_sentence(&self, text: &str) -> EngineResult<SentenceAnalysisResult> {
        let start = Instant::now();

        let words = self.tokenize_simple(text);

        if words.is_empty() {
            return Err(canopy_engine::EngineError::data_load(
                "Empty sentence - no tokens found".to_string(),
            ));
        }

        let mut tokens: Vec<Layer1SemanticResult> = Vec::with_capacity(words.len());
        for word in &words {
            let mut result = self.analyze(word)?;
            if result.pos.is_none() {
                result.pos = self.infer_pos_from_semantics(&result);
            }
            tokens.push(result);
        }

        let dependencies = self.infer_dependencies(&tokens);
        let metadata = self.detect_sentence_metadata(&tokens, &dependencies, text);

        let processing_time = start.elapsed().as_micros() as u64;

        Ok(SentenceAnalysisResult {
            text: text.to_string(),
            tokens,
            dependencies,
            metadata,
            processing_time_us: processing_time,
        })
    }

    /// Analyze multi-sentence text with hierarchical processing
    ///
    /// This method handles text containing multiple sentences:
    /// 1. Splits text into sentences
    /// 2. Analyzes each sentence
    /// 3. Aggregates statistics
    ///
    /// For single sentences, use `analyze_sentence()` directly.
    pub fn analyze_text(&self, text: &str) -> EngineResult<TextAnalysisResult> {
        let start = Instant::now();

        // Split into sentences
        let sentence_texts = self.split_sentences(text);

        if sentence_texts.is_empty() {
            return Ok(TextAnalysisResult {
                text: text.to_string(),
                sentences: Vec::new(),
                stats: TextAnalysisStats::default(),
            });
        }

        // Analyze each sentence
        let mut sentences = Vec::with_capacity(sentence_texts.len());
        let mut total_tokens = 0;

        for sentence_text in &sentence_texts {
            match self.analyze_sentence(sentence_text) {
                Ok(result) => {
                    total_tokens += result.tokens.len();
                    sentences.push(result);
                }
                Err(e) => {
                    // Log but continue with other sentences
                    tracing::warn!("Failed to analyze sentence '{}': {:?}", sentence_text, e);
                }
            }
        }

        // Gather cache stats
        let (cache_hits, cache_misses) = if let Ok(stats) = self.stats.lock() {
            (stats.cache_hits as u64, stats.cache_misses as u64)
        } else {
            (0u64, 0u64)
        };

        let total_time = start.elapsed().as_micros() as u64;

        Ok(TextAnalysisResult {
            text: text.to_string(),
            sentences,
            stats: TextAnalysisStats {
                total_time_us: total_time,
                sentences_processed: sentence_texts.len(),
                tokens_processed: total_tokens,
                unique_words: 0, // Would require tracking unique lemmas
                cache_hits,
                cache_misses,
            },
        })
    }

    /// Split text into sentences using simple heuristics
    fn split_sentences(&self, text: &str) -> Vec<String> {
        let mut sentences = Vec::new();
        let mut current = String::new();

        for c in text.chars() {
            current.push(c);
            // Sentence-ending punctuation
            if c == '.' || c == '!' || c == '?' {
                let trimmed = current.trim().to_string();
                if !trimmed.is_empty() {
                    sentences.push(trimmed);
                }
                current.clear();
            }
        }

        // Don't forget the last sentence if it doesn't end with punctuation
        let trimmed = current.trim().to_string();
        if !trimmed.is_empty() {
            sentences.push(trimmed);
        }

        sentences
    }

    /// Simple tokenization: split on whitespace and separate punctuation
    fn tokenize_simple(&self, text: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        let mut current = String::new();

        for c in text.chars() {
            if c.is_whitespace() {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
            } else if c.is_ascii_punctuation() && c != '\'' && c != '-' {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
                tokens.push(c.to_string());
            } else {
                current.push(c);
            }
        }

        if !current.is_empty() {
            tokens.push(current);
        }

        tokens
    }

    /// Infer POS from semantic evidence in the Layer 1 result
    fn infer_pos_from_semantics(&self, result: &Layer1SemanticResult) -> Option<UPos> {
        // Priority 1: VerbNet match → Verb
        if let Some(ref vn) = result.verbnet {
            if !vn.verb_classes.is_empty() {
                return Some(UPos::Verb);
            }
        }

        // Priority 2: WordNet POS
        if let Some(ref wn) = result.wordnet {
            if !wn.synsets.is_empty() {
                return Some(wordnet_pos_to_upos(wn.pos));
            }
        }

        // Priority 3: Lexicon (closed-class words)
        if let Some(ref lex) = result.lexicon {
            if let Some(classification) = lex.classifications.first() {
                use canopy_semantic_engines::lexicon::WordClassType;
                let pos = match classification.word_class_type {
                    WordClassType::Quantifiers => Some(UPos::Det),
                    WordClassType::Modal => Some(UPos::Aux),
                    WordClassType::Pronouns => Some(UPos::Pron),
                    WordClassType::Prepositions => Some(UPos::Adp),
                    WordClassType::Conjunctions => Some(UPos::Cconj),
                    WordClassType::Intensifiers => Some(UPos::Adv),
                    WordClassType::Negation => Some(UPos::Part),
                    WordClassType::StopWords => Some(UPos::Det),
                    _ => None,
                };
                if pos.is_some() {
                    return pos;
                }
            }
        }

        // Priority 4: Morphological suffix heuristics
        if let Some(wn_pos) = guess_pos_from_suffix(&result.lemma) {
            return Some(wordnet_pos_to_upos(wn_pos));
        }

        // Priority 5: PropBank presence suggests verb
        if let Some(ref pb) = result.propbank {
            if pb.predicate.is_some() || !pb.alternative_rolesets.is_empty() {
                return Some(UPos::Verb);
            }
        }

        // Priority 6: Treebank word→POS statistics
        if let Some(ref provider) = self.treebank_provider {
            if let Some(pos) = provider.get_pos_for_word(&result.lemma) {
                return Some(pos);
            }
            if let Some(pos) = provider.get_pos_for_word(&result.original_word) {
                return Some(pos);
            }
        }

        None
    }

    /// Infer dependency arcs using SVO heuristics
    fn infer_dependencies(&self, tokens: &[Layer1SemanticResult]) -> Vec<DependencyArc> {
        let mut deps = Vec::new();

        let verb_idx = tokens
            .iter()
            .position(|t| matches!(t.pos, Some(UPos::Verb) | Some(UPos::Aux)));

        let Some(root_idx) = verb_idx else {
            return deps;
        };

        // Find subject: first noun/pronoun BEFORE the verb
        for (i, token) in tokens.iter().enumerate().take(root_idx) {
            if matches!(
                token.pos,
                Some(UPos::Noun) | Some(UPos::Propn) | Some(UPos::Pron)
            ) {
                deps.push(DependencyArc::with_confidence(
                    root_idx,
                    i,
                    DependencyRelation::NominalSubject,
                    0.8,
                ));
                break;
            }
        }

        // Find object: first noun/pronoun AFTER the verb
        for (i, token) in tokens.iter().enumerate().skip(root_idx + 1) {
            if matches!(
                token.pos,
                Some(UPos::Noun) | Some(UPos::Propn) | Some(UPos::Pron)
            ) {
                deps.push(DependencyArc::with_confidence(
                    root_idx,
                    i,
                    DependencyRelation::Object,
                    0.8,
                ));
                break;
            }
        }

        // Attach determiners to following nouns
        for (i, token) in tokens.iter().enumerate() {
            if token.pos == Some(UPos::Det) {
                for (j, following) in tokens.iter().enumerate().skip(i + 1) {
                    if matches!(following.pos, Some(UPos::Noun) | Some(UPos::Propn)) {
                        deps.push(DependencyArc::with_confidence(
                            j,
                            i,
                            DependencyRelation::Determiner,
                            0.9,
                        ));
                        break;
                    }
                }
            }
        }

        // Attach adjectives to following nouns
        for (i, token) in tokens.iter().enumerate() {
            if token.pos == Some(UPos::Adj) {
                for (j, following) in tokens.iter().enumerate().skip(i + 1) {
                    if matches!(following.pos, Some(UPos::Noun) | Some(UPos::Propn)) {
                        deps.push(DependencyArc::with_confidence(
                            j,
                            i,
                            DependencyRelation::AdjectivalModifier,
                            0.85,
                        ));
                        break;
                    }
                }
            }
        }

        // Attach adverbs to the verb
        for (i, token) in tokens.iter().enumerate() {
            if token.pos == Some(UPos::Adv) {
                deps.push(DependencyArc::with_confidence(
                    root_idx,
                    i,
                    DependencyRelation::AdverbialModifier,
                    0.75,
                ));
            }
        }

        deps
    }

    /// Detect sentence-level metadata
    fn detect_sentence_metadata(
        &self,
        tokens: &[Layer1SemanticResult],
        _deps: &[DependencyArc],
        text: &str,
    ) -> SentenceMetadata {
        let mut metadata = SentenceMetadata::default();

        // Check for interrogative
        if text.ends_with('?') {
            metadata.is_interrogative = true;
        } else {
            let first_word = tokens.first().map(|t| t.lemma.to_lowercase());
            if matches!(
                first_word.as_deref(),
                Some("who")
                    | Some("what")
                    | Some("where")
                    | Some("when")
                    | Some("why")
                    | Some("how")
            ) {
                metadata.is_interrogative = true;
            }
        }

        // Check for negation
        for token in tokens {
            let lemma = token.lemma.to_lowercase();
            if lemma == "not" || lemma == "n't" || lemma == "never" || lemma == "no" {
                metadata.is_negated = true;
                break;
            }
        }

        // Check for passive voice
        let lemmas: Vec<_> = tokens.iter().map(|t| t.lemma.to_lowercase()).collect();
        for i in 0..lemmas.len().saturating_sub(1) {
            if matches!(
                lemmas[i].as_str(),
                "was" | "were" | "been" | "being" | "is" | "are" | "be"
            ) && matches!(tokens.get(i + 1).and_then(|t| t.pos), Some(UPos::Verb))
            {
                let next_lemma = &lemmas[i + 1];
                if next_lemma.ends_with("ed") || next_lemma.ends_with("en") {
                    metadata.is_passive = true;
                    break;
                }
            }
        }

        // Check for imperative
        if let Some(first) = tokens.first() {
            if matches!(first.pos, Some(UPos::Verb)) {
                metadata.is_imperative = true;
            }
        }

        metadata
    }
}

/// Create a Layer 1 analyzer with default configuration
pub fn create_l1_analyzer() -> EngineResult<SemanticCoordinator> {
    SemanticCoordinator::new(CoordinatorConfig::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_coordinator() -> Option<SemanticCoordinator> {
        SemanticCoordinator::new(CoordinatorConfig::default()).ok()
    }

    #[test]
    fn test_tokenize_simple() {
        let Some(coordinator) = create_test_coordinator() else {
            eprintln!("Skipping test: coordinator not available");
            return;
        };

        let tokens = coordinator.tokenize_simple("John runs fast.");
        assert_eq!(tokens, vec!["John", "runs", "fast", "."]);

        let tokens2 = coordinator.tokenize_simple("The big dog");
        assert_eq!(tokens2, vec!["The", "big", "dog"]);
    }

    #[test]
    fn test_analyze_sentence_basic() {
        let Some(coordinator) = create_test_coordinator() else {
            eprintln!("Skipping test: coordinator not available");
            return;
        };

        let result = coordinator.analyze_sentence("John runs").unwrap();
        assert_eq!(result.tokens.len(), 2);
        assert_eq!(result.tokens[0].original_word, "John");
        assert_eq!(result.tokens[1].original_word, "runs");
        assert!(result.processing_time_us > 0);
    }

    #[test]
    fn test_analyze_sentence_with_dependencies() {
        let Some(coordinator) = create_test_coordinator() else {
            eprintln!("Skipping test: coordinator not available");
            return;
        };

        let result = coordinator.analyze_sentence("John broke the vase").unwrap();
        assert_eq!(result.tokens.len(), 4);

        println!("Tokens:");
        for (i, token) in result.tokens.iter().enumerate() {
            println!("  {}: {} -> {:?}", i, token.original_word, token.pos);
        }

        println!("Dependencies:");
        for dep in &result.dependencies {
            println!(
                "  {}({}, {})",
                dep.relation,
                result.tokens[dep.head_idx].original_word,
                result.tokens[dep.dependent_idx].original_word
            );
        }
    }

    #[test]
    fn test_sentence_metadata_interrogative() {
        let Some(coordinator) = create_test_coordinator() else {
            eprintln!("Skipping test: coordinator not available");
            return;
        };

        let result = coordinator.analyze_sentence("What runs?").unwrap();
        assert!(result.metadata.is_interrogative);
    }

    #[test]
    fn test_sentence_metadata_negation() {
        let Some(coordinator) = create_test_coordinator() else {
            eprintln!("Skipping test: coordinator not available");
            return;
        };

        let result = coordinator.analyze_sentence("John did not run").unwrap();
        assert!(result.metadata.is_negated);
    }

    #[test]
    fn test_empty_sentence_error() {
        let Some(coordinator) = create_test_coordinator() else {
            eprintln!("Skipping test: coordinator not available");
            return;
        };

        let result = coordinator.analyze_sentence("");
        assert!(result.is_err());
    }

    #[test]
    fn test_find_predicates() {
        let Some(coordinator) = create_test_coordinator() else {
            eprintln!("Skipping test: coordinator not available");
            return;
        };

        let result = coordinator
            .analyze_sentence("John gave Mary a book")
            .unwrap();
        let predicates = result.find_predicates();

        println!("Predicate indices: {:?}", predicates);
        for idx in &predicates {
            println!(
                "  Predicate: {} at index {}",
                result.tokens[*idx].original_word, idx
            );
        }
    }

    #[test]
    fn test_dependency_arc_creation() {
        let arc = DependencyArc::new(1, 0, DependencyRelation::NominalSubject);
        assert_eq!(arc.head_idx, 1);
        assert_eq!(arc.dependent_idx, 0);
        assert_eq!(arc.relation, DependencyRelation::NominalSubject);
        assert_eq!(arc.confidence, 1.0);

        let arc2 = DependencyArc::with_confidence(2, 1, DependencyRelation::Object, 0.9);
        assert_eq!(arc2.confidence, 0.9);
    }

    #[test]
    fn test_dependency_relation_display() {
        assert_eq!(format!("{}", DependencyRelation::NominalSubject), "nsubj");
        assert_eq!(format!("{}", DependencyRelation::Object), "obj");
        assert_eq!(format!("{}", DependencyRelation::Determiner), "det");
    }
}
