//! Integration module for connecting treebank engine with semantic analysis
//!
//! This module provides a bridge between the multi-tier treebank caching system
//! and the SemanticCoordinator, allowing dependency patterns to be added to
//! Layer 1 semantic analysis results.

use crate::{
    DependencyPattern, PatternCache, PatternCacheFactory, PatternIndexer, SemanticSignature,
    TreebankResult,
};
use canopy_engine::{EngineError, LemmaSource};
use canopy_tokenizer::{
    coordinator::{CoordinatorConfig, Layer1SemanticResult},
    SemanticCoordinator,
};
use std::time::Instant;
use tracing::{debug, info};

/// Extended semantic result that includes dependency patterns
#[derive(Debug, Clone)]
pub struct ExtendedSemanticResult {
    /// Base semantic result from Layer 1
    pub semantic_result: Layer1SemanticResult,
    /// Matched dependency pattern (if found)
    pub dependency_pattern: Option<DependencyPattern>,
    /// Pattern matching confidence
    pub pattern_confidence: f32,
    /// Time taken for pattern matching (microseconds)
    pub pattern_lookup_time_us: u64,
    /// Whether pattern came from cache
    pub pattern_from_cache: bool,
}

/// Configuration for treebank-enhanced semantic coordinator
#[derive(Debug, Clone)]
pub struct TreebankSemanticConfig {
    /// Base semantic coordinator configuration
    pub coordinator_config: CoordinatorConfig,
    /// Enable dependency pattern matching
    pub enable_dependency_patterns: bool,
    /// Path to treebank corpus for pattern indexing
    pub corpus_path: Option<std::path::PathBuf>,
    /// Minimum pattern confidence threshold
    pub min_pattern_confidence: f32,
}

impl Default for TreebankSemanticConfig {
    fn default() -> Self {
        Self {
            coordinator_config: CoordinatorConfig::default(),
            enable_dependency_patterns: true,
            corpus_path: None,
            min_pattern_confidence: 0.5,
        }
    }
}

/// Enhanced semantic coordinator with dependency pattern analysis
pub struct TreebankSemanticCoordinator {
    /// Base semantic coordinator
    coordinator: SemanticCoordinator,
    /// Multi-tier pattern cache
    pattern_cache: PatternCache,
    /// Configuration
    config: TreebankSemanticConfig,
    /// Performance statistics
    stats: TreebankSemanticStats,
}

#[derive(Debug, Default, Clone)]
pub struct TreebankSemanticStats {
    pub total_analyses: u64,
    pub pattern_matches: u64,
    pub cache_hits: u64,
    pub pattern_synthesis_attempts: u64,
    pub avg_lookup_time_us: f64,
}

impl TreebankSemanticCoordinator {
    /// Create a new treebank-enhanced semantic coordinator
    pub fn new(config: TreebankSemanticConfig) -> TreebankResult<Self> {
        info!("Creating TreebankSemanticCoordinator with dependency pattern support");

        // Create base semantic coordinator
        let coordinator =
            SemanticCoordinator::new(config.coordinator_config.clone()).map_err(|e| {
                EngineError::ConfigError {
                    message: format!("Failed to create SemanticCoordinator: {:?}", e),
                }
            })?;

        // Create pattern cache
        let mut pattern_cache = PatternCacheFactory::create_m6_optimized(None)?;

        // Index patterns from corpus if provided
        if let Some(ref corpus_path) = config.corpus_path {
            if corpus_path.exists() {
                info!("Indexing patterns from corpus: {:?}", corpus_path);
                let indexing_start = Instant::now();

                let mut indexer = PatternIndexer::new();
                indexer.index_from_corpus(corpus_path)?;

                let patterns = indexer.get_patterns_by_frequency();
                pattern_cache.populate_core_cache(&patterns[..2000.min(patterns.len())]);

                info!(
                    "Indexed {} patterns in {:.2}s",
                    patterns.len(),
                    indexing_start.elapsed().as_secs_f32()
                );
            } else {
                debug!(
                    "Corpus path {:?} does not exist, starting with empty cache",
                    corpus_path
                );
            }
        }

        Ok(Self {
            coordinator,
            pattern_cache,
            config,
            stats: TreebankSemanticStats::default(),
        })
    }

    /// Analyze text with enhanced dependency pattern matching
    pub fn analyze_enhanced(&mut self, text: &str) -> TreebankResult<ExtendedSemanticResult> {
        let start_time = Instant::now();

        // Step 1: Perform base semantic analysis
        let semantic_result =
            self.coordinator
                .analyze(text)
                .map_err(|e| EngineError::ConfigError {
                    message: format!("Semantic analysis failed: {:?}", e),
                })?;

        // Step 2: Attempt dependency pattern matching if enabled
        let (dependency_pattern, pattern_confidence, pattern_from_cache, pattern_lookup_time_us) =
            if self.config.enable_dependency_patterns && !semantic_result.lemma.is_empty() {
                self.match_dependency_pattern(&semantic_result)
            } else {
                (None, 0.0, false, 0)
            };

        // Update statistics
        self.stats.total_analyses += 1;
        if dependency_pattern.is_some() {
            self.stats.pattern_matches += 1;
            if pattern_from_cache {
                self.stats.cache_hits += 1;
            }
        }

        let total_time = start_time.elapsed().as_micros() as u64;
        self.stats.avg_lookup_time_us = (self.stats.avg_lookup_time_us
            * (self.stats.total_analyses - 1) as f64
            + total_time as f64)
            / self.stats.total_analyses as f64;

        Ok(ExtendedSemanticResult {
            semantic_result,
            dependency_pattern,
            pattern_confidence,
            pattern_lookup_time_us,
            pattern_from_cache,
        })
    }

    /// Match dependency patterns for the semantic result using indexed patterns
    fn match_dependency_pattern(
        &mut self,
        semantic_result: &Layer1SemanticResult,
    ) -> (Option<DependencyPattern>, f32, bool, u64) {
        let lookup_start = Instant::now();

        // Create semantic signature for lookup
        let signature = SemanticSignature {
            lemma: semantic_result.lemma.clone(),
            verbnet_class: None,
            framenet_frame: None,
            pos_category: self.infer_pos_category(&semantic_result.original_word),
            lemma_source: LemmaSource::UDGold,
            lemma_confidence: semantic_result.lemmatization_confidence.unwrap_or(0.8),
            hash_code: 0,
        };

        // Try pattern lookup using pre-indexed treebank patterns
        if let Some(pattern) = self.pattern_cache.get_pattern(&signature) {
            let lookup_time = lookup_start.elapsed().as_micros() as u64;
            let confidence = if pattern.confidence >= self.config.min_pattern_confidence {
                pattern.confidence
            } else {
                0.0
            };

            debug!(
                "Found dependency pattern for '{}' with confidence {:.2}",
                semantic_result.lemma, confidence
            );

            (Some(pattern), confidence, true, lookup_time)
        } else {
            // Pattern synthesis could be attempted here
            self.stats.pattern_synthesis_attempts += 1;
            let lookup_time = lookup_start.elapsed().as_micros() as u64;

            debug!(
                "No dependency pattern found for '{}'",
                semantic_result.lemma
            );
            (None, 0.0, false, lookup_time)
        }
    }

    /// Simple POS category inference from word
    fn infer_pos_category(&self, word: &str) -> crate::signature::PosCategory {
        use crate::signature::PosCategory;

        // Simple heuristics - would be more sophisticated in real implementation
        if word.ends_with("ing") || word.ends_with("ed") || word.ends_with("s") {
            PosCategory::Verb
        } else if word.ends_with("ly") {
            PosCategory::Adverb
        } else if word.ends_with("tion") || word.ends_with("ness") {
            PosCategory::Noun
        } else {
            PosCategory::Other
        }
    }

    /// Get performance statistics
    pub fn get_statistics(&self) -> &TreebankSemanticStats {
        &self.stats
    }

    /// Get cache statistics
    pub fn get_cache_statistics(&self) -> &crate::CacheStatistics {
        self.pattern_cache.get_statistics()
    }

    /// Populate cache with demo patterns for testing
    pub fn populate_demo_patterns(&mut self, patterns: &[(String, DependencyPattern)]) {
        self.pattern_cache.populate_core_cache(patterns);
    }

    /// Print detailed performance report
    pub fn print_performance_report(&self) {
        println!("🚀 Treebank-Enhanced Semantic Analysis Performance");
        println!("   Total analyses: {}", self.stats.total_analyses);
        println!(
            "   Pattern matches: {} ({:.1}%)",
            self.stats.pattern_matches,
            if self.stats.total_analyses > 0 {
                self.stats.pattern_matches as f64 / self.stats.total_analyses as f64 * 100.0
            } else {
                0.0
            }
        );
        println!(
            "   Cache hits: {} ({:.1}%)",
            self.stats.cache_hits,
            if self.stats.pattern_matches > 0 {
                self.stats.cache_hits as f64 / self.stats.pattern_matches as f64 * 100.0
            } else {
                0.0
            }
        );
        println!(
            "   Average lookup time: {:.1}μs",
            self.stats.avg_lookup_time_us
        );
        println!(
            "   Pattern synthesis attempts: {}",
            self.stats.pattern_synthesis_attempts
        );

        println!("\n🔍 Pattern Cache Statistics:");
        self.pattern_cache.print_statistics();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_treebank_coordinator_creation() {
        let config = TreebankSemanticConfig::default();
        let coordinator = TreebankSemanticCoordinator::new(config);
        // Skip if data isn't available
        if let Err(e) = &coordinator {
            if e.to_string().contains("not found") || e.to_string().contains("No such file") {
                eprintln!("Skipping test: semantic data not available");
                return;
            }
        }
        assert!(coordinator.is_ok());
    }

    #[test]
    fn test_enhanced_analysis() -> Result<(), Box<dyn std::error::Error>> {
        let config = TreebankSemanticConfig::default();
        let mut coordinator = match TreebankSemanticCoordinator::new(config) {
            Ok(c) => c,
            Err(e) => {
                if e.to_string().contains("not found") || e.to_string().contains("No such file") {
                    eprintln!("Skipping test: semantic data not available");
                    return Ok(());
                }
                return Err(e.into());
            }
        };

        let result = coordinator.analyze_enhanced("running")?;

        // Should have base semantic analysis
        assert!(!result.semantic_result.original_word.is_empty());
        assert_eq!(result.semantic_result.original_word, "running");

        // Pattern matching might not find anything without corpus data
        // but should not fail
        assert!(result.pattern_lookup_time_us < 1000); // Should be fast

        Ok(())
    }

    #[test]
    fn test_performance_tracking() -> Result<(), Box<dyn std::error::Error>> {
        let config = TreebankSemanticConfig::default();
        let mut coordinator = match TreebankSemanticCoordinator::new(config) {
            Ok(c) => c,
            Err(e) => {
                if e.to_string().contains("not found") || e.to_string().contains("No such file") {
                    eprintln!("Skipping test: semantic data not available");
                    return Ok(());
                }
                return Err(e.into());
            }
        };

        // Perform multiple analyses
        for word in &["run", "walk", "eat", "sleep"] {
            let _result = coordinator.analyze_enhanced(word)?;
        }

        let stats = coordinator.get_statistics();
        assert_eq!(stats.total_analyses, 4);
        assert!(stats.avg_lookup_time_us > 0.0);

        Ok(())
    }
}
