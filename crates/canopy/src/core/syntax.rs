//! Syntactic categories following Universal Dependencies.
//!
//! These types mirror the UD specification for interoperability.

use serde::{Deserialize, Serialize};

/// Universal part-of-speech tags (UPOS).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum UPos {
    Adj,   // adjective
    Adp,   // adposition
    Adv,   // adverb
    Aux,   // auxiliary
    Cconj, // coordinating conjunction
    Det,   // determiner
    Intj,  // interjection
    Noun,  // noun
    Num,   // numeral
    Part,  // particle
    Pron,  // pronoun
    Propn, // proper noun
    Punct, // punctuation
    Sconj, // subordinating conjunction
    Sym,   // symbol
    Verb,  // verb
    #[default]
    X, // other
}

impl UPos {
    /// Check if this is a content word POS.
    #[must_use]
    pub const fn is_content_word(&self) -> bool {
        matches!(
            self,
            UPos::Noun | UPos::Verb | UPos::Adj | UPos::Adv | UPos::Propn
        )
    }

    /// Check if this is a verbal POS (verb or auxiliary).
    #[must_use]
    pub const fn is_verbal(&self) -> bool {
        matches!(self, UPos::Verb | UPos::Aux)
    }

    /// Check if this is a nominal POS.
    #[must_use]
    pub const fn is_nominal(&self) -> bool {
        matches!(self, UPos::Noun | UPos::Propn | UPos::Pron)
    }
}

/// Universal Dependencies dependency relations.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum DepRel {
    // Core arguments
    Nsubj,     // nominal subject
    NsubjPass, // passive nominal subject
    Obj,       // direct object
    Iobj,      // indirect object
    Csubj,     // clausal subject
    CsubjPass, // clausal passive subject
    Ccomp,     // clausal complement
    Xcomp,     // open clausal complement

    // Non-core dependents
    Obl, // oblique nominal
    Vocative,
    Expl, // expletive
    Dislocated,
    Advcl,  // adverbial clause
    Advmod, // adverbial modifier
    Discourse,
    Aux,     // auxiliary
    AuxPass, // passive auxiliary
    Cop,     // copula
    Mark,    // marker
    Nmod,    // nominal modifier
    Appos,   // appositional modifier
    Nummod,  // numeric modifier
    Acl,     // clausal modifier of noun
    Amod,    // adjectival modifier
    Det,     // determiner
    Clf,     // classifier
    Case,    // case marking

    // Coordination
    Conj, // conjunct
    Cc,   // coordinating conjunction

    // Other
    Fixed,       // fixed multiword expression
    Flat,        // flat multiword expression
    Compound,    // compound (generic)
    CompoundPrt, // compound:prt - verb particle
    List,
    Parataxis,
    Orphan,
    Goeswith,
    Reparandum,
    Punct, // punctuation
    #[default]
    Root, // root
    Dep,   // unspecified dependency
    Neg,   // negation modifier
    Other(String),
}

impl DepRel {
    /// Check if this is a subject relation.
    #[must_use]
    pub const fn is_subject(&self) -> bool {
        matches!(
            self,
            DepRel::Nsubj | DepRel::NsubjPass | DepRel::Csubj | DepRel::CsubjPass
        )
    }

    /// Check if this is an object relation.
    #[must_use]
    pub const fn is_object(&self) -> bool {
        matches!(self, DepRel::Obj | DepRel::Iobj)
    }

    /// Check if this is a core argument relation.
    #[must_use]
    pub const fn is_core_argument(&self) -> bool {
        self.is_subject() || self.is_object() || matches!(self, DepRel::Ccomp | DepRel::Xcomp)
    }

    /// Check if this is a verb particle relation.
    #[must_use]
    pub const fn is_particle(&self) -> bool {
        matches!(self, DepRel::CompoundPrt)
    }

    /// Check if this is a compound/MWE relation.
    #[must_use]
    pub const fn is_mwe(&self) -> bool {
        matches!(
            self,
            DepRel::Compound | DepRel::CompoundPrt | DepRel::Flat | DepRel::Fixed
        )
    }

    /// Parse from string representation.
    #[must_use]
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
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
            "compound:prt" => DepRel::CompoundPrt,
            "list" => DepRel::List,
            "parataxis" => DepRel::Parataxis,
            "orphan" => DepRel::Orphan,
            "goeswith" => DepRel::Goeswith,
            "reparandum" => DepRel::Reparandum,
            "punct" => DepRel::Punct,
            "root" => DepRel::Root,
            "dep" => DepRel::Dep,
            "neg" => DepRel::Neg,
            other => DepRel::Other(other.to_string()),
        }
    }
}

/// Morphological features following UD specification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct MorphFeatures {
    pub person: Option<Person>,
    pub number: Option<Number>,
    pub tense: Option<Tense>,
    pub mood: Option<Mood>,
    pub voice: Option<MorphVoice>,
    pub verb_form: Option<VerbForm>,
    pub case: Option<Case>,
    pub gender: Option<Gender>,
    pub definiteness: Option<Definiteness>,
    /// For gerunds: nominal, verbal, or adjectival usage.
    pub gerund_usage: Option<GerundUsage>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Person {
    First,
    Second,
    Third,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Number {
    Singular,
    Plural,
    Dual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tense {
    Past,
    Present,
    Future,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mood {
    Indicative,
    Imperative,
    Subjunctive,
    Conditional,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MorphVoice {
    Active,
    Passive,
    Middle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerbForm {
    Finite,
    Infinitive,
    Participle,
    Gerund,
}

/// Usage type for gerunds (-ing forms).
///
/// Distinguishes how a gerund is functioning syntactically,
/// which affects semantic interpretation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GerundUsage {
    /// Nominal gerund: "Swimming is fun" (subject/object position)
    Nominal,
    /// Verbal/progressive: "He is swimming"
    Verbal,
    /// Adjectival/participial: "The boring lecture"
    Adjectival,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Case {
    Nominative,
    Accusative,
    Dative,
    Genitive,
    Instrumental,
    Locative,
    Vocative,
    Ablative,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Gender {
    Masculine,
    Feminine,
    Neuter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Definiteness {
    Definite,
    Indefinite,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upos_categories() {
        assert!(UPos::Verb.is_content_word());
        assert!(UPos::Verb.is_verbal());
        assert!(!UPos::Verb.is_nominal());

        assert!(UPos::Noun.is_nominal());
        assert!(!UPos::Det.is_content_word());
    }

    #[test]
    fn test_upos_all_variants() {
        // Content words
        assert!(UPos::Adj.is_content_word());
        assert!(UPos::Adv.is_content_word());
        assert!(UPos::Noun.is_content_word());
        assert!(UPos::Verb.is_content_word());
        assert!(UPos::Propn.is_content_word());

        // Non-content words
        assert!(!UPos::Adp.is_content_word());
        assert!(!UPos::Aux.is_content_word());
        assert!(!UPos::Cconj.is_content_word());
        assert!(!UPos::Det.is_content_word());
        assert!(!UPos::Intj.is_content_word());
        assert!(!UPos::Num.is_content_word());
        assert!(!UPos::Part.is_content_word());
        assert!(!UPos::Pron.is_content_word());
        assert!(!UPos::Punct.is_content_word());
        assert!(!UPos::Sconj.is_content_word());
        assert!(!UPos::Sym.is_content_word());
        assert!(!UPos::X.is_content_word());
    }

    #[test]
    fn test_upos_verbal() {
        assert!(UPos::Verb.is_verbal());
        assert!(UPos::Aux.is_verbal());
        assert!(!UPos::Noun.is_verbal());
        assert!(!UPos::Adj.is_verbal());
    }

    #[test]
    fn test_upos_nominal() {
        assert!(UPos::Noun.is_nominal());
        assert!(UPos::Propn.is_nominal());
        assert!(UPos::Pron.is_nominal());
        assert!(!UPos::Verb.is_nominal());
        assert!(!UPos::Adj.is_nominal());
    }

    #[test]
    fn test_upos_default() {
        let default_pos = UPos::default();
        assert_eq!(default_pos, UPos::X);
    }

    #[test]
    fn test_deprel_categories() {
        assert!(DepRel::Nsubj.is_subject());
        assert!(DepRel::Obj.is_object());
        assert!(DepRel::Nsubj.is_core_argument());
        assert!(!DepRel::Advmod.is_core_argument());
    }

    #[test]
    fn test_deprel_subject_variants() {
        assert!(DepRel::Nsubj.is_subject());
        assert!(DepRel::NsubjPass.is_subject());
        assert!(DepRel::Csubj.is_subject());
        assert!(DepRel::CsubjPass.is_subject());
        assert!(!DepRel::Obj.is_subject());
    }

    #[test]
    fn test_deprel_object_variants() {
        assert!(DepRel::Obj.is_object());
        assert!(DepRel::Iobj.is_object());
        assert!(!DepRel::Nsubj.is_object());
    }

    #[test]
    fn test_deprel_core_argument() {
        assert!(DepRel::Nsubj.is_core_argument());
        assert!(DepRel::Obj.is_core_argument());
        assert!(DepRel::Ccomp.is_core_argument());
        assert!(DepRel::Xcomp.is_core_argument());
        assert!(!DepRel::Advmod.is_core_argument());
        assert!(!DepRel::Obl.is_core_argument());
    }

    #[test]
    fn test_deprel_from_str_all() {
        // Core arguments
        assert_eq!(DepRel::parse("nsubj"), DepRel::Nsubj);
        assert_eq!(DepRel::parse("nsubj:pass"), DepRel::NsubjPass);
        assert_eq!(DepRel::parse("obj"), DepRel::Obj);
        assert_eq!(DepRel::parse("iobj"), DepRel::Iobj);
        assert_eq!(DepRel::parse("csubj"), DepRel::Csubj);
        assert_eq!(DepRel::parse("csubj:pass"), DepRel::CsubjPass);
        assert_eq!(DepRel::parse("ccomp"), DepRel::Ccomp);
        assert_eq!(DepRel::parse("xcomp"), DepRel::Xcomp);

        // Non-core dependents
        assert_eq!(DepRel::parse("obl"), DepRel::Obl);
        assert_eq!(DepRel::parse("vocative"), DepRel::Vocative);
        assert_eq!(DepRel::parse("expl"), DepRel::Expl);
        assert_eq!(DepRel::parse("dislocated"), DepRel::Dislocated);
        assert_eq!(DepRel::parse("advcl"), DepRel::Advcl);
        assert_eq!(DepRel::parse("advmod"), DepRel::Advmod);
        assert_eq!(DepRel::parse("discourse"), DepRel::Discourse);
        assert_eq!(DepRel::parse("aux"), DepRel::Aux);
        assert_eq!(DepRel::parse("aux:pass"), DepRel::AuxPass);
        assert_eq!(DepRel::parse("cop"), DepRel::Cop);
        assert_eq!(DepRel::parse("mark"), DepRel::Mark);
        assert_eq!(DepRel::parse("nmod"), DepRel::Nmod);
        assert_eq!(DepRel::parse("appos"), DepRel::Appos);
        assert_eq!(DepRel::parse("nummod"), DepRel::Nummod);
        assert_eq!(DepRel::parse("acl"), DepRel::Acl);
        assert_eq!(DepRel::parse("amod"), DepRel::Amod);
        assert_eq!(DepRel::parse("det"), DepRel::Det);
        assert_eq!(DepRel::parse("clf"), DepRel::Clf);
        assert_eq!(DepRel::parse("case"), DepRel::Case);

        // Coordination
        assert_eq!(DepRel::parse("conj"), DepRel::Conj);
        assert_eq!(DepRel::parse("cc"), DepRel::Cc);

        // Other
        assert_eq!(DepRel::parse("fixed"), DepRel::Fixed);
        assert_eq!(DepRel::parse("flat"), DepRel::Flat);
        assert_eq!(DepRel::parse("compound"), DepRel::Compound);
        assert_eq!(DepRel::parse("list"), DepRel::List);
        assert_eq!(DepRel::parse("parataxis"), DepRel::Parataxis);
        assert_eq!(DepRel::parse("orphan"), DepRel::Orphan);
        assert_eq!(DepRel::parse("goeswith"), DepRel::Goeswith);
        assert_eq!(DepRel::parse("reparandum"), DepRel::Reparandum);
        assert_eq!(DepRel::parse("punct"), DepRel::Punct);
        assert_eq!(DepRel::parse("root"), DepRel::Root);
        assert_eq!(DepRel::parse("dep"), DepRel::Dep);
        assert_eq!(DepRel::parse("neg"), DepRel::Neg);

        // Unknown
        assert_eq!(DepRel::parse("custom"), DepRel::Other("custom".to_string()));
    }

    #[test]
    fn test_deprel_from_str_case_insensitive() {
        assert_eq!(DepRel::parse("NSUBJ"), DepRel::Nsubj);
        assert_eq!(DepRel::parse("Obj"), DepRel::Obj);
    }

    #[test]
    fn test_deprel_default() {
        let default_rel = DepRel::default();
        assert_eq!(default_rel, DepRel::Root);
    }

    #[test]
    fn test_morph_features_default() {
        let features = MorphFeatures::default();
        assert!(features.person.is_none());
        assert!(features.number.is_none());
        assert!(features.tense.is_none());
        assert!(features.mood.is_none());
        assert!(features.voice.is_none());
        assert!(features.verb_form.is_none());
        assert!(features.case.is_none());
        assert!(features.gender.is_none());
        assert!(features.definiteness.is_none());
    }

    #[test]
    fn test_morph_features_with_values() {
        let features = MorphFeatures {
            person: Some(Person::First),
            number: Some(Number::Singular),
            tense: Some(Tense::Present),
            mood: Some(Mood::Indicative),
            voice: Some(MorphVoice::Active),
            verb_form: Some(VerbForm::Finite),
            case: Some(Case::Nominative),
            gender: Some(Gender::Masculine),
            definiteness: Some(Definiteness::Definite),
            gerund_usage: None,
        };

        assert_eq!(features.person, Some(Person::First));
        assert_eq!(features.number, Some(Number::Singular));
        assert_eq!(features.tense, Some(Tense::Present));
        assert_eq!(features.mood, Some(Mood::Indicative));
        assert_eq!(features.voice, Some(MorphVoice::Active));
        assert_eq!(features.verb_form, Some(VerbForm::Finite));
        assert_eq!(features.case, Some(Case::Nominative));
        assert_eq!(features.gender, Some(Gender::Masculine));
        assert_eq!(features.definiteness, Some(Definiteness::Definite));
    }

    #[test]
    fn test_person_variants() {
        assert_eq!(format!("{:?}", Person::First), "First");
        assert_eq!(format!("{:?}", Person::Second), "Second");
        assert_eq!(format!("{:?}", Person::Third), "Third");
    }

    #[test]
    fn test_number_variants() {
        assert_eq!(format!("{:?}", Number::Singular), "Singular");
        assert_eq!(format!("{:?}", Number::Plural), "Plural");
        assert_eq!(format!("{:?}", Number::Dual), "Dual");
    }

    #[test]
    fn test_tense_variants() {
        assert_eq!(format!("{:?}", Tense::Past), "Past");
        assert_eq!(format!("{:?}", Tense::Present), "Present");
        assert_eq!(format!("{:?}", Tense::Future), "Future");
    }

    #[test]
    fn test_mood_variants() {
        assert_eq!(format!("{:?}", Mood::Indicative), "Indicative");
        assert_eq!(format!("{:?}", Mood::Imperative), "Imperative");
        assert_eq!(format!("{:?}", Mood::Subjunctive), "Subjunctive");
        assert_eq!(format!("{:?}", Mood::Conditional), "Conditional");
    }

    #[test]
    fn test_morph_voice_variants() {
        assert_eq!(format!("{:?}", MorphVoice::Active), "Active");
        assert_eq!(format!("{:?}", MorphVoice::Passive), "Passive");
        assert_eq!(format!("{:?}", MorphVoice::Middle), "Middle");
    }

    #[test]
    fn test_verb_form_variants() {
        assert_eq!(format!("{:?}", VerbForm::Finite), "Finite");
        assert_eq!(format!("{:?}", VerbForm::Infinitive), "Infinitive");
        assert_eq!(format!("{:?}", VerbForm::Participle), "Participle");
        assert_eq!(format!("{:?}", VerbForm::Gerund), "Gerund");
    }

    #[test]
    fn test_case_variants() {
        assert_eq!(format!("{:?}", Case::Nominative), "Nominative");
        assert_eq!(format!("{:?}", Case::Accusative), "Accusative");
        assert_eq!(format!("{:?}", Case::Dative), "Dative");
        assert_eq!(format!("{:?}", Case::Genitive), "Genitive");
        assert_eq!(format!("{:?}", Case::Instrumental), "Instrumental");
        assert_eq!(format!("{:?}", Case::Locative), "Locative");
        assert_eq!(format!("{:?}", Case::Vocative), "Vocative");
        assert_eq!(format!("{:?}", Case::Ablative), "Ablative");
    }

    #[test]
    fn test_gender_variants() {
        assert_eq!(format!("{:?}", Gender::Masculine), "Masculine");
        assert_eq!(format!("{:?}", Gender::Feminine), "Feminine");
        assert_eq!(format!("{:?}", Gender::Neuter), "Neuter");
    }

    #[test]
    fn test_definiteness_variants() {
        assert_eq!(format!("{:?}", Definiteness::Definite), "Definite");
        assert_eq!(format!("{:?}", Definiteness::Indefinite), "Indefinite");
    }
}
