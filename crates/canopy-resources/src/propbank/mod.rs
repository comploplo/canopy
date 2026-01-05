//! `PropBank` semantic role labeling engine
//!
//! Predicate-argument structures with semantic roles.

pub mod config;
pub mod engine;
pub mod parser;
pub mod types;

// Re-export main types
pub use config::PropBankConfig;
pub use engine::PropBankEngine;
pub use types::{
    ArgumentModifier, PropBankAnalysis, PropBankArgument, PropBankFrameset, PropBankPredicate,
    SemanticRole,
};
