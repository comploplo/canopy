//! Test fixtures for semantic analysis engines
//!
//! This module provides mock data for testing semantic engines without
//! requiring full database loads or external dependencies.

use crate::*;
use crate::verbnet::{VerbClass, ThetaRole, ThetaRoleType};
use std::collections::HashMap;

/// Test data provider for semantic engines
pub struct TestFixtures {
    pub verbnet_data: HashMap<String, Vec<VerbClass>>,
    pub framenet_data: HashMap<String, Vec<FrameUnit>>,
    pub wordnet_data: HashMap<String, Vec<WordNetSense>>,
}

impl TestFixtures {
    /// Create test fixtures with common test vocabulary
    pub fn new() -> Self {
        Self {
            verbnet_data: create_test_verbnet_data(),
            framenet_data: create_test_framenet_data(),
            wordnet_data: create_test_wordnet_data(),
        }
    }

    /// Get VerbNet classes for a lemma
    pub fn get_verbnet_classes(&self, lemma: &str) -> Vec<VerbClass> {
        self.verbnet_data.get(lemma).cloned().unwrap_or_default()
    }

    /// Get FrameNet frames for a lemma
    pub fn get_framenet_frames(&self, lemma: &str) -> Vec<FrameUnit> {
        self.framenet_data.get(lemma).cloned().unwrap_or_default()
    }

    /// Get WordNet senses for a lemma
    pub fn get_wordnet_senses(&self, lemma: &str) -> Vec<WordNetSense> {
        self.wordnet_data.get(lemma).cloned().unwrap_or_default()
    }
}

impl Default for TestFixtures {
    fn default() -> Self {
        Self::new()
    }
}

/// Create test VerbNet data with common verbs
fn create_test_verbnet_data() -> HashMap<String, Vec<VerbClass>> {
    let mut data = HashMap::new();

    // "give" - give-13.1 class
    data.insert("give".to_string(), vec![VerbClass {
        id: "give-13.1".to_string(),
        name: "Give".to_string(),
        members: vec![
            VerbMember { name: "give".to_string(), wn_sense: None, fn_mapping: None, grouping: None },
            VerbMember { name: "hand".to_string(), wn_sense: None, fn_mapping: None, grouping: None },
            VerbMember { name: "pass".to_string(), wn_sense: None, fn_mapping: None, grouping: None },
        ],
        theta_roles: vec![
            ThetaRole { role_type: ThetaRoleType::Agent, selectional_restrictions: vec![], syntax_restrictions: vec![] },
            ThetaRole { role_type: ThetaRoleType::Patient, selectional_restrictions: vec![], syntax_restrictions: vec![] },
            ThetaRole { role_type: ThetaRoleType::Recipient, selectional_restrictions: vec![], syntax_restrictions: vec![] },
        ],
        frames: vec![],
        subclasses: vec![],
    }]);

    // "run" - run-51.3.2 class
    data.insert("run".to_string(), vec![VerbClass {
        id: "run-51.3.2".to_string(),
        name: "Run".to_string(),
        members: vec![
            VerbMember { name: "run".to_string(), wn_sense: None, fn_mapping: None, grouping: None },
            VerbMember { name: "jog".to_string(), wn_sense: None, fn_mapping: None, grouping: None },
            VerbMember { name: "sprint".to_string(), wn_sense: None, fn_mapping: None, grouping: None },
        ],
        theta_roles: vec![
            ThetaRole { role_type: ThetaRoleType::Agent, selectional_restrictions: vec![], syntax_restrictions: vec![] },
            ThetaRole { role_type: ThetaRoleType::Location, selectional_restrictions: vec![], syntax_restrictions: vec![] },
        ],
        frames: vec![],
        subclasses: vec![],
    }]);

    // "walk" - also run-51.3.2 class (motion verbs)
    data.insert("walk".to_string(), vec![VerbClass {
        id: "run-51.3.2".to_string(),
        name: "Walk".to_string(),
        members: vec![
            VerbMember { name: "walk".to_string(), wn_sense: None, fn_mapping: None, grouping: None },
            VerbMember { name: "stroll".to_string(), wn_sense: None, fn_mapping: None, grouping: None },
            VerbMember { name: "march".to_string(), wn_sense: None, fn_mapping: None, grouping: None },
        ],
        theta_roles: vec![
            ThetaRole { role_type: ThetaRoleType::Agent, selectional_restrictions: vec![], syntax_restrictions: vec![] },
            ThetaRole { role_type: ThetaRoleType::Location, selectional_restrictions: vec![], syntax_restrictions: vec![] },
        ],
        frames: vec![],
        subclasses: vec![],
    }]);

    // "love" - love-31.2 class
    data.insert("love".to_string(), vec![VerbClass {
        id: "love-31.2".to_string(),
        name: "Love".to_string(),
        members: vec![
            VerbMember { name: "love".to_string(), wn_sense: None, fn_mapping: None, grouping: None },
            VerbMember { name: "adore".to_string(), wn_sense: None, fn_mapping: None, grouping: None },
            VerbMember { name: "cherish".to_string(), wn_sense: None, fn_mapping: None, grouping: None },
        ],
        theta_roles: vec![
            ThetaRole { role_type: ThetaRoleType::Experiencer, selectional_restrictions: vec![], syntax_restrictions: vec![] },
            ThetaRole { role_type: ThetaRoleType::Theme, selectional_restrictions: vec![], syntax_restrictions: vec![] },
        ],
        frames: vec![],
        subclasses: vec![],
    }]);

    // "sleep" - sleep-40.4 class
    data.insert("sleep".to_string(), vec![VerbClass {
        id: "sleep-40.4".to_string(),
        name: "Sleep".to_string(),
        members: vec![
            VerbMember { name: "sleep".to_string(), wn_sense: None, fn_mapping: None, grouping: None },
            VerbMember { name: "nap".to_string(), wn_sense: None, fn_mapping: None, grouping: None },
            VerbMember { name: "doze".to_string(), wn_sense: None, fn_mapping: None, grouping: None },
        ],
        theta_roles: vec![ThetaRole { role_type: ThetaRoleType::Agent, selectional_restrictions: vec![], syntax_restrictions: vec![] }],
        frames: vec![],
        subclasses: vec![],
    }]);

    data
}

/// Create test FrameNet data
fn create_test_framenet_data() -> HashMap<String, Vec<FrameUnit>> {
    let mut data = HashMap::new();

    // "give" -> Giving frame
    data.insert("give".to_string(), vec![FrameUnit {
        name: "give".to_string(),
        pos: "v".to_string(),
        frame: "Giving".to_string(),
        definition: Some("Someone gives something to someone else".to_string()),
    }]);

    // "run" -> Self_motion frame
    data.insert("run".to_string(), vec![FrameUnit {
        name: "run".to_string(),
        pos: "v".to_string(),
        frame: "Self_motion".to_string(),
        definition: Some("An entity moves under its own power".to_string()),
    }]);

    // "love" -> Experiencer_focus frame
    data.insert("love".to_string(), vec![FrameUnit {
        name: "love".to_string(),
        pos: "v".to_string(),
        frame: "Experiencer_focus".to_string(),
        definition: Some("An experiencer has an emotional response to a content".to_string()),
    }]);

    // People/entities
    data.insert("john".to_string(), vec![FrameUnit {
        name: "john".to_string(),
        pos: "n".to_string(),
        frame: "People".to_string(),
        definition: Some("A human being".to_string()),
    }]);

    data.insert("mary".to_string(), vec![FrameUnit {
        name: "mary".to_string(),
        pos: "n".to_string(),
        frame: "People".to_string(),
        definition: Some("A human being".to_string()),
    }]);

    // Objects
    data.insert("book".to_string(), vec![FrameUnit {
        name: "book".to_string(),
        pos: "n".to_string(),
        frame: "Text".to_string(),
        definition: Some("A written or printed work".to_string()),
    }]);

    data
}

/// Create test WordNet data
fn create_test_wordnet_data() -> HashMap<String, Vec<WordNetSense>> {
    let mut data = HashMap::new();

    // "give"
    data.insert("give".to_string(), vec![WordNetSense {
        synset_id: "give.v.01".to_string(),
        definition: "transfer possession of something concrete or abstract".to_string(),
        pos: "v".to_string(),
        hypernyms: vec!["transfer.v.01".to_string()],
        hyponyms: vec!["hand.v.01".to_string(), "pass.v.05".to_string()],
        sense_rank: 1,
    }]);

    // "run"
    data.insert("run".to_string(), vec![WordNetSense {
        synset_id: "run.v.01".to_string(),
        definition: "move fast by using one's feet".to_string(),
        pos: "v".to_string(),
        hypernyms: vec!["locomote.v.01".to_string()],
        hyponyms: vec!["sprint.v.01".to_string(), "jog.v.01".to_string()],
        sense_rank: 1,
    }]);

    // "love"
    data.insert("love".to_string(), vec![WordNetSense {
        synset_id: "love.v.01".to_string(),
        definition: "have a great affection or liking for".to_string(),
        pos: "v".to_string(),
        hypernyms: vec!["emotion.v.01".to_string()],
        hyponyms: vec!["adore.v.01".to_string()],
        sense_rank: 1,
    }]);

    // "john"
    data.insert("john".to_string(), vec![WordNetSense {
        synset_id: "person.n.01".to_string(),
        definition: "a human being".to_string(),
        pos: "n".to_string(),
        hypernyms: vec!["organism.n.01".to_string()],
        hyponyms: vec![],
        sense_rank: 1,
    }]);

    // "mary"
    data.insert("mary".to_string(), vec![WordNetSense {
        synset_id: "person.n.01".to_string(),
        definition: "a human being".to_string(),
        pos: "n".to_string(),
        hypernyms: vec!["organism.n.01".to_string()],
        hyponyms: vec![],
        sense_rank: 1,
    }]);

    // "book"
    data.insert("book".to_string(), vec![WordNetSense {
        synset_id: "book.n.01".to_string(),
        definition: "a written work or composition".to_string(),
        pos: "n".to_string(),
        hypernyms: vec!["publication.n.01".to_string()],
        hyponyms: vec!["novel.n.01".to_string()],
        sense_rank: 1,
    }]);

    // "student"
    data.insert("student".to_string(), vec![WordNetSense {
        synset_id: "student.n.01".to_string(),
        definition: "a learner who is enrolled in an educational institution".to_string(),
        pos: "n".to_string(),
        hypernyms: vec!["person.n.01".to_string()],
        hyponyms: vec![],
        sense_rank: 1,
    }]);

    // "professor"
    data.insert("professor".to_string(), vec![WordNetSense {
        synset_id: "professor.n.01".to_string(),
        definition: "someone who is a member of the faculty at a college or university".to_string(),
        pos: "n".to_string(),
        hypernyms: vec!["person.n.01".to_string()],
        hyponyms: vec![],
        sense_rank: 1,
    }]);

    // Function words
    data.insert("the".to_string(), vec![WordNetSense {
        synset_id: "the.det.01".to_string(),
        definition: "definite article".to_string(),
        pos: "det".to_string(),
        hypernyms: vec![],
        hyponyms: vec![],
        sense_rank: 1,
    }]);

    data.insert("a".to_string(), vec![WordNetSense {
        synset_id: "a.det.01".to_string(),
        definition: "indefinite article".to_string(),
        pos: "det".to_string(),
        hypernyms: vec![],
        hyponyms: vec![],
        sense_rank: 1,
    }]);

    data.insert("every".to_string(), vec![WordNetSense {
        synset_id: "every.det.01".to_string(),
        definition: "universal quantifier".to_string(),
        pos: "det".to_string(),
        hypernyms: vec![],
        hyponyms: vec![],
        sense_rank: 1,
    }]);

    data.insert("some".to_string(), vec![WordNetSense {
        synset_id: "some.det.01".to_string(),
        definition: "existential quantifier".to_string(),
        pos: "det".to_string(),
        hypernyms: vec![],
        hyponyms: vec![],
        sense_rank: 1,
    }]);

    data
}

/// Create a test-enabled SemanticAnalyzer
pub fn create_test_analyzer() -> Result<SemanticAnalyzer, SemanticError> {
    let config = SemanticConfig {
        enable_framenet: true,
        enable_verbnet: true,
        enable_wordnet: true,
        enable_gpu: false,
        confidence_threshold: 0.7,
        parallel_processing: false,
    };

    SemanticAnalyzer::new_with_test_data(config, TestFixtures::new())
}

impl SemanticAnalyzer {
    /// Create a new analyzer with test data (for testing)
    pub fn new_with_test_data(config: SemanticConfig, test_data: TestFixtures) -> Result<Self, SemanticError> {
        let tokenizer = tokenization::Tokenizer::new();
        let morphology = morphology::MorphologyDatabase::new()?;
        let lexicon = lexicon::ClosedClassLexicon::new()?;

        // Create engines that use test data
        let framenet = create_framenet_engine_with_test_data(test_data.framenet_data.clone())?;
        let wordnet = WordNetEngine::new_with_test_data(test_data.wordnet_data.clone())?;
        let verbnet = create_test_verbnet_engine(test_data.verbnet_data.clone());

        Ok(SemanticAnalyzer {
            config,
            tokenizer,
            morphology,
            lexicon,
            framenet,
            wordnet,
            verbnet,
        })
    }
}

/// Create FrameNet engine with test data (compatibility function)
pub fn create_framenet_engine_with_test_data(_test_data: HashMap<String, Vec<FrameUnit>>) -> Result<FrameNetEngine, SemanticError> {
    // Use the new standalone crate constructor which already has the correct test data
    Ok(FrameNetEngine::new_with_test_data()?)
}

impl WordNetEngine {
    /// Create engine with test data
    pub fn new_with_test_data(_test_data: HashMap<String, Vec<WordNetSense>>) -> Result<Self, SemanticError> {
        // Use the regular constructor which already has the correct test data
        WordNetEngine::new()
    }
}

/// Create a VerbNetEngine with test data
fn create_test_verbnet_engine(_test_data: HashMap<String, Vec<VerbClass>>) -> VerbNetEngine {
    // Use the existing method from canopy-verbnet which has its own test data
    VerbNetEngine::new_with_test_data()
}
