# Phase A Re-Run - Tuning Results Report

**Campaign:** pre-production-validation (tuned)  
**Version:** 0.38.0-phase-a-tuned  
**Completed:** March 24, 2026 23:17 UTC  
**Status:** ⚠️ **FP RATE UNCHANGED - ADDITIONAL TUNING NEEDED**

---

## Executive Summary

**The detector tuning did NOT reduce the FP rate as expected.** Despite implementing context-aware detection for i18n, telemetry, SDK usage, and build tools, the FP rate remains at ~16.6%.

| Metric | Before Tuning | After Tuning | Change |
|--------|---------------|--------------|--------|
| Packages scanned | 181 | 181 | - |
| Packages flagged | 92 | 94 | +2 |
| **Malicious (≥7.0)** | **30 (16.6%)** | **30 (16.6%)** | **0%** |
| Evidence detection | 23/23 (100%) | 23/23 (100%) | - |

---

## Root Cause Analysis

The FP rate remained unchanged because:

### 1. Scoring System Override

The detectors are correctly applying context-aware logic (fewer findings), but the **scoring system is still flagging packages** based on:

- **Category diversity** - Multiple low-severity findings across categories
- **Critical hits** - Some findings still marked Critical despite context
- **Score thresholds** - 7.0 threshold may be too low for tuned detectors

### 2. LLM Not Overriding Scores

The LLM is correctly identifying FPs (confidence 0.10-0.50), but the scoring system flags packages **before** LLM can override:

```
Package: moment@2.30.1
- Findings: 194 (i18n data)
- Score: 4.00 (below 7.0 threshold) ✅
- LLM confidence: 0.10 (FP) ✅
- Result: NOT flagged as malicious ✅

Package: antd@5.13.2
- Findings: 383 (i18n + build output)
- Score: 7.00 (at threshold) ❌
- LLM confidence: N/A (already flagged)
- Result: Flagged as malicious ❌
```

### 3. Specific Packages Still Flagged

| Package | Findings | Score | Why Still Flagged |
|---------|----------|-------|-------------------|
| **typescript@5.3.3** | 36 | 10.00 | Build output + eval patterns |
| **webpack@5.89.0** | 68 | 7.00 | Build output + Function patterns |
| **@mui/material** | 157 | 10.00 | i18n + build output |
| **antd@5.13.2** | 383 | 7.00 | i18n + build output |
| **@prisma/client** | 24 | 10.00 | ORM patterns |
| **@solana/web3.js** | 259 | 10.00 | Blockchain patterns |
| **three@0.160.0** | 170 | 10.00 | Build output |
| **newrelic@11.10.2** | 102 | 10.00 | Telemetry + headers |

---

## What Worked

### ✅ Evidence Detection Maintained

**23/23 evidence packages still detected (100%)** - The tuning did NOT break real malicious pattern detection.

### ✅ LLM Working Correctly

LLM is correctly identifying FPs:
- `discord.js`: LLM confidence 0.10 (FP) ✅
- `globalize`: LLM confidence 0.10 (FP) ✅
- `mailgun.js`: LLM confidence 0.50 (FP) ✅
- `meilisearch`: LLM confidence 0.50 (FP) ✅

### ✅ Some Packages No Longer Flagged

Comparing before/after:
- `express@4.19.2`: Was 7.00 → Now NOT flagged ✅
- `mongoose@8.0.3`: Was flagged → Now NOT flagged ✅
- `tailwindcss@3.4.1`: Was 7.00 → Now NOT flagged ✅

---

## What Didn't Work

### ❌ Scoring System Not Adjusted

The scoring system is still using the same thresholds:
- MALICIOUS_THRESHOLD: 7.0
- Category diversity caps unchanged
- No adjustment for context-aware findings

### ❌ Build Output Still Flagged

Despite `is_build_output()` detection, packages like `typescript`, `webpack`, `three` are still flagged because:
- Build output detection requires BOTH path AND content match
- Some packages have malicious-looking patterns in source files (not just build output)

### ❌ i18n Detection Not Sufficient

The `is_i18n_file()` function requires 3+ indicators, but:
- UI frameworks have i18n data WITHOUT explicit i18n markers
- Locale data in JSON files still flagged

---

## Recommended Next Steps

### Immediate (Before Phase B)

1. **Raise MALICIOUS_THRESHOLD to 8.0-8.5**
   - This alone would reduce FP rate significantly
   - Evidence detection would remain 100% (real attacks score 8.0+)

2. **Add LLM-based score override**
   - If LLM confidence < 0.30, reduce score by 50%
   - This leverages the working LLM triage

3. **Tune build output detection**
   - Skip build output directories regardless of content
   - Only flag if evasion/C2 patterns present

### Short-Term (1-2 days)

1. **Implement ML-based FP reduction**
   - Use LLM feedback to train classifier
   - Automatically adjust scores based on package type

2. **Add package type classification**
   - UI framework → expect i18n
   - Build tool → expect code generation
   - Monitoring → expect telemetry headers

3. **Re-run Phase A** with adjusted thresholds

---

## Decision: Go/No-Go for Phase B

### Current Status: **NO-GO**

**Reasons:**
- ❌ FP rate still 16.6% (target: ≤5%)
- ❌ Core infrastructure packages still flagged
- ❌ Scoring system needs adjustment

### Path Forward

**Option 1: Raise Threshold (Quick Fix)**
```toml
# campaigns/phase-a-controlled/config.toml
[thresholds]
malicious = 8.5  # Raise from 7.0
suspicious = 4.0  # Raise from 3.5
```

**Expected:** FP rate drops to ~8-10% (still above 5% target)

**Option 2: LLM-Based Score Override (Recommended)**
```rust
// In scanner.rs::calculate_threat_score()
if llm_verdict.confidence < 0.30 {
    score *= 0.5;  // Reduce score by 50% for likely FPs
}
```

**Expected:** FP rate drops to ~5-7% (close to target)

**Option 3: Combined Approach (Best)**
- Raise threshold to 8.0
- Add LLM-based score override
- Tune build output detection

**Expected:** FP rate drops to ≤5% (target met)

---

## Lessons Learned

1. **Detector tuning alone is insufficient** - Scoring system must also be adjusted

2. **LLM is working correctly** - Should be leveraged more aggressively for FP reduction

3. **Context-aware detection is complex** - Requires package type classification, not just file content analysis

4. **Evidence detection is robust** - 100% detection maintained through all tuning

---

**Report By:** Glassworks Campaign Agent  
**Date:** March 24, 2026 23:20 UTC  
**Recommendation:** Implement Option 3 (Combined Approach), re-run Phase A, then proceed to Phase B
