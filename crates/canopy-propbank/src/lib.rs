//! # Canopy PropBank Engine
//!
//! This crate provides a semantic role labeling (SRL) engine based on PropBank annotations.
//! PropBank provides predicate-argument structures with semantic roles for natural language understanding.
//!
//! ## Overview
//!
//! PropBank annotates predicates (typically verbs) with their arguments and semantic roles:
//! - **ARG0**: Agent (typically the subject performing the action)
//! - **ARG1**: Patient/Theme (typically the direct object or thing being acted upon)
//! - **ARG2**: Indirect object, beneficiary, or instrument
//! - **ARG3**: Starting point, beneficiary, attribute
//! - **ARG4**: Ending point
//! - **ARGM-***: Modifiers (location, time, manner, purpose, etc.)
//!
//! ## Features
//!
//! - Fast predicate-argument lookup
//! - Support for PropBank framesets (e.g., "give.01", "take.02")
//! - Semantic role classification
//! - Integration with BaseEngine architecture
//! - Caching for performance optimization
//!
//! ## Usage
//!
//! ```rust,no_run
//! use canopy_propbank::{PropBankEngine, PropBankConfig};
//! use canopy_engine::EngineResult;
//!
//! # fn main() -> EngineResult<()> {
//! let engine = PropBankEngine::new()?;
//! let result = engine.analyze_predicate("give", "01")?;
//!
//! println!("Lemma: {}", result.data.lemma);
//! println!("Roleset: {}", result.data.roleset);
//! for arg in &result.data.arguments {
//!     println!("  {:?}: {}", arg.role, arg.description);
//! }
//! # Ok(())
//! # }
//! ```

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

// Re-export engine errors
pub use canopy_engine::EngineResult;
