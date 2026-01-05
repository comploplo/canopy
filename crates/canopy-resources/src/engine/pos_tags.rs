//! Universal Dependencies POS tag constants.
//!
//! Defines constants for UD UPOS (universal part-of-speech) tags.
//! These are standardized tags that shouldn't change, so constants
//! are appropriate rather than loading from a data file.
//!
//! Reference: <https://universaldependencies.org/u/pos>/

/// Verb-related POS tags
pub const VERB_POS: &[&str] = &["VERB", "AUX"];

/// Noun-related POS tags
pub const NOUN_POS: &[&str] = &["NOUN", "PROPN"];

/// Adjective POS tag
pub const ADJ: &str = "ADJ";

/// Adverb POS tag
pub const ADV: &str = "ADV";

/// Adposition (preposition/postposition) POS tag
pub const ADP: &str = "ADP";

/// Auxiliary verb POS tag
pub const AUX: &str = "AUX";

/// Coordinating conjunction POS tag
pub const CCONJ: &str = "CCONJ";

/// Determiner POS tag
pub const DET: &str = "DET";

/// Interjection POS tag
pub const INTJ: &str = "INTJ";

/// Noun POS tag
pub const NOUN: &str = "NOUN";

/// Numeral POS tag
pub const NUM: &str = "NUM";

/// Particle POS tag
pub const PART: &str = "PART";

/// Pronoun POS tag
pub const PRON: &str = "PRON";

/// Proper noun POS tag
pub const PROPN: &str = "PROPN";

/// Punctuation POS tag
pub const PUNCT: &str = "PUNCT";

/// Subordinating conjunction POS tag
pub const SCONJ: &str = "SCONJ";

/// Symbol POS tag
pub const SYM: &str = "SYM";

/// Verb POS tag
pub const VERB: &str = "VERB";

/// Other/unknown POS tag
pub const X: &str = "X";

/// Check if a POS tag represents a verb (VERB or AUX)
#[must_use]
pub fn is_verb_pos(pos: &str) -> bool {
    VERB_POS.contains(&pos)
}

/// Check if a POS tag represents a noun (NOUN or PROPN)
#[must_use]
pub fn is_noun_pos(pos: &str) -> bool {
    NOUN_POS.contains(&pos)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_verb_pos() {
        assert!(is_verb_pos("VERB"));
        assert!(is_verb_pos("AUX"));
        assert!(!is_verb_pos("NOUN"));
        assert!(!is_verb_pos("ADJ"));
    }

    #[test]
    fn test_is_noun_pos() {
        assert!(is_noun_pos("NOUN"));
        assert!(is_noun_pos("PROPN"));
        assert!(!is_noun_pos("VERB"));
        assert!(!is_noun_pos("ADJ"));
    }

    #[test]
    fn test_verb_pos_array() {
        assert_eq!(VERB_POS.len(), 2);
        assert!(VERB_POS.contains(&"VERB"));
        assert!(VERB_POS.contains(&"AUX"));
    }

    #[test]
    fn test_noun_pos_array() {
        assert_eq!(NOUN_POS.len(), 2);
        assert!(NOUN_POS.contains(&"NOUN"));
        assert!(NOUN_POS.contains(&"PROPN"));
    }
}
