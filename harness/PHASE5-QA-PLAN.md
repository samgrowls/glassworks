# Phase 5 Large Scan - QA Plan

**Version:** v0.8.8.0  
**Date:** 2026-03-21  
**Status:** 🔄 Running

---

## Objective

Run a large-scale real-world version history scan to validate Phase 5 implementation.

---

## Scan Configuration

### Parameters

| Parameter | Value |
|-----------|-------|
| Packages | ~200 (4 categories × 40 samples + popular) |
| Versions per package | last-10 |
| Total versions | ~2,000 |
| Workers | 10 |
| Estimated time | ~15-20 minutes |

### Categories

1. **ai-ml** - AI/ML packages (high-risk, trending)
2. **native-build** - Native build tools (install scripts)
3. **utils** - Utility packages (widely used)
4. **crypto** - Crypto packages (legitimate crypto usage)

---

## Commands

### 1. Sample Packages

```bash
cd harness

# Running in background
nohup python3 version_sampler.py \
  --output packages-500.txt \
  --samples 40 \
  --categories ai-ml native-build utils crypto \
  --days 30 \
  --include-popular \
  > /tmp/sampler-output.log 2>&1 &

# Check progress
tail -f /tmp/sampler-output.log
```

### 2. Run Background Scan

```bash
# Start scan
python3 background_scanner.py \
  --packages packages-500.txt \
  --policy last-10 \
  --output phase5-qa-results.db \
  --state phase5-qa-state.json \
  --log phase5-qa.log \
  --workers 10

# Or run in background for very long scans
nohup python3 background_scanner.py \
  --packages packages-500.txt \
  --policy last-10 \
  --output phase5-qa-results.db \
  --workers 10 \
  > /tmp/scan-output.log 2>&1 &
```

### 3. Monitor Progress

```bash
# Watch log
tail -f phase5-qa.log

# Check state
cat phase5-qa-state.json | jq .versions_scanned

# Query results in real-time
sqlite3 phase5-qa-results.db "SELECT COUNT(*) FROM version_scans;"
```

### 4. Analyze Results

```bash
# Summary statistics
sqlite3 phase5-qa-results.db <<EOF
SELECT 
  COUNT(*) as total_scans,
  SUM(findings_count) as total_findings,
  COUNT(CASE WHEN is_malicious = 1 THEN 1 END) as malicious,
  AVG(threat_score) as avg_threat_score
FROM version_scans;
EOF

# Find malicious versions
sqlite3 phase5-qa-results.db \
  "SELECT package_name, version, threat_score, findings_count FROM version_scans WHERE is_malicious = 1 ORDER BY threat_score DESC;"

# View report
cat version-scan-report.md
```

---

## Success Criteria

- [ ] Sampler completes without errors
- [ ] Background scanner processes all packages
- [ ] Checkpoint/resume works (test by interrupting and resuming)
- [ ] Database populated correctly
- [ ] Log file shows progress
- [ ] Report generated
- [ ] No memory leaks or crashes
- [ ] Performance within expected range (~4 ver/s)

---

## Expected Performance

| Metric | Expected |
|--------|----------|
| Sampling time | ~5-10 min |
| Scan time (200 pkgs × 10 ver) | ~15-20 min |
| Scan rate | ~4 ver/s |
| Database size | ~50-100 MB |
| Log file size | ~10-20 MB |

---

## Troubleshooting

### Rate Limiting

**Symptom:** 429 errors in log

**Fix:**
```bash
# Reduce workers
python3 background_scanner.py --workers 5 ...
```

### npm 404 Errors

**Symptom:** Many "Package not found" errors

**Cause:** Old versions unpublished

**Fix:** Normal behavior, use `last-N` policy

### Scan Timeout

**Symptom:** Individual scans timing out

**Fix:**
```bash
# Reduce version count
--policy last-5

# Or increase timeout in background_scanner.py (line 171)
timeout=300
```

---

## Files Generated

| File | Purpose |
|------|---------|
| `packages-500.txt` | Sampled package list |
| `phase5-qa-results.db` | SQLite scan results |
| `phase5-qa-state.json` | Checkpoint state |
| `phase5-qa.log` | Progress log |
| `version-scan-report.md` | Auto-generated report |

---

## Post-Scan Analysis

### 1. Performance Analysis

```bash
# Calculate actual scan rate
sqlite3 phase5-qa-results.db <<EOF
SELECT 
  COUNT(*) as total_versions,
  (julianday(MAX(scan_timestamp)) - julianday(MIN(scan_timestamp))) * 86400 as duration_seconds,
  COUNT(*) / ((julianday(MAX(scan_timestamp)) - julianday(MIN(scan_timestamp))) * 86400) as versions_per_second
FROM version_scans;
EOF
```

### 2. Finding Analysis

```bash
# Findings by category
sqlite3 phase5-qa-results.db <<EOF
SELECT p.category, SUM(v.findings_count) as findings
FROM version_scans v
JOIN packages p ON v.package_name = p.name
GROUP BY p.category
ORDER BY findings DESC;
EOF
```

### 3. Malicious Package Review

```bash
# Export malicious versions
sqlite3 -header -csv phase5-qa-results.db \
  "SELECT package_name, version, threat_score, findings_count FROM version_scans WHERE is_malicious = 1;" \
  > malicious-versions.csv
```

---

## QA Checklist

### Pre-Scan
- [ ] Environment ready (Python 3.10+, SQLite3)
- [ ] glassware-orchestrator built
- [ ] Sufficient disk space (>1GB)
- [ ] Network connectivity

### During Scan
- [ ] Log file updating
- [ ] State file checkpointing
- [ ] No memory issues
- [ ] Reasonable scan rate

### Post-Scan
- [ ] All packages processed
- [ ] Database consistent
- [ ] Report generated
- [ ] Performance within expectations
- [ ] No data loss on checkpoint/resume test

---

## Contact

For issues or questions during the scan, check:
- `phase5-qa.log` for errors
- `phase5-qa-state.json` for progress
- GitHub Issues for bug reports

---

**End of QA Plan**
