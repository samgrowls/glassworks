# Wave22 Investigation - REVISED FINDINGS

**Date:** 2026-03-27
**Status:** CRITICAL RE-EVALUATION

---

## Critical Findings

### You Were Right To Question The Initial Analysis

**Key Issues Identified:**

1. ❌ **LLM Analysis Was DISABLED** - No AI verification was performed
2. ❌ **Initial grep missed invisible chars** - Python analysis found what grep missed
3. ❌ **High scores without GlassWorm patterns** - Scoring system may be misconfigured for build tools

---

## Revised Package Analysis

### 1. systemjs-plugin-babel@0.0.25

**Threat Score:** 10.00
**Invisible Unicode:** ✅ FOUND (4 ZWNJ characters U+200C)
**Location:** `systemjs-babel-browser.js` (bundled build file)
**Context:** Unicode identifier ranges for i18n support

**Analysis:**
```
Found ZWNJ at position 289940 in 748KB file
Context: Unicode character ranges for international identifier support
Pattern: ᴀ-᷵᷼-ἕἘ-Ἕ...ἰ-ΐῖ-Ί... (legitimate Unicode ranges)
```

**Verdict:** 🟡 **REQUIRES DEEPER ANALYSIS**

The ZWNJ characters are embedded in what appears to be legitimate Unicode identifier support code (Babel needs to support international variable names). However:
- 4 ZWNJ chars in a 748KB bundled file = very low density
- Context suggests legitimate i18n support
- No decoder function found
- No C2 patterns found

**Could this be GlassWorm?** Unlikely but not impossible. The invisible chars are in a bundled file which makes analysis difficult.

**Recommendation:** 
- Compare with official SystemJS repository
- Check if ZWNJ chars are in source or only in build
- If only in build → likely build artifact
- If in source → investigate further

---

### 2. babel-plugin-angularjs-annotate@0.10.0

**Threat Score:** 9.00
**Invisible Unicode:** ❌ **NONE FOUND**
**Decoder:** ❌ None
**C2 Patterns:** ❌ None

**Verdict:** 🟢 **LIKELY FALSE POSITIVE**

**CRITICAL:** This package has **NO invisible Unicode characters** but scored 9.00. This indicates:

1. **Scoring system issue** - Package scored high without GlassWorm patterns
2. **Wrong detectors firing** - Obfuscation/encrypted payload detectors triggered on legitimate Babel plugin code
3. **Build tool patterns misidentified** - Babel visitor patterns look like code injection

**This should NOT have scored 9.00** under our GlassWorm-focused detection system.

---

### 3. vite-plugin-vue-devtools@8.1.1

**Threat Score:** 6.78 (below 7.0 threshold)
**Status:** ✅ **CORRECTLY NOT FLAGGED**

System worked as intended - borderline score did NOT trigger malicious flag.

---

## Root Cause Analysis

### Why Did These Score So High?

**Without LLM analysis and without proper detector breakdown, we can't be certain. But likely causes:**

1. **Obfuscation Detector** - Triggered on minified/bundled code
2. **Encrypted Payload Detector** - Triggered on high-entropy strings in bundles
3. **Glassware Pattern Detector** - May have false matches on build tool patterns

### The Real Problem

**Our detection system is designed for GlassWorm attacks which require:**
- Invisible Unicode characters ✅ (systemjs only)
- Decoder function ❌ (neither package)
- C2 communication ❌ (neither package)

**But these packages scored 9.00-10.00 without the full pattern!**

This suggests:
- Individual detectors are too sensitive
- Category diversity scoring is broken for build tools
- We need better build artifact detection

---

## LLM Analysis Was Disabled

**Wave22 Config:**
```toml
[settings.llm]
tier1_enabled = false  # ❌ DISABLED
tier2_enabled = false  # ❌ DISABLED
```

**Impact:** No AI verification of findings. We should have run:
```bash
./target/debug/glassware scan-tarball evidence/wave22-investigation/systemjs-plugin-babel-0.0.25.tgz --llm
```

---

## Recommendations

### Immediate Actions

1. **DO NOT skip bundled files** - You're right, this could miss real attacks
2. **Re-scan with LLM enabled** - Get AI analysis on these packages
3. **Investigate detector sensitivity** - Why do build tools score so high?
4. **Check official repositories** - Compare with upstream source

### Investigation Steps

1. **For systemjs-plugin-babel:**
   ```bash
   # Check official repo for ZWNJ chars
   git clone https://github.com/systemjs/plugin-babel.git
   # Compare with our package
   ```

2. **For babel-plugin-angularjs-annotate:**
   ```bash
   # Check why it scored 9.00 without invisible chars
   ./target/debug/glassware scan-tarball evidence/wave22-investigation/babel-plugin-angularjs-annotate-0.10.0.tgz --llm
   ```

3. **Review detector weights:**
   - Obfuscation detector may be too sensitive for build tools
   - Need to distinguish minification from malicious obfuscation

### Long-term Improvements

1. **Require full GlassWorm pattern** for high scores:
   - Invisible chars + decoder + C2 = high score
   - Invisible chars alone = low score
   - No invisible chars = very low score (unless other strong signals)

2. **Enable LLM for high-score packages**:
   - Auto-trigger LLM analysis for score >= 7.0
   - Use LLM verdict to override detector scores

3. **Better build artifact handling**:
   - Don't skip, but weight differently
   - Check if patterns are in source or only in build output

---

## Revised Verdict

| Package | Initial | Revised | Confidence |
|---------|---------|---------|------------|
| systemjs-plugin-babel | ❌ FP | 🟡 Unclear | Medium |
| babel-plugin-angularjs-annotate | ❌ FP | ❌ FP (scoring bug) | High |
| vite-plugin-vue-devtools | ✅ Correct | ✅ Correct | High |

**Bottom Line:** You were right to question the findings. We need:
1. LLM analysis enabled
2. Better detector calibration for build tools
3. More careful investigation before declaring FP

---

**Next Steps:**
1. Re-scan suspicious packages with `--llm` flag
2. Compare systemjs-plugin-babel with official repo
3. Investigate why babel-plugin-angularjs-annotate scored 9.00 without invisible chars
4. Consider enabling auto-LLM for high-score packages

---

**Last Updated:** 2026-03-27
**Status:** UNDER RE-EVALUATION
**Analyst:** AI Agent (with valuable user skepticism!)
