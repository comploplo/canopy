//! Tense-Aspect-Modality (TAM) inference for event composition.
//!
//! Infers temporal frames and aspectual viewpoints from morphological features.
//! Uses Reichenbachian analysis (Speech time, Reference time, Event time).
//!
//! ## Tense-Frame Mapping
//!
//! | Tense           | Frame           | Example              |
//! |-----------------|-----------------|----------------------|
//! | Past            | E < R = S       | "John left"          |
//! | Present         | E = R = S       | "John leaves"        |
//! | Future          | S = R < E       | "John will leave"    |
//! | Past Perfect    | E < R < S       | "John had left"      |
//! | Present Perfect | E < R = S       | "John has left"      |
//! | Past Progressive| E ○ R < S       | "John was leaving"   |

use crate::core::{MorphFeatures, Tense, VerbForm};
use crate::kernel::discourse::{AspectualViewpoint, TemporalFrame};

/// Markers indicating aspectual constructions.
#[derive(Debug, Clone, Default)]
pub struct AspectMarkers {
    /// Whether the construction has a perfect auxiliary ("have" + participle).
    pub has_perfect: bool,
    /// Whether the construction has a progressive auxiliary ("be" + participle).
    pub has_progressive: bool,
    /// Whether this is a habitual construction.
    pub is_habitual: bool,
}

impl AspectMarkers {
    /// Create from auxiliary analysis.
    #[must_use]
    pub fn new(has_perfect: bool, has_progressive: bool) -> Self {
        Self {
            has_perfect,
            has_progressive,
            is_habitual: false,
        }
    }

    /// Check if any aspect marker is set.
    #[must_use]
    pub fn has_any(&self) -> bool {
        self.has_perfect || self.has_progressive || self.is_habitual
    }
}

/// Infer temporal frame from morphological features and aspect markers.
///
/// Combines tense (from `MorphFeatures`) with aspectual information
/// to produce the appropriate Reichenbachian frame.
#[must_use]
pub fn infer_temporal_frame(
    morph: Option<&MorphFeatures>,
    aspect_markers: &AspectMarkers,
) -> TemporalFrame {
    let tense = morph.and_then(|m| m.tense);

    match (
        tense,
        aspect_markers.has_perfect,
        aspect_markers.has_progressive,
    ) {
        // Past tenses
        (Some(Tense::Past), false, false) => TemporalFrame::past(),
        (Some(Tense::Past), true, false) => TemporalFrame::past_perfect(),
        (Some(Tense::Past), false, true) => TemporalFrame::past_progressive(),
        (Some(Tense::Past), true, true) => {
            // Past perfect progressive: "had been running"
            // E ○ R < S (ongoing event before past reference)
            TemporalFrame::past_progressive() // Simplified
        }

        // Present tenses
        (Some(Tense::Present), false, false) => TemporalFrame::present(),
        (Some(Tense::Present), true, false) => TemporalFrame::present_perfect(),
        (Some(Tense::Present), false, true) => TemporalFrame::present_progressive(),
        (Some(Tense::Present), true, true) => {
            // Present perfect progressive: "has been running"
            TemporalFrame::present_progressive() // Simplified
        }

        // Future tenses
        (Some(Tense::Future), false, false) => TemporalFrame::future(),
        (Some(Tense::Future), true, false) => TemporalFrame::future_perfect(),
        (Some(Tense::Future), false, true) => TemporalFrame::future_progressive(),
        (Some(Tense::Future), true, true) => {
            // Future perfect progressive: "will have been running"
            TemporalFrame::future_progressive() // Simplified
        }

        // No tense information - return underspecified
        (None, _, _) => TemporalFrame::underspecified(),
    }
}

/// Infer aspectual viewpoint from morphological features and markers.
#[must_use]
pub fn infer_aspectual_viewpoint(
    morph: Option<&MorphFeatures>,
    aspect_markers: &AspectMarkers,
) -> AspectualViewpoint {
    // Progressive takes precedence
    if aspect_markers.has_progressive {
        return AspectualViewpoint::Progressive;
    }

    // Perfect
    if aspect_markers.has_perfect {
        return AspectualViewpoint::Perfect;
    }

    // Habitual (would need more context to detect)
    if aspect_markers.is_habitual {
        return AspectualViewpoint::Habitual;
    }

    // Check verb form for non-finite
    if let Some(form) = morph.and_then(|m| m.verb_form) {
        match form {
            VerbForm::Participle => {
                // Bare participle without auxiliary - could be adjectival
                return AspectualViewpoint::Perfective;
            }
            VerbForm::Gerund => {
                // Gerund - ongoing/imperfective
                return AspectualViewpoint::Imperfective;
            }
            _ => {}
        }
    }

    // Default to perfective (completed action)
    AspectualViewpoint::Perfective
}

/// Detect aspect markers from auxiliary verbs in the sentence.
///
/// This is a simplified heuristic. In practice, you'd want to use
/// dependency parsing to find auxiliary relationships.
#[must_use]
pub fn detect_aspect_markers_from_lemmas(lemmas: &[&str]) -> AspectMarkers {
    let mut markers = AspectMarkers::default();

    // Check for perfect auxiliary "have" (or its forms)
    let have_forms = ["have", "has", "had", "'ve", "'d"];
    markers.has_perfect = lemmas
        .iter()
        .any(|&l| have_forms.contains(&l.to_lowercase().as_str()));

    // Check for progressive auxiliary "be" followed by -ing
    // This is simplified - proper detection would check for aux relation
    let be_forms = ["be", "is", "am", "are", "was", "were", "been", "being"];
    let has_be = lemmas
        .iter()
        .any(|&l| be_forms.contains(&l.to_lowercase().as_str()));
    let has_ing = lemmas.iter().any(|&l| l.ends_with("ing"));
    markers.has_progressive = has_be && has_ing;

    markers
}

/// Combined TAM inference result.
#[derive(Debug, Clone)]
pub struct TamFeatures {
    /// The inferred temporal frame.
    pub temporal_frame: TemporalFrame,
    /// The inferred aspectual viewpoint.
    pub aspectual_viewpoint: AspectualViewpoint,
    /// Aspect markers that were detected.
    pub markers: AspectMarkers,
}

impl TamFeatures {
    /// Infer TAM features from morphology and aspect markers.
    #[must_use]
    pub fn infer(morph: Option<&MorphFeatures>, markers: AspectMarkers) -> Self {
        Self {
            temporal_frame: infer_temporal_frame(morph, &markers),
            aspectual_viewpoint: infer_aspectual_viewpoint(morph, &markers),
            markers,
        }
    }

    /// Infer from morphology alone (no explicit markers).
    #[must_use]
    pub fn from_morph(morph: Option<&MorphFeatures>) -> Self {
        Self::infer(morph, AspectMarkers::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn morph_with_tense(tense: Tense) -> MorphFeatures {
        MorphFeatures {
            tense: Some(tense),
            ..Default::default()
        }
    }

    #[test]
    fn test_simple_past() {
        let morph = morph_with_tense(Tense::Past);
        let frame = infer_temporal_frame(Some(&morph), &AspectMarkers::default());
        assert!(frame.is_simple_past());
    }

    #[test]
    fn test_past_perfect() {
        let morph = morph_with_tense(Tense::Past);
        let markers = AspectMarkers::new(true, false);
        let frame = infer_temporal_frame(Some(&morph), &markers);
        assert!(frame.is_past_perfect());
    }

    #[test]
    fn test_progressive() {
        let morph = morph_with_tense(Tense::Past);
        let markers = AspectMarkers::new(false, true);
        let frame = infer_temporal_frame(Some(&morph), &markers);
        assert!(frame.is_progressive());
    }

    #[test]
    fn test_viewpoint_progressive() {
        let markers = AspectMarkers::new(false, true);
        let viewpoint = infer_aspectual_viewpoint(None, &markers);
        assert_eq!(viewpoint, AspectualViewpoint::Progressive);
    }

    #[test]
    fn test_viewpoint_perfect() {
        let markers = AspectMarkers::new(true, false);
        let viewpoint = infer_aspectual_viewpoint(None, &markers);
        assert_eq!(viewpoint, AspectualViewpoint::Perfect);
    }

    #[test]
    fn test_detect_markers_perfect() {
        let lemmas = vec!["John", "has", "left"];
        let markers = detect_aspect_markers_from_lemmas(&lemmas);
        assert!(markers.has_perfect);
        assert!(!markers.has_progressive);
    }

    #[test]
    fn test_detect_markers_progressive() {
        let lemmas = vec!["John", "is", "running"];
        let markers = detect_aspect_markers_from_lemmas(&lemmas);
        assert!(markers.has_progressive);
    }

    #[test]
    fn test_tam_features_infer() {
        let morph = morph_with_tense(Tense::Past);
        let markers = AspectMarkers::new(true, false);
        let tam = TamFeatures::infer(Some(&morph), markers);

        assert!(tam.temporal_frame.is_past_perfect());
        assert_eq!(tam.aspectual_viewpoint, AspectualViewpoint::Perfect);
    }

    #[test]
    fn test_no_tense_underspecified() {
        let frame = infer_temporal_frame(None, &AspectMarkers::default());
        // Should return underspecified frame
        assert!(matches!(
            frame.reference_time,
            crate::kernel::discourse::TimePoint::Underspecified(_)
        ));
    }
}
