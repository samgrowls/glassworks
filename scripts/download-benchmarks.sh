#!/bin/bash
# Download benchmark packages for autoresearch loop
# Usage: ./download-benchmarks.sh [output_dir]

set -e

OUTPUT_DIR="${1:-benchmarks/clean-packages}"
PACKAGE_LIST="benchmarks/clean-packages/packages.txt"

echo "=== Downloading Clean Benchmark Packages ==="
echo "Output directory: $OUTPUT_DIR"
echo "Package list: $PACKAGE_LIST"
echo ""

mkdir -p "$OUTPUT_DIR"

if [ ! -f "$PACKAGE_LIST" ]; then
    echo "ERROR: Package list not found at $PACKAGE_LIST"
    exit 1
fi

TOTAL=0
SUCCESS=0
FAILED=0

# Read packages and download
while IFS= read -r package; do
    # Skip comments and empty lines
    [[ "$package" =~ ^#.*$ ]] && continue
    [[ -z "$package" ]] && continue
    
    TOTAL=$((TOTAL + 1))
    echo "[$TOTAL] Downloading: $package"
    
    # Download package
    if npm pack "$package" --pack-destination "$OUTPUT_DIR/" 2>/dev/null; then
        SUCCESS=$((SUCCESS + 1))
        echo "  ✓ Success"
    else
        FAILED=$((FAILED + 1))
        echo "  ✗ Failed (continuing...)"
    fi
done < "$PACKAGE_LIST"

echo ""
echo "=== Download Summary ==="
echo "Total packages: $TOTAL"
echo "Successful: $SUCCESS"
echo "Failed: $FAILED"
echo ""

if [ $SUCCESS -gt 0 ]; then
    echo "Downloaded packages:"
    ls -lh "$OUTPUT_DIR"/*.tgz 2>/dev/null | wc -l
    echo ""
    echo "Total size:"
    du -sh "$OUTPUT_DIR"/*.tgz 2>/dev/null | tail -1
fi

if [ $FAILED -gt 0 ]; then
    echo ""
    echo "WARNING: $FAILED packages failed to download"
    echo "This is OK if you have at least 50 packages for iteration testing"
fi

exit 0
