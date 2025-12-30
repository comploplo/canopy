//! Core types for event composition
//!
//! This module defines the input and output types for Layer 2 event composition.

use canopy_core::{Entity, Event, ThetaRole};
use canopy_tokenizer::coordinator::Layer1SemanticResult;
use canopy_treebank::types::DependencyRelation;
use serde::{Deserialize, Serialize};

/// Input for event composition - a sentence's complete Layer 1 analysis
#[derive(Debug, Clone)]
pub struct SentenceAnalysis {
    /// Original sentence text
    pub text: String,

    /// Token-level Layer 1 semantic results
    pub tokens: Vec<Layer1SemanticResult>,

    /// Dependency arcs between tokens
    pub dependencies: Vec<DependencyArc>,

    /// Sentence-level metadata
    pub metadata: SentenceMetadata,
}

impl SentenceAnalysis {
    /// Create a new sentence analysis
    pub fn new(text: String, tokens: Vec<Layer1SemanticResult>) -> Self {
        Self {
            text,
            tokens,
            dependencies: Vec::new(),
            metadata: SentenceMetadata::default(),
        }
    }

    /// Add dependency arcs
    pub fn with_dependencies(mut self, deps: Vec<DependencyArc>) -> Self {
        self.dependencies = deps;
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, metadata: SentenceMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// Get token by index
    pub fn get_token(&self, idx: usize) -> Option<&Layer1SemanticResult> {
        self.tokens.get(idx)
    }

    /// Find predicates (verbs) in the sentence
    pub fn find_predicates(&self) -> Vec<usize> {
        self.tokens
            .iter()
            .enumerate()
            .filter(|(_, t)| {
                matches!(
                    t.pos,
                    Some(canopy_core::UPos::Verb) | Some(canopy_core::UPos::Aux)
                )
            })
            .map(|(i, _)| i)
            .collect()
    }

    /// Get dependents of a token
    pub fn get_dependents(&self, head_idx: usize) -> Vec<&DependencyArc> {
        self.dependencies
            .iter()
            .filter(|arc| arc.head_idx == head_idx)
            .collect()
    }
}

/// A dependency arc between two tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyArc {
    /// Index of the head token
    pub head_idx: usize,

    /// Index of the dependent token
    pub dependent_idx: usize,

    /// Dependency relation type
    pub relation: DependencyRelation,

    /// Confidence score for this arc
    pub confidence: f32,
}

impl DependencyArc {
    /// Create a new dependency arc
    pub fn new(head_idx: usize, dependent_idx: usize, relation: DependencyRelation) -> Self {
        Self {
            head_idx,
            dependent_idx,
            relation,
            confidence: 1.0,
        }
    }

    /// Create with explicit confidence
    pub fn with_confidence(
        head_idx: usize,
        dependent_idx: usize,
        relation: DependencyRelation,
        confidence: f32,
    ) -> Self {
        Self {
            head_idx,
            dependent_idx,
            relation,
            confidence,
        }
    }
}

/// Sentence-level metadata affecting event composition
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SentenceMetadata {
    /// Optional sentence ID for tracking
    pub sentence_id: Option<String>,

    /// Whether the sentence is in passive voice
    pub is_passive: bool,

    /// Whether the sentence is interrogative
    pub is_interrogative: bool,

    /// Whether the sentence is negated
    pub is_negated: bool,

    /// Whether the sentence is imperative
    pub is_imperative: bool,
}

/// Result of event composition for a sentence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposedEvents {
    /// Primary event(s) in the sentence
    pub events: Vec<ComposedEvent>,

    /// Entities that couldn't be assigned a theta role
    pub unbound_entities: Vec<UnboundEntity>,

    /// Overall composition confidence
    pub confidence: f32,

    /// Processing time in microseconds
    pub processing_time_us: u64,

    /// Sources of semantic data used
    pub sources: Vec<String>,
}

impl ComposedEvents {
    /// Create an empty result
    pub fn empty() -> Self {
        Self {
            events: Vec::new(),
            unbound_entities: Vec::new(),
            confidence: 0.0,
            processing_time_us: 0,
            sources: Vec::new(),
        }
    }

    /// Check if any events were composed
    pub fn has_events(&self) -> bool {
        !self.events.is_empty()
    }

    /// Get the primary (first) event
    pub fn primary_event(&self) -> Option<&ComposedEvent> {
        self.events.first()
    }

    /// Get total participant count across all events
    pub fn total_participants(&self) -> usize {
        self.events.iter().map(|e| e.event.participants.len()).sum()
    }
}

/// A single composed event with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposedEvent {
    /// Event ID within the sentence (0-indexed)
    pub id: usize,

    /// The core event structure from canopy-core
    pub event: Event,

    /// Token indices that contribute to this event (start, end inclusive)
    pub token_span: (usize, usize),

    /// VerbNet class that sourced this event
    pub verbnet_source: Option<String>,

    /// FrameNet frame used (if applicable)
    pub framenet_source: Option<String>,

    /// Confidence from the decomposition step
    pub decomposition_confidence: f32,

    /// Confidence from the binding step
    pub binding_confidence: f32,

    /// Presuppositions triggered by this event
    #[serde(default)]
    pub presuppositions: Vec<Presupposition>,

    /// Event polarity: true = affirmative, false = negated
    #[serde(default = "default_polarity")]
    pub polarity: bool,
}

/// Default polarity is affirmative
fn default_polarity() -> bool {
    true
}

impl ComposedEvent {
    /// Get the overall confidence for this event
    pub fn overall_confidence(&self) -> f32 {
        (self.decomposition_confidence + self.binding_confidence) / 2.0
    }

    /// Check if a theta role is filled
    pub fn has_role(&self, role: ThetaRole) -> bool {
        self.event.participants.contains_key(&role)
    }

    /// Get participant by role
    pub fn get_participant(&self, role: ThetaRole) -> Option<&Entity> {
        self.event.participants.get(&role)
    }
}

/// An entity that couldn't be assigned to a theta role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnboundEntity {
    /// Token index in the sentence
    pub token_idx: usize,

    /// Surface text of the entity
    pub text: String,

    /// Suggested role if ambiguous
    pub suggested_role: Option<ThetaRole>,

    /// Reason for failure to bind
    pub reason: UnbindingReason,
}

/// Reasons why an entity couldn't be bound to a theta role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UnbindingReason {
    /// No predicate was found to assign roles
    NoPredicateFound,

    /// Multiple roles were equally valid
    AmbiguousRole,

    /// All core argument slots were already filled
    ExtraCoreArgument,

    /// No dependency arc connected this entity to a predicate
    MissingDependency,

    /// The entity's semantic type didn't match any role
    SemanticMismatch,
}

/// Information about a predicate extracted from Layer 1
#[derive(Debug, Clone)]
pub struct PredicateInfo {
    /// Lemma of the predicate
    pub lemma: String,

    /// Token index in the sentence
    pub token_idx: usize,

    /// VerbNet analysis if available
    pub verbnet_analysis: Option<canopy_semantic_engines::verbnet::VerbNetAnalysis>,

    /// FrameNet analysis if available
    pub framenet_analysis: Option<canopy_semantic_engines::framenet::FrameNetAnalysis>,

    /// Confidence from Layer 1
    pub l1_confidence: f32,
}

impl PredicateInfo {
    /// Check if VerbNet data is available
    pub fn has_verbnet(&self) -> bool {
        self.verbnet_analysis.is_some()
    }

    /// Check if FrameNet data is available
    pub fn has_framenet(&self) -> bool {
        self.framenet_analysis.is_some()
    }

    /// Get VerbNet class ID if available
    pub fn verbnet_class_id(&self) -> Option<&str> {
        self.verbnet_analysis
            .as_ref()
            .and_then(|v| v.verb_classes.first())
            .map(|c| c.id.as_str())
    }
}

/// Result of decomposing a predicate into LittleV structure
#[derive(Debug, Clone)]
pub struct DecomposedEvent {
    /// The primary LittleV type
    pub primary_type: LittleVType,

    /// Expected theta roles based on decomposition
    pub expected_roles: Vec<ThetaRole>,

    /// Optional sub-event (e.g., Cause contains Become)
    pub sub_event: Option<Box<DecomposedEvent>>,

    /// Decomposition confidence
    pub confidence: f32,

    /// VerbNet confidence if used
    pub verbnet_confidence: Option<f32>,

    /// Source attribution
    pub sources: Vec<String>,
}

/// Simplified LittleV type enum for decomposition logic
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
        write!(f, "{}", s)
    }
}

impl LittleVType {
    /// Get default expected roles for this LittleV type
    pub fn default_roles(&self) -> Vec<ThetaRole> {
        match self {
            LittleVType::Cause => vec![ThetaRole::Agent, ThetaRole::Patient],
            LittleVType::Become => vec![ThetaRole::Theme],
            LittleVType::Be => vec![ThetaRole::Theme],
            LittleVType::Do => vec![ThetaRole::Agent],
            LittleVType::Experience => vec![ThetaRole::Experiencer, ThetaRole::Stimulus],
            LittleVType::Go => vec![ThetaRole::Theme, ThetaRole::Goal],
            LittleVType::Have => vec![ThetaRole::Agent, ThetaRole::Theme],
            LittleVType::Say => vec![ThetaRole::Agent, ThetaRole::Recipient],
            LittleVType::Exist => vec![ThetaRole::Theme, ThetaRole::Location],
        }
    }
}

// ============================================================================
// Presupposition Types
// ============================================================================

/// A presupposition triggered by the event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Presupposition {
    /// Type of trigger that generated this presupposition
    pub trigger_type: PresuppositionTrigger,

    /// The presupposed content
    pub content: PresupposedContent,

    /// Whether this presupposition projects through negation/embedding
    pub projectable: bool,
}

/// Types of presupposition triggers
///
/// Note: These are detected via VerbNet class patterns and FrameNet frames,
/// NOT via hardcoded word lists. See presupposition.rs detector for details.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PresuppositionTrigger {
    /// Factive verbs: "know", "regret", "realize" - presuppose truth of complement
    /// Detected via VerbNet classes: admire-31.2, marvel-31.3, etc.
    /// And FrameNet frames: Awareness, Experiencer_focus
    Factive,

    /// Aspectual verbs: "stop", "continue", "start" - presuppose prior state
    /// Detected via VerbNet classes: stop-55.4, continue-55.3, begin-55.1
    Aspectual,

    /// Cleft constructions: "It was X who..." - presuppose existence
    Cleft,

    /// Definite descriptions: "the X" - presuppose existence
    /// Detected via Entity.definiteness == Definite
    Definite,

    /// Change-of-state expressions: "again", "still" - presuppose prior state
    Change,
}

/// Content that is presupposed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PresupposedContent {
    /// A presupposed event (e.g., "regrets leaving" → presupposes "left")
    Event(Box<Event>),

    /// A presupposed state (e.g., "stopped running" → was running)
    State {
        /// Description of the state
        description: String,
        /// Entity to which the state applies
        entity_text: String,
    },

    /// Existence presupposition (e.g., "the book" → book exists)
    Existence {
        /// Text of the entity whose existence is presupposed
        entity_text: String,
    },
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

impl std::fmt::Display for PresupposedContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PresupposedContent::Event(e) => write!(f, "event({})", e.predicate),
            PresupposedContent::State { description, .. } => write!(f, "state({})", description),
            PresupposedContent::Existence { entity_text } => {
                write!(f, "∃ \"{}\"", entity_text)
            }
        }
    }
}

impl std::fmt::Display for Presupposition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let proj = if self.projectable { "↑" } else { "↓" };
        write!(f, "[{} {} {}]", self.trigger_type, self.content, proj)
    }
}
