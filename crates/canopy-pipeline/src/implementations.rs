//! Test implementations and mock objects for dependency injection

#[cfg(test)]
pub mod test_doubles {
    use crate::error::{AnalysisError, PipelineError};
    use crate::traits::*;
    use async_trait::async_trait;
    use canopy_core::{DepRel, MorphFeatures, UPos, Word};
    use canopy_semantics::{Event, EventId, SemanticAnalysis};
    use std::collections::HashMap;

    /// Mock component factory for testing
    pub struct MockComponentFactory;

    impl MockComponentFactory {
        pub fn new() -> Self {
            Self
        }
    }

    impl ComponentFactory for MockComponentFactory {
        fn create_parser(
            &self,
            _config: &ParserConfig,
        ) -> Result<Box<dyn MorphosyntacticParser>, PipelineError> {
            Ok(Box::new(MockParser::new()))
        }

        fn create_analyzer(
            &self,
            _config: &AnalyzerConfig,
        ) -> Result<Box<dyn SemanticAnalyzer>, PipelineError> {
            Ok(Box::new(MockAnalyzer::new()))
        }

        fn create_extractor(
            &self,
            _config: &ExtractorConfig,
        ) -> Result<Box<dyn FeatureExtractor>, PipelineError> {
            Ok(Box::new(MockExtractor::new()))
        }

        fn create_cache(
            &self,
            _config: &CacheConfig,
        ) -> Result<Box<dyn CacheProvider>, PipelineError> {
            Ok(Box::new(MockCache::new()))
        }

        fn create_metrics(
            &self,
            _config: &MetricsConfig,
        ) -> Result<Box<dyn MetricsCollector>, PipelineError> {
            Ok(Box::new(MockMetrics::new()))
        }
    }

    /// Mock parser for testing
    pub struct MockParser {
        ready: bool,
    }

    impl MockParser {
        pub fn new() -> Self {
            Self { ready: true }
        }
    }

    #[async_trait]
    impl MorphosyntacticParser for MockParser {
        async fn parse(&self, text: &str) -> Result<Vec<Word>, AnalysisError> {
            // Simple mock parsing
            let words: Vec<Word> = text
                .split_whitespace()
                .enumerate()
                .map(|(i, word)| Word {
                    id: i + 1,
                    text: word.to_string(),
                    lemma: word.to_lowercase(),
                    upos: UPos::Noun,
                    xpos: None,
                    feats: MorphFeatures::default(),
                    head: Some(0),
                    deprel: DepRel::Root,
                    deps: None,
                    misc: None,
                    start: 0,
                    end: word.len(),
                })
                .collect();
            Ok(words)
        }

        fn info(&self) -> ParserInfo {
            ParserInfo {
                name: "MockParser".to_string(),
                version: "1.0".to_string(),
                model_type: "mock".to_string(),
                supported_languages: vec!["en".to_string()],
                capabilities: ParserCapabilities {
                    supports_tokenization: true,
                    supports_pos_tagging: true,
                    supports_lemmatization: true,
                    supports_dependency_parsing: true,
                    supports_morphological_features: true,
                    max_sentence_length: None,
                },
            }
        }

        fn is_ready(&self) -> bool {
            self.ready
        }
    }

    /// Mock semantic analyzer for testing
    pub struct MockAnalyzer {
        ready: bool,
    }

    impl MockAnalyzer {
        pub fn new() -> Self {
            Self { ready: true }
        }
    }

    #[async_trait]
    impl SemanticAnalyzer for MockAnalyzer {
        async fn analyze(&mut self, words: Vec<Word>) -> Result<SemanticAnalysis, AnalysisError> {
            // Create mock semantic analysis
            let enhanced_words: Vec<canopy_core::EnhancedWord> = words
                .into_iter()
                .map(|word| canopy_core::EnhancedWord {
                    base: word,
                    semantic_features: canopy_core::SemanticFeatures::default(),
                    confidence: canopy_core::FeatureConfidence::default(),
                })
                .collect();

            Ok(SemanticAnalysis {
                words: enhanced_words,
                events: vec![], // Mock empty events
                theta_assignments: HashMap::new(),
                confidence: 0.8,
                metrics: canopy_semantics::Layer2Metrics::default(),
            })
        }

        fn info(&self) -> AnalyzerInfo {
            AnalyzerInfo {
                name: "MockAnalyzer".to_string(),
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

        fn configure(&mut self, _config: AnalyzerConfig) -> Result<(), AnalysisError> {
            Ok(())
        }
    }

    /// Mock feature extractor for testing
    pub struct MockExtractor;

    impl MockExtractor {
        pub fn new() -> Self {
            Self
        }
    }

    #[async_trait]
    impl FeatureExtractor for MockExtractor {
        async fn extract_features(&self, _word: &Word) -> Result<FeatureSet, AnalysisError> {
            Ok(FeatureSet::default())
        }

        fn capabilities(&self) -> ExtractorCapabilities {
            ExtractorCapabilities {
                name: "MockExtractor".to_string(),
                supported_features: vec!["mock".to_string()],
                requires_pos_tags: false,
                requires_lemmas: false,
                batch_optimized: false,
            }
        }
    }

    /// Mock cache provider for testing
    pub struct MockCache {
        storage: HashMap<String, CachedResult>,
    }

    impl MockCache {
        pub fn new() -> Self {
            Self {
                storage: HashMap::new(),
            }
        }
    }

    #[async_trait]
    impl CacheProvider for MockCache {
        async fn get(&self, key: &str) -> Option<CachedResult> {
            self.storage.get(key).cloned()
        }

        async fn set(&self, _key: &str, _result: CachedResult) -> Result<(), AnalysisError> {
            // Mock cache doesn't actually store
            Ok(())
        }

        async fn clear(&self) -> Result<(), AnalysisError> {
            Ok(())
        }

        fn stats(&self) -> CacheStats {
            CacheStats::default()
        }
    }

    /// Mock metrics collector for testing
    pub struct MockMetrics {
        metrics: Metrics,
    }

    impl MockMetrics {
        pub fn new() -> Self {
            Self {
                metrics: Metrics::default(),
            }
        }
    }

    impl MetricsCollector for MockMetrics {
        fn record_timing(&self, _operation: &str, _duration_ms: u64) {
            // Mock recording
        }

        fn record_count(&self, _operation: &str, _count: u64) {
            // Mock recording
        }

        fn record_error(&self, _operation: &str, _error: &str) {
            // Mock recording
        }

        fn get_metrics(&self) -> Metrics {
            self.metrics.clone()
        }
    }
}
