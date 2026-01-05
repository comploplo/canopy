//! TOML-based mapping loaders for semantic mappings.
//!
//! Loads configuration files that map between different linguistic representations,
//! such as `VerbNet` predicates to `LittleVType` or dependency relations to theta roles.

use crate::paths::data_path;
use canopy::kernel::events::LittleVType;
use canopy::ThetaRole;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

use super::{EngineError, EngineResult};

/// Mapping from `VerbNet` predicates to `LittleVType`.
#[derive(Debug, Clone)]
pub struct PredicateToLittleVMap {
    /// Predicate name (lowercase) -> `LittleVType`
    mappings: HashMap<String, LittleVType>,
    /// Default type when no mapping is found
    default: LittleVType,
}

impl Default for PredicateToLittleVMap {
    fn default() -> Self {
        Self {
            mappings: HashMap::new(),
            default: LittleVType::Do,
        }
    }
}

impl PredicateToLittleVMap {
    /// Load mappings from the default TOML file.
    ///
    /// # Errors
    /// Returns an error if the file cannot be read or parsed.
    pub fn load() -> EngineResult<Self> {
        let path = data_path("data/mappings/predicate-to-littlev.toml");
        Self::load_from_path(&path)
    }

    /// Load mappings from a specific path.
    ///
    /// # Errors
    /// Returns an error if the file cannot be read or parsed.
    pub fn load_from_path(path: &Path) -> EngineResult<Self> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            EngineError::data_load(format!("Failed to read predicate-to-littlev.toml: {e}"))
        })?;

        Self::parse_toml(&content)
    }

    /// Parse TOML content into mappings.
    fn parse_toml(content: &str) -> EngineResult<Self> {
        let toml: TomlPredicateMapping = toml::from_str(content).map_err(|e| {
            EngineError::data_load(format!("Failed to parse predicate-to-littlev.toml: {e}"))
        })?;

        let mut mappings = HashMap::new();

        // Parse default
        let default = parse_little_v_type(&toml.defaults.default).unwrap_or(LittleVType::Do);

        // Parse each LittleVType mapping
        for (little_v_name, mapping) in &toml.mappings {
            if let Some(little_v) = parse_little_v_type(little_v_name) {
                for predicate in &mapping.predicates {
                    mappings.insert(predicate.to_lowercase(), little_v);
                }
            }
        }

        Ok(Self { mappings, default })
    }

    /// Get the `LittleVType` for a predicate name.
    #[must_use]
    pub fn get(&self, predicate: &str) -> LittleVType {
        self.mappings
            .get(&predicate.to_lowercase())
            .copied()
            .unwrap_or(self.default)
    }

    /// Get the default `LittleVType`.
    #[must_use]
    pub fn default_type(&self) -> LittleVType {
        self.default
    }

    /// Check if a mapping exists for a predicate.
    #[must_use]
    pub fn contains(&self, predicate: &str) -> bool {
        self.mappings.contains_key(&predicate.to_lowercase())
    }
}

/// TOML structure for predicate mapping file.
#[derive(Debug, Deserialize)]
struct TomlPredicateMapping {
    #[serde(default)]
    #[allow(dead_code)] // Used for documentation in TOML
    metadata: TomlMetadata,
    defaults: TomlDefaults,
    mappings: HashMap<String, TomlMappingEntry>,
}

#[derive(Debug, Deserialize, Default)]
#[allow(dead_code)] // Fields used for documentation in TOML
struct TomlMetadata {
    #[serde(default)]
    source: String,
    #[serde(default)]
    theory: String,
    #[serde(default)]
    references: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct TomlDefaults {
    default: String,
}

#[derive(Debug, Deserialize)]
struct TomlMappingEntry {
    predicates: Vec<String>,
}

/// Parse a `LittleVType` from a string.
fn parse_little_v_type(s: &str) -> Option<LittleVType> {
    match s.to_lowercase().as_str() {
        "cause" => Some(LittleVType::Cause),
        "become" => Some(LittleVType::Become),
        "be" => Some(LittleVType::Be),
        "go" => Some(LittleVType::Go),
        "do" => Some(LittleVType::Do),
        "experience" => Some(LittleVType::Experience),
        "say" => Some(LittleVType::Say),
        "have" => Some(LittleVType::Have),
        "exist" => Some(LittleVType::Exist),
        _ => None,
    }
}

/// Mapping from dependency relations to theta roles.
#[derive(Debug, Clone, Default)]
pub struct DepRelToThetaMap {
    /// Deprel (lowercase) -> `ThetaRole`
    mappings: HashMap<String, ThetaRole>,
}

impl DepRelToThetaMap {
    /// Load mappings from the default TOML file.
    ///
    /// # Errors
    /// Returns an error if the file cannot be read or parsed.
    pub fn load() -> EngineResult<Self> {
        let path = data_path("data/mappings/deprel-to-theta.toml");
        Self::load_from_path(&path)
    }

    /// Load mappings from a specific path.
    ///
    /// # Errors
    /// Returns an error if the file cannot be read or parsed.
    pub fn load_from_path(path: &Path) -> EngineResult<Self> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            EngineError::data_load(format!("Failed to read deprel-to-theta.toml: {e}"))
        })?;

        Self::parse_toml(&content)
    }

    /// Parse TOML content into mappings.
    fn parse_toml(content: &str) -> EngineResult<Self> {
        let toml: TomlDepRelMapping = toml::from_str(content).map_err(|e| {
            EngineError::data_load(format!("Failed to parse deprel-to-theta.toml: {e}"))
        })?;

        let mut mappings = HashMap::new();

        for (deprel, theta_name) in &toml.mappings {
            if let Some(theta) = ThetaRole::parse(theta_name) {
                mappings.insert(deprel.to_lowercase(), theta);
            }
        }

        Ok(Self { mappings })
    }

    /// Get the `ThetaRole` for a dependency relation.
    #[must_use]
    pub fn get(&self, deprel: &str) -> Option<ThetaRole> {
        self.mappings.get(&deprel.to_lowercase()).copied()
    }

    /// Check if a mapping exists for a dependency relation.
    #[must_use]
    pub fn contains(&self, deprel: &str) -> bool {
        self.mappings.contains_key(&deprel.to_lowercase())
    }
}

/// TOML structure for deprel mapping file.
#[derive(Debug, Deserialize)]
struct TomlDepRelMapping {
    #[serde(default)]
    #[allow(dead_code)] // Used for documentation in TOML
    metadata: TomlMetadata,
    mappings: HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_predicate_map_parse() {
        let toml = r#"
[metadata]
source = "Test"

[defaults]
default = "Do"

[mappings.Cause]
predicates = ["cause"]

[mappings.Become]
predicates = ["become", "start"]
"#;

        let map = PredicateToLittleVMap::parse_toml(toml).unwrap();
        assert_eq!(map.get("cause"), LittleVType::Cause);
        assert_eq!(map.get("become"), LittleVType::Become);
        assert_eq!(map.get("start"), LittleVType::Become);
        assert_eq!(map.get("unknown"), LittleVType::Do);
    }

    #[test]
    fn test_predicate_map_case_insensitive() {
        let toml = r#"
[defaults]
default = "Do"

[mappings.Cause]
predicates = ["cause"]
"#;

        let map = PredicateToLittleVMap::parse_toml(toml).unwrap();
        assert_eq!(map.get("CAUSE"), LittleVType::Cause);
        assert_eq!(map.get("Cause"), LittleVType::Cause);
        assert_eq!(map.get("cause"), LittleVType::Cause);
    }

    #[test]
    fn test_deprel_map_parse() {
        let toml = r#"
[metadata]
source = "Test"

[mappings]
nsubj = "Agent"
obj = "Patient"
"#;

        let map = DepRelToThetaMap::parse_toml(toml).unwrap();
        assert_eq!(map.get("nsubj"), Some(ThetaRole::Agent));
        assert_eq!(map.get("obj"), Some(ThetaRole::Patient));
        assert_eq!(map.get("unknown"), None);
    }

    #[test]
    fn test_default_predicate_map() {
        let map = PredicateToLittleVMap::default();
        assert_eq!(map.default_type(), LittleVType::Do);
    }
}
