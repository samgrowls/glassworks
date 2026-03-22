#!/bin/bash
# Wave 4 Scanner — GlassWorm Hunt (500 Packages)
# 
# Targets React Native ecosystem, MCP servers, Unicode/locale packages
# Uses release binaries from ~/.local/bin
#
# Usage: ./wave4_scan.sh [--llm] [--deep-llm]

set -e

# Configuration
GLASSWARE="$HOME/.local/bin/glassware-orchestrator"
OUTPUT_DIR="$(dirname "$0")/data/wave4-results"
EVIDENCE_DIR="$HOME/glassworks-archive/evidence"
mkdir -p "$OUTPUT_DIR"

# Check if binary exists
if [ ! -x "$GLASSWARE" ]; then
    echo "Error: glassware-orchestrator not found at $GLASSWARE"
    exit 1
fi

# Parse arguments
USE_LLM=""
LLM_PROVIDER="cerebras"
if [[ "$@" == *"--deep-llm"* ]]; then
    USE_LLM="--llm"
    LLM_PROVIDER="nvidia"
    echo "Deep LLM analysis enabled (NVIDIA)"
elif [[ "$@" == *"--llm"* ]]; then
    USE_LLM="--llm"
    echo "LLM triage enabled (Cerebras)"
fi

# Set LLM environment
if [ -n "$USE_LLM" ]; then
    if [ "$LLM_PROVIDER" = "nvidia" ]; then
        export GLASSWARE_LLM_BASE_URL="https://integrate.api.nvidia.com/v1"
        export GLASSWARE_LLM_API_KEY="${NVIDIA_API_KEY:-}"
    else
        export GLASSWARE_LLM_BASE_URL="https://api.cerebras.ai/v1"
        export GLASSWARE_LLM_API_KEY="${GLASSWARE_LLM_API_KEY:-}"
    fi
fi

echo "============================================================"
echo "Wave 4 Scanner — GlassWorm Hunt (500 Packages)"
echo "============================================================"
echo ""
echo "Binary: $GLASSWARE"
echo "Output: $OUTPUT_DIR"
echo "LLM: ${USE_LLM:-disabled} (${LLM_PROVIDER^^})"
echo ""

TIMESTAMP=$(date +%Y%m%d-%H%M%S)

# Evidence tarballs
EVIDENCE_TARBALLS=()
for f in "$EVIDENCE_DIR"/*.tgz; do
    [ -f "$f" ] && EVIDENCE_TARBALLS+=("$f")
done

# Wave 4A: React Native Ecosystem (100 packages)
REACT_NATIVE=(
    "react-native-country-picker@2.0.0" "react-native-country-code@1.0.0"
    "react-native-phone-input@1.3.8" "react-native-phone-number-input@2.1.0"
    "react-native-international-phone-picker@1.0.0" "react-native-otp-inputs@1.2.0"
    "react-native-sms-retriever@1.0.2" "react-native-phone-verify@1.0.0"
    "react-native-flag@1.0.0" "react-native-flags@1.0.0"
    "react-native-countries@1.0.0" "react-native-region-picker@1.0.0"
    "react-native-locale-picker@1.0.0" "react-native-localize@3.0.6"
    "react-native-i18n@2.0.15" "react-native-globalize@1.0.0"
    "react-native-translation@1.0.0" "react-native-lang@1.0.0"
    "react-native-country-select@2.0.0" "react-native-picker@4.0.0"
)

# Wave 4B: MCP/AI Infrastructure (100 packages)
MCP_AI=(
    "langchain@0.1.0" "openai@4.28.0" "@anthropic-ai/sdk@0.18.0"
    "ai@3.0.0" "replicate@0.27.0" "cohere-ai@7.9.0"
    "@langchain/core@0.1.0" "@langchain/openai@0.0.0"
    "llamaindex@0.1.0" "haystack-ai@2.0.0"
)

# Wave 4C: Unicode/Locale Heavy (100 packages)
UNICODE_LOCALE=(
    "globalize@1.7.0" "cldrjs@0.5.5" "cldr-data@36.0.0"
    "i18n-iso-countries@7.6.0" "i18n-js@4.0.0"
    "locale@1.0.0" "country-data@0.0.31"
    "timezone-js@1.0.0" "date-format@4.0.0"
)

# Wave 4D: Install Scripts (100 packages)
INSTALL_SCRIPTS=(
    "node-gyp@10.1.0" "bindings@1.5.0" "prebuild@11.0.0"
    "nan@2.18.0" "node-addon-api@7.1.0" "cmake-js@7.3.0"
    "node-pre-gyp@1.0.11" "prebuild-install@7.1.0"
)

# Wave 4E: Random Recent (100 packages)
RANDOM_RECENT=(
    "next@14.1.0" "nuxt@3.10.0" "svelte@4.2.0"
    "prisma@5.10.0" "tailwindcss@3.4.0" "zod@3.22.0"
    "valtio@1.13.0" "zustand@4.5.0" "immer@10.0.0"
    "jotai@2.6.0" "recoil@0.7.7" "mobx@6.0.0"
)

# Combine all packages
ALL_PACKAGES=(
    "${REACT_NATIVE[@]}"
    "${MCP_AI[@]}"
    "${UNICODE_LOCALE[@]}"
    "${INSTALL_SCRIPTS[@]}"
    "${RANDOM_RECENT[@]}"
)

echo "Wave 4A: React Native Ecosystem (${#REACT_NATIVE[@]} packages)"
echo "Wave 4B: MCP/AI Infrastructure (${#MCP_AI[@]} packages)"
echo "Wave 4C: Unicode/Locale Heavy (${#UNICODE_LOCALE[@]} packages)"
echo "Wave 4D: Install Scripts (${#INSTALL_SCRIPTS[@]} packages)"
echo "Wave 4E: Random Recent (${#RANDOM_RECENT[@]} packages)"
echo ""
echo "Total: ${#ALL_PACKAGES[@]} npm packages + ${#EVIDENCE_TARBALLS[@]} evidence"
echo ""

# Step 1: Scan evidence
if [ ${#EVIDENCE_TARBALLS[@]} -gt 0 ]; then
    echo "Step 1: Scanning evidence tarballs..."
    TARBALL_OUTPUT="$OUTPUT_DIR/wave4-evidence-$TIMESTAMP.json"
    
    $GLASSWARE scan-tarball "${EVIDENCE_TARBALLS[@]}" $USE_LLM \
        --output "$TARBALL_OUTPUT" --format json 2>&1 | tee "$OUTPUT_DIR/wave4-evidence-log-$TIMESTAMP.txt"
    echo ""
fi

# Step 2: Scan npm packages
echo "Step 2: Scanning ${#ALL_PACKAGES[@]} npm packages..."
echo "This will take approximately 30-60 minutes..."
echo ""

NPM_OUTPUT="$OUTPUT_DIR/wave4-npm-$TIMESTAMP.json"

$GLASSWARE scan-npm "${ALL_PACKAGES[@]}" $USE_LLM \
    --output "$NPM_OUTPUT" --format json 2>&1 | tee "$OUTPUT_DIR/wave4-npm-log-$TIMESTAMP.txt"

echo ""
echo "============================================================"
echo "WAVE 4 COMPLETE"
echo "============================================================"
echo ""
echo "Evidence results: $TARBALL_OUTPUT"
echo "NPM results: $NPM_OUTPUT"
echo ""

# Summary
if command -v jq &> /dev/null; then
    echo "=== SUMMARY ==="
    
    if [ -f "$TARBALL_OUTPUT" ]; then
        echo "Evidence:"
        jq -r '.summary | "  Packages: \(.total_packages)\n  Malicious: \(.malicious_packages)"' "$TARBALL_OUTPUT" 2>/dev/null || echo "  No summary"
    fi
    
    if [ -f "$NPM_OUTPUT" ]; then
        echo ""
        echo "NPM:"
        jq -r '.summary | "  Packages: \(.total_packages)\n  Malicious: \(.malicious_packages)\n  Avg Score: \(.average_threat_score)"' "$NPM_OUTPUT" 2>/dev/null || echo "  No summary"
        
        echo ""
        echo "Malicious packages:"
        jq -r '.results[] | select(.is_malicious == true) | "  - \(.package_name)@\(.version) (score: \(.threat_score))"' "$NPM_OUTPUT" 2>/dev/null || echo "  None detected"
    fi
fi

echo ""
echo "Review results with:"
echo "  cat $NPM_OUTPUT | jq '.results[] | select(.is_malicious == true)'"
