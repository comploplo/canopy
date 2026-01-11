//! Runtime provider traits for dependency injection.
//!
//! This module defines the provider traits that form the dependency wall between
//! the semantic kernel (canopy) and the heavy resource loaders (canopy-resources).
//!
//! The kernel can only access external resources through these traits, ensuring:
//! - The kernel can be tested with mock implementations
//! - Different backends can be swapped in (`VerbNet`, `FrameNet`, custom resources)
//! - The kernel builds and tests WITHOUT dataset downloads present

pub mod ids;
pub mod ir;
pub mod providers;

pub use ids::{FrameId, NodeId, SenseId, TokenId};
pub use ir::{AnnotatedSyntax, AnnotatedToken, MweInfo, MweType, PhrasalVerb, SyntaxTree};
pub use providers::{
    CanopyProvider, DecompositionSource, DiscourseCueProvider, DiscourseRelation,
    PredicateDecomposition, RoleBinding, RoleProvider, RoleSource, SenseInfo, SenseProvider,
    SenseSource, SyntaxProvider,
};
