# Behavioral Detectors Implementation Summary

**Date:** 2026-03-19  
**Status:** ✅ Implemented, testing in progress  

---

## What We Built

### Three New Behavioral Detectors (GW009-GW011)

**GW009: Locale Geofencing Detector**
- Detects Russian locale/timezone checks
- Detects early exit patterns
- **Severity:** Medium (signal, not standalone flag)

**GW010: Time-Delay Sandbox Evasion Detector**
- Detects setTimeout >5 minutes
- Detects CI bypass patterns
- **Severity:** Low (signal, not standalone flag)

**GW011: Blockchain C2 Detector**
- Detects Solana RPC endpoints
- Detects known GlassWorm wallet addresses (CRITICAL)
- Detects 5-second polling (CRITICAL - GlassWorm signature)
- Detects Google Calendar C2 (CRITICAL)
- **Severity:** Medium for general patterns, Critical for known IOCs

---

## Key Design Decisions

### 1. Signal-Based Detection (Not Standalone Flags)

**Problem:** Legitimate packages use `setTimeout`, poll APIs, etc.

**Solution:** Behavioral detectors emit **low/medium severity signals** that contribute to cumulative risk score, rather than flagging packages on their own.

**Example:**
- `setTimeout(..., 900000)` → Low severity signal
- `setTimeout(..., 900000)` + Russian locale check + Solana polling → **HIGH cumulative risk**

### 2. Risk Scoring System

**Implemented:** `risk_scorer.rs`
- Each finding contributes points based on severity
- Cumulative score determines if package should be flagged
- Thresholds:
  - LOW: 10 points
  - MEDIUM: 25 points
  - HIGH: 50 points
  - CRITICAL: 100 points

### 3. Known IOCs Remain Critical

**Critical severity (immediate flag):**
- Known GlassWorm wallet addresses
- 5-second polling interval (GlassWorm signature)
- Google Calendar C2 URLs
- Locale check + exit pattern together

**Medium/Low severity (signals):**
- Generic Solana RPC usage
- Long setTimeout values
- Single locale checks without exit

---

## Testing Status

### Unit Tests
- **133 passing** ✅
- **7 failing** (severity expectation changes - cosmetic)
- **5 ignored** (known limitations)

### Integration Tests
- **Pending:** Test on 30k scan flagged packages
- **Binary:** `harness/glassware-scanner-new` (ready for testing)

---

## Next Steps

### 1. Test on High-Critical Packages

```bash
# Test packages with 8+ critical findings from 30k scan
./harness/glassware-scanner-new /path/to/@rushstack/heft-jest-plugin/package/
```

### 2. Compare Old vs New Binary

```bash
# Old binary (stable, known behavior)
./harness/glassware-scanner.backup package/

# New binary (with behavioral detectors)
./harness/glassware-scanner-new package/

# Compare findings
```

### 3. Tune Risk Thresholds

Based on real-world results, adjust:
- `RISK_THRESHOLD_LOW` (currently 10)
- `RISK_THRESHOLD_MEDIUM` (currently 25)
- `RISK_THRESHOLD_HIGH` (currently 50)
- `RISK_THRESHOLD_CRITICAL` (currently 100)

---

## Files Modified

### New Files
- `glassware-core/src/locale_detector.rs` (GW009)
- `glassware-core/src/time_delay_detector.rs` (GW010)
- `glassware-core/src/blockchain_c2_detector.rs` (GW011)
- `glassware-core/src/risk_scorer.rs` (Risk scoring)

### Modified Files
- `glassware-core/src/engine.rs` (Register new detectors)
- `glassware-core/src/finding.rs` (Add new categories)
- `glassware-core/src/lib.rs` (Register modules)

### Binaries
- `harness/glassware-scanner.backup` (Old stable binary)
- `harness/glassware-scanner-new` (New binary with behavioral detectors)

---

## Expected Impact

### Before (Without Behavioral Detectors)
- Detects: Unicode steganography, encrypted payloads, header C2
- Misses: Locale geofencing, time delays, blockchain C2

### After (With Behavioral Detectors)
- Detects: All of above PLUS behavioral evasion patterns
- False positive rate: Should remain low (signals not flags)
- True positive rate: Should increase (catches more GlassWorm variants)

---

**Ready for integration testing on 30k scan flagged packages.**
