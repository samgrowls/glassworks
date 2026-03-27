# 10k Package Scan - Preparation Status

**Date:** 2026-03-27
**Status:** READY FOR EXECUTION ON CURRENT VM

---

## Current State

### Completed Waves

| Wave | Category | Scanned | Malicious | Status |
|------|----------|---------|-----------|--------|
| Wave22 | Build Tools | 987 | 3 | ✅ Complete |
| Wave23 | Testing & CLI | 1971 | 2 | ✅ Complete |
| Wave24 | Frameworks | 251/999 | 0 | ⏳ Restarted |
| Wave25 | Scoring Validation | 101 | 2 | ✅ Complete |

**Total Scanned:** 3,310 packages
**Total Malicious:** 7 packages (0.21%)

### Malicious Packages Found

1. **systemjs-plugin-babel@0.0.25** (Score: 10.00) - Bundled code, needs review
2. **babel-plugin-angularjs-annotate@0.10.0** (Score: 9.00) - Babel patterns, needs review
3. **monopw@1.0.3** (Score: 5.65) - Wave23
4. **pomera-ai-commander@1.4.4** (Score: 8.23) - Wave23
5. **Wave25:** 2 packages (same as above)

### Known Issues

**Tiered Scoring Not Applied:**
- Campaign TOML has tiered scoring config
- BUT Scanner's `scan_package()` uses `ScoringConfig::default()`
- Architectural fix requires refactoring Scanner to use campaign config
- **Impact:** Bundled/minified code may score higher than intended
- **Mitigation:** Manual review of high-score packages

---

## 10k Scan Preparation

### Disk Space

**Current:** 158GB available (25% used)
**Required:** ~50GB for 10k packages
**Status:** ✅ Sufficient space

### Cleanup Options

**Safe to Delete:**
- `.glassware-checkpoints.db*` - Auto-regenerates (~12KB)
- `.glassware-orchestrator-cache.db*` - Scan cache (~50KB)
- `logs/*.log` - Historical logs (~1-2GB if accumulated)
- `evidence/wave22-investigation/` - Investigation complete (~5MB)
- `target/debug/` - Debug builds (~20GB, but requires rebuild)

**Recommended:** Keep target/debug for now (rebuild takes 15-20 min)

### 10k Scan Configuration

**File:** `campaigns/wave-10k-master.toml`
**Waves:** 8 waves, 10,000 packages total
**Expected Duration:** 8-12 hours
**Concurrency:** 20 packages

---

## Execution Plan

### Phase 1: Complete Wave24
- Currently restarting
- ~750 packages remaining
- Expected: 30-45 minutes

### Phase 2: Prepare for 10k
1. Clear cache databases
2. Verify disk space (>100GB free)
3. Start 10k scan

### Phase 3: Run 10k Scan
```bash
cd ~/glassworks
rm -f .glassware*.db*
nohup ./target/debug/glassware campaign run campaigns/wave-10k-master.toml \
    > logs/10k-scan.log 2>&1 &
```

### Monitoring
```bash
# Progress
tail -f logs/10k-scan.log | grep -E "Wave.*completed|packages scanned"

# Malicious count
grep -c "flagged as malicious" logs/10k-scan.log

# Package count
grep -c "Package.*scanned:" logs/10k-scan.log
```

---

## Risk Mitigation

### If Scan Fails Mid-Way
```bash
# Resume from checkpoint
./target/debug/glassware campaign resume <case-id>
```

### If Disk Space Runs Low
```bash
# Clear cache
rm -f .glassware*.db*

# Clear old logs
rm -f logs/*.log.*

# If desperate, clear target/debug
rm -rf target/debug  # Requires rebuild
```

### If Too Many False Positives
- Manual review required
- Document patterns
- Adjust detectors AFTER scan (not during)

---

## Success Criteria

- ✅ All 10,000 packages scanned
- ✅ Evidence preserved for flagged packages
- ✅ Report generated
- ✅ Malicious packages documented
- ✅ FP rate < 1%

---

**Ready to proceed with 10k scan on current VM.**
