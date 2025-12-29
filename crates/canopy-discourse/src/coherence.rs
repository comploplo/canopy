//! Discourse Coherence Relations
//!
//! Implements coherence relation detection based on:
//! - Hobbs (1979) "Coherence and Coreference"
//! - Asher & Lascarides (2003) "Logics of Conversation" (SDRT)
//!
//! Coherence relations describe how discourse segments connect semantically.

use crate::referent::ReferentId;
use crate::temporal::{AllenRelation, TemporalReasoner};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Discourse coherence relations
///
/// Based on Hobbs (1979) and Asher & Lascarides (2003) SDRT.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CoherenceRelation {
    // === Cause-Effect Relations ===
    /// e1 causes e2: "John pushed Bill. He fell."
    Result,

    /// e2 explains e1: "Bill fell. John pushed him."
    Explanation,

    // === Temporal Relations ===
    /// e1 then e2 (narrative progression): "John entered. He sat down."
    Narration,

    /// e2 provides temporal background for e1: "John was reading. The phone rang."
    Background,

    // === Similarity Relations ===
    /// e1 and e2 share structure: "John likes Mary. Bill likes Sue."
    Parallel,

    /// e1 and e2 contrast: "John won. Bill lost."
    Contrast,

    // === Elaboration Relations ===
    /// e2 elaborates on e1: "John cleaned the room. He dusted the shelves."
    Elaboration,

    /// e2 exemplifies e1: "John has hobbies. He plays chess."
    Exemplification,

    // === Default ===
    /// Generic continuation (no specific relation detected)
    Continuation,
}

impl CoherenceRelation {
    /// Coherence strength score (some relations are stronger indicators)
    #[must_use]
    pub fn strength(self) -> f32 {
        match self {
            Self::Result | Self::Explanation => 1.0,
            Self::Contrast => 0.95,
            Self::Elaboration => 0.9,
            Self::Parallel => 0.85,
            Self::Narration => 0.8,
            Self::Background => 0.75,
            Self::Exemplification => 0.7,
            Self::Continuation => 0.5,
        }
    }

    /// Whether this relation implies temporal ordering
    #[must_use]
    pub fn implies_temporal_order(self) -> bool {
        matches!(
            self,
            Self::Result | Self::Explanation | Self::Narration | Self::Background
        )
    }
}

/// A discourse segment for coherence analysis
#[derive(Debug, Clone)]
pub struct DrsSegment {
    /// Segment identifier (typically sentence index)
    pub id: usize,

    /// Main event/predicate of the segment
    pub main_event: Option<ReferentId>,

    /// Main predicate name
    pub predicate: Option<String>,

    /// Entities mentioned in this segment
    pub entities: Vec<ReferentId>,

    /// Leading discourse marker (if any)
    pub discourse_marker: Option<String>,
}

impl DrsSegment {
    /// Create a new segment
    #[must_use]
    pub fn new(id: usize) -> Self {
        Self {
            id,
            main_event: None,
            predicate: None,
            entities: Vec::new(),
            discourse_marker: None,
        }
    }

    /// Add an entity to the segment
    pub fn add_entity(&mut self, entity: ReferentId) {
        if !self.entities.contains(&entity) {
            self.entities.push(entity);
        }
    }
}

/// Coherence relation analyzer
///
/// Detects coherence relations between discourse segments using:
/// - Discourse markers (explicit cues)
/// - VerbNet causative patterns
/// - Shared referents
/// - Temporal relations
#[derive(Debug, Clone)]
pub struct CoherenceAnalyzer {
    /// Discourse markers mapped to relations
    discourse_markers: HashMap<String, CoherenceRelation>,

    /// Causative verbs (from VerbNet cause classes)
    causative_verbs: HashSet<String>,

    /// Antonym pairs for contrast detection
    antonym_pairs: HashMap<String, String>,

    /// Part-whole relations for elaboration detection
    part_whole: HashMap<String, Vec<String>>,
}

impl CoherenceAnalyzer {
    /// Create a new coherence analyzer with default lexicon
    #[must_use]
    pub fn new() -> Self {
        Self {
            discourse_markers: Self::default_discourse_markers(),
            causative_verbs: Self::default_causative_verbs(),
            antonym_pairs: Self::default_antonyms(),
            part_whole: Self::default_part_whole(),
        }
    }

    /// Default discourse markers
    fn default_discourse_markers() -> HashMap<String, CoherenceRelation> {
        let mut markers = HashMap::new();

        // Result/Cause markers
        markers.insert("therefore".to_string(), CoherenceRelation::Result);
        markers.insert("thus".to_string(), CoherenceRelation::Result);
        markers.insert("consequently".to_string(), CoherenceRelation::Result);
        markers.insert("as a result".to_string(), CoherenceRelation::Result);
        markers.insert("so".to_string(), CoherenceRelation::Result);
        markers.insert("hence".to_string(), CoherenceRelation::Result);

        // Explanation markers
        markers.insert("because".to_string(), CoherenceRelation::Explanation);
        markers.insert("since".to_string(), CoherenceRelation::Explanation);
        markers.insert("for".to_string(), CoherenceRelation::Explanation);

        // Contrast markers
        markers.insert("however".to_string(), CoherenceRelation::Contrast);
        markers.insert("but".to_string(), CoherenceRelation::Contrast);
        markers.insert("although".to_string(), CoherenceRelation::Contrast);
        markers.insert("nevertheless".to_string(), CoherenceRelation::Contrast);
        markers.insert("yet".to_string(), CoherenceRelation::Contrast);
        markers.insert("on the other hand".to_string(), CoherenceRelation::Contrast);
        markers.insert("in contrast".to_string(), CoherenceRelation::Contrast);

        // Narration markers
        markers.insert("then".to_string(), CoherenceRelation::Narration);
        markers.insert("next".to_string(), CoherenceRelation::Narration);
        markers.insert("afterward".to_string(), CoherenceRelation::Narration);
        markers.insert("afterwards".to_string(), CoherenceRelation::Narration);
        markers.insert("later".to_string(), CoherenceRelation::Narration);
        markers.insert("subsequently".to_string(), CoherenceRelation::Narration);

        // Background markers
        markers.insert("meanwhile".to_string(), CoherenceRelation::Background);
        markers.insert(
            "at the same time".to_string(),
            CoherenceRelation::Background,
        );
        markers.insert("while".to_string(), CoherenceRelation::Background);

        // Elaboration markers
        markers.insert("specifically".to_string(), CoherenceRelation::Elaboration);
        markers.insert("in particular".to_string(), CoherenceRelation::Elaboration);
        markers.insert("namely".to_string(), CoherenceRelation::Elaboration);
        markers.insert("that is".to_string(), CoherenceRelation::Elaboration);

        // Exemplification markers
        markers.insert(
            "for example".to_string(),
            CoherenceRelation::Exemplification,
        );
        markers.insert(
            "for instance".to_string(),
            CoherenceRelation::Exemplification,
        );
        markers.insert("such as".to_string(), CoherenceRelation::Exemplification);

        markers
    }

    /// Default causative verbs (VerbNet cause classes)
    fn default_causative_verbs() -> HashSet<String> {
        let verbs = [
            // Physical causation
            "push", "pull", "hit", "strike", "break", "kill", "destroy",
            // Psychological causation
            "frighten", "scare", "upset", "anger", "please", "annoy", // Caused motion
            "throw", "drop", "move", "send", // Caused change of state
            "melt", "freeze", "open", "close", "bend", "fold",
        ];
        verbs.iter().map(|&s| s.to_string()).collect()
    }

    /// Default antonym pairs
    fn default_antonyms() -> HashMap<String, String> {
        let pairs = [
            ("win", "lose"),
            ("buy", "sell"),
            ("come", "go"),
            ("open", "close"),
            ("start", "stop"),
            ("love", "hate"),
            ("succeed", "fail"),
            ("rise", "fall"),
            ("increase", "decrease"),
            ("accept", "reject"),
        ];

        let mut map = HashMap::new();
        for (a, b) in pairs {
            map.insert(a.to_string(), b.to_string());
            map.insert(b.to_string(), a.to_string());
        }
        map
    }

    /// Default part-whole relations for elaboration
    fn default_part_whole() -> HashMap<String, Vec<String>> {
        let mut map = HashMap::new();

        map.insert(
            "clean".to_string(),
            vec![
                "dust".to_string(),
                "sweep".to_string(),
                "mop".to_string(),
                "vacuum".to_string(),
            ],
        );
        map.insert(
            "cook".to_string(),
            vec![
                "chop".to_string(),
                "stir".to_string(),
                "fry".to_string(),
                "boil".to_string(),
            ],
        );
        map.insert(
            "write".to_string(),
            vec![
                "draft".to_string(),
                "edit".to_string(),
                "revise".to_string(),
                "type".to_string(),
            ],
        );

        map
    }

    /// Infer coherence relation between two segments
    ///
    /// Returns the relation and a confidence score.
    #[must_use]
    pub fn infer_relation(
        &self,
        segment1: &DrsSegment,
        segment2: &DrsSegment,
        temporal_reasoner: Option<&TemporalReasoner>,
    ) -> (CoherenceRelation, f32) {
        // 1. Check for explicit discourse marker (highest confidence)
        if let Some(marker) = &segment2.discourse_marker {
            if let Some(&relation) = self.discourse_markers.get(&marker.to_lowercase()) {
                return (relation, 0.95);
            }
        }

        // 2. Check for shared referents
        let shared: Vec<_> = segment1
            .entities
            .iter()
            .filter(|e| segment2.entities.contains(e))
            .collect();

        // 3. Check for causation pattern
        if let (Some(pred1), Some(pred2)) = (&segment1.predicate, &segment2.predicate) {
            // Check if first predicate is causative
            if self.causative_verbs.contains(&pred1.to_lowercase()) {
                // Causative verb followed by result state
                return (CoherenceRelation::Result, 0.8);
            }

            // Check for antonym (contrast)
            if let Some(antonym) = self.antonym_pairs.get(&pred1.to_lowercase()) {
                if antonym == &pred2.to_lowercase() {
                    return (CoherenceRelation::Contrast, 0.85);
                }
            }

            // Check for part-whole (elaboration)
            if let Some(parts) = self.part_whole.get(&pred1.to_lowercase()) {
                if parts.contains(&pred2.to_lowercase()) {
                    return (CoherenceRelation::Elaboration, 0.8);
                }
            }
        }

        // 4. Check temporal relation for narrative/background
        if let (Some(e1), Some(e2), Some(reasoner)) =
            (segment1.main_event, segment2.main_event, temporal_reasoner)
        {
            if let Some(temporal_rel) = reasoner.get_relation(e1, e2) {
                match temporal_rel {
                    AllenRelation::Before | AllenRelation::Meets => {
                        return (CoherenceRelation::Narration, 0.75);
                    }
                    AllenRelation::Contains | AllenRelation::During => {
                        return (CoherenceRelation::Background, 0.7);
                    }
                    _ => {}
                }
            }
        }

        // 5. Shared referents suggest continuation
        if !shared.is_empty() {
            // Multiple shared referents might suggest parallel structure
            if shared.len() >= 2 {
                return (CoherenceRelation::Parallel, 0.6);
            }
            return (CoherenceRelation::Continuation, 0.5);
        }

        // Default: continuation
        (CoherenceRelation::Continuation, 0.3)
    }

    /// Calculate overall coherence score for a discourse
    ///
    /// Analyzes pairwise relations and computes average coherence.
    #[must_use]
    pub fn discourse_coherence_score(&self, segments: &[DrsSegment]) -> f32 {
        if segments.len() < 2 {
            return 1.0; // Single segment is maximally coherent
        }

        let mut total_score = 0.0;
        let mut count = 0;

        for window in segments.windows(2) {
            let (relation, confidence) = self.infer_relation(&window[0], &window[1], None);
            total_score += relation.strength() * confidence;
            count += 1;
        }

        if count > 0 {
            total_score / count as f32
        } else {
            1.0
        }
    }

    /// Get all coherence relations in a discourse
    #[must_use]
    pub fn analyze_discourse(
        &self,
        segments: &[DrsSegment],
    ) -> Vec<(usize, usize, CoherenceRelation, f32)> {
        let mut relations = Vec::new();

        for window in segments.windows(2) {
            let (relation, confidence) = self.infer_relation(&window[0], &window[1], None);
            relations.push((window[0].id, window[1].id, relation, confidence));
        }

        relations
    }

    /// Check if a word is a discourse marker
    #[must_use]
    pub fn is_discourse_marker(&self, word: &str) -> bool {
        self.discourse_markers.contains_key(&word.to_lowercase())
    }

    /// Get the relation indicated by a discourse marker
    #[must_use]
    pub fn marker_relation(&self, marker: &str) -> Option<CoherenceRelation> {
        self.discourse_markers.get(&marker.to_lowercase()).copied()
    }
}

impl Default for CoherenceAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discourse_marker_detection() {
        let analyzer = CoherenceAnalyzer::new();

        assert!(analyzer.is_discourse_marker("however"));
        assert!(analyzer.is_discourse_marker("Therefore"));
        assert!(analyzer.is_discourse_marker("then"));
        assert!(!analyzer.is_discourse_marker("quickly"));
    }

    #[test]
    fn test_marker_relations() {
        let analyzer = CoherenceAnalyzer::new();

        assert_eq!(
            analyzer.marker_relation("however"),
            Some(CoherenceRelation::Contrast)
        );
        assert_eq!(
            analyzer.marker_relation("therefore"),
            Some(CoherenceRelation::Result)
        );
        assert_eq!(
            analyzer.marker_relation("then"),
            Some(CoherenceRelation::Narration)
        );
    }

    #[test]
    fn test_result_from_marker() {
        let analyzer = CoherenceAnalyzer::new();

        let mut seg1 = DrsSegment::new(0);
        seg1.predicate = Some("push".to_string());
        seg1.add_entity(ReferentId(1));
        seg1.add_entity(ReferentId(2));

        let mut seg2 = DrsSegment::new(1);
        seg2.predicate = Some("fall".to_string());
        seg2.discourse_marker = Some("therefore".to_string());
        seg2.add_entity(ReferentId(2));

        let (relation, confidence) = analyzer.infer_relation(&seg1, &seg2, None);
        assert_eq!(relation, CoherenceRelation::Result);
        assert!(confidence > 0.9);
    }

    #[test]
    fn test_contrast_from_antonyms() {
        let analyzer = CoherenceAnalyzer::new();

        let mut seg1 = DrsSegment::new(0);
        seg1.predicate = Some("win".to_string());
        seg1.add_entity(ReferentId(1));

        let mut seg2 = DrsSegment::new(1);
        seg2.predicate = Some("lose".to_string());
        seg2.add_entity(ReferentId(2));

        let (relation, confidence) = analyzer.infer_relation(&seg1, &seg2, None);
        assert_eq!(relation, CoherenceRelation::Contrast);
        assert!(confidence > 0.8);
    }

    #[test]
    fn test_causative_result() {
        let analyzer = CoherenceAnalyzer::new();

        let mut seg1 = DrsSegment::new(0);
        seg1.predicate = Some("push".to_string());
        seg1.add_entity(ReferentId(1));
        seg1.add_entity(ReferentId(2));

        let mut seg2 = DrsSegment::new(1);
        seg2.predicate = Some("fall".to_string());
        seg2.add_entity(ReferentId(2));

        let (relation, _) = analyzer.infer_relation(&seg1, &seg2, None);
        assert_eq!(relation, CoherenceRelation::Result);
    }

    #[test]
    fn test_elaboration_from_part_whole() {
        let analyzer = CoherenceAnalyzer::new();

        let mut seg1 = DrsSegment::new(0);
        seg1.predicate = Some("clean".to_string());
        seg1.add_entity(ReferentId(1));

        let mut seg2 = DrsSegment::new(1);
        seg2.predicate = Some("dust".to_string());
        seg2.add_entity(ReferentId(1));

        let (relation, _) = analyzer.infer_relation(&seg1, &seg2, None);
        assert_eq!(relation, CoherenceRelation::Elaboration);
    }

    #[test]
    fn test_discourse_coherence_score() {
        let analyzer = CoherenceAnalyzer::new();

        // Coherent narrative with explicit markers
        let mut seg1 = DrsSegment::new(0);
        seg1.predicate = Some("enter".to_string());
        seg1.add_entity(ReferentId(1));

        let mut seg2 = DrsSegment::new(1);
        seg2.predicate = Some("sit".to_string());
        seg2.discourse_marker = Some("then".to_string());
        seg2.add_entity(ReferentId(1));

        let score = analyzer.discourse_coherence_score(&[seg1, seg2]);
        assert!(score > 0.5);
    }

    #[test]
    fn test_relation_strength() {
        assert!(CoherenceRelation::Result.strength() > CoherenceRelation::Continuation.strength());
        assert!(CoherenceRelation::Contrast.strength() > CoherenceRelation::Narration.strength());
    }
}
