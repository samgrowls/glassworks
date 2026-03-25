#!/bin/bash
# Autoresearch benchmark wrapper for GlassWorm FP rate tuning
# 
# This script runs the FP/success rate benchmark and outputs metrics
# in a format that autoresearch can parse.
#
# Usage: ./autoresearch.sh
# Output: METRIC combined_score=<value>
#         METRIC evidence_detection_rate=<value>
#         METRIC fp_rate=<value>

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Run the benchmark
echo "Running GlassWorm FP/Success Benchmark..."
./benchmarks/fp-success-benchmark.sh --quick 2>&1 | tee /tmp/benchmark-output.txt

# Extract metrics from JSON output (last JSON block in the output)
JSON_OUTPUT=$(grep -A5 '"evidence_detection_rate"' /tmp/benchmark-output.txt)

# Parse values using awk for more reliable extraction
EVIDENCE_RATE=$(echo "$JSON_OUTPUT" | awk -F': ' '/evidence_detection_rate/ {gsub(/[,}]/,"",$2); print $2}')
FP_RATE=$(echo "$JSON_OUTPUT" | awk -F': ' '/fp_rate/ {gsub(/[,}]/,"",$2); print $2}')
COMBINED=$(echo "$JSON_OUTPUT" | awk -F': ' '/combined_score/ {gsub(/[,}]/,"",$2); print $2}')

# Fallback: try direct grep if awk fails
if [[ -z "$COMBINED" ]]; then
    COMBINED=$(grep 'combined_score' /tmp/benchmark-output.txt | awk -F': ' '{gsub(/[,}]/,"",$2); print $2}')
fi

# Output in autoresearch format
echo ""
echo "=== Autoresearch Metrics ==="
echo "METRIC combined_score=$COMBINED"
echo "METRIC evidence_detection_rate=$EVIDENCE_RATE"
echo "METRIC fp_rate=$FP_RATE"

# Validate metrics were extracted
if [[ -z "$COMBINED" || -z "$EVIDENCE_RATE" || -z "$FP_RATE" ]]; then
    echo "ERROR: Failed to parse metrics from benchmark output"
    exit 1
fi
