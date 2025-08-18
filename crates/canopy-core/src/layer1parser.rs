//! Layer 1 Parser: Integration bridge between parser and semantics
//!
//! This module provides handlers that integrate the Layer 1 parser with semantic
//! analysis without creating circular dependencies. It acts as a coordination
//! layer that can access both canopy-parser and canopy-semantics.

use crate::{AnalysisResult, CanopyError, Word};
// Layer handler traits (defined here until moved to LSP crate)
pub trait LayerHandler<Input, Output> {
    /// Process input through this layer
    fn process(&self, input: Input) -> AnalysisResult<Output>;
    
    /// Get handler configuration
    fn config(&self) -> &dyn LayerConfig;
    
    /// Get handler health status
    fn health(&self) -> ComponentHealth;
}

/// Configuration interface for layer handlers
pub trait LayerConfig {
    /// Get configuration as key-value pairs
    fn to_map(&self) -> HashMap<String, String>;
    
    /// Validate configuration
    fn validate(&self) -> Result<(), String>;
    
    /// Get layer name
    fn layer_name(&self) -> &str;
}

/// Health status of individual components
#[derive(Debug, Clone)]
pub struct ComponentHealth {
    /// Component name
    pub name: String,
    
    /// Is this component healthy
    pub healthy: bool,
    
    /// Last error (if any)
    pub last_error: Option<String>,
    
    /// Component-specific metrics
    pub metrics: HashMap<String, f64>,
}
use std::collections::HashMap;

/// Configuration for the Layer 1 integration helper
#[derive(Debug, Clone)]
pub struct Layer1HelperConfig {
    /// Enable UDPipe parsing
    pub enable_udpipe: bool,
    
    /// Enable basic semantic features
    pub enable_basic_features: bool,
    
    /// Enable VerbNet integration
    pub enable_verbnet: bool,
    
    /// Maximum sentence length to process
    pub max_sentence_length: usize,
    
    /// Enable debugging output
    pub debug: bool,
    
    /// Confidence threshold for features
    pub confidence_threshold: f64,
}

impl Default for Layer1HelperConfig {
    fn default() -> Self {
        Self {
            enable_udpipe: true,
            enable_basic_features: true,
            enable_verbnet: true,
            max_sentence_length: 100,
            debug: false,
            confidence_threshold: 0.5,
        }
    }
}

impl LayerConfig for Layer1HelperConfig {
    fn to_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("enable_udpipe".to_string(), self.enable_udpipe.to_string());
        map.insert("enable_basic_features".to_string(), self.enable_basic_features.to_string());
        map.insert("enable_verbnet".to_string(), self.enable_verbnet.to_string());
        map.insert("max_sentence_length".to_string(), self.max_sentence_length.to_string());
        map.insert("debug".to_string(), self.debug.to_string());
        map.insert("confidence_threshold".to_string(), self.confidence_threshold.to_string());
        map
    }
    
    fn validate(&self) -> Result<(), String> {
        if self.max_sentence_length == 0 {
            return Err("max_sentence_length must be greater than 0".to_string());
        }
        
        if !(0.0..=1.0).contains(&self.confidence_threshold) {
            return Err("confidence_threshold must be between 0.0 and 1.0".to_string());
        }
        
        Ok(())
    }
    
    fn layer_name(&self) -> &str {
        "layer1_helper"
    }
}

/// Layer 1 parser handler that integrates UDPipe with basic semantic features
/// 
/// This handler bridges the gap between raw parsing and semantic analysis
/// without creating circular dependencies between crates.
pub struct Layer1ParserHandler {
    /// Configuration for this handler
    config: Layer1HelperConfig,
    
    /// Statistics for health monitoring
    stats: HandlerStats,
}

/// Statistics tracking for handler health
#[derive(Debug, Clone, Default)]
pub struct HandlerStats {
    /// Number of requests processed
    pub requests: u64,
    
    /// Number of successful requests
    pub successes: u64,
    
    /// Number of failed requests
    pub failures: u64,
    
    /// Total processing time in microseconds
    pub total_time_us: u64,
    
    /// Last error message
    pub last_error: Option<String>,
    
    /// Average words per request
    pub avg_words_per_request: f64,
}

impl Layer1ParserHandler {
    /// Create new Layer 1 parser handler
    pub fn new() -> Self {
        Self {
            config: Layer1HelperConfig::default(),
            stats: HandlerStats::default(),
        }
    }
    
    /// Create handler with custom configuration
    pub fn with_config(config: Layer1HelperConfig) -> Self {
        Self {
            config,
            stats: HandlerStats::default(),
        }
    }
    
    /// Process text using UDPipe and basic feature extraction
    /// 
    /// This method integrates:
    /// 1. UDPipe tokenization and parsing
    /// 2. Basic semantic feature extraction
    /// 3. Confidence scoring
    fn process_with_udpipe(&self, text: &str) -> AnalysisResult<Vec<Word>> {
        // TODO: Replace with actual UDPipe integration once circular dependency is resolved
        // For now, we'll use a simplified approach that creates Word structures
        
        if self.config.debug {
            eprintln!("Layer1ParserHandler: Processing text: {text}");
        }
        
        // Validate input
        if text.trim().is_empty() {
            return Err(CanopyError::ParseError {
                context: "Empty text input".to_string(),
            });
        }
        
        // Simple tokenization for now (will be replaced with UDPipe)
        let words: Vec<Word> = self.tokenize_and_parse(text)?;
        
        if self.config.debug {
            eprintln!("Layer1ParserHandler: Created {} words", words.len());
        }
        
        Ok(words)
    }
    
    /// Simple tokenization and Word creation
    /// TODO: Replace with actual UDPipe integration
    fn tokenize_and_parse(&self, text: &str) -> AnalysisResult<Vec<Word>> {
        let tokens: Vec<&str> = text.split_whitespace().collect();
        
        if tokens.len() > self.config.max_sentence_length {
            return Err(CanopyError::ParseError {
                context: format!(
                    "Sentence too long: {} words (max: {})",
                    tokens.len(),
                    self.config.max_sentence_length
                ),
            });
        }
        
        let mut words = Vec::new();
        let mut position = 0;
        
        for (i, token) in tokens.iter().enumerate() {
            // Find the actual position of this token in the original text
            if let Some(start_pos) = text[position..].find(token) {
                let actual_start = position + start_pos;
                let actual_end = actual_start + token.len();
                
                let mut word = Word::new(i + 1, token.to_string(), actual_start, actual_end);
                
                // Add basic morphological analysis
                self.analyze_morphology(&mut word);
                
                // Set POS tag based on simple heuristics
                self.assign_pos_tag(&mut word);
                
                words.push(word);
                position = actual_end;
            } else {
                // Fallback to sequential positioning
                let start = position;
                let end = start + token.len();
                
                let mut word = Word::new(i + 1, token.to_string(), start, end);
                self.analyze_morphology(&mut word);
                self.assign_pos_tag(&mut word);
                
                words.push(word);
                position = end + 1; // Account for space
            }
        }
        
        Ok(words)
    }
    
    /// Basic morphological analysis
    fn analyze_morphology(&self, word: &mut Word) {
        use crate::{UDPerson, UDNumber, UDTense};
        
        // Simple heuristic-based morphological analysis
        let text = &word.text.to_lowercase();
        
        // Detect person and number for pronouns
        match text.as_str() {
            "i" => {
                word.feats.person = Some(UDPerson::First);
                word.feats.number = Some(UDNumber::Singular);
            }
            "you" => {
                word.feats.person = Some(UDPerson::Second);
                // Number is ambiguous for "you"
            }
            "he" | "she" | "it" => {
                word.feats.person = Some(UDPerson::Third);
                word.feats.number = Some(UDNumber::Singular);
            }
            "we" => {
                word.feats.person = Some(UDPerson::First);
                word.feats.number = Some(UDNumber::Plural);
            }
            "they" => {
                word.feats.person = Some(UDPerson::Third);
                word.feats.number = Some(UDNumber::Plural);
            }
            _ => {}
        }
        
        // Detect tense for common verbs
        if text.ends_with("ed") {
            word.feats.tense = Some(UDTense::Past);
        } else if text.ends_with("ing") {
            // Could be present participle or gerund
            word.feats.tense = Some(UDTense::Present);
        }
        
        // Detect number for nouns
        if text.ends_with('s') && !["is", "was", "has", "does"].contains(&text.as_str()) {
            word.feats.number = Some(UDNumber::Plural);
        }
    }
    
    /// Assign POS tags using simple heuristics
    fn assign_pos_tag(&self, word: &mut Word) {
        use crate::UPos;
        
        let text = &word.text.to_lowercase();
        
        // Common determiners
        if ["the", "a", "an", "this", "that", "these", "those", "my", "your", "his", "her", "its", "our", "their"]
            .contains(&text.as_str()) {
            word.upos = UPos::Det;
            return;
        }
        
        // Common prepositions
        if ["in", "on", "at", "by", "for", "with", "to", "from", "of", "about", "under", "over"]
            .contains(&text.as_str()) {
            word.upos = UPos::Adp;
            return;
        }
        
        // Common pronouns
        if ["i", "you", "he", "she", "it", "we", "they", "me", "him", "her", "us", "them"]
            .contains(&text.as_str()) {
            word.upos = UPos::Pron;
            return;
        }
        
        // Common auxiliary verbs
        if ["is", "am", "are", "was", "were", "be", "been", "being", "have", "has", "had", "do", "does", "did"]
            .contains(&text.as_str()) {
            word.upos = UPos::Aux;
            return;
        }
        
        // Common conjunctions
        if ["and", "or", "but", "so", "yet"].contains(&text.as_str()) {
            word.upos = UPos::Cconj;
            return;
        }
        
        // Punctuation
        if text.chars().all(|c| c.is_ascii_punctuation()) {
            word.upos = UPos::Punct;
            return;
        }
        
        // Simple verb detection
        if text.ends_with("ed") || text.ends_with("ing") || text.ends_with('s') && text.len() > 3 {
            word.upos = UPos::Verb;
            return;
        }
        
        // Default to noun for unknown words
        word.upos = UPos::Noun;
    }
    
    /// Update handler statistics
    #[allow(dead_code)] // TODO: Use in M3 for performance monitoring
    fn update_stats(&mut self, success: bool, processing_time_us: u64, word_count: usize, error: Option<String>) {
        self.stats.requests += 1;
        self.stats.total_time_us += processing_time_us;
        
        if success {
            self.stats.successes += 1;
            
            // Update running average for words per request
            let total_words = self.stats.avg_words_per_request * (self.stats.successes - 1) as f64 + word_count as f64;
            self.stats.avg_words_per_request = total_words / self.stats.successes as f64;
        } else {
            self.stats.failures += 1;
            self.stats.last_error = error;
        }
    }
}

impl Default for Layer1ParserHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl LayerHandler<String, Vec<Word>> for Layer1ParserHandler {
    fn process(&self, input: String) -> AnalysisResult<Vec<Word>> {
        let start_time = std::time::Instant::now();
        
        let result = self.process_with_udpipe(&input);
        
        let processing_time = start_time.elapsed().as_micros() as u64;
        
        // Note: We can't update stats here because process() takes &self, not &mut self
        // In a real implementation, we'd use interior mutability (e.g., Mutex<HandlerStats>)
        
        match &result {
            Ok(words) => {
                if self.config.debug {
                    eprintln!("Layer1ParserHandler: Success - {} words in {}μs", 
                             words.len(), processing_time);
                }
            }
            Err(e) => {
                if self.config.debug {
                    eprintln!("Layer1ParserHandler: Error - {e:?} in {processing_time}μs");
                }
            }
        }
        
        result
    }
    
    fn config(&self) -> &dyn LayerConfig {
        &self.config
    }
    
    fn health(&self) -> ComponentHealth {
        let success_rate = if self.stats.requests > 0 {
            self.stats.successes as f64 / self.stats.requests as f64
        } else {
            1.0 // No requests yet, assume healthy
        };
        
        let avg_response_time = if self.stats.requests > 0 {
            self.stats.total_time_us as f64 / self.stats.requests as f64
        } else {
            0.0
        };
        
        let healthy = success_rate >= 0.95 && avg_response_time < 10_000.0; // < 10ms average
        
        let mut metrics = HashMap::new();
        metrics.insert("success_rate".to_string(), success_rate);
        metrics.insert("avg_response_time_us".to_string(), avg_response_time);
        metrics.insert("requests".to_string(), self.stats.requests as f64);
        metrics.insert("avg_words_per_request".to_string(), self.stats.avg_words_per_request);
        
        ComponentHealth {
            name: "layer1_parser_handler".to_string(),
            healthy,
            last_error: self.stats.last_error.clone(),
            metrics,
        }
    }
}

/// Semantic analysis handler that enhances words with VerbNet features
/// 
/// This handler can access canopy-semantics without creating circular dependencies
/// since it lives in canopy-core.
pub struct SemanticAnalysisHandler {
    /// Configuration for semantic analysis
    config: SemanticConfig,
    
    /// Handler statistics
    stats: HandlerStats,
}

/// Configuration for semantic analysis
#[derive(Debug, Clone)]
pub struct SemanticConfig {
    /// Enable VerbNet theta role assignment
    pub enable_theta_roles: bool,
    
    /// Enable animacy detection
    pub enable_animacy: bool,
    
    /// Enable definiteness detection
    pub enable_definiteness: bool,
    
    /// Confidence threshold for semantic features
    pub confidence_threshold: f64,
    
    /// Enable debugging output
    pub debug: bool,
}

impl Default for SemanticConfig {
    fn default() -> Self {
        Self {
            enable_theta_roles: true,
            enable_animacy: true,
            enable_definiteness: true,
            confidence_threshold: 0.6,
            debug: false,
        }
    }
}

impl LayerConfig for SemanticConfig {
    fn to_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("enable_theta_roles".to_string(), self.enable_theta_roles.to_string());
        map.insert("enable_animacy".to_string(), self.enable_animacy.to_string());
        map.insert("enable_definiteness".to_string(), self.enable_definiteness.to_string());
        map.insert("confidence_threshold".to_string(), self.confidence_threshold.to_string());
        map.insert("debug".to_string(), self.debug.to_string());
        map
    }
    
    fn validate(&self) -> Result<(), String> {
        if !(0.0..=1.0).contains(&self.confidence_threshold) {
            return Err("confidence_threshold must be between 0.0 and 1.0".to_string());
        }
        Ok(())
    }
    
    fn layer_name(&self) -> &str {
        "semantic_analysis"
    }
}

impl SemanticAnalysisHandler {
    /// Create new semantic analysis handler
    pub fn new() -> Self {
        Self {
            config: SemanticConfig::default(),
            stats: HandlerStats::default(),
        }
    }
    
    /// Create handler with custom configuration
    pub fn with_config(config: SemanticConfig) -> Self {
        Self {
            config,
            stats: HandlerStats::default(),
        }
    }
    
    /// Enhance words with semantic features
    /// TODO: Integrate with actual VerbNet engine when circular dependency is resolved
    fn enhance_with_semantics(&self, words: Vec<Word>) -> AnalysisResult<Vec<Word>> {
        if self.config.debug {
            eprintln!("SemanticAnalysisHandler: Enhancing {} words", words.len());
        }
        
        // For now, just pass through the words
        // TODO: Add actual semantic enhancement:
        // 1. VerbNet theta role assignment
        // 2. Animacy detection
        // 3. Definiteness analysis
        // 4. Confidence scoring
        
        let enhanced_words = words.into_iter().map(|mut word| {
            // Add placeholder semantic enhancements
            self.add_basic_semantic_features(&mut word);
            word
        }).collect();
        
        Ok(enhanced_words)
    }
    
    /// Add basic semantic features to a word
    fn add_basic_semantic_features(&self, word: &mut Word) {
        use crate::{UDAnimacy, UDDefiniteness};
        
        // Basic animacy detection
        if self.config.enable_animacy {
            let text = &word.text.to_lowercase();
            if ["person", "man", "woman", "child", "people", "john", "mary"].contains(&text.as_str()) {
                word.feats.animacy = Some(UDAnimacy::Animate);
            } else if ["table", "chair", "book", "house", "car"].contains(&text.as_str()) {
                word.feats.animacy = Some(UDAnimacy::Inanimate);
            }
        }
        
        // Basic definiteness detection
        if self.config.enable_definiteness
            && word.text.starts_with(char::is_uppercase) {
                // Proper nouns are typically definite
                word.feats.definiteness = Some(UDDefiniteness::Definite);
            }
    }
}

impl Default for SemanticAnalysisHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl LayerHandler<Vec<Word>, Vec<Word>> for SemanticAnalysisHandler {
    fn process(&self, input: Vec<Word>) -> AnalysisResult<Vec<Word>> {
        let start_time = std::time::Instant::now();
        
        let result = self.enhance_with_semantics(input);
        
        let processing_time = start_time.elapsed().as_micros() as u64;
        
        match &result {
            Ok(words) => {
                if self.config.debug {
                    eprintln!("SemanticAnalysisHandler: Enhanced {} words in {}μs", 
                             words.len(), processing_time);
                }
            }
            Err(e) => {
                if self.config.debug {
                    eprintln!("SemanticAnalysisHandler: Error - {e:?} in {processing_time}μs");
                }
            }
        }
        
        result
    }
    
    fn config(&self) -> &dyn LayerConfig {
        &self.config
    }
    
    fn health(&self) -> ComponentHealth {
        let success_rate = if self.stats.requests > 0 {
            self.stats.successes as f64 / self.stats.requests as f64
        } else {
            1.0
        };
        
        let avg_response_time = if self.stats.requests > 0 {
            self.stats.total_time_us as f64 / self.stats.requests as f64
        } else {
            0.0
        };
        
        let healthy = success_rate >= 0.95;
        
        let mut metrics = HashMap::new();
        metrics.insert("success_rate".to_string(), success_rate);
        metrics.insert("avg_response_time_us".to_string(), avg_response_time);
        metrics.insert("requests".to_string(), self.stats.requests as f64);
        
        ComponentHealth {
            name: "semantic_analysis_handler".to_string(),
            healthy,
            last_error: self.stats.last_error.clone(),
            metrics,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_layer1_parser_handler() {
        let handler = Layer1ParserHandler::new();
        
        let result = handler.process("The cat sat on the mat".to_string()).unwrap();
        assert_eq!(result.len(), 6);
        
        // Check that POS tags were assigned
        assert_eq!(result[0].upos, crate::UPos::Det); // "The"
        assert_eq!(result[1].upos, crate::UPos::Noun); // "cat"
        
        // Check health
        let health = handler.health();
        assert!(health.healthy);
    }
    
    #[test]
    fn test_semantic_analysis_handler() {
        let handler = SemanticAnalysisHandler::new();
        
        let mut words = vec![
            Word::new(1, "John".to_string(), 0, 4),
            Word::new(2, "person".to_string(), 5, 11),
        ];
        words[0].upos = crate::UPos::Propn;
        words[1].upos = crate::UPos::Noun;
        
        let result = handler.process(words).unwrap();
        assert_eq!(result.len(), 2);
        
        // Check that semantic features were added
        assert_eq!(result[0].feats.definiteness, Some(crate::UDDefiniteness::Definite));
        assert_eq!(result[1].feats.animacy, Some(crate::UDAnimacy::Animate));
    }
    
    #[test]
    fn test_config_validation() {
        let mut config = Layer1HelperConfig::default();
        assert!(config.validate().is_ok());
        
        config.max_sentence_length = 0;
        assert!(config.validate().is_err());
        
        config.max_sentence_length = 100;
        config.confidence_threshold = -0.1;
        assert!(config.validate().is_err());
        
        config.confidence_threshold = 1.1;
        assert!(config.validate().is_err());
    }
}