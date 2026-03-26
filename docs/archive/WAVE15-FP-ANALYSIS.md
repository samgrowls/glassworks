# Wave15 FP Analysis Report

**Date:** 2026-03-26
**Campaign:** wave15-validation (204 packages)
**Status:** COMPLETE

---

## Summary Statistics

| Metric | Value |
|--------|-------|
| Packages Scanned | 197 |
| Packages Flagged | 70 (35.5%) |
| Malicious (score >= 7.0) | 11 (5.6%) |
| **True Positives** | **0** (0%) |
| **False Positives** | **11** (100% of flagged) |

**Evidence Detection:** N/A (evidence packages not on npm, scanned via tarball)
- iflow-mcp-watercrawl-mcp-1.3.4: 8.50 score ✅ (tarball scan)

---

## False Positive Analysis

### FP Breakdown by Package

| Package | Score | Primary Detectors | Root Cause |
|---------|-------|-------------------|------------|
| @prisma/client | 10.00 | HeaderC2, TimeDelay, EncryptedPayload | Build tool patterns |
| @solana/web3.js | 10.00 | BlockchainC2 (245 findings) | Legitimate SDK |
| prisma | 10.00 | Same as @prisma/client | Build tool patterns |
| playwright-core | 10.00 | TimeDelay, HeaderC2 | Browser automation |
| mongodb | 9.00 | BlockchainC2, HeaderC2 | Database driver |
| pm2 | 8.92 | TimeDelay, HeaderC2 | Process manager |
| firebase | 8.83 | BlockchainC2, HeaderC2 | Firebase SDK |
| testcafe | 9.00 | TimeDelay, HeaderC2 | Test framework |
| prettier | 6.32 | Various | Code formatter |
| playwright | 6.45 | TimeDelay, HeaderC2 | Browser automation |
| pino | 5.67 | Various | Logger |

---

## LLM Tier 2 Analysis (Deep Dive)

### @prisma/client@5.8.1

**Detector Score:** 10.00 (malicious=true)
**LLM Tier 2 Verdict:** malicious=false, confidence=0.15

**LLM Assessment:**
> "This is a legitimate database ORM tool. The detected patterns are:
> - CI-aware time delays: Used for build optimization, not sandbox evasion
> - Header C2 patterns: Standard HTTP headers for telemetry/analytics
> - RC4 patterns: False positive on build tool string manipulation
> 
> **Recommendation:** FALSE POSITIVE - Prisma is a well-known, legitimate ORM."

**Root Cause:**
1. **TimeDelaySandboxEvasion** - CI checks in build tools look like sandbox evasion
2. **HeaderC2** - Telemetry headers look like C2 communication
3. **EncryptedPayload** - Build tool string encoding looks like encryption

**Fix Required:**
- Add build tool detection (check for webpack/babel/rollup signatures)
- Skip CI checks in build output directories
- Whitelist known build tool patterns

---

### @solana/web3.js@1.87.6

**Detector Score:** 10.00 (malicious=true)
**LLM Tier 2 Verdict:** Not run (rate limited)

**Root Cause:**
- **BlockchainC2** detector flags ANY Solana RPC usage
- 245 findings from legitimate SDK methods

**Fix Required:**
- BlockchainC2 should only flag known malicious wallets/IPs
- Skip generic SDK usage (getSignaturesForAddress as a function, not C2 call)

---

### firebase@10.7.2

**Detector Score:** 8.83 (malicious=true)
**LLM Tier 2 Verdict:** Not run (rate limited)

**Root Cause:**
- **BlockchainC2** - Firebase uses blockchain-like patterns
- **HeaderC2** - Analytics/telemetry headers

**Fix Required:**
- Firebase is a known legitimate SDK
- Add Firebase to known-good SDK detection

---

## Detector-Specific FP Analysis

### 1. BlockchainC2 Detector

**FP Count:** ~5 packages (@solana/web3.js, firebase, mongodb, etc.)
**Root Cause:** Flags ANY blockchain API usage

**Fix:**
```rust
// Current (too aggressive):
if content.contains("getSignaturesForAddress") { flag() }

// Fixed (specific):
if KNOWN_C2_WALLETS.iter().any(|w| content.contains(w)) {
    flag()  // Only known malicious wallets
}
```

### 2. TimeDelaySandboxEvasion Detector

**FP Count:** ~6 packages (prisma, playwright, pm2, testcafe)
**Root Cause:** CI checks in build tools look like sandbox evasion

**Fix:**
```rust
// Skip if build tool output
if is_build_output(content) {
    return findings  // Skip
}

// Only flag if CI check + delay + no legitimate context
if has_ci_check(content) && has_delay(content) {
    if is_build_context(content) {
        return findings  // Legitimate build optimization
    }
    flag()  // Actual evasion
}
```

### 3. HeaderC2 Detector

**FP Count:** ~8 packages (prisma, firebase, playwright, etc.)
**Root Cause:** Telemetry/analytics headers look like C2

**Fix:**
```rust
// Known telemetry headers (not C2)
const LEGIT_HEADERS = [
    "X-Telemetry",
    "X-Analytics",
    "X-Build-Id",
    "X-Prisma-",
];

if header.starts_with(LEGIT_HEADERS) {
    return findings  // Skip telemetry
}
```

### 4. EncryptedPayload/Rc4Pattern Detectors

**FP Count:** ~3 packages (prisma, prettier)
**Root Cause:** Build tool string manipulation looks like encryption

**Fix:**
```rust
// Skip build tool output
if is_minified(content) || has_webpack_signature(content) {
    return findings
}

// Only flag if high-entropy + dynamic exec + no build context
if has_high_entropy(content) && has_dynamic_exec(content) {
    if !is_build_context(content) {
        flag()
    }
}
```

---

## Priority Fixes

### Immediate (Next Session)

1. **Build Tool Detection**
   - Add `is_build_output()` helper
   - Skip webpack/babel/rollup/tsc output
   - Check for source maps, minification signatures

2. **BlockchainC2 Specificity**
   - Only flag known malicious wallets/IPs
   - Skip generic SDK method calls

3. **Telemetry Header Whitelist**
   - Add known telemetry headers
   - Skip X-Prisma-, X-Telemetry, X-Analytics, etc.

### Short-Term

4. **CI Check Context**
   - Distinguish build-time CI checks from runtime evasion
   - Check file path (generator-build/ vs runtime/)

5. **LLM Tier 1 Re-enable**
   - Fix rate limiting (increase threshold to 8.0)
   - Use for borderline cases (5.0-8.0 score)

---

## Expected FP Rate After Fixes

| Fix | Expected FP Reduction |
|-----|----------------------|
| Build tool detection | -5 FPs (prisma, playwright, etc.) |
| BlockchainC2 specificity | -3 FPs (@solana, firebase, mongodb) |
| Telemetry whitelist | -2 FPs (firebase, prisma) |
| CI check context | -2 FPs (prisma, pm2) |

**Current FP Rate:** 11/197 = 5.6%
**Target FP Rate:** < 1% (2 FPs max)
**Expected After Fixes:** 0-1 FPs

---

## Evidence Detection Status

**Evidence Packages:** 4 (scanned via tarball, not npm)
- iflow-mcp-watercrawl-mcp-1.3.4: 8.50 score ✅
- glassworm-combo-002: 7.00 score ✅
- glassworm-combo-003: 7.00 score ✅
- glassworm-combo-004: 7.00 score ✅

**Evidence Detection Rate:** 4/4 = 100% ✅

---

## Recommendations

1. **Implement build tool detection FIRST** - This fixes the majority of FPs
2. **Fix BlockchainC2 specificity** - Second highest impact
3. **Re-enable LLM Tier 1 with higher threshold (8.0)** - For borderline cases
4. **Run wave16 (500 pkg) after fixes** - Validate FP rate < 1%

---

**Next Session Priority:** Build tool detection + BlockchainC2 fix
**Expected Outcome:** FP rate < 1%, 100% evidence detection maintained
