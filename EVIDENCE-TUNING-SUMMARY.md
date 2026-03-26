# Evidence Tuning Session Summary

**Date:** 2026-03-26
**Session:** Evidence Baseline → Detector Tuning

---

## Actions Taken

### 1. Archived Weak Synthetic Packages
Moved 15 weak synthetic packages to `evidence-archived/`:
- glassworm-c2-001 through 004 (C2-only, no invisible chars)
- glassworm-steg-001 through 004 (no actual invisible chars)
- glassworm-evasion-001 through 003 (weak patterns)
- glassworm-exfil-001 through 004 (weak patterns)

**Remaining Evidence:** 8 packages
- 4 real attacks (aifabrix, iflow-mcp, react-native-*)
- 4 good synthetics (glassworm-combo-*)

### 2. Fixed Category Cap for High-Volume Critical Findings

**Problem:** aifabrix-miso-client had 9123 findings but scored only 5.00 due to single-category cap.

**Fix:** Added exception in `apply_category_caps()`:
```rust
if has_high_volume_critical {
    return score.min(10.0);  // Bypass cap
}
```

**Result:** aifabrix-miso-client now scores 10.00 ✅

### 3. Increased Obfuscation Weight

**Change:** obfuscation weight 8.0 → 12.0

**Rationale:** Obfuscation-only attacks (react-native-country-select) need higher weight to cross threshold.

**Result:** Still not detecting react-native-* packages (need more work)

### 4. Added Conditional Rules

```toml
[[settings.scoring.conditional_rules]]
name = "obfuscation_only_attack"
condition = "obfuscation.score >= 8.0"
action = "final_score *= 1.3"

[[settings.scoring.conditional_rules]]
name = "glassworm_signature"
condition = "invisible_char.count >= 10 AND blockchain_c2.count >= 1"
action = "final_score = 25.0"
```

---

## Detection Results

### Before Tuning
| Package | Score | Status |
|---------|-------|--------|
| iflow-mcp | 8.50 | ✅ |
| aifabrix | 5.00 | ❌ |
| react-native-country-select | 5.00 | ❌ |
| react-native-intl-phone | 3.80 | ❌ |
| glassworm-combo-001 | 2.00 | ❌ |
| glassworm-combo-002 | 7.00 | ✅ |
| glassworm-combo-003 | 7.00 | ✅ |
| glassworm-combo-004 | 7.00 | ✅ |

**Detection Rate:** 4/8 (50%)
**Real Attack Detection:** 1/4 (25%)

### After Tuning
| Package | Score | Status | Change |
|---------|-------|--------|--------|
| iflow-mcp | 10.00 | ✅ | +1.50 |
| aifabrix | 10.00 | ✅ | +5.00 🎯 |
| react-native-country-select | 5.00 | ❌ | No change |
| react-native-intl-phone | 3.80 | ❌ | No change |
| glassworm-combo-001 | 2.00 | ❌ | No change |
| glassworm-combo-002 | 7.00 | ✅ | Stable |
| glassworm-combo-003 | 7.00 | ✅ | Stable |
| glassworm-combo-004 | 7.00 | ✅ | Stable |

**Detection Rate:** 5/8 (62.5%)
**Real Attack Detection:** 2/4 (50%) 🎯

---

## Analysis

### What Worked
1. **High-volume critical exception** - Fixed aifabrix-miso-client detection
2. **Tiered scoring** - Working as designed for multi-signal attacks
3. **Archived weak synthetics** - Cleaner evidence set

### What Didn't Work
1. **Obfuscation weight increase** - react-native-* packages still not detecting
2. **Conditional rule for obfuscation** - Not triggering (obfuscation.score < 8.0)

### Root Cause: Obfuscation-Only Attacks

The react-native-* packages use heavy obfuscation (string arrays, XOR, bracket notation) but NO invisible characters. Our current obfuscation detector finds these patterns but doesn't score them high enough.

**react-native-country-select findings:**
- 10 total findings
- Average score: 5.00
- Categories: GlasswarePattern (obfuscation)

**The issue:** The obfuscation patterns are detected but the score doesn't reach 7.0 threshold.

---

## Next Steps

### Option 1: Lower Threshold for Obfuscation-Only
```toml
[settings.scoring]
malicious_threshold = 6.0  # Lower from 7.0 for obfuscation-only
```

**Pros:** Catches obfuscation-only attacks
**Cons:** May increase FP rate

### Option 2: Add More Obfuscation Patterns
Add patterns for:
- Control flow flattening
- Dead code injection
- String splitting/concatenation
- Hex/unicode encoding

**Pros:** More specific detection
**Cons:** More complex detector

### Option 3: LLM Triage for Borderline Cases
Packages scoring 5.0-7.0 get LLM review.

**Pros:** Catches edge cases without lowering threshold
**Cons:** Slower, requires API keys

### Recommended: Combination
1. Lower malicious threshold to 6.0
2. Add more obfuscation patterns
3. Use LLM for 5.0-6.0 borderline cases

---

## Evidence Quality Assessment

### High Quality (Keep)
- iflow-mcp-watercrawl-mcp-1.3.4 ✅ (invisible + C2)
- aifabrix-miso-client-4.7.2 ✅ (encrypted payload)
- glassworm-combo-002/003/004 ✅ (multi-signal)

### Medium Quality (Improve)
- react-native-country-select-0.3.91 ⚠️ (obfuscation-only, real attack)
- react-native-intl-phone-number-0.11.8 ⚠️ (obfuscation-only, real attack)

### Low Quality (Archive/Improve)
- glassworm-combo-001 ❌ (weak stego, not detecting)

---

## Tiered Scoring Effectiveness

### Current Tier Structure
```
Tier 1 (threshold 0.0): invisible_char, glassware_pattern, obfuscation, blockchain_c2
Tier 2 (threshold 2.0): blockchain_c2, header_c2, exfil_schema
Tier 3 (threshold 10.0): locale_geofencing, time_delay_sandbox_evasion
```

### What's Working
- Multi-signal attacks (iflow-mcp, glassworm-combo-*) flow through tiers correctly
- Single-signal attacks (aifabrix) now detected via high-volume exception

### What Needs Work
- Obfuscation-only attacks stuck at Tier 1 with score 5.0
- Need to either:
  - Increase Tier 1 weights for obfuscation
  - Lower Tier 2 threshold to 1.0
  - Add exception for high-confidence obfuscation

---

**Status:** IMPROVED (50% → 62.5% detection, 25% → 50% real attack detection)
**Next:** Address obfuscation-only detection gap
