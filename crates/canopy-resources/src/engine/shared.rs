//! Shared engine instances for pipeline components.
//!
//! Provides a way to share expensive engine instances (`VerbNet`, `WordNet`, Lexicon)
//! across multiple pipeline components, avoiding duplicate initialization.

use crate::lexicon::LexiconEngine;
use crate::verbnet::VerbNetEngine;
use crate::wordnet::WordNetEngine;
use canopy::CanopyError;
use std::sync::Arc;

/// Shared engine instances for pipeline components.
///
/// Engines are expensive to initialize (loading data files, building indexes).
/// This struct allows sharing engine instances across multiple components:
/// - `TreebankSyntaxProvider` (via `ResourceBackedTagger`)
/// - `VerbNetSenseProvider`
/// - `VerbNetRoleProvider`
///
/// # Example
///
/// ```rust,no_run
/// use canopy_resources::engine::SharedEngines;
///
/// let engines = SharedEngines::new()?;
/// // Pass to multiple components...
/// # Ok::<(), canopy::CanopyError>(())
/// ```
#[derive(Clone)]
pub struct SharedEngines {
    /// `VerbNet` engine (optional, may fail to load).
    pub verbnet: Option<Arc<VerbNetEngine>>,
    /// `WordNet` engine (optional, may fail to load).
    pub wordnet: Option<Arc<WordNetEngine>>,
    /// Lexicon engine (always available).
    pub lexicon: Arc<LexiconEngine>,
}

impl SharedEngines {
    /// Create shared engines, loading all available data.
    ///
    /// Engines that fail to load (e.g., missing data files) will be `None`.
    /// The lexicon is always initialized (may be empty if data unavailable).
    ///
    /// # Errors
    /// This function currently cannot fail but returns `Result` for API consistency.
    pub fn new() -> Result<Self, CanopyError> {
        // Create and load lexicon
        let mut lexicon = LexiconEngine::new();
        let _ = lexicon.load_data();
        let lexicon = Arc::new(lexicon);

        // Try to load VerbNet (optional)
        let verbnet = VerbNetEngine::new().ok().map(Arc::new);

        // Try to load WordNet (optional)
        let wordnet = WordNetEngine::new().ok().map(Arc::new);

        tracing::info!(
            "SharedEngines initialized: VerbNet={}, WordNet={}",
            verbnet.is_some(),
            wordnet.is_some()
        );

        Ok(Self {
            verbnet,
            wordnet,
            lexicon,
        })
    }

    /// Create with explicit engines (for testing or custom setups).
    #[must_use]
    pub fn with_engines(
        verbnet: Option<Arc<VerbNetEngine>>,
        wordnet: Option<Arc<WordNetEngine>>,
        lexicon: Arc<LexiconEngine>,
    ) -> Self {
        Self {
            verbnet,
            wordnet,
            lexicon,
        }
    }
}

impl std::fmt::Debug for SharedEngines {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SharedEngines")
            .field("verbnet", &self.verbnet.is_some())
            .field("wordnet", &self.wordnet.is_some())
            .field("lexicon", &"<loaded>")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn data_available() -> bool {
        crate::paths::data_path("data/lexicon").exists()
    }

    #[test]
    fn test_shared_engines_creation() {
        if !data_available() {
            eprintln!("Skipping: Data not available");
            return;
        }

        let engines = SharedEngines::new();
        assert!(engines.is_ok());
    }

    #[test]
    fn test_shared_engines_clone() {
        if !data_available() {
            eprintln!("Skipping: Data not available");
            return;
        }

        let engines = SharedEngines::new().unwrap();
        let engines2 = engines.clone();

        // Should share the same Arc instances
        if let (Some(v1), Some(v2)) = (&engines.verbnet, &engines2.verbnet) {
            assert!(Arc::ptr_eq(v1, v2));
        }
    }
}
