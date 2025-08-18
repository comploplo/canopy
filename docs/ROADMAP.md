# canopy.rs Roadmap
**Rust Implementation of spaCy-LSP V2**

**Philosophy**: Infrastructure-first development with rigorous benchmarking, developer experience excellence, and performance-driven milestones.

---

## Development Principles

### 🎯 **Performance-First Mindset**
- Establish baselines before building features
- Continuous benchmarking with regression gates
- 10x improvement target over Python V1 (sub-50ms LSP responses)
- Memory efficiency and zero-copy parsing where possible

### 🛠️ **Developer Experience Excellence**
- Comprehensive tooling from day one
- Fast feedback loops (sub-second test runs)
- Clear error messages and debugging support
- Automated quality gates that prevent regressions

### 📊 **Measurement-Driven Development**
- Benchmark every major component
- Track memory usage, latency, and throughput
- Performance regression detection in CI
- Regular baseline updates and analysis

---

## Milestone Overview

| Milestone | Focus | Duration | Key Deliverable |
|-----------|-------|----------|-----------------|
| **M1** | Foundation & Tooling | 2 weeks | ✅ Benchmarked development environment |
| **M2** | Core Types & Parsing | 3 weeks | ✅ UDPipe-first integration with unified semantic features |
| **M3** | Event Structure | 3 weeks | Event structures with theta role assignment |
| **M4** | Compositional Semantics | 4 weeks | DRT and lambda calculus implementation |
| **M5** | LSP Integration | 3 weeks | Working language server with diagnostics |
| **M6** | Performance & Polish | 2 weeks | 10x performance goal achievement |
| **M7** | Advanced Theory | 4 weeks | Full null node inventory and complex syntax |

**Total Timeline**: ~5 months with infrastructure-heavy front-loading

---

## M1: Foundation & Developer Tooling (Weeks 1-2)
**Goal**: Establish world-class development environment with performance infrastructure

### 🏗️ **Project Scaffolding**
- [ ] **Cargo workspace setup** with library + binary crates
- [ ] **CI/CD pipeline** (GitHub Actions) with Rust toolchain
- [ ] **Development scripts** (`just` command runner for common tasks)
- [ ] **Docker environment** for reproducible builds and testing
- [ ] **Documentation infrastructure** (mdbook for design docs)

### 📊 **Benchmarking Infrastructure** 
- [ ] **Criterion.rs integration** for micro-benchmarks
- [ ] **Performance regression detection** with automatic baseline comparison
- [ ] **Memory profiling setup** (heaptrack, valgrind integration)
- [ ] **Flamegraph generation** for performance analysis
- [ ] **Benchmark CI integration** with performance gates

### 🧪 **Testing Framework**
- [ ] **Property-based testing** with proptest for linguistic invariants
- [ ] **Golden test framework** for deterministic output validation
- [ ] **Integration test harness** for end-to-end LSP scenarios
- [ ] **Fuzzing setup** (cargo-fuzz) for parser robustness
- [ ] **Coverage tracking** with tarpaulin

### 🛠️ **Developer Experience**
- [ ] **Fast compilation** optimization (sccache, parallel builds)
- [ ] **IDE setup** (rust-analyzer configuration)
- [ ] **Pre-commit hooks** (formatting, linting, quick tests)
- [ ] **Development documentation** (contributing guide, architecture overview)
- [ ] **Error handling strategy** (custom error types, good error messages)

### ✅ **Quality Gates (M1)**
- All tests pass in <10 seconds
- Benchmark suite runs in <30 seconds
- Documentation builds without warnings
- Linting passes (clippy::pedantic level)
- Zero security vulnerabilities (cargo audit)

---

## M2: Core Types & UDPipe Integration (Weeks 3-5)
**Goal**: Foundational parsing with rigorous performance measurement

### 🏗️ **Type System Foundation**
- [ ] **Core linguistic types** (Word, Sentence, Document with builder patterns)
- [ ] **Morphological features** (POS, dependencies, morphology)
- [ ] **Error handling** (parsing failures, malformed input)
- [ ] **Serialization support** (serde integration for caching)
- [ ] **Memory layout optimization** (investigate repr(C), alignment)

### 🔌 **UDPipe Integration**
- [ ] **Rust bindings** for UDPipe C++ library
- [ ] **Model loading** and caching strategy
- [ ] **Batch processing** support for multi-sentence documents
- [ ] **Error recovery** for malformed or unusual input
- [ ] **Cross-linguistic support** planning (English first, extensible design)

### 📊 **Performance Baseline Establishment**
- [ ] **Parsing benchmarks** (sentences/second, memory per sentence)
- [ ] **Python V1 comparison** (establish 1x baseline for improvement tracking)
- [ ] **Memory profiling** (heap allocation patterns, potential leaks)
- [ ] **Latency analysis** (p50, p95, p99 parsing times)
- [ ] **Throughput testing** (concurrent parsing, resource utilization)

### 🧪 **Comprehensive Testing**
- [ ] **Unit tests** for all public APIs
- [ ] **Property tests** (parse→serialize→parse roundtrips)
- [ ] **Golden tests** vs Python V1 outputs where applicable
- [ ] **Stress testing** (large documents, malformed input)
- [ ] **Cross-platform testing** (Linux, macOS, Windows)

### ✅ **Quality Gates (M2) - COMPLETE**
- ✅ **Real UDPipe FFI integration** with enhanced tokenization (7-76μs)
- ✅ **Comprehensive golden test validation** (6 tests covering all aspects)
- ✅ **UDPipe-first feature extraction** (12 morphological features)
- ✅ **Unified semantic system** (UDPipe + VerbNet + legacy compatibility)
- ✅ **Clean codebase** (zero warnings, zero TODO comments)
- ✅ **Exceptional performance** (12,500-40,000 sentences/second)
- ✅ **Memory-efficient infrastructure** (bounded allocation)
- ✅ **Complete test coverage** (95+ tests across all crates)
- ✅ **Zero panics on malformed input**
- ✅ **Deterministic output for identical input**

### 🚀 **M3 READY TO PROCEED**
M3 event structure implementation can now begin with:
✅ Validated UDPipe integration providing syntactic foundation
✅ VerbNet framework ready for theta role assignment
✅ Clean, optimized codebase with massive performance headroom

---

## M3: Event Structure & Movement Detection (Weeks 6-8)
**Goal**: Neo-Davidsonian events with theta role assignment and movement signal detection

### 🎯 **Event Structure Implementation** 
- [ ] **Neo-Davidsonian events** (Event, Participant, Predicate types)
- [ ] **Theta role assignment** (Agent, Patient, Theme, etc. - 30 roles from VerbNet)
- [ ] **VerbNet semantic predicates** (Motion, Transfer, Location - 50+ predicate types)
- [ ] **Confidence scoring** for role assignments
- [ ] **Event composition** (simple coordination, modification)

### 🔄 **Movement Signal Detection (GB Foundation)**
- [ ] **Movement signal detection** (PassiveVoice, WhConstruction, RaisingPattern, ToughConstruction)
- [ ] **Basic GB movement chains** (antecedent-trace relationships)
- [ ] **Expanded little v decomposition** (Cause, Become, Do, Be, Go, Have)
- [ ] **Voice detection** (active, passive, middle voice)
- [ ] **Hybrid tree preparation** (semantic complexity detection for selective phrase structure)

### 📊 **Semantic Performance Optimization**
- [ ] **Role assignment benchmarks** (accuracy vs speed tradeoffs)
- [ ] **VerbNet lookup optimization** (trie structures, caching)
- [ ] **Batch semantic analysis** for multi-sentence efficiency
- [ ] **Memory pool allocation** for frequent small objects (roles, participants)

### 🧪 **Linguistic Testing**
- [ ] **VerbNet accuracy testing** (>90% agreement with manual annotation)
- [ ] **Cross-validation** with Python V1 outputs
- [ ] **Edge case handling** (unknown verbs, ambiguous structures)
- [ ] **Performance regression tests** (semantic analysis speed gates)

### ✅ **Quality Gates (M3)**
- Theta role assignment >90% accuracy on VerbNet test corpus
- Total analysis time stays <500μs per sentence (tokenizer compatibility)
- Semantic analysis adds <200μs to parsing baseline
- VerbNet integration covers 1000+ verb patterns with semantic predicates
- Movement signal detection for basic constructions
- Graceful degradation for unknown verbs

---

## M4: A/A-bar Movement & DRT Foundation (Weeks 9-12)
**Goal**: A/A-bar movement distinction with basic DRT compositional semantics

### 🔄 **A/A-bar Movement Implementation**
- [ ] **A-movement chains** (Passive, raising, unaccusatives) 
- [ ] **A-bar movement chains** (Wh-movement, topicalization)
- [ ] **Movement chain representation** (moved element, positions, chain type)
- [ ] **Selective phrase structure** (dependency backbone + phrase fragments)
- [ ] **Hybrid tree construction** (semantic triggers determine structure building)

### 🧮 **Basic Lambda Calculus & DRT**
- [ ] **Type system** (e, t, s types with function composition)
- [ ] **Lambda term construction** and normalization  
- [ ] **Basic DRS construction** from event structures
- [ ] **Beta reduction** with performance optimization
- [ ] **Type inference** and unification

### 📖 **Discourse Representation Theory**
- [ ] **DRS construction** from event structures
- [ ] **Quantifier scope** resolution (basic cases)
- [ ] **Presupposition handling** (simple presupposition projection)
- [ ] **Discourse merging** operations
- [ ] **Truth condition evaluation** framework
- [ ] **Temporal nodes** as semantic operators (null nodes for time expressions)

### 🔧 **Composition Framework**
- [ ] **Compositional rules** (function application, predicate modification)
- [ ] **Determiner phrase** composition
- [ ] **Coordination semantics** (Boolean conjunction)
- [ ] **Adjunct integration** (manner, temporal, locative modifiers)
- [ ] **Complex predicate** handling

### 📊 **Composition Performance**
- [ ] **Lambda term benchmarks** (construction, reduction, normalization times)
- [ ] **DRS complexity analysis** (memory usage vs discourse complexity)
- [ ] **Composition depth limits** (prevent exponential blowup)
- [ ] **Caching strategies** for repeated subcomputations

### ✅ **Quality Gates (M4)**
- A/A-bar movement distinction correctly identifies construction types
- Selective phrase structure triggers on semantic complexity
- Total analysis time stays <500μs per sentence (tokenizer compatibility)
- Compositional semantics adds <250μs to total analysis time
- DRS construction handles basic quantifier scope cases
- Lambda calculus passes Church-Rosser property tests
- Hybrid tree architecture maintains 95% dependency-only performance

---

## M5: Minimalist Movement & LSP Integration (Weeks 13-15)
**Goal**: Minimalist movement features with enhanced LSP semantic information

### 🔄 **Minimalist Movement Theory**
- [ ] **Feature-driven movement** (trigger features, target positions)
- [ ] **Copy chain representation** (multiple copies with feature checking)
- [ ] **Feature matrix system** (EPP, Case, Agreement features)
- [ ] **Merge and Move operations** with feature satisfaction
- [ ] **Phase-based derivation** (basic phase boundaries)

### 🌐 **Enhanced LSP Server Implementation**
- [ ] **Semantic hover** (theta roles, movement chains, lambda terms)
- [ ] **Movement chain diagnostics** (binding violations, improper movement)
- [ ] **Text synchronization** (document updates, incremental parsing)
- [ ] **Request handling** with rich semantic information
- [ ] **Response caching** for repeated requests

### 🔍 **Rich Diagnostics**
- [ ] **Semantic diagnostics** (theta role violations, binding errors)
- [ ] **Contradiction detection** (simple logical contradictions)
- [ ] **Binding theory** violations (Principles A, B, C)
- [ ] **Agreement checking** (subject-verb, determiner-noun)
- [ ] **Performance diagnostics** (analysis time warnings)
- [ ] **Null node analysis** (PRO in control structures, traces in movement)

### 💡 **Intelligent Features**
- [ ] **Semantic hover** (theta roles, lambda terms, truth conditions)
- [ ] **Code actions** (voice transformation, pronoun resolution)
- [ ] **Semantic navigation** (entity references, coreference chains)
- [ ] **Code lens** (truth evaluation, semantic complexity metrics)

### 📊 **LSP Performance Optimization**
- [ ] **Response time benchmarks** (<50ms target for hover requests)
- [ ] **Incremental analysis** (only recompute changed sentences)
- [ ] **Background processing** for expensive computations
- [ ] **Memory management** (bounded cache sizes, LRU eviction)

### ✅ **Quality Gates (M5)**
- Minimalist movement correctly handles feature-driven derivations
- Total analysis time stays <500μs per sentence (tokenizer compatibility)
- LSP responses <100μs for hover requests (including full semantic analysis)
- Movement chain information available in LSP hover
- Rich semantic diagnostics provide actionable feedback
- Performance evaluation on UD treebanks demonstrates accuracy
- Integration tests pass with VS Code, Neovim, Emacs

---

## M6: Multi-dominance & GPU Acceleration (Weeks 16-17)
**Goal**: Complete multi-dominance implementation with GPU compute acceleration

### 🔄 **Multi-dominance Structures**
- [ ] **Shared argument structures** (control, raising with multiple parents)
- [ ] **Complex movement chains** (successive-cyclic movement)
- [ ] **Multi-dominance handling** for shared constituents
- [ ] **Advanced little v interactions** (complex event decomposition)
- [ ] **Cross-linguistic parameterization** framework

### ⚡ **GPU Compute Acceleration**
- [ ] **wgpu integration** (cross-platform GPU compute)
- [ ] **VerbNet GPU kernels** (parallel lookup and scoring)
- [ ] **Theta role GPU acceleration** (massive parallel assignment)
- [ ] **Smart batching architecture** (CPU for n<5, GPU for n≥5)
- [ ] **Memory buffer management** (efficient GPU-CPU transfers)

### 🏭 **Production Readiness**
- [ ] **Comprehensive logging** (structured logging with tracing)
- [ ] **Configuration system** (TOML config files, environment variables)
- [ ] **Configurable memory-efficient parser** (selective VerbNet metadata attachment)
- [ ] **Resource management** (bounded memory usage, graceful shutdown)
- [ ] **Security audit** (dependency vulnerabilities, input validation)
- [ ] **Documentation polish** (user guides, API docs, troubleshooting)

### 📊 **Final Performance Validation**
- [ ] **10x improvement verification** vs Python V1 baseline
- [ ] **Stress testing** (sustained load, memory pressure)
- [ ] **Real-world benchmarks** (large codebases, complex documents)
- [ ] **Performance regression CI** (automated performance gates)

### 🚀 **Release Preparation**
- [ ] **Packaging** (cargo install, pre-built binaries)
- [ ] **Installation guide** (multiple platforms, editor integration)
- [ ] **Migration guide** from Python V1
- [ ] **Performance comparison** documentation
- [ ] **Roadmap for future development**

### ✅ **Quality Gates (M6)**
- Multi-dominance structures handle complex coordination correctly
- Single sentence analysis stays <500μs (tokenizer compatibility)
- GPU acceleration achieves <1μs per sentence for large batches (n≥100)
- Smart batching routes requests optimally (CPU vs GPU)
- 200x performance improvement over Python V1 verified
- Production stability with bounded memory usage
- Ready for web-scale corpus processing and ML tokenizer integration

---

## M7: Advanced Theoretical Features (Weeks 18-21)
**Goal**: Complete null node inventory and complex syntactic structures

### 🧠 **Advanced Null Nodes**
- [ ] **PRO in control structures** ("John wants [PRO to leave]")
- [ ] **Traces in movement chains** ("What did John see t?", "John was seen t")
- [ ] **Empty operators** (relative clauses, tough constructions)
- [ ] **Expletive subjects** (there-insertion, weather-it)
- [ ] **Parasitic gaps** and complex dependencies

### 🔄 **Complex Multi-dominance**
- [ ] **Full little v interactions** (complex event decomposition)
- [ ] **Multi-dominance structures** (shared arguments, control)
- [ ] **Complex movement chains** (successive-cyclic movement)
- [ ] **Optimality Theory** constraint evaluation
- [ ] **Cross-linguistic parameterization** framework

### 🧮 **Advanced Compositional Semantics**
- [ ] **Complex quantifier interactions** (inverse scope, polyadic quantification)
- [ ] **Information structure** (topic/focus marking)
- [ ] **Presupposition projection** (complex cases)
- [ ] **Discourse coherence** relations (temporal, causal)
- [ ] **Modal operators** and possible worlds semantics

### 🔬 **Research Integration**
- [ ] **Theory testing framework** (constraint violation detection)
- [ ] **Cross-linguistic analysis** tools
- [ ] **Corpus pattern discovery** (unsupervised linguistic universals)
- [ ] **Academic publication** support (LaTeX export, citations)

### 🔧 **API Enhancement & Feature Discovery**
- [ ] **Evaluate VerbNet API expansion** (review M2 VerbNet integration for additional features)
- [ ] **UDPipe feature exploration** (identify underutilized morphological features, enhanced dependencies)
- [ ] **Cross-reference integration** (WordNet synsets, FrameNet mappings, PropBank roles)
- [ ] **Advanced selectional restriction** utilization (complex feature inference, semantic typing)
- [ ] **Aspectual class expansion** (leverage VerbNet predicates for fine-grained Vendler classification)
- [ ] **Corpus-driven enhancement** (identify high-value features from large-scale analysis)

### 📄 **Context Window & Discourse Processing**
- [ ] **Paragraph-level processing architecture** (extend from sentence to paragraph context windows)
- [ ] **Memory scaling analysis** (optimize for 5-50 sentence paragraphs: 250KB-2.5MB memory budget)
- [ ] **Discourse coherence tracking** (entity persistence, topic continuity across sentences)
- [ ] **Cross-sentence semantic effects** (coreference resolution, presupposition projection)
- [ ] **Incremental discourse processing** (streaming paragraph analysis for large documents)
- [ ] **Context window configuration** (user-configurable processing units: sentence/paragraph/document)

### ✅ **Quality Gates (M7)**
- Complete null node coverage for English syntax
- Multi-dominance structures handle complex coordination
- OT constraint evaluation produces optimal candidates
- Information structure interfaces with prosody
- Research framework supports linguistic hypothesis testing

---

## M8: Neurosymbolic AI Integration (Weeks 22-25)
**Goal**: ML training integration and linguistic tokenization for neural language models

### 🤖 **Linguistic Tokenization**
- [ ] **PyO3 bindings** for Python ML frameworks (PyTorch, JAX)
- [ ] **HuggingFace tokenizer integration** (replacing BPE with linguistic analysis)
- [ ] **Structured token representation** (theta roles, event structure, movement chains)
- [ ] **Backward compatibility** with existing transformer models
- [ ] **Zero-copy tensor integration** (efficient GPU memory sharing)

### 🧠 **Training Pipeline Integration**
- [ ] **Real-time augmentation** (process training batches with 0.5μs overhead)
- [ ] **Linguistic curriculum learning** (sort 100M sentences by complexity in 45 minutes)
- [ ] **Compositional loss functions** (penalize violations of linguistic constraints)
- [ ] **Structured attention mechanisms** (guide transformer attention with syntactic structure)
- [ ] **Cross-linguistic transfer** (universal dependencies for multilingual models)

### 📊 **Web-Scale Processing Applications**
- [ ] **Common Crawl processing** (linguistic analysis of web-scale corpora)
- [ ] **Quality filtering** (grammaticality scoring for training data)
- [ ] **Corpus pattern mining** (discover linguistic universals at scale)
- [ ] **Academic publication pipeline** (ACL/NeurIPS paper preparation)

### ✅ **Quality Gates (M8)**
- PyO3 bindings achieve <0.02% training overhead
- Linguistic tokenization stays <500μs per sentence (faster than BPE!)
- Compositional understanding improves generalization by 10-50%
- Web-scale processing demonstrates 500x speedup over existing linguistic systems
- Successful integration with major ML frameworks (PyTorch, JAX)
- Published results on systematic generalization benchmarks

---

## Continuous Development Practices

### 📊 **Daily Performance Monitoring**
- **Benchmark runs** in CI with trend analysis
- **Memory usage tracking** with alerts for regressions
- **Performance dashboard** with historical data
- **Automated performance reports** for significant changes

### 🧪 **Quality Assurance**
- **Property-based testing** for linguistic invariants
- **Fuzz testing** for parser robustness
- **Golden test maintenance** with Python V1 cross-validation
- **Coverage tracking** with minimum thresholds

### 🛠️ **Developer Tooling Evolution**
- **IDE integration** improvements (better debugging, profiling)
- **Documentation updates** (architecture decisions, performance insights)
- **Developer onboarding** streamlining
- **Contribution guidelines** refinement

### 🔄 **Technical Debt Management**
- **Weekly technical debt review** (code quality, architecture decisions)
- **Performance debt tracking** (known optimizations, measurement gaps)
- **Dependency maintenance** (security updates, version management)
- **Code review standards** evolution

---

## Performance Targets Summary

| Metric | Python V1 Baseline | canopy.rs M2 Achieved | Tokenizer Target | GPU Batch Target |
|--------|--------------------|--------------------|------------------|------------------|
| **Parse Latency** | ~100ms | **7-76μs** ✅ | **<500μs** (full semantic) | **0.15-0.5μs** (large batches) |
| **LSP Response** | 200ms | **<50μs** ✅ | **<100μs** (full analysis) | **<10μs** (batched) |
| **Throughput** | 10 sent/sec | **12,500-40,000 sent/sec** ✅ | **2,000+ sent/sec** | **2M+ sent/sec** (GPU) |
| **Memory/Sentence** | 250KB | **Bounded allocation** ✅ | **<10KB** | **<1KB** (optimized) |
| **Startup Time** | 2-3s | **<100ms** ✅ | **<100ms** | **<50ms** |
| **Binary Size** | N/A (Python) | **<20MB** ✅ | **<20MB** | **<10MB** (optimized) |

---

## Success Metrics

### 🎯 **Technical Excellence**
- [ ] **10x performance** improvement verified across all metrics
- [ ] **Zero regressions** in CI for 30+ consecutive days
- [ ] **Sub-50ms LSP responses** for 95% of requests
- [ ] **Production stability** (24+ hour continuous operation)
- [ ] **Comprehensive test coverage** (>95% with property tests)

### 🧠 **Linguistic Accuracy**
- [ ] **Theta role assignment** >95% accuracy on VerbNet test suite
- [ ] **Compositional semantics** handles complex discourse correctly
- [ ] **Cross-validation** with Python V1 outputs maintains accuracy
- [ ] **Theory compliance** (binding theory, movement chains, DRT)

### 👥 **Developer Experience**
- [ ] **Fast feedback loops** (<10s test runs, <30s benchmark runs)
- [ ] **Clear documentation** with examples and troubleshooting
- [ ] **Easy installation** across platforms (cargo install works)
- [ ] **Editor integration** guides for VS Code, Neovim, Emacs
- [ ] **Active community** adoption and contribution

### 🚀 **Production Readiness**
- [ ] **Security compliance** (no critical vulnerabilities)
- [ ] **Resource efficiency** (bounded memory, predictable performance)
- [ ] **Operational excellence** (logging, monitoring, configuration)
- [ ] **Migration path** from Python V1 clearly documented
- [ ] **Future extensibility** (plugin system, research framework)

---

## Risk Mitigation Strategy

### 🔧 **Technical Risks**
- **UDPipe integration complexity**: Early prototyping, fallback to alternative parsers
- **Performance target unrealistic**: Conservative estimates, incremental optimization
- **Rust learning curve**: Mentorship, extensive documentation, gradual complexity

### 🧠 **Linguistic Risks**
- **Theory-practice gap**: Start with well-understood phenomena, expand gradually
- **Accuracy regressions**: Continuous cross-validation with Python V1
- **Complexity explosion**: Careful scope management, incremental complexity

### 📊 **Process Risks**
- **Performance debt accumulation**: Daily benchmark monitoring, regression gates
- **Technical debt buildup**: Weekly review cycles, refactoring budget
- **Quality gate erosion**: Automated enforcement, no manual overrides

---

## Post-V2 Roadmap (Future Milestones)

### 🔬 **Research Platform (M7+)**
- **Theory testing framework** (swappable linguistic theories)
- **Optimality Theory** full implementation
- **Cross-linguistic support** expansion
- **Academic collaboration** integration

### 🌐 **Ecosystem Integration (M8+)**
- **WebAssembly compilation** for browser usage
- **Language server extensions** (custom protocols)
- **IDE plugin development** (dedicated VS Code extension)
- **Research tool integration** (corpus analysis, annotation tools)

### ⚡ **Advanced Optimization (M9+)**
- **Neural enhancement** (hybrid symbolic-neural approaches)
- **Distributed processing** for large-scale analysis
- **Real-time collaboration** support
- **Advanced caching** (persistent, distributed)

---

## Getting Started

```bash
# 1. Clone and setup
git clone <repo-url> canopy
cd canopy

# 2. Install development tools
cargo install just cargo-criterion cargo-tarpaulin

# 3. Run development setup
just setup

# 4. Verify everything works
just test
just benchmark
just lint

# 5. Start development
just dev  # Watch mode with fast feedback
```

**Next Steps**: Begin with M1 foundation work, focusing on benchmarking infrastructure and developer experience before any feature development.