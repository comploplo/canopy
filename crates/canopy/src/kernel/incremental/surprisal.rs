//! Surprisal computation and garden-path detection.
//!
//! Surprisal measures processing difficulty: S(w) = -log₂ P(w|context)
//! Higher surprisal = more unexpected = harder to process.

use std::fmt;

/// Surprisal at a word position, measured in bits.
///
/// Surprisal = -log₂ P(word | context)
///
/// # Interpretation
///
/// - 0 bits: Completely predictable (P = 1.0)
/// - 1 bit: 50% likely
/// - 3.3 bits: 10% likely
/// - 6.6 bits: 1% likely
/// - 10+ bits: Very unexpected (potential garden-path)
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Surprisal(f64);

impl Surprisal {
    /// Create surprisal from probability P(word|context).
    ///
    /// # Panics
    ///
    /// Panics if probability is not in (0, 1].
    #[must_use]
    pub fn from_probability(p: f64) -> Self {
        assert!(
            p > 0.0 && p <= 1.0,
            "Probability must be in (0, 1], got {p}"
        );
        Self(-p.log2())
    }

    /// Create surprisal from raw bits value.
    #[must_use]
    pub const fn from_bits(bits: f64) -> Self {
        Self(bits)
    }

    /// Get surprisal in bits (information content).
    #[must_use]
    pub const fn bits(&self) -> f64 {
        self.0
    }

    /// Convert back to probability.
    #[must_use]
    pub fn to_probability(&self) -> f64 {
        2.0_f64.powf(-self.0)
    }

    /// Zero surprisal (probability = 1.0).
    pub const ZERO: Self = Self(0.0);

    /// Check if this surprisal indicates a garden-path effect.
    ///
    /// Typically, surprisal > 10 bits suggests reanalysis.
    #[must_use]
    pub fn is_garden_path(&self, threshold: f64) -> bool {
        self.0 > threshold
    }
}

impl Default for Surprisal {
    fn default() -> Self {
        Self::ZERO
    }
}

impl fmt::Display for Surprisal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2} bits", self.0)
    }
}

impl std::ops::Add for Surprisal {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl std::ops::AddAssign for Surprisal {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0;
    }
}

impl std::iter::Sum for Surprisal {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::ZERO, |a, b| a + b)
    }
}

/// Detector for garden-path effects in incremental processing.
///
/// Garden-paths occur when initial parse commitments must be revised,
/// typically indicated by high surprisal at disambiguating words.
///
/// Classic example: "The horse raced past the barn fell"
///   - "fell" has very high surprisal because "raced" was parsed as main verb
///   - Requires reanalysis: "raced past the barn" is a reduced relative clause
#[derive(Debug, Clone)]
pub struct GardenPathDetector {
    /// Surprisal threshold for detecting garden-paths (in bits).
    /// Default: 10.0 bits (probability < 0.1%)
    pub threshold: f64,

    /// Minimum prefix length before detection is active.
    /// Prevents false positives at sentence start.
    pub min_prefix_length: usize,
}

impl Default for GardenPathDetector {
    fn default() -> Self {
        Self {
            threshold: 10.0,
            min_prefix_length: 3,
        }
    }
}

impl GardenPathDetector {
    /// Create detector with custom threshold.
    #[must_use]
    pub const fn with_threshold(threshold: f64) -> Self {
        Self {
            threshold,
            min_prefix_length: 3,
        }
    }

    /// Detect garden-path events in a surprisal trace.
    ///
    /// Returns the first garden-path event found, if any.
    #[must_use]
    pub fn detect(&self, trace: &[Surprisal]) -> Option<GardenPathEvent> {
        if trace.len() < self.min_prefix_length {
            return None;
        }

        for (idx, &surprisal) in trace.iter().enumerate().skip(self.min_prefix_length - 1) {
            if surprisal.bits() > self.threshold {
                return Some(GardenPathEvent {
                    word_index: idx,
                    surprisal,
                    severity: self.compute_severity(surprisal),
                });
            }
        }

        None
    }

    /// Detect all garden-path events in a trace.
    #[must_use]
    pub fn detect_all(&self, trace: &[Surprisal]) -> Vec<GardenPathEvent> {
        if trace.len() < self.min_prefix_length {
            return Vec::new();
        }

        trace
            .iter()
            .enumerate()
            .skip(self.min_prefix_length - 1)
            .filter(|(_, s)| s.bits() > self.threshold)
            .map(|(idx, &surprisal)| GardenPathEvent {
                word_index: idx,
                surprisal,
                severity: self.compute_severity(surprisal),
            })
            .collect()
    }

    fn compute_severity(&self, surprisal: Surprisal) -> GardenPathSeverity {
        let excess = surprisal.bits() - self.threshold;
        if excess > 10.0 {
            GardenPathSeverity::Severe
        } else if excess > 5.0 {
            GardenPathSeverity::Moderate
        } else {
            GardenPathSeverity::Mild
        }
    }
}

/// A detected garden-path event.
#[derive(Debug, Clone)]
pub struct GardenPathEvent {
    /// Word index where garden-path was detected.
    pub word_index: usize,

    /// Surprisal at the disambiguating word.
    pub surprisal: Surprisal,

    /// Severity of the garden-path effect.
    pub severity: GardenPathSeverity,
}

/// Severity of a garden-path effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GardenPathSeverity {
    /// Mild garden-path (5-10 bits above threshold).
    Mild,
    /// Moderate garden-path (10-15 bits above threshold).
    Moderate,
    /// Severe garden-path (>15 bits above threshold).
    Severe,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_surprisal_from_probability() {
        // P = 1.0 -> 0 bits
        let s = Surprisal::from_probability(1.0);
        assert!((s.bits() - 0.0).abs() < f64::EPSILON);

        // P = 0.5 -> 1 bit
        let s = Surprisal::from_probability(0.5);
        assert!((s.bits() - 1.0).abs() < f64::EPSILON);

        // P = 0.25 -> 2 bits
        let s = Surprisal::from_probability(0.25);
        assert!((s.bits() - 2.0).abs() < f64::EPSILON);

        // P = 0.001 -> ~10 bits
        let s = Surprisal::from_probability(0.001);
        assert!((s.bits() - 9.97).abs() < 0.01);
    }

    #[test]
    fn test_surprisal_round_trip() {
        let original = 5.5;
        let s = Surprisal::from_bits(original);
        let p = s.to_probability();
        let s2 = Surprisal::from_probability(p);
        assert!((s2.bits() - original).abs() < 1e-10);
    }

    #[test]
    fn test_surprisal_addition() {
        let s1 = Surprisal::from_bits(2.0);
        let s2 = Surprisal::from_bits(3.0);
        let sum = s1 + s2;
        assert!((sum.bits() - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_surprisal_sum() {
        let surprisals = vec![
            Surprisal::from_bits(1.0),
            Surprisal::from_bits(2.0),
            Surprisal::from_bits(3.0),
        ];
        let total: Surprisal = surprisals.into_iter().sum();
        assert!((total.bits() - 6.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_garden_path_detection() {
        let detector = GardenPathDetector::with_threshold(10.0);

        // Normal processing - no garden path
        let trace = vec![
            Surprisal::from_bits(3.0),
            Surprisal::from_bits(4.0),
            Surprisal::from_bits(2.0),
            Surprisal::from_bits(5.0),
        ];
        assert!(detector.detect(&trace).is_none());

        // Garden path at word 4
        let trace = vec![
            Surprisal::from_bits(3.0),
            Surprisal::from_bits(4.0),
            Surprisal::from_bits(2.0),
            Surprisal::from_bits(15.0), // Garden path!
        ];
        let event = detector.detect(&trace).expect("Should detect garden path");
        assert_eq!(event.word_index, 3);
        assert!((event.surprisal.bits() - 15.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_garden_path_severity() {
        let detector = GardenPathDetector::with_threshold(10.0);

        // Mild: 0-5 bits above threshold (14 - 10 = 4)
        let trace = vec![
            Surprisal::from_bits(2.0),
            Surprisal::from_bits(2.0),
            Surprisal::from_bits(14.0),
        ];
        let event = detector.detect(&trace).unwrap();
        assert_eq!(event.severity, GardenPathSeverity::Mild);

        // Moderate: 5-10 bits above threshold (18 - 10 = 8)
        let trace = vec![
            Surprisal::from_bits(2.0),
            Surprisal::from_bits(2.0),
            Surprisal::from_bits(18.0),
        ];
        let event = detector.detect(&trace).unwrap();
        assert_eq!(event.severity, GardenPathSeverity::Moderate);

        // Severe: >10 bits above threshold (25 - 10 = 15)
        let trace = vec![
            Surprisal::from_bits(2.0),
            Surprisal::from_bits(2.0),
            Surprisal::from_bits(25.0),
        ];
        let event = detector.detect(&trace).unwrap();
        assert_eq!(event.severity, GardenPathSeverity::Severe);
    }

    #[test]
    fn test_min_prefix_length() {
        let detector = GardenPathDetector {
            threshold: 10.0,
            min_prefix_length: 3,
        };

        // Too short - should not detect
        let trace = vec![Surprisal::from_bits(20.0), Surprisal::from_bits(20.0)];
        assert!(detector.detect(&trace).is_none());
    }
}
