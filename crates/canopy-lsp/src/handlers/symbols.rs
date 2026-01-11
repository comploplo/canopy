//! Document symbols handler
//!
//! Provides document outline showing events and entities.

use canopy_resources::SemanticAnalysis;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

use crate::analysis::PositionMapper;
use crate::backend::CanopyBackend;
use crate::state::SentenceSpan;

/// Handle document symbol request.
pub async fn handle_document_symbol(
    backend: &CanopyBackend,
    params: DocumentSymbolParams,
) -> Result<Option<DocumentSymbolResponse>> {
    let uri = &params.text_document.uri;

    // Get document content and cached sentences
    let (content, sentences) = {
        let doc = match backend.documents().get(uri) {
            Some(d) => d,
            None => return Ok(None),
        };
        (doc.content.clone(), doc.sentences.clone())
    };

    let mapper = PositionMapper::new(&content);

    let mut symbols = Vec::new();

    // Process each sentence using cached analysis
    for (sent_idx, sentence) in sentences.iter().enumerate() {
        let analysis = match backend.analyze_sentence(&sentence.text).await {
            Ok(a) => a,
            Err(_) => continue,
        };

        let sentence_range = mapper
            .byte_span_to_range(sentence.byte_start, sentence.byte_end)
            .unwrap_or_else(|| {
                Range::new(
                    Position::new(sentence.line_start, 0),
                    Position::new(sentence.line_end, 0),
                )
            });

        // Create symbols for events
        if let Some(events) = &analysis.events {
            for event in &events.events {
                // Find predicate token using token_span (more reliable than lemma matching)
                let predicate_range = analysis
                    .syntax
                    .tokens
                    .iter()
                    .find(|t| t.id == event.token_span.0)
                    .and_then(|t| {
                        let global_start = sentence.byte_start + t.span.0;
                        let global_end = sentence.byte_start + t.span.1;
                        mapper.byte_span_to_range(global_start, global_end)
                    })
                    .unwrap_or(sentence_range);

                // Build children for participants
                let children: Vec<DocumentSymbol> = event
                    .participants
                    .iter()
                    .filter_map(|(role, participant)| {
                        let token = analysis
                            .syntax
                            .tokens
                            .iter()
                            .find(|t| t.id == participant.token_id)?;

                        let global_start = sentence.byte_start + token.span.0;
                        let global_end = sentence.byte_start + token.span.1;
                        let range = mapper.byte_span_to_range(global_start, global_end)?;

                        #[allow(deprecated)]
                        Some(DocumentSymbol {
                            name: format!("{role:?}: \"{}\"", participant.text),
                            detail: Some(format!(
                                "{}% confidence",
                                (participant.confidence * 100.0).round() as u32
                            )),
                            kind: SymbolKind::VARIABLE,
                            tags: None,
                            deprecated: None,
                            range,
                            selection_range: range,
                            children: None,
                        })
                    })
                    .collect();

                #[allow(deprecated)]
                let event_symbol = DocumentSymbol {
                    name: format!("{} [{:?}]", event.predicate, event.little_v_type),
                    detail: Some(format!("{:?} {:?}", event.aspect, event.voice)),
                    kind: SymbolKind::EVENT,
                    tags: None,
                    deprecated: None,
                    range: sentence_range,
                    selection_range: predicate_range,
                    children: if children.is_empty() {
                        None
                    } else {
                        Some(children)
                    },
                };

                symbols.push(event_symbol);
            }
        }

        // If no events, create a sentence-level symbol
        if analysis.events.is_none()
            || analysis
                .events
                .as_ref()
                .is_some_and(|e| e.events.is_empty())
        {
            // Create symbol for sentence with discourse move if available
            let name = if let Some(dm) = &analysis.discourse_move {
                format!("Sentence {} [{:?}]", sent_idx + 1, dm)
            } else {
                format!("Sentence {}", sent_idx + 1)
            };

            #[allow(deprecated)]
            let sent_symbol = DocumentSymbol {
                name,
                detail: Some(sentence.text.chars().take(50).collect::<String>() + "..."),
                kind: SymbolKind::STRING,
                tags: None,
                deprecated: None,
                range: sentence_range,
                selection_range: sentence_range,
                children: None,
            };

            symbols.push(sent_symbol);
        }
    }

    if symbols.is_empty() {
        return Ok(None);
    }

    Ok(Some(DocumentSymbolResponse::Nested(symbols)))
}

/// Build document symbols for a single sentence's analysis.
///
/// This is the core logic extracted for testability.
#[allow(deprecated)]
pub fn build_sentence_symbols(
    analysis: &SemanticAnalysis,
    sentence: &SentenceSpan,
    sent_idx: usize,
    mapper: &PositionMapper,
) -> Vec<DocumentSymbol> {
    let mut symbols = Vec::new();

    let sentence_range = mapper
        .byte_span_to_range(sentence.byte_start, sentence.byte_end)
        .unwrap_or_else(|| {
            Range::new(
                Position::new(sentence.line_start, 0),
                Position::new(sentence.line_end, 0),
            )
        });

    // Create symbols for events
    if let Some(events) = &analysis.events {
        for event in &events.events {
            // Find predicate token using token_span (more reliable than lemma matching)
            let predicate_range = analysis
                .syntax
                .tokens
                .iter()
                .find(|t| t.id == event.token_span.0)
                .and_then(|t| {
                    let global_start = sentence.byte_start + t.span.0;
                    let global_end = sentence.byte_start + t.span.1;
                    mapper.byte_span_to_range(global_start, global_end)
                })
                .unwrap_or(sentence_range);

            // Build children for participants
            let children: Vec<DocumentSymbol> = event
                .participants
                .iter()
                .filter_map(|(role, participant)| {
                    let token = analysis
                        .syntax
                        .tokens
                        .iter()
                        .find(|t| t.id == participant.token_id)?;

                    let global_start = sentence.byte_start + token.span.0;
                    let global_end = sentence.byte_start + token.span.1;
                    let range = mapper.byte_span_to_range(global_start, global_end)?;

                    Some(DocumentSymbol {
                        name: format!("{role:?}: \"{}\"", participant.text),
                        detail: Some(format!(
                            "{}% confidence",
                            (participant.confidence * 100.0).round() as u32
                        )),
                        kind: SymbolKind::VARIABLE,
                        tags: None,
                        deprecated: None,
                        range,
                        selection_range: range,
                        children: None,
                    })
                })
                .collect();

            let event_symbol = DocumentSymbol {
                name: format!("{} [{:?}]", event.predicate, event.little_v_type),
                detail: Some(format!("{:?} {:?}", event.aspect, event.voice)),
                kind: SymbolKind::EVENT,
                tags: None,
                deprecated: None,
                range: sentence_range,
                selection_range: predicate_range,
                children: if children.is_empty() {
                    None
                } else {
                    Some(children)
                },
            };

            symbols.push(event_symbol);
        }
    }

    // If no events, create a sentence-level symbol
    if analysis.events.is_none()
        || analysis
            .events
            .as_ref()
            .is_some_and(|e| e.events.is_empty())
    {
        // Create symbol for sentence with discourse move if available
        let name = if let Some(dm) = &analysis.discourse_move {
            format!("Sentence {} [{:?}]", sent_idx + 1, dm)
        } else {
            format!("Sentence {}", sent_idx + 1)
        };

        let sent_symbol = DocumentSymbol {
            name,
            detail: Some(sentence.text.chars().take(50).collect::<String>() + "..."),
            kind: SymbolKind::STRING,
            tags: None,
            deprecated: None,
            range: sentence_range,
            selection_range: sentence_range,
            children: None,
        };

        symbols.push(sent_symbol);
    }

    symbols
}

#[cfg(test)]
mod tests {
    use super::*;
    use canopy_resources::CanopyPipeline;

    #[test]
    fn test_build_sentence_symbols_simple() {
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
        let symbols = build_sentence_symbols(&analysis, &sentence, 0, &mapper);

        // Should have at least one symbol (sentence or event)
        assert!(!symbols.is_empty(), "Expected at least one symbol");
    }

    #[test]
    fn test_build_sentence_symbols_with_verb() {
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
        let symbols = build_sentence_symbols(&analysis, &sentence, 0, &mapper);

        // Should have at least one symbol
        assert!(!symbols.is_empty());

        // Check that symbols have valid ranges
        for sym in &symbols {
            assert!(sym.range.start.line <= sym.range.end.line);
        }
    }

    #[test]
    fn test_symbols_have_correct_kind() {
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
        let symbols = build_sentence_symbols(&analysis, &sentence, 0, &mapper);

        // Symbols should be EVENT or STRING
        for sym in &symbols {
            assert!(
                sym.kind == SymbolKind::EVENT || sym.kind == SymbolKind::STRING,
                "Unexpected symbol kind: {:?}",
                sym.kind
            );
        }
    }
}
