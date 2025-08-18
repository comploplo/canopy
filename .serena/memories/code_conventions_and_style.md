# Code Conventions and Style Guide

## Python V1 System Conventions (Reference)

### Code Style
- **Formatter**: ruff with line length 80
- **Linting**: ruff with Google-style checks (E, F, W, C, N, UP, B, A, COM, DTZ, EM, G, ICN, PIE, T20, RET, SIM, ARG, ERA, PGH, FLY)
- **Type Hints**: Required throughout all public APIs
- **Docstrings**: Document all public functions and classes
- **Import Style**: Organized and auto-sorted

### Architecture Patterns
- **Word-Centric Design**: All components work with Word objects (not JSON dicts)
- **Enum Usage**: Extensive use of enums for type safety (PartOfSpeech, ThetaRole, DependencyRelation)
- **Dataclass Pattern**: Core data structures use @dataclass
- **Strategy Pattern**: Multiple implementations with shared interfaces (database backends)
- **Pipeline Pattern**: Sequential processing through analysis stages

### Naming Conventions
- **Classes**: PascalCase (Word, ThetaRole, SemanticFeatures)
- **Functions**: snake_case (beta_reduce, build_sentence_term)
- **Constants**: UPPER_SNAKE_CASE (ROLE_SPECS, NEG_TOKENS)
- **Variables**: snake_case
- **Files**: snake_case.py
- **Modules**: lowercase with underscores

### Testing Standards
- **Coverage**: 85% minimum (targeting restoration to 90%)
- **Golden Tests**: Protocol buffer-based deterministic validation
- **Performance Tests**: pytest-benchmark for regression detection
- **Test Organization**: Mirror source structure in tests/

## Rust V2 System Conventions (To Be Established)

### Code Style (Standard Rust)
- **Formatter**: rustfmt (default settings)
- **Linting**: clippy with standard checks
- **Documentation**: /// doc comments for all public items
- **Error Handling**: Result<T, E> pattern, avoid panics in library code

### Architecture Patterns (Planned)
- **Type-Driven Design**: Leverage Rust's type system for compile-time guarantees
- **Trait-Based Interfaces**: Define traits for swappable components
- **Zero-Copy Where Possible**: Avoid unnecessary allocations
- **Error Types**: Custom error enums for different failure modes

### Naming Conventions (Rust Standard)
- **Types**: PascalCase (Word, ThetaRole, EventId)
- **Functions**: snake_case (parse_sentence, extract_features)
- **Constants**: SCREAMING_SNAKE_CASE (DEFAULT_TIMEOUT)
- **Variables**: snake_case
- **Files**: snake_case.rs
- **Modules**: lowercase

### Performance Principles
- **Explicit Over Implicit**: No hidden features or black-box processing
- **Compositionality**: Each layer's output is next layer's well-typed input  
- **Incremental Complexity**: Start simple, add complexity only where needed
- **Performance Through Design**: Efficiency from architecture, not micro-optimization

## Shared Principles (V1 and V2)
- **Theory-Driven Design**: Every architectural decision grounded in linguistic theory
- **Type Safety First**: Use type systems to enforce linguistic constraints
- **Comprehensive Testing**: High coverage with multiple testing strategies
- **Performance Consciousness**: Track and prevent regressions
- **Documentation Standards**: Keep docs current with implementation