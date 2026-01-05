# Performance Guide

## Current Metrics

| Operation         | Time                |
| ----------------- | ------------------- |
| Pipeline init     | ~730ms (one-time)   |
| Sentence analysis | 30-200μs            |
| Engine loading    | ~300ms (from cache) |
| Memory            | ~100MB              |

## Where Time Goes

Pipeline initialization (one-time):

- VerbNet: ~200ms (333 XML files)
- WordNet: ~150ms (117k synsets)
- FrameNet: ~150ms (1200+ frames)
- Treebank: ~150ms (UD patterns)

Per-sentence (fast path):

- Tokenization: \<10μs
- POS tagging: ~20μs
- Pattern matching: ~50μs
- Event composition: ~50μs

## Optimization Strategies

### 1. Reuse Pipeline

```rust
use canopy_resources::CanopyPipeline;

// ✅ Load once, reuse
let pipeline = CanopyPipeline::new()?;  // 730ms once
for sentence in sentences {
    pipeline.analyze(sentence)?;  // 30-200μs each
}
```

### 2. Use SharedEngines

```rust
use canopy_resources::SharedEngines;

// Load engines once, share across components
let engines = SharedEngines::new()?;

let syntax = TreebankSyntaxProvider::with_shared_engines(config, &engines)?;
let sense = VerbNetSenseProvider::with_engine(engines.verbnet.clone());
```

### 3. Release Mode

Always run release mode for production:

```bash
# Debug mode: ~10x slower
cargo run --example demo

# Release mode: full speed
cargo run --example demo --release
```

## Memory Usage

| Component | Memory     |
| --------- | ---------- |
| VerbNet   | ~20MB      |
| FrameNet  | ~30MB      |
| WordNet   | ~40MB      |
| Treebank  | ~10MB      |
| **Total** | **~100MB** |

## Benchmarking

```bash
# Run demo with timing info
cargo run -p canopy-resources --example demo --release

# Profile with flamegraph
cargo flamegraph --example demo --release
```

## Troubleshooting

### Slow First Analysis

**Cause**: Engine loading (expected one-time cost)

**Solution**: Pre-load pipeline at application start

### Memory Growth

**Cause**: Cache growth

**Solution**: Configure cache budget or restart periodically
