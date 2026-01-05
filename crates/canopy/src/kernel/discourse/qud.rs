use super::moves::QuestionType;
use crate::core::ThetaRole;
use crate::kernel::events::{ComposedEvent, ComposedEvents};
use crate::runtime::AnnotatedSyntax;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Origin of a question under discussion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum QudOrigin {
    /// A surface-form interrogative (ends with '?').
    #[default]
    ExplicitInterrogative,
    /// Raised because a semantic participant could not be bound.
    ImplicitMissingArgument,
}

/// Lifecycle state of a QUD issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum QudStatus {
    /// Awaiting an answer.
    #[default]
    Open,
    /// Resolved by a later assertion.
    Resolved,
}

/// Update emitted when the QUD stack changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QudUpdateAction {
    /// New question pushed on the stack.
    Pushed,
    /// Question resolved and popped.
    Resolved,
}

/// A single QUD issue.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QudIssue {
    /// Identifier assigned by the manager.
    pub id: usize,
    /// Sentence index where the issue was introduced.
    pub introduced_at: usize,
    /// Human-friendly description of the question.
    pub question: String,
    /// Where the question came from.
    pub origin: QudOrigin,
    /// Predicate focus, if we were able to recover it.
    pub predicate_focus: Option<String>,
    /// Roles we expect to be filled to mark resolution.
    pub focus_roles: Vec<ThetaRole>,
    /// Specific filler text we are looking for (for implicit questions).
    pub expected_filler: Option<String>,
    /// Tracking whether it has been answered.
    pub status: QudStatus,
    /// Type of question (wh-word, yes/no, etc.).
    pub question_type: Option<QuestionType>,
    /// The wh-word that triggered this question (if any).
    pub wh_word: Option<String>,
    /// Parent question ID (for sub-questions).
    pub parent_id: Option<usize>,
    /// Partial answers received so far.
    pub partial_answers: Vec<PartialAnswer>,
}

/// A partial answer to a question.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialAnswer {
    /// Sentence index where this partial answer was given.
    pub sentence: usize,
    /// The content of the partial answer.
    pub content: String,
    /// How much of the question this addresses (0.0 - 1.0).
    pub completeness: f32,
}

impl QudIssue {
    pub(crate) fn matches_event(&self, event: &ComposedEvent) -> bool {
        self.predicate_focus
            .as_ref()
            .is_none_or(|p| event.predicate.eq_ignore_ascii_case(p))
    }

    pub(crate) fn filler_matches(&self, event: &ComposedEvent) -> bool {
        if let Some(expected) = &self.expected_filler {
            event
                .participants
                .values()
                .any(|p| p.text.eq_ignore_ascii_case(expected))
        } else {
            true
        }
    }

    fn resolved_by(&self, events: &ComposedEvents) -> bool {
        match self.origin {
            QudOrigin::ExplicitInterrogative => {
                events.events.iter().any(|event| self.matches_event(event))
            }
            QudOrigin::ImplicitMissingArgument => events.events.iter().any(|event| {
                self.matches_event(event)
                    && self.filler_matches(event)
                    && self.focus_roles.iter().all(|role| event.has_role(*role))
            }),
        }
    }
}

/// A concrete change emitted by the QUD manager.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QudUpdate {
    /// Snapshot of the affected issue.
    pub issue: QudIssue,
    /// What happened to it.
    pub action: QudUpdateAction,
}

impl QudUpdate {
    fn pushed(issue: QudIssue) -> Self {
        Self {
            issue,
            action: QudUpdateAction::Pushed,
        }
    }

    fn resolved(issue: QudIssue) -> Self {
        Self {
            issue,
            action: QudUpdateAction::Resolved,
        }
    }
}

/// Compact report summarizing QUD activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QudReportEntry {
    /// Identifier of the issue.
    pub issue_id: usize,
    /// Action that occurred.
    pub action: QudUpdateAction,
    /// Question text.
    pub question: String,
    /// Origin type.
    pub origin: QudOrigin,
}

/// Summary of the QUD stack for trace output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QudReport {
    /// Current stack depth.
    pub stack_depth: usize,
    /// Question on top of the stack (if any).
    pub active_question: Option<String>,
    /// Chronological update history.
    pub history: Vec<QudReportEntry>,
    /// Tree structure showing sub-question relationships.
    pub tree: QudTreeInfo,
}

/// Tree structure information for QUD reporting.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QudTreeInfo {
    /// IDs of root questions (no parent).
    pub root_ids: Vec<usize>,
    /// Mapping of parent ID to child IDs.
    pub children: HashMap<usize, Vec<usize>>,
    /// All questions with their hierarchy depth.
    pub questions: Vec<QudTreeNode>,
}

/// A node in the QUD tree visualization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QudTreeNode {
    /// Question ID.
    pub id: usize,
    /// Question text.
    pub question: String,
    /// Status (open/resolved).
    pub status: QudStatus,
    /// Depth in tree (0 = root).
    pub depth: usize,
    /// Parent question ID if any.
    pub parent_id: Option<usize>,
}

impl QudReport {
    fn from_state(stack: &QudStack, history: &[QudUpdate]) -> Self {
        let active_question = stack.peek().map(|issue| issue.question.clone());

        // Collect all unique issues from history for tree building
        let mut all_issues: HashMap<usize, QudIssue> = HashMap::new();
        for update in history {
            all_issues.insert(update.issue.id, update.issue.clone());
        }
        // Also include current stack issues (may have partial answers not yet in history)
        for issue in &stack.issues {
            all_issues.insert(issue.id, issue.clone());
        }

        // Build tree info
        let issues_vec: Vec<&QudIssue> = all_issues.values().collect();
        let tree = QudTreeInfo::from_issues(&issues_vec);

        let history_entries = history
            .iter()
            .map(|update| QudReportEntry {
                issue_id: update.issue.id,
                action: update.action,
                question: update.issue.question.clone(),
                origin: update.issue.origin,
            })
            .collect();

        Self {
            stack_depth: stack.len(),
            active_question,
            history: history_entries,
            tree,
        }
    }
}

impl QudTreeInfo {
    /// Build tree info from a collection of issues.
    fn from_issues(issues: &[&QudIssue]) -> Self {
        let mut root_ids = Vec::new();
        let mut children: HashMap<usize, Vec<usize>> = HashMap::new();
        let mut questions = Vec::new();

        for issue in issues {
            if let Some(parent_id) = issue.parent_id {
                children.entry(parent_id).or_default().push(issue.id);
            } else {
                root_ids.push(issue.id);
            }
        }

        // Sort for deterministic output
        root_ids.sort_unstable();
        for children_list in children.values_mut() {
            children_list.sort_unstable();
        }

        // Build nodes with depth information
        for issue in issues {
            let depth = Self::compute_depth(issue.id, issues);
            questions.push(QudTreeNode {
                id: issue.id,
                question: issue.question.clone(),
                status: issue.status,
                depth,
                parent_id: issue.parent_id,
            });
        }

        // Sort questions by id for deterministic output
        questions.sort_by_key(|q| q.id);

        Self {
            root_ids,
            children,
            questions,
        }
    }

    /// Compute the depth of a question in the tree.
    fn compute_depth(id: usize, issues: &[&QudIssue]) -> usize {
        let issue = issues.iter().find(|i| i.id == id);
        match issue.and_then(|i| i.parent_id) {
            Some(parent_id) => 1 + Self::compute_depth(parent_id, issues),
            None => 0,
        }
    }
}

/// Default maximum depth for the QUD stack.
pub const DEFAULT_QUD_STACK_DEPTH: usize = 100;

/// Stack of QUD issues with detection helpers.
#[derive(Debug, Clone)]
pub struct QudStack {
    issues: Vec<QudIssue>,
    next_id: usize,
    /// Maximum allowed stack depth to prevent unbounded growth.
    max_depth: usize,
}

impl Default for QudStack {
    fn default() -> Self {
        Self {
            issues: Vec::new(),
            next_id: 0,
            max_depth: DEFAULT_QUD_STACK_DEPTH,
        }
    }
}

impl QudStack {
    /// Observe cues from the incoming sentence and push new questions if needed.
    ///
    /// Implicit questions (from unbound arguments) are pushed first, then
    /// explicit questions (interrogatives ending with '?') are pushed on top,
    /// ensuring explicit questions have higher priority for resolution.
    ///
    /// Questions are silently dropped if the stack depth limit is reached.
    pub fn observe_sentence(
        &mut self,
        sentence_index: usize,
        syntax: &AnnotatedSyntax,
        events: Option<&ComposedEvents>,
    ) -> Vec<QudUpdate> {
        let mut updates = Vec::new();

        // Implicit questions first (lower priority, pushed to bottom of new questions)
        if let Some(events) = events {
            updates.extend(self.detect_implicit_questions(events, sentence_index));
        }

        // Explicit questions last (higher priority, pushed on top)
        if let Some(mut issue) = Self::detect_explicit_question(syntax) {
            if let Some(update) = self.push_issue(&mut issue, sentence_index) {
                updates.push(update);
            }
        }

        updates
    }

    /// Resolve the top-of-stack question if the new assertions answer it.
    pub fn resolve_with_events(
        &mut self,
        events: &ComposedEvents,
        current_sentence: usize,
    ) -> Vec<QudUpdate> {
        let mut updates = Vec::new();

        loop {
            // Check if top issue can be resolved
            let should_pop = self.issues.last().is_some_and(|issue| {
                issue.introduced_at < current_sentence && issue.resolved_by(events)
            });

            if !should_pop {
                break;
            }

            // Pop is safe because we just verified the issue exists
            if let Some(mut resolved) = self.issues.pop() {
                resolved.status = QudStatus::Resolved;
                updates.push(QudUpdate::resolved(resolved));
            }
        }

        updates
    }

    /// Current stack depth.
    #[must_use]
    pub fn len(&self) -> usize {
        self.issues.len()
    }

    /// Whether the stack currently has no active QUDs.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.issues.is_empty()
    }

    /// Peek at the active issue.
    #[must_use]
    pub fn peek(&self) -> Option<&QudIssue> {
        self.issues.last()
    }

    /// Produce a snapshot suitable for trace output.
    #[must_use]
    pub fn report(&self, history: &[QudUpdate]) -> QudReport {
        QudReport::from_state(self, history)
    }

    fn push_issue(&mut self, issue: &mut QudIssue, sentence_index: usize) -> Option<QudUpdate> {
        // Prevent unbounded stack growth
        if self.issues.len() >= self.max_depth {
            return None;
        }

        // Skip duplicate questions that are still open
        if self
            .issues
            .iter()
            .any(|e| e.question == issue.question && e.status == QudStatus::Open)
        {
            return None;
        }

        issue.id = self.next_id;
        issue.introduced_at = sentence_index;
        self.next_id += 1;
        let update = QudUpdate::pushed(issue.clone());
        self.issues.push(std::mem::take(issue));
        Some(update)
    }

    fn detect_explicit_question(syntax: &AnnotatedSyntax) -> Option<QudIssue> {
        let trimmed = syntax.text.trim();
        if !trimmed.ends_with('?') {
            return None;
        }

        let predicate_focus = syntax.root().map(|token| token.lemma.clone());

        // Detect question type from tokens
        let (question_type, wh_word) = Self::detect_question_type(syntax);

        // Map question type to expected theta roles
        let focus_roles = question_type
            .map(Self::expected_roles_for_question)
            .unwrap_or_default();

        Some(QudIssue {
            id: 0,
            introduced_at: 0,
            question: trimmed.to_string(),
            origin: QudOrigin::ExplicitInterrogative,
            predicate_focus,
            focus_roles,
            expected_filler: None,
            status: QudStatus::Open,
            question_type,
            wh_word,
            parent_id: None,
            partial_answers: Vec::new(),
        })
    }

    /// Detect question type and wh-word from syntax.
    fn detect_question_type(syntax: &AnnotatedSyntax) -> (Option<QuestionType>, Option<String>) {
        // Check first few tokens for wh-words
        for token in syntax.tokens.iter().take(3) {
            let lower = token.lemma.to_lowercase();

            // Check for wh-word
            if let Some(qt) = QuestionType::from_wh_word(&lower) {
                return (Some(qt), Some(token.form.clone()));
            }

            // Check for "how many" / "how much"
            if lower == "how" {
                // Look ahead for many/much
                if let Some(next) = syntax.tokens.get(1) {
                    let next_lower = next.lemma.to_lowercase();
                    if next_lower == "many" {
                        return (Some(QuestionType::HowMany), Some("how many".to_string()));
                    }
                    if next_lower == "much" {
                        return (Some(QuestionType::HowMuch), Some("how much".to_string()));
                    }
                }
                return (Some(QuestionType::How), Some(token.form.clone()));
            }
        }

        // Check for yes/no question (auxiliary at start)
        let aux_words = [
            "is", "are", "was", "were", "do", "does", "did", "can", "could", "will", "would",
            "have", "has", "had",
        ];
        if let Some(first) = syntax.tokens.first() {
            if aux_words.contains(&first.lemma.to_lowercase().as_str()) {
                return (Some(QuestionType::YesNo), None);
            }
        }

        // Check for alternative question (contains "or")
        if syntax
            .tokens
            .iter()
            .any(|t| t.form.eq_ignore_ascii_case("or"))
        {
            return (Some(QuestionType::Alternative), None);
        }

        (None, None)
    }

    /// Map question type to expected theta roles for the answer.
    fn expected_roles_for_question(qt: QuestionType) -> Vec<ThetaRole> {
        match qt {
            QuestionType::Who => vec![ThetaRole::Agent, ThetaRole::Experiencer],
            QuestionType::What | QuestionType::Which => vec![ThetaRole::Theme, ThetaRole::Patient],
            QuestionType::Where => vec![ThetaRole::Location, ThetaRole::Goal],
            QuestionType::When => vec![ThetaRole::Temporal],
            QuestionType::Why => vec![ThetaRole::Cause],
            QuestionType::How => vec![ThetaRole::Manner],
            QuestionType::HowMany | QuestionType::HowMuch => vec![ThetaRole::Measure],
            // No specific theta roles for these question types
            QuestionType::Whose
            | QuestionType::YesNo
            | QuestionType::Alternative
            | QuestionType::Embedded => vec![],
        }
    }

    fn detect_implicit_questions(
        &mut self,
        events: &ComposedEvents,
        sentence_index: usize,
    ) -> Vec<QudUpdate> {
        let mut updates = Vec::new();

        for missing in &events.unbound_participants {
            let mut issue = QudIssue {
                id: 0,
                introduced_at: 0,
                question: format!("Where does \"{}\" attach in this discourse?", missing.text),
                origin: QudOrigin::ImplicitMissingArgument,
                predicate_focus: None,
                focus_roles: missing
                    .suggested_role
                    .into_iter()
                    .collect::<Vec<ThetaRole>>(),
                expected_filler: Some(missing.text.clone()),
                status: QudStatus::Open,
                question_type: None, // Implicit questions don't have surface form
                wh_word: None,
                parent_id: None,
                partial_answers: Vec::new(),
            };

            if let Some(update) = self.push_issue(&mut issue, sentence_index) {
                updates.push(update);
            }
        }

        updates
    }

    /// Record a partial answer to the active question.
    pub fn record_partial_answer(&mut self, content: String, completeness: f32, sentence: usize) {
        if let Some(issue) = self.issues.last_mut() {
            issue.partial_answers.push(PartialAnswer {
                sentence,
                content,
                completeness,
            });
        }
    }

    /// Create a sub-question under the current active question.
    pub fn create_subquestion(
        &mut self,
        question: String,
        question_type: Option<QuestionType>,
        sentence_index: usize,
    ) -> Option<QudUpdate> {
        let parent_id = self.issues.last().map(|i| i.id);

        let mut issue = QudIssue {
            id: 0,
            introduced_at: 0,
            question,
            origin: QudOrigin::ExplicitInterrogative,
            predicate_focus: None,
            focus_roles: question_type
                .map(Self::expected_roles_for_question)
                .unwrap_or_default(),
            expected_filler: None,
            status: QudStatus::Open,
            question_type,
            wh_word: None,
            parent_id,
            partial_answers: Vec::new(),
        };

        self.push_issue(&mut issue, sentence_index)
    }
}

/// Hierarchical tree of questions under discussion.
///
/// Represents sub-question relationships where answering one question
/// may require first answering related sub-questions.
#[derive(Debug, Clone, Default)]
pub struct QudTree {
    /// Mapping from question ID to its children.
    children: HashMap<usize, Vec<usize>>,
    /// Root questions (no parent).
    roots: Vec<usize>,
}

impl QudTree {
    /// Create a new empty QUD tree.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Build tree structure from a list of QUD issues.
    #[must_use]
    pub fn from_issues(issues: &[QudIssue]) -> Self {
        let mut tree = Self::new();

        for issue in issues {
            if let Some(parent_id) = issue.parent_id {
                tree.children.entry(parent_id).or_default().push(issue.id);
            } else {
                tree.roots.push(issue.id);
            }
        }

        tree
    }

    /// Get child questions of a given question.
    #[must_use]
    pub fn children_of(&self, id: usize) -> &[usize] {
        self.children.get(&id).map_or(&[], Vec::as_slice)
    }

    /// Get root questions (no parent).
    #[must_use]
    pub fn roots(&self) -> &[usize] {
        &self.roots
    }

    /// Check if a question has any sub-questions.
    #[must_use]
    pub fn has_children(&self, id: usize) -> bool {
        self.children.get(&id).is_some_and(|c| !c.is_empty())
    }

    /// Get the depth of a question in the tree.
    #[must_use]
    pub fn depth(id: usize, issues: &[QudIssue]) -> usize {
        let issue = issues.iter().find(|i| i.id == id);
        match issue.and_then(|i| i.parent_id) {
            Some(parent_id) => 1 + Self::depth(parent_id, issues),
            None => 0,
        }
    }
}

impl QudReportEntry {
    #[must_use]
    pub fn action_label(&self) -> &'static str {
        match self.action {
            QudUpdateAction::Pushed => "PUSH",
            QudUpdateAction::Resolved => "RESOLVE",
        }
    }

    #[must_use]
    pub fn origin_label(&self) -> &'static str {
        match self.origin {
            QudOrigin::ExplicitInterrogative => "explicit",
            QudOrigin::ImplicitMissingArgument => "implicit",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::{AnnotatedSyntax, AnnotatedToken, TokenId};
    use crate::DepRel;

    fn make_syntax(text: &str) -> AnnotatedSyntax {
        AnnotatedSyntax::new(
            text.to_string(),
            vec![AnnotatedToken::new(
                TokenId::new(0),
                text.to_string(),
                text.to_lowercase(),
                crate::UPos::Verb,
                DepRel::Root,
                (0, text.len()),
            )],
        )
    }

    #[test]
    fn test_qud_stack_default() {
        let stack = QudStack::default();
        assert!(stack.is_empty());
        assert_eq!(stack.len(), 0);
        assert!(stack.peek().is_none());
    }

    #[test]
    fn test_explicit_question_detection() {
        let mut stack = QudStack::default();
        let syntax = make_syntax("Who left?");
        let updates = stack.observe_sentence(0, &syntax, None);

        assert_eq!(updates.len(), 1);
        assert_eq!(updates[0].issue.origin, QudOrigin::ExplicitInterrogative);
        assert_eq!(stack.len(), 1);
    }

    #[test]
    fn test_duplicate_question_ignored() {
        let mut stack = QudStack::default();
        let syntax = make_syntax("Who left?");

        // First question should be pushed
        let updates1 = stack.observe_sentence(0, &syntax, None);
        assert_eq!(updates1.len(), 1);
        assert_eq!(stack.len(), 1);

        // Same question again should be ignored
        let updates2 = stack.observe_sentence(1, &syntax, None);
        assert_eq!(updates2.len(), 0);
        assert_eq!(stack.len(), 1); // Still only 1
    }

    #[test]
    fn test_stack_depth_limit() {
        let mut stack = QudStack {
            issues: Vec::new(),
            next_id: 0,
            max_depth: 2, // Small limit for testing
        };

        // Push 2 different questions
        let syntax1 = make_syntax("Who left?");
        let syntax2 = make_syntax("Where went?");
        let syntax3 = make_syntax("What happened?");

        let updates1 = stack.observe_sentence(0, &syntax1, None);
        assert_eq!(updates1.len(), 1);

        let updates2 = stack.observe_sentence(1, &syntax2, None);
        assert_eq!(updates2.len(), 1);

        // Third should be rejected due to depth limit
        let updates3 = stack.observe_sentence(2, &syntax3, None);
        assert_eq!(updates3.len(), 0);
        assert_eq!(stack.len(), 2);
    }

    #[test]
    fn test_qud_report_entry_labels() {
        let entry = QudReportEntry {
            issue_id: 1,
            action: QudUpdateAction::Pushed,
            question: "Who?".to_string(),
            origin: QudOrigin::ExplicitInterrogative,
        };
        assert_eq!(entry.action_label(), "PUSH");
        assert_eq!(entry.origin_label(), "explicit");

        let entry2 = QudReportEntry {
            issue_id: 2,
            action: QudUpdateAction::Resolved,
            question: "Where?".to_string(),
            origin: QudOrigin::ImplicitMissingArgument,
        };
        assert_eq!(entry2.action_label(), "RESOLVE");
        assert_eq!(entry2.origin_label(), "implicit");
    }

    // Edge case tests

    #[test]
    fn test_empty_stack_operations() {
        let stack = QudStack::default();

        assert!(stack.is_empty());
        assert_eq!(stack.len(), 0);
        assert!(stack.peek().is_none());
    }

    #[test]
    fn test_resolve_on_empty_stack() {
        use crate::core::{AspectualClass, Voice};
        use crate::kernel::events::{ComposedEvent, ComposedEvents, LittleVType};
        use std::collections::HashMap;

        let mut stack = QudStack::default();
        let events = ComposedEvents {
            events: vec![ComposedEvent {
                id: 0,
                predicate: "test".to_string(),
                little_v_type: LittleVType::Do,
                participants: HashMap::new(),
                aspect: AspectualClass::State,
                voice: Voice::Active,
                token_span: (TokenId::new(0), TokenId::new(1)),
                source_sense: None,
                decomposition_confidence: 1.0,
                binding_confidence: 1.0,
                presuppositions: vec![],
                polarity: true,
            }],
            unbound_participants: vec![],
            confidence: 1.0,
            sources: vec![],
        };

        // Should not panic on empty stack
        let updates = stack.resolve_with_events(&events, 1);
        assert!(updates.is_empty());
    }

    #[test]
    fn test_non_question_syntax() {
        let mut stack = QudStack::default();
        let syntax = make_syntax("The cat sat.");

        // Non-question should not push anything
        let updates = stack.observe_sentence(0, &syntax, None);
        assert!(updates.is_empty());
        assert!(stack.is_empty());
    }

    #[test]
    fn test_qud_tree_empty() {
        let tree = QudTree::new();

        assert!(tree.roots().is_empty());
        assert!(tree.children_of(0).is_empty());
        assert!(!tree.has_children(0));
    }

    #[test]
    fn test_qud_tree_info_empty() {
        let info = QudTreeInfo::from_issues(&[]);

        assert!(info.root_ids.is_empty());
        assert!(info.children.is_empty());
        assert!(info.questions.is_empty());
    }

    #[test]
    fn test_partial_answer_recording() {
        let mut stack = QudStack::default();
        let syntax = make_syntax("Who left?");
        stack.observe_sentence(0, &syntax, None);

        // Record a partial answer
        stack.record_partial_answer("John might have".to_string(), 0.5, 1);

        let issue = stack.peek().unwrap();
        assert_eq!(issue.partial_answers.len(), 1);
        assert!((issue.partial_answers[0].completeness - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_partial_answer_on_empty_stack() {
        let mut stack = QudStack::default();

        // Should not panic when no question on stack
        stack.record_partial_answer("Test".to_string(), 0.5, 0);

        // Stack still empty
        assert!(stack.is_empty());
    }

    #[test]
    fn test_create_subquestion() {
        let mut stack = QudStack::default();
        let syntax = make_syntax("Who left?");
        stack.observe_sentence(0, &syntax, None);

        // Create a subquestion
        let update = stack.create_subquestion(
            "When did they leave?".to_string(),
            Some(super::super::moves::QuestionType::When),
            1,
        );

        assert!(update.is_some());
        assert_eq!(stack.len(), 2);

        // The subquestion should have parent_id set
        let sub = stack.peek().unwrap();
        assert!(sub.parent_id.is_some());
    }
}
