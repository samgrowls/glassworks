#!/bin/bash
# Post-Scan Deep LLM Analysis
# 
# Analyzes flagged packages from a wave scan using NVIDIA (deep analysis)
# 
# Usage: ./analyze_flagged.sh <results.json> [--all]
#   --all: Analyze all packages, not just flagged ones

set -e

GLASSWARE="$HOME/.local/bin/glassware-orchestrator"
OUTPUT_DIR="$(dirname "$0")/data/deep-analysis"
mkdir -p "$OUTPUT_DIR"

if [ ! -x "$GLASSWARE" ]; then
    echo "Error: glassware-orchestrator not found at $GLASSWARE"
    exit 1
fi

if [ -z "$1" ]; then
    echo "Usage: $0 <results.json> [--all]"
    echo ""
    echo "Analyzes flagged packages from a wave scan using NVIDIA deep analysis."
    echo ""
    echo "Arguments:"
    echo "  results.json  - JSON output from wave scan"
    echo "  --all         - Analyze all packages, not just flagged ones"
    exit 1
fi

RESULTS_FILE="$1"
ANALYZE_ALL=""
if [[ "$2" == "--all" ]]; then
    ANALYZE_ALL="yes"
fi

if [ ! -f "$RESULTS_FILE" ]; then
    echo "Error: Results file not found: $RESULTS_FILE"
    exit 1
fi

# Extract flagged packages
echo "Extracting flagged packages from $RESULTS_FILE..."

if [ -n "$ANALYZE_ALL" ]; then
    # Analyze all packages
    PACKAGES=$(jq -r '.results[] | "\(.package_name)@\(.version)"' "$RESULTS_FILE")
else
    # Only analyze flagged/malicious packages
    PACKAGES=$(jq -r '.results[] | select(.is_malicious == true or .threat_score >= 5.0) | "\(.package_name)@\(.version)"' "$RESULTS_FILE")
fi

if [ -z "$PACKAGES" ]; then
    echo "No packages to analyze!"
    exit 0
fi

echo ""
echo "Packages to analyze:"
echo "$PACKAGES" | while read pkg; do echo "  - $pkg"; done
echo ""

# Convert to array
PKG_ARRAY=()
while IFS= read -r line; do
    PKG_ARRAY+=("$line")
done <<< "$PACKAGES"

TIMESTAMP=$(date +%Y%m%d-%H%M%S)
DEEP_OUTPUT="$OUTPUT_DIR/deep-analysis-$TIMESTAMP.json"

# Set NVIDIA environment
export GLASSWARE_LLM_BASE_URL="https://integrate.api.nvidia.com/v1"
export GLASSWARE_LLM_API_KEY="${NVIDIA_API_KEY:-}"
export GLASSWARE_LLM_MODEL="qwen/qwen3.5-397b-a17b"

if [ -z "$NVIDIA_API_KEY" ]; then
    echo "Warning: NVIDIA_API_KEY not set. Deep analysis will fail."
    echo "Set it with: export NVIDIA_API_KEY='nvapi-...'"
    echo ""
fi

echo "============================================================"
echo "Running Deep LLM Analysis (NVIDIA)"
echo "============================================================"
echo ""
echo "Model: qwen/qwen3.5-397b-a17b (397B parameters)"
echo "Expected time: ~15-30 seconds per package"
echo ""

$GLASSWARE scan-npm "${PKG_ARRAY[@]}" --llm \
    --output "$DEEP_OUTPUT" --format json 2>&1 | tee "$OUTPUT_DIR/deep-analysis-log-$TIMESTAMP.txt"

echo ""
echo "============================================================"
echo "DEEP ANALYSIS COMPLETE"
echo "============================================================"
echo ""
echo "Results: $DEEP_OUTPUT"
echo "Log: $OUTPUT_DIR/deep-analysis-log-$TIMESTAMP.txt"
echo ""

# Show verdicts
if [ -f "$DEEP_OUTPUT" ]; then
    echo "LLM Verdicts:"
    jq -r '.results[] | "  \(.package_name): \(.llm_verdict.malicious // "N/A") - \(.llm_verdict.reason // "No reason")"' "$DEEP_OUTPUT" 2>/dev/null || echo "  No verdicts available"
    
    echo ""
    echo "Summary:"
    jq -r '.summary | "  Total analyzed: \(.total_packages)\n  Malicious: \(.malicious_packages)"' "$DEEP_OUTPUT" 2>/dev/null || echo "  No summary"
fi
