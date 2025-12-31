# Data Setup Guide

Canopy requires external linguistic resources for semantic analysis. This guide covers downloading and configuring them.

## Quick Start

Run the automated setup script:

```bash
./scripts/setup-data.sh
```

This downloads all freely available resources (~40MB total).

## Resources

| Resource       | Size  | Source                                                            | License               |
| -------------- | ----- | ----------------------------------------------------------------- | --------------------- |
| VerbNet 3.4    | ~1MB  | [GitHub](https://github.com/cu-clear/verbnet)                     | Academic              |
| WordNet 3.1    | ~30MB | [Princeton](https://wordnet.princeton.edu/)                       | Princeton             |
| UD English-EWT | ~5MB  | [GitHub](https://github.com/UniversalDependencies/UD_English-EWT) | CC-BY-SA 4.0          |
| PropBank       | ~2MB  | [GitHub](https://github.com/propbank/propbank-frames)             | Academic              |
| FrameNet 1.7   | ~50MB | [Berkeley](https://framenet.icsi.berkeley.edu/)                   | Requires registration |

## Directory Structure

After setup, your `data/` directory should look like:

```
data/
├── verbnet/
│   └── vn-gl/           # VerbNet XML files (333 verb classes)
├── wordnet/
│   └── dict/            # WordNet database files
├── framenet/            # FrameNet XML (requires manual download)
├── propbank/            # PropBank frames
├── ud_english-ewt/
│   └── UD_English-EWT/  # CoNLL-U treebank files
├── canopy-lexicon/      # Built-in lexicon data
└── test-corpus/         # Test data (Moby Dick)
```

## Manual Installation

### VerbNet 3.4

```bash
cd data
git clone https://github.com/cu-clear/verbnet.git
```

### WordNet 3.1

```bash
cd data
mkdir -p wordnet && cd wordnet
curl -L -o wn3.1.dict.tar.gz https://wordnetcode.princeton.edu/wn3.1.dict.tar.gz
tar -xzf wn3.1.dict.tar.gz
rm wn3.1.dict.tar.gz
```

### UD English-EWT

```bash
cd data
mkdir -p ud_english-ewt && cd ud_english-ewt
git clone https://github.com/UniversalDependencies/UD_English-EWT.git
```

### PropBank

```bash
cd data
git clone https://github.com/propbank/propbank-frames.git propbank
```

### FrameNet 1.7 (Requires Registration)

1. Visit [framenet.icsi.berkeley.edu](https://framenet.icsi.berkeley.edu/)
1. Click "Get the Data" or "Request FrameNet Data"
1. Complete the registration form
1. Download FrameNet 1.7 when approved
1. Extract to `data/framenet/`

## Verification

After installation, verify everything works:

```bash
cargo run --example demo --release
```

Expected output:

```
CANOPY - Semantic Analysis Pipeline
====================================

Loading semantic engines... done (110ms)
...
```

## Troubleshooting

### "Data not found" errors

Ensure the directory structure matches exactly what the code expects:

- VerbNet: `data/verbnet/vn-gl/` must contain XML files
- WordNet: `data/wordnet/dict/` must contain `data.noun`, `data.verb`, etc.
- UD-EWT: `data/ud_english-ewt/UD_English-EWT/` must contain `.conllu` files

### Slow first analysis

The first analysis takes ~900ms to load engines. Subsequent analyses are fast (~19ms/sentence).

### Memory usage

Loaded engines use ~50-100MB of RAM. This is expected for full semantic analysis.
