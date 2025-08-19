//! Main pipeline orchestrator using dependency injection
//!
//! This module provides the core pipeline that orchestrates the analysis
//! process using injected dependencies, making it highly testable and flexible.

use crate::container::PipelineContainer;
use crate::error::{AnalysisError, PipelineError};
use crate::traits::*;
use async_trait::async_trait;
use canopy_core::Word;
use canopy_semantics::SemanticAnalysis;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, instrument};

/// Main linguistic analysis pipeline with dependency injection
pub struct LinguisticPipeline {
    /// Dependency injection container
    container: PipelineContainer,

    /// Pipeline configuration
    config: PipelineConfig,

    /// Pipeline metrics
    metrics: PipelineMetrics,
}

/// Pipeline configuration
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    /// Enable caching for improved performance
    pub enable_caching: bool,

    /// Enable metrics collection
    pub enable_metrics: bool,

    /// Maximum text length to process
    pub max_text_length: usize,

    /// Timeout for analysis operations
    pub timeout_seconds: u64,

    /// Performance mode
    pub performance_mode: PerformanceMode,

    /// Enable parallel processing
    pub enable_parallel: bool,

    /// Batch size for parallel processing
    pub batch_size: usize,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            enable_caching: true,
            enable_metrics: true,
            max_text_length: 10_000,
            timeout_seconds: 30,
            performance_mode: PerformanceMode::Balanced,
            enable_parallel: false,
            batch_size: 10,
        }
    }
}

/// Pipeline performance metrics
#[derive(Debug, Clone, Default)]
pub struct PipelineMetrics {
    /// Total texts processed
    pub texts_processed: u64,

    /// Total processing time
    pub total_time: Duration,

    /// Layer 1 processing time
    pub layer1_time: Duration,

    /// Layer 2 processing time
    pub layer2_time: Duration,

    /// Feature extraction time
    pub feature_extraction_time: Duration,

    /// Cache hits
    pub cache_hits: u64,

    /// Cache misses
    pub cache_misses: u64,

    /// Errors encountered
    pub errors: u64,

    /// Performance by text length
    pub performance_by_length: HashMap<String, Duration>,
}

impl PipelineMetrics {
    /// Calculate average processing time per text
    pub fn avg_processing_time(&self) -> Duration {
        if self.texts_processed == 0 {
            Duration::ZERO
        } else {
            self.total_time / self.texts_processed as u32
        }
    }

    /// Calculate cache hit rate
    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            self.cache_hits as f64 / total as f64
        }
    }

    /// Calculate texts per second throughput
    pub fn throughput(&self) -> f64 {
        if self.total_time.is_zero() {
            0.0
        } else {
            self.texts_processed as f64 / self.total_time.as_secs_f64()
        }
    }
}

/// Result of a single pipeline stage
#[derive(Debug, Clone)]
pub struct StageResult<T> {
    /// The result data
    pub result: T,

    /// Time taken for this stage
    pub duration: Duration,

    /// Stage-specific metrics
    pub metrics: HashMap<String, f64>,

    /// Any warnings generated
    pub warnings: Vec<String>,
}

/// Pipeline execution context
#[derive(Debug, Clone)]
pub struct PipelineContext {
    /// Unique request ID for tracing
    pub request_id: String,

    /// Original input text
    pub input_text: String,

    /// Start time of processing
    pub start_time: Instant,

    /// Configuration for this request
    pub config: PipelineConfig,

    /// Custom context data
    pub custom_data: HashMap<String, String>,
}

impl PipelineContext {
    pub fn new(text: String, config: PipelineConfig) -> Self {
        Self {
            request_id: uuid::Uuid::new_v4().to_string(),
            input_text: text,
            start_time: Instant::now(),
            config,
            custom_data: HashMap::new(),
        }
    }

    /// Get elapsed time since start
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Check if processing has timed out
    pub fn is_timed_out(&self) -> bool {
        self.elapsed().as_secs() >= self.config.timeout_seconds
    }
}

/// Pipeline stage enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum PipelineStage {
    Input,
    Layer1Parsing,
    FeatureExtraction,
    Layer2Analysis,
    Output,
}

impl LinguisticPipeline {
    /// Create a new pipeline with dependency injection
    pub fn new(container: PipelineContainer, config: PipelineConfig) -> Self {
        Self {
            container,
            config,
            metrics: PipelineMetrics::default(),
        }
    }

    /// Process text through the complete pipeline
    #[instrument(skip(self, text), fields(text_len = text.len()))]
    pub async fn analyze(&mut self, text: &str) -> Result<SemanticAnalysis, PipelineError> {
        let context = PipelineContext::new(text.to_string(), self.config.clone());
        self.analyze_with_context(context).await
    }

    /// Process text with full context and tracing
    #[instrument(skip(self, context), fields(request_id = %context.request_id))]
    pub async fn analyze_with_context(
        &mut self,
        mut context: PipelineContext,
    ) -> Result<SemanticAnalysis, PipelineError> {
        info!(
            "Starting pipeline analysis for request {}",
            context.request_id
        );

        // Validate input
        self.validate_input(&context)?;

        // Check cache first
        if let Some(cached) = self.check_cache(&context).await? {
            info!("Cache hit for request {}", context.request_id);
            self.metrics.cache_hits += 1;
            return Ok(cached);
        }
        self.metrics.cache_misses += 1;

        // Stage 1: Layer 1 Parsing (Morphosyntactic)
        let layer1_result = self.run_layer1(&context).await.map_err(|e| {
            error!("Layer 1 failed for request {}: {:?}", context.request_id, e);
            self.metrics.errors += 1;
            e
        })?;

        debug!(
            "Layer 1 completed in {:?} with {} words",
            layer1_result.duration,
            layer1_result.result.len()
        );

        // Check timeout
        if context.is_timed_out() {
            return Err(PipelineError::Timeout(context.elapsed()));
        }

        // Stage 2: Feature Extraction (Optional)
        let enhanced_words = if !self.container.extractors.is_empty() {
            self.run_feature_extraction(&context, layer1_result.result)
                .await?
        } else {
            layer1_result.result
        };

        // Stage 3: Layer 2 Analysis (Semantic)
        let layer2_result = self
            .run_layer2(&context, enhanced_words)
            .await
            .map_err(|e| {
                error!("Layer 2 failed for request {}: {:?}", context.request_id, e);
                self.metrics.errors += 1;
                e
            })?;

        debug!(
            "Layer 2 completed in {:?} with {} events",
            layer2_result.duration,
            layer2_result.result.events.len()
        );

        // Update metrics
        self.update_metrics(&context, &layer1_result, &layer2_result);

        // Cache result
        self.cache_result(&context, &layer2_result.result).await?;

        info!(
            "Pipeline completed for request {} in {:?}",
            context.request_id,
            context.elapsed()
        );

        Ok(layer2_result.result)
    }

    /// Validate input text
    fn validate_input(&self, context: &PipelineContext) -> Result<(), PipelineError> {
        if context.input_text.is_empty() {
            return Err(PipelineError::InvalidInput("Empty input text".to_string()));
        }

        if context.input_text.len() > self.config.max_text_length {
            return Err(PipelineError::InvalidInput(format!(
                "Text too long: {} > {}",
                context.input_text.len(),
                self.config.max_text_length
            )));
        }

        Ok(())
    }

    /// Check cache for existing result
    async fn check_cache(
        &self,
        context: &PipelineContext,
    ) -> Result<Option<SemanticAnalysis>, PipelineError> {
        if !self.config.enable_caching {
            return Ok(None);
        }

        if let Some(cache) = self.container.cache() {
            let cache_key = self.generate_cache_key(&context.input_text);
            if let Some(cached) = cache.get(&cache_key).await {
                // Check TTL
                if cached.timestamp.elapsed().unwrap_or(Duration::MAX) < cached.ttl {
                    return Ok(Some(cached.analysis));
                }
            }
        }

        Ok(None)
    }

    /// Run Layer 1 parsing
    #[instrument(skip(self, context))]
    async fn run_layer1(
        &self,
        context: &PipelineContext,
    ) -> Result<StageResult<Vec<Word>>, PipelineError> {
        let start = Instant::now();

        let words = self
            .container
            .parser()
            .parse(&context.input_text)
            .await
            .map_err(PipelineError::AnalysisError)?;

        let duration = start.elapsed();

        Ok(StageResult {
            result: words,
            duration,
            metrics: HashMap::from([("words_parsed".to_string(), words.len() as f64)]),
            warnings: Vec::new(),
        })
    }

    /// Run feature extraction
    #[instrument(skip(self, context, words))]
    async fn run_feature_extraction(
        &self,
        context: &PipelineContext,
        words: Vec<Word>,
    ) -> Result<Vec<Word>, PipelineError> {
        let start = Instant::now();

        // For now, just return the words as-is
        // In a full implementation, we'd enhance words with extracted features
        let enhanced_words = words;

        let duration = start.elapsed();
        self.metrics.feature_extraction_time = self.metrics.feature_extraction_time + duration;

        Ok(enhanced_words)
    }

    /// Run Layer 2 semantic analysis
    #[instrument(skip(self, context, words))]
    async fn run_layer2(
        &self,
        context: &PipelineContext,
        words: Vec<Word>,
    ) -> Result<StageResult<SemanticAnalysis>, PipelineError> {
        let start = Instant::now();

        // Clone the analyzer to get mutable access (in practice, we'd use Arc<Mutex<>> or similar)
        let analysis = {
            // This is a simplified approach - in practice we'd need proper async coordination
            let mut analyzer = self.container.analyzer().clone();
            analyzer
                .analyze(words)
                .await
                .map_err(PipelineError::AnalysisError)?
        };

        let duration = start.elapsed();

        Ok(StageResult {
            result: analysis,
            duration,
            metrics: HashMap::from([
                ("events_created".to_string(), analysis.events.len() as f64),
                (
                    "theta_assignments".to_string(),
                    analysis.theta_assignments.len() as f64,
                ),
            ]),
            warnings: Vec::new(),
        })
    }

    /// Update pipeline metrics
    fn update_metrics(
        &mut self,
        context: &PipelineContext,
        layer1: &StageResult<Vec<Word>>,
        layer2: &StageResult<SemanticAnalysis>,
    ) {
        self.metrics.texts_processed += 1;
        self.metrics.total_time += context.elapsed();
        self.metrics.layer1_time += layer1.duration;
        self.metrics.layer2_time += layer2.duration;

        // Track performance by text length category
        let length_category = match context.input_text.len() {
            0..=50 => "short",
            51..=200 => "medium",
            201..=1000 => "long",
            _ => "very_long",
        };

        self.metrics
            .performance_by_length
            .entry(length_category.to_string())
            .and_modify(|d| *d += context.elapsed())
            .or_insert(context.elapsed());
    }

    /// Cache analysis result
    async fn cache_result(
        &self,
        context: &PipelineContext,
        analysis: &SemanticAnalysis,
    ) -> Result<(), PipelineError> {
        if !self.config.enable_caching {
            return Ok(());
        }

        if let Some(cache) = self.container.cache() {
            let cache_key = self.generate_cache_key(&context.input_text);
            let cached_result = CachedResult {
                text_hash: cache_key.clone(),
                analysis: analysis.clone(),
                timestamp: std::time::SystemTime::now(),
                ttl: Duration::from_secs(3600), // 1 hour TTL
            };

            cache
                .set(&cache_key, cached_result)
                .await
                .map_err(PipelineError::AnalysisError)?;
        }

        Ok(())
    }

    /// Generate cache key for input text
    fn generate_cache_key(&self, text: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        format!("canopy_cache_{:x}", hasher.finish())
    }

    /// Get current pipeline metrics
    pub fn metrics(&self) -> &PipelineMetrics {
        &self.metrics
    }

    /// Check if pipeline is ready for processing
    pub fn is_ready(&self) -> bool {
        self.container.is_ready()
    }

    /// Process multiple texts in batch
    pub async fn analyze_batch(
        &mut self,
        texts: Vec<String>,
    ) -> Result<Vec<SemanticAnalysis>, PipelineError> {
        let mut results = Vec::new();

        if self.config.enable_parallel && texts.len() > 1 {
            // TODO: Implement parallel processing
            // For now, process sequentially
            for text in texts {
                results.push(self.analyze(&text).await?);
            }
        } else {
            // Sequential processing
            for text in texts {
                results.push(self.analyze(&text).await?);
            }
        }

        Ok(results)
    }
}

/// Builder for creating linguistic pipelines
pub struct PipelineBuilder {
    container: Option<PipelineContainer>,
    config: PipelineConfig,
}

impl PipelineBuilder {
    pub fn new() -> Self {
        Self {
            container: None,
            config: PipelineConfig::default(),
        }
    }

    /// Set the dependency injection container
    pub fn with_container(mut self, container: PipelineContainer) -> Self {
        self.container = Some(container);
        self
    }

    /// Set pipeline configuration
    pub fn with_config(mut self, config: PipelineConfig) -> Self {
        self.config = config;
        self
    }

    /// Configure caching
    pub fn with_caching(mut self, enabled: bool) -> Self {
        self.config.enable_caching = enabled;
        self
    }

    /// Configure metrics
    pub fn with_metrics(mut self, enabled: bool) -> Self {
        self.config.enable_metrics = enabled;
        self
    }

    /// Set performance mode
    pub fn with_performance_mode(mut self, mode: PerformanceMode) -> Self {
        self.config.performance_mode = mode;
        self
    }

    /// Build the pipeline
    pub fn build(self) -> Result<LinguisticPipeline, PipelineError> {
        let container = self.container.ok_or_else(|| {
            PipelineError::ConfigurationError("Container is required".to_string())
        })?;

        Ok(LinguisticPipeline::new(container, self.config))
    }
}

impl Default for PipelineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::container::ContainerBuilder;
    use crate::implementations::test_doubles::*;

    #[test]
    fn test_pipeline_creation() {
        let factory = std::sync::Arc::new(MockComponentFactory::new());

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
            .await
            .unwrap();

        let pipeline = PipelineBuilder::new()
            .with_container(container)
            .with_caching(false)
            .build()
            .unwrap();

        assert!(pipeline.is_ready());
    }

    #[test]
    fn test_pipeline_metrics() {
        // Test that metrics are properly tracked
        let metrics = PipelineMetrics {
            texts_processed: 10,
            total_time: Duration::from_secs(5),
            cache_hits: 3,
            cache_misses: 7,
            ..Default::default()
        };

        assert_eq!(metrics.avg_processing_time(), Duration::from_millis(500));
        assert_eq!(metrics.cache_hit_rate(), 0.3);
        assert_eq!(metrics.throughput(), 2.0);
    }
}
