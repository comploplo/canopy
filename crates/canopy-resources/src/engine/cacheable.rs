//! Cacheable data trait for binary serialization
//!
//! Provides common patterns for loading and saving engine data to binary cache files.
//! Used by `VerbNet`, `FrameNet`, `WordNet`, and other semantic engines.

use crate::paths;
use serde::{de::DeserializeOwned, Serialize};
use std::path::PathBuf;
use tracing::{info, warn};

/// Trait for engine data that can be cached to disk
///
/// Implement this trait to get automatic binary cache support.
/// Uses bincode for serialization and the workspace data/cache directory.
///
/// # Example
/// ```rust,ignore
/// use crate::engine::CacheableData;
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize)]
/// struct MyEngineData {
///     entries: Vec<String>,
/// }
///
/// impl CacheableData for MyEngineData {
///     fn cache_filename() -> &'static str {
///         "my_engine.bin"
///     }
///
///     fn engine_name() -> &'static str {
///         "MyEngine"
///     }
/// }
///
/// // Now you can use:
/// // let data = MyEngineData::load_from_cache();
/// // data.save_to_cache()?;
/// ```
pub trait CacheableData: Serialize + DeserializeOwned + Sized {
    /// The filename for the cache file (e.g., "verbnet.bin")
    fn cache_filename() -> &'static str;

    /// The engine name for logging (e.g., "`VerbNet`")
    fn engine_name() -> &'static str;

    /// Get the full path to the cache file
    #[must_use]
    fn cache_path() -> PathBuf {
        paths::cache_path(Self::cache_filename())
    }

    /// Load data from binary cache file
    ///
    /// Returns `Some(data)` if cache exists and deserializes successfully,
    /// `None` otherwise (with warnings logged on failure).
    fn load_from_cache() -> Option<Self> {
        let path = Self::cache_path();
        if !path.exists() {
            return None;
        }

        match std::fs::read(&path) {
            Ok(bytes) => match bincode::deserialize(&bytes) {
                Ok(data) => {
                    info!(
                        "Loaded {} data from cache ({} bytes)",
                        Self::engine_name(),
                        bytes.len()
                    );
                    Some(data)
                }
                Err(e) => {
                    warn!("Failed to deserialize {} cache: {}", Self::engine_name(), e);
                    None
                }
            },
            Err(e) => {
                warn!("Failed to read {} cache: {}", Self::engine_name(), e);
                None
            }
        }
    }

    /// Save data to binary cache file
    ///
    /// Serializes with bincode and writes to the cache directory.
    ///
    /// # Errors
    /// Returns an error if serialization or file writing fails.
    fn save_to_cache(&self) -> Result<(), String> {
        let path = Self::cache_path();
        let bytes = bincode::serialize(self)
            .map_err(|e| format!("Failed to serialize {} data: {}", Self::engine_name(), e))?;

        std::fs::write(&path, &bytes)
            .map_err(|e| format!("Failed to write {} cache: {}", Self::engine_name(), e))?;

        info!(
            "Saved {} data to cache ({} bytes)",
            Self::engine_name(),
            bytes.len()
        );
        Ok(())
    }

    /// Check if cache file exists
    #[must_use]
    fn cache_exists() -> bool {
        Self::cache_path().exists()
    }

    /// Delete the cache file if it exists
    ///
    /// # Errors
    /// Returns an error if the file cannot be deleted.
    fn clear_cache() -> Result<(), std::io::Error> {
        let path = Self::cache_path();
        if path.exists() {
            std::fs::remove_file(path)?;
            info!("Cleared {} cache", Self::engine_name());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestData {
        value: String,
        count: u32,
    }

    impl CacheableData for TestData {
        fn cache_filename() -> &'static str {
            "test_cacheable_resources.bin"
        }

        fn engine_name() -> &'static str {
            "Test"
        }
    }

    #[test]
    fn test_cache_path() {
        let path = TestData::cache_path();
        assert!(path
            .to_string_lossy()
            .contains("test_cacheable_resources.bin"));
    }

    #[test]
    fn test_cache_round_trip() {
        let data = TestData {
            value: "hello".to_string(),
            count: 42,
        };

        // Try to save - may fail if not in workspace
        if data.save_to_cache().is_ok() {
            // Should be able to load back
            let loaded = TestData::load_from_cache();
            assert!(loaded.is_some());
            assert_eq!(loaded.unwrap(), data);

            // Clean up
            let _ = TestData::clear_cache();
        }
    }

    #[test]
    fn test_cache_exists() {
        // Clear any existing cache first
        let _ = TestData::clear_cache();
        assert!(!TestData::cache_exists());
    }
}
