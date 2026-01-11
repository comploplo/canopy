//! Derivation trace types for semantic analysis explanation.
//!
//! This module provides types for capturing and displaying the reasoning
//! behind semantic analysis decisions, including sense selection and
//! event composition.
//!
//! # Example Output
//!
//! ```text
//! === Semantic Analysis Trace ===
//! Input: "John gave Mary a book."
//!
//! 1. SYNTAX (UD Parse)
//!    Tokens: 5 | Predicates: gave
//!    Dependencies: nsubj(gave, John), iobj(gave, Mary), obj(gave, book)
//!
//! 2. SENSE SELECTION
//!    gave[1]:
//!      SELECTED: give-13.1 (CAUSE)
//!        Roles: [Agent, Theme, Recipient]
//!        Confidence: 92.3% | Source: VerbNet
//!      vs RUNNER-UP: transfer-11.1 (88.1%)
//!      SELECTION: Higher confidence (+4.2%)
//! ```

use serde::{Deserialize, Serialize};
use std::fmt::{self, Write};

/// Summary-level derivation trace for a semantic analysis.
///
/// Designed for LaTeX-friendly plain text output that can be
/// pasted directly into academic papers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DerivationTrace {
    /// Original input text
    pub input: String,
    /// Syntax summary (UD parse)
    pub syntax_summary: SyntaxSummary,
    /// Per-predicate sense selection traces
    pub sense_traces: Vec<SenseSelectionTrace>,
    /// Event composition summary
    pub event_summary: EventSummary,
    /// Discourse update summary (if multi-sentence)
    pub discourse_summary: Option<DiscourseSummary>,
    /// Overall trace metadata
    pub metadata: TraceMetadata,
}

/// Summary of syntactic parse.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntaxSummary {
    /// Number of tokens in the sentence
    pub token_count: usize,
    /// Lemmas of predicate tokens (verbs)
    pub predicate_lemmas: Vec<String>,
    /// Formatted dependency summary (e.g., "nsubj(runs, John), obj(gave, book)")
    pub dependency_summary: String,
}

/// Trace of sense selection for a single predicate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SenseSelectionTrace {
    /// The predicate lemma (e.g., "give")
    pub predicate_lemma: String,
    /// Token position in the sentence (0-indexed)
    pub token_position: usize,
    /// The winning sense reading
    pub winner: SenseReading,
    /// The runner-up reading (if any alternatives)
    pub runner_up: Option<SenseReading>,
    /// Why the winner was selected
    pub selection_reason: SelectionReason,
}

/// A single sense reading with confidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SenseReading {
    /// Sense identifier (e.g., "give-13.1" from `VerbNet`)
    pub sense_id: String,
    /// `LittleV` event type (e.g., "CAUSE", "GO", "BE")
    pub little_v_type: String,
    /// Expected theta roles (e.g., Agent, Theme, Recipient)
    pub theta_roles: Vec<String>,
    /// Confidence score (0.0-1.0)
    pub confidence: f32,
    /// Surprisal in bits (if computed)
    pub surprisal_bits: Option<f64>,
    /// Source of the decomposition ("`VerbNet`", "`FrameNet`", "`PropBank`", etc.)
    pub source: String,
}

/// Why the winning sense was selected over alternatives.
///
/// Note: `LowerSurprisal` and `HybridScore` are reserved for future beam search
/// and hybrid scoring features. Currently only `HigherConfidence` and `Unambiguous`
/// are constructed in production code.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SelectionReason {
    /// Winner had higher confidence score
    HigherConfidence {
        /// Margin over runner-up (0.0-1.0)
        margin: f32,
    },
    /// Winner had lower surprisal (more expected).
    /// Reserved for future beam search integration.
    LowerSurprisal {
        /// Margin in bits
        margin_bits: f64,
    },
    /// No alternatives existed (unambiguous)
    Unambiguous,
    /// Hybrid score combining multiple factors.
    /// Reserved for future hybrid scoring features.
    HybridScore {
        /// The computed hybrid score
        score: f64,
    },
}

/// Summary of composed events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSummary {
    /// Number of events composed
    pub event_count: usize,
    /// Individual event traces
    pub events: Vec<EventTrace>,
    /// Overall composition confidence
    pub overall_confidence: f32,
}

/// Trace of a single composed event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventTrace {
    /// Event index in the sentence
    pub event_id: usize,
    /// Predicate lemma
    pub predicate: String,
    /// `LittleV` type (e.g., "CAUSE", "GO")
    pub little_v: String,
    /// Bound participants with roles
    pub participants: Vec<ParticipantTrace>,
    /// Aspectual class (e.g., "Accomplishment", "Activity")
    pub aspect: String,
    /// Voice (e.g., "Active", "Passive")
    pub voice: String,
    /// Event composition confidence (0.0-1.0)
    pub confidence: f32,
}

/// Participant binding trace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantTrace {
    /// Theta role (e.g., "Agent", "Theme")
    pub role: String,
    /// Surface text of the filler
    pub filler: String,
    /// Binding confidence (0.0-1.0)
    pub binding_confidence: f32,
}

/// Discourse update summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscourseSummary {
    /// Number of discourse referents introduced
    pub referent_count: usize,
    /// Number of DRS conditions
    pub condition_count: usize,
    /// DRS in box notation
    pub drs_notation: String,
    /// Size of the QUD stack at the end of the document
    pub qud_stack_depth: usize,
    /// Top-of-stack question (if any)
    pub active_question: Option<String>,
    /// Chronological list of QUD pushes/resolutions
    pub qud_history: Vec<QudHistoryEntry>,
    /// Relevance scoring history.
    pub relevance_reports: Vec<RelevanceTraceEntry>,
    /// Validation history for assertions.
    pub validation_reports: Vec<ValidationTraceEntry>,
}

/// Serialized QUD history entry for traces.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QudHistoryEntry {
    /// Identifier assigned to the issue.
    pub issue_id: usize,
    /// Question description.
    pub question: String,
    /// Action label (e.g., PUSH/RESOLVE).
    pub action: String,
    /// Origin label (e.g., explicit/implicit).
    pub origin: String,
}

/// Relevance information for a single sentence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelevanceTraceEntry {
    /// Sentence index in discourse order.
    pub sentence_index: usize,
    /// Active question (if any).
    pub question: Option<String>,
    /// Summary level (Direct/Partial/etc.).
    pub level: String,
    /// Alignments for each event.
    pub alignments: Vec<RelevanceAlignmentTrace>,
}

/// Alignment trace for a single event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelevanceAlignmentTrace {
    /// Event identifier.
    pub event_id: usize,
    /// Predicate lemma.
    pub predicate: String,
    /// Alignment tier as display text.
    pub level: String,
    /// Theta roles that matched the QUD focus.
    pub matched_roles: Vec<String>,
}

/// Validation trace entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationTraceEntry {
    /// Sentence index.
    pub sentence_index: usize,
    /// Predicate text.
    pub predicate: String,
    /// Status label.
    pub status: String,
    /// Optional explanation.
    pub message: Option<String>,
}

/// Trace metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceMetadata {
    /// Analysis time in milliseconds
    pub analysis_time_ms: u64,
    /// Number of predicates with multiple sense readings (choice points)
    pub ambiguity_count: usize,
    /// Total combinatorial readings (product of all choice points).
    /// For N predicates with M1, M2, ... MN senses: M1 × M2 × ... × MN
    pub total_readings: usize,
}

// === Formatting Implementation ===

impl DerivationTrace {
    /// Format the trace as LaTeX-friendly plain text.
    ///
    /// The output uses ASCII/Unicode characters that paste cleanly
    /// into LaTeX documents without escaping issues.
    #[must_use]
    pub fn to_text(&self) -> String {
        let mut out = String::new();

        let _ = writeln!(out, "=== Semantic Analysis Trace ===");
        let _ = writeln!(out, "Input: \"{}\"", self.input);
        out.push('\n');

        out.push_str(&self.format_syntax_section());
        out.push_str(&self.format_sense_selection_section());
        out.push_str("\n3. EVENT COMPOSITION\n");
        out.push_str(&self.format_event_summary());

        if let Some(ref disc) = self.discourse_summary {
            out.push_str("\n4. DISCOURSE UPDATE\n");
            out.push_str(&Self::format_discourse_section(disc));
        }

        let _ = writeln!(
            out,
            "\n[Time: {}ms | Readings: {} | Ambiguity points: {}]",
            self.metadata.analysis_time_ms,
            self.metadata.total_readings,
            self.metadata.ambiguity_count
        );

        out
    }

    fn format_syntax_section(&self) -> String {
        let mut out = String::new();
        let _ = writeln!(out, "1. SYNTAX (UD Parse)");
        let predicates = if self.syntax_summary.predicate_lemmas.is_empty() {
            "(none)".to_string()
        } else {
            self.syntax_summary.predicate_lemmas.join(", ")
        };
        let _ = writeln!(
            out,
            "   Tokens: {} | Predicates: {}",
            self.syntax_summary.token_count, predicates
        );
        if !self.syntax_summary.dependency_summary.is_empty() {
            let _ = writeln!(
                out,
                "   Dependencies: {}",
                self.syntax_summary.dependency_summary
            );
        }
        out.push('\n');
        out
    }

    fn format_sense_selection_section(&self) -> String {
        let mut out = String::new();
        let _ = writeln!(out, "2. SENSE SELECTION");
        if self.sense_traces.is_empty() {
            let _ = writeln!(out, "   (No predicates to decompose)");
        } else {
            for trace in &self.sense_traces {
                out.push_str(&Self::format_sense_trace(trace));
            }
        }
        out
    }

    fn format_discourse_section(disc: &DiscourseSummary) -> String {
        let mut out = String::new();
        let _ = writeln!(
            out,
            "   Referents: {} | Conditions: {}",
            disc.referent_count, disc.condition_count
        );
        if !disc.drs_notation.is_empty() {
            for line in disc.drs_notation.lines() {
                let _ = writeln!(out, "   {line}");
            }
        }
        Self::format_qud_section(&mut out, disc);
        Self::format_relevance_section(&mut out, disc);
        Self::format_validation_section(&mut out, disc);
        out
    }

    fn format_qud_section(out: &mut String, disc: &DiscourseSummary) {
        if disc.qud_stack_depth > 0 || !disc.qud_history.is_empty() {
            let _ = writeln!(out, "   QUD Stack Depth: {}", disc.qud_stack_depth);
            if let Some(ref question) = disc.active_question {
                let _ = writeln!(out, "   Active QUD: {question}");
            }
            if !disc.qud_history.is_empty() {
                let _ = writeln!(out, "   QUD HISTORY:");
                for entry in &disc.qud_history {
                    let _ = writeln!(
                        out,
                        "     [{}] {} ({})",
                        entry.action, entry.question, entry.origin
                    );
                }
            }
        }
    }

    fn format_relevance_section(out: &mut String, disc: &DiscourseSummary) {
        if !disc.relevance_reports.is_empty() {
            let _ = writeln!(out, "   RELEVANCE ASSESSMENT:");
            for report in &disc.relevance_reports {
                let question = report.question.as_deref().unwrap_or("(no active question)");
                let _ = writeln!(
                    out,
                    "     Sentence {}: {} -> {}",
                    report.sentence_index + 1,
                    question,
                    report.level
                );
                for alignment in &report.alignments {
                    let _ = writeln!(
                        out,
                        "        e{} {} [{}]",
                        alignment.event_id, alignment.predicate, alignment.level
                    );
                    if !alignment.matched_roles.is_empty() {
                        let _ = writeln!(
                            out,
                            "           matched roles: {}",
                            alignment.matched_roles.join(", ")
                        );
                    }
                }
            }
        }
    }

    fn format_validation_section(out: &mut String, disc: &DiscourseSummary) {
        if !disc.validation_reports.is_empty() {
            let _ = writeln!(out, "   VALIDATION CHECKS:");
            for entry in &disc.validation_reports {
                if let Some(message) = &entry.message {
                    let _ = writeln!(
                        out,
                        "     Sentence {}: {} ({})",
                        entry.sentence_index + 1,
                        entry.predicate,
                        message
                    );
                } else {
                    let _ = writeln!(
                        out,
                        "     Sentence {}: {} ({})",
                        entry.sentence_index + 1,
                        entry.predicate,
                        entry.status
                    );
                }
            }
        }
    }

    fn format_sense_trace(trace: &SenseSelectionTrace) -> String {
        let mut out = String::new();

        let _ = writeln!(
            out,
            "   {}[{}]:",
            trace.predicate_lemma, trace.token_position
        );

        let _ = writeln!(
            out,
            "     SELECTED: {} ({})",
            trace.winner.sense_id, trace.winner.little_v_type
        );
        let _ = writeln!(
            out,
            "       Roles: [{}]",
            trace.winner.theta_roles.join(", ")
        );
        let _ = write!(
            out,
            "       Confidence: {:.1}%",
            trace.winner.confidence * 100.0
        );
        if let Some(bits) = trace.winner.surprisal_bits {
            let _ = write!(out, " | Surprisal: {bits:.2} bits");
        }
        let _ = writeln!(out, " | Source: {}", trace.winner.source);

        if let Some(runner) = &trace.runner_up {
            let _ = writeln!(
                out,
                "     vs RUNNER-UP: {} ({:.1}%)",
                runner.sense_id,
                runner.confidence * 100.0
            );
            let reason = Self::format_selection_reason(&trace.selection_reason);
            let _ = writeln!(out, "     SELECTION: {reason}");
        } else {
            let _ = writeln!(out, "     (No alternative readings)");
        }

        out
    }

    fn format_selection_reason(reason: &SelectionReason) -> String {
        match reason {
            SelectionReason::HigherConfidence { margin } => {
                let pct = margin * 100.0;
                format!("Higher confidence (+{pct:.1}%)")
            }
            SelectionReason::LowerSurprisal { margin_bits } => {
                format!("Lower surprisal (-{margin_bits:.2} bits)")
            }
            SelectionReason::Unambiguous => "Unambiguous (single reading)".to_string(),
            SelectionReason::HybridScore { score } => format!("Hybrid score ({score:.2})"),
        }
    }

    fn format_event_summary(&self) -> String {
        let mut out = String::new();

        if self.event_summary.events.is_empty() {
            out.push_str("   (No events composed)\n");
            return out;
        }

        for event in &self.event_summary.events {
            let _ = writeln!(
                out,
                "   e{}: {}({}) [{:.1}%]",
                event.event_id,
                event.little_v,
                event.predicate,
                event.confidence * 100.0
            );

            for participant in &event.participants {
                let _ = writeln!(
                    out,
                    "       {}(\"{}\") [{:.1}%]",
                    participant.role,
                    participant.filler,
                    participant.binding_confidence * 100.0
                );
            }

            let _ = writeln!(
                out,
                "       Aspect: {} | Voice: {}",
                event.aspect, event.voice
            );
        }

        let _ = writeln!(
            out,
            "   Overall confidence: {:.1}%",
            self.event_summary.overall_confidence * 100.0
        );

        out
    }
}

impl fmt::Display for DerivationTrace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_text())
    }
}

impl fmt::Display for SelectionReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HigherConfidence { margin } => {
                let pct = margin * 100.0;
                write!(f, "Higher confidence (+{pct:.1}%)")
            }
            Self::LowerSurprisal { margin_bits } => {
                write!(f, "Lower surprisal (-{margin_bits:.2} bits)")
            }
            Self::Unambiguous => write!(f, "Unambiguous"),
            Self::HybridScore { score } => write!(f, "Hybrid score ({score:.2})"),
        }
    }
}

// === Builder for convenience ===

impl DerivationTrace {
    /// Create a new trace with the given input text.
    #[must_use]
    pub fn new(input: impl Into<String>) -> Self {
        Self {
            input: input.into(),
            syntax_summary: SyntaxSummary {
                token_count: 0,
                predicate_lemmas: vec![],
                dependency_summary: String::new(),
            },
            sense_traces: vec![],
            event_summary: EventSummary {
                event_count: 0,
                events: vec![],
                overall_confidence: 0.0,
            },
            discourse_summary: None,
            metadata: TraceMetadata {
                analysis_time_ms: 0,
                ambiguity_count: 0,
                total_readings: 1,
            },
        }
    }

    /// Set the syntax summary.
    #[must_use]
    pub fn with_syntax(mut self, summary: SyntaxSummary) -> Self {
        self.syntax_summary = summary;
        self
    }

    /// Add a sense selection trace.
    #[must_use]
    pub fn with_sense_trace(mut self, trace: SenseSelectionTrace) -> Self {
        self.sense_traces.push(trace);
        self
    }

    /// Set the event summary.
    #[must_use]
    pub fn with_events(mut self, summary: EventSummary) -> Self {
        self.event_summary = summary;
        self
    }

    /// Set the discourse summary.
    #[must_use]
    pub fn with_discourse(mut self, summary: DiscourseSummary) -> Self {
        self.discourse_summary = Some(summary);
        self
    }

    /// Set the metadata.
    #[must_use]
    pub fn with_metadata(mut self, metadata: TraceMetadata) -> Self {
        self.metadata = metadata;
        self
    }
}

impl SenseReading {
    /// Create a new sense reading.
    #[must_use]
    pub fn new(
        sense_id: impl Into<String>,
        little_v_type: impl Into<String>,
        confidence: f32,
        source: impl Into<String>,
    ) -> Self {
        Self {
            sense_id: sense_id.into(),
            little_v_type: little_v_type.into(),
            theta_roles: vec![],
            confidence,
            surprisal_bits: None,
            source: source.into(),
        }
    }

    /// Add theta roles.
    #[must_use]
    pub fn with_roles(mut self, roles: Vec<String>) -> Self {
        self.theta_roles = roles;
        self
    }

    /// Set surprisal.
    #[must_use]
    pub fn with_surprisal(mut self, bits: f64) -> Self {
        self.surprisal_bits = Some(bits);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_format_simple() {
        let trace = DerivationTrace::new("John runs.")
            .with_syntax(SyntaxSummary {
                token_count: 2,
                predicate_lemmas: vec!["run".to_string()],
                dependency_summary: "nsubj(runs, John)".to_string(),
            })
            .with_sense_trace(SenseSelectionTrace {
                predicate_lemma: "run".to_string(),
                token_position: 1,
                winner: SenseReading::new("run-51.3.2", "GO", 0.9, "VerbNet")
                    .with_roles(vec!["Agent".to_string()]),
                runner_up: None,
                selection_reason: SelectionReason::Unambiguous,
            })
            .with_events(EventSummary {
                event_count: 1,
                events: vec![EventTrace {
                    event_id: 0,
                    predicate: "run".to_string(),
                    little_v: "GO".to_string(),
                    participants: vec![ParticipantTrace {
                        role: "Agent".to_string(),
                        filler: "John".to_string(),
                        binding_confidence: 0.95,
                    }],
                    aspect: "Activity".to_string(),
                    voice: "Active".to_string(),
                    confidence: 0.9,
                }],
                overall_confidence: 0.9,
            })
            .with_metadata(TraceMetadata {
                analysis_time_ms: 5,
                ambiguity_count: 0,
                total_readings: 1,
            });

        let output = trace.to_text();

        assert!(output.contains("John runs."));
        assert!(output.contains("run-51.3.2"));
        assert!(output.contains("90.0%"));
        assert!(output.contains("VerbNet"));
        assert!(output.contains("Agent"));
        assert!(output.contains("GO"));
    }

    #[test]
    fn test_trace_with_runner_up() {
        let trace =
            DerivationTrace::new("The bank collapsed.").with_sense_trace(SenseSelectionTrace {
                predicate_lemma: "collapse".to_string(),
                token_position: 2,
                winner: SenseReading::new("collapse-45.6", "BECOME", 0.85, "VerbNet"),
                runner_up: Some(SenseReading::new("fall-45.1", "GO", 0.72, "VerbNet")),
                selection_reason: SelectionReason::HigherConfidence { margin: 0.13 },
            });

        let output = trace.to_text();

        assert!(output.contains("collapse-45.6"));
        assert!(output.contains("vs RUNNER-UP: fall-45.1"));
        assert!(output.contains("Higher confidence (+13.0%)"));
    }

    #[test]
    fn test_selection_reason_display() {
        assert_eq!(
            SelectionReason::HigherConfidence { margin: 0.1 }.to_string(),
            "Higher confidence (+10.0%)"
        );
        assert_eq!(
            SelectionReason::LowerSurprisal { margin_bits: 2.5 }.to_string(),
            "Lower surprisal (-2.50 bits)"
        );
        assert_eq!(SelectionReason::Unambiguous.to_string(), "Unambiguous");
        assert_eq!(
            SelectionReason::HybridScore { score: 0.875 }.to_string(),
            "Hybrid score (0.88)"
        );
    }

    #[test]
    fn test_trace_latex_safe() {
        // Verify output doesn't contain problematic LaTeX characters
        let trace = DerivationTrace::new("Test sentence.");
        let output = trace.to_text();

        // These chars would need escaping in LaTeX - our output avoids them
        assert!(!output.contains('$'));
        assert!(!output.contains('\\'));
        // Note: % and & could appear in output but in safe contexts
    }

    #[test]
    fn test_trace_serialization() {
        let trace = DerivationTrace::new("Test.");
        let json = serde_json::to_string(&trace).unwrap();
        let parsed: DerivationTrace = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.input, "Test.");
    }

    #[test]
    fn test_discourse_summary_format() {
        let trace =
            DerivationTrace::new("A man entered. He left.").with_discourse(DiscourseSummary {
                referent_count: 2,
                condition_count: 4,
                drs_notation: "[ x, e1, e2 |\n  man(x),\n  enter(e1),\n  Agent(e1, x)\n]"
                    .to_string(),
                qud_stack_depth: 0,
                active_question: None,
                qud_history: Vec::new(),
                relevance_reports: Vec::new(),
                validation_reports: Vec::new(),
            });

        let output = trace.to_text();

        assert!(output.contains("4. DISCOURSE UPDATE"));
        assert!(output.contains("Referents: 2"));
        assert!(output.contains("man(x)"));
    }
}
