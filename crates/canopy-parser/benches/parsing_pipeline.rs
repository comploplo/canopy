use canopy_parser::udpipe::{UDPipeEngine, UDPipeParser};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

fn benchmark_parsing_pipeline(c: &mut Criterion) {
    // Test sentences of varying complexity
    let test_sentences = vec![
        ("simple", "The cat sat."),
        ("medium", "The quick brown fox jumps over the lazy dog."),
        ("complex", "Although the weather was terrible, the determined students continued their research project."),
        ("very_complex", "The professor, who had been working on this groundbreaking research for over ten years, finally published the results that would revolutionize our understanding of quantum mechanics."),
    ];

    // Create a dummy parser (without real UDPipe model for benchmarking)
    let dummy_engine = UDPipeEngine::for_testing();
    let parser = UDPipeParser::new_with_engine(dummy_engine);

    let mut group = c.benchmark_group("parsing_pipeline");

    for (name, sentence) in test_sentences {
        group.bench_with_input(
            BenchmarkId::new("parse_document", name),
            sentence,
            |b, sentence| {
                b.iter(|| {
                    // This will use the dummy tokenization for benchmarking
                    let result = parser.parse_document(sentence);
                    let _ = criterion::black_box(result);
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("parse_words", name),
            sentence,
            |b, sentence| {
                b.iter(|| {
                    let result = parser.parse_words(sentence);
                    let _ = criterion::black_box(result);
                });
            },
        );
    }

    group.finish();

    // Single sentence latency benchmark (M2 target: <10ms)
    c.bench_function("single_sentence_latency", |b| {
        let sentence = "The quick brown fox jumps over the lazy dog.";
        b.iter(|| {
            let result = parser.parse_document(sentence);
            let _ = criterion::black_box(result);
        });
    });

    // Throughput benchmark
    c.bench_function("throughput_10_sentences", |b| {
        let sentences = vec![
            "The cat sat on the mat.",
            "She loves reading books.",
            "The weather is beautiful today.",
            "Programming in Rust is fun.",
            "The conference was very informative.",
            "Coffee helps me stay awake.",
            "The project deadline is approaching.",
            "Natural language processing is complex.",
            "The algorithm performs well.",
            "Testing is an important part of development.",
        ];

        b.iter(|| {
            for sentence in &sentences {
                let result = parser.parse_document(sentence);
                let _ = criterion::black_box(result);
            }
        });
    });
}

fn benchmark_memory_efficiency(c: &mut Criterion) {
    let dummy_engine = UDPipeEngine::for_testing();
    let parser = UDPipeParser::new_with_engine(dummy_engine);

    // Test memory usage patterns
    c.bench_function("memory_allocation_pattern", |b| {
        let sentence = "This is a test sentence for memory allocation benchmarking.";
        b.iter(|| {
            // Parse and immediately drop to test allocation patterns
            let document = parser.parse_document(sentence).unwrap();
            let word_count = document
                .sentences
                .iter()
                .map(|s| s.words.len())
                .sum::<usize>();
            criterion::black_box(word_count);
        });
    });

    // Test with varying sentence lengths
    let long_sentence = "The extraordinarily complex and multifaceted nature of human language comprehension and production mechanisms requires sophisticated computational models that can adequately capture the intricate relationships between syntactic structures, semantic representations, pragmatic interpretations, and contextual dependencies that emerge from the dynamic interaction of multiple cognitive systems operating simultaneously across different levels of linguistic organization.";

    c.bench_function("long_sentence_memory", |b| {
        b.iter(|| {
            let result = parser.parse_document(long_sentence);
            let _ = criterion::black_box(result);
        });
    });
}

fn benchmark_error_handling(c: &mut Criterion) {
    let dummy_engine = UDPipeEngine::for_testing();
    let parser = UDPipeParser::new_with_engine(dummy_engine);

    c.bench_function("error_handling_empty_input", |b| {
        b.iter(|| {
            let result = parser.parse_document("");
            let _ = criterion::black_box(result);
        });
    });

    c.bench_function("error_handling_whitespace", |b| {
        b.iter(|| {
            let result = parser.parse_document("   \n\t  ");
            let _ = criterion::black_box(result);
        });
    });
}

criterion_group!(
    benches,
    benchmark_parsing_pipeline,
    benchmark_memory_efficiency,
    benchmark_error_handling
);
criterion_main!(benches);
