//! Stable ID types for the Canopy semantic kernel.
//!
//! These are simple, Copy types that provide strong typing for different
//! identifiers used throughout the system.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Token position identifier (0-indexed within a sentence).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TokenId(pub usize);

impl TokenId {
    /// Create a new token ID.
    #[must_use]
    pub const fn new(id: usize) -> Self {
        Self(id)
    }

    /// Get the underlying index.
    #[must_use]
    pub const fn index(self) -> usize {
        self.0
    }
}

impl fmt::Display for TokenId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "t{}", self.0)
    }
}

impl From<usize> for TokenId {
    fn from(id: usize) -> Self {
        Self(id)
    }
}

/// Syntax tree node identifier (for internal tree structure).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub usize);

impl NodeId {
    /// Create a new node ID.
    #[must_use]
    pub const fn new(id: usize) -> Self {
        Self(id)
    }

    /// Get the underlying index.
    #[must_use]
    pub const fn index(self) -> usize {
        self.0
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "n{}", self.0)
    }
}

impl From<usize> for NodeId {
    fn from(id: usize) -> Self {
        Self(id)
    }
}

/// Word sense identifier (`VerbNet` class, `FrameNet` frame, `WordNet` synset, etc.).
///
/// Format examples:
/// - `VerbNet`: "give-13.1-1"
/// - `FrameNet`: "Giving"
/// - `WordNet`: "give.v.01"
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SenseId(pub String);

impl SenseId {
    /// Create a new sense ID.
    pub fn new<S: Into<String>>(id: S) -> Self {
        Self(id.into())
    }

    /// Create a `VerbNet` sense ID.
    pub fn verbnet<S: Into<String>>(class: S) -> Self {
        Self(class.into())
    }

    /// Get the underlying string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Check if this is a `VerbNet` sense (format: class-number or class-number-subclass).
    ///
    /// Examples: "give-13.1", "run-51.3.2", "break-45.1"
    #[must_use]
    pub fn is_verbnet(&self) -> bool {
        // VerbNet format: word-digits.digits (e.g., "give-13.1")
        // Must have hyphen followed by a digit
        self.0.contains('-')
            && self
                .0
                .split('-')
                .next_back()
                .is_some_and(|s| s.chars().next().is_some_and(|c| c.is_ascii_digit()))
    }

    /// Check if this is a `WordNet` sense (format: `lemma.pos.sense_number`).
    ///
    /// Examples: "give.v.01", "book.n.02", "run.v.03"
    #[must_use]
    pub fn is_wordnet(&self) -> bool {
        // WordNet format: word.pos.number where pos is a single letter (n, v, a, r)
        let parts: Vec<&str> = self.0.split('.').collect();
        parts.len() >= 3
            && parts[1].len() == 1
            && parts[1]
                .chars()
                .all(|c| matches!(c, 'n' | 'v' | 'a' | 'r' | 's'))
    }
}

impl fmt::Display for SenseId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for SenseId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for SenseId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Frame identifier (`FrameNet` frame name).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FrameId(pub String);

impl FrameId {
    /// Create a new frame ID.
    pub fn new<S: Into<String>>(id: S) -> Self {
        Self(id.into())
    }

    /// Get the underlying string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for FrameId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for FrameId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for FrameId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_id() {
        let id = TokenId::new(5);
        assert_eq!(id.index(), 5);
        assert_eq!(format!("{id}"), "t5");
        assert_eq!(TokenId::from(3), TokenId::new(3));
    }

    #[test]
    fn test_node_id() {
        let id = NodeId::new(10);
        assert_eq!(id.index(), 10);
        assert_eq!(format!("{id}"), "n10");
        assert_eq!(NodeId::from(7), NodeId::new(7));
    }

    #[test]
    fn test_sense_id() {
        // VerbNet style
        let vn = SenseId::new("give-13.1");
        assert!(vn.is_verbnet());
        assert!(!vn.is_wordnet());

        // VerbNet with subclass
        let vn2 = SenseId::new("run-51.3.2");
        assert!(vn2.is_verbnet());
        assert!(!vn2.is_wordnet());

        // WordNet style
        let wn = SenseId::new("give.v.01");
        assert!(wn.is_wordnet());
        assert!(!wn.is_verbnet());

        // WordNet noun
        let wn2 = SenseId::new("book.n.02");
        assert!(wn2.is_wordnet());
        assert!(!wn2.is_verbnet());

        // FrameNet style (neither pattern)
        let fn_id = SenseId::new("Giving");
        assert!(!fn_id.is_verbnet());
        assert!(!fn_id.is_wordnet());
    }

    #[test]
    fn test_frame_id() {
        let id = FrameId::new("Commerce_buy");
        assert_eq!(id.as_str(), "Commerce_buy");
        assert_eq!(format!("{id}"), "Commerce_buy");
    }

    #[test]
    fn test_sense_id_from_impls() {
        let from_string: SenseId = String::from("test").into();
        let from_str: SenseId = "test".into();
        assert_eq!(from_string, from_str);
    }
}
