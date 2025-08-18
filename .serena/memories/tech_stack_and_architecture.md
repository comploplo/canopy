# Tech Stack and Architecture

## V1 Python System (Reference Implementation)
- **Language**: Python 3.13+ 
- **Core Libraries**: spaCy, pygls (Language Server Protocol), protobuf
- **Architecture**: Word-centric with protobuf optimization (M8-M11 complete)
- **Pipeline**: spaCy → Skeleton → Semantic Roles → λ-calculus → Truth Evaluation → LSP
- **Database**: SQLite for corpus-trained verb patterns
- **Testing**: pytest with 85% coverage, pytest-benchmark for performance

## V2 Rust System (To Be Implemented)
- **Language**: Rust (systems programming, type safety)
- **Core Libraries**: 
  - `tower-lsp` for LSP server implementation
  - `tokio` for async runtime
  - `serde` for serialization
  - `ufal-udpipe` for Universal Dependencies parsing (replaces spaCy)
- **Architecture**: 4-layer modular design with compile-time type safety

## Rust Project Structure (Planned)
```
canopy/
├── src/
│   ├── layer1/           # Morphosyntactic
│   │   ├── parser.rs     # UDPipe integration
│   │   ├── features.rs   # Semantic feature extraction
│   │   └── mod.rs
│   ├── layer2/           # Event structure  
│   │   ├── events.rs     # Neo-Davidsonian events
│   │   ├── theta.rs      # Theta role assignment
│   │   ├── little_v.rs   # Event decomposition
│   │   └── mod.rs
│   ├── layer3/           # Compositional semantics
│   │   ├── drt.rs        # Discourse Representation Theory
│   │   ├── lambda.rs     # Lambda calculus
│   │   ├── composition.rs # Semantic composition
│   │   └── mod.rs
│   ├── layer4/           # Discourse & LSP
│   │   ├── context.rs    # Discourse context
│   │   ├── lsp_server.rs # LSP integration
│   │   └── mod.rs
│   └── lib.rs
├── tests/
├── benches/
└── Cargo.toml
```

## Key Architectural Differences
- **Parser**: UDPipe (transparent UD format) vs spaCy (black box)
- **Type System**: Compile-time Rust types vs Python runtime types
- **Performance**: Zero-copy parsing, 10x speed target
- **Theory**: Multi-dominance, Optimality Theory, full DRT vs simplified λ-calculus

## External Dependencies (V2)
- **UDPipe**: Models and binaries for dependency parsing
- **VerbNet**: XML database (port from Python system)
- **WordNet**: Optional for semantic features
- **Corpora**: Penn Treebank, UD treebanks for validation