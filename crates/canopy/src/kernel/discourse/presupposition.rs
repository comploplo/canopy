//! Presupposition detection and accommodation.
//!
//! Presuppositions are implicit assumptions that must hold for an utterance
//! to be felicitous. This module detects presupposition triggers and manages
//! their status in discourse.
//!
//! # Presupposition Types
//!
//! - **Existential**: "the X" presupposes X exists
//! - **Factive**: "knew that P" presupposes P is true
//! - **Iterative**: "again" presupposes prior occurrence
//! - **Change**: "stopped X-ing" presupposes was X-ing
//!
//! # Accommodation
//!
//! When a presupposition is not satisfied by existing discourse context,
//! it can be "accommodated" by adding it to the context (if plausible).

use super::drs::{Drs, DrsCondition};
use super::referent::ReferentId;
use serde::{Deserialize, Serialize};

/// Types of presuppositions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Presupposition {
    /// "the X" presupposes X exists and is uniquely identifiable.
    Existential {
        /// The referent whose existence is presupposed.
        referent: ReferentId,
        /// Description used to identify the referent.
        description: String,
    },

    /// Factive verbs ("know", "realize", "regret") presuppose their complement.
    Factive {
        /// The factive verb.
        verb: String,
        /// The presupposed proposition (simplified as description).
        proposition: String,
    },

    /// "again", "another", "return" presuppose prior occurrence.
    Iterative {
        /// The trigger word.
        trigger: String,
        /// What is presupposed to have occurred before.
        prior_event: String,
    },

    /// Aspectual verbs presuppose a change from a prior state.
    /// "stopped X-ing" presupposes was X-ing.
    /// "started X-ing" presupposes was not X-ing.
    Change {
        /// The change verb (stop, start, continue, etc.).
        verb: String,
        /// The activity.
        activity: String,
        /// The presupposed prior state.
        prior_state: PriorState,
    },

    /// Cleft constructions ("It was John who...") presuppose the proposition.
    Cleft {
        /// The focused element.
        focus: String,
        /// The presupposed background.
        background: String,
    },
}

/// Prior state for change presuppositions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PriorState {
    /// Was in the state/activity.
    WasActive,
    /// Was not in the state/activity.
    WasInactive,
    /// Was in a different state.
    WasDifferent(String),
}

/// Status of a presupposition in discourse.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PresuppositionStatus {
    /// Already satisfied by existing DRS content.
    Satisfied,
    /// Added to DRS through accommodation.
    Accommodated,
    /// Cannot be accommodated (inconsistent or implausible).
    Failed,
    /// Explicitly cancelled in context ("if" clauses, negation).
    Cancelled,
    /// Not yet evaluated.
    Pending,
}

/// A tracked presupposition with its status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackedPresupposition {
    /// The presupposition itself.
    pub presupposition: Presupposition,
    /// Current status.
    pub status: PresuppositionStatus,
    /// Sentence where it was triggered.
    pub triggered_at: usize,
    /// Optional note about resolution.
    pub resolution_note: Option<String>,
}

/// Trigger patterns for presupposition detection.
#[derive(Debug, Clone)]
pub struct PresuppositionDetector {
    /// Factive verbs that presuppose their complement is true.
    factive_verbs: Vec<String>,
    /// Aspectual change verbs.
    change_verbs: Vec<(String, PriorState)>,
    /// Iterative triggers.
    iterative_triggers: Vec<String>,
}

impl Default for PresuppositionDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl PresuppositionDetector {
    /// Create a new detector with default patterns.
    #[must_use]
    pub fn new() -> Self {
        let factive_verbs = vec![
            "know".to_string(),
            "knew".to_string(),
            "knows".to_string(),
            "realize".to_string(),
            "realized".to_string(),
            "realizes".to_string(),
            "regret".to_string(),
            "regretted".to_string(),
            "regrets".to_string(),
            "discover".to_string(),
            "discovered".to_string(),
            "discovers".to_string(),
            "notice".to_string(),
            "noticed".to_string(),
            "notices".to_string(),
            "remember".to_string(),
            "remembered".to_string(),
            "remembers".to_string(),
            "forget".to_string(),
            "forgot".to_string(),
            "forgets".to_string(),
        ];

        let change_verbs = vec![
            ("stop".to_string(), PriorState::WasActive),
            ("stopped".to_string(), PriorState::WasActive),
            ("stops".to_string(), PriorState::WasActive),
            ("start".to_string(), PriorState::WasInactive),
            ("started".to_string(), PriorState::WasInactive),
            ("starts".to_string(), PriorState::WasInactive),
            ("begin".to_string(), PriorState::WasInactive),
            ("began".to_string(), PriorState::WasInactive),
            ("begins".to_string(), PriorState::WasInactive),
            ("continue".to_string(), PriorState::WasActive),
            ("continued".to_string(), PriorState::WasActive),
            ("continues".to_string(), PriorState::WasActive),
            ("resume".to_string(), PriorState::WasActive),
            ("resumed".to_string(), PriorState::WasActive),
            ("resumes".to_string(), PriorState::WasActive),
            ("finish".to_string(), PriorState::WasActive),
            ("finished".to_string(), PriorState::WasActive),
            ("finishes".to_string(), PriorState::WasActive),
        ];

        let iterative_triggers = vec![
            "again".to_string(),
            "another".to_string(),
            "return".to_string(),
            "returned".to_string(),
            "returns".to_string(),
            "re-".to_string(), // prefix
            "back".to_string(),
        ];

        Self {
            factive_verbs,
            change_verbs,
            iterative_triggers,
        }
    }

    /// Check if a word is a factive verb.
    #[must_use]
    pub fn is_factive(&self, word: &str) -> bool {
        self.factive_verbs
            .iter()
            .any(|v| v.eq_ignore_ascii_case(word))
    }

    /// Check if a word is a change verb, returning the prior state if so.
    #[must_use]
    pub fn is_change_verb(&self, word: &str) -> Option<PriorState> {
        self.change_verbs
            .iter()
            .find(|(v, _)| v.eq_ignore_ascii_case(word))
            .map(|(_, state)| state.clone())
    }

    /// Check if a word is an iterative trigger.
    #[must_use]
    pub fn is_iterative(&self, word: &str) -> bool {
        let lower = word.to_lowercase();
        self.iterative_triggers.iter().any(|t| {
            if t == "re-" {
                lower.starts_with("re") && lower.len() > 2
            } else {
                t.eq_ignore_ascii_case(word)
            }
        })
    }

    /// Check if a token sequence represents a definite description.
    /// Simplified: looks for "the X" pattern.
    #[must_use]
    pub fn is_definite_description(tokens: &[String]) -> bool {
        if tokens.is_empty() {
            return false;
        }
        tokens[0].eq_ignore_ascii_case("the")
    }

    /// Detect presuppositions from a sequence of tokens.
    ///
    /// Returns a list of detected presuppositions.
    #[must_use]
    pub fn detect(&self, tokens: &[String], sentence: usize) -> Vec<TrackedPresupposition> {
        let mut presuppositions = Vec::new();

        for (i, token) in tokens.iter().enumerate() {
            // Check for factive verbs
            if self.is_factive(token) {
                // The complement is the following clause (simplified)
                let complement: String = tokens
                    .iter()
                    .skip(i + 1)
                    .take(5)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(" ");

                if !complement.is_empty() {
                    presuppositions.push(TrackedPresupposition {
                        presupposition: Presupposition::Factive {
                            verb: token.clone(),
                            proposition: complement,
                        },
                        status: PresuppositionStatus::Pending,
                        triggered_at: sentence,
                        resolution_note: None,
                    });
                }
            }

            // Check for change verbs
            if let Some(prior_state) = self.is_change_verb(token) {
                // The activity is the following word(s)
                let activity: String = tokens
                    .iter()
                    .skip(i + 1)
                    .take(3)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(" ");

                if !activity.is_empty() {
                    presuppositions.push(TrackedPresupposition {
                        presupposition: Presupposition::Change {
                            verb: token.clone(),
                            activity,
                            prior_state,
                        },
                        status: PresuppositionStatus::Pending,
                        triggered_at: sentence,
                        resolution_note: None,
                    });
                }
            }

            // Check for iterative triggers
            if self.is_iterative(token) {
                // What is being repeated
                let prior_event: String = tokens
                    .iter()
                    .filter(|t| !t.eq_ignore_ascii_case(token))
                    .take(5)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(" ");

                if !prior_event.is_empty() {
                    presuppositions.push(TrackedPresupposition {
                        presupposition: Presupposition::Iterative {
                            trigger: token.clone(),
                            prior_event,
                        },
                        status: PresuppositionStatus::Pending,
                        triggered_at: sentence,
                        resolution_note: None,
                    });
                }
            }
        }

        // Check for definite descriptions ("the X")
        let mut i = 0;
        while i < tokens.len() {
            if tokens[i].eq_ignore_ascii_case("the") && i + 1 < tokens.len() {
                // Get the noun phrase
                let description: String = tokens
                    .iter()
                    .skip(i + 1)
                    .take_while(|t| {
                        // Simple heuristic: take words until punctuation or verb
                        !t.contains('.') && !t.contains(',') && !t.contains('?')
                    })
                    .take(3)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(" ");

                if !description.is_empty() {
                    presuppositions.push(TrackedPresupposition {
                        presupposition: Presupposition::Existential {
                            referent: ReferentId::new(0), // Will be resolved later
                            description,
                        },
                        status: PresuppositionStatus::Pending,
                        triggered_at: sentence,
                        resolution_note: None,
                    });
                }
                i += 2; // Skip past "the X"
            } else {
                i += 1;
            }
        }

        presuppositions
    }
}

/// Manager for tracking and resolving presuppositions.
#[derive(Debug, Clone, Default)]
pub struct PresuppositionManager {
    /// Detector for finding presupposition triggers.
    detector: PresuppositionDetector,
    /// All tracked presuppositions.
    tracked: Vec<TrackedPresupposition>,
}

impl PresuppositionManager {
    /// Create a new manager.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Detect and track presuppositions from tokens.
    pub fn detect_from_tokens(&mut self, tokens: &[String], sentence: usize) {
        let detected = self.detector.detect(tokens, sentence);
        self.tracked.extend(detected);
    }

    /// Try to resolve a presupposition against the DRS.
    ///
    /// Returns the updated status.
    pub fn resolve(&mut self, index: usize, drs: &Drs) -> PresuppositionStatus {
        if index >= self.tracked.len() {
            return PresuppositionStatus::Failed;
        }

        let presup = &self.tracked[index];
        let status = match &presup.presupposition {
            Presupposition::Existential { description, .. } => {
                // Check if there's a matching referent in DRS
                if Self::drs_contains_entity(drs, description) {
                    PresuppositionStatus::Satisfied
                } else {
                    // Can accommodate existence presupposition
                    PresuppositionStatus::Accommodated
                }
            }
            Presupposition::Factive { proposition: _, .. } => {
                // Factive presuppositions are typically accommodated
                // unless explicitly contradicted
                PresuppositionStatus::Accommodated
            }
            Presupposition::Iterative { prior_event, .. } => {
                // Check if the event occurred before
                if Self::drs_contains_event(drs, prior_event) {
                    PresuppositionStatus::Satisfied
                } else {
                    PresuppositionStatus::Accommodated
                }
            }
            Presupposition::Change { activity: _, .. } => {
                // Change presuppositions require the prior state
                PresuppositionStatus::Accommodated
            }
            Presupposition::Cleft { .. } => {
                // Cleft presuppositions are typically accommodated
                PresuppositionStatus::Accommodated
            }
        };

        self.tracked[index].status = status;
        status
    }

    /// Resolve all pending presuppositions.
    pub fn resolve_all(&mut self, drs: &Drs) {
        for i in 0..self.tracked.len() {
            if self.tracked[i].status == PresuppositionStatus::Pending {
                self.resolve(i, drs);
            }
        }
    }

    /// Check if DRS contains an entity matching the description.
    fn drs_contains_entity(drs: &Drs, description: &str) -> bool {
        // Simplified: check if any predicate matches the description
        drs.conditions.iter().any(|c| {
            if let DrsCondition::Predicate { name, .. } = c {
                name.eq_ignore_ascii_case(description)
                    || description.contains(name.as_str())
                    || name.contains(description.split_whitespace().next().unwrap_or(""))
            } else {
                false
            }
        })
    }

    /// Check if DRS contains an event matching the description.
    fn drs_contains_event(drs: &Drs, event_desc: &str) -> bool {
        drs.conditions.iter().any(|c| {
            if let DrsCondition::Predicate { name, .. } = c {
                event_desc.contains(name.as_str()) || name.eq_ignore_ascii_case(event_desc)
            } else {
                false
            }
        })
    }

    /// Mark a presupposition as cancelled.
    pub fn cancel(&mut self, index: usize, reason: &str) {
        if index < self.tracked.len() {
            self.tracked[index].status = PresuppositionStatus::Cancelled;
            self.tracked[index].resolution_note = Some(reason.to_string());
        }
    }

    /// Get all tracked presuppositions.
    #[must_use]
    pub fn all(&self) -> &[TrackedPresupposition] {
        &self.tracked
    }

    /// Get presuppositions with a specific status.
    #[must_use]
    pub fn with_status(&self, status: PresuppositionStatus) -> Vec<&TrackedPresupposition> {
        self.tracked.iter().filter(|p| p.status == status).collect()
    }

    /// Get count of presuppositions by status.
    #[must_use]
    pub fn count_by_status(&self, status: PresuppositionStatus) -> usize {
        self.tracked.iter().filter(|p| p.status == status).count()
    }

    /// Get presuppositions from a specific sentence.
    #[must_use]
    pub fn from_sentence(&self, sentence: usize) -> Vec<&TrackedPresupposition> {
        self.tracked
            .iter()
            .filter(|p| p.triggered_at == sentence)
            .collect()
    }

    /// Get the detector for direct access.
    #[must_use]
    pub fn detector(&self) -> &PresuppositionDetector {
        &self.detector
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_factive_detection() {
        let detector = PresuppositionDetector::new();
        assert!(detector.is_factive("know"));
        assert!(detector.is_factive("Realize"));
        assert!(detector.is_factive("REGRET"));
        assert!(!detector.is_factive("think"));
        assert!(!detector.is_factive("believe"));
    }

    #[test]
    fn test_change_verb_detection() {
        let detector = PresuppositionDetector::new();

        assert_eq!(detector.is_change_verb("stop"), Some(PriorState::WasActive));
        assert_eq!(
            detector.is_change_verb("started"),
            Some(PriorState::WasInactive)
        );
        assert_eq!(
            detector.is_change_verb("continue"),
            Some(PriorState::WasActive)
        );
        assert_eq!(detector.is_change_verb("walk"), None);
    }

    #[test]
    fn test_iterative_detection() {
        let detector = PresuppositionDetector::new();
        assert!(detector.is_iterative("again"));
        assert!(detector.is_iterative("return"));
        assert!(detector.is_iterative("rebuild")); // re- prefix
        assert!(!detector.is_iterative("build"));
    }

    #[test]
    fn test_definite_description() {
        assert!(PresuppositionDetector::is_definite_description(&[
            "the".to_string(),
            "cat".to_string()
        ]));
        assert!(PresuppositionDetector::is_definite_description(&[
            "The".to_string(),
            "big".to_string(),
            "dog".to_string()
        ]));
        assert!(!PresuppositionDetector::is_definite_description(&[
            "a".to_string(),
            "cat".to_string()
        ]));
        assert!(!PresuppositionDetector::is_definite_description(&[]));
    }

    #[test]
    fn test_detect_factive_presupposition() {
        let detector = PresuppositionDetector::new();
        let tokens = vec![
            "John".to_string(),
            "knew".to_string(),
            "that".to_string(),
            "Mary".to_string(),
            "was".to_string(),
            "there".to_string(),
        ];

        let presups = detector.detect(&tokens, 0);

        assert!(presups.iter().any(
            |p| matches!(&p.presupposition, Presupposition::Factive { verb, .. } if verb == "knew")
        ));
    }

    #[test]
    fn test_detect_change_presupposition() {
        let detector = PresuppositionDetector::new();
        let tokens = vec![
            "He".to_string(),
            "stopped".to_string(),
            "smoking".to_string(),
        ];

        let presups = detector.detect(&tokens, 0);

        assert!(presups.iter().any(|p| matches!(
            &p.presupposition,
            Presupposition::Change { verb, prior_state, .. }
            if verb == "stopped" && *prior_state == PriorState::WasActive
        )));
    }

    #[test]
    fn test_detect_iterative_presupposition() {
        let detector = PresuppositionDetector::new();
        let tokens = vec![
            "She".to_string(),
            "visited".to_string(),
            "again".to_string(),
        ];

        let presups = detector.detect(&tokens, 0);

        assert!(presups.iter().any(|p| matches!(
            &p.presupposition,
            Presupposition::Iterative { trigger, .. } if trigger == "again"
        )));
    }

    #[test]
    fn test_detect_existential_presupposition() {
        let detector = PresuppositionDetector::new();
        let tokens = vec![
            "The".to_string(),
            "king".to_string(),
            "of".to_string(),
            "France".to_string(),
            "is".to_string(),
            "bald".to_string(),
        ];

        let presups = detector.detect(&tokens, 0);

        assert!(presups.iter().any(|p| matches!(
            &p.presupposition,
            Presupposition::Existential { description, .. } if description.contains("king")
        )));
    }

    #[test]
    fn test_manager_tracking() {
        let mut manager = PresuppositionManager::new();
        let tokens = vec![
            "John".to_string(),
            "stopped".to_string(),
            "smoking".to_string(),
        ];

        manager.detect_from_tokens(&tokens, 0);

        assert!(!manager.all().is_empty());
        assert_eq!(
            manager.count_by_status(PresuppositionStatus::Pending),
            manager.all().len()
        );
    }

    #[test]
    fn test_presupposition_status() {
        let mut manager = PresuppositionManager::new();
        let tokens = vec!["The".to_string(), "cat".to_string(), "sat".to_string()];

        manager.detect_from_tokens(&tokens, 0);

        // Should have existential presupposition for "the cat"
        assert!(!manager
            .with_status(PresuppositionStatus::Pending)
            .is_empty());

        // Cancel one
        if !manager.all().is_empty() {
            manager.cancel(0, "Test cancellation");
            assert_eq!(manager.count_by_status(PresuppositionStatus::Cancelled), 1);
        }
    }
}
