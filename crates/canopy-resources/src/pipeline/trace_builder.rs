//! Trace builder for accumulating derivation data during analysis.
//!
//! `TraceBuilder` collects information at each step of the pipeline
//! and produces a `DerivationTrace` at the end.

use canopy::kernel::discourse::{Drs, QudReport, RelevanceReport, ValidationReport};
use canopy::kernel::events::ComposedEvents;
use canopy::kernel::trace::{
    DerivationTrace, DiscourseSummary, EventSummary, EventTrace, ParticipantTrace, QudHistoryEntry,
    RelevanceAlignmentTrace, RelevanceTraceEntry, SelectionReason, SenseReading,
    SenseSelectionTrace, SyntaxSummary, TraceMetadata, ValidationTraceEntry,
};
use canopy::runtime::{AnnotatedSyntax, PredicateDecomposition};
use canopy::DepRel;
use std::convert::TryFrom;
use std::time::Instant;

/// Builder that accumulates trace data during semantic analysis.
#[derive(Debug)]
pub struct TraceBuilder {
    /// Original input text
    input: String,
    /// When analysis started
    start_time: Instant,
    /// Accumulated syntax summary
    syntax_summary: Option<SyntaxSummary>,
    /// Accumulated sense selection traces
    sense_traces: Vec<SenseSelectionTrace>,
    /// Accumulated event summary
    event_summary: Option<EventSummary>,
    /// Accumulated discourse summary
    discourse_summary: Option<DiscourseSummary>,
    /// Number of predicates with multiple sense readings (choice points)
    ambiguity_count: usize,
    /// Total combinatorial readings (product of all choice points).
    /// For N predicates with M1, M2, ... MN senses each: M1 × M2 × ... × MN
    /// E.g., 3 predicates with 2 senses each = 2 × 2 × 2 = 8 readings.
    total_readings: usize,
}

impl TraceBuilder {
    /// Create a new trace builder for the given input text.
    #[must_use]
    pub fn new(input: &str) -> Self {
        Self {
            input: input.to_string(),
            start_time: Instant::now(),
            syntax_summary: None,
            sense_traces: Vec::new(),
            event_summary: None,
            discourse_summary: None,
            ambiguity_count: 0,
            total_readings: 1,
        }
    }

    /// Record the syntax parse result.
    pub fn record_syntax(&mut self, syntax: &AnnotatedSyntax) {
        let predicate_lemmas: Vec<String> = syntax.predicates().map(|t| t.lemma.clone()).collect();

        // Build dependency summary like "nsubj(runs, John), obj(gave, book)"
        let dependency_summary = Self::format_dependencies(syntax);

        self.syntax_summary = Some(SyntaxSummary {
            token_count: syntax.tokens.len(),
            predicate_lemmas,
            dependency_summary,
        });
    }

    /// Record combined syntax from multiple sentences.
    ///
    /// Accumulates token counts, predicate lemmas, and dependencies
    /// across all provided syntax trees.
    pub fn record_combined_syntax(&mut self, all_syntax: &[AnnotatedSyntax]) {
        if all_syntax.is_empty() {
            return;
        }

        let mut total_tokens = 0;
        let mut all_predicates = Vec::new();
        let mut all_deps = Vec::new();

        for syntax in all_syntax {
            total_tokens += syntax.tokens.len();
            all_predicates.extend(syntax.predicates().map(|t| t.lemma.clone()));
            let deps = Self::format_dependencies(syntax);
            if !deps.is_empty() {
                all_deps.push(deps);
            }
        }

        self.syntax_summary = Some(SyntaxSummary {
            token_count: total_tokens,
            predicate_lemmas: all_predicates,
            dependency_summary: all_deps.join("; "),
        });
    }

    /// Format dependencies for display.
    fn format_dependencies(syntax: &AnnotatedSyntax) -> String {
        let mut deps = Vec::new();

        for token in &syntax.tokens {
            if let Some(head_id) = token.head {
                if token.deprel != DepRel::Root && token.deprel != DepRel::Punct {
                    // Find head token
                    if let Some(head) = syntax.get_token(head_id) {
                        deps.push(format!(
                            "{}({}, {})",
                            format_deprel(&token.deprel),
                            head.form,
                            token.form
                        ));
                    }
                }
            }
        }

        deps.join(", ")
    }

    /// Record a sense selection decision.
    ///
    /// `all_decomps` should include all candidate decompositions before filtering.
    /// The winner is determined by highest confidence.
    pub fn record_sense_selection(
        &mut self,
        predicate_lemma: &str,
        token_position: usize,
        all_decomps: &[PredicateDecomposition],
    ) {
        if all_decomps.is_empty() {
            return;
        }

        // Sort by confidence to find winner and runner-up
        let mut sorted: Vec<_> = all_decomps.iter().collect();
        sorted.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let winner = sorted[0];
        let runner_up = sorted.get(1).copied();

        // Determine selection reason
        let selection_reason = if sorted.len() == 1 {
            SelectionReason::Unambiguous
        } else {
            let margin = winner.confidence - runner_up.map_or(0.0, |r| r.confidence);
            SelectionReason::HigherConfidence { margin }
        };

        let winner_reading = decomp_to_reading(winner);
        let runner_up_reading = runner_up.map(decomp_to_reading);

        self.sense_traces.push(SenseSelectionTrace {
            predicate_lemma: predicate_lemma.to_string(),
            token_position,
            winner: winner_reading,
            runner_up: runner_up_reading,
            selection_reason,
        });

        // Update ambiguity tracking
        if sorted.len() > 1 {
            self.ambiguity_count += 1;
            self.total_readings *= sorted.len();
        }
    }

    /// Record event composition result.
    pub fn record_event_composition(&mut self, events: &ComposedEvents) {
        let event_traces: Vec<EventTrace> = events
            .events
            .iter()
            .enumerate()
            .map(|(idx, event)| {
                let participants: Vec<ParticipantTrace> = event
                    .participants
                    .iter()
                    .map(|(role, participant)| ParticipantTrace {
                        role: format!("{role:?}"),
                        filler: participant.text.clone(),
                        binding_confidence: participant.confidence,
                    })
                    .collect();

                EventTrace {
                    event_id: idx,
                    predicate: event.predicate.clone(),
                    little_v: format!("{:?}", event.little_v_type),
                    participants,
                    aspect: format!("{:?}", event.aspect),
                    voice: format!("{:?}", event.voice),
                    confidence: event.overall_confidence(),
                }
            })
            .collect();

        self.event_summary = Some(EventSummary {
            event_count: events.events.len(),
            events: event_traces,
            overall_confidence: events.confidence,
        });
    }

    /// Record discourse update (DRS).
    pub fn record_discourse(
        &mut self,
        drs: &Drs,
        qud_report: QudReport,
        relevance_history: &[RelevanceReport],
        validation_history: &[ValidationReport],
    ) {
        let drs_notation = format_drs_notation(drs);
        let qud_history: Vec<QudHistoryEntry> = qud_report
            .history
            .into_iter()
            .map(|entry| {
                let action = entry.action_label().to_string();
                let origin = entry.origin_label().to_string();
                QudHistoryEntry {
                    issue_id: entry.issue_id,
                    question: entry.question,
                    action,
                    origin,
                }
            })
            .collect();
        let relevance_reports = relevance_history
            .iter()
            .map(|report| {
                let alignments = report
                    .alignments
                    .iter()
                    .map(|alignment| RelevanceAlignmentTrace {
                        event_id: alignment.event_id,
                        predicate: alignment.predicate.clone(),
                        level: format!("{:?}", alignment.level),
                        matched_roles: alignment
                            .matched_roles
                            .iter()
                            .map(|role| format!("{role:?}"))
                            .collect(),
                    })
                    .collect();
                RelevanceTraceEntry {
                    sentence_index: report.sentence_index,
                    question: report.question.clone(),
                    level: format!("{:?}", report.level),
                    alignments,
                }
            })
            .collect();

        let validation_reports = validation_history
            .iter()
            .map(|report| ValidationTraceEntry {
                sentence_index: report.sentence_index,
                predicate: report.predicate.clone(),
                status: format!("{:?}", report.status),
                message: report.message.clone(),
            })
            .collect();

        self.discourse_summary = Some(DiscourseSummary {
            referent_count: drs.universe.len(),
            condition_count: drs.conditions.len(),
            drs_notation,
            qud_stack_depth: qud_report.stack_depth,
            active_question: qud_report.active_question,
            qud_history,
            relevance_reports,
            validation_reports,
        });
    }

    /// Build the final derivation trace.
    #[must_use]
    pub fn build(self) -> DerivationTrace {
        let elapsed = self.start_time.elapsed();
        let analysis_time_ms = u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX);

        DerivationTrace {
            input: self.input,
            syntax_summary: self.syntax_summary.unwrap_or_else(|| SyntaxSummary {
                token_count: 0,
                predicate_lemmas: vec![],
                dependency_summary: String::new(),
            }),
            sense_traces: self.sense_traces,
            event_summary: self.event_summary.unwrap_or_else(|| EventSummary {
                event_count: 0,
                events: vec![],
                overall_confidence: 0.0,
            }),
            discourse_summary: self.discourse_summary,
            metadata: TraceMetadata {
                analysis_time_ms,
                ambiguity_count: self.ambiguity_count,
                total_readings: self.total_readings,
            },
        }
    }
}

/// Convert a `PredicateDecomposition` to a `SenseReading`.
fn decomp_to_reading(decomp: &PredicateDecomposition) -> SenseReading {
    let theta_roles: Vec<String> = decomp
        .expected_roles
        .iter()
        .map(|r| format!("{r:?}"))
        .collect();

    SenseReading {
        sense_id: decomp.sense_id.to_string(),
        little_v_type: format!("{:?}", decomp.little_v_type),
        theta_roles,
        confidence: decomp.confidence,
        surprisal_bits: None, // Not computed in basic analysis
        source: format!("{:?}", decomp.source),
    }
}

/// Format a dependency relation for display.
fn format_deprel(deprel: &DepRel) -> String {
    match deprel {
        DepRel::Nsubj => "nsubj".to_string(),
        DepRel::Obj => "obj".to_string(),
        DepRel::Iobj => "iobj".to_string(),
        DepRel::Obl => "obl".to_string(),
        DepRel::Advmod => "advmod".to_string(),
        DepRel::Amod => "amod".to_string(),
        DepRel::Det => "det".to_string(),
        DepRel::Case => "case".to_string(),
        DepRel::Nmod => "nmod".to_string(),
        DepRel::Compound => "compound".to_string(),
        DepRel::Mark => "mark".to_string(),
        DepRel::Aux => "aux".to_string(),
        DepRel::Cop => "cop".to_string(),
        DepRel::Conj => "conj".to_string(),
        DepRel::Cc => "cc".to_string(),
        DepRel::NsubjPass => "nsubj:pass".to_string(),
        DepRel::AuxPass => "aux:pass".to_string(),
        DepRel::Csubj => "csubj".to_string(),
        DepRel::CsubjPass => "csubj:pass".to_string(),
        DepRel::Ccomp => "ccomp".to_string(),
        DepRel::Xcomp => "xcomp".to_string(),
        DepRel::Advcl => "advcl".to_string(),
        DepRel::Acl => "acl".to_string(),
        DepRel::Appos => "appos".to_string(),
        DepRel::Nummod => "nummod".to_string(),
        DepRel::Punct => "punct".to_string(),
        DepRel::Root => "root".to_string(),
        DepRel::Dep => "dep".to_string(),
        DepRel::Parataxis => "parataxis".to_string(),
        DepRel::Discourse => "discourse".to_string(),
        DepRel::Expl => "expl".to_string(),
        DepRel::Fixed => "fixed".to_string(),
        DepRel::Flat => "flat".to_string(),
        DepRel::Goeswith => "goeswith".to_string(),
        DepRel::List => "list".to_string(),
        DepRel::Orphan => "orphan".to_string(),
        DepRel::Reparandum => "reparandum".to_string(),
        DepRel::Vocative => "vocative".to_string(),
        DepRel::Dislocated => "dislocated".to_string(),
        DepRel::Clf => "clf".to_string(),
        DepRel::Neg => "neg".to_string(),
        DepRel::Other(s) => s.clone(),
    }
}

/// Format DRS in box notation.
fn format_drs_notation(drs: &Drs) -> String {
    let mut out = String::new();

    // Referents
    let referents: Vec<String> = drs.universe.keys().map(|id| format!("x{}", id.0)).collect();

    out.push_str("[ ");
    out.push_str(&referents.join(", "));
    out.push_str(" |\n");

    // Conditions
    for (i, condition) in drs.conditions.iter().enumerate() {
        let cond_str = format_condition(condition);
        out.push_str("  ");
        out.push_str(&cond_str);
        if i < drs.conditions.len() - 1 {
            out.push(',');
        }
        out.push('\n');
    }

    out.push(']');
    out
}

/// Format a single DRS condition.
fn format_condition(condition: &canopy::kernel::discourse::DrsCondition) -> String {
    use canopy::kernel::discourse::DrsCondition;

    match condition {
        DrsCondition::Predicate { name, referent } => {
            format!("{}(x{})", name, referent.0)
        }
        DrsCondition::Relation { name, arg1, arg2 } => {
            format!("{}(x{}, x{})", name, arg1.0, arg2.0)
        }
        DrsCondition::EventPredicate {
            event_id,
            predicate,
            participants,
        } => {
            let args: Vec<String> = participants
                .iter()
                .map(|(role, id)| format!("{:?}=x{}", role, id.0))
                .collect();
            format!("{}(e{}, {})", predicate, event_id.0, args.join(", "))
        }
        DrsCondition::ThetaRole {
            event_id,
            role,
            filler,
        } => {
            format!("{:?}(e{}, x{})", role, event_id.0, filler.0)
        }
        DrsCondition::Equality { ref1, ref2 } => {
            format!("x{} = x{}", ref1.0, ref2.0)
        }
        DrsCondition::Negation(inner_drs) => {
            format!("NOT({})", format_drs_compact(inner_drs))
        }
        DrsCondition::Disjunction(drs1, drs2) => {
            format!(
                "OR({}, {})",
                format_drs_compact(drs1),
                format_drs_compact(drs2)
            )
        }
        DrsCondition::Implication {
            antecedent,
            consequent,
        } => {
            format!(
                "IF {} THEN {}",
                format_drs_compact(antecedent),
                format_drs_compact(consequent)
            )
        }
        DrsCondition::TemporalRelation {
            relation,
            event1,
            event2,
        } => {
            format!("{:?}(e{}, e{})", relation, event1.0, event2.0)
        }
    }
}

/// Format a DRS in compact notation for nested structures.
fn format_drs_compact(drs: &canopy::kernel::discourse::Drs) -> String {
    let refs: Vec<String> = drs.universe.iter().map(|r| format!("x{}", r.0)).collect();
    let conds: Vec<String> = drs.conditions.iter().map(format_condition).collect();
    format!("[{} | {}]", refs.join(","), conds.join("; "))
}

#[cfg(test)]
mod tests {
    use super::*;
    use canopy::runtime::AnnotatedToken;
    use canopy::TokenId;

    #[test]
    fn test_trace_builder_basic() {
        let mut builder = TraceBuilder::new("John runs.");

        let syntax = AnnotatedSyntax::new(
            "John runs.".to_string(),
            vec![
                AnnotatedToken::new(
                    TokenId::new(0),
                    "John".to_string(),
                    "john".to_string(),
                    canopy::UPos::Propn,
                    DepRel::Nsubj,
                    (0, 4),
                ),
                AnnotatedToken::new(
                    TokenId::new(1),
                    "runs".to_string(),
                    "run".to_string(),
                    canopy::UPos::Verb,
                    DepRel::Root,
                    (5, 9),
                ),
            ],
        );

        builder.record_syntax(&syntax);

        let trace = builder.build();
        assert_eq!(trace.input, "John runs.");
        assert_eq!(trace.syntax_summary.token_count, 2);
        assert!(trace.metadata.analysis_time_ms < u64::MAX);
    }

    #[test]
    fn test_sense_selection_trace() {
        use canopy::kernel::events::LittleVType;
        use canopy::runtime::{DecompositionSource, SenseId};

        let mut builder = TraceBuilder::new("Test");

        let decomps = vec![
            PredicateDecomposition::new(SenseId::new("run-51.3.2"), LittleVType::Go, vec![])
                .with_confidence(0.9)
                .with_source(DecompositionSource::VerbNet),
            PredicateDecomposition::new(SenseId::new("run-51.1"), LittleVType::Do, vec![])
                .with_confidence(0.7)
                .with_source(DecompositionSource::VerbNet),
        ];

        builder.record_sense_selection("run", 1, &decomps);

        let trace = builder.build();
        assert_eq!(trace.sense_traces.len(), 1);
        assert_eq!(trace.sense_traces[0].winner.sense_id, "run-51.3.2");
        assert!(trace.sense_traces[0].runner_up.is_some());
        assert_eq!(trace.metadata.ambiguity_count, 1);
        assert_eq!(trace.metadata.total_readings, 2);
    }
}
