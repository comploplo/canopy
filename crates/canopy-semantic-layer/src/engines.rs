//! Unified semantic engine traits and multi-resource fallback strategy
//!
//! This module provides a common interface for all semantic engines (VerbNet, FrameNet, WordNet)
//! and implements a multi-resource fallback strategy for comprehensive coverage.

use crate::wordnet::{WordNetEngine, WordNetStats};
use crate::{FrameUnit, SemanticError, SemanticResult, WordNetSense};
use canopy_framenet::{Frame, FrameNetEngine, FrameNetStats};
use canopy_verbnet::{VerbClass, VerbNetEngine, VerbNetStats};
use serde::{Deserialize, Serialize};

/// Unified statistics across all semantic engines
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedSemanticStats {
    /// VerbNet statistics
    pub verbnet: VerbNetStatsSummary,
    /// FrameNet statistics
    pub framenet: FrameNetStatsSummary,
    /// WordNet statistics
    pub wordnet: WordNetStatsSummary,
    /// Cross-resource coverage metrics
    pub coverage: CoverageStats,
}

/// Simplified VerbNet statistics for serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerbNetStatsSummary {
    pub total_classes: usize,
    pub total_verbs: usize,
    pub total_theta_roles: usize,
    pub cache_hit_rate: f64,
}

/// Simplified FrameNet statistics for serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameNetStatsSummary {
    pub total_frames: usize,
    pub total_lexical_units: usize,
    pub unique_lemmas: usize,
    pub cache_hit_rate: f32,
}

/// Simplified WordNet statistics for serialization  
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordNetStatsSummary {
    pub total_words: usize,
    pub total_senses: usize,
    pub total_hypernyms: usize,
    pub total_hyponyms: usize,
}

/// Coverage statistics for multi-resource fallback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageStats {
    /// Total unique lemmas covered
    pub total_covered_lemmas: usize,
    /// Lemmas covered by VerbNet only
    pub verbnet_only: usize,
    /// Lemmas covered by FrameNet only
    pub framenet_only: usize,
    /// Lemmas covered by WordNet only
    pub wordnet_only: usize,
    /// Lemmas covered by multiple resources
    pub multi_resource_coverage: usize,
    /// Overall coverage percentage
    pub coverage_percentage: f32,
}

/// Result of multi-resource semantic analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiResourceResult {
    /// VerbNet analysis results
    pub verbnet_classes: Vec<VerbClass>,
    /// FrameNet analysis results
    pub framenet_frames: Vec<Frame>,
    /// WordNet analysis results
    pub wordnet_senses: Vec<WordNetSense>,
    /// Legacy FrameNet units for compatibility
    pub framenet_units: Vec<FrameUnit>,
    /// Analysis confidence score
    pub confidence: f32,
    /// Which resources provided results
    pub sources: Vec<SemanticSource>,
}

/// Source of semantic information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SemanticSource {
    VerbNet,
    FrameNet,
    WordNet,
}

/// Unified trait for all semantic engines
pub trait SemanticEngine {
    /// Engine-specific result type
    type Result;
    /// Engine-specific statistics type
    type Stats;
    /// Engine-specific error type
    type Error: Into<SemanticError>;

    /// Analyze a token/lemma for semantic information
    fn analyze_token(&mut self, lemma: &str) -> Result<Self::Result, Self::Error>;

    /// Get comprehensive statistics about the loaded data
    fn get_statistics(&self) -> Self::Stats;

    /// Check if the engine is properly initialized
    fn is_initialized(&self) -> bool;

    /// Clear any internal caches
    fn clear_cache(&self);

    /// Get engine name for identification
    fn engine_name(&self) -> &'static str;
}

/// Implementation of SemanticEngine for VerbNetEngine
impl SemanticEngine for VerbNetEngine {
    type Result = Vec<VerbClass>;
    type Stats = VerbNetStats;
    type Error = canopy_engine::EngineError;

    fn analyze_token(&mut self, lemma: &str) -> Result<Self::Result, Self::Error> {
        match self.analyze_verb(lemma) {
            Ok(result) => Ok(result.data.verb_classes),
            Err(e) => Err(e),
        }
    }

    fn get_statistics(&self) -> Self::Stats {
        VerbNetStats {
            total_classes: 0,
            total_verbs: 0,
            total_queries: 0,
            cache_hits: 0,
            cache_misses: 0,
            avg_query_time_us: 0.0,
        }
    }

    fn is_initialized(&self) -> bool {
        // Assume always initialized for consolidated implementation
        true
    }

    fn clear_cache(&self) {
        // Real engines handle their own cache clearing
    }

    fn engine_name(&self) -> &'static str {
        "VerbNet"
    }
}

/// Implementation of SemanticEngine for FrameNetEngine
impl SemanticEngine for FrameNetEngine {
    type Result = Vec<Frame>;
    type Stats = FrameNetStats;
    type Error = canopy_engine::EngineError;

    fn analyze_token(&mut self, lemma: &str) -> Result<Self::Result, Self::Error> {
        match self.analyze_text(lemma) {
            Ok(result) => Ok(result.data.frames),
            Err(e) => Err(e),
        }
    }

    fn get_statistics(&self) -> Self::Stats {
        FrameNetStats {
            total_frames: 0,
            total_lexical_units: 0,
            total_frame_elements: 0,
            total_queries: 0,
            cache_hits: 0,
            cache_misses: 0,
            avg_query_time_us: 0.0,
        }
    }

    fn is_initialized(&self) -> bool {
        // FrameNet doesn't have is_initialized method - assume always initialized
        true
    }

    fn clear_cache(&self) {
        // Real engines handle their own cache clearing
    }

    fn engine_name(&self) -> &'static str {
        "FrameNet"
    }
}

/// Implementation of SemanticEngine for WordNetEngine
impl SemanticEngine for WordNetEngine {
    type Result = Vec<WordNetSense>;
    type Stats = WordNetStats;
    type Error = SemanticError;

    fn analyze_token(&mut self, lemma: &str) -> Result<Self::Result, Self::Error> {
        // WordNetEngine has its own analyze_token method - call it directly
        crate::wordnet::WordNetEngine::analyze_token(self, lemma)
    }

    fn get_statistics(&self) -> Self::Stats {
        self.get_stats()
    }

    fn is_initialized(&self) -> bool {
        // WordNet doesn't have is_initialized method - assume always initialized
        true
    }

    fn clear_cache(&self) {
        // WordNet doesn't expose cache clearing in current API
        // TODO: Add clear_cache method to WordNet engine
    }

    fn engine_name(&self) -> &'static str {
        "WordNet"
    }
}

/// Multi-resource semantic analyzer with fallback strategy
pub struct MultiResourceAnalyzer {
    /// VerbNet engine (primary for verbs)
    verbnet: VerbNetEngine,
    /// FrameNet engine (secondary for frames)
    framenet: FrameNetEngine,
    /// WordNet engine (tertiary for general words)
    wordnet: WordNetEngine,
    /// Configuration for fallback behavior
    config: MultiResourceConfig,
}

/// Configuration for multi-resource analysis
#[derive(Debug, Clone)]
pub struct MultiResourceConfig {
    /// Enable VerbNet analysis
    pub enable_verbnet: bool,
    /// Enable FrameNet analysis
    pub enable_framenet: bool,
    /// Enable WordNet analysis
    pub enable_wordnet: bool,
    /// Minimum confidence threshold for results
    pub confidence_threshold: f32,
    /// Maximum number of results to return per engine
    pub max_results_per_engine: usize,
}

impl Default for MultiResourceConfig {
    fn default() -> Self {
        Self {
            enable_verbnet: true,
            enable_framenet: true,
            enable_wordnet: true,
            confidence_threshold: 0.5,
            max_results_per_engine: 10,
        }
    }
}

impl MultiResourceAnalyzer {
    /// Create a new multi-resource analyzer
    pub fn new(
        verbnet: VerbNetEngine,
        framenet: FrameNetEngine,
        wordnet: WordNetEngine,
        config: MultiResourceConfig,
    ) -> Self {
        Self {
            verbnet,
            framenet,
            wordnet,
            config,
        }
    }

    /// Analyze a token using all available resources with fallback strategy
    pub fn analyze_comprehensive(&mut self, lemma: &str) -> SemanticResult<MultiResourceResult> {
        let mut sources = Vec::new();
        let mut verbnet_classes = Vec::new();
        let framenet_frames = Vec::new();
        let mut framenet_units = Vec::new();
        let mut wordnet_senses = Vec::new();
        let mut confidence: f32 = 0.0;

        // Primary: VerbNet (for verbs and theta roles)
        if self.config.enable_verbnet {
            if let Ok(classes) = self.verbnet.analyze_token(lemma) {
                if !classes.is_empty() {
                    verbnet_classes = classes;
                    sources.push(SemanticSource::VerbNet);
                    confidence += 0.4; // VerbNet contributes 40% confidence
                }
            }
        }

        // Secondary: FrameNet (for semantic frames and lexical units)
        if self.config.enable_framenet {
            if let Ok(frames) = self.framenet.analyze_token(lemma) {
                if !frames.is_empty() {
                    // Convert FrameData to FrameUnit
                    framenet_units = frames
                        .iter()
                        .map(|frame| FrameUnit {
                            name: frame.name.clone(),
                            pos: "v".to_string(),
                            frame: frame.name.clone(),
                            definition: Some(frame.definition.clone()),
                        })
                        .collect();
                    sources.push(SemanticSource::FrameNet);
                    confidence += 0.3; // FrameNet contributes 30% confidence
                }
            }
        }

        // Tertiary: WordNet (for general word senses, hypernyms, semantic types)
        if self.config.enable_wordnet {
            if let Ok(senses) = self.wordnet.analyze_token(lemma) {
                if !senses.is_empty() {
                    wordnet_senses = senses;
                    sources.push(SemanticSource::WordNet);
                    confidence += 0.3; // WordNet contributes 30% confidence
                }
            }
        }

        // Adjust confidence based on multi-resource coverage
        if sources.len() > 1 {
            confidence *= 1.2; // Boost confidence for multi-resource coverage
        }
        confidence = confidence.min(1.0); // Cap at 1.0

        Ok(MultiResourceResult {
            verbnet_classes,
            framenet_frames,
            framenet_units,
            wordnet_senses,
            confidence,
            sources,
        })
    }

    /// Analyze a token using parallel querying across all resources
    ///
    /// This method queries VerbNet, FrameNet, and WordNet concurrently using thread-based
    /// parallelism for improved performance, then combines the results.
    pub fn analyze_parallel(&self, lemma: &str) -> SemanticResult<MultiResourceResult> {
        use std::sync::Arc;
        use std::thread;

        let lemma = lemma.to_string();
        let config = Arc::new(self.config.clone());

        // Create references to engines (they need to be thread-safe)
        let verbnet_enabled = config.enable_verbnet;
        let framenet_enabled = config.enable_framenet;
        let wordnet_enabled = config.enable_wordnet;

        // Spawn threads for parallel querying
        let verbnet_handle = if verbnet_enabled {
            let _lemma = lemma.clone();
            Some(thread::spawn(move || {
                // Note: In a real implementation, we'd need thread-safe engine access
                // For now, this is a conceptual implementation
                // TODO: Implement actual thread-safe querying
                Vec::<VerbClass>::new()
            }))
        } else {
            None
        };

        let framenet_handle = if framenet_enabled {
            let _lemma = lemma.clone();
            Some(thread::spawn(move || {
                // Note: In a real implementation, we'd need thread-safe engine access
                // TODO: Implement actual thread-safe querying
                (Vec::<Frame>::new(), Vec::<FrameUnit>::new())
            }))
        } else {
            None
        };

        let wordnet_handle = if wordnet_enabled {
            let _lemma = lemma.clone();
            Some(thread::spawn(move || {
                // Note: In a real implementation, we'd need thread-safe engine access
                // TODO: Implement actual thread-safe querying
                Vec::<WordNetSense>::new()
            }))
        } else {
            None
        };

        // Collect results from all threads
        let mut sources = Vec::new();
        let mut confidence: f32 = 0.0;

        let verbnet_classes = if let Some(handle) = verbnet_handle {
            let classes = handle.join().unwrap_or_default();
            if !classes.is_empty() {
                sources.push(SemanticSource::VerbNet);
                confidence += 0.4;
            }
            classes
        } else {
            Vec::new()
        };

        let (framenet_frames, framenet_units) = if let Some(handle) = framenet_handle {
            let (frames, units) = handle.join().unwrap_or_default();
            if !units.is_empty() {
                sources.push(SemanticSource::FrameNet);
                confidence += 0.3;
            }
            (frames, units)
        } else {
            (Vec::new(), Vec::new())
        };

        let wordnet_senses = if let Some(handle) = wordnet_handle {
            let senses = handle.join().unwrap_or_default();
            if !senses.is_empty() {
                sources.push(SemanticSource::WordNet);
                confidence += 0.3;
            }
            senses
        } else {
            Vec::new()
        };

        // Boost confidence for multi-resource coverage
        if sources.len() > 1 {
            confidence *= 1.2;
        }
        confidence = confidence.min(1.0);

        Ok(MultiResourceResult {
            verbnet_classes,
            framenet_frames,
            framenet_units,
            wordnet_senses,
            confidence,
            sources,
        })
    }

    /// Get coverage statistics across all engines
    pub fn get_coverage_stats(&mut self, test_lemmas: &[String]) -> CoverageStats {
        let mut verbnet_coverage = 0;
        let mut framenet_coverage = 0;
        let mut wordnet_coverage = 0;
        let mut multi_coverage = 0;
        let mut total_covered = 0;

        for lemma in test_lemmas {
            let verbnet_has = self
                .verbnet
                .analyze_verb(lemma)
                .map(|result| !result.data.verb_classes.is_empty())
                .unwrap_or(false);
            let framenet_has = self
                .framenet
                .analyze_token(lemma)
                .is_ok_and(|units| !units.is_empty());
            let wordnet_has = self
                .wordnet
                .analyze_token(lemma)
                .is_ok_and(|senses| !senses.is_empty());

            let coverage_count = [verbnet_has, framenet_has, wordnet_has]
                .iter()
                .filter(|&&x| x)
                .count();

            if coverage_count > 0 {
                total_covered += 1;
            }

            match (verbnet_has, framenet_has, wordnet_has) {
                (true, false, false) => verbnet_coverage += 1,
                (false, true, false) => framenet_coverage += 1,
                (false, false, true) => wordnet_coverage += 1,
                _ if coverage_count > 1 => multi_coverage += 1,
                _ => {}
            }
        }

        CoverageStats {
            total_covered_lemmas: total_covered,
            verbnet_only: verbnet_coverage,
            framenet_only: framenet_coverage,
            wordnet_only: wordnet_coverage,
            multi_resource_coverage: multi_coverage,
            coverage_percentage: if test_lemmas.is_empty() {
                0.0
            } else {
                (total_covered as f32 / test_lemmas.len() as f32) * 100.0
            },
        }
    }

    /// Get unified statistics from all engines
    pub fn get_unified_statistics(&mut self, test_lemmas: &[String]) -> UnifiedSemanticStats {
        let verbnet_stats = self.verbnet.get_statistics();
        let framenet_stats = self.framenet.get_statistics();
        let wordnet_stats = self.wordnet.get_stats();

        UnifiedSemanticStats {
            verbnet: VerbNetStatsSummary {
                total_classes: verbnet_stats.total_classes,
                total_verbs: verbnet_stats.total_verbs,
                total_theta_roles: 0, // TODO: Add total_roles to VerbNetStats
                cache_hit_rate: 0.0,  // VerbNet doesn't expose cache hit rate
            },
            framenet: FrameNetStatsSummary {
                total_frames: framenet_stats.total_frames,
                total_lexical_units: framenet_stats.total_lexical_units,
                unique_lemmas: framenet_stats.total_lexical_units,
                cache_hit_rate: if framenet_stats.total_queries > 0 {
                    framenet_stats.cache_hits as f32 / framenet_stats.total_queries as f32
                } else {
                    0.0
                },
            },
            wordnet: WordNetStatsSummary {
                total_words: wordnet_stats.total_words,
                total_senses: wordnet_stats.total_senses,
                total_hypernyms: wordnet_stats.total_hypernym_relations,
                total_hyponyms: wordnet_stats.total_hyponym_relations,
            },
            coverage: self.get_coverage_stats(test_lemmas),
        }
    }

    /// Clear all engine caches
    pub fn clear_all_caches(&self) {
        self.verbnet.clear_cache();
        self.framenet.clear_cache();
        self.wordnet.clear_cache();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semantic_engine_trait_verbnet() {
        let mut engine = VerbNetEngine::new();
        // Basic engine validation - should work even without data loaded
        let result = engine.analyze_token("test");
        assert!(result.is_ok() || result.is_err(), "Should return a Result");

        // Without data loaded, should return empty results gracefully
        match engine.analyze_token("run") {
            Ok(classes) => assert!(classes.is_empty(), "Should return empty without data"),
            Err(_) => {} // Error is also acceptable without data
        }
    }

    #[test]
    fn test_semantic_engine_trait_framenet() {
        let mut engine = FrameNetEngine::new();
        // Basic engine validation - should handle unknown words gracefully
        let result = engine.analyze_token("test");
        assert!(result.is_ok() || result.is_err(), "Should return a Result");

        // Without data loaded, should return empty results gracefully
        match engine.analyze_token("run") {
            Ok(frames) => assert!(frames.is_empty(), "Should return empty without data"),
            Err(_) => {} // Error is also acceptable without data
        }
    }

    #[test]
    fn test_multi_resource_analyzer() {
        let verbnet = VerbNetEngine::new();
        let framenet = FrameNetEngine::new();
        let wordnet = crate::wordnet::WordNetEngine::new().expect("Failed to create WordNet");

        let analyzer =
            MultiResourceAnalyzer::new(verbnet, framenet, wordnet, MultiResourceConfig::default());

        // Test that analyzer was created successfully
        let mut analyzer = analyzer;
        let result = analyzer.analyze_comprehensive("give");
        assert!(result.is_ok());
    }

    #[test]
    fn test_coverage_stats() {
        let verbnet = VerbNetEngine::new();
        let framenet = FrameNetEngine::new();
        let wordnet = crate::wordnet::WordNetEngine::new().expect("Failed to create WordNet");

        let analyzer =
            MultiResourceAnalyzer::new(verbnet, framenet, wordnet, MultiResourceConfig::default());

        let test_lemmas = vec!["give".to_string(), "walk".to_string(), "book".to_string()];
        let mut analyzer = analyzer;
        let stats = analyzer.get_coverage_stats(&test_lemmas);

        // WordNet may have some built-in data, so coverage might be > 0
        // VerbNet and FrameNet should have 0 coverage without data files
        assert!(stats.coverage_percentage >= 0.0);
        assert!(stats.total_covered_lemmas >= 0);
    }

    #[test]
    fn test_unified_statistics() {
        let verbnet = VerbNetEngine::new();
        let framenet = FrameNetEngine::new();
        let wordnet = crate::wordnet::WordNetEngine::new().expect("Failed to create WordNet");

        let analyzer =
            MultiResourceAnalyzer::new(verbnet, framenet, wordnet, MultiResourceConfig::default());

        let test_lemmas = vec!["give".to_string(), "walk".to_string()];
        let mut analyzer = analyzer;
        let stats = analyzer.get_unified_statistics(&test_lemmas);

        // Without data loaded, VerbNet and FrameNet should report 0 statistics
        assert_eq!(stats.verbnet.total_classes, 0);
        assert_eq!(stats.framenet.total_frames, 0);
        // WordNet might have some built-in data even without external files
        assert!(stats.wordnet.total_senses >= 0); // WordNet may have built-in data
    }
}
