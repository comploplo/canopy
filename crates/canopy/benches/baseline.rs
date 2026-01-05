use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use std::time::Duration;

/// Baseline benchmarks for canopy semantic analysis performance
///
/// These benchmarks measure real semantic analysis operations.
/// Run with: `cargo bench` or `cargo bench --release`
fn engine_lookup_benchmark(c: &mut Criterion) {
    use canopy_resources::{PartOfSpeech, VerbNetEngine, WordNetEngine};

    let mut group = c.benchmark_group("engine_lookups");
    group.sample_size(50);
    group.measurement_time(Duration::from_secs(5));

    // VerbNet lookup benchmark
    if let Ok(verbnet) = VerbNetEngine::new() {
        group.bench_function("verbnet_analyze", |b| {
            b.iter(|| verbnet.analyze_verb(black_box("give")).ok());
        });
    } else {
        eprintln!("Skipping VerbNet benchmark: data not available");
    }

    // WordNet lookup benchmark
    if let Ok(wordnet) = WordNetEngine::new() {
        group.bench_function("wordnet_analyze", |b| {
            b.iter(|| {
                wordnet
                    .analyze_word(black_box("run"), PartOfSpeech::Verb)
                    .ok()
            });
        });
    } else {
        eprintln!("Skipping WordNet benchmark: data not available");
    }

    group.finish();
}

fn provider_benchmark(c: &mut Criterion) {
    use canopy::runtime::{AnnotatedSyntax, AnnotatedToken, RoleProvider, SenseProvider, TokenId};
    use canopy::{DepRel, UPos};
    use canopy_resources::providers::DefaultProvider;

    let mut group = c.benchmark_group("provider");
    group.sample_size(50);
    group.measurement_time(Duration::from_secs(5));

    // Create provider
    let provider = match DefaultProvider::new() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Skipping provider benchmark: {e}");
            return;
        }
    };

    // Create test syntax
    let syntax = AnnotatedSyntax::new(
        "give".to_string(),
        vec![AnnotatedToken::new(
            TokenId::new(0),
            "give".to_string(),
            "give".to_string(),
            UPos::Verb,
            DepRel::Root,
            (0, 4),
        )],
    );

    group.bench_function("decompose_predicate", |b| {
        b.iter(|| {
            provider
                .decompose_predicate(black_box(&syntax), black_box(TokenId::new(0)))
                .ok()
        });
    });

    group.bench_function("bind_roles", |b| {
        b.iter(|| {
            provider
                .bind_roles(black_box(&syntax), black_box(TokenId::new(0)), None)
                .ok()
        });
    });

    group.finish();
}

/// Create benchmark test syntax for event composition
fn create_composition_syntax() -> (
    canopy::runtime::AnnotatedSyntax,
    Vec<canopy::kernel::events::DependencyArc>,
) {
    use canopy::kernel::events::DependencyArc;
    use canopy::runtime::{AnnotatedSyntax, AnnotatedToken, TokenId};
    use canopy::{DepRel, UPos};

    let syntax = AnnotatedSyntax::new(
        "John gives Mary a book".to_string(),
        vec![
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
        ],
    );
    let deps: Vec<DependencyArc> = syntax
        .tokens
        .iter()
        .filter_map(|token| {
            token
                .head
                .map(|head| DependencyArc::new(head, token.id, token.deprel.clone()))
        })
        .collect();
    (syntax, deps)
}

fn event_composition_benchmark(c: &mut Criterion) {
    use canopy::kernel::events::LittleVType;
    use canopy::kernel::events::{EventComposer, EventComposerConfig, SentenceAnalysis};
    use canopy::runtime::{PredicateDecomposition, RoleBinding, SenseId, TokenId};
    use canopy::ThetaRole;
    use std::collections::HashMap;

    let mut group = c.benchmark_group("event_composition");
    group.sample_size(100);

    let composer = EventComposer::new(EventComposerConfig::default());
    let (syntax, deps) = create_composition_syntax();
    let analysis = SentenceAnalysis::new(&syntax.text, syntax.clone()).with_dependencies(deps);

    let mut decompositions = HashMap::new();
    decompositions.insert(
        TokenId::new(1),
        vec![PredicateDecomposition::new(
            SenseId::new("give-13.1"),
            LittleVType::Cause,
            vec![ThetaRole::Agent, ThetaRole::Recipient, ThetaRole::Theme],
        )],
    );

    let mut role_bindings = HashMap::new();
    role_bindings.insert(
        TokenId::new(1),
        vec![
            RoleBinding::new(TokenId::new(0), ThetaRole::Agent, 0.9),
            RoleBinding::new(TokenId::new(2), ThetaRole::Recipient, 0.8),
            RoleBinding::new(TokenId::new(4), ThetaRole::Theme, 0.9),
        ],
    );

    group.bench_function("compose_event", |b| {
        b.iter(|| {
            composer
                .compose(
                    black_box(&analysis),
                    black_box(&decompositions),
                    black_box(&role_bindings),
                )
                .ok()
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    engine_lookup_benchmark,
    provider_benchmark,
    event_composition_benchmark,
);
criterion_main!(benches);
