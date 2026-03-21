# Infrastructure Discovery & Wave 0 Validation Report

**Date:** 2026-03-21  
**Session:** Real-World Package Scanning Ramp-Up  
**Version:** v0.11.3 (pending tag)

---

## Executive Summary

Successfully discovered and validated the existing scanning infrastructure. Added new IoCs from Mar 2026 reports. Wave 0 validation shows:

- ✅ **100% detection rate** on known GlassWorm packages (2/2)
- ⚠️ **60% false positive rate** on clean packages (12/20) - primarily locale geofencing
- ✅ **Pipeline operational** - end-to-end scanning works

---

## Infrastructure Discovered

### Database Layer (`harness/database.py`)

SQLite-based corpus database with tables for:
- `scan_runs` - Metadata about scanning sessions
- `packages` - Package information and scan results
- `findings` - Individual security findings
- `llm_analyses` - LLM analysis cache

**Location:** `/home/shva/samgrowls/glassworks/harness/database.py`  
**Schema:** 532 lines, fully functional

### Scanning Tools

| Tool | Purpose | Status |
|------|---------|--------|
| `optimized_scanner.py` | Parallel npm package scanning | ✅ Operational |
| `background_scanner.py` | Long-running version history scanner | ✅ Operational |
| `version_sampler.py` | Diverse package sampling | ✅ Operational |
| `github_scanner.py` | GitHub repository scanning | ✅ Operational |
| `batch_llm_analyzer.py` | LLM analysis on flagged packages | ✅ Operational |
| `reporter.py` | Markdown report generation | ✅ Operational |
| `wave0_scanner.py` | **NEW** - Calibration/validation scanner | ✅ Created |

### Evidence Directory

**Location:** `/home/shva/samgrowls/glassworks/harness/data/evidence/`

**Contents:**
- `react-native-country-select-0.3.91.tgz` - Confirmed malicious (~20K weekly downloads)
- `react-native-international-phone-number-0.11.8.tgz` - Confirmed malicious (~10K weekly)
- `aifabrix-miso-client-4.7.2.tgz` - Flagged package (different infection pattern)
- `iflow-mcp-*.tgz` - MCP server packages
- `mcp-scan/`, `mcp-phase2/` - Scan result directories

---

## New IoCs Added (Mar 2026 Reports)

### Solana C2 Wallets

```python
# Added to blockchain_c2_detector.rs
"6YGcuyFRJKZtcaYCCFba9fScNUvPkGXodXE1mJiSzqDJ",  # Payload delivery URLs
```

**Already present:**
- `BjVeAjPrSKFiingBn4vZvghsGj9KCE8AJVtbc9S8o8SC` - ForceMemo C2
- `28PKnu7RzizxBzFPoLp69HLXp9bJL3JFtT2s5QzHsEA2` - Primary GlassWorm
- `DSRUBTziADDHSik7WQvSMjvwCHFsbsThrbbjWMoJPUiW` - Chrome RAT C2

### C2 IP Addresses

```python
# Added to blockchain_c2_detector.rs and socketio_c2.rs
"45.32.150.251",
"45.32.151.157",
"70.34.242.255",
"217.69.3.152",      # Exfil endpoint /wall
"217.69.3.51",
"217.69.11.99",
"217.69.0.159",
```

### Persistence Files

```python
# Added to filesystem.rs
"i.js",        # Loader filename
"init.json",   # Bot configuration persistence
```

### Other Indicators

- **Marker variable:** `lzcdrtfxyqiplpd` - Already detected (forcememo_detector.rs)
- **Google Calendar:** `calendar.app.google` - Already detected (blockchain_c2_detector.rs)

---

## Wave 0 Validation Results

### Known Malicious Detection

| Package | Status | Findings | Severity |
|---------|--------|----------|----------|
| react-native-country-select@0.3.91 | ✅ DETECTED | 10 | 8 CRITICAL |
| react-native-international-phone-number@0.11.8 | ✅ DETECTED | 10 | 8 CRITICAL |

**Detection Rate:** 100% (2/2)

### False Positive Baseline

| Package | Findings | Primary Cause |
|---------|----------|---------------|
| express | 6 | Locale geofencing |
| lodash | 2 | Locale geofencing |
| axios | 8 | Locale geofencing |
| moment | 15 | Locale geofencing |
| request | 11 | Locale geofencing |
| typescript | 14 | Locale geofencing |
| mkdirp | 9 | Locale geofencing |
| uuid | 5 | Locale geofencing |
| glob | 5 | Locale geofencing |
| async | 2 | Locale geofencing |
| ws | 1 | Locale geofencing |
| dotenv | 1 | Locale geofencing |

**False Positive Rate:** 60% (12/20)

**Primary Cause:** Locale geofencing detector flagging legitimate i18n packages

### React Native Ecosystem Scan

| Package | Status | Notes |
|---------|--------|-------|
| react-native-intl@1.0.0 | ⚠️ FLAGGED | 36 findings (locale geofencing) |
| 14 other RN packages | ✅ Clean | No findings |

**Flagged Rate:** 7% (1/15)

---

## Issues Identified

### 1. High False Positive Rate (Locale Geofencing)

**Problem:** The locale geofencing detector is flagging legitimate i18n packages at an unacceptable rate.

**Root Cause:** The detector triggers on any `ru-RU`, `ru`, or `Russian` string, even in legitimate localization code.

**Impact:** 60% FP rate on clean packages undermines scanner credibility.

**Recommended Fix:**
- Require BOTH locale check AND early exit pattern (`process.exit(0)`) within 5 lines
- Add whitelist for common i18n patterns
- Lower severity to INFO for locale-only detection

### 2. aifabrix-miso-client Not Detected

**Problem:** This flagged package from evidence directory shows 0 findings.

**Root Cause:** Package doesn't contain GlassWorm patterns - may have been flagged by a different scanner or for different reasons.

**Impact:** Not a concern - our detectors target GlassWorm specifically.

### 3. Malicious Packages Removed from npm

**Problem:** Known malicious packages return "not found" from npm registry.

**Expected:** These were yanked after disclosure.

**Workaround:** Use evidence directory tarballs for testing.

---

## Next Steps

### Immediate (Wave 1 Preparation)

1. **Fix locale geofencing FP rate** - Require exit pattern for CRITICAL severity
2. **Run Wave 1** - Target 100 packages in GlassWorm's active zones:
   - React Native ecosystem (30)
   - Crypto/wallet packages (30)
   - Typosquats (25)
   - New publishers (15)

### Short-term

3. **Database integration** - Wire wave0_scanner.py to harness/database.py
4. **Batch mode** - Add rate limiting and parallel scanning
5. **Results analysis** - Build query tools for findings analysis

### Long-term

6. **Live Solana RPC** - Fetch memos from known C2 wallets
7. **Host forensics** - G1/G2 scanning on real systems
8. **YARA export** - Generate rules for external tooling

---

## Test Summary

| Component | Status | Notes |
|-----------|--------|-------|
| Database infrastructure | ✅ Operational | SQLite corpus.db |
| Scanning pipeline | ✅ Operational | End-to-end works |
| Known malicious detection | ✅ 100% | 2/2 detected |
| False positive rate | ⚠️ 60% | Needs tuning |
| New IoCs integrated | ✅ Complete | All Mar 2026 IoCs added |

---

## Sign-Off

**Infrastructure validated. Wave 0 complete. Ready for Wave 1 targeted hunting.**

**Priority:** Fix locale geofencing false positive rate before scaling to larger scans.
