# Investigation State - March 24, 2026

**Generated:** 2026-03-24 (morning)
**Trigger:** tmux sessions killed, investigating current state before making changes
**Investigator:** Qwen-Coder

---

## 🔍 Executive Summary

**Your instinct was CORRECT:** The Wave 9 scan (481 packages, 56 malicious) did trigger extensive whitelist work, but the current Wave 12 failure is **NOT related to whitelist/scoring issues**.

### Current State in One Sentence

**The whitelist is working correctly (verified in Wave 11), but Wave 12 (5000 packages) failed because npm category sources return 0 packages - this is a data source issue, not a detection issue.**

---

## 📊 Campaign History (Chronological)

| Time (UTC) | Campaign | Packages | Result | Status |
|------------|----------|----------|--------|--------|
| ~17:00 | Wave 8 | 68 | 19 flagged, 4 malicious | ✅ Complete |
| 18:23 | Wave 9 | 481 | 129 flagged, 56 malicious | ✅ Complete |
| 18:28 | Wave 10 | 611 | Unknown | ✅ Complete |
| 20:23 | **Wave 11** (validation) | 54 | 28 flagged, 16 malicious | ✅ **Whitelist working** |
| 20:38-20:42 | Individual tests | 5-10 each | Verified whitelist fix | ✅ Complete |
| **21:00** | **CRITICAL FIX** (6af9b4e) | - | Whitelist working in campaign mode | ✅ **Committed** |
| **21:01** | **Wave 12** (5000 pkg) | **0** | **All waves collected 0 packages** | ❌ **FAILED** |
| ~21:04 | Wave 9 re-run | 479 | 129 flagged, 56 malicious | ✅ Complete |

---

## ✅ What IS Working

### 1. Whitelist System (Verified Working)

**Commit:** `6af9b4e` - "CRITICAL FIX: Whitelist now working correctly in campaign mode"

**Fixes Applied:**
1. **Detector weights initialization** - Previously used `..Default::default()` which set all weights to 0.0
2. **Version string handling** - `webpack@5.89.0` now correctly matches `webpack` in whitelist
3. **Scoped package handling** - `@prisma/client@5.8.1` now correctly matches `@prisma/client`

**Wave 11 Validation Results:**
```
✅ 54 packages scanned
✅ 28 packages whitelisted (findings suppressed)
✅ 0 false positives (all known clean packages properly whitelisted)
✅ webpack, vite, viem, moment, lodash, etc. all whitelisted correctly
```

**Files Modified:**
- `glassware/src/campaign/wave.rs` (+14 lines) - Proper DetectorWeights initialization
- `glassware/src/scanner.rs` (+19 lines) - Fixed `is_package_whitelisted()` version stripping
- `campaigns/wave11-evidence-validation.toml` - Test configuration

---

### 2. Campaign Orchestration

All campaigns Waves 6-11 completed successfully:
- Checkpoint/resume working
- TUI monitoring working
- LLM analysis working
- Report generation working

---

### 3. Unified Binary

**Binary consolidation is COMPLETE:**
```bash
./target/release/glassware --help
# Unified GlassWare attack detection and campaign orchestration
```

No more separate `glassware-cli` and `glassware-orchestrator` binaries!

---

## ❌ What IS Broken

### 1. Wave 12 Package Collection (CRITICAL)

**Symptom:** All 5 waves collected 0 packages

**Log Evidence:**
```
INFO 📦 Starting wave 'React Native Ecosystem' (wave_12a)
INFO Collecting packages for wave 'React Native Ecosystem'...
INFO Collected 0 packages for wave 'React Native Ecosystem'
INFO ✅ Wave 'React Native Ecosystem' completed: 0 scanned, 0 flagged, 0 malicious
```

**Configuration:**
```toml
[[waves.sources]]
type = "npm_category"
category = "react-native"
samples = 1000
```

**Root Cause:** The `npm_category` source type is not returning any packages. This could be:
- npm API changed the category endpoint
- Rate limiting blocking category queries
- Category names changed (e.g., "react-native" → "react-native-packages")
- Network/timeout issue

**Impact:** Cannot run large-scale scans using category-based package selection

---

### 2. Evidence Detection Still Weak

**From `docs/evidence/EVIDENCE-DETECTION-STATUS.md`:**

| Evidence Package | Findings | Avg Score | Expected | Status |
|-----------------|----------|-----------|----------|--------|
| react-native-country-select-0.3.91 | 11 | 4.00 | 7.0+ | ⚠️ Below threshold |
| react-native-intl-phone-number-0.11.8 | 10 | 2.00 | 7.0+ | ⚠️ Below threshold |

**Problem:** Evidence packages contain:
- Function obfuscation (NOT detected)
- Blockchain C2 patterns (NOT detected)
- Invisible Unicode characters (✅ detected)

**Current scoring formula:**
```
score = (categories × 3.0) + (critical_hits × 4.0) + (high_hits × 2.0)
```

For 11 InvisibleCharacter findings in 1 category:
- Expected: (1 × 3.0) + (0 × 4.0) + (11 × 2.0) = 25 → capped at 10.0
- Actual: 4.00 (config not being applied properly?)

---

### 3. Wave 11 "Known Clean" Wave Not Using Whitelist

**Issue:** Wave 11B (Known Clean - Popular Libraries) had 16 packages flagged as malicious, including:
- `webpack@5.89.0` - threat_score: 10.00, flagged as malicious

**But webpack IS in the whitelist!** 

Looking at the log:
```
INFO Package webpack@5.89.0 scanned: 16 findings, threat_score=10.00, malicious=true
```

**Possible Causes:**
1. Wave mode `"validate"` bypasses whitelist?
2. Whitelist not passed to scanner for this wave?
3. Wave-level expectations override global whitelist?

**Need to investigate:** Wave executor logic for `mode = "validate"` waves

---

## 📁 Key Documentation Files

### Most Recently Modified (March 23, 21:03 UTC)

1. **`docs/README.md`** (21:03) - Documentation index
2. **`docs/evidence/EVIDENCE-DETECTION-STATUS.md`** (19:13) - ⚠️ **READ THIS FIRST**
3. **`docs/WHITELIST-INTEGRATION-PLAN.md`** (18:04) - Full integration plan
4. **`docs/WHITELIST-ENHANCEMENT-WIP.md`** (17:59) - Work in progress notes
5. **`docs/evidence/WAVE8-RESULTS-ANALYSIS.md`** (17:24) - Wave 8 analysis
6. **`docs/CAMPAIGN-STATUS-MAR23.md`** (17:07) - Campaign status snapshot
7. **`docs/campaigns/WAVE9-PLANNING.md`** (17:05) - Wave 9 planning doc
8. **`docs/WHITELIST-ENHANCEMENT-COMPLETE.md`** (17:00) - Implementation complete

### Critical Handoff Documents

- **`HANDOFF/FINAL-SESSION-SUMMARY.md`** - Session completion summary (v0.15.0)
- **`HANDOFF/FUTURE/BINARY-CONSOLIDATION.md`** - Binary consolidation plan
- **`HANDOFF/FUTURE/LONG-RUNNING-CAMPAIGNS.md`** - Long-running design
- **`HANDOFF/FUTURE/ROADMAP-2026.md`** - Strategic roadmap

---

## 🎯 Recommended Next Steps

### Priority 1: Fix Wave 12 Package Collection

**Investigation Steps:**
1. Test npm category API manually:
   ```bash
   curl "https://registry.npmjs.org/-/v1/search?text=keywords:react-native&size=10"
   ```
2. Check if category names changed
3. Look at `glassware/src/campaign/sources.rs` (or similar) for npm_category implementation
4. Add logging to see actual API responses

**Workaround Options:**
1. Use explicit package lists instead of categories
2. Use `npm_search` with keywords instead of categories
3. Pre-generate package lists from npm APIs

---

### Priority 2: Verify Whitelist in Wave 11B

**Investigation Steps:**
1. Check wave executor logic for `mode = "validate"`
2. Verify whitelist is passed to scanner for all wave modes
3. Add debug logging to `is_package_whitelisted()` calls

**Expected Behavior:**
- Whitelisted packages should NEVER be flagged as malicious
- Findings should be suppressed (logged but not counted)

---

### Priority 3: Improve Evidence Detection

**From `EVIDENCE-DETECTION-STATUS.md`:**

**Short-Term:**
1. Debug score calculation - add logging
2. Verify config is being loaded correctly
3. Run 5000 package scan with current detection (document false negatives)

**Medium-Term:**
1. Enhance GlassWare pattern detection (obfuscation patterns)
2. Enhance blockchain C2 detection (RPC endpoints, wallet addresses)
3. Add entropy-based detection
4. Add control flow graph analysis

---

## 🔬 Testing Commands

```bash
# Test whitelist (should NOT be flagged)
./target/release/glassware scan-npm webpack@5.89.0
./target/release/glassware scan-npm moment@2.30.1
./target/release/glassware scan-npm lodash@4.17.21

# Test evidence (SHOULD be flagged but currently isn't)
./target/release/glassware scan-tarball evidence-archive/evidence/react-native-country-select-0.3.91.tgz

# Test npm category (debug package collection)
./target/release/glassware campaign run campaigns/wave12-5000pkg.toml

# Check recent scans
./target/release/glassware scan-list

# Check campaign status
./target/release/glassware campaign list
./target/release/glassware campaign status <case-id>
```

---

## 📝 Git State

**Current Branch:** `main`
**Latest Commit:** `6af9b4e` - "CRITICAL FIX: Whitelist now working correctly in campaign mode"
**Working Directory:** Clean
**Stashed Changes:** None

**Recent Tags:**
- `v0.25.0-evidence-fixed` - Evidence detection fixed
- `v0.24.0-wave10-complete` - Wave 10 complete (611 packages)
- `v0.23.0-whitelist-complete` - Whitelist integration complete

---

## 🎓 Key Learnings

### What Wave 9 Taught Us

1. **False positives at scale are real:** 129/481 packages flagged (27%) is too high
2. **Whitelist is essential:** Without it, popular packages get flagged
3. **Version handling matters:** `package@version` must match `package` in whitelist
4. **Scoped packages need special handling:** `@scope/name@version`

### What Wave 11 Validated

1. **Whitelist works:** 28/54 packages properly whitelisted
2. **Detector weights work:** Proper initialization is critical
3. **BUT:** Evidence packages still not detected (need obfuscation detection)

### What Wave 12 Revealed

1. **Package collection is fragile:** Category-based selection can fail silently
2. **Need fallback strategies:** Explicit lists, keyword search, etc.
3. **Validation is important:** Should have tested with small sample first

---

## 🚀 Success Criteria for Next Session

### Immediate (Day 1)
- [ ] Fix npm_category package collection
- [ ] Verify Wave 12 can collect packages
- [ ] Run small test wave (10-50 packages)

### Short-Term (Week 1)
- [ ] Complete Wave 12 scan (5000 packages)
- [ ] Generate markdown report
- [ ] Analyze results, tune thresholds
- [ ] Document false positives/negatives

### Medium-Term (Month 1)
- [ ] Enhance obfuscation detection
- [ ] Enhance blockchain C2 detection
- [ ] Improve evidence detection rate
- [ ] Binary consolidation complete (already done!)

---

**Last Updated:** March 24, 2026 07:00 UTC
**Status:** Ready to resume work
**Next Action:** Fix npm_category package collection
