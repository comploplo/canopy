//! Core traits for dependency injection in the Canopy pipeline
//!
//! This module defines the injectable interfaces that allow different
//! implementations to be swapped in for testing, different models,
//! or alternative backends.

use crate::error::{AnalysisError, PipelineError};
use async_trait::async_trait;
use canopy_core::ThetaRole as ThetaRoleType;
use canopy_core::Word;
use canopy_tokenizer::SemanticLayer1Output as SemanticAnalysis;
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
#[derive(Debug, Clone, Default, PartialEq)]
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

#[cfg(test)]
mod tests {
    use super::*;

    // ======== ParserInfo Tests ========

    #[test]
    fn test_parser_info_construction() {
        let info = ParserInfo {
            name: "TestParser".to_string(),
            version: "1.0".to_string(),
            model_type: "udpipe".to_string(),
            supported_languages: vec!["en".to_string(), "es".to_string()],
            capabilities: ParserCapabilities {
                supports_tokenization: true,
                supports_pos_tagging: true,
                supports_lemmatization: true,
                supports_dependency_parsing: true,
                supports_morphological_features: true,
                max_sentence_length: Some(500),
            },
        };
        assert_eq!(info.name, "TestParser");
        assert_eq!(info.supported_languages.len(), 2);
        assert!(info.capabilities.supports_tokenization);
    }

    #[test]
    fn test_parser_info_clone_debug() {
        let info = ParserInfo {
            name: "Test".to_string(),
            version: "1.0".to_string(),
            model_type: "test".to_string(),
            supported_languages: vec![],
            capabilities: ParserCapabilities {
                supports_tokenization: false,
                supports_pos_tagging: false,
                supports_lemmatization: false,
                supports_dependency_parsing: false,
                supports_morphological_features: false,
                max_sentence_length: None,
            },
        };
        let cloned = info.clone();
        assert_eq!(cloned.name, "Test");
        let debug = format!("{:?}", info);
        assert!(debug.contains("ParserInfo"));
    }

    // ======== ParserCapabilities Tests ========

    #[test]
    fn test_parser_capabilities_all_enabled() {
        let caps = ParserCapabilities {
            supports_tokenization: true,
            supports_pos_tagging: true,
            supports_lemmatization: true,
            supports_dependency_parsing: true,
            supports_morphological_features: true,
            max_sentence_length: Some(1000),
        };
        assert!(caps.supports_tokenization);
        assert!(caps.supports_pos_tagging);
        assert!(caps.supports_lemmatization);
        assert!(caps.supports_dependency_parsing);
        assert!(caps.supports_morphological_features);
        assert_eq!(caps.max_sentence_length, Some(1000));
    }

    // ======== AnalyzerInfo Tests ========

    #[test]
    fn test_analyzer_info_construction() {
        let info = AnalyzerInfo {
            name: "VerbNetAnalyzer".to_string(),
            version: "2.0".to_string(),
            approach: "verbnet".to_string(),
            capabilities: AnalyzerCapabilities {
                supports_theta_roles: true,
                supports_event_structure: true,
                supports_movement_chains: false,
                supports_little_v: true,
                theta_role_inventory: vec![ThetaRoleType::Agent, ThetaRoleType::Theme],
            },
        };
        assert_eq!(info.name, "VerbNetAnalyzer");
        assert_eq!(info.approach, "verbnet");
        assert!(info.capabilities.supports_theta_roles);
    }

    #[test]
    fn test_analyzer_info_clone_debug() {
        let info = AnalyzerInfo {
            name: "Test".to_string(),
            version: "1.0".to_string(),
            approach: "theory".to_string(),
            capabilities: AnalyzerCapabilities {
                supports_theta_roles: false,
                supports_event_structure: false,
                supports_movement_chains: false,
                supports_little_v: false,
                theta_role_inventory: vec![],
            },
        };
        let cloned = info.clone();
        assert_eq!(cloned.approach, "theory");
        let debug = format!("{:?}", info);
        assert!(debug.contains("AnalyzerInfo"));
    }

    // ======== AnalyzerConfig Tests ========

    #[test]
    fn test_analyzer_config_default() {
        let config = AnalyzerConfig::default();
        assert!(!config.enable_theta_assignment);
        assert!(!config.enable_event_creation);
        assert!(!config.enable_movement_detection);
        assert_eq!(config.performance_mode, PerformanceMode::Balanced);
        assert!(config.custom_settings.is_empty());
    }

    #[test]
    fn test_analyzer_config_construction() {
        let mut settings = HashMap::new();
        settings.insert("key".to_string(), "value".to_string());
        let config = AnalyzerConfig {
            enable_theta_assignment: true,
            enable_event_creation: true,
            enable_movement_detection: true,
            performance_mode: PerformanceMode::Accuracy,
            custom_settings: settings,
        };
        assert!(config.enable_theta_assignment);
        assert!(config.enable_event_creation);
        assert!(config.enable_movement_detection);
        assert_eq!(config.performance_mode, PerformanceMode::Accuracy);
        assert_eq!(
            config.custom_settings.get("key"),
            Some(&"value".to_string())
        );
    }

    // ======== PerformanceMode Tests ========

    #[test]
    fn test_performance_mode_default() {
        let mode = PerformanceMode::default();
        assert_eq!(mode, PerformanceMode::Balanced);
    }

    #[test]
    fn test_performance_mode_variants() {
        assert_eq!(format!("{:?}", PerformanceMode::Balanced), "Balanced");
        assert_eq!(format!("{:?}", PerformanceMode::Speed), "Speed");
        assert_eq!(format!("{:?}", PerformanceMode::Accuracy), "Accuracy");
    }

    #[test]
    fn test_performance_mode_clone_eq() {
        let mode = PerformanceMode::Speed;
        let cloned = mode.clone();
        assert_eq!(mode, cloned);
        assert_ne!(mode, PerformanceMode::Accuracy);
    }

    // ======== FeatureSet Tests ========

    #[test]
    fn test_feature_set_default() {
        let fs = FeatureSet::default();
        assert!(fs.morphological.is_empty());
        assert!(fs.semantic.is_empty());
        assert!(fs.verbnet.is_none());
        assert!(fs.custom.is_empty());
    }

    #[test]
    fn test_feature_set_with_verbnet() {
        let vn_features = VerbNetFeatures {
            verb_class: Some("give-13.1".to_string()),
            theta_roles: vec![
                ThetaRoleType::Agent,
                ThetaRoleType::Theme,
                ThetaRoleType::Goal,
            ],
            selectional_restrictions: vec!["animate".to_string()],
        };
        let fs = FeatureSet {
            morphological: HashMap::new(),
            semantic: HashMap::new(),
            verbnet: Some(vn_features),
            custom: HashMap::new(),
        };
        assert!(fs.verbnet.is_some());
        assert_eq!(
            fs.verbnet.as_ref().unwrap().verb_class,
            Some("give-13.1".to_string())
        );
    }

    // ======== VerbNetFeatures Tests ========

    #[test]
    fn test_verbnet_features_construction() {
        let vn = VerbNetFeatures {
            verb_class: Some("run-51.3.2".to_string()),
            theta_roles: vec![ThetaRoleType::Agent],
            selectional_restrictions: vec!["animate".to_string(), "concrete".to_string()],
        };
        assert_eq!(vn.verb_class, Some("run-51.3.2".to_string()));
        assert_eq!(vn.theta_roles.len(), 1);
        assert_eq!(vn.selectional_restrictions.len(), 2);
    }

    #[test]
    fn test_verbnet_features_clone_debug() {
        let vn = VerbNetFeatures {
            verb_class: None,
            theta_roles: vec![],
            selectional_restrictions: vec![],
        };
        let cloned = vn.clone();
        assert!(cloned.verb_class.is_none());
        let debug = format!("{:?}", vn);
        assert!(debug.contains("VerbNetFeatures"));
    }

    // ======== ExtractorCapabilities Tests ========

    #[test]
    fn test_extractor_capabilities_construction() {
        let caps = ExtractorCapabilities {
            name: "VerbNetExtractor".to_string(),
            supported_features: vec![
                "theta_roles".to_string(),
                "selectional_restrictions".to_string(),
            ],
            requires_pos_tags: true,
            requires_lemmas: true,
            batch_optimized: true,
        };
        assert_eq!(caps.name, "VerbNetExtractor");
        assert!(caps.requires_pos_tags);
        assert!(caps.batch_optimized);
    }

    // ======== ModelMetadata Tests ========

    #[test]
    fn test_model_metadata_construction() {
        let meta = ModelMetadata {
            identifier: "english-ewt-2.15".to_string(),
            name: "English EWT".to_string(),
            version: "2.15".to_string(),
            language: "en".to_string(),
            model_type: ModelType::UDPipe215,
            file_size: Some(50_000_000),
            download_url: Some("https://example.com/model.udpipe".to_string()),
            checksum: Some("abc123".to_string()),
        };
        assert_eq!(meta.identifier, "english-ewt-2.15");
        assert_eq!(meta.model_type, ModelType::UDPipe215);
        assert_eq!(meta.file_size, Some(50_000_000));
    }

    #[test]
    fn test_model_metadata_clone_debug() {
        let meta = ModelMetadata {
            identifier: "test".to_string(),
            name: "Test".to_string(),
            version: "1.0".to_string(),
            language: "en".to_string(),
            model_type: ModelType::UDPipe12,
            file_size: None,
            download_url: None,
            checksum: None,
        };
        let cloned = meta.clone();
        assert_eq!(cloned.identifier, "test");
        let debug = format!("{:?}", meta);
        assert!(debug.contains("ModelMetadata"));
    }

    // ======== ModelType Tests ========

    #[test]
    fn test_model_type_variants() {
        assert_eq!(format!("{:?}", ModelType::UDPipe12), "UDPipe12");
        assert_eq!(format!("{:?}", ModelType::UDPipe215), "UDPipe215");
        let custom = ModelType::Custom("spacy".to_string());
        assert!(format!("{:?}", custom).contains("spacy"));
    }

    #[test]
    fn test_model_type_eq() {
        assert_eq!(ModelType::UDPipe12, ModelType::UDPipe12);
        assert_ne!(ModelType::UDPipe12, ModelType::UDPipe215);
        assert_eq!(
            ModelType::Custom("x".to_string()),
            ModelType::Custom("x".to_string())
        );
        assert_ne!(
            ModelType::Custom("x".to_string()),
            ModelType::Custom("y".to_string())
        );
    }

    // ======== ModelCapabilities Tests ========

    #[test]
    fn test_model_capabilities_construction() {
        let caps = ModelCapabilities {
            accuracy_metrics: Some(AccuracyMetrics {
                pos_accuracy: 0.95,
                lemma_accuracy: 0.97,
                dependency_accuracy: 0.92,
            }),
            performance_metrics: Some(PerformanceMetrics {
                tokens_per_second: 10000.0,
                memory_usage_mb: 256.0,
                model_size_mb: 50.0,
            }),
            supported_features: vec!["tokenization".to_string()],
        };
        assert!(caps.accuracy_metrics.is_some());
        assert!(caps.performance_metrics.is_some());
    }

    // ======== AccuracyMetrics Tests ========

    #[test]
    fn test_accuracy_metrics_construction() {
        let acc = AccuracyMetrics {
            pos_accuracy: 0.96,
            lemma_accuracy: 0.98,
            dependency_accuracy: 0.94,
        };
        assert_eq!(acc.pos_accuracy, 0.96);
        assert_eq!(acc.lemma_accuracy, 0.98);
        assert_eq!(acc.dependency_accuracy, 0.94);
    }

    // ======== PerformanceMetrics Tests ========

    #[test]
    fn test_performance_metrics_construction() {
        let perf = PerformanceMetrics {
            tokens_per_second: 5000.0,
            memory_usage_mb: 128.0,
            model_size_mb: 25.0,
        };
        assert_eq!(perf.tokens_per_second, 5000.0);
        assert_eq!(perf.memory_usage_mb, 128.0);
        assert_eq!(perf.model_size_mb, 25.0);
    }

    // ======== CachedResult Tests ========

    #[test]
    fn test_cached_result_construction() {
        use canopy_tokenizer::{AnalysisMetrics, LogicalForm};
        use std::collections::HashMap;
        use std::time::{Duration, SystemTime};

        let analysis = SemanticAnalysis {
            tokens: vec![],
            frames: vec![],
            predicates: vec![],
            logical_form: LogicalForm {
                predicates: vec![],
                variables: HashMap::new(),
                quantifiers: vec![],
            },
            metrics: AnalysisMetrics {
                total_time_us: 0,
                tokenization_time_us: 0,
                framenet_time_us: 0,
                verbnet_time_us: 0,
                wordnet_time_us: 0,
                token_count: 0,
                frame_count: 0,
                predicate_count: 0,
            },
        };

        let result = CachedResult {
            text_hash: "abc123".to_string(),
            analysis,
            timestamp: SystemTime::now(),
            ttl: Duration::from_secs(3600),
        };
        assert_eq!(result.text_hash, "abc123");
        assert_eq!(result.ttl, Duration::from_secs(3600));
    }

    // ======== CacheStats Tests ========

    #[test]
    fn test_cache_stats_default() {
        let stats = CacheStats::default();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.size_bytes, 0);
        assert_eq!(stats.entry_count, 0);
    }

    #[test]
    fn test_cache_stats_construction() {
        let stats = CacheStats {
            hits: 100,
            misses: 20,
            size_bytes: 1024000,
            entry_count: 50,
        };
        assert_eq!(stats.hits, 100);
        assert_eq!(stats.misses, 20);
        // Hit rate: 100 / 120 = 83.3%
        let hit_rate = stats.hits as f64 / (stats.hits + stats.misses) as f64;
        assert!((hit_rate - 0.833).abs() < 0.01);
    }

    // ======== Metrics Tests ========

    #[test]
    fn test_metrics_default() {
        let metrics = Metrics::default();
        assert!(metrics.timings.is_empty());
        assert!(metrics.counts.is_empty());
        assert!(metrics.errors.is_empty());
    }

    #[test]
    fn test_metrics_construction() {
        let mut timings = HashMap::new();
        timings.insert("parse".to_string(), vec![10, 20, 15]);
        let mut counts = HashMap::new();
        counts.insert("tokens".to_string(), 100);
        let mut errors = HashMap::new();
        errors.insert("timeout".to_string(), 2);

        let metrics = Metrics {
            timings,
            counts,
            errors,
        };
        assert_eq!(metrics.timings.get("parse").unwrap().len(), 3);
        assert_eq!(metrics.counts.get("tokens"), Some(&100));
        assert_eq!(metrics.errors.get("timeout"), Some(&2));
    }

    // ======== ParserConfig Tests ========

    #[test]
    fn test_parser_config_construction() {
        let config = ParserConfig {
            model_path: Some("/path/to/model.udpipe".to_string()),
            model_type: ModelType::UDPipe215,
            performance_mode: PerformanceMode::Speed,
            enable_caching: true,
        };
        assert!(config.model_path.is_some());
        assert_eq!(config.model_type, ModelType::UDPipe215);
        assert_eq!(config.performance_mode, PerformanceMode::Speed);
        assert!(config.enable_caching);
    }

    // ======== ExtractorConfig Tests ========

    #[test]
    fn test_extractor_config_construction() {
        let config = ExtractorConfig {
            extractor_type: "verbnet".to_string(),
            enable_verbnet: true,
            custom_rules: vec!["rule1".to_string(), "rule2".to_string()],
        };
        assert_eq!(config.extractor_type, "verbnet");
        assert!(config.enable_verbnet);
        assert_eq!(config.custom_rules.len(), 2);
    }

    // ======== CacheConfig Tests ========

    #[test]
    fn test_cache_config_construction() {
        let config = CacheConfig {
            cache_type: "lru".to_string(),
            max_size_mb: 512,
            ttl_seconds: 3600,
        };
        assert_eq!(config.cache_type, "lru");
        assert_eq!(config.max_size_mb, 512);
        assert_eq!(config.ttl_seconds, 3600);
    }

    // ======== MetricsConfig Tests ========

    #[test]
    fn test_metrics_config_construction() {
        let config = MetricsConfig {
            enabled: true,
            backend: "prometheus".to_string(),
            collection_interval_ms: 1000,
        };
        assert!(config.enabled);
        assert_eq!(config.backend, "prometheus");
        assert_eq!(config.collection_interval_ms, 1000);
    }

    #[test]
    fn test_metrics_config_clone_debug() {
        let config = MetricsConfig {
            enabled: false,
            backend: "none".to_string(),
            collection_interval_ms: 0,
        };
        let cloned = config.clone();
        assert!(!cloned.enabled);
        let debug = format!("{:?}", config);
        assert!(debug.contains("MetricsConfig"));
    }
}
