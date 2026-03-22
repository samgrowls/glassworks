# Wave 4 Results — GlassWorm Hunt (Preliminary)

**Date:** 2026-03-22  
**Status:** PARTIAL COMPLETE  
**Version:** v0.11.7

---

## Summary

| Metric | Value |
|--------|-------|
| **Total Packages** | 41 scanned (of ~150 planned) |
| **Malicious Detected** | 2 |
| **Errors** | ~20 (version not found) |
| **LLM Triage** | Enabled (Cerebras) |

---

## Malicious/Suspicious Detections

| Package | Score | Analysis |
|---------|-------|----------|
| i18n-iso-countries@7.6.0 | 10.00 | ⚠️ Likely FP - locale package |
| mobx@6.0.0 | 10.00 | ⚠️ Likely FP - state management lib |

**Note:** Both detections are likely false positives:
- **i18n-iso-countries**: Contains locale/country data (similar to moment.js)
- **mobx**: Complex JavaScript patterns may trigger GlasswarePattern detector

**Action Required:** Manual review of flagged packages recommended.

---

## Errors (Version Not Found)

Many packages failed due to incorrect version specifications:

| Package | Issue |
|---------|-------|
| react-native-country-picker@2.0.0 | Version not found (available: 1.0.2) |
| react-native-phone-input@1.3.8 | Version not found (available: 1.3.3) |
| react-native-otp-inputs@1.2.0 | Version not found (available: 0.3.1, 3.3.1) |
| i18n-js@4.0.0 | Version not found (available: 4.0.0-alpha.7, 4.4.0) |
| date-format@4.0.0 | Version not found (available: 4.0.14) |

**Fix:** Use `npm view pkg versions` to get correct versions, or omit version to get latest.

---

## Detection Categories

| Category | Count |
|----------|-------|
| InvisibleCharacter | 294 |
| GlasswarePattern | 14 |
| Unknown (Socket.IO) | 92 |
| EncryptedPayload | 9 |
| BidirectionalOverride | 2 |
| Homoglyph | 3 |
| TimeDelaySandboxEvasion | 4 |
| HeaderC2 | 3 |

**Note:** High InvisibleCharacter count is expected for locale packages.

---

## Next Steps

### Immediate
1. **Manual review** of i18n-iso-countries and mobx
2. **Fix version specifications** in wave4_scan.sh
3. **Re-run Wave 4** with correct versions

### Wave 4B (Full Scan)
1. Use `npm search` to get correct package names/versions
2. Focus on React Native ecosystem packages that actually exist
3. Add evidence tarballs to scan
4. Run with `--llm` for Cerebras triage

### Package List Corrections

```bash
# React Native packages (correct versions)
react-native-country-picker@1.0.2  # Not 2.0.0
react-native-phone-input@1.3.3     # Not 1.3.8
react-native-otp-inputs@0.3.1      # Not 1.2.0
i18n-js@4.4.0                       # Not 4.0.0
date-format@4.0.14                  # Not 4.0.0
```

---

## Evidence Scan

**Status:** NOT RUN

Evidence directory scan was skipped. Run manually:

```bash
glassware-orchestrator scan-tarball \
  ~/glassworks-archive/evidence/*.tgz \
  --llm --output evidence-results.json
```

---

## Conclusions

1. **Configuration working** - moment, prettier, typescript, ethers, webpack NOT flagged
2. **Some FPs remain** - i18n-iso-countries, mobx need review
3. **Version specifications** need to be corrected for full Wave 4
4. **LLM triage working** - Cerebras integration functional

**Overall:** System is functional, needs package list refinement for full hunt.

---

**Files Generated:**
- `harness/data/wave4-results/wave4-npm-log-*.txt` - Full scan log
- `harness/WAVE4-PLAN.md` - Original plan document
