//! Core types for event composition.
//!
//! These types are designed to be resource-independent. The kernel
//! receives pre-processed data from providers and composes events.
//!
//! # Packed Representations
//!
//! For handling ambiguity, this module provides packed event types:
//! - `PackedEvents`: Shares structure across readings using choice points
//! - `SenseChoicePoint`: Represents sense disambiguation alternatives
//! - `SharedEventStructure`: Common event structure across all readings

use crate::core::{DepRel, Distributivity, SemanticNumber, ThetaRole, Voice};
use crate::kernel::discourse::{AspectualViewpoint, TemporalFrame};
use crate::kernel::underspec::{
    Alternative, AmbiguitySummary, ChoiceId, ChoicePoint, ChoiceType, PackedSemantics, ReadingId,
    SharedStructure,
};
use crate::runtime::{AnnotatedSyntax, PredicateDecomposition, SenseId, TokenId};
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

/// Grammatical mood of a sentence.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SentenceMood {
    /// Statement (default mood).
    #[default]
    Declarative,
    /// Question.
    Interrogative,
    /// Command.
    Imperative,
}

/// Sentence-level metadata affecting event composition.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SentenceMetadata {
    /// Optional sentence ID for tracking.
    pub sentence_id: Option<String>,

    /// Whether the sentence is in passive voice.
    pub is_passive: bool,

    /// Grammatical mood (declarative, interrogative, imperative).
    pub mood: SentenceMood,

    /// Whether the sentence is negated.
    pub is_negated: bool,
}

impl SentenceMetadata {
    /// Returns true if the sentence is interrogative.
    #[must_use]
    pub fn is_interrogative(&self) -> bool {
        self.mood == SentenceMood::Interrogative
    }

    /// Returns true if the sentence is imperative.
    #[must_use]
    pub fn is_imperative(&self) -> bool {
        self.mood == SentenceMood::Imperative
    }
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

    /// Temporal frame (Reichenbachian S/R/E configuration).
    #[serde(default)]
    pub temporal_frame: Option<TemporalFrame>,

    /// Aspectual viewpoint (perfective, progressive, perfect, etc.).
    #[serde(default)]
    pub aspectual_viewpoint: Option<AspectualViewpoint>,
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

// ============================================================================
// Packed Event Types (for ambiguity handling)
// ============================================================================

/// Packed event representation preserving sense ambiguity.
///
/// Uses shared structure with choice points, achieving O(n) memory
/// instead of O(2^n) for explicit enumeration of readings.
#[derive(Debug, Clone)]
pub struct PackedEvents {
    /// Shared structure common to all readings.
    pub shared: SharedEventStructure,

    /// Sense choice points for each predicate.
    pub sense_choices: Vec<SenseChoicePoint>,

    /// Overall composition confidence (min across choices).
    pub confidence: f32,

    /// Sources of semantic data used.
    pub sources: Vec<String>,
}

impl PackedEvents {
    /// Create a new packed events structure.
    #[must_use]
    pub fn new(shared: SharedEventStructure) -> Self {
        Self {
            shared,
            sense_choices: Vec::new(),
            confidence: 1.0,
            sources: Vec::new(),
        }
    }

    /// Add a sense choice point.
    pub fn add_sense_choice(&mut self, choice: SenseChoicePoint) {
        // Update confidence as minimum across choices
        let choice_confidence = choice.best_confidence();
        if choice_confidence < self.confidence {
            self.confidence = choice_confidence;
        }
        self.sense_choices.push(choice);
    }

    /// Get the total number of readings (product of alternatives).
    #[must_use]
    pub fn reading_count(&self) -> usize {
        if self.sense_choices.is_empty() {
            return 1;
        }

        self.sense_choices
            .iter()
            .filter(|c| c.alternatives.len() > 1)
            .map(|c| c.alternatives.len())
            .product()
    }

    /// Check if there's any sense ambiguity.
    #[must_use]
    pub fn is_ambiguous(&self) -> bool {
        self.sense_choices.iter().any(|c| c.alternatives.len() > 1)
    }

    /// Get summary of ambiguity.
    #[must_use]
    pub fn ambiguity_summary(&self) -> AmbiguitySummary {
        let lexical = self
            .sense_choices
            .iter()
            .filter(|c| c.alternatives.len() > 1)
            .count();

        AmbiguitySummary {
            lexical,
            structural: 0,
            scope: 0,
            referential: 0,
            total_readings: self.reading_count(),
        }
    }

    /// Get the best reading (highest combined confidence).
    #[must_use]
    pub fn best_reading(&self) -> ComposedEvents {
        let mut events = Vec::new();
        let mut sources = self.sources.clone();

        for choice in &self.sense_choices {
            if let Some((event, event_sources)) = choice.best_event() {
                events.push(event);
                sources.extend(event_sources);
            }
        }

        sources.sort();
        sources.dedup();

        let confidence = if events.is_empty() {
            0.0
        } else {
            let count = u16::try_from(events.len()).unwrap_or(u16::MAX);
            events
                .iter()
                .map(ComposedEvent::overall_confidence)
                .sum::<f32>()
                / f32::from(count)
        };

        ComposedEvents {
            events,
            unbound_participants: Vec::new(),
            confidence,
            sources,
        }
    }

    /// Convert to underspec choice points for unified handling.
    #[must_use]
    pub fn to_choice_points(&self) -> Vec<ChoicePoint> {
        self.sense_choices
            .iter()
            .map(SenseChoicePoint::to_choice_point)
            .collect()
    }

    /// Check if any events were composed.
    #[must_use]
    pub fn has_events(&self) -> bool {
        !self.sense_choices.is_empty()
    }

    /// Convert to `PackedSemantics` for underspecified processing.
    #[must_use]
    pub fn to_underspec(&self) -> PackedSemantics {
        let shared = SharedStructure {
            text: self.shared.text.clone(),
            token_count: self.shared.token_count,
            predicate_positions: self.shared.predicate_ids.clone(),
        };

        let mut packed = PackedSemantics::new(shared);

        for choice_point in self.to_choice_points() {
            packed.add_choice(choice_point);
        }

        packed
    }

    /// Get a specific reading by ID as composed events.
    ///
    /// The reading ID corresponds to a specific combination of choices
    /// across all sense choice points.
    #[must_use]
    pub fn reading_to_composed(&self, reading_id: ReadingId) -> Option<ComposedEvents> {
        if self.sense_choices.is_empty() {
            return Some(ComposedEvents {
                events: Vec::new(),
                unbound_participants: Vec::new(),
                confidence: 0.0,
                sources: self.sources.clone(),
            });
        }

        // Decompose reading ID into indices for each choice
        let mut id = reading_id.0 as usize;
        let mut indices = Vec::with_capacity(self.sense_choices.len());

        for choice in self.sense_choices.iter().rev() {
            let alt_count = choice.alternatives.len().max(1);
            indices.push(id % alt_count);
            id /= alt_count;
        }
        indices.reverse();

        // Build events from the selected alternatives
        let mut events = Vec::new();
        let mut sources = self.sources.clone();

        for (event_id, (choice, &idx)) in self.sense_choices.iter().zip(indices.iter()).enumerate()
        {
            if let Some(alt) = choice.alternatives.get(idx) {
                let event = alt.to_composed_event(choice.predicate_id, event_id);
                let event_sources = vec![format!("{:?}", alt.decomposition.source)];
                events.push(event);
                sources.extend(event_sources);
            }
        }

        sources.sort();
        sources.dedup();

        let confidence = if events.is_empty() {
            0.0
        } else {
            let count = u16::try_from(events.len()).unwrap_or(u16::MAX);
            events
                .iter()
                .map(ComposedEvent::overall_confidence)
                .sum::<f32>()
                / f32::from(count)
        };

        Some(ComposedEvents {
            events,
            unbound_participants: Vec::new(),
            confidence,
            sources,
        })
    }
}

/// Shared event structure common to all readings.
#[derive(Debug, Clone, Default)]
pub struct SharedEventStructure {
    /// Original sentence text.
    pub text: String,

    /// Predicate token IDs in the sentence.
    pub predicate_ids: Vec<TokenId>,

    /// Token count.
    pub token_count: usize,

    /// Sentence-level metadata.
    pub metadata: SentenceMetadata,

    /// Dependency arcs (shared across readings).
    pub dependencies: Vec<DependencyArc>,
}

impl SharedEventStructure {
    /// Create a new shared structure from sentence analysis.
    #[must_use]
    pub fn from_analysis(analysis: &SentenceAnalysis) -> Self {
        Self {
            text: analysis.text.clone(),
            predicate_ids: analysis.find_predicates(),
            token_count: analysis.syntax.tokens.len(),
            metadata: analysis.metadata.clone(),
            dependencies: analysis.dependencies.clone(),
        }
    }
}

/// A sense choice point for a single predicate.
///
/// Captures all possible senses and their decompositions.
#[derive(Debug, Clone)]
pub struct SenseChoicePoint {
    /// Unique identifier.
    pub id: ChoiceId,

    /// Predicate token ID.
    pub predicate_id: TokenId,

    /// Predicate lemma.
    pub predicate_lemma: String,

    /// Alternative decompositions (one per sense).
    pub alternatives: Vec<SenseAlternative>,

    /// Default alternative index (highest confidence).
    pub default_idx: Option<usize>,
}

impl SenseChoicePoint {
    /// Create a new sense choice point.
    #[must_use]
    pub fn new(id: ChoiceId, predicate_id: TokenId, predicate_lemma: impl Into<String>) -> Self {
        Self {
            id,
            predicate_id,
            predicate_lemma: predicate_lemma.into(),
            alternatives: Vec::new(),
            default_idx: None,
        }
    }

    /// Add an alternative sense/decomposition.
    pub fn add_alternative(&mut self, alt: SenseAlternative) {
        let idx = self.alternatives.len();
        let confidence = alt.decomposition.confidence;

        self.alternatives.push(alt);

        // Update default to highest confidence
        if let Some(default_idx) = self.default_idx {
            if confidence > self.alternatives[default_idx].decomposition.confidence {
                self.default_idx = Some(idx);
            }
        } else {
            self.default_idx = Some(idx);
        }
    }

    /// Get the best (highest confidence) alternative.
    #[must_use]
    pub fn best_alternative(&self) -> Option<&SenseAlternative> {
        self.default_idx
            .and_then(|idx| self.alternatives.get(idx))
            .or_else(|| {
                self.alternatives.iter().max_by(|a, b| {
                    a.decomposition
                        .confidence
                        .partial_cmp(&b.decomposition.confidence)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
            })
    }

    /// Get best confidence score.
    #[must_use]
    pub fn best_confidence(&self) -> f32 {
        self.best_alternative()
            .map_or(0.0, |a| a.decomposition.confidence)
    }

    /// Get the best event from this choice point.
    #[must_use]
    pub fn best_event(&self) -> Option<(ComposedEvent, Vec<String>)> {
        self.best_alternative().map(|alt| {
            let sources = vec![format!("{:?}", alt.decomposition.source)];
            (alt.to_composed_event(self.predicate_id, 0), sources)
        })
    }

    /// Convert to a generic `ChoicePoint`.
    #[must_use]
    pub fn to_choice_point(&self) -> ChoicePoint {
        let alternatives = self
            .alternatives
            .iter()
            .enumerate()
            .map(|(idx, alt)| {
                Alternative::new(
                    idx,
                    f64::from(alt.decomposition.confidence),
                    alt.decomposition.sense_id.to_string(),
                )
            })
            .collect();

        let senses = self
            .alternatives
            .iter()
            .map(|a| a.decomposition.sense_id.clone())
            .collect();

        let mut cp = ChoicePoint::new(
            self.id,
            ChoiceType::LexicalSense {
                token_id: self.predicate_id,
                senses,
            },
            alternatives,
        );

        if let Some(default_idx) = self.default_idx {
            cp = cp.with_default(default_idx);
        }

        cp
    }
}

/// A single sense alternative with its decomposition and bindings.
#[derive(Debug, Clone)]
pub struct SenseAlternative {
    /// The predicate decomposition for this sense.
    pub decomposition: PredicateDecomposition,

    /// Bound participants for this reading.
    pub participants: HashMap<ThetaRole, Participant>,

    /// Unbound participants.
    pub unbound: Vec<UnboundParticipant>,

    /// Voice detected for this reading.
    pub voice: Voice,

    /// Token span (start, end inclusive).
    pub token_span: (TokenId, TokenId),

    /// Binding confidence.
    pub binding_confidence: f32,
}

impl SenseAlternative {
    /// Create a new sense alternative.
    #[must_use]
    pub fn new(decomposition: PredicateDecomposition) -> Self {
        Self {
            decomposition,
            participants: HashMap::new(),
            unbound: Vec::new(),
            voice: Voice::Active,
            token_span: (TokenId::new(0), TokenId::new(0)),
            binding_confidence: 1.0,
        }
    }

    /// Convert to a composed event.
    #[must_use]
    pub fn to_composed_event(&self, _predicate_id: TokenId, event_id: usize) -> ComposedEvent {
        ComposedEvent {
            id: event_id,
            predicate: self.decomposition.sense_id.to_string(),
            little_v_type: self.decomposition.little_v_type,
            participants: self.participants.clone(),
            aspect: self.decomposition.little_v_type.aspectual_class(),
            voice: self.voice,
            token_span: self.token_span,
            source_sense: Some(self.decomposition.sense_id.clone()),
            decomposition_confidence: self.decomposition.confidence,
            binding_confidence: self.binding_confidence,
            presuppositions: Vec::new(),
            polarity: true,
            temporal_frame: None,
            aspectual_viewpoint: None,
        }
    }

    /// Set participants.
    #[must_use]
    pub fn with_participants(mut self, participants: HashMap<ThetaRole, Participant>) -> Self {
        self.participants = participants;
        self
    }

    /// Set voice.
    #[must_use]
    pub fn with_voice(mut self, voice: Voice) -> Self {
        self.voice = voice;
        self
    }

    /// Set token span.
    #[must_use]
    pub fn with_span(mut self, span: (TokenId, TokenId)) -> Self {
        self.token_span = span;
        self
    }

    /// Set binding confidence.
    #[must_use]
    pub fn with_binding_confidence(mut self, confidence: f32) -> Self {
        self.binding_confidence = confidence;
        self
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

    // =========== Packed Events Tests ===========

    #[test]
    fn test_packed_events_empty() {
        let packed = PackedEvents::new(SharedEventStructure::default());
        assert!(!packed.has_events());
        assert_eq!(packed.reading_count(), 1);
        assert!(!packed.is_ambiguous());
    }

    #[test]
    fn test_sense_choice_point_creation() {
        use crate::runtime::{DecompositionSource, PredicateDecomposition, SenseId};

        let decomp1 = PredicateDecomposition::new(
            SenseId::new("bank.01"),
            LittleVType::Be,
            vec![ThetaRole::Theme],
        )
        .with_confidence(0.7)
        .with_source(DecompositionSource::VerbNet);

        let decomp2 = PredicateDecomposition::new(
            SenseId::new("bank.02"),
            LittleVType::Be,
            vec![ThetaRole::Theme],
        )
        .with_confidence(0.3)
        .with_source(DecompositionSource::VerbNet);

        let mut choice = SenseChoicePoint::new(ChoiceId::new(0), TokenId::new(0), "bank");
        choice.add_alternative(SenseAlternative::new(decomp1));
        choice.add_alternative(SenseAlternative::new(decomp2));

        assert_eq!(choice.alternatives.len(), 2);
        assert_eq!(choice.default_idx, Some(0)); // First one has higher confidence
        assert!((choice.best_confidence() - 0.7).abs() < f32::EPSILON);
    }

    #[test]
    fn test_packed_events_reading_count() {
        use crate::runtime::{DecompositionSource, PredicateDecomposition, SenseId};

        let mut packed = PackedEvents::new(SharedEventStructure::default());

        // Add choice with 2 alternatives
        let mut choice1 = SenseChoicePoint::new(ChoiceId::new(0), TokenId::new(0), "bank");
        choice1.add_alternative(SenseAlternative::new(
            PredicateDecomposition::new(SenseId::new("bank.01"), LittleVType::Be, vec![])
                .with_confidence(0.6)
                .with_source(DecompositionSource::VerbNet),
        ));
        choice1.add_alternative(SenseAlternative::new(
            PredicateDecomposition::new(SenseId::new("bank.02"), LittleVType::Be, vec![])
                .with_confidence(0.4)
                .with_source(DecompositionSource::VerbNet),
        ));
        packed.add_sense_choice(choice1);

        // Add choice with 3 alternatives
        let mut choice2 = SenseChoicePoint::new(ChoiceId::new(1), TokenId::new(1), "run");
        for i in 0..3 {
            choice2.add_alternative(SenseAlternative::new(
                PredicateDecomposition::new(
                    SenseId::new(format!("run.0{}", i + 1)),
                    LittleVType::Go,
                    vec![],
                )
                .with_confidence(0.33)
                .with_source(DecompositionSource::VerbNet),
            ));
        }
        packed.add_sense_choice(choice2);

        // 2 * 3 = 6 readings
        assert_eq!(packed.reading_count(), 6);
        assert!(packed.is_ambiguous());
        assert!(packed.has_events());
    }

    #[test]
    fn test_packed_events_best_reading() {
        use crate::runtime::{DecompositionSource, PredicateDecomposition, SenseId};

        let mut packed = PackedEvents::new(SharedEventStructure::default());

        let mut choice = SenseChoicePoint::new(ChoiceId::new(0), TokenId::new(0), "eat");
        choice.add_alternative(SenseAlternative::new(
            PredicateDecomposition::new(
                SenseId::new("consume-39.1"),
                LittleVType::Do,
                vec![ThetaRole::Agent],
            )
            .with_confidence(0.9)
            .with_source(DecompositionSource::VerbNet),
        ));
        packed.add_sense_choice(choice);

        let best = packed.best_reading();
        assert!(best.has_events());
        let event = best.primary_event().unwrap();
        assert_eq!(event.little_v_type, LittleVType::Do);
    }

    #[test]
    fn test_packed_events_ambiguity_summary() {
        use crate::runtime::{DecompositionSource, PredicateDecomposition, SenseId};

        let mut packed = PackedEvents::new(SharedEventStructure::default());

        // Add choice with 2 alternatives (ambiguous)
        let mut choice = SenseChoicePoint::new(ChoiceId::new(0), TokenId::new(0), "bank");
        choice.add_alternative(SenseAlternative::new(
            PredicateDecomposition::new(SenseId::new("bank.01"), LittleVType::Be, vec![])
                .with_confidence(0.6)
                .with_source(DecompositionSource::VerbNet),
        ));
        choice.add_alternative(SenseAlternative::new(
            PredicateDecomposition::new(SenseId::new("bank.02"), LittleVType::Be, vec![])
                .with_confidence(0.4)
                .with_source(DecompositionSource::VerbNet),
        ));
        packed.add_sense_choice(choice);

        let summary = packed.ambiguity_summary();
        assert_eq!(summary.lexical, 1);
        assert_eq!(summary.total_readings, 2);
        assert!(summary.is_ambiguous());
    }

    #[test]
    fn test_sense_alternative_to_composed_event() {
        use crate::runtime::{DecompositionSource, PredicateDecomposition, SenseId};

        let decomp = PredicateDecomposition::new(
            SenseId::new("run-51.3"),
            LittleVType::Go,
            vec![ThetaRole::Agent, ThetaRole::Goal],
        )
        .with_confidence(0.9)
        .with_source(DecompositionSource::VerbNet);

        let alt = SenseAlternative::new(decomp)
            .with_voice(Voice::Active)
            .with_span((TokenId::new(0), TokenId::new(2)))
            .with_binding_confidence(0.8);

        let event = alt.to_composed_event(TokenId::new(1), 0);
        assert_eq!(event.little_v_type, LittleVType::Go);
        assert_eq!(event.voice, Voice::Active);
        assert!((event.decomposition_confidence - 0.9).abs() < f32::EPSILON);
        assert!((event.binding_confidence - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn test_sense_choice_to_underspec_choice_point() {
        use crate::runtime::{DecompositionSource, PredicateDecomposition, SenseId};

        let mut choice = SenseChoicePoint::new(ChoiceId::new(0), TokenId::new(0), "bank");
        choice.add_alternative(SenseAlternative::new(
            PredicateDecomposition::new(SenseId::new("bank.01"), LittleVType::Be, vec![])
                .with_confidence(0.7)
                .with_source(DecompositionSource::VerbNet),
        ));
        choice.add_alternative(SenseAlternative::new(
            PredicateDecomposition::new(SenseId::new("bank.02"), LittleVType::Be, vec![])
                .with_confidence(0.3)
                .with_source(DecompositionSource::VerbNet),
        ));

        let cp = choice.to_choice_point();
        assert_eq!(cp.id, ChoiceId::new(0));
        assert_eq!(cp.alternative_count(), 2);
        assert!(!cp.is_trivial());
    }
}
