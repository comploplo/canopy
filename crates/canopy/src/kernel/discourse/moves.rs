//! Discourse move classification.
//!
//! Classifies each sentence's communicative function in the discourse.
//! Follows speech act theory and discourse structure analysis.

use super::qud::{QudStack, QudStatus};
use super::relevance::RelevanceLevel;
use crate::kernel::events::ComposedEvents;
use crate::runtime::AnnotatedSyntax;
use serde::{Deserialize, Serialize};

/// Discourse move - the communicative function of a sentence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum DiscourseMove {
    /// Declarative statement asserting a proposition.
    #[default]
    Assertion,

    /// Interrogative raising a question (explicit QUD).
    Question,

    /// Response that directly addresses the active QUD.
    Answer,

    /// Adds detail or explanation to a previous assertion.
    Elaboration,

    /// Contradicts or revises a previous assertion.
    Correction,

    /// Confirms understanding (yes, okay, I see).
    Acknowledgment,

    /// Initiates a new topic or discourse segment.
    Opening,

    /// Concludes a topic or discourse segment.
    Closing,
}

impl std::fmt::Display for DiscourseMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Assertion => write!(f, "Assertion"),
            Self::Question => write!(f, "Question"),
            Self::Answer => write!(f, "Answer"),
            Self::Elaboration => write!(f, "Elaboration"),
            Self::Correction => write!(f, "Correction"),
            Self::Acknowledgment => write!(f, "Acknowledgment"),
            Self::Opening => write!(f, "Opening"),
            Self::Closing => write!(f, "Closing"),
        }
    }
}

/// Question type classification based on wh-words and structure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QuestionType {
    // Wh-questions (constituent questions)
    /// "Who did it?" - Agent/experiencer focus
    Who,
    /// "What happened?" - Theme/patient focus
    What,
    /// "Where is it?" - Location focus
    Where,
    /// "When did it happen?" - Time focus
    When,
    /// "Why did it happen?" - Reason/cause focus
    Why,
    /// "How did it happen?" - Manner/method focus
    How,
    /// "Which one?" - Selection from set
    Which,
    /// "Whose is it?" - Possession focus
    Whose,
    /// "How many?" - Quantity focus
    HowMany,
    /// "How much?" - Degree/amount focus
    HowMuch,

    // Polar questions
    /// "Is X true?" - Yes/no question via auxiliary inversion
    YesNo,
    /// "Is it A or B?" - Alternative question
    Alternative,

    // Indirect
    /// "I wonder who..." - Embedded question
    Embedded,
}

impl QuestionType {
    /// Parse a question type from a wh-word.
    #[must_use]
    pub fn from_wh_word(word: &str) -> Option<Self> {
        let lower = word.to_lowercase();
        match lower.as_str() {
            "who" | "whom" => Some(Self::Who),
            "what" => Some(Self::What),
            "where" => Some(Self::Where),
            "when" => Some(Self::When),
            "why" => Some(Self::Why),
            "how" => Some(Self::How),
            "which" => Some(Self::Which),
            "whose" => Some(Self::Whose),
            _ => None,
        }
    }

    /// Check if a word is a wh-word.
    #[must_use]
    pub fn is_wh_word(word: &str) -> bool {
        Self::from_wh_word(word).is_some()
    }

    /// Get expected theta roles for this question type.
    #[must_use]
    pub fn expected_roles(&self) -> &'static [&'static str] {
        match self {
            Self::Who => &["Agent", "Experiencer", "Theme"],
            Self::What => &["Theme", "Patient"],
            Self::Where => &["Location", "Goal", "Source"],
            Self::When => &["Time"],
            Self::Why => &["Cause", "Reason"],
            Self::How => &["Manner", "Instrument"],
            Self::Which => &["Theme"],
            Self::Whose => &["Possessor"],
            Self::HowMany | Self::HowMuch => &["Quantity"],
            Self::YesNo | Self::Alternative | Self::Embedded => &[],
        }
    }
}

/// Result of discourse move classification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveClassification {
    /// The classified discourse move.
    pub move_type: DiscourseMove,

    /// Confidence in the classification (0.0 to 1.0).
    pub confidence: f32,

    /// If this is a question, what type.
    pub question_type: Option<QuestionType>,

    /// The wh-word if detected.
    pub wh_word: Option<String>,

    /// Reason for classification.
    pub reason: String,
}

impl MoveClassification {
    /// Create a new move classification.
    #[must_use]
    pub fn new(move_type: DiscourseMove, confidence: f32, reason: impl Into<String>) -> Self {
        Self {
            move_type,
            confidence,
            question_type: None,
            wh_word: None,
            reason: reason.into(),
        }
    }

    /// Set question type.
    #[must_use]
    pub fn with_question_type(mut self, qt: QuestionType) -> Self {
        self.question_type = Some(qt);
        self
    }

    /// Set wh-word.
    #[must_use]
    pub fn with_wh_word(mut self, word: impl Into<String>) -> Self {
        self.wh_word = Some(word.into());
        self
    }
}

/// Classifier for discourse moves.
#[derive(Debug, Clone, Default)]
pub struct MoveClassifier {
    /// Acknowledgment words/phrases.
    acknowledgment_words: Vec<String>,

    /// Opening markers.
    opening_markers: Vec<String>,

    /// Closing markers.
    closing_markers: Vec<String>,

    /// Auxiliaries that can invert for yes/no questions.
    aux_words: Vec<String>,
}

impl MoveClassifier {
    /// Create a new move classifier with default patterns.
    #[must_use]
    pub fn new() -> Self {
        Self {
            acknowledgment_words: vec![
                "yes".into(),
                "yeah".into(),
                "yep".into(),
                "no".into(),
                "nope".into(),
                "okay".into(),
                "ok".into(),
                "sure".into(),
                "right".into(),
                "exactly".into(),
                "agreed".into(),
                "understood".into(),
            ],
            opening_markers: vec![
                "well".into(),
                "so".into(),
                "now".into(),
                "first".into(),
                "let me".into(),
                "i want to".into(),
                "speaking of".into(),
                "by the way".into(),
            ],
            closing_markers: vec![
                "finally".into(),
                "in conclusion".into(),
                "to summarize".into(),
                "in summary".into(),
                "that's all".into(),
                "anyway".into(),
                "so that's".into(),
            ],
            aux_words: vec![
                "is".into(),
                "are".into(),
                "was".into(),
                "were".into(),
                "do".into(),
                "does".into(),
                "did".into(),
                "have".into(),
                "has".into(),
                "had".into(),
                "can".into(),
                "could".into(),
                "will".into(),
                "would".into(),
                "should".into(),
                "shall".into(),
                "may".into(),
                "might".into(),
                "must".into(),
            ],
        }
    }

    /// Classify the discourse move of a sentence.
    #[must_use]
    pub fn classify(
        &self,
        syntax: &AnnotatedSyntax,
        _events: Option<&ComposedEvents>,
        qud_stack: &QudStack,
        relevance: Option<RelevanceLevel>,
        prev_move: Option<DiscourseMove>,
    ) -> MoveClassification {
        let text = &syntax.text;
        let tokens: Vec<&str> = syntax.tokens.iter().map(|t| t.form.as_str()).collect();

        // Check for question (highest priority)
        if let Some(classification) = self.check_question(&tokens, text) {
            return classification;
        }

        // Check for acknowledgment
        if let Some(classification) = self.check_acknowledgment(&tokens) {
            return classification;
        }

        // Check for answer (if there's an active QUD and we're relevant)
        if let Some(classification) = Self::check_answer(qud_stack, relevance) {
            return classification;
        }

        // Check for correction (contradicts previous)
        if let Some(classification) = Self::check_correction(&tokens, prev_move) {
            return classification;
        }

        // Check for opening
        if let Some(classification) = self.check_opening(&tokens, text) {
            return classification;
        }

        // Check for closing
        if let Some(classification) = self.check_closing(&tokens, text) {
            return classification;
        }

        // Check for elaboration (continues previous topic)
        if let Some(classification) = Self::check_elaboration(&tokens, prev_move) {
            return classification;
        }

        // Default: assertion
        MoveClassification::new(DiscourseMove::Assertion, 0.6, "default declarative")
    }

    /// Check if the sentence is a question.
    fn check_question(&self, tokens: &[&str], text: &str) -> Option<MoveClassification> {
        // Check for question mark
        let has_question_mark = text.trim_end().ends_with('?');

        // Check for sentence-initial wh-word
        if let Some(first_token) = tokens.first() {
            if let Some(qt) = QuestionType::from_wh_word(first_token) {
                let confidence = if has_question_mark { 0.95 } else { 0.8 };

                // Check for "how many" / "how much"
                let qt = if qt == QuestionType::How {
                    if let Some(second) = tokens.get(1) {
                        match second.to_lowercase().as_str() {
                            "many" => QuestionType::HowMany,
                            "much" => QuestionType::HowMuch,
                            _ => qt,
                        }
                    } else {
                        qt
                    }
                } else {
                    qt
                };

                return Some(
                    MoveClassification::new(
                        DiscourseMove::Question,
                        confidence,
                        format!("wh-question ({first_token})"),
                    )
                    .with_question_type(qt)
                    .with_wh_word(first_token.to_string()),
                );
            }

            // Check for auxiliary inversion (yes/no question)
            let first_lower = first_token.to_lowercase();
            if self.aux_words.contains(&first_lower) && has_question_mark {
                return Some(
                    MoveClassification::new(
                        DiscourseMove::Question,
                        0.9,
                        "yes/no question (aux inversion)",
                    )
                    .with_question_type(QuestionType::YesNo),
                );
            }
        }

        // Check for question mark alone (could be echo question or tag question)
        if has_question_mark {
            return Some(MoveClassification::new(
                DiscourseMove::Question,
                0.7,
                "question mark",
            ));
        }

        None
    }

    /// Check if the sentence is an acknowledgment.
    fn check_acknowledgment(&self, tokens: &[&str]) -> Option<MoveClassification> {
        // Very short utterance starting with acknowledgment word
        if tokens.len() <= 3 {
            if let Some(first) = tokens.first() {
                let first_lower = first.to_lowercase();
                if self.acknowledgment_words.contains(&first_lower) {
                    return Some(MoveClassification::new(
                        DiscourseMove::Acknowledgment,
                        0.85,
                        format!("acknowledgment word ({first})"),
                    ));
                }
            }
        }
        None
    }

    /// Check if the sentence is an answer to an active QUD.
    fn check_answer(
        qud_stack: &QudStack,
        relevance: Option<RelevanceLevel>,
    ) -> Option<MoveClassification> {
        // Need an active question
        let active_qud = qud_stack.peek()?;

        // Check if resolved or directly relevant
        if active_qud.status == QudStatus::Resolved {
            return Some(MoveClassification::new(
                DiscourseMove::Answer,
                0.9,
                "resolved active QUD",
            ));
        }

        if let Some(rel) = relevance {
            match rel {
                RelevanceLevel::Direct => {
                    return Some(MoveClassification::new(
                        DiscourseMove::Answer,
                        0.85,
                        "directly relevant to QUD",
                    ));
                }
                RelevanceLevel::Partial => {
                    return Some(MoveClassification::new(
                        DiscourseMove::Answer,
                        0.7,
                        "partially relevant to QUD",
                    ));
                }
                _ => {}
            }
        }

        None
    }

    /// Check if the sentence is a correction.
    fn check_correction(
        tokens: &[&str],
        prev_move: Option<DiscourseMove>,
    ) -> Option<MoveClassification> {
        // Look for correction markers
        let correction_markers = [
            "no",
            "not",
            "actually",
            "but",
            "however",
            "wrong",
            "incorrect",
        ];

        if let Some(first) = tokens.first() {
            let first_lower = first.to_lowercase();
            if correction_markers.contains(&first_lower.as_str()) {
                // Higher confidence if following an assertion
                let confidence = if prev_move == Some(DiscourseMove::Assertion) {
                    0.8
                } else {
                    0.6
                };
                return Some(MoveClassification::new(
                    DiscourseMove::Correction,
                    confidence,
                    format!("correction marker ({first})"),
                ));
            }
        }

        None
    }

    /// Check if the sentence is an opening.
    fn check_opening(&self, tokens: &[&str], text: &str) -> Option<MoveClassification> {
        let text_lower = text.to_lowercase();

        for marker in &self.opening_markers {
            if text_lower.starts_with(marker) {
                return Some(MoveClassification::new(
                    DiscourseMove::Opening,
                    0.7,
                    format!("opening marker ({marker})"),
                ));
            }
        }

        // Check for first token match
        if let Some(first) = tokens.first() {
            let first_lower = first.to_lowercase();
            if self.opening_markers.contains(&first_lower) {
                return Some(MoveClassification::new(
                    DiscourseMove::Opening,
                    0.65,
                    format!("opening word ({first})"),
                ));
            }
        }

        None
    }

    /// Check if the sentence is a closing.
    fn check_closing(&self, _tokens: &[&str], text: &str) -> Option<MoveClassification> {
        let text_lower = text.to_lowercase();

        for marker in &self.closing_markers {
            if text_lower.contains(marker) {
                return Some(MoveClassification::new(
                    DiscourseMove::Closing,
                    0.75,
                    format!("closing marker ({marker})"),
                ));
            }
        }

        None
    }

    /// Check if the sentence is an elaboration.
    fn check_elaboration(
        tokens: &[&str],
        prev_move: Option<DiscourseMove>,
    ) -> Option<MoveClassification> {
        // Elaboration markers
        let elaboration_markers = [
            "also",
            "moreover",
            "furthermore",
            "additionally",
            "specifically",
            "in particular",
            "for example",
            "for instance",
            "that is",
            "namely",
            "because",
            "since",
            "so",
            "therefore",
            "thus",
        ];

        if let Some(first) = tokens.first() {
            let first_lower = first.to_lowercase();
            if elaboration_markers.contains(&first_lower.as_str()) {
                return Some(MoveClassification::new(
                    DiscourseMove::Elaboration,
                    0.75,
                    format!("elaboration marker ({first})"),
                ));
            }
        }

        // If previous was an assertion, this might be elaboration
        if prev_move == Some(DiscourseMove::Assertion) {
            // Would need coherence relation analysis to be sure
            // For now, don't classify as elaboration without markers
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_syntax(text: &str) -> AnnotatedSyntax {
        use crate::core::{DepRel, UPos};
        use crate::runtime::{AnnotatedToken, TokenId};

        let mut offset = 0;
        let tokens: Vec<AnnotatedToken> = text
            .split_whitespace()
            .enumerate()
            .map(|(i, word)| {
                let start = offset;
                let end = start + word.len();
                offset = end + 1; // +1 for space
                AnnotatedToken::new(
                    TokenId::new(i),
                    word.to_string(),
                    word.to_lowercase(),
                    UPos::X,
                    DepRel::Dep,
                    (start, end),
                )
            })
            .collect();

        AnnotatedSyntax::new(text.to_string(), tokens)
    }

    #[test]
    fn test_question_detection_wh() {
        let classifier = MoveClassifier::new();
        let syntax = mock_syntax("Who did this?");
        let qud = QudStack::default();

        let result = classifier.classify(&syntax, None, &qud, None, None);

        assert_eq!(result.move_type, DiscourseMove::Question);
        assert_eq!(result.question_type, Some(QuestionType::Who));
        assert!(result.confidence > 0.9);
    }

    #[test]
    fn test_question_detection_yesno() {
        let classifier = MoveClassifier::new();
        let syntax = mock_syntax("Is this correct?");
        let qud = QudStack::default();

        let result = classifier.classify(&syntax, None, &qud, None, None);

        assert_eq!(result.move_type, DiscourseMove::Question);
        assert_eq!(result.question_type, Some(QuestionType::YesNo));
    }

    #[test]
    fn test_acknowledgment() {
        let classifier = MoveClassifier::new();
        let syntax = mock_syntax("Yes okay");
        let qud = QudStack::default();

        let result = classifier.classify(&syntax, None, &qud, None, None);

        assert_eq!(result.move_type, DiscourseMove::Acknowledgment);
    }

    #[test]
    fn test_default_assertion() {
        let classifier = MoveClassifier::new();
        let syntax = mock_syntax("The cat sat on the mat.");
        let qud = QudStack::default();

        let result = classifier.classify(&syntax, None, &qud, None, None);

        assert_eq!(result.move_type, DiscourseMove::Assertion);
    }

    #[test]
    fn test_question_type_from_wh_word() {
        assert_eq!(QuestionType::from_wh_word("who"), Some(QuestionType::Who));
        assert_eq!(QuestionType::from_wh_word("What"), Some(QuestionType::What));
        assert_eq!(
            QuestionType::from_wh_word("WHERE"),
            Some(QuestionType::Where)
        );
        assert_eq!(QuestionType::from_wh_word("the"), None);
    }

    #[test]
    fn test_how_many_how_much() {
        let classifier = MoveClassifier::new();

        let syntax = mock_syntax("How many people?");
        let qud = QudStack::default();
        let result = classifier.classify(&syntax, None, &qud, None, None);
        assert_eq!(result.question_type, Some(QuestionType::HowMany));

        let syntax = mock_syntax("How much does it cost?");
        let result = classifier.classify(&syntax, None, &qud, None, None);
        assert_eq!(result.question_type, Some(QuestionType::HowMuch));
    }

    #[test]
    fn test_correction_marker() {
        let classifier = MoveClassifier::new();
        let syntax = mock_syntax("Actually that is wrong.");
        let qud = QudStack::default();

        let result = classifier.classify(&syntax, None, &qud, None, Some(DiscourseMove::Assertion));

        assert_eq!(result.move_type, DiscourseMove::Correction);
    }

    #[test]
    fn test_elaboration_marker() {
        let classifier = MoveClassifier::new();
        let syntax = mock_syntax("Furthermore the evidence suggests...");
        let qud = QudStack::default();

        let result = classifier.classify(&syntax, None, &qud, None, None);

        assert_eq!(result.move_type, DiscourseMove::Elaboration);
    }
}
