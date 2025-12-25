//! WordNet data loader
//!
//! This module handles loading WordNet data files (data.*, index.*, *.exc)
//! and building the complete WordNet database structure.

use crate::parser::{WordNetParser, WordNetParserConfig, utils};
use crate::types::{
    ExceptionEntry, IndexEntry, PartOfSpeech, SemanticPointer, Synset, SynsetWord, VerbFrame,
    WordNetDatabase,
};
use canopy_engine::{EngineError, EngineResult};
use std::collections::HashMap;
use std::io::BufRead;
use std::path::Path;

/// WordNet data loader
#[derive(Debug)]
pub struct WordNetLoader {
    parser: WordNetParser,
}

impl WordNetLoader {
    /// Create a new WordNet loader
    pub fn new(config: WordNetParserConfig) -> Self {
        Self {
            parser: WordNetParser::with_config(config),
        }
    }

    /// Load complete WordNet database from data directory
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

                match self.parse_synset_line(&line, pos) {
                    Ok(synset) => synsets.push(synset),
                    Err(e) => {
                        if self.parser.config().strict_mode {
                            return Err(e);
                        } else {
                            tracing::warn!("Failed to parse synset line: {}", e);
                        }
                    }
                }
            }
            Ok(synsets)
        })
    }

    /// Parse a single synset line from data file
    fn parse_synset_line(&self, line: &str, pos: PartOfSpeech) -> EngineResult<Synset> {
        let fields = utils::split_fields(line);

        if fields.len() < 6 {
            return Err(EngineError::data_load(
                "Invalid synset line: not enough fields".to_string(),
            ));
        }

        // Parse basic synset info
        let offset = utils::parse_synset_offset(&fields[0])?;
        let lex_filenum = utils::parse_numeric_field::<u8>(&fields[1], "lex_filenum")?;
        let ss_type = utils::parse_pos(fields[2].chars().next().unwrap_or('n'))?;
        let w_cnt = utils::parse_numeric_field::<u16>(&fields[3], "w_cnt")?;

        // Parse words
        let mut words = Vec::new();
        let mut field_idx = 4;
        for _ in 0..w_cnt {
            if field_idx >= fields.len() {
                return Err(EngineError::data_load("Not enough word fields".to_string()));
            }

            let word = fields[field_idx].replace('_', " ");
            let lex_id = if field_idx + 1 < fields.len() {
                fields[field_idx + 1].parse().unwrap_or(0)
            } else {
                0
            };

            words.push(SynsetWord {
                word,
                lex_id,
                tag_count: None, // Will be populated from separate TagCount files if available
            });

            field_idx += 2; // word + lex_id
        }

        // Parse pointer count
        if field_idx >= fields.len() {
            return Err(EngineError::data_load("Missing pointer count".to_string()));
        }
        let p_cnt = utils::parse_numeric_field::<u16>(&fields[field_idx], "p_cnt")?;
        field_idx += 1;

        // Parse pointers
        let mut pointers = Vec::new();
        for _ in 0..p_cnt {
            if field_idx + 3 >= fields.len() {
                return Err(EngineError::data_load(
                    "Not enough pointer fields".to_string(),
                ));
            }

            let relation = utils::parse_pointer_symbol(&fields[field_idx])?;
            let target_offset = utils::parse_synset_offset(&fields[field_idx + 1])?;
            let target_pos = utils::parse_pos(fields[field_idx + 2].chars().next().unwrap_or('n'))?;
            let source_target = &fields[field_idx + 3];

            let source_word = if source_target.len() >= 2 {
                source_target.chars().next().unwrap_or('0') as u8 - b'0'
            } else {
                0
            };
            let target_word = if source_target.len() >= 2 {
                source_target.chars().nth(1).unwrap_or('0') as u8 - b'0'
            } else {
                0
            };

            pointers.push(SemanticPointer {
                relation,
                target_offset,
                target_pos,
                source_word,
                target_word,
            });

            field_idx += 4;
        }

        // Parse verb frames (only for verbs)
        let mut frames = Vec::new();
        if pos == PartOfSpeech::Verb
            && field_idx < fields.len()
            && let Ok(f_cnt) = utils::parse_numeric_field::<u16>(&fields[field_idx], "f_cnt")
        {
            field_idx += 1;

            for _ in 0..f_cnt {
                if field_idx + 1 < fields.len() {
                    if fields[field_idx] == "+" {
                        let frame_number = utils::parse_numeric_field::<u8>(
                            &fields[field_idx + 1],
                            "frame_number",
                        )?;
                        let word_number = if field_idx + 2 < fields.len() {
                            utils::parse_numeric_field::<u8>(&fields[field_idx + 2], "word_number")
                                .unwrap_or(0)
                        } else {
                            0
                        };

                        frames.push(VerbFrame {
                            frame_number,
                            word_number,
                            template: format!("Frame {frame_number}"), // Simplified
                        });

                        field_idx += 3;
                    } else {
                        field_idx += 1;
                    }
                } else {
                    break;
                }
            }
        }

        // Extract gloss
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

                match self.parse_index_line(&line, pos) {
                    Ok(entry) => entries.push(entry),
                    Err(e) => {
                        if self.parser.config().strict_mode {
                            return Err(e);
                        } else {
                            tracing::warn!("Failed to parse index line: {}", e);
                        }
                    }
                }
            }
            Ok(entries)
        })
    }

    /// Parse a single index line
    fn parse_index_line(&self, line: &str, _pos: PartOfSpeech) -> EngineResult<IndexEntry> {
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

                match self.parse_exception_line(&line) {
                    Ok((key, entry)) => {
                        exceptions.insert(key, entry);
                    }
                    Err(e) => {
                        if self.parser.config().strict_mode {
                            return Err(e);
                        } else {
                            tracing::warn!("Failed to parse exception line: {}", e);
                        }
                    }
                }
            }
            Ok(exceptions)
        })
    }

    /// Parse a single exception line
    fn parse_exception_line(&self, line: &str) -> EngineResult<(String, ExceptionEntry)> {
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
