//! Semantic signature generation for pattern matching
//!
//! This module creates hash-based signatures from Layer 1 semantic analysis
//! results to enable efficient pattern lookup and matching.

use ahash::AHasher;
use canopy_engine::LemmaSource;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use tracing::debug;

/// Semantic signature for pattern matching
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SemanticSignature {
    /// Lemmatized word
    pub lemma: String,
    /// VerbNet class (if available)
    pub verbnet_class: Option<String>,
    /// FrameNet frame (if available)
    pub framenet_frame: Option<String>,
    /// Simplified POS category
    pub pos_category: PosCategory,
    /// Source of the lemmatization
    pub lemma_source: LemmaSource,
    /// Confidence in the lemmatization (0.0-1.0)
    pub lemma_confidence: f32,
    /// Hash code for efficient lookup
    pub hash_code: u64,
}

impl Hash for SemanticSignature {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.lemma.hash(state);
        self.verbnet_class.hash(state);
        self.framenet_frame.hash(state);
        self.pos_category.hash(state);
        // Skip f32 confidence field and LemmaSource for hashing
        self.hash_code.hash(state);
    }
}

impl Eq for SemanticSignature {}

/// Simplified POS categories for signature matching
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PosCategory {
    Verb,
    Noun,
    Adjective,
    Adverb,
    Other,
}

impl SemanticSignature {
    /// Create a new semantic signature
    pub fn new(
        lemma: String,
        verbnet_class: Option<String>,
        framenet_frame: Option<String>,
        pos_category: PosCategory,
        lemma_source: LemmaSource,
        lemma_confidence: f32,
    ) -> Self {
        let hash_code = Self::compute_hash(&lemma, &verbnet_class, &framenet_frame, &pos_category);

        Self {
            lemma,
            verbnet_class,
            framenet_frame,
            pos_category,
            lemma_source,
            lemma_confidence,
            hash_code,
        }
    }

    /// Compute hash for efficient lookups
    fn compute_hash(
        lemma: &str,
        verbnet_class: &Option<String>,
        framenet_frame: &Option<String>,
        pos_category: &PosCategory,
    ) -> u64 {
        let mut hasher = AHasher::default();
        lemma.hash(&mut hasher);
        verbnet_class.hash(&mut hasher);
        framenet_frame.hash(&mut hasher);
        pos_category.hash(&mut hasher);
        hasher.finish()
    }

    /// Create a simplified signature with just lemma and POS
    pub fn simple(lemma: String, pos_category: PosCategory) -> Self {
        Self::new(
            lemma,
            None,
            None,
            pos_category,
            LemmaSource::SimpleLemmatizer,
            0.5,
        )
    }

    /// Create a signature with specified lemma source and confidence
    pub fn with_lemma_info(
        lemma: String,
        pos_category: PosCategory,
        lemma_source: LemmaSource,
        lemma_confidence: f32,
    ) -> Self {
        Self::new(
            lemma,
            None,
            None,
            pos_category,
            lemma_source,
            lemma_confidence,
        )
    }

    /// Check if signatures are compatible for fallback matching
    pub fn is_compatible(&self, other: &Self) -> bool {
        // Same lemma is most important
        if self.lemma == other.lemma {
            return true;
        }

        // Same VerbNet class indicates similar verb semantics
        if let (Some(class1), Some(class2)) = (&self.verbnet_class, &other.verbnet_class) {
            if class1 == class2 {
                return true;
            }
        }

        // Same FrameNet frame indicates similar event structure
        if let (Some(frame1), Some(frame2)) = (&self.framenet_frame, &other.framenet_frame) {
            if frame1 == frame2 {
                return true;
            }
        }

        false
    }

    /// Get signature priority for conflict resolution
    /// Higher priority = more specific/reliable
    pub fn priority(&self) -> u32 {
        let mut priority = 0;

        // Lemma specificity
        priority += 1;

        // VerbNet class adds specificity
        if self.verbnet_class.is_some() {
            priority += 10;
        }

        // FrameNet frame adds specificity
        if self.framenet_frame.is_some() {
            priority += 5;
        }

        priority
    }
}

/// Builder for creating semantic signatures from Layer 1 analysis
#[derive(Debug)]
pub struct SignatureBuilder {
    /// Enable debug logging
    verbose: bool,
}

impl SignatureBuilder {
    /// Create a new signature builder
    pub fn new(verbose: bool) -> Self {
        Self { verbose }
    }

    /// Build signature from Layer 1 semantic result
    pub fn build_from_layer1(
        &self,
        lemma: &str,
        verbnet_analysis: Option<&canopy_verbnet::VerbNetAnalysis>,
        framenet_analysis: Option<&canopy_framenet::FrameNetAnalysis>,
        pos_hint: Option<&str>,
        lemma_source: LemmaSource,
        lemma_confidence: f32,
    ) -> SemanticSignature {
        let verbnet_class = verbnet_analysis
            .and_then(|analysis| analysis.verb_classes.first())
            .map(|class| class.id.clone());

        let framenet_frame = framenet_analysis
            .and_then(|analysis| analysis.frames.first())
            .map(|frame| frame.name.clone());

        let pos_category =
            self.infer_pos_category(lemma, pos_hint, &verbnet_class, &framenet_frame);

        let signature = SemanticSignature::new(
            lemma.to_string(),
            verbnet_class,
            framenet_frame,
            pos_category,
            lemma_source,
            lemma_confidence,
        );

        if self.verbose {
            debug!(
                "Created signature for '{}': VN={:?}, FN={:?}, POS={:?}, hash={:016x}",
                lemma,
                signature.verbnet_class,
                signature.framenet_frame,
                signature.pos_category,
                signature.hash_code
            );
        }

        signature
    }

    /// Build simplified signature for pattern synthesis
    pub fn build_simplified(&self, lemma: &str, pos_hint: Option<&str>) -> SemanticSignature {
        let pos_category = self.infer_pos_category(lemma, pos_hint, &None, &None);
        SemanticSignature::simple(lemma.to_string(), pos_category)
    }

    /// Build signature variants for fuzzy matching
    pub fn build_variants(&self, base_signature: &SemanticSignature) -> Vec<SemanticSignature> {
        let mut variants = vec![base_signature.clone()];

        // Variant without FrameNet (VerbNet only)
        if base_signature.framenet_frame.is_some() {
            variants.push(SemanticSignature::new(
                base_signature.lemma.clone(),
                base_signature.verbnet_class.clone(),
                None,
                base_signature.pos_category.clone(),
                base_signature.lemma_source,
                base_signature.lemma_confidence,
            ));
        }

        // Variant without VerbNet (FrameNet only)
        if base_signature.verbnet_class.is_some() {
            variants.push(SemanticSignature::new(
                base_signature.lemma.clone(),
                None,
                base_signature.framenet_frame.clone(),
                base_signature.pos_category.clone(),
                base_signature.lemma_source,
                base_signature.lemma_confidence,
            ));
        }

        // Minimal variant (lemma + POS only)
        variants.push(SemanticSignature::with_lemma_info(
            base_signature.lemma.clone(),
            base_signature.pos_category.clone(),
            base_signature.lemma_source,
            base_signature.lemma_confidence,
        ));

        if self.verbose {
            debug!(
                "Generated {} signature variants for '{}'",
                variants.len(),
                base_signature.lemma
            );
        }

        variants
    }

    /// Infer POS category from available information
    fn infer_pos_category(
        &self,
        lemma: &str,
        pos_hint: Option<&str>,
        verbnet_class: &Option<String>,
        framenet_frame: &Option<String>,
    ) -> PosCategory {
        // Use explicit POS hint if available
        if let Some(pos) = pos_hint {
            match pos.to_uppercase().as_str() {
                "VERB" | "VBZ" | "VBP" | "VBD" | "VBG" | "VBN" | "VB" => return PosCategory::Verb,
                "NOUN" | "NN" | "NNS" | "NNP" | "NNPS" => return PosCategory::Noun,
                "ADJ" | "JJ" | "JJR" | "JJS" => return PosCategory::Adjective,
                "ADV" | "RB" | "RBR" | "RBS" => return PosCategory::Adverb,
                _ => {}
            }
        }

        // Infer from VerbNet class (verbs only)
        if verbnet_class.is_some() {
            return PosCategory::Verb;
        }

        // Infer from FrameNet frame (usually verbs or event-denoting nouns)
        if let Some(frame) = framenet_frame {
            // Common verb frames
            if frame.contains("Motion") || frame.contains("Action") || frame.contains("Change") {
                return PosCategory::Verb;
            }
        }

        // Morphological heuristics
        if (lemma.ends_with("ing") || lemma.ends_with("ed") || lemma.ends_with("s"))
            && self.is_likely_verb(lemma)
        {
            return PosCategory::Verb;
        }

        if lemma.ends_with("ly") {
            return PosCategory::Adverb;
        }

        if lemma.ends_with("tion") || lemma.ends_with("ness") || lemma.ends_with("ment") {
            return PosCategory::Noun;
        }

        // Default fallback
        PosCategory::Other
    }

    /// Simple heuristic to check if a word is likely a verb
    fn is_likely_verb(&self, lemma: &str) -> bool {
        // Common verb patterns (this is a simplified heuristic)
        const COMMON_VERB_ENDINGS: &[&str] = &["ing", "ed", "ize", "ise", "ate", "fy"];
        const COMMON_VERBS: &[&str] = &[
            "run", "walk", "give", "take", "make", "go", "come", "see", "know", "think", "say",
            "tell", "ask", "work", "play", "live", "move", "put", "get", "have",
        ];

        COMMON_VERBS.contains(&lemma)
            || COMMON_VERB_ENDINGS
                .iter()
                .any(|ending| lemma.ends_with(ending))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semantic_signature_creation() {
        let sig = SemanticSignature::new(
            "run".to_string(),
            Some("run-51.3.2".to_string()),
            Some("Motion".to_string()),
            PosCategory::Verb,
            LemmaSource::SimpleLemmatizer,
            0.5,
        );

        assert_eq!(sig.lemma, "run");
        assert_eq!(sig.verbnet_class, Some("run-51.3.2".to_string()));
        assert_eq!(sig.framenet_frame, Some("Motion".to_string()));
        assert_eq!(sig.pos_category, PosCategory::Verb);
        assert_ne!(sig.hash_code, 0);
    }

    #[test]
    fn test_simple_signature() {
        let sig = SemanticSignature::simple("walk".to_string(), PosCategory::Verb);

        assert_eq!(sig.lemma, "walk");
        assert_eq!(sig.verbnet_class, None);
        assert_eq!(sig.framenet_frame, None);
        assert_eq!(sig.pos_category, PosCategory::Verb);
    }

    #[test]
    fn test_signature_compatibility() {
        let sig1 = SemanticSignature::new(
            "run".to_string(),
            Some("run-51.3.2".to_string()),
            Some("Motion".to_string()),
            PosCategory::Verb,
            LemmaSource::SimpleLemmatizer,
            0.5,
        );

        let sig2 = SemanticSignature::new(
            "run".to_string(),
            Some("run-51.3.1".to_string()),
            Some("Motion".to_string()),
            PosCategory::Verb,
            LemmaSource::SimpleLemmatizer,
            0.5,
        );

        // Same lemma should be compatible
        assert!(sig1.is_compatible(&sig2));
    }

    #[test]
    fn test_signature_priority() {
        let full_sig = SemanticSignature::new(
            "run".to_string(),
            Some("run-51.3.2".to_string()),
            Some("Motion".to_string()),
            PosCategory::Verb,
            LemmaSource::SimpleLemmatizer,
            0.5,
        );

        let simple_sig = SemanticSignature::simple("run".to_string(), PosCategory::Verb);

        assert!(full_sig.priority() > simple_sig.priority());
    }

    #[test]
    fn test_signature_builder() {
        let builder = SignatureBuilder::new(false);

        // Test POS inference
        let verb_sig = builder.build_simplified("running", Some("VBG"));
        assert_eq!(verb_sig.pos_category, PosCategory::Verb);

        let noun_sig = builder.build_simplified("house", Some("NN"));
        assert_eq!(noun_sig.pos_category, PosCategory::Noun);
    }

    #[test]
    fn test_pos_inference_heuristics() {
        let builder = SignatureBuilder::new(false);

        // Test verb detection
        assert!(matches!(
            builder.infer_pos_category("running", None, &None, &None),
            PosCategory::Verb | PosCategory::Other
        ));

        // Test adverb detection
        assert_eq!(
            builder.infer_pos_category("quickly", None, &None, &None),
            PosCategory::Adverb
        );

        // Test noun detection
        assert_eq!(
            builder.infer_pos_category("information", None, &None, &None),
            PosCategory::Noun
        );
    }

    #[test]
    fn test_signature_variants() {
        let builder = SignatureBuilder::new(false);
        let base_sig = SemanticSignature::new(
            "run".to_string(),
            Some("run-51.3.2".to_string()),
            Some("Motion".to_string()),
            PosCategory::Verb,
            LemmaSource::SimpleLemmatizer,
            0.5,
        );

        let variants = builder.build_variants(&base_sig);

        // Should include original + variants
        assert!(variants.len() >= 2);
        assert!(variants.iter().any(|v| v == &base_sig));
        assert!(variants.iter().any(|v| v.verbnet_class.is_none()));
    }

    #[test]
    fn test_hash_consistency() {
        let sig1 = SemanticSignature::new(
            "run".to_string(),
            Some("run-51.3.2".to_string()),
            Some("Motion".to_string()),
            PosCategory::Verb,
            LemmaSource::SimpleLemmatizer,
            0.5,
        );

        let sig2 = SemanticSignature::new(
            "run".to_string(),
            Some("run-51.3.2".to_string()),
            Some("Motion".to_string()),
            PosCategory::Verb,
            LemmaSource::SimpleLemmatizer,
            0.5,
        );

        assert_eq!(sig1.hash_code, sig2.hash_code);
        assert_eq!(sig1, sig2);
    }
}
