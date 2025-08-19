//! Simple movement signal detection
//!
//! This module detects syntactic constructions that involve A-movement and A'-movement:
//! - Passive voice (A-movement: NP movement to subject position)
//! - Wh-questions (A'-movement: wh-phrase to Spec-CP)
//! - Relativization (A'-movement: relative pronoun/operator movement)
//! - Topicalization (A'-movement: topic to Spec-CP)
//!
//! Following minimalist movement theory (Chomsky 1995, 2000) but keeping
//! implementation simple and practical using dependency parsing cues.

use crate::syntax::{VoiceDetector, VoiceType};
use canopy_core::{DepRel, UDVerbForm, UPos, Word};
use serde::{Deserialize, Serialize};
use tracing::{debug, trace};

/// Simple movement detector using dependency parsing signals
#[derive(Debug)]
pub struct MovementDetector {
    /// Voice detector for passive movement
    voice_detector: VoiceDetector,
}

/// Movement analysis result
#[derive(Debug, Clone)]
pub struct MovementAnalysis {
    /// Types of movement detected
    pub movement_types: Vec<MovementType>,

    /// Overall confidence (0.0-1.0)
    pub confidence: f32,

    /// Specific movement signals found
    pub signals: MovementSignals,
}

/// Types of movement constructions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MovementType {
    /// A-movement: NP-movement (passive, raising, etc.)
    PassiveMovement,

    /// A'-movement: Wh-movement
    WhMovement,

    /// A'-movement: Relative clause
    RelativeMovement,

    /// A'-movement: Topicalization
    TopicMovement,

    /// A-movement: Subject raising
    RaisingMovement,

    /// No movement detected
    None,
}

/// Specific signals detected
#[derive(Debug, Clone, Default)]
pub struct MovementSignals {
    /// Passive voice indicators
    pub passive_voice: bool,
    pub passive_subject: bool,
    pub by_phrase: bool,

    /// Wh-movement indicators
    pub wh_word: Option<String>,
    pub fronted_wh: bool,
    pub wh_trace_gap: bool,

    /// Relative clause indicators
    pub relative_pronoun: Option<String>,
    pub relative_complementizer: bool,

    /// Topicalization indicators
    pub fronted_object: bool,
    pub topic_comma: bool,

    /// Raising indicators
    pub seems_construction: bool,
    pub infinitival_complement: bool,
}

impl MovementDetector {
    /// Create new movement detector
    pub fn new() -> Self {
        Self {
            voice_detector: VoiceDetector::new(),
        }
    }

    /// Detect movement in sentence
    pub fn detect_movement(&self, words: &[Word]) -> MovementAnalysis {
        debug!("Detecting movement in sentence with {} words", words.len());

        let mut analysis = MovementAnalysis::default();

        // 1. Check for passive movement (A-movement)
        self.detect_passive_movement(words, &mut analysis);

        // 2. Check for wh-movement (A'-movement)
        self.detect_wh_movement(words, &mut analysis);

        // 3. Check for relative movement (A'-movement)
        self.detect_relative_movement(words, &mut analysis);

        // 4. Check for topicalization (A'-movement)
        self.detect_topicalization(words, &mut analysis);

        // 5. Check for raising movement (A-movement)
        self.detect_raising_movement(words, &mut analysis);

        // Calculate overall confidence
        analysis.confidence = self.calculate_confidence(&analysis);

        trace!("Movement analysis: {:?}", analysis.movement_types);
        analysis
    }

    /// Detect passive movement using voice detection
    fn detect_passive_movement(&self, words: &[Word], analysis: &mut MovementAnalysis) {
        // Find main verb
        if let Some(main_verb) = words
            .iter()
            .find(|w| w.deprel == DepRel::Root && w.upos == UPos::Verb)
        {
            let voice_analysis = self.voice_detector.detect_voice(words, &main_verb.lemma);

            if voice_analysis.voice_type == VoiceType::Passive {
                analysis.movement_types.push(MovementType::PassiveMovement);
                analysis.signals.passive_voice = true;
                analysis.signals.passive_subject = voice_analysis.has_passive_subject;
                analysis.signals.by_phrase = voice_analysis.has_by_phrase;

                trace!("Detected passive movement in verb: {}", main_verb.lemma);
            }
        }
    }

    /// Detect wh-movement (questions, embedded questions)
    fn detect_wh_movement(&self, words: &[Word], analysis: &mut MovementAnalysis) {
        let wh_words = [
            "what", "who", "whom", "which", "where", "when", "why", "how",
        ];

        for word in words {
            if wh_words.contains(&word.lemma.as_str()) {
                analysis.movement_types.push(MovementType::WhMovement);
                analysis.signals.wh_word = Some(word.lemma.clone());

                // Check if wh-word is fronted (near beginning of sentence)
                if word.id <= 3 {
                    analysis.signals.fronted_wh = true;
                }

                // Look for potential gap/trace site
                if self.has_wh_gap(words, word) {
                    analysis.signals.wh_trace_gap = true;
                }

                trace!("Detected wh-movement with word: {}", word.lemma);
                break; // Only count first wh-word
            }
        }
    }

    /// Detect relative clause movement
    fn detect_relative_movement(&self, words: &[Word], analysis: &mut MovementAnalysis) {
        let relative_pronouns = ["that", "which", "who", "whom", "whose", "where", "when"];

        for word in words {
            if relative_pronouns.contains(&word.lemma.as_str()) {
                // Check if this is actually a relative pronoun (not a complementizer)
                if self.is_relative_context(words, word) {
                    analysis.movement_types.push(MovementType::RelativeMovement);
                    analysis.signals.relative_pronoun = Some(word.lemma.clone());

                    if word.lemma == "that" {
                        analysis.signals.relative_complementizer = true;
                    }

                    trace!("Detected relative movement with pronoun: {}", word.lemma);
                }
            }
        }
    }

    /// Detect topicalization (fronted objects/adjuncts)
    fn detect_topicalization(&self, words: &[Word], analysis: &mut MovementAnalysis) {
        // Look for objects/adjuncts that appear before the subject
        if let Some(subject_pos) = self.find_subject_position(words) {
            for word in words {
                if word.id < subject_pos
                    && (word.deprel == DepRel::Obj || word.deprel == DepRel::Obl)
                {
                    analysis.movement_types.push(MovementType::TopicMovement);
                    analysis.signals.fronted_object = true;

                    // Check for comma after topic
                    if self.has_comma_after(words, word) {
                        analysis.signals.topic_comma = true;
                    }

                    trace!("Detected topicalization: {} before subject", word.lemma);
                    break;
                }
            }
        }
    }

    /// Detect raising constructions
    fn detect_raising_movement(&self, words: &[Word], analysis: &mut MovementAnalysis) {
        // Subject-to-subject raising verbs (seem-type)
        let subject_raising_verbs = [
            "seem", "appear", "happen", "tend", "prove", "turn", "come", "get", "begin", "start",
            "continue", "cease", "fail", "manage", "threaten",
        ];

        // Subject-to-object raising verbs (believe-type)
        let object_raising_verbs = [
            "believe", "consider", "find", "think", "want", "expect", "know", "assume", "suppose",
            "imagine", "declare", "report", "claim",
        ];

        for word in words {
            if word.upos == UPos::Verb {
                // Check for subject-to-subject raising
                if subject_raising_verbs.contains(&word.lemma.as_str()) {
                    if self.detect_subject_raising(words, word) {
                        analysis.movement_types.push(MovementType::RaisingMovement);
                        analysis.signals.seems_construction = true;
                        analysis.signals.infinitival_complement = true;

                        trace!(
                            "Detected subject-to-subject raising with verb: {}",
                            word.lemma
                        );
                    }
                }

                // Check for subject-to-object raising (ECM)
                if object_raising_verbs.contains(&word.lemma.as_str()) {
                    if self.detect_object_raising(words, word) {
                        analysis.movement_types.push(MovementType::RaisingMovement);
                        analysis.signals.seems_construction = false; // Different type
                        analysis.signals.infinitival_complement = true;

                        trace!(
                            "Detected subject-to-object raising (ECM) with verb: {}",
                            word.lemma
                        );
                    }
                }
            }
        }
    }

    /// Detect subject-to-subject raising (e.g., "John seems to be happy")
    fn detect_subject_raising(&self, words: &[Word], raising_verb: &Word) -> bool {
        // Must have subject
        let has_subject = words
            .iter()
            .any(|w| w.head == Some(raising_verb.id) && w.deprel == DepRel::Nsubj);

        if !has_subject {
            return false;
        }

        // Must have infinitival complement
        if !self.has_infinitival_complement(words, raising_verb) {
            return false;
        }

        // Additional check: the complement should be non-finite
        let has_nonfinite_complement = words.iter().any(|w| {
            w.head == Some(raising_verb.id)
                && (w.deprel == DepRel::Xcomp || w.deprel == DepRel::Ccomp)
                && w.feats.verbform.as_ref().map_or(false, |vf| {
                    matches!(vf, canopy_core::UDVerbForm::Infinitive)
                })
        });

        has_nonfinite_complement
    }

    /// Detect subject-to-object raising (ECM, e.g., "I believe John to be smart")
    fn detect_object_raising(&self, words: &[Word], raising_verb: &Word) -> bool {
        // Must have subject
        let has_subject = words
            .iter()
            .any(|w| w.head == Some(raising_verb.id) && w.deprel == DepRel::Nsubj);

        // Must have object (the raised element)
        let has_object = words
            .iter()
            .any(|w| w.head == Some(raising_verb.id) && w.deprel == DepRel::Obj);

        // Must have infinitival complement
        let has_infinitive = self.has_infinitival_complement(words, raising_verb);

        has_subject && has_object && has_infinitive
    }

    /// Check if wh-word has associated gap/trace
    fn has_wh_gap(&self, words: &[Word], wh_word: &Word) -> bool {
        // Simple heuristic: look for missing expected arguments
        // In a full implementation, this would be more sophisticated

        // If wh-word is "what" or "who", look for missing object
        match wh_word.lemma.as_str() {
            "what" | "who" | "whom" => {
                // Find main verb and check if it's missing an object
                if let Some(verb) = words.iter().find(|w| w.deprel == DepRel::Root) {
                    let has_object = words
                        .iter()
                        .any(|w| w.head == Some(verb.id) && w.deprel == DepRel::Obj);
                    return !has_object; // Gap if no object found
                }
            }
            "where" => {
                // Look for missing locative adjunct
                if let Some(verb) = words.iter().find(|w| w.deprel == DepRel::Root) {
                    let has_locative = words.iter().any(|w| {
                        w.head == Some(verb.id)
                            && w.deprel == DepRel::Obl
                            && words
                                .iter()
                                .any(|prep| prep.head == Some(w.id) && prep.upos == UPos::Adp)
                    });
                    return !has_locative;
                }
            }
            _ => {}
        }

        false
    }

    /// Check if word is in relative clause context
    fn is_relative_context(&self, words: &[Word], _word: &Word) -> bool {
        // Look for noun that this relative pronoun modifies
        for potential_head in words {
            if potential_head.upos == UPos::Noun {
                // Check if there's a clause that depends on this noun
                let has_relative_clause = words
                    .iter()
                    .any(|w| w.head == Some(potential_head.id) && w.deprel == DepRel::Acl);

                if has_relative_clause {
                    return true;
                }
            }
        }

        false
    }

    /// Find position of subject in sentence
    fn find_subject_position(&self, words: &[Word]) -> Option<usize> {
        for word in words {
            if word.deprel == DepRel::Nsubj || word.deprel == DepRel::NsubjPass {
                return Some(word.id);
            }
        }
        None
    }

    /// Check if word has comma immediately after it
    fn has_comma_after(&self, words: &[Word], word: &Word) -> bool {
        words
            .iter()
            .any(|w| w.id == word.id + 1 && w.lemma == "," && w.upos == UPos::Punct)
    }

    /// Check if verb has infinitival complement
    fn has_infinitival_complement(&self, words: &[Word], verb: &Word) -> bool {
        for word in words {
            if word.head == Some(verb.id) && word.deprel == DepRel::Xcomp {
                // Check if it's infinitival
                if let Some(verbform) = &word.feats.verbform {
                    if matches!(verbform, UDVerbForm::Infinitive) {
                        return true;
                    }
                }

                // Also check for "to" + infinitive
                for to_word in words {
                    if to_word.head == Some(word.id)
                        && to_word.lemma == "to"
                        && to_word.upos == UPos::Part
                    {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Calculate overall confidence based on signals
    fn calculate_confidence(&self, analysis: &MovementAnalysis) -> f32 {
        if analysis.movement_types.is_empty() {
            return 0.0;
        }

        let mut confidence = 0.0;

        // Passive movement signals
        if analysis.signals.passive_voice {
            confidence += 0.4;
            if analysis.signals.passive_subject {
                confidence += 0.2;
            }
            if analysis.signals.by_phrase {
                confidence += 0.2;
            }
        }

        // Wh-movement signals
        if analysis.signals.wh_word.is_some() {
            confidence += 0.3;
            if analysis.signals.fronted_wh {
                confidence += 0.2;
            }
            if analysis.signals.wh_trace_gap {
                confidence += 0.3;
            }
        }

        // Relative movement signals
        if analysis.signals.relative_pronoun.is_some() {
            confidence += 0.3;
            if analysis.signals.relative_complementizer {
                confidence += 0.1;
            }
        }

        // Topic movement signals
        if analysis.signals.fronted_object {
            confidence += 0.2;
            if analysis.signals.topic_comma {
                confidence += 0.1;
            }
        }

        // Raising signals
        if analysis.signals.seems_construction {
            confidence += 0.3;
            if analysis.signals.infinitival_complement {
                confidence += 0.2;
            }
        }

        // Normalize by number of movement types to avoid over-confidence
        confidence / analysis.movement_types.len() as f32
    }

    /// Get primary movement type (most confident)
    pub fn get_primary_movement(&self, words: &[Word]) -> MovementType {
        let analysis = self.detect_movement(words);

        if analysis.movement_types.is_empty() {
            return MovementType::None;
        }

        // Return first detected movement type
        // In a more sophisticated implementation, we'd rank by confidence
        analysis.movement_types[0]
    }

    /// Check if sentence has any movement
    pub fn has_movement(&self, words: &[Word]) -> bool {
        let analysis = self.detect_movement(words);
        !analysis.movement_types.is_empty()
    }
}

impl Default for MovementAnalysis {
    fn default() -> Self {
        Self {
            movement_types: Vec::new(),
            confidence: 0.0,
            signals: MovementSignals::default(),
        }
    }
}

impl Default for MovementDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for MovementType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MovementType::PassiveMovement => write!(f, "Passive"),
            MovementType::WhMovement => write!(f, "Wh-Movement"),
            MovementType::RelativeMovement => write!(f, "Relative"),
            MovementType::TopicMovement => write!(f, "Topic"),
            MovementType::RaisingMovement => write!(f, "Raising"),
            MovementType::None => write!(f, "None"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use canopy_core::MorphFeatures;

    fn create_test_word(
        id: usize,
        text: &str,
        lemma: &str,
        upos: UPos,
        head: usize,
        deprel: DepRel,
    ) -> Word {
        let mut word = Word {
            id,
            text: text.to_string(),
            lemma: lemma.to_string(),
            upos,
            xpos: None,
            feats: MorphFeatures::default(),
            head: Some(head),
            deprel,
            deps: None,
            misc: None,
            start: 0,
            end: text.len(),
        };

        // Add some realistic morphological features
        match lemma {
            "was" | "were" | "is" | "am" | "are" => {
                word.feats.verbform = Some(UDVerbForm::Finite);
            }
            _ if text.ends_with("ed") => {
                word.feats.verbform = Some(UDVerbForm::Participle);
            }
            _ => {}
        }

        word
    }

    #[test]
    fn test_passive_movement_detection() {
        let detector = MovementDetector::new();

        // "The ball was hit by John"
        let words = vec![
            create_test_word(1, "The", "the", UPos::Det, 2, DepRel::Det),
            create_test_word(2, "ball", "ball", UPos::Noun, 4, DepRel::NsubjPass),
            create_test_word(3, "was", "be", UPos::Aux, 4, DepRel::AuxPass),
            create_test_word(4, "hit", "hit", UPos::Verb, 0, DepRel::Root),
            create_test_word(5, "by", "by", UPos::Adp, 6, DepRel::Case),
            create_test_word(6, "John", "John", UPos::Propn, 4, DepRel::Obl),
        ];

        let analysis = detector.detect_movement(&words);

        assert!(
            analysis
                .movement_types
                .contains(&MovementType::PassiveMovement)
        );
        assert!(analysis.signals.passive_voice);
        assert!(analysis.confidence > 0.0);
    }

    #[test]
    fn test_wh_movement_detection() {
        let detector = MovementDetector::new();

        // "What did John see?"
        let words = vec![
            create_test_word(1, "What", "what", UPos::Pron, 4, DepRel::Obj),
            create_test_word(2, "did", "do", UPos::Aux, 4, DepRel::Aux),
            create_test_word(3, "John", "John", UPos::Propn, 4, DepRel::Nsubj),
            create_test_word(4, "see", "see", UPos::Verb, 0, DepRel::Root),
        ];

        let analysis = detector.detect_movement(&words);

        assert!(analysis.movement_types.contains(&MovementType::WhMovement));
        assert_eq!(analysis.signals.wh_word, Some("what".to_string()));
        assert!(analysis.signals.fronted_wh);
    }

    #[test]
    fn test_relative_movement_detection() {
        let detector = MovementDetector::new();

        // "The book that John read" (simplified)
        let words = vec![
            create_test_word(1, "The", "the", UPos::Det, 2, DepRel::Det),
            create_test_word(2, "book", "book", UPos::Noun, 0, DepRel::Root),
            create_test_word(3, "that", "that", UPos::Pron, 5, DepRel::Obj),
            create_test_word(4, "John", "John", UPos::Propn, 5, DepRel::Nsubj),
            create_test_word(5, "read", "read", UPos::Verb, 2, DepRel::Acl),
        ];

        let analysis = detector.detect_movement(&words);

        assert!(
            analysis
                .movement_types
                .contains(&MovementType::RelativeMovement)
        );
        assert_eq!(analysis.signals.relative_pronoun, Some("that".to_string()));
    }

    #[test]
    fn test_raising_movement_detection() {
        let detector = MovementDetector::new();

        // "John seems to be happy"
        let mut words = vec![
            create_test_word(1, "John", "John", UPos::Propn, 2, DepRel::Nsubj),
            create_test_word(2, "seems", "seem", UPos::Verb, 0, DepRel::Root),
            create_test_word(3, "to", "to", UPos::Part, 5, DepRel::Mark),
            create_test_word(4, "be", "be", UPos::Aux, 5, DepRel::Cop),
            create_test_word(5, "happy", "happy", UPos::Adj, 2, DepRel::Xcomp),
        ];

        // Add infinitive feature
        words[4].feats.verbform = Some(UDVerbForm::Infinitive);

        let analysis = detector.detect_movement(&words);

        assert!(
            analysis
                .movement_types
                .contains(&MovementType::RaisingMovement)
        );
        assert!(analysis.signals.seems_construction);
    }

    #[test]
    fn test_subject_to_object_raising_detection() {
        let detector = MovementDetector::new();

        // "I believe John to be smart" (ECM construction)
        let mut words = vec![
            create_test_word(1, "I", "I", UPos::Pron, 2, DepRel::Nsubj),
            create_test_word(2, "believe", "believe", UPos::Verb, 0, DepRel::Root),
            create_test_word(3, "John", "John", UPos::Propn, 2, DepRel::Obj),
            create_test_word(4, "to", "to", UPos::Part, 6, DepRel::Mark),
            create_test_word(5, "be", "be", UPos::Aux, 6, DepRel::Cop),
            create_test_word(6, "smart", "smart", UPos::Adj, 2, DepRel::Xcomp),
        ];

        // Add infinitive feature
        words[4].feats.verbform = Some(UDVerbForm::Infinitive);

        let analysis = detector.detect_movement(&words);

        assert!(
            analysis
                .movement_types
                .contains(&MovementType::RaisingMovement)
        );
        assert!(!analysis.signals.seems_construction); // ECM, not subject raising
        assert!(analysis.signals.infinitival_complement);
    }

    #[test]
    fn test_aspectual_raising_verbs() {
        let detector = MovementDetector::new();

        // "John began to run"
        let mut words = vec![
            create_test_word(1, "John", "John", UPos::Propn, 2, DepRel::Nsubj),
            create_test_word(2, "began", "begin", UPos::Verb, 0, DepRel::Root),
            create_test_word(3, "to", "to", UPos::Part, 4, DepRel::Mark),
            create_test_word(4, "run", "run", UPos::Verb, 2, DepRel::Xcomp),
        ];

        // Add infinitive feature
        words[3].feats.verbform = Some(UDVerbForm::Infinitive);

        let analysis = detector.detect_movement(&words);

        assert!(
            analysis
                .movement_types
                .contains(&MovementType::RaisingMovement)
        );
        assert!(analysis.signals.seems_construction); // Subject-to-subject type
    }

    #[test]
    fn test_no_raising_with_control_verb() {
        let detector = MovementDetector::new();

        // "John tried to run" (control, not raising)
        let mut words = vec![
            create_test_word(1, "John", "John", UPos::Propn, 2, DepRel::Nsubj),
            create_test_word(2, "tried", "try", UPos::Verb, 0, DepRel::Root),
            create_test_word(3, "to", "to", UPos::Part, 4, DepRel::Mark),
            create_test_word(4, "run", "run", UPos::Verb, 2, DepRel::Xcomp),
        ];

        // Add infinitive feature
        words[3].feats.verbform = Some(UDVerbForm::Infinitive);

        let analysis = detector.detect_movement(&words);

        // Should NOT detect raising for control verbs
        assert!(
            !analysis
                .movement_types
                .contains(&MovementType::RaisingMovement)
        );
    }

    #[test]
    fn test_no_movement() {
        let detector = MovementDetector::new();

        // "John likes coffee" - simple active sentence
        let words = vec![
            create_test_word(1, "John", "John", UPos::Propn, 2, DepRel::Nsubj),
            create_test_word(2, "likes", "like", UPos::Verb, 0, DepRel::Root),
            create_test_word(3, "coffee", "coffee", UPos::Noun, 2, DepRel::Obj),
        ];

        let analysis = detector.detect_movement(&words);

        assert!(analysis.movement_types.is_empty());
        assert_eq!(analysis.confidence, 0.0);
    }

    #[test]
    fn test_multiple_movements() {
        let detector = MovementDetector::new();

        // "What was John given?" - both wh-movement and passive
        let words = vec![
            create_test_word(1, "What", "what", UPos::Pron, 4, DepRel::Obj),
            create_test_word(2, "was", "be", UPos::Aux, 4, DepRel::AuxPass),
            create_test_word(3, "John", "John", UPos::Propn, 4, DepRel::NsubjPass),
            create_test_word(4, "given", "give", UPos::Verb, 0, DepRel::Root),
        ];

        let analysis = detector.detect_movement(&words);

        assert!(analysis.movement_types.contains(&MovementType::WhMovement));
        assert!(
            analysis
                .movement_types
                .contains(&MovementType::PassiveMovement)
        );
        assert!(analysis.confidence > 0.0);
    }
}
