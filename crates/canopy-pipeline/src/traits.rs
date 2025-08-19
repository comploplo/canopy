//! Core traits for dependency injection in the Canopy pipeline
//!
//! This module defines the injectable interfaces that allow different
//! implementations to be swapped in for testing, different models,
//! or alternative backends.

use crate::error::{AnalysisError, PipelineError};
use async_trait::async_trait;
use canopy_core::{UPos, Word};
use canopy_semantics::{Event, SemanticAnalysis, ThetaRoleType};
use std::collections::HashMap;

/// Core trait for morphosyntactic parsing (Layer 1)
///
/// This trait abstracts over different parsing backends:
/// - UDPipe 1.2 models
/// - UDPipe 2.15 models
/// - Mock parsers for testing
/// - Future: Stanza, spaCy, custom models
#[async_trait]
pub trait MorphosyntacticParser: Send + Sync {
    /// Parse text into morphologically annotated words
    async fn parse(&self, text: &str) -> Result<Vec<Word>, AnalysisError>;

    /// Get parser information and capabilities
    fn info(&self) -> ParserInfo;

    /// Check if parser is ready (model loaded, etc.)
    fn is_ready(&self) -> bool;

    /// Warm up parser (optional pre-loading)
    async fn warm_up(&mut self) -> Result<(), AnalysisError> {
        Ok(()) // Default: no-op
    }
}

/// Core trait for semantic analysis (Layer 2)
///
/// This trait abstracts over different semantic backends:
/// - VerbNet-based analysis
/// - Pure theory-based derivations
/// - ML-based semantic parsers
/// - Custom semantic analyzers
#[async_trait]
pub trait SemanticAnalyzer: Send + Sync {
    /// Analyze semantically annotated words into events and theta roles
    async fn analyze(&mut self, words: Vec<Word>) -> Result<SemanticAnalysis, AnalysisError>;

    /// Get analyzer capabilities and configuration
    fn info(&self) -> AnalyzerInfo;

    /// Check if analyzer is ready
    fn is_ready(&self) -> bool;

    /// Configure analyzer settings
    fn configure(&mut self, config: AnalyzerConfig) -> Result<(), AnalysisError>;
}

/// Trait for feature extraction services
///
/// This allows pluggable feature extraction:
/// - VerbNet feature extraction
/// - Custom semantic features
/// - ML-based feature detection
/// - Rule-based extractors
#[async_trait]
pub trait FeatureExtractor: Send + Sync {
    /// Extract semantic features from a word
    async fn extract_features(&self, word: &Word) -> Result<FeatureSet, AnalysisError>;

    /// Extract features for multiple words (batch optimization)
    async fn extract_features_batch(
        &self,
        words: &[Word],
    ) -> Result<Vec<FeatureSet>, AnalysisError> {
        let mut results = Vec::new();
        for word in words {
            results.push(self.extract_features(word).await?);
        }
        Ok(results)
    }

    /// Get extractor capabilities
    fn capabilities(&self) -> ExtractorCapabilities;
}

/// Trait for model loading and management
///
/// This abstracts model lifecycle:
/// - Loading from disk
/// - Downloading from remote
/// - Model validation
/// - Version management
#[async_trait]
pub trait ModelLoader: Send + Sync {
    /// Load a model by path or identifier
    async fn load_model(&self, identifier: &str) -> Result<Box<dyn Model>, AnalysisError>;

    /// Check if model is available
    async fn is_model_available(&self, identifier: &str) -> bool;

    /// List available models
    async fn list_models(&self) -> Result<Vec<ModelMetadata>, AnalysisError>;

    /// Download model if not available
    async fn ensure_model(&self, identifier: &str) -> Result<(), AnalysisError>;
}

/// Trait for language models (UDPipe, etc.)
pub trait Model: Send + Sync {
    /// Get model metadata
    fn metadata(&self) -> &ModelMetadata;

    /// Get model capabilities
    fn capabilities(&self) -> ModelCapabilities;

    /// Validate model integrity
    fn validate(&self) -> Result<(), AnalysisError>;
}

/// Trait for caching layer
#[async_trait]
pub trait CacheProvider: Send + Sync {
    /// Get cached analysis result
    async fn get(&self, key: &str) -> Option<CachedResult>;

    /// Store analysis result
    async fn set(&self, key: &str, result: CachedResult) -> Result<(), AnalysisError>;

    /// Clear cache
    async fn clear(&self) -> Result<(), AnalysisError>;

    /// Get cache statistics
    fn stats(&self) -> CacheStats;
}

/// Trait for metrics collection
pub trait MetricsCollector: Send + Sync {
    /// Record operation timing
    fn record_timing(&self, operation: &str, duration_ms: u64);

    /// Record operation count
    fn record_count(&self, operation: &str, count: u64);

    /// Record error
    fn record_error(&self, operation: &str, error: &str);

    /// Get collected metrics
    fn get_metrics(&self) -> Metrics;
}

/// Information about a parser implementation
#[derive(Debug, Clone)]
pub struct ParserInfo {
    pub name: String,
    pub version: String,
    pub model_type: String,
    pub supported_languages: Vec<String>,
    pub capabilities: ParserCapabilities,
}

/// Parser capabilities
#[derive(Debug, Clone)]
pub struct ParserCapabilities {
    pub supports_tokenization: bool,
    pub supports_pos_tagging: bool,
    pub supports_lemmatization: bool,
    pub supports_dependency_parsing: bool,
    pub supports_morphological_features: bool,
    pub max_sentence_length: Option<usize>,
}

/// Information about a semantic analyzer
#[derive(Debug, Clone)]
pub struct AnalyzerInfo {
    pub name: String,
    pub version: String,
    pub approach: String, // "verbnet", "theory-based", "ml", etc.
    pub capabilities: AnalyzerCapabilities,
}

/// Semantic analyzer capabilities
#[derive(Debug, Clone)]
pub struct AnalyzerCapabilities {
    pub supports_theta_roles: bool,
    pub supports_event_structure: bool,
    pub supports_movement_chains: bool,
    pub supports_little_v: bool,
    pub theta_role_inventory: Vec<ThetaRoleType>,
}

/// Configuration for semantic analyzers
#[derive(Debug, Clone, Default)]
pub struct AnalyzerConfig {
    pub enable_theta_assignment: bool,
    pub enable_event_creation: bool,
    pub enable_movement_detection: bool,
    pub performance_mode: PerformanceMode,
    pub custom_settings: HashMap<String, String>,
}

/// Performance mode configuration
#[derive(Debug, Clone, Default)]
pub enum PerformanceMode {
    #[default]
    Balanced,
    Speed,
    Accuracy,
}

/// Set of extracted features
#[derive(Debug, Clone, Default)]
pub struct FeatureSet {
    pub morphological: HashMap<String, String>,
    pub semantic: HashMap<String, String>,
    pub verbnet: Option<VerbNetFeatures>,
    pub custom: HashMap<String, String>,
}

/// VerbNet-specific features
#[derive(Debug, Clone)]
pub struct VerbNetFeatures {
    pub verb_class: Option<String>,
    pub theta_roles: Vec<ThetaRoleType>,
    pub selectional_restrictions: Vec<String>,
}

/// Feature extractor capabilities
#[derive(Debug, Clone)]
pub struct ExtractorCapabilities {
    pub name: String,
    pub supported_features: Vec<String>,
    pub requires_pos_tags: bool,
    pub requires_lemmas: bool,
    pub batch_optimized: bool,
}

/// Model metadata
#[derive(Debug, Clone)]
pub struct ModelMetadata {
    pub identifier: String,
    pub name: String,
    pub version: String,
    pub language: String,
    pub model_type: ModelType,
    pub file_size: Option<u64>,
    pub download_url: Option<String>,
    pub checksum: Option<String>,
}

/// Model type enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum ModelType {
    UDPipe12,
    UDPipe215,
    Custom(String),
}

/// Model capabilities
#[derive(Debug, Clone)]
pub struct ModelCapabilities {
    pub accuracy_metrics: Option<AccuracyMetrics>,
    pub performance_metrics: Option<PerformanceMetrics>,
    pub supported_features: Vec<String>,
}

/// Accuracy metrics for models
#[derive(Debug, Clone)]
pub struct AccuracyMetrics {
    pub pos_accuracy: f64,
    pub lemma_accuracy: f64,
    pub dependency_accuracy: f64,
}

/// Performance metrics for models
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub tokens_per_second: f64,
    pub memory_usage_mb: f64,
    pub model_size_mb: f64,
}

/// Cached analysis result
#[derive(Debug, Clone)]
pub struct CachedResult {
    pub text_hash: String,
    pub analysis: SemanticAnalysis,
    pub timestamp: std::time::SystemTime,
    pub ttl: std::time::Duration,
}

/// Cache statistics
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub size_bytes: u64,
    pub entry_count: u64,
}

/// Collected metrics
#[derive(Debug, Clone, Default)]
pub struct Metrics {
    pub timings: HashMap<String, Vec<u64>>,
    pub counts: HashMap<String, u64>,
    pub errors: HashMap<String, u64>,
}

/// Factory trait for creating pipeline components
pub trait ComponentFactory: Send + Sync {
    /// Create morphosyntactic parser
    fn create_parser(
        &self,
        config: &ParserConfig,
    ) -> Result<Box<dyn MorphosyntacticParser>, PipelineError>;

    /// Create semantic analyzer
    fn create_analyzer(
        &self,
        config: &AnalyzerConfig,
    ) -> Result<Box<dyn SemanticAnalyzer>, PipelineError>;

    /// Create feature extractor
    fn create_extractor(
        &self,
        config: &ExtractorConfig,
    ) -> Result<Box<dyn FeatureExtractor>, PipelineError>;

    /// Create cache provider
    fn create_cache(&self, config: &CacheConfig) -> Result<Box<dyn CacheProvider>, PipelineError>;

    /// Create metrics collector
    fn create_metrics(
        &self,
        config: &MetricsConfig,
    ) -> Result<Box<dyn MetricsCollector>, PipelineError>;
}

/// Parser configuration
#[derive(Debug, Clone)]
pub struct ParserConfig {
    pub model_path: Option<String>,
    pub model_type: ModelType,
    pub performance_mode: PerformanceMode,
    pub enable_caching: bool,
}

/// Extractor configuration
#[derive(Debug, Clone)]
pub struct ExtractorConfig {
    pub extractor_type: String,
    pub enable_verbnet: bool,
    pub custom_rules: Vec<String>,
}

/// Cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub cache_type: String,
    pub max_size_mb: u64,
    pub ttl_seconds: u64,
}

/// Metrics configuration
#[derive(Debug, Clone)]
pub struct MetricsConfig {
    pub enabled: bool,
    pub backend: String,
    pub collection_interval_ms: u64,
}
