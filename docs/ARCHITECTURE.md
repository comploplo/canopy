# Architecture

## Overview

Canopy is a semantic linguistic analysis library in Rust. It uses a provider-based architecture separating the kernel (pure semantic operations) from heavy resources (VerbNet, FrameNet, WordNet, etc.).

## Crate Structure

```
canopy/
├── crates/
│   ├── canopy/                 # KERNEL: Core types + semantic operations
│   │   ├── core/               # ThetaRole, CanopyError, DepRel, UPos
│   │   ├── kernel/             # EventComposer, DiscourseContext, DRS
│   │   └── runtime/            # Provider traits, AnnotatedSyntax, IR
│   │
│   ├── canopy-resources/       # RESOURCES: Engines + pipeline
│   │   ├── engine/             # Caching, XML parsing, SharedEngines
│   │   ├── verbnet/            # VerbNet (333 classes)
│   │   ├── framenet/           # FrameNet (1200+ frames)
│   │   ├── wordnet/            # WordNet (117k synsets)
│   │   ├── propbank/           # PropBank
│   │   ├── lexicon/            # Closed-class words
│   │   ├── syntax/             # TreebankSyntaxProvider, ResourceBackedTagger
│   │   ├── tokenizer/          # SimpleTokenizer, UnicodeTokenizer
│   │   ├── providers/          # DefaultProvider implementations
│   │   └── pipeline/           # CanopyPipeline orchestrator
│   │
│   └── canopy-cli/             # CLI + demos
│
└── data/                       # Linguistic resources (gitignored)
```

## Dependency Graph

```
         canopy-cli
              │
              ▼
      canopy-resources
              │
              ▼
           canopy
```

**Key property**: The `canopy` kernel has NO dependencies on `canopy-resources`. This enables testing the kernel in isolation.

## Core Types

### ThetaRole

```rust
pub enum ThetaRole {
    Agent,       // Initiator: "John broke the vase"
    Patient,     // Affected: "John broke the vase"
    Theme,       // Moved/transferred: "John gave Mary a book"
    Experiencer, // Mental state: "John fears spiders"
    Recipient,   // Receiving: "John gave Mary a book"
    // ... more roles
}
```

### LittleVType

```rust
pub enum LittleVType {
    Cause,      // Causative: "break", "kill"
    Become,     // Change of state: "open", "melt"
    Be,         // State: "know", "love"
    Do,         // Activity: "run", "swim"
    Experience, // Psych: "fear", "admire"
    Go,         // Motion: "go", "run"
    Have,       // Possession: "have", "own"
    Say,        // Communication: "say", "tell"
    Exist,      // Existence: "exist", "be"
}
```

### Provider Traits

```rust
// Predicate decomposition
pub trait SenseProvider: Send + Sync {
    fn decompose_predicate(
        &self,
        syntax: &AnnotatedSyntax,
        pred_id: TokenId,
    ) -> Result<Vec<PredicateDecomposition>, CanopyError>;
}

// Thematic role binding
pub trait RoleProvider: Send + Sync {
    fn bind_roles(
        &self,
        syntax: &AnnotatedSyntax,
        pred_id: TokenId,
        sense: Option<&SenseId>,
    ) -> Result<Vec<RoleBinding>, CanopyError>;
}

// Syntax parsing
pub trait SyntaxProvider: Send + Sync {
    fn parse(&self, text: &str) -> Result<AnnotatedSyntax, CanopyError>;
}
```

## Usage

### Full Pipeline

```rust
use canopy_resources::CanopyPipeline;

let pipeline = CanopyPipeline::new()?;
let analysis = pipeline.analyze("John gave Mary a book.")?;

// Access results
println!("Tokens: {}", analysis.syntax.tokens.len());
println!("Decompositions: {}", analysis.decompositions.len());
println!("Role bindings: {}", analysis.role_bindings.len());
```

### Syntax Only

```rust
use canopy_resources::TreebankSyntaxProvider;
use canopy::runtime::SyntaxProvider;

let provider = TreebankSyntaxProvider::new()?;
let syntax = provider.parse("The cat runs.")?;

for token in &syntax.tokens {
    println!("{}: {:?} -> {:?}", token.form, token.upos, token.deprel);
}
```

### Individual Engines

```rust
use canopy_resources::{VerbNetEngine, WordNetEngine, PartOfSpeech};

let verbnet = VerbNetEngine::new()?;
let result = verbnet.analyze_verb("give")?;

let wordnet = WordNetEngine::new()?;
let result = wordnet.analyze_word("book", PartOfSpeech::Noun)?;
```

## Design Principles

### 1. Clean Kernel Boundary

The `canopy` kernel contains:

- Core types (ThetaRole, LittleV, CanopyError)
- Provider trait definitions
- Event composition logic
- Discourse processing logic

It has NO knowledge of VerbNet XML parsing, FrameNet loading, etc.

### 2. Provider-Based Injection

Heavy resources are injected via provider traits:

- Tests can use mock providers
- Different resources can be swapped
- Kernel remains testable in isolation

### 3. Shared Engines

`SharedEngines` enables efficient resource sharing:

```rust
let engines = SharedEngines::new()?;  // Load once

// Share across components
let syntax = TreebankSyntaxProvider::with_shared_engines(config, &engines)?;
let sense = VerbNetSenseProvider::with_engine(engines.verbnet.clone());
```

## Performance

| Operation         | Time                |
| ----------------- | ------------------- |
| Pipeline init     | ~730ms (one-time)   |
| Sentence analysis | 30-200μs            |
| Engine loading    | ~300ms (from cache) |
| Memory            | ~350 MB             |
