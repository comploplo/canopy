//! Underspecified semantic representations.
//!
//! This module implements underspecification theory for representing
//! semantic ambiguity without premature commitment to a single reading.
//!
//! # Academic Foundations
//!
//! - **UDRT** (Reyle 1993): Underspecified DRS with labeled boxes
//! - **MRS** (Copestake et al. 2005): Handle-based scope underspecification
//! - **Packed Representations** (Alshawi & Crouch 1992): Shared structure, O(n) memory
//!
//! # Architecture
//!
//! ```text
//! PackedSemantics
//!     │
//!     ├── SharedStructure (common across all readings)
//!     │
//!     └── ChoicePoints (where readings diverge)
//!             │
//!             ├── LexicalSense (word sense ambiguity)
//!             ├── Attachment (PP attachment)
//!             ├── Scope (quantifier scope)
//!             └── Reference (pronoun resolution)
//!
//! Reading = assignment of choices at each ChoicePoint
//! ```

mod disambiguation;
mod scope;
mod types;

pub use disambiguation::{
    ConfidenceDisambiguator, DisambiguationContext, Disambiguator, EntropyReductionDisambiguator,
    HybridDisambiguator, InteractiveDisambiguator, MinSurprisalDisambiguator,
};
pub use scope::{
    ElementaryPredication, Handle, HandleConstraint, HandleConstraintType, ScopeOrdering,
    ScopeUnderspec, Variable,
};
pub use types::{
    Alternative, AmbiguitySummary, ChoiceId, ChoicePoint, ChoiceType, PackedSemantics, Reading,
    ReadingId, ReadingsAccess, ReadingsIterator, SemanticConstraint, SharedStructure,
};
