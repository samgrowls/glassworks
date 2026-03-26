# GlassWorm v0.57.0-Longwave - Long Horizon Scanning

**Created:** 2026-03-25  
**Based on:** v0.41.0 + autoresearch FP fix (v0.41.1-fp-fix)  
**Branch:** `autoresearch/fp-rate-tuning-2026-03-25`  
**Tag:** `v0.41.1-fp-fix`

---

## What's Different in This Version

### Critical FP Fix Applied

**Problem:** InvisibleCharacter detector was flagging legitimate Unicode characters in i18n/locale data files (e.g., moment.js Persian/Arabic locale files).

**Root Cause:** Scanner was passing **directory path** (`/tmp/.tmpXYZ`) instead of **file path** (`/tmp/.tmpXYZ/locale/si.js`) to detectors, breaking path-based i18n filtering.

**Fix Applied:**
1. `glassware/src/scanner.rs` - Fixed `engine.scan(path, ...)` → `engine.scan(&entry_path, ...)` (lines 268 & 326)
2. `glassware-core/src/detectors/homoglyph.rs` - Added i18n path skip for `/locale/`, `/locales/`, `/i18n/`, etc.

**Result:**
- FP Rate: 10% → **0%** ✅
- Evidence Detection: 100% → **100%** ✅
- Combined Score: 0.96 → **1.00** ✅

---

## Current Campaign: Wave 10

**Started:** 2026-03-25 17:31 UTC  
**Campaign:** Wave 10 - Production-Scale Hunt (1000+ packages)  
**Case ID:** `wave-10---production-scale-hunt-(1000+-packages)-20260325-173138`

### Wave Structure

| Wave | Packages | Description | Mode |
|------|----------|-------------|------|
| 10A | 2 | Known Malicious Baseline | validate |
| 10B | 100 | Clean Baseline - Top 100 npm | validate |
| 10C | 150 | React Native Ecosystem | hunt |
| 10D | 150 | Vue.js Ecosystem | hunt |
| 10E | 150 | Angular Ecosystem | hunt |
| 10F | 200 | Node.js Core Ecosystem | hunt |

**Total:** ~752 packages  
**Expected Duration:** ~5-10 minutes (0.3-0.5s per package)

### Monitoring

```bash
# Check campaign status
./monitor-wave10.sh

# Follow logs in real-time
tail -f logs/wave10-*.log

# Check if still running
ps aux | grep glassware | grep -v grep
```

### Expected Results

With the FP fix applied:
- **Wave 10A (evidence):** Should detect both malicious packages
- **Wave 10B (clean baseline):** Should flag 0 packages (previously moment.js was flagged)
- **Waves 10C-F (hunt):** Should only flag truly suspicious packages (<5% detection rate expected)

---

## Next Steps After Wave 10

### 1. Validate Results

```bash
# Check final summary
cat reports/wave10/*/summary.md

# Check for any flagged packages
grep -r "flagged as malicious" logs/wave10-*.log
```

### 2. Run Wave 11 (Evidence Validation)

```bash
./target/release/glassware campaign run campaigns/wave11-evidence-validation.toml
```

Wave 11 validates detection against our synthetic evidence packages:
- blockchain_c2 (4 packages)
- combined (4 packages)
- exfiltration (4 packages)
- steganography (4 packages)
- time_delay (3 packages)

### 3. Run Wave 12 (5000 Packages)

Large-scale production scan:
```bash
./target/release/glassware campaign run campaigns/wave12-5000pkg.toml
```

Expected duration: ~25-30 minutes

### 4. Create New Tag

If all waves pass:
```bash
git tag v0.57.0-longwave-ready
git push origin v0.57.0-longwave-ready
```

---

## Directory Structure

```
glassworks-v0.57.0-longwave/
├── campaigns/
│   ├── wave6.toml              # Calibration (skip - empty package lists)
│   ├── wave7-real-hunt.toml    # Real hunt (~100 packages)
│   ├── wave8-expanded-hunt.toml # Expanded hunt (~500 packages)
│   ├── wave9-500plus.toml      # 500+ packages
│   ├── wave10-1000plus.toml    # CURRENT: 1000+ packages
│   ├── wave11-evidence-validation.toml
│   └── wave12-5000pkg.toml
├── logs/                        # Campaign logs
├── reports/                     # Generated reports
├── evidence/                    # Evidence packages (23 tarballs)
├── target/release/glassware    # Main binary
└── monitor-wave10.sh           # Monitoring script
```

---

## Configuration

### LLM Settings

Tier 1 (Cerebras - fast triage):
- Enabled for score >= 6.0
- Reduces API calls by ~80%

Tier 2 (NVIDIA - deep analysis):
- Enabled for score >= 7.0
- Models: Qwen3.5-397B, Kimi K2.5, GLM-5, Llama-3.3-70B

### Performance

- Concurrency: 20 parallel operations
- NPM rate limit: 10.0 requests/sec
- Cache enabled: 7 day TTL

---

## Troubleshooting

### Campaign Stuck

```bash
# Check process
ps aux | grep glassware

# Kill if needed
kill <PID>

# Check cache
ls -la .glassware-orchestrator-cache.db

# Clear cache if corrupted
rm -f .glassware-orchestrator-cache.db*
```

### High FP Rate

If Wave 10B (clean baseline) shows >5% detection:
1. Check which packages flagged
2. Review if they're truly clean
3. May need additional detector tuning

### Evidence Not Detected

If Wave 10A doesn't detect evidence:
1. Check package versions (may need to update to match evidence tarballs)
2. Verify detector thresholds not too high
3. Review threat score calculations

---

## Contact & Resources

- **Original Repo:** https://github.com/samgrowls/glassworks
- **Autoresearch PR:** https://github.com/samgrowls/glassworks/pull/new/autoresearch/fp-rate-tuning-2026-03-25
- **Documentation:** See `AUTORESEARCH-FP-FIX-SUMMARY.md` in parent directory

---

**Last Updated:** 2026-03-25 17:31 UTC  
**Status:** Wave 10 campaign running
