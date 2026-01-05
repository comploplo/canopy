//! Pipeline configuration options.

/// Configuration for the semantic analysis pipeline.
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    /// Whether to enable discourse processing across sentences.
    pub enable_discourse: bool,
    /// Whether to use treebank pattern matching for syntax.
    pub use_treebank_patterns: bool,
    /// Maximum number of sentences to process (None = unlimited).
    pub max_sentences: Option<usize>,
    /// Confidence threshold for predicate decomposition.
    pub decomposition_confidence_threshold: f32,
    /// Confidence threshold for role binding.
    pub role_binding_confidence_threshold: f32,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            enable_discourse: true,
            use_treebank_patterns: true,
            max_sentences: None,
            decomposition_confidence_threshold: 0.5,
            role_binding_confidence_threshold: 0.5,
        }
    }
}

impl PipelineConfig {
    /// Create a minimal configuration for testing.
    #[must_use]
    pub fn minimal() -> Self {
        Self {
            enable_discourse: false,
            use_treebank_patterns: false,
            max_sentences: Some(10),
            decomposition_confidence_threshold: 0.3,
            role_binding_confidence_threshold: 0.3,
        }
    }

    /// Create a full configuration for production use.
    #[must_use]
    pub fn full() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = PipelineConfig::default();
        assert!(config.enable_discourse);
        assert!(config.use_treebank_patterns);
        assert!(config.max_sentences.is_none());
    }

    #[test]
    fn test_minimal_config() {
        let config = PipelineConfig::minimal();
        assert!(!config.enable_discourse);
        assert_eq!(config.max_sentences, Some(10));
    }
}
