//! Negation scope handling
//!
//! Handles negation detection and scope resolution, including neg-raising verbs.
//!
//! ## Neg-Raising Verbs
//!
//! Some verbs allow negation to be interpreted as scoping over their complement:
//! - "I don't think he left" → interpreted as "I think he didn't leave"
//!
//! These verbs form a relatively closed class in English and are detected
//! via VerbNet classes (want-32.1, conjecture-29.5, etc.).
//!
//! ## Negation Detection
//!
//! Negation is detected from:
//! - SentenceMetadata.is_negated
//! - Negative dependency markers in the sentence

use crate::config::EventComposerConfig;
use crate::error::EventResult;
use crate::types::{PredicateInfo, SentenceMetadata};

/// Result of applying negation scope
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NegationResult {
    /// Negation was applied to the main event
    Applied,
    /// Negation was raised to the complement
    Raised,
    /// No negation detected
    None,
}

/// Handles negation scope for events
pub struct NegationHandler {
    /// VerbNet classes that allow neg-raising
    neg_raising_patterns: Vec<&'static str>,
}

impl NegationHandler {
    /// Create a new negation handler
    pub fn new(_config: &EventComposerConfig) -> EventResult<Self> {
        Ok(Self {
            // VerbNet classes for neg-raising verbs
            // These verbs allow negation to scope over their complement
            neg_raising_patterns: vec![
                "want-32.1",         // want, desire
                "wish-62",           // wish
                "conjecture-29.5",   // think, believe, suppose
                "consider-29.9",     // consider, expect
                "characterize-29.2", // seem, appear (raising verbs)
            ],
        })
    }

    /// Apply negation scope to an event
    ///
    /// Returns the polarity for the event and whether neg-raising occurred.
    pub fn apply_scope(
        &self,
        predicate: &PredicateInfo,
        metadata: &SentenceMetadata,
    ) -> (bool, NegationResult) {
        // Check if sentence is negated
        if !metadata.is_negated {
            return (true, NegationResult::None);
        }

        // Check if this is a neg-raising verb
        if self.is_neg_raising_verb(predicate) {
            // Negation is raised to complement - main event stays positive
            // (The complement event would get polarity: false)
            return (true, NegationResult::Raised);
        }

        // Standard negation - event gets negative polarity
        (false, NegationResult::Applied)
    }

    /// Check if predicate allows neg-raising
    fn is_neg_raising_verb(&self, predicate: &PredicateInfo) -> bool {
        if let Some(ref vn) = predicate.verbnet_analysis {
            for verb_class in &vn.verb_classes {
                let class_id = &verb_class.id;

                // Check if class matches any neg-raising pattern
                for pattern in &self.neg_raising_patterns {
                    let pattern_prefix = pattern.split('-').next().unwrap_or("");
                    if class_id.starts_with(pattern_prefix) {
                        return true;
                    }
                }
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_metadata(is_negated: bool) -> SentenceMetadata {
        SentenceMetadata {
            is_negated,
            ..Default::default()
        }
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
    fn test_non_negated_sentence() {
        let handler = NegationHandler::new(&EventComposerConfig::default()).unwrap();

        let predicate = make_predicate("run");
        let metadata = make_metadata(false);

        let (polarity, result) = handler.apply_scope(&predicate, &metadata);

        assert!(polarity);
        assert_eq!(result, NegationResult::None);
    }

    #[test]
    fn test_simple_negation() {
        let handler = NegationHandler::new(&EventComposerConfig::default()).unwrap();

        let predicate = make_predicate("run");
        let metadata = make_metadata(true);

        let (polarity, result) = handler.apply_scope(&predicate, &metadata);

        assert!(!polarity); // Negated
        assert_eq!(result, NegationResult::Applied);
    }
}
