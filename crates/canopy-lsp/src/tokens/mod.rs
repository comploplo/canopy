//! Semantic token infrastructure
//!
//! Defines token types for theta roles and handles encoding.

pub mod encoder;
pub mod legend;

pub use encoder::SemanticTokenEncoder;
pub use legend::{semantic_token_legend, ThetaTokenType, TOKEN_MODIFIERS, TOKEN_TYPES};
