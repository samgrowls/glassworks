# Wave22 Investigation - Final Assessment

**Date:** 2026-03-27
**Status:** MANUAL REVIEW REQUIRED - NOT FALSE POSITIVES

---

## Security-First Assessment

**Key Principle:** High scores on legitimate packages = **MANUAL REVIEW**, NOT detector adjustment.

We should NOT:
- ❌ Overfit detectors to avoid specific findings
- ❌ Auto-declare false positives
- ❌ Skip bundled files (could miss real attacks)
- ❌ Weaken detection to reduce manual review workload

We SHOULD:
- ✅ Flag suspicious packages for manual review
- ✅ Maintain detector sensitivity
- ✅ Accept some manual review as necessary
- ✅ Prefer false positives over false negatives

---

## Package Status

### 1. systemjs-plugin-babel@0.0.25

**Threat Score:** 10.00 (MALICIOUS)
**Invisible Unicode:** ✅ FOUND (4 ZWNJ characters)
**Status:** 🟡 **REQUIRES MANUAL REVIEW**

**Findings:**
- 4 ZWNJ (U+200C) characters in 748KB bundled file
- Context: Unicode identifier support (legitimate Babel feature)
- No decoder function detected
- No C2 patterns detected

**Assessment:** Likely legitimate, but requires manual verification:
- Compare with official SystemJS GitHub repository
- Check if ZWNJ chars exist in upstream source
- Verify package hasn't been compromised

**Action:** Manual review required. DO NOT auto-clear.

---

### 2. babel-plugin-angularjs-annotate@0.10.0

**Threat Score:** 9.00 (MALICIOUS)
**Invisible Unicode:** ❌ NONE FOUND
**Status:** 🟡 **REQUIRES MANUAL REVIEW**

**Findings:**
- No invisible Unicode characters
- No decoder function
- No C2 patterns
- High score likely from obfuscation/encrypted payload detectors

**Assessment:** High score without GlassWorm patterns suggests:
- Possible detector sensitivity issue for build tools
- OR legitimate suspicious patterns we haven't categorized
- Requires manual code review

**Action:** Manual review required. Investigate why score is high.

---

### 3. vite-plugin-vue-devtools@8.1.1

**Threat Score:** 6.78 (BORDERLINE)
**Status:** ✅ **CORRECTLY NOT FLAGGED**

**Assessment:** System working correctly - below 7.0 threshold, not flagged as malicious.

---

## Wave22 Summary

| Metric | Value |
|--------|-------|
| Packages Scanned | 987 (Wave 22b) |
| Flagged | 65 (6.6%) |
| Malicious (≥7.0) | 3 (0.3%) |
| Requires Manual Review | 3 packages |

**Detection Rate:** Working as intended
**False Positive Rate:** Unknown (requires manual review)

---

## Process Improvements

### For Wave23-24

1. **Manual Review Workflow:**
   - All packages with score ≥ 7.0 flagged for manual review
   - No auto-clearing of findings
   - Document review decisions

2. **LLM Analysis:**
   - Tier 1 (Cerebras): DISABLED (rate limiting issues)
   - Tier 2 (Nvidia): Available for deep analysis of confirmed malicious packages
   - Use for case studies, not bulk screening

3. **Detector Calibration:**
   - NO changes to reduce FPs
   - Maintain current sensitivity
   - Accept manual review as necessary cost

4. **Evidence Preservation:**
   - All flagged packages saved to `evidence/wave22-investigation/`
   - Maintain chain of custody
   - Document review findings

---

## Manual Review Checklist

For each flagged package:

- [ ] Download from npm registry
- [ ] Compare with official GitHub repository
- [ ] Check package metadata (author, downloads, age)
- [ ] Review code for:
  - [ ] Invisible Unicode characters
  - [ ] Decoder functions
  - [ ] C2 communication patterns
  - [ ] Obfuscation (malicious vs legitimate)
- [ ] Check for typosquatting
- [ ] Review npm dependencies
- [ ] Document findings

---

## Wave22 Packages Requiring Review

1. **systemjs-plugin-babel@0.0.25**
   - Score: 10.00
   - Priority: HIGH (max score)
   - Action: Compare with https://github.com/systemjs/plugin-babel

2. **babel-plugin-angularjs-annotate@0.10.0**
   - Score: 9.00
   - Priority: HIGH (no invisible chars, high score = investigate)
   - Action: Review code, check for typosquat of official babel-plugin-angularjs-annotate

3. **vite-plugin-vue-devtools@8.1.1**
   - Score: 6.78 (not flagged)
   - Priority: LOW (below threshold)
   - Action: None required

---

## Moving Forward

**Wave23-24 Approach:**
- Same detection sensitivity
- Manual review for all score ≥ 7.0
- No detector adjustments to reduce FPs
- Document all review decisions
- Build knowledge base of legitimate vs malicious patterns

**Long-term:**
- Build whitelist of REVIEWED legitimate packages (not auto-whitelist)
- Improve manual review efficiency
- Case studies on confirmed malicious packages (with LLM Tier 2)

---

**Last Updated:** 2026-03-27
**Status:** AWAITING MANUAL REVIEW
**Principle:** Better 100 false positives than 1 missed attack
