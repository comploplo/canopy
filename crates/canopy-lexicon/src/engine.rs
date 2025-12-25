//! Canopy Lexicon Engine
//!
//! This module provides the main lexicon engine that implements canopy-engine traits
//! for classification and analysis of closed-class words and functional lexical items.

use crate::parser::LexiconXmlResource;
use crate::types::{LexiconAnalysis, LexiconDatabase, WordClassType};
use canopy_core::paths::data_path_string;
use canopy_engine::{
    BaseEngine, CacheKeyFormat, EngineConfig, EngineCore, EngineResult, EngineStats,
    PerformanceMetrics, SemanticResult, XmlParser, XmlResource,
    traits::{CachedEngine, DataInfo, DataLoader, SemanticEngine, StatisticsProvider},
};
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::sync::Arc;
use tracing::{debug, info};

/// Input type for Lexicon analysis
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexiconInput {
    pub word: String,
}

impl Hash for LexiconInput {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.word.hash(state);
    }
}

/// Configuration for Lexicon engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LexiconConfig {
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
    /// Enable caching
    pub enable_cache: bool,
    /// Cache capacity
    pub cache_capacity: usize,
}

impl Default for LexiconConfig {
    fn default() -> Self {
        Self {
            data_path: data_path_string("data/canopy-lexicon"),
            enable_patterns: true,
            max_classifications: 10,
            min_confidence: 0.1,
            enable_fuzzy_matching: false,
            enable_cache: true,
            cache_capacity: 10000,
        }
    }
}

/// Canopy Lexicon Engine
#[derive(Debug)]
pub struct LexiconEngine {
    /// Base engine handling cache, stats, and metrics
    base_engine: BaseEngine<LexiconInput, LexiconAnalysis>,
    /// Lexicon database
    database: Arc<LexiconDatabase>,
    /// Lexicon-specific configuration
    lexicon_config: LexiconConfig,
    /// Is data loaded flag
    is_loaded: bool,
}

impl LexiconEngine {
    /// Create a new lexicon engine
    pub fn new() -> Self {
        Self::with_config(LexiconConfig::default())
    }

    /// Create a new lexicon engine with custom configuration
    pub fn with_config(lexicon_config: LexiconConfig) -> Self {
        // Convert LexiconConfig to EngineConfig
        let engine_config = EngineConfig {
            enable_cache: lexicon_config.enable_cache,
            cache_capacity: lexicon_config.cache_capacity,
            enable_metrics: true,
            enable_parallel: false,
            max_threads: 4,
            confidence_threshold: lexicon_config.min_confidence,
        };

        Self {
            base_engine: BaseEngine::new(engine_config, "Lexicon".to_string()),
            database: Arc::new(LexiconDatabase::new()),
            lexicon_config,
            is_loaded: false,
        }
    }

    /// Analyze a word and return lexical classifications
    pub fn analyze_word(&self, word: &str) -> EngineResult<SemanticResult<LexiconAnalysis>> {
        let input = LexiconInput {
            word: word.to_string(),
        };
        self.base_engine.analyze(&input, self)
    }

    /// Load lexicon data from the configured path
    pub fn load_data(&mut self) -> EngineResult<()> {
        let data_file = Path::new(&self.lexicon_config.data_path).join("english-lexicon.xml");
        if !data_file.exists() {
            return Err(canopy_engine::EngineError::data_load(format!(
                "Lexicon data file not found: {}",
                data_file.display()
            )));
        }

        let parser = XmlParser::new();
        let resource = parser.parse_file::<LexiconXmlResource>(&data_file)?;
        resource.validate()?;

        self.database = Arc::new(resource.database);
        self.is_loaded = true;

        let stats = self.database.stats();
        info!(
            "Lexicon database loaded with {} word classes, {} words, {} patterns",
            stats.total_word_classes, stats.total_words, stats.total_patterns
        );

        Ok(())
    }

    /// Check if a word is a stop word
    pub fn is_stop_word(&self, word: &str) -> EngineResult<bool> {
        let analysis = self.analyze_word(word)?;
        Ok(!analysis.data.get_stop_words().is_empty())
    }

    /// Check if a word is a negation indicator
    pub fn is_negation(&self, word: &str) -> EngineResult<bool> {
        let analysis = self.analyze_word(word)?;
        Ok(!analysis.data.get_negations().is_empty())
    }

    /// Check if a word is a discourse marker
    pub fn is_discourse_marker(&self, word: &str) -> EngineResult<bool> {
        let analysis = self.analyze_word(word)?;
        Ok(!analysis.data.get_discourse_markers().is_empty())
    }

    /// Get all words of a specific class type
    pub fn get_words_by_type(&self, class_type: WordClassType) -> EngineResult<Vec<String>> {
        if !self.is_loaded {
            return Err(canopy_engine::EngineError::data_load(
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
                if analysis.data.has_results() {
                    results.push(analysis.data);
                }
            }
        }

        Ok(results)
    }

    /// Get semantic weight for a word (useful for stop word filtering)
    pub fn get_semantic_weight(&self, word: &str) -> EngineResult<f32> {
        let analysis = self.analyze_word(word)?;

        if analysis.data.classifications.is_empty() {
            return Ok(1.0); // Default weight for unknown words
        }

        // Use the weight from the highest priority classification
        let weight = analysis
            .data
            .classifications
            .first()
            .map(|c| c.semantic_weight())
            .unwrap_or(1.0);

        Ok(weight)
    }

    // Backward compatibility methods for BaseEngine integration
    pub fn config(&self) -> &LexiconConfig {
        &self.lexicon_config
    }

    pub fn performance_metrics(&self) -> PerformanceMetrics {
        self.base_engine.get_performance_metrics()
    }

    pub fn cache_stats(&self) -> canopy_engine::CacheStats {
        self.base_engine.cache_stats()
    }

    pub fn clear_cache(&mut self) -> EngineResult<()> {
        self.base_engine.clear_cache();
        Ok(())
    }
}

// EngineCore trait implementation for BaseEngine integration
impl EngineCore<LexiconInput, LexiconAnalysis> for LexiconEngine {
    fn perform_analysis(&self, input: &LexiconInput) -> EngineResult<LexiconAnalysis> {
        if !self.is_loaded {
            debug!("Lexicon database not loaded for analysis");
            return Ok(LexiconAnalysis::new(input.word.clone()));
        }

        let mut analysis = LexiconAnalysis::new(input.word.clone());

        // Get exact word classifications
        analysis.classifications = self.database.classify_word(&input.word);

        // Get pattern matches if enabled
        if self.lexicon_config.enable_patterns {
            analysis.pattern_matches = self.database.analyze_patterns(&input.word);
        }

        // Filter by confidence threshold
        analysis
            .classifications
            .retain(|c| c.confidence >= self.lexicon_config.min_confidence);
        analysis
            .pattern_matches
            .retain(|p| p.confidence >= self.lexicon_config.min_confidence);

        // Limit results
        analysis
            .classifications
            .truncate(self.lexicon_config.max_classifications);
        analysis
            .pattern_matches
            .truncate(self.lexicon_config.max_classifications);

        // Calculate overall confidence
        analysis.calculate_confidence();

        debug!(
            "Lexicon analysis for '{}': {} classifications, {} patterns, confidence: {:.2}",
            input.word,
            analysis.classifications.len(),
            analysis.pattern_matches.len(),
            analysis.confidence
        );

        Ok(analysis)
    }

    fn calculate_confidence(&self, _input: &LexiconInput, output: &LexiconAnalysis) -> f32 {
        output.confidence
    }

    fn generate_cache_key(&self, input: &LexiconInput) -> String {
        CacheKeyFormat::Typed("lexicon".to_string(), input.word.to_lowercase()).to_string()
    }

    fn engine_name(&self) -> &'static str {
        "Lexicon"
    }

    fn engine_version(&self) -> &'static str {
        "1.0"
    }

    fn is_initialized(&self) -> bool {
        self.is_loaded
    }
}

impl SemanticEngine for LexiconEngine {
    type Input = String;
    type Output = LexiconAnalysis;
    type Config = LexiconConfig;

    fn analyze(&self, input: &Self::Input) -> EngineResult<SemanticResult<Self::Output>> {
        let lexicon_input = LexiconInput {
            word: input.clone(),
        };
        self.base_engine.analyze(&lexicon_input, self)
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
        &self.lexicon_config
    }
}

impl CachedEngine for LexiconEngine {
    fn cache_stats(&self) -> canopy_engine::CacheStats {
        self.base_engine.cache_stats()
    }

    fn clear_cache(&self) {
        self.base_engine.clear_cache();
    }

    fn set_cache_capacity(&mut self, capacity: usize) {
        self.lexicon_config.cache_capacity = capacity;
    }
}

impl StatisticsProvider for LexiconEngine {
    fn statistics(&self) -> EngineStats {
        self.base_engine.get_stats()
    }

    fn performance_metrics(&self) -> PerformanceMetrics {
        self.base_engine.get_performance_metrics()
    }
}

impl DataLoader for LexiconEngine {
    fn load_from_directory<P: AsRef<Path>>(&mut self, path: P) -> EngineResult<()> {
        let path = path.as_ref();
        info!("Loading Lexicon data from: {}", path.display());

        self.lexicon_config.data_path = path.to_string_lossy().to_string();
        self.load_data()
    }

    fn load_test_data(&mut self) -> EngineResult<()> {
        // Create minimal test data
        self.database = Arc::new(LexiconDatabase::new());
        self.is_loaded = true;
        Ok(())
    }

    fn reload(&mut self) -> EngineResult<()> {
        self.is_loaded = false;
        self.database = Arc::new(LexiconDatabase::new());
        self.load_data()
    }

    fn data_info(&self) -> DataInfo {
        if self.is_loaded {
            let stats = self.database.stats();
            DataInfo::new(
                format!(
                    "lexicon: {}/english-lexicon.xml",
                    self.lexicon_config.data_path
                ),
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

            for marker in analysis.data.get_discourse_markers() {
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

        for classification in &analysis.data.classifications {
            if matches!(classification.word_class_type, WordClassType::Intensifiers) {
                return Ok(classification.context.clone());
            }
        }

        Ok(None)
    }
}

impl Default for LexiconEngine {
    fn default() -> Self {
        Self::new()
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
        let mut engine = LexiconEngine::with_config(config);

        engine.load_data().expect("Failed to load test lexicon");
        assert!(SemanticEngine::is_initialized(&engine));

        let stats = engine.database.stats();
        assert_eq!(stats.total_word_classes, 2);
        assert_eq!(stats.total_words, 5);
        assert_eq!(stats.total_patterns, 1);
    }

    #[test]
    fn test_word_classification() {
        let (_temp_dir, config) = create_test_lexicon();
        let mut engine = LexiconEngine::with_config(config);
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
        let mut engine = LexiconEngine::with_config(config);
        engine.load_data().unwrap();

        let analysis = engine.analyze_word("unhappy").unwrap();
        assert!(!analysis.data.pattern_matches.is_empty());

        let pattern_match = &analysis.data.pattern_matches[0];
        assert_eq!(pattern_match.pattern_id, "neg-prefix-un");
        assert_eq!(pattern_match.matched_text, "unhappy");
    }

    #[test]
    fn test_semantic_engine_trait() {
        let (_temp_dir, config) = create_test_lexicon();
        let mut engine = LexiconEngine::with_config(config);
        engine.load_data().unwrap();

        let result = engine.analyze(&"the".to_string()).unwrap();
        assert!(result.data.has_results());
        assert!(result.confidence > 0.0);

        assert_eq!(engine.name(), "Lexicon");
        assert_eq!(engine.version(), "1.0");
    }
}
