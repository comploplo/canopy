//! Workspace path resolution for resource loading.
//!
//! Resolves paths relative to the workspace root, handling the case where
//! tests or binaries run from different working directories.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// Cached workspace root to avoid repeated filesystem traversal.
static WORKSPACE_ROOT: OnceLock<Option<PathBuf>> = OnceLock::new();

/// Find the workspace root by looking for Cargo.toml with [workspace].
fn find_workspace_root() -> Option<PathBuf> {
    let start = std::env::current_dir().ok()?;
    let mut current = start.as_path();

    loop {
        let cargo_toml = current.join("Cargo.toml");
        if cargo_toml.exists() {
            if let Ok(contents) = std::fs::read_to_string(&cargo_toml) {
                if contents.contains("[workspace]") {
                    return Some(current.to_path_buf());
                }
            }
        }

        current = current.parent()?;
    }
}

/// Get the workspace root, caching the result.
pub fn workspace_root() -> Option<PathBuf> {
    WORKSPACE_ROOT.get_or_init(find_workspace_root).clone()
}

/// Resolve a workspace-relative data path.
///
/// Takes a path relative to the workspace root (e.g., `data/verbnet/vn-gl`)
/// and returns the resolved path, trying various fallback strategies.
pub fn data_path<P: AsRef<Path>>(relative: P) -> PathBuf {
    let relative = relative.as_ref();

    // If path exists relative to CWD, use it
    if relative.exists() {
        return relative.to_path_buf();
    }

    // Try workspace root
    if let Some(root) = workspace_root() {
        let full = root.join(relative);
        if full.exists() {
            return full;
        }
    }

    // Fallback: common relative paths from crate subdirectories
    for prefix in ["../..", "../../..", "../../../.."] {
        let try_path = PathBuf::from(prefix).join(relative);
        if try_path.exists() {
            return try_path;
        }
    }

    // Return original (will fail later with clear error)
    relative.to_path_buf()
}

/// Get data path as String for config compatibility.
pub fn data_path_string<P: AsRef<Path>>(relative: P) -> String {
    data_path(relative).to_string_lossy().into_owned()
}

/// Get cache directory path, creating it if necessary.
///
/// Uses `data/cache/` in the workspace root.
pub fn cache_path<P: AsRef<Path>>(filename: P) -> PathBuf {
    let cache_dir = if let Some(root) = workspace_root() {
        root.join("data").join("cache")
    } else {
        PathBuf::from("data").join("cache")
    };

    // Ensure cache directory exists
    if !cache_dir.exists() {
        let _ = std::fs::create_dir_all(&cache_dir);
    }

    cache_dir.join(filename)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_root_detection() {
        if let Some(root) = workspace_root() {
            assert!(root.join("Cargo.toml").exists());
        }
    }

    #[test]
    fn test_data_path_returns_something() {
        let path = data_path("data/verbnet");
        assert!(!path.as_os_str().is_empty());
    }

    #[test]
    fn test_data_path_string_returns_string() {
        let s = data_path_string("data/test");
        assert!(!s.is_empty());
    }
}
