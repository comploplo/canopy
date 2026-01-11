//! Confidence calibration for semantic engines.
//!
//! Different engines produce confidence scores on different scales.
//! This module provides calibration to normalize scores for comparison.

use super::lemma_query::ResourceSource;

/// Confidence calibration parameters for an engine.
///
/// Applies a linear transformation: `calibrated = raw * scale + offset`
/// with clamping to [0.0, 0.98].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConfidenceCalibration {
    /// Multiplicative scale factor.
    pub scale: f32,
    /// Additive offset.
    pub offset: f32,
}

impl ConfidenceCalibration {
    /// Create a new calibration.
    #[must_use]
    pub const fn new(scale: f32, offset: f32) -> Self {
        Self { scale, offset }
    }

    /// Identity calibration (no change).
    #[must_use]
    pub const fn identity() -> Self {
        Self {
            scale: 1.0,
            offset: 0.0,
        }
    }

    /// Calibration for `VerbNet`.
    ///
    /// `VerbNet` confidence tends to be slightly high, scale down.
    #[must_use]
    pub const fn verbnet() -> Self {
        Self {
            scale: 0.9,
            offset: 0.0,
        }
    }

    /// Calibration for `FrameNet`.
    ///
    /// `FrameNet` annotations are high-quality, slight boost.
    #[must_use]
    pub const fn framenet() -> Self {
        Self {
            scale: 1.0,
            offset: 0.05,
        }
    }

    /// Calibration for `PropBank`.
    ///
    /// `PropBank` is more conservative, slight scale down.
    #[must_use]
    pub const fn propbank() -> Self {
        Self {
            scale: 0.85,
            offset: 0.0,
        }
    }

    /// Calibration for `WordNet`.
    ///
    /// `WordNet` provides broad coverage but less precise.
    #[must_use]
    pub const fn wordnet() -> Self {
        Self {
            scale: 0.75,
            offset: 0.0,
        }
    }

    /// Calibration for Lexicon.
    ///
    /// Lexicon is deterministic, high confidence.
    #[must_use]
    pub const fn lexicon() -> Self {
        Self {
            scale: 0.95,
            offset: 0.0,
        }
    }

    /// Get calibration for a resource source.
    #[must_use]
    pub const fn for_source(source: ResourceSource) -> Self {
        match source {
            ResourceSource::VerbNet => Self::verbnet(),
            ResourceSource::FrameNet => Self::framenet(),
            ResourceSource::PropBank => Self::propbank(),
            ResourceSource::WordNet => Self::wordnet(),
            ResourceSource::Lexicon => Self::lexicon(),
        }
    }

    /// Apply calibration to a raw confidence score.
    ///
    /// Returns a value clamped to [0.0, 0.98] to leave room
    /// for perfect scores from human annotation.
    #[must_use]
    pub fn calibrate(&self, raw: f32) -> f32 {
        (raw * self.scale + self.offset).clamp(0.0, 0.98)
    }

    /// Apply inverse calibration (for debugging).
    #[must_use]
    pub fn uncalibrate(&self, calibrated: f32) -> f32 {
        if self.scale.abs() < f32::EPSILON {
            return 0.0;
        }
        (calibrated - self.offset) / self.scale
    }
}

impl Default for ConfidenceCalibration {
    fn default() -> Self {
        Self::identity()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_calibration() {
        let cal = ConfidenceCalibration::identity();
        assert!((cal.calibrate(0.5) - 0.5).abs() < f32::EPSILON);
        assert!((cal.calibrate(0.9) - 0.9).abs() < f32::EPSILON);
    }

    #[test]
    fn test_verbnet_calibration() {
        let cal = ConfidenceCalibration::verbnet();
        // 0.9 * 0.9 = 0.81
        assert!((cal.calibrate(0.9) - 0.81).abs() < 0.001);
    }

    #[test]
    fn test_framenet_calibration() {
        let cal = ConfidenceCalibration::framenet();
        // 0.8 * 1.0 + 0.05 = 0.85
        assert!((cal.calibrate(0.8) - 0.85).abs() < 0.001);
    }

    #[test]
    fn test_calibration_clamping() {
        let cal = ConfidenceCalibration::framenet();
        // Even 1.0 + offset should clamp to 0.98
        assert!((cal.calibrate(1.0) - 0.98).abs() < f32::EPSILON);

        // Negative should clamp to 0
        let negative_cal = ConfidenceCalibration::new(1.0, -2.0);
        assert!((negative_cal.calibrate(0.5) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_for_source() {
        let vn = ConfidenceCalibration::for_source(ResourceSource::VerbNet);
        let fn_ = ConfidenceCalibration::for_source(ResourceSource::FrameNet);
        let pb = ConfidenceCalibration::for_source(ResourceSource::PropBank);
        let wn = ConfidenceCalibration::for_source(ResourceSource::WordNet);
        let lex = ConfidenceCalibration::for_source(ResourceSource::Lexicon);

        assert_eq!(vn, ConfidenceCalibration::verbnet());
        assert_eq!(fn_, ConfidenceCalibration::framenet());
        assert_eq!(pb, ConfidenceCalibration::propbank());
        assert_eq!(wn, ConfidenceCalibration::wordnet());
        assert_eq!(lex, ConfidenceCalibration::lexicon());
    }

    #[test]
    fn test_uncalibrate() {
        let cal = ConfidenceCalibration::verbnet();
        let raw = 0.8;
        let calibrated = cal.calibrate(raw);
        let restored = cal.uncalibrate(calibrated);
        assert!((restored - raw).abs() < 0.001);
    }

    #[test]
    fn test_uncalibrate_zero_scale() {
        let cal = ConfidenceCalibration::new(0.0, 0.5);
        assert!((cal.uncalibrate(0.5) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_propbank_calibration() {
        let cal = ConfidenceCalibration::propbank();
        // 0.8 * 0.85 = 0.68
        assert!((cal.calibrate(0.8) - 0.68).abs() < 0.001);
    }

    #[test]
    fn test_wordnet_calibration() {
        let cal = ConfidenceCalibration::wordnet();
        // 0.8 * 0.75 = 0.6
        assert!((cal.calibrate(0.8) - 0.6).abs() < 0.001);
    }

    #[test]
    fn test_lexicon_calibration() {
        let cal = ConfidenceCalibration::lexicon();
        // 1.0 * 0.95 = 0.95
        assert!((cal.calibrate(1.0) - 0.95).abs() < 0.001);
    }

    #[test]
    fn test_default_is_identity() {
        let cal = ConfidenceCalibration::default();
        assert_eq!(cal, ConfidenceCalibration::identity());
    }
}
