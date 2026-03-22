#!/bin/bash
# Wave 3 Scanner — 100 Package Hunt with LLM Integration
# 
# Uses release binaries from ~/.local/bin
# Uses existing scan-npm and scan-tarball commands
#
# Usage: ./wave3_scan.sh [--llm] [--deep-llm]

set -e

# Configuration
GLASSWARE="$HOME/.local/bin/glassware-orchestrator"
OUTPUT_DIR="$(dirname "$0")/data/wave3-results"
EVIDENCE_DIR="$HOME/glassworks-archive/evidence"
mkdir -p "$OUTPUT_DIR"

# Check if binary exists
if [ ! -x "$GLASSWARE" ]; then
    echo "Error: glassware-orchestrator not found at $GLASSWARE"
    echo "Please run: cargo build --release -p glassware-orchestrator && cp target/release/glassware-orchestrator ~/.local/bin/"
    exit 1
fi

# Parse arguments
USE_LLM=""
LLM_PROVIDER="cerebras"  # Default to Cerebras for fast triage
if [[ "$@" == *"--deep-llm"* ]]; then
    USE_LLM="--llm"
    LLM_PROVIDER="nvidia"
    echo "Deep LLM analysis enabled (NVIDIA)"
elif [[ "$@" == *"--llm"* ]]; then
    USE_LLM="--llm"
    echo "LLM triage enabled (Cerebras)"
fi

# Set LLM environment based on provider
if [ -n "$USE_LLM" ]; then
    if [ "$LLM_PROVIDER" = "nvidia" ]; then
        export GLASSWARE_LLM_BASE_URL="https://integrate.api.nvidia.com/v1"
        export GLASSWARE_LLM_API_KEY="${NVIDIA_API_KEY:-}"
        export GLASSWARE_LLM_MODEL="qwen/qwen3.5-397b-a17b"
    else
        export GLASSWARE_LLM_BASE_URL="https://api.cerebras.ai/v1"
        export GLASSWARE_LLM_API_KEY="${GLASSWARE_LLM_API_KEY:-}"
        export GLASSWARE_LLM_MODEL="llama-3.3-70b"
    fi
fi

echo "============================================================"
echo "Wave 3 Scanner — 100 Package Hunt"
echo "============================================================"
echo ""
echo "Binary: $GLASSWARE"
echo "Output: $OUTPUT_DIR"
echo "LLM: ${USE_LLM:-disabled} (${LLM_PROVIDER^^})"
echo ""

# Evidence tarballs (malicious packages)
EVIDENCE_TARBALLS=()
for f in "$EVIDENCE_DIR"/*.tgz; do
    if [ -f "$f" ]; then
        EVIDENCE_TARBALLS+=("$f")
    fi
done

# NPM packages for Wave 3 (100 total target)
# Mix of high-risk categories and random samples
NPM_PACKAGES=(
    # High-download packages (FP baseline) - 20
    "express@4.19.2" "lodash@4.17.21" "axios@1.6.7" "chalk@5.3.0"
    "debug@4.3.4" "moment@2.30.1" "uuid@9.0.1" "async@3.2.5"
    "commander@12.0.0" "glob@10.3.10" "mkdirp@3.0.1" "semver@7.6.0"
    "ws@8.16.0" "yargs@17.7.2" "dotenv@16.4.5" "prettier@3.2.5"
    "typescript@5.4.2" "jest@29.7.0" "react@18.3.1" "vue@3.4.21"
    
    # Native/build tools (medium risk) - 7
    "node-gyp@10.1.0" "bindings@1.5.0" "prebuild@11.0.0" "nan@2.18.0"
    "node-addon-api@7.1.0" "cmake-js@7.3.0" "node-pre-gyp@1.0.11"
    
    # Crypto (legitimate crypto usage) - 7
    "ethers@6.11.1" "web3@4.6.0" "viem@2.9.1" "wagmi@2.5.0"
    "@solana/web3.js@1.87.0" "bitcoinjs-lib@6.1.0" "hdkey@2.1.0"
    
    # AI/ML (currently targeted) - 6
    "langchain@0.1.0" "openai@4.28.0" "@anthropic-ai/sdk@0.18.0"
    "ai@3.0.0" "replicate@0.27.0" "cohere-ai@7.9.0"
    
    # Dev tools (high trust) - 7
    "eslint@8.57.0" "webpack@5.90.0" "vite@5.1.0" "rollup@4.12.0"
    "esbuild@0.20.0" "@babel/core@7.24.0" "swc-core@0.90.0"
    
    # Utilities (widely used) - 7
    "dayjs@1.11.10" "got@14.2.0" "node-fetch@3.3.2" "undici@6.5.0"
    "chalk@5.3.0" "ora@8.0.1" "inquirer@9.2.0"
    
    # Recent publishes (last 3 months) - 10
    "next@14.1.0" "nuxt@3.10.0" "svelte@4.2.0" "prisma@5.10.0"
    "tailwindcss@3.4.0" "zod@3.22.0" "valtio@1.13.0" "zustand@4.5.0"
    "immer@10.0.0" "jotai@2.6.0"
    
    # Install scripts (high risk) - 4
    "core-js@3.36.0" "postinstall-postinstall@2.1.0"
    "node-sass@9.0.0" "chromedriver@122.0.0"
    
    # Random samples - 10
    "left-pad@1.3.0" "is-odd@3.0.1" "fsevents@2.3.3"
    "colors@1.4.0" "flat-cache@5.0.0" "rimraf@5.0.5"
    "cross-env@7.0.3" "del@7.1.0" "cpy@11.0.0" "globby@14.0.0"
)

TIMESTAMP=$(date +%Y%m%d-%H%M%S)

# Step 1: Scan evidence tarballs
if [ ${#EVIDENCE_TARBALLS[@]} -gt 0 ]; then
    echo "Step 1: Scanning ${#EVIDENCE_TARBALLS[@]} evidence tarballs..."
    TARBALL_OUTPUT="$OUTPUT_DIR/wave3-evidence-$TIMESTAMP.json"
    
    $GLASSWARE scan-tarball "${EVIDENCE_TARBALLS[@]}" $USE_LLM \
        --output "$TARBALL_OUTPUT" --format json 2>&1 | tee "$OUTPUT_DIR/wave3-evidence-log-$TIMESTAMP.txt"
    
    echo ""
fi

# Step 2: Scan npm packages
echo "Step 2: Scanning ${#NPM_PACKAGES[@]} npm packages..."
NPM_OUTPUT="$OUTPUT_DIR/wave3-npm-$TIMESTAMP.json"

$GLASSWARE scan-npm "${NPM_PACKAGES[@]}" $USE_LLM \
    --output "$NPM_OUTPUT" --format json 2>&1 | tee "$OUTPUT_DIR/wave3-npm-log-$TIMESTAMP.txt"

echo ""
echo "============================================================"
echo "WAVE 3 COMPLETE"
echo "============================================================"
echo ""
echo "Evidence results: $TARBALL_OUTPUT"
echo "Evidence log: $OUTPUT_DIR/wave3-evidence-log-$TIMESTAMP.txt"
echo ""
echo "NPM results: $NPM_OUTPUT"
echo "NPM log: $OUTPUT_DIR/wave3-npm-log-$TIMESTAMP.txt"
echo ""

# Show summary
if command -v jq &> /dev/null; then
    echo "=== EVIDENCE SUMMARY ==="
    jq -r '.summary | "Total: \(.total_packages)\nMalicious: \(.malicious_packages)\nAvg Score: \(.average_threat_score)"' "$TARBALL_OUTPUT" 2>/dev/null || echo "No summary available"
    
    echo ""
    echo "Malicious (evidence):"
    jq -r '.results[] | select(.is_malicious == true) | "  - \(.package_name)@\(.version) (score: \(.threat_score))"' "$TARBALL_OUTPUT" 2>/dev/null || echo "None detected"
    
    echo ""
    echo "=== NPM SUMMARY ==="
    jq -r '.summary | "Total: \(.total_packages)\nMalicious: \(.malicious_packages)\nAvg Score: \(.average_threat_score)"' "$NPM_OUTPUT" 2>/dev/null || echo "No summary available"
    
    echo ""
    echo "Malicious (npm):"
    jq -r '.results[] | select(.is_malicious == true) | "  - \(.package_name)@\(.version) (score: \(.threat_score))"' "$NPM_OUTPUT" 2>/dev/null || echo "None detected"
fi
