//! Proof and explanation structures.
//!
//! Provides structures for explaining how answers were derived from the DRS.

use crate::kernel::discourse::PresuppositionStatus;
use serde::{Deserialize, Serialize};

/// An explanation of how an answer was derived.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Explanation {
    /// The reasoning steps.
    pub steps: Vec<ExplanationStep>,
    /// A human-readable summary.
    pub summary: String,
}

impl Explanation {
    /// Create a new explanation.
    #[must_use]
    pub fn new(summary: impl Into<String>) -> Self {
        Self {
            steps: Vec::new(),
            summary: summary.into(),
        }
    }

    /// Add a step.
    #[must_use]
    pub fn with_step(mut self, step: ExplanationStep) -> Self {
        self.steps.push(step);
        self
    }

    /// Add multiple steps.
    #[must_use]
    pub fn with_steps(mut self, steps: Vec<ExplanationStep>) -> Self {
        self.steps.extend(steps);
        self
    }

    /// Check if there are any steps.
    #[must_use]
    pub fn has_steps(&self) -> bool {
        !self.steps.is_empty()
    }

    /// Get the number of steps.
    #[must_use]
    pub fn step_count(&self) -> usize {
        self.steps.len()
    }

    /// Format as a readable string.
    #[must_use]
    pub fn format(&self) -> String {
        use std::fmt::Write;
        let mut output = self.summary.clone();
        if !self.steps.is_empty() {
            output.push_str("\n\nReasoning:");
            for (i, step) in self.steps.iter().enumerate() {
                let _ = write!(output, "\n  {}. {}", i + 1, step.format());
            }
        }
        output
    }
}

impl Default for Explanation {
    fn default() -> Self {
        Self::new("")
    }
}

/// A single step in an explanation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExplanationStep {
    /// The type of reasoning step.
    pub kind: StepKind,
    /// Reference to the condition involved.
    pub condition: ConditionRef,
    /// The sentence index this step relates to.
    pub sentence_index: usize,
    /// Optional text span from the original input.
    pub text_span: Option<String>,
}

impl ExplanationStep {
    /// Create an assertion step.
    #[must_use]
    pub fn asserted(condition: ConditionRef, sentence: usize, text: Option<String>) -> Self {
        Self {
            kind: StepKind::Asserted { sentence },
            condition,
            sentence_index: sentence,
            text_span: text,
        }
    }

    /// Create an inference step.
    #[must_use]
    pub fn inferred(condition: ConditionRef, from: Vec<ConditionRef>, sentence: usize) -> Self {
        Self {
            kind: StepKind::Inferred { from },
            condition,
            sentence_index: sentence,
            text_span: None,
        }
    }

    /// Create a presupposition step.
    #[must_use]
    pub fn presupposed(
        condition: ConditionRef,
        status: PresuppositionStatus,
        sentence: usize,
    ) -> Self {
        Self {
            kind: StepKind::Presupposed { status },
            condition,
            sentence_index: sentence,
            text_span: None,
        }
    }

    /// Create a modus ponens step.
    #[must_use]
    pub fn modus_ponens(
        condition: ConditionRef,
        premise: ConditionRef,
        implication: ConditionRef,
        sentence: usize,
    ) -> Self {
        Self {
            kind: StepKind::ModusPonens {
                premise,
                implication,
            },
            condition,
            sentence_index: sentence,
            text_span: None,
        }
    }

    /// Create a contradiction step.
    #[must_use]
    pub fn contradiction(cond1: ConditionRef, cond2: ConditionRef, sentence: usize) -> Self {
        Self {
            kind: StepKind::Contradiction {
                cond1: cond1.clone(),
                cond2,
            },
            condition: cond1,
            sentence_index: sentence,
            text_span: None,
        }
    }

    /// Format this step as a readable string.
    #[must_use]
    pub fn format(&self) -> String {
        match &self.kind {
            StepKind::Asserted { sentence } => {
                let text = self
                    .text_span
                    .as_deref()
                    .map_or(String::new(), |t| format!(" \"{t}\""));
                format!("Asserted in sentence {sentence}{text}")
            }
            StepKind::Inferred { from } => {
                let refs: Vec<_> = from
                    .iter()
                    .map(|r| format!("cond{}", r.condition_index))
                    .collect();
                format!("Inferred from: {}", refs.join(", "))
            }
            StepKind::Presupposed { status } => {
                format!("Presupposition ({status:?})")
            }
            StepKind::ModusPonens {
                premise,
                implication,
            } => {
                format!(
                    "Modus ponens: cond{} + cond{} → this",
                    premise.condition_index, implication.condition_index
                )
            }
            StepKind::Contradiction { cond1, cond2 } => {
                format!(
                    "Contradiction between cond{} and cond{}",
                    cond1.condition_index, cond2.condition_index
                )
            }
        }
    }
}

/// The kind of reasoning step.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StepKind {
    /// Direct assertion from discourse.
    Asserted {
        /// The sentence where this was asserted.
        sentence: usize,
    },

    /// Inferred from other conditions.
    Inferred {
        /// The conditions this was inferred from.
        from: Vec<ConditionRef>,
    },

    /// Presupposition (accommodated or satisfied).
    Presupposed {
        /// The status of the presupposition.
        status: PresuppositionStatus,
    },

    /// Derived via modus ponens.
    ModusPonens {
        /// The premise condition.
        premise: ConditionRef,
        /// The implication condition.
        implication: ConditionRef,
    },

    /// Contradiction detected.
    Contradiction {
        /// First conflicting condition.
        cond1: ConditionRef,
        /// Second conflicting condition.
        cond2: ConditionRef,
    },
}

/// Reference to a specific condition with provenance.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConditionRef {
    /// Index of the condition in the DRS.
    pub condition_index: usize,
    /// Path through subordinate DRSs (empty for main DRS).
    pub drs_path: Vec<usize>,
    /// Sentence index where this condition was introduced.
    pub introduced_at: usize,
}

impl ConditionRef {
    /// Create a reference to a condition in the main DRS.
    #[must_use]
    pub fn main(condition_index: usize, introduced_at: usize) -> Self {
        Self {
            condition_index,
            drs_path: Vec::new(),
            introduced_at,
        }
    }

    /// Create a reference to a condition in a subordinate DRS.
    #[must_use]
    pub fn subordinate(condition_index: usize, drs_path: Vec<usize>, introduced_at: usize) -> Self {
        Self {
            condition_index,
            drs_path,
            introduced_at,
        }
    }

    /// Check if this is in the main DRS.
    #[must_use]
    pub fn is_main(&self) -> bool {
        self.drs_path.is_empty()
    }
}

impl Default for ConditionRef {
    fn default() -> Self {
        Self::main(0, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_explanation_creation() {
        let explanation = Explanation::new("John left because he was told to.");
        assert_eq!(explanation.summary, "John left because he was told to.");
        assert!(!explanation.has_steps());
    }

    #[test]
    fn test_explanation_with_steps() {
        let explanation = Explanation::new("Answer: Yes")
            .with_step(ExplanationStep::asserted(
                ConditionRef::main(0, 0),
                0,
                Some("John left".into()),
            ))
            .with_step(ExplanationStep::inferred(
                ConditionRef::main(1, 0),
                vec![ConditionRef::main(0, 0)],
                0,
            ));

        assert!(explanation.has_steps());
        assert_eq!(explanation.step_count(), 2);
    }

    #[test]
    fn test_condition_ref() {
        let main_ref = ConditionRef::main(5, 2);
        assert!(main_ref.is_main());
        assert_eq!(main_ref.condition_index, 5);
        assert_eq!(main_ref.introduced_at, 2);

        let sub_ref = ConditionRef::subordinate(3, vec![0, 1], 1);
        assert!(!sub_ref.is_main());
        assert_eq!(sub_ref.drs_path, vec![0, 1]);
    }

    #[test]
    fn test_step_formatting() {
        let step = ExplanationStep::asserted(ConditionRef::main(0, 0), 0, Some("John left".into()));
        let formatted = step.format();
        assert!(formatted.contains("Asserted in sentence 0"));
        assert!(formatted.contains("John left"));
    }

    #[test]
    fn test_explanation_formatting() {
        let explanation = Explanation::new("John is mortal.")
            .with_step(ExplanationStep::asserted(
                ConditionRef::main(0, 0),
                0,
                Some("All men are mortal".into()),
            ))
            .with_step(ExplanationStep::asserted(
                ConditionRef::main(1, 1),
                1,
                Some("John is a man".into()),
            ));

        let formatted = explanation.format();
        assert!(formatted.contains("John is mortal."));
        assert!(formatted.contains("Reasoning:"));
        assert!(formatted.contains("1."));
        assert!(formatted.contains("2."));
    }
}
