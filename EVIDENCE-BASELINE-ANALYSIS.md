# Evidence Baseline Analysis

**Date:** 2026-03-26
**Status:** BASELINE COMPLETE

---

## Evidence Test Results (23 packages)

### ✅ DETECTED (Malicious, score >= 7.0)

| Package | Score | Findings | Type | Notes |
|---------|-------|----------|------|-------|
| iflow-mcp-watercrawl-mcp-1.3.4 | 8.50 | 9126 | REAL | Invisible chars + C2 patterns |
| glassworm-combo-002 | 7.00 | 4 | Synthetic | Combo attack |
| glassworm-combo-003 | 7.00 | 3 | Synthetic | Combo attack |
| glassworm-combo-004 | 7.00 | 4 | Synthetic | Combo attack |

**Detection Rate:** 4/23 (17%)
**Real Attack Detection:** 1/4 (25%) ❌

---

### ⚠️ BORDERLINE (Score 5.0-6.9, needs attention)

| Package | Score | Findings | Type | Issue |
|---------|-------|----------|------|-------|
| aifabrix-miso-client-4.7.2 | 5.00 | 9123 | REAL | High findings, low score |
| react-native-country-select-0.3.91 | 5.00 | 10 | REAL | Obfuscation-only (no invisible chars) |
| glassworm-evasion-001 | 5.00 | 6 | Synthetic | Evasion patterns |
| glassworm-exfil-001 | 5.00 | 3 | Synthetic | Exfil patterns |

**Issue:** These are REAL attacks scoring below threshold!

---

### ❌ NOT DETECTED (Score < 5.0)

| Package | Score | Findings | Type | Issue |
|---------|-------|----------|------|-------|
| react-native-intl-phone-number-0.11.8 | 3.80 | 3 | REAL | Obfuscation-only |
| glassworm-c2-001 | 2.00 | 2 | Synthetic | C2 without invisible chars |
| glassworm-c2-004 | 2.65 | 3 | Synthetic | C2 without invisible chars |
| glassworm-evasion-002 | 0.55 | 1 | Synthetic | Weak evasion |
| glassworm-evasion-003 | 3.25 | 2 | Synthetic | Weak evasion |
| glassworm-exfil-002 | 2.64 | 1 | Synthetic | Weak exfil |
| glassworm-exfil-003 | 1.56 | 1 | Synthetic | Weak exfil |
| glassworm-exfil-004 | 2.64 | 1 | Synthetic | Weak exfil |
| glassworm-c2-002 | 0.00 | 0 | Synthetic | No detection |
| glassworm-c2-003 | 0.00 | 0 | Synthetic | No detection |
| glassworm-steg-001 | 0.00 | 0 | Synthetic | No invisible chars |
| glassworm-steg-002 | 0.00 | 0 | Synthetic | No invisible chars |
| glassworm-steg-003 | 0.00 | 0 | Synthetic | No invisible chars |
| glassworm-steg-004 | 0.00 | 0 | Synthetic | No invisible chars |

---

## Key Findings

### 1. Real Attack Detection Gap

**Problem:** Only 1/4 real attacks detected (25%)

| Real Attack | Detection | Reason |
|-------------|-----------|--------|
| iflow-mcp | ✅ 8.50 | Has invisible chars + C2 |
| react-native-country-select | ❌ 5.00 | Obfuscation-only, no invisible chars |
| react-native-intl-phone-number | ❌ 3.80 | Obfuscation-only |
| aifabrix-miso-client | ❌ 5.00 | 9123 findings but score capped |

**Root Cause:** Our detector design requires invisible characters for high-confidence detection. Real GlassWorm attacks that use obfuscation-only (no invisible chars) score in the 5.0 range but don't cross the 7.0 threshold.

### 2. Synthetic Evidence Quality Issues

**Problem:** Many synthetic packages don't match real attack patterns

| Synthetic Type | Detection Rate | Issue |
|----------------|----------------|-------|
| glassworm-c2-* | 0/4 (0%) | No invisible chars, C2-only |
| glassworm-steg-* | 0/4 (0%) | No actual invisible chars in files |
| glassworm-evasion-* | 0/3 (0%) | Weak evasion patterns |
| glassworm-exfil-* | 0/4 (0%) | Weak exfil patterns |
| glassworm-combo-* | 3/4 (75%) | Has multiple signals |

**Root Cause:** Synthetic packages were created without invisible characters, making them fundamentally different from real GlassWorm attacks.

### 3. Scoring System Issues

**Problem:** aifabrix-miso-client has 9123 findings but only scores 5.00

**Root Cause:** Category diversity caps and deduplication are working TOO well - thousands of similar findings are being collapsed into a single pattern with diminishing returns.

---

## Recommendations

### Immediate Actions (Next Session)

1. **Remove weak synthetic packages**
   - Delete glassworm-steg-* (no invisible chars)
   - Delete glassworm-c2-* (C2-only, doesn't match real attacks)
   - Delete glassworm-evasion-* and glassworm-exfil-* (weak patterns)
   - Keep glassworm-combo-* (detects correctly)

2. **Improve obfuscation detection**
   - Increase weight for obfuscation patterns
   - Add more obfuscation patterns (string arrays, control flow)
   - Lower threshold for obfuscation-only attacks OR add exception

3. **Fix aifabrix-miso-client scoring**
   - Investigate why 9123 findings = 5.00 score
   - May need to adjust category caps or deduplication

### Medium-Term Actions

1. **Create better synthetic evidence**
   - Use real attacks as templates
   - Include invisible characters
   - Include obfuscation patterns
   - Include C2 patterns

2. **Adjust scoring thresholds**
   - Consider lowering malicious threshold to 6.0 for obfuscation-only
   - OR add exception for high-confidence obfuscation patterns

3. **LLM triage for borderline cases**
   - Packages scoring 5.0-7.0 should get LLM review
   - LLM can catch obfuscation-only attacks

---

## Evidence Inventory (Post-Cleanup)

### Keep (Detects correctly)
- iflow-mcp-watercrawl-watercrawl-mcp-1.3.4.tgz ✅
- glassworm-combo-002.tgz ✅
- glassworm-combo-003.tgz ✅
- glassworm-combo-004.tgz ✅

### Keep (Needs scoring fix)
- react-native-country-select-0.3.91.tgz ⚠️ (obfuscation-only, real attack)
- react-native-international-phone-number-0.11.8.tgz ⚠️ (obfuscation-only, real attack)
- aifabrix-miso-client-4.7.2.tgz ⚠️ (scoring issue, 9123 findings)

### Remove (Weak/incorrect synthetics)
- glassworm-c2-001.tgz through 004.tgz ❌
- glassworm-steg-001.tgz through 004.tgz ❌
- glassworm-evasion-001.tgz through 003.tgz ❌
- glassworm-exfil-001.tgz through 004.tgz ❌

---

## Next Steps

1. **Remove weak synthetic packages** from evidence directory
2. **Improve obfuscation detection** weights and patterns
3. **Test on remaining evidence** - should detect 6/7 real attacks
4. **Create wave15** with validated evidence + 500 clean packages
5. **Measure FP rate** on clean baseline

**Target:** 85%+ detection rate on real attacks, <5% FP rate on clean packages

---

**Status:** READY FOR ACTION
**Priority:** Remove weak synthetics, improve obfuscation detection
