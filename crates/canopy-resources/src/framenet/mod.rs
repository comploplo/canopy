//! `FrameNet` integration for semantic analysis
//!
//! `FrameNet` frame-based semantic parsing and analysis capabilities.

pub mod engine;
pub mod parser;
pub mod types;

// Re-export main types
pub use engine::FrameNetEngine;
pub use parser::FrameParser;
pub use types::{
    CoreType, Frame, FrameElement, FrameElementAssignment, FrameElementRealization,
    FrameElementRelation, FrameNetAnalysis, FrameNetConfig, FrameNetStats, FrameRelation, Lexeme,
    LexicalUnit, LexicalUnitRef, SemanticType, SubcategorizationPattern, ValencePattern,
    ValenceUnit,
};

/// `FrameNet` version information
pub const FRAMENET_VERSION: &str = "1.7";

/// Default `FrameNet` frames directory
pub const DEFAULT_FRAMES_DIR: &str = "data/framenet/archive/framenet_v17/framenet_v17/frame";

/// Default `FrameNet` lexical units directory
pub const DEFAULT_LU_DIR: &str = "data/framenet/archive/framenet_v17/framenet_v17/lu";

/// Utility functions for `FrameNet` operations
pub mod utils {
    use super::types::{CoreType, Frame, FrameElement, LexicalUnit};

    /// Check if a frame contains a specific frame element
    #[must_use]
    pub fn frame_has_element(frame: &Frame, fe_name: &str) -> bool {
        frame.frame_elements.iter().any(|fe| fe.name == fe_name)
    }

    /// Get core frame elements from a frame
    #[must_use]
    pub fn get_core_elements(frame: &Frame) -> Vec<&FrameElement> {
        frame
            .frame_elements
            .iter()
            .filter(|fe| fe.core_type == CoreType::Core)
            .collect()
    }

    /// Get all lexical units from a list that belong to a specific frame
    #[must_use]
    pub fn filter_lus_by_frame<'a>(
        lus: &'a [LexicalUnit],
        frame_name: &str,
    ) -> Vec<&'a LexicalUnit> {
        lus.iter()
            .filter(|lu| lu.frame_name == frame_name)
            .collect()
    }

    /// Extract base word from lexical unit name (e.g., "give.v" -> "give")
    #[must_use]
    pub fn extract_base_word(lu_name: &str) -> &str {
        lu_name.split('.').next().unwrap_or(lu_name)
    }

    /// Check if a lexical unit name matches a word
    #[must_use]
    pub fn lu_matches_word(lu_name: &str, word: &str) -> bool {
        let base_word = extract_base_word(lu_name);
        base_word.eq_ignore_ascii_case(word)
    }

    /// Get the most specific (highest annotation count) lexical unit from a list
    #[must_use]
    pub fn most_annotated_lu(lus: &[LexicalUnit]) -> Option<&LexicalUnit> {
        lus.iter().max_by_key(|lu| lu.total_annotated)
    }

    /// Parse frame element colors to RGB values
    #[must_use]
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
    #[must_use]
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

    #[test]
    fn test_framenet_version() {
        assert_eq!(FRAMENET_VERSION, "1.7");
    }

    #[test]
    fn test_default_dirs() {
        assert!(DEFAULT_FRAMES_DIR.contains("framenet"));
        assert!(DEFAULT_LU_DIR.contains("framenet"));
    }

    #[test]
    fn test_extract_base_word() {
        assert_eq!(utils::extract_base_word("give.v"), "give");
        assert_eq!(utils::extract_base_word("run.v"), "run");
        assert_eq!(utils::extract_base_word("happy.a"), "happy");
        assert_eq!(utils::extract_base_word("quickly.adv"), "quickly");
        assert_eq!(utils::extract_base_word("noperiod"), "noperiod");
    }

    #[test]
    fn test_lu_matches_word() {
        assert!(utils::lu_matches_word("give.v", "give"));
        assert!(utils::lu_matches_word("give.v", "Give"));
        assert!(utils::lu_matches_word("GIVE.v", "give"));
        assert!(!utils::lu_matches_word("give.v", "take"));
    }

    #[test]
    fn test_parse_fe_color_valid() {
        assert_eq!(utils::parse_fe_color("FF0000"), Some((255, 0, 0))); // Red
        assert_eq!(utils::parse_fe_color("00FF00"), Some((0, 255, 0))); // Green
        assert_eq!(utils::parse_fe_color("0000FF"), Some((0, 0, 255))); // Blue
        assert_eq!(utils::parse_fe_color("FFFFFF"), Some((255, 255, 255))); // White
        assert_eq!(utils::parse_fe_color("000000"), Some((0, 0, 0))); // Black
    }

    #[test]
    fn test_parse_fe_color_invalid() {
        assert_eq!(utils::parse_fe_color(""), None);
        assert_eq!(utils::parse_fe_color("FFF"), None); // Too short
        assert_eq!(utils::parse_fe_color("FFFFFFF"), None); // Too long
        assert_eq!(utils::parse_fe_color("GGGGGG"), None); // Invalid hex
    }

    #[test]
    fn test_frame_has_element() {
        let frame = Frame {
            id: "1".to_string(),
            name: "Giving".to_string(),
            created_by: None,
            created_date: None,
            definition: "Transfer of possession".to_string(),
            frame_elements: vec![
                FrameElement {
                    id: "1".to_string(),
                    name: "Donor".to_string(),
                    abbrev: "Donor".to_string(),
                    definition: "The person giving".to_string(),
                    core_type: CoreType::Core,
                    semantic_types: vec![],
                    fg_color: None,
                    bg_color: None,
                    created_by: None,
                    created_date: None,
                    fe_relations: vec![],
                },
                FrameElement {
                    id: "2".to_string(),
                    name: "Theme".to_string(),
                    abbrev: "Theme".to_string(),
                    definition: "The thing given".to_string(),
                    core_type: CoreType::Core,
                    semantic_types: vec![],
                    fg_color: None,
                    bg_color: None,
                    created_by: None,
                    created_date: None,
                    fe_relations: vec![],
                },
            ],
            lexical_units: vec![],
            frame_relations: vec![],
        };

        assert!(utils::frame_has_element(&frame, "Donor"));
        assert!(utils::frame_has_element(&frame, "Theme"));
        assert!(!utils::frame_has_element(&frame, "Agent"));
    }

    #[test]
    fn test_get_core_elements() {
        let frame = Frame {
            id: "1".to_string(),
            name: "Test".to_string(),
            created_by: None,
            created_date: None,
            definition: "Test frame".to_string(),
            frame_elements: vec![
                FrameElement {
                    id: "1".to_string(),
                    name: "CoreElement".to_string(),
                    abbrev: "Core".to_string(),
                    definition: "Core".to_string(),
                    core_type: CoreType::Core,
                    semantic_types: vec![],
                    fg_color: None,
                    bg_color: None,
                    created_by: None,
                    created_date: None,
                    fe_relations: vec![],
                },
                FrameElement {
                    id: "2".to_string(),
                    name: "PeripheralElement".to_string(),
                    abbrev: "Periph".to_string(),
                    definition: "Peripheral".to_string(),
                    core_type: CoreType::Peripheral,
                    semantic_types: vec![],
                    fg_color: None,
                    bg_color: None,
                    created_by: None,
                    created_date: None,
                    fe_relations: vec![],
                },
            ],
            lexical_units: vec![],
            frame_relations: vec![],
        };

        let core = utils::get_core_elements(&frame);
        assert_eq!(core.len(), 1);
        assert_eq!(core[0].name, "CoreElement");
    }

    #[test]
    fn test_filter_lus_by_frame() {
        let lus = vec![
            LexicalUnit {
                id: "1".to_string(),
                name: "give.v".to_string(),
                pos: "V".to_string(),
                frame_id: "1".to_string(),
                frame_name: "Giving".to_string(),
                definition: String::new(),
                lexemes: vec![],
                status: String::new(),
                total_annotated: 100,
                valences: vec![],
                subcategorization: vec![],
            },
            LexicalUnit {
                id: "2".to_string(),
                name: "take.v".to_string(),
                pos: "V".to_string(),
                frame_id: "2".to_string(),
                frame_name: "Taking".to_string(),
                definition: String::new(),
                lexemes: vec![],
                status: String::new(),
                total_annotated: 50,
                valences: vec![],
                subcategorization: vec![],
            },
        ];

        let giving_lus = utils::filter_lus_by_frame(&lus, "Giving");
        assert_eq!(giving_lus.len(), 1);
        assert_eq!(giving_lus[0].name, "give.v");

        let taking_lus = utils::filter_lus_by_frame(&lus, "Taking");
        assert_eq!(taking_lus.len(), 1);
        assert_eq!(taking_lus[0].name, "take.v");

        let empty = utils::filter_lus_by_frame(&lus, "NonExistent");
        assert!(empty.is_empty());
    }

    #[test]
    fn test_most_annotated_lu() {
        let lus = vec![
            LexicalUnit {
                id: "1".to_string(),
                name: "give.v".to_string(),
                pos: "V".to_string(),
                frame_id: "1".to_string(),
                frame_name: "Giving".to_string(),
                definition: String::new(),
                lexemes: vec![],
                status: String::new(),
                total_annotated: 100,
                valences: vec![],
                subcategorization: vec![],
            },
            LexicalUnit {
                id: "2".to_string(),
                name: "donate.v".to_string(),
                pos: "V".to_string(),
                frame_id: "1".to_string(),
                frame_name: "Giving".to_string(),
                definition: String::new(),
                lexemes: vec![],
                status: String::new(),
                total_annotated: 200,
                valences: vec![],
                subcategorization: vec![],
            },
        ];

        let best = utils::most_annotated_lu(&lus);
        assert!(best.is_some());
        assert_eq!(best.unwrap().name, "donate.v");
    }

    #[test]
    fn test_most_annotated_lu_empty() {
        let lus: Vec<LexicalUnit> = vec![];
        assert!(utils::most_annotated_lu(&lus).is_none());
    }
}
