use canopy_semantics::verbnet::{VerbNetEngine, VerbNetFeatureExtractor};
use canopy_core::{Word, UPos, DepRel};
use criterion::{criterion_group, criterion_main, Criterion};

fn benchmark_verbnet_lookup(c: &mut Criterion) {
    let mut engine = VerbNetEngine::new();
    engine.add_test_data();
    
    c.bench_function("verbnet_class_lookup", |b| {
        b.iter(|| {
            let classes = engine.get_verb_classes("give");
            criterion::black_box(classes);
        });
    });
    
    c.bench_function("verbnet_theta_roles", |b| {
        b.iter(|| {
            let roles = engine.get_theta_roles("give");
            criterion::black_box(roles);
        });
    });
    
    let mut extractor = VerbNetFeatureExtractor::new(engine);
    let test_word = Word {
        id: 1,
        text: "give".to_string(),
        lemma: "give".to_string(),
        upos: UPos::Verb,
        xpos: None,
        feats: Default::default(),
        head: None,
        deprel: DepRel::Root,
        deps: None,
        misc: None,
        start: 0,
        end: 4,
    };
    
    c.bench_function("verbnet_feature_extraction", |b| {
        b.iter(|| {
            let features = extractor.extract_features(&test_word);
            criterion::black_box(features);
        });
    });
}

criterion_group!(benches, benchmark_verbnet_lookup);
criterion_main!(benches);