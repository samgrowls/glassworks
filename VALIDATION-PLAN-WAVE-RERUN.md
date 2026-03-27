# Wave Validation Plan - Semantic Context Filtering

**Date:** 2026-03-27
**Version:** v0.76.0-semantic-context-filtering
**Purpose:** Validate FP reduction on previous waves before scaling to larger scans

---

## Baseline Results (Pre-Fix)

| Wave | Category | Scanned | Flagged | Malicious (FP) | Key FPs |
|------|----------|---------|---------|----------------|---------|
| Wave17 | Mixed (1000+) | 1000+ | TBD | TBD | - |
| Wave18 | React Native | 263 | 40 (15.2%) | 0 | - |
| Wave19 | Crypto/Web3 | 299 | 54 (18.1%) | 0 | - |
| Wave20 | i18n/Translation | 193 | 24 (12.4%) | **4 FP** | pseudo-localization, @commercetools/l10n |
| Wave21 | UI Components | 300 | 40 (13.3%) | **2 FP** | @ag-grid-devtools/cli, vue-tel-input |
| **Total** | **4 categories** | **1055** | **158** | **6 FP** | **0.57% FP rate** |

---

## Validation Goals

### Primary Objectives

1. **Verify FP Elimination**
   - Wave20: 4 FPs → 0 FPs (test files, data files)
   - Wave21: 2 FPs → 0 FPs (build artifacts, data files)
   - Overall: 0.57% → < 0.2% FP rate

2. **Verify Detection Maintained**
   - Evidence packages still detected (100% detection rate)
   - No real attacks missed

3. **Measure Performance Impact**
   - Expected: ~10% slowdown (acceptable trade-off)
   - Scan speed: ~50k LOC/s → ~45k LOC/s

### Secondary Objectives

1. **Validate Context Filtering**
   - Test files correctly skipped
   - Data files correctly skipped
   - Build output correctly skipped
   - Production code still fully scanned

2. **Gather Metrics**
   - Packages reclassified as "skip"
   - Detection categories breakdown
   - Scan time per package

---

## Validation Waves

### Wave 20 Re-Run (Priority: HIGH)

**Why:** Contains 4 FPs from test files and data files

**Expected Results:**
- `pseudo-localization@3.1.1` → NOT flagged (test files skipped)
- `@commercetools-frontend/l10n@27.1.0` → NOT flagged (data files skipped)
- Other 2 FPs → NOT flagged
- Flagged rate: 12.4% → ~5% (real signals only)

**Command:**
```bash
rm -f .glassware-checkpoints.db
./target/release/glassware campaign run campaigns/wave20-i18n.toml
```

### Wave 21 Re-Run (Priority: HIGH)

**Why:** Contains 2 FPs from build artifacts and data files

**Expected Results:**
- `@ag-grid-devtools/cli@35.0.0` → NOT flagged (build output skipped)
- `vue-tel-input-vuetify@1.5.3` → NOT flagged (data files skipped)
- Flagged rate: 13.3% → ~7% (real signals only)

**Command:**
```bash
rm -f .glassware-checkpoints.db
./target/release/glassware campaign run campaigns/wave21-ui-components.toml
```

### Wave 18 Re-Run (Priority: MEDIUM)

**Why:** Largest wave with borderline packages (should remain borderline)

**Expected Results:**
- `country-select-js@2.1.0` → Still flagged (borderline, not FP)
- `libphonenumber-js@1.12.40` → Still flagged (borderline, not FP)
- Flagged rate: 15.2% → ~15% (unchanged, these are real signals)

**Command:**
```bash
rm -f .glassware-checkpoints.db
./target/release/glassware campaign run campaigns/wave18-react-native.toml
```

### Wave 19 Re-Run (Priority: LOW)

**Why:** Clean wave with 0 FPs (sanity check)

**Expected Results:**
- No malicious packages
- Flagged rate: 18.1% → ~18% (unchanged)

**Command:**
```bash
rm -f .glassware-checkpoints.db
./target/release/glassware campaign run campaigns/wave19-crypto-web3.toml
```

---

## Success Criteria

| Metric | Baseline | Target | Pass/Fail |
|--------|----------|--------|-----------|
| Wave20 FPs | 4 | 0 | ✅ Pass if 0 |
| Wave21 FPs | 2 | 0 | ✅ Pass if 0 |
| Overall FP Rate | 0.57% | < 0.2% | ✅ Pass if < 0.2% |
| Evidence Detection | 100% | 100% | ✅ Pass if 100% |
| Performance | 50k LOC/s | > 40k LOC/s | ✅ Pass if > 40k |

---

## Execution Plan

### Phase 1: Quick Validation (2-3 hours)

1. Clear cache
2. Run Wave20 (193 packages, ~10 min)
3. Run Wave21 (300 packages, ~15 min)
4. Analyze results

### Phase 2: Full Validation (4-6 hours)

1. Clear cache
2. Run Wave18 (263 packages, ~15 min)
3. Run Wave19 (299 packages, ~15 min)
4. Analyze results

### Phase 3: Documentation

1. Compare baseline vs new results
2. Document FP elimination
3. Update success metrics
4. Plan larger waves

---

## Cache Management

**CRITICAL:** Clear cache before each wave to avoid cached results:

```bash
# Clear SQLite checkpoint database
rm -f .glassware-checkpoints.db

# Clear orchestrator cache (if enabled)
rm -f .glassware-orchestrator-cache.db

# Verify cache cleared
ls -la .glassware*.db 2>/dev/null || echo "Cache cleared"
```

---

## Monitoring

During execution, monitor:

```bash
# Watch campaign progress
tail -f logs/wave*.log | grep -E "Scanning|Flagged|completed"

# Check for FPs in real-time
grep -E "pseudo-localization|@commercetools|@ag-grid|vue-tel" logs/wave*.log

# Measure scan speed
grep "packages/min" logs/wave*.log
```

---

## Next Steps After Validation

Once validation passes:

1. **Create New Large Waves**
   - Wave22: 2000+ packages (build tools, devtools)
   - Wave23: 2000+ packages (testing frameworks, CLI tools)
   - Wave24: 2000+ packages (web frameworks, state management)

2. **Scale Up**
   - 10,000 package hunt
   - 100,000 package comprehensive scan

3. **Evidence Expansion**
   - Add new synthetic evidence packages
   - Test FN rate with known attacks

---

**Last Updated:** 2026-03-27
**Status:** Ready to execute
