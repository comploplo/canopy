//! Coherence relation classification using SDRT-inspired relations.
//!
//! Detects relations between adjacent discourse segments using:
//! - Discourse markers (then, because, but, etc.)
//! - Participant overlap (shared referents)
//! - Event similarity (same predicate/frame)
//! - Temporal reasoning (tense/aspect cues)
//! - Polarity (negation patterns)

use super::drs::TemporalRelationType;
use super::referent::{ReferentId, ReferentRegistry};
use super::temporal::TemporalFrame;
use crate::kernel::events::ComposedEvents;
use serde::{Deserialize, Serialize};

/// SDRT-inspired coherence relations between discourse segments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CoherenceRelation {
    // Temporal
    /// e1 then e2 - temporal sequence
    Narration,
    /// e2 provides setting for e1
    Background,

    // Causal/Logical
    /// e2 explains why e1 happened
    Explanation,
    /// e2 is consequence of e1
    Result,

    // Contrastive
    /// e1 and e2 are compared/opposed
    Contrast,
    /// e2 despite e1
    Concession,

    // Elaborative
    /// e2 adds detail to e1
    Elaboration,
    /// e1 and e2 are structurally similar
    Parallel,

    // Topic
    /// e2 introduces new topic
    TopicShift,
    /// e2 continues same topic
    Continuation,
}

impl CoherenceRelation {
    /// Return true if this is a coordinating relation (segments at same level).
    #[must_use]
    pub fn is_coordinating(&self) -> bool {
        matches!(
            self,
            Self::Narration | Self::Contrast | Self::Parallel | Self::Continuation
        )
    }

    /// Return true if this is a subordinating relation (segment depends on another).
    #[must_use]
    pub fn is_subordinating(&self) -> bool {
        matches!(
            self,
            Self::Background
                | Self::Explanation
                | Self::Result
                | Self::Elaboration
                | Self::Concession
        )
    }

    /// Return typical discourse markers for this relation.
    #[must_use]
    pub fn typical_markers(&self) -> &'static [&'static str] {
        match self {
            Self::Narration => &["then", "later", "next", "afterwards", "subsequently"],
            Self::Background => &["meanwhile", "while", "as"],
            Self::Explanation => &["because", "since", "for", "as"],
            Self::Result => &["so", "therefore", "thus", "hence", "consequently"],
            Self::Contrast => &["but", "however", "yet", "although", "though", "whereas"],
            Self::Concession => &["despite", "nevertheless", "nonetheless", "even though"],
            Self::Elaboration => &["in fact", "indeed", "specifically", "for example", "namely"],
            Self::Parallel => &["similarly", "likewise", "also", "too"],
            Self::TopicShift => &["anyway", "incidentally", "by the way"],
            Self::Continuation => &["and", "also", "moreover", "furthermore"],
        }
    }

    /// Return the default temporal constraint induced by this coherence relation.
    ///
    /// Based on SDRT principles:
    /// - Narration: e1 < e2 (events in sequence)
    /// - Background: e2 contains e1 (background provides setting)
    /// - Explanation: e2 < e1 (cause precedes effect being explained)
    /// - Result: e1 < e2 (cause precedes effect)
    /// - Elaboration: e2 during e1 (detail is part of larger event)
    /// - Parallel: e1 ≈ e2 (simultaneous or concurrent events)
    #[must_use]
    pub fn default_temporal_constraint(&self) -> Option<TemporalRelationType> {
        match self {
            // Temporal sequence, explanation, and result all push time forward
            Self::Narration | Self::Explanation | Self::Result => {
                Some(TemporalRelationType::Before)
            }
            // Background provides setting - contains the foreground event
            Self::Background => Some(TemporalRelationType::Contains),
            // Elaboration: the detail happens during the main event
            Self::Elaboration => Some(TemporalRelationType::During),
            // Parallel: events are simultaneous or overlapping
            Self::Parallel => Some(TemporalRelationType::Simultaneous),
            // Contrastive and topic relations don't impose temporal constraints
            Self::Contrast | Self::Concession | Self::TopicShift | Self::Continuation => None,
        }
    }

    /// Detect if this is a flashback context.
    ///
    /// A flashback occurs when:
    /// - We're in a Narration context (sequential telling)
    /// - The current sentence uses past perfect while previous was simple past
    ///
    /// This indicates a jump backward in narrative time.
    #[must_use]
    pub fn is_flashback_context(
        &self,
        prev_frame: Option<&TemporalFrame>,
        curr_frame: Option<&TemporalFrame>,
    ) -> bool {
        // Only detect flashbacks in Narration context
        if *self != Self::Narration {
            return false;
        }

        match (prev_frame, curr_frame) {
            (Some(prev), Some(curr)) => {
                // Flashback: previous was simple past, current is past perfect
                prev.is_simple_past() && curr.is_past_perfect()
            }
            _ => false,
        }
    }
}

/// Classification result with confidence score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoherenceClassification {
    /// The detected relation.
    pub relation: CoherenceRelation,
    /// Confidence score (0.0 - 1.0).
    pub confidence: f32,
    /// Primary signal that triggered this classification.
    pub primary_signal: CoherenceSignal,
    /// Supporting signals that contributed.
    pub supporting_signals: Vec<CoherenceSignal>,
    /// Surprisal delta: how much this relation reduces expected surprisal.
    /// Higher is better (more reduction in surprisal = more expected).
    /// None if no language model was used.
    pub surprisal_delta: Option<f64>,
}

/// Signals used for coherence detection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CoherenceSignal {
    /// Explicit discourse marker found.
    DiscourseMarker(String),
    /// Shared referents between segments.
    ParticipantOverlap(usize),
    /// Similar event predicates.
    EventSimilarity,
    /// Temporal ordering from tense/aspect.
    TemporalCue,
    /// Polarity contrast (negation).
    PolarityContrast,
    /// No shared entities detected.
    NoSharedEntities,
    /// Default fallback.
    Default,
}

/// Edge in the coherence graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoherenceEdge {
    /// Source sentence index.
    pub from_sentence: usize,
    /// Target sentence index.
    pub to_sentence: usize,
    /// Classification of the relation.
    pub classification: CoherenceClassification,
}

/// Graph of coherence relations between discourse segments.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CoherenceGraph {
    /// All edges (relations between sentences).
    edges: Vec<CoherenceEdge>,
    /// Sentence count tracked.
    sentence_count: usize,
}

impl CoherenceGraph {
    /// Create a new empty coherence graph.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an edge between two sentences.
    pub fn add_edge(&mut self, edge: CoherenceEdge) {
        self.sentence_count = self.sentence_count.max(edge.to_sentence + 1);
        self.edges.push(edge);
    }

    /// Get all edges from a given sentence.
    #[must_use]
    pub fn edges_from(&self, sentence: usize) -> Vec<&CoherenceEdge> {
        self.edges
            .iter()
            .filter(|e| e.from_sentence == sentence)
            .collect()
    }

    /// Get all edges to a given sentence.
    #[must_use]
    pub fn edges_to(&self, sentence: usize) -> Vec<&CoherenceEdge> {
        self.edges
            .iter()
            .filter(|e| e.to_sentence == sentence)
            .collect()
    }

    /// Get the relation between two specific sentences, if any.
    #[must_use]
    pub fn relation_between(&self, from: usize, to: usize) -> Option<&CoherenceEdge> {
        self.edges
            .iter()
            .find(|e| e.from_sentence == from && e.to_sentence == to)
    }

    /// Get all edges in the graph.
    #[must_use]
    pub fn all_edges(&self) -> &[CoherenceEdge] {
        &self.edges
    }

    /// Get the number of sentences tracked.
    #[must_use]
    pub fn sentence_count(&self) -> usize {
        self.sentence_count
    }
}

/// Input data for a single sentence in coherence classification.
#[derive(Debug, Default)]
pub struct SentenceData<'a> {
    /// Events from this sentence (if any).
    pub events: Option<&'a ComposedEvents>,
    /// Referent IDs introduced in this sentence.
    pub referents: &'a [ReferentId],
    /// Whether this sentence contains negation.
    pub has_negation: bool,
}

impl<'a> SentenceData<'a> {
    /// Create new sentence data.
    #[must_use]
    pub fn new(
        events: Option<&'a ComposedEvents>,
        referents: &'a [ReferentId],
        has_negation: bool,
    ) -> Self {
        Self {
            events,
            referents,
            has_negation,
        }
    }
}

/// Classifier for coherence relations.
#[derive(Debug, Clone)]
pub struct CoherenceClassifier {
    /// Marker → relation mappings.
    marker_map: Vec<(String, CoherenceRelation)>,
    /// Negation words for polarity detection.
    negation_words: Vec<String>,
}

impl Default for CoherenceClassifier {
    fn default() -> Self {
        Self::new()
    }
}

impl CoherenceClassifier {
    /// Create a new classifier with default marker mappings.
    #[must_use]
    pub fn new() -> Self {
        let mut marker_map = Vec::new();

        // Build marker → relation map from relation definitions
        for relation in [
            CoherenceRelation::Narration,
            CoherenceRelation::Background,
            CoherenceRelation::Explanation,
            CoherenceRelation::Result,
            CoherenceRelation::Contrast,
            CoherenceRelation::Concession,
            CoherenceRelation::Elaboration,
            CoherenceRelation::Parallel,
            CoherenceRelation::TopicShift,
            CoherenceRelation::Continuation,
        ] {
            for marker in relation.typical_markers() {
                marker_map.push((marker.to_lowercase(), relation));
            }
        }

        let negation_words = vec![
            "not".to_string(),
            "no".to_string(),
            "never".to_string(),
            "none".to_string(),
            "nobody".to_string(),
            "nothing".to_string(),
            "neither".to_string(),
            "nor".to_string(),
            "cannot".to_string(),
            "can't".to_string(),
            "won't".to_string(),
            "don't".to_string(),
            "doesn't".to_string(),
            "didn't".to_string(),
            "isn't".to_string(),
            "aren't".to_string(),
            "wasn't".to_string(),
            "weren't".to_string(),
        ];

        Self {
            marker_map,
            negation_words,
        }
    }

    /// Classify the coherence relation between previous and current segments.
    ///
    /// # Arguments
    /// * `prev` - Data from previous sentence
    /// * `curr` - Data from current sentence
    /// * `curr_tokens` - Tokens of current sentence (for marker detection)
    #[must_use]
    pub fn classify(
        &self,
        prev: &SentenceData<'_>,
        curr: &SentenceData<'_>,
        curr_tokens: &[String],
    ) -> CoherenceClassification {
        let mut signals = Vec::new();

        // 1. Check for explicit discourse markers (highest priority)
        if let Some((marker, relation)) = self.detect_marker(curr_tokens) {
            signals.push(CoherenceSignal::DiscourseMarker(marker.clone()));
            return Self::build_classification(
                relation,
                CoherenceSignal::DiscourseMarker(marker),
                signals,
                0.9,
            );
        }

        // 2. Check polarity contrast
        if prev.has_negation != curr.has_negation {
            signals.push(CoherenceSignal::PolarityContrast);
        }

        // 3. Check participant overlap
        let overlap_count = Self::count_referent_overlap(prev.referents, curr.referents);
        if overlap_count > 0 {
            signals.push(CoherenceSignal::ParticipantOverlap(overlap_count));
        }

        // 4. Check event similarity
        if let (Some(prev_ev), Some(curr_ev)) = (prev.events, curr.events) {
            if Self::events_similar(prev_ev, curr_ev) {
                signals.push(CoherenceSignal::EventSimilarity);
            }
        }

        // 5. Decide relation based on combined signals
        Self::decide_from_signals(&signals, overlap_count)
    }

    /// Detect discourse marker in tokens.
    fn detect_marker(&self, tokens: &[String]) -> Option<(String, CoherenceRelation)> {
        // Check first few tokens (discourse markers typically appear early)
        let check_range = tokens.len().min(5);
        let lower_tokens: Vec<String> = tokens[..check_range]
            .iter()
            .map(|t| t.to_lowercase())
            .collect();

        // Check multi-word markers first
        let joined = lower_tokens.join(" ");
        for (marker, relation) in &self.marker_map {
            if marker.contains(' ') && joined.starts_with(marker) {
                return Some((marker.clone(), *relation));
            }
        }

        // Check single-word markers
        for token in &lower_tokens {
            for (marker, relation) in &self.marker_map {
                if !marker.contains(' ') && token == marker {
                    return Some((marker.clone(), *relation));
                }
            }
        }

        None
    }

    /// Count overlapping referents between two sets.
    fn count_referent_overlap(prev: &[ReferentId], curr: &[ReferentId]) -> usize {
        prev.iter().filter(|p| curr.contains(p)).count()
    }

    /// Check if events are similar (same predicate type).
    fn events_similar(prev: &ComposedEvents, curr: &ComposedEvents) -> bool {
        // Simple heuristic: check if predicates overlap
        for p_event in &prev.events {
            for c_event in &curr.events {
                if p_event.predicate.eq_ignore_ascii_case(&c_event.predicate) {
                    return true;
                }
            }
        }
        false
    }

    /// Check if tokens contain negation.
    #[must_use]
    pub fn has_negation(&self, tokens: &[String]) -> bool {
        tokens
            .iter()
            .any(|t| self.negation_words.contains(&t.to_lowercase()))
    }

    /// Decide relation from accumulated signals.
    fn decide_from_signals(
        signals: &[CoherenceSignal],
        overlap_count: usize,
    ) -> CoherenceClassification {
        // Priority ordering for non-marker signals:
        // 1. Polarity contrast → Contrast
        // 2. No overlap → TopicShift
        // 3. High overlap + event similarity → Elaboration or Parallel
        // 4. Some overlap → Continuation or Narration

        if signals.contains(&CoherenceSignal::PolarityContrast) {
            return Self::build_classification(
                CoherenceRelation::Contrast,
                CoherenceSignal::PolarityContrast,
                signals.to_vec(),
                0.7,
            );
        }

        if overlap_count == 0 {
            return Self::build_classification(
                CoherenceRelation::TopicShift,
                CoherenceSignal::NoSharedEntities,
                signals.to_vec(),
                0.6,
            );
        }

        if signals.contains(&CoherenceSignal::EventSimilarity) {
            // Same predicate suggests parallel structure
            return Self::build_classification(
                CoherenceRelation::Parallel,
                CoherenceSignal::EventSimilarity,
                signals.to_vec(),
                0.65,
            );
        }

        if overlap_count >= 2 {
            // High overlap suggests elaboration
            return Self::build_classification(
                CoherenceRelation::Elaboration,
                CoherenceSignal::ParticipantOverlap(overlap_count),
                signals.to_vec(),
                0.6,
            );
        }

        // Default: Continuation (same topic, moving forward)
        let primary = if overlap_count > 0 {
            CoherenceSignal::ParticipantOverlap(overlap_count)
        } else {
            CoherenceSignal::Default
        };

        Self::build_classification(
            CoherenceRelation::Continuation,
            primary,
            signals.to_vec(),
            0.5,
        )
    }

    /// Build a classification result.
    fn build_classification(
        relation: CoherenceRelation,
        primary: CoherenceSignal,
        mut signals: Vec<CoherenceSignal>,
        confidence: f32,
    ) -> CoherenceClassification {
        // Remove primary from supporting signals
        signals.retain(|s| *s != primary);

        CoherenceClassification {
            relation,
            confidence,
            primary_signal: primary,
            supporting_signals: signals,
            surprisal_delta: None, // Set by surprisal-aware methods
        }
    }

    /// Adjust confidence based on surprisal from a language model.
    ///
    /// Uses the LM to estimate how expected the current sentence is given
    /// the relation. More expected continuations get higher confidence.
    ///
    /// # Arguments
    /// * `classification` - The initial classification to adjust
    /// * `prev_tokens` - Tokens from the previous sentence
    /// * `curr_tokens` - Tokens from the current sentence
    /// * `lm` - Surprisal model for surprisal computation
    pub fn adjust_with_surprisal<L: crate::kernel::incremental::SurprisalModel + ?Sized>(
        &self,
        mut classification: CoherenceClassification,
        prev_tokens: &[String],
        curr_tokens: &[String],
        lm: &L,
    ) -> CoherenceClassification {
        // Baseline surprisal: how surprising is the current sentence with no context?
        let curr_refs: Vec<&str> = curr_tokens
            .iter()
            .map(std::string::String::as_str)
            .collect();
        let baseline = lm.sentence_surprisal(&curr_refs);

        // Contextual surprisal: how surprising given the previous sentence?
        let context: Vec<&str> = prev_tokens
            .iter()
            .chain(curr_tokens.iter())
            .map(std::string::String::as_str)
            .collect();

        // Get surprisal for current sentence starting from where previous ended
        let mut contextual = crate::kernel::incremental::Surprisal::ZERO;
        for i in prev_tokens.len()..context.len() {
            let prefix = &context[..i];
            contextual += lm.word_surprisal(context[i], prefix);
        }

        // Delta: how much surprisal was reduced by having context
        // Positive delta means the relation made the sentence more expected
        let delta = baseline.bits() - contextual.bits();
        classification.surprisal_delta = Some(delta);

        // Adjust confidence based on delta
        // More expected (higher delta) = higher confidence
        // Clamp delta to [-5, 5] then scale to [-0.5, 0.5] for confidence adjustment
        // Use integer bucketing to avoid f64->f32 truncation lint
        let boost = match delta.clamp(-5.0, 5.0).round() {
            x if x <= -5.0 => -0.5,
            x if x <= -4.0 => -0.4,
            x if x <= -3.0 => -0.3,
            x if x <= -2.0 => -0.2,
            x if x <= -1.0 => -0.1,
            x if x <= 0.0 => 0.0,
            x if x <= 1.0 => 0.1,
            x if x <= 2.0 => 0.2,
            x if x <= 3.0 => 0.3,
            x if x <= 4.0 => 0.4,
            _ => 0.5,
        };
        classification.confidence = (classification.confidence + boost).clamp(0.0, 1.0);

        classification
    }

    /// Infer temporal constraints from coherence classification.
    ///
    /// Returns the temporal constraint between events in the two sentences,
    /// based on the coherence relation and any flashback detection.
    ///
    /// # Arguments
    /// * `classification` - The coherence classification result
    /// * `prev_frame` - Temporal frame from previous sentence's main event
    /// * `curr_frame` - Temporal frame from current sentence's main event
    ///
    /// # Returns
    /// A tuple of (constraint, `is_flashback`) where constraint is the temporal
    /// relation to enforce between the events.
    #[must_use]
    pub fn infer_temporal_constraints(
        &self,
        classification: &CoherenceClassification,
        prev_frame: Option<&TemporalFrame>,
        curr_frame: Option<&TemporalFrame>,
    ) -> (Option<TemporalRelationType>, bool) {
        let relation = classification.relation;

        // Check for flashback (past perfect in narration context)
        let is_flashback = relation.is_flashback_context(prev_frame, curr_frame);

        if is_flashback {
            // In a flashback, the event ordering is reversed from typical narration
            // e2 (past perfect) happened before e1 (simple past)
            return (Some(TemporalRelationType::After), true);
        }

        // Get default constraint from coherence relation
        let constraint = relation.default_temporal_constraint();

        (constraint, false)
    }
}

/// Result of temporal constraint inference.
#[derive(Debug, Clone)]
pub struct TemporalConstraintResult {
    /// The temporal constraint (if any).
    pub constraint: Option<TemporalRelationType>,
    /// Whether this represents a flashback.
    pub is_flashback: bool,
    /// The coherence relation that induced this constraint.
    pub source_relation: CoherenceRelation,
}

impl TemporalConstraintResult {
    /// Create a new result.
    #[must_use]
    pub fn new(
        constraint: Option<TemporalRelationType>,
        is_flashback: bool,
        source_relation: CoherenceRelation,
    ) -> Self {
        Self {
            constraint,
            is_flashback,
            source_relation,
        }
    }
}

/// Helper for tracking referents per sentence.
#[derive(Debug, Clone, Default)]
pub struct SentenceReferents {
    /// Referents introduced in each sentence.
    sentence_referents: Vec<Vec<ReferentId>>,
}

impl SentenceReferents {
    /// Create new tracker.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record referents for a sentence.
    pub fn record(&mut self, sentence: usize, referents: Vec<ReferentId>) {
        // Extend if needed
        while self.sentence_referents.len() <= sentence {
            self.sentence_referents.push(Vec::new());
        }
        self.sentence_referents[sentence] = referents;
    }

    /// Get referents for a sentence.
    #[must_use]
    pub fn get(&self, sentence: usize) -> &[ReferentId] {
        self.sentence_referents
            .get(sentence)
            .map_or(&[], Vec::as_slice)
    }

    /// Extract referent IDs from registry that were introduced at a specific sentence.
    #[must_use]
    pub fn extract_from_registry(registry: &ReferentRegistry, sentence: usize) -> Vec<ReferentId> {
        // Combine entities and events
        registry
            .entities()
            .into_iter()
            .chain(registry.events())
            .filter(|r| r.introduced_at == sentence)
            .map(|r| r.id)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relation_properties() {
        assert!(CoherenceRelation::Narration.is_coordinating());
        assert!(!CoherenceRelation::Narration.is_subordinating());

        assert!(CoherenceRelation::Explanation.is_subordinating());
        assert!(!CoherenceRelation::Explanation.is_coordinating());
    }

    #[test]
    fn test_marker_detection_single() {
        let classifier = CoherenceClassifier::new();
        let tokens = vec!["However".to_string(), "the".to_string(), "cat".to_string()];
        let prev = SentenceData::default();
        let curr = SentenceData::default();

        let result = classifier.classify(&prev, &curr, &tokens);

        assert_eq!(result.relation, CoherenceRelation::Contrast);
        assert!(result.confidence > 0.8);
        assert!(matches!(
            result.primary_signal,
            CoherenceSignal::DiscourseMarker(_)
        ));
    }

    #[test]
    fn test_marker_detection_then() {
        let classifier = CoherenceClassifier::new();
        let tokens = vec!["Then".to_string(), "he".to_string(), "left".to_string()];
        let prev = SentenceData::default();
        let curr = SentenceData::default();

        let result = classifier.classify(&prev, &curr, &tokens);

        assert_eq!(result.relation, CoherenceRelation::Narration);
    }

    #[test]
    fn test_marker_detection_because() {
        let classifier = CoherenceClassifier::new();
        let tokens = vec![
            "Because".to_string(),
            "it".to_string(),
            "rained".to_string(),
        ];
        let prev = SentenceData::default();
        let curr = SentenceData::default();

        let result = classifier.classify(&prev, &curr, &tokens);

        assert_eq!(result.relation, CoherenceRelation::Explanation);
    }

    #[test]
    fn test_polarity_contrast() {
        let classifier = CoherenceClassifier::new();
        let tokens = vec!["The".to_string(), "cat".to_string(), "sat".to_string()];
        let prev = SentenceData::new(None, &[], true);
        let curr = SentenceData::new(None, &[], false);

        let result = classifier.classify(&prev, &curr, &tokens);

        assert_eq!(result.relation, CoherenceRelation::Contrast);
        assert!(matches!(
            result.primary_signal,
            CoherenceSignal::PolarityContrast
        ));
    }

    #[test]
    fn test_topic_shift_no_overlap() {
        let classifier = CoherenceClassifier::new();
        let tokens = vec![
            "The".to_string(),
            "weather".to_string(),
            "changed".to_string(),
        ];

        // Different referent sets with no overlap
        let prev_refs = vec![ReferentId::new(0), ReferentId::new(1)];
        let curr_refs = vec![ReferentId::new(2), ReferentId::new(3)];
        let prev = SentenceData::new(None, &prev_refs, false);
        let curr = SentenceData::new(None, &curr_refs, false);

        let result = classifier.classify(&prev, &curr, &tokens);

        assert_eq!(result.relation, CoherenceRelation::TopicShift);
    }

    #[test]
    fn test_continuation_with_overlap() {
        let classifier = CoherenceClassifier::new();
        let tokens = vec!["The".to_string(), "cat".to_string(), "slept".to_string()];

        // Same referent appears in both
        let prev_refs = vec![ReferentId::new(0), ReferentId::new(1)];
        let curr_refs = vec![ReferentId::new(0), ReferentId::new(2)];
        let prev = SentenceData::new(None, &prev_refs, false);
        let curr = SentenceData::new(None, &curr_refs, false);

        let result = classifier.classify(&prev, &curr, &tokens);

        assert_eq!(result.relation, CoherenceRelation::Continuation);
    }

    #[test]
    fn test_elaboration_high_overlap() {
        let classifier = CoherenceClassifier::new();
        let tokens = vec!["It".to_string(), "was".to_string(), "large".to_string()];

        // Multiple shared referents
        let prev_refs = vec![ReferentId::new(0), ReferentId::new(1), ReferentId::new(2)];
        let curr_refs = vec![ReferentId::new(0), ReferentId::new(1), ReferentId::new(3)];
        let prev = SentenceData::new(None, &prev_refs, false);
        let curr = SentenceData::new(None, &curr_refs, false);

        let result = classifier.classify(&prev, &curr, &tokens);

        assert_eq!(result.relation, CoherenceRelation::Elaboration);
    }

    #[test]
    fn test_negation_detection() {
        let classifier = CoherenceClassifier::new();

        assert!(classifier.has_negation(&[
            "The".to_string(),
            "cat".to_string(),
            "did".to_string(),
            "not".to_string(),
            "sit".to_string()
        ]));
        assert!(classifier.has_negation(&["Never".to_string(), "again".to_string()]));
        assert!(!classifier.has_negation(&[
            "The".to_string(),
            "cat".to_string(),
            "sat".to_string()
        ]));
    }

    #[test]
    fn test_coherence_graph() {
        let mut graph = CoherenceGraph::new();

        let edge = CoherenceEdge {
            from_sentence: 0,
            to_sentence: 1,
            classification: CoherenceClassification {
                relation: CoherenceRelation::Narration,
                confidence: 0.9,
                primary_signal: CoherenceSignal::DiscourseMarker("then".to_string()),
                supporting_signals: vec![],
                surprisal_delta: None,
            },
        };

        graph.add_edge(edge);

        assert_eq!(graph.sentence_count(), 2);
        assert_eq!(graph.edges_from(0).len(), 1);
        assert_eq!(graph.edges_to(1).len(), 1);
        assert!(graph.relation_between(0, 1).is_some());
        assert!(graph.relation_between(1, 0).is_none());
    }

    #[test]
    fn test_sentence_referents_tracker() {
        let mut tracker = SentenceReferents::new();

        tracker.record(0, vec![ReferentId::new(0), ReferentId::new(1)]);
        tracker.record(1, vec![ReferentId::new(1), ReferentId::new(2)]);

        assert_eq!(tracker.get(0).len(), 2);
        assert_eq!(tracker.get(1).len(), 2);
        assert_eq!(tracker.get(2).len(), 0); // Not recorded yet
    }

    // Edge case tests

    #[test]
    fn test_empty_tokens() {
        let classifier = CoherenceClassifier::new();
        let tokens: Vec<String> = vec![];
        let prev = SentenceData::default();
        let curr = SentenceData::default();

        // Should not panic, should return default classification
        let result = classifier.classify(&prev, &curr, &tokens);

        // With no tokens and no referents, defaults to TopicShift
        assert!(matches!(
            result.relation,
            CoherenceRelation::TopicShift | CoherenceRelation::Continuation
        ));
    }

    #[test]
    fn test_single_token() {
        let classifier = CoherenceClassifier::new();
        let tokens = vec!["Hello".to_string()];
        let prev = SentenceData::default();
        let curr = SentenceData::default();

        // Should not panic
        let result = classifier.classify(&prev, &curr, &tokens);
        assert!(result.confidence >= 0.0 && result.confidence <= 1.0);
    }

    #[test]
    fn test_empty_referents() {
        let classifier = CoherenceClassifier::new();
        let tokens = vec!["The".to_string(), "cat".to_string()];
        let prev = SentenceData::new(None, &[], false);
        let curr = SentenceData::new(None, &[], false);

        // Should not panic, no shared referents means TopicShift or Continuation
        let result = classifier.classify(&prev, &curr, &tokens);
        assert!(result.confidence >= 0.0);
    }

    #[test]
    fn test_coherence_graph_empty() {
        let graph = CoherenceGraph::new();

        assert_eq!(graph.sentence_count(), 0);
        assert!(graph.edges_from(0).is_empty());
        assert!(graph.edges_to(0).is_empty());
        assert!(graph.relation_between(0, 1).is_none());
    }

    #[test]
    fn test_sentence_referents_empty_get() {
        let tracker = SentenceReferents::new();

        // Getting from unrecorded sentence should return empty
        assert!(tracker.get(0).is_empty());
        assert!(tracker.get(100).is_empty());
    }

    #[test]
    fn test_has_negation_empty() {
        let classifier = CoherenceClassifier::new();
        let tokens: Vec<String> = vec![];

        assert!(!classifier.has_negation(&tokens));
    }

    // Temporal integration tests

    #[test]
    fn test_default_temporal_constraint_narration() {
        let constraint = CoherenceRelation::Narration.default_temporal_constraint();
        assert_eq!(constraint, Some(TemporalRelationType::Before));
    }

    #[test]
    fn test_default_temporal_constraint_background() {
        let constraint = CoherenceRelation::Background.default_temporal_constraint();
        assert_eq!(constraint, Some(TemporalRelationType::Contains));
    }

    #[test]
    fn test_default_temporal_constraint_result() {
        let constraint = CoherenceRelation::Result.default_temporal_constraint();
        assert_eq!(constraint, Some(TemporalRelationType::Before));
    }

    #[test]
    fn test_default_temporal_constraint_parallel() {
        let constraint = CoherenceRelation::Parallel.default_temporal_constraint();
        assert_eq!(constraint, Some(TemporalRelationType::Simultaneous));
    }

    #[test]
    fn test_default_temporal_constraint_contrast_none() {
        // Contrast doesn't impose temporal constraints
        let constraint = CoherenceRelation::Contrast.default_temporal_constraint();
        assert!(constraint.is_none());
    }

    #[test]
    fn test_flashback_detection() {
        let prev_frame = TemporalFrame::past();
        let curr_frame = TemporalFrame::past_perfect();

        // Flashback: simple past followed by past perfect in narration
        let is_flashback =
            CoherenceRelation::Narration.is_flashback_context(Some(&prev_frame), Some(&curr_frame));
        assert!(is_flashback);

        // Not a flashback if not narration
        let is_flashback =
            CoherenceRelation::Contrast.is_flashback_context(Some(&prev_frame), Some(&curr_frame));
        assert!(!is_flashback);

        // Not a flashback if frames are the same tense
        let same_frame = TemporalFrame::past();
        let is_flashback =
            CoherenceRelation::Narration.is_flashback_context(Some(&prev_frame), Some(&same_frame));
        assert!(!is_flashback);
    }

    #[test]
    fn test_infer_temporal_constraints_narration() {
        let classifier = CoherenceClassifier::new();
        let classification = CoherenceClassification {
            relation: CoherenceRelation::Narration,
            confidence: 0.9,
            primary_signal: CoherenceSignal::DiscourseMarker("then".to_string()),
            supporting_signals: vec![],
            surprisal_delta: None,
        };

        let prev_frame = TemporalFrame::past();
        let curr_frame = TemporalFrame::past();

        let (constraint, is_flashback) = classifier.infer_temporal_constraints(
            &classification,
            Some(&prev_frame),
            Some(&curr_frame),
        );

        assert_eq!(constraint, Some(TemporalRelationType::Before));
        assert!(!is_flashback);
    }

    #[test]
    fn test_infer_temporal_constraints_flashback() {
        let classifier = CoherenceClassifier::new();
        let classification = CoherenceClassification {
            relation: CoherenceRelation::Narration,
            confidence: 0.9,
            primary_signal: CoherenceSignal::TemporalCue,
            supporting_signals: vec![],
            surprisal_delta: None,
        };

        // Flashback scenario: simple past then past perfect
        let prev_frame = TemporalFrame::past();
        let curr_frame = TemporalFrame::past_perfect();

        let (constraint, is_flashback) = classifier.infer_temporal_constraints(
            &classification,
            Some(&prev_frame),
            Some(&curr_frame),
        );

        // Flashback reverses the constraint
        assert_eq!(constraint, Some(TemporalRelationType::After));
        assert!(is_flashback);
    }

    #[test]
    fn test_temporal_constraint_result_struct() {
        let result = TemporalConstraintResult::new(
            Some(TemporalRelationType::Before),
            false,
            CoherenceRelation::Narration,
        );

        assert_eq!(result.constraint, Some(TemporalRelationType::Before));
        assert!(!result.is_flashback);
        assert_eq!(result.source_relation, CoherenceRelation::Narration);
    }
}
