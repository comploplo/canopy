//! Shared test infrastructure for fast test execution
//!
//! This module provides shared singleton instances of semantic engines and coordinators
//! that are initialized once and reused across all tests. This reduces test time from
//! minutes (loading engines per-test) to seconds (one-time load, instant reuse).
//!
//! # Usage
//!
//! ```rust,ignore
//! use crate::test_support::{shared_coordinator, shared_verbnet};
//!
//! #[test]
//! fn test_something() {
//!     let coordinator = shared_coordinator();
//!     let result = coordinator.analyze("run").unwrap();
//!     assert!(result.has_results());
//! }
//! ```

use once_cell::sync::OnceCell;
use std::sync::{Arc, Mutex};

use crate::coordinator::{CoordinatorConfig, SemanticCoordinator};
use crate::engines::{MultiResourceAnalyzer, MultiResourceConfig, SemanticEngine};
use crate::wordnet::WordNetEngine;
use crate::{SemanticAnalyzer, SemanticConfig};

// Use engine types that match what engines.rs expects
type VerbNetEngine = canopy_verbnet::VerbNetEngine;
type FrameNetEngine = canopy_framenet::FrameNetEngine;

// Shared engine singletons - initialized once per test binary
// Note: Engines that require &mut self are wrapped in Mutex for thread-safe access
static SHARED_VERBNET: OnceCell<Mutex<VerbNetEngine>> = OnceCell::new();
static SHARED_FRAMENET: OnceCell<Mutex<FrameNetEngine>> = OnceCell::new();
static SHARED_WORDNET: OnceCell<Mutex<WordNetEngine>> = OnceCell::new();
static SHARED_COORDINATOR: OnceCell<Arc<SemanticCoordinator>> = OnceCell::new();
static SHARED_ANALYZER: OnceCell<SemanticAnalyzer> = OnceCell::new();
static SHARED_MULTI_RESOURCE: OnceCell<Mutex<MultiResourceAnalyzer>> = OnceCell::new();

/// Get shared VerbNet engine instance (wrapped in Mutex for mutable access).
///
/// The engine is loaded once on first call (taking ~10-15s), then reused
/// for all subsequent calls (<1ms).
///
/// # Panics
/// Panics if VerbNet data is not available at `data/verbnet/`.
pub fn shared_verbnet() -> &'static Mutex<VerbNetEngine> {
    SHARED_VERBNET.get_or_init(|| {
        eprintln!("🔧 Loading shared VerbNet engine (one-time)...");
        Mutex::new(VerbNetEngine::new().expect("VerbNet data required for tests"))
    })
}

/// Execute a closure with the shared VerbNet engine.
///
/// Returns None if engines are not available.
pub fn with_verbnet<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut VerbNetEngine) -> R,
{
    if !engines_available() {
        return None;
    }
    let mut engine = shared_verbnet().lock().unwrap();
    Some(f(&mut engine))
}

/// Get shared FrameNet engine instance (wrapped in Mutex for mutable access).
///
/// The engine is loaded once on first call (taking ~10-15s), then reused
/// for all subsequent calls (<1ms).
///
/// # Panics
/// Panics if FrameNet data is not available at `data/framenet/`.
pub fn shared_framenet() -> &'static Mutex<FrameNetEngine> {
    SHARED_FRAMENET.get_or_init(|| {
        eprintln!("🔧 Loading shared FrameNet engine (one-time)...");
        Mutex::new(FrameNetEngine::new().expect("FrameNet data required for tests"))
    })
}

/// Execute a closure with the shared FrameNet engine.
///
/// Returns None if engines are not available.
pub fn with_framenet<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut FrameNetEngine) -> R,
{
    if !engines_available() {
        return None;
    }
    let mut engine = shared_framenet().lock().unwrap();
    Some(f(&mut engine))
}

/// Get shared WordNet engine instance (wrapped in Mutex for mutable access).
///
/// The engine is loaded once on first call (taking ~5-10s), then reused
/// for all subsequent calls (<1ms).
///
/// # Panics
/// Panics if WordNet data is not available at `data/wordnet/`.
pub fn shared_wordnet() -> &'static Mutex<WordNetEngine> {
    SHARED_WORDNET.get_or_init(|| {
        eprintln!("🔧 Loading shared WordNet engine (one-time)...");
        Mutex::new(WordNetEngine::new().expect("WordNet data required for tests"))
    })
}

/// Execute a closure with the shared WordNet engine.
///
/// Returns None if engines are not available.
pub fn with_wordnet<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut WordNetEngine) -> R,
{
    if !engines_available() {
        return None;
    }
    let mut engine = shared_wordnet().lock().unwrap();
    Some(f(&mut engine))
}

/// Get shared SemanticCoordinator instance with all engines loaded.
///
/// The coordinator and all engines are loaded once on first call (taking ~30-40s),
/// then reused for all subsequent calls (<1ms).
///
/// # Configuration
/// Uses default configuration with all engines enabled:
/// - VerbNet: enabled
/// - FrameNet: enabled
/// - WordNet: enabled
/// - Lexicon: disabled (not needed for most tests)
/// - Lemmatization: enabled
///
/// # Panics
/// Panics if semantic data is not available.
pub fn shared_coordinator() -> Arc<SemanticCoordinator> {
    SHARED_COORDINATOR
        .get_or_init(|| {
            eprintln!("🔧 Loading shared SemanticCoordinator (one-time, ~30s)...");
            let config = CoordinatorConfig {
                enable_verbnet: true,
                enable_framenet: true,
                enable_wordnet: true,
                enable_lexicon: false, // Typically not needed for tests
                enable_treebank: false,
                enable_lemmatization: true,
                ..CoordinatorConfig::default()
            };
            Arc::new(SemanticCoordinator::new(config).expect("Semantic data required for tests"))
        })
        .clone()
}

/// Get shared SemanticAnalyzer instance with all engines loaded.
///
/// The analyzer and all engines are loaded once on first call (taking ~30-40s),
/// then reused for all subsequent calls (<1ms).
///
/// # Panics
/// Panics if semantic data is not available.
pub fn shared_analyzer() -> &'static SemanticAnalyzer {
    SHARED_ANALYZER.get_or_init(|| {
        eprintln!("🔧 Loading shared SemanticAnalyzer (one-time, ~30s)...");
        SemanticAnalyzer::new(SemanticConfig::default()).expect("Semantic data required for tests")
    })
}

/// Try to get shared SemanticAnalyzer, returning None if data unavailable.
///
/// Use this for tests that should skip gracefully when data is unavailable.
pub fn try_shared_analyzer() -> Option<&'static SemanticAnalyzer> {
    if !engines_available() {
        return None;
    }
    Some(shared_analyzer())
}

/// Get shared MultiResourceAnalyzer instance with all engines loaded.
///
/// The analyzer is wrapped in a Mutex for thread-safe mutable access.
/// Use `with_multi_resource_analyzer` for a more ergonomic API.
///
/// # Panics
/// Panics if semantic data is not available.
pub fn shared_multi_resource_analyzer() -> &'static Mutex<MultiResourceAnalyzer> {
    SHARED_MULTI_RESOURCE.get_or_init(|| {
        eprintln!("🔧 Loading shared MultiResourceAnalyzer (one-time, ~30s)...");
        let verbnet = VerbNetEngine::new().expect("VerbNet data required for tests");
        let framenet = FrameNetEngine::new().expect("FrameNet data required for tests");
        let wordnet = WordNetEngine::new().expect("WordNet data required for tests");
        Mutex::new(MultiResourceAnalyzer::new(
            verbnet,
            framenet,
            wordnet,
            MultiResourceConfig::default(),
        ))
    })
}

/// Execute a closure with the shared MultiResourceAnalyzer.
///
/// This provides a convenient way to use the shared analyzer without
/// explicitly locking/unlocking the mutex.
///
/// Returns None if engines are not available, or the closure result if they are.
pub fn with_multi_resource_analyzer<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut MultiResourceAnalyzer) -> R,
{
    if !engines_available() {
        return None;
    }
    let mut analyzer = shared_multi_resource_analyzer().lock().unwrap();
    Some(f(&mut analyzer))
}

/// Check if shared engines are available (for conditional test execution).
///
/// Returns `true` if all required data paths exist, `false` otherwise.
/// Use this for tests that should skip gracefully when data is unavailable.
pub fn engines_available() -> bool {
    use std::path::Path;

    Path::new("data/verbnet").exists()
        && Path::new("data/framenet").exists()
        && Path::new("data/wordnet").exists()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shared_verbnet_reuse() {
        if !engines_available() {
            eprintln!("Skipping: semantic data not available");
            return;
        }

        // First call initializes
        let engine1 = shared_verbnet();
        // Second call should return same static reference
        let engine2 = shared_verbnet();

        // Verify same static instance (pointer equality)
        assert!(std::ptr::eq(engine1, engine2));
    }

    #[test]
    fn test_shared_coordinator_reuse() {
        if !engines_available() {
            eprintln!("Skipping: semantic data not available");
            return;
        }

        let coord1 = shared_coordinator();
        let coord2 = shared_coordinator();

        assert!(Arc::ptr_eq(&coord1, &coord2));
    }

    #[test]
    fn test_with_verbnet_helper() {
        let result = with_verbnet(|engine| {
            // Just verify we can use the engine
            engine.analyze_token("run")
        });

        if result.is_none() {
            eprintln!("Skipping: semantic data not available");
            return;
        }

        let analysis = result.unwrap();
        assert!(analysis.is_ok(), "VerbNet should analyze 'run'");
    }
}
