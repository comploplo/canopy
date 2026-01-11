//! Semantic token encoder
//!
//! Encodes semantic tokens in the LSP delta format.

use tower_lsp::lsp_types::SemanticToken;

use super::legend::{ThetaTokenType, TokenModifier};

/// Encodes semantic tokens for LSP response.
#[derive(Debug, Default)]
pub struct SemanticTokenEncoder {
    tokens: Vec<EncodedToken>,
}

/// A token with position and type information.
#[derive(Debug, Clone)]
struct EncodedToken {
    line: u32,
    start_char: u32,
    length: u32,
    token_type: u32,
    token_modifiers: u32,
}

impl SemanticTokenEncoder {
    /// Create a new encoder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a token with the given position and type.
    pub fn push(
        &mut self,
        line: u32,
        start_char: u32,
        length: u32,
        token_type: ThetaTokenType,
        modifiers: &[TokenModifier],
    ) {
        let token_modifiers = modifiers.iter().fold(0, |acc, m| acc | m.bitmask());

        self.tokens.push(EncodedToken {
            line,
            start_char,
            length,
            token_type: token_type as u32,
            token_modifiers,
        });
    }

    /// Build the final semantic tokens in LSP delta format.
    ///
    /// The LSP format uses relative positions:
    /// - delta_line: line difference from previous token
    /// - delta_start: column difference (reset on new line)
    #[must_use]
    pub fn build(mut self) -> Vec<SemanticToken> {
        // Sort by position
        self.tokens
            .sort_by(|a, b| (a.line, a.start_char).cmp(&(b.line, b.start_char)));

        let mut result = Vec::with_capacity(self.tokens.len());
        let mut prev_line = 0u32;
        let mut prev_start = 0u32;

        for token in &self.tokens {
            let delta_line = token.line - prev_line;
            let delta_start = if delta_line == 0 {
                token.start_char - prev_start
            } else {
                token.start_char
            };

            result.push(SemanticToken {
                delta_line,
                delta_start,
                length: token.length,
                token_type: token.token_type,
                token_modifiers_bitset: token.token_modifiers,
            });

            prev_line = token.line;
            prev_start = token.start_char;
        }

        result
    }

    /// Get the number of tokens.
    #[must_use]
    pub fn len(&self) -> usize {
        self.tokens.len()
    }

    /// Check if there are no tokens.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encoder_single_token() {
        let mut encoder = SemanticTokenEncoder::new();
        encoder.push(0, 5, 4, ThetaTokenType::Agent, &[]);

        let tokens = encoder.build();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].delta_line, 0);
        assert_eq!(tokens[0].delta_start, 5);
        assert_eq!(tokens[0].length, 4);
        assert_eq!(tokens[0].token_type, ThetaTokenType::Agent as u32);
    }

    #[test]
    fn test_encoder_same_line() {
        let mut encoder = SemanticTokenEncoder::new();
        encoder.push(0, 0, 4, ThetaTokenType::Agent, &[]);
        encoder.push(0, 10, 4, ThetaTokenType::Theme, &[]);

        let tokens = encoder.build();
        assert_eq!(tokens.len(), 2);

        // First token: absolute position
        assert_eq!(tokens[0].delta_line, 0);
        assert_eq!(tokens[0].delta_start, 0);

        // Second token: relative to first
        assert_eq!(tokens[1].delta_line, 0);
        assert_eq!(tokens[1].delta_start, 10); // 10 - 0 = 10
    }

    #[test]
    fn test_encoder_different_lines() {
        let mut encoder = SemanticTokenEncoder::new();
        encoder.push(0, 5, 4, ThetaTokenType::Agent, &[]);
        encoder.push(2, 3, 4, ThetaTokenType::Theme, &[]);

        let tokens = encoder.build();
        assert_eq!(tokens.len(), 2);

        // First token
        assert_eq!(tokens[0].delta_line, 0);
        assert_eq!(tokens[0].delta_start, 5);

        // Second token: new line, column resets
        assert_eq!(tokens[1].delta_line, 2);
        assert_eq!(tokens[1].delta_start, 3); // Absolute on new line
    }

    #[test]
    fn test_encoder_with_modifiers() {
        let mut encoder = SemanticTokenEncoder::new();
        encoder.push(
            0,
            0,
            4,
            ThetaTokenType::Agent,
            &[TokenModifier::HighConfidence],
        );

        let tokens = encoder.build();
        assert_eq!(tokens[0].token_modifiers_bitset, 1);
    }

    #[test]
    fn test_encoder_multiple_modifiers() {
        let mut encoder = SemanticTokenEncoder::new();
        encoder.push(
            0,
            0,
            4,
            ThetaTokenType::Agent,
            &[TokenModifier::HighConfidence, TokenModifier::Ambiguous],
        );

        let tokens = encoder.build();
        // 1 (HighConfidence) | 4 (Ambiguous) = 5
        assert_eq!(tokens[0].token_modifiers_bitset, 5);
    }

    #[test]
    fn test_encoder_sorting() {
        let mut encoder = SemanticTokenEncoder::new();
        // Add in reverse order
        encoder.push(1, 0, 4, ThetaTokenType::Theme, &[]);
        encoder.push(0, 0, 4, ThetaTokenType::Agent, &[]);

        let tokens = encoder.build();

        // Should be sorted by position
        assert_eq!(tokens[0].delta_line, 0); // Line 0 first
        assert_eq!(tokens[1].delta_line, 1); // Then line 1
    }
}
