//! Engine infrastructure for semantic analysis
//!
//! This module provides common traits, caching mechanisms, and utilities
//! shared across all semantic engines (`VerbNet`, `FrameNet`, `WordNet`, `PropBank`, Lexicon).

/// Convert usize to f32 for confidence calculations (saturates at `u16::MAX` for lossless f32).
/// Use for small counts where values are typically < 1000.
#[inline]
#[must_use]
pub fn count_to_f32(n: usize) -> f32 {
    f32::from(u16::try_from(n).unwrap_or(u16::MAX))
}

/// Convert i32 to f32 for confidence calculations (saturates to u16 range for lossless f32).
#[inline]
#[must_use]
pub fn i32_count_to_f32(n: i32) -> f32 {
    f32::from(u16::try_from(n.max(0)).unwrap_or(u16::MAX))
}

/// Convert u128 microseconds to u64 (saturates at `u64::MAX`).
/// Use for timing values from `Duration::as_micros()`.
#[inline]
#[must_use]
pub fn micros_to_u64(n: u128) -> u64 {
    u64::try_from(n).unwrap_or(u64::MAX)
}

pub mod base;
pub mod cache;
pub mod cacheable;
pub mod conllu;
pub mod data_loader;
pub mod error;
pub mod macros;
pub mod mappings;
pub mod pos_tags;
pub mod shared;
pub mod stats;
pub mod traits;
pub mod xml;

// Re-export main types for convenience
pub use base::{BaseEngine, CacheKeyFormat, ConfidenceCalculator, EngineCore, QualityMetrics};
pub use cache::{CacheKey, CacheStats, EngineCache};
pub use cacheable::CacheableData;
pub use conllu::{ConlluParser, ConlluParserConfig, ConlluSentence, ConlluToken};
pub use data_loader::{CommonDataLoader, DataLoaderBuilder, DataLoaderConfig, LoadingStats};
pub use error::{EngineError, EngineResult, ErrorCategory};
pub use mappings::{DepRelToThetaMap, PredicateToLittleVMap};
pub use shared::SharedEngines;
pub use stats::{DataStats, EngineStats, PerformanceMetrics};
pub use traits::{
    CachedEngine, DataInfo, DataLoader, EngineConfig, EngineConfigurable, ParallelEngine,
    SemanticEngine, SemanticResult, StatisticsProvider,
};
pub use xml::{
    extract_text_content, get_attribute, skip_element, XmlParser, XmlParserConfig, XmlResource,
    XmlSource,
};
