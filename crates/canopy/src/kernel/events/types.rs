//! Core types for event composition.
//!
//! These types are designed to be resource-independent. The kernel
//! receives pre-processed data from providers and composes events.

use crate::core::{DepRel, Distributivity, SemanticNumber, ThetaRole, Voice};
use crate::runtime::{AnnotatedSyntax, SenseId, TokenId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Input Types
// ============================================================================

/// Input for event composition - a sentence's complete analysis.
#[derive(Debug, Clone)]
pub struct SentenceAnalysis {
    /// Original sentence text.
    pub text: String,

    /// Annotated syntax from the `SyntaxProvider`.
    pub syntax: AnnotatedSyntax,

    /// Dependency arcs between tokens.
    pub dependencies: Vec<DependencyArc>,

    /// Sentence-level metadata.
    pub metadata: SentenceMetadata,
}

impl SentenceAnalysis {
    /// Create a new sentence analysis.
    pub fn new(text: impl Into<String>, syntax: AnnotatedSyntax) -> Self {
        Self {
            text: text.into(),
            syntax,
            dependencies: Vec::new(),
            metadata: SentenceMetadata::default(),
        }
    }

    /// Add dependency arcs.
    #[must_use]
    pub fn with_dependencies(mut self, deps: Vec<DependencyArc>) -> Self {
        self.dependencies = deps;
        self
    }

    /// Add metadata.
    #[must_use]
    pub fn with_metadata(mut self, metadata: SentenceMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// Find predicate tokens (verbs).
    #[must_use]
    pub fn find_predicates(&self) -> Vec<TokenId> {
        self.syntax.predicates().map(|t| t.id).collect()
    }

    /// Get dependents of a token.
    #[must_use]
    pub fn get_dependents(&self, head_id: TokenId) -> Vec<&DependencyArc> {
        self.dependencies
            .iter()
            .filter(|arc| arc.head_id == head_id)
            .collect()
    }
}

/// A dependency arc between two tokens.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyArc {
    /// ID of the head token.
    pub head_id: TokenId,

    /// ID of the dependent token.
    pub dependent_id: TokenId,

    /// Dependency relation type.
    pub relation: DepRel,

    /// Confidence score for this arc.
    pub confidence: f32,
}

impl DependencyArc {
    /// Create a new dependency arc.
    #[must_use]
    pub fn new(head_id: TokenId, dependent_id: TokenId, relation: DepRel) -> Self {
        Self {
            head_id,
            dependent_id,
            relation,
            confidence: 1.0,
        }
    }

    /// Create with explicit confidence.
    #[must_use]
    pub fn with_confidence(
        head_id: TokenId,
        dependent_id: TokenId,
        relation: DepRel,
        confidence: f32,
    ) -> Self {
        Self {
            head_id,
            dependent_id,
            relation,
            confidence,
        }
    }
}

/// Sentence-level metadata affecting event composition.
#[allow(clippy::struct_excessive_bools)] // Sentence properties are naturally boolean
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SentenceMetadata {
    /// Optional sentence ID for tracking.
    pub sentence_id: Option<String>,

    /// Whether the sentence is in passive voice.
    pub is_passive: bool,

    /// Whether the sentence is interrogative.
    pub is_interrogative: bool,

    /// Whether the sentence is negated.
    pub is_negated: bool,

    /// Whether the sentence is imperative.
    pub is_imperative: bool,
}

// ============================================================================
// Output Types
// ============================================================================

/// Result of event composition for a sentence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposedEvents {
    /// Primary event(s) in the sentence.
    pub events: Vec<ComposedEvent>,

    /// Participants that couldn't be assigned a theta role.
    pub unbound_participants: Vec<UnboundParticipant>,

    /// Overall composition confidence.
    pub confidence: f32,

    /// Sources of semantic data used.
    pub sources: Vec<String>,
}

impl ComposedEvents {
    /// Create an empty result.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            events: Vec::new(),
            unbound_participants: Vec::new(),
            confidence: 0.0,
            sources: Vec::new(),
        }
    }

    /// Check if any events were composed.
    #[must_use]
    pub fn has_events(&self) -> bool {
        !self.events.is_empty()
    }

    /// Get the primary (first) event.
    #[must_use]
    pub fn primary_event(&self) -> Option<&ComposedEvent> {
        self.events.first()
    }

    /// Get total participant count across all events.
    #[must_use]
    pub fn total_participants(&self) -> usize {
        self.events.iter().map(|e| e.participants.len()).sum()
    }
}

/// A single composed event with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposedEvent {
    /// Event ID within the sentence (0-indexed).
    pub id: usize,

    /// The predicate lemma.
    pub predicate: String,

    /// Primary `LittleV` type.
    pub little_v_type: LittleVType,

    /// Bound participants.
    pub participants: HashMap<ThetaRole, Participant>,

    /// Aspectual class.
    pub aspect: crate::core::AspectualClass,

    /// Voice (active/passive/middle).
    pub voice: Voice,

    /// Token span (start, end inclusive).
    pub token_span: (TokenId, TokenId),

    /// Sense ID that sourced this decomposition.
    pub source_sense: Option<SenseId>,

    /// Decomposition confidence.
    pub decomposition_confidence: f32,

    /// Binding confidence.
    pub binding_confidence: f32,

    /// Presuppositions triggered by this event.
    #[serde(default)]
    pub presuppositions: Vec<Presupposition>,

    /// Event polarity: true = affirmative, false = negated.
    #[serde(default = "default_true")]
    pub polarity: bool,
}

fn default_true() -> bool {
    true
}

impl ComposedEvent {
    /// Get the overall confidence for this event.
    #[must_use]
    pub fn overall_confidence(&self) -> f32 {
        f32::midpoint(self.decomposition_confidence, self.binding_confidence)
    }

    /// Check if a theta role is filled.
    #[must_use]
    pub fn has_role(&self, role: ThetaRole) -> bool {
        self.participants.contains_key(&role)
    }

    /// Get participant by role.
    #[must_use]
    pub fn get_participant(&self, role: ThetaRole) -> Option<&Participant> {
        self.participants.get(&role)
    }
}

/// A participant bound to a theta role.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    /// Token ID.
    pub token_id: TokenId,

    /// Surface text.
    pub text: String,

    /// Semantic number if known.
    pub number: Option<SemanticNumber>,

    /// Distributivity for plurals.
    pub distributivity: Option<Distributivity>,

    /// Binding confidence.
    pub confidence: f32,
}

impl Participant {
    /// Create a new participant.
    pub fn new(token_id: TokenId, text: impl Into<String>) -> Self {
        Self {
            token_id,
            text: text.into(),
            number: None,
            distributivity: None,
            confidence: 1.0,
        }
    }
}

/// A participant that couldn't be bound to a theta role.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnboundParticipant {
    /// Token ID.
    pub token_id: TokenId,

    /// Surface text.
    pub text: String,

    /// Suggested role if ambiguous.
    pub suggested_role: Option<ThetaRole>,

    /// Reason for failure to bind.
    pub reason: UnbindingReason,
}

/// Reasons why a participant couldn't be bound.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UnbindingReason {
    /// No predicate was found.
    NoPredicateFound,

    /// Multiple roles were equally valid.
    AmbiguousRole,

    /// All core argument slots were filled.
    ExtraCoreArgument,

    /// No dependency arc connected this to a predicate.
    MissingDependency,

    /// Semantic type didn't match any role.
    SemanticMismatch,
}

// ============================================================================
// Decomposition Types
// ============================================================================

/// Simplified `LittleV` type enum for decomposition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LittleVType {
    Cause,
    Become,
    Be,
    Do,
    Experience,
    Go,
    Have,
    Say,
    Exist,
}

impl std::fmt::Display for LittleVType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            LittleVType::Cause => "CAUSE",
            LittleVType::Become => "BECOME",
            LittleVType::Be => "BE",
            LittleVType::Do => "DO",
            LittleVType::Experience => "EXPERIENCE",
            LittleVType::Go => "GO",
            LittleVType::Have => "HAVE",
            LittleVType::Say => "SAY",
            LittleVType::Exist => "EXIST",
        };
        write!(f, "{s}")
    }
}

impl LittleVType {
    /// Get default expected roles for this `LittleV` type.
    #[must_use]
    pub fn default_roles(&self) -> Vec<ThetaRole> {
        match self {
            LittleVType::Cause => vec![ThetaRole::Agent, ThetaRole::Patient],
            LittleVType::Become | LittleVType::Be => vec![ThetaRole::Theme],
            LittleVType::Do => vec![ThetaRole::Agent],
            LittleVType::Experience => vec![ThetaRole::Experiencer, ThetaRole::Stimulus],
            LittleVType::Go => vec![ThetaRole::Theme, ThetaRole::Goal],
            LittleVType::Have => vec![ThetaRole::Agent, ThetaRole::Theme],
            LittleVType::Say => vec![ThetaRole::Agent, ThetaRole::Recipient],
            LittleVType::Exist => vec![ThetaRole::Theme, ThetaRole::Location],
        }
    }

    /// Get the aspectual class for this `LittleV` type.
    #[must_use]
    pub const fn aspectual_class(&self) -> crate::core::AspectualClass {
        match self {
            LittleVType::Be | LittleVType::Have | LittleVType::Experience => {
                crate::core::AspectualClass::State
            }
            LittleVType::Do => crate::core::AspectualClass::Activity,
            LittleVType::Become => crate::core::AspectualClass::Achievement,
            LittleVType::Cause | LittleVType::Go | LittleVType::Say | LittleVType::Exist => {
                crate::core::AspectualClass::Accomplishment
            }
        }
    }
}

// ============================================================================
// Presupposition Types
// ============================================================================

/// A presupposition triggered by the event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Presupposition {
    /// Type of trigger.
    pub trigger_type: PresuppositionTrigger,

    /// The presupposed content.
    pub content: PresupposedContent,

    /// Whether this projects through negation/embedding.
    pub projectable: bool,
}

/// Types of presupposition triggers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PresuppositionTrigger {
    /// Factive verbs: "know", "regret" - presuppose truth of complement.
    Factive,

    /// Aspectual verbs: "stop", "continue" - presuppose prior state.
    Aspectual,

    /// Cleft constructions: "It was X who..." - presuppose existence.
    Cleft,

    /// Definite descriptions: "the X" - presuppose existence.
    Definite,

    /// Change-of-state: "again", "still" - presuppose prior state.
    Change,
}

impl std::fmt::Display for PresuppositionTrigger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PresuppositionTrigger::Factive => write!(f, "factive"),
            PresuppositionTrigger::Aspectual => write!(f, "aspectual"),
            PresuppositionTrigger::Cleft => write!(f, "cleft"),
            PresuppositionTrigger::Definite => write!(f, "definite"),
            PresuppositionTrigger::Change => write!(f, "change"),
        }
    }
}

/// Content that is presupposed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PresupposedContent {
    /// A presupposed event.
    Event {
        predicate: String,
        description: String,
    },

    /// A presupposed state.
    State {
        description: String,
        entity_text: String,
    },

    /// Existence presupposition.
    Existence { entity_text: String },
}

impl std::fmt::Display for PresupposedContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PresupposedContent::Event { predicate, .. } => write!(f, "event({predicate})"),
            PresupposedContent::State { description, .. } => write!(f, "state({description})"),
            PresupposedContent::Existence { entity_text } => {
                write!(f, "\u{2203} \"{entity_text}\"")
            }
        }
    }
}

impl std::fmt::Display for Presupposition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let proj = if self.projectable {
            "\u{2191}"
        } else {
            "\u{2193}"
        };
        write!(f, "[{} {} {}]", self.trigger_type, self.content, proj)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_little_v_type_display() {
        assert_eq!(LittleVType::Cause.to_string(), "CAUSE");
        assert_eq!(LittleVType::Become.to_string(), "BECOME");
        assert_eq!(LittleVType::Be.to_string(), "BE");
        assert_eq!(LittleVType::Do.to_string(), "DO");
    }

    #[test]
    fn test_little_v_type_default_roles() {
        let cause_roles = LittleVType::Cause.default_roles();
        assert!(cause_roles.contains(&ThetaRole::Agent));
        assert!(cause_roles.contains(&ThetaRole::Patient));

        let become_roles = LittleVType::Become.default_roles();
        assert!(become_roles.contains(&ThetaRole::Theme));
    }

    #[test]
    fn test_little_v_type_aspectual_class() {
        assert_eq!(
            LittleVType::Be.aspectual_class(),
            crate::core::AspectualClass::State
        );
        assert_eq!(
            LittleVType::Do.aspectual_class(),
            crate::core::AspectualClass::Activity
        );
        assert_eq!(
            LittleVType::Become.aspectual_class(),
            crate::core::AspectualClass::Achievement
        );
    }

    #[test]
    fn test_composed_events_empty() {
        let events = ComposedEvents::empty();
        assert!(events.events.is_empty());
        assert!(!events.has_events());
        assert!(events.primary_event().is_none());
    }

    #[test]
    fn test_dependency_arc() {
        let arc = DependencyArc::new(TokenId::new(0), TokenId::new(1), DepRel::Nsubj);
        assert_eq!(arc.head_id.index(), 0);
        assert_eq!(arc.dependent_id.index(), 1);
        assert!((arc.confidence - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_participant() {
        let p = Participant::new(TokenId::new(0), "John");
        assert_eq!(p.text, "John");
    }

    #[test]
    fn test_presupposition_display() {
        let presup = Presupposition {
            trigger_type: PresuppositionTrigger::Factive,
            content: PresupposedContent::Existence {
                entity_text: "the book".to_string(),
            },
            projectable: true,
        };
        let display = presup.to_string();
        assert!(display.contains("factive"));
        assert!(display.contains("the book"));
    }

    #[test]
    fn test_little_v_type_all_display() {
        assert_eq!(LittleVType::Experience.to_string(), "EXPERIENCE");
        assert_eq!(LittleVType::Go.to_string(), "GO");
        assert_eq!(LittleVType::Have.to_string(), "HAVE");
        assert_eq!(LittleVType::Say.to_string(), "SAY");
        assert_eq!(LittleVType::Exist.to_string(), "EXIST");
    }

    #[test]
    fn test_little_v_type_all_default_roles() {
        let be_roles = LittleVType::Be.default_roles();
        assert!(be_roles.contains(&ThetaRole::Theme));

        let do_roles = LittleVType::Do.default_roles();
        assert!(do_roles.contains(&ThetaRole::Agent));

        let exp_roles = LittleVType::Experience.default_roles();
        assert!(exp_roles.contains(&ThetaRole::Experiencer));
        assert!(exp_roles.contains(&ThetaRole::Stimulus));

        let go_roles = LittleVType::Go.default_roles();
        assert!(go_roles.contains(&ThetaRole::Theme));
        assert!(go_roles.contains(&ThetaRole::Goal));

        let have_roles = LittleVType::Have.default_roles();
        assert!(have_roles.contains(&ThetaRole::Agent));
        assert!(have_roles.contains(&ThetaRole::Theme));

        let say_roles = LittleVType::Say.default_roles();
        assert!(say_roles.contains(&ThetaRole::Agent));
        assert!(say_roles.contains(&ThetaRole::Recipient));

        let exist_roles = LittleVType::Exist.default_roles();
        assert!(exist_roles.contains(&ThetaRole::Theme));
        assert!(exist_roles.contains(&ThetaRole::Location));
    }

    #[test]
    fn test_little_v_type_all_aspectual_classes() {
        assert_eq!(
            LittleVType::Have.aspectual_class(),
            crate::core::AspectualClass::State
        );
        assert_eq!(
            LittleVType::Experience.aspectual_class(),
            crate::core::AspectualClass::State
        );
        assert_eq!(
            LittleVType::Cause.aspectual_class(),
            crate::core::AspectualClass::Accomplishment
        );
        assert_eq!(
            LittleVType::Go.aspectual_class(),
            crate::core::AspectualClass::Accomplishment
        );
        assert_eq!(
            LittleVType::Say.aspectual_class(),
            crate::core::AspectualClass::Accomplishment
        );
        assert_eq!(
            LittleVType::Exist.aspectual_class(),
            crate::core::AspectualClass::Accomplishment
        );
    }

    #[test]
    fn test_presupposition_trigger_display() {
        assert_eq!(PresuppositionTrigger::Factive.to_string(), "factive");
        assert_eq!(PresuppositionTrigger::Aspectual.to_string(), "aspectual");
        assert_eq!(PresuppositionTrigger::Cleft.to_string(), "cleft");
        assert_eq!(PresuppositionTrigger::Definite.to_string(), "definite");
        assert_eq!(PresuppositionTrigger::Change.to_string(), "change");
    }

    #[test]
    fn test_presupposed_content_display() {
        let event = PresupposedContent::Event {
            predicate: "run".to_string(),
            description: "running event".to_string(),
        };
        assert!(event.to_string().contains("event(run)"));

        let state = PresupposedContent::State {
            description: "happy state".to_string(),
            entity_text: "John".to_string(),
        };
        assert!(state.to_string().contains("state(happy state)"));

        let existence = PresupposedContent::Existence {
            entity_text: "the king".to_string(),
        };
        assert!(existence.to_string().contains("the king"));
    }

    #[test]
    fn test_presupposition_not_projectable() {
        let presup = Presupposition {
            trigger_type: PresuppositionTrigger::Aspectual,
            content: PresupposedContent::State {
                description: "running".to_string(),
                entity_text: "John".to_string(),
            },
            projectable: false,
        };
        let display = presup.to_string();
        assert!(display.contains("aspectual"));
        // Non-projectable should use down arrow
        assert!(display.contains("↓"));
    }

    #[test]
    fn test_dependency_arc_with_confidence() {
        let arc =
            DependencyArc::with_confidence(TokenId::new(0), TokenId::new(1), DepRel::Obj, 0.9);
        assert!((arc.confidence - 0.9).abs() < f32::EPSILON);
        assert_eq!(arc.relation, DepRel::Obj);
    }

    #[test]
    fn test_participant_full() {
        let p = Participant {
            token_id: TokenId::new(0),
            text: "John".to_string(),
            number: Some(SemanticNumber::Singular),
            distributivity: Some(Distributivity::Collective),
            confidence: 0.9,
        };
        assert_eq!(p.number, Some(SemanticNumber::Singular));
        assert_eq!(p.distributivity, Some(Distributivity::Collective));
        assert!((p.confidence - 0.9).abs() < f32::EPSILON);
    }

    #[test]
    fn test_sentence_analysis_new() {
        use crate::runtime::AnnotatedSyntax;

        let syntax = AnnotatedSyntax::new("test".to_string(), vec![]);
        let analysis = SentenceAnalysis::new("test", syntax);
        assert_eq!(analysis.text, "test");
        assert!(analysis.dependencies.is_empty());
    }

    #[test]
    fn test_sentence_analysis_with_dependencies() {
        use crate::runtime::AnnotatedSyntax;

        let syntax = AnnotatedSyntax::new("test".to_string(), vec![]);
        let arc = DependencyArc::new(TokenId::new(0), TokenId::new(1), DepRel::Nsubj);
        let analysis = SentenceAnalysis::new("test", syntax).with_dependencies(vec![arc]);
        assert_eq!(analysis.dependencies.len(), 1);
    }

    #[test]
    fn test_get_dependents() {
        use crate::runtime::AnnotatedSyntax;

        let syntax = AnnotatedSyntax::new("test".to_string(), vec![]);
        let arc = DependencyArc::new(TokenId::new(0), TokenId::new(1), DepRel::Nsubj);
        let analysis = SentenceAnalysis::new("test", syntax).with_dependencies(vec![arc]);
        let deps = analysis.get_dependents(TokenId::new(0));
        assert_eq!(deps.len(), 1);
    }
}
