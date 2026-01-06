//! Temporal frame semantics.
//!
//! Implements Reichenbach's (1947) tense semantics with three temporal points:
//! - Speech time (S): when the utterance occurs
//! - Reference time (R): the perspective from which the event is viewed
//! - Event time (E): when the event actually occurs
//!
//! Tense is the relation between S and R; aspect is the relation between R and E.
//!
//! ## Examples
//!
//! | Tense           | Structure     | Example                |
//! |-----------------|---------------|------------------------|
//! | Simple past     | E < R = S     | "John left"            |
//! | Past perfect    | E < R < S     | "John had left"        |
//! | Future perfect  | E < R, S < R  | "John will have left"  |
//! | Past progressive| E ○ R < S     | "John was leaving"     |

use super::ReferentId;
use serde::{Deserialize, Serialize};

// ============================================================================
// Time Points and Intervals
// ============================================================================

/// A point in time.
///
/// Time points can be absolute (anchored to speech time), relative to other
/// points, or bound to event referents.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum TimePoint {
    /// Speech time (now, utterance time).
    #[default]
    Now,

    /// Relative to another time point.
    Relative {
        anchor: Box<TimePoint>,
        offset: TemporalOffset,
    },

    /// Bound to an event referent's temporal location.
    EventBound(ReferentId),

    /// Underspecified (for temporal anaphora resolution).
    Underspecified(u32),
}

impl TimePoint {
    /// Create a time point before another.
    #[must_use]
    pub fn before(anchor: TimePoint) -> Self {
        Self::Relative {
            anchor: Box::new(anchor),
            offset: TemporalOffset::Before,
        }
    }

    /// Create a time point after another.
    #[must_use]
    pub fn after(anchor: TimePoint) -> Self {
        Self::Relative {
            anchor: Box::new(anchor),
            offset: TemporalOffset::After,
        }
    }

    /// Check if this is the speech time.
    #[must_use]
    pub const fn is_now(&self) -> bool {
        matches!(self, Self::Now)
    }

    /// Check if this is underspecified.
    #[must_use]
    pub const fn is_underspecified(&self) -> bool {
        matches!(self, Self::Underspecified(_))
    }
}

/// Temporal offset from an anchor point.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TemporalOffset {
    /// Precedes the anchor.
    Before,
    /// Follows the anchor.
    After,
    /// Overlaps with the anchor.
    Overlapping,
    /// Simultaneous with the anchor.
    At,
}

/// A time interval with start and end points.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimeInterval {
    /// Start of the interval.
    pub start: TimePoint,
    /// End of the interval.
    pub end: TimePoint,
    /// Boundedness (telicity).
    pub boundedness: Boundedness,
}

impl TimeInterval {
    /// Create an interval before a time point.
    #[must_use]
    pub fn before(point: TimePoint) -> Self {
        Self {
            start: TimePoint::Underspecified(0),
            end: TimePoint::before(point),
            boundedness: Boundedness::Bounded,
        }
    }

    /// Create an interval at a time point (punctual).
    #[must_use]
    pub fn at(point: TimePoint) -> Self {
        Self {
            start: point.clone(),
            end: point,
            boundedness: Boundedness::Bounded,
        }
    }

    /// Create an interval after a time point.
    #[must_use]
    pub fn after(point: TimePoint) -> Self {
        Self {
            start: TimePoint::after(point),
            end: TimePoint::Underspecified(0),
            boundedness: Boundedness::Bounded,
        }
    }

    /// Create an interval overlapping a time point.
    #[must_use]
    pub fn overlapping(point: TimePoint) -> Self {
        Self {
            start: TimePoint::before(point.clone()),
            end: TimePoint::after(point),
            boundedness: Boundedness::Unbounded,
        }
    }

    /// Check if this interval is bounded (telic).
    #[must_use]
    pub const fn is_bounded(&self) -> bool {
        matches!(self.boundedness, Boundedness::Bounded)
    }
}

impl Default for TimeInterval {
    fn default() -> Self {
        Self::at(TimePoint::Now)
    }
}

/// Boundedness of an interval (telicity).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Boundedness {
    /// Telic: has an inherent endpoint ("build a house").
    #[default]
    Bounded,
    /// Atelic: no inherent endpoint ("run").
    Unbounded,
    /// Stative: no internal structure ("know").
    Stative,
}

// ============================================================================
// Temporal Frame (Reichenbachian)
// ============================================================================

/// Reichenbachian temporal frame.
///
/// Encodes tense as the configuration of three temporal points:
/// - Speech time (S): utterance time
/// - Reference time (R): perspective time
/// - Event time (E): when the event occurs
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemporalFrame {
    /// Speech time (S) - when the utterance occurs.
    pub speech_time: TimePoint,
    /// Reference time (R) - the temporal perspective.
    pub reference_time: TimePoint,
    /// Event time (E) - when the event occurs.
    pub event_time: TimeInterval,
}

impl TemporalFrame {
    /// Create an underspecified temporal frame.
    #[must_use]
    pub fn underspecified() -> Self {
        Self {
            speech_time: TimePoint::Now,
            reference_time: TimePoint::Underspecified(0),
            event_time: TimeInterval {
                start: TimePoint::Underspecified(1),
                end: TimePoint::Underspecified(2),
                boundedness: Boundedness::Bounded,
            },
        }
    }

    /// Create a simple past frame: E < R = S.
    #[must_use]
    pub fn past() -> Self {
        Self {
            speech_time: TimePoint::Now,
            reference_time: TimePoint::Now,
            event_time: TimeInterval::before(TimePoint::Now),
        }
    }

    /// Create a simple present frame: E = R = S.
    #[must_use]
    pub fn present() -> Self {
        Self {
            speech_time: TimePoint::Now,
            reference_time: TimePoint::Now,
            event_time: TimeInterval::at(TimePoint::Now),
        }
    }

    /// Create a simple future frame: S = R < E.
    #[must_use]
    pub fn future() -> Self {
        Self {
            speech_time: TimePoint::Now,
            reference_time: TimePoint::Now,
            event_time: TimeInterval::after(TimePoint::Now),
        }
    }

    /// Create a past perfect frame: E < R < S.
    #[must_use]
    pub fn past_perfect() -> Self {
        let reference = TimePoint::before(TimePoint::Now);
        Self {
            speech_time: TimePoint::Now,
            reference_time: reference.clone(),
            event_time: TimeInterval::before(reference),
        }
    }

    /// Create a present perfect frame: E < R = S (with result state at R).
    #[must_use]
    pub fn present_perfect() -> Self {
        Self {
            speech_time: TimePoint::Now,
            reference_time: TimePoint::Now,
            event_time: TimeInterval::before(TimePoint::Now),
        }
    }

    /// Create a future perfect frame: S < R, E < R.
    #[must_use]
    pub fn future_perfect() -> Self {
        let reference = TimePoint::after(TimePoint::Now);
        Self {
            speech_time: TimePoint::Now,
            reference_time: reference.clone(),
            event_time: TimeInterval::before(reference),
        }
    }

    /// Create a past progressive frame: E ○ R < S.
    #[must_use]
    pub fn past_progressive() -> Self {
        let reference = TimePoint::before(TimePoint::Now);
        Self {
            speech_time: TimePoint::Now,
            reference_time: reference.clone(),
            event_time: TimeInterval::overlapping(reference),
        }
    }

    /// Create a present progressive frame: E ○ R = S.
    #[must_use]
    pub fn present_progressive() -> Self {
        Self {
            speech_time: TimePoint::Now,
            reference_time: TimePoint::Now,
            event_time: TimeInterval::overlapping(TimePoint::Now),
        }
    }

    /// Create a future progressive frame: S < R, E ○ R.
    #[must_use]
    pub fn future_progressive() -> Self {
        let reference = TimePoint::after(TimePoint::Now);
        Self {
            speech_time: TimePoint::Now,
            reference_time: reference.clone(),
            event_time: TimeInterval::overlapping(reference),
        }
    }

    /// Check if this is a simple past tense (E < R = S).
    #[must_use]
    pub fn is_simple_past(&self) -> bool {
        self.reference_time.is_now() && !self.event_time.start.is_now()
    }

    /// Check if this is a past perfect (E < R < S).
    #[must_use]
    pub fn is_past_perfect(&self) -> bool {
        matches!(
            &self.reference_time,
            TimePoint::Relative {
                offset: TemporalOffset::Before,
                ..
            }
        )
    }

    /// Check if this is progressive (E overlaps R).
    #[must_use]
    pub fn is_progressive(&self) -> bool {
        matches!(self.event_time.boundedness, Boundedness::Unbounded)
    }
}

impl Default for TemporalFrame {
    fn default() -> Self {
        Self::present()
    }
}

// ============================================================================
// Aspectual Viewpoint
// ============================================================================

/// Aspectual viewpoint (how the event is viewed).
///
/// Distinct from Vendlerian aspectual class (what kind of event it is).
/// Viewpoint is grammatical aspect; class is lexical aspect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AspectualViewpoint {
    /// Perfective: views event from outside, as completed whole.
    #[default]
    Perfective,
    /// Imperfective: views event from inside, ongoing.
    Imperfective,
    /// Perfect: focuses on result state at reference time.
    Perfect,
    /// Progressive: event in progress at reference time.
    Progressive,
    /// Prospective: event about to happen.
    Prospective,
    /// Habitual: repeated/characteristic events.
    Habitual,
}

impl AspectualViewpoint {
    /// Check if this viewpoint is compatible with stative predicates.
    #[must_use]
    pub const fn compatible_with_state(&self) -> bool {
        matches!(
            self,
            Self::Perfective | Self::Imperfective | Self::Perfect | Self::Habitual
        )
    }

    /// Check if this viewpoint implies ongoing action.
    #[must_use]
    pub const fn is_ongoing(&self) -> bool {
        matches!(self, Self::Imperfective | Self::Progressive)
    }
}

// ============================================================================
// Aspectual Operators (for DRS)
// ============================================================================

/// Aspectual operators for DRS conditions.
///
/// These modify how an event is temporally located relative to reference time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AspectualOperator {
    /// PROG(e): progressive, event ongoing at reference time.
    Progressive,
    /// PERF(e): perfect, event completed with result state at reference time.
    Perfect,
    /// HAB(e): habitual, event occurs regularly.
    Habitual,
    /// INCH(e): inchoative, event beginning.
    Inchoative,
    /// TERM(e): terminative, event ending.
    Terminative,
}

impl std::fmt::Display for AspectualOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Progressive => write!(f, "PROG"),
            Self::Perfect => write!(f, "PERF"),
            Self::Habitual => write!(f, "HAB"),
            Self::Inchoative => write!(f, "INCH"),
            Self::Terminative => write!(f, "TERM"),
        }
    }
}

// ============================================================================
// Temporal Anchor Type
// ============================================================================

/// How an event is anchored to a reference point.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TemporalAnchorType {
    /// Event precedes anchor.
    Before,
    /// Event at anchor (simultaneous).
    At,
    /// Event follows anchor.
    After,
    /// Event overlaps anchor.
    Overlapping,
}

impl From<TemporalOffset> for TemporalAnchorType {
    fn from(offset: TemporalOffset) -> Self {
        match offset {
            TemporalOffset::Before => Self::Before,
            TemporalOffset::After => Self::After,
            TemporalOffset::At => Self::At,
            TemporalOffset::Overlapping => Self::Overlapping,
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_past_frame() {
        let frame = TemporalFrame::past();
        assert!(frame.reference_time.is_now());
        assert!(frame.is_simple_past());
        assert!(!frame.is_past_perfect());
    }

    #[test]
    fn test_past_perfect_frame() {
        let frame = TemporalFrame::past_perfect();
        assert!(frame.is_past_perfect());
        assert!(!frame.is_simple_past());
    }

    #[test]
    fn test_present_frame() {
        let frame = TemporalFrame::present();
        assert!(frame.reference_time.is_now());
        assert!(frame.speech_time.is_now());
    }

    #[test]
    fn test_progressive_frames() {
        let past_prog = TemporalFrame::past_progressive();
        assert!(past_prog.is_progressive());

        let present_prog = TemporalFrame::present_progressive();
        assert!(present_prog.is_progressive());
    }

    #[test]
    fn test_time_interval_before() {
        let interval = TimeInterval::before(TimePoint::Now);
        assert!(interval.is_bounded());
    }

    #[test]
    fn test_time_interval_overlapping() {
        let interval = TimeInterval::overlapping(TimePoint::Now);
        assert!(!interval.is_bounded());
    }

    #[test]
    fn test_aspectual_viewpoint() {
        assert!(AspectualViewpoint::Progressive.is_ongoing());
        assert!(AspectualViewpoint::Imperfective.is_ongoing());
        assert!(!AspectualViewpoint::Perfective.is_ongoing());

        assert!(AspectualViewpoint::Perfective.compatible_with_state());
        assert!(!AspectualViewpoint::Progressive.compatible_with_state());
    }

    #[test]
    fn test_aspectual_operator_display() {
        assert_eq!(format!("{}", AspectualOperator::Progressive), "PROG");
        assert_eq!(format!("{}", AspectualOperator::Perfect), "PERF");
    }

    #[test]
    fn test_temporal_anchor_from_offset() {
        assert_eq!(
            TemporalAnchorType::from(TemporalOffset::Before),
            TemporalAnchorType::Before
        );
        assert_eq!(
            TemporalAnchorType::from(TemporalOffset::After),
            TemporalAnchorType::After
        );
    }
}
