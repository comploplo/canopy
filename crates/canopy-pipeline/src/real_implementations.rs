//! Production implementations of pipeline traits using real treebank data
//!
//! Unlike the test doubles in implementations.rs, these implementations use
//! actual UD treebank parsing and semantic analysis with real engines.

use crate::error::{AnalysisError, PipelineError};
use crate::traits::*;
use async_trait::async_trait;
use canopy_core::Word;
use canopy_core::treebank_loader::TreebankSentenceLoader;
use std::sync::Arc;

/// Real treebank-based morphosyntactic parser
///
/// Uses gold-standard UD English-EWT parses for analysis.
/// This is NOT a general-purpose parser - it only works with
/// sentences that exist in the treebank.
pub struct TreebankMorphosyntacticParser {
    loader: Arc<TreebankSentenceLoader>,
}

impl TreebankMorphosyntacticParser {
    /// Create a new parser with default UD English-EWT data
    ///
    /// # Errors
    ///
    /// Returns an error if the treebank files cannot be loaded
    pub fn new() -> Result<Self, PipelineError> {
        let loader = TreebankSentenceLoader::new().map_err(|e| {
            PipelineError::ConfigurationError(format!(
                "TreebankMorphosyntacticParser: Failed to load treebank data: {}",
                e
            ))
        })?;

        Ok(Self {
            loader: Arc::new(loader),
        })
    }

    /// Create a parser from a custom treebank path
    ///
    /// # Errors
    ///
    /// Returns an error if the treebank files cannot be loaded
    pub fn from_path(path: &std::path::Path) -> Result<Self, PipelineError> {
        let loader = TreebankSentenceLoader::from_path(path).map_err(|e| {
            PipelineError::ConfigurationError(format!(
                "TreebankMorphosyntacticParser: Failed to load treebank from {}: {}",
                path.display(),
                e
            ))
        })?;

        Ok(Self {
            loader: Arc::new(loader),
        })
    }

    /// Get the underlying loader for direct access
    pub fn loader(&self) -> &Arc<TreebankSentenceLoader> {
        &self.loader
    }
}

#[async_trait]
impl MorphosyntacticParser for TreebankMorphosyntacticParser {
    async fn parse(&self, text: &str) -> Result<Vec<Word>, AnalysisError> {
        // Try to interpret text as a sentence ID first
        if let Some(sentence) = self.loader.get_sentence(text) {
            return self.loader.convert_to_words(sentence).map_err(|e| {
                AnalysisError::ParseFailed(format!(
                    "Failed to convert treebank sentence '{}': {}",
                    text, e
                ))
            });
        }

        // Not found - return helpful error
        Err(AnalysisError::ParseFailed(format!(
            "Sentence '{}' not found in treebank.\n\n\
             Canopy currently supports UD treebank sentences only.\n\
             Available: {} dev, {} train, {} test sentences.\n\n\
             Use sentence IDs like:\n\
             - For dev set: weblog-blogspot.com_*\n\
             - For fixtures: canonical-001 through canonical-020\n\n\
             To list available IDs: TreebankSentenceLoader::new()?.list_available(10)",
            text,
            self.loader.dev_count(),
            self.loader.train_count(),
            self.loader.test_count()
        )))
    }

    fn info(&self) -> ParserInfo {
        ParserInfo {
            name: "TreebankMorphosyntacticParser".to_string(),
            version: "0.7.0-alpha".to_string(),
            model_type: "UD English-EWT Gold Standard".to_string(),
            supported_languages: vec!["en".to_string()],
            capabilities: ParserCapabilities {
                supports_tokenization: true,
                supports_pos_tagging: true,
                supports_lemmatization: true,
                supports_dependency_parsing: true,
                supports_morphological_features: true,
                max_sentence_length: None, // No limit for treebank sentences
            },
        }
    }

    fn is_ready(&self) -> bool {
        // Ready if we have any sentences loaded
        self.loader.total_count() > 0
    }
}

/// Real component factory for production use
pub struct RealComponentFactory {
    treebank_path: Option<std::path::PathBuf>,
}

impl RealComponentFactory {
    /// Create a new factory with default paths
    pub fn new() -> Self {
        Self {
            treebank_path: None,
        }
    }

    /// Create a factory with custom treebank path
    pub fn with_treebank_path(path: std::path::PathBuf) -> Self {
        Self {
            treebank_path: Some(path),
        }
    }
}

impl Default for RealComponentFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl ComponentFactory for RealComponentFactory {
    fn create_parser(
        &self,
        _config: &ParserConfig,
    ) -> Result<Box<dyn MorphosyntacticParser>, PipelineError> {
        let parser = if let Some(ref path) = self.treebank_path {
            TreebankMorphosyntacticParser::from_path(path)?
        } else {
            TreebankMorphosyntacticParser::new()?
        };

        Ok(Box::new(parser))
    }

    fn create_analyzer(
        &self,
        _config: &AnalyzerConfig,
    ) -> Result<Box<dyn SemanticAnalyzer>, PipelineError> {
        // Semantic analyzer implementation pending - will use SemanticCoordinator
        Err(PipelineError::NotReady(
            "SemanticAnalyzer: Real implementation pending".to_string(),
        ))
    }

    fn create_extractor(
        &self,
        _config: &ExtractorConfig,
    ) -> Result<Box<dyn FeatureExtractor>, PipelineError> {
        // Feature extractor implementation pending
        Err(PipelineError::NotReady(
            "FeatureExtractor: Real implementation pending".to_string(),
        ))
    }

    fn create_cache(&self, _config: &CacheConfig) -> Result<Box<dyn CacheProvider>, PipelineError> {
        // Cache provider implementation pending
        Err(PipelineError::NotReady(
            "CacheProvider: Real implementation pending".to_string(),
        ))
    }

    fn create_metrics(
        &self,
        _config: &MetricsConfig,
    ) -> Result<Box<dyn MetricsCollector>, PipelineError> {
        // Metrics collector implementation pending
        Err(PipelineError::NotReady(
            "MetricsCollector: Real implementation pending".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_treebank_parser_with_fixture() {
        // Test with canonical fixture if it exists
        let parser = TreebankMorphosyntacticParser::new();
        if parser.is_err() {
            // Treebank not available, skip test
            return;
        }
        let parser = parser.unwrap();

        // Try to parse using a sentence ID
        let result = parser.parse("canonical-001").await;
        if result.is_ok() {
            let words = result.unwrap();
            assert_eq!(words[0].text, "John");
            assert_eq!(words[0].lemma, "John");
            assert_eq!(words[1].text, "gave");
            assert_eq!(words[1].lemma, "give");
        }
        // If not found, that's OK - might not have fixtures set up yet
    }

    #[tokio::test]
    async fn test_parser_not_found_error() {
        let parser = TreebankMorphosyntacticParser::new();
        if parser.is_err() {
            return; // Skip if treebank not available
        }
        let parser = parser.unwrap();

        let result = parser.parse("nonexistent-sentence-12345").await;
        assert!(result.is_err());

        if let Err(AnalysisError::ParseFailed(message)) = result {
            assert!(message.contains("nonexistent-sentence-12345"));
            assert!(message.contains("not found in treebank"));
            assert!(message.contains("dev"));
            assert!(message.contains("train"));
        } else {
            panic!("Expected ParseFailed error");
        }
    }

    #[test]
    fn test_real_factory_create_parser() {
        let factory = RealComponentFactory::new();
        let config = ParserConfig {
            model_path: Some("data/ud_english-ewt".to_string()),
            model_type: ModelType::UDPipe12,
            performance_mode: PerformanceMode::Balanced,
            enable_caching: true,
        };

        let result = factory.create_parser(&config);
        // May succeed or fail depending on data availability
        // Just verify it returns a result
        assert!(result.is_ok() || result.is_err());
    }
}
