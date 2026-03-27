# Wave 14 Analysis & Detector Recalibration Plan

**Date:** 2026-03-26
**Status:** ANALYSIS COMPLETE
**Next:** Implement detector recalibration

---

## Wave 14 Results Analysis

### Campaign Summary
- **Packages scanned:** 123
- **Packages flagged:** 44 (36%)
- **Malicious packages:** 9 (7.3%)
- **Duration:** 140 seconds (~0.88 pkg/sec)

### Wave Breakdown

| Wave | Packages | Flagged | Malicious | FP Rate |
|------|----------|---------|-----------|---------|
| 14A (Known Malicious) | 0 | 0 | 0 | N/A |
| 14B (Clean Baseline) | 20 | 12 | 4 | **100% FP** ❌ |
| 14C (High Risk) | 102 | 32 | 5 | Unknown |

### False Positive Analysis

#### Clean Baseline FPs (Wave 14B)

| Package | Score | Primary Detector | Issue |
|---------|-------|-----------------|-------|
| @solana/web3.js | 10.00 | BlockchainC2 (245 findings) | Legitimate SDK flagged |
| firebase | 10.00 | BlockchainC2 + HeaderC2 | Legitimate SDK flagged |
| prettier | 6.32 | Various | Build tool FP |
| webpack | 5.62 | Various | Build tool FP |

#### High Risk FPs (Wave 14C)

| Package | Score | Likely Cause |
|---------|-------|--------------|
| casual | 8.50 | Obfuscation patterns |
| nx | 9.00 | Build tool complexity |
| pino | 5.67 | Logger complexity |
| playwright | 7.10 | Test framework patterns |
| pm2 | 9.23 | Process manager patterns |
| selenium-webdriver | 9.00 | Browser automation |
| testcafe | 9.00 | Test framework patterns |

---

## Root Cause Analysis

### Problem 1: BlockchainC2 Detector Too Aggressive

**Current Behavior:**
- Flags ANY usage of Solana/Firebase APIs
- 245 findings on @solana/web3.js (legitimate SDK)
- 30 findings on firebase (legitimate SDK)

**Why This Is Wrong:**
- BlockchainC2 should only flag KNOWN malicious indicators
- Legitimate SDK usage ≠ C2 communication
- This is like flagging ANY HTTP usage as "HTTP C2"

**Correct Behavior:**
```
IF (known_malicious_wallet OR known_malicious_IP) THEN
    flag_as_critical()
ELSE IF (getSignaturesForAddress + setInterval + 5sec_polling) THEN
    flag_as_glassworm_signature()
ELSE
    do_not_flag()  # Legitimate SDK usage
```

### Problem 2: Detectors Not Conditionally Applied

**Current Architecture:**
```
All detectors run independently
    ↓
Scores are summed
    ↓
Category diversity bonus
    ↓
Final threat score
```

**Problem:** A single aggressive detector (BlockchainC2) can push score over threshold even when package is clean.

**Proposed Architecture:**
```
Tier 1 Detectors (Primary signals)
    ↓
IF Tier 1 indicates suspicious THEN
    Run Tier 2 Detectors (Secondary confirmation)
    ↓
IF Tier 2 confirms THEN
    Run Tier 3 Detectors (Tertiary - BlockchainC2, etc.)
    ↓
Calculate final score
```

### Problem 3: No "Only If" Logic

**Current:** All detectors contribute equally
**Problem:** BlockchainC2 contributes 10.0 points for legitimate SDK usage

**Proposed:** BlockchainC2 only contributes IF other detectors already flagged

```rust
// Pseudocode
let base_score = calculate_base_score(invisible_chars, obfuscation, etc.);

// BlockchainC2 only adds score if base_score already suspicious
if base_score >= 5.0 {
    // Already suspicious, BlockchainC2 confirms
    final_score = base_score + blockchain_c2_bonus;
} else {
    // Not suspicious, ignore BlockchainC2 (likely legitimate SDK)
    final_score = base_score;
}
```

---

## Detector Recalibration Plan

### Phase 1: Fix BlockchainC2 (Priority: CRITICAL)

**File:** `glassware-core/src/blockchain_c2_detector.rs`

**Changes:**
1. Remove generic "Solana RPC detected" findings
2. Only flag:
   - Known malicious wallets (CRITICAL)
   - Known malicious IPs (CRITICAL)
   - GlassWorm C2 signature: `getSignaturesForAddress` + `setInterval` + 5-min polling (CRITICAL)
3. Add "legitimate SDK" skip logic

**Expected Impact:**
- @solana/web3.js: 10.00 → ~0.00 ✅
- firebase: 10.00 → ~2.00 (other detectors) ✅

### Phase 2: Add Conditional Detector Logic (Priority: HIGH)

**File:** `glassware/src/scanner.rs` (scoring logic)

**Changes:**
1. Add detector tiers:
   - Tier 1: InvisibleCharacter, Obfuscation, GlasswarePattern
   - Tier 2: BlockchainC2, HeaderC2, ExfilSchema
   - Tier 3: LocaleGeofencing, TimeDelay, etc.

2. Conditional scoring:
   - Tier 2 detectors only contribute if Tier 1 score >= 3.0
   - Tier 3 detectors only contribute if Tier 1 + Tier 2 >= 5.0

**Expected Impact:**
- Clean packages with single-detector FPs: score stays low
- Real attacks with multiple signals: score stays high

### Phase 3: Tune Obfuscation Detection (Priority: MEDIUM)

**File:** `glassware-core/src/detectors/glassware.rs`

**Changes:**
1. Lower threshold from 2+ indicators to 2+ indicators (already done)
2. Increase weight for bracket_notation + dynamic_exec combination
3. Add new patterns:
   - Control flow flattening
   - Dead code injection
   - String splitting/concatenation

**Expected Impact:**
- react-native-country-select: 5.00 → 7.0+ ✅
- Maintain low FP rate on clean packages

### Phase 4: Add "Confidence Calibration" (Priority: LOW)

**File:** `glassware/src/scoring.rs`

**Changes:**
1. Track detector confidence per package type
2. Downweight detectors with high FP rate on package type
3. Upweight detectors with high TP rate on package type

**Example:**
```
IF package_type == "blockchain_sdk" THEN
    blockchain_c2_weight = 0.1  # Downweight
ELSE IF package_type == "unknown" THEN
    blockchain_c2_weight = 1.0  # Normal weight
```

---

## Implementation Order

### Iteration 1: BlockchainC2 Fix
1. Edit `blockchain_c2_detector.rs`
2. Test on @solana/web3.js, firebase
3. Verify evidence still detected
4. Tag: v0.62.0-blockchain-fix

### Iteration 2: Conditional Scoring
1. Edit `scanner.rs` scoring logic
2. Add detector tiers
3. Test on wave14
4. Verify FP reduction
5. Tag: v0.63.0-conditional-scoring

### Iteration 3: Obfuscation Tuning
1. Edit `glassware.rs` obfuscation patterns
2. Test on react-native-country-select
3. Verify clean packages not flagged
4. Tag: v0.64.0-obfuscation-tuning

### Iteration 4: Scale Testing
1. Run wave15 (500 packages)
2. Run wave16 (1000 packages)
3. Measure FP rate, evidence detection
4. Tag: v0.65.0-1000pkg-validation

---

## Success Criteria

### BlockchainC2 Fix
- [ ] @solana/web3.js score < 2.0
- [ ] firebase score < 3.0
- [ ] Known malicious wallets still detected (CRITICAL)
- [ ] GlassWorm C2 signature still detected

### Conditional Scoring
- [ ] Clean baseline FP rate < 5%
- [ ] Evidence detection rate = 100%
- [ ] Wave14 malicious count: 9 → ~3 (real attacks only)

### Obfuscation Tuning
- [ ] react-native-country-select score >= 7.0
- [ ] Clean packages with obfuscation-like patterns not flagged

### Scale Testing
- [ ] Wave15 (500 pkg): FP rate < 5%
- [ ] Wave16 (1000 pkg): FP rate < 5%
- [ ] Scan rate > 0.5 pkg/sec

---

## Risk Analysis

### Risk 1: Over-Correction
**Risk:** Fixing BlockchainC2 too aggressively, miss real C2
**Mitigation:** Keep known wallet/IP detection, test on evidence

### Risk 2: Conditional Logic Bugs
**Risk:** Conditional scoring introduces bugs
**Mitigation:** Extensive testing, start with conservative thresholds

### Risk 3: Performance Regression
**Risk:** Additional logic slows scanning
**Mitigation:** Profile performance, optimize hot paths

---

## Next Actions

1. **Immediate:** Fix BlockchainC2 detector (this session)
2. **Next Session:** Implement conditional scoring
3. **Following:** Obfuscation tuning + scale testing

**Estimated Time:** 2-3 sessions to complete all phases

---

**Status:** READY TO IMPLEMENT
**Priority:** BlockchainC2 fix (CRITICAL)
