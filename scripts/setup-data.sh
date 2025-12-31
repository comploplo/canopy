#!/bin/bash
# Canopy Data Setup Script
# Downloads linguistic resources required for semantic analysis
#
# Usage: ./scripts/setup-data.sh
#
# Resources downloaded:
# - VerbNet 3.4 (~1MB) - Verb classes and semantic roles
# - WordNet 3.1 (~30MB) - Lexical database
# - UD English-EWT (~5MB) - Universal Dependencies treebank
# - PropBank (~2MB) - Predicate-argument structures
# - FrameNet 1.7 - REQUIRES MANUAL DOWNLOAD (see below)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
DATA_DIR="$PROJECT_ROOT/data"

echo "Canopy Data Setup"
echo "================="
echo ""

# Create data directory if needed
mkdir -p "$DATA_DIR"

# ============================================================================
# VerbNet 3.4
# ============================================================================
VERBNET_DIR="$DATA_DIR/verbnet"
if [ -d "$VERBNET_DIR/vn-gl" ]; then
    echo "✓ VerbNet already installed at $VERBNET_DIR"
else
    echo "Downloading VerbNet 3.4..."
    cd "$DATA_DIR"

    # Clone from official repository
    if [ -d "verbnet" ]; then
        mv verbnet verbnet.backup
    fi

    git clone --depth 1 https://github.com/cu-clear/verbnet.git verbnet

    if [ -d "$VERBNET_DIR/vn-gl" ]; then
        echo "✓ VerbNet installed successfully"
        rm -rf verbnet.backup 2>/dev/null || true
    else
        echo "✗ VerbNet installation failed"
        rm -rf verbnet 2>/dev/null || true
        mv verbnet.backup verbnet 2>/dev/null || true
        exit 1
    fi
fi

# ============================================================================
# WordNet 3.1
# ============================================================================
WORDNET_DIR="$DATA_DIR/wordnet"
if [ -d "$WORDNET_DIR" ] && [ -f "$WORDNET_DIR/dict/data.noun" ]; then
    echo "✓ WordNet already installed at $WORDNET_DIR"
else
    echo "Downloading WordNet 3.1..."
    cd "$DATA_DIR"

    # Download from Princeton
    WORDNET_URL="https://wordnetcode.princeton.edu/wn3.1.dict.tar.gz"

    if [ -d "wordnet" ]; then
        mv wordnet wordnet.backup
    fi

    mkdir -p wordnet
    cd wordnet

    if command -v curl &> /dev/null; then
        curl -L -o wn3.1.dict.tar.gz "$WORDNET_URL"
    elif command -v wget &> /dev/null; then
        wget -O wn3.1.dict.tar.gz "$WORDNET_URL"
    else
        echo "Error: curl or wget required"
        exit 1
    fi

    tar -xzf wn3.1.dict.tar.gz
    rm wn3.1.dict.tar.gz

    if [ -f "$WORDNET_DIR/dict/data.noun" ]; then
        echo "✓ WordNet installed successfully"
        rm -rf "$DATA_DIR/wordnet.backup" 2>/dev/null || true
    else
        echo "✗ WordNet installation failed"
        cd "$DATA_DIR"
        rm -rf wordnet 2>/dev/null || true
        mv wordnet.backup wordnet 2>/dev/null || true
        exit 1
    fi
fi

# ============================================================================
# UD English-EWT
# ============================================================================
UD_DIR="$DATA_DIR/ud_english-ewt/UD_English-EWT"
if [ -d "$UD_DIR" ] && [ -f "$UD_DIR/en_ewt-ud-train.conllu" ]; then
    echo "✓ UD English-EWT already installed at $UD_DIR"
else
    echo "Downloading UD English-EWT..."
    cd "$DATA_DIR"

    if [ -d "ud_english-ewt" ]; then
        mv ud_english-ewt ud_english-ewt.backup
    fi

    mkdir -p ud_english-ewt
    cd ud_english-ewt

    git clone --depth 1 https://github.com/UniversalDependencies/UD_English-EWT.git

    if [ -f "$UD_DIR/en_ewt-ud-train.conllu" ]; then
        echo "✓ UD English-EWT installed successfully"
        rm -rf "$DATA_DIR/ud_english-ewt.backup" 2>/dev/null || true
    else
        echo "✗ UD English-EWT installation failed"
        cd "$DATA_DIR"
        rm -rf ud_english-ewt 2>/dev/null || true
        mv ud_english-ewt.backup ud_english-ewt 2>/dev/null || true
        exit 1
    fi
fi

# ============================================================================
# PropBank
# ============================================================================
PROPBANK_DIR="$DATA_DIR/propbank"
if [ -d "$PROPBANK_DIR" ] && [ "$(ls -A $PROPBANK_DIR 2>/dev/null)" ]; then
    echo "✓ PropBank already installed at $PROPBANK_DIR"
else
    echo "Downloading PropBank..."
    cd "$DATA_DIR"

    if [ -d "propbank" ]; then
        mv propbank propbank.backup
    fi

    git clone --depth 1 https://github.com/propbank/propbank-frames.git propbank

    if [ -d "$PROPBANK_DIR" ] && [ "$(ls -A $PROPBANK_DIR 2>/dev/null)" ]; then
        echo "✓ PropBank installed successfully"
        rm -rf propbank.backup 2>/dev/null || true
    else
        echo "✗ PropBank installation failed"
        rm -rf propbank 2>/dev/null || true
        mv propbank.backup propbank 2>/dev/null || true
        exit 1
    fi
fi

# ============================================================================
# FrameNet (Requires manual download)
# ============================================================================
FRAMENET_DIR="$DATA_DIR/framenet"
if [ -d "$FRAMENET_DIR" ] && [ "$(ls -A $FRAMENET_DIR 2>/dev/null)" ]; then
    echo "✓ FrameNet already installed at $FRAMENET_DIR"
else
    echo ""
    echo "⚠ FrameNet requires manual download:"
    echo "  1. Visit https://framenet.icsi.berkeley.edu/"
    echo "  2. Request access to FrameNet data"
    echo "  3. Download FrameNet 1.7"
    echo "  4. Extract to: $FRAMENET_DIR"
    echo ""
fi

# ============================================================================
# Summary
# ============================================================================
echo ""
echo "Data Setup Complete"
echo "==================="
echo ""

# Verify installations
PASS=0
FAIL=0

check_data() {
    if [ -d "$1" ] && [ "$(ls -A $1 2>/dev/null)" ]; then
        echo "  ✓ $2"
        PASS=$((PASS + 1))
    else
        echo "  ✗ $2 (missing)"
        FAIL=$((FAIL + 1))
    fi
}

check_data "$VERBNET_DIR/vn-gl" "VerbNet 3.4"
check_data "$WORDNET_DIR/dict" "WordNet 3.1"
check_data "$UD_DIR" "UD English-EWT"
check_data "$PROPBANK_DIR" "PropBank"
check_data "$FRAMENET_DIR" "FrameNet 1.7"

echo ""
echo "Installed: $PASS/5"
if [ $FAIL -gt 0 ]; then
    echo "Missing: $FAIL (see above for details)"
fi
echo ""

# Validate with demo
echo "To verify installation, run:"
echo "  cargo run --example demo --release"
echo ""
