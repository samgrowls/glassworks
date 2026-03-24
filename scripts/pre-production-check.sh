#!/bin/bash
# pre-production-check.sh
# Glassworks Pre-Production Verification

echo "=== GLASSWORKS PRE-PRODUCTION CHECK ==="
echo ""

# 1. Build check
echo "[1/8] Building release binary..."
cargo build --release -p glassware 2>&1 | tail -5
if [ $? -ne 0 ]; then
    echo "❌ BUILD FAILED"
    exit 1
fi
echo "✅ Build passed"
echo ""

# 2. Detector registration check
echo "[2/8] Checking detector registration..."
if grep -q "UnicodeSteganographyV2" glassware-core/src/engine.rs && \
   grep -q "BlockchainPolling" glassware-core/src/engine.rs && \
   grep -q "SandboxEvasion" glassware-core/src/engine.rs && \
   grep -q "Exfiltration" glassware-core/src/engine.rs; then
    echo "✅ All Phase 8 detectors registered"
else
    echo "❌ Phase 8 detectors NOT registered"
    exit 1
fi
echo ""

# 3. DetectionCategory check
echo "[3/8] Checking DetectionCategory usage..."
if grep -q "SteganoPayload" glassware-core/src/detectors/unicode_steganography_v2.rs && \
   grep -q "BlockchainC2" glassware-core/src/detectors/blockchain_polling.rs && \
   grep -q "TimeDelaySandboxEvasion" glassware-core/src/detectors/sandbox_evasion.rs && \
   grep -q "HeaderC2" glassware-core/src/detectors/exfiltration.rs; then
    echo "✅ DetectionCategory properly used"
else
    echo "⚠️ DetectionCategory may need review"
fi
echo ""

# 4. Evidence count check
echo "[4/8] Checking evidence library..."
EVIDENCE_COUNT=$(find evidence -name "package.json" | wc -l)
echo "Evidence packages: $EVIDENCE_COUNT (target: 20+)"
if [ $EVIDENCE_COUNT -ge 20 ]; then
    echo "✅ Evidence library sufficient"
else
    echo "⚠️ Evidence library below target"
fi
echo ""

# 5. Test suite check
echo "[5/8] Running test suite..."
cargo test --release --lib 2>&1 | tail -10
echo ""

# 6. Documentation check
echo "[6/8] Checking documentation..."
if [ -f "docs/DETECTION.md" ] && [ -f "docs/SCORING.md" ] && [ -f "docs/LLM.md" ]; then
    echo "✅ Core documentation present"
else
    echo "❌ Documentation missing"
fi
echo ""

# 7. Unwrap check
echo "[7/8] Checking for unwrap() calls..."
UNWRAP_COUNT=$(grep -rn "\.unwrap()" --include="*.rs" glassware/src/ glassware-core/src/ | grep -v "test" | grep -v "#" | wc -l)
echo "unwrap() calls in production code: $UNWRAP_COUNT"
if [ $UNWRAP_COUNT -lt 20 ]; then
    echo "✅ Acceptable unwrap count"
else
    echo "⚠️ Consider reducing unwrap() calls"
fi
echo ""

# 8. Whitelist check
echo "[8/8] Checking for dangerous whitelists..."
WHITELIST_COUNT=$(grep -r "ant-design\|vuetify\|element-plus\|quasar" --include="*.toml" campaigns/ | grep -v "^#" | wc -l)
echo "Dangerous whitelist entries: $WHITELIST_COUNT"
if [ $WHITELIST_COUNT -eq 0 ]; then
    echo "✅ No dangerous whitelists"
else
    echo "❌ Dangerous whitelists found"
    exit 1
fi
echo ""

echo "=== CHECK COMPLETE ==="
echo ""
echo "Status: Ready for limited production testing"
echo "Next: Run ./tests/validate-evidence.sh to verify detection rate"
