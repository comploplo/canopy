# Codebase Structure

## Python V1 System Structure
Located at `/Users/gabe/projects/canopy/spacy-lsp/`

### Core Source Code (`src/spacy_lsp/`)
```
src/spacy_lsp/
├── __init__.py
├── word.py                     # Core Word class with enums (PartOfSpeech, ThetaRole, etc.)
├── server.py                   # LSP server implementation
├── proto_pipeline.py           # Main analysis pipeline
├── adapters.py                 # JSON/Word conversion utilities
├── cache.py                    # Caching infrastructure
│
├── parsing/                    # Document structure
│   ├── document_structure.py
│   └── enhanced_document_structure.py
│
├── semantics/                  # Core semantic analysis
│   ├── roles.py               # ThetaRole enum and role specifications
│   ├── frames.py              # Verb frame system
│   ├── lambda_calculus.py     # λ-calculus implementation with types
│   ├── composition.py         # Semantic composition rules
│   ├── context.py             # Truth evaluation and discourse context
│   ├── dp.py                  # Determiner phrase composition
│   ├── verbnet.py            # VerbNet integration
│   ├── word_semantics.py     # Word-level semantic analysis
│   ├── corpus_logging.py     # Performance tracking
│   ├── proto_verb_database.py # Corpus-trained verb patterns
│   └── data/corpus_verbs.db   # SQLite database
│
├── diagnostics/               # LSP diagnostics
│   ├── agreement.py          # Subject-verb agreement
│   ├── enhanced.py           # Advanced semantic diagnostics
│   └── polarity.py           # Negation detection
│
├── actions/                   # LSP code actions
│   └── intelligent_actions.py # Voice changes, pronoun resolution
│
├── hover/                     # LSP hover information
│   └── rich_content.py       # Multi-modal hover display
│
├── navigation/                # LSP navigation
│   └── semantic_navigation.py # Entity-based navigation
│
├── resources/                 # Linguistic resources
│   ├── features.json         # Feature specifications
│   ├── *.schema.json         # JSON schemas
│   └── interfaces.py         # Resource interfaces
│
└── tools/                     # Development utilities
    ├── corpus_extractor.py   # Corpus pattern extraction
    └── model_checksum.py     # spaCy model validation
```

### Test Structure (`tests/`)
- **Unit tests**: Mirror source structure
- **Golden tests**: Deterministic output validation in `tests/testdata/`
- **Benchmarks**: Performance tests in `tests/benchmarks/`
- **Integration**: End-to-end LSP testing

### Key Data Structures (Python V1)
- **Word**: Core class with text, POS, features, semantic roles, dependencies
- **ThetaRole**: 19 semantic roles (Agent, Patient, Theme, Recipient, etc.)
- **SemanticRole**: Role assignment with confidence scores
- **Document/Sentence**: Hierarchical text structure
- **PredicateFrame**: Verb argument structure
- **Term**: Lambda calculus AST with types

## Rust V2 System Structure (Planned)
Located at `/Users/gabe/projects/canopy/` (root level)

### Planned Architecture
```
canopy/
├── Cargo.toml
├── src/
│   ├── lib.rs                 # Main library entry
│   │
│   ├── layer1/                # Morphosyntactic Analysis
│   │   ├── mod.rs
│   │   ├── parser.rs          # UDPipe integration
│   │   ├── features.rs        # Semantic feature extraction
│   │   └── types.rs           # Word, EnhancedWord, MorphFeatures
│   │
│   ├── layer2/                # Event Structure
│   │   ├── mod.rs
│   │   ├── events.rs          # Neo-Davidsonian events
│   │   ├── theta.rs           # Theta role assignment
│   │   ├── little_v.rs        # Event decomposition
│   │   └── movement.rs        # Movement chains
│   │
│   ├── layer3/                # Compositional Semantics
│   │   ├── mod.rs
│   │   ├── drt.rs             # Discourse Representation Theory
│   │   ├── lambda.rs          # Lambda calculus with types
│   │   ├── composition.rs     # Semantic composition rules
│   │   └── scope.rs           # Quantifier scope resolution
│   │
│   ├── layer4/                # Discourse & LSP
│   │   ├── mod.rs
│   │   ├── context.rs         # Discourse context management
│   │   ├── lsp_server.rs      # Tower-LSP implementation
│   │   ├── diagnostics.rs     # LSP diagnostics
│   │   └── actions.rs         # Code actions
│   │
│   ├── resources/             # Linguistic resources
│   │   ├── mod.rs
│   │   ├── verbnet.rs         # VerbNet interface
│   │   └── corpus.rs          # Corpus patterns
│   │
│   └── utils/                 # Utilities
│       ├── mod.rs
│       ├── types.rs           # Common type definitions
│       └── errors.rs          # Error handling
│
├── tests/                     # Unit and integration tests
├── benches/                   # Performance benchmarks
├── examples/                  # Usage examples
└── resources/                 # External linguistic data
    ├── verbnet/              # VerbNet XML data
    └── models/               # UDPipe models
```

### Key Type Hierarchies (Rust V2)
- **Layer 1**: Word → EnhancedWord → SemanticFeatures
- **Layer 2**: Event → Participants → MovementChain → OTTableau  
- **Layer 3**: DRS → DRSCondition → Term → SemanticType
- **Layer 4**: DiscourseContext → SemanticAnalysis → Diagnostic

## Migration Notes
- **V1 to V2**: Port core algorithms, VerbNet patterns, test cases
- **Architecture**: 4 clean layers vs mixed Python modules
- **Performance**: Target 10x improvement through zero-copy, compile-time optimization
- **Theory**: Add multi-dominance, OT, full DRT vs simplified λ-calculus