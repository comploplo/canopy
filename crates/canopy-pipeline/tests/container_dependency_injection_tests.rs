//! Comprehensive dependency injection tests for PipelineContainer
//!
//! Tests all container functionality including builder patterns, service resolution,
//! error handling, and lifecycle management with 95%+ coverage target.

use async_trait::async_trait;
use canopy_core::Word;
use canopy_pipeline::container::{ContainerBuilder, PipelineContainer};
use canopy_pipeline::error::{AnalysisError, PipelineError};
use canopy_pipeline::traits::*;
use canopy_semantic_layer::SemanticLayer1Output as SemanticAnalysis;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[cfg(test)]
mod tests {
    use super::*;

    // Mock implementations for dependency injection testing

    #[derive(Debug)]
    struct MockParser {
        ready: bool,
        should_fail: bool,
        parse_count: Arc<Mutex<usize>>,
    }

    impl MockParser {
        fn new(ready: bool, should_fail: bool) -> Self {
            Self {
                ready,
                should_fail,
                parse_count: Arc::new(Mutex::new(0)),
            }
        }

        fn parse_calls(&self) -> usize {
            *self.parse_count.lock().unwrap()
        }
    }

    #[async_trait]
    impl MorphosyntacticParser for MockParser {
        fn is_ready(&self) -> bool {
            self.ready
        }

        async fn parse(&self, text: &str) -> Result<Vec<Word>, AnalysisError> {
            *self.parse_count.lock().unwrap() += 1;

            if self.should_fail {
                return Err(AnalysisError::ParseFailed(format!(
                    "Mock parser failed for: {}",
                    text
                )));
            }

            Ok(vec![Word::new(1, text.to_string(), 0, text.len())])
        }

        fn info(&self) -> ParserInfo {
            ParserInfo {
                name: "Mock Parser".to_string(),
                version: "1.0".to_string(),
                model_type: "mock".to_string(),
                supported_languages: vec!["en".to_string()],
                capabilities: ParserCapabilities {
                    supports_tokenization: true,
                    supports_pos_tagging: true,
                    supports_lemmatization: true,
                    supports_dependency_parsing: true,
                    supports_morphological_features: true,
                    max_sentence_length: Some(100),
                },
            }
        }
    }

    #[derive(Debug)]
    struct MockAnalyzer {
        ready: bool,
        should_fail: bool,
        analyze_count: Arc<Mutex<usize>>,
        config: Mutex<Option<AnalyzerConfig>>,
    }

    impl MockAnalyzer {
        fn new(ready: bool, should_fail: bool) -> Self {
            Self {
                ready,
                should_fail,
                analyze_count: Arc::new(Mutex::new(0)),
                config: Mutex::new(None),
            }
        }

        fn analyze_calls(&self) -> usize {
            *self.analyze_count.lock().unwrap()
        }
    }

    #[async_trait]
    impl SemanticAnalyzer for MockAnalyzer {
        async fn analyze(&mut self, _words: Vec<Word>) -> Result<SemanticAnalysis, AnalysisError> {
            *self.analyze_count.lock().unwrap() += 1;

            if self.should_fail {
                return Err(AnalysisError::ParseFailed(
                    "Mock analyzer failed".to_string(),
                ));
            }

            Ok(SemanticAnalysis {
                tokens: vec![],
                frames: vec![],
                predicates: vec![],
                logical_form: canopy_semantic_layer::LogicalForm {
                    predicates: vec![],
                    quantifiers: vec![],
                    variables: std::collections::HashMap::new(),
                },
                metrics: canopy_semantic_layer::AnalysisMetrics {
                    total_time_us: 0,
                    tokenization_time_us: 0,
                    framenet_time_us: 0,
                    verbnet_time_us: 0,
                    wordnet_time_us: 0,
                    token_count: 0,
                    frame_count: 0,
                    predicate_count: 0,
                },
            })
        }

        fn info(&self) -> AnalyzerInfo {
            AnalyzerInfo {
                name: "Mock Analyzer".to_string(),
                version: "1.0".to_string(),
                approach: "mock".to_string(),
                capabilities: AnalyzerCapabilities {
                    supports_theta_roles: true,
                    supports_event_structure: true,
                    supports_movement_chains: true,
                    supports_little_v: true,
                    theta_role_inventory: vec![],
                },
            }
        }

        fn is_ready(&self) -> bool {
            self.ready
        }

        fn configure(&mut self, config: AnalyzerConfig) -> Result<(), AnalysisError> {
            *self.config.lock().unwrap() = Some(config);
            Ok(())
        }
    }

    #[derive(Debug)]
    struct MockExtractor {
        should_fail: bool,
        extract_count: Arc<Mutex<usize>>,
    }

    impl MockExtractor {
        fn new(should_fail: bool) -> Self {
            Self {
                should_fail,
                extract_count: Arc::new(Mutex::new(0)),
            }
        }

        fn extract_calls(&self) -> usize {
            *self.extract_count.lock().unwrap()
        }
    }

    #[async_trait]
    impl FeatureExtractor for MockExtractor {
        async fn extract_features(&self, _word: &Word) -> Result<FeatureSet, AnalysisError> {
            *self.extract_count.lock().unwrap() += 1;

            if self.should_fail {
                return Err(AnalysisError::ParseFailed(
                    "Mock extractor failed".to_string(),
                ));
            }

            Ok(FeatureSet::default())
        }

        fn capabilities(&self) -> ExtractorCapabilities {
            ExtractorCapabilities {
                name: "Mock Extractor".to_string(),
                supported_features: vec!["verbnet".to_string()],
                requires_pos_tags: false,
                requires_lemmas: false,
                batch_optimized: false,
            }
        }
    }

    #[derive(Debug)]
    struct MockModelLoader {
        should_fail: bool,
        models: Vec<ModelMetadata>,
        load_count: Arc<Mutex<usize>>,
    }

    impl MockModelLoader {
        fn new(should_fail: bool) -> Self {
            Self {
                should_fail,
                models: vec![ModelMetadata {
                    identifier: "test-model".to_string(),
                    name: "Test Model".to_string(),
                    version: "1.0".to_string(),
                    language: "en".to_string(),
                    model_type: ModelType::UDPipe12,
                    file_size: Some(1024),
                    download_url: None,
                    checksum: None,
                }],
                load_count: Arc::new(Mutex::new(0)),
            }
        }

        fn load_calls(&self) -> usize {
            *self.load_count.lock().unwrap()
        }
    }

    #[async_trait]
    impl ModelLoader for MockModelLoader {
        async fn load_model(&self, _identifier: &str) -> Result<Box<dyn Model>, AnalysisError> {
            *self.load_count.lock().unwrap() += 1;

            if self.should_fail {
                return Err(AnalysisError::ParseFailed(
                    "Mock model loader failed".to_string(),
                ));
            }

            Ok(Box::new(MockModel::new()))
        }

        async fn is_model_available(&self, identifier: &str) -> bool {
            if self.should_fail {
                return false;
            }
            self.models.iter().any(|m| m.identifier == identifier)
        }

        async fn list_models(&self) -> Result<Vec<ModelMetadata>, AnalysisError> {
            if self.should_fail {
                return Err(AnalysisError::ParseFailed(
                    "Mock model listing failed".to_string(),
                ));
            }
            Ok(self.models.clone())
        }

        async fn ensure_model(&self, identifier: &str) -> Result<(), AnalysisError> {
            if self.should_fail {
                return Err(AnalysisError::ParseFailed(
                    "Mock model ensure failed".to_string(),
                ));
            }

            if !self.is_model_available(identifier).await {
                return Err(AnalysisError::ModelNotFound(identifier.to_string()));
            }

            Ok(())
        }
    }

    struct MockModel {
        metadata: ModelMetadata,
    }

    impl MockModel {
        fn new() -> Self {
            Self {
                metadata: ModelMetadata {
                    identifier: "mock-model".to_string(),
                    name: "Mock Model".to_string(),
                    version: "1.0".to_string(),
                    language: "en".to_string(),
                    model_type: ModelType::UDPipe12,
                    file_size: Some(1024),
                    download_url: None,
                    checksum: None,
                },
            }
        }
    }

    impl Model for MockModel {
        fn metadata(&self) -> &ModelMetadata {
            &self.metadata
        }

        fn capabilities(&self) -> ModelCapabilities {
            ModelCapabilities {
                accuracy_metrics: Some(AccuracyMetrics {
                    pos_accuracy: 0.95,
                    lemma_accuracy: 0.93,
                    dependency_accuracy: 0.89,
                }),
                performance_metrics: Some(PerformanceMetrics {
                    tokens_per_second: 1000.0,
                    memory_usage_mb: 50.0,
                    model_size_mb: 15.0,
                }),
                supported_features: vec!["tokenization".to_string(), "pos_tagging".to_string()],
            }
        }

        fn validate(&self) -> Result<(), AnalysisError> {
            Ok(())
        }
    }

    #[derive(Debug)]
    struct MockCacheProvider {
        should_fail: bool,
        cache: Arc<Mutex<HashMap<String, CachedResult>>>,
        stats: Arc<Mutex<CacheStats>>,
    }

    impl MockCacheProvider {
        fn new(should_fail: bool) -> Self {
            Self {
                should_fail,
                cache: Arc::new(Mutex::new(HashMap::new())),
                stats: Arc::new(Mutex::new(CacheStats::default())),
            }
        }
    }

    #[async_trait]
    impl CacheProvider for MockCacheProvider {
        async fn get(&self, key: &str) -> Option<CachedResult> {
            if self.should_fail {
                return None;
            }

            let mut stats = self.stats.lock().unwrap();
            if let Some(result) = self.cache.lock().unwrap().get(key) {
                stats.hits += 1;
                Some(result.clone())
            } else {
                stats.misses += 1;
                None
            }
        }

        async fn set(&self, key: &str, result: CachedResult) -> Result<(), AnalysisError> {
            if self.should_fail {
                return Err(AnalysisError::ParseFailed(
                    "Mock cache set failed".to_string(),
                ));
            }

            self.cache.lock().unwrap().insert(key.to_string(), result);
            Ok(())
        }

        async fn clear(&self) -> Result<(), AnalysisError> {
            if self.should_fail {
                return Err(AnalysisError::ParseFailed(
                    "Mock cache clear failed".to_string(),
                ));
            }

            self.cache.lock().unwrap().clear();
            Ok(())
        }

        fn stats(&self) -> CacheStats {
            self.stats.lock().unwrap().clone()
        }
    }

    #[derive(Debug)]
    struct MockMetricsCollector {
        should_fail: bool,
        metrics: Arc<Mutex<Metrics>>,
    }

    impl MockMetricsCollector {
        fn new(should_fail: bool) -> Self {
            Self {
                should_fail,
                metrics: Arc::new(Mutex::new(Metrics::default())),
            }
        }
    }

    impl MetricsCollector for MockMetricsCollector {
        fn record_timing(&self, operation: &str, duration_ms: u64) {
            if self.should_fail {
                return;
            }

            let mut metrics = self.metrics.lock().unwrap();
            metrics
                .timings
                .entry(operation.to_string())
                .or_insert_with(Vec::new)
                .push(duration_ms);
        }

        fn record_count(&self, operation: &str, count: u64) {
            if self.should_fail {
                return;
            }

            let mut metrics = self.metrics.lock().unwrap();
            let current = *metrics.counts.get(operation).unwrap_or(&0);
            metrics
                .counts
                .insert(operation.to_string(), current + count);
        }

        fn record_error(&self, operation: &str, _error: &str) {
            if self.should_fail {
                return;
            }

            let mut metrics = self.metrics.lock().unwrap();
            let current = *metrics.errors.get(operation).unwrap_or(&0);
            metrics.errors.insert(operation.to_string(), current + 1);
        }

        fn get_metrics(&self) -> Metrics {
            self.metrics.lock().unwrap().clone()
        }
    }

    struct MockComponentFactory {
        fail_parser: bool,
        fail_analyzer: bool,
        fail_extractor: bool,
        fail_cache: bool,
        fail_metrics: bool,
    }

    impl MockComponentFactory {
        fn new() -> Self {
            Self {
                fail_parser: false,
                fail_analyzer: false,
                fail_extractor: false,
                fail_cache: false,
                fail_metrics: false,
            }
        }

        fn with_parser_failure(mut self) -> Self {
            self.fail_parser = true;
            self
        }

        fn with_analyzer_failure(mut self) -> Self {
            self.fail_analyzer = true;
            self
        }

        fn with_extractor_failure(mut self) -> Self {
            self.fail_extractor = true;
            self
        }

        fn with_cache_failure(mut self) -> Self {
            self.fail_cache = true;
            self
        }

        fn with_metrics_failure(mut self) -> Self {
            self.fail_metrics = true;
            self
        }
    }

    impl ComponentFactory for MockComponentFactory {
        fn create_parser(
            &self,
            _config: &ParserConfig,
        ) -> Result<Box<dyn MorphosyntacticParser>, PipelineError> {
            if self.fail_parser {
                return Err(PipelineError::ConfigurationError(
                    "Failed to create parser".to_string(),
                ));
            }
            Ok(Box::new(MockParser::new(true, false)))
        }

        fn create_analyzer(
            &self,
            _config: &AnalyzerConfig,
        ) -> Result<Box<dyn SemanticAnalyzer>, PipelineError> {
            if self.fail_analyzer {
                return Err(PipelineError::ConfigurationError(
                    "Failed to create analyzer".to_string(),
                ));
            }
            Ok(Box::new(MockAnalyzer::new(true, false)))
        }

        fn create_extractor(
            &self,
            _config: &ExtractorConfig,
        ) -> Result<Box<dyn FeatureExtractor>, PipelineError> {
            if self.fail_extractor {
                return Err(PipelineError::ConfigurationError(
                    "Failed to create extractor".to_string(),
                ));
            }
            Ok(Box::new(MockExtractor::new(false)))
        }

        fn create_cache(
            &self,
            _config: &CacheConfig,
        ) -> Result<Box<dyn CacheProvider>, PipelineError> {
            if self.fail_cache {
                return Err(PipelineError::ConfigurationError(
                    "Failed to create cache".to_string(),
                ));
            }
            Ok(Box::new(MockCacheProvider::new(false)))
        }

        fn create_metrics(
            &self,
            _config: &MetricsConfig,
        ) -> Result<Box<dyn MetricsCollector>, PipelineError> {
            if self.fail_metrics {
                return Err(PipelineError::ConfigurationError(
                    "Failed to create metrics".to_string(),
                ));
            }
            Ok(Box::new(MockMetricsCollector::new(false)))
        }
    }

    // Container Creation Tests

    #[test]
    fn test_container_new() {
        let parser = Arc::new(MockParser::new(true, false));
        let analyzer = Arc::new(MockAnalyzer::new(true, false));
        let model_loader = Arc::new(MockModelLoader::new(false));
        let factory = Arc::new(MockComponentFactory::new());

        let container = PipelineContainer::new(
            parser.clone(),
            analyzer.clone(),
            model_loader.clone(),
            factory,
        );

        assert!(container.is_ready());
        assert!(container.parser().is_ready());
        assert!(container.analyzer().is_ready());
    }

    #[test]
    fn test_container_not_ready_when_parser_not_ready() {
        let parser = Arc::new(MockParser::new(false, false));
        let analyzer = Arc::new(MockAnalyzer::new(true, false));
        let model_loader = Arc::new(MockModelLoader::new(false));
        let factory = Arc::new(MockComponentFactory::new());

        let container = PipelineContainer::new(parser, analyzer, model_loader, factory);

        assert!(!container.is_ready());
    }

    #[test]
    fn test_container_not_ready_when_analyzer_not_ready() {
        let parser = Arc::new(MockParser::new(true, false));
        let analyzer = Arc::new(MockAnalyzer::new(false, false));
        let model_loader = Arc::new(MockModelLoader::new(false));
        let factory = Arc::new(MockComponentFactory::new());

        let container = PipelineContainer::new(parser, analyzer, model_loader, factory);

        assert!(!container.is_ready());
    }

    #[tokio::test]
    async fn test_container_warm_up_success() {
        let parser = Arc::new(MockParser::new(true, false));
        let analyzer = Arc::new(MockAnalyzer::new(true, false));
        let model_loader = Arc::new(MockModelLoader::new(false));
        let factory = Arc::new(MockComponentFactory::new());

        let mut container = PipelineContainer::new(parser, analyzer, model_loader, factory);

        let result = container.warm_up().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_container_warm_up_failure_when_not_ready() {
        let parser = Arc::new(MockParser::new(false, false));
        let analyzer = Arc::new(MockAnalyzer::new(true, false));
        let model_loader = Arc::new(MockModelLoader::new(false));
        let factory = Arc::new(MockComponentFactory::new());

        let mut container = PipelineContainer::new(parser, analyzer, model_loader, factory);

        let result = container.warm_up().await;
        assert!(result.is_err());
        if let Err(PipelineError::NotReady(_)) = result {
            // Expected error type
        } else {
            panic!("Expected NotReady error");
        }
    }

    // Container Builder Tests

    #[test]
    fn test_container_builder_new() {
        let _builder = ContainerBuilder::new();
        let _builder1 = ContainerBuilder::new();
        let _builder2 = ContainerBuilder::default();
        // Test builder creation doesn't panic
    }

    #[test]
    fn test_container_builder_method_chaining() {
        let factory = Arc::new(MockComponentFactory::new());
        let parser_config = ParserConfig {
            model_path: Some("test".to_string()),
            model_type: ModelType::UDPipe12,
            performance_mode: PerformanceMode::Balanced,
            enable_caching: false,
        };
        let analyzer_config = AnalyzerConfig::default();

        let builder = ContainerBuilder::new()
            .with_parser(parser_config)
            .with_analyzer(analyzer_config)
            .with_factory(factory);

        // Test chaining doesn't panic
        std::mem::drop(builder);
    }

    #[tokio::test]
    async fn test_container_builder_build_success() {
        let factory = Arc::new(MockComponentFactory::new());
        let parser_config = ParserConfig {
            model_path: Some("test".to_string()),
            model_type: ModelType::UDPipe12,
            performance_mode: PerformanceMode::Balanced,
            enable_caching: false,
        };
        let analyzer_config = AnalyzerConfig::default();

        let container = ContainerBuilder::new()
            .with_parser(parser_config)
            .with_analyzer(analyzer_config)
            .with_factory(factory)
            .build()
            .await;

        assert!(container.is_ok());
        let container = container.unwrap();
        assert!(container.is_ready());
    }

    #[tokio::test]
    async fn test_container_builder_missing_factory() {
        let parser_config = ParserConfig {
            model_path: Some("test".to_string()),
            model_type: ModelType::UDPipe12,
            performance_mode: PerformanceMode::Balanced,
            enable_caching: false,
        };
        let analyzer_config = AnalyzerConfig::default();

        let result = ContainerBuilder::new()
            .with_parser(parser_config)
            .with_analyzer(analyzer_config)
            .build()
            .await;

        assert!(result.is_err());
        if let Err(PipelineError::ConfigurationError(msg)) = result {
            assert!(msg.contains("factory"));
        } else {
            panic!("Expected ConfigurationError for missing factory");
        }
    }

    #[tokio::test]
    async fn test_container_builder_missing_parser_config() {
        let factory = Arc::new(MockComponentFactory::new());
        let analyzer_config = AnalyzerConfig::default();

        let result = ContainerBuilder::new()
            .with_analyzer(analyzer_config)
            .with_factory(factory)
            .build()
            .await;

        assert!(result.is_err());
        if let Err(PipelineError::ConfigurationError(msg)) = result {
            assert!(msg.contains("Parser"));
        } else {
            panic!("Expected ConfigurationError for missing parser config");
        }
    }

    #[tokio::test]
    async fn test_container_builder_missing_analyzer_config() {
        let factory = Arc::new(MockComponentFactory::new());
        let parser_config = ParserConfig {
            model_path: Some("test".to_string()),
            model_type: ModelType::UDPipe12,
            performance_mode: PerformanceMode::Balanced,
            enable_caching: false,
        };

        let result = ContainerBuilder::new()
            .with_parser(parser_config)
            .with_factory(factory)
            .build()
            .await;

        assert!(result.is_err());
        if let Err(PipelineError::ConfigurationError(msg)) = result {
            assert!(msg.contains("Analyzer"));
        } else {
            panic!("Expected ConfigurationError for missing analyzer config");
        }
    }

    #[tokio::test]
    async fn test_container_builder_with_extractors() {
        let factory = Arc::new(MockComponentFactory::new());
        let parser_config = ParserConfig {
            model_path: Some("test".to_string()),
            model_type: ModelType::UDPipe12,
            performance_mode: PerformanceMode::Balanced,
            enable_caching: false,
        };
        let analyzer_config = AnalyzerConfig::default();
        let extractor_config = ExtractorConfig {
            extractor_type: "verbnet".to_string(),
            enable_verbnet: true,
            custom_rules: vec![],
        };

        let container = ContainerBuilder::new()
            .with_parser(parser_config)
            .with_analyzer(analyzer_config)
            .with_extractor("verbnet".to_string(), extractor_config)
            .with_factory(factory)
            .build()
            .await;

        assert!(container.is_ok());
        let container = container.unwrap();
        assert!(container.extractor("verbnet").is_some());
        assert!(container.extractor("nonexistent").is_none());
    }

    #[tokio::test]
    async fn test_container_builder_with_cache() {
        let factory = Arc::new(MockComponentFactory::new());
        let parser_config = ParserConfig {
            model_path: Some("test".to_string()),
            model_type: ModelType::UDPipe12,
            performance_mode: PerformanceMode::Balanced,
            enable_caching: false,
        };
        let analyzer_config = AnalyzerConfig::default();
        let cache_config = CacheConfig {
            cache_type: "memory".to_string(),
            max_size_mb: 100,
            ttl_seconds: 3600,
        };

        let container = ContainerBuilder::new()
            .with_parser(parser_config)
            .with_analyzer(analyzer_config)
            .with_cache(cache_config)
            .with_factory(factory)
            .build()
            .await;

        assert!(container.is_ok());
        let container = container.unwrap();
        assert!(container.cache().is_some());
    }

    #[tokio::test]
    async fn test_container_builder_with_metrics() {
        let factory = Arc::new(MockComponentFactory::new());
        let parser_config = ParserConfig {
            model_path: Some("test".to_string()),
            model_type: ModelType::UDPipe12,
            performance_mode: PerformanceMode::Balanced,
            enable_caching: false,
        };
        let analyzer_config = AnalyzerConfig::default();
        let metrics_config = MetricsConfig {
            enabled: true,
            backend: "memory".to_string(),
            collection_interval_ms: 1000,
        };

        let container = ContainerBuilder::new()
            .with_parser(parser_config)
            .with_analyzer(analyzer_config)
            .with_metrics(metrics_config)
            .with_factory(factory)
            .build()
            .await;

        assert!(container.is_ok());
        let container = container.unwrap();
        assert!(container.metrics().is_some());
    }

    // Component Factory Error Tests

    #[tokio::test]
    async fn test_container_builder_parser_creation_failure() {
        let factory = Arc::new(MockComponentFactory::new().with_parser_failure());
        let parser_config = ParserConfig {
            model_path: Some("test".to_string()),
            model_type: ModelType::UDPipe12,
            performance_mode: PerformanceMode::Balanced,
            enable_caching: false,
        };
        let analyzer_config = AnalyzerConfig::default();

        let result = ContainerBuilder::new()
            .with_parser(parser_config)
            .with_analyzer(analyzer_config)
            .with_factory(factory)
            .build()
            .await;

        assert!(result.is_err());
        if let Err(PipelineError::ConfigurationError(msg)) = result {
            assert!(msg.contains("parser"));
        } else {
            panic!("Expected ConfigurationError for parser creation failure");
        }
    }

    #[tokio::test]
    async fn test_container_builder_analyzer_creation_failure() {
        let factory = Arc::new(MockComponentFactory::new().with_analyzer_failure());
        let parser_config = ParserConfig {
            model_path: Some("test".to_string()),
            model_type: ModelType::UDPipe12,
            performance_mode: PerformanceMode::Balanced,
            enable_caching: false,
        };
        let analyzer_config = AnalyzerConfig::default();

        let result = ContainerBuilder::new()
            .with_parser(parser_config)
            .with_analyzer(analyzer_config)
            .with_factory(factory)
            .build()
            .await;

        assert!(result.is_err());
        if let Err(PipelineError::ConfigurationError(msg)) = result {
            assert!(msg.contains("analyzer"));
        } else {
            panic!("Expected ConfigurationError for analyzer creation failure");
        }
    }

    #[tokio::test]
    async fn test_container_builder_extractor_creation_failure() {
        let factory = Arc::new(MockComponentFactory::new().with_extractor_failure());
        let parser_config = ParserConfig {
            model_path: Some("test".to_string()),
            model_type: ModelType::UDPipe12,
            performance_mode: PerformanceMode::Balanced,
            enable_caching: false,
        };
        let analyzer_config = AnalyzerConfig::default();
        let extractor_config = ExtractorConfig {
            extractor_type: "verbnet".to_string(),
            enable_verbnet: true,
            custom_rules: vec![],
        };

        let result = ContainerBuilder::new()
            .with_parser(parser_config)
            .with_analyzer(analyzer_config)
            .with_extractor("verbnet".to_string(), extractor_config)
            .with_factory(factory)
            .build()
            .await;

        assert!(result.is_err());
        if let Err(PipelineError::ConfigurationError(msg)) = result {
            assert!(msg.contains("extractor"));
        } else {
            panic!("Expected ConfigurationError for extractor creation failure");
        }
    }

    #[tokio::test]
    async fn test_container_builder_cache_creation_failure() {
        let factory = Arc::new(MockComponentFactory::new().with_cache_failure());
        let parser_config = ParserConfig {
            model_path: Some("test".to_string()),
            model_type: ModelType::UDPipe12,
            performance_mode: PerformanceMode::Balanced,
            enable_caching: false,
        };
        let analyzer_config = AnalyzerConfig::default();
        let cache_config = CacheConfig {
            cache_type: "memory".to_string(),
            max_size_mb: 100,
            ttl_seconds: 3600,
        };

        let result = ContainerBuilder::new()
            .with_parser(parser_config)
            .with_analyzer(analyzer_config)
            .with_cache(cache_config)
            .with_factory(factory)
            .build()
            .await;

        assert!(result.is_err());
        if let Err(PipelineError::ConfigurationError(msg)) = result {
            assert!(msg.contains("cache"));
        } else {
            panic!("Expected ConfigurationError for cache creation failure");
        }
    }

    #[tokio::test]
    async fn test_container_builder_metrics_creation_failure() {
        let factory = Arc::new(MockComponentFactory::new().with_metrics_failure());
        let parser_config = ParserConfig {
            model_path: Some("test".to_string()),
            model_type: ModelType::UDPipe12,
            performance_mode: PerformanceMode::Balanced,
            enable_caching: false,
        };
        let analyzer_config = AnalyzerConfig::default();
        let metrics_config = MetricsConfig {
            enabled: true,
            backend: "memory".to_string(),
            collection_interval_ms: 1000,
        };

        let result = ContainerBuilder::new()
            .with_parser(parser_config)
            .with_analyzer(analyzer_config)
            .with_metrics(metrics_config)
            .with_factory(factory)
            .build()
            .await;

        assert!(result.is_err());
        if let Err(PipelineError::ConfigurationError(msg)) = result {
            assert!(msg.contains("metrics"));
        } else {
            panic!("Expected ConfigurationError for metrics creation failure");
        }
    }

    // Container Service Methods Tests

    #[tokio::test]
    async fn test_container_service_getters() {
        let parser = Arc::new(MockParser::new(true, false));
        let analyzer = Arc::new(MockAnalyzer::new(true, false));
        let model_loader = Arc::new(MockModelLoader::new(false));
        let factory = Arc::new(MockComponentFactory::new());

        let container = PipelineContainer::new(
            parser.clone(),
            analyzer.clone(),
            model_loader.clone(),
            factory,
        );

        // Verify components are correctly stored
        assert!(container.parser().is_ready());
        assert!(container.analyzer().is_ready());
        assert!(
            container
                .model_loader()
                .is_model_available("test-model")
                .await
        );
    }

    #[test]
    fn test_container_builder_static_method() {
        let _container = PipelineContainer::builder();
        // Test builder creation via static method
    }

    // Container Modification Tests

    #[test]
    fn test_container_add_extractor() {
        let parser = Arc::new(MockParser::new(true, false));
        let analyzer = Arc::new(MockAnalyzer::new(true, false));
        let model_loader = Arc::new(MockModelLoader::new(false));
        let factory = Arc::new(MockComponentFactory::new());

        let mut container = PipelineContainer::new(parser, analyzer, model_loader, factory);

        let extractor = Arc::new(MockExtractor::new(false));
        container.add_extractor("test-extractor".to_string(), extractor);

        assert!(container.extractor("test-extractor").is_some());
    }

    #[test]
    fn test_container_set_cache() {
        let parser = Arc::new(MockParser::new(true, false));
        let analyzer = Arc::new(MockAnalyzer::new(true, false));
        let model_loader = Arc::new(MockModelLoader::new(false));
        let factory = Arc::new(MockComponentFactory::new());

        let mut container = PipelineContainer::new(parser, analyzer, model_loader, factory);

        let cache = Arc::new(MockCacheProvider::new(false));
        container.set_cache(cache);

        assert!(container.cache().is_some());
    }

    #[test]
    fn test_container_set_metrics() {
        let parser = Arc::new(MockParser::new(true, false));
        let analyzer = Arc::new(MockAnalyzer::new(true, false));
        let model_loader = Arc::new(MockModelLoader::new(false));
        let factory = Arc::new(MockComponentFactory::new());

        let mut container = PipelineContainer::new(parser, analyzer, model_loader, factory);

        let metrics = Arc::new(MockMetricsCollector::new(false));
        container.set_metrics(metrics);

        assert!(container.metrics().is_some());
    }
}
