# Medium Waves Hunting Campaign - Findings & Tuning Strategy

**Date:** 2026-03-27
**Campaign:** Wave18-21 (React Native, Crypto/Web3, i18n, UI Components)
**Status:** IN PROGRESS

---

## Wave18 Results: React Native Hunt (263 packages)

### Summary
- **Packages scanned:** 263
- **Packages flagged:** 40 (15.2%)
- **Malicious (score >= 7.0):** 0
- **Borderline (score 5.0-7.0):** 3

### Borderline Packages Analysis

#### 1. country-select-js@2.1.0 (Score: 6.83)
**Findings:**
- BidirectionalOverride: 78
- InvisibleCharacter: 1

**Analysis:**
- High number of bidi overrides (78) is suspicious
- Package name similar to original GlassWorm (react-native-country-select)
- **RECOMMENDATION:** Manual review required - could be GlassWorm variant

#### 2. libphonenumber-js@1.12.40 (Score: 6.84)
**Findings:**
- BidirectionalOverride: 16
- InvisibleCharacter: 16

**Analysis:**
- Popular phone number parsing library (47M weekly downloads)
- Bidi chars could be legitimate for international phone formatting
- **RECOMMENDATION:** Likely FP but worth monitoring

#### 3. rn-country-code-picker-modal@1.2.6 (Score: 5.00)
**Findings:** Unknown

**Analysis:**
- Country code picker - similar category to original GlassWorm
- **RECOMMENDATION:** Manual review

---

## Tuning Strategy

### Current Approach: NO WHITELISTING

We maintain detection based on code patterns, not package popularity.

### Observations from Wave18

1. **Bidi-heavy packages in React Native ecosystem**
   - Country/phone input packages have legitimate Unicode for international support
   - But this is ALSO perfect cover for GlassWorm attacks
   - **Action:** Keep detection, manual review for borderline cases

2. **Score threshold working correctly**
   - Packages scoring 5.0-6.84 NOT flagged as malicious (threshold 7.0)
   - Tier 1 signal requirement preventing FPs
   - **Action:** Maintain current threshold

3. **High-risk categories identified**
   - Country/phone selectors (original GlassWorm category)
   - Auth/biometric packages
   - **Action:** Continue focused hunting in these categories

---

## Wave19-21 Status

| Wave | Category | Target | Status |
|------|----------|--------|--------|
| Wave18 | React Native | 263 | ✅ Complete |
| Wave19 | Crypto/Web3 | 300 | 🔄 Running |
| Wave20 | i18n/Translation | 200 | ⏳ Pending |
| Wave21 | UI Components | 300 | ⏳ Pending |

---

## Next Steps

1. **Manual review of country-select-js**
   - Download and inspect source code
   - Check for decoder patterns
   - Compare with original GlassWorm signatures

2. **Continue Wave19-21**
   - Let campaigns complete
   - Analyze findings

3. **Evidence expansion**
   - If country-select-js confirmed as GlassWorm, add to evidence set
   - Create synthetic variants for testing

---

## Key Insight

**The detector is working correctly:**
- Borderline packages (5.0-6.84) are NOT flagged as malicious
- This allows human review of suspicious packages without auto-blocking
- Tier 1 signal requirement prevents FPs while catching real attacks

**No tuning needed at this time.** Continue hunting.
