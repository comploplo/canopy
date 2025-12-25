//! Main pipeline orchestrator using dependency injection
//!
//! This module provides the core pipeline that orchestrates the analysis
//! process using injected dependencies, making it highly testable and flexible.

use crate::container::PipelineContainer;
use crate::error::PipelineError;
use crate::traits::*;
use canopy_core::Word;
use canopy_events::{ComposedEvents, DependencyArc, EventComposer, SentenceAnalysis};
use canopy_tokenizer::SemanticLayer1Output as SemanticL1Output;
use canopy_tokenizer::{SemanticCoordinator, coordinator::CoordinatorConfig};
use canopy_treebank::types::DependencyRelation;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, instrument, warn};

/// Main linguistic analysis pipeline with dependency injection
pub struct LinguisticPipeline {
    /// Dependency injection container
    container: PipelineContainer,

    /// Pipeline configuration
    config: PipelineConfig,

    /// Pipeline metrics
    metrics: PipelineMetrics,
}

/// Result of full pipeline analysis including event composition
#[derive(Debug, Clone)]
pub struct FullAnalysisResult {
    /// Layer 1/2 semantic analysis (tokens, frames, predicates)
    pub analysis: SemanticL1Output,

    /// Layer 2 event composition (Neo-Davidsonian events with theta roles)
    pub events: Option<ComposedEvents>,

    /// Timing breakdown for each stage
    pub timing: AnalysisTiming,
}

/// Timing breakdown for pipeline stages
#[derive(Debug, Clone)]
pub struct AnalysisTiming {
    /// Total end-to-end processing time
    pub total: Duration,

    /// Layer 1 morphosyntactic parsing + semantic enhancement
    pub layer1: Duration,

    /// Layer 2 semantic token extraction
    pub layer2: Duration,

    /// Event composition time
    pub event_composition: Duration,
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

    /// Enable semantic analysis (VerbNet/FrameNet/WordNet)
    /// Disable for faster tests that don't need semantic data
    pub enable_semantic_analysis: bool,
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
            enable_semantic_analysis: true,
        }
    }
}

impl PipelineConfig {
    /// Create a config suitable for fast tests (no semantic analysis)
    pub fn for_testing() -> Self {
        Self {
            enable_semantic_analysis: false,
            ..Default::default()
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
    pub async fn analyze(&mut self, text: &str) -> Result<SemanticL1Output, PipelineError> {
        let context = PipelineContext::new(text.to_string(), self.config.clone());
        self.analyze_with_context(context).await
    }

    /// Process text through the complete pipeline including event composition
    ///
    /// This method runs the full analysis pipeline including:
    /// - Layer 1: Morphosyntactic parsing with semantic enhancement
    /// - Layer 2: Semantic token extraction
    /// - Event Composition: Neo-Davidsonian event structures
    ///
    /// Returns both the semantic analysis and composed events.
    #[instrument(skip(self, text), fields(text_len = text.len()))]
    pub async fn analyze_with_events(
        &mut self,
        text: &str,
    ) -> Result<FullAnalysisResult, PipelineError> {
        let context = PipelineContext::new(text.to_string(), self.config.clone());
        self.analyze_full_with_context(context).await
    }

    /// Process text with full context including event composition
    #[instrument(skip(self, context), fields(request_id = %context.request_id))]
    pub async fn analyze_full_with_context(
        &mut self,
        context: PipelineContext,
    ) -> Result<FullAnalysisResult, PipelineError> {
        let start = Instant::now();

        info!(
            "Starting full pipeline analysis for request {}",
            context.request_id
        );

        // Validate input
        self.validate_input(&context)?;

        // Stage 1: Layer 1 Parsing (Morphosyntactic)
        let layer1_result = self.run_layer1(&context).await.map_err(|e| {
            error!("Layer 1 failed for request {}: {:?}", context.request_id, e);
            self.metrics.errors += 1;
            e
        })?;

        let layer1_time = layer1_result.duration;
        debug!(
            "Layer 1 completed in {:?} with {} words",
            layer1_time,
            layer1_result.result.len()
        );

        // Check timeout
        if context.is_timed_out() {
            return Err(PipelineError::Timeout(context.elapsed()));
        }

        // Keep reference to words for event composition
        let words = layer1_result.result.clone();

        // Stage 2: Layer 2 Analysis (Semantic tokens)
        let layer2_result = self
            .run_layer2(&context, words.clone())
            .await
            .map_err(|e| {
                error!("Layer 2 failed for request {}: {:?}", context.request_id, e);
                self.metrics.errors += 1;
                e
            })?;

        let layer2_time = layer2_result.duration;
        debug!(
            "Layer 2 completed in {:?} with {} predicates",
            layer2_time,
            layer2_result.result.predicates.len()
        );

        // Stage 3: Event Composition (using EventComposer)
        let event_result = if self.config.enable_semantic_analysis {
            match self.run_event_composition(&context, &words).await {
                Ok(result) => {
                    debug!(
                        "Event composition completed in {:?} with {} events",
                        result.duration,
                        result.result.events.len()
                    );
                    Some(result)
                }
                Err(e) => {
                    warn!("Event composition failed: {:?}", e);
                    None
                }
            }
        } else {
            None
        };

        let event_time = event_result
            .as_ref()
            .map(|r| r.duration)
            .unwrap_or(Duration::ZERO);
        let total_time = start.elapsed();

        // Update metrics
        self.metrics.texts_processed += 1;
        self.metrics.total_time += total_time;
        self.metrics.layer1_time += layer1_time;
        self.metrics.layer2_time += layer2_time;

        info!(
            "Full pipeline completed for request {} in {:?} (L1: {:?}, L2: {:?}, Events: {:?})",
            context.request_id, total_time, layer1_time, layer2_time, event_time
        );

        Ok(FullAnalysisResult {
            analysis: layer2_result.result,
            events: event_result.map(|r| r.result),
            timing: AnalysisTiming {
                total: total_time,
                layer1: layer1_time,
                layer2: layer2_time,
                event_composition: event_time,
            },
        })
    }

    /// Process text with full context and tracing
    #[instrument(skip(self, context), fields(request_id = %context.request_id))]
    pub async fn analyze_with_context(
        &mut self,
        context: PipelineContext,
    ) -> Result<SemanticL1Output, PipelineError> {
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
        let enhanced_words = if false {
            // TODO: Re-enable when extractors are accessible
            self.run_feature_extraction(&context, layer1_result.result.clone())
                .await?
        } else {
            layer1_result.result.clone()
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
            layer2_result.result.predicates.len()
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
    ) -> Result<Option<SemanticL1Output>, PipelineError> {
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

    /// Run Layer 1 parsing with semantic-first approach
    #[instrument(skip(self, context))]
    async fn run_layer1(
        &self,
        context: &PipelineContext,
    ) -> Result<StageResult<Vec<Word>>, PipelineError> {
        let start = Instant::now();

        // First tokenize using traditional parser
        let base_words = self
            .container
            .parser()
            .parse(&context.input_text)
            .await
            .map_err(PipelineError::AnalysisError)?;

        // If semantic analysis is disabled, return base words without enhancement
        if !self.config.enable_semantic_analysis {
            let duration = start.elapsed();
            return Ok(StageResult {
                result: base_words,
                duration,
                metrics: HashMap::new(),
                warnings: Vec::new(),
            });
        }

        // Enhanced Layer 1: Use semantic coordinator for token-level analysis
        let coordinator_config = CoordinatorConfig {
            enable_verbnet: true,
            enable_framenet: true,
            enable_wordnet: true,
            enable_lexicon: true,
            confidence_threshold: 0.1,
            l1_cache_memory_mb: 100, // Allocate 100MB for L1 cache
            ..CoordinatorConfig::default()
        };

        let coordinator = SemanticCoordinator::new(coordinator_config)
            .map_err(|e| PipelineError::InvalidInput(e.to_string()))?;

        // Enhance each word with semantic analysis
        let mut enhanced_words = Vec::new();
        let mut total_semantic_time = Duration::ZERO;

        for word in base_words {
            let semantic_start = Instant::now();

            // Get unified semantic analysis for the lemma
            let semantic_result = coordinator.analyze(&word.lemma).unwrap_or_else(|_| {
                // Graceful degradation: return empty result
                canopy_tokenizer::coordinator::Layer1SemanticResult::new(
                    word.lemma.clone(),
                    word.lemma.clone(),
                )
            });

            total_semantic_time += semantic_start.elapsed();

            // Create enhanced word with semantic features in misc field
            let mut enhanced_word = word.clone();

            // Build semantic metadata as a formatted string
            let mut semantic_metadata = Vec::new();

            if let Some(ref verbnet) = semantic_result.verbnet {
                semantic_metadata.push(format!("verbnet_classes={}", verbnet.verb_classes.len()));
                semantic_metadata.push(format!("verbnet_confidence={:.2}", verbnet.confidence));
            }

            if let Some(ref framenet) = semantic_result.framenet {
                semantic_metadata.push(format!("framenet_frames={}", framenet.frames.len()));
                semantic_metadata.push(format!("framenet_confidence={:.2}", framenet.confidence));
            }

            if let Some(ref wordnet) = semantic_result.wordnet {
                semantic_metadata.push(format!("wordnet_synsets={}", wordnet.synsets.len()));
                semantic_metadata.push(format!("wordnet_confidence={:.2}", wordnet.confidence));
            }

            if let Some(ref lexicon) = semantic_result.lexicon {
                semantic_metadata
                    .push(format!("lexicon_classes={}", lexicon.classifications.len()));
                semantic_metadata.push(format!(
                    "is_stop_word={}",
                    !lexicon.get_stop_words().is_empty()
                ));
            }

            semantic_metadata.push(format!(
                "semantic_confidence={:.2}",
                semantic_result.confidence
            ));

            // Add semantic metadata to misc field
            let semantic_misc = semantic_metadata.join("|");
            enhanced_word.misc = Some(match enhanced_word.misc {
                Some(existing) => format!("{existing};{semantic_misc}"),
                None => semantic_misc,
            });

            enhanced_words.push(enhanced_word);
        }

        let duration = start.elapsed();
        let word_count = enhanced_words.len();

        Ok(StageResult {
            result: enhanced_words,
            duration,
            metrics: HashMap::from([
                ("words_parsed".to_string(), word_count as f64),
                (
                    "semantic_time_ms".to_string(),
                    total_semantic_time.as_millis() as f64,
                ),
                (
                    "avg_semantic_time_per_word_ms".to_string(),
                    if word_count == 0 {
                        0.0
                    } else {
                        total_semantic_time.as_millis() as f64 / word_count as f64
                    },
                ),
            ]),
            warnings: Vec::new(),
        })
    }

    /// Run feature extraction
    #[instrument(skip(self, _context, words))]
    async fn run_feature_extraction(
        &self,
        _context: &PipelineContext,
        words: Vec<Word>,
    ) -> Result<Vec<Word>, PipelineError> {
        let _start = Instant::now();

        // For now, just return the words as-is
        // In a full implementation, we'd enhance words with extracted features
        let enhanced_words = words;

        let _duration = _start.elapsed();
        // Note: metrics updates would be handled differently in async context

        Ok(enhanced_words)
    }

    /// Run Layer 2 semantic analysis with enhanced semantic features
    #[instrument(skip(self, _context, words))]
    async fn run_layer2(
        &self,
        _context: &PipelineContext,
        words: Vec<Word>,
    ) -> Result<StageResult<SemanticL1Output>, PipelineError> {
        let start = Instant::now();

        // Enhanced Layer 2: Convert enhanced words to semantic tokens and create full analysis
        let mut semantic_tokens = Vec::new();
        let frames = Vec::new();
        let mut predicates = Vec::new();
        let mut semantic_confidence_sum = 0.0;
        let mut semantic_word_count = 0;

        // Convert each word to a semantic token
        for word in words {
            // Parse semantic metadata from misc field
            let mut semantic_confidence = 0.0;
            let mut verbnet_class_count = 0;

            if let Some(ref misc_str) = word.misc {
                // Parse semantic metadata from the misc field
                for part in misc_str.split(';') {
                    for item in part.split('|') {
                        if let Some((key, value)) = item.split_once('=') {
                            match key {
                                "semantic_confidence" => {
                                    semantic_confidence = value.parse().unwrap_or(0.0);
                                }
                                "verbnet_classes" => {
                                    verbnet_class_count = value.parse().unwrap_or(0);
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }

            if semantic_confidence > 0.0 {
                semantic_confidence_sum += semantic_confidence;
                semantic_word_count += 1;
            }

            // Create semantic token
            let semantic_token = canopy_tokenizer::SemanticToken {
                text: word.text.clone(),
                lemma: word.lemma.clone(),
                semantic_class: canopy_tokenizer::SemanticClass::Predicate, // Default classification
                frames: Vec::new(), // Would be populated from FrameNet analysis
                verbnet_classes: Vec::new(), // Would be populated from VerbNet analysis
                wordnet_senses: Vec::new(), // Would be populated from WordNet analysis
                morphology: canopy_tokenizer::MorphologicalAnalysis {
                    lemma: word.lemma.clone(),
                    features: HashMap::new(), // Convert from MorphFeatures if needed
                    inflection_type: canopy_tokenizer::InflectionType::None,
                    is_recognized: true,
                },
                confidence: semantic_confidence,
            };

            semantic_tokens.push(semantic_token);

            // Extract predicates from VerbNet analysis
            if verbnet_class_count > 0 && word.upos == canopy_core::UPos::Verb {
                let predicate = canopy_tokenizer::SemanticPredicate {
                    lemma: word.lemma.clone(),
                    verbnet_class: Some(format!("class_{verbnet_class_count}")),
                    theta_grid: vec![canopy_core::ThetaRole::Agent], // Simplified
                    selectional_restrictions: HashMap::new(),
                    aspectual_class: canopy_tokenizer::AspectualClass::Unknown,
                    confidence: semantic_confidence,
                };
                predicates.push(predicate);
            }
        }

        // Calculate overall confidence
        let overall_confidence = if semantic_word_count > 0 {
            semantic_confidence_sum / semantic_word_count as f32
        } else {
            0.0
        };

        // Create logical form (simplified)
        let logical_form = canopy_tokenizer::LogicalForm {
            predicates: Vec::new(), // Would be populated by deeper semantic analysis
            variables: HashMap::new(),
            quantifiers: Vec::new(),
        };

        // Create analysis metrics
        let analysis_metrics = canopy_tokenizer::AnalysisMetrics {
            total_time_us: start.elapsed().as_micros() as u64,
            tokenization_time_us: 0, // Already done in Layer 1
            framenet_time_us: 0,
            verbnet_time_us: 0,
            wordnet_time_us: 0,
            token_count: semantic_tokens.len(),
            frame_count: frames.len(),
            predicate_count: predicates.len(),
        };

        // Create comprehensive semantic analysis
        let analysis = SemanticL1Output {
            tokens: semantic_tokens,
            frames,
            predicates,
            logical_form,
            metrics: analysis_metrics,
        };

        let duration = start.elapsed();
        let token_count = analysis.tokens.len();
        let predicate_count = analysis.predicates.len();

        Ok(StageResult {
            result: analysis,
            duration,
            metrics: HashMap::from([
                ("semantic_tokens_created".to_string(), token_count as f64),
                ("predicates_extracted".to_string(), predicate_count as f64),
                ("semantic_confidence".to_string(), overall_confidence as f64),
                (
                    "enhanced_words_processed".to_string(),
                    semantic_word_count as f64,
                ),
            ]),
            warnings: Vec::new(),
        })
    }

    /// Run Layer 2 event composition using EventComposer
    ///
    /// This integrates the Neo-Davidsonian event composition system to create
    /// structured events from the Layer 1 semantic analysis and dependency structure.
    #[instrument(skip(self, _context, words))]
    async fn run_event_composition(
        &self,
        _context: &PipelineContext,
        words: &[Word],
    ) -> Result<StageResult<ComposedEvents>, PipelineError> {
        let start = Instant::now();

        // Create semantic coordinator for L1 analysis
        let coordinator_config = CoordinatorConfig {
            enable_verbnet: true,
            enable_framenet: true,
            enable_wordnet: true,
            enable_lexicon: true,
            confidence_threshold: 0.1,
            l1_cache_memory_mb: 100,
            ..CoordinatorConfig::default()
        };

        let coordinator = SemanticCoordinator::new(coordinator_config).map_err(|e| {
            PipelineError::InvalidInput(format!("Failed to create coordinator: {}", e))
        })?;

        // Create event composer
        let composer = EventComposer::new().map_err(|e| {
            PipelineError::InvalidInput(format!("Failed to create event composer: {}", e))
        })?;

        // Analyze each word through Layer 1 to get rich semantic results
        let mut l1_tokens = Vec::with_capacity(words.len());
        for word in words {
            let mut l1_result = coordinator.analyze(&word.lemma).unwrap_or_else(|_| {
                canopy_tokenizer::coordinator::Layer1SemanticResult::new(
                    word.text.clone(),
                    word.lemma.clone(),
                )
            });
            // Set POS from the parsed word
            l1_result.pos = Some(word.upos);
            l1_tokens.push(l1_result);
        }

        // Extract dependency arcs from parsed words
        let deps = Self::extract_dependency_arcs(words);

        // Build sentence analysis using the builder
        let sentence_text = words
            .iter()
            .map(|w| w.text.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        let sentence_analysis =
            SentenceAnalysis::new(sentence_text, l1_tokens).with_dependencies(deps);

        // Compose events using the EventComposer
        let composed_events = composer.compose_sentence(&sentence_analysis).map_err(|e| {
            PipelineError::AnalysisError(crate::error::AnalysisError::ParseFailed(format!(
                "Event composition failed: {}",
                e
            )))
        })?;

        let duration = start.elapsed();
        let event_count = composed_events.events.len();
        let unbound_count = composed_events.unbound_entities.len();

        debug!(
            "Event composition completed: {} events in {:?}",
            event_count, duration
        );

        Ok(StageResult {
            result: composed_events,
            duration,
            metrics: HashMap::from([
                ("events_composed".to_string(), event_count as f64),
                ("unbound_entities".to_string(), unbound_count as f64),
            ]),
            warnings: Vec::new(),
        })
    }

    /// Extract DependencyArc structures from Words with head/deprel information
    fn extract_dependency_arcs(words: &[Word]) -> Vec<DependencyArc> {
        let mut arcs = Vec::new();

        for word in words {
            if let Some(head) = word.head {
                // Skip root (head = 0)
                if head > 0 {
                    let relation = Self::deprel_to_dependency_relation(&word.deprel);
                    arcs.push(DependencyArc::new(
                        head.saturating_sub(1),    // Convert 1-indexed to 0-indexed
                        word.id.saturating_sub(1), // Convert 1-indexed to 0-indexed
                        relation,
                    ));
                }
            }
        }

        arcs
    }

    /// Convert canopy_core::DepRel to canopy_treebank::DependencyRelation
    fn deprel_to_dependency_relation(deprel: &canopy_core::DepRel) -> DependencyRelation {
        use canopy_core::DepRel;

        match deprel {
            DepRel::Nsubj => DependencyRelation::NominalSubject,
            DepRel::Obj => DependencyRelation::Object,
            DepRel::Iobj => DependencyRelation::IndirectObject,
            DepRel::Obl => DependencyRelation::Oblique,
            DepRel::Advmod => DependencyRelation::AdverbialModifier,
            DepRel::Amod => DependencyRelation::AdjectivalModifier,
            DepRel::Det => DependencyRelation::Determiner,
            DepRel::Case => DependencyRelation::Case,
            DepRel::Nmod => DependencyRelation::NominalModifier,
            DepRel::Conj => DependencyRelation::Conjunction,
            DepRel::Cc => DependencyRelation::CoordinatingConjunction,
            DepRel::Compound => DependencyRelation::Compound,
            DepRel::Aux => DependencyRelation::Auxiliary,
            DepRel::Cop => DependencyRelation::Copula,
            DepRel::Mark => DependencyRelation::Mark,
            DepRel::Punct => DependencyRelation::Punctuation,
            _ => DependencyRelation::Other(format!("{:?}", deprel)), // Fallback for unmapped relations
        }
    }

    /// Update pipeline metrics
    fn update_metrics(
        &mut self,
        context: &PipelineContext,
        layer1: &StageResult<Vec<Word>>,
        layer2: &StageResult<SemanticL1Output>,
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
        analysis: &SemanticL1Output,
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
    ) -> Result<Vec<SemanticL1Output>, PipelineError> {
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

// #[cfg(test)]
// mod tests {  // Temporarily disabled due to deprecated dependencies
//     use super::*;
//     use crate::container::ContainerBuilder;
//     use crate::implementations::test_doubles::*;
//
//     #[tokio::test]
//     async fn test_pipeline_creation() {
//         let factory = std::sync::Arc::new(MockComponentFactory::new());
//
//         let container = ContainerBuilder::new()
//             .with_parser(ParserConfig {
//                 model_path: Some("test".to_string()),
//                 model_type: ModelType::UDPipe12,
//                 performance_mode: PerformanceMode::Balanced,
//                 enable_caching: false,
//             })
//             .with_analyzer(AnalyzerConfig::default())
//             .with_factory(factory)
//             .build()
//             .await
//             .unwrap();
//
//         let pipeline = PipelineBuilder::new()
//             .with_container(container)
//             .with_caching(false)
//             .build()
//             .unwrap();
//
//         assert!(pipeline.is_ready());
//     }
//
//     #[test]
//     fn test_pipeline_metrics() {
//         // Test that metrics are properly tracked
//         let metrics = PipelineMetrics {
//             texts_processed: 10,
//             total_time: Duration::from_secs(5),
//             cache_hits: 3,
//             cache_misses: 7,
//             ..Default::default()
//         };
//
//         assert_eq!(metrics.avg_processing_time(), Duration::from_millis(500));
//         assert_eq!(metrics.cache_hit_rate(), 0.3);
//         assert_eq!(metrics.throughput(), 2.0);
//     }
// }
