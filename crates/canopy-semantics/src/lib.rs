//! # Canopy Semantics
//!
//! Semantic analysis components for canopy.rs following theoretical linguistics:
//! - Neo-Davidsonian event structures (Davidson 1967, Parsons 1990)
//! - Aspectual classification (Vendler 1967)
//! - VerbNet integration for verb class and theta role assignment
//! - Movement theory and syntactic structures (Chomsky, Lasnik)
//!
//! ## Architecture
//!
//! - `events`: Event-based semantic representation
//! - `layer2`: Clean semantic analysis without redundant theta assignment
//! - `syntax`: Movement theory and syntactic structures
//! - `verbnet`: VerbNet integration for semantic predicates
//!
//! ## VerbNet Integration
//!
//! This crate includes VerbNet 3.4 data and integration:
//! - Copyright 2005 by University of Pennsylvania
//! - Licensed under permissive terms allowing commercial use
//! - See LICENSE-VERBNET for full license text

#![allow(clippy::uninlined_format_args)] // Allow old format style in this crate
#![allow(clippy::needless_borrow)] // Allow explicit borrows for clarity
#![allow(clippy::len_zero)] // Allow len() > 0 for clarity
#![allow(clippy::field_reassign_with_default)] // Allow field assignment after default()
#![allow(clippy::collapsible_if)] // Allow nested if for clarity
#![allow(clippy::let_and_return)] // Allow explicit let binding before return
#![allow(clippy::useless_conversion)] // Allow explicit conversions for clarity
#![allow(clippy::redundant_closure_call)] // Allow closure patterns
#![allow(clippy::manual_contains)] // Allow iter().any() for expressiveness
#![allow(clippy::bind_instead_of_map)] // Allow and_then patterns
#![allow(clippy::new_without_default)] // Allow explicit new() functions
#![allow(clippy::clone_on_copy)] // Allow explicit clones for clarity
#![allow(clippy::unnecessary_map_or)] // Allow map_or patterns for compatibility
#![allow(clippy::enum_variant_names)] // Allow consistent enum naming
#![allow(clippy::needless_range_loop)] // Allow index-based loops for clarity
#![allow(clippy::manual_clamp)] // Allow manual min/max for compatibility

// Core modules for M3 event structure implementation
pub mod events;
pub mod features;
pub mod layer2;
pub mod performance_benchmarks;
pub mod syntax;
pub mod verbnet;

// Testing modules
#[cfg(test)]
pub mod error_handling_tests;

#[cfg(test)]
pub mod theta_role_inventory_test;

// Include coverage improvement tests for M3
#[cfg(test)]
mod coverage_improvement_tests;

// Re-export commonly used types from events module
pub use events::{
    Animacy, AspectualClass, CaseType, CausalRelation, ChainLink, CompositeEvent, CompositionType,
    Concreteness, Definiteness, Event, EventBuildError, EventBuilder, EventId, EventModifier,
    EventMovementType, EventStructure, EventTime, LandingSite, MovementChain, MovementFeature,
    Number, Participant, Predicate, PredicateType, SemanticFeature, TemporalRelation,
};

// Re-export Layer 2 types
pub use layer2::{
    Layer2Analyzer, Layer2Config, Layer2Error, Layer2Metrics, PerformanceMode, SemanticAnalysis,
    create_word_from_parse,
};

// Re-export VerbNet types
pub use verbnet::{
    AspectualInfo, SelectionalRestriction, SemanticPredicate, SyntacticFrame, ThetaRole,
    ThetaRoleType, VerbClass, VerbNetEngine, VerbNetFeatureExtractor, VerbNetFeatures,
};

// Re-export feature extraction types
pub use features::{
    EventTime as PredicateEventTime, ExtractedFeatures, ExtractedPredicate, PredicateExtractor,
    SemanticProperty, VerbNetFeatureExtractor as SimpleFeatureExtractor, validate_features,
};

// Re-export syntax types
pub use syntax::{
    DecompositionCondition, DecompositionRule, EventDecomposer, LittleVFeatures, LittleVShell,
    LittleVType, MovementAnalysis, MovementDetector, MovementSignals, MovementType, VPComplement,
    VerbPattern, VoiceAnalysis, VoiceDetector, VoiceFeatures, VoiceType,
};

// Re-export performance types
pub use performance_benchmarks::{
    BenchmarkError, BenchmarkResults, ComponentBreakdown, MemoryStats, PerformanceBenchmark,
    PerformanceReport, TestConfiguration,
};
