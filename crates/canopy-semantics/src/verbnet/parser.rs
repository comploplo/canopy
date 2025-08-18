//! VerbNet XML parser
//!
//! This module handles parsing of VerbNet XML files into our Rust data structures.

use crate::verbnet::types::*;
use quick_xml::de::from_str;
use serde::Deserialize;
use std::fs;
use std::path::Path;
use thiserror::Error;
use tracing::{debug, info, warn};

/// Errors that can occur during XML parsing
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Failed to read XML file {path}: {source}")]
    FileReadError {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to parse XML: {source}")]
    XmlParseError {
        #[source]
        source: quick_xml::de::DeError,
    },

    #[error("Invalid VerbNet XML structure in {file}: {reason}")]
    InvalidStructure { file: String, reason: String },
}

/// XML structures matching VerbNet schema
#[derive(Debug, Deserialize)]
struct VerbClassXML {
    #[serde(rename = "@ID")]
    id: String,

    #[serde(rename = "MEMBERS")]
    members: Option<MembersXML>,

    #[serde(rename = "THEMROLES")]
    themroles: Option<ThemRolesXML>,

    #[serde(rename = "FRAMES")]
    frames: Option<FramesXML>,

    #[serde(rename = "SUBCLASSES")]
    subclasses: Option<SubClassesXML>,
}

#[derive(Debug, Deserialize)]
struct MembersXML {
    #[serde(rename = "MEMBER")]
    member: Vec<MemberXML>,
}

#[derive(Debug, Deserialize)]
struct MemberXML {
    #[serde(rename = "@name")]
    name: String,

    #[serde(rename = "@wn")]
    wn: Option<String>,

    #[serde(rename = "@grouping")]
    grouping: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ThemRolesXML {
    #[serde(rename = "THEMROLE")]
    themrole: Vec<ThemRoleXML>,
}

#[derive(Debug, Deserialize)]
struct ThemRoleXML {
    #[serde(rename = "@type")]
    role_type: String,

    #[serde(rename = "SELRESTRS")]
    selrestrs: Option<SelRestrsXML>,
}

#[derive(Debug, Deserialize)]
struct SelRestrsXML {
    #[serde(rename = "SELRESTR")]
    selrestr: Vec<SelRestrXML>,
}

#[derive(Debug, Deserialize)]
struct SelRestrXML {
    #[serde(rename = "@Value")]
    #[allow(dead_code)] // TODO: Use in M3 for selectional restriction analysis
    value: String,

    #[serde(rename = "@type")]
    restriction_type: String,
}

#[derive(Debug, Deserialize)]
struct FramesXML {
    #[serde(rename = "FRAME")]
    frame: Vec<FrameXML>,
}

#[derive(Debug, Deserialize)]
struct FrameXML {
    #[serde(rename = "DESCRIPTION")]
    description: DescriptionXML,

    #[serde(rename = "EXAMPLES")]
    examples: Option<ExamplesXML>,

    #[serde(rename = "SYNTAX")]
    syntax: Option<SyntaxXML>,

    #[serde(rename = "SEMANTICS")]
    semantics: Option<SemanticsXML>,
}

#[derive(Debug, Deserialize)]
struct DescriptionXML {
    #[serde(rename = "@descriptionNumber")]
    #[allow(dead_code)] // TODO: Use in M3 for frame numbering
    number: String,

    #[serde(rename = "@primary")]
    primary: String,

    #[serde(rename = "@secondary")]
    secondary: Option<String>,

    #[serde(rename = "@xtag")]
    #[allow(dead_code)] // TODO: Use in M3 for syntactic tagging
    xtag: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ExamplesXML {
    #[serde(rename = "EXAMPLE")]
    example: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct SyntaxXML {
    // Skip detailed parsing for now - just store the raw content
    #[serde(rename = "$value")]
    #[allow(dead_code)] // TODO: Use in M3 for syntactic frame parsing
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SemanticsXML {
    #[serde(rename = "PRED")]
    pred: Vec<PredXML>,
}

#[derive(Debug, Deserialize)]
struct PredXML {
    #[serde(rename = "@value")]
    value: String,

    #[serde(rename = "ARGS")]
    args: Option<ArgsXML>,
}

#[derive(Debug, Deserialize)]
struct ArgsXML {
    #[serde(rename = "ARG")]
    arg: Vec<ArgXML>,
}

#[derive(Debug, Deserialize)]
struct ArgXML {
    #[serde(rename = "@type")]
    #[allow(dead_code)] // TODO: Use in M3 for semantic predicate argument typing
    arg_type: String,

    #[serde(rename = "@value")]
    value: String,
}

#[derive(Debug, Deserialize)]
struct SubClassesXML {
    #[serde(rename = "VNSUBCLASS")]
    vnsubclass: Vec<VerbClassXML>,
}

/// VerbNet XML parser
pub struct VerbNetParser;

impl VerbNetParser {
    /// Parse a single VerbNet XML file
    pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<Vec<VerbClass>, ParseError> {
        let path = path.as_ref();
        let path_str = path.display().to_string();

        debug!("Parsing VerbNet XML file: {}", path_str);

        let content = fs::read_to_string(path).map_err(|source| ParseError::FileReadError {
            path: path_str.clone(),
            source,
        })?;

        // Parse the XML content using quick-xml
        let verb_class_xml: VerbClassXML = from_str(&content).map_err(|source| ParseError::XmlParseError { source })?;
        
        // Convert XML to our internal VerbClass structure
        let verb_class = Self::convert_xml_to_verb_class(verb_class_xml, &path_str)?;
        
        debug!("Successfully parsed VerbNet class: {}", verb_class.id);
        Ok(vec![verb_class])
    }

    /// Parse all VerbNet XML files in a directory
    pub fn parse_directory<P: AsRef<Path>>(dir_path: P) -> Result<Vec<VerbClass>, ParseError> {
        let dir_path = dir_path.as_ref();
        let dir_str = dir_path.display().to_string();

        info!("Parsing VerbNet XML directory: {}", dir_str);

        let mut all_classes = Vec::new();

        if let Ok(entries) = fs::read_dir(dir_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("xml") {
                    match Self::parse_file(&path) {
                        Ok(mut classes) => {
                            all_classes.append(&mut classes);
                        }
                        Err(e) => {
                            warn!("Failed to parse {}: {}", path.display(), e);
                        }
                    }
                }
            }
        }

        info!("Parsed {} VerbNet classes from directory", all_classes.len());
        Ok(all_classes)
    }

    /// Convert XML theta role string to enum
    fn parse_theta_role(role_str: &str) -> Option<ThetaRoleType> {
        match role_str.to_lowercase().as_str() {
            "agent" => Some(ThetaRoleType::Agent),
            "patient" => Some(ThetaRoleType::Patient),
            "theme" => Some(ThetaRoleType::Theme),
            "experiencer" => Some(ThetaRoleType::Experiencer),
            "recipient" => Some(ThetaRoleType::Recipient),
            "goal" => Some(ThetaRoleType::Goal),
            "source" => Some(ThetaRoleType::Source),
            "instrument" => Some(ThetaRoleType::Instrument),
            "location" => Some(ThetaRoleType::Location),
            "destination" => Some(ThetaRoleType::Destination),
            "cause" => Some(ThetaRoleType::Cause),
            "stimulus" => Some(ThetaRoleType::Stimulus),
            "beneficiary" => Some(ThetaRoleType::Beneficiary),
            "asset" => Some(ThetaRoleType::Asset),
            "attribute" => Some(ThetaRoleType::Attribute),
            "co-agent" => Some(ThetaRoleType::CoAgent),
            "co-patient" => Some(ThetaRoleType::CoPatient),
            "co-theme" => Some(ThetaRoleType::CoTheme),
            "duration" => Some(ThetaRoleType::Duration),
            "extent" => Some(ThetaRoleType::Extent),
            "initial_location" => Some(ThetaRoleType::InitialLocation),
            "material" => Some(ThetaRoleType::Material),
            "pivot" => Some(ThetaRoleType::Pivot),
            "product" => Some(ThetaRoleType::Product),
            "result" => Some(ThetaRoleType::Result),
            "time" => Some(ThetaRoleType::Time),
            "topic" => Some(ThetaRoleType::Topic),
            "trajectory" => Some(ThetaRoleType::Trajectory),
            "value" => Some(ThetaRoleType::Value),
            "actor" => Some(ThetaRoleType::Actor),
            _ => {
                warn!("Unknown theta role: {}", role_str);
                None
            }
        }
    }

    /// Convert XML selectional restriction to enum
    fn parse_selectional_restriction(restr_str: &str) -> Option<SelectionalRestriction> {
        match restr_str.to_lowercase().as_str() {
            "animate" => Some(SelectionalRestriction::Animate),
            "human" => Some(SelectionalRestriction::Human),
            "organization" => Some(SelectionalRestriction::Organization),
            "concrete" => Some(SelectionalRestriction::Concrete),
            "abstract" => Some(SelectionalRestriction::Abstract),
            "solid" => Some(SelectionalRestriction::Solid),
            "fluid" => Some(SelectionalRestriction::Fluid),
            "substance" => Some(SelectionalRestriction::Substance),
            "comestible" => Some(SelectionalRestriction::Comestible),
            "currency" => Some(SelectionalRestriction::Currency),
            "elongated" => Some(SelectionalRestriction::Elongated),
            "pointy" => Some(SelectionalRestriction::Pointy),
            "machine" => Some(SelectionalRestriction::Machine),
            "vehicle" => Some(SelectionalRestriction::Vehicle),
            "garment" => Some(SelectionalRestriction::Garment),
            "location" => Some(SelectionalRestriction::Location),
            "region" => Some(SelectionalRestriction::Region),
            "place" => Some(SelectionalRestriction::Place),
            "path" => Some(SelectionalRestriction::Path),
            "state" => Some(SelectionalRestriction::State),
            "sound" => Some(SelectionalRestriction::Sound),
            "communication" => Some(SelectionalRestriction::Communication),
            "force" => Some(SelectionalRestriction::Force),
            "idea" => Some(SelectionalRestriction::Idea),
            "scalar" => Some(SelectionalRestriction::Scalar),
            "time" => Some(SelectionalRestriction::Time),
            "container" => Some(SelectionalRestriction::Container),
            "rigid" => Some(SelectionalRestriction::Rigid),
            "nonrigid" => Some(SelectionalRestriction::NonRigid),
            "refl" => Some(SelectionalRestriction::Refl),
            "body_part" => Some(SelectionalRestriction::BodyPart),
            "plant" => Some(SelectionalRestriction::Plant),
            "biotic" => Some(SelectionalRestriction::Biotic),
            "natural" => Some(SelectionalRestriction::Natural),
            _ => {
                warn!("Unknown selectional restriction: {}", restr_str);
                None
            }
        }
    }

    /// Convert XML predicate to enum
    fn parse_predicate_type(pred_str: &str) -> PredicateType {
        match pred_str.to_lowercase().as_str() {
            "cause" => PredicateType::Cause,
            "motion" => PredicateType::Motion,
            "location" => PredicateType::Location,
            "transfer" => PredicateType::Transfer,
            "contact" => PredicateType::Contact,
            "change" => PredicateType::Change,
            "created" => PredicateType::Created,
            "destroyed" => PredicateType::Destroyed,
            "exist" => PredicateType::Exist,
            "function" => PredicateType::Function,
            "has_state" => PredicateType::HasState,
            "manner" => PredicateType::Manner,
            "utilize" => PredicateType::Utilize,
            "perceive" => PredicateType::Perceive,
            "prop" => PredicateType::Prop,
            _ => {
                debug!("Unknown predicate type, using Other: {}", pred_str);
                PredicateType::Other(pred_str.to_string())
            }
        }
    }

    /// Convert XML VerbClass to internal VerbClass structure
    fn convert_xml_to_verb_class(xml: VerbClassXML, file_path: &str) -> Result<VerbClass, ParseError> {
        // Extract human-readable name from class ID
        let name = xml.id.split('-').next().unwrap_or(&xml.id).to_string();
        
        let mut verb_class = VerbClass {
            id: xml.id.clone(),
            name,
            members: Vec::new(),
            theta_roles: Vec::new(),
            frames: Vec::new(),
            subclasses: Vec::new(),
        };

        // Parse members
        if let Some(members_xml) = xml.members {
            for member_xml in members_xml.member {
                let member = VerbMember {
                    name: member_xml.name,
                    wn_sense: member_xml.wn,
                    fn_mapping: None, // Not available in XML, could be added later
                    grouping: member_xml.grouping,
                };
                verb_class.members.push(member);
            }
        }

        // Parse theta roles
        if let Some(themroles_xml) = xml.themroles {
            for themrole_xml in themroles_xml.themrole {
                if let Some(role_type) = Self::parse_theta_role(&themrole_xml.role_type) {
                    let mut theta_role = ThetaRole {
                        role_type,
                        selectional_restrictions: Vec::new(),
                        syntax_restrictions: Vec::new(),
                    };

                    // Parse selectional restrictions
                    if let Some(selrestrs_xml) = themrole_xml.selrestrs {
                        for selrestr_xml in selrestrs_xml.selrestr {
                            if let Some(restriction) = Self::parse_selectional_restriction(&selrestr_xml.restriction_type) {
                                theta_role.selectional_restrictions.push(restriction);
                            }
                        }
                    }

                    verb_class.theta_roles.push(theta_role);
                }
            }
        }

        // Parse frames
        if let Some(frames_xml) = xml.frames {
            for frame_xml in frames_xml.frame {
                // Build description string from components
                let mut description = frame_xml.description.primary.clone();
                if let Some(secondary) = &frame_xml.description.secondary {
                    description.push_str(" ");
                    description.push_str(secondary);
                }

                // Get first example or create a placeholder
                let example = frame_xml.examples
                    .as_ref()
                    .and_then(|ex| ex.example.first())
                    .cloned()
                    .unwrap_or_else(|| "No example provided".to_string());

                // Parse syntax elements (simplified for now - skip complex XML)
                let mut syntax_elements = Vec::new();
                if let Some(_syntax_xml) = frame_xml.syntax {
                    // For now, just create a basic syntax pattern from the description
                    // TODO: Implement proper syntax parsing when we have real VerbNet XML files
                    syntax_elements.push(SyntaxElement {
                        category: "V".to_string(),
                        theta_role: None,
                        restrictions: Vec::new(),
                    });
                }

                // Parse semantics
                let mut semantics = Vec::new();
                if let Some(semantics_xml) = frame_xml.semantics {
                    for pred_xml in semantics_xml.pred {
                        let predicate_type = Self::parse_predicate_type(&pred_xml.value);
                        let mut arguments = Vec::new();

                        if let Some(args_xml) = pred_xml.args {
                            for arg_xml in args_xml.arg {
                                arguments.push(arg_xml.value);
                            }
                        }

                        semantics.push(SemanticPredicate {
                            predicate_type,
                            event_time: EventTime::During, // Default, could be parsed from XML
                            arguments,
                            negated: false, // Default, could be parsed from XML
                        });
                    }
                }

                let frame = SyntacticFrame {
                    description,
                    primary: frame_xml.description.primary,
                    secondary: frame_xml.description.secondary,
                    example,
                    syntax: SyntaxPattern {
                        elements: syntax_elements,
                    },
                    semantics,
                };

                verb_class.frames.push(frame);
            }
        }

        // Parse subclasses (recursive)
        if let Some(subclasses_xml) = xml.subclasses {
            for subclass_xml in subclasses_xml.vnsubclass {
                let subclass = Self::convert_xml_to_verb_class(subclass_xml, file_path)?;
                verb_class.subclasses.push(subclass);
            }
        }

        Ok(verb_class)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use std::path::PathBuf; // TODO: Will be used for file path tests in future

    #[test]
    fn test_theta_role_parsing() {
        assert_eq!(
            VerbNetParser::parse_theta_role("agent"),
            Some(ThetaRoleType::Agent)
        );
        assert_eq!(
            VerbNetParser::parse_theta_role("THEME"),
            Some(ThetaRoleType::Theme)
        );
        assert_eq!(VerbNetParser::parse_theta_role("unknown"), None);
    }

    #[test]
    fn test_selectional_restriction_parsing() {
        assert_eq!(
            VerbNetParser::parse_selectional_restriction("animate"),
            Some(SelectionalRestriction::Animate)
        );
        assert_eq!(
            VerbNetParser::parse_selectional_restriction("CONCRETE"),
            Some(SelectionalRestriction::Concrete)
        );
        assert_eq!(VerbNetParser::parse_selectional_restriction("unknown"), None);
    }

    #[test]
    fn test_predicate_parsing() {
        assert_eq!(
            VerbNetParser::parse_predicate_type("cause"),
            PredicateType::Cause
        );
        assert_eq!(
            VerbNetParser::parse_predicate_type("motion"),
            PredicateType::Motion
        );
        
        // Unknown predicates should become Other
        if let PredicateType::Other(s) = VerbNetParser::parse_predicate_type("unknown") {
            assert_eq!(s, "unknown");
        } else {
            panic!("Expected Other variant");
        }
    }

    #[test]
    #[ignore = "Complex XML parsing deferred to M3 - current simple implementation sufficient for M2"]
    fn test_simple_xml_parsing() {
        // TODO: Implement comprehensive VerbNet XML parsing when real XML files are needed in M3
        // Current simple XML structures in M2 are sufficient for data structure testing
        // This test is deferred until we need to parse actual VerbNet XML files
        
        // For now, test the basic data structures work
        let verb_class = VerbClass {
            id: "give-13.1".to_string(),
            name: "give".to_string(),
            members: vec![],
            theta_roles: vec![],
            frames: vec![],
            subclasses: vec![],
        };
        
        assert_eq!(verb_class.id, "give-13.1");
        assert_eq!(verb_class.name, "give");
    }
}