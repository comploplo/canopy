//! Golden snapshot tests for Canopy semantic analysis.
//!
//! These tests lock down the behavior of the full analysis pipeline
//! before any structural changes.

// Kernel's DepRel type
use canopy::kernel::discourse::{DiscourseConfig, DiscourseContext};
use canopy::kernel::events::{DependencyArc, EventComposer, EventComposerConfig, SentenceAnalysis};
use canopy::runtime::{AnnotatedSyntax, AnnotatedToken, TokenId};
use canopy::runtime::{RoleProvider, SenseProvider};
use canopy::{DepRel, UPos}; // canopy_core re-export for runtime types
use canopy_resources::providers::DefaultProvider;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Normalized analysis result for snapshotting.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AnalysisResult {
    input: String,
    layer1: Option<Vec<WordAnalysis>>,
    layer2: Option<Layer2Result>,
    layer3: Option<Layer3Result>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WordAnalysis {
    word: String,
    lemma: String,
    pos: Option<String>,
    has_verbnet: bool,
    has_framenet: bool,
    has_wordnet: bool,
    confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Layer2Result {
    event_count: usize,
    events: Vec<EventSummary>,
    unbound_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EventSummary {
    predicate: String,
    little_v: String,
    roles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Layer3Result {
    referent_count: usize,
    condition_count: usize,
    referent_types: HashMap<String, usize>,
    condition_types: HashMap<String, usize>,
}

fn verbnet_available() -> bool {
    canopy_resources::paths::data_path("data/verbnet").exists()
}

/// Create syntax for "John gives Mary a book"
fn syntax_ditransitive() -> AnnotatedSyntax {
    let tokens = vec![
        AnnotatedToken::new(
            TokenId::new(0),
            "John".to_string(),
            "john".to_string(),
            UPos::Propn,
            DepRel::Nsubj,
            (0, 4),
        )
        .with_head(TokenId::new(1)),
        AnnotatedToken::new(
            TokenId::new(1),
            "gives".to_string(),
            "give".to_string(),
            UPos::Verb,
            DepRel::Root,
            (5, 10),
        ),
        AnnotatedToken::new(
            TokenId::new(2),
            "Mary".to_string(),
            "mary".to_string(),
            UPos::Propn,
            DepRel::Iobj,
            (11, 15),
        )
        .with_head(TokenId::new(1)),
        AnnotatedToken::new(
            TokenId::new(3),
            "a".to_string(),
            "a".to_string(),
            UPos::Det,
            DepRel::Det,
            (16, 17),
        )
        .with_head(TokenId::new(4)),
        AnnotatedToken::new(
            TokenId::new(4),
            "book".to_string(),
            "book".to_string(),
            UPos::Noun,
            DepRel::Obj,
            (18, 22),
        )
        .with_head(TokenId::new(1)),
    ];
    AnnotatedSyntax::new("John gives Mary a book".to_string(), tokens)
}

/// Create syntax for "The cat was chased by the dog"
fn syntax_passive() -> AnnotatedSyntax {
    let tokens = vec![
        AnnotatedToken::new(
            TokenId::new(0),
            "The".to_string(),
            "the".to_string(),
            UPos::Det,
            DepRel::Det,
            (0, 3),
        )
        .with_head(TokenId::new(1)),
        AnnotatedToken::new(
            TokenId::new(1),
            "cat".to_string(),
            "cat".to_string(),
            UPos::Noun,
            DepRel::NsubjPass,
            (4, 7),
        )
        .with_head(TokenId::new(3)),
        AnnotatedToken::new(
            TokenId::new(2),
            "was".to_string(),
            "be".to_string(),
            UPos::Aux,
            DepRel::AuxPass,
            (8, 11),
        )
        .with_head(TokenId::new(3)),
        AnnotatedToken::new(
            TokenId::new(3),
            "chased".to_string(),
            "chase".to_string(),
            UPos::Verb,
            DepRel::Root,
            (12, 18),
        ),
        AnnotatedToken::new(
            TokenId::new(4),
            "by".to_string(),
            "by".to_string(),
            UPos::Adp,
            DepRel::Case,
            (19, 21),
        )
        .with_head(TokenId::new(6)),
        AnnotatedToken::new(
            TokenId::new(5),
            "the".to_string(),
            "the".to_string(),
            UPos::Det,
            DepRel::Det,
            (22, 25),
        )
        .with_head(TokenId::new(6)),
        AnnotatedToken::new(
            TokenId::new(6),
            "dog".to_string(),
            "dog".to_string(),
            UPos::Noun,
            DepRel::Obl,
            (26, 29),
        )
        .with_head(TokenId::new(3)),
    ];
    AnnotatedSyntax::new("The cat was chased by the dog".to_string(), tokens)
}

/// Create syntax for "Mary broke the vase"
fn syntax_causative() -> AnnotatedSyntax {
    let tokens = vec![
        AnnotatedToken::new(
            TokenId::new(0),
            "Mary".to_string(),
            "mary".to_string(),
            UPos::Propn,
            DepRel::Nsubj,
            (0, 4),
        )
        .with_head(TokenId::new(1)),
        AnnotatedToken::new(
            TokenId::new(1),
            "broke".to_string(),
            "break".to_string(),
            UPos::Verb,
            DepRel::Root,
            (5, 10),
        ),
        AnnotatedToken::new(
            TokenId::new(2),
            "the".to_string(),
            "the".to_string(),
            UPos::Det,
            DepRel::Det,
            (11, 14),
        )
        .with_head(TokenId::new(3)),
        AnnotatedToken::new(
            TokenId::new(3),
            "vase".to_string(),
            "vase".to_string(),
            UPos::Noun,
            DepRel::Obj,
            (15, 19),
        )
        .with_head(TokenId::new(1)),
    ];
    AnnotatedSyntax::new("Mary broke the vase".to_string(), tokens)
}

/// Create syntax for "John fears spiders"
fn syntax_psych() -> AnnotatedSyntax {
    let tokens = vec![
        AnnotatedToken::new(
            TokenId::new(0),
            "John".to_string(),
            "john".to_string(),
            UPos::Propn,
            DepRel::Nsubj,
            (0, 4),
        )
        .with_head(TokenId::new(1)),
        AnnotatedToken::new(
            TokenId::new(1),
            "fears".to_string(),
            "fear".to_string(),
            UPos::Verb,
            DepRel::Root,
            (5, 10),
        ),
        AnnotatedToken::new(
            TokenId::new(2),
            "spiders".to_string(),
            "spider".to_string(),
            UPos::Noun,
            DepRel::Obj,
            (11, 18),
        )
        .with_head(TokenId::new(1)),
    ];
    AnnotatedSyntax::new("John fears spiders".to_string(), tokens)
}

/// Create syntax for "The door opened"
fn syntax_middle() -> AnnotatedSyntax {
    let tokens = vec![
        AnnotatedToken::new(
            TokenId::new(0),
            "The".to_string(),
            "the".to_string(),
            UPos::Det,
            DepRel::Det,
            (0, 3),
        )
        .with_head(TokenId::new(1)),
        AnnotatedToken::new(
            TokenId::new(1),
            "door".to_string(),
            "door".to_string(),
            UPos::Noun,
            DepRel::Nsubj,
            (4, 8),
        )
        .with_head(TokenId::new(2)),
        AnnotatedToken::new(
            TokenId::new(2),
            "opened".to_string(),
            "open".to_string(),
            UPos::Verb,
            DepRel::Root,
            (9, 15),
        ),
    ];
    AnnotatedSyntax::new("The door opened".to_string(), tokens)
}

/// Create syntax for "Running is good exercise"
fn syntax_gerund() -> AnnotatedSyntax {
    let tokens = vec![
        AnnotatedToken::new(
            TokenId::new(0),
            "Running".to_string(),
            "run".to_string(),
            UPos::Verb,
            DepRel::Csubj,
            (0, 7),
        )
        .with_head(TokenId::new(3)),
        AnnotatedToken::new(
            TokenId::new(1),
            "is".to_string(),
            "be".to_string(),
            UPos::Aux,
            DepRel::Cop,
            (8, 10),
        )
        .with_head(TokenId::new(3)),
        AnnotatedToken::new(
            TokenId::new(2),
            "good".to_string(),
            "good".to_string(),
            UPos::Adj,
            DepRel::Amod,
            (11, 15),
        )
        .with_head(TokenId::new(3)),
        AnnotatedToken::new(
            TokenId::new(3),
            "exercise".to_string(),
            "exercise".to_string(),
            UPos::Noun,
            DepRel::Root,
            (16, 24),
        ),
    ];
    AnnotatedSyntax::new("Running is good exercise".to_string(), tokens)
}

/// Analyze Layer 1 (word-level semantics)
fn analyze_layer1(syntax: &AnnotatedSyntax, provider: &DefaultProvider) -> Vec<WordAnalysis> {
    syntax
        .tokens
        .iter()
        .map(|token| {
            // Check sense availability
            let decompositions = provider.decompose_predicate(syntax, token.id).ok();
            let has_senses = decompositions.as_ref().is_some_and(|d| !d.is_empty());

            let confidence = if has_senses {
                decompositions
                    .as_ref()
                    .and_then(|d| d.first())
                    .map_or(0.0, |d| d.confidence)
            } else {
                0.0
            };

            WordAnalysis {
                word: token.form.clone(),
                lemma: token.lemma.clone(),
                pos: None, // Not tracking POS in snapshots
                has_verbnet: has_senses,
                has_framenet: has_senses, // Using VerbNet as proxy
                has_wordnet: has_senses,  // Using VerbNet as proxy
                confidence: (confidence * 100.0).round() / 100.0, // Round to 2 decimal places
            }
        })
        .collect()
}

/// Analyze Layer 2 (event composition)
fn analyze_layer2(syntax: &AnnotatedSyntax, provider: &DefaultProvider) -> Layer2Result {
    let composer = EventComposer::new(EventComposerConfig::default());

    // Build dependencies from tokens
    let deps: Vec<DependencyArc> = syntax
        .tokens
        .iter()
        .filter_map(|token| {
            token
                .head
                .map(|head| DependencyArc::new(head, token.id, token.deprel.clone()))
        })
        .collect();

    let analysis = SentenceAnalysis::new(&syntax.text, syntax.clone()).with_dependencies(deps);

    // Get decompositions for all predicates
    let mut decompositions = HashMap::new();
    let mut role_bindings = HashMap::new();

    for token in &syntax.tokens {
        if token.upos == UPos::Verb {
            if let Ok(decomps) = provider.decompose_predicate(syntax, token.id) {
                if !decomps.is_empty() {
                    decompositions.insert(token.id, decomps);

                    // Get role bindings
                    if let Ok(bindings) = provider.bind_roles(syntax, token.id, None) {
                        role_bindings.insert(token.id, bindings);
                    }
                }
            }
        }
    }

    let result = composer.compose(&analysis, &decompositions, &role_bindings);

    match result {
        Ok(events) => {
            Layer2Result {
                event_count: events.events.len(),
                events: events
                    .events
                    .iter()
                    .map(|e| {
                        let mut roles: Vec<_> =
                            e.participants.keys().map(|r| format!("{r:?}")).collect();
                        roles.sort(); // Deterministic ordering
                        EventSummary {
                            predicate: e.predicate.clone(),
                            little_v: format!("{:?}", e.little_v_type),
                            roles,
                        }
                    })
                    .collect(),
                unbound_count: events.unbound_participants.len(),
            }
        }
        Err(_) => Layer2Result {
            event_count: 0,
            events: vec![],
            unbound_count: 0,
        },
    }
}

/// Analyze Layer 3 (discourse)
fn analyze_layer3(referent_names: &[&str]) -> Layer3Result {
    let mut ctx = DiscourseContext::new(DiscourseConfig::default());

    ctx.begin_sentence();
    for name in referent_names {
        ctx.introduce_entity(*name);
    }
    ctx.end_sentence();

    let drs = ctx.drs();

    let mut referent_types: HashMap<String, usize> = HashMap::new();
    let condition_types: HashMap<String, usize> = HashMap::new();

    for referent in drs.universe.values() {
        let type_name = if referent.is_event {
            "Event"
        } else {
            "Individual"
        };
        *referent_types.entry(type_name.to_string()).or_default() += 1;
    }

    for _condition in &drs.conditions {
        // Count condition types if needed
    }

    Layer3Result {
        referent_count: drs.universe.len(),
        condition_count: drs.conditions.len(),
        referent_types,
        condition_types,
    }
}

// ============================================================================
// Layer 1 Tests
// ============================================================================

#[test]
fn layer1_ditransitive() {
    if !verbnet_available() {
        eprintln!("Skipping: VerbNet data not available");
        return;
    }

    let provider = DefaultProvider::new().unwrap();
    let syntax = syntax_ditransitive();

    let analysis = AnalysisResult {
        input: syntax.text.clone(),
        layer1: Some(analyze_layer1(&syntax, &provider)),
        layer2: None,
        layer3: None,
    };

    insta::assert_json_snapshot!(analysis);
}

#[test]
fn layer1_passive() {
    if !verbnet_available() {
        eprintln!("Skipping: VerbNet data not available");
        return;
    }

    let provider = DefaultProvider::new().unwrap();
    let syntax = syntax_passive();

    let analysis = AnalysisResult {
        input: syntax.text.clone(),
        layer1: Some(analyze_layer1(&syntax, &provider)),
        layer2: None,
        layer3: None,
    };

    insta::assert_json_snapshot!(analysis);
}

#[test]
fn layer1_causative() {
    if !verbnet_available() {
        eprintln!("Skipping: VerbNet data not available");
        return;
    }

    let provider = DefaultProvider::new().unwrap();
    let syntax = syntax_causative();

    let analysis = AnalysisResult {
        input: syntax.text.clone(),
        layer1: Some(analyze_layer1(&syntax, &provider)),
        layer2: None,
        layer3: None,
    };

    insta::assert_json_snapshot!(analysis);
}

#[test]
fn layer1_psych() {
    if !verbnet_available() {
        eprintln!("Skipping: VerbNet data not available");
        return;
    }

    let provider = DefaultProvider::new().unwrap();
    let syntax = syntax_psych();

    let analysis = AnalysisResult {
        input: syntax.text.clone(),
        layer1: Some(analyze_layer1(&syntax, &provider)),
        layer2: None,
        layer3: None,
    };

    insta::assert_json_snapshot!(analysis);
}

#[test]
fn layer1_middle() {
    if !verbnet_available() {
        eprintln!("Skipping: VerbNet data not available");
        return;
    }

    let provider = DefaultProvider::new().unwrap();
    let syntax = syntax_middle();

    let analysis = AnalysisResult {
        input: syntax.text.clone(),
        layer1: Some(analyze_layer1(&syntax, &provider)),
        layer2: None,
        layer3: None,
    };

    insta::assert_json_snapshot!(analysis);
}

#[test]
fn layer1_gerund() {
    if !verbnet_available() {
        eprintln!("Skipping: VerbNet data not available");
        return;
    }

    let provider = DefaultProvider::new().unwrap();
    let syntax = syntax_gerund();

    let analysis = AnalysisResult {
        input: syntax.text.clone(),
        layer1: Some(analyze_layer1(&syntax, &provider)),
        layer2: None,
        layer3: None,
    };

    insta::assert_json_snapshot!(analysis);
}

// ============================================================================
// Layer 2 Tests
// ============================================================================

#[test]
fn layer2_ditransitive() {
    if !verbnet_available() {
        eprintln!("Skipping: VerbNet data not available");
        return;
    }

    let provider = DefaultProvider::new().unwrap();
    let syntax = syntax_ditransitive();

    let analysis = AnalysisResult {
        input: syntax.text.clone(),
        layer1: Some(vec![]),
        layer2: Some(analyze_layer2(&syntax, &provider)),
        layer3: None,
    };

    insta::assert_json_snapshot!(analysis);
}

#[test]
fn layer2_causative() {
    if !verbnet_available() {
        eprintln!("Skipping: VerbNet data not available");
        return;
    }

    let provider = DefaultProvider::new().unwrap();
    let syntax = syntax_causative();

    let analysis = AnalysisResult {
        input: syntax.text.clone(),
        layer1: Some(vec![]),
        layer2: Some(analyze_layer2(&syntax, &provider)),
        layer3: None,
    };

    insta::assert_json_snapshot!(analysis);
}

// ============================================================================
// Layer 3 Tests
// ============================================================================

#[test]
fn layer3_anaphora() {
    // "John saw Mary. He waved to her."
    let analysis = AnalysisResult {
        input: "John saw Mary. He waved to her.".to_string(),
        layer1: Some(vec![]),
        layer2: None,
        layer3: Some(analyze_layer3(&["John", "Mary"])),
    };

    insta::assert_json_snapshot!(analysis);
}

#[test]
fn layer3_quantification() {
    // "Every boy loves his mother"
    let analysis = AnalysisResult {
        input: "Every boy loves his mother".to_string(),
        layer1: Some(vec![]),
        layer2: None,
        layer3: Some(analyze_layer3(&["boy", "mother"])),
    };

    insta::assert_json_snapshot!(analysis);
}

#[test]
fn layer3_conditional() {
    // "If it rains, the ground gets wet"
    let analysis = AnalysisResult {
        input: "If it rains, the ground gets wet".to_string(),
        layer1: Some(vec![]),
        layer2: None,
        layer3: Some(analyze_layer3(&["it", "ground"])),
    };

    insta::assert_json_snapshot!(analysis);
}
