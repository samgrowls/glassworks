#!/bin/bash
# Glassworks Benchmark Script
# Runs glassware on benchmark packages and outputs F1 score
#
# Usage: ./scripts/run-benchmark.sh [evidence_dir] [clean_dir]
# Output: METRIC f1_score=X.XX fp_rate=X.XX detection_rate=X.XX

set -e

EVIDENCE_DIR="${1:-evidence}"
CLEAN_DIR="${2:-benchmarks/clean-packages}"
BINARY="${3:-target/release/glassware}"
OUTPUT_FILE="${4:-output/autoresearch/benchmark-result.json}"

echo "=== Glassworks Benchmark ==="
echo "Evidence: $EVIDENCE_DIR"
echo "Clean: $CLEAN_DIR"
echo "Binary: $BINARY"
echo ""

# Check binary exists
if [ ! -f "$BINARY" ]; then
    echo "ERROR: Binary not found at $BINARY"
    echo "Run: cargo build --release"
    exit 1
fi

# Initialize counters
EVIDENCE_TOTAL=0
EVIDENCE_DETECTED=0
CLEAN_TOTAL=0
CLEAN_FLAGGED=0

# Scan evidence packages (should be flagged)
echo "Scanning evidence packages..."
for tarball in "$EVIDENCE_DIR"/*.tgz; do
    [ -f "$tarball" ] || continue
    EVIDENCE_TOTAL=$((EVIDENCE_TOTAL + 1))
    
    OUTPUT=$("$BINARY" scan-tarball "$tarball" 2>&1 || true)
    
    if echo "$OUTPUT" | grep -qi "malicious\|flagged"; then
        EVIDENCE_DETECTED=$((EVIDENCE_DETECTED + 1))
    fi
done

# Scan evidence subdirectories
for category_dir in "$EVIDENCE_DIR"/*/; do
    [ -d "$category_dir" ] || continue
    
    for package_dir in "$category_dir"*/; do
        [ -d "$package_dir" ] || continue
        
        EVIDENCE_TOTAL=$((EVIDENCE_TOTAL + 1))
        
        # Create temp tarball
        TEMP_TARBALL="/tmp/evidence_$(basename "$package_dir").tgz"
        tar -czf "$TEMP_TARBALL" -C "$package_dir" . 2>/dev/null
        
        OUTPUT=$("$BINARY" scan-tarball "$TEMP_TARBALL" 2>&1 || true)
        rm -f "$TEMP_TARBALL"
        
        if echo "$OUTPUT" | grep -qi "malicious\|flagged"; then
            EVIDENCE_DETECTED=$((EVIDENCE_DETECTED + 1))
        fi
    done
done

# Scan clean packages (should NOT be flagged)
echo "Scanning clean packages..."
for tarball in "$CLEAN_DIR"/*.tgz; do
    [ -f "$tarball" ] || continue
    CLEAN_TOTAL=$((CLEAN_TOTAL + 1))
    
    OUTPUT=$("$BINARY" scan-tarball "$tarball" 2>&1 || true)
    
    if echo "$OUTPUT" | grep -qi "malicious\|flagged"; then
        CLEAN_FLAGGED=$((CLEAN_FLAGGED + 1))
    fi
done

# Calculate metrics
if [ $EVIDENCE_TOTAL -eq 0 ]; then
    echo "ERROR: No evidence packages found"
    exit 1
fi

if [ $CLEAN_TOTAL -eq 0 ]; then
    echo "ERROR: No clean packages found"
    exit 1
fi

DETECTION_RATE=$(echo "scale=4; $EVIDENCE_DETECTED / $EVIDENCE_TOTAL" | bc)
FP_RATE=$(echo "scale=4; $CLEAN_FLAGGED / $CLEAN_TOTAL" | bc)
PRECISION=$(echo "scale=4; $EVIDENCE_DETECTED / ($EVIDENCE_DETECTED + $CLEAN_FLAGGED)" | bc 2>/dev/null || echo "0")

# Calculate F1 score
if (( $(echo "$PRECISION + $DETECTION_RATE == 0" | bc -l) )); then
    F1_SCORE="0.0000"
else
    F1_SCORE=$(echo "scale=4; 2 * ($PRECISION * $DETECTION_RATE) / ($PRECISION + $DETECTION_RATE)" | bc)
fi

# Output results
echo ""
echo "=== Results ==="
echo "Evidence: $EVIDENCE_DETECTED / $EVIDENCE_TOTAL detected (${DETECTION_RATE})"
echo "Clean FP: $CLEAN_FLAGGED / $CLEAN_TOTAL flagged (${FP_RATE})"
echo "Precision: ${PRECISION}"
echo "F1 Score: ${F1_SCORE}"
echo ""

# Output in parseable format
echo "METRIC f1_score=$F1_SCORE"
echo "METRIC fp_rate=$FP_RATE"
echo "METRIC detection_rate=$DETECTION_RATE"
echo "METRIC evidence_detected=$EVIDENCE_DETECTED"
echo "METRIC evidence_total=$EVIDENCE_TOTAL"
echo "METRIC clean_flagged=$CLEAN_FLAGGED"
echo "METRIC clean_total=$CLEAN_TOTAL"

# Save to JSON file
mkdir -p "$(dirname "$OUTPUT_FILE")"
cat > "$OUTPUT_FILE" << EOF
{
    "timestamp": "$(date -Iseconds)",
    "f1_score": $F1_SCORE,
    "fp_rate": $FP_RATE,
    "detection_rate": $DETECTION_RATE,
    "precision": $PRECISION,
    "evidence_detected": $EVIDENCE_DETECTED,
    "evidence_total": $EVIDENCE_TOTAL,
    "clean_flagged": $CLEAN_FLAGGED,
    "clean_total": $CLEAN_TOTAL
}
EOF

echo ""
echo "Results saved to: $OUTPUT_FILE"

# Exit with success if F1 > 0.5 and FP < 10%
if (( $(echo "$F1_SCORE > 0.5 && $FP_RATE < 0.10" | bc -l) )); then
    exit 0
else
    exit 0  # Still exit 0, let caller decide
fi
