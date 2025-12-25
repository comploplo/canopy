# Canopy

**High-performance semantic linguistic analysis library in Rust**

Canopy is a semantic-first linguistic analysis library built for production use. It provides word-level semantic analysis through VerbNet, FrameNet, WordNet, and custom lexicon engines.

> ⚠️ **Current Limitation**: Canopy currently supports UD treebank sentences only. Arbitrary sentence analysis is planned for future releases.

## 🚀 Quick Start

```bash
git clone https://github.com/username/canopy
cd canopy
cargo build --workspace
cargo run --example basic_demo
```

## 🎯 Current Status: M7 Complete - Layer 2 Event Composition

✅ **Layer 1 Semantic Analysis**: VerbNet, FrameNet, WordNet, Lexicon engines
✅ **Layer 2 Event Composition**: Neo-Davidsonian events with theta roles
✅ **Full Pipeline**: L1→L2 integration with honest end-to-end timing
✅ **Lemmatization**: 54.4% cache hit improvement
✅ **Production Performance**: ~19ms per sentence (L1 + L2)

## 🏗️ Architecture

```text
Text → Tokenization → Lemmatization → Layer 1: Semantic Analysis → Layer 2: Event Composition
                                      [VerbNet + FrameNet + WordNet]  [Neo-Davidsonian Events]
```

**Current Implementation**: Full L1→L2 pipeline with semantic analysis and event composition.

## 📊 Performance

**Layer 1 (Semantic Analysis)**:

- **Single word**: 85.4μs with lemmatization (11,703 words/sec)
- **Cache hit rate**: 54.4% with lemmatization optimization

**Layer 2 (Event Composition)**:

- **Event composition**: 78-148μs per sentence
- **End-to-end**: ~19ms per sentence (L1 dominates)
- **Engine loading**: ~900ms one-time startup

## 🔧 Key Features

- **Two-Layer Pipeline**: L1 semantic analysis → L2 event composition
- **Real Linguistic Data**: VerbNet/FrameNet/WordNet engines
- **Neo-Davidsonian Events**: Theta roles, LittleV primitives, voice detection
- **Lemmatization System**: Intelligent morphological analysis with confidence
- **Parallel Processing**: Concurrent multi-engine analysis
- **Smart Caching**: 54.4% hit rate improvement with lemmatization
- **Production Ready**: ~67% test coverage with real-world benchmarks

## 📖 Documentation

- [**Roadmap**](docs/ROADMAP.md) - Current milestone progress and status
- [**Architecture**](docs/ARCHITECTURE.md) - Current semantic-first design
- [**Performance**](docs/reference/performance.md) - Benchmarks and optimization
- [**Contributing**](docs/CONTRIBUTING.md) - Development guidelines

## 🧪 Examples

```bash
# Basic semantic analysis demo
cargo run --example basic_demo

# Comprehensive semantic analysis
cargo run --example comprehensive_semantic_demo
```

## 🔬 Technology Stack

- **Rust 2024 Edition**: Memory safety and performance
- **Semantic Engines**: VerbNet, FrameNet, WordNet, custom lexicon
- **XML Parsing**: Real linguistic resource loading
- **Parallel Processing**: Multi-engine concurrent analysis
- **Smart Caching**: L1/L2 memory-budgeted cache system

## 📋 Requirements

- Rust 1.75+ (2024 edition)
- 4GB RAM recommended for full semantic data
- Data files: VerbNet XML, FrameNet XML, WordNet database

## 🤝 Contributing

See [CONTRIBUTING.md](docs/CONTRIBUTING.md) for development guidelines.

## 📄 License

MIT License - see [LICENSE](LICENSE) for details.

______________________________________________________________________

**Current Milestone**: M7 Complete - Layer 2 Event Composition ✅
**Next Milestone**: M8 Discourse Representation Theory (DRT)
**Performance Achieved**: ~19ms per sentence (L1 + L2 end-to-end) ✅

> Note: Canopy currently supports UD treebank sentences. Arbitrary sentence parsing is planned for future releases.
