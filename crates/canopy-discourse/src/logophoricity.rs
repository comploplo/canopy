//! Logophoric context detection for exempt anaphora
//!
//! Based on Charnavel (2019) "Locality and Logophoricity: A Theory of Exempt Anaphora"
//! and Sells (1987) "Aspects of Logophoricity".
//!
//! Key insight: Some anaphors that appear to violate locality constraints are
//! actually **exempt** anaphors bound by a silent logophoric operator.
//!
//! Examples of exempt anaphora:
//! - "Pictures of himself₁ upset John₁" (empathy locus)
//! - "John₁ thinks that Mary likes himself₁" (attitude holder)
//!
//! Logophoric contexts license non-local binding by providing an implicit
//! perspective center that acts as a local binder.

use crate::referent::ReferentId;
use std::collections::HashSet;

/// Attitude verbs that introduce perspective shift (Sells 1987, Charnavel 2019)
///
/// These verbs introduce a logophoric context where the subject is
/// the perspective center (SOURCE, SELF, or PIVOT).
const ATTITUDE_VERBS: &[&str] = &[
    // Cognitive attitudes (SELF - internal mental state)
    "think",
    "believe",
    "know",
    "realize",
    "understand",
    "recognize",
    "remember",
    "forget",
    "doubt",
    "suspect",
    "imagine",
    "dream",
    "assume",
    "suppose",
    "consider",
    // Desiderative attitudes
    "want",
    "wish",
    "hope",
    "desire",
    "prefer",
    "expect",
    "intend",
    "plan",
    "decide",
    // Communication verbs (SOURCE - speaker of reported speech)
    "say",
    "tell",
    "claim",
    "argue",
    "assert",
    "state",
    "report",
    "announce",
    "declare",
    "explain",
    "suggest",
    "propose",
    "promise",
    "warn",
    // Emotional attitudes (often overlaps with SELF)
    "fear",
    "worry",
    "regret",
    "resent",
];

/// Experiencer predicates with non-agentive subjects
///
/// These predicates license exempt anaphora in subject position
/// because the object is the empathy locus (perspective center).
///
/// "Stories about himself₁ bother John₁" → John is empathy locus
const EXPERIENCER_PREDICATES: &[&str] = &[
    // Psychological predicates (cause of emotion)
    "bother",
    "upset",
    "disturb",
    "trouble",
    "worry",
    "concern",
    // Positive affect
    "please",
    "delight",
    "amuse",
    "entertain",
    "satisfy",
    "gratify",
    // Negative affect
    "annoy",
    "irritate",
    "anger",
    "infuriate",
    "frustrate",
    "disappoint",
    // Fear/surprise
    "frighten",
    "scare",
    "terrify",
    "alarm",
    "startle",
    "surprise",
    "shock",
    "astonish",
    "amaze",
    // Interest/attention
    "interest",
    "intrigue",
    "fascinate",
    "bore",
    "tire",
];

/// Types of logophoric contexts (after Sells 1987)
#[derive(Debug, Clone, PartialEq)]
pub enum LogophoricContext {
    /// Attitude holder context: subject of attitude verb is perspective center
    ///
    /// "John₁ thinks [that Mary likes himself₁]"
    /// John is the attitude holder whose perspective licenses "himself"
    AttitudeHolder {
        /// The referent who holds the attitude (perspective center)
        holder: ReferentId,
        /// The attitude verb
        verb: String,
    },

    /// Empathy locus context: object of experiencer predicate is perspective center
    ///
    /// "Pictures of himself₁ upset John₁"
    /// John is the experiencer whose perspective licenses "himself"
    EmpathyLocus {
        /// The referent who experiences the emotion (perspective center)
        experiencer: ReferentId,
        /// The experiencer predicate
        predicate: String,
    },

    /// Picture noun phrase context: provides syntactic context for exempt reading
    ///
    /// "Pictures of himself₁ are on the wall" (with contextual antecedent)
    PictureNounContext {
        /// Description of the picture NP
        description: String,
    },

    /// No logophoric context detected (plain binding required)
    None,
}

impl LogophoricContext {
    /// Check if this context licenses exempt anaphora
    #[must_use]
    pub fn is_logophoric(&self) -> bool {
        !matches!(self, LogophoricContext::None)
    }

    /// Get the perspective center (if any)
    #[must_use]
    pub fn perspective_center(&self) -> Option<ReferentId> {
        match self {
            LogophoricContext::AttitudeHolder { holder, .. } => Some(*holder),
            LogophoricContext::EmpathyLocus { experiencer, .. } => Some(*experiencer),
            LogophoricContext::PictureNounContext { .. } => None,
            LogophoricContext::None => None,
        }
    }
}

/// Detector for logophoric contexts
#[derive(Debug, Clone)]
pub struct LogophoricDetector {
    /// Set of attitude verbs
    attitude_verbs: HashSet<String>,
    /// Set of experiencer predicates
    experiencer_predicates: HashSet<String>,
}

impl LogophoricDetector {
    /// Create a new logophoric detector
    #[must_use]
    pub fn new() -> Self {
        Self {
            attitude_verbs: ATTITUDE_VERBS.iter().map(|s| s.to_string()).collect(),
            experiencer_predicates: EXPERIENCER_PREDICATES
                .iter()
                .map(|s| s.to_string())
                .collect(),
        }
    }

    /// Check if a verb is an attitude verb
    #[must_use]
    pub fn is_attitude_verb(&self, verb_lemma: &str) -> bool {
        self.attitude_verbs.contains(&verb_lemma.to_lowercase())
    }

    /// Check if a predicate is an experiencer predicate
    #[must_use]
    pub fn is_experiencer_predicate(&self, predicate_lemma: &str) -> bool {
        self.experiencer_predicates
            .contains(&predicate_lemma.to_lowercase())
    }

    /// Detect logophoric context from predicate and arguments
    ///
    /// # Arguments
    /// * `predicate` - The main predicate (verb lemma)
    /// * `subject_id` - Referent ID of the subject (if any)
    /// * `object_id` - Referent ID of the object (if any)
    /// * `is_embedded` - Whether this is an embedded clause
    /// * `matrix_subject` - Subject of the matrix clause (if embedded)
    #[must_use]
    pub fn detect(
        &self,
        predicate: &str,
        subject_id: Option<ReferentId>,
        object_id: Option<ReferentId>,
        is_embedded: bool,
        matrix_subject: Option<ReferentId>,
    ) -> LogophoricContext {
        let predicate_lower = predicate.to_lowercase();

        // Check for attitude holder context (embedded under attitude verb)
        if is_embedded {
            if let Some(holder) = matrix_subject {
                // The matrix subject is the attitude holder/perspective center
                return LogophoricContext::AttitudeHolder {
                    holder,
                    verb: predicate_lower.clone(),
                };
            }
        }

        // Check for experiencer predicate context
        if self.is_experiencer_predicate(&predicate_lower) {
            if let Some(experiencer) = object_id {
                // Object of experiencer predicate is the empathy locus
                return LogophoricContext::EmpathyLocus {
                    experiencer,
                    predicate: predicate_lower,
                };
            }
        }

        // Check if current predicate introduces attitude context for embedded clause
        if self.is_attitude_verb(&predicate_lower) {
            if let Some(holder) = subject_id {
                // Subject of attitude verb is potential holder for embedded content
                return LogophoricContext::AttitudeHolder {
                    holder,
                    verb: predicate_lower,
                };
            }
        }

        LogophoricContext::None
    }

    /// Check if a referent can be a logophoric antecedent
    ///
    /// In logophoric contexts, the perspective center can bind anaphors
    /// that would otherwise be non-local.
    #[must_use]
    pub fn can_bind_logophorically(
        &self,
        context: &LogophoricContext,
        candidate_id: ReferentId,
    ) -> bool {
        match context {
            LogophoricContext::AttitudeHolder { holder, .. } => *holder == candidate_id,
            LogophoricContext::EmpathyLocus { experiencer, .. } => *experiencer == candidate_id,
            LogophoricContext::PictureNounContext { .. } => {
                // Picture NP contexts allow any salient antecedent
                true
            }
            LogophoricContext::None => false,
        }
    }
}

impl Default for LogophoricDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if a noun phrase head suggests a picture noun context
///
/// Picture nouns license exempt anaphora in their complement:
/// "pictures of himself", "stories about herself", "rumors about themselves"
#[must_use]
pub fn is_picture_noun(noun: &str) -> bool {
    let noun_lower = noun.to_lowercase();
    matches!(
        noun_lower.as_str(),
        "picture"
            | "pictures"
            | "photo"
            | "photos"
            | "photograph"
            | "photographs"
            | "story"
            | "stories"
            | "rumor"
            | "rumors"
            | "rumour"
            | "rumours"
            | "report"
            | "reports"
            | "description"
            | "descriptions"
            | "account"
            | "accounts"
            | "book"
            | "books"
            | "article"
            | "articles"
            | "essay"
            | "essays"
            | "portrait"
            | "portraits"
            | "image"
            | "images"
            | "video"
            | "videos"
            | "film"
            | "films"
            | "movie"
            | "movies"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attitude_verbs() {
        let detector = LogophoricDetector::new();

        assert!(detector.is_attitude_verb("think"));
        assert!(detector.is_attitude_verb("believe"));
        assert!(detector.is_attitude_verb("say"));
        assert!(detector.is_attitude_verb("fear"));
        assert!(!detector.is_attitude_verb("run"));
        assert!(!detector.is_attitude_verb("hit"));
    }

    #[test]
    fn test_experiencer_predicates() {
        let detector = LogophoricDetector::new();

        assert!(detector.is_experiencer_predicate("bother"));
        assert!(detector.is_experiencer_predicate("upset"));
        assert!(detector.is_experiencer_predicate("frighten"));
        assert!(!detector.is_experiencer_predicate("criticize"));
        assert!(!detector.is_experiencer_predicate("eat"));
    }

    #[test]
    fn test_detect_attitude_context() {
        let detector = LogophoricDetector::new();
        let john = ReferentId(1);

        // Embedded clause under attitude verb
        let ctx = detector.detect("like", None, None, true, Some(john));

        assert!(matches!(
            ctx,
            LogophoricContext::AttitudeHolder { holder, .. } if holder == john
        ));
    }

    #[test]
    fn test_detect_experiencer_context() {
        let detector = LogophoricDetector::new();
        let john = ReferentId(1);

        // "Pictures of himself bother John"
        let ctx = detector.detect("bother", None, Some(john), false, None);

        assert!(matches!(
            ctx,
            LogophoricContext::EmpathyLocus { experiencer, .. } if experiencer == john
        ));
    }

    #[test]
    fn test_no_logophoric_context() {
        let detector = LogophoricDetector::new();

        let ctx = detector.detect("run", Some(ReferentId(1)), None, false, None);
        assert!(matches!(ctx, LogophoricContext::None));
    }

    #[test]
    fn test_can_bind_logophorically() {
        let detector = LogophoricDetector::new();
        let john = ReferentId(1);
        let mary = ReferentId(2);

        let ctx = LogophoricContext::AttitudeHolder {
            holder: john,
            verb: "think".to_string(),
        };

        assert!(detector.can_bind_logophorically(&ctx, john));
        assert!(!detector.can_bind_logophorically(&ctx, mary));
    }

    #[test]
    fn test_picture_nouns() {
        assert!(is_picture_noun("picture"));
        assert!(is_picture_noun("stories"));
        assert!(is_picture_noun("rumor"));
        assert!(!is_picture_noun("table"));
        assert!(!is_picture_noun("dog"));
    }
}
