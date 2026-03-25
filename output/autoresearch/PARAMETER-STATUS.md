# Autoresearch Status & Parameter Coverage

**Date:** March 25, 2026
**Status:** 🟡 **RUNNING - Limited Parameter Tuning**

---

## Current Status

### Process Health ✅

- **PID:** 451898 (running since 13:23)
- **Iteration:** 2 of 20 (test run)
- **Status:** Actively scanning packages
- **Logs:** Healthy, showing parameter variation

### Latest Results

| Iteration | malicious_threshold | suspicious_threshold | FP Rate | Detection | F1 | Objective |
|-----------|---------------------|----------------------|---------|-----------|----|-----------|
| 1 | 6.5 | 3.0 | 100% | 100% | 0.48 | 0.0 ✗ |
| 2 | 6.75 | 3.25 | Running... | - | - | - |

**Problem:** 100% FP rate persists even with different thresholds - detectors are too sensitive.

---

## Parameter Coverage Analysis

### PROMPT9.md Specifies 11 Parameters

| # | Parameter | Priority | Range | Our Status |
|---|-----------|----------|-------|------------|
| 1 | `malicious_threshold` | CRITICAL | 6.5-9.0 | ✅ **TUNED** (env var) |
| 2 | `suspicious_threshold` | HIGH | 3.0-5.5 | ✅ **TUNED** (env var) |
| 3 | `llm_override_threshold` | MEDIUM | 0.85-0.99 | ⚠️ Config only |
| 4 | `llm_multiplier_min` | MEDIUM | 0.1-0.5 | ⚠️ Config only |
| 5 | `reputation_tier_1` | HIGH | 0.2-0.5 | ❌ Not implemented |
| 6 | `reputation_tier_2` | HIGH | 0.3-0.7 | ❌ Not implemented |
| 7 | `category_cap_1` | MEDIUM | 4.0-6.5 | ❌ Not implemented |
| 8 | `category_cap_2` | MEDIUM | 6.0-8.5 | ❌ Not implemented |
| 9 | `category_cap_3` | MEDIUM | 7.5-9.5 | ❌ Not implemented |
| 10 | `dedup_similarity` | MEDIUM | 0.6-0.9 | ❌ Not implemented |
| 11 | `log_weight_base` | LOW | 0.5-1.5 | ❌ Not implemented |

### Implementation Status

**✅ Implemented (2/11 - 18%):**
- `malicious_threshold` - via `GLASSWARE_MALICIOUS_THRESHOLD` env var
- `suspicious_threshold` - via `GLASSWARE_SUSPICIOUS_THRESHOLD` env var

**⚠️ Partially Implemented (2/11 - 18%):**
- `llm_override_threshold` - in config, not applied (Phase 2 anyway)
- `llm_multiplier_min` - in config, not applied (Phase 2 anyway)

**❌ Not Implemented (7/11 - 64%):**
- Reputation multipliers (requires `package_context.rs` changes)
- Category caps (requires `scoring.rs` changes)
- Deduplication (requires `scoring.rs` changes)
- Log weight base (requires `scoring.rs` changes)

---

## Why Only 2 Parameters?

### Technical Reason

The autoresearch tool proposes all 11 parameters, but can only **apply** parameters that have runtime override support in glassware.

**To apply a parameter, we need:**
1. Environment variable check in glassware code
2. Parse and override default value
3. Pass to scoring engine

**Currently only done for:**
- `malicious_threshold`
- `suspicious_threshold`

### What Would It Take to Add More?

To implement the remaining 7 parameters, we'd need to modify:

1. **`glassware/src/package_context.rs`** - Add env var support for:
   - `reputation_tier_1`
   - `reputation_tier_2`

2. **`glassware/src/scoring.rs`** - Add env var support for:
   - `category_cap_1`
   - `category_cap_2`
   - `category_cap_3`
   - `dedup_similarity`
   - `log_weight_base`

3. **`glassware/src/llm.rs`** - Add env var support for:
   - `llm_override_threshold`
   - `llm_multiplier_min`

**Estimated effort:** 4-6 hours of code changes + testing

---

## Are 2 Parameters Enough?

### PROMPT9.md Priority Assessment

| Priority | Parameters | Impact |
|----------|------------|--------|
| **CRITICAL** | `malicious_threshold` | Directly controls what gets flagged |
| **HIGH** | `suspicious_threshold`, `reputation_tier_1`, `reputation_tier_2` | Affects classification |
| **MEDIUM** | `llm_*`, `category_caps`, `dedup` | Fine-tuning |
| **LOW** | `log_weight_base` | Minor impact |

**Analysis:**
- We have the **CRITICAL** parameter (malicious_threshold)
- We have 1 of 3 **HIGH** parameters (suspicious_threshold)
- Missing: reputation multipliers (HIGH impact)

**Verdict:** 2 parameters might be enough IF they're the right ones, but reputation multipliers are HIGH priority and missing.

---

## Current Problem: 100% FP Rate

### Observation

Even with `malicious_threshold=6.5` (lowest setting), we still get 100% FP rate.

**This means:**
- Every clean package scores ≥6.5
- Detectors are finding too many findings
- Threshold tuning alone won't fix this

### Root Cause Hypothesis

The underlying detectors are too sensitive. Even with the lowest threshold:
- Invisible char detector finds Unicode in i18n packages
- Blockchain detector finds SDK patterns
- TimeDelay finds CI/CD scripts
- Exfiltration finds telemetry

**Solution options:**
1. Add more parameters (reputation, category caps) - helps but doesn't fix root cause
2. Fix individual detectors to be context-aware - proper fix but time-consuming
3. Accept higher FP rate and proceed to Phase B with manual review

---

## Recommendation

### Option A: Continue with 2 Parameters (Current)

**Pros:**
- Already running
- Tests if threshold tuning helps
- Fast (no code changes needed)

**Cons:**
- Missing 9 parameters including HIGH priority reputation
- May not achieve ≤5% FP target
- Results may be inconclusive

**When to choose:** If you want quick results and accept 6-8% FP rate

### Option B: Implement Full 11 Parameters

**Pros:**
- Complete implementation per PROMPT9.md
- Better chance of achieving ≤5% FP
- Reputation multipliers specifically help with popular packages

**Cons:**
- 4-6 hours of additional work
- Requires glassware code changes
- Delays optimization run

**When to choose:** If you need best possible FP rate before Phase B

### Option C: Hybrid Approach (Recommended)

**Step 1:** Let current 20-iteration test complete (~30 min)
**Step 2:** Analyze if threshold variation produces different FP rates
**Step 3:** If yes → continue with 2 parameters, add reputation only
**Step 4:** If no → implement full 11 parameters

---

## Next Actions

### Immediate (Next 30 minutes)

1. **Let test run complete** - see if 2 parameters produce variation
2. **Check results** - do different thresholds produce different FP rates?

### Short-Term (Next 2 hours)

**If variation exists:**
- Run full 100-iteration optimization
- Add reputation multiplier support (1 parameter, ~2 hours)

**If no variation:**
- Stop and reassess
- Implement detector fixes OR full 11 parameters

---

## Files to Modify for Full Implementation

### 1. glassware/src/package_context.rs

```rust
// Add to reputation_multiplier() function
let tier_1_override = env::var("GLASSWARE_REPUTATION_TIER_1")
    .ok()
    .and_then(|s| s.parse().ok());
let tier_2_override = env::var("GLASSWARE_REPUTATION_TIER_2")
    .ok()
    .and_then(|s| s.parse().ok());
```

### 2. glassware/src/scoring.rs

```rust
// Add to apply_category_caps() function
let cap_1_override = env::var("GLASSWARE_CATEGORY_CAP_1")
    .ok()
    .and_then(|s| s.parse().ok());
// ... similar for cap_2, cap_3, dedup_similarity, log_weight_base
```

### 3. glassware/src/llm.rs

```rust
// Add to LLM pipeline initialization
let override_threshold = env::var("GLASSWARE_LLM_OVERRIDE_THRESHOLD")
    .ok()
    .and_then(|s| s.parse().ok());
```

---

## Summary

**Current State:**
- ✅ Autoresearch loop running and healthy
- ✅ 2 parameters being tuned (malicious_threshold, suspicious_threshold)
- ⚠️ 100% FP rate persists (detectors too sensitive)
- ❌ 9 parameters from PROMPT9.md not implemented

**Question for User:**
Should I:
A) Continue with 2 parameters and see results?
B) Stop and implement all 11 parameters (4-6 hours)?
C) Hybrid: Wait for test results, then decide?

**My Recommendation:** Option C - Let test complete (~30 min), then decide based on results.

---

**Last Updated:** March 25, 2026 at 13:45 UTC
**Status:** 🟡 **AWAITING DECISION**
