#!/bin/bash
# Quick Benchmark Script (5 clean packages for speed)
# Usage: ./scripts/quick-benchmark.sh

set -e

BINARY="${1:-target/release/glassware}"
CLEAN_DIR="benchmarks/clean-packages"
EVIDENCE_DIR="evidence"

echo "=== Quick Benchmark ==="

# Scan 5 clean packages (random sample)
CLEAN_FP=0
echo "Scanning 5 clean packages..."
for tarball in $(ls "$CLEAN_DIR"/*.tgz 2>/dev/null | shuf | head -5); do
    OUTPUT=$("$BINARY" scan-tarball "$tarball" 2>&1 || true)
    if echo "$OUTPUT" | grep -qi "malicious\|flagged"; then
        CLEAN_FP=$((CLEAN_FP + 1))
        echo "  FP: $(basename "$tarball")"
    fi
done

# Scan 3 evidence packages
EVIDENCE_DETECTED=0
echo "Scanning 3 evidence packages..."
for tarball in $(ls "$EVIDENCE_DIR"/*.tgz 2>/dev/null | head -3); do
    OUTPUT=$("$BINARY" scan-tarball "$tarball" 2>&1 || true)
    if echo "$OUTPUT" | grep -qi "malicious\|flagged"; then
        EVIDENCE_DETECTED=$((EVIDENCE_DETECTED + 1))
    fi
done

# Calculate simple metrics
FP_RATE=$(echo "scale=2; $CLEAN_FP / 5" | bc)
DETECTION_RATE=$(echo "scale=2; $EVIDENCE_DETECTED / 3" | bc)

echo ""
echo "Results:"
echo "  Clean FP: $CLEAN_FP / 5 (${FP_RATE})"
echo "  Evidence: $EVIDENCE_DETECTED / 3 (${DETECTION_RATE})"
echo ""
echo "METRIC fp_rate=$FP_RATE"
echo "METRIC detection_rate=$DETECTION_RATE"
