#!/bin/bash
# Evidence baseline test script

echo "=== EVIDENCE BASELINE TEST ==="
echo ""

cd /home/shva/samgrowls/glassworks-v0.57.0-longwave

# Clean cache
rm -rf .glassware*

for tgz in evidence/*.tgz; do
    pkg=$(basename "$tgz" .tgz)
    echo "=== $pkg ==="
    ./target/release/glassware scan-tarball "$tgz" 2>&1 | grep -E "Malicious packages:|threat score:|Total findings:"
    echo ""
done

echo "=== TEST COMPLETE ==="
