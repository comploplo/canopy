//! # Canopy Semantics
//!
//! Semantic analysis components for canopy.rs including:
//! - VerbNet integration for verb class and theta role assignment
//! - Event structure representation
//! - Semantic feature extraction
//!
//! ## VerbNet Integration
//!
//! This crate includes VerbNet 3.4 data and integration:
//! - Copyright 2005 by University of Pennsylvania
//! - Licensed under permissive terms allowing commercial use
//! - See LICENSE-VERBNET for full license text

pub mod verbnet;

// Re-export commonly used types
pub use verbnet::{
    VerbNetEngine, VerbClass, ThetaRole, ThetaRoleType,
    SelectionalRestriction, SyntacticFrame, SemanticPredicate,
    PredicateType, EventTime, AspectualInfo,
    VerbNetFeatureExtractor, VerbNetFeatures, Animacy, Concreteness,
};
