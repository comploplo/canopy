//! Pipeline module for end-to-end semantic analysis.
//!
//! The `CanopyPipeline` orchestrates the full analysis flow:
//! text → tokens → syntax → predicate decomposition → role binding → events → discourse

mod analysis;
mod config;
mod orchestrator;
mod trace_builder;

pub use analysis::{DocumentAnalysis, SemanticAnalysis, UnderspecifiedAnalysis};
pub use config::PipelineConfig;
pub use orchestrator::CanopyPipeline;
pub use trace_builder::TraceBuilder;
