# Hybrid Architecture: Semantic-Driven Tree Building & GPU Acceleration

## Executive Summary

canopy.rs implements a revolutionary hybrid architecture that combines:

1. **Semantic-Driven Tree Building**: Keep dependency parsing as foundation, build phrase structure only when semantic analysis requires it
2. **CPU-GPU Smart Routing**: Single sentences stay CPU (25-80μs), batches leverage GPU acceleration (0.15-0.5μs per sentence)
3. **Layered Processing**: 4-layer pipeline with intelligent complexity detection

## Core Architecture Strategy

### Dependency-First with Selective Enhancement

Keep dependency parsing as foundation (25-80μs), build phrase structure only when semantic analysis requires it.

```rust
// Detection phase in Layer 2
struct SemanticComplexityDetector {
    movement_signals: Vec<MovementSignal>,
    binding_complexity: BindingComplexity,
    event_depth: usize,
    attachment_ambiguity: f32,
}

impl SemanticComplexityDetector {
    fn needs_phrase_structure(&self) -> Option<StructureNeeds> {
        if self.has_movement() || self.has_complex_binding() {
            Some(StructureNeeds::MinimalXBar)
        } else {
            None  // Stay with dependencies
        }
    }
}
```

### Hybrid Tree Representation

```rust
enum HybridTree {
    // Fast path (95% of sentences)
    DependencyOnly(UDPipeParse),

    // Semantic analysis needs structure (4%)
    Hybrid {
        dependency_backbone: UDPipeParse,
        phrase_structure: Vec<XBarFragment>,
        movement_domains: Vec<StructuralDomain>,
    },

    // Complex cases (1%)
    PhraseStructure(FullXBarTree),
}
```

### Performance Profile (Tokenizer Compatibility)

- **95% of sentences**: 25-80μs (unchanged, dependencies only)
- **4% of sentences**: 150-350μs (selective phrase structure)
- **1% of sentences**: 300-500μs (full phrase structure)
- **Average**: ~100-200μs (faster than BPE tokenization!)
- **Ceiling**: <500μs per sentence (maintains tokenizer compatibility)

## CPU-GPU Smart Routing Architecture

### Processing Mode Selection

```rust
enum ProcessingMode {
    Single(CPUPipeline),           // 1-5 sentences (25-80μs)
    SmallBatch(HybridPipeline),    // 5-100 sentences (GPU starts winning)
    MassiveBatch(GPUPipeline),     // 100+ sentences (massive speedup)
}

struct SmartBatchProcessor {
    pending_requests: Vec<LSPRequest>,
    batch_timeout: Duration,        // 1-5ms window
    batch_size_threshold: usize,    // 5-10 sentences
}
```

### Layer-Specific GPU Opportunities

#### **Layer 1 (Morphosyntax + VerbNet)** 🔥 **High Priority**
- Parallel VerbNet lookup across all words
- Batch feature extraction (animacy, definiteness, etc.)
- Selectional restriction evaluation

```rust
struct Layer1Pipeline {
    cpu_parser: UDPipeParser,        // Sequential parsing (CPU)
    gpu_verbnet: GPUVerbNetEngine,   // Parallel semantic enhancement (GPU)
}
```

#### **Layer 2 (Event Structure)** 🚀 **Massive Wins**
- Parallel theta role scoring (word × role combinations)
- Movement hypothesis exploration
- Event structure pattern matching

#### **Layer 3 (Compositional Semantics)** 🎯 **Biggest Opportunity**
- Parallel lambda term β-reduction
- Quantifier scope enumeration (massive parallelism potential)
- DRS construction and merging

#### **Layer 4 (Discourse)** 📊 **Batch Processing**
- All-pairs coreference scoring
- Parallel contradiction detection
- Batch consistency checking across propositions

## Performance Scaling by Architecture

### Single vs Batch Performance (Tokenizer Focus)
| Batch Size | CPU Time | GPU Time | Speedup | Architecture Used | Tokenizer Compat |
|------------|----------|----------|---------|------------------|------------------|
| 1 | 25-80μs | 25-80μs | 1x | CPU + Dependencies | ✅ <500μs |
| 1 (complex) | 300-500μs | 300-500μs | 1x | CPU + Full Analysis | ✅ <500μs |
| 10 | 250-800μs | 50-150μs | 5-16x | CPU + Selective Structure | ✅ Individual <500μs |
| 100 | 2.5-8ms | 100-200μs | 12-80x | GPU + Hybrid Trees | ✅ Per-sentence <500μs |
| 1000 | 25-80ms | 150-500μs | 50-533x | GPU + Massive Parallel | ✅ Per-sentence <500μs |

### Intelligent Batching for LSP

```rust
impl SmartBatchProcessor {
    async fn process_request(&mut self, request: LSPRequest) -> LSPResponse {
        self.pending_requests.push(request);
        
        // Immediate processing for small requests
        if self.pending_requests.len() == 1 {
            // Start timeout timer
            let timeout = tokio::time::sleep(self.batch_timeout);
        }
        
        // Batch processing when threshold reached
        if self.pending_requests.len() >= self.batch_size_threshold {
            return self.process_batch().await;
        }
        
        // ... timeout handling
    }
    
    async fn process_batch(&mut self) -> Vec<LSPResponse> {
        let batch = std::mem::take(&mut self.pending_requests);
        
        if batch.len() < self.gpu_advantage_threshold {
            // CPU processing with selective structure
            self.cpu_pipeline.process_batch(batch).await
        } else {
            // GPU processing with hybrid trees
            self.gpu_pipeline.process_batch(batch).await
        }
    }
}
```

## Layer Integration Architecture

### Layer 1 → Layer 2 Pipeline

```rust
struct Layer1Output {
    words: Vec<EnhancedWord>,
    sentence_features: SentenceFeatures,
    parse_metadata: ParseMetadata,
}

struct EnhancedWord {
    // UDPipe outputs
    udpipe_analysis: UDPipeWord,

    // VerbNet enhancements (current implementation)
    theta_potential: Vec<ThetaRole>,        // All possible theta grids
    verbnet_class: Option<VerbNetClass>,    // All matching classes
    selectional_restrictions: Vec<Constraint>,

    // Derived features
    animacy: Option<Animacy>,
    voice_contribution: Option<Voice>,
    semantic_type: Option<SemanticType>,

    // Movement signals (to be added)
    movement_signals: Vec<MovementSignal>,
}
```

### Layer 2 → Layer 3 Pipeline

```rust
struct Layer2Output {
    events: Vec<Event>,
    discourse_entities: Vec<Entity>,
    semantic_relations: Vec<Relation>,
    movement_chains: Vec<MovementChain>,  // Basic chains
}

// Neo-Davidsonian event representation
struct Event {
    id: EventId,
    predicate: Predicate,
    participants: HashMap<ThetaRole, Participant>,
    modifiers: Vec<Modifier>,
    aspect: AspectualClass,
    little_v: Option<LittleV>,
    movement_chains: Vec<MovementChain>,
}
```

## Movement Theory Progression Architecture

### Phase 1: Government & Binding (M3-M4)
```rust
// Detection without full representation
enum MovementSignal {
    PassiveVoice(Participant),        // "John was seen"
    WhConstruction(WhElement),         // "What did John see?"
    RaisingPattern(Subject),          // "John seems to leave"
    ToughConstruction(Object),        // "John is easy to please"
}

struct GBMovementChain {
    antecedent: Participant,
    trace: TraceElement,
    governing_category: SyntacticDomain,
    case_assignment: CasePosition,
}
```

### Phase 2: A/A-bar Movement (M4-M5)
```rust
enum ChainType {
    AMovement,      // Passive, raising, unaccusatives
    ABarMovement,   // Wh-movement, topicalization
    HeadMovement,   // V-to-T (defer to M6+)
}

struct MovementChain {
    moved_element: Participant,
    chain_type: ChainType,
    positions: Vec<ChainPosition>,
}
```

### Phase 3: Minimalist Movement (M5-M6)
```rust
struct MinimalistMovement {
    trigger_feature: Feature,
    target_position: TargetHead,
    copy_chain: Vec<Copy>,
    feature_checking: FeatureMatrix,
}
```

### Phase 4: Multi-dominance (M6+)
```rust
struct SharedStructure {
    shared_node: SyntacticNode,
    parent_positions: Vec<StructuralPosition>,
    sharing_mechanism: SharingType,
}
```

## GPU Memory Management Architecture

### GPU Memory Layout
```rust
struct GPUVerbNetDatabase {
    // Optimized for GPU access patterns
    classes: GPUBuffer<VerbNetClass>,       // Linear array for coalesced access
    verbs: GPUBuffer<VerbEntry>,            // Hash table for O(1) lookup
    theta_roles: GPUBuffer<ThetaRole>,      // All possible roles
    
    // Precomputed similarity matrices for faster scoring
    semantic_similarity: GPUBuffer<f32>,    // Precomputed semantic distances
    selectional_matrix: GPUBuffer<u8>,      // Restriction compatibility
}

// Efficient data structures for GPU
#[repr(C)]
struct GPUWord {
    lemma_hash: u32,        // Hash for fast comparison
    pos_tag: u8,            // Packed POS information
    features: u32,          // Bit-packed morphological features
    // Align to 16 bytes for GPU efficiency
}
```

### Buffer Management
```rust
struct GPUBufferPool {
    input_buffers: Vec<GPUBuffer<GPUWord>>,
    result_buffers: Vec<GPUBuffer<VerbNetResult>>,
    scratch_buffers: Vec<GPUBuffer<f32>>,
    
    // Reuse buffers to avoid allocation overhead
    available_buffers: VecDeque<usize>,
}
```

## Technology Stack Integration

### Core Technologies
- **wgpu**: Cross-platform WebGPU implementation
- **WGSL**: WebGPU Shading Language for compute shaders
- **Tower-LSP**: Async LSP server implementation
- **UDPipe**: Lightweight dependency parsing
- **VerbNet**: Comprehensive semantic role database

### Cross-Platform Support
- **Platforms**: Windows, macOS, Linux, Web (via WASM)
- **GPU Vendors**: NVIDIA, AMD, Intel, Apple Silicon
- **Fallback**: Automatic CPU fallback when GPU unavailable

## Implementation Timeline

### M3: Event Structure & Movement Detection (Weeks 6-8)
- Implement semantic complexity detector
- Basic GB movement signal detection
- Hybrid tree foundation (dependency + selective phrase structure)

### M4: A/A-bar Movement & DRT Foundation (Weeks 9-12) 
- A/A-bar movement distinction
- Selective phrase structure triggers
- Basic GPU acceleration proof-of-concept

### M5: Minimalist Movement & LSP Integration (Weeks 13-15)
- Minimalist movement features
- Enhanced LSP with semantic information
- Performance evaluation on UD treebanks

### M6: Multi-dominance & GPU Acceleration (Weeks 16-17)
- Complete multi-dominance implementation
- Full GPU compute acceleration
- Smart batching architecture

## Success Metrics

### Architecture Performance (Tokenizer Compatibility)
- **95% sentences**: Stay dependency-only (25-80μs)
- **4% sentences**: Selective structure (150-350μs)  
- **1% sentences**: Full phrase structure (300-500μs)
- **Tokenizer ceiling**: <500μs per sentence (faster than BPE!)
- **Average overhead**: <20% for semantic complexity detection

### GPU Acceleration
- **Single sentence**: 25-80μs (CPU, unchanged)
- **Small batches (10)**: 15μs per sentence (2-5x speedup)
- **Large batches (1000+)**: 0.15-0.5μs per sentence (50-533x speedup)

### LSP Integration (Tokenizer Performance)
- **Hover responses**: <100μs for full semantic analysis
- **Tokenizer integration**: <500μs per sentence (compatible with ML training)
- **Semantic diagnostics**: Movement violations, binding errors
- **Code actions**: Voice transformation, pronoun resolution

## Risk Mitigation

### Complexity Management
- **Incremental implementation**: Start with simple detection, build complexity gradually
- **Performance monitoring**: Continuous benchmarking with regression detection
- **Graceful degradation**: Always fallback to dependency parsing

### GPU Availability
- **Universal fallback**: CPU pipeline always available
- **Runtime detection**: Automatic GPU capability assessment
- **Cross-platform testing**: Validate on all target platforms

## Future Extensions

### Research Applications (Tokenizer Revolution)
- **Linguistic tokenizer**: Replace BPE with <500μs full semantic analysis
- **ML training integration**: Real-time augmentation faster than traditional tokenization
- **Corpus processing**: Web-scale linguistic analysis
- **Theory testing**: Computational linguistic hypothesis evaluation

### Production Features
- **Streaming analysis**: Large document processing
- **Distributed processing**: Multi-GPU scaling
- **Plugin architecture**: Custom linguistic theories

## Conclusion

The hybrid architecture represents a fundamental breakthrough in computational linguistics:

1. **Semantic Intelligence**: Build structure only when semantics requires it
2. **Performance Scaling**: Maintain 25-80μs for simple cases, scale to sub-microsecond for batches
3. **Theoretical Grounding**: Implement movement theory progression from GB to Minimalism
4. **Practical Integration**: Real-time LSP with rich semantic information

This positions canopy.rs as the first system to successfully bridge formal linguistic theory with practical, high-performance NLP applications.