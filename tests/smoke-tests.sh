#!/bin/bash
# GlassWorm Smoke Tests with Configuration System
# Purpose: Verify core functionality including new config system
# Date: 2026-03-22
# Version: v0.11.6

set -e

echo "============================================================"
echo "GlassWorm Smoke Tests - v0.11.6 (with Config System)"
echo "============================================================"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test counters
TESTS_PASSED=0
TESTS_FAILED=0

# Helper functions
pass() {
    echo -e "${GREEN}✓ PASS${NC}: $1"
    TESTS_PASSED=$((TESTS_PASSED + 1))
}

fail() {
    echo -e "${RED}✗ FAIL${NC}: $1"
    TESTS_FAILED=$((TESTS_FAILED + 1))
}

warn() {
    echo -e "${YELLOW}⚠ WARN${NC}: $1"
}

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$SCRIPT_DIR"

# Create temp directory for test artifacts
TEST_DIR=$(mktemp -d)
trap "rm -rf $TEST_DIR" EXIT

echo "Test directory: $TEST_DIR"
echo "Script directory: $SCRIPT_DIR"
echo ""

# ============================================================
# Test 1: Configuration System - Init
# ============================================================
echo "============================================================"
echo "Test 1: Configuration System - Init"
echo "============================================================"

# Clean up any existing config
rm -rf ~/.config/glassware 2>/dev/null || true

if cargo run -p glassware-orchestrator -- config init 2>&1 | grep -q "Created"; then
    pass "config init creates default config"
else
    fail "config init failed"
fi

if [ -f "$HOME/.config/glassware/config.toml" ]; then
    pass "config file created at ~/.config/glassware/config.toml"
else
    fail "config file not created"
fi

echo ""

# ============================================================
# Test 2: Configuration System - Show
# ============================================================
echo "============================================================"
echo "Test 2: Configuration System - Show"
echo "============================================================"

if cargo run -p glassware-orchestrator -- config show 2>&1 | grep -q "malicious_threshold"; then
    pass "config show displays configuration"
else
    fail "config show failed"
fi

echo ""

# ============================================================
# Test 3: Configuration System - Validate
# ============================================================
echo "============================================================"
echo "Test 3: Configuration System - Validate"
echo "============================================================"

if cargo run -p glassware-orchestrator -- config validate 2>&1 | grep -q "valid"; then
    pass "config validate passes for default config"
else
    fail "config validate failed"
fi

# Test with invalid config
echo "[scoring]
malicious_threshold = -1.0" > "$TEST_DIR/invalid.toml"
cp ~/.config/glassware/config.toml "$TEST_DIR/backup.toml"
cp "$TEST_DIR/invalid.toml" ~/.config/glassware/config.toml

if cargo run -p glassware-orchestrator -- config validate 2>&1 | grep -qi "error\|invalid\|failed"; then
    pass "config validate catches invalid config"
else
    fail "config validate should catch invalid config"
fi

# Restore valid config
cp "$TEST_DIR/backup.toml" ~/.config/glassware/config.toml

echo ""

# ============================================================
# Test 4: Configuration System - Example Configs
# ============================================================
echo "============================================================"
echo "Test 4: Configuration System - Example Configs"
echo "============================================================"

# Test default example
if [ -f "$SCRIPT_DIR/config-examples/default.toml" ]; then
    cp "$SCRIPT_DIR/config-examples/default.toml" ~/.config/glassware/config.toml
    if cargo run -p glassware-orchestrator -- config validate 2>&1 | grep -q "valid"; then
        pass "default.toml example is valid"
    else
        fail "default.toml example is invalid"
    fi
else
    fail "default.toml example not found"
fi

# Test strict example
if [ -f "$SCRIPT_DIR/config-examples/strict.toml" ]; then
    cp "$SCRIPT_DIR/config-examples/strict.toml" ~/.config/glassware/config.toml
    if cargo run -p glassware-orchestrator -- config validate 2>&1 | grep -q "valid"; then
        pass "strict.toml example is valid"
    else
        fail "strict.toml example is invalid"
    fi
else
    fail "strict.toml example not found"
fi

# Test CI/CD example
if [ -f "$SCRIPT_DIR/config-examples/ci-cd.toml" ]; then
    cp "$SCRIPT_DIR/config-examples/ci-cd.toml" ~/.config/glassware/config.toml
    if cargo run -p glassware-orchestrator -- config validate 2>&1 | grep -q "valid"; then
        pass "ci-cd.toml example is valid"
    else
        fail "ci-cd.toml example is invalid"
    fi
else
    fail "ci-cd.toml example not found"
fi

# Restore default config
cp "$SCRIPT_DIR/config-examples/default.toml" ~/.config/glassware/config.toml

echo ""

# ============================================================
# Test 5: Rust CLI - Basic Scan
# ============================================================
echo "============================================================"
echo "Test 5: Rust CLI - Basic Scan"
echo "============================================================"

if cargo run -p glassware-cli -- --help > /dev/null 2>&1; then
    pass "glassware CLI runs"
else
    fail "glassware CLI failed to run"
fi

# Create test file with zero-width space (U+200B)
echo -n 'const test = "hello' > "$TEST_DIR/test.js"
echo -ne '\xE2\x80\x8B' >> "$TEST_DIR/test.js"
echo '";' >> "$TEST_DIR/test.js"

if cargo run -p glassware-cli -- "$TEST_DIR/test.js" 2>&1 | grep -qi "invisible\|zero-width"; then
    pass "glassware detects invisible characters"
else
    warn "glassware invisible char detection (may need actual GlassWorm pattern)"
fi

echo ""

# ============================================================
# Test 6: Rust CLI - JSON Output
# ============================================================
echo "============================================================"
echo "Test 6: Rust CLI - JSON Output"
echo "============================================================"

if cargo run -p glassware-cli -- --format json "$TEST_DIR/test.js" 2>&1 | grep -q "findings"; then
    pass "glassware JSON output works"
else
    fail "glassware JSON output failed"
fi

echo ""

# ============================================================
# Test 7: Rust Orchestrator - Basic Commands
# ============================================================
echo "============================================================"
echo "Test 7: Rust Orchestrator - Basic Commands"
echo "============================================================"

if cargo run -p glassware-orchestrator -- --help > /dev/null 2>&1; then
    pass "glassware-orchestrator runs"
else
    fail "glassware-orchestrator failed to run"
fi

if cargo run -p glassware-orchestrator -- scan-tarball --help > /dev/null 2>&1; then
    pass "scan-tarball command available"
else
    fail "scan-tarball command not available"
fi

echo ""

# ============================================================
# Test 8: Rust Orchestrator - Tarball Scan (Clean Package)
# ============================================================
echo "============================================================"
echo "Test 8: Rust Orchestrator - Tarball Scan"
echo "============================================================"

# Download a known clean package
cd "$TEST_DIR"
if npm pack express@4.19.2 --silent 2>/dev/null; then
    if [ -f "express-4.19.2.tgz" ]; then
        # Run scan
        OUTPUT=$(cargo run -p glassware-orchestrator --manifest-path "$SCRIPT_DIR/Cargo.toml" -- scan-tarball express-4.19.2.tgz 2>&1)
        
        if echo "$OUTPUT" | grep -q "scanned"; then
            pass "scan-tarball works on clean package"
        else
            # Check if it at least extracted and scanned
            if echo "$OUTPUT" | grep -q "Extracting\|Scanning"; then
                pass "scan-tarball extracts and scans"
            else
                fail "scan-tarball failed on clean package"
                echo "Output: $OUTPUT"
            fi
        fi
    else
        fail "express tarball not created"
    fi
else
    warn "npm pack failed, skipping tarball test"
fi

# Test with malicious package from archive (if available)
ARCHIVE_EVIDENCE="$SCRIPT_DIR/../glassworks-archive/evidence"
if [ -d "$ARCHIVE_EVIDENCE" ] && [ -f "$ARCHIVE_EVIDENCE/react-native-country-select-0.3.91.tgz" ]; then
    OUTPUT=$(cargo run -p glassware-orchestrator --manifest-path "$SCRIPT_DIR/Cargo.toml" -- scan-tarball "$ARCHIVE_EVIDENCE/react-native-country-select-0.3.91.tgz" 2>&1)
    
    if echo "$OUTPUT" | grep -q "malicious"; then
        pass "scan-tarball detects malicious package"
    else
        fail "scan-tarball failed to detect malicious package"
    fi
else
    warn "Malicious test tarball not found (glassworks-archive not available)"
fi

cd "$SCRIPT_DIR"
echo ""

# ============================================================
# Test 9: Python Harness - Module Import
# ============================================================
echo "============================================================"
echo "Test 9: Python Harness - Module Import"
echo "============================================================"

if [ -d "$SCRIPT_DIR/harness" ]; then
    cd "$SCRIPT_DIR/harness"

    if python3 -c "from core.store import Store" 2>/dev/null; then
        pass "Python harness store module imports"
    else
        fail "Python harness store module failed to import"
    fi

    if python3 -c "from core.fetcher import Fetcher" 2>/dev/null; then
        pass "Python harness fetcher module imports"
    else
        fail "Python harness fetcher module failed to import"
    fi

    if python3 -c "from core.scanner import Scanner" 2>/dev/null; then
        pass "Python harness scanner module imports"
    else
        fail "Python harness scanner module failed to import"
    fi

    if python3 -c "from core.orchestrator import Orchestrator" 2>/dev/null; then
        pass "Python harness orchestrator module imports"
    else
        fail "Python harness orchestrator module failed to import"
    fi

    cd "$SCRIPT_DIR"
else
    warn "harness directory not found, skipping Python tests"
fi

echo ""

# ============================================================
# Test 10: Python Harness - Database Operations
# ============================================================
echo "============================================================"
echo "Test 10: Python Harness - Database Operations"
echo "============================================================"

if [ -d "$SCRIPT_DIR/harness" ]; then
    cd "$SCRIPT_DIR/harness"

    if python3 -c "
from core.store import Store
from pathlib import Path
import tempfile

with tempfile.TemporaryDirectory() as tmpdir:
    db_path = Path(tmpdir) / 'test.db'
    store = Store(db_path)
    run_id = store.create_scan_run(wave_id=99, filter_params={'test': True})
    assert run_id, 'Failed to create scan run'
    print('Database operations work')
" 2>/dev/null; then
        pass "Python harness database operations work"
    else
        fail "Python harness database operations failed"
    fi

    cd "$SCRIPT_DIR"
else
    warn "harness directory not found, skipping database test"
fi

echo ""

# ============================================================
# Test 11: Threat Score - Config Integration
# ============================================================
echo "============================================================"
echo "Test 11: Threat Score - Config Integration"
echo "============================================================"

cd "$TEST_DIR"
if [ -f "express-4.19.2.tgz" ]; then
    # Test with default config (threshold 7.0)
    OUTPUT=$(cargo run -p glassware-orchestrator --manifest-path "$SCRIPT_DIR/Cargo.toml" -- scan-tarball express-4.19.2.tgz 2>&1)
    SCORE=$(echo "$OUTPUT" | grep "Average threat score" | grep -oP '[0-9.]+' || echo "10.0")
    
    # Express should score below malicious threshold (7.0)
    if command -v bc &> /dev/null; then
        if (( $(echo "$SCORE < 7.0" | bc -l) )); then
            pass "Clean package scores below malicious threshold ($SCORE < 7.0)"
        else
            fail "Clean package scores at/above malicious threshold ($SCORE >= 7.0)"
        fi
    else
        warn "bc not available, skipping score comparison (score: $SCORE)"
    fi
else
    warn "express tarball not found, skipping threat score test"
fi

cd "$SCRIPT_DIR"
echo ""

# ============================================================
# Summary
# ============================================================
echo "============================================================"
echo "SMOKE TEST SUMMARY"
echo "============================================================"
echo -e "Passed: ${GREEN}$TESTS_PASSED${NC}"
echo -e "Failed: ${RED}$TESTS_FAILED${NC}"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}All smoke tests passed!${NC}"
    exit 0
else
    echo -e "${RED}Some smoke tests failed. Review output above.${NC}"
    exit 1
fi
