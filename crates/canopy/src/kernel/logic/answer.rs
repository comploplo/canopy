//! Answer types for query results.
//!
//! Defines the structures returned when answering queries against a DRS.

use super::proof::Explanation;
use crate::kernel::discourse::ReferentId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Result of answering a query.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QueryResult {
    /// The answers found (may be multiple for wh-questions).
    pub answers: Vec<Answer>,
    /// Whether the query was fully resolved.
    pub query_resolved: bool,
    /// Explanation of how the answer was derived.
    pub explanation: Option<Explanation>,
}

impl QueryResult {
    /// Create an empty result (no answers found).
    #[must_use]
    pub fn empty() -> Self {
        Self {
            answers: Vec::new(),
            query_resolved: true,
            explanation: None,
        }
    }

    /// Create a yes answer.
    #[must_use]
    pub fn yes() -> Self {
        Self {
            answers: vec![Answer::yes()],
            query_resolved: true,
            explanation: None,
        }
    }

    /// Create a no answer.
    #[must_use]
    pub fn no() -> Self {
        Self {
            answers: vec![Answer::no()],
            query_resolved: true,
            explanation: None,
        }
    }

    /// Create an unknown answer.
    #[must_use]
    pub fn unknown() -> Self {
        Self {
            answers: vec![Answer::unknown()],
            query_resolved: false,
            explanation: None,
        }
    }

    /// Create a result with bindings (for wh-questions).
    #[must_use]
    pub fn with_bindings(bindings: Vec<HashMap<String, AnswerBinding>>) -> Self {
        let answers = bindings
            .into_iter()
            .map(|b| Answer {
                bindings: b,
                confidence: 1.0,
                scope_reading: None,
                supporting_sentences: Vec::new(),
                is_yes: true,
            })
            .collect();
        Self {
            answers,
            query_resolved: true,
            explanation: None,
        }
    }

    /// Add an explanation.
    #[must_use]
    pub fn with_explanation(mut self, explanation: Explanation) -> Self {
        self.explanation = Some(explanation);
        self
    }

    /// Check if the query has a positive answer.
    #[must_use]
    pub fn is_yes(&self) -> bool {
        self.answers.first().is_some_and(|a| a.is_yes)
    }

    /// Check if the query has a negative answer.
    #[must_use]
    pub fn is_no(&self) -> bool {
        self.answers.first().is_some_and(|a| !a.is_yes)
    }

    /// Check if the query is unresolved.
    #[must_use]
    pub fn is_unknown(&self) -> bool {
        !self.query_resolved
    }

    /// Get the first answer's bindings.
    #[must_use]
    pub fn first_bindings(&self) -> Option<&HashMap<String, AnswerBinding>> {
        self.answers.first().map(|a| &a.bindings)
    }

    /// Get all binding values for a variable.
    #[must_use]
    pub fn all_values_for(&self, variable: &str) -> Vec<&str> {
        self.answers
            .iter()
            .filter_map(|a| a.bindings.get(variable).map(|b| b.text.as_str()))
            .collect()
    }
}

impl Default for QueryResult {
    fn default() -> Self {
        Self::empty()
    }
}

/// A single answer to a query.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Answer {
    /// Variable bindings for wh-questions.
    pub bindings: HashMap<String, AnswerBinding>,
    /// Confidence in this answer (0.0-1.0).
    pub confidence: f32,
    /// Which scope reading this answer comes from (if applicable).
    pub scope_reading: Option<usize>,
    /// Sentence indices that support this answer.
    pub supporting_sentences: Vec<usize>,
    /// Whether this is a positive answer (yes) or negative (no).
    pub is_yes: bool,
}

impl Answer {
    /// Create a "yes" answer.
    #[must_use]
    pub fn yes() -> Self {
        Self {
            bindings: HashMap::new(),
            confidence: 1.0,
            scope_reading: None,
            supporting_sentences: Vec::new(),
            is_yes: true,
        }
    }

    /// Create a "no" answer.
    #[must_use]
    pub fn no() -> Self {
        Self {
            bindings: HashMap::new(),
            confidence: 1.0,
            scope_reading: None,
            supporting_sentences: Vec::new(),
            is_yes: false,
        }
    }

    /// Create an "unknown" answer.
    #[must_use]
    pub fn unknown() -> Self {
        Self {
            bindings: HashMap::new(),
            confidence: 0.0,
            scope_reading: None,
            supporting_sentences: Vec::new(),
            is_yes: false,
        }
    }

    /// Create an answer with bindings.
    #[must_use]
    pub fn with_binding(variable: impl Into<String>, binding: AnswerBinding) -> Self {
        let mut bindings = HashMap::new();
        bindings.insert(variable.into(), binding);
        Self {
            bindings,
            confidence: 1.0,
            scope_reading: None,
            supporting_sentences: Vec::new(),
            is_yes: true,
        }
    }

    /// Add a supporting sentence.
    #[must_use]
    pub fn with_support(mut self, sentence: usize) -> Self {
        self.supporting_sentences.push(sentence);
        self
    }

    /// Set confidence.
    #[must_use]
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence;
        self
    }
}

impl Default for Answer {
    fn default() -> Self {
        Self::unknown()
    }
}

/// A binding of a variable to a discourse referent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnswerBinding {
    /// The discourse referent this variable is bound to.
    pub referent_id: ReferentId,
    /// The text/name of the referent.
    pub text: String,
    /// The sentence index where this referent was introduced.
    pub introduced_at: usize,
}

impl AnswerBinding {
    /// Create a new binding.
    #[must_use]
    pub fn new(referent_id: ReferentId, text: impl Into<String>, introduced_at: usize) -> Self {
        Self {
            referent_id,
            text: text.into(),
            introduced_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_result_yes() {
        let result = QueryResult::yes();
        assert!(result.is_yes());
        assert!(!result.is_no());
        assert!(!result.is_unknown());
    }

    #[test]
    fn test_query_result_no() {
        let result = QueryResult::no();
        assert!(!result.is_yes());
        assert!(result.is_no());
        assert!(!result.is_unknown());
    }

    #[test]
    fn test_query_result_unknown() {
        let result = QueryResult::unknown();
        assert!(!result.is_yes());
        assert!(result.is_unknown());
    }

    #[test]
    fn test_query_result_bindings() {
        let binding = AnswerBinding::new(ReferentId::new(1), "John", 0);
        let mut bindings = HashMap::new();
        bindings.insert("?who".to_string(), binding);

        let result = QueryResult::with_bindings(vec![bindings]);
        assert!(result.is_yes());
        assert_eq!(result.answers.len(), 1);

        let values = result.all_values_for("?who");
        assert_eq!(values, vec!["John"]);
    }

    #[test]
    fn test_answer_with_support() {
        let answer = Answer::yes().with_support(0).with_support(2);
        assert_eq!(answer.supporting_sentences, vec![0, 2]);
    }

    #[test]
    fn test_answer_binding() {
        let binding = AnswerBinding::new(ReferentId::new(5), "Mary", 1);
        assert_eq!(binding.referent_id, ReferentId::new(5));
        assert_eq!(binding.text, "Mary");
        assert_eq!(binding.introduced_at, 1);
    }
}
