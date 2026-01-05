//! Pipeline module for end-to-end semantic analysis.
//!
//! The `CanopyPipeline` orchestrates the full analysis flow:
//! text → tokens → syntax → predicate decomposition → role binding → events → discourse

mod analysis;
mod config;
mod orchestrator;

pub use analysis::{DocumentAnalysis, SemanticAnalysis, UnderspecifiedAnalysis};
pub use config::PipelineConfig;
pub use orchestrator::CanopyPipeline;
