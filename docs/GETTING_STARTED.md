# Getting Started

## Prerequisites

- **Rust 1.75+** ([rustup.rs](https://rustup.rs))
- **~2GB disk space** for linguistic data

## Quick Start

```bash
# Build
cargo build --release

# Download linguistic data
./scripts/setup-data.sh

# Run demo
cargo run -p canopy-resources --example demo --release
```

## Data Setup

Canopy requires linguistic datasets. See [DATA_SETUP.md](DATA_SETUP.md) or run:

```bash
./scripts/setup-data.sh
```

Expected structure:

```
data/
├── verbnet/          # VerbNet 3.4 (333 verb classes)
├── framenet/         # FrameNet 1.7 (1200+ frames)
├── wordnet/          # WordNet 3.1 (117k synsets)
├── propbank/         # PropBank semantic roles
├── ud_english-ewt/   # Universal Dependencies treebank
└── lexicon/          # Closed-class word lists
```

## Basic Usage

### Analyze a Sentence

```rust
use canopy_resources::CanopyPipeline;

fn main() -> Result<(), canopy::CanopyError> {
    let pipeline = CanopyPipeline::new()?;
    let analysis = pipeline.analyze("John gave Mary a book.")?;

    // Print tokens with POS tags
    for token in &analysis.syntax.tokens {
        println!("{}: {:?}", token.form, token.upos);
    }
    // Output:
    // John: Propn
    // gave: Verb
    // Mary: Propn
    // a: Det
    // book: Noun
    // .: Punct

    Ok(())
}
```

### Query Individual Engines

```rust
use canopy_resources::{VerbNetEngine, WordNetEngine, PartOfSpeech};

// VerbNet: verb class lookup
let verbnet = VerbNetEngine::new()?;
let result = verbnet.analyze_verb("give")?;
for class in &result.data.verb_classes {
    println!("Class: {}", class.id);  // give-13.1
}

// WordNet: definitions
let wordnet = WordNetEngine::new()?;
let result = wordnet.analyze_word("whale", PartOfSpeech::Noun)?;
for synset in &result.data.synsets {
    println!("{}", synset.definition);
}
```

### Parse Syntax Only

```rust
use canopy_resources::TreebankSyntaxProvider;
use canopy::runtime::SyntaxProvider;

let provider = TreebankSyntaxProvider::new()?;
let syntax = provider.parse("The cat runs.")?;

// Find the root verb
if let Some(root) = syntax.root() {
    println!("Root: {} ({:?})", root.lemma, root.upos);
}

// Find subject
for token in syntax.tokens.iter() {
    if token.deprel == canopy::DepRel::Nsubj {
        println!("Subject: {}", token.form);
    }
}
```

## Project Structure

```
canopy/
├── crates/
│   ├── canopy/           # Core types, event composition, discourse
│   ├── canopy-resources/ # 5 semantic engines + pipeline
│   └── canopy-cli/       # Command-line interface
├── data/                 # Linguistic resources
└── scripts/              # Development tools
```

## Running Tests

```bash
cargo test --workspace
```

## Performance

| Operation         | Time              |
| ----------------- | ----------------- |
| Pipeline init     | ~730ms (one-time) |
| Sentence analysis | 30-200μs          |
| Memory usage      | ~100MB            |

## Next Steps

- [ARCHITECTURE.md](ARCHITECTURE.md) — System design
- [PERFORMANCE.md](PERFORMANCE.md) — Optimization guide
- `cargo doc --open` — API documentation
