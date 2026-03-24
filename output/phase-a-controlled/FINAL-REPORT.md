# Phase A Campaign - FINAL REPORT

**Campaign:** pre-production-validation  
**Version:** 0.35.0  
**Completed:** March 24, 2026 20:49 UTC  
**Status:** ❌ **HIGH FALSE POSITIVE RATE - TUNING REQUIRED**

---

## Executive Summary

**The campaign revealed a critical issue:** After removing dangerous whitelists (Phase 1), the detectors are now flagging many legitimate, widely-used packages as malicious.

| Metric | Result | Target | Status |
|--------|--------|--------|--------|
| Packages scanned | 181/200 (90.5%) | ≥95% | ⚠️ Slightly below |
| Packages flagged | 92 (50.8%) | - | - |
| **Malicious (≥7.0)** | **30 (16.6%)** | **≤5% FP** | ❌ **CRITICAL** |
| Scan duration | 420 seconds | - | - |
| Scan speed | ~50k LOC/sec | ≥30k LOC/sec | ✅ PASS |

---

## Critical Finding: High False Positive Rate

### Flagged Legitimate Packages (Sample)

These are **well-known, legitimate packages** that should NOT be flagged:

| Package | Findings | Score | Likely FP Reason |
|---------|----------|-------|------------------|
| **express@4.19.2** | 9 | 7.00 | Web framework |
| **typescript@5.3.3** | 42 | 7.00 | Compiler |
| **webpack@5.89.0** | 68 | 7.00 | Build tool |
| **prettier@3.1.1** | 18 | 8.50 | Code formatter |
| **@angular/core@17.1.0** | 11 | 2.00 | UI framework |
| **@mui/material@5.15.5** | 157 | 10.00 | UI framework |
| **antd@5.13.2** | 383 | 7.00 | UI framework |
| **@prisma/client@5.8.1** | 21 | 10.00 | ORM |
| **tailwindcss@3.4.1** | 5 | 7.00 | CSS framework |
| **playwright@1.41.0** | 22 | 10.00 | Testing |
| **three@0.160.0** | 170 | 10.00 | 3D graphics |
| **newrelic@11.10.2** | 99 | 10.00 | Monitoring |

### Analysis

**Root Cause:** The detectors are correctly finding patterns, but these patterns are **legitimate** in these contexts:

1. **UI Frameworks (antd, MUI, Angular):**
   - High invisible character counts (i18n data, icons)
   - Complex build output in /dist/ directories
   - **Should be:** Context-aware detection (i18n files, build artifacts)

2. **Build Tools (webpack, typescript, prettier):**
   - Complex code generation patterns
   - eval/Function for dynamic code
   - **Should be:** Skip code generation in compilers

3. **Blockchain Libraries (@solana/web3.js):**
   - Legitimate blockchain API calls
   - **Should be:** Only flag known C2 wallets (already done)

4. **Monitoring (newrelic, @sentry/node):**
   - HTTP headers for telemetry
   - **Should be:** Distinguish telemetry from exfil

---

## Evidence Validation (Pre-Campaign)

✅ **23/23 evidence packages detected (100%)**

This confirms the detectors WORK - they detect real malicious patterns. The issue is **over-detection** on legitimate packages.

---

## LLM Performance

The LLM integration is working correctly:

| Package | Findings | Score | LLM Verdict | Confidence |
|---------|----------|-------|-------------|------------|
| moment@2.30.1 | 194 | 4.00 | FP | 0.10 ✅ |
| lodash@4.17.21 | 1 | 0.00 | FP | 0.10 ✅ |
| mongodb@6.3.0 | 18 | 4.00 | Malicious | 0.85 ⚠️ |
| amqplib@0.12.0 | 2 | 4.00 | FP | 0.50 ✅ |

**LLM is correctly identifying FPs** but the scoring system is flagging packages before LLM can override.

---

## Recommended Actions

### Immediate (Before Phase B)

1. **Pause wild testing** - Do NOT proceed to Phase B (500 packages)

2. **Tune detectors for context-awareness:**
   - InvisibleChar: Skip i18n files, locale data
   - GlasswarePattern: Skip build tools, compilers
   - BlockchainPolling: Already correct (only known C2)
   - Exfiltration: Distinguish telemetry from exfil

3. **Adjust scoring thresholds:**
   - Consider raising MALICIOUS_THRESHOLD to 8.0-8.5 temporarily
   - OR add more scoring exceptions for legitimate patterns

4. **Re-run evidence validation** - Ensure tuning doesn't break detection

### Short-Term (1-3 days)

1. **Manual review of top 10 flagged packages:**
   - Identify common patterns
   - Create detector-specific tuning rules

2. **Implement detector-specific fixes:**
   - InvisibleChar: Add file type awareness
   - GlasswarePattern: Add build tool context
   - Exfiltration: Add telemetry exceptions

3. **Re-run Phase A** with tuned detectors

### Long-Term (Before v1.0)

1. **Context-aware detection framework:**
   - File type awareness
   - Package type classification
   - Intent analysis

2. **ML-based FP reduction:**
   - Train on labeled dataset
   - Pattern recognition

---

## Detector-Specific Analysis

### InvisibleChar Detector

**Issue:** Flagging i18n data in UI frameworks

**Fix:**
```rust
// Skip i18n/locale files
if path.contains("/locale/") || path.contains("/i18n/") {
    return findings;
}
// Skip files with high legitimate Unicode density
if unicode_density > threshold && is_i18n_context(content) {
    return findings;
}
```

### GlasswarePattern Detector

**Issue:** Flagging build tools and compilers

**Fix:**
```rust
// Skip if package is a known build tool
if is_build_tool_package(package_name) {
    // Only flag if combined with other suspicious patterns
    if !has_evasion_or_c2(findings) {
        return findings;
    }
}
```

### Exfiltration Detector

**Issue:** Flagging telemetry/monitoring

**Fix:**
```rust
// Skip known monitoring packages
if is_monitoring_package(package_name) {
    // Only flag if sending to suspicious endpoints
    if !is_known_telemetry_endpoint(url) {
        return findings;
    }
}
```

---

## Decision: Go/No-Go for Phase B

### Current Status: **NO-GO**

**Reasons:**
- ❌ FP rate 16.6% (target: ≤5%)
- ❌ Flagging core infrastructure packages
- ❌ Would waste manual review time on known-legitimate packages

### Conditions to Proceed to Phase B

- [ ] FP rate reduced to ≤5% on re-run of Phase A
- [ ] Core infrastructure packages (express, webpack, typescript) no longer flagged
- [ ] Evidence detection rate still ≥90%
- [ ] Manual review of top 20 flagged packages completed

---

## Next Steps

1. **Create tuning plan** for each detector
2. **Implement fixes** (1-2 days)
3. **Re-run Phase A** (2-3 hours)
4. **Verify FP rate ≤5%**
5. **Then proceed to Phase B**

---

**Report By:** Glassworks Campaign Agent  
**Date:** March 24, 2026 20:50 UTC  
**Recommendation:** **PAUSE wild testing, tune detectors, re-validate**
