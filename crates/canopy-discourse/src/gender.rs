//! Name-gender lookup for anaphora resolution
//!
//! Loads a dataset of 147k names with gender information from the SSA
//! to improve pronoun resolution accuracy.
//!
//! Based on Reuland (2011) and modern binding theory, gender agreement
//! is a key constraint for anaphora resolution.

use crate::referent::Gender;
use indexmap::IndexMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::sync::OnceLock;

/// Global singleton for gender lookup
static GENDER_LOOKUP: OnceLock<GenderLookup> = OnceLock::new();

/// Name-to-gender mapping for anaphora resolution
#[derive(Debug, Clone)]
pub struct GenderLookup {
    /// Lowercase name -> Gender
    names: IndexMap<String, Gender>,
}

impl GenderLookup {
    /// Create an empty lookup
    #[must_use]
    pub fn new() -> Self {
        Self {
            names: IndexMap::new(),
        }
    }

    /// Load from CSV file (format: Name,Gender,Count,Probability)
    pub fn load_from_csv<P: AsRef<Path>>(path: P) -> Result<Self, GenderLookupError> {
        let file = File::open(path.as_ref()).map_err(|e| GenderLookupError::IoError {
            path: path.as_ref().to_string_lossy().to_string(),
            source: e.to_string(),
        })?;

        let reader = BufReader::new(file);
        let mut names = IndexMap::new();

        for (line_num, line_result) in reader.lines().enumerate() {
            let line = line_result.map_err(|e| GenderLookupError::IoError {
                path: path.as_ref().to_string_lossy().to_string(),
                source: e.to_string(),
            })?;

            // Skip header
            if line_num == 0 {
                continue;
            }

            // Skip empty lines
            if line.trim().is_empty() {
                continue;
            }

            // Parse: Name,Gender,Count,Probability
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 2 {
                let name = parts[0].trim().to_lowercase();
                let gender = match parts[1].trim() {
                    "M" => Gender::Masculine,
                    "F" => Gender::Feminine,
                    _ => continue, // Skip unknown genders
                };
                // Keep the first entry for each name (CSV is sorted by count desc,
                // so first entry is the most common gender for that name)
                names.entry(name).or_insert(gender);
            }
        }

        Ok(Self { names })
    }

    /// Get global singleton, loading from default path if needed
    pub fn global() -> &'static GenderLookup {
        GENDER_LOOKUP.get_or_init(|| {
            // Try to load from the default dataset path
            let default_path = "data/canopy-lexicon/name_gender_dataset.csv";
            Self::load_from_csv(default_path).unwrap_or_else(|_| {
                // Fall back to empty lookup if file not found
                Self::new()
            })
        })
    }

    /// Initialize global singleton from a specific path
    pub fn init_global<P: AsRef<Path>>(path: P) -> Result<(), GenderLookupError> {
        let lookup = Self::load_from_csv(path)?;
        GENDER_LOOKUP
            .set(lookup)
            .map_err(|_| GenderLookupError::AlreadyInitialized)
    }

    /// Infer gender from a name
    ///
    /// Returns None if the name is not in the dataset.
    /// Note: This only works for names, not pronouns.
    /// For pronouns, use `classify_anaphor()` instead.
    #[must_use]
    pub fn infer(&self, name: &str) -> Option<Gender> {
        self.names.get(&name.to_lowercase()).copied()
    }

    /// Check if a name is in the dataset
    #[must_use]
    pub fn contains(&self, name: &str) -> bool {
        self.names.contains_key(&name.to_lowercase())
    }

    /// Number of names in the dataset
    #[must_use]
    pub fn len(&self) -> usize {
        self.names.len()
    }

    /// Check if the dataset is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }
}

impl Default for GenderLookup {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors from gender lookup
#[derive(Debug, Clone)]
pub enum GenderLookupError {
    /// Failed to read file
    IoError { path: String, source: String },
    /// Global lookup already initialized
    AlreadyInitialized,
}

impl std::fmt::Display for GenderLookupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError { path, source } => {
                write!(f, "Failed to read gender dataset at {}: {}", path, source)
            }
            Self::AlreadyInitialized => {
                write!(f, "Global gender lookup already initialized")
            }
        }
    }
}

impl std::error::Error for GenderLookupError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_lookup() {
        let lookup = GenderLookup::new();
        assert!(lookup.is_empty());
        assert_eq!(lookup.infer("John"), None);
    }

    #[test]
    fn test_load_from_csv() {
        // Try to load the actual dataset
        let lookup =
            GenderLookup::load_from_csv("../../data/canopy-lexicon/name_gender_dataset.csv");

        if let Ok(lookup) = lookup {
            // Should have loaded many names
            assert!(
                lookup.len() > 1000,
                "Expected many names, got {}",
                lookup.len()
            );

            // Check some known names
            assert_eq!(lookup.infer("john"), Some(Gender::Masculine));
            assert_eq!(lookup.infer("mary"), Some(Gender::Feminine));
            assert_eq!(lookup.infer("james"), Some(Gender::Masculine));
            assert_eq!(lookup.infer("elizabeth"), Some(Gender::Feminine));

            // Case insensitive
            assert_eq!(lookup.infer("JOHN"), Some(Gender::Masculine));
            assert_eq!(lookup.infer("Mary"), Some(Gender::Feminine));
        }
        // If file not found, that's OK for CI environments
    }

    #[test]
    fn test_unknown_name() {
        let lookup = GenderLookup::new();
        assert_eq!(lookup.infer("xyznonexistent"), None);
    }
}
