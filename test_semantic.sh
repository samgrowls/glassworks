#!/bin/bash
# Test script to verify semantic detection

set -e

echo "=== Testing Semantic Detection ==="
echo ""

# Build with semantic feature
echo "Building with semantic feature..."
cargo build -p glassware --features full 2>&1 | tail -5

echo ""
echo "Scanning wave5_aes_decrypt_eval.js fixture..."
echo ""

# Scan the fixture
./target/debug/glassware scan glassware-core/tests/fixtures/glassworm/wave5_aes_decrypt_eval.js 2>&1 | tail -30

echo ""
echo "=== Test Complete ==="
