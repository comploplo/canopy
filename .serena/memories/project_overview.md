# Project Overview: spaCy-LSP V2 (canopy.rs)

## Project Purpose

This project involves implementing **canopy.rs**, a complete Rust redesign of spaCy-LSP V1 (the existing Python system). The goal is to transform from Python's surface-level semantic mapping to a theoretically-grounded linguistic analysis platform in Rust, prioritizing:

- Explicit feature extraction
- Formal linguistic theory implementation
- Compile-time type safety
- 10x performance improvement over Python version
- LSP integration for real-time natural language analysis

## Core Transformation

**V1 Python**: `spaCy → JSON → Proto → LSP`
**V2 Rust**: `UDPipe → Events → DRT → LSP` with typed, theory-driven intermediate representations

## Current State

- **V1 Python system**: Fully functional at spacy-lsp/ symlink, M11 complete (Word-centric architecture, corpus-trained verb database, 85% test coverage)
- **V2 Rust system**: Not yet started - this is the implementation task

## Key Components to Implement in Rust

### Layer 1: Morphosyntactic Types

- `Word`, `EnhancedWord` structs with UDPipe integration
- `SemanticFeatures`, `FeatureExtractor` for animacy, definiteness detection
- Replace spaCy with UDPipe for transparent, theory-aligned parsing

### Layer 2: Event Structure Types

- Neo-Davidsonian `Event` representation
- `ThetaRole` enum (19 roles from Python system: Agent, Patient, Theme, etc.)
- `LittleV` for event decomposition (Cause, Become, Do, Be)
- `MovementChain` for A-movement, A-bar movement, head movement
- `OTTableau` for Optimality Theory constraint evaluation

### Layer 3: Compositional Semantics

- `DRS` (Discourse Representation Structure) for discourse semantics
- `DRSCondition` enum for predication, equality, negation, implication
- `Term` enum for lambda calculus with semantic types
- `DRTComposer` for compositional semantic construction

### Layer 4: Discourse & LSP Integration

- `DiscourseContext` for cross-sentence entity tracking
- `SemanticAnalysis` output structure
- `DiagnosticKind` for theta violations, binding violations, contradictions
- `CodeAction` for voice changes, pronoun resolution, scope disambiguation

## Migration Strategy

1. **Phase 1**: Parallel development (keep Python system running)
2. **Phase 2**: Achieve feature parity with Python V1
3. **Phase 3**: Add V2-exclusive features and deprecate Python

## Success Criteria

- Sub-50ms LSP response times (vs 200ms Python)
- 10x throughput improvement
- Accurate theta role assignment (>95% on VerbNet)
- Theory-testing framework operational
- Feature parity with Python version's LSP capabilities
