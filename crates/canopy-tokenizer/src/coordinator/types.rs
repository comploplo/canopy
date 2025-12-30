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
