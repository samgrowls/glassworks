# 🚨 CRITICAL BUG - Wave 10 Investigation

**Date:** 2026-03-25 18:44 UTC  
**Status:** CRITICAL - DO NOT PUSH TAG  
**Severity:** BLOCKER

---

## Problem Summary

**Wave 10 Results (INITIAL - with Tier 2 enabled):**
- **Clean Baseline:** 11/111 packages flagged as malicious (10% FP!) ❌
- **Flagged packages:** typescript, prettier, @solana/web3.js, firebase, ethers, mongodb, mysql2, etc.
- **Threat scores:** 5.0 - 10.0 (absurdly high for legitimate packages)

**Root Cause Found:**
1. **Tier 2 LLM enabled** - Causing hangs/timeouts
2. **Scanner hanging** - Even without LLM, `scan-npm typescript@5.3.3` hangs indefinitely
3. **Detector engine bug** - Something in the detector loop is causing infinite execution

---

## Evidence

### Wave 10 Initial Results (WRONG - Tier 2 was enabled)

```
Wave 'Clean Baseline - Top 100 npm' completed: 111 scanned, 49 flagged, 11 malicious

Malicious packages flagged:
- typescript@5.3.3 (score: 9.00) ❌
- prettier@3.1.1 (score: 6.37) ❌
- @solana/web3.js@1.87.6 (score: 10.00) ❌
- firebase@10.7.2 (score: 10.00) ❌
- ethers@6.9.2 (score: 8.50) ❌
- mongodb@6.3.0 (score: 9.00) ❌
- mysql2@3.7.1 (score: 9.00) ❌
```

**This is CATASTROPHICALLY WRONG** - these are all legitimate, popular packages!

### Scanner Hang Investigation

```bash
# This command HANGS indefinitely:
./target/release/glassware scan-npm typescript@5.3.3

# Output:
INFO Downloaded typescript@5.3.3
INFO Scanning package: typescript@5.3.3
[THEN NOTHING - hangs forever]
```

**Hypothesis:** One of the detectors is entering an infinite loop on certain package structures.

---

## Actions Taken

### 1. Disabled Tier 2 LLM
**File:** `campaigns/wave10-1000plus.toml`

```toml
[settings.llm]
tier1_enabled = false
tier2_enabled = false  # DISABLED - was causing hangs
```

### 2. Identified Scanner Hang
- Scan hangs even without LLM
- Problem is in detector engine itself
- Need to identify which detector is looping

---

## Next Steps (CRITICAL)

### Step 1: Identify Hanging Detector

```bash
# Test individual detectors by scanning simple packages
./target/release/glassware scan-npm express@4.19.2  # Does this hang?
./target/release/glassware scan-npm lodash@4.17.21  # Does this hang?

# If simple packages work, test complex ones
./target/release/glassware scan-npm typescript@5.3.3  # Hangs
./target/release/glassware scan-npm @angular/core@17.1.0  # Test this too
```

### Step 2: Check Recent Changes

**What changed between v0.41.0 and now?**
- Autoresearch FP fix (file path change)
- Whitelist removal
- Scoring system changes

**Likely culprit:** Scoring system exception logic or detector infinite loop

### Step 3: Fix Detector/Scoring Bug

**Suspects:**
1. **Scanner.rs scoring loop** - Exception handling for GlassWorm patterns
2. **BlockchainC2 detector** - Pattern matching on large files
3. **GlasswarePattern detector** - Encoding/decoder pattern matching

### Step 4: Re-run Wave 10

After fix:
```bash
./target/release/glassware campaign run campaigns/wave10-1000plus.toml
# Expected: 0 malicious in clean baseline
```

---

## DO NOT PUSH TAG

**Current state is BROKEN:**
- 10% FP rate on legitimate packages
- Scanner hangs on certain packages
- Detector engine bug

**Must fix before any release.**

---

## Hypothesis: Scoring System Exception Bug

Looking at `scanner.rs::calculate_threat_score()`:

```rust
// Exception logic that might be looping
for finding in findings {
    // Check for GlassWorm patterns
    if finding.category == DetectionCategory::BlockchainC2
        && finding.severity == Severity::Critical
        && finding.description.contains("polling") {
        has_glassworm_c2_polling = true;
    }
    // ... more checks
}

// Apply exceptions
if has_glassworm_c2_polling {
    score = score.max(9.0);  // THIS MIGHT BE TRIGGERING WRONG
    return score.min(10.0);
}
```

**Problem:** The "GlassWorm C2 polling" detection might be too broad, triggering on legitimate blockchain API usage.

---

## Immediate Action Required

1. **DO NOT PUSH TAG** - Current state is broken
2. **Identify hanging detector** - Test packages individually
3. **Fix scoring logic** - Exception triggers are too broad
4. **Re-test Wave 10** - Verify 0% FP on clean baseline
5. **THEN consider tag**

---

**Status:** INVESTIGATING  
**Priority:** CRITICAL  
**Blocked:** Yes - cannot proceed until fixed
