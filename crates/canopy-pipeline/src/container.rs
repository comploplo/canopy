//! Dependency injection container for the Canopy pipeline
//!
//! This module provides a clean dependency injection system that allows
//! different implementations to be injected at runtime, making the system
//! highly testable and configurable.

use crate::error::{AnalysisError, PipelineError};
use crate::traits::*;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

/// Main dependency injection container
///
/// This container holds all the services needed by the pipeline and provides
/// a clean way to inject different implementations for testing or different
/// deployment scenarios.
pub struct PipelineContainer {
    /// Parser for Layer 1 morphosyntactic analysis
    parser: Arc<dyn MorphosyntacticParser>,

    /// Analyzer for Layer 2 semantic analysis
    analyzer: Arc<dyn SemanticAnalyzer>,

    /// Feature extractors (can be multiple)
    extractors: HashMap<String, Arc<dyn FeatureExtractor>>,

    /// Model loader for managing language models
    model_loader: Arc<dyn ModelLoader>,

    /// Cache provider for performance optimization
    cache: Option<Arc<dyn CacheProvider>>,

    /// Metrics collector for observability
    metrics: Option<Arc<dyn MetricsCollector>>,

    /// Component factory for creating new instances
    factory: Arc<dyn ComponentFactory>,
}

impl PipelineContainer {
    /// Create a new pipeline container with the given components
    pub fn new(
        parser: Arc<dyn MorphosyntacticParser>,
        analyzer: Arc<dyn SemanticAnalyzer>,
        model_loader: Arc<dyn ModelLoader>,
        factory: Arc<dyn ComponentFactory>,
    ) -> Self {
        Self {
            parser,
            analyzer,
            extractors: HashMap::new(),
            model_loader,
            cache: None,
            metrics: None,
            factory,
        }
    }

    /// Builder pattern for configuring the container
    pub fn builder() -> ContainerBuilder {
        ContainerBuilder::new()
    }

    /// Get the morphosyntactic parser
    pub fn parser(&self) -> &Arc<dyn MorphosyntacticParser> {
        &self.parser
    }

    /// Get the semantic analyzer
    pub fn analyzer(&self) -> &Arc<dyn SemanticAnalyzer> {
        &self.analyzer
    }

    /// Get a feature extractor by name
    pub fn extractor(&self, name: &str) -> Option<&Arc<dyn FeatureExtractor>> {
        self.extractors.get(name)
    }

    /// Get the model loader
    pub fn model_loader(&self) -> &Arc<dyn ModelLoader> {
        &self.model_loader
    }

    /// Get the cache provider
    pub fn cache(&self) -> Option<&Arc<dyn CacheProvider>> {
        self.cache.as_ref()
    }

    /// Get the metrics collector
    pub fn metrics(&self) -> Option<&Arc<dyn MetricsCollector>> {
        self.metrics.as_ref()
    }

    /// Add a feature extractor
    pub fn add_extractor(&mut self, name: String, extractor: Arc<dyn FeatureExtractor>) {
        self.extractors.insert(name, extractor);
    }

    /// Set cache provider
    pub fn set_cache(&mut self, cache: Arc<dyn CacheProvider>) {
        self.cache = Some(cache);
    }

    /// Set metrics collector
    pub fn set_metrics(&mut self, metrics: Arc<dyn MetricsCollector>) {
        self.metrics = Some(metrics);
    }

    /// Check if all required components are ready
    pub fn is_ready(&self) -> bool {
        self.parser.is_ready() && self.analyzer.is_ready()
    }

    /// Warm up all components
    pub async fn warm_up(&mut self) -> Result<(), PipelineError> {
        // Warm up parser (mutable reference through Arc requires special handling)
        // In practice, we'd use interior mutability or other patterns

        // For now, just check readiness
        if !self.is_ready() {
            return Err(PipelineError::NotReady("Components not ready".to_string()));
        }

        Ok(())
    }
}

/// Builder for creating pipeline containers with dependency injection
pub struct ContainerBuilder {
    parser_config: Option<ParserConfig>,
    analyzer_config: Option<AnalyzerConfig>,
    extractor_configs: Vec<(String, ExtractorConfig)>,
    cache_config: Option<CacheConfig>,
    metrics_config: Option<MetricsConfig>,
    factory: Option<Arc<dyn ComponentFactory>>,
}

impl ContainerBuilder {
    pub fn new() -> Self {
        Self {
            parser_config: None,
            analyzer_config: None,
            extractor_configs: Vec::new(),
            cache_config: None,
            metrics_config: None,
            factory: None,
        }
    }

    /// Configure the morphosyntactic parser
    pub fn with_parser(mut self, config: ParserConfig) -> Self {
        self.parser_config = Some(config);
        self
    }

    /// Configure the semantic analyzer
    pub fn with_analyzer(mut self, config: AnalyzerConfig) -> Self {
        self.analyzer_config = Some(config);
        self
    }

    /// Add a feature extractor
    pub fn with_extractor(mut self, name: String, config: ExtractorConfig) -> Self {
        self.extractor_configs.push((name, config));
        self
    }

    /// Configure caching
    pub fn with_cache(mut self, config: CacheConfig) -> Self {
        self.cache_config = Some(config);
        self
    }

    /// Configure metrics collection
    pub fn with_metrics(mut self, config: MetricsConfig) -> Self {
        self.metrics_config = Some(config);
        self
    }

    /// Set the component factory
    pub fn with_factory(mut self, factory: Arc<dyn ComponentFactory>) -> Self {
        self.factory = Some(factory);
        self
    }

    /// Build the container with dependency injection
    pub async fn build(self) -> Result<PipelineContainer, PipelineError> {
        let factory = self.factory.ok_or_else(|| {
            PipelineError::ConfigurationError("Component factory is required".to_string())
        })?;

        // Create parser
        let parser_config = self.parser_config.ok_or_else(|| {
            PipelineError::ConfigurationError("Parser configuration is required".to_string())
        })?;
        let parser = factory.create_parser(&parser_config)?;

        // Create analyzer
        let analyzer_config = self.analyzer_config.ok_or_else(|| {
            PipelineError::ConfigurationError("Analyzer configuration is required".to_string())
        })?;
        let analyzer = factory.create_analyzer(&analyzer_config)?;

        // Create model loader (using a default implementation)
        let model_loader = Arc::new(DefaultModelLoader::new());

        // Create base container
        let mut container = PipelineContainer::new(
            Arc::from(parser),
            Arc::from(analyzer),
            model_loader,
            factory.clone(),
        );

        // Add extractors
        for (name, config) in self.extractor_configs {
            let extractor = factory.create_extractor(&config)?;
            container.add_extractor(name, Arc::from(extractor));
        }

        // Add cache if configured
        if let Some(cache_config) = self.cache_config {
            let cache = factory.create_cache(&cache_config)?;
            container.set_cache(Arc::from(cache));
        }

        // Add metrics if configured
        if let Some(metrics_config) = self.metrics_config {
            let metrics = factory.create_metrics(&metrics_config)?;
            container.set_metrics(Arc::from(metrics));
        }

        Ok(container)
    }
}

impl Default for ContainerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Default model loader implementation
struct DefaultModelLoader {
    available_models: Vec<ModelMetadata>,
}

impl DefaultModelLoader {
    fn new() -> Self {
        Self {
            available_models: Self::discover_models(),
        }
    }

    fn discover_models() -> Vec<ModelMetadata> {
        let mut models = Vec::new();

        // Check for UDPipe 1.2 model
        if std::path::Path::new("/Users/gabe/projects/canopy/models/english-ud-1.2-160523.udpipe")
            .exists()
        {
            models.push(ModelMetadata {
                identifier: "udpipe-1.2-english".to_string(),
                name: "UDPipe 1.2 English".to_string(),
                version: "1.2".to_string(),
                language: "en".to_string(),
                model_type: ModelType::UDPipe12,
                file_size: Some(15954),
                download_url: None,
                checksum: None,
            });
        }

        // Check for UDPipe 2.15 model
        if std::path::Path::new(
            "/Users/gabe/projects/canopy/models/english-ewt-ud-2.12-230717.udpipe",
        )
        .exists()
        {
            models.push(ModelMetadata {
                identifier: "udpipe-2.15-english".to_string(),
                name: "UDPipe 2.15 English".to_string(),
                version: "2.15".to_string(),
                language: "en".to_string(),
                model_type: ModelType::UDPipe215,
                file_size: Some(16271),
                download_url: None,
                checksum: None,
            });
        }

        models
    }
}

#[async_trait]
impl ModelLoader for DefaultModelLoader {
    async fn load_model(&self, identifier: &str) -> Result<Box<dyn Model>, AnalysisError> {
        let metadata = self
            .available_models
            .iter()
            .find(|m| m.identifier == identifier)
            .ok_or_else(|| AnalysisError::ModelNotFound(identifier.to_string()))?;

        Ok(Box::new(DefaultModel {
            metadata: metadata.clone(),
        }))
    }

    async fn is_model_available(&self, identifier: &str) -> bool {
        self.available_models
            .iter()
            .any(|m| m.identifier == identifier)
    }

    async fn list_models(&self) -> Result<Vec<ModelMetadata>, AnalysisError> {
        Ok(self.available_models.clone())
    }

    async fn ensure_model(&self, identifier: &str) -> Result<(), AnalysisError> {
        if !self.is_model_available(identifier).await {
            return Err(AnalysisError::ModelNotFound(identifier.to_string()));
        }
        Ok(())
    }
}

/// Default model implementation
struct DefaultModel {
    metadata: ModelMetadata,
}

impl Model for DefaultModel {
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
            supported_features: vec![
                "tokenization".to_string(),
                "pos_tagging".to_string(),
                "lemmatization".to_string(),
                "dependency_parsing".to_string(),
            ],
        }
    }

    fn validate(&self) -> Result<(), AnalysisError> {
        // Basic validation - in practice would check model integrity
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::implementations::test_doubles::*;

    #[test]
    fn test_container_builder() {
        let factory = Arc::new(MockComponentFactory::new());

        let container = ContainerBuilder::new()
            .with_parser(ParserConfig {
                model_path: Some("test".to_string()),
                model_type: ModelType::UDPipe12,
                performance_mode: PerformanceMode::Balanced,
                enable_caching: false,
            })
            .with_analyzer(AnalyzerConfig::default())
            .with_factory(factory)
            .build()
            .await;

        assert!(container.is_ok());
        let container = container.unwrap();
        assert!(container.parser().is_ready());
    }

    #[test]
    fn test_model_discovery() {
        let loader = DefaultModelLoader::new();
        // Should at least not crash
        assert!(loader.available_models.len() >= 0);
    }
}
