# Canopy Performance Guide

This guide covers performance characteristics, optimization strategies, and benchmarking for Canopy.

## Performance Overview

### Current Metrics

| Operation           | Time               | Notes                       |
| ------------------- | ------------------ | --------------------------- |
| Engine loading      | ~900ms             | One-time startup cost       |
| Layer 1 analysis    | 15-22ms/sentence   | Dominated by engine lookups |
| Layer 2 composition | 78-148μs/sentence  | In-memory processing        |
| Layer 3 discourse   | \<1ms/sentence     | Context updates             |
| **End-to-end**      | **~19ms/sentence** | L1 dominates (~99%)         |

### Resource Usage

| Resource      | Usage     | Budget                |
| ------------- | --------- | --------------------- |
| Engine memory | ~50-100MB | Loaded once           |
| Cache memory  | \<5MB     | Configurable          |
| Per-sentence  | ~1KB      | Temporary allocations |

## Where Time Goes

Layer 1 dominates execution time:

```
L1 (Lexical):     ████████████████████████████████ 99%
L2 (Events):      █ <1%
L3 (Discourse):   █ <1%
```

This is expected - L1 involves:

- VerbNet XML parsing and matching
- FrameNet frame lookup
- WordNet synset resolution
- Treebank pattern matching

L2 and L3 work primarily with in-memory data structures.

## Optimization Strategies

### 1. Reuse Engines

**Bad**: Creating new engines per sentence

```rust
// ❌ Slow - loads engines for every sentence
for sentence in sentences {
    let l1 = create_l1_analyzer_with_treebank()?; // ~900ms each!
    l1.analyze_sentence(sentence)?;
}
```

**Good**: Load once, reuse

```rust
// ✅ Fast - load once, reuse
let l1 = create_l1_analyzer_with_treebank()?; // 900ms once
for sentence in sentences {
    l1.analyze_sentence(sentence)?; // ~19ms each
}
```

### 2. Batch Processing

For corpus analysis, use batch methods:

```rust
// Process multiple sentences efficiently
let results = l1.analyze_batch(&sentences)?;
```

### 3. Cache Warm-up

The cache improves with usage. Common words get cached:

```rust
// Warm-up with common vocabulary
let warmup_words = ["the", "be", "have", "do", "say", "get", "make", "go"];
for word in warmup_words {
    let _ = l1.analyze(word);
}
// Subsequent analyses benefit from cached results
```

### 4. Release Mode

Always run in release mode for production:

```bash
# Debug mode: ~200ms/sentence (10x slower)
cargo run --example demo

# Release mode: ~19ms/sentence
cargo run --example demo --release
```

### 5. Parallel Processing (Future)

Engine lookups can run in parallel. The `parallel` feature enables this:

```toml
[dependencies]
canopy-semantic-engines = { version = "0.1", features = ["parallel"] }
```

## Memory Management

### Engine Memory

Engines load data into memory at startup:

| Engine   | Memory | Data Size     |
| -------- | ------ | ------------- |
| VerbNet  | ~20MB  | 333 XML files |
| FrameNet | ~30MB  | 1200+ frames  |
| WordNet  | ~40MB  | 117k+ synsets |
| PropBank | ~5MB   | Role frames   |
| Treebank | ~10MB  | UD patterns   |

Total: ~100MB peak during loading, ~50MB steady state.

### Cache Memory

The L1/L2 cache has a configurable budget:

```rust
use canopy_tokenizer::coordinator::CoordinatorConfig;

let config = CoordinatorConfig {
    cache_budget_mb: 10, // Default: 10MB
    ..Default::default()
};
```

Cache hit rates improve with usage:

- Cold: ~0%
- Warm: ~50-60%
- Hot (repeated text): ~80%+

## Benchmarking

### Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench --bench baseline

# With detailed output
cargo bench -- --verbose
```

### Custom Benchmarking

```rust
use std::time::Instant;

let l1 = create_l1_analyzer_with_treebank()?;

let start = Instant::now();
for sentence in &sentences {
    l1.analyze_sentence(sentence)?;
}
let elapsed = start.elapsed();

println!("Analyzed {} sentences in {:?}", sentences.len(), elapsed);
println!("Per sentence: {:?}", elapsed / sentences.len() as u32);
```

### Profiling Tips

Use `cargo flamegraph` for detailed profiling:

```bash
cargo install flamegraph
cargo flamegraph --example demo --release
```

Key hotspots to look for:

- XML parsing (VerbNet/FrameNet)
- HashMap lookups
- String allocations

## Performance by Use Case

### Interactive Analysis

For single sentences (e.g., LSP integration):

- Latency: ~19ms acceptable
- No special optimization needed
- Focus on engine preloading

### Corpus Processing

For processing large corpora:

- Use batch processing
- Consider parallel processing
- Monitor memory for very large corpora
- Stream results if memory-constrained

### Real-time Applications

For low-latency requirements:

- Warm up cache aggressively
- Pre-load engines at application start
- Consider caching full analysis results

## Comparison to Other Tools

| Tool             | Latency | Coverage          |
| ---------------- | ------- | ----------------- |
| Canopy           | ~19ms   | Full L1/L2/L3     |
| spaCy (Python)   | ~5ms    | Syntax only       |
| Stanford CoreNLP | ~200ms  | Full pipeline     |
| NLTK (Python)    | ~50ms   | Limited semantics |

Canopy prioritizes semantic depth over raw speed.

## Troubleshooting Performance

### Slow First Analysis

**Symptom**: First sentence takes 1+ seconds

**Cause**: Engine loading (expected)

**Solution**: Pre-load engines at application start

### Memory Growth

**Symptom**: Memory increases over time

**Cause**: Cache growth

**Solution**: Set cache budget or restart periodically

### Inconsistent Timing

**Symptom**: Some sentences much slower

**Cause**: Complex sentences with many verb classes

**Solution**: Consider sentence length limits for real-time use

## Future Optimizations

Planned improvements:

1. **Lazy loading**: Load engines on demand
1. **Compiled patterns**: Pre-compile frequent lookups
1. **SIMD acceleration**: For string matching
1. **GPU offload**: For large-scale batch processing
