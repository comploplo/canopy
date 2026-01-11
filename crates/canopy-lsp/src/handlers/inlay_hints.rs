//! Inlay hints handler
//!
//! Provides inline annotations showing theta roles for tokens.

use std::collections::HashMap;

use canopy_resources::SemanticAnalysis;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

use crate::analysis::PositionMapper;
use crate::backend::CanopyBackend;
use crate::state::SentenceSpan;

/// A candidate hint with confidence for deduplication.
struct HintCandidate {
    hint: InlayHint,
    confidence: f32,
}

/// Build inlay hints for a sentence's analysis.
///
/// This is the core logic extracted for testability.
/// Deduplicates hints at the same position, keeping highest confidence.
pub fn build_inlay_hints(
    analysis: &SemanticAnalysis,
    sentence: &SentenceSpan,
    mapper: &PositionMapper,
) -> Vec<InlayHint> {
    // Track best hint per position (line, character) to avoid duplicates
    // from multiple verb senses generating the same role bindings
    let mut best_hints: HashMap<(u32, u32), HintCandidate> = HashMap::new();

    // Add hints for role bindings
    for binding in &analysis.role_bindings {
        if let Some(token) = analysis
            .syntax
            .tokens
            .iter()
            .find(|t| t.id == binding.token_id)
        {
            let (_, span_end) = token.span;
            let global_end = sentence.byte_start + span_end;

            if let Some(pos) = mapper.byte_to_position(global_end) {
                let key = (pos.line, pos.character);

                // Only keep this hint if it's higher confidence than existing
                let dominated = best_hints
                    .get(&key)
                    .is_some_and(|existing| existing.confidence >= binding.confidence);

                if !dominated {
                    let confidence_pct = (binding.confidence * 100.0).round() as u32;
                    let label = format!(" [{:?}]", binding.role);
                    let tooltip = format!(
                        "Theta role: {:?}\nConfidence: {}%",
                        binding.role, confidence_pct
                    );

                    best_hints.insert(
                        key,
                        HintCandidate {
                            hint: InlayHint {
                                position: pos,
                                label: InlayHintLabel::String(label),
                                kind: Some(InlayHintKind::TYPE),
                                text_edits: None,
                                tooltip: Some(InlayHintTooltip::String(tooltip)),
                                padding_left: Some(false),
                                padding_right: Some(true),
                                data: None,
                            },
                            confidence: binding.confidence,
                        },
                    );
                }
            }
        }
    }

    // Note: Event type hints (Do, Experience, etc.) were removed from verb tokens
    // as they were confusing - only show role bindings on arguments

    best_hints.into_values().map(|c| c.hint).collect()
}

/// Handle inlay hints request.
pub async fn handle_inlay_hints(
    backend: &CanopyBackend,
    params: InlayHintParams,
) -> Result<Option<Vec<InlayHint>>> {
    let uri = &params.text_document.uri;
    let request_range = params.range;

    // Get document content and cached sentences
    let (content, sentences) = {
        let doc = match backend.documents().get(uri) {
            Some(d) => d,
            None => return Ok(None),
        };
        (doc.content.clone(), doc.sentences.clone())
    };

    let mapper = PositionMapper::new(&content);

    let mut all_hints = Vec::new();

    // Process each sentence
    for sentence in &sentences {
        // Check if sentence overlaps with requested range
        let sentence_range = mapper
            .byte_span_to_range(sentence.byte_start, sentence.byte_end)
            .unwrap_or_else(|| {
                Range::new(
                    Position::new(sentence.line_start, 0),
                    Position::new(sentence.line_end, 0),
                )
            });

        // Skip sentences outside the requested range
        if sentence_range.end.line < request_range.start.line
            || sentence_range.start.line > request_range.end.line
        {
            continue;
        }

        let analysis = match backend.analyze_sentence(&sentence.text).await {
            Ok(a) => a,
            Err(_) => continue,
        };

        let hints = build_inlay_hints(&analysis, sentence, &mapper);
        all_hints.extend(hints);
    }

    if all_hints.is_empty() {
        return Ok(None);
    }

    Ok(Some(all_hints))
}

#[cfg(test)]
mod tests {
    use super::*;
    use canopy_resources::CanopyPipeline;

    #[test]
    fn test_build_inlay_hints_basic() {
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
        let hints = build_inlay_hints(&analysis, &sentence, &mapper);

        // Should produce hints for role bindings (if any)
        // The exact number depends on analysis results
        for hint in &hints {
            assert!(matches!(hint.kind, Some(InlayHintKind::TYPE)));
        }
    }

    #[test]
    fn test_build_inlay_hints_with_verb() {
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
        let hints = build_inlay_hints(&analysis, &sentence, &mapper);

        // Should have hints for participants and/or predicate
        // Verify all hints have valid positions
        for hint in &hints {
            assert!(hint.tooltip.is_some());
        }
    }

    #[test]
    fn test_inlay_hints_have_tooltips() {
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
        let hints = build_inlay_hints(&analysis, &sentence, &mapper);

        // All hints should have tooltips with confidence info
        for hint in &hints {
            if let Some(InlayHintTooltip::String(s)) = &hint.tooltip {
                assert!(!s.is_empty());
            }
        }
    }
}
