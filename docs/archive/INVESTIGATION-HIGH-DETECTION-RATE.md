# Investigation: High Malicious Detection Rate

**Date:** 2026-03-24
**Status:** 🔴 CRITICAL - 10% detection rate is unsustainable
**Tag:** v0.28.1-investigation

---

## Problem Statement

**Current Detection Rate:** ~10% (48-56 malicious out of 479 packages in Wave 9)

**Expected Detection Rate:** <1% (real GlassWare attacks are rare)

**Conclusion:** Our detectors are FAR TOO SENSITIVE, causing massive false positives.

---

## Root Cause Analysis

### Issue 1: LLM Override Inconsistent in Campaign Mode

**Symptom:**
- Direct scan: `@react-native-firebase/app` → LLM `malicious=false, confidence=0.35`
- Campaign scan: Same package → LLM `malicious=true, confidence=0.95`

**Cause:**
- Each wave creates NEW WaveExecutor with separate LLM analyzer
- LLM API responses vary (rate limiting, non-determinism, or different findings)
- Campaign mode LLM override is UNRELIABLE

**Action:** Disabled LLM override in campaign mode (commit pending)

**Note:** LLM override still works in CLI mode (`scan-npm --llm`)

---

### Issue 2: Detectors Flagging Legitimate Packages

**Evidence from Wave 9:**

| Package Category | Examples | Why Flagged | Should Flag? |
|-----------------|----------|-------------|--------------|
| **Web Frameworks** | @angular/core, react-native, vue | Invisible chars in i18n data, eval in minified code | ❌ NO |
| **i18n Libraries** | @formatjs/*, cldr-*, intl | U+FFFD replacement chars in locale data | ❌ NO |
| **Crypto Libraries** | @ethersproject/*, ethereumjs-wallet | Blockchain API calls (legitimate) | ❌ NO |
| **Dev Tools** | graphql, dotenv, protractor | Various patterns | ❌ NO |
| **UI Libraries** | ant-design-vue, vuetify, element-plus | Minified code patterns | ❌ NO |
| **Cloud SDKs** | @azure/msal-browser, firebase | Auth/crypto APIs (legitimate) | ❌ NO |

**Total:** 48-56 packages flagged, almost all FALSE POSITIVES

---

### Issue 3: Specific Detector Problems

#### InvisibleCharacter Detector
**Problem:** Flagging U+FFFD (replacement character) in i18n/locale data

**Affected:**
- All @formatjs/* packages
- cldr-* packages
- moment-timezone, date-fns-tz
- Any package with locale data

**Fix Needed:**
- Skip U+FFFD in *.d.ts, *.min.js, *.json files
- Skip files in /locale/, /i18n/, /lang/ directories
- Better heuristics for legitimate Unicode usage

#### GlasswarePattern (eval_pattern) Detector
**Problem:** Flagging minified/bundled code

**Affected:**
- All Angular packages (bundled)
- React Native packages
- Any package with build artifacts

**Fix Needed:**
- Check for minification markers (short variable names, no whitespace)
- Skip *.min.js files
- Higher confidence threshold for eval patterns

#### BlockchainC2 Detector
**Problem:** Flagging legitimate blockchain API usage

**Affected:**
- @ethersproject/* packages
- ethereumjs-* packages
- web3, viem, wagmi

**Fix Needed:**
- This detector should only flag SUSPICIOUS blockchain usage
- Legitimate crypto libraries should be excluded
- Better pattern matching for C2 vs normal API calls

#### EncryptedPayload Detector
**Problem:** Flagging normal encryption/encoding

**Affected:**
- dotenv (config encoding)
- Crypto libraries
- Any package with base64/hex encoding

**Fix Needed:**
- Better heuristics for encrypted vs encoded data
- Check for decryption + execution chain
- Skip known encoding patterns

---

## LLM Integration Status

### Working ✅
- CLI mode (`scan-npm --llm`)
- LLM verdict override in orchestrator.rs
- Confidence-based logic (0.25/0.75 thresholds)

### Broken ❌
- Campaign mode LLM override (inconsistent verdicts)
- Root cause: Each wave creates separate LLM analyzer
- LLM API responses vary between calls

### Action Taken
- Disabled LLM override in campaign mode
- LLM still runs for logging/analysis
- Will re-enable after fixing campaign LLM integration

---

## Recommended Fix Strategy

### Phase 1: Detector Tuning (IMMEDIATE)

**Priority: CRITICAL**

1. **InvisibleCharacter detector:**
   - Add file extension exclusions (*.d.ts, *.min.js, *.json)
   - Add directory exclusions (/locale/, /i18n/)
   - Skip U+FFFD (replacement character)

2. **GlasswarePattern detector:**
   - Add minification detection
   - Skip *.min.js files
   - Require multiple patterns for high confidence

3. **BlockchainC2 detector:**
   - Add whitelist for known crypto libraries
   - Require suspicious patterns (not just API calls)
   - Better C2 vs normal usage differentiation

4. **EncryptedPayload detector:**
   - Require decrypt + execute chain
   - Skip normal encoding (base64, hex)
   - Better entropy thresholds

### Phase 2: LLM Integration Fix (SHORT-TERM)

1. Fix campaign LLM analyzer sharing
2. Add LLM request caching
3. Handle rate limiting gracefully
4. Re-enable LLM override after testing

### Phase 3: Validation (MEDIUM-TERM)

1. Run Wave 8-11 with tuned detectors
2. Target: <5% detection rate
3. Manual review of remaining flags
4. Document FP patterns for future tuning

---

## Testing Plan

### Before Detector Tuning
- Run Wave 8 (66 packages) - expect ~10% detection
- Run Wave 9 (479 packages) - expect ~10% detection
- Document all flagged packages

### After Detector Tuning
- Run Wave 8 - target <5% detection
- Run Wave 9 - target <5% detection
- Run Wave 10 (1000+ packages) - stress test
- Run Wave 11 (evidence) - ensure still detecting real threats

### Success Criteria
- Detection rate <5% across all waves
- Evidence packages still detected (react-native-country-select, etc.)
- No new false positives introduced

---

## Key Insight

**We cannot LLM our way out of this problem.**

The LLM is correctly identifying that most flagged packages are safe (low confidence), but we're relying on it to clean up detector mess instead of fixing the detectors themselves.

**Correct approach:**
1. Tune detectors to reduce FP rate at source
2. Use LLM as safety net for edge cases
3. Whitelist only confirmed FPs (last resort)

---

## Next Actions

1. ✅ Disable campaign LLM override (done)
2. ⏳ Tune InvisibleCharacter detector (skip i18n data)
3. ⏳ Tune GlasswarePattern detector (skip minified code)
4. ⏳ Tune BlockchainC2 detector (exclude legitimate crypto)
5. ⏳ Tune EncryptedPayload detector (require decrypt+exec)
6. ⏳ Re-run Wave 8-9 with tuned detectors
7. ⏳ Validate evidence detection still works
8. ⏳ Fix campaign LLM integration
9. ⏳ Re-enable LLM override

---

**Last Updated:** 2026-03-24 11:00 UTC
**Investigator:** Qwen-Coder
**Status:** Phase 1 Ready to Start
**Priority:** CRITICAL - Blocks all production use
