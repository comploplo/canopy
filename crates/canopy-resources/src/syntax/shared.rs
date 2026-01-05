//! Shared syntax utilities for POS tagging and dependency parsing.
//!
//! Contains common functions used by multiple syntax components:
//! - UPOS/DepRel string parsing
//! - Suffix-based lemmatization
//! - Dependency relation guessing
//! - Noun-finding helpers

use crate::lexicon::LexiconEngine;
use crate::tokenizer::RawToken;
use canopy::{DepRel, UPos};

/// Subordinating conjunctions (closed set).
pub const SUBORDINATING_CONJUNCTIONS: &[&str] = &[
    "if", "because", "although", "while", "when", "since", "unless", "until", "before", "after",
    "that", "whether",
];

/// Parse UPOS string to `UPos` enum.
#[must_use]
pub fn parse_upos(upos: &str) -> UPos {
    match upos.to_uppercase().as_str() {
        "ADJ" => UPos::Adj,
        "ADP" => UPos::Adp,
        "ADV" => UPos::Adv,
        "AUX" => UPos::Aux,
        "CCONJ" => UPos::Cconj,
        "DET" => UPos::Det,
        "INTJ" => UPos::Intj,
        "NOUN" => UPos::Noun,
        "NUM" => UPos::Num,
        "PART" => UPos::Part,
        "PRON" => UPos::Pron,
        "PROPN" => UPos::Propn,
        "PUNCT" => UPos::Punct,
        "SCONJ" => UPos::Sconj,
        "SYM" => UPos::Sym,
        "VERB" => UPos::Verb,
        _ => UPos::X,
    }
}

/// Parse deprel string to `DepRel` enum.
#[must_use]
pub fn parse_deprel(deprel: &str) -> DepRel {
    match deprel.to_lowercase().as_str() {
        "root" => DepRel::Root,
        "nsubj" => DepRel::Nsubj,
        "nsubj:pass" => DepRel::NsubjPass,
        "obj" => DepRel::Obj,
        "iobj" => DepRel::Iobj,
        "csubj" => DepRel::Csubj,
        "csubj:pass" => DepRel::CsubjPass,
        "ccomp" => DepRel::Ccomp,
        "xcomp" => DepRel::Xcomp,
        "obl" => DepRel::Obl,
        "vocative" => DepRel::Vocative,
        "expl" => DepRel::Expl,
        "dislocated" => DepRel::Dislocated,
        "advcl" => DepRel::Advcl,
        "advmod" => DepRel::Advmod,
        "discourse" => DepRel::Discourse,
        "aux" => DepRel::Aux,
        "aux:pass" => DepRel::AuxPass,
        "cop" => DepRel::Cop,
        "mark" => DepRel::Mark,
        "nmod" => DepRel::Nmod,
        "appos" => DepRel::Appos,
        "nummod" => DepRel::Nummod,
        "acl" => DepRel::Acl,
        "amod" => DepRel::Amod,
        "det" => DepRel::Det,
        "clf" => DepRel::Clf,
        "case" => DepRel::Case,
        "conj" => DepRel::Conj,
        "cc" => DepRel::Cc,
        "fixed" => DepRel::Fixed,
        "flat" => DepRel::Flat,
        "compound" => DepRel::Compound,
        "list" => DepRel::List,
        "parataxis" => DepRel::Parataxis,
        "orphan" => DepRel::Orphan,
        "goeswith" => DepRel::Goeswith,
        "reparandum" => DepRel::Reparandum,
        "punct" => DepRel::Punct,
        _ => DepRel::Dep,
    }
}

/// Apply suffix-based lemmatization rules.
///
/// Handles common English morphological patterns:
/// - `-ing` (running → run)
/// - `-ed` (walked → walk)
/// - `-ies` (tries → try)
/// - `-es` (watches → watch)
/// - `-s` (cats → cat)
#[must_use]
pub fn lemmatize_by_suffix(form: &str) -> String {
    let lower = form.to_lowercase();

    // -ing handling
    if lower.ends_with("ing") && lower.len() > 4 {
        let stem = &lower[..lower.len() - 3];
        if stem.ends_with('n') && stem.len() > 1 {
            // running -> run
            return stem[..stem.len() - 1].to_string();
        }
        return stem.to_string();
    }

    // -ed handling
    if lower.ends_with("ed") && lower.len() > 3 {
        let stem = &lower[..lower.len() - 2];
        if let Some(stripped) = stem.strip_suffix('i') {
            // tried -> try
            return format!("{stripped}y");
        }
        return stem.to_string();
    }

    // -ies handling
    if lower.ends_with("ies") && lower.len() > 4 {
        // tries -> try
        return format!("{}y", &lower[..lower.len() - 3]);
    }

    // -es handling
    if lower.ends_with("es") && lower.len() > 3 {
        if lower == "goes" || lower == "does" {
            return lower[..lower.len() - 2].to_string();
        }
        let stem = &lower[..lower.len() - 2];
        if stem.ends_with("sh")
            || stem.ends_with("ch")
            || stem.ends_with('x')
            || stem.ends_with('s')
            || stem.ends_with('z')
        {
            return stem.to_string();
        }
        return lower[..lower.len() - 1].to_string();
    }

    // -s handling
    if lower.ends_with('s') && lower.len() > 2 {
        return lower[..lower.len() - 1].to_string();
    }

    lower
}

/// Check if word is a content word (not function word or punctuation).
#[must_use]
pub fn is_content_word(form: &str, lexicon: &LexiconEngine) -> bool {
    let lower = form.to_lowercase();
    !lexicon.is_pronoun(&lower).unwrap_or(false)
        && !lexicon.is_preposition(&lower).unwrap_or(false)
        && !lexicon.is_conjunction(&lower).unwrap_or(false)
        && !lexicon.is_auxiliary(&lower).unwrap_or(false)
        && !form.chars().all(|c| c.is_ascii_punctuation())
}

/// Find next content word (noun-like) token after index.
#[must_use]
pub fn find_next_noun(idx: usize, tokens: &[RawToken], lexicon: &LexiconEngine) -> Option<usize> {
    for (i, token) in tokens.iter().enumerate().skip(idx + 1) {
        if is_content_word(&token.form, lexicon) {
            return Some(i);
        }
    }
    None
}

/// Find nearest content word to the given index (forward then backward).
#[must_use]
pub fn find_nearest_noun(
    idx: usize,
    tokens: &[RawToken],
    lexicon: &LexiconEngine,
) -> Option<usize> {
    // Check forward first
    if let Some(next) = find_next_noun(idx, tokens, lexicon) {
        return Some(next);
    }
    // Check backward
    (0..idx)
        .rev()
        .find(|&i| is_content_word(&tokens[i].form, lexicon))
}

/// Guess dependency relation based on POS and position.
#[must_use]
pub fn guess_dependency(
    idx: usize,
    root_idx: usize,
    upos: UPos,
    tokens: &[RawToken],
    lexicon: &LexiconEngine,
) -> (usize, DepRel) {
    match upos {
        UPos::Det => {
            let next = find_next_noun(idx, tokens, lexicon);
            (next.unwrap_or(root_idx), DepRel::Det)
        }
        UPos::Adj => {
            if idx < root_idx {
                let next = find_next_noun(idx, tokens, lexicon);
                (next.unwrap_or(root_idx), DepRel::Amod)
            } else {
                (root_idx, DepRel::Xcomp)
            }
        }
        UPos::Adv => (root_idx, DepRel::Advmod),
        UPos::Adp => (root_idx, DepRel::Case),
        UPos::Punct => (root_idx, DepRel::Punct),
        UPos::Noun | UPos::Propn | UPos::Pron => {
            if idx < root_idx {
                (root_idx, DepRel::Nsubj)
            } else {
                (root_idx, DepRel::Obj)
            }
        }
        UPos::Aux => (root_idx, DepRel::Aux),
        UPos::Cconj => (root_idx, DepRel::Cc),
        UPos::Sconj => (root_idx, DepRel::Mark),
        UPos::Num => {
            let nearest = find_nearest_noun(idx, tokens, lexicon);
            (nearest.unwrap_or(root_idx), DepRel::Nummod)
        }
        _ => (root_idx, DepRel::Dep),
    }
}

/// Apply suffix-based heuristics for unknown words.
#[must_use]
pub fn suffix_heuristics(word: &str, position: usize) -> UPos {
    let form = word;

    // Check for punctuation
    if form.len() == 1 {
        if let Some(c) = form.chars().next() {
            if c.is_ascii_punctuation() {
                return UPos::Punct;
            }
        }
    }

    // Check for numbers
    if form
        .chars()
        .all(|c| c.is_ascii_digit() || c == '.' || c == ',')
    {
        return UPos::Num;
    }

    // Check for proper nouns (capitalized, not sentence-initial)
    if position > 0 && form.chars().next().is_some_and(char::is_uppercase) {
        return UPos::Propn;
    }

    // Morphological clues for verbs
    if form.ends_with("ing") || form.ends_with("ed") {
        return UPos::Verb;
    }

    if form.ends_with("ly") && form.len() > 3 {
        return UPos::Adv;
    }

    // Common verb endings
    if form.ends_with("ize")
        || form.ends_with("ise")
        || form.ends_with("fy")
        || form.ends_with("ate")
    {
        return UPos::Verb;
    }

    // Third person singular verbs
    if form.ends_with('s') && form.len() > 3 {
        let stem = &form[..form.len() - 1];
        if stem.ends_with("un")
            || stem.ends_with("ive")
            || stem.ends_with("ake")
            || stem.ends_with("ow")
            || stem.ends_with("ay")
            || stem.ends_with("ee")
        {
            return UPos::Verb;
        }
    }

    // Position-based heuristics
    if position == 0 && form.chars().next().is_some_and(char::is_uppercase) {
        return UPos::Noun;
    }

    // Default to noun
    UPos::Noun
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_upos() {
        assert_eq!(parse_upos("VERB"), UPos::Verb);
        assert_eq!(parse_upos("verb"), UPos::Verb);
        assert_eq!(parse_upos("NOUN"), UPos::Noun);
        assert_eq!(parse_upos("DET"), UPos::Det);
        assert_eq!(parse_upos("unknown"), UPos::X);
    }

    #[test]
    fn test_parse_deprel() {
        assert_eq!(parse_deprel("root"), DepRel::Root);
        assert_eq!(parse_deprel("nsubj"), DepRel::Nsubj);
        assert_eq!(parse_deprel("nsubj:pass"), DepRel::NsubjPass);
        assert_eq!(parse_deprel("unknown"), DepRel::Dep);
    }

    #[test]
    fn test_lemmatize_by_suffix() {
        assert_eq!(lemmatize_by_suffix("running"), "run");
        assert_eq!(lemmatize_by_suffix("walked"), "walk");
        assert_eq!(lemmatize_by_suffix("tries"), "try");
        assert_eq!(lemmatize_by_suffix("goes"), "go");
        assert_eq!(lemmatize_by_suffix("cats"), "cat");
        assert_eq!(lemmatize_by_suffix("watches"), "watch");
    }

    #[test]
    fn test_suffix_heuristics() {
        assert_eq!(suffix_heuristics("running", 1), UPos::Verb);
        assert_eq!(suffix_heuristics("walked", 1), UPos::Verb);
        assert_eq!(suffix_heuristics("quickly", 1), UPos::Adv);
        assert_eq!(suffix_heuristics(".", 1), UPos::Punct);
        assert_eq!(suffix_heuristics("123", 1), UPos::Num);
        assert_eq!(suffix_heuristics("John", 1), UPos::Propn);
    }
}
