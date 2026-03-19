# Detector Tuning Log

**Date:** 2026-03-19  
**Reason:** False positive on @websolutespa/llm-ernestomeda  

---

## Tuning Attempt 1

### Change 1: setInterval Pattern
**Attempt:** Only flag setInterval if it contains `fetch()` (network calls)  
**Result:** ❌ Reverted - Too restrictive, misses legitimate C2 patterns

### Change 2: Intl.DateTimeFormat Pattern
**Attempt:** Remove Intl patterns entirely  
**Result:** ✅ Partial success - Reduces i18n FPs but loses some detection capability

### Change 3: i18n File Skip
**Attempt:** Skip files with i18n patterns  
**Result:** ❌ Reverted - Too aggressive, could miss malicious i18n files

---

## Final Decision

**Keep original detectors UNCHANGED.**

**Rationale:**
1. Behavioral detectors are working as designed (signals, not flags)
2. False positive was caught and reviewed - system working correctly
3. Over-tuning risks missing real threats
4. Risk scoring + human review is the correct approach

---

## Recommended Approach

**Instead of tuning detectors:**

1. ✅ **Document known FPs** - Create allowlist for reviewed legitimate packages
2. ✅ **Improve risk scoring** - Adjust thresholds based on real data
3. ✅ **Human review workflow** - Make review faster/easier
4. ✅ **Context analysis** - Add network request analysis for setInterval

---

**Status:** Detectors unchanged, system working as designed  
**Next:** Continue scanning remaining flagged packages
