//! `WordNet` data loader
//!
//! This module handles loading `WordNet` data files (data.*, index.*, *.exc)
//! and building the complete `WordNet` database structure.

use super::parser::{utils, WordNetParser, WordNetParserConfig};
use super::types::{
    ExceptionEntry, IndexEntry, PartOfSpeech, SemanticPointer, Synset, SynsetWord, VerbFrame,
    WordNetDatabase,
};
use crate::engine::{EngineError, EngineResult};
use std::collections::HashMap;
use std::io::BufRead;
use std::path::Path;

/// `WordNet` data loader
#[derive(Debug)]
pub struct WordNetLoader {
    parser: WordNetParser,
}

impl WordNetLoader {
    /// Create a new `WordNet` loader
    #[must_use]
    pub fn new(config: WordNetParserConfig) -> Self {
        Self {
            parser: WordNetParser::with_config(config),
        }
    }

    /// Load complete `WordNet` database from data directory
    ///
    /// # Errors
    /// Returns an error if the data directory does not exist or data files cannot be parsed.
    pub fn load_database(&self, data_dir: &str) -> EngineResult<WordNetDatabase> {
        let data_path = Path::new(data_dir);
        if !data_path.exists() {
            return Err(EngineError::data_load(format!(
                "WordNet data directory not found: {data_dir}"
            )));
        }

        let mut database = WordNetDatabase::new();

        // Load synsets from data files
        for pos in &[
            PartOfSpeech::Noun,
            PartOfSpeech::Verb,
            PartOfSpeech::Adjective,
            PartOfSpeech::Adverb,
        ] {
            let data_file = data_path.join(format!("data.{}", pos.name()));
            if data_file.exists() {
                tracing::info!("Loading synsets from {}", data_file.display());
                let synsets = self.load_synsets(&data_file, *pos)?;
                for synset in synsets {
                    // Update synset_words reverse lookup
                    let words: Vec<String> = synset.words.iter().map(|w| w.word.clone()).collect();
                    database.synset_words.insert(synset.offset, words);
                    database.synsets.insert(synset.offset, synset);
                }
            }
        }

        // Load index entries
        for pos in &[
            PartOfSpeech::Noun,
            PartOfSpeech::Verb,
            PartOfSpeech::Adjective,
            PartOfSpeech::Adverb,
        ] {
            let index_file = data_path.join(format!("index.{}", pos.name()));
            if index_file.exists() {
                tracing::info!("Loading index from {}", index_file.display());
                let entries = self.load_index(&index_file, *pos)?;
                for entry in entries {
                    database.index.insert((entry.lemma.clone(), *pos), entry);
                }
            }
        }

        // Load exception lists
        for pos in &[
            PartOfSpeech::Noun,
            PartOfSpeech::Verb,
            PartOfSpeech::Adjective,
            PartOfSpeech::Adverb,
        ] {
            let exc_file = data_path.join(format!("{}.exc", pos.name()));
            if exc_file.exists() {
                tracing::info!("Loading exceptions from {}", exc_file.display());
                let exceptions = self.load_exceptions(&exc_file)?;
                database.exceptions.insert(*pos, exceptions);
            }
        }

        tracing::info!(
            "WordNet database loaded: {} synsets, {} index entries",
            database.synsets.len(),
            database.index.len()
        );

        Ok(database)
    }

    /// Load synsets from a data file
    fn load_synsets(&self, file_path: &Path, pos: PartOfSpeech) -> EngineResult<Vec<Synset>> {
        let mut synsets = Vec::new();

        self.parser.parse_file(file_path, |reader| {
            for line in reader.lines() {
                let line =
                    line.map_err(|e| EngineError::data_load(format!("Failed to read line: {e}")))?;

                // Skip license text and empty lines
                if utils::is_license_or_empty(&line) {
                    continue;
                }

                match Self::parse_synset_line(&line, pos) {
                    Ok(synset) => synsets.push(synset),
                    Err(e) => {
                        if self.parser.config().strict_mode {
                            return Err(e);
                        }
                        tracing::warn!("Failed to parse synset line: {}", e);
                    }
                }
            }
            Ok(synsets)
        })
    }

    /// Parse words from synset fields
    fn parse_words(
        fields: &[String],
        w_cnt: u16,
        field_idx: &mut usize,
    ) -> EngineResult<Vec<SynsetWord>> {
        let mut words = Vec::new();
        for _ in 0..w_cnt {
            if *field_idx >= fields.len() {
                return Err(EngineError::data_load("Not enough word fields".to_string()));
            }
            let word = fields[*field_idx].replace('_', " ");
            let lex_id = fields
                .get(*field_idx + 1)
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            words.push(SynsetWord {
                word,
                lex_id,
                tag_count: None,
            });
            *field_idx += 2;
        }
        Ok(words)
    }

    /// Parse pointers from synset fields
    fn parse_pointers(
        fields: &[String],
        p_cnt: u16,
        field_idx: &mut usize,
    ) -> EngineResult<Vec<SemanticPointer>> {
        let mut pointers = Vec::new();
        for _ in 0..p_cnt {
            if *field_idx + 3 >= fields.len() {
                return Err(EngineError::data_load(
                    "Not enough pointer fields".to_string(),
                ));
            }
            let relation = utils::parse_pointer_symbol(&fields[*field_idx])?;
            let target_offset = utils::parse_synset_offset(&fields[*field_idx + 1])?;
            let target_pos =
                utils::parse_pos(fields[*field_idx + 2].chars().next().unwrap_or('n'))?;
            let source_target = &fields[*field_idx + 3];
            let source_word = source_target.chars().next().map_or(0, |c| c as u8 - b'0');
            let target_word = source_target.chars().nth(1).map_or(0, |c| c as u8 - b'0');
            pointers.push(SemanticPointer {
                relation,
                target_offset,
                target_pos,
                source_word,
                target_word,
            });
            *field_idx += 4;
        }
        Ok(pointers)
    }

    /// Parse verb frames from synset fields
    fn parse_verb_frames(
        fields: &[String],
        pos: PartOfSpeech,
        field_idx: &mut usize,
    ) -> EngineResult<Vec<VerbFrame>> {
        let mut frames = Vec::new();
        if pos != PartOfSpeech::Verb || *field_idx >= fields.len() {
            return Ok(frames);
        }
        let Ok(f_cnt) = utils::parse_numeric_field::<u16>(&fields[*field_idx], "f_cnt") else {
            return Ok(frames);
        };
        *field_idx += 1;
        for _ in 0..f_cnt {
            if *field_idx + 1 >= fields.len() || fields[*field_idx] != "+" {
                *field_idx += 1;
                continue;
            }
            let frame_number =
                utils::parse_numeric_field::<u8>(&fields[*field_idx + 1], "frame_number")?;
            let word_number = fields
                .get(*field_idx + 2)
                .and_then(|s| utils::parse_numeric_field::<u8>(s, "word_number").ok())
                .unwrap_or(0);
            frames.push(VerbFrame {
                frame_number,
                word_number,
                template: format!("Frame {frame_number}"),
            });
            *field_idx += 3;
        }
        Ok(frames)
    }

    /// Parse a single synset line from data file
    fn parse_synset_line(line: &str, pos: PartOfSpeech) -> EngineResult<Synset> {
        let fields = utils::split_fields(line);
        if fields.len() < 6 {
            return Err(EngineError::data_load(
                "Invalid synset line: not enough fields".to_string(),
            ));
        }

        let offset = utils::parse_synset_offset(&fields[0])?;
        let lex_filenum = utils::parse_numeric_field::<u8>(&fields[1], "lex_filenum")?;
        let ss_type = utils::parse_pos(fields[2].chars().next().unwrap_or('n'))?;
        let w_cnt = utils::parse_numeric_field::<u16>(&fields[3], "w_cnt")?;

        let mut field_idx = 4;
        let words = Self::parse_words(&fields, w_cnt, &mut field_idx)?;

        if field_idx >= fields.len() {
            return Err(EngineError::data_load("Missing pointer count".to_string()));
        }
        let p_cnt = utils::parse_numeric_field::<u16>(&fields[field_idx], "p_cnt")?;
        field_idx += 1;

        let pointers = Self::parse_pointers(&fields, p_cnt, &mut field_idx)?;
        let frames = Self::parse_verb_frames(&fields, pos, &mut field_idx)?;
        let gloss = utils::extract_gloss(line).unwrap_or_default();

        Ok(Synset {
            offset,
            lex_filenum,
            pos: ss_type,
            words,
            pointers,
            frames,
            gloss,
        })
    }

    /// Load index entries from an index file
    fn load_index(&self, file_path: &Path, pos: PartOfSpeech) -> EngineResult<Vec<IndexEntry>> {
        let mut entries = Vec::new();

        self.parser.parse_file(file_path, |reader| {
            for line in reader.lines() {
                let line =
                    line.map_err(|e| EngineError::data_load(format!("Failed to read line: {e}")))?;

                // Skip license text and empty lines
                if utils::is_license_or_empty(&line) {
                    continue;
                }

                match Self::parse_index_line(&line, pos) {
                    Ok(entry) => entries.push(entry),
                    Err(e) => {
                        if self.parser.config().strict_mode {
                            return Err(e);
                        }
                        tracing::warn!("Failed to parse index line: {}", e);
                    }
                }
            }
            Ok(entries)
        })
    }

    /// Parse a single index line
    fn parse_index_line(line: &str, _pos: PartOfSpeech) -> EngineResult<IndexEntry> {
        let fields = utils::split_fields(line);

        if fields.len() < 4 {
            return Err(EngineError::data_load(
                "Invalid index line: not enough fields".to_string(),
            ));
        }

        let lemma = fields[0].replace('_', " ");
        let pos_char = fields[1].chars().next().unwrap_or('n');
        let entry_pos = utils::parse_pos(pos_char)?;
        let synset_count = utils::parse_numeric_field::<u32>(&fields[2], "synset_count")?;
        let pointer_count = utils::parse_numeric_field::<u32>(&fields[3], "pointer_count")?;

        // Parse pointer symbols
        let mut relations = Vec::new();
        let mut field_idx = 4;
        for _ in 0..pointer_count {
            if field_idx < fields.len() {
                if let Ok(relation) = utils::parse_pointer_symbol(&fields[field_idx]) {
                    relations.push(relation);
                }
                field_idx += 1;
            }
        }

        // Parse tag sense count
        let tag_sense_count = if field_idx < fields.len() {
            utils::parse_numeric_field::<u32>(&fields[field_idx], "tag_sense_count").unwrap_or(0)
        } else {
            0
        };
        if field_idx < fields.len() {
            field_idx += 1;
        }

        // Parse synset offsets
        let mut synset_offsets = Vec::new();
        for _ in 0..synset_count {
            if field_idx < fields.len() {
                if let Ok(offset) = utils::parse_synset_offset(&fields[field_idx]) {
                    synset_offsets.push(offset);
                }
                field_idx += 1;
            }
        }

        Ok(IndexEntry {
            lemma,
            pos: entry_pos,
            synset_count,
            pointer_count,
            relations,
            tag_sense_count,
            synset_offsets,
        })
    }

    /// Load exception entries from an exception file
    fn load_exceptions(&self, file_path: &Path) -> EngineResult<HashMap<String, ExceptionEntry>> {
        let mut exceptions = HashMap::new();

        self.parser.parse_file(file_path, |reader| {
            for line in reader.lines() {
                let line =
                    line.map_err(|e| EngineError::data_load(format!("Failed to read line: {e}")))?;

                // Skip license text and empty lines
                if utils::is_license_or_empty(&line) {
                    continue;
                }

                match Self::parse_exception_line(&line) {
                    Ok((key, entry)) => {
                        exceptions.insert(key, entry);
                    }
                    Err(e) => {
                        if self.parser.config().strict_mode {
                            return Err(e);
                        }
                        tracing::warn!("Failed to parse exception line: {}", e);
                    }
                }
            }
            Ok(exceptions)
        })
    }

    /// Parse a single exception line
    fn parse_exception_line(line: &str) -> EngineResult<(String, ExceptionEntry)> {
        let fields = utils::split_fields(line);

        if fields.len() < 2 {
            return Err(EngineError::data_load(
                "Invalid exception line: not enough fields".to_string(),
            ));
        }

        let inflected = fields[0].clone();
        let base_forms = fields[1..].to_vec();

        let entry = ExceptionEntry {
            inflected: inflected.clone(),
            base_forms,
        };

        Ok((inflected, entry))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wordnet_loader_new() {
        let config = WordNetParserConfig::default();
        let loader = WordNetLoader::new(config);
        // Just verify it constructs without panic
        let debug = format!("{loader:?}");
        assert!(debug.contains("WordNetLoader"));
    }

    #[test]
    fn test_wordnet_loader_load_nonexistent_dir() {
        let config = WordNetParserConfig::default();
        let loader = WordNetLoader::new(config);
        let result = loader.load_database("/nonexistent/path/to/wordnet");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn test_parse_exception_line_valid() {
        let result = WordNetLoader::parse_exception_line("ran run");
        assert!(result.is_ok());
        let (key, entry) = result.unwrap();
        assert_eq!(key, "ran");
        assert_eq!(entry.inflected, "ran");
        assert_eq!(entry.base_forms, vec!["run"]);
    }

    #[test]
    fn test_parse_exception_line_multiple_bases() {
        let result = WordNetLoader::parse_exception_line("better good well");
        assert!(result.is_ok());
        let (key, entry) = result.unwrap();
        assert_eq!(key, "better");
        assert_eq!(entry.base_forms, vec!["good", "well"]);
    }

    #[test]
    fn test_parse_exception_line_invalid() {
        let result = WordNetLoader::parse_exception_line("single");
        assert!(result.is_err());
    }

    #[test]
    fn test_part_of_speech_name() {
        assert_eq!(PartOfSpeech::Noun.name(), "noun");
        assert_eq!(PartOfSpeech::Verb.name(), "verb");
        assert_eq!(PartOfSpeech::Adjective.name(), "adjective");
        assert_eq!(PartOfSpeech::Adverb.name(), "adverb");
    }

    #[test]
    fn test_parse_index_line_valid() {
        // Format: lemma pos synset_count pointer_count [pointers] tag_sense_count [synset_offsets]
        let line = "dog n 1 1 @ 1 00000001";
        let result = WordNetLoader::parse_index_line(line, PartOfSpeech::Noun);
        assert!(result.is_ok());
        let entry = result.unwrap();
        assert_eq!(entry.lemma, "dog");
        assert_eq!(entry.pos, PartOfSpeech::Noun);
        assert_eq!(entry.synset_count, 1);
    }

    #[test]
    fn test_parse_index_line_insufficient_fields() {
        let line = "dog n 1"; // Missing pointer_count
        let result = WordNetLoader::parse_index_line(line, PartOfSpeech::Noun);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_index_line_with_underscore() {
        // Underscores should be converted to spaces
        let line = "hot_dog n 1 0 1 00000001";
        let result = WordNetLoader::parse_index_line(line, PartOfSpeech::Noun);
        assert!(result.is_ok());
        let entry = result.unwrap();
        assert_eq!(entry.lemma, "hot dog");
    }

    #[test]
    fn test_parse_synset_line_insufficient_fields() {
        let line = "00000001 01 n 01"; // Too few fields
        let result = WordNetLoader::parse_synset_line(line, PartOfSpeech::Noun);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_synset_line_basic() {
        // Minimal valid synset line: offset lex_filenum ss_type w_cnt word lex_id p_cnt | gloss
        let line = "00000001 01 n 01 dog 0 0 | a domestic animal";
        let result = WordNetLoader::parse_synset_line(line, PartOfSpeech::Noun);
        assert!(result.is_ok());
        let synset = result.unwrap();
        assert_eq!(synset.offset, 1);
        assert_eq!(synset.words.len(), 1);
        assert_eq!(synset.words[0].word, "dog");
        assert!(synset.gloss.contains("domestic"));
    }

    #[test]
    fn test_parse_synset_line_missing_word_fields() {
        // w_cnt says 2 words but only provides 1
        let line = "00000001 01 n 02 dog 0 0 | a domestic animal";
        let result = WordNetLoader::parse_synset_line(line, PartOfSpeech::Noun);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_synset_line_with_pointer() {
        // Synset with one hypernym pointer
        let line = "00000001 01 n 01 dog 0 1 @ 00000002 n 0000 | a domestic animal";
        let result = WordNetLoader::parse_synset_line(line, PartOfSpeech::Noun);
        assert!(result.is_ok());
        let synset = result.unwrap();
        assert_eq!(synset.pointers.len(), 1);
    }

    #[test]
    fn test_parse_synset_line_missing_pointer_fields() {
        // p_cnt says 1 pointer but incomplete pointer fields
        let line = "00000001 01 n 01 dog 0 1 @ | a domestic animal";
        let result = WordNetLoader::parse_synset_line(line, PartOfSpeech::Noun);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_synset_line_multiple_words() {
        let line = "00000001 01 n 02 dog 0 hound 0 0 | a domestic animal";
        let result = WordNetLoader::parse_synset_line(line, PartOfSpeech::Noun);
        assert!(result.is_ok());
        let synset = result.unwrap();
        assert_eq!(synset.words.len(), 2);
        assert_eq!(synset.words[0].word, "dog");
        assert_eq!(synset.words[1].word, "hound");
    }

    #[test]
    fn test_parse_synset_line_with_underscores() {
        let line = "00000001 01 n 01 hot_dog 0 0 | a frankfurter";
        let result = WordNetLoader::parse_synset_line(line, PartOfSpeech::Noun);
        assert!(result.is_ok());
        let synset = result.unwrap();
        assert_eq!(synset.words[0].word, "hot dog");
    }

    #[test]
    fn test_parse_synset_line_verb_with_frames() {
        // Verb with frame information
        let line = "00000001 01 v 01 run 0 0 1 + 01 00 | move fast by using legs";
        let result = WordNetLoader::parse_synset_line(line, PartOfSpeech::Verb);
        assert!(result.is_ok());
        let synset = result.unwrap();
        assert_eq!(synset.pos, PartOfSpeech::Verb);
        // Frames might be parsed depending on format
    }

    #[test]
    fn test_parse_index_line_no_pointers() {
        // 'r' is the correct code for adverb in WordNet
        let line = "quickly r 1 0 0 00000001";
        let result = WordNetLoader::parse_index_line(line, PartOfSpeech::Adverb);
        assert!(result.is_ok());
        let entry = result.unwrap();
        assert_eq!(entry.pointer_count, 0);
        assert!(entry.relations.is_empty());
    }

    #[test]
    fn test_parse_index_line_multiple_synsets() {
        let line = "run v 3 0 0 00000001 00000002 00000003";
        let result = WordNetLoader::parse_index_line(line, PartOfSpeech::Verb);
        assert!(result.is_ok());
        let entry = result.unwrap();
        assert_eq!(entry.synset_count, 3);
        assert_eq!(entry.synset_offsets.len(), 3);
    }
}
