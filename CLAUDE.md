# Canopy Project Instructions

## Current Status

**3 crates**: `canopy`, `canopy-resources`, `canopy-cli`
**Coverage gate**: 80%
**All systems operational**: Tokenizer, POS tagger, syntax provider, 5 semantic engines, event composition, discourse

______________________________________________________________________

## Critical Rules

### 1. No Stubs

- Never return empty/placeholder data
- Fail fast if data unavailable
- Be honest about capabilities

### 2. No Word Lists in Code

All word knowledge lives in data files:

| Type         | Source                               |
| ------------ | ------------------------------------ |
| Closed-class | `data/lexicon/` via `LexiconEngine`  |
| Open-class   | VerbNet, FrameNet, WordNet, PropBank |

```rust
// WRONG
fn is_auxiliary(word: &str) -> bool {
    matches!(word, "is" | "are" | "was")
}

// RIGHT
lexicon.is_auxiliary(word)?
```

### 3. Verify Before Claiming

- Run `cargo check` before claiming compilation works
- Run tests before claiming they pass
- Run the demo before claiming it works

### 4. Follow Plans

When working on a plan, don't skip steps. If a step seems complex, explain and ask before reordering.

______________________________________________________________________

## What Works

### Full Pipeline

```rust
use canopy_resources::CanopyPipeline;

let pipeline = CanopyPipeline::new()?;
let analysis = pipeline.analyze("John gave Mary a book.")?;

// Syntax with POS tags and dependencies
for token in &analysis.syntax.tokens {
    println!("{}: {:?}", token.form, token.upos);
}

// Predicate decompositions (VerbNet-backed)
for decomp in &analysis.decompositions {
    println!("{}: {:?}", decomp.sense_id, decomp.little_v_type);
}

// Theta role bindings
for binding in &analysis.role_bindings {
    println!("{:?} -> {:?}", binding.role, binding.token_id);
}
```

### Engines

| Engine           | API                          | What It Does                     |
| ---------------- | ---------------------------- | -------------------------------- |
| `VerbNetEngine`  | `analyze_verb("give")`       | Verb classes, theta roles        |
| `WordNetEngine`  | `analyze_word("whale", POS)` | Synsets, definitions             |
| `FrameNetEngine` | `analyze("give")`            | Frames, frame elements           |
| `PropBankEngine` | `analyze("give")`            | Predicate-argument structures    |
| `LexiconEngine`  | `is_pronoun("he")`           | Closed-class word classification |

### Syntax Provider

```rust
use canopy_resources::TreebankSyntaxProvider;
use canopy::runtime::SyntaxProvider;

let provider = TreebankSyntaxProvider::new()?;
let syntax = provider.parse("The cat runs.")?;

// Returns AnnotatedSyntax with:
// - POS tags (UPos::Verb, UPos::Noun, etc.)
// - Dependency relations (DepRel::Nsubj, DepRel::Root, etc.)
// - Lemmas
```

______________________________________________________________________

## Project Policies

### Demo

- One demo: `crates/canopy-resources/examples/demo.rs`
- Run: `cargo run -p canopy-resources --example demo --release`

### Coverage

- Gate: 80%
- Check: `./scripts/check-coverage.sh`

### Scripts

- Never write to `/tmp/`
- Use project's `scripts/` directory

______________________________________________________________________

## Architecture

```
canopy/
├── crates/
│   ├── canopy/           # Core types + kernel
│   │   ├── core/         # ThetaRole, DepRel, UPos, CanopyError
│   │   ├── kernel/       # EventComposer, DiscourseContext
│   │   └── runtime/      # Provider traits, AnnotatedSyntax
│   │
│   ├── canopy-resources/ # Engines + providers + pipeline
│   │   ├── engine/       # Caching, XML parsing, SharedEngines
│   │   ├── verbnet/      # VerbNet (333 classes)
│   │   ├── framenet/     # FrameNet (1200+ frames)
│   │   ├── wordnet/      # WordNet (117k synsets)
│   │   ├── propbank/     # PropBank
│   │   ├── lexicon/      # Lexicon
│   │   ├── syntax/       # TreebankSyntaxProvider, ResourceBackedTagger
│   │   ├── tokenizer/    # SimpleTokenizer, UnicodeTokenizer
│   │   ├── providers/    # DefaultProvider, VerbNetSenseProvider
│   │   └── pipeline/     # CanopyPipeline orchestrator
│   │
│   └── canopy-cli/       # CLI + demos
│
└── data/                 # Linguistic resources (gitignored)
    ├── verbnet/
    ├── framenet/
    ├── wordnet/
    ├── propbank/
    ├── lexicon/
    └── ud_english-ewt/
```

______________________________________________________________________

## Performance

| Operation       | Time            |
| --------------- | --------------- |
| Pipeline init   | ~730ms          |
| Simple sentence | ~30-70μs        |
| Engine loading  | ~300ms (cached) |
| Coverage        | 81%+            |
