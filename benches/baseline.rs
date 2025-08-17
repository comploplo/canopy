use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use std::time::Duration;

/// Baseline benchmarks for canopy performance monitoring
///
/// These benchmarks establish performance baselines and detect regressions.
/// Run with: `just bench` or `cargo bench`
fn dummy_parsing_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("parsing");

    // Set reasonable sample sizes and measurement time for development
    group.sample_size(50);
    group.measurement_time(Duration::from_secs(5));

    let sentences = [
        "The quick brown fox jumps over the lazy dog.",
        "John gives Mary a book in the library.",
        "Complex sentences with multiple clauses are harder to parse.",
        "She said that she would come to the party if she had time.",
    ];

    for (i, sentence) in sentences.iter().enumerate() {
        group.bench_with_input(
            BenchmarkId::new("dummy_parse", i),
            sentence,
            |b, sentence| {
                b.iter(|| {
                    // Dummy parsing operation - will be replaced with real UDPipe parsing
                    dummy_parse_sentence(black_box(sentence))
                });
            },
        );
    }

    group.finish();
}

fn memory_allocation_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory");

    group.bench_function("word_creation", |b| {
        b.iter(|| {
            // Dummy word creation - will be replaced with real Word struct
            create_dummy_words(black_box(50))
        });
    });

    group.bench_function("sentence_processing", |b| {
        let words = create_dummy_words(20);
        b.iter(|| {
            // Dummy sentence processing - will be replaced with real semantic analysis
            process_dummy_sentence(black_box(&words))
        });
    });

    group.finish();
}

fn semantic_analysis_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("semantics");

    // Benchmark theta role assignment (dummy implementation for now)
    group.bench_function("theta_roles", |b| {
        let sentence = "John gives Mary a book";
        b.iter(|| dummy_theta_assignment(black_box(sentence)));
    });

    // Benchmark lambda calculus operations (dummy implementation for now)
    group.bench_function("lambda_composition", |b| {
        b.iter(|| dummy_lambda_composition(black_box("give(john, mary, book)")));
    });

    group.finish();
}

fn lsp_response_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("lsp");

    // Target: sub-50ms response times
    group.bench_function("hover_request", |b| {
        let text = "The cat sat on the mat.";
        let position = 4; // position of "cat"

        b.iter(|| dummy_hover_response(black_box(text), black_box(position)));
    });

    group.bench_function("diagnostics", |b| {
        let text = "John give Mary a books."; // intentional errors

        b.iter(|| dummy_diagnostics(black_box(text)));
    });

    group.finish();
}

// === Dummy implementations (will be replaced with real code) ===

fn dummy_parse_sentence(sentence: &str) -> Vec<String> {
    // Dummy implementation: just split by whitespace
    sentence.split_whitespace().map(String::from).collect()
}

fn create_dummy_words(count: usize) -> Vec<String> {
    (0..count).map(|i| format!("word_{i}")).collect()
}

fn process_dummy_sentence(words: &[String]) -> String {
    // Dummy implementation: just join words
    words.join(" ")
}

fn dummy_theta_assignment(_sentence: &str) -> Vec<(&'static str, &'static str)> {
    // Dummy implementation: hardcoded roles
    vec![
        ("John", "agent"),
        ("Mary", "recipient"),
        ("book", "patient"),
    ]
}

fn dummy_lambda_composition(term: &str) -> String {
    // Dummy implementation: just return the input
    format!("λx.{term}")
}

fn dummy_hover_response(_text: &str, _position: usize) -> String {
    // Dummy implementation: return simple info
    "Hover: noun, animate, singular".to_string()
}

fn dummy_diagnostics(_text: &str) -> Vec<String> {
    // Dummy implementation: return sample diagnostics
    vec!["Subject-verb disagreement".to_string()]
}

criterion_group!(
    benches,
    dummy_parsing_benchmark,
    memory_allocation_benchmark,
    semantic_analysis_benchmark,
    lsp_response_benchmark
);

criterion_main!(benches);
