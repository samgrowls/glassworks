# Evidence Detection Status

**Date:** March 23, 2026
**Status:** ⚠️ PARTIAL - Whitelist Fixed, Scoring Needs Tuning

---

## Critical Fixes Completed

### ✅ Whitelist Matching Fixed

**Problem:** The whitelist `.contains()` check was too broad
- "intl" matched "react-native-international-phone-number"
- This accidentally whitelisted confirmed malicious packages

**Fix:** Implemented precise matching rules:
- Exact match: "lodash" matches "lodash" only
- Wildcard: "@metamask/*" matches "@metamask/anything"
- Prefix with dash: "webpack-" matches "webpack-cli"
- Prefix with slash: "@babel/" matches "@babel/core"

**Result:** Malicious packages are no longer accidentally whitelisted

### ✅ Malicious Packages Removed from Whitelist

Removed from all campaign configs:
- react-native-country-select (confirmed malicious by Koi Security)
- react-native-international-phone-number (confirmed malicious)

---

## Current Detection Status

### Evidence Archive Testing

| Package | Findings | Avg Score | Status | Notes |
|---------|----------|-----------|--------|-------|
| react-native-country-select-0.3.91 | 11 | 4.00 | ⚠️ Below threshold | Has obfuscation + blockchain C2 |
| react-native-intl-phone-number-0.11.8 | 10 | 2.00 | ⚠️ Below threshold | Has obfuscation patterns |

### Issue Identified

**The evidence packages contain:**
- Function obfuscation (renaming, string encryption)
- Blockchain RPC endpoints (Solana, Ethereum)
- Invisible Unicode characters

**Our detectors find:**
- ✅ Invisible characters (11 findings)
- ❌ Obfuscation patterns (NOT detected)
- ❌ Blockchain C2 patterns (NOT detected)

**Root Cause:**
The GlassWare pattern detector and blockchain C2 detector aren't triggering on the obfuscation style used in these samples. The scoring formula relies on category diversity, but all findings are in one category (InvisibleCharacter).

---

## Scoring Adjustments Made

### Threshold Changes
- `malicious_threshold`: 7.0 → 5.0 (more sensitive)
- `suspicious_threshold`: 3.0 → 2.0 (more sensitive)

### Weight Changes
- `category_weight`: 2.0 → 3.0 (more weight per category)
- `critical_weight`: 3.0 → 4.0 (more weight for critical)
- `high_weight`: 1.5 → 2.0 (more weight for high severity)

### Current Formula
```
score = (categories × 3.0) + (critical_hits × 4.0) + (high_hits × 2.0)
```

**For 11 InvisibleCharacter findings:**
- categories = 1 (obfuscation)
- critical_hits = 0
- high_hits = 11 × 1.0 = 11
- score = (1 × 3.0) + (0 × 4.0) + (11 × 2.0) = 25 → capped at 10.0

**Expected:** Should trigger at score 10.0 ≥ threshold 5.0 ✅

**Actual:** Score shows 4.00 - indicates config not being applied properly

---

## Next Steps

### Immediate (Before Large Scale Scan)

1. **Debug score calculation**
   - Add logging to see actual calculated score
   - Verify config is being loaded correctly
   - Check if tarball scanner uses different config

2. **Enhance GlassWare pattern detection**
   - The evidence has obfuscation patterns we're not catching
   - Need to detect: function renaming, string encryption, control flow flattening

3. **Enhance blockchain C2 detection**
   - Evidence has RPC endpoints we're not catching
   - Need to detect: blockchain RPC URLs, wallet address patterns

### Short-Term

1. **Run 5000 package scan** with current detection
   - Even with conservative scoring, we'll catch high-confidence malicious packages
   - Document false negatives for later analysis

2. **Collect more evidence samples**
   - Reach out to Koi Security for more samples
   - Test against known malicious packages from npm

### Medium-Term

1. **Improve obfuscation detection**
   - Add entropy-based detection
   - Add control flow graph analysis
   - Add string decryption pattern detection

2. **Improve blockchain C2 detection**
   - Add RPC endpoint URL patterns
   - Add wallet address extraction
   - Add transaction pattern analysis

---

## Known Malicious Packages (For Testing)

These should ALWAYS trigger as malicious:

| Package | Version | Source | Status |
|---------|---------|--------|--------|
| react-native-country-select | 0.3.1 | Koi Security | ⚠️ Needs detection fix |
| react-native-international-phone-number | 0.10.7 | Koi Security | ⚠️ Needs detection fix |

---

## False Positives (Correctly Whitelisted)

These should NEVER trigger:

| Package | Reason | Whitelist Category |
|---------|--------|-------------------|
| moment, moment-timezone | i18n data | packages |
| lodash, underscore | utility library | packages |
| webpack, vite, rollup | build tools | build_tools |
| firebase, firebase-admin | cloud SDK | crypto_packages |
| prisma, @prisma/client | ORM tool | build_tools |
| node-forge | crypto library | crypto_packages |
| crypto-js | crypto library | crypto_packages |

---

## Testing Commands

```bash
# Test evidence package (should be malicious)
./target/release/glassware scan-tarball evidence-archive/evidence/react-native-country-select-0.3.91.tgz

# Test whitelist (should NOT be flagged)
./target/release/glassware scan-npm lodash@4.17.21

# Test with LLM analysis
./target/release/glassware scan-npm react-native-country-select@0.3.1 --llm
```

---

**Last Updated:** March 23, 2026
**Priority:** HIGH - Fix evidence detection before large-scale scans
