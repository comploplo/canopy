//! Canopy Server: Dependency injection architecture for linguistic analysis
//!
//! This module implements a dependency injection pattern that separates concerns
//! between parsing, semantic analysis, and handling. The server coordinates
//! between different layer handlers without creating circular dependencies.

use canopy_core::layer1parser::{ComponentHealth, LayerHandler};
use canopy_core::{AnalysisResult, CanopyError, Document, Sentence, Word};
use std::collections::HashMap;

/// Core server trait for linguistic analysis
pub trait CanopyServer {
    /// Process text through the complete linguistic analysis pipeline
    fn process_text(&self, text: &str) -> AnalysisResult<AnalysisResponse>;

    /// Get server health and statistics
    fn health(&self) -> ServerHealth;
}

/// Response from the linguistic analysis pipeline
#[derive(Debug, Clone)]
pub struct AnalysisResponse {
    /// Processed document with all layers applied
    pub document: Document,

    /// Layer-specific analysis results
    pub layer_results: HashMap<String, LayerResult>,

    /// Performance metrics
    pub metrics: AnalysisMetrics,
}

/// Result from a specific analysis layer
#[derive(Debug, Clone)]
pub struct LayerResult {
    /// Layer name (e.g., "layer1", "semantics", "events")
    pub layer: String,

    /// Processing time in microseconds
    pub processing_time_us: u64,

    /// Number of items processed (words, events, etc.)
    pub items_processed: usize,

    /// Confidence scores for this layer's analysis
    pub confidence: f64,

    /// Layer-specific metadata
    pub metadata: HashMap<String, String>,
}

/// Performance metrics for analysis
#[derive(Debug, Clone)]
pub struct AnalysisMetrics {
    /// Total processing time in microseconds
    pub total_time_us: u64,

    /// Time breakdown by layer
    pub layer_times: HashMap<String, u64>,

    /// Memory usage statistics
    pub memory_stats: MemoryStats,

    /// Input characteristics
    pub input_stats: InputStats,
}

/// Memory usage statistics
#[derive(Debug, Clone)]
pub struct MemoryStats {
    /// Peak memory usage in bytes
    pub peak_bytes: usize,

    /// Final memory usage in bytes
    pub final_bytes: usize,

    /// Number of allocations
    pub allocations: usize,
}

/// Input text statistics
#[derive(Debug, Clone)]
pub struct InputStats {
    /// Character count
    pub char_count: usize,

    /// Word count (estimated)
    pub word_count: usize,

    /// Sentence count (estimated)
    pub sentence_count: usize,
}

/// Server health and status information
#[derive(Debug, Clone)]
pub struct ServerHealth {
    /// Is the server operational
    pub healthy: bool,

    /// Component status (parser, semantics, etc.)
    pub components: HashMap<String, ComponentHealth>,

    /// Server uptime in seconds
    pub uptime_seconds: u64,

    /// Number of requests processed
    pub requests_processed: u64,

    /// Average response time in microseconds
    pub avg_response_time_us: u64,
}

// Use ComponentHealth from canopy_core::layer1parser

// Use LayerHandler and LayerConfig from canopy_core::layer1parser

/// Default server implementation using dependency injection
pub struct DefaultCanopyServer<P, S>
where
    P: LayerHandler<String, Vec<Word>>,
    S: LayerHandler<Vec<Word>, Vec<Word>>, // Simplified for now
{
    /// Layer 1 parser handler (UDPipe + basic features)
    parser: P,

    /// Semantic analysis handler (VerbNet + features)
    semantics: S,

    /// Server configuration
    #[allow(dead_code)] // TODO: Use config in M3 for LSP server configuration
    config: ServerConfig,

    /// Request statistics (using RwLock for thread-safe tracking)
    stats: std::sync::RwLock<ServerStats>,
}

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Enable performance tracking
    pub enable_metrics: bool,

    /// Maximum processing time in milliseconds
    pub timeout_ms: u64,

    /// Enable debugging output
    pub debug: bool,

    /// Layer-specific configurations
    pub layer_configs: HashMap<String, HashMap<String, String>>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            enable_metrics: true,
            timeout_ms: 5000, // 5 second timeout
            debug: false,
            layer_configs: HashMap::new(),
        }
    }
}

/// Server statistics tracking
#[derive(Debug, Clone)]
pub struct ServerStats {
    /// Number of requests processed
    pub requests: u64,

    /// Total processing time
    pub total_time_us: u64,

    /// Number of errors
    pub errors: u64,

    /// Start time (for uptime calculation)
    pub start_time: std::time::Instant,
}

impl Default for ServerStats {
    fn default() -> Self {
        Self {
            requests: 0,
            total_time_us: 0,
            errors: 0,
            start_time: std::time::Instant::now(),
        }
    }
}

impl<P, S> DefaultCanopyServer<P, S>
where
    P: LayerHandler<String, Vec<Word>>,
    S: LayerHandler<Vec<Word>, Vec<Word>>,
{
    /// Create new server with injected handlers
    pub fn new(parser: P, semantics: S) -> Self {
        Self {
            parser,
            semantics,
            config: ServerConfig::default(),
            stats: std::sync::RwLock::new(ServerStats {
                start_time: std::time::Instant::now(),
                ..Default::default()
            }),
        }
    }

    /// Create server with custom configuration
    pub fn with_config(parser: P, semantics: S, config: ServerConfig) -> Self {
        Self {
            parser,
            semantics,
            config,
            stats: std::sync::RwLock::new(ServerStats {
                start_time: std::time::Instant::now(),
                ..Default::default()
            }),
        }
    }

    /// Process text through the analysis pipeline
    fn process_pipeline(&self, text: &str) -> AnalysisResult<AnalysisResponse> {
        let start_time = std::time::Instant::now();

        // Layer 1: Parse text into words with basic features
        let layer1_start = std::time::Instant::now();
        let words = self.parser.process(text.to_string())?;
        let layer1_time = layer1_start.elapsed().as_micros() as u64;

        // Layer 2: Add semantic features
        let layer2_start = std::time::Instant::now();
        let enhanced_words = self.semantics.process(words.clone())?;
        let layer2_time = layer2_start.elapsed().as_micros() as u64;

        let total_time = start_time.elapsed().as_micros() as u64;

        // Create document from enhanced words
        let word_count = enhanced_words.len();
        let sentence = Sentence::new(enhanced_words);
        let document = Document::new(text.to_string(), vec![sentence]);

        // Build response with metrics
        let mut layer_results = HashMap::new();

        layer_results.insert(
            "layer1".to_string(),
            LayerResult {
                layer: "layer1".to_string(),
                processing_time_us: layer1_time,
                items_processed: words.len(),
                confidence: 0.85, // TODO: Calculate from parser confidence
                metadata: HashMap::new(),
            },
        );

        layer_results.insert(
            "semantics".to_string(),
            LayerResult {
                layer: "semantics".to_string(),
                processing_time_us: layer2_time,
                items_processed: word_count,
                confidence: 0.75, // TODO: Calculate from semantic confidence
                metadata: HashMap::new(),
            },
        );

        let mut layer_times = HashMap::new();
        layer_times.insert("layer1".to_string(), layer1_time);
        layer_times.insert("semantics".to_string(), layer2_time);

        let metrics = AnalysisMetrics {
            total_time_us: total_time,
            layer_times,
            memory_stats: MemoryStats {
                peak_bytes: text.len() * 8 + words.len() * 64, // Estimate based on text + word structures
                final_bytes: text.len() * 4 + words.len() * 32, // Estimate after processing
                allocations: words.len() + 5,                  // Approximate allocations
            },
            input_stats: InputStats {
                char_count: text.len(),
                word_count: words.len(),
                sentence_count: 1, // Simplified
            },
        };

        Ok(AnalysisResponse {
            document,
            layer_results,
            metrics,
        })
    }
}

impl<P, S> CanopyServer for DefaultCanopyServer<P, S>
where
    P: LayerHandler<String, Vec<Word>>,
    S: LayerHandler<Vec<Word>, Vec<Word>>,
{
    fn process_text(&self, text: &str) -> AnalysisResult<AnalysisResponse> {
        if text.trim().is_empty() {
            return Err(CanopyError::ParseError {
                context: "Empty input text".to_string(),
            });
        }

        let start_time = std::time::Instant::now();

        // Process the request
        let result = self.process_pipeline(text);

        // Update statistics
        let processing_time = start_time.elapsed().as_micros() as u64;
        if let Ok(mut stats) = self.stats.write() {
            stats.requests += 1;
            stats.total_time_us += processing_time;
            if result.is_err() {
                stats.errors += 1;
            }
        }

        result
    }

    fn health(&self) -> ServerHealth {
        let mut components = HashMap::new();

        // Check layer1 parser health (using correct component name)
        components.insert("layer1".to_string(), self.parser.health());

        // Check semantics health
        components.insert("semantics".to_string(), self.semantics.health());

        // Overall health is healthy if all components are healthy
        let healthy = components.values().all(|c| c.healthy);

        let (uptime, requests_processed, avg_response_time) = if let Ok(stats) = self.stats.read() {
            let uptime = stats.start_time.elapsed().as_secs();
            let avg_response_time = if stats.requests > 0 {
                stats.total_time_us / stats.requests
            } else {
                0
            };
            (uptime, stats.requests, avg_response_time)
        } else {
            (0, 0, 0)
        };

        ServerHealth {
            healthy,
            components,
            uptime_seconds: uptime,
            requests_processed,
            avg_response_time_us: avg_response_time,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use canopy_core::layer1parser::LayerConfig;

    // Mock parser handler for testing
    struct MockParser {
        config: MockConfig,
    }

    struct MockSemantics {
        config: MockConfig,
    }

    struct MockConfig {
        layer_name: String,
    }

    impl LayerConfig for MockConfig {
        fn to_map(&self) -> HashMap<String, String> {
            let mut map = HashMap::new();
            map.insert("layer".to_string(), self.layer_name.clone());
            map
        }

        fn validate(&self) -> Result<(), String> {
            Ok(())
        }

        fn layer_name(&self) -> &'static str {
            "mock_config"
        }
    }

    impl LayerHandler<String, Vec<Word>> for MockParser {
        fn process(&self, input: String) -> AnalysisResult<Vec<Word>> {
            // Simple mock: split on whitespace and create words
            let words: Vec<Word> = input
                .split_whitespace()
                .enumerate()
                .map(|(i, word)| {
                    let start = i * (word.len() + 1); // Approximate positions
                    let end = start + word.len();
                    Word::new(i + 1, word.to_string(), start, end)
                })
                .collect();

            Ok(words)
        }

        fn config(&self) -> &dyn LayerConfig {
            &self.config
        }

        fn health(&self) -> ComponentHealth {
            ComponentHealth {
                name: "mock_parser".to_string(),
                healthy: true,
                last_error: None,
                metrics: HashMap::new(),
            }
        }
    }

    impl LayerHandler<Vec<Word>, Vec<Word>> for MockSemantics {
        fn process(&self, input: Vec<Word>) -> AnalysisResult<Vec<Word>> {
            // Pass through for now
            Ok(input)
        }

        fn config(&self) -> &dyn LayerConfig {
            &self.config
        }

        fn health(&self) -> ComponentHealth {
            ComponentHealth {
                name: "mock_semantics".to_string(),
                healthy: true,
                last_error: None,
                metrics: HashMap::new(),
            }
        }
    }

    #[test]
    fn test_server_dependency_injection() {
        let parser = MockParser {
            config: MockConfig {
                layer_name: "parser".to_string(),
            },
        };

        let semantics = MockSemantics {
            config: MockConfig {
                layer_name: "semantics".to_string(),
            },
        };

        let server = DefaultCanopyServer::new(parser, semantics);

        // Test health check
        let health = server.health();
        assert!(health.healthy);
        assert_eq!(health.components.len(), 2);

        // Test text processing
        let response = server.process_text("The cat sat on the mat").unwrap();
        assert_eq!(response.document.sentences.len(), 1);
        assert_eq!(response.document.sentences[0].words.len(), 6);

        // Check that we have results from both layers
        assert!(response.layer_results.contains_key("layer1"));
        assert!(response.layer_results.contains_key("semantics"));

        // Verify metrics
        assert!(response.metrics.total_time_us > 0);
        assert_eq!(response.metrics.input_stats.word_count, 6);
    }

    #[test]
    fn test_empty_input_handling() {
        let parser = MockParser {
            config: MockConfig {
                layer_name: "parser".to_string(),
            },
        };

        let semantics = MockSemantics {
            config: MockConfig {
                layer_name: "semantics".to_string(),
            },
        };

        let server = DefaultCanopyServer::new(parser, semantics);

        let result = server.process_text("");
        assert!(result.is_err());

        if let Err(CanopyError::ParseError { context }) = result {
            assert_eq!(context, "Empty input text");
        } else {
            panic!("Expected ParseError");
        }
    }
}
