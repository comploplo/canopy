# Historical Milestones (M1-M4.5)

This file contains archived milestone details for M1 through M4.5.
These milestones established the project foundation and used stub/test implementations.
Their performance metrics should be ignored. See the main ROADMAP.md for current milestones (M5+).

______________________________________________________________________

## M1: Foundation & Developer Tooling - COMPLETE

**Goal**: Establish world-class development environment with performance infrastructure

### Completed Achievements

- Cargo workspace setup with multi-crate architecture
- Testing infrastructure with comprehensive test suites
- Development scripts and automation
- Performance monitoring with coverage tracking
- Quality gates ensuring code reliability

______________________________________________________________________

## M2: Core Types & UDPipe Integration (ARCHIVED)

**Goal**: Foundational parsing infrastructure

### Completed Achievements

- Core linguistic types (Word, Sentence, Document)
- UDPipe integration with FFI bindings
- Morphological features extraction
- Unified semantic features system
- Memory efficiency infrastructure with object pooling
- Test infrastructure and comprehensive test suites

**Note**: Performance metrics from this phase used stub/test data and should be ignored.

______________________________________________________________________

## M3: Event Structure & Movement Detection (ARCHIVED)

**Goal**: Event semantics framework development

### Completed Achievements

- Neo-Davidsonian events with EventBuilder pattern
- Theta role assignment framework (19 roles)
- Movement chain representation integrated
- LittleV decomposition (Cause, Become, Do, Be, Go, Have)
- VerbNet integration framework established

**Note**: Performance metrics were based on test scaffolding, not real semantic analysis.

______________________________________________________________________

## M3.5: Semantic-First Layer 1 Implementation (ARCHIVED)

**Goal**: Early semantic processing exploration

### Completed Achievements

- Semantic-first processing bypassing UDPipe where possible
- VerbNet standalone crate with comprehensive tests
- Direct database access for linguistic resources
- Clean architecture separating semantic from syntactic
- VerbNet coverage with 99.7% XML file parsing success

**Note**: Performance metrics from stub implementations should be ignored.

______________________________________________________________________

## M4: Multi-Resource Semantic Integration (ARCHIVED)

**Goal**: Multi-engine infrastructure development

### Completed Achievements

- VerbNet engine with verb class analysis and theta roles
- FrameNet engine with frame analysis and frame elements
- WordNet engine with lexical semantic analysis
- Multi-resource fallback strategy (VerbNet -> FrameNet -> WordNet)
- Parallel querying capability across all engines
- Base engine infrastructure with unified traits

**Note**: Performance numbers were from stub/test implementations and should be ignored.

______________________________________________________________________

## M4.5: Architecture Consolidation (ARCHIVED)

**Goal**: Codebase consolidation and cleanup

### Completed Achievements

- Unified semantic-layer consolidating all engines
- Base engine infrastructure (canopy-engine) with common traits
- Deprecated legacy packages (canopy-parser, canopy-semantics removed)
- Updated dependencies across all crates to new architecture
- Fixed compilation errors and test migrations
- Working coverage scripts (scripts/check-coverage.sh functional)
- Clean codebase with removed RealServerFactory references

**Note**: Performance metrics from this phase used stub implementations and should be ignored.
