//! Analysis utilities for bridging LSP and Canopy
//!
//! Handles document parsing, position mapping, and caching.

pub mod cache;
pub mod document;
pub mod mapper;

pub use cache::AnalysisCache;
pub use document::{extract_sentences, LanguageType};
pub use mapper::PositionMapper;
