//! Canopy Lexicon Engine
//!
//! This module provides the main lexicon engine that implements canopy-engine traits
//! for classification and analysis of closed-class words and functional lexical items.

use crate::parser::LexiconXmlResource;
use crate::types::{LexiconAnalysis, LexiconDatabase, WordClassType};
use canopy_engine::{
    CacheStats, CachedEngine, DataLoader, EngineCache, EngineConfig, EngineError, EngineResult,
    EngineStats, SemanticEngine, SemanticResult, StatisticsProvider, XmlParser, XmlResource,
    traits::DataInfo,
};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

/// Configuration for Lexicon engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LexiconConfig {
    /// Base engine configuration
    pub base: EngineConfig,
    /// Path to lexicon data directory
    pub data_path: String,
    /// Enable pattern matching
    pub enable_patterns: bool,
    /// Maximum number of classifications per word
    pub max_classifications: usize,
    /// Minimum confidence threshold for results
    pub min_confidence: f32,
    /// Enable fuzzy matching
    pub enable_fuzzy_matching: bool,
}

impl Default for LexiconConfig {
    fn default() -> Self {
        Self {
            base: EngineConfig::default(),
            data_path: "data/canopy-lexicon".to_string(),
            enable_patterns: true,
            max_classifications: 10,
            min_confidence: 0.1,
            enable_fuzzy_matching: false,
        }
    }
}

/// Canopy Lexicon Engine
#[derive(Debug)]
pub struct LexiconEngine {
    config: LexiconConfig,
    database: Arc<LexiconDatabase>,
    cache: EngineCache<String, LexiconAnalysis>,
    stats: EngineStats,
    is_loaded: bool,
}

impl LexiconEngine {
    /// Create a new lexicon engine
    pub fn new(config: LexiconConfig) -> Self {
        let cache_capacity = config.base.cache_capacity;

        Self {
            config,
            database: Arc::new(LexiconDatabase::new()),
            cache: EngineCache::new(cache_capacity),
            stats: EngineStats::new("Lexicon".to_string()),
            is_loaded: false,
        }
    }

    /// Load lexicon data from the configured path
    pub fn load_data(&mut self) -> EngineResult<()> {
        let start_time = Instant::now();

        let data_file = Path::new(&self.config.data_path).join("english-lexicon.xml");
        if !data_file.exists() {
            return Err(EngineError::data_load(format!(
                "Lexicon data file not found: {}",
                data_file.display()
            )));
        }

        let parser = XmlParser::new();
        let resource = parser.parse_file::<LexiconXmlResource>(&data_file)?;
        resource.validate()?;

        self.database = Arc::new(resource.database);
        self.is_loaded = true;

        let load_time = start_time.elapsed();
        let stats = self.database.stats();
        tracing::info!(
            "Lexicon database loaded in {:.2}ms with {} word classes, {} words, {} patterns",
            load_time.as_secs_f64() * 1000.0,
            stats.total_word_classes,
            stats.total_words,
            stats.total_patterns
        );

        Ok(())
    }

    /// Analyze a word and return lexical classifications
    pub fn analyze_word(&self, word: &str) -> EngineResult<LexiconAnalysis> {
        if !self.is_loaded {
            return Err(EngineError::data_load(
                "Lexicon database not loaded".to_string(),
            ));
        }

        let cache_key = word.to_lowercase();

        // Check cache first
        if self.config.base.enable_cache {
            if let Some(cached_result) = self.cache.get(&cache_key) {
                return Ok(cached_result);
            }
        }

        let start_time = Instant::now();
        let mut analysis = LexiconAnalysis::new(word.to_string());

        // Get exact word classifications
        analysis.classifications = self.database.classify_word(word);

        // Get pattern matches if enabled
        if self.config.enable_patterns {
            analysis.pattern_matches = self.database.analyze_patterns(word);
        }

        // Filter by confidence threshold
        analysis
            .classifications
            .retain(|c| c.confidence >= self.config.min_confidence);
        analysis
            .pattern_matches
            .retain(|p| p.confidence >= self.config.min_confidence);

        // Limit results
        analysis
            .classifications
            .truncate(self.config.max_classifications);
        analysis
            .pattern_matches
            .truncate(self.config.max_classifications);

        // Calculate overall confidence
        analysis.calculate_confidence();

        let _processing_time = start_time.elapsed();

        // Cache the result
        if self.config.base.enable_cache {
            self.cache.insert(cache_key, analysis.clone());
        }

        Ok(analysis)
    }

    /// Check if a word is a stop word
    pub fn is_stop_word(&self, word: &str) -> EngineResult<bool> {
        let analysis = self.analyze_word(word)?;
        Ok(!analysis.get_stop_words().is_empty())
    }

    /// Check if a word is a negation indicator
    pub fn is_negation(&self, word: &str) -> EngineResult<bool> {
        let analysis = self.analyze_word(word)?;
        Ok(!analysis.get_negations().is_empty())
    }

    /// Check if a word is a discourse marker
    pub fn is_discourse_marker(&self, word: &str) -> EngineResult<bool> {
        let analysis = self.analyze_word(word)?;
        Ok(!analysis.get_discourse_markers().is_empty())
    }

    /// Get all words of a specific class type
    pub fn get_words_by_type(&self, class_type: WordClassType) -> EngineResult<Vec<String>> {
        if !self.is_loaded {
            return Err(EngineError::data_load(
                "Lexicon database not loaded".to_string(),
            ));
        }

        let mut words = Vec::new();
        let classes = self.database.get_classes_by_type(&class_type);

        for word_class in classes {
            for word in &word_class.words {
                words.push(word.word.clone());
                words.extend(word.variants.clone());
            }
        }

        words.sort();
        words.dedup();
        Ok(words)
    }

    /// Analyze multiple words in a text
    pub fn analyze_text(&self, text: &str) -> EngineResult<Vec<LexiconAnalysis>> {
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut results = Vec::new();

        for word in words {
            // Clean word of punctuation
            let clean_word = word.trim_matches(|c: char| c.is_ascii_punctuation());
            if !clean_word.is_empty() {
                let analysis = self.analyze_word(clean_word)?;
                if analysis.has_results() {
                    results.push(analysis);
                }
            }
        }

        Ok(results)
    }

    /// Get semantic weight for a word (useful for stop word filtering)
    pub fn get_semantic_weight(&self, word: &str) -> EngineResult<f32> {
        let analysis = self.analyze_word(word)?;

        if analysis.classifications.is_empty() {
            return Ok(1.0); // Default weight for unknown words
        }

        // Use the weight from the highest priority classification
        let weight = analysis
            .classifications
            .first()
            .map(|c| c.semantic_weight())
            .unwrap_or(1.0);

        Ok(weight)
    }
}

impl SemanticEngine for LexiconEngine {
    type Input = String;
    type Output = LexiconAnalysis;
    type Config = LexiconConfig;

    fn analyze(&self, input: &Self::Input) -> EngineResult<SemanticResult<Self::Output>> {
        let start_time = Instant::now();
        let analysis = self.analyze_word(input)?;
        let processing_time = start_time.elapsed();

        let confidence = if analysis.has_results() {
            analysis.confidence
        } else {
            0.0
        };

        Ok(SemanticResult::new(
            analysis,
            confidence,
            false,
            processing_time.as_micros() as u64,
        ))
    }

    fn name(&self) -> &'static str {
        "Lexicon"
    }

    fn version(&self) -> &'static str {
        "1.0"
    }

    fn is_initialized(&self) -> bool {
        self.is_loaded
    }

    fn config(&self) -> &Self::Config {
        &self.config
    }
}

impl CachedEngine for LexiconEngine {
    fn clear_cache(&self) {
        self.cache.clear();
    }

    fn cache_stats(&self) -> CacheStats {
        self.cache.stats()
    }

    fn set_cache_capacity(&mut self, capacity: usize) {
        self.config.base.cache_capacity = capacity;
        // Note: This would require rebuilding the cache in a full implementation
    }
}

impl StatisticsProvider for LexiconEngine {
    fn statistics(&self) -> EngineStats {
        self.stats.clone()
    }

    fn performance_metrics(&self) -> canopy_engine::PerformanceMetrics {
        self.stats.performance.clone()
    }
}

impl DataLoader for LexiconEngine {
    fn load_from_directory<P: AsRef<Path>>(&mut self, path: P) -> EngineResult<()> {
        self.config.data_path = path.as_ref().to_string_lossy().to_string();
        self.load_data()
    }

    fn load_test_data(&mut self) -> EngineResult<()> {
        // For testing, we could create a minimal lexicon subset
        Err(EngineError::data_load(
            "Test data loading not implemented".to_string(),
        ))
    }

    fn reload(&mut self) -> EngineResult<()> {
        self.is_loaded = false;
        self.load_data()
    }

    fn data_info(&self) -> DataInfo {
        if self.is_loaded {
            let stats = self.database.stats();
            DataInfo::new(
                format!("{}/english-lexicon.xml", self.config.data_path),
                stats.total_words,
            )
        } else {
            DataInfo::new("Not loaded".to_string(), 0)
        }
    }
}

/// Specialized analysis methods
impl LexiconEngine {
    /// Analyze negation scope in a sentence
    pub fn analyze_negation_scope(&self, text: &str) -> EngineResult<Vec<(String, usize, usize)>> {
        let mut negations = Vec::new();
        let words: Vec<&str> = text.split_whitespace().collect();

        for word in words.iter() {
            let clean_word = word.trim_matches(|c: char| c.is_ascii_punctuation());
            if self.is_negation(clean_word)? {
                // Calculate byte positions
                let start_byte = text.find(word).unwrap_or(0);
                let end_byte = start_byte + word.len();
                negations.push((clean_word.to_string(), start_byte, end_byte));
            }
        }

        Ok(negations)
    }

    /// Extract discourse structure from text
    pub fn extract_discourse_structure(&self, text: &str) -> EngineResult<Vec<(String, String)>> {
        let mut discourse_markers = Vec::new();
        let words: Vec<&str> = text.split_whitespace().collect();

        for word in words {
            let clean_word = word.trim_matches(|c: char| c.is_ascii_punctuation());
            let analysis = self.analyze_word(clean_word)?;

            for marker in analysis.get_discourse_markers() {
                if let Some(context) = &marker.context {
                    discourse_markers.push((clean_word.to_string(), context.clone()));
                }
            }
        }

        Ok(discourse_markers)
    }

    /// Filter stop words from a list
    pub fn filter_stop_words(&self, words: &[String]) -> EngineResult<Vec<String>> {
        let mut filtered = Vec::new();

        for word in words {
            if !self.is_stop_word(word)? {
                filtered.push(word.clone());
            }
        }

        Ok(filtered)
    }

    /// Get intensifier strength for a word
    pub fn get_intensifier_strength(&self, word: &str) -> EngineResult<Option<String>> {
        let analysis = self.analyze_word(word)?;

        for classification in &analysis.classifications {
            if matches!(classification.word_class_type, WordClassType::Intensifiers) {
                return Ok(classification.context.clone());
            }
        }

        Ok(None)
    }
}

impl Default for LexiconEngine {
    fn default() -> Self {
        Self::new(LexiconConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_lexicon() -> (TempDir, LexiconConfig) {
        let temp_dir = TempDir::new().unwrap();
        let lexicon_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<lexicon version="1.0" language="en" xmlns="http://canopy.rs/lexicon">
  <metadata>
    <title>Test Lexicon</title>
    <description>Test lexicon for unit tests</description>
    <created>2024-01-01</created>
    <author>Test</author>
    <license>MIT</license>
  </metadata>
  
  <word-classes>
    <word-class id="test-stop-words" name="Test Stop Words" type="stop-words" priority="10">
      <description>Test stop words</description>
      <properties>
        <property name="semantic-weight" value="0.1" type="float"/>
      </properties>
      <words>
        <word pos="DT">the</word>
        <word pos="DT">a</word>
        <word pos="CC">and</word>
      </words>
    </word-class>
    
    <word-class id="test-negation" name="Test Negation" type="negation" priority="9">
      <description>Test negation words</description>
      <words>
        <word pos="RB">not</word>
        <word pos="DT">no</word>
      </words>
      <patterns>
        <pattern id="neg-prefix-un" type="prefix" confidence="0.8">
          <regex>^un[a-z]+</regex>
          <description>Un- prefix</description>
          <examples>
            <example>unhappy</example>
          </examples>
        </pattern>
      </patterns>
    </word-class>
  </word-classes>
</lexicon>"#;

        fs::write(temp_dir.path().join("english-lexicon.xml"), lexicon_xml).unwrap();

        let config = LexiconConfig {
            data_path: temp_dir.path().to_string_lossy().to_string(),
            ..LexiconConfig::default()
        };

        (temp_dir, config)
    }

    #[test]
    fn test_lexicon_loading() {
        let (_temp_dir, config) = create_test_lexicon();
        let mut engine = LexiconEngine::new(config);

        engine.load_data().expect("Failed to load test lexicon");
        assert!(engine.is_initialized());

        let stats = engine.database.stats();
        assert_eq!(stats.total_word_classes, 2);
        assert_eq!(stats.total_words, 5);
        assert_eq!(stats.total_patterns, 1);
    }

    #[test]
    fn test_word_classification() {
        let (_temp_dir, config) = create_test_lexicon();
        let mut engine = LexiconEngine::new(config);
        engine.load_data().unwrap();

        // Test stop word
        assert!(engine.is_stop_word("the").unwrap());
        assert!(engine.is_stop_word("and").unwrap());
        assert!(!engine.is_stop_word("happy").unwrap());

        // Test negation
        assert!(engine.is_negation("not").unwrap());
        assert!(engine.is_negation("no").unwrap());
        assert!(!engine.is_negation("yes").unwrap());
    }

    #[test]
    fn test_pattern_matching() {
        let (_temp_dir, config) = create_test_lexicon();
        let mut engine = LexiconEngine::new(config);
        engine.load_data().unwrap();

        let analysis = engine.analyze_word("unhappy").unwrap();
        assert!(!analysis.pattern_matches.is_empty());

        let pattern_match = &analysis.pattern_matches[0];
        assert_eq!(pattern_match.pattern_id, "neg-prefix-un");
        assert_eq!(pattern_match.matched_text, "unhappy");
    }

    #[test]
    fn test_semantic_engine_trait() {
        let (_temp_dir, config) = create_test_lexicon();
        let mut engine = LexiconEngine::new(config);
        engine.load_data().unwrap();

        let result = engine.analyze(&"the".to_string()).unwrap();
        assert!(result.data.has_results());
        assert!(result.confidence > 0.0);

        assert_eq!(engine.name(), "Lexicon");
        assert_eq!(engine.version(), "1.0");
    }
}
