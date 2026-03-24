#!/bin/bash
# Evidence Validation Test Script
# Tests all evidence packages and reports detection rate

set -e

EVIDENCE_DIR="${1:-evidence}"
BINARY="${2:-target/release/glassware}"

echo "=== Evidence Validation Test ==="
echo "Evidence directory: $EVIDENCE_DIR"
echo "Binary: $BINARY"
echo ""

if [ ! -f "$BINARY" ]; then
    echo "ERROR: Binary not found at $BINARY"
    echo "Please build first: cargo build --release -p glassware"
    exit 1
fi

if [ ! -d "$EVIDENCE_DIR" ]; then
    echo "ERROR: Evidence directory not found at $EVIDENCE_DIR"
    exit 1
fi

TOTAL=0
DETECTED=0
MISSED=()

for tarball in "$EVIDENCE_DIR"/*.tgz; do
    if [ ! -f "$tarball" ]; then
        continue
    fi
    
    TOTAL=$((TOTAL + 1))
    PACKAGE=$(basename "$tarball")
    
    echo "Testing: $PACKAGE"
    
    # Run scan and capture output
    OUTPUT=$("$BINARY" scan-tarball "$tarball" 2>&1 || true)
    
    # Check if flagged as malicious
    if echo "$OUTPUT" | grep -q "malicious\|flagged"; then
        DETECTED=$((DETECTED + 1))
        SCORE=$(echo "$OUTPUT" | grep -oP "threat score: \K[0-9.]+" || echo "unknown")
        echo "  ✓ DETECTED (score: $SCORE)"
    else
        MISSED+=("$PACKAGE")
        SCORE=$(echo "$OUTPUT" | grep -oP "threat score: \K[0-9.]+" || echo "unknown")
        echo "  ✗ MISSED (score: $SCORE)"
    fi
    echo ""
done

echo "=== Results ==="
echo "Total packages: $TOTAL"
echo "Detected: $DETECTED"
echo "Missed: ${#MISSED[@]}"

if [ $TOTAL -gt 0 ]; then
    RATE=$(echo "scale=2; $DETECTED * 100 / $TOTAL" | bc)
    echo "Detection rate: ${RATE}%"
    
    if [ ${#MISSED[@]} -gt 0 ]; then
        echo ""
        echo "Missed packages:"
        for pkg in "${MISSED[@]}"; do
            echo "  - $pkg"
        done
    fi
    
    # Check if we meet target (≥90%)
    if (( $(echo "$RATE >= 90" | bc -l) )); then
        echo ""
        echo "✓ Target met: Detection rate ≥90%"
        exit 0
    else
        echo ""
        echo "✗ Target NOT met: Detection rate <90%"
        exit 1
    fi
else
    echo "ERROR: No evidence packages found"
    exit 1
fi
