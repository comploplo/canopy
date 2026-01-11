//! Hover handler
//!
//! Provides semantic information when hovering over text.
//!
//! ## Hover Content
//!
//! The hover shows rich semantic information for tokens:
//!
//! - **Basic info**: Form, lemma, POS, dependency relation
//! - **Semantic roles**: Theta role bindings with confidence
//! - **Predicate analysis**: Sense, event type, expected roles
//! - **Event structure**: Full event decomposition
//! - **Pronoun binding**: Antecedent candidates for pronouns
//! - **Logical form**: DRS notation for predicates
//! - **Sense derivation**: Why this sense was selected

use canopy::runtime::AnnotatedToken;
use canopy::UPos;
use canopy_resources::SemanticAnalysis;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

use crate::analysis::PositionMapper;
use crate::backend::CanopyBackend;
use crate::state::SentenceSpan;

/// Build hover content for a token.
///
/// This is the core logic extracted for testability.
pub fn build_hover_content(analysis: &SemanticAnalysis, token: &AnnotatedToken) -> Vec<String> {
    let mut lines = Vec::new();

    // Token form and lemma
    lines.push(format!("**\"{}\"**", token.form));
    if token.lemma != token.form {
        lines.push(format!("Lemma: {}", token.lemma));
    }

    // POS tag
    lines.push(format!("POS: {:?}", token.upos));

    // Dependency relation
    lines.push(format!("Dependency: {:?}", token.deprel));

    // Theta role bindings for this token
    let bindings: Vec<_> = analysis
        .role_bindings
        .iter()
        .filter(|b| b.token_id == token.id)
        .collect();

    if !bindings.is_empty() {
        lines.push(String::new());
        lines.push("**Semantic Roles:**".to_string());
        for binding in bindings {
            let confidence_pct = (binding.confidence * 100.0).round() as u32;
            lines.push(format!(
                "- {:?} ({}% confidence)",
                binding.role, confidence_pct
            ));
        }
    }

    // If this is a predicate, show decomposition
    let decomp = analysis
        .decompositions
        .iter()
        .find(|d| d.token_id == Some(token.id));

    if let Some(d) = decomp {
        lines.push(String::new());
        lines.push("**Predicate Analysis:**".to_string());
        lines.push(format!("- Sense: {}", d.sense_id));
        lines.push(format!("- Event type: {:?}", d.little_v_type));

        if !d.expected_roles.is_empty() {
            let roles: Vec<_> = d.expected_roles.iter().map(|r| format!("{r:?}")).collect();
            lines.push(format!("- Expected roles: {}", roles.join(", ")));
        }

        let confidence_pct = (d.confidence * 100.0).round() as u32;
        lines.push(format!("- Confidence: {confidence_pct}%"));
    }

    // Event structure if available
    if let Some(events) = &analysis.events {
        for event in &events.events {
            // Check if this token is the predicate of an event
            if event.predicate == token.lemma {
                lines.push(String::new());
                lines.push("**Event Structure:**".to_string());
                lines.push(format!("- Predicate: {}", event.predicate));
                lines.push(format!("- Type: {:?}", event.little_v_type));
                lines.push(format!("- Aspect: {:?}", event.aspect));
                lines.push(format!("- Voice: {:?}", event.voice));

                if !event.participants.is_empty() {
                    lines.push("- Participants:".to_string());
                    for (role, participant) in &event.participants {
                        lines.push(format!("  - {:?}: \"{}\"", role, participant.text));
                    }
                }
            }
        }
    }

    // Pronoun binding information (for pronouns)
    if is_pronoun_pos(token.upos) {
        if let Some(binding) = analysis
            .pronoun_bindings
            .iter()
            .find(|b| b.token_id == token.id)
        {
            lines.push(String::new());
            lines.push("**Pronoun Binding:**".to_string());

            if let Some(ref resolved) = binding.resolved {
                lines.push(format!("- Resolved to: {resolved}"));
            } else if binding.is_ambiguous {
                lines.push("- Ambiguous binding".to_string());
            } else {
                lines.push("- Unresolved".to_string());
            }

            if !binding.candidates.is_empty() {
                lines.push("- Candidates:".to_string());
                for candidate in binding.candidates.iter().take(5) {
                    let dist_str = if candidate.sentence_distance == 0 {
                        String::new()
                    } else {
                        format!(", {} sent. back", candidate.sentence_distance)
                    };
                    lines.push(format!(
                        "  - {} ({:.0}%{})",
                        candidate.text,
                        candidate.confidence * 100.0,
                        dist_str
                    ));
                }
            }

            if !binding.violations.is_empty() {
                lines.push("- Violations:".to_string());
                for violation in &binding.violations {
                    lines.push(format!("  - {violation:?}"));
                }
            }
        }
    }

    // Sense selection trace (for predicates)
    if let Some(trace) = analysis
        .sense_traces
        .iter()
        .find(|t| t.token_position == token.id.0)
    {
        lines.push(String::new());
        lines.push("**Sense Derivation:**".to_string());
        lines.push(format!(
            "- Selected: {} ({:.0}%)",
            trace.winner.sense_id,
            trace.winner.confidence * 100.0
        ));
        lines.push(format!("- Source: {}", trace.winner.source));

        if let Some(ref runner_up) = trace.runner_up {
            lines.push(format!(
                "- Runner-up: {} ({:.0}%)",
                runner_up.sense_id,
                runner_up.confidence * 100.0
            ));
        }

        lines.push(format!("- Reason: {}", trace.selection_reason));
    }

    // DRS logical form (for predicates)
    if let Some(ref drs) = analysis.sentence_drs {
        // Only show for predicates/verbs
        if matches!(token.upos, UPos::Verb | UPos::Aux) {
            // Find conditions related to this predicate
            let related_conditions: Vec<_> = drs
                .conditions
                .iter()
                .filter(|c| {
                    let cond_str = c.to_string();
                    cond_str.contains(&token.lemma)
                })
                .take(5)
                .collect();

            if !related_conditions.is_empty() {
                lines.push(String::new());
                lines.push("**Logical Form:**".to_string());
                lines.push("```".to_string());
                for cond in related_conditions {
                    lines.push(format!("  {cond}"));
                }
                lines.push("```".to_string());
            }
        }
    }

    lines
}

/// Check if a POS tag indicates a pronoun.
fn is_pronoun_pos(pos: UPos) -> bool {
    matches!(pos, UPos::Pron)
}

/// Handle hover request.
pub async fn handle_hover(backend: &CanopyBackend, params: HoverParams) -> Result<Option<Hover>> {
    let uri = &params.text_document_position_params.text_document.uri;
    let position = params.text_document_position_params.position;

    // Get document content and cached sentences
    let (content, sentences) = {
        let doc = match backend.documents().get(uri) {
            Some(d) => d,
            None => return Ok(None),
        };
        (doc.content.clone(), doc.sentences.clone())
    };

    let mapper = PositionMapper::new(&content);

    // Convert position to byte offset
    let byte_offset = match mapper.position_to_byte(position) {
        Some(offset) => offset,
        None => return Ok(None),
    };

    // Find the sentence containing this position (use cached sentences)
    let sentence: &SentenceSpan = match sentences
        .iter()
        .find(|s| byte_offset >= s.byte_start && byte_offset < s.byte_end)
    {
        Some(s) => s,
        None => return Ok(None),
    };

    // Analyze the sentence using cache
    let analysis = match backend.analyze_sentence(&sentence.text).await {
        Ok(a) => a,
        Err(_) => return Ok(None),
    };

    // Find token at position within sentence
    let offset_in_sentence = byte_offset - sentence.byte_start;

    // Find which token contains this offset
    let token = match analysis.syntax.tokens.iter().find(|t| {
        let (start, end) = t.span;
        offset_in_sentence >= start && offset_in_sentence < end
    }) {
        Some(t) => t,
        None => return Ok(None),
    };

    // Build hover content using extracted function
    let lines = build_hover_content(&analysis, token);
    let content = lines.join("\n");

    Ok(Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: content,
        }),
        range: mapper.byte_span_to_range(
            sentence.byte_start + token.span.0,
            sentence.byte_start + token.span.1,
        ),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use canopy_resources::CanopyPipeline;

    #[test]
    fn test_build_hover_content_basic() {
        let pipeline = match CanopyPipeline::new() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Skipping test: {e}");
                return;
            }
        };

        let analysis = pipeline.analyze("The cat runs.").unwrap();

        // Find a token (e.g., "cat")
        let token = analysis
            .syntax
            .tokens
            .iter()
            .find(|t| t.form == "cat")
            .unwrap();

        let lines = build_hover_content(&analysis, token);

        // Should have basic token info
        assert!(lines.iter().any(|l| l.contains("cat")));
        assert!(lines.iter().any(|l| l.contains("POS:")));
        assert!(lines.iter().any(|l| l.contains("Dependency:")));
    }

    #[test]
    fn test_build_hover_content_verb() {
        let pipeline = match CanopyPipeline::new() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Skipping test: {e}");
                return;
            }
        };

        let analysis = pipeline.analyze("John gives Mary a book.").unwrap();

        // Find the verb
        let token = analysis
            .syntax
            .tokens
            .iter()
            .find(|t| t.form == "gives")
            .unwrap();

        let lines = build_hover_content(&analysis, token);

        // Should have verb-specific info
        assert!(lines.iter().any(|l| l.contains("gives")));
    }

    #[test]
    fn test_build_hover_content_with_roles() {
        let pipeline = match CanopyPipeline::new() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Skipping test: {e}");
                return;
            }
        };

        let analysis = pipeline.analyze("The dog chased the cat.").unwrap();

        // Check that we can build content for each token
        for token in &analysis.syntax.tokens {
            let lines = build_hover_content(&analysis, token);
            // Every token should have at least form and POS
            assert!(!lines.is_empty());
            assert!(lines[0].contains(&token.form));
        }
    }
}
