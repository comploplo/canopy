//! Pattern extraction from UD treebank for tokenization.
//!
//! Extracts contraction patterns (e.g., `don't` → `["do", "n't"]`) from
//! UD English-EWT multiword token annotations.

use canopy::CanopyError;
use std::collections::HashMap;
use std::path::Path;

/// A contraction pattern learned from treebank data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractionPattern {
    /// The contracted form (e.g., "don't").
    pub form: String,
    /// The expanded tokens (e.g., `["do", "n't"]`).
    pub tokens: Vec<String>,
    /// How often this pattern appears in the treebank.
    pub frequency: u32,
}

impl ContractionPattern {
    /// Create a new contraction pattern.
    #[must_use]
    pub fn new(form: String, tokens: Vec<String>) -> Self {
        Self {
            form,
            tokens,
            frequency: 1,
        }
    }

    /// Increment the frequency count.
    pub fn increment(&mut self) {
        self.frequency += 1;
    }
}

/// Extract contraction patterns from a UD treebank file.
///
/// Parses multiword tokens (lines like "1-2\tdon't") and their expansions
/// to learn how contractions should be split.
///
/// # Errors
/// Returns an error if the file cannot be opened or parsed.
pub fn extract_patterns_from_treebank<P: AsRef<Path>>(
    path: P,
) -> Result<Vec<ContractionPattern>, CanopyError> {
    // Parse file manually to extract multiword tokens
    // (the standard parser skips range token lines)
    extract_multiword_patterns(path.as_ref())
}

/// Extract multiword token patterns by parsing CoNLL-U manually.
///
/// Multiword tokens have ID ranges like "1-2" and the following lines
/// contain the actual tokens that make up the contraction.
fn extract_multiword_patterns<P: AsRef<Path>>(
    path: P,
) -> Result<Vec<ContractionPattern>, CanopyError> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    let path = path.as_ref();
    let file = File::open(path)
        .map_err(|e| CanopyError::data_load(format!("Failed to open {}: {}", path.display(), e)))?;

    let reader = BufReader::new(file);
    let mut patterns: HashMap<String, ContractionPattern> = HashMap::new();

    let mut current_multiword: Option<(String, u32, u32)> = None; // (form, start, end)
    let mut collected_tokens: Vec<String> = Vec::new();

    for line_result in reader.lines() {
        let line =
            line_result.map_err(|e| CanopyError::data_load(format!("Failed to read line: {e}")))?;

        let line = line.trim();

        // Empty line ends a sentence
        if line.is_empty() {
            // Finalize any pending multiword
            if let Some((form, _, _)) = current_multiword.take() {
                if !collected_tokens.is_empty() {
                    let key = form.to_lowercase();
                    patterns
                        .entry(key)
                        .and_modify(ContractionPattern::increment)
                        .or_insert_with(|| ContractionPattern::new(form, collected_tokens.clone()));
                }
                collected_tokens.clear();
            }
            continue;
        }

        // Skip comments
        if line.starts_with('#') {
            continue;
        }

        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() < 2 {
            continue;
        }

        let id_str = fields[0];
        let form = fields[1];

        // Check for multiword token (ID contains '-')
        if let Some(dash_pos) = id_str.find('-') {
            // Finalize previous multiword if any
            if let Some((prev_form, _, _)) = current_multiword.take() {
                if !collected_tokens.is_empty() {
                    let key = prev_form.to_lowercase();
                    patterns
                        .entry(key)
                        .and_modify(ContractionPattern::increment)
                        .or_insert_with(|| {
                            ContractionPattern::new(prev_form, collected_tokens.clone())
                        });
                }
                collected_tokens.clear();
            }

            // Parse the range
            let start: u32 = id_str[..dash_pos].parse().unwrap_or(0);
            let end: u32 = id_str[dash_pos + 1..].parse().unwrap_or(0);
            current_multiword = Some((form.to_string(), start, end));
            collected_tokens.clear();
        } else if let Some((_, start, end)) = &current_multiword {
            // Regular token - check if it's part of current multiword
            if let Ok(id) = id_str.parse::<u32>() {
                if id >= *start && id <= *end {
                    collected_tokens.push(form.to_string());
                }
                // If we've collected all tokens for this multiword
                if id == *end {
                    if let Some((mw_form, _, _)) = current_multiword.take() {
                        let key = mw_form.to_lowercase();
                        patterns
                            .entry(key)
                            .and_modify(ContractionPattern::increment)
                            .or_insert_with(|| {
                                ContractionPattern::new(mw_form, collected_tokens.clone())
                            });
                    }
                    collected_tokens.clear();
                }
            }
        }
    }

    // Convert to sorted vector (by frequency, descending)
    let mut result: Vec<ContractionPattern> = patterns.into_values().collect();
    result.sort_by(|a, b| b.frequency.cmp(&a.frequency));

    Ok(result)
}

/// Load patterns from all UD English-EWT files.
///
/// # Errors
/// Returns an error if the treebank is not found or cannot be parsed.
pub fn load_ewt_patterns() -> Result<Vec<ContractionPattern>, CanopyError> {
    use crate::paths::data_path;

    // Check both possible locations for the treebank
    let ud_dir = data_path("data/ud_english-ewt/UD_English-EWT");
    let ud_dir = if ud_dir.exists() {
        ud_dir
    } else {
        let alt = data_path("data/ud_english-ewt");
        if alt.exists() {
            alt
        } else {
            return Err(CanopyError::data_load(format!(
                "UD English-EWT treebank not found at {}",
                ud_dir.display()
            )));
        }
    };

    let mut all_patterns: HashMap<String, ContractionPattern> = HashMap::new();

    // Load from all three splits
    for split in &["train", "dev", "test"] {
        let file_path = ud_dir.join(format!("en_ewt-ud-{split}.conllu"));
        if file_path.exists() {
            let patterns = extract_patterns_from_treebank(&file_path)?;
            for pattern in patterns {
                let key = pattern.form.to_lowercase();
                all_patterns
                    .entry(key)
                    .and_modify(|p| p.frequency += pattern.frequency)
                    .or_insert(pattern);
            }
        }
    }

    let mut result: Vec<ContractionPattern> = all_patterns.into_values().collect();
    result.sort_by(|a, b| b.frequency.cmp(&a.frequency));

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contraction_pattern_creation() {
        let pattern = ContractionPattern::new(
            "don't".to_string(),
            vec!["do".to_string(), "n't".to_string()],
        );
        assert_eq!(pattern.form, "don't");
        assert_eq!(pattern.tokens, vec!["do", "n't"]);
        assert_eq!(pattern.frequency, 1);
    }

    #[test]
    fn test_contraction_pattern_increment() {
        let mut pattern = ContractionPattern::new(
            "won't".to_string(),
            vec!["will".to_string(), "not".to_string()],
        );
        pattern.increment();
        pattern.increment();
        assert_eq!(pattern.frequency, 3);
    }

    #[test]
    fn test_load_ewt_patterns() {
        // This test requires the treebank data
        let ud_path = crate::paths::data_path("data/ud_english-ewt");
        if !ud_path.exists() {
            eprintln!("Skipping: UD English-EWT data not available");
            return;
        }

        let patterns = load_ewt_patterns().expect("Failed to load patterns");

        // Should have found some contractions
        assert!(!patterns.is_empty(), "Should find contraction patterns");

        // Check for common contractions
        let found_do_not = patterns.iter().any(|p| p.form.to_lowercase() == "don't");
        let found_will_not = patterns.iter().any(|p| p.form.to_lowercase() == "won't");
        let found_can_not = patterns.iter().any(|p| p.form.to_lowercase() == "can't");

        // At least one common contraction should be present
        assert!(
            found_do_not || found_will_not || found_can_not,
            "Should find common contractions like don't, won't, or can't"
        );

        // Patterns should be sorted by frequency
        for window in patterns.windows(2) {
            assert!(
                window[0].frequency >= window[1].frequency,
                "Patterns should be sorted by frequency descending"
            );
        }
    }
}
