- **Current Status**: Codebase simplified - 5 semantic engine crates consolidated into `canopy-semantic-engines`, unified error handling via `CanopyError`. All tests pass. Test coverage gate at 70%.

# 🧹 CRATE CONSOLIDATION COMPLETED - 2025-12-30

## Simplification Summary

### Crate Consolidation ✅ COMPLETE

Merged 5 separate semantic engine crates into one:

- `canopy-verbnet` → `canopy-semantic-engines::verbnet`
- `canopy-framenet` → `canopy-semantic-engines::framenet`
- `canopy-wordnet` → `canopy-semantic-engines::wordnet`
- `canopy-lexicon` → `canopy-semantic-engines::lexicon`
- `canopy-propbank` → `canopy-semantic-engines::propbank`

**Import changes**: Use `canopy_semantic_engines::verbnet::VerbNetEngine` instead of `canopy_verbnet::VerbNetEngine`

### Error Unification ✅ COMPLETE

- Expanded `CanopyError` in `canopy-core` to be the unified error type (24 variants)
- Added `From` implementations for all crate-specific errors
- All errors now convert to `CanopyError` for consistent handling

### Current Workspace (11 crates)

- `canopy` - Main crate
- `canopy-core` - Core types, unified `CanopyError`
- `canopy-engine` - Shared infrastructure
- `canopy-semantic-engines` - VerbNet, FrameNet, WordNet, PropBank, Lexicon
- `canopy-tokenizer` - Layer 1 semantic coordination
- `canopy-treebank` - UD treebank patterns
- `canopy-events` - Layer 2 event composition
- `canopy-discourse` - Layer 3 DRT and anaphora
- `canopy-pipeline` - High-level orchestration
- `canopy-cli` - CLI tool

# 🚨 CRITICAL: NO SKIPPING PLAN STEPS WITHOUT ASKING 🚨

## Never Skip or Reorder Plan Steps Unilaterally

**When working on a plan, you MUST follow the agreed-upon order. If a step seems complex or you want to change the order:**

1. **STOP** - Do not make the decision yourself
1. **EXPLAIN** - Present the facts about why the step is complex
1. **ASK** - Request guidance on how to proceed
1. **WAIT** - Do not proceed until the user responds

### What NOT to do:

- ❌ "Phase 4 seems complex, so I'll skip to Phase 6" - WRONG
- ❌ "Given your recent request, I'll prioritize X over Y" - WRONG (unless explicitly asked)
- ❌ Reordering tasks based on your own assessment of complexity - WRONG
- ❌ Assuming the user wants certain things prioritized - WRONG

### What TO do:

- ✅ "Phase 4 requires changes to 6 files. Here's what's involved: [details]. Should I proceed, or would you like to discuss the approach first?"
- ✅ "I notice Phase 4 has more scope than I initially thought. Here are the options: [A, B, C]. Which would you prefer?"
- ✅ Follow the plan in order unless the user explicitly approves a change

**Rationale**: The user created the plan with intent. Skipping steps without asking wastes their time and creates confusion about project state.

# 🧹 PREVIOUS CLEANUP (2025-10-01)

Previous cleanup removed GPU code, deleted disabled examples, fixed clippy lints. See git history for details.

# 🚨 CRITICAL: NO STUB IMPLEMENTATIONS - EXTREMELY IMPORTANT 🚨

## ABSOLUTE PROHIBITION ON STUB/MOCK IMPLEMENTATIONS

**STUBS ARE BANNED AND EXTREMELY NOT DESIRED. NEVER CREATE STUB IMPLEMENTATIONS.**

### What Constitutes a Stub (FORBIDDEN):

- ❌ Functions that return empty/placeholder data instead of real analysis
- ❌ Engines that claim to load data but return empty results
- ❌ "Graceful degradation" that falls back to meaningless placeholder responses
- ❌ Tests that return hardcoded fake data instead of real processing
- ❌ Performance benchmarks measuring empty operations instead of real work
- ❌ Demos that show unrealistic performance (microseconds instead of milliseconds)
- ❌ Any code that lies about system capabilities or performance

### REQUIRED: Real Implementation Only

- ✅ **ALWAYS** load actual semantic data (VerbNet XMLs, WordNet database, FrameNet)
- ✅ **ALWAYS** fail fast if data cannot be loaded - do not fall back to stubs
- ✅ **ALWAYS** show realistic performance metrics (1-10ms per sentence, not microseconds)
- ✅ **ALWAYS** return actual semantic analysis or clear errors
- ✅ **ALWAYS** test with real data that produces meaningful results
- ✅ **ALWAYS** be honest about system capabilities and limitations

### Enforcement Rules:

1. **Data Loading**: If semantic data doesn't exist, FAIL initialization - don't use stubs
1. **Performance**: Real semantic analysis takes milliseconds, not microseconds - be honest
1. **Testing**: Tests must use real data and produce meaningful results
1. **Benchmarks**: Show actual performance with real workloads, not infrastructure overhead
1. **Documentation**: Never claim capabilities that don't exist with real data

### Examples of Previously Problematic Code:

```rust
// ❌ FORBIDDEN - This is a stub that lies about performance
fn analyze_word(&self, word: &str) -> Result<Analysis> {
    Ok(Analysis::empty()) // This is dishonest!
}

// ✅ REQUIRED - Real implementation or failure
fn analyze_word(&self, word: &str) -> Result<Analysis> {
    if !self.data_loaded {
        return Err("VerbNet data not loaded - cannot perform analysis");
    }
    // Actually process the word through loaded VerbNet XML data
    self.process_verb_with_loaded_data(word)
}
```

**Any code that violates these rules must be immediately fixed to use real implementations.**

# Script Policy

## Do Not Write Scripts to /tmp/

- ❌ Never write scripts to `/tmp/` directory
- ✅ Either run commands directly or create scripts in the project file structure
- ✅ Use the project's `scripts/` directory for utility scripts

# Examples and Demos Policy

## ONE Demo Only

- ❌ Do NOT create examples in individual crate directories
- ❌ Do NOT create per-crate example files (no crates/\*/examples/)
- ✅ The project has ONE central demo: `examples/demo.rs`
- ✅ Run with: `cargo run --example demo --release`
- ✅ Extend the existing demo when adding new features to showcase them

**Rationale**: Scattered examples become stale and unmaintained. One comprehensive demo ensures quality and demonstrates the full pipeline integration.

# CRITICAL COVERAGE REQUIREMENTS - DO NOT IGNORE

## Coverage Gates for Releases

- **CURRENT GATE**: 70% (M9 milestone achieved)
- **M4 REQUIREMENT**: 80% minimum + clippy tech debt resolution

## What Makes a Good Test (NOT these)

- ❌ `assert!(true)` - Always passes, tests nothing
- ❌ `assert!(result.is_ok() || result.is_err())` - Tautology, always true
- ❌ `assert!(vec.len() >= 0)` - Unsigned can never be negative
- ❌ Tests that exercise stubs and verify empty returns

## Current Coverage Achievements

- **VerbNet integration**: 99.7% success rate (332/333 XML files)
- **Lemmatization integration**: 10 comprehensive integration tests with 100% accuracy

# M5 LEMMATIZATION IMPLEMENTATION - COMPLETED ✅

## Lemmatization Architecture Implementation

### COMPLETED Features:

1. **✅ Lemmatizer Trait Architecture**

   - `Lemmatizer` trait with confidence scoring
   - `SimpleLemmatizer` with rule-based processing
   - `NLPRuleLemmatizer` with nlprule integration (optional feature)
   - `LemmatizerFactory` for creating appropriate lemmatizer instances

1. **✅ SemanticCoordinator Integration**

   - Lemmatization preprocessing in analysis pipeline
   - Cache keys based on lemmatized forms for better hit rates
   - Graceful fallback when lemmatization fails
   - Batch processing with lemmatization support

1. **✅ Performance Optimizations**

   - 54.4% cache hit rate improvement with lemmatization
   - Batch processing performs better with lemmatization (-51.7% overhead)
   - Memory efficient: \<0.5MB usage (0.5% of budget)
   - 100% lemmatization accuracy on test cases

1. **✅ Comprehensive Testing**

   - 10 integration tests covering all lemmatization scenarios
   - Performance benchmarking with detailed metrics
   - Accuracy verification with confidence scoring
   - Cache effectiveness validation

## Performance Benchmarks (Release Mode)

### Single Word Analysis:

- **Without lemmatization**: 53.7μs per word (18,626 words/sec)
- **With lemmatization**: 85.4μs per word (11,703 words/sec)
- **Overhead**: 59.2% (acceptable for improved semantic accuracy)

### Batch Processing:

- **Batch overhead**: -51.7% (lemmatization IMPROVES batch performance due to caching)
- **Cache hit rate**: 54.4% (increases with usage)
- **Memory efficiency**: 0.5MB (0.5% of budget)

### Quality Metrics:

- **Lemmatization accuracy**: 100% on test cases
- **Confidence scoring**: Irregular verbs 95%, regular rules 80%, unchanged 60%
- **Fallback reliability**: Graceful degradation when engines fail

## Implementation Details

### Files Modified/Created:

- **`crates/canopy-semantic-layer/src/lemmatizer.rs`**: Complete lemmatization module
- **`crates/canopy-semantic-layer/src/coordinator.rs`**: Integration with SemanticCoordinator
- **`crates/canopy-semantic-layer/tests/lemmatization_integration_tests.rs`**: Comprehensive tests
- **`crates/canopy-semantic-layer/examples/lemmatization_benchmark.rs`**: Performance benchmark

### Configuration Options:

```rust
pub struct CoordinatorConfig {
    pub enable_lemmatization: bool,          // Default: true
    pub use_advanced_lemmatization: bool,    // Default: false (simple)
    // ... existing config options
}
```

### Usage Example:

```rust
let coordinator = SemanticCoordinator::new(CoordinatorConfig::default())?;
let result = coordinator.analyze("running")?;
// result.original_word = "running"
// result.lemma = "run"
// result.lemmatization_confidence = Some(0.8)
```

## Quality Assurance Results

### Test Coverage:

- **Unit tests**: 6 lemmatizer module tests (100% passing)
- **Integration tests**: 10 coordinator integration tests (100% passing)
- **Performance tests**: Benchmark suite with detailed metrics
- **Accuracy tests**: 100% accuracy on representative test cases

### Error Handling:

- Graceful degradation when lemmatization fails
- Fallback to simple lemmatizer when advanced features unavailable
- Proper error propagation in coordinator pipeline
- Cache invalidation on lemmatization failures

## Next Steps for M6

### Future Enhancements (Optional):

1. **Advanced NLP Rule Integration**

   - Full nlprule feature integration with proper morphological analysis
   - Context-aware lemmatization based on POS tags
   - Multilingual lemmatization support

1. **Performance Optimizations**

   - Lemmatizer warm-up with common words
   - Predictive caching based on morphological patterns
   - SIMD-accelerated string processing for rules

1. **Quality Improvements**

   - Machine learning-based confidence calibration
   - Corpus-based irregular verb discovery
   - Domain-specific lemmatization rules

## Summary

The lemmatization implementation is **COMPLETE and PRODUCTION-READY** with:

- 100% test coverage for new functionality
- Excellent performance characteristics (54.4% cache hit improvement)
- 100% accuracy on test cases with confidence scoring
- Graceful error handling and fallback strategies
- Comprehensive benchmarking and validation

This implementation provides the semantic analysis foundation for improved cache efficiency and analysis accuracy while maintaining production performance requirements.

## M4 CLIPPY TECH DEBT REQUIREMENTS

**All clippy allows must be resolved for M4 release:**

### Current Tech Debt (to be fixed by M4):

- `#![allow(clippy::uninlined_format_args)]` - Convert to modern format syntax
- `#![allow(clippy::needless_borrow)]` - Remove unnecessary explicit borrows
- `#![allow(clippy::field_reassign_with_default)]` - Use struct initialization syntax
- `#![allow(clippy::collapsible_if)]` - Simplify nested conditionals
- `#![allow(clippy::useless_vec)]` - Use arrays where appropriate
- `#![allow(clippy::manual_clamp)]` - Use `.clamp()` method
- `#![allow(clippy::enum_variant_names)]` - Improve enum naming
- `#![allow(clippy::needless_range_loop)]` - Use iterators instead of indexing
- `#![allow(clippy::new_without_default)]` - Add Default implementations
- `#![allow(clippy::clone_on_copy)]` - Remove unnecessary clones

### 🚨 CRITICAL: VERIFICATION REQUIREMENTS - EXTREMELY IMPORTANT 🚨

### **ABSOLUTE PROHIBITION ON UNVERIFIED CLAIMS**

**❌ NEVER CLAIM SOMETHING WORKS WITHOUT ACTUALLY TESTING IT ❌**

### What Constitutes Unverified Claims (FORBIDDEN):

- ❌ Saying "this will work" or "this should work" without running the code
- ❌ Claiming "COMPLETED" or "SUCCESS" without running actual verification commands
- ❌ Declaring performance improvements without measuring them
- ❌ Stating compilation works without running `cargo check` or `cargo build`
- ❌ Claiming tests pass without running them
- ❌ Saying coverage requirements are met without running coverage checks
- ❌ Making assertions about system behavior without executing the code

### REQUIRED: Test Everything Before Claims

- ✅ **ALWAYS** compile and run code before claiming it works
- ✅ **ALWAYS** run `cargo check` and `cargo build` before saying compilation succeeds
- ✅ **ALWAYS** run tests before claiming they pass
- ✅ **ALWAYS** execute coverage checks before claiming coverage requirements are met
- ✅ **ALWAYS** run the actual program/demo before claiming it functions correctly
- ✅ **ALWAYS** measure performance before claiming improvements
- ✅ **ALWAYS** verify all claims with concrete evidence

### Verification Workflow:

1. **Implementation**: Write the code changes
1. **Compilation Check**: Run `cargo check` and verify no errors
1. **Build Check**: Run `cargo build` and verify success
1. **Test Execution**: Run relevant tests and verify they pass
1. **Coverage Verification**: Run `scripts/check-coverage.sh` and verify it passes
1. **Demo/Program Testing**: Actually run the program and verify expected behavior
1. **Performance Measurement**: If claiming performance gains, measure and show actual results
1. **Documentation**: Only then document what was **actually verified to work**

### Examples of Previously Problematic Claims:

```
❌ FORBIDDEN: "The caching improvements should increase hit rate to 40-70%"
✅ REQUIRED: "After running the demo, cache hit rate increased from 0.3% to 43.2%"

❌ FORBIDDEN: "This fixes the compilation errors"
✅ REQUIRED: "Ran `cargo check` - no compilation errors found"

❌ FORBIDDEN: "The demo runs successfully now"
✅ REQUIRED: "Executed `cargo run --example basic_demo` - completed successfully with output: [paste actual output]"
```

### Release Requirements:

1. ✅ Always run `scripts/check-coverage.sh` and verify it PASSES before declaring completion
1. ✅ Always run `cargo build` for entire workspace and verify no errors
1. ✅ Always run relevant tests and verify they pass
1. ✅ Always execute the actual program/demo and verify expected behavior
1. ✅ Always measure and report actual performance metrics when claiming improvements
1. ❌ NEVER modify coverage thresholds to make releases easier
1. ❌ NEVER skip verification steps to save time
1. ❌ NEVER make claims based on theoretical analysis alone

**VERIFICATION SAVES TIME AND PREVENTS BUGS - UNVERIFIED CLAIMS WASTE TIME AND CREATE PROBLEMS**

______________________________________________________________________

THE BELOW IS A NEW SET OF INSTRUCTIONS TO BE REVIEWED, INTEGRATED, AND CLEAND UP.

# UD TREEBANK PATTERN MATCHING PLAN FOR LAYER 1 EXTENSION

## Overview

Extend Layer 1 semantic analysis with lightweight dependency pattern matching using UD English-EWT treebank. This will add basic syntactic structure without requiring a full parser, using semantic signatures from our existing engines to match against treebank patterns.

## Architecture Design

### Core Concept

```text
Layer 1 Semantic Analysis → Semantic Signature → Treebank Pattern Match → Dependency Structure
         ↓                          ↓                      ↓                    ↓
[VerbNet + FrameNet + WordNet] [Hash key]  [UD_English-EWT patterns]  [Lightweight deps]
```

### Key Components to Add

```rust
// New module: crates/canopy-semantic-layer/src/treebank_matcher.rs

pub struct TreebankMatcher {
    // Core patterns from high-frequency verbs (~500KB memory)
    core_patterns: HashMap<SemanticSignature, DependencyPattern>,

    // Adaptive cache that grows based on usage (~1MB limit)
    adaptive_cache: LruCache<SemanticSignature, DependencyPattern>,

    // Lazy-loaded treebank index (not patterns themselves)
    treebank_index: TreebankIndex,

    // Pattern synthesis for unseen cases
    pattern_synthesizer: PatternSynthesizer,
}

pub struct SemanticSignature {
    lemma: String,
    verbnet_class: Option<String>,
    framenet_frame: Option<String>,
    theta_roles: Vec<ThetaRole>,
}

pub struct DependencyPattern {
    verb_lemma: String,
    dependencies: Vec<(DepRel, String)>,  // e.g., [(nsubj, "agent"), (obj, "patient")]
    confidence: f32,
    frequency: u32,
}
```

## Implementation Phases

### Phase 1: Parse UD Treebank and Build Index (Week 1)

#### Step 1.1: Create CoNLL-U Parser

```rust
// New file: crates/canopy-semantic-layer/src/conllu.rs

pub struct ConlluReader {
    // Parse UD_English-EWT format
}

pub fn load_treebank(path: &str) -> Result<Vec<ParsedSentence>, Error> {
    // Load from data/UD_English-EWT/en_ewt-ud-train.conllu
    // Parse ~16,000 sentences
}
```

#### Step 1.2: Build Semantic Index

```rust
// Process treebank through Layer 1 to create signature→pattern mappings

pub fn index_treebank(treebank_path: &str) -> Result<TreebankIndex, Error> {
    let analyzer = create_l1_analyzer()?;
    let sentences = load_treebank(treebank_path)?;
    let mut index = TreebankIndex::new();

    for sentence in sentences {
        // Run through existing Layer 1
        let semantic_result = analyzer.analyze(&sentence.text)?;

        // Extract dependency pattern from treebank
        let dep_pattern = extract_dependencies(&sentence);

        // Create mapping
        let signature = create_signature(&semantic_result);
        index.add_pattern(signature, dep_pattern);
    }

    // Save index for fast loading
    index.save_to_disk("data/cache/treebank_index.bin")?;
    Ok(index)
}
```

### Phase 2: Implement Adaptive Caching (Week 1-2)

#### Step 2.1: Core Pattern Extraction

```rust
// Extract top 500 most frequent patterns for core cache

pub fn extract_core_patterns(index: &TreebankIndex) -> HashMap<SemanticSignature, DependencyPattern> {
    // Sort by frequency
    // Take top 500 patterns (~500KB memory)
    // These cover ~70% of common sentences
}
```

#### Step 2.2: Adaptive Cache Implementation

```rust
impl AdaptiveCache {
    pub fn get_or_load(&mut self, signature: &SemanticSignature) -> Option<DependencyPattern> {
        // 1. Check core patterns (instant)
        // 2. Check adaptive cache (fast)
        // 3. Load from index if exists (slower, rare)
        // 4. Track usage for cache promotion
    }

    pub fn should_promote(&self, signature: &SemanticSignature) -> bool {
        // Promote to cache if used >N times
        self.usage_count.get(signature).map_or(false, |&count| count > 3)
    }
}
```

### Phase 3: Pattern Synthesis for Unknown Cases (Week 2)

#### Step 3.1: VerbNet to Dependency Rules

```rust
// Use VerbNet frames to synthesize dependency patterns

pub fn verbnet_to_dependencies(verbnet_class: &str) -> DependencyPattern {
    match verbnet_class {
        "give-13.1" => DependencyPattern {
            dependencies: vec![
                (DepRel::Nsubj, "agent"),
                (DepRel::Obj, "theme"),
                (DepRel::Iobj, "recipient"),
            ],
            confidence: 0.8,
        },
        // ~200 VerbNet classes map to common patterns
    }
}
```

#### Step 3.2: Fallback Pattern Generator

```rust
impl PatternSynthesizer {
    pub fn synthesize(&self, layer1: &Layer1SemanticResult) -> DependencyPattern {
        // 1. Try VerbNet mapping
        // 2. Try FrameNet frame elements
        // 3. Use positional heuristics
        // 4. Default to basic SVO pattern
    }
}
```

### Phase 4: Integration with Layer 1 (Week 2-3)

#### Step 4.1: Extend Layer1SemanticResult

```rust
// Modify existing structure
pub struct Layer1SemanticResult {
    // ... existing fields ...

    // NEW: Optional dependency structure
    pub dependencies: Option<DependencyPattern>,
    pub dependency_confidence: f32,
}
```

#### Step 4.2: Update SemanticCoordinator

```rust
impl SemanticCoordinator {
    pub fn analyze_with_dependencies(&mut self, text: &str) -> Result<Layer1SemanticResult> {
        // Existing semantic analysis
        let mut result = self.analyze(text)?;

        // NEW: Add dependency pattern matching
        if self.config.enable_dependency_matching {
            let signature = create_signature(&result);
            if let Some(pattern) = self.treebank_matcher.get_pattern(&signature) {
                result.dependencies = Some(pattern);
            }
        }

        Ok(result)
    }
}
```

## Resource Requirements

### Memory Budget

```text
Core patterns (500):        ~500KB
Adaptive cache (variable):  ~500KB-1MB
Treebank index (on disk):   ~5MB (memory-mapped as needed)
Pattern synthesizer:        ~100KB
-----------------------------------
Total RAM usage:            1-2MB additional
```

### Treebank Files

```text
Location: data/UD_English-EWT/
Files to use:
- en_ewt-ud-train.conllu (12,543 sentences)
- en_ewt-ud-dev.conllu   (2,001 sentences)
- en_ewt-ud-test.conllu  (2,077 sentences)
Total: ~16,600 sentences
```

### Performance Targets

```text
Cache hit latency:       <1μs
Pattern synthesis:       <10μs
Index lookup:           <100μs (rare)
Memory overhead:        <2MB
Coverage target:        85%+ sentences
```

## Testing Strategy

### Unit Tests

```rust
#[test]
fn test_treebank_loading() {
    let patterns = load_treebank("data/UD_English-EWT/en_ewt-ud-test.conllu")?;
    assert!(patterns.len() > 2000);
}

#[test]
fn test_pattern_matching() {
    let matcher = TreebankMatcher::new();
    let semantic = /* ... Layer 1 result ... */;
    let pattern = matcher.get_pattern(&semantic);
    assert!(pattern.confidence > 0.7);
}

#[test]
fn test_memory_budget() {
    let matcher = TreebankMatcher::with_memory_limit(2_000_000); // 2MB
    // Load patterns and verify memory usage
    assert!(matcher.memory_used() < 2_000_000);
}
```

### Integration Tests

```rust
#[test]
fn test_end_to_end_with_dependencies() {
    let coordinator = SemanticCoordinator::with_dependencies()?;
    let result = coordinator.analyze("John gave Mary a book")?;

    assert!(result.dependencies.is_some());
    let deps = result.dependencies.unwrap();
    assert!(deps.has_relation(DepRel::Nsubj));
    assert!(deps.has_relation(DepRel::Obj));
}
```

## Success Criteria

1. **Coverage**: 85%+ of sentences get dependency patterns
1. **Memory**: Total additional memory \<2MB
1. **Performance**: No significant slowdown to Layer 1 (\<10μs additional per analysis)
1. **Quality**: 80%+ accuracy on common constructions
1. **Integration**: Clean API that doesn't break existing Layer 1 users

## File Changes Required

### New Files

```
crates/canopy-semantic-layer/src/
├── treebank_matcher.rs     # Main pattern matching logic
├── conllu.rs               # CoNLL-U parser
├── dependency_pattern.rs   # Pattern types and synthesis
└── semantic_signature.rs   # Signature creation and hashing

crates/canopy-semantic-layer/tests/
└── treebank_integration_tests.rs

data/cache/
└── treebank_index.bin     # Preprocessed index (generated)
```

### Modified Files

```
crates/canopy-semantic-layer/src/
├── coordinator.rs          # Add dependency matching option
├── lib.rs                 # Export new types
└── types.rs               # Extend Layer1SemanticResult

crates/canopy-semantic-layer/
└── Cargo.toml             # Add conllu crate dependency
```

## Configuration

```rust
pub struct CoordinatorConfig {
    // ... existing fields ...

    // NEW: Dependency matching configuration
    pub enable_dependency_matching: bool,  // Default: false initially
    pub treebank_cache_size: usize,       // Default: 500 patterns
    pub adaptive_cache_size: usize,       // Default: 1000 patterns
    pub dependency_confidence_threshold: f32, // Default: 0.7
}
```

## Rollout Plan

1. **Week 1**: Implement basic treebank loading and indexing
1. **Week 1-2**: Add core pattern cache and adaptive caching
1. **Week 2**: Implement pattern synthesis for unseen cases
1. **Week 2-3**: Integrate with Layer 1, maintain backward compatibility
1. **Week 3**: Testing, optimization, and documentation
1. **Week 4**: Performance tuning and coverage analysis

## Dependencies to Add

```toml
[dependencies]
conllu = "0.4"  # For parsing CoNLL-U format
lru = "0.12"    # For adaptive cache
bincode = "1.3" # For index serialization
```

______________________________________________________________________

**END OF UD TREEBANK PATTERN MATCHING PLAN**
