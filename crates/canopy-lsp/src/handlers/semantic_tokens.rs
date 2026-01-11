//! Semantic tokens handler
//!
//! Provides semantic highlighting based on theta roles.

use canopy::core::UPos;
use canopy_resources::SemanticAnalysis;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

use crate::analysis::PositionMapper;
use crate::backend::CanopyBackend;
use crate::state::SentenceSpan;
use crate::tokens::encoder::SemanticTokenEncoder;
use crate::tokens::legend::{ThetaTokenType, TokenModifier};

/// Build semantic tokens from analysis results.
///
/// This is the core logic extracted for testability.
pub fn build_semantic_tokens(
    analysis: &SemanticAnalysis,
    sentence: &SentenceSpan,
    mapper: &PositionMapper,
    encoder: &mut SemanticTokenEncoder,
) {
    // Add tokens for predicates (verbs)
    for decomp in &analysis.decompositions {
        let Some(token_id) = decomp.token_id else {
            continue;
        };
        if let Some(token) = analysis.syntax.tokens.iter().find(|t| t.id == token_id) {
            let (span_start, span_end) = token.span;
            let global_start = sentence.byte_start + span_start;
            let global_end = sentence.byte_start + span_end;

            if let Some(pos) = mapper.byte_to_position(global_start) {
                let length = (global_end - global_start) as u32;

                // Determine modifiers
                let mut modifiers = Vec::new();
                if decomp.confidence > 0.9 {
                    modifiers.push(TokenModifier::HighConfidence);
                } else if decomp.confidence < 0.7 {
                    modifiers.push(TokenModifier::LowConfidence);
                }

                encoder.push(
                    pos.line,
                    pos.character,
                    length,
                    ThetaTokenType::Predicate,
                    &modifiers,
                );
            }
        }
    }

    // Add tokens for role bindings
    for binding in &analysis.role_bindings {
        if let Some(token) = analysis
            .syntax
            .tokens
            .iter()
            .find(|t| t.id == binding.token_id)
        {
            let (span_start, span_end) = token.span;
            let global_start = sentence.byte_start + span_start;
            let global_end = sentence.byte_start + span_end;

            if let Some(pos) = mapper.byte_to_position(global_start) {
                let length = (global_end - global_start) as u32;

                // Determine token type from theta role
                let token_type = ThetaTokenType::from_theta_role(binding.role);

                // Determine modifiers
                let mut modifiers = Vec::new();
                if binding.confidence > 0.9 {
                    modifiers.push(TokenModifier::HighConfidence);
                } else if binding.confidence < 0.7 {
                    modifiers.push(TokenModifier::LowConfidence);
                }

                encoder.push(pos.line, pos.character, length, token_type, &modifiers);
            }
        }
    }

    // Add tokens for syntactic categories (auxiliaries, determiners, conjunctions)
    for token in &analysis.syntax.tokens {
        let token_type = match token.upos {
            UPos::Aux => ThetaTokenType::Auxiliary,
            UPos::Det => ThetaTokenType::Determiner,
            UPos::Cconj | UPos::Sconj => ThetaTokenType::Conjunction,
            _ => continue,
        };

        let (span_start, span_end) = token.span;
        let global_start = sentence.byte_start + span_start;
        let global_end = sentence.byte_start + span_end;

        if let Some(pos) = mapper.byte_to_position(global_start) {
            let length = (global_end - global_start) as u32;
            encoder.push(pos.line, pos.character, length, token_type, &[]);
        }
    }
}

/// Handle semantic tokens full request.
pub async fn handle_semantic_tokens_full(
    backend: &CanopyBackend,
    params: SemanticTokensParams,
) -> Result<Option<SemanticTokensResult>> {
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

    let mut encoder = SemanticTokenEncoder::new();

    // Process each sentence using cached analysis
    for sentence in &sentences {
        let analysis = match backend.analyze_sentence(&sentence.text).await {
            Ok(a) => a,
            Err(_) => continue,
        };

        build_semantic_tokens(&analysis, sentence, &mapper, &mut encoder);
    }

    if encoder.is_empty() {
        return Ok(None);
    }

    let tokens = encoder.build();

    Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
        result_id: None,
        data: tokens,
    })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use canopy_resources::CanopyPipeline;

    #[test]
    fn test_build_semantic_tokens_basic() {
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
        let mut encoder = SemanticTokenEncoder::new();

        build_semantic_tokens(&analysis, &sentence, &mapper, &mut encoder);

        // Should have produced at least one token (the verb "runs")
        assert!(!encoder.is_empty(), "Expected at least one semantic token");
    }

    #[test]
    fn test_build_semantic_tokens_verb_with_args() {
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
        let mut encoder = SemanticTokenEncoder::new();

        build_semantic_tokens(&analysis, &sentence, &mapper, &mut encoder);

        // Should have tokens for predicate and role bindings
        assert!(
            !encoder.is_empty(),
            "Expected tokens for predicate and/or arguments"
        );
    }

    #[test]
    fn test_build_semantic_tokens_empty_analysis() {
        let pipeline = match CanopyPipeline::new() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Skipping test: {e}");
                return;
            }
        };

        // Simple sentence without complex semantics
        let text = "Hello.";
        let analysis = pipeline.analyze(text).unwrap();

        let sentence = SentenceSpan {
            text: text.to_string(),
            byte_start: 0,
            byte_end: text.len(),
            line_start: 0,
            line_end: 0,
        };

        let mapper = PositionMapper::new(text);
        let mut encoder = SemanticTokenEncoder::new();

        build_semantic_tokens(&analysis, &sentence, &mapper, &mut encoder);

        // Should not panic, even if no tokens are produced
        let tokens = encoder.build();
        // Result can be empty or have tokens - just ensure no panic
        let _ = tokens;
    }
}
