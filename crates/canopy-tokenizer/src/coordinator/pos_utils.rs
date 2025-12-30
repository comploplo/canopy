//! POS conversion and inference utilities
//!
//! Functions for converting between POS tag formats and inferring POS from morphology.

use canopy_core::UPos;
use canopy_semantic_engines::wordnet::PartOfSpeech;

/// Convert Universal Dependencies POS to WordNet PartOfSpeech
/// Returns None for function words that don't have WordNet entries
pub fn upos_to_wordnet_pos(upos: UPos) -> Option<PartOfSpeech> {
    match upos {
        UPos::Adj => Some(PartOfSpeech::Adjective),
        UPos::Adv => Some(PartOfSpeech::Adverb),
        UPos::Verb | UPos::Aux => Some(PartOfSpeech::Verb),
        UPos::Noun | UPos::Propn => Some(PartOfSpeech::Noun),
        // Function words and others don't have WordNet entries
        _ => None,
    }
}

/// Convert WordNet PartOfSpeech to Universal Dependencies POS
pub fn wordnet_pos_to_upos(pos: PartOfSpeech) -> UPos {
    match pos {
        PartOfSpeech::Noun => UPos::Noun,
        PartOfSpeech::Verb => UPos::Verb,
        PartOfSpeech::Adjective | PartOfSpeech::AdjectiveSatellite => UPos::Adj,
        PartOfSpeech::Adverb => UPos::Adv,
    }
}

/// Check if this POS should query VerbNet (verbs/auxiliaries only)
pub fn should_query_verbnet(upos: Option<UPos>) -> bool {
    matches!(upos, Some(UPos::Verb) | Some(UPos::Aux) | None)
}

/// Check if this POS should query FrameNet (content words)
pub fn should_query_framenet(upos: Option<UPos>) -> bool {
    matches!(
        upos,
        Some(UPos::Verb)
            | Some(UPos::Aux)
            | Some(UPos::Noun)
            | Some(UPos::Propn)
            | Some(UPos::Adj)
            | None
    )
}

/// Guess likely POS from word suffix (for WordNet optimization)
/// Returns the most likely POS based on common English morphological patterns
pub fn guess_pos_from_suffix(word: &str) -> Option<PartOfSpeech> {
    let w = word.to_lowercase();
    let len = w.len();

    if len < 3 {
        return None;
    }

    // Adverb suffixes (check first - most distinctive)
    if w.ends_with("ly") && len > 4 {
        return Some(PartOfSpeech::Adverb);
    }

    // Verb suffixes
    if w.ends_with("ing") && len > 5 {
        return Some(PartOfSpeech::Verb);
    }
    if w.ends_with("ed") && len > 4 && !w.ends_with("eed") {
        return Some(PartOfSpeech::Verb);
    }
    if w.ends_with("ize") || w.ends_with("ise") || w.ends_with("ate") {
        return Some(PartOfSpeech::Verb);
    }

    // Noun suffixes
    if w.ends_with("tion") || w.ends_with("sion") || w.ends_with("ment") {
        return Some(PartOfSpeech::Noun);
    }
    if w.ends_with("ness") || w.ends_with("ity") || w.ends_with("ance") || w.ends_with("ence") {
        return Some(PartOfSpeech::Noun);
    }
    if (w.ends_with("er") || w.ends_with("or")) && len > 4 {
        return Some(PartOfSpeech::Noun); // agent nouns
    }

    // Adjective suffixes
    if w.ends_with("ful") || w.ends_with("less") || w.ends_with("ous") || w.ends_with("ive") {
        return Some(PartOfSpeech::Adjective);
    }
    if w.ends_with("able") || w.ends_with("ible") || w.ends_with("ical") {
        return Some(PartOfSpeech::Adjective);
    }

    None
}

/// All WordNet POS types for parallel querying
pub const WORDNET_ALL_POS: [PartOfSpeech; 4] = [
    PartOfSpeech::Noun,
    PartOfSpeech::Verb,
    PartOfSpeech::Adjective,
    PartOfSpeech::Adverb,
];

/// Confidence threshold for early exit when suffix heuristics match
pub const WORDNET_EARLY_EXIT_CONFIDENCE: f32 = 0.7;
