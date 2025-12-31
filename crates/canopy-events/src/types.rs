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

#[cfg(test)]
mod tests {
    use super::*;
    use canopy_core::{Action, AspectualClass, LittleV, UPos, Voice};
    use std::collections::HashMap;

    /// Helper to create a minimal Entity for testing
    fn make_entity(text: &str) -> canopy_core::Entity {
        canopy_core::Entity {
            id: 0,
            text: text.to_string(),
            animacy: None,
            definiteness: None,
            number: None,
            distributivity: None,
        }
    }

    /// Helper to create a minimal Event for testing
    fn make_event(predicate: &str) -> canopy_core::Event {
        let agent = make_entity("agent");
        let action = Action {
            predicate: predicate.to_string(),
            manner: None,
            instrument: None,
        };
        canopy_core::Event {
            id: 0,
            predicate: predicate.to_string(),
            little_v: LittleV::Do { agent, action },
            participants: HashMap::new(),
            aspect: AspectualClass::Activity,
            voice: Voice::Active,
            modality: None,
        }
    }

    // ========== DependencyArc Tests ==========

    #[test]
    fn test_dependency_arc_new() {
        let arc = DependencyArc::new(0, 1, DependencyRelation::NominalSubject);
        assert_eq!(arc.head_idx, 0);
        assert_eq!(arc.dependent_idx, 1);
        assert_eq!(arc.relation, DependencyRelation::NominalSubject);
        assert_eq!(arc.confidence, 1.0);
    }

    #[test]
    fn test_dependency_arc_with_confidence() {
        let arc = DependencyArc::with_confidence(2, 3, DependencyRelation::Object, 0.8);
        assert_eq!(arc.head_idx, 2);
        assert_eq!(arc.dependent_idx, 3);
        assert_eq!(arc.relation, DependencyRelation::Object);
        assert_eq!(arc.confidence, 0.8);
    }

    #[test]
    fn test_dependency_arc_clone_debug() {
        let arc = DependencyArc::new(0, 1, DependencyRelation::Root);
        let cloned = arc.clone();
        assert_eq!(cloned.head_idx, 0);
        let debug = format!("{:?}", arc);
        assert!(debug.contains("Root"));
    }

    // ========== SentenceMetadata Tests ==========

    #[test]
    fn test_sentence_metadata_default() {
        let meta = SentenceMetadata::default();
        assert!(meta.sentence_id.is_none());
        assert!(!meta.is_passive);
        assert!(!meta.is_interrogative);
        assert!(!meta.is_negated);
        assert!(!meta.is_imperative);
    }

    #[test]
    fn test_sentence_metadata_clone_debug() {
        let meta = SentenceMetadata {
            sentence_id: Some("s1".to_string()),
            is_passive: true,
            is_interrogative: false,
            is_negated: true,
            is_imperative: false,
        };
        let cloned = meta.clone();
        assert_eq!(cloned.sentence_id, Some("s1".to_string()));
        let debug = format!("{:?}", meta);
        assert!(debug.contains("passive"));
    }

    #[test]
    fn test_sentence_metadata_serializable() {
        // Test that types derive Serialize/Deserialize (compile-time check)
        fn _assert_serializable<T: serde::Serialize + serde::de::DeserializeOwned>() {}
        _assert_serializable::<SentenceMetadata>();
    }

    // ========== SentenceAnalysis Tests ==========

    #[test]
    fn test_sentence_analysis_new() {
        let tokens = vec![Layer1SemanticResult::new(
            "runs".to_string(),
            "run".to_string(),
        )];
        let analysis = SentenceAnalysis::new("He runs".to_string(), tokens);
        assert_eq!(analysis.text, "He runs");
        assert_eq!(analysis.tokens.len(), 1);
        assert!(analysis.dependencies.is_empty());
    }

    #[test]
    fn test_sentence_analysis_with_dependencies() {
        let tokens = vec![Layer1SemanticResult::new(
            "runs".to_string(),
            "run".to_string(),
        )];
        let deps = vec![DependencyArc::new(1, 0, DependencyRelation::NominalSubject)];
        let analysis = SentenceAnalysis::new("He runs".to_string(), tokens).with_dependencies(deps);
        assert_eq!(analysis.dependencies.len(), 1);
    }

    #[test]
    fn test_sentence_analysis_with_metadata() {
        let tokens = vec![];
        let meta = SentenceMetadata {
            sentence_id: Some("s1".to_string()),
            is_passive: true,
            ..Default::default()
        };
        let analysis = SentenceAnalysis::new("test".to_string(), tokens).with_metadata(meta);
        assert!(analysis.metadata.is_passive);
        assert_eq!(analysis.metadata.sentence_id, Some("s1".to_string()));
    }

    #[test]
    fn test_sentence_analysis_get_token() {
        let tokens = vec![
            Layer1SemanticResult::new("John".to_string(), "john".to_string()),
            Layer1SemanticResult::new("runs".to_string(), "run".to_string()),
        ];
        let analysis = SentenceAnalysis::new("John runs".to_string(), tokens);
        assert_eq!(analysis.get_token(0).unwrap().original_word, "John");
        assert!(analysis.get_token(5).is_none());
    }

    #[test]
    fn test_sentence_analysis_find_predicates() {
        let mut tokens = vec![
            Layer1SemanticResult::new("John".to_string(), "john".to_string()),
            Layer1SemanticResult::new("runs".to_string(), "run".to_string()),
        ];
        tokens[1].pos = Some(UPos::Verb);
        let analysis = SentenceAnalysis::new("John runs".to_string(), tokens);
        let predicates = analysis.find_predicates();
        assert_eq!(predicates, vec![1]);
    }

    #[test]
    fn test_sentence_analysis_find_predicates_with_aux() {
        let mut tokens = vec![
            Layer1SemanticResult::new("is".to_string(), "be".to_string()),
            Layer1SemanticResult::new("running".to_string(), "run".to_string()),
        ];
        tokens[0].pos = Some(UPos::Aux);
        tokens[1].pos = Some(UPos::Verb);
        let analysis = SentenceAnalysis::new("is running".to_string(), tokens);
        let predicates = analysis.find_predicates();
        assert_eq!(predicates.len(), 2);
        assert!(predicates.contains(&0));
        assert!(predicates.contains(&1));
    }

    #[test]
    fn test_sentence_analysis_get_dependents() {
        let tokens = vec![
            Layer1SemanticResult::new("John".to_string(), "john".to_string()),
            Layer1SemanticResult::new("runs".to_string(), "run".to_string()),
        ];
        let deps = vec![DependencyArc::new(1, 0, DependencyRelation::NominalSubject)];
        let analysis =
            SentenceAnalysis::new("John runs".to_string(), tokens).with_dependencies(deps);
        let head_deps = analysis.get_dependents(1);
        assert_eq!(head_deps.len(), 1);
        assert_eq!(head_deps[0].dependent_idx, 0);
        let no_deps = analysis.get_dependents(0);
        assert!(no_deps.is_empty());
    }

    // ========== ComposedEvents Tests ==========

    #[test]
    fn test_composed_events_empty() {
        let events = ComposedEvents::empty();
        assert!(events.events.is_empty());
        assert!(events.unbound_entities.is_empty());
        assert_eq!(events.confidence, 0.0);
        assert_eq!(events.processing_time_us, 0);
        assert!(events.sources.is_empty());
    }

    #[test]
    fn test_composed_events_has_events() {
        let empty = ComposedEvents::empty();
        assert!(!empty.has_events());

        let mut with_events = ComposedEvents::empty();
        with_events.events.push(ComposedEvent {
            id: 0,
            event: make_event("run"),
            token_span: (0, 0),
            verbnet_source: None,
            framenet_source: None,
            decomposition_confidence: 0.8,
            binding_confidence: 0.9,
            presuppositions: Vec::new(),
            polarity: true,
        });
        assert!(with_events.has_events());
    }

    #[test]
    fn test_composed_events_primary_event() {
        let empty = ComposedEvents::empty();
        assert!(empty.primary_event().is_none());

        let mut with_events = ComposedEvents::empty();
        with_events.events.push(ComposedEvent {
            id: 0,
            event: make_event("run"),
            token_span: (0, 1),
            verbnet_source: Some("run-51.3.2".to_string()),
            framenet_source: None,
            decomposition_confidence: 0.8,
            binding_confidence: 0.9,
            presuppositions: Vec::new(),
            polarity: true,
        });
        let primary = with_events.primary_event().unwrap();
        assert_eq!(primary.id, 0);
        assert_eq!(primary.event.predicate, "run");
    }

    #[test]
    fn test_composed_events_total_participants() {
        let mut events = ComposedEvents::empty();
        let mut event1 = make_event("give");
        event1
            .participants
            .insert(ThetaRole::Agent, make_entity("John"));
        event1
            .participants
            .insert(ThetaRole::Theme, make_entity("book"));
        events.events.push(ComposedEvent {
            id: 0,
            event: event1,
            token_span: (0, 3),
            verbnet_source: None,
            framenet_source: None,
            decomposition_confidence: 0.8,
            binding_confidence: 0.9,
            presuppositions: Vec::new(),
            polarity: true,
        });
        assert_eq!(events.total_participants(), 2);
    }

    // ========== ComposedEvent Tests ==========

    #[test]
    fn test_composed_event_overall_confidence() {
        let composed = ComposedEvent {
            id: 0,
            event: make_event("run"),
            token_span: (0, 0),
            verbnet_source: None,
            framenet_source: None,
            decomposition_confidence: 0.8,
            binding_confidence: 0.6,
            presuppositions: Vec::new(),
            polarity: true,
        };
        // Use approximate comparison for floating point
        let conf = composed.overall_confidence();
        assert!((conf - 0.7).abs() < 0.001, "Expected ~0.7, got {}", conf);
    }

    #[test]
    fn test_composed_event_has_role() {
        let mut event = make_event("give");
        event
            .participants
            .insert(ThetaRole::Agent, make_entity("John"));
        let composed = ComposedEvent {
            id: 0,
            event,
            token_span: (0, 0),
            verbnet_source: None,
            framenet_source: None,
            decomposition_confidence: 0.8,
            binding_confidence: 0.9,
            presuppositions: Vec::new(),
            polarity: true,
        };
        assert!(composed.has_role(ThetaRole::Agent));
        assert!(!composed.has_role(ThetaRole::Theme));
    }

    #[test]
    fn test_composed_event_get_participant() {
        let mut event = make_event("give");
        event
            .participants
            .insert(ThetaRole::Agent, make_entity("John"));
        let composed = ComposedEvent {
            id: 0,
            event,
            token_span: (0, 0),
            verbnet_source: None,
            framenet_source: None,
            decomposition_confidence: 0.8,
            binding_confidence: 0.9,
            presuppositions: Vec::new(),
            polarity: true,
        };
        let agent = composed.get_participant(ThetaRole::Agent).unwrap();
        assert_eq!(agent.text, "John");
        assert!(composed.get_participant(ThetaRole::Theme).is_none());
    }

    // ========== UnboundEntity Tests ==========

    #[test]
    fn test_unbound_entity_clone_debug() {
        let unbound = UnboundEntity {
            token_idx: 0,
            text: "something".to_string(),
            suggested_role: Some(ThetaRole::Theme),
            reason: UnbindingReason::AmbiguousRole,
        };
        let cloned = unbound.clone();
        assert_eq!(cloned.text, "something");
        let debug = format!("{:?}", unbound);
        assert!(debug.contains("AmbiguousRole"));
    }

    // ========== UnbindingReason Tests ==========

    #[test]
    fn test_unbinding_reason_variants() {
        let reasons = [
            UnbindingReason::NoPredicateFound,
            UnbindingReason::AmbiguousRole,
            UnbindingReason::ExtraCoreArgument,
            UnbindingReason::MissingDependency,
            UnbindingReason::SemanticMismatch,
        ];
        for reason in &reasons {
            let cloned = reason.clone();
            let debug = format!("{:?}", cloned);
            assert!(!debug.is_empty());
        }
    }

    // ========== PredicateInfo Tests ==========

    #[test]
    fn test_predicate_info_has_verbnet() {
        let info = PredicateInfo {
            lemma: "run".to_string(),
            token_idx: 0,
            verbnet_analysis: None,
            framenet_analysis: None,
            l1_confidence: 0.8,
        };
        assert!(!info.has_verbnet());
        assert!(!info.has_framenet());
    }

    #[test]
    fn test_predicate_info_verbnet_class_id() {
        let info = PredicateInfo {
            lemma: "run".to_string(),
            token_idx: 0,
            verbnet_analysis: None,
            framenet_analysis: None,
            l1_confidence: 0.8,
        };
        assert!(info.verbnet_class_id().is_none());
    }

    // ========== LittleVType Tests ==========

    #[test]
    fn test_little_v_type_display() {
        assert_eq!(LittleVType::Cause.to_string(), "CAUSE");
        assert_eq!(LittleVType::Become.to_string(), "BECOME");
        assert_eq!(LittleVType::Be.to_string(), "BE");
        assert_eq!(LittleVType::Do.to_string(), "DO");
        assert_eq!(LittleVType::Experience.to_string(), "EXPERIENCE");
        assert_eq!(LittleVType::Go.to_string(), "GO");
        assert_eq!(LittleVType::Have.to_string(), "HAVE");
        assert_eq!(LittleVType::Say.to_string(), "SAY");
        assert_eq!(LittleVType::Exist.to_string(), "EXIST");
    }

    #[test]
    fn test_little_v_type_default_roles() {
        let cause_roles = LittleVType::Cause.default_roles();
        assert!(cause_roles.contains(&ThetaRole::Agent));
        assert!(cause_roles.contains(&ThetaRole::Patient));

        let become_roles = LittleVType::Become.default_roles();
        assert!(become_roles.contains(&ThetaRole::Theme));

        let experience_roles = LittleVType::Experience.default_roles();
        assert!(experience_roles.contains(&ThetaRole::Experiencer));
        assert!(experience_roles.contains(&ThetaRole::Stimulus));
    }

    #[test]
    fn test_little_v_type_equality_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(LittleVType::Cause);
        set.insert(LittleVType::Become);
        assert!(set.contains(&LittleVType::Cause));
        assert!(!set.contains(&LittleVType::Do));
    }

    // ========== PresuppositionTrigger Tests ==========

    #[test]
    fn test_presupposition_trigger_display() {
        assert_eq!(PresuppositionTrigger::Factive.to_string(), "factive");
        assert_eq!(PresuppositionTrigger::Aspectual.to_string(), "aspectual");
        assert_eq!(PresuppositionTrigger::Cleft.to_string(), "cleft");
        assert_eq!(PresuppositionTrigger::Definite.to_string(), "definite");
        assert_eq!(PresuppositionTrigger::Change.to_string(), "change");
    }

    #[test]
    fn test_presupposition_trigger_equality_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(PresuppositionTrigger::Factive);
        set.insert(PresuppositionTrigger::Aspectual);
        assert!(set.contains(&PresuppositionTrigger::Factive));
        assert!(!set.contains(&PresuppositionTrigger::Cleft));
    }

    // ========== PresupposedContent Tests ==========

    #[test]
    fn test_presupposed_content_existence_display() {
        let content = PresupposedContent::Existence {
            entity_text: "the book".to_string(),
        };
        let display = format!("{}", content);
        assert!(display.contains("the book"));
    }

    #[test]
    fn test_presupposed_content_state_display() {
        let content = PresupposedContent::State {
            description: "was running".to_string(),
            entity_text: "John".to_string(),
        };
        let display = format!("{}", content);
        assert!(display.contains("was running"));
    }

    #[test]
    fn test_presupposed_content_event_display() {
        let event = make_event("leave");
        let content = PresupposedContent::Event(Box::new(event));
        let display = format!("{}", content);
        assert!(display.contains("leave"));
    }

    // ========== Presupposition Tests ==========

    #[test]
    fn test_presupposition_display_projectable() {
        let presup = Presupposition {
            trigger_type: PresuppositionTrigger::Factive,
            content: PresupposedContent::Existence {
                entity_text: "x".to_string(),
            },
            projectable: true,
        };
        let display = format!("{}", presup);
        assert!(display.contains("↑"));
        assert!(display.contains("factive"));
    }

    #[test]
    fn test_presupposition_display_not_projectable() {
        let presup = Presupposition {
            trigger_type: PresuppositionTrigger::Definite,
            content: PresupposedContent::Existence {
                entity_text: "y".to_string(),
            },
            projectable: false,
        };
        let display = format!("{}", presup);
        assert!(display.contains("↓"));
        assert!(display.contains("definite"));
    }

    #[test]
    fn test_presupposition_clone_debug() {
        let presup = Presupposition {
            trigger_type: PresuppositionTrigger::Aspectual,
            content: PresupposedContent::State {
                description: "was running".to_string(),
                entity_text: "John".to_string(),
            },
            projectable: true,
        };
        let cloned = presup.clone();
        assert_eq!(cloned.trigger_type, PresuppositionTrigger::Aspectual);
        let debug = format!("{:?}", presup);
        assert!(debug.contains("Aspectual"));
    }

    // ========== DecomposedEvent Tests ==========

    #[test]
    fn test_decomposed_event_clone_debug() {
        let decomposed = DecomposedEvent {
            primary_type: LittleVType::Cause,
            expected_roles: vec![ThetaRole::Agent, ThetaRole::Patient],
            sub_event: None,
            confidence: 0.9,
            verbnet_confidence: Some(0.85),
            sources: vec!["verbnet".to_string()],
        };
        let cloned = decomposed.clone();
        assert_eq!(cloned.primary_type, LittleVType::Cause);
        let debug = format!("{:?}", decomposed);
        assert!(debug.contains("Cause"));
    }

    #[test]
    fn test_decomposed_event_with_sub_event() {
        let sub = DecomposedEvent {
            primary_type: LittleVType::Become,
            expected_roles: vec![ThetaRole::Theme],
            sub_event: None,
            confidence: 0.8,
            verbnet_confidence: None,
            sources: vec![],
        };
        let parent = DecomposedEvent {
            primary_type: LittleVType::Cause,
            expected_roles: vec![ThetaRole::Agent, ThetaRole::Patient],
            sub_event: Some(Box::new(sub)),
            confidence: 0.9,
            verbnet_confidence: Some(0.85),
            sources: vec!["verbnet".to_string()],
        };
        assert!(parent.sub_event.is_some());
        let sub = parent.sub_event.as_ref().unwrap();
        assert_eq!(sub.primary_type, LittleVType::Become);
    }
}
