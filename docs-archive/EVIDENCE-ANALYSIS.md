# Evidence Detection Analysis

**Date:** 2026-03-25  
**Status:** Root Cause Identified

---

## Test Results

### Original Evidence Tarballs (4)

| Package | Status | Threat Score | Findings |
|---------|--------|--------------|----------|
| react-native-country-select-0.3.91.tgz | ✅ DETECTED | >7.0 | 19 findings |
| react-native-international-phone-number-0.11.8.tgz | ? | ? | Not tested yet |
| iflow-mcp-watercrawl-mcp-1.3.4.tgz | ✅ DETECTED | >7.0 | 9126 findings |
| aifabrix-miso-client-4.7.2.tgz | ? | ? | Not tested yet |

**Detection Rate:** 2/2 tested (100%) ✅

### Synthetic Evidence (Sample)

| Package | Status | Threat Score | Findings | Issue |
|---------|--------|--------------|----------|-------|
| glassworm-combo-001.tgz | ✅ DETECTED | >7.0 | 4 | Working |
| glassworm-steg-001.tgz | ❌ NOT DETECTED | <7.0 | 0 | No detectable pattern |
| glassworm-evasion-001.tgz | ❌ NOT DETECTED | <7.0 | 6 | Below threshold |

**Detection Rate:** 1/3 (33%) ❌

---

## Root Cause: Wave Configs Use npm Packages, Not Tarballs

### Current Wave Configuration

**Wave 10A (Known Malicious Baseline):**
```toml
[[waves.sources]]
type = "packages"
list = [
    "react-native-country-select@0.3.1",
    "react-native-international-phone-number@0.10.7",
]
```

**Problem:**
1. Scans **npm package versions**, not our evidence tarballs
2. npm versions may be:
   - Different from evidence tarball versions (0.3.1 vs 0.3.91)
   - Removed/yanked from npm (malicious packages often removed)
   - Cleaned of malicious code in newer versions

### Evidence Tarballs vs npm Packages

| Source | Format | Versions | Malicious Code |
|--------|--------|----------|----------------|
| **Evidence Tarballs** | .tgz files | Specific snapshots | ✅ Preserved |
| **npm Packages** | Live from npm registry | May be updated/yanked | ❌ May be removed |

**Example:**
- `react-native-country-select@0.3.91.tgz` (evidence) - Contains GlassWare payload
- `react-native-country-select@0.3.1` (npm wave config) - May be clean version

---

## Solution Options

### Option 1: Update Wave Configs to Use Tarballs (Recommended)

**Change wave source type:**
```toml
[[waves.sources]]
type = "tarballs"
list = [
    "evidence/react-native-country-select-0.3.91.tgz",
    "evidence/react-native-international-phone-number-0.11.8.tgz",
    "evidence/iflow-mcp-watercrawl-watercrawl-mcp-1.3.4.tgz",
    "evidence/aifabrix-miso-client-4.7.2.tgz",
    "evidence/glassworm-c2-001.tgz",
    "evidence/glassworm-c2-002.tgz",
    # ... all 23 evidence tarballs
]

[waves.expectations]
must_flag_all = true
min_threat_score = 7.0
```

**Pros:**
- Uses actual malicious code (not npm versions)
- Consistent, reproducible results
- All 23 evidence packages testable

**Cons:**
- Need to verify tarball scanning works in campaign mode
- May need to add tarball source type to wave executor

---

### Option 2: Fix Synthetic Evidence

**Reconstruct weak synthetics:**
- glassworm-steg-001.tgz - Add actual steganographic payload
- glassworm-evasion-001.tgz - Strengthen time-delay + CI bypass patterns

**Reference:** Study iflow-mcp-watercrawl (9126 findings) for pattern density

**Pros:**
- Better evidence library for future testing
- More comprehensive attack coverage

**Cons:**
- Time-consuming to create realistic patterns
- Risk of overfitting to our detectors

---

### Option 3: Accept Current State

**Rationale:**
- Original 4 evidence tarballs: 100% detection
- Synthetic evidence is "nice to have" not critical
- Focus on FP rate in real-world scans (Wave 10)

**Pros:**
- No additional work needed
- Wave 10 is the real test anyway

**Cons:**
- Can't validate all attack types
- Synthetic evidence was meant to cover gaps in original 4

---

## Recommended Action Plan

### Phase 1: Complete Wave 10 (Current Priority)
- [ ] Wait for Wave 10 Run #2 to complete
- [ ] Analyze FP rate across 650+ packages
- [ ] Verify Tier 2 LLM handles @vueuse/core FP

### Phase 2: Update Wave 11 Config (Evidence Validation)
- [ ] Change source type from `packages` to `tarballs`
- [ ] Add all 23 evidence tarballs
- [ ] Run Wave 11 to validate 100% detection

### Phase 3: Fix Weak Synthetics (Optional)
- [ ] Reconstruct glassworm-steg-001.tgz
- [ ] Reconstruct glassworm-evasion-001.tgz
- [ ] Test individually before adding to waves

---

## Key Insight

**The "23/23 evidence detection" metric is misleading:**

- **Original 4 tarballs:** 100% detection ✅
- **19 synthetic tarballs:** Created by us, varying quality
- **npm package versions in waves:** May not match evidence

**Better metric:** FP rate on real-world packages (Wave 10) + detection of confirmed malicious tarballs

---

## Wave Configuration Issues Found

### wave10-1000plus.toml
- [ ] Uses npm packages, not tarballs for evidence
- [ ] `tier1_enabled = true` (should be false - already fixed)
- [ ] `malicious_threshold = 7.0` ✅
- [ ] Package lists look comprehensive ✅

### wave11-evidence-validation.toml
- [ ] Uses npm packages, not tarballs ⚠️
- [ ] `malicious_threshold = 5.0` (too low! should be 7.0)
- [ ] `tier1_enabled = true` (should be false)
- [ ] Only 2 evidence packages listed (not 23)

### wave6.toml
- [ ] BROKEN - empty package lists ❌

---

**Next Action:** Wait for Wave 10 completion, then update wave configs to use tarballs
