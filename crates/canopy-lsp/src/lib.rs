//! Canopy LSP Server
//!
//! Language Server Protocol implementation for Canopy semantic analysis.
//! Provides IDE features like semantic highlighting, hover info, diagnostics,
//! and document symbols powered by Canopy's linguistic analysis.

pub mod analysis;
pub mod backend;
pub mod handlers;
pub mod state;
pub mod tokens;

pub use backend::CanopyBackend;
pub use state::{CachedDocument, DocumentState, SentenceSpan};
