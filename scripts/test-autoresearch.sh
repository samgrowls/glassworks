#!/bin/bash
# Test autoresearch infrastructure
# Usage: ./test-autoresearch.sh

set -e

echo "=== Testing Autoresearch Infrastructure ==="
echo ""

# Test 1: Check binary builds
echo "Test 1: Building glassware-tools..."
cd /home/shva/samgrowls/glassworks
cargo build -p glassware-tools --release 2>&1 | tail -5
echo "✓ Build successful"
echo ""

# Test 2: Check configuration loads
echo "Test 2: Testing configuration loading..."
if [ -f "glassware-tools/autoresearch.toml" ]; then
    echo "✓ Configuration file exists"
else
    echo "✗ Configuration file not found"
    exit 1
fi
echo ""

# Test 3: Check benchmark package list
echo "Test 3: Checking benchmark package list..."
if [ -f "benchmarks/clean-packages/packages.txt" ]; then
    COUNT=$(grep -v "^#" benchmarks/clean-packages/packages.txt | grep -v "^$" | wc -l)
    echo "✓ Package list exists ($COUNT packages)"
else
    echo "✗ Package list not found"
    exit 1
fi
echo ""

# Test 4: Check evidence directory
echo "Test 4: Checking evidence directory..."
if [ -d "evidence" ]; then
    EVIDENCE_COUNT=$(find evidence -name "*.tgz" | wc -l)
    echo "✓ Evidence directory exists ($EVIDENCE_COUNT tarballs)"
else
    echo "✗ Evidence directory not found"
    exit 1
fi
echo ""

# Test 5: Check glassware binary exists
echo "Test 5: Checking glassware binary..."
if [ -f "target/release/glassware" ]; then
    VERSION=$(./target/release/glassware --version)
    echo "✓ Glassware binary exists ($VERSION)"
else
    echo "⚠ Glassware binary not found (building...)"
    cargo build --release -p glassware 2>&1 | tail -3
    if [ -f "target/release/glassware" ]; then
        echo "✓ Build complete"
    else
        echo "✗ Build failed"
        exit 1
    fi
fi
echo ""

echo "=== All Tests Passed ==="
echo ""
echo "Next steps:"
echo "1. Download benchmark packages:"
echo "   cargo run -p glassware-tools --bin download-benchmarks"
echo ""
echo "2. Run autoresearch (test run with 5 iterations):"
echo "   cargo run -p glassware-tools --bin autoresearch -- --max-iterations 5"
echo ""
