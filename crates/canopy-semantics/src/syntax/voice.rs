//! Simple voice detection using UDPipe morphological features
//!
//! This module detects voice alternations (active/passive/middle) using:
//! - UDPipe Voice morphological feature
//! - Dependency relations (nsubj vs nsubj:pass)
//! - Simple heuristics for English patterns
//!
//! Following Kratzer (1996) "Severing the external argument from its verb"
//! but keeping implementation simple and practical.

use crate::events::Event;
use canopy_core::{DepRel, UPos, Word};
use serde::{Deserialize, Serialize};
use tracing::{debug, trace};

/// Simple voice detector using morphological cues
#[derive(Debug)]
pub struct VoiceDetector;

impl VoiceDetector {
    /// Create new voice detector
    pub fn new() -> Self {
        Self
    }

    /// Detect voice from sentence words
    pub fn detect_voice(&self, words: &[Word], verb_lemma: &str) -> VoiceAnalysis {
        debug!("Detecting voice for verb: {}", verb_lemma);

        // Find main verb
        let verb_word = words
            .iter()
            .find(|w| w.lemma == verb_lemma && w.upos == UPos::Verb);

        if let Some(verb) = verb_word {
            self.analyze_voice_features(words, verb)
        } else {
            VoiceAnalysis::default()
        }
    }

    /// Analyze voice features from verb and dependents
    fn analyze_voice_features(&self, words: &[Word], verb: &Word) -> VoiceAnalysis {
        let mut analysis = VoiceAnalysis::default();

        // 1. Check UDPipe Voice feature
        if let Some(voice_feature) = &verb.feats.voice {
            match voice_feature {
                canopy_core::UDVoice::Active => analysis.voice_type = VoiceType::Active,
                canopy_core::UDVoice::Passive => analysis.voice_type = VoiceType::Passive,
                canopy_core::UDVoice::Middle => analysis.voice_type = VoiceType::Middle,
            }
            analysis.confidence += 0.4; // High confidence from morphology
            trace!(
                "UDPipe Voice feature: {:?} -> {:?}",
                voice_feature, analysis.voice_type
            );
        }

        // 2. Check dependency relations for subjects
        let subject_signals = self.analyze_subject_relations(words, verb);
        match subject_signals {
            SubjectPattern::ActiveSubject => {
                if analysis.voice_type == VoiceType::Unknown {
                    analysis.voice_type = VoiceType::Active;
                }
                analysis.confidence += 0.3;
            }
            SubjectPattern::PassiveSubject => {
                analysis.voice_type = VoiceType::Passive;
                analysis.confidence += 0.4;
                analysis.has_passive_subject = true;
            }
            SubjectPattern::NoSubject => {
                // Could be imperative or impersonal
                analysis.confidence += 0.1;
            }
        }

        // 3. Check for passive auxiliaries (be, get)
        if self.has_passive_auxiliary(words, verb) {
            analysis.voice_type = VoiceType::Passive;
            analysis.has_passive_auxiliary = true;
            analysis.confidence += 0.3;
        }

        // 4. Check for by-phrase (agent demoted to oblique)
        if self.has_by_phrase(words, verb) {
            analysis.voice_type = VoiceType::Passive;
            analysis.has_by_phrase = true;
            analysis.confidence += 0.2;
        }

        // 5. Check for middle voice (before reflexive, as it's more specific)
        if self.is_middle_voice(words, verb, &analysis) {
            analysis.voice_type = VoiceType::Middle;
            analysis.confidence += 0.4;
        }

        // 6. Check for reflexive/reciprocal markers
        if self.has_reflexive_marker(words, verb) {
            analysis.voice_type = VoiceType::Reflexive;
            analysis.has_reflexive_marker = true;
            analysis.confidence += 0.3;
        }

        // 7. Apply English-specific heuristics
        self.apply_english_heuristics(&mut analysis, words, verb);

        // Clamp confidence to [0.0, 1.0]
        analysis.confidence = analysis.confidence.min(1.0);

        trace!(
            "Voice analysis: {:?} (confidence: {:.2})",
            analysis.voice_type, analysis.confidence
        );
        analysis
    }

    /// Analyze subject dependency relations
    fn analyze_subject_relations(&self, words: &[Word], verb: &Word) -> SubjectPattern {
        for word in words {
            if word.head == Some(verb.id) {
                match word.deprel {
                    DepRel::Nsubj => return SubjectPattern::ActiveSubject,
                    DepRel::NsubjPass => return SubjectPattern::PassiveSubject,
                    DepRel::Csubj => return SubjectPattern::ActiveSubject, // Clausal subjects usually active
                    _ => {}
                }
            }
        }
        SubjectPattern::NoSubject
    }

    /// Check for passive auxiliaries (be + past participle, get + past participle)
    fn has_passive_auxiliary(&self, words: &[Word], verb: &Word) -> bool {
        // Look for auxiliary "be" or "get" that depends on or governs this verb
        for word in words {
            if (word.lemma == "be" || word.lemma == "get") && word.upos == UPos::Aux {
                // Check if this aux is related to our verb
                if word.head == Some(verb.id) || verb.head == Some(word.id) {
                    // Check if main verb is past participle
                    if let Some(verbform) = &verb.feats.verbform {
                        if matches!(verbform, canopy_core::UDVerbForm::Participle) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    /// Check for by-phrase indicating agent demotion
    fn has_by_phrase(&self, words: &[Word], verb: &Word) -> bool {
        for word in words {
            if word.head == Some(verb.id) && word.deprel == DepRel::Obl {
                // Check if this oblique is introduced by "by"
                for prep in words {
                    if prep.head == Some(word.id) && prep.lemma == "by" && prep.upos == UPos::Adp {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Check for reflexive markers (pronouns, "each other", etc.)
    fn has_reflexive_marker(&self, words: &[Word], verb: &Word) -> bool {
        for word in words {
            if word.head == Some(verb.id) {
                // Check for reflexive pronouns
                if word.upos == UPos::Pron {
                    // Simple heuristic: check if it's a reflexive form
                    if word.lemma.ends_with("self") || word.lemma.ends_with("selves") {
                        return true;
                    }
                }

                // Check for "each other"
                if word.lemma == "each" {
                    for other_word in words {
                        if other_word.head == Some(word.id) && other_word.lemma == "other" {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    /// Check for middle voice construction
    fn is_middle_voice(&self, words: &[Word], verb: &Word, analysis: &VoiceAnalysis) -> bool {
        // Middle voice criteria:
        // 1. No passive morphology (no aux:pass, no nsubj:pass)
        // 2. Subject is nsubj (not nsubj:pass)
        // 3. Verb is typically change-of-state
        // 4. Subject is semantically theme/patient, not agent

        // Check for passive markers first - if present, not middle
        if analysis.has_passive_auxiliary || analysis.has_passive_subject {
            return false;
        }

        // Must have regular subject (not passive)
        let has_active_subject = words
            .iter()
            .any(|word| word.head == Some(verb.id) && word.deprel == DepRel::Nsubj);

        if !has_active_subject {
            return false;
        }

        // Check for middle voice verbs (change-of-state, inchoative)
        let is_middle_verb = matches!(
            verb.lemma.as_str(),
            "open"
                | "close"
                | "break"
                | "melt"
                | "freeze"
                | "dissolve"
                | "move"
                | "turn"
                | "bend"
                | "fold"
                | "split"
                | "crack"
                | "start"
                | "begin"
                | "stop"
                | "end"
                | "change"
                | "improve"
                | "worsen"
                | "increase"
                | "decrease"
        );

        if !is_middle_verb {
            return false;
        }

        // Check if subject is inanimate (typical for middle voice)

        words
            .iter()
            .find(|word| word.head == Some(verb.id) && word.deprel == DepRel::Nsubj)
            .map(|subj| {
                // Simple heuristic: if it's not a proper noun or human pronoun, likely inanimate
                !matches!(subj.upos, UPos::Propn)
                    && !matches!(
                        subj.lemma.as_str(),
                        "I" | "you" | "he" | "she" | "we" | "they"
                    )
            })
            .unwrap_or(false)
    }

    /// Apply English-specific voice heuristics
    fn apply_english_heuristics(&self, analysis: &mut VoiceAnalysis, words: &[Word], verb: &Word) {
        // Check for middle voice patterns (subject = theme, no agent)
        if analysis.voice_type == VoiceType::Unknown {
            // Look for typical middle voice verbs
            match verb.lemma.as_str() {
                "break" | "open" | "close" | "move" | "turn" | "bend" | "fold" => {
                    // Check if there's no clear agent
                    let has_agent_subject = self.subject_is_likely_agent(words, verb);
                    if !has_agent_subject {
                        analysis.voice_type = VoiceType::Middle;
                        analysis.confidence += 0.2;
                    }
                }
                _ => {}
            }
        }

        // Default to active if no other evidence
        if analysis.voice_type == VoiceType::Unknown && analysis.confidence < 0.2 {
            analysis.voice_type = VoiceType::Active;
            analysis.confidence = 0.1; // Low confidence default
        }
    }

    /// Check if subject is likely an agent (for middle voice detection)
    fn subject_is_likely_agent(&self, words: &[Word], verb: &Word) -> bool {
        for word in words {
            if word.head == Some(verb.id) && word.deprel == DepRel::Nsubj {
                // Heuristic: proper nouns and human pronouns are likely agents
                match word.upos {
                    UPos::Propn => return true, // Proper nouns often agents
                    UPos::Pron => {
                        match word.lemma.as_str() {
                            "I" | "you" | "he" | "she" | "we" | "they" => return true,
                            "it" => return false, // "it" rarely agent
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }
        false
    }

    /// Update event with voice information
    pub fn annotate_event(&self, event: &mut Event, words: &[Word]) {
        let analysis = self.detect_voice(words, &event.predicate.lemma);

        // Update event predicate type based on voice
        match analysis.voice_type {
            VoiceType::Passive => {
                if !matches!(
                    event.predicate.semantic_type,
                    crate::events::PredicateType::Causative
                ) {
                    // Mark as passive variant
                    // In a full implementation, we'd have a PassiveOf wrapper
                }
            }
            VoiceType::Middle => {
                event.predicate.semantic_type = crate::events::PredicateType::Inchoative;
            }
            _ => {} // Keep existing type
        }

        // Store voice analysis in event metadata (if we had that)
        // For now, we could add it as a modifier or feature
    }
}

/// Voice analysis result
#[derive(Debug, Clone)]
pub struct VoiceAnalysis {
    /// Detected voice type
    pub voice_type: VoiceType,

    /// Confidence score (0.0-1.0)
    pub confidence: f32,

    /// Specific features detected
    pub has_passive_subject: bool,
    pub has_passive_auxiliary: bool,
    pub has_by_phrase: bool,
    pub has_reflexive_marker: bool,
}

impl Default for VoiceAnalysis {
    fn default() -> Self {
        Self {
            voice_type: VoiceType::Unknown,
            confidence: 0.0,
            has_passive_subject: false,
            has_passive_auxiliary: false,
            has_by_phrase: false,
            has_reflexive_marker: false,
        }
    }
}

/// Voice types following Kratzer (1996) and traditional grammar
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoiceType {
    /// Active voice: agent in subject position
    Active,

    /// Passive voice: patient promoted to subject, agent demoted/suppressed
    Passive,

    /// Middle voice: agent suppressed, no passive morphology
    Middle,

    /// Reflexive voice: agent and patient co-refer
    Reflexive,

    /// Reciprocal voice: agents act on each other
    Reciprocal,

    /// Unknown/indeterminate
    Unknown,
}

/// Subject pattern analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SubjectPattern {
    /// Normal subject (nsubj)
    ActiveSubject,

    /// Passive subject (nsubj:pass)
    PassiveSubject,

    /// No subject found
    NoSubject,
}

impl Default for VoiceDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for VoiceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VoiceType::Active => write!(f, "Active"),
            VoiceType::Passive => write!(f, "Passive"),
            VoiceType::Middle => write!(f, "Middle"),
            VoiceType::Reflexive => write!(f, "Reflexive"),
            VoiceType::Reciprocal => write!(f, "Reciprocal"),
            VoiceType::Unknown => write!(f, "Unknown"),
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
                word.feats.verbform = Some(canopy_core::UDVerbForm::Finite);
            }
            _ if text.ends_with("ed") => {
                word.feats.verbform = Some(canopy_core::UDVerbForm::Participle);
            }
            _ => {}
        }

        word
    }

    #[test]
    fn test_active_voice_detection() {
        let detector = VoiceDetector::new();

        // "John hits the ball"
        let words = vec![
            create_test_word(1, "John", "John", UPos::Propn, 2, DepRel::Nsubj),
            create_test_word(2, "hits", "hit", UPos::Verb, 0, DepRel::Root),
            create_test_word(3, "the", "the", UPos::Det, 4, DepRel::Det),
            create_test_word(4, "ball", "ball", UPos::Noun, 2, DepRel::Obj),
        ];

        let analysis = detector.detect_voice(&words, "hit");

        assert_eq!(analysis.voice_type, VoiceType::Active);
        assert!(analysis.confidence > 0.0);
        assert!(analysis.has_passive_subject == false);
    }

    #[test]
    fn test_passive_voice_detection() {
        let detector = VoiceDetector::new();

        // "The ball was hit by John"
        let words = vec![
            create_test_word(1, "The", "the", UPos::Det, 2, DepRel::Det),
            create_test_word(2, "ball", "ball", UPos::Noun, 4, DepRel::NsubjPass),
            create_test_word(3, "was", "be", UPos::Aux, 4, DepRel::AuxPass),
            create_test_word(4, "hit", "hit", UPos::Verb, 0, DepRel::Root),
            create_test_word(5, "by", "by", UPos::Adp, 6, DepRel::Case),
            create_test_word(6, "John", "John", UPos::Propn, 4, DepRel::Obl),
        ];

        let analysis = detector.detect_voice(&words, "hit");

        assert_eq!(analysis.voice_type, VoiceType::Passive);
        assert!(analysis.confidence > 0.5);
        assert!(analysis.has_passive_subject);
        assert!(analysis.has_by_phrase);
    }

    #[test]
    fn test_reflexive_voice_detection() {
        let detector = VoiceDetector::new();

        // "John hurt himself"
        let words = vec![
            create_test_word(1, "John", "John", UPos::Propn, 2, DepRel::Nsubj),
            create_test_word(2, "hurt", "hurt", UPos::Verb, 0, DepRel::Root),
            create_test_word(3, "himself", "himself", UPos::Pron, 2, DepRel::Obj),
        ];

        let analysis = detector.detect_voice(&words, "hurt");

        assert_eq!(analysis.voice_type, VoiceType::Reflexive);
        assert!(analysis.has_reflexive_marker);
    }

    #[test]
    fn test_middle_voice_detection() {
        let detector = VoiceDetector::new();

        // "The door opened" (middle voice - no agent)
        let words = vec![
            create_test_word(1, "The", "the", UPos::Det, 2, DepRel::Det),
            create_test_word(2, "door", "door", UPos::Noun, 3, DepRel::Nsubj),
            create_test_word(3, "opened", "open", UPos::Verb, 0, DepRel::Root),
        ];

        let analysis = detector.detect_voice(&words, "open");

        // Should detect middle voice (subject is theme, not agent)
        assert_eq!(analysis.voice_type, VoiceType::Middle);
    }

    #[test]
    fn test_voice_with_morphological_features() {
        let detector = VoiceDetector::new();

        let mut words = vec![
            create_test_word(1, "John", "John", UPos::Propn, 2, DepRel::Nsubj),
            create_test_word(2, "was", "be", UPos::Aux, 3, DepRel::AuxPass),
            create_test_word(3, "hit", "hit", UPos::Verb, 0, DepRel::Root),
        ];

        // Add Voice=Pass feature
        words[2].feats.voice = Some(canopy_core::UDVoice::Passive);

        let analysis = detector.detect_voice(&words, "hit");

        assert_eq!(analysis.voice_type, VoiceType::Passive);
        assert!(analysis.confidence > 0.4); // Should get boost from morphological feature
    }

    #[test]
    fn test_unknown_verb() {
        let detector = VoiceDetector::new();

        let words = vec![
            create_test_word(1, "Something", "something", UPos::Pron, 2, DepRel::Nsubj),
            create_test_word(2, "unknown", "unknown", UPos::Verb, 0, DepRel::Root),
        ];

        let analysis = detector.detect_voice(&words, "nonexistent");

        // Should still provide some analysis
        assert!(analysis.confidence >= 0.0);
    }
}
