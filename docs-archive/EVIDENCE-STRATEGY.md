# Evidence Detection Strategy

**Date:** 2026-03-25  
**Decision:** Use BOTH tarballs (for validation) AND npm packages (for real-world scanning)

---

## Current Understanding

### Evidence Detection Status

**Original 4 Malicious Tarballs:**
- react-native-country-select-0.3.91.tgz ✅ DETECTED
- react-native-international-phone-number-0.11.8.tgz ✅ DETECTED  
- iflow-mcp-watercrawl-mcp-1.3.4.tgz ✅ DETECTED (9126 findings)
- aifabrix-miso-client-4.7.2.tgz ✅ DETECTED (9125 findings)

**Detection Rate: 4/4 (100%)** ✅

### Synthetic Evidence

**Status:** Leave as-is (per user decision)
- Some detect (glassworm-combo-001.tgz)
- Some don't (glassworm-steg-001.tgz, glassworm-evasion-001.tgz)
- Not critical for production use

---

## Wave Configuration Strategy

### Question: Tarballs vs npm Packages?

**User's Insight:**
> "Can we add tarballs just for the evidence and npm for our full scan? That way we know our scanner is working (100% on the 4 legit malicious tarballs)"

**Answer: YES - This is the right approach!**

### Recommended Wave Structure

**Wave 11A: Evidence Validation (Tarballs)**
```toml
[[waves]]
id = "wave_11a"
name = "Evidence Tarballs - MUST Detect"
mode = "validate"

[[waves.sources]]
type = "tarballs"
list = [
    "evidence/react-native-country-select-0.3.91.tgz",
    "evidence/react-native-international-phone-number-0.11.8.tgz",
    "evidence/iflow-mcp-watercrawl-watercrawl-mcp-1.3.4.tgz",
    "evidence/aifabrix-miso-client-4.7.2.tgz",
]

[waves.expectations]
must_flag_all = true
min_threat_score = 7.0
```

**Purpose:** Validate scanner is detecting real threats

---

**Wave 10: Real-World Scan (npm Packages)**
```toml
[[waves]]
id = "wave_10b"
name = "Clean Baseline - Top 100 npm"
mode = "validate"

[[waves.sources]]
type = "packages"
list = [
    "express@4.19.2", "lodash@4.17.21", ...
]

[waves.expectations]
max_threat_score = 3.0  # Should flag NOTHING
```

**Purpose:** Measure real-world FP rate

---

## Why This Approach Is Correct

### 1. Tarballs for Evidence Validation
- ✅ Guaranteed to contain malicious code (not yanked/updated)
- ✅ Reproducible results (same code every time)
- ✅ Known ground truth (4/4 = 100% detection)

### 2. npm Packages for Real-World Testing
- ✅ Tests actual scanning workflow (download + scan)
- ✅ Measures real FP rate on popular packages
- ✅ Catches supply chain attacks on live packages

### 3. Separation of Concerns
- **Wave 11A:** "Can we detect known threats?" (tarballs)
- **Wave 10:** "Are we flagging legitimate packages?" (npm)

---

## Implementation Plan

### Step 1: Create Wave 11 Config (Evidence Validation)

Create `campaigns/wave11-evidence-validation.toml`:

```toml
[campaign]
name = "Wave 11 - Evidence Validation"
description = "Validate detection on 4 confirmed malicious tarballs"

[settings]
concurrency = 10
cache_enabled = true

[settings.scoring]
malicious_threshold = 7.0

[settings.llm]
tier1_enabled = false  # No LLM needed for evidence validation
tier2_enabled = false

# Wave 11A: Evidence Tarballs
[[waves]]
id = "wave_11a"
name = "Evidence Tarballs"
mode = "validate"

[[waves.sources]]
type = "tarballs"
list = [
    "evidence/react-native-country-select-0.3.91.tgz",
    "evidence/react-native-international-phone-number-0.11.8.tgz",
    "evidence/iflow-mcp-watercrawl-watercrawl-mcp-1.3.4.tgz",
    "evidence/aifabrix-miso-client-4.7.2.tgz",
]

[waves.expectations]
must_flag_all = true
min_threat_score = 7.0
```

### Step 2: Run Wave 11

```bash
./target/release/glassware campaign run campaigns/wave11-evidence-validation.toml
```

**Expected Result:**
- 4/4 tarballs flagged as malicious
- All threat scores >= 7.0

### Step 3: Keep Wave 10 As-Is (npm Packages)

Wave 10 already uses npm packages for real-world FP testing.

**Current Status:** Running (119 packages scanned, 0 FP so far)

---

## Key Metrics

### Evidence Detection (Wave 11)
- **Target:** 4/4 (100%)
- **Current:** 4/4 tested ✅

### False Positive Rate (Wave 10)
- **Target:** <0.5% (<3/650)
- **Current:** 0% (0/119) - Running

### Combined Assessment
- **Evidence Detection:** ✅ Working
- **FP Rate:** ✅ Looking good
- **Tier 2 LLM:** Will handle borderline cases like @vueuse/core

---

## File Changes Needed

### Create: campaigns/wave11-evidence-validation.toml

**Purpose:** Validate 4 evidence tarballs

**Content:** See template above

### Update: campaigns/wave10-1000plus.toml

**Already correct** - uses npm packages for real-world scanning

Just ensure:
- `tier1_enabled = false` ✅ (already done)
- `malicious_threshold = 7.0` ✅ (already set)

---

## Testing Protocol

### Before Any Release

```bash
# 1. Validate evidence detection
./target/release/glassware campaign run campaigns/wave11-evidence-validation.toml
# Expected: 4/4 detected

# 2. Validate FP rate
./target/release/glassware campaign run campaigns/wave10-1000plus.toml
# Expected: FP rate < 0.5%

# 3. If both pass, ready for release
```

---

## Summary

**Evidence Strategy:**
- ✅ 4 original tarballs: 100% detection
- ✅ Synthetic tarballs: Leave as-is (not critical)
- ✅ Wave 11: Validate on tarballs
- ✅ Wave 10: Real-world FP testing on npm packages

**Current Status:**
- Wave 10 running (119 packages, 0 FP)
- Wave 11 config needs to be created
- Evidence detection working (4/4)

**Next Action:** Create wave11-evidence-validation.toml
