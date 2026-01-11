//! Diagnostics handler
//!
//! Generates diagnostics from Canopy's semantic analysis.
//!
//! ## Diagnostic Codes
//!
//! | Code | Severity | Description |
//! |------|----------|-------------|
//! | `contradiction` | Warning | Logical contradiction detected |
//! | `presupposition-failure` | Info | Presupposition failure |
//! | `unbound-argument` | Hint | Argument without role binding |
//! | `low-confidence` | Info | Role binding below 50% confidence |
//! | `ambiguous-predicate` | Info | Multiple sense readings within 15% |
//! | `pronoun-ambiguous` | Info | Pronoun has multiple candidate antecedents |
//! | `pronoun-unresolved` | Warning | No accessible antecedent found |
//! | `binding-violation` | Hint | Binding theory constraint violation |
//! | `scope-ambiguous` | Info | Quantifier scope ambiguity |
//! | `conflict-detail` | Warning | Detailed conflict information |

use canopy::kernel::discourse::{BindingConstraint, ValidationStatus};
use canopy::kernel::trace::SelectionReason;
use canopy_resources::{ConflictType, SemanticAnalysis};
use tower_lsp::lsp_types::*;

use crate::analysis::PositionMapper;
use crate::backend::CanopyBackend;
use crate::state::SentenceSpan;

/// Generate diagnostics for a document.
pub async fn generate_diagnostics(backend: &CanopyBackend, uri: &Url) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    // Get document content and cached sentences
    let (content, sentences) = {
        let doc = match backend.documents().get(uri) {
            Some(d) => d,
            None => return diagnostics,
        };
        (doc.content.clone(), doc.sentences.clone())
    };

    let mapper = PositionMapper::new(&content);

    // Analyze each sentence using cached analysis
    for sentence in &sentences {
        let sentence_range = mapper
            .byte_span_to_range(sentence.byte_start, sentence.byte_end)
            .unwrap_or_else(|| {
                Range::new(
                    Position::new(sentence.line_start, 0),
                    Position::new(sentence.line_end, 0),
                )
            });

        let analysis = match backend.analyze_sentence(&sentence.text).await {
            Ok(a) => a,
            Err(e) => {
                // Analysis failed for this sentence
                diagnostics.push(Diagnostic {
                    range: sentence_range,
                    severity: Some(DiagnosticSeverity::WARNING),
                    source: Some("canopy".to_string()),
                    message: format!("Analysis failed: {e}"),
                    ..Default::default()
                });
                continue;
            }
        };

        // Check for validation issues
        for validation in &analysis.validations {
            match validation.status {
                ValidationStatus::Accepted => {}
                ValidationStatus::Contradiction => {
                    diagnostics.push(Diagnostic {
                        range: sentence_range,
                        severity: Some(DiagnosticSeverity::WARNING),
                        source: Some("canopy".to_string()),
                        message: validation
                            .message
                            .clone()
                            .unwrap_or_else(|| "Contradiction detected".to_string()),
                        code: Some(NumberOrString::String("contradiction".to_string())),
                        ..Default::default()
                    });
                }
                ValidationStatus::PresuppositionFailure => {
                    diagnostics.push(Diagnostic {
                        range: sentence_range,
                        severity: Some(DiagnosticSeverity::INFORMATION),
                        source: Some("canopy".to_string()),
                        message: validation
                            .message
                            .clone()
                            .unwrap_or_else(|| "Presupposition failure".to_string()),
                        code: Some(NumberOrString::String("presupposition-failure".to_string())),
                        ..Default::default()
                    });
                }
            }
        }

        // Check for unbound participants
        if let Some(events) = &analysis.events {
            for unbound in &events.unbound_participants {
                if let Some(token) = analysis
                    .syntax
                    .tokens
                    .iter()
                    .find(|t| t.id == unbound.token_id)
                {
                    let (span_start, span_end) = token.span;
                    let global_start = sentence.byte_start + span_start;
                    let global_end = sentence.byte_start + span_end;

                    let range = mapper
                        .byte_span_to_range(global_start, global_end)
                        .unwrap_or(sentence_range);

                    diagnostics.push(Diagnostic {
                        range,
                        severity: Some(DiagnosticSeverity::HINT),
                        source: Some("canopy".to_string()),
                        message: format!("Unbound argument: {:?}", unbound.reason),
                        code: Some(NumberOrString::String("unbound-argument".to_string())),
                        ..Default::default()
                    });
                }
            }
        }

        // Check for low-confidence role bindings
        for binding in &analysis.role_bindings {
            if binding.confidence < 0.5 {
                if let Some(token) = analysis
                    .syntax
                    .tokens
                    .iter()
                    .find(|t| t.id == binding.token_id)
                {
                    let (span_start, span_end) = token.span;
                    let global_start = sentence.byte_start + span_start;
                    let global_end = sentence.byte_start + span_end;

                    let range = mapper
                        .byte_span_to_range(global_start, global_end)
                        .unwrap_or(sentence_range);

                    let confidence_pct = (binding.confidence * 100.0).round() as u32;

                    diagnostics.push(Diagnostic {
                        range,
                        severity: Some(DiagnosticSeverity::INFORMATION),
                        source: Some("canopy".to_string()),
                        message: format!(
                            "Low confidence ({confidence_pct}%) for role {:?}",
                            binding.role
                        ),
                        code: Some(NumberOrString::String("low-confidence".to_string())),
                        ..Default::default()
                    });
                }
            }
        }

        // Check for ambiguous predicates (multiple decompositions with similar confidence)
        // Only warn when there's genuine ambiguity: top senses have similar confidence
        let high_confidence_decomps: Vec<_> = analysis
            .decompositions
            .iter()
            .filter(|d| d.confidence > 0.7 && d.token_id.is_some())
            .collect();

        if high_confidence_decomps.len() > 1 {
            // Group by token
            let token_ids: std::collections::HashSet<_> = high_confidence_decomps
                .iter()
                .filter_map(|d| d.token_id)
                .collect();

            for token_id in token_ids {
                let mut decomps_for_token: Vec<_> = high_confidence_decomps
                    .iter()
                    .filter(|d| d.token_id == Some(token_id))
                    .collect();

                // Sort by confidence descending
                decomps_for_token.sort_by(|a, b| {
                    b.confidence
                        .partial_cmp(&a.confidence)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });

                // Only warn if top 2 senses have similar confidence (within 15%)
                if decomps_for_token.len() >= 2 {
                    let top_conf = decomps_for_token[0].confidence;
                    let second_conf = decomps_for_token[1].confidence;

                    // Skip if there's a clear winner (top sense is significantly better)
                    if top_conf - second_conf > 0.15 {
                        continue;
                    }

                    if let Some(token) = analysis.syntax.tokens.iter().find(|t| t.id == token_id) {
                        let (span_start, span_end) = token.span;
                        let global_start = sentence.byte_start + span_start;
                        let global_end = sentence.byte_start + span_end;

                        let range = mapper
                            .byte_span_to_range(global_start, global_end)
                            .unwrap_or(sentence_range);

                        // Limit to top 3 senses
                        let senses: Vec<_> = decomps_for_token
                            .iter()
                            .take(3)
                            .map(|d| d.sense_id.to_string())
                            .collect();

                        let suffix = if decomps_for_token.len() > 3 {
                            format!(" (+{} more)", decomps_for_token.len() - 3)
                        } else {
                            String::new()
                        };

                        diagnostics.push(Diagnostic {
                            range,
                            severity: Some(DiagnosticSeverity::INFORMATION),
                            source: Some("canopy".to_string()),
                            message: format!(
                                "Ambiguous predicate: {}{}",
                                senses.join(", "),
                                suffix
                            ),
                            code: Some(NumberOrString::String("ambiguous-predicate".to_string())),
                            ..Default::default()
                        });
                    }
                }
            }
        }
    }

    diagnostics
}

/// Build diagnostics for a single sentence's analysis.
///
/// This is the core logic extracted for testability.
pub fn build_sentence_diagnostics(
    analysis: &SemanticAnalysis,
    sentence: &SentenceSpan,
    mapper: &PositionMapper,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    let sentence_range = mapper
        .byte_span_to_range(sentence.byte_start, sentence.byte_end)
        .unwrap_or_else(|| {
            Range::new(
                Position::new(sentence.line_start, 0),
                Position::new(sentence.line_end, 0),
            )
        });

    // Check for validation issues
    for validation in &analysis.validations {
        match validation.status {
            ValidationStatus::Accepted => {}
            ValidationStatus::Contradiction => {
                diagnostics.push(Diagnostic {
                    range: sentence_range,
                    severity: Some(DiagnosticSeverity::WARNING),
                    source: Some("canopy".to_string()),
                    message: validation
                        .message
                        .clone()
                        .unwrap_or_else(|| "Contradiction detected".to_string()),
                    code: Some(NumberOrString::String("contradiction".to_string())),
                    ..Default::default()
                });
            }
            ValidationStatus::PresuppositionFailure => {
                diagnostics.push(Diagnostic {
                    range: sentence_range,
                    severity: Some(DiagnosticSeverity::INFORMATION),
                    source: Some("canopy".to_string()),
                    message: validation
                        .message
                        .clone()
                        .unwrap_or_else(|| "Presupposition failure".to_string()),
                    code: Some(NumberOrString::String("presupposition-failure".to_string())),
                    ..Default::default()
                });
            }
        }
    }

    // Check for low-confidence role bindings
    for binding in &analysis.role_bindings {
        if binding.confidence < 0.5 {
            if let Some(token) = analysis
                .syntax
                .tokens
                .iter()
                .find(|t| t.id == binding.token_id)
            {
                let (span_start, span_end) = token.span;
                let global_start = sentence.byte_start + span_start;
                let global_end = sentence.byte_start + span_end;

                let range = mapper
                    .byte_span_to_range(global_start, global_end)
                    .unwrap_or(sentence_range);

                let confidence_pct = (binding.confidence * 100.0).round() as u32;

                diagnostics.push(Diagnostic {
                    range,
                    severity: Some(DiagnosticSeverity::INFORMATION),
                    source: Some("canopy".to_string()),
                    message: format!(
                        "Low confidence ({confidence_pct}%) for role {:?}",
                        binding.role
                    ),
                    code: Some(NumberOrString::String("low-confidence".to_string())),
                    ..Default::default()
                });
            }
        }
    }

    // Generate pronoun binding diagnostics
    diagnostics.extend(generate_pronoun_diagnostics(
        analysis,
        sentence,
        mapper,
        sentence_range,
    ));

    // Generate scope ambiguity diagnostics
    diagnostics.extend(generate_scope_diagnostics(
        analysis,
        sentence,
        mapper,
        sentence_range,
    ));

    // Generate enhanced conflict diagnostics
    diagnostics.extend(generate_conflict_diagnostics(
        analysis,
        sentence,
        mapper,
        sentence_range,
    ));

    // Generate sense ambiguity diagnostics from traces
    diagnostics.extend(generate_sense_trace_diagnostics(
        analysis,
        sentence,
        mapper,
        sentence_range,
    ));

    diagnostics
}

// =============================================================================
// New Diagnostic Generators
// =============================================================================

/// Generate diagnostics for pronoun binding ambiguity and violations.
fn generate_pronoun_diagnostics(
    analysis: &SemanticAnalysis,
    sentence: &SentenceSpan,
    mapper: &PositionMapper,
    sentence_range: Range,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for binding in &analysis.pronoun_bindings {
        // Find the token for this pronoun
        let range = analysis
            .syntax
            .tokens
            .iter()
            .find(|t| t.id == binding.token_id)
            .map_or(sentence_range, |t| {
                let (span_start, span_end) = t.span;
                let global_start = sentence.byte_start + span_start;
                let global_end = sentence.byte_start + span_end;
                mapper
                    .byte_span_to_range(global_start, global_end)
                    .unwrap_or(sentence_range)
            });

        // Check for ambiguous binding
        if binding.is_ambiguous && binding.candidates.len() > 1 {
            let candidates_str: Vec<_> = binding
                .candidates
                .iter()
                .take(3)
                .map(|c| format!("{} ({:.0}%)", c.text, c.confidence * 100.0))
                .collect();

            let suffix = if binding.candidates.len() > 3 {
                format!(" (+{} more)", binding.candidates.len() - 3)
            } else {
                String::new()
            };

            diagnostics.push(Diagnostic {
                range,
                severity: Some(DiagnosticSeverity::INFORMATION),
                source: Some("canopy".to_string()),
                message: format!(
                    "\"{}\" has multiple antecedents: {}{}",
                    binding.form,
                    candidates_str.join(", "),
                    suffix
                ),
                code: Some(NumberOrString::String("pronoun-ambiguous".to_string())),
                ..Default::default()
            });
        }

        // Check for unresolved binding
        if binding.resolved.is_none() && binding.candidates.is_empty() {
            diagnostics.push(Diagnostic {
                range,
                severity: Some(DiagnosticSeverity::WARNING),
                source: Some("canopy".to_string()),
                message: format!("\"{}\" has no accessible antecedent", binding.form),
                code: Some(NumberOrString::String("pronoun-unresolved".to_string())),
                ..Default::default()
            });
        }

        // Check for binding constraint violations
        for violation in &binding.violations {
            let message = format_binding_violation(*violation, &binding.form);
            diagnostics.push(Diagnostic {
                range,
                severity: Some(DiagnosticSeverity::HINT),
                source: Some("canopy".to_string()),
                message,
                code: Some(NumberOrString::String("binding-violation".to_string())),
                ..Default::default()
            });
        }
    }

    diagnostics
}

/// Format a binding constraint violation message.
fn format_binding_violation(violation: BindingConstraint, form: &str) -> String {
    match violation {
        BindingConstraint::ConditionA => {
            format!("Condition A: \"{form}\" (reflexive) must be locally bound")
        }
        BindingConstraint::ConditionB => {
            format!("Condition B: \"{form}\" (pronoun) must be free in local domain")
        }
        BindingConstraint::ConditionC => {
            format!("Condition C: R-expression \"{form}\" must be free")
        }
        BindingConstraint::GenderMismatch => format!("Gender mismatch for \"{form}\""),
        BindingConstraint::NumberMismatch => format!("Number mismatch for \"{form}\""),
        BindingConstraint::PersonMismatch => format!("Person mismatch for \"{form}\""),
        BindingConstraint::NoAccessibleAntecedent => {
            format!("No accessible antecedent for \"{form}\"")
        }
    }
}

/// Generate diagnostics for scope ambiguity.
fn generate_scope_diagnostics(
    analysis: &SemanticAnalysis,
    _sentence: &SentenceSpan,
    _mapper: &PositionMapper,
    sentence_range: Range,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for ambiguity in &analysis.scope_ambiguities {
        if ambiguity.ordering_count > 1 {
            let quantifier_desc: Vec<_> = ambiguity
                .quantifiers
                .iter()
                .map(|q| format!("{} ({})", q.text, q.quantifier_type))
                .collect();

            let readings_preview = if ambiguity.reading_descriptions.len() <= 2 {
                ambiguity.reading_descriptions.join(" vs ")
            } else {
                format!("{} readings possible", ambiguity.reading_descriptions.len())
            };

            diagnostics.push(Diagnostic {
                range: sentence_range, // Could refine to span of quantifiers
                severity: Some(DiagnosticSeverity::INFORMATION),
                source: Some("canopy".to_string()),
                message: format!(
                    "Scope ambiguity: {} ({}) - {}",
                    quantifier_desc.join(", "),
                    ambiguity.ordering_count,
                    readings_preview
                ),
                code: Some(NumberOrString::String("scope-ambiguous".to_string())),
                ..Default::default()
            });
        }
    }

    diagnostics
}

/// Generate enhanced diagnostics for logical conflicts.
fn generate_conflict_diagnostics(
    analysis: &SemanticAnalysis,
    _sentence: &SentenceSpan,
    _mapper: &PositionMapper,
    sentence_range: Range,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for conflict in &analysis.conflict_details {
        let conflict_type_str = match conflict.conflict_type {
            ConflictType::Polarity => "Polarity",
            ConflictType::Temporal => "Temporal",
            ConflictType::Modal => "Modal",
            ConflictType::FeatureAgreement => "Feature",
            ConflictType::TypeMismatch => "Type",
            ConflictType::Cardinality => "Cardinality",
        };

        diagnostics.push(Diagnostic {
            range: sentence_range,
            severity: Some(DiagnosticSeverity::WARNING),
            source: Some("canopy".to_string()),
            message: format!(
                "{} conflict: {} vs {} - {}",
                conflict_type_str, conflict.condition1, conflict.condition2, conflict.explanation
            ),
            code: Some(NumberOrString::String("conflict-detail".to_string())),
            ..Default::default()
        });
    }

    diagnostics
}

/// Generate diagnostics from sense selection traces.
fn generate_sense_trace_diagnostics(
    analysis: &SemanticAnalysis,
    sentence: &SentenceSpan,
    mapper: &PositionMapper,
    sentence_range: Range,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for trace in &analysis.sense_traces {
        // Only report if there's genuine ambiguity (runner-up exists and reason shows it)
        if let Some(ref runner_up) = trace.runner_up {
            // Check if this is actually ambiguous based on selection reason
            let is_ambiguous = match &trace.selection_reason {
                SelectionReason::HigherConfidence { margin } => *margin < 0.15,
                SelectionReason::Unambiguous => false,
                _ => true,
            };

            if is_ambiguous {
                // Find the token position
                let range =
                    analysis
                        .syntax
                        .tokens
                        .get(trace.token_position)
                        .map_or(sentence_range, |t| {
                            let (span_start, span_end) = t.span;
                            let global_start = sentence.byte_start + span_start;
                            let global_end = sentence.byte_start + span_end;
                            mapper
                                .byte_span_to_range(global_start, global_end)
                                .unwrap_or(sentence_range)
                        });

                let margin_str = match &trace.selection_reason {
                    SelectionReason::HigherConfidence { margin } => {
                        format!(" (margin: {:.1}%)", margin * 100.0)
                    }
                    _ => String::new(),
                };

                diagnostics.push(Diagnostic {
                    range,
                    severity: Some(DiagnosticSeverity::INFORMATION),
                    source: Some("canopy".to_string()),
                    message: format!(
                        "\"{}\" sense ambiguity: {} ({:.0}%) vs {} ({:.0}%){}",
                        trace.predicate_lemma,
                        trace.winner.sense_id,
                        trace.winner.confidence * 100.0,
                        runner_up.sense_id,
                        runner_up.confidence * 100.0,
                        margin_str
                    ),
                    code: Some(NumberOrString::String("ambiguous-predicate".to_string())),
                    ..Default::default()
                });
            }
        }
    }

    diagnostics
}

#[cfg(test)]
mod tests {
    use super::*;
    use canopy_resources::CanopyPipeline;

    #[test]
    fn test_build_sentence_diagnostics_clean() {
        let pipeline = match CanopyPipeline::new() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Skipping test: {e}");
                return;
            }
        };

        let text = "The cat runs.";
        let analysis = pipeline.analyze(text).unwrap();

        let sentence = SentenceSpan {
            text: text.to_string(),
            byte_start: 0,
            byte_end: text.len(),
            line_start: 0,
            line_end: 0,
        };

        let mapper = PositionMapper::new(text);
        let diagnostics = build_sentence_diagnostics(&analysis, &sentence, &mapper);

        // Simple sentences typically don't have contradictions
        let contradictions: Vec<_> = diagnostics
            .iter()
            .filter(|d| d.code == Some(NumberOrString::String("contradiction".to_string())))
            .collect();
        assert!(
            contradictions.is_empty(),
            "Simple sentence should not have contradictions"
        );
    }

    #[test]
    fn test_build_sentence_diagnostics_with_verb() {
        let pipeline = match CanopyPipeline::new() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Skipping test: {e}");
                return;
            }
        };

        let text = "John gives Mary a book.";
        let analysis = pipeline.analyze(text).unwrap();

        let sentence = SentenceSpan {
            text: text.to_string(),
            byte_start: 0,
            byte_end: text.len(),
            line_start: 0,
            line_end: 0,
        };

        let mapper = PositionMapper::new(text);
        let diagnostics = build_sentence_diagnostics(&analysis, &sentence, &mapper);

        // Should not panic, diagnostics can be empty or have entries
        for diag in &diagnostics {
            assert!(diag.source == Some("canopy".to_string()));
        }
    }

    #[test]
    fn test_diagnostics_have_correct_source() {
        let pipeline = match CanopyPipeline::new() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Skipping test: {e}");
                return;
            }
        };

        let text = "The dog chased the cat.";
        let analysis = pipeline.analyze(text).unwrap();

        let sentence = SentenceSpan {
            text: text.to_string(),
            byte_start: 0,
            byte_end: text.len(),
            line_start: 0,
            line_end: 0,
        };

        let mapper = PositionMapper::new(text);
        let diagnostics = build_sentence_diagnostics(&analysis, &sentence, &mapper);

        // All diagnostics should have "canopy" as source
        for diag in &diagnostics {
            assert_eq!(diag.source, Some("canopy".to_string()));
        }
    }

    // Tests for new diagnostic generators

    #[test]
    fn test_pronoun_diagnostics_empty_when_no_pronouns() {
        let pipeline = match CanopyPipeline::new() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Skipping test: {e}");
                return;
            }
        };

        let text = "The cat runs.";
        let analysis = pipeline.analyze(text).unwrap();

        let sentence = SentenceSpan {
            text: text.to_string(),
            byte_start: 0,
            byte_end: text.len(),
            line_start: 0,
            line_end: 0,
        };

        let mapper = PositionMapper::new(text);
        let sentence_range = Range::new(Position::new(0, 0), Position::new(0, text.len() as u32));

        let diags = generate_pronoun_diagnostics(&analysis, &sentence, &mapper, sentence_range);
        assert!(diags.is_empty(), "No pronouns, no pronoun diagnostics");
    }

    #[test]
    fn test_pronoun_diagnostics_with_mock_data() {
        use canopy::kernel::discourse::AnaphorType;
        use canopy::runtime::TokenId;
        use canopy_resources::{BindingCandidate, PronounBindingInfo};

        let pipeline = match CanopyPipeline::new() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Skipping test: {e}");
                return;
            }
        };

        let text = "John saw him.";
        let mut analysis = pipeline.analyze(text).unwrap();

        // Add mock pronoun binding info
        // Find "him" token
        if let Some(him_token) = analysis.syntax.tokens.iter().find(|t| t.form == "him") {
            analysis.pronoun_bindings.push(PronounBindingInfo {
                token_id: him_token.id,
                form: "him".to_string(),
                anaphor_type: AnaphorType::Personal,
                candidates: vec![
                    BindingCandidate {
                        text: "John".to_string(),
                        confidence: 0.8,
                        token_id: Some(TokenId(0)),
                        sentence_distance: 0,
                    },
                    BindingCandidate {
                        text: "someone".to_string(),
                        confidence: 0.5,
                        token_id: None,
                        sentence_distance: 1,
                    },
                ],
                violations: vec![],
                is_ambiguous: true,
                resolved: None,
            });
        }

        let sentence = SentenceSpan {
            text: text.to_string(),
            byte_start: 0,
            byte_end: text.len(),
            line_start: 0,
            line_end: 0,
        };

        let mapper = PositionMapper::new(text);
        let sentence_range = Range::new(Position::new(0, 0), Position::new(0, text.len() as u32));

        let diags = generate_pronoun_diagnostics(&analysis, &sentence, &mapper, sentence_range);

        // Should have ambiguous pronoun diagnostic
        let ambiguous: Vec<_> = diags
            .iter()
            .filter(|d| d.code == Some(NumberOrString::String("pronoun-ambiguous".to_string())))
            .collect();
        assert!(!ambiguous.is_empty(), "Should detect ambiguous pronoun");
        assert!(
            ambiguous[0].message.contains("him"),
            "Message should mention the pronoun"
        );
    }

    #[test]
    fn test_scope_diagnostics_empty_when_no_ambiguity() {
        let pipeline = match CanopyPipeline::new() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Skipping test: {e}");
                return;
            }
        };

        let text = "The cat runs.";
        let analysis = pipeline.analyze(text).unwrap();

        let sentence = SentenceSpan {
            text: text.to_string(),
            byte_start: 0,
            byte_end: text.len(),
            line_start: 0,
            line_end: 0,
        };

        let mapper = PositionMapper::new(text);
        let sentence_range = Range::new(Position::new(0, 0), Position::new(0, text.len() as u32));

        let diags = generate_scope_diagnostics(&analysis, &sentence, &mapper, sentence_range);
        assert!(diags.is_empty(), "No quantifiers, no scope ambiguity");
    }

    #[test]
    fn test_scope_diagnostics_with_mock_data() {
        use canopy_resources::{QuantifierInfo, ScopeAmbiguityInfo};

        let pipeline = match CanopyPipeline::new() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Skipping test: {e}");
                return;
            }
        };

        let text = "Every student read some book.";
        let mut analysis = pipeline.analyze(text).unwrap();

        // Add mock scope ambiguity
        analysis.scope_ambiguities.push(ScopeAmbiguityInfo {
            token_range: (0, 5),
            ordering_count: 2,
            quantifiers: vec![
                QuantifierInfo {
                    token_position: 0,
                    text: "every".to_string(),
                    quantifier_type: "universal".to_string(),
                },
                QuantifierInfo {
                    token_position: 3,
                    text: "some".to_string(),
                    quantifier_type: "existential".to_string(),
                },
            ],
            reading_descriptions: vec!["every > some".to_string(), "some > every".to_string()],
        });

        let sentence = SentenceSpan {
            text: text.to_string(),
            byte_start: 0,
            byte_end: text.len(),
            line_start: 0,
            line_end: 0,
        };

        let mapper = PositionMapper::new(text);
        let sentence_range = Range::new(Position::new(0, 0), Position::new(0, text.len() as u32));

        let diags = generate_scope_diagnostics(&analysis, &sentence, &mapper, sentence_range);

        assert_eq!(diags.len(), 1, "Should have one scope ambiguity diagnostic");
        assert_eq!(
            diags[0].code,
            Some(NumberOrString::String("scope-ambiguous".to_string()))
        );
        assert!(diags[0].message.contains("every"));
        assert!(diags[0].message.contains("some"));
    }

    #[test]
    fn test_conflict_diagnostics_empty_when_no_conflicts() {
        let pipeline = match CanopyPipeline::new() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Skipping test: {e}");
                return;
            }
        };

        let text = "The cat runs.";
        let analysis = pipeline.analyze(text).unwrap();

        let sentence = SentenceSpan {
            text: text.to_string(),
            byte_start: 0,
            byte_end: text.len(),
            line_start: 0,
            line_end: 0,
        };

        let mapper = PositionMapper::new(text);
        let sentence_range = Range::new(Position::new(0, 0), Position::new(0, text.len() as u32));

        let diags = generate_conflict_diagnostics(&analysis, &sentence, &mapper, sentence_range);
        assert!(diags.is_empty(), "No conflicts in simple sentence");
    }

    #[test]
    fn test_conflict_diagnostics_with_mock_data() {
        use canopy_resources::{ConflictDetail, ConflictType};

        let pipeline = match CanopyPipeline::new() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Skipping test: {e}");
                return;
            }
        };

        let text = "The cat is running and not running.";
        let mut analysis = pipeline.analyze(text).unwrap();

        // Add mock conflict
        analysis.conflict_details.push(ConflictDetail {
            conflict_type: ConflictType::Polarity,
            condition1: "running(e)".to_string(),
            condition2: "NOT running(e)".to_string(),
            token1_pos: Some(3),
            token2_pos: Some(6),
            explanation: "Direct contradiction".to_string(),
        });

        let sentence = SentenceSpan {
            text: text.to_string(),
            byte_start: 0,
            byte_end: text.len(),
            line_start: 0,
            line_end: 0,
        };

        let mapper = PositionMapper::new(text);
        let sentence_range = Range::new(Position::new(0, 0), Position::new(0, text.len() as u32));

        let diags = generate_conflict_diagnostics(&analysis, &sentence, &mapper, sentence_range);

        assert_eq!(diags.len(), 1, "Should have one conflict diagnostic");
        assert_eq!(
            diags[0].code,
            Some(NumberOrString::String("conflict-detail".to_string()))
        );
        assert!(diags[0].message.contains("Polarity"));
    }

    #[test]
    fn test_binding_violation_format() {
        assert!(
            format_binding_violation(BindingConstraint::ConditionA, "himself")
                .contains("Condition A")
        );
        assert!(
            format_binding_violation(BindingConstraint::ConditionB, "he").contains("Condition B")
        );
        assert!(
            format_binding_violation(BindingConstraint::ConditionC, "John").contains("Condition C")
        );
        assert!(
            format_binding_violation(BindingConstraint::GenderMismatch, "he").contains("Gender")
        );
        assert!(
            format_binding_violation(BindingConstraint::NumberMismatch, "they").contains("Number")
        );
        assert!(
            format_binding_violation(BindingConstraint::PersonMismatch, "I").contains("Person")
        );
    }
}
