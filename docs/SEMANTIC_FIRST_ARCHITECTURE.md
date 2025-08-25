# Pure Semantic-First Architecture

## Executive Summary

canopy.rs has transitioned to a **pure semantic-first architecture** that eliminates dependency on black-box parsers (UDPipe) in favor of direct manipulation of semantic databases (FrameNet, VerbNet, WordNet). This architectural shift provides complete control over linguistic analysis while maintaining theoretical rigor.

## Architecture Philosophy

### From Syntax-First to Semantics-First

**Previous Architecture (UDPipe-First)**:
```text
Text → UDPipe Parse → Semantic Analysis → Events → DRT
       (Black Box)    (Derived from Syntax)
```

**Current Architecture (Semantic-First)**:
```text
Text → Semantic DB Query → Event Construction → DRT
       (FrameNet/VerbNet)  (Direct from Semantics)
```

### Core Principles

1. **Complete Transparency**: Every linguistic decision traceable to specific database entries
2. **Theory-Driven**: All processing based on established linguistic frameworks
3. **Direct Control**: No black-box dependencies - we control the entire pipeline
4. **Expandable Coverage**: Add new vocabulary through semantic database extensions

## Key Components

### Layer 1: Pure Semantic Analysis

```rust
struct SemanticLayer1 {
    framenet: FrameNetEngine,     // Frame detection & lexical units
    verbnet: VerbNetEngine,       // Verb classes & theta roles
    wordnet: WordNetEngine,       // Sense disambiguation & hypernyms
    morphology: MorphologyDB,     // Inflection analysis
    lexicon: ClosedClassLexicon,  // Function words
}
```

**Input**: Raw text
**Output**: Semantic structures with frames, predicates, and logical forms

### Resource Integration Strategy

#### Primary Semantic Resources
- **FrameNet**: 1,200+ frames, 13,000+ lexical units
- **VerbNet**: 6,791 verb senses, 270+ classes with theta roles
- **WordNet**: 155,287+ words with semantic relations
- **PropBank**: Predicate-argument structures
- **Custom Lexicons**: Closed-class words, morphological patterns

#### Coverage Strategy
- **High-Frequency Vocabulary**: Core 10,000 words through semantic databases
- **Domain Extensions**: Expandable lexicons for specialized vocabulary
- **Compositional Analysis**: Handle compounds and derivatives
- **Context Inference**: Use semantic patterns for unknown words

## Verb Detection Without UDPipe

### Multi-Resource Verb Detection
```rust
fn detect_verbs(token: &str, db: &SemanticDatabase) -> Option<VerbAnalysis> {
    // Priority order:
    // 1. VerbNet classes (primary verb identifier)
    // 2. FrameNet predicates (frame-evoking elements)
    // 3. Morphological patterns (inflection matching)
    // 4. Auxiliary patterns (be/have/will/can/etc.)
}
```

### Advantages Over UDPipe
- **Semantic Context**: Verbs identified through meaning, not just form
- **Theta Role Information**: Immediate access to argument structure
- **Aspectual Classes**: Built-in Vendler classification
- **Frame Information**: Rich semantic context from FrameNet

## Performance Characteristics

### Target Performance
- **Single Sentence**: <200μs (with GPU acceleration)
- **Batch Processing**: 0.15-0.5μs per sentence (GPU parallel)
- **Coverage**: 90%+ on common vocabulary
- **Accuracy**: 95%+ verb detection, 100% semantic role assignment

### GPU Acceleration Strategy
```rust
struct GPUSemanticEngine {
    framenet_tensors: DeviceBuffer,     // Preloaded frames
    verbnet_tensors: DeviceBuffer,      // Preloaded verb classes
    batch_processor: ParallelProcessor, // Concurrent analysis
}
```

## Testing & Quality Requirements

### Coverage Requirements
- **New Code**: 95% minimum test coverage (raised from 80%)
- **Critical Paths**: 100% coverage for semantic analysis
- **Integration Tests**: Full pipeline validation

### Quality Metrics
- **Semantic Accuracy**: 95%+ correct frame/verb detection
- **Completeness**: 90%+ vocabulary coverage through databases
- **Performance**: <200μs semantic analysis per sentence
- **Transparency**: 100% traceable decisions

## Migration Benefits

### What We Gain
1. **Complete Control**: Every analysis decision under our control
2. **Theoretical Soundness**: Direct implementation of linguistic theory
3. **Expandability**: Easy to add new vocabulary and patterns
4. **Debuggability**: No black-box parsing - full traceability
5. **GPU Optimization**: Semantic databases ideal for parallel processing

### What We Accept
1. **Limited Initial Vocabulary**: ~20,000 words vs unlimited (but expandable)
2. **Development Overhead**: Must build our own tokenization/morphology
3. **Coverage Gaps**: Need robust expansion strategy for unknown words

## Implementation Roadmap

### Phase 1: Foundation (Current)
- [x] Document architecture transition
- [ ] Create semantic-layer module structure
- [ ] Integrate FrameNet database
- [ ] Build basic tokenization

### Phase 2: Core Semantic Engine
- [ ] Implement multi-resource verb detection
- [ ] Build semantic role assignment from frames
- [ ] Create morphological analysis system
- [ ] Develop closed-class lexicon

### Phase 3: Integration
- [ ] Modify Layer 2 for semantic input
- [ ] Remove UDPipe dependencies
- [ ] Update pipeline architecture
- [ ] Comprehensive testing at 95% coverage

### Phase 4: Optimization
- [ ] GPU acceleration implementation
- [ ] Semantic caching system
- [ ] Performance optimization
- [ ] Coverage expansion tools

## Theoretical Foundations

This architecture directly implements:

- **Frame Semantics** (Fillmore): FrameNet frames as primary representation
- **Neo-Davidsonian Semantics**: Event structures from semantic analysis
- **Theta Theory**: Direct VerbNet theta role assignment
- **Lexical Semantics**: WordNet for sense disambiguation and relations

## Success Metrics

### Technical Success
- [x] Architecture transition documented
- [ ] 95% test coverage on all new code
- [ ] <200μs semantic analysis performance
- [ ] 90%+ vocabulary coverage
- [ ] GPU acceleration operational

### Linguistic Success
- [ ] 95%+ verb detection accuracy
- [ ] Semantic frame assignment correctness
- [ ] Theta role accuracy maintained at 100%
- [ ] Event structures from pure semantic input

This pure semantic-first architecture represents a fundamental paradigm shift toward complete theoretical transparency and control over linguistic analysis.