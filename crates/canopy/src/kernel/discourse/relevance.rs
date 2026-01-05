use super::qud::{QudIssue, QudOrigin};
use crate::core::ThetaRole;
use crate::kernel::events::{ComposedEvent, ComposedEvents};
use serde::{Deserialize, Serialize};

/// Level of relevance between an assertion and the active QUD.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelevanceLevel {
    /// Completely answers the question.
    Direct,
    /// Answers part of the question (fills some expectations).
    Partial,
    /// Does not help resolve the question.
    OffTopic,
    /// No active QUD to compare against.
    NoQuestion,
}

impl RelevanceLevel {
    fn rank(self) -> u8 {
        match self {
            RelevanceLevel::Direct => 3,
            RelevanceLevel::Partial => 2,
            RelevanceLevel::OffTopic => 1,
            RelevanceLevel::NoQuestion => 0,
        }
    }
}

/// Alignment between a single event and the current QUD.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelevanceAlignment {
    /// Event index in the sentence.
    pub event_id: usize,
    /// Predicate lemma.
    pub predicate: String,
    /// Alignment tier.
    pub level: RelevanceLevel,
    /// Roles that matched the QUD focus.
    pub matched_roles: Vec<ThetaRole>,
}

/// Relevance assessment for an entire sentence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelevanceReport {
    /// Sentence index in the discourse.
    pub sentence_index: usize,
    /// ID of the active question (if any).
    pub question_id: Option<usize>,
    /// Human-friendly question text.
    pub question: Option<String>,
    /// Summary level.
    pub level: RelevanceLevel,
    /// Per-event alignments.
    pub alignments: Vec<RelevanceAlignment>,
}

/// Simple heuristic scorer comparing events to the active QUD.
pub struct RelevanceScorer;

impl RelevanceScorer {
    /// Create a report for the provided events/QUD context.
    #[must_use]
    pub fn score(
        sentence_index: usize,
        question: Option<&QudIssue>,
        events: &ComposedEvents,
    ) -> RelevanceReport {
        match question {
            Some(issue) => Self::score_with_question(sentence_index, issue, events),
            None => Self::no_question_report(sentence_index, events),
        }
    }

    fn score_with_question(
        sentence_index: usize,
        issue: &QudIssue,
        events: &ComposedEvents,
    ) -> RelevanceReport {
        let mut alignments = Vec::new();
        let mut best = RelevanceLevel::OffTopic;

        for event in &events.events {
            // Compute level and matched_roles together to avoid duplicate iteration
            let (level, matched_roles) = Self::score_event(issue, event);
            if level.rank() > best.rank() {
                best = level;
            }
            alignments.push(RelevanceAlignment {
                event_id: event.id,
                predicate: event.predicate.clone(),
                level,
                matched_roles,
            });
        }

        if alignments.is_empty() {
            best = RelevanceLevel::OffTopic;
        }

        RelevanceReport {
            sentence_index,
            question_id: Some(issue.id),
            question: Some(issue.question.clone()),
            level: best,
            alignments,
        }
    }

    fn no_question_report(sentence_index: usize, events: &ComposedEvents) -> RelevanceReport {
        let alignments = events
            .events
            .iter()
            .map(|event| RelevanceAlignment {
                event_id: event.id,
                predicate: event.predicate.clone(),
                level: RelevanceLevel::NoQuestion,
                matched_roles: Vec::new(),
            })
            .collect();

        RelevanceReport {
            sentence_index,
            question_id: None,
            question: None,
            level: RelevanceLevel::NoQuestion,
            alignments,
        }
    }

    /// Score an event against a QUD issue, returning both the relevance level
    /// and the matched roles in a single pass.
    ///
    /// Scoring priority:
    /// 1. Predicate match + all/required roles → Direct
    /// 2. Predicate match + some roles → Partial
    /// 3. `ImplicitMissingArgument` + filler match → Direct/Partial
    /// 4. Any role match (no predicate) → Partial (permissive: captures indirect answers)
    /// 5. No matches → `OffTopic`
    ///
    /// Note: Step 4 is intentionally permissive to capture potential indirect answers
    /// where an entity mentioned in the question appears in a different predicate context.
    fn score_event(issue: &QudIssue, event: &ComposedEvent) -> (RelevanceLevel, Vec<ThetaRole>) {
        let predicate_match = issue.matches_event(event);
        let matched_roles: Vec<_> = issue
            .focus_roles
            .iter()
            .copied()
            .filter(|role| event.has_role(*role))
            .collect();
        let expected_roles = issue.focus_roles.len();

        let level = if predicate_match
            && (expected_roles == 0 || matched_roles.len() == expected_roles)
        {
            RelevanceLevel::Direct
        } else if predicate_match && !matched_roles.is_empty() {
            RelevanceLevel::Partial
        } else if issue.origin == QudOrigin::ImplicitMissingArgument && issue.filler_matches(event)
        {
            if matched_roles.len() == expected_roles && expected_roles > 0 {
                RelevanceLevel::Direct
            } else {
                RelevanceLevel::Partial
            }
        } else if !matched_roles.is_empty() {
            RelevanceLevel::Partial
        } else {
            RelevanceLevel::OffTopic
        };

        (level, matched_roles)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{AspectualClass, Voice};
    use crate::kernel::events::{LittleVType, Participant};
    use crate::runtime::TokenId;
    use std::collections::HashMap;

    fn make_issue(origin: QudOrigin) -> QudIssue {
        QudIssue {
            id: 1,
            introduced_at: 0,
            question: "Who left?".to_string(),
            origin,
            predicate_focus: Some("leave".to_string()),
            focus_roles: vec![ThetaRole::Agent],
            expected_filler: None,
            status: super::super::qud::QudStatus::Open,
            question_type: Some(super::super::moves::QuestionType::Who),
            wh_word: Some("Who".to_string()),
            parent_id: None,
            partial_answers: Vec::new(),
        }
    }

    fn make_event(predicate: &str, role: ThetaRole, filler: &str, id: usize) -> ComposedEvent {
        let mut participants = HashMap::new();
        participants.insert(role, Participant::new(TokenId::new(0), filler));

        ComposedEvent {
            id,
            predicate: predicate.to_string(),
            little_v_type: LittleVType::Go,
            participants,
            aspect: AspectualClass::Activity,
            voice: Voice::Active,
            token_span: (TokenId::new(0), TokenId::new(1)),
            source_sense: None,
            decomposition_confidence: 1.0,
            binding_confidence: 1.0,
            presuppositions: Vec::new(),
            polarity: true,
        }
    }

    fn make_events(event: ComposedEvent) -> ComposedEvents {
        ComposedEvents {
            events: vec![event],
            unbound_participants: Vec::new(),
            confidence: 1.0,
            sources: Vec::new(),
        }
    }

    #[test]
    fn test_direct_match() {
        let issue = make_issue(QudOrigin::ExplicitInterrogative);
        let event = make_event("leave", ThetaRole::Agent, "John", 0);
        let events = make_events(event);
        let report = RelevanceScorer::score(0, Some(&issue), &events);
        assert_eq!(report.level, RelevanceLevel::Direct);
    }

    #[test]
    fn test_partial_match() {
        let issue = make_issue(QudOrigin::ExplicitInterrogative);
        let event = make_event("run", ThetaRole::Agent, "John", 0);
        let events = make_events(event);
        let report = RelevanceScorer::score(0, Some(&issue), &events);
        assert_eq!(report.level, RelevanceLevel::Partial);
    }

    #[test]
    fn test_off_topic() {
        let issue = make_issue(QudOrigin::ExplicitInterrogative);
        let mut event = make_event("describe", ThetaRole::Experiencer, "Mary", 0);
        event.participants.clear();
        let events = make_events(event);
        let report = RelevanceScorer::score(0, Some(&issue), &events);
        assert_eq!(report.level, RelevanceLevel::OffTopic);
    }

    #[test]
    fn test_no_question() {
        let event = make_event("leave", ThetaRole::Agent, "John", 0);
        let events = make_events(event);
        let report = RelevanceScorer::score(0, None, &events);
        assert_eq!(report.level, RelevanceLevel::NoQuestion);
    }
}
