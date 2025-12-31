# Getting Started with Canopy

Canopy is a semantic-first linguistic analysis platform for Rust. This guide will help you get up and running quickly.

## Prerequisites

- **Rust**: 1.75+ (install from [rustup.rs](https://rustup.rs))
- **Git**: For cloning the repository
- **~500MB disk space**: For linguistic data files

## Installation

### 1. Get the Source

Clone or download the repository to your local machine.

### 2. Build the Project

```bash
cargo build --workspace --release
```

### 3. Verify Data Files

Canopy requires linguistic data files to function. Check that these directories exist:

```
data/
├── verbnet/          # VerbNet 3.4 XML files (333 verb classes)
├── framenet/         # FrameNet v15 frames
├── wordnet/          # WordNet 3.1 database
├── propbank/         # PropBank frames
├── ud_english-ewt/   # Universal Dependencies treebank
└── canopy-lexicon/   # Name-gender dataset (147k entries)
```

If data is missing, see [DATA_SETUP.md](DATA_SETUP.md) for download instructions.

## Quick Demo

Run the demo to see Canopy in action:

```bash
cargo run --example demo --release
```

Expected output:

```
CANOPY - Semantic Analysis Pipeline
====================================

Loading semantic engines... done (110ms)

(1) "The captain saw the whale."

    LAYER 1 (Lexical)
    The/Det  captain/N  saw/V  the/Det  whale/V
    captain --NominalSubject--> saw
    VerbNet: see -> discover-84

    LAYER 2 (Events)
    DO(captain, see) : see
      Agent = "captain"

    LAYER 3 (Discourse)
    Events: e1, e3
    Entities in context: 1
...
```

## Basic Usage

### Analyzing a Sentence

```rust
use canopy_pipeline::{create_l1_analyzer_with_treebank, DiscourseProcessor};
use canopy_events::EventComposer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Layer 1: Load semantic engines
    let l1 = create_l1_analyzer_with_treebank()?;

    // Layer 2: Create event composer
    let l2 = EventComposer::new()?;

    // Layer 3: Create discourse processor
    let mut l3 = DiscourseProcessor::new();

    // Analyze a sentence
    let sentence = "The captain saw the whale.";

    // Layer 1: Lexical analysis
    let l1_result = l1.analyze_sentence(sentence)?;
    println!("Tokens: {:?}", l1_result.tokens.len());

    // Layer 2: Event composition
    let analysis = convert_to_sentence_analysis(sentence, &l1_result);
    let events = l2.compose_sentence(&analysis)?;
    println!("Events: {:?}", events.events.len());

    // Layer 3: Discourse processing
    l3.process_sentence(sentence, &events)?;
    println!("Entities: {:?}", l3.drs().universe.len());

    Ok(())
}
```

### Accessing Layer 1 Data

```rust
for token in &l1_result.tokens {
    // Basic info
    println!("{} ({})", token.original_word, token.lemma);

    // Part of speech
    if let Some(pos) = token.pos {
        println!("  POS: {:?}", pos);
    }

    // VerbNet classes
    if let Some(vn) = &token.verbnet {
        for class in &vn.verb_classes {
            println!("  VerbNet: {}", class.id);
        }
    }

    // FrameNet frames
    if let Some(fn_) = &token.framenet {
        for frame in &fn_.frames {
            println!("  FrameNet: {}", frame.name);
        }
    }
}
```

### Working with Events

```rust
for composed in &events.events {
    let event = &composed.event;

    // Event type (LittleV primitive)
    println!("Event: {} : {}", event.little_v, event.predicate);

    // Participants with theta roles
    for (role, entity) in &event.participants {
        println!("  {} = \"{}\"", role, entity.text);
    }

    // Polarity
    if !composed.polarity {
        println!("  [NEGATED]");
    }

    // Modality
    if let Some(modality) = &event.modality {
        println!("  Modality: {:?}", modality);
    }
}
```

### Building Discourse Context

```rust
// Process multiple sentences
let sentences = vec![
    "The captain saw the whale.",
    "He decided to chase it.",
];

for sentence in sentences {
    let l1_result = l1.analyze_sentence(sentence)?;
    let analysis = convert_to_sentence_analysis(sentence, &l1_result);
    let events = l2.compose_sentence(&analysis)?;
    l3.process_sentence(sentence, &events)?;
}

// Access accumulated discourse state
let drs = l3.drs();
for (id, referent) in &drs.universe {
    if !referent.is_event {
        println!("Entity x{}: {:?}", id.0, referent.name);
    }
}
```

## Project Structure

```
canopy/
├── crates/
│   ├── canopy-core/              # Core types (ThetaRole, LittleV, etc.)
│   ├── canopy-semantic-engines/  # VerbNet, FrameNet, WordNet, etc.
│   ├── canopy-tokenizer/         # Layer 1 coordinator
│   ├── canopy-events/            # Layer 2 event composition
│   ├── canopy-discourse/         # Layer 3 discourse
│   └── canopy-pipeline/          # High-level API
├── examples/
│   └── demo.rs                   # Main demonstration
└── data/                         # Linguistic resources
```

## Running Tests

```bash
# Run all tests
cargo test --workspace

# Run with nextest (faster)
cargo nextest run --workspace
```

## Performance

| Operation           | Time               |
| ------------------- | ------------------ |
| Engine loading      | ~900ms (one-time)  |
| Layer 1 analysis    | 15-22ms/sentence   |
| Layer 2 composition | 78-148μs/sentence  |
| Layer 3 discourse   | \<1ms/sentence     |
| **End-to-end**      | **~19ms/sentence** |

## Next Steps

- **[ARCHITECTURE.md](ARCHITECTURE.md)**: Deep dive into the three-layer design
- **[PERFORMANCE.md](PERFORMANCE.md)**: Optimization tips and benchmarking
- **API Documentation**: Run `cargo doc --open` to browse rustdoc

## Troubleshooting

### "Data not found" errors

Ensure all data directories exist under `data/`. See [DATA_SETUP.md](DATA_SETUP.md).

### Slow first analysis

The first analysis takes ~900ms to load engines. Subsequent analyses are fast.

### Memory usage

Canopy uses ~50-100MB for loaded engines. Use batch processing for large corpora.
