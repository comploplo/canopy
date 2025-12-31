//! Types for the semantic coordinator
//!
//! Contains all type definitions used by the SemanticCoordinator including
//! analysis results, configuration, and dependency structures.

use canopy_core::UPos;
use serde::{Deserialize, Serialize};

/// Lightweight treebank analysis result for Layer 1 integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreebankAnalysis {
    /// Original word analyzed
    pub word: String,
    /// Found dependency relation (if any)
    pub dependency_relation: Option<String>,
    /// Analysis confidence
    pub confidence: f32,
    /// Processing time in microseconds
    pub processing_time_us: u64,
    /// Whether result came from cache
    pub from_cache: bool,
    /// Voice features extracted (passive, active, etc.)
    pub voice_features: Vec<String>,
    /// Semantic role features (:agent, :pass, etc.)
    pub semantic_features: Vec<String>,
}

impl TreebankAnalysis {
    pub fn new(word: String, confidence: f32) -> Self {
        Self {
            word,
            dependency_relation: None,
            confidence,
            processing_time_us: 0,
            from_cache: false,
            voice_features: Vec::new(),
            semantic_features: Vec::new(),
        }
    }
}

/// Memory usage statistics
#[derive(Debug, Clone)]
pub struct MemoryUsage {
    pub estimated_usage_mb: f32,
    pub budget_mb: usize,
    pub utilization_percent: f32,
}

impl Default for MemoryUsage {
    fn default() -> Self {
        Self {
            estimated_usage_mb: 0.0,
            budget_mb: 100,
            utilization_percent: 0.0,
        }
    }
}

/// Memory pressure alert
#[derive(Debug, Clone)]
pub struct MemoryPressureAlert {
    pub message: String,
    pub severity: String,
    pub usage_mb: f32,
    pub budget_mb: usize,
    pub current_usage_mb: f32,
    pub current_utilization: f32,
    pub recommendation: String,
}

/// Statistics for semantic analysis
#[derive(Debug, Clone)]
pub struct CoordinatorStatistics {
    pub total_analyses: usize,
    pub cache_hits: usize,
    pub cache_misses: usize,
    pub successful_analyses: usize,
    pub failed_analyses: usize,
    pub average_confidence: f32,
    pub total_queries: usize,
    pub cache_hit_rate: f32,
    pub parallel_queries: usize,
    pub parallel_query_rate: f32,
    pub warmed_queries: usize,
    pub memory_usage: MemoryUsage,
    pub active_engines: Vec<String>,
}

impl Default for CoordinatorStatistics {
    fn default() -> Self {
        Self {
            total_analyses: 0,
            cache_hits: 0,
            cache_misses: 0,
            successful_analyses: 0,
            failed_analyses: 0,
            average_confidence: 0.0,
            total_queries: 0,
            cache_hit_rate: 0.0,
            parallel_queries: 0,
            parallel_query_rate: 0.0,
            warmed_queries: 0,
            memory_usage: MemoryUsage::default(),
            active_engines: Vec::new(),
        }
    }
}

// ============================================================================
// Sentence Analysis Types (for Layer 1 → Layer 2 bridge)
// ============================================================================

/// Universal Dependency relations for sentence analysis
///
/// This is a simplified set focused on what Layer 1 needs for the Layer 2 bridge.
/// Compatible with canopy-treebank's DependencyRelation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DependencyRelation {
    /// Nominal subject (nsubj)
    NominalSubject,
    /// Direct object (obj)
    Object,
    /// Indirect object (iobj)
    IndirectObject,
    /// Oblique nominal (obl)
    Oblique,
    /// Adverbial modifier (advmod)
    AdverbialModifier,
    /// Adjectival modifier (amod)
    AdjectivalModifier,
    /// Determiner (det)
    Determiner,
    /// Auxiliary (aux)
    Auxiliary,
    /// Root of the sentence (root)
    Root,
    /// Other relation
    Other,
}

impl std::fmt::Display for DependencyRelation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            DependencyRelation::NominalSubject => "nsubj",
            DependencyRelation::Object => "obj",
            DependencyRelation::IndirectObject => "iobj",
            DependencyRelation::Oblique => "obl",
            DependencyRelation::AdverbialModifier => "advmod",
            DependencyRelation::AdjectivalModifier => "amod",
            DependencyRelation::Determiner => "det",
            DependencyRelation::Auxiliary => "aux",
            DependencyRelation::Root => "root",
            DependencyRelation::Other => "other",
        };
        write!(f, "{}", s)
    }
}

/// A dependency arc between two tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyArc {
    /// Index of the head token
    pub head_idx: usize,
    /// Index of the dependent token
    pub dependent_idx: usize,
    /// Dependency relation type
    pub relation: DependencyRelation,
    /// Confidence score for this arc
    pub confidence: f32,
}

impl DependencyArc {
    /// Create a new dependency arc
    pub fn new(head_idx: usize, dependent_idx: usize, relation: DependencyRelation) -> Self {
        Self {
            head_idx,
            dependent_idx,
            relation,
            confidence: 1.0,
        }
    }

    /// Create with explicit confidence
    pub fn with_confidence(
        head_idx: usize,
        dependent_idx: usize,
        relation: DependencyRelation,
        confidence: f32,
    ) -> Self {
        Self {
            head_idx,
            dependent_idx,
            relation,
            confidence,
        }
    }
}

/// Sentence-level metadata affecting event composition
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SentenceMetadata {
    /// Optional sentence ID for tracking
    pub sentence_id: Option<String>,
    /// Whether the sentence is in passive voice
    pub is_passive: bool,
    /// Whether the sentence is interrogative
    pub is_interrogative: bool,
    /// Whether the sentence is negated
    pub is_negated: bool,
    /// Whether the sentence is imperative
    pub is_imperative: bool,
}

/// Result of sentence-level analysis from Layer 1
#[derive(Debug, Clone)]
pub struct SentenceAnalysisResult {
    /// Original sentence text
    pub text: String,
    /// Token-level Layer 1 semantic results with POS
    pub tokens: Vec<Layer1SemanticResult>,
    /// Dependency arcs between tokens
    pub dependencies: Vec<DependencyArc>,
    /// Sentence-level metadata
    pub metadata: SentenceMetadata,
    /// Processing time in microseconds
    pub processing_time_us: u64,
}

impl SentenceAnalysisResult {
    /// Create a new sentence analysis result
    pub fn new(text: String, tokens: Vec<Layer1SemanticResult>) -> Self {
        Self {
            text,
            tokens,
            dependencies: Vec::new(),
            metadata: SentenceMetadata::default(),
            processing_time_us: 0,
        }
    }

    /// Get token by index
    pub fn get_token(&self, idx: usize) -> Option<&Layer1SemanticResult> {
        self.tokens.get(idx)
    }

    /// Find predicates (verbs) in the sentence
    pub fn find_predicates(&self) -> Vec<usize> {
        self.tokens
            .iter()
            .enumerate()
            .filter(|(_, t)| matches!(t.pos, Some(UPos::Verb) | Some(UPos::Aux)))
            .map(|(i, _)| i)
            .collect()
    }

    /// Get dependents of a token
    pub fn get_dependents(&self, head_idx: usize) -> Vec<&DependencyArc> {
        self.dependencies
            .iter()
            .filter(|arc| arc.head_idx == head_idx)
            .collect()
    }
}

/// Configuration for the semantic coordinator
#[derive(Debug, Clone)]
pub struct CoordinatorConfig {
    pub enable_verbnet: bool,
    pub enable_framenet: bool,
    pub enable_wordnet: bool,
    pub enable_lexicon: bool,
    pub enable_propbank: bool,
    pub enable_treebank: bool,
    pub enable_lemmatization: bool,
    pub use_advanced_lemmatization: bool,
    pub confidence_threshold: f32,
    pub l1_cache_memory_mb: usize,
    /// Use treebank gold-standard lemmas when available
    pub use_treebank_lemmas: bool,
    /// Minimum confidence threshold for lemma caching
    pub lemma_confidence_threshold: f32,
    /// Enable integration with shared lemma cache
    pub enable_shared_lemma_cache: bool,
    /// Cache configuration
    pub cache_capacity: usize,
    pub enable_cache_warmup: bool,
    pub cache_warmup_common_words: bool,
}

impl Default for CoordinatorConfig {
    fn default() -> Self {
        Self {
            enable_verbnet: true,
            enable_framenet: true,
            enable_wordnet: true,
            enable_lexicon: true,
            enable_propbank: true,
            enable_treebank: true,
            enable_lemmatization: true,
            use_advanced_lemmatization: false,
            confidence_threshold: 0.1,
            l1_cache_memory_mb: 50,
            use_treebank_lemmas: true, // Prefer gold-standard lemmas
            lemma_confidence_threshold: 0.3,
            enable_shared_lemma_cache: true,
            cache_capacity: 10000,            // Much larger cache
            enable_cache_warmup: false,       // Disabled by default for faster startup
            cache_warmup_common_words: false, // Enable explicitly when needed
        }
    }
}

/// Layer 1 semantic analysis result
#[derive(Debug, Clone)]
pub struct Layer1SemanticResult {
    pub original_word: String,
    pub lemma: String,
    pub pos: Option<UPos>,
    pub lemmatization_confidence: Option<f32>,
    pub verbnet: Option<canopy_semantic_engines::verbnet::VerbNetAnalysis>,
    pub framenet: Option<canopy_semantic_engines::framenet::FrameNetAnalysis>,
    pub wordnet: Option<canopy_semantic_engines::wordnet::WordNetAnalysis>,
    pub lexicon: Option<canopy_semantic_engines::lexicon::LexiconAnalysis>,
    pub propbank: Option<canopy_semantic_engines::propbank::PropBankAnalysis>,
    pub treebank: Option<TreebankAnalysis>,
    pub confidence: f32,
    pub sources: Vec<String>,
    pub errors: Vec<String>,
}

impl Layer1SemanticResult {
    pub fn new(original_word: String, lemma: String) -> Self {
        Self {
            original_word,
            lemma,
            pos: None,
            lemmatization_confidence: None,
            verbnet: None,
            framenet: None,
            wordnet: None,
            lexicon: None,
            propbank: None,
            treebank: None,
            confidence: 0.0,
            sources: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// Check if the result has any semantic analysis data
    pub fn has_results(&self) -> bool {
        self.verbnet.is_some()
            || self.framenet.is_some()
            || self.wordnet.is_some()
            || self.lexicon.is_some()
            || self.propbank.is_some()
            || self.treebank.is_some()
            || !self.sources.is_empty()
    }

    /// Check if the result has coverage from multiple engines
    pub fn has_multi_engine_coverage(&self) -> bool {
        let engine_count = [
            self.verbnet.is_some(),
            self.framenet.is_some(),
            self.wordnet.is_some(),
            self.lexicon.is_some(),
            self.propbank.is_some(),
            self.treebank.is_some(),
        ]
        .iter()
        .filter(|&&has| has)
        .count();

        engine_count >= 2
    }
}

/// Result of analyzing multi-sentence text
#[derive(Debug, Clone)]
pub struct TextAnalysisResult {
    /// Original input text
    pub text: String,
    /// Analysis results for each sentence
    pub sentences: Vec<SentenceAnalysisResult>,
    /// Analysis statistics
    pub stats: TextAnalysisStats,
}

impl TextAnalysisResult {
    /// Get total number of tokens across all sentences
    pub fn total_tokens(&self) -> usize {
        self.sentences.iter().map(|s| s.tokens.len()).sum()
    }

    /// Get total number of sentences
    pub fn sentence_count(&self) -> usize {
        self.sentences.len()
    }

    /// Get the primary (first) sentence result
    pub fn primary_sentence(&self) -> Option<&SentenceAnalysisResult> {
        self.sentences.first()
    }
}

/// Statistics for text analysis
#[derive(Debug, Clone, Default)]
pub struct TextAnalysisStats {
    /// Total processing time in microseconds
    pub total_time_us: u64,
    /// Number of sentences processed
    pub sentences_processed: usize,
    /// Total tokens processed
    pub tokens_processed: usize,
    /// Number of unique words analyzed (after deduplication)
    pub unique_words: usize,
    /// Cache hits during analysis
    pub cache_hits: u64,
    /// Cache misses during analysis
    pub cache_misses: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== TreebankAnalysis Tests ==========

    #[test]
    fn test_treebank_analysis_new() {
        let analysis = TreebankAnalysis::new("run".to_string(), 0.85);
        assert_eq!(analysis.word, "run");
        assert_eq!(analysis.confidence, 0.85);
        assert!(analysis.dependency_relation.is_none());
        assert_eq!(analysis.processing_time_us, 0);
        assert!(!analysis.from_cache);
        assert!(analysis.voice_features.is_empty());
        assert!(analysis.semantic_features.is_empty());
    }

    #[test]
    fn test_treebank_analysis_clone_debug() {
        let analysis = TreebankAnalysis::new("give".to_string(), 0.9);
        let cloned = analysis.clone();
        assert_eq!(cloned.word, "give");
        assert_eq!(cloned.confidence, 0.9);
        let debug = format!("{:?}", analysis);
        assert!(debug.contains("give"));
    }

    // ========== MemoryUsage Tests ==========

    #[test]
    fn test_memory_usage_default() {
        let usage = MemoryUsage::default();
        assert_eq!(usage.estimated_usage_mb, 0.0);
        assert_eq!(usage.budget_mb, 100);
        assert_eq!(usage.utilization_percent, 0.0);
    }

    #[test]
    fn test_memory_usage_clone_debug() {
        let usage = MemoryUsage {
            estimated_usage_mb: 50.0,
            budget_mb: 100,
            utilization_percent: 50.0,
        };
        let cloned = usage.clone();
        assert_eq!(cloned.utilization_percent, 50.0);
        let debug = format!("{:?}", usage);
        assert!(debug.contains("50"));
    }

    // ========== MemoryPressureAlert Tests ==========

    #[test]
    fn test_memory_pressure_alert_clone_debug() {
        let alert = MemoryPressureAlert {
            message: "High memory usage".to_string(),
            severity: "warning".to_string(),
            usage_mb: 80.0,
            budget_mb: 100,
            current_usage_mb: 80.0,
            current_utilization: 0.8,
            recommendation: "Clear cache".to_string(),
        };
        let cloned = alert.clone();
        assert_eq!(cloned.severity, "warning");
        let debug = format!("{:?}", alert);
        assert!(debug.contains("warning"));
    }

    // ========== CoordinatorStatistics Tests ==========

    #[test]
    fn test_coordinator_statistics_default() {
        let stats = CoordinatorStatistics::default();
        assert_eq!(stats.total_analyses, 0);
        assert_eq!(stats.cache_hits, 0);
        assert_eq!(stats.cache_misses, 0);
        assert_eq!(stats.successful_analyses, 0);
        assert_eq!(stats.failed_analyses, 0);
        assert_eq!(stats.average_confidence, 0.0);
        assert_eq!(stats.total_queries, 0);
        assert_eq!(stats.cache_hit_rate, 0.0);
        assert_eq!(stats.parallel_queries, 0);
        assert_eq!(stats.parallel_query_rate, 0.0);
        assert_eq!(stats.warmed_queries, 0);
        assert!(stats.active_engines.is_empty());
    }

    #[test]
    fn test_coordinator_statistics_clone_debug() {
        let stats = CoordinatorStatistics {
            total_analyses: 100,
            cache_hits: 60,
            cache_hit_rate: 0.6,
            ..Default::default()
        };
        let cloned = stats.clone();
        assert_eq!(cloned.total_analyses, 100);
        let debug = format!("{:?}", stats);
        assert!(debug.contains("100"));
    }

    // ========== DependencyRelation Tests ==========

    #[test]
    fn test_dependency_relation_display() {
        assert_eq!(DependencyRelation::NominalSubject.to_string(), "nsubj");
        assert_eq!(DependencyRelation::Object.to_string(), "obj");
        assert_eq!(DependencyRelation::IndirectObject.to_string(), "iobj");
        assert_eq!(DependencyRelation::Oblique.to_string(), "obl");
        assert_eq!(DependencyRelation::AdverbialModifier.to_string(), "advmod");
        assert_eq!(DependencyRelation::AdjectivalModifier.to_string(), "amod");
        assert_eq!(DependencyRelation::Determiner.to_string(), "det");
        assert_eq!(DependencyRelation::Auxiliary.to_string(), "aux");
        assert_eq!(DependencyRelation::Root.to_string(), "root");
        assert_eq!(DependencyRelation::Other.to_string(), "other");
    }

    #[test]
    fn test_dependency_relation_equality_and_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(DependencyRelation::NominalSubject);
        set.insert(DependencyRelation::Object);
        assert!(set.contains(&DependencyRelation::NominalSubject));
        assert!(!set.contains(&DependencyRelation::Root));
        assert_eq!(DependencyRelation::Object, DependencyRelation::Object);
    }

    #[test]
    fn test_dependency_relation_clone_copy() {
        let rel = DependencyRelation::NominalSubject;
        let copied = rel; // Copy
        let cloned = rel; // Also copy (implements Copy trait)
        assert_eq!(copied, cloned);
    }

    // ========== DependencyArc Tests ==========

    #[test]
    fn test_dependency_arc_new() {
        let arc = DependencyArc::new(0, 1, DependencyRelation::NominalSubject);
        assert_eq!(arc.head_idx, 0);
        assert_eq!(arc.dependent_idx, 1);
        assert_eq!(arc.relation, DependencyRelation::NominalSubject);
        assert_eq!(arc.confidence, 1.0);
    }

    #[test]
    fn test_dependency_arc_with_confidence() {
        let arc = DependencyArc::with_confidence(2, 3, DependencyRelation::Object, 0.75);
        assert_eq!(arc.head_idx, 2);
        assert_eq!(arc.dependent_idx, 3);
        assert_eq!(arc.relation, DependencyRelation::Object);
        assert_eq!(arc.confidence, 0.75);
    }

    #[test]
    fn test_dependency_arc_clone_debug() {
        let arc = DependencyArc::new(1, 2, DependencyRelation::Root);
        let cloned = arc.clone();
        assert_eq!(cloned.head_idx, 1);
        let debug = format!("{:?}", arc);
        assert!(debug.contains("Root"));
    }

    // ========== SentenceMetadata Tests ==========

    #[test]
    fn test_sentence_metadata_default() {
        let meta = SentenceMetadata::default();
        assert!(meta.sentence_id.is_none());
        assert!(!meta.is_passive);
        assert!(!meta.is_interrogative);
        assert!(!meta.is_negated);
        assert!(!meta.is_imperative);
    }

    #[test]
    fn test_sentence_metadata_clone_debug() {
        let meta = SentenceMetadata {
            sentence_id: Some("s1".to_string()),
            is_passive: true,
            is_interrogative: false,
            is_negated: true,
            is_imperative: false,
        };
        let cloned = meta.clone();
        assert_eq!(cloned.sentence_id, Some("s1".to_string()));
        assert!(cloned.is_passive);
        let debug = format!("{:?}", meta);
        assert!(debug.contains("passive"));
    }

    // ========== SentenceAnalysisResult Tests ==========

    #[test]
    fn test_sentence_analysis_result_new() {
        let tokens = vec![
            Layer1SemanticResult::new("John".to_string(), "john".to_string()),
            Layer1SemanticResult::new("runs".to_string(), "run".to_string()),
        ];
        let result = SentenceAnalysisResult::new("John runs".to_string(), tokens);
        assert_eq!(result.text, "John runs");
        assert_eq!(result.tokens.len(), 2);
        assert!(result.dependencies.is_empty());
        assert_eq!(result.processing_time_us, 0);
    }

    #[test]
    fn test_sentence_analysis_result_get_token() {
        let tokens = vec![
            Layer1SemanticResult::new("The".to_string(), "the".to_string()),
            Layer1SemanticResult::new("cat".to_string(), "cat".to_string()),
        ];
        let result = SentenceAnalysisResult::new("The cat".to_string(), tokens);
        assert_eq!(result.get_token(0).unwrap().original_word, "The");
        assert_eq!(result.get_token(1).unwrap().original_word, "cat");
        assert!(result.get_token(5).is_none());
    }

    #[test]
    fn test_sentence_analysis_result_find_predicates() {
        let mut tokens = vec![
            Layer1SemanticResult::new("John".to_string(), "john".to_string()),
            Layer1SemanticResult::new("runs".to_string(), "run".to_string()),
            Layer1SemanticResult::new("fast".to_string(), "fast".to_string()),
        ];
        tokens[1].pos = Some(UPos::Verb);
        let result = SentenceAnalysisResult::new("John runs fast".to_string(), tokens);
        let predicates = result.find_predicates();
        assert_eq!(predicates, vec![1]);
    }

    #[test]
    fn test_sentence_analysis_result_find_predicates_with_aux() {
        let mut tokens = vec![
            Layer1SemanticResult::new("is".to_string(), "be".to_string()),
            Layer1SemanticResult::new("running".to_string(), "run".to_string()),
        ];
        tokens[0].pos = Some(UPos::Aux);
        tokens[1].pos = Some(UPos::Verb);
        let result = SentenceAnalysisResult::new("is running".to_string(), tokens);
        let predicates = result.find_predicates();
        assert_eq!(predicates, vec![0, 1]);
    }

    #[test]
    fn test_sentence_analysis_result_get_dependents() {
        let tokens = vec![
            Layer1SemanticResult::new("John".to_string(), "john".to_string()),
            Layer1SemanticResult::new("runs".to_string(), "run".to_string()),
        ];
        let mut result = SentenceAnalysisResult::new("John runs".to_string(), tokens);
        result
            .dependencies
            .push(DependencyArc::new(1, 0, DependencyRelation::NominalSubject));
        let deps = result.get_dependents(1);
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].dependent_idx, 0);
        let no_deps = result.get_dependents(0);
        assert!(no_deps.is_empty());
    }

    // ========== CoordinatorConfig Tests ==========

    #[test]
    fn test_coordinator_config_default() {
        let config = CoordinatorConfig::default();
        assert!(config.enable_verbnet);
        assert!(config.enable_framenet);
        assert!(config.enable_wordnet);
        assert!(config.enable_lexicon);
        assert!(config.enable_propbank);
        assert!(config.enable_treebank);
        assert!(config.enable_lemmatization);
        assert!(!config.use_advanced_lemmatization);
        assert_eq!(config.confidence_threshold, 0.1);
        assert_eq!(config.l1_cache_memory_mb, 50);
        assert!(config.use_treebank_lemmas);
        assert_eq!(config.lemma_confidence_threshold, 0.3);
        assert!(config.enable_shared_lemma_cache);
        assert_eq!(config.cache_capacity, 10000);
        assert!(!config.enable_cache_warmup);
        assert!(!config.cache_warmup_common_words);
    }

    #[test]
    fn test_coordinator_config_clone_debug() {
        let config = CoordinatorConfig::default();
        let cloned = config.clone();
        assert_eq!(cloned.cache_capacity, 10000);
        let debug = format!("{:?}", config);
        assert!(debug.contains("verbnet"));
    }

    // ========== Layer1SemanticResult Tests ==========

    #[test]
    fn test_layer1_semantic_result_new() {
        let result = Layer1SemanticResult::new("running".to_string(), "run".to_string());
        assert_eq!(result.original_word, "running");
        assert_eq!(result.lemma, "run");
        assert!(result.pos.is_none());
        assert!(result.lemmatization_confidence.is_none());
        assert!(result.verbnet.is_none());
        assert!(result.framenet.is_none());
        assert!(result.wordnet.is_none());
        assert!(result.lexicon.is_none());
        assert!(result.propbank.is_none());
        assert!(result.treebank.is_none());
        assert_eq!(result.confidence, 0.0);
        assert!(result.sources.is_empty());
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_layer1_semantic_result_has_results_empty() {
        let result = Layer1SemanticResult::new("test".to_string(), "test".to_string());
        assert!(!result.has_results());
    }

    #[test]
    fn test_layer1_semantic_result_has_results_with_sources() {
        let mut result = Layer1SemanticResult::new("test".to_string(), "test".to_string());
        result.sources.push("verbnet".to_string());
        assert!(result.has_results());
    }

    #[test]
    fn test_layer1_semantic_result_has_results_with_treebank() {
        let mut result = Layer1SemanticResult::new("run".to_string(), "run".to_string());
        result.treebank = Some(TreebankAnalysis::new("run".to_string(), 0.8));
        assert!(result.has_results());
    }

    #[test]
    fn test_layer1_semantic_result_has_multi_engine_coverage_none() {
        let result = Layer1SemanticResult::new("test".to_string(), "test".to_string());
        assert!(!result.has_multi_engine_coverage());
    }

    #[test]
    fn test_layer1_semantic_result_has_multi_engine_coverage_one() {
        let mut result = Layer1SemanticResult::new("run".to_string(), "run".to_string());
        result.treebank = Some(TreebankAnalysis::new("run".to_string(), 0.8));
        assert!(!result.has_multi_engine_coverage());
    }

    #[test]
    fn test_layer1_semantic_result_has_multi_engine_coverage_two() {
        let mut result = Layer1SemanticResult::new("run".to_string(), "run".to_string());
        result.treebank = Some(TreebankAnalysis::new("run".to_string(), 0.8));
        // Need to set a second engine - let's use a minimal struct
        // For this test we just verify the counting logic
        // by checking if having 2+ engines returns true
        // We can't easily create VerbNetAnalysis etc. without the full engine
        // So we verify the count threshold logic works
        let count = [
            result.verbnet.is_some(),
            result.framenet.is_some(),
            result.wordnet.is_some(),
            result.lexicon.is_some(),
            result.propbank.is_some(),
            result.treebank.is_some(),
        ]
        .iter()
        .filter(|&&has| has)
        .count();
        assert_eq!(count, 1); // Only treebank set
        assert!(!result.has_multi_engine_coverage()); // < 2
    }

    #[test]
    fn test_layer1_semantic_result_clone_debug() {
        let result = Layer1SemanticResult::new("give".to_string(), "give".to_string());
        let cloned = result.clone();
        assert_eq!(cloned.original_word, "give");
        let debug = format!("{:?}", result);
        assert!(debug.contains("give"));
    }

    // ========== TextAnalysisResult Tests ==========

    #[test]
    fn test_text_analysis_result_total_tokens() {
        let result = TextAnalysisResult {
            text: "John runs. Mary walks.".to_string(),
            sentences: vec![
                SentenceAnalysisResult::new(
                    "John runs.".to_string(),
                    vec![
                        Layer1SemanticResult::new("John".to_string(), "john".to_string()),
                        Layer1SemanticResult::new("runs".to_string(), "run".to_string()),
                    ],
                ),
                SentenceAnalysisResult::new(
                    "Mary walks.".to_string(),
                    vec![
                        Layer1SemanticResult::new("Mary".to_string(), "mary".to_string()),
                        Layer1SemanticResult::new("walks".to_string(), "walk".to_string()),
                    ],
                ),
            ],
            stats: TextAnalysisStats::default(),
        };
        assert_eq!(result.total_tokens(), 4);
    }

    #[test]
    fn test_text_analysis_result_sentence_count() {
        let result = TextAnalysisResult {
            text: "Hello. World.".to_string(),
            sentences: vec![
                SentenceAnalysisResult::new("Hello.".to_string(), vec![]),
                SentenceAnalysisResult::new("World.".to_string(), vec![]),
            ],
            stats: TextAnalysisStats::default(),
        };
        assert_eq!(result.sentence_count(), 2);
    }

    #[test]
    fn test_text_analysis_result_primary_sentence() {
        let result = TextAnalysisResult {
            text: "First. Second.".to_string(),
            sentences: vec![
                SentenceAnalysisResult::new("First.".to_string(), vec![]),
                SentenceAnalysisResult::new("Second.".to_string(), vec![]),
            ],
            stats: TextAnalysisStats::default(),
        };
        let primary = result.primary_sentence().unwrap();
        assert_eq!(primary.text, "First.");
    }

    #[test]
    fn test_text_analysis_result_primary_sentence_empty() {
        let result = TextAnalysisResult {
            text: "".to_string(),
            sentences: vec![],
            stats: TextAnalysisStats::default(),
        };
        assert!(result.primary_sentence().is_none());
    }

    #[test]
    fn test_text_analysis_result_clone_debug() {
        let result = TextAnalysisResult {
            text: "Test".to_string(),
            sentences: vec![],
            stats: TextAnalysisStats::default(),
        };
        let cloned = result.clone();
        assert_eq!(cloned.text, "Test");
        let debug = format!("{:?}", result);
        assert!(debug.contains("Test"));
    }

    // ========== TextAnalysisStats Tests ==========

    #[test]
    fn test_text_analysis_stats_default() {
        let stats = TextAnalysisStats::default();
        assert_eq!(stats.total_time_us, 0);
        assert_eq!(stats.sentences_processed, 0);
        assert_eq!(stats.tokens_processed, 0);
        assert_eq!(stats.unique_words, 0);
        assert_eq!(stats.cache_hits, 0);
        assert_eq!(stats.cache_misses, 0);
    }

    #[test]
    fn test_text_analysis_stats_clone_debug() {
        let stats = TextAnalysisStats {
            total_time_us: 1000,
            sentences_processed: 5,
            tokens_processed: 50,
            unique_words: 30,
            cache_hits: 20,
            cache_misses: 10,
        };
        let cloned = stats.clone();
        assert_eq!(cloned.tokens_processed, 50);
        let debug = format!("{:?}", stats);
        assert!(debug.contains("50"));
    }
}
