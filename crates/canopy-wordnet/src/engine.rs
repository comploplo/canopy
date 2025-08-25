//! WordNet semantic engine implementation
//!
//! This module provides the main WordNet engine that implements the canopy-engine traits
//! for semantic analysis using Princeton WordNet 3.1 data.

use crate::loader::WordNetLoader;
use crate::parser::WordNetParserConfig;
use crate::types::{PartOfSpeech, WordNetAnalysis, WordNetDatabase};
use canopy_engine::{EngineCache, EngineConfig, EngineError, EngineResult, EngineStats};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;

/// Configuration for WordNet engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordNetConfig {
    /// Base engine configuration
    pub base: EngineConfig,
    /// Path to WordNet data directory
    pub data_path: String,
    /// Parser configuration
    pub parser_config: WordNetParserConfig,
    /// Enable morphological processing
    pub enable_morphology: bool,
    /// Maximum search depth for semantic relations
    pub max_search_depth: usize,
    /// Minimum confidence threshold for results
    pub min_confidence: f32,
}

impl Default for WordNetConfig {
    fn default() -> Self {
        Self {
            base: EngineConfig::default(),
            data_path: "data/wordnet/dict".to_string(),
            parser_config: WordNetParserConfig::default(),
            enable_morphology: true,
            max_search_depth: 5,
            min_confidence: 0.1,
        }
    }
}

/// WordNet semantic engine
#[derive(Debug)]
pub struct WordNetEngine {
    config: WordNetConfig,
    database: Arc<WordNetDatabase>,
    cache: EngineCache<String, WordNetAnalysis>,
    stats: EngineStats,
    is_loaded: bool,
}

impl WordNetEngine {
    /// Create a new WordNet engine
    pub fn new(config: WordNetConfig) -> Self {
        let cache_capacity = config.base.cache_capacity;

        Self {
            config,
            database: Arc::new(WordNetDatabase::new()),
            cache: EngineCache::new(cache_capacity),
            stats: EngineStats::new("WordNet".to_string()),
            is_loaded: false,
        }
    }

    /// Load WordNet data from the configured path
    pub fn load_data(&mut self) -> EngineResult<()> {
        let start_time = Instant::now();

        let loader = WordNetLoader::new(self.config.parser_config.clone());
        let database = loader.load_database(&self.config.data_path)?;

        self.database = Arc::new(database);
        self.is_loaded = true;

        let load_time = start_time.elapsed();
        tracing::info!(
            "WordNet database loaded in {:.2}ms with {} synsets",
            load_time.as_secs_f64() * 1000.0,
            self.database.synsets.len()
        );

        Ok(())
    }

    /// Analyze a word and return semantic information
    pub fn analyze_word(&mut self, word: &str, pos: PartOfSpeech) -> EngineResult<WordNetAnalysis> {
        if !self.is_loaded {
            return Err(EngineError::data_load(
                "WordNet database not loaded".to_string(),
            ));
        }

        let cache_key = format!("{}:{}", word, pos.code());

        // Check cache first
        if self.config.base.enable_cache {
            if let Some(cached_result) = self.cache.get(&cache_key) {
                return Ok(cached_result.clone());
            }
        }

        let start_time = Instant::now();
        let mut analysis = WordNetAnalysis::new(word.to_string(), pos);

        // Look up the word in the index
        if let Some(index_entry) = self.database.lookup_word(word, pos) {
            // Get all synsets for this word
            analysis.synsets = self
                .database
                .get_synsets_for_word(word, pos)
                .into_iter()
                .cloned()
                .collect();

            // Extract definitions and examples
            for synset in &analysis.synsets {
                analysis.definitions.push(synset.definition());
                analysis.examples.extend(synset.examples());
            }

            // Find semantic relations
            for synset in &analysis.synsets {
                let hypernyms = self.database.get_hypernyms(synset);
                if !hypernyms.is_empty() {
                    analysis.relations.push((
                        crate::types::SemanticRelation::Hypernym,
                        hypernyms.into_iter().cloned().collect(),
                    ));
                }

                let hyponyms = self.database.get_hyponyms(synset);
                if !hyponyms.is_empty() {
                    analysis.relations.push((
                        crate::types::SemanticRelation::Hyponym,
                        hyponyms.into_iter().cloned().collect(),
                    ));
                }
            }

            // Calculate confidence based on number of senses and tag counts
            analysis.confidence = self.calculate_confidence(&analysis, index_entry);
        }

        let _processing_time = start_time.elapsed();

        // Cache the result if successful
        if self.config.base.enable_cache && !analysis.synsets.is_empty() {
            self.cache.insert(cache_key, analysis.clone());
        }

        Ok(analysis)
    }

    /// Calculate confidence score for an analysis
    fn calculate_confidence(
        &self,
        analysis: &WordNetAnalysis,
        _index_entry: &crate::types::IndexEntry,
    ) -> f32 {
        if analysis.synsets.is_empty() {
            return 0.0;
        }

        // Base confidence from synset count
        let synset_count_factor = (analysis.synsets.len() as f32 * 0.1).min(0.5);

        // Relations factor (more relations indicate richer semantic data)
        let relations_factor = (analysis.relations.len() as f32 * 0.05).min(0.2);

        (0.3 + synset_count_factor + relations_factor).min(1.0)
    }

    /// Get hypernyms for a word
    pub fn get_hypernyms(&self, _synset_id: &str) -> Vec<String> {
        // Simplified implementation - would need proper database lookup
        Vec::new()
    }

    /// Get hyponyms for a word
    pub fn get_hyponyms(&self, _synset_id: &str) -> Vec<String> {
        // Simplified implementation - would need proper database lookup
        Vec::new()
    }

    /// Get synonyms for a word
    pub fn get_synonyms(&self, _word: &str, _pos: PartOfSpeech) -> Vec<String> {
        // Simplified implementation - would need proper database lookup
        Vec::new()
    }

    /// Check if the engine is ready for analysis
    pub fn is_ready(&self) -> bool {
        self.is_loaded && !self.database.synsets.is_empty()
    }
}
