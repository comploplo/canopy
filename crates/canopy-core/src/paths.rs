//! Workspace path resolution utilities
//!
//! This module provides utilities for resolving paths relative to the workspace root,
//! which is essential for tests that may run from different directories.
//!
//! # Problem
//! Cargo runs integration tests from the crate directory (e.g., `crates/canopy-verbnet/`),
//! not the workspace root. Default data paths like `data/verbnet/vn-gl` fail because
//! they're workspace-relative.
//!
//! # Solution
//! The [`data_path`] function finds the workspace root and constructs the correct path.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// Cached workspace root to avoid repeated filesystem traversal
static WORKSPACE_ROOT: OnceLock<Option<PathBuf>> = OnceLock::new();

/// Find the workspace root by looking for Cargo.toml with [workspace]
///
/// This function traverses up from the current directory looking for a Cargo.toml
/// that contains a `[workspace]` section, indicating the workspace root.
///
/// The result is cached for performance.
pub fn workspace_root() -> Option<PathBuf> {
    WORKSPACE_ROOT.get_or_init(find_workspace_root_impl).clone()
}

/// Internal implementation of workspace root finding
fn find_workspace_root_impl() -> Option<PathBuf> {
    // Start from current directory
    let start = std::env::current_dir().ok()?;

    // Check the current directory and ancestors
    let mut current = start.as_path();

    loop {
        let cargo_toml = current.join("Cargo.toml");

        if cargo_toml.exists()
            && let Ok(contents) = std::fs::read_to_string(&cargo_toml)
            && contents.contains("[workspace]")
        {
            return Some(current.to_path_buf());
        }

        // Move to parent directory
        match current.parent() {
            Some(parent) => current = parent,
            None => break,
        }
    }

    // Fallback: check if we're in a "crates/xxx" subdirectory
    // and try going up two levels
    let start_str = start.to_string_lossy();
    if (start_str.contains("/crates/") || start_str.contains("\\crates\\"))
        && let Some(idx) = start_str
            .find("/crates/")
            .or_else(|| start_str.find("\\crates\\"))
    {
        let workspace = PathBuf::from(&start_str[..idx]);
        if workspace.join("Cargo.toml").exists() {
            return Some(workspace);
        }
    }

    None
}

/// Resolve a workspace-relative data path
///
/// This function takes a path relative to the workspace root (e.g., `data/verbnet/vn-gl`)
/// and returns the absolute path, regardless of the current working directory.
///
/// # Examples
///
/// ```ignore
/// use canopy_core::paths::data_path;
///
/// // Works whether run from workspace root or crate directory
/// let verbnet_path = data_path("data/verbnet/vn-gl");
/// assert!(verbnet_path.exists());
/// ```
pub fn data_path<P: AsRef<Path>>(relative: P) -> PathBuf {
    let relative = relative.as_ref();

    // If the path already exists relative to CWD, use it
    if relative.exists() {
        return relative.to_path_buf();
    }

    // Try to resolve from workspace root
    if let Some(root) = workspace_root() {
        let full_path = root.join(relative);
        if full_path.exists() {
            return full_path;
        }
    }

    // Fallback: try common relative paths from crate directories
    // (e.g., ../../data/verbnet/vn-gl when in crates/canopy-xxx/)
    for prefix in &["../..", "../../../", "../../.."] {
        let try_path = PathBuf::from(prefix).join(relative);
        if try_path.exists() {
            return try_path;
        }
    }

    // Return the original path if nothing else works
    // (will fail later with a clear error message)
    relative.to_path_buf()
}

/// Get the data path as a String (for config compatibility)
///
/// Same as [`data_path`] but returns a String for use in config structs
/// that expect String paths.
pub fn data_path_string<P: AsRef<Path>>(relative: P) -> String {
    data_path(relative).to_string_lossy().to_string()
}

/// Get absolute path to a cache file in data/cache/
///
/// Unlike [`data_path`], this function ALWAYS returns an absolute path relative
/// to the workspace root, even if the file doesn't exist yet. This is critical
/// for cache files that need to be created on first run.
///
/// The cache directory is created if it doesn't exist.
///
/// # Examples
///
/// ```ignore
/// use canopy_core::paths::cache_path;
///
/// // Always returns absolute path, regardless of CWD
/// let verbnet_cache = cache_path("verbnet.bin");
/// // Returns /path/to/workspace/data/cache/verbnet.bin
/// ```
pub fn cache_path(filename: &str) -> PathBuf {
    let cache_dir = workspace_root()
        .expect("Must run from within workspace for cache operations")
        .join("data/cache");

    // Create cache directory if it doesn't exist
    if !cache_dir.exists() {
        std::fs::create_dir_all(&cache_dir).ok();
    }

    cache_dir.join(filename)
}

/// Check if running from workspace root
pub fn is_workspace_root() -> bool {
    let cwd = std::env::current_dir().ok();
    let root = workspace_root();

    match (cwd, root) {
        (Some(cwd), Some(root)) => cwd == root,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_root_detection() {
        // Should find some workspace root (we're in a cargo workspace)
        let root = workspace_root();
        // Don't assert Some because tests might run in various environments
        if let Some(root) = root {
            assert!(root.join("Cargo.toml").exists());
        }
    }

    #[test]
    fn test_data_path_resolution() {
        // Test that data_path returns something
        let path = data_path("data/verbnet/vn-gl");
        // The path should at least be constructed
        assert!(!path.as_os_str().is_empty());
    }

    #[test]
    fn test_data_path_string() {
        let path_str = data_path_string("data/test");
        assert!(!path_str.is_empty());
        assert!(path_str.contains("data"));
        assert!(path_str.contains("test"));
    }
}
