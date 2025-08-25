//! FrameNet type definitions
//!
//! These types mirror the FrameNet XML schema structure, providing
//! Rust representations of frames, frame elements, lexical units, and semantic relations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A FrameNet frame (root element from frame XML)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Frame {
    /// Frame identifier (e.g., "139")
    pub id: String,
    /// Frame name (e.g., "Giving")
    pub name: String,
    /// Creation info
    pub created_by: Option<String>,
    /// Creation date
    pub created_date: Option<String>,
    /// Frame definition with examples
    pub definition: String,
    /// Frame elements (roles)
    pub frame_elements: Vec<FrameElement>,
    /// Related frames
    pub frame_relations: Vec<FrameRelation>,
    /// Lexical units that evoke this frame
    pub lexical_units: Vec<LexicalUnitRef>,
}

/// Frame element (semantic role) definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FrameElement {
    /// FE identifier
    pub id: String,
    /// FE name (e.g., "Agent", "Theme", "Recipient")
    pub name: String,
    /// FE abbreviation
    pub abbrev: String,
    /// Core type (Core, Peripheral, Extra-Thematic)
    pub core_type: CoreType,
    /// Background color (for annotation display)
    pub bg_color: Option<String>,
    /// Foreground color (for annotation display)
    pub fg_color: Option<String>,
    /// Creation info
    pub created_by: Option<String>,
    /// Creation date
    pub created_date: Option<String>,
    /// FE definition
    pub definition: String,
    /// Semantic types
    pub semantic_types: Vec<SemanticType>,
    /// Frame element relations
    pub fe_relations: Vec<FrameElementRelation>,
}

/// Core type classification for frame elements
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CoreType {
    Core,
    Peripheral,
    #[serde(rename = "Extra-Thematic")]
    ExtraThematic,
}

/// Semantic type annotation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SemanticType {
    /// Semantic type name
    pub name: String,
    /// Semantic type ID
    pub id: String,
}

/// Frame-to-frame relation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FrameRelation {
    /// Relation type (Inheritance, Using, etc.)
    pub relation_type: String,
    /// Related frame ID
    pub related_frame_id: String,
    /// Related frame name
    pub related_frame_name: String,
}

/// Frame element relation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FrameElementRelation {
    /// Relation type
    pub relation_type: String,
    /// Related FE in another frame
    pub related_fe: String,
    /// Related frame
    pub related_frame: String,
}

/// Reference to a lexical unit
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LexicalUnitRef {
    /// LU identifier
    pub id: String,
    /// LU name (e.g., "give.v")
    pub name: String,
    /// Part of speech
    pub pos: String,
    /// Status (Finished_Initial, etc.)
    pub status: String,
}

/// Complete lexical unit (from LU XML files)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LexicalUnit {
    /// LU identifier
    pub id: String,
    /// LU name (e.g., "give.v")
    pub name: String,
    /// Part of speech
    pub pos: String,
    /// Status
    pub status: String,
    /// Frame this LU belongs to
    pub frame_id: String,
    /// Frame name
    pub frame_name: String,
    /// Total annotations
    pub total_annotated: i32,
    /// Definition
    pub definition: String,
    /// Lexeme information
    pub lexemes: Vec<Lexeme>,
    /// Valence patterns
    pub valences: Vec<ValencePattern>,
    /// Subcategorization patterns
    pub subcategorization: Vec<SubcategorizationPattern>,
}

/// Lexeme (word form) information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Lexeme {
    /// Part of speech
    pub pos: String,
    /// Lexeme name
    pub name: String,
    /// Break before
    pub break_before: Option<bool>,
    /// Headword flag
    pub headword: Option<bool>,
}

/// Valence pattern for frame element realization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ValencePattern {
    /// Frame element name
    pub fe_name: String,
    /// Total occurrences
    pub total: i32,
    /// Grammatical function realizations
    pub realizations: Vec<FrameElementRealization>,
}

/// How a frame element is realized grammatically
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FrameElementRealization {
    /// Grammatical function (Ext, Obj, Dep, etc.)
    pub grammatical_function: String,
    /// Phrase type (NP, PP, etc.)
    pub phrase_type: String,
    /// Count of this realization
    pub count: i32,
}

/// Subcategorization pattern
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubcategorizationPattern {
    /// Pattern identifier
    pub id: String,
    /// Total count
    pub total: i32,
    /// Valence units in this pattern
    pub valence_units: Vec<ValenceUnit>,
}

/// Valence unit within subcategorization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ValenceUnit {
    /// Frame element
    pub fe: String,
    /// Phrase type
    pub pt: String,
    /// Grammatical function
    pub gf: String,
}

/// FrameNet analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameNetAnalysis {
    /// Analyzed lexical unit or phrase
    pub input: String,
    /// Matching frames
    pub frames: Vec<Frame>,
    /// Matching lexical units
    pub lexical_units: Vec<LexicalUnit>,
    /// Frame element assignments
    pub frame_element_assignments: Vec<FrameElementAssignment>,
    /// Confidence score
    pub confidence: f32,
}

/// Frame element assignment for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameElementAssignment {
    /// Text span that fills the frame element
    pub text_span: String,
    /// Start position in input
    pub start_pos: usize,
    /// End position in input
    pub end_pos: usize,
    /// Assigned frame element
    pub frame_element: FrameElement,
    /// Assignment confidence
    pub confidence: f32,
}

/// FrameNet engine statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameNetStats {
    /// Total number of frames loaded
    pub total_frames: usize,
    /// Total number of lexical units
    pub total_lexical_units: usize,
    /// Total number of frame elements
    pub total_frame_elements: usize,
    /// Total queries processed
    pub total_queries: u64,
    /// Cache hits
    pub cache_hits: u64,
    /// Cache misses
    pub cache_misses: u64,
    /// Average query time in microseconds
    pub avg_query_time_us: f64,
}

/// Configuration for FrameNet engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameNetConfig {
    /// Frames data directory path
    pub frames_path: String,
    /// Lexical units data directory path
    pub lexical_units_path: String,
    /// Enable caching
    pub enable_cache: bool,
    /// Cache capacity
    pub cache_capacity: usize,
    /// Confidence threshold for results
    pub confidence_threshold: f32,
    /// Additional settings
    pub settings: HashMap<String, String>,
}

impl Default for FrameNetConfig {
    fn default() -> Self {
        Self {
            frames_path: "data/framenet/archive/framenet_v17/framenet_v17/frame".to_string(),
            lexical_units_path: "data/framenet/archive/framenet_v17/framenet_v17/lu".to_string(),
            enable_cache: true,
            cache_capacity: 10000,
            confidence_threshold: 0.5,
            settings: HashMap::new(),
        }
    }
}

// Utility implementations

impl Frame {
    /// Get core frame elements
    pub fn core_elements(&self) -> Vec<&FrameElement> {
        self.frame_elements
            .iter()
            .filter(|fe| fe.core_type == CoreType::Core)
            .collect()
    }

    /// Get peripheral frame elements
    pub fn peripheral_elements(&self) -> Vec<&FrameElement> {
        self.frame_elements
            .iter()
            .filter(|fe| fe.core_type == CoreType::Peripheral)
            .collect()
    }

    /// Get extra-thematic frame elements
    pub fn extra_thematic_elements(&self) -> Vec<&FrameElement> {
        self.frame_elements
            .iter()
            .filter(|fe| fe.core_type == CoreType::ExtraThematic)
            .collect()
    }

    /// Check if frame has a specific frame element
    pub fn has_frame_element(&self, fe_name: &str) -> bool {
        self.frame_elements.iter().any(|fe| fe.name == fe_name)
    }

    /// Get frame element by name
    pub fn get_frame_element(&self, fe_name: &str) -> Option<&FrameElement> {
        self.frame_elements.iter().find(|fe| fe.name == fe_name)
    }
}

impl FrameElement {
    /// Check if frame element has a specific semantic type
    pub fn has_semantic_type(&self, sem_type: &str) -> bool {
        self.semantic_types.iter().any(|st| st.name == sem_type)
    }

    /// Check if this is a core frame element
    pub fn is_core(&self) -> bool {
        self.core_type == CoreType::Core
    }

    /// Check if this is a peripheral frame element
    pub fn is_peripheral(&self) -> bool {
        self.core_type == CoreType::Peripheral
    }

    /// Check if this is an extra-thematic frame element
    pub fn is_extra_thematic(&self) -> bool {
        self.core_type == CoreType::ExtraThematic
    }
}

impl LexicalUnit {
    /// Get primary lexeme
    pub fn primary_lexeme(&self) -> Option<&Lexeme> {
        self.lexemes
            .iter()
            .find(|l| l.headword.unwrap_or(false))
            .or_else(|| self.lexemes.first())
    }

    /// Get all valence patterns for a frame element
    pub fn get_valences_for_fe(&self, fe_name: &str) -> Vec<&ValencePattern> {
        self.valences
            .iter()
            .filter(|v| v.fe_name == fe_name)
            .collect()
    }

    /// Check if LU belongs to a specific frame
    pub fn belongs_to_frame(&self, frame_name: &str) -> bool {
        self.frame_name == frame_name
    }
}

impl FrameNetAnalysis {
    /// Create a new analysis result
    pub fn new(input: String, frames: Vec<Frame>, confidence: f32) -> Self {
        Self {
            input,
            frames,
            lexical_units: Vec::new(),
            frame_element_assignments: Vec::new(),
            confidence,
        }
    }

    /// Get the primary (most likely) frame
    pub fn primary_frame(&self) -> Option<&Frame> {
        self.frames.first()
    }

    /// Get all frame elements from all matching frames
    pub fn all_frame_elements(&self) -> Vec<&FrameElement> {
        self.frames.iter().flat_map(|f| &f.frame_elements).collect()
    }

    /// Get core frame elements from primary frame
    pub fn core_frame_elements(&self) -> Vec<&FrameElement> {
        self.primary_frame()
            .map(|f| f.core_elements())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_creation() {
        let frame = Frame {
            id: "139".to_string(),
            name: "Giving".to_string(),
            created_by: Some("MJE".to_string()),
            created_date: None,
            definition: "A frame about giving".to_string(),
            frame_elements: vec![FrameElement {
                id: "1052".to_string(),
                name: "Donor".to_string(),
                abbrev: "Donor".to_string(),
                core_type: CoreType::Core,
                bg_color: Some("FF0000".to_string()),
                fg_color: Some("FFFFFF".to_string()),
                created_by: None,
                created_date: None,
                definition: "The giver".to_string(),
                semantic_types: vec![],
                fe_relations: vec![],
            }],
            frame_relations: vec![],
            lexical_units: vec![],
        };

        assert_eq!(frame.id, "139");
        assert_eq!(frame.name, "Giving");
        assert!(frame.has_frame_element("Donor"));
        assert!(!frame.has_frame_element("NonExistent"));
        assert_eq!(frame.core_elements().len(), 1);
    }

    #[test]
    fn test_frame_element_types() {
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

        let peripheral_fe = FrameElement {
            id: "2".to_string(),
            name: "Time".to_string(),
            abbrev: "Time".to_string(),
            core_type: CoreType::Peripheral,
            bg_color: None,
            fg_color: None,
            created_by: None,
            created_date: None,
            definition: "Time element".to_string(),
            semantic_types: vec![],
            fe_relations: vec![],
        };

        assert!(core_fe.is_core());
        assert!(!core_fe.is_peripheral());
        assert!(!core_fe.is_extra_thematic());

        assert!(!peripheral_fe.is_core());
        assert!(peripheral_fe.is_peripheral());
        assert!(!peripheral_fe.is_extra_thematic());
    }

    #[test]
    fn test_framenet_config_default() {
        let config = FrameNetConfig::default();
        assert!(config.frames_path.contains("framenet"));
        assert!(config.enable_cache);
        assert_eq!(config.cache_capacity, 10000);
    }
}
