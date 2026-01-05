//! # Canopy
//!
//! High-performance semantic linguistic analysis library.
//!
//! ## Architecture
//!
//! Canopy is structured as a clean kernel with provider-based dependency injection:
//!
//! - **core**: Foundational types (`ThetaRole`, `LittleV`, `CanopyError`)
//! - **kernel**: Event composition and discourse processing
//! - **runtime**: Provider traits for external resources
//!
//! The kernel has no dependencies on heavy resources (`VerbNet`, `FrameNet`, etc.).
//! Resources are injected via provider traits defined in `runtime`.
//!
//! ## Example
//!
//! ```rust,ignore
//! use canopy::runtime::{SenseProvider, PredicateDecomposition};
//! use canopy::kernel::events::{EventComposer, ComposedEvents};
//! use canopy::kernel::discourse::{DiscourseContext, Drs};
//!
//! // Create a provider (from canopy-resources)
//! let provider = canopy_resources::providers::DefaultProvider::new()?;
//!
//! // Use the kernel for event composition
//! let composer = EventComposer::new(EventComposerConfig::default());
//! let events = composer.compose(&analysis, &decompositions, &bindings)?;
//!
//! // Build discourse representation
//! let mut ctx = DiscourseContext::default();
//! ctx.process_events(&events);
//! let drs = ctx.drs();
//! ```

// === Clean Architecture ===
pub mod core;
pub mod kernel;
pub mod runtime;

// Re-export runtime provider types (the dependency wall)
pub use runtime::{
    AnnotatedSyntax, AnnotatedToken, CanopyProvider, DecompositionSource, DiscourseCueProvider,
    DiscourseRelation, FrameId, NodeId, PredicateDecomposition, RoleBinding, RoleProvider,
    RoleSource, SenseId, SenseInfo, SenseProvider, SenseSource, SyntaxProvider, SyntaxTree,
    TokenId,
};

// Re-export core types for convenience
pub use core::{CanopyError, DepRel, MorphFeatures, ThetaRole, UPos};

// Re-export kernel types
pub use kernel::discourse::{
    // Core discourse types
    AnaphorType,
    BindingResult,
    // Coherence relations
    CoherenceClassification,
    CoherenceClassifier,
    CoherenceEdge,
    CoherenceGraph,
    CoherenceRelation,
    CoherenceSignal,
    DiscourseConfig,
    DiscourseContext,
    // Discourse moves
    DiscourseMove,
    DiscourseReferent,
    Drs,
    DrsCondition,
    DrsId,
    Gender,
    MoveClassification,
    MoveClassifier,
    NumberFeature,
    // QUD structures
    PartialAnswer,
    // Presuppositions
    Presupposition,
    PresuppositionDetector,
    PresuppositionManager,
    PresuppositionStatus,
    PriorState,
    PronounResolver,
    QudIssue,
    QudOrigin,
    QudReport,
    QudReportEntry,
    QudStack,
    QudStatus,
    QudTree,
    QudTreeInfo,
    QudTreeNode,
    QudUpdate,
    QudUpdateAction,
    QuestionType,
    ReferentId,
    ReferentRegistry,
    RelevanceAlignment,
    RelevanceLevel,
    RelevanceReport,
    SentenceData,
    SentenceReferents,
    TrackedPresupposition,
    ValidationReport,
    ValidationStatus,
};
pub use kernel::events::{
    ComposedEvent, ComposedEvents, DependencyArc, EventComposer, EventComposerConfig, LittleVType,
    PackedEvents, Participant, SentenceAnalysis,
};
pub use kernel::incremental::{
    BeamSearch, BeamSearchConfig, GardenPathDetector, GardenPathEvent, IncrementalProcessor,
    IncrementalState, ReadingPrefix, Surprisal, SurprisalModel, UniformSurprisalModel,
};
pub use kernel::trace::{
    DerivationTrace, DiscourseSummary, EventSummary, EventTrace, ParticipantTrace, SelectionReason,
    SenseReading, SenseSelectionTrace, SyntaxSummary, TraceMetadata,
};
pub use kernel::underspec::{
    AmbiguitySummary, ChoiceId, ChoicePoint, ChoiceType, ConfidenceDisambiguator,
    DisambiguationContext, Disambiguator, EntropyReductionDisambiguator, HybridDisambiguator,
    InteractiveDisambiguator, MinSurprisalDisambiguator, PackedSemantics, Reading, ReadingId,
    ReadingsAccess,
};

/// Version information for the library
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_info() {
        assert_eq!(VERSION, "0.1.0");
    }

    #[test]
    fn test_kernel_modules_exist() {
        use crate::kernel::discourse::Drs;
        use crate::kernel::events::LittleVType;
        let _ = LittleVType::Do;
        let _ = Drs::default();
    }

    #[test]
    fn test_runtime_types() {
        use crate::runtime::{AnnotatedSyntax, SenseId, TokenId};
        let _ = TokenId::new(0);
        let _ = SenseId::new("test");
        let _ = AnnotatedSyntax::new("test".to_string(), vec![]);
    }

    #[test]
    fn test_reexports() {
        // Verify key types are re-exported at crate root
        let _ = ThetaRole::Agent;
        let _ = LittleVType::Cause;
        let _ = TokenId::new(0);
    }
}
