# Roadmap

## Current Status: Production Ready

All core milestones complete. The system provides end-to-end semantic analysis.

## Completed Milestones

| Milestone | Focus                        | Status   |
| --------- | ---------------------------- | -------- |
| M1-M4     | Foundation, parsing, engines | Archived |
| M5        | Lemmatization + caching      | Complete |
| M6        | Engine infrastructure        | Complete |
| M7        | Event composition            | Complete |
| M8        | DRT + discourse              | Complete |
| M9        | Documentation                | Complete |

## What's Working

- **5 semantic engines**: VerbNet (333 classes), FrameNet (1200+ frames), WordNet (117k synsets), PropBank, Lexicon
- **Full pipeline**: Text → Tokenization → POS tagging → Syntax → Event composition → Discourse
- **Discourse analysis**: Coherence relations, QUD tracking, presupposition detection, anaphora resolution
- **Underspecification**: Packed semantics, 5 disambiguation strategies, MRS-style scope
- **Documentation**: Formal semantics appendix, API guides, theoretical foundations
- **Performance**: ~730ms init, 30-200μs per sentence, 80%+ test coverage

## Architecture

Simplified from 11 crates to 3:

```
canopy/           # Core types + kernel
canopy-resources/ # Engines + pipeline
canopy-cli/       # CLI + demos
```

## Future Work (Deferred)

| Feature            | Priority | Notes                            |
| ------------------ | -------- | -------------------------------- |
| GPU acceleration   | Low      | Symbolic approach sufficient     |
| UDPipe integration | Low      | CoNLL-U input works for research |
| Cross-linguistic   | Medium   | UD support available             |

## Quality Gates

- **Coverage**: 80% minimum (currently 81%+)
- **Performance**: No regressions allowed
- **Linting**: Clippy pedantic, zero warnings

## Development

```bash
cargo build --release
cargo test --workspace
./scripts/check-coverage.sh
```
