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

# Test tarballs in root evidence directory
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

# Test packages in subdirectories (GlassWorm evidence)
for category_dir in "$EVIDENCE_DIR"/*/; do
    if [ ! -d "$category_dir" ]; then
        continue
    fi
    
    category=$(basename "$category_dir")
    echo "=== Category: $category ==="
    
    for package_dir in "$category_dir"*/; do
        if [ ! -d "$package_dir" ]; then
            continue
        fi
        
        PACKAGE=$(basename "$package_dir")
        TOTAL=$((TOTAL + 1))
        
        echo "Testing: $category/$PACKAGE"
        
        # Create temporary tarball for scanning
        TEMP_TARBALL="/tmp/${category}_${PACKAGE}.tgz"
        tar -czf "$TEMP_TARBALL" -C "$package_dir" . 2>/dev/null
        
        # Run scan and capture output
        OUTPUT=$("$BINARY" scan-tarball "$TEMP_TARBALL" 2>&1 || true)
        
        # Clean up temp tarball
        rm -f "$TEMP_TARBALL"
        
        # Check if flagged as malicious
        if echo "$OUTPUT" | grep -q "malicious\|flagged"; then
            DETECTED=$((DETECTED + 1))
            SCORE=$(echo "$OUTPUT" | grep -oP "threat score: \K[0-9.]+" || echo "unknown")
            echo "  ✓ DETECTED (score: $SCORE)"
        else
            MISSED+=("$category/$PACKAGE")
            SCORE=$(echo "$OUTPUT" | grep -oP "threat score: \K[0-9.]+" || echo "unknown")
            echo "  ✗ MISSED (score: $SCORE)"
        fi
    done
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
