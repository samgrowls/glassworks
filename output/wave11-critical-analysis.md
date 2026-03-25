# Wave 11 Results Analysis - CRITICAL FINDINGS

**Date:** March 25, 2026  
**Status:** ⚠️ **CRITICAL - FP Investigation Required**

---

## The Problem

**9 packages flagged as malicious** - but we need to understand WHY before proceeding.

---

## Root Cause discovered

**Wave 11 config has `malicious_threshold = 5.0`** (not 7.0 or 8.0)!

This means packages with score ≥ 5.0 are flagged as malicious, not ≥ 7.0 or 8.0.

---

## The 9 Malicious Packages - Detailed Analysis

### 1. @prisma/client@5.8.1
- **Score:** 10.00 (MAX!)
- **Findings:** 24
- **LLM:** Rate limited (no override)
- **Assessment:** ⚠️ **LIKELY FP** - ORM package, database patterns
- **Why 10.00?** Scoring exception not working? Need to investigate findings

### 2. @solana/web3.js@1.87.6
- **Score:** 10.00 (MAX!)
- **Findings:** 259
- **LLM:** Rate limited (no override)
- **Assessment:** ⚠️ **LIKELY FP** - Blockchain SDK (legitimate)
- **Why 10.00?** Blockchain patterns triggering exceptions incorrectly

### 3. ethers@6.9.2
- **Score:** 6.70
- **Findings:** 292
- **LLM:** confidence 0.90 → **OVERRIDDEN to malicious**
- **Assessment:** ⚠️ **LIKELY FP** - Blockchain SDK (legitimate)
- **Why LLM says malicious?** LLM prompt may be too aggressive for blockchain

### 4. firebase@10.7.2
- **Score:** 9.00
- **Findings:** 29
- **LLM:** confidence 0.95 → **OVERRIDDEN to malicious**
- **Assessment:** ⚠️ **LIKELY FP** - Cloud SDK (legitimate)
- **Why LLM says malicious?** Firebase patterns look suspicious to LLM

### 5. prettier@3.1.1
- **Score:** 6.37
- **Findings:** 18
- **LLM:** confidence 0.90 → **OVERRIDDEN to malicious**
- **Assessment:** ⚠️ **LIKELY FP** - Code formatter (legitimate)
- **Why LLM says malicious?** Code generation patterns?

### 6. prisma@5.8.1
- **Score:** 10.00 (MAX!)
- **Findings:** 24
- **LLM:** Rate limited (no override)
- **Assessment:** ⚠️ **LIKELY FP** - ORM package (legitimate)
- **Why 10.00?** Same as @prisma/client

### 7. typescript@5.3.3
- **Score:** 9.00
- **Findings:** 36
- **LLM:** confidence 0.95 → **OVERRIDDEN to malicious**
- **Assessment:** ⚠️ **LIKELY FP** - Compiler (legitimate)
- **Why LLM says malicious?** Code generation, eval patterns?

### 8. viem@1.21.4
- **Score:** 5.27
- **Findings:** 370
- **LLM:** Not run (score < 6.0 tier1_threshold)
- **Assessment:** ⚠️ **LIKELY FP** - Blockchain SDK (legitimate)
- **Why flagged?** Score 5.27 > 5.0 threshold

### 9. webpack@5.89.0
- **Score:** 5.83
- **Findings:** 68
- **LLM:** Not run (score < 6.0 tier1_threshold)
- **Assessment:** ⚠️ **LIKELY FP** - Build tool (legitimate)
- **Why flagged?** Score 5.83 > 5.0 threshold

---

## Critical Issues Identified

### Issue 1: LLM Overriding Too Aggressively

**Pattern:** LLM confidence 0.90-0.95 flagging legitimate packages

**Affected:**
- ethers (blockchain SDK)
- firebase (cloud SDK)
- prettier (code formatter)
- typescript (compiler)

**Root Cause:** LLM prompts may be too aggressive, flagging legitimate patterns as malicious.

**Fix Needed:**
- Adjust LLM prompts to recognize legitimate blockchain/cloud SDKs
- Add package reputation to LLM context
- Raise LLM override confidence threshold from 0.75 to 0.95

---

### Issue 2: Scoring Exceptions Not Working

**Pattern:** @prisma/client and prisma scoring 10.00 (maximum)

**Root Cause:** Scoring exceptions for known patterns not working correctly.

**Fix Needed:**
- Debug why @prisma packages score 10.00
- Check if reputation multiplier is being applied
- Verify category caps are working

---

### Issue 3: Rate Limiting Still Occurring

**Pattern:** @prisma/client, @solana/web3.js hit rate limits

**Root Cause:** Even with tier1_threshold = 6.0, high-score packages still trigger rate limits.

**Fix Needed:**
- This is expected behavior for high-score packages
- Consider raising tier1_threshold to 7.0 for Phase A

---

### Issue 4: Low Threshold (5.0) Too Aggressive

**Pattern:** viem (5.27) and webpack (5.83) flagged due to 5.0 threshold

**Root Cause:** Wave 11 config uses malicious_threshold = 5.0 for sensitivity.

**Fix Needed:**
- For Phase A, use malicious_threshold = 7.0 or 8.0
- Phase A config already has correct threshold (7.0)

---

## Recommended Actions BEFORE Phase A

### Immediate (Today)

1. **Investigate @prisma/client findings**
   ```bash
   # Extract and examine @prisma/client tarball
   ./target/release/glassware scan-npm @prisma/client@5.8.1 --verbose 2>&1 | grep -E "Finding|category:"
   ```

2. **Check LLM prompt aggressiveness**
   - Review LLM prompts in glassware/src/llm.rs
   - Add legitimate package patterns to prompts

3. **Raise LLM override confidence threshold**
   ```rust
   // In wave.rs or scanner.rs
   if verdict.confidence >= 0.95 {  // Was 0.75
       // Override
   }
   ```

4. **Verify Phase A config thresholds**
   ```toml
   # campaigns/phase-a-controlled/config.toml
   [settings.scoring]
   malicious_threshold = 7.0  # NOT 5.0!
   ```

### Before Phase A Re-Run

- [ ] Understand why @prisma scores 10.00
- [ ] Understand why LLM flags typescript/firebase/prettier
- [ ] Adjust LLM prompts or confidence threshold
- [ ] Verify Phase A config has correct thresholds
- [ ] Consider raising tier1_threshold to 7.0

---

## Phase A Config Check

**Current Phase A config:**
```toml
[settings.scoring]
malicious_threshold = 7.0  # ✅ Correct
suspicious_threshold = 3.5

[settings.llm]
tier1_threshold = 6.0
tier2_threshold = 7.0
```

**This is CORRECT** - Phase A uses 7.0 threshold, not 5.0.

**Expected Phase A Results:**
- viem (5.27) → NOT flagged (< 7.0)
- webpack (5.83) → NOT flagged (< 7.0)
- ethers (6.70) → NOT flagged (< 7.0)
- prettier (6.37) → NOT flagged (< 7.0)
- typescript (9.00) → FLAGGED (≥ 7.0) - needs investigation
- firebase (9.00) → FLAGGED (≥ 7.0) - needs investigation
- @prisma/* (10.00) → FLAGGED (≥ 7.0) - needs investigation

---

## Conclusion

**DO NOT proceed to Phase A re-run yet!**

We need to:
1. Investigate WHY legitimate packages score so high
2. Fix LLM prompt aggressiveness
3. Verify scoring exceptions work correctly
4. THEN run Phase A

**Estimated time to fix:** 2-4 hours of investigation + tuning

---

**Analysis By:** Glassworks Development Agent  
**Date:** March 25, 2026  
**Status:** ⚠️ **BLOCKED - Investigation Required**
