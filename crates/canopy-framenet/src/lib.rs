//! FrameNet integration for semantic analysis
//!
//! This crate provides FrameNet frame-based semantic parsing and analysis capabilities
//! using the canopy-engine infrastructure. FrameNet is a lexical database that maps
//! semantic frames to syntactic realizations.
//!
//! # Features
//!
//! - **Complete FrameNet Support**: Parse FrameNet XML files with full schema coverage
//! - **Engine Integration**: Uses canopy-engine traits for caching, statistics, and performance
//! - **Frame Analysis**: Maps words and phrases to semantic frames and frame elements
//! - **High Performance**: LRU caching and optimized data structures
//! - **Lexical Unit Processing**: Handle both frames and lexical units
//!
//! # Example
//!
//! ```rust
//! use canopy_framenet::{FrameNetEngine, DataLoader};
//!
//! let mut engine = FrameNetEngine::new();
//! engine.load_from_directory("data/framenet/archive/framenet_v17/framenet_v17").unwrap();
//!
//! let result = engine.analyze_text("give").unwrap();
//! println!("FrameNet frames for 'give': {:?}", result.data.frames);
//! ```

pub mod engine;
pub mod parser;
pub mod types;

// Re-export main types for convenience
pub use types::{
    CoreType, Frame, FrameElement, FrameElementAssignment, FrameElementRealization,
    FrameElementRelation, FrameNetAnalysis, FrameNetConfig, FrameNetStats, FrameRelation, Lexeme,
    LexicalUnit, LexicalUnitRef, SemanticType, SubcategorizationPattern, ValencePattern,
    ValenceUnit,
};

pub use engine::FrameNetEngine;
pub use parser::FrameParser;

// Re-export engine traits for convenience
pub use canopy_engine::{
    CachedEngine, DataLoader, EngineError, EngineResult, SemanticEngine, SemanticResult,
    StatisticsProvider,
};

/// FrameNet version information
pub const FRAMENET_VERSION: &str = "1.7";

/// Default FrameNet frames directory
pub const DEFAULT_FRAMES_DIR: &str = "data/framenet/archive/framenet_v17/framenet_v17/frame";

/// Default FrameNet lexical units directory
pub const DEFAULT_LU_DIR: &str = "data/framenet/archive/framenet_v17/framenet_v17/lu";

/// Utility functions for FrameNet operations
pub mod utils {
    use crate::types::{CoreType, Frame, FrameElement, LexicalUnit};

    /// Check if a frame contains a specific frame element
    pub fn frame_has_element(frame: &Frame, fe_name: &str) -> bool {
        frame.frame_elements.iter().any(|fe| fe.name == fe_name)
    }

    /// Get core frame elements from a frame
    pub fn get_core_elements(frame: &Frame) -> Vec<&FrameElement> {
        frame
            .frame_elements
            .iter()
            .filter(|fe| fe.core_type == CoreType::Core)
            .collect()
    }

    /// Get all lexical units from a list that belong to a specific frame
    pub fn filter_lus_by_frame<'a>(
        lus: &'a [LexicalUnit],
        frame_name: &str,
    ) -> Vec<&'a LexicalUnit> {
        lus.iter()
            .filter(|lu| lu.frame_name == frame_name)
            .collect()
    }

    /// Extract base word from lexical unit name (e.g., "give.v" -> "give")
    pub fn extract_base_word(lu_name: &str) -> &str {
        lu_name.split('.').next().unwrap_or(lu_name)
    }

    /// Check if a lexical unit name matches a word
    pub fn lu_matches_word(lu_name: &str, word: &str) -> bool {
        let base_word = extract_base_word(lu_name);
        base_word.eq_ignore_ascii_case(word)
    }

    /// Get the most specific (highest annotation count) lexical unit from a list
    pub fn most_annotated_lu(lus: &[LexicalUnit]) -> Option<&LexicalUnit> {
        lus.iter().max_by_key(|lu| lu.total_annotated)
    }

    /// Parse frame element colors to RGB values
    pub fn parse_fe_color(color_str: &str) -> Option<(u8, u8, u8)> {
        if color_str.len() == 6 {
            let r = u8::from_str_radix(&color_str[0..2], 16).ok()?;
            let g = u8::from_str_radix(&color_str[2..4], 16).ok()?;
            let b = u8::from_str_radix(&color_str[4..6], 16).ok()?;
            Some((r, g, b))
        } else {
            None
        }
    }

    /// Check if a frame is related to another frame
    pub fn frames_are_related(frame1: &Frame, frame2: &Frame) -> bool {
        frame1
            .frame_relations
            .iter()
            .any(|rel| rel.related_frame_id == frame2.id)
            || frame2
                .frame_relations
                .iter()
                .any(|rel| rel.related_frame_id == frame1.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::*;

    #[test]
    fn test_version_info() {
        assert_eq!(FRAMENET_VERSION, "1.7");
        assert!(DEFAULT_FRAMES_DIR.contains("framenet"));
        assert!(DEFAULT_LU_DIR.contains("framenet"));
    }

    #[test]
    fn test_extract_base_word() {
        assert_eq!(extract_base_word("give.v"), "give");
        assert_eq!(extract_base_word("run_away.v"), "run_away");
        assert_eq!(extract_base_word("simple"), "simple");
    }

    #[test]
    fn test_lu_matches_word() {
        assert!(lu_matches_word("give.v", "give"));
        assert!(lu_matches_word("GIVE.V", "give"));
        assert!(lu_matches_word("give.v", "GIVE"));
        assert!(!lu_matches_word("take.v", "give"));
    }

    #[test]
    fn test_parse_fe_color() {
        assert_eq!(parse_fe_color("FF0000"), Some((255, 0, 0)));
        assert_eq!(parse_fe_color("00FF00"), Some((0, 255, 0)));
        assert_eq!(parse_fe_color("0000FF"), Some((0, 0, 255)));
        assert_eq!(parse_fe_color("FFFFFF"), Some((255, 255, 255)));
        assert_eq!(parse_fe_color("invalid"), None);
        assert_eq!(parse_fe_color("FF"), None);
    }

    #[test]
    fn test_core_type_classification() {
        let core_fe = FrameElement {
            id: "1".to_string(),
            name: "Agent".to_string(),
            abbrev: "Agt".to_string(),
            core_type: CoreType::Core,
            bg_color: None,
            fg_color: None,
            created_by: None,
            created_date: None,
            definition: "Core element".to_string(),
            semantic_types: vec![],
            fe_relations: vec![],
        };

        assert!(core_fe.is_core());
        assert!(!core_fe.is_peripheral());
        assert!(!core_fe.is_extra_thematic());
    }
}
