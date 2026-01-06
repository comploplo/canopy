//! Canopy kernel: event composition and discourse processing.
//!
//! The kernel contains pure linguistic processing logic with no
//! dependencies on heavy resources. External data (`VerbNet`, `FrameNet`, etc.)
//! is accessed through provider traits defined in `runtime`.
//!
//! # Architecture
//!
//! ```text
//! Layer 1 (Syntax)          Layer 2 (Events)          Layer 3 (Discourse)
//! AnnotatedSyntax  ──────►  ComposedEvents  ──────►  DRS
//!        │                         │                     │
//!        ▼                         ▼                     ▼
//!   SyntaxProvider          SenseProvider          ReferentRegistry
//!   RoleProvider                                   DiscourseContext
//! ```
//!
//! The kernel is **pure** - it contains NO word-level knowledge.
//! All lexical information comes through provider traits.
//!
//! # Modules
//!
//! - [`discourse`]: DRS construction and anaphora resolution (Layer 3)
//! - [`events`]: Event composition from predicates and roles (Layer 2)
//! - [`incremental`]: Surprisal-based incremental processing
//! - [`trace`]: Derivation trace types for explanation/debugging
//! - [`underspec`]: Underspecified semantic representations (packed readings)

pub mod discourse;
pub mod events;
pub mod incremental;
pub mod logic;
pub mod trace;
pub mod underspec;
