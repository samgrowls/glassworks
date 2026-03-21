# Glassware Orchestrator — QA & Validation Plan

**Document Version:** 1.0.0  
**Status:** 📋 READY FOR EXECUTION  
**Priority:** P0 (Pre-Release Validation)  
**Estimated Time:** 4-8 hours

---

## Executive Summary

**Compilation success ≠ functional correctness.** We need systematic validation before any release.

Here's a **prioritized QA plan** based on your actual codebase capabilities:

---

## Phase 1: Smoke Tests (30 minutes)

**Goal:** Verify basic commands don't crash.

### 1.1 Help & Version
```bash
cd glassware-orchestrator

# Should show help without errors
./target/debug/glassware-orchestrator --help

# Should show version
./target/debug/glassware-orchestrator --version
```

**Expected:** Clean output, no panics.

### 1.2 Cache Commands (No Network Required)
```bash
# Should work even with empty cache
./target/debug/glassware-orchestrator cache-stats

# Should cleanup without errors
./target/debug/glassware-orchestrator cache-cleanup
```

**Expected:** 
- `cache-stats` shows 0 entries (or creates DB)
- `cache-cleanup` reports 0 removed
- SQLite database created at `.glassware-orchestrator-cache.db`

### 1.3 Single Package Scan (Smallest Test)
```bash
# Scan a tiny, well-known package
./target/debug/glassware-orchestrator scan-npm is-odd --quiet

# Check exit code
echo $?
```

**Expected:** 
- Exit code 0 (no findings) or 1 (findings detected)
- No panics or timeouts
- Cache DB updated

---

## Phase 2: Core Functionality Tests (2 hours)

**Goal:** Validate each major feature works correctly.

### 2.1 npm Package Scanning

| Test | Command | Expected |
|------|---------|----------|
| Single package | `scan-npm lodash` | Completes in <30s |
| Multiple packages | `scan-npm express lodash axios` | All 3 scanned |
| Non-existent package | `scan-npm this-package-does-not-exist-xyz123` | Graceful error |
| Invalid name | `scan-npm "!!!invalid!!!"` | Validation error |

```bash
# Run tests
./target/debug/glassware-orchestrator scan-npm lodash --format json 2>&1 | head -50
./target/debug/glassware-orchestrator scan-npm express axios --concurrency 2
./target/debug/glassware-orchestrator scan-npm this-package-does-not-exist-xyz123 2>&1
```

### 2.2 Output Format Validation

```bash
# JSON output - should be valid JSON
./target/debug/glassware-orchestrator scan-npm lodash --format json > /tmp/test.json
cat /tmp/test.json | jq '.summary'  # Should work

# SARIF output - should be valid SARIF 2.1.0
./target/debug/glassware-orchestrator scan-npm lodash --format sarif > /tmp/test.sarif
cat /tmp/test.sarif | jq '.version'  # Should be "2.1.0"

# Pretty output - should be human readable
./target/debug/glassware-orchestrator scan-npm lodash --format pretty 2>&1 | head -30
```

**Validation Script:**
```bash
#!/bin/bash
# Save as /tmp/validate-output.sh

echo "=== JSON Validation ==="
if jq empty /tmp/test.json 2>/dev/null; then
    echo "✅ JSON is valid"
else
    echo "❌ JSON is invalid"
fi

echo "=== SARIF Validation ==="
if jq -e '.version == "2.1.0"' /tmp/test.sarif >/dev/null 2>&1; then
    echo "✅ SARIF version correct"
else
    echo "❌ SARIF version wrong or invalid"
fi

echo "=== Summary Fields ==="
jq '.summary | keys' /tmp/test.json
```

### 2.3 Severity Filtering

```bash
# Should show all findings
./target/debug/glassware-orchestrator scan-npm some-pkg --severity info

# Should filter to high+critical only
./target/debug/glassware-orchestrator scan-npm some-pkg --severity high

# Compare output sizes
./target/debug/glassware-orchestrator scan-npm lodash --severity info --format json | wc -c
./target/debug/glassware-orchestrator scan-npm lodash --severity critical --format json | wc -c
```

### 2.4 Caching Validation

```bash
# First scan (should be slow, creates cache)
time ./target/debug/glassware-orchestrator scan-npm lodash --quiet

# Second scan (should be instant from cache)
time ./target/debug/glassware-orchestrator scan-npm lodash --quiet

# Check cache stats
./target/debug/glassware-orchestrator cache-stats

# Clear cache and re-test
./target/debug/glassware-orchestrator cache-stats --clear
time ./target/debug/glassware-orchestrator scan-npm lodash --quiet  # Should be slow again
```

**Expected:** Second scan should be 10-100x faster.

---

## Phase 3: GitHub Integration Tests (1 hour)

**Goal:** Validate GitHub repository scanning.

### 3.1 Public Repository Scan

```bash
# Small public repo (fast scan)
./target/debug/glassware-orchestrator scan-github rust-lang/rustfmt --quiet

# With specific ref
./target/debug/glassware-orchestrator scan-github tokio-rs/tokio --ref main --quiet
```

**Note:** These may take 2-5 minutes depending on repo size.

### 3.2 GitHub Token (If Available)

```bash
# With token for private repos (if you have one)
export GLASSWARE_GITHUB_TOKEN=your_token_here
./target/debug/glassware-orchestrator scan-github owner/private-repo --quiet
```

### 3.3 Invalid Repository

```bash
# Should handle gracefully
./target/debug/glassware-orchestrator scan-github this-repo-does-not-exist-xyz/abc 2>&1
```

**Expected:** Clear error message, no panic.

---

## Phase 4: File-Based Scanning (30 minutes)

**Goal:** Validate batch scanning from file.

### 4.1 Create Test Package List

```bash
# Create test file
cat > /tmp/test-packages.txt << EOF
lodash
express
axios
EOF

# Scan from file
./target/debug/glassware-orchestrator scan-file /tmp/test-packages.txt --format json > /tmp/batch-results.json

# Validate
jq '.summary.total_packages' /tmp/batch-results.json  # Should be 3
```

### 4.2 Large Batch Test (Optional)

```bash
# Extract from real package.json if available
jq -r '.dependencies | keys[]' package.json > /tmp/deps.txt 2>/dev/null

# Or create larger list
seq 1 20 | xargs -I{} echo "package-{}" > /tmp/large-list.txt

# Test with concurrency
./target/debug/glassware-orchestrator scan-file /tmp/large-list.txt --concurrency 5 --quiet 2>&1 | tail -20
```

---

## Phase 5: Error Handling Tests (30 minutes)

**Goal:** Verify graceful error handling.

### 5.1 Network Errors

```bash
# Simulate network issue (disconnect or use invalid registry)
./target/debug/glassware-orchestrator scan-npm lodash --npm-rate-limit 0.1 2>&1 | head -20
```

### 5.2 Rate Limiting

```bash
# Aggressive rate limit should trigger throttling
./target/debug/glassware-orchestrator scan-npm lodash express axios --npm-rate-limit 0.5 --verbose 2>&1 | grep -i "rate\|throttl\|wait"
```

### 5.3 Disk Space / Permissions

```bash
# Invalid cache path
./target/debug/glassware-orchestrator scan-npm lodash --cache-db /root/invalid-path/cache.db 2>&1

# Should show clear error, not panic
```

### 5.4 Interrupted Scan (Checkpoint/Resume)

```bash
# Start a scan, then Ctrl+C
./target/debug/glassware-orchestrator scan-file /tmp/large-list.txt &
PID=$!
sleep 5
kill -INT $PID

# Check if checkpoint was created
ls -la .glassware-checkpoints/

# Try to resume
./target/debug/glassware-orchestrator resume npm --packages lodash 2>&1
```

---

## Phase 6: Performance Benchmarking (1 hour)

**Goal:** Compare vs Python harness (from P3 sprint goals).

### 6.1 Baseline Measurement

```bash
# Create consistent test set
cat > /tmp/benchmark-packages.txt << EOF
lodash
express
axios
moment
request
EOF

# Rust orchestrator
echo "=== Rust Orchestrator ==="
time ./target/debug/glassware-orchestrator scan-file /tmp/benchmark-packages.txt --concurrency 10 --quiet

# Python harness (if still available)
echo "=== Python Harness ==="
cd harness
time python3 optimized_scanner.py /tmp/benchmark-packages.txt -w 10 -o /tmp/python-results.json
```

### 6.2 Memory Usage

```bash
# Monitor memory during scan
/usr/bin/time -v ./target/debug/glassware-orchestrator scan-file /tmp/benchmark-packages.txt --concurrency 10 2>&1 | grep -i "maximum resident"

# Should be <500MB per P3 spec
```

### 6.3 Concurrency Scaling

```bash
# Test different concurrency levels
for concurrency in 1 5 10 20; do
    echo "Concurrency: $concurrency"
    time ./target/debug/glassware-orchestrator scan-file /tmp/benchmark-packages.txt --concurrency $concurrency --quiet
done
```

---

## Phase 7: Adversarial Testing Integration (30 minutes)

**Goal:** Verify adversarial testing works (from P3 sprint).

### 7.1 Run Adversarial Test

```bash
# If adversarial flag is enabled
./target/debug/glassware-orchestrator scan-npm lodash --adversarial --quiet 2>&1 | head -30

# Or run adversarial tests directly (if separate command)
./target/debug/glassware-orchestrator adversarial-test --package lodash 2>&1
```

### 7.2 Validate Detection Evasion

```bash
# The adversarial module should test mutation strategies
# Check if evasion rate is reported (<10% per P3 spec)
./target/debug/glassware-orchestrator scan-npm lodash --adversarial --format json 2>&1 | jq '.adversarial.evasion_rate'
```

---

## Phase 8: Output Validation Checklist

**Goal:** Ensure all outputs are production-ready.

### 8.1 JSON Schema Validation

```bash
# Check required fields exist
jq 'has("summary") and has("results")' /tmp/test.json

# Check summary fields
jq '.summary | has("total_packages") and has("malicious_packages")' /tmp/test.json

# Check result structure
jq '.results[0] | has("package") and has("findings")' /tmp/test.json
```

### 8.2 SARIF GitHub Compatibility

```bash
# Upload test to GitHub (optional, if you have a test repo)
# gh api repos/your-user/your-repo/code-scanning/sarif -f sarif_file=@/tmp/test.sarif

# Or validate schema locally
# Download SARIF schema and validate
curl -s https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json > /tmp/sarif-schema.json
# Use a JSON schema validator
```

### 8.3 Log Output (Verbose Mode)

```bash
# Should show structured logs
RUST_LOG=debug ./target/debug/glassware-orchestrator scan-npm lodash --verbose 2>&1 | head -50

# Should include:
# - Package download start/complete
# - Scan start/complete
# - Cache hit/miss
# - Any errors with context
```

---

## Test Results Template

Use this to track results:

```markdown
## QA Test Results

**Date:** ___________  
**Tester:** ___________  
**Version:** v0.8.0

### Phase 1: Smoke Tests
- [ ] Help command works
- [ ] Version command works
- [ ] Cache-stats works
- [ ] Single package scan completes

### Phase 2: Core Functionality
- [ ] npm scanning works
- [ ] JSON output valid
- [ ] SARIF output valid
- [ ] Severity filtering works
- [ ] Caching works (2nd scan faster)

### Phase 3: GitHub Integration
- [ ] Public repo scan works
- [ ] Invalid repo handled gracefully

### Phase 4: File-Based Scanning
- [ ] scan-file works
- [ ] Batch results valid

### Phase 5: Error Handling
- [ ] Network errors handled
- [ ] Rate limiting works
- [ ] Permission errors clear

### Phase 6: Performance
- [ ] Rust faster than Python (target: 2x)
- [ ] Memory <500MB
- [ ] Concurrency scales

### Phase 7: Adversarial
- [ ] Adversarial testing runs
- [ ] Evasion rate reported

### Issues Found
| Severity | Description | File | Line |
|----------|-------------|------|------|
| | | | |

### Go/No-Go Decision
- [ ] **GO** - Ready for release
- [ ] **NO-GO** - Critical issues found
```

---

## Critical Issues to Watch For

| Issue | Severity | Action |
|-------|----------|--------|
| Panic on any command | 🔴 Critical | Block release |
| Cache corruption | 🔴 Critical | Block release |
| Memory >500MB | 🟠 High | Optimize before release |
| SARIF invalid | 🟠 High | Fix before release |
| Scan 50% slower than Python | 🟠 High | Profile and optimize |
| Minor CLI bugs | 🟡 Medium | Document, fix in patch |
| Warning spam | 🟡 Medium | Clean up before release |

---

## Recommended Test Order

**Minimum Viable QA (2 hours):**
1. Phase 1 (Smoke tests)
2. Phase 2.1-2.2 (npm + JSON/SARIF)
3. Phase 2.4 (Caching)
4. Phase 5.1-5.2 (Error handling)

**Full QA (6-8 hours):**
- All 8 phases above

---

## After QA Complete

### If All Tests Pass:
```bash
# Tag release
git tag -a v0.8.0 -m "Rust Orchestrator v0.8.0 - Production Ready"
git push origin v0.8.0

# Build release
cargo build --release

# Publish to crates.io (if desired)
cargo publish --package glassware-orchestrator
```

### If Issues Found:
```bash
# Create issue template
cat > /tmp/qa-issue-template.md << EOF
## Issue: [Description]

**Severity:** Critical/High/Medium/Low  
**Reproduction Steps:**
1. 
2. 
3. 

**Expected:** 

**Actual:** 

**Environment:**
- OS: 
- Rust version: 
- Package scanned: 

**Logs:**
\`\`\`
[paste relevant logs]
\`\`\`
EOF
```

---

## My Recommendation

**Start with Phase 1-2 today (2 hours).** This will tell you:
1. ✅ Does it not crash?
2. ✅ Does scanning work?
3. ✅ Do outputs validate?

**Then decide:**
- If issues found → Fix before continuing
- If clean → Continue with Phase 3-8

**Want me to help interpret test results?** Share any errors or unexpected behavior and I'll provide targeted fixes.

---

**Ready to start testing?** Run Phase 1 and let me know the results. 🎯