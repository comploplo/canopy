//! Plurality and distributivity inference
//!
//! Infers semantic number (singular/plural/mass) and distributivity
//! (collective/distributive) for entities in events.
//!
//! ## Semantic Number
//!
//! Unlike morphological number, semantic number captures:
//! - Singular: individual entities
//! - Plural: collections of discrete entities
//! - Mass: non-countable quantities (water, furniture)
//!
//! ## Distributivity
//!
//! For plural entities, determines whether predicates apply:
//! - Collectively: "The boys lifted the piano" (together)
//! - Distributively: "The boys each ran a mile"
//! - Unspecified: when context doesn't disambiguate

use crate::config::EventComposerConfig;
use crate::error::EventResult;
use crate::types::PredicateInfo;
use canopy_core::{Distributivity, SemanticNumber};
use canopy_tokenizer::coordinator::Layer1SemanticResult;

/// Infers plurality and distributivity for entities
pub struct PluralityInferrer {
    /// VerbNet predicates that indicate collective reading
    collective_predicates: Vec<&'static str>,
}

impl PluralityInferrer {
    /// Create a new plurality inferrer
    pub fn new(_config: &EventComposerConfig) -> EventResult<Self> {
        Ok(Self {
            // VerbNet semantic predicates that indicate collective reading
            collective_predicates: vec![
                "together", // "The boys gathered together"
                "group",    // "They formed a group"
                "meet",     // "They met at noon"
            ],
        })
    }

    /// Infer semantic number from a token
    ///
    /// Currently uses WordNet heuristics. In the future, this could be enhanced
    /// to use morphological features from UD parsing.
    pub fn infer_number(&self, token: &Layer1SemanticResult) -> Option<SemanticNumber> {
        // Check for mass nouns via WordNet
        // Mass nouns typically have no plural form or are uncountable
        if self.is_likely_mass_noun(token) {
            return Some(SemanticNumber::Mass);
        }

        // Check word endings as a heuristic
        // (This is imperfect but useful until we have full morphological parsing)
        let word = token.lemma.to_lowercase();
        if word.ends_with('s') && !word.ends_with("ss") && word.len() > 2 {
            // Likely plural (imperfect heuristic)
            return Some(SemanticNumber::Plural);
        }

        // Default to singular for most nouns
        Some(SemanticNumber::Singular)
    }

    /// Check if token is likely a mass noun
    fn is_likely_mass_noun(&self, token: &Layer1SemanticResult) -> bool {
        // Check WordNet analysis for mass noun indicators
        if let Some(ref wn) = token.wordnet {
            for synset in &wn.synsets {
                // Mass nouns often have lexical markers or specific hypernyms
                // This is a heuristic - real implementation would check WordNet directly
                let def = synset.definition();
                if def.contains("substance") || def.contains("material") || def.contains("liquid") {
                    return true;
                }
            }
        }

        // Common mass nouns can be detected by lemma pattern
        // (but we avoid hardcoding specific words per project requirements)
        false
    }

    /// Infer distributivity for a plural subject
    pub fn infer_distributivity(
        &self,
        number: SemanticNumber,
        predicate: &PredicateInfo,
        has_each_adverb: bool,
    ) -> Option<Distributivity> {
        // Only relevant for plural entities
        if !matches!(number, SemanticNumber::Plural) {
            return None;
        }

        // Priority 1: "each" adverb forces distributive reading
        if has_each_adverb {
            return Some(Distributivity::Distributive);
        }

        // Priority 2: Check VerbNet predicates for collective indicators
        if let Some(ref vn) = predicate.verbnet_analysis {
            for verb_class in &vn.verb_classes {
                // Check frames for semantic predicates
                for frame in &verb_class.frames {
                    for pred in &frame.semantics {
                        let pred_name = pred.value.to_lowercase();
                        if self.collective_predicates.contains(&pred_name.as_str()) {
                            return Some(Distributivity::Collective);
                        }
                    }
                }
            }
        }

        // Default: Unspecified when we can't disambiguate
        Some(Distributivity::Unspecified)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_token(lemma: &str) -> Layer1SemanticResult {
        let mut token = Layer1SemanticResult::new(lemma.to_string(), lemma.to_string());
        token.pos = Some(canopy_core::UPos::Noun);
        token.confidence = 1.0;
        token
    }

    fn make_predicate(lemma: &str) -> PredicateInfo {
        PredicateInfo {
            lemma: lemma.to_string(),
            token_idx: 0,
            verbnet_analysis: None,
            framenet_analysis: None,
            l1_confidence: 1.0,
        }
    }

    #[test]
    fn test_singular_number() {
        let inferrer = PluralityInferrer::new(&EventComposerConfig::default()).unwrap();
        let token = make_token("book"); // No 's' ending

        let number = inferrer.infer_number(&token);

        assert_eq!(number, Some(SemanticNumber::Singular));
    }

    #[test]
    fn test_plural_number() {
        let inferrer = PluralityInferrer::new(&EventComposerConfig::default()).unwrap();
        let token = make_token("books"); // Has 's' ending

        let number = inferrer.infer_number(&token);

        assert_eq!(number, Some(SemanticNumber::Plural));
    }

    #[test]
    fn test_each_forces_distributive() {
        let inferrer = PluralityInferrer::new(&EventComposerConfig::default()).unwrap();
        let predicate = make_predicate("run");

        let distributivity =
            inferrer.infer_distributivity(SemanticNumber::Plural, &predicate, true);

        assert_eq!(distributivity, Some(Distributivity::Distributive));
    }

    #[test]
    fn test_singular_no_distributivity() {
        let inferrer = PluralityInferrer::new(&EventComposerConfig::default()).unwrap();
        let predicate = make_predicate("run");

        let distributivity =
            inferrer.infer_distributivity(SemanticNumber::Singular, &predicate, false);

        assert_eq!(distributivity, None);
    }
}
