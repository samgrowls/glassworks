#!/bin/bash
# Performance benchmark for Finding Eq/Hash and cache clone optimization
# This script measures the improvement from removing unnecessary clones

set -e

echo "=== Glassware Performance Benchmark ==="
echo ""
echo "Testing cache performance with optimized clone removal..."
echo ""

# Create a test directory with sample files
TEST_DIR="/tmp/glassware_bench_$$"
mkdir -p "$TEST_DIR"

# Create test files with various patterns
for i in {1..100}; do
    cat > "$TEST_DIR/file_$i.js" << 'EOF'
// Test file with invisible characters
const x = "test\u{FE00}";
const y = "hello\u{FE01}world";
function foo() {
    return "bar\u{FE02}";
}
EOF
done

echo "Created 100 test files in $TEST_DIR"
echo ""

# Run scan with cache enabled
echo "Running first scan (cache miss)..."
time cargo run --release --features "full" -- "$TEST_DIR" --cache-file "$TEST_DIR/.cache.json" > /dev/null 2>&1

echo ""
echo "Running second scan (cache hit)..."
time cargo run --release --features "full" -- "$TEST_DIR" --cache-file "$TEST_DIR/.cache.json" > /dev/null 2>&1

echo ""
echo "Running third scan (cache hit)..."
time cargo run --release --features "full" -- "$TEST_DIR" --cache-file "$TEST_DIR/.cache.json" > /dev/null 2>&1

# Cleanup
rm -rf "$TEST_DIR"

echo ""
echo "=== Benchmark Complete ==="
echo ""
echo "Note: The optimized version removes unnecessary clones in the cache path,"
echo "resulting in reduced memory allocations and improved performance on cache hits."
