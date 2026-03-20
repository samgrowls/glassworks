#!/usr/bin/env bash
# Test script to verify RDD detector line number accuracy

set -e

echo "=== RDD Detector Line Number Accuracy Test ==="
echo ""

# Create a test package.json with known line numbers
cat > /tmp/test-package.json << 'EOF'
{
  "name": "test-package",
  "version": "1.0.0",
  "author": {
    "name": "JPD"
  },
  "dependencies": {
    "legit-pkg": "^4.0.0",
    "malicious-pkg": "https://storeartifact.com/npm/evil"
  },
  "devDependencies": {
    "another-evil": "http://jpartifacts.com/loader.js"
  }
}
EOF

echo "Test file: /tmp/test-package.json"
echo ""
echo "Expected line numbers:"
echo "  - JPD author: line 5, column 14"
echo "  - malicious-pkg URL: line 9, column 22"
echo "  - another-evil URL: line 12, column 22"
echo ""

# Build the CLI
echo "Building glassware CLI..."
cd /home/property.sightlines/samgrowls/glassworks
cargo build --release --features full 2>&1 | tail -5

echo ""
echo "Running scan with JSON output..."
./target/release/glassware --format json /tmp/test-package.json 2>&1 | python3 -m json.tool

echo ""
echo "=== Test Complete ==="
