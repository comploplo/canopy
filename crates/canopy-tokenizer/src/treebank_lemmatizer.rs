//! Treebank-trained lemmatizer using UD English-EWT data
//!
//! This lemmatizer learns from the full UD English-EWT training data (~247K lines)
//! to provide high-accuracy lemmatization based on real corpus statistics.

use crate::lemmatizer::{Lemmatizer, LemmatizerError, LemmatizerResult};
use std::collections::HashMap;
use std::path::Path;
use tracing::{debug, info};

/// Treebank-trained lemmatizer with learned mappings
pub struct TreebankLemmatizer {
    /// Direct word→lemma mappings learned from training data
    word_to_lemma: HashMap<String, String>,
    /// Suffix transformation rules learned from patterns
    suffix_rules: Vec<(String, String, f32)>, // (from_suffix, to_suffix, confidence)
    /// Fallback to simple rules
    fallback: super::SimpleLemmatizer,
    /// Training statistics
    total_mappings: usize,
    unique_words: usize,
}

impl TreebankLemmatizer {
    /// Create a new treebank lemmatizer
    pub fn new() -> LemmatizerResult<Self> {
        let fallback = super::SimpleLemmatizer::new()?;

        Ok(Self {
            word_to_lemma: HashMap::new(),
            suffix_rules: Vec::new(),
            fallback,
            total_mappings: 0,
            unique_words: 0,
        })
    }

    /// Train from UD English-EWT training data
    pub fn train_from_file<P: AsRef<Path>>(&mut self, conllu_path: P) -> LemmatizerResult<()> {
        info!(
            "Training TreebankLemmatizer from {:?}",
            conllu_path.as_ref()
        );

        // Parse the training file
        let content = std::fs::read_to_string(conllu_path.as_ref()).map_err(|e| {
            LemmatizerError::InitializationError(format!("Failed to read training file: {e}"))
        })?;

        let mut line_count = 0;
        let mut word_lemma_pairs = Vec::new();

        for line in content.lines() {
            line_count += 1;
            if line_count % 50000 == 0 {
                info!("Processed {} lines", line_count);
            }

            // Skip comments and empty lines
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }

            // Parse CoNLL-U format: ID FORM LEMMA UPOS XPOS FEATS HEAD DEPREL DEPS MISC
            let fields: Vec<&str> = line.split('\t').collect();
            if fields.len() >= 3 {
                let word = fields[1].to_lowercase(); // FORM (normalize case)
                let lemma = fields[2].to_lowercase(); // LEMMA

                // Skip punctuation and special tokens
                if Self::should_skip_token(&word, &lemma) {
                    continue;
                }

                word_lemma_pairs.push((word, lemma));
            }
        }

        info!(
            "Extracted {} word-lemma pairs from {} lines",
            word_lemma_pairs.len(),
            line_count
        );

        // Build direct mappings
        self.build_direct_mappings(&word_lemma_pairs);

        // Learn suffix rules
        self.learn_suffix_rules(&word_lemma_pairs);

        self.total_mappings = word_lemma_pairs.len();
        self.unique_words = self.word_to_lemma.len();

        info!(
            "Training complete: {} total mappings, {} unique words, {} suffix rules",
            self.total_mappings,
            self.unique_words,
            self.suffix_rules.len()
        );

        Ok(())
    }

    /// Check if token should be skipped during training
    fn should_skip_token(word: &str, lemma: &str) -> bool {
        // Skip punctuation
        if word.chars().all(|c| !c.is_alphanumeric()) {
            return true;
        }

        // Skip very short tokens
        if word.len() < 2 {
            return true;
        }

        // Skip if lemma is underscore (missing)
        if lemma == "_" {
            return true;
        }

        // Skip numbers
        if word
            .chars()
            .all(|c| c.is_numeric() || c == '.' || c == ',' || c == '-')
        {
            return true;
        }

        false
    }

    /// Build direct word→lemma mappings
    fn build_direct_mappings(&mut self, pairs: &[(String, String)]) {
        let mut frequency_map = HashMap::new();

        // Count frequencies of each word→lemma mapping
        for (word, lemma) in pairs {
            // Store all mappings, including identical ones for completeness
            *frequency_map
                .entry((word.clone(), lemma.clone()))
                .or_insert(0) += 1;
        }

        // Keep only the most frequent mapping for each word
        let mut word_best_lemma = HashMap::new();
        for ((word, lemma), count) in frequency_map {
            let current_count = word_best_lemma.get(&word).map(|(_, c)| *c).unwrap_or(0);
            if count > current_count {
                word_best_lemma.insert(word, (lemma, count));
            }
        }

        // Store the best mappings
        for (word, (lemma, count)) in word_best_lemma {
            // For testing with small data, use minimum frequency of 1
            if count >= 1 {
                self.word_to_lemma.insert(word, lemma);
            }
        }

        debug!("Built {} direct mappings", self.word_to_lemma.len());
    }

    /// Learn suffix transformation rules
    fn learn_suffix_rules(&mut self, pairs: &[(String, String)]) {
        let mut suffix_stats = HashMap::new();

        // Analyze suffix patterns
        for (word, lemma) in pairs {
            if word == lemma {
                continue;
            }

            // Try different suffix lengths (character-aware)
            let word_chars: Vec<char> = word.chars().collect();
            let lemma_chars: Vec<char> = lemma.chars().collect();

            for suffix_len in 1..=4 {
                if word_chars.len() > suffix_len && !lemma_chars.is_empty() {
                    let word_suffix: String =
                        word_chars[word_chars.len() - suffix_len..].iter().collect();
                    let word_root: String =
                        word_chars[..word_chars.len() - suffix_len].iter().collect();

                    // Find what the suffix should become
                    let common_len =
                        std::cmp::min(word_chars.len() - suffix_len, lemma_chars.len());
                    if common_len > 0 {
                        let lemma_root: String = lemma_chars[..common_len].iter().collect();
                        if word_root == lemma_root {
                            let lemma_suffix: String = lemma_chars[common_len..].iter().collect();
                            let rule = (word_suffix, lemma_suffix);
                            *suffix_stats.entry(rule).or_insert(0) += 1;
                        }
                    }
                }
            }
        }

        // Convert to rules with confidence scores
        for ((from_suffix, to_suffix), count) in suffix_stats {
            if count >= 5 {
                // Minimum frequency threshold
                let confidence = (count as f32).ln() / 10.0; // Logarithmic confidence
                self.suffix_rules
                    .push((from_suffix, to_suffix, confidence.min(1.0)));
            }
        }

        // Sort by confidence (best first)
        self.suffix_rules
            .sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());

        // Keep top 200 rules
        self.suffix_rules.truncate(200);

        debug!("Learned {} suffix rules", self.suffix_rules.len());
    }

    /// Apply suffix rules for unknown words
    fn apply_suffix_rules(&self, word: &str) -> Option<String> {
        for (from_suffix, to_suffix, confidence) in &self.suffix_rules {
            if word.ends_with(from_suffix) && confidence > &0.1 {
                // Use character-aware string manipulation
                let word_chars: Vec<char> = word.chars().collect();
                let from_suffix_chars: Vec<char> = from_suffix.chars().collect();

                if word_chars.len() >= from_suffix_chars.len() {
                    let root_chars = &word_chars[..word_chars.len() - from_suffix_chars.len()];
                    if root_chars.len() >= 2 {
                        // Keep reasonable root length
                        let root: String = root_chars.iter().collect();
                        let lemma = format!("{root}{to_suffix}");
                        debug!(
                            "Applied rule: {} -> {} (confidence: {:.2})",
                            word, lemma, confidence
                        );
                        return Some(lemma);
                    }
                }
            }
        }
        None
    }

    /// Get training statistics
    pub fn get_stats(&self) -> TreebankLemmatizerStats {
        TreebankLemmatizerStats {
            total_mappings: self.total_mappings,
            unique_words: self.unique_words,
            direct_mappings: self.word_to_lemma.len(),
            suffix_rules: self.suffix_rules.len(),
        }
    }
}

impl Lemmatizer for TreebankLemmatizer {
    fn lemmatize(&self, word: &str) -> String {
        let normalized = word.to_lowercase();

        // 1. Check direct mappings first (highest accuracy)
        if let Some(lemma) = self.word_to_lemma.get(&normalized) {
            return lemma.clone();
        }

        // 2. Try suffix rules
        if let Some(lemma) = self.apply_suffix_rules(&normalized) {
            return lemma;
        }

        // 3. Fallback to simple lemmatizer
        self.fallback.lemmatize(word)
    }

    fn lemmatize_with_confidence(&self, word: &str) -> (String, f32) {
        let normalized = word.to_lowercase();

        // 1. Direct mapping (highest confidence)
        if let Some(lemma) = self.word_to_lemma.get(&normalized) {
            return (lemma.clone(), 0.95);
        }

        // 2. Suffix rules (moderate confidence)
        if let Some(lemma) = self.apply_suffix_rules(&normalized) {
            return (lemma, 0.8);
        }

        // 3. Fallback (lower confidence)
        let (lemma, _) = self.fallback.lemmatize_with_confidence(word);
        (lemma, 0.6)
    }

    fn supports_batch(&self) -> bool {
        true
    }

    fn lemmatize_batch(&self, words: &[String]) -> Vec<String> {
        words.iter().map(|word| self.lemmatize(word)).collect()
    }
}

/// Statistics for treebank lemmatizer training
#[derive(Debug, Clone)]
pub struct TreebankLemmatizerStats {
    pub total_mappings: usize,
    pub unique_words: usize,
    pub direct_mappings: usize,
    pub suffix_rules: usize,
}

/// Create a TreebankLemmatizer trained on UD English-EWT data
pub fn create_trained_lemmatizer<P: AsRef<Path>>(
    training_path: P,
) -> LemmatizerResult<TreebankLemmatizer> {
    let mut lemmatizer = TreebankLemmatizer::new()?;
    lemmatizer.train_from_file(training_path)?;
    Ok(lemmatizer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_conllu_data() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "# sent_id = train-001").unwrap();
        writeln!(file, "# text = Children were running quickly.").unwrap();
        writeln!(
            file,
            "1\tChildren\tchild\tNOUN\tNNS\tNumber=Plur\t3\tnsubj\t3:nsubj\t_"
        )
        .unwrap();
        writeln!(
            file,
            "2\twere\tbe\tAUX\tVBD\tMood=Ind|Number=Plur\t3\taux\t3:aux\t_"
        )
        .unwrap();
        writeln!(
            file,
            "3\trunning\trun\tVERB\tVBG\tAspect=Prog\t0\troot\t0:root\t_"
        )
        .unwrap();
        writeln!(
            file,
            "4\tquickly\tquickly\tADV\tRB\t_\t3\tadvmod\t3:advmod\tSpaceAfter=No"
        )
        .unwrap();
        writeln!(file, "5\t.\t.\tPUNCT\t.\t_\t3\tpunct\t3:punct\t_").unwrap();
        writeln!(file).unwrap();
        writeln!(file, "# sent_id = train-002").unwrap();
        writeln!(file, "# text = Dogs bark loudly at cats.").unwrap();
        writeln!(
            file,
            "1\tDogs\tdog\tNOUN\tNNS\tNumber=Plur\t2\tnsubj\t2:nsubj\t_"
        )
        .unwrap();
        writeln!(
            file,
            "2\tbark\tbark\tVERB\tVBP\tMood=Ind|Number=Plur\t0\troot\t0:root\t_"
        )
        .unwrap();
        writeln!(
            file,
            "3\tloudly\tloudly\tADV\tRB\t_\t2\tadvmod\t2:advmod\t_"
        )
        .unwrap();
        writeln!(file, "4\tat\tat\tADP\tIN\t_\t5\tcase\t5:case\t_").unwrap();
        writeln!(
            file,
            "5\tcats\tcat\tNOUN\tNNS\tNumber=Plur\t2\tobl\t2:obl:at\tSpaceAfter=No"
        )
        .unwrap();
        writeln!(file, "6\t.\t.\tPUNCT\t.\t_\t2\tpunct\t2:punct\t_").unwrap();
        file
    }

    #[test]
    fn test_treebank_lemmatizer_training() {
        let test_file = create_test_conllu_data();
        let mut lemmatizer = TreebankLemmatizer::new().unwrap();

        lemmatizer.train_from_file(test_file.path()).unwrap();

        let stats = lemmatizer.get_stats();
        assert!(stats.total_mappings > 0);
        assert!(stats.unique_words > 0);
    }

    #[test]
    fn test_treebank_lemmatizer_functionality() {
        let test_file = create_test_conllu_data();
        let mut lemmatizer = TreebankLemmatizer::new().unwrap();
        lemmatizer.train_from_file(test_file.path()).unwrap();

        // Test learned mappings
        assert_eq!(lemmatizer.lemmatize("children"), "child");
        assert_eq!(lemmatizer.lemmatize("dogs"), "dog");
        assert_eq!(lemmatizer.lemmatize("cats"), "cat");
        assert_eq!(lemmatizer.lemmatize("running"), "run");

        // Test confidence scores
        let (lemma, confidence) = lemmatizer.lemmatize_with_confidence("children");
        assert_eq!(lemma, "child");
        assert!(confidence > 0.9); // Direct mapping should have high confidence
    }

    #[test]
    fn test_batch_lemmatization() {
        let test_file = create_test_conllu_data();
        let mut lemmatizer = TreebankLemmatizer::new().unwrap();
        lemmatizer.train_from_file(test_file.path()).unwrap();

        let words = vec![
            "children".to_string(),
            "running".to_string(),
            "dogs".to_string(),
        ];
        let lemmas = lemmatizer.lemmatize_batch(&words);

        assert_eq!(lemmas, vec!["child", "run", "dog"]);
    }

    #[test]
    fn test_fallback_to_simple_lemmatizer() {
        let test_file = create_test_conllu_data();
        let mut lemmatizer = TreebankLemmatizer::new().unwrap();
        lemmatizer.train_from_file(test_file.path()).unwrap();

        // Word not in training data should fall back
        let unknown_lemma = lemmatizer.lemmatize("unknown");
        assert!(!unknown_lemma.is_empty());
    }
}
