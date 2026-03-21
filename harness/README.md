# Glassware Harness - Version History Scanning

**Version:** v0.8.8.0  
**Last Updated:** 2026-03-21

---

## Quick Start

### 1. Sample Packages

```bash
cd harness

# Sample 500 packages from diverse categories
python3 version_sampler.py \
  --output packages-500.txt \
  --samples 50 \
  --categories ai-ml native-build web-frameworks utils \
  --days 30
```

### 2. Run Background Scan

```bash
# Scan last 10 versions of each package
python3 background_scanner.py \
  --packages packages-500.txt \
  --policy last-10 \
  --output results.db \
  --state scan-state.json \
  --log scan.log \
  --workers 5
```

### 3. Monitor Progress

```bash
# Watch log file
tail -f scan.log

# Check state
cat scan-state.json | jq .versions_scanned
```

### 4. Analyze Results

```bash
# Find malicious versions
sqlite3 results.db "SELECT package_name, version, threat_score FROM version_scans WHERE is_malicious = 1;"

# Generate report
cat version-scan-report.md
```

---

## Tools

### version_sampler.py

Samples diverse npm packages for scanning.

**Options:**
- `--output, -o` - Output file (required)
- `--samples` - Samples per category (default: 50)
- `--categories` - Categories to sample (default: all)
- `--days` - Sample from last N days (default: 30)
- `--format` - Output format: plain, json, csv (default: plain)

**Example:**
```bash
python3 version_sampler.py \
  --output packages.txt \
  --samples 100 \
  --categories ai-ml crypto utils \
  --days 60
```

### background_scanner.py

Long-running version history scanner with checkpoint/resume.

**Options:**
- `--packages, -p` - Package list file (required)
- `--policy` - Version policy: last-10, last-180d, all (default: last-10)
- `--output, -o` - SQLite database output (required)
- `--state, -s` - Checkpoint state file (default: scan-state.json)
- `--log, -l` - Log file (default: scan.log)
- `--workers, -w` - Parallel workers (default: 5)
- `--resume` - Resume from checkpoint

**Example:**
```bash
python3 background_scanner.py \
  --packages packages.txt \
  --policy last-10 \
  --output results.db \
  --workers 10
```

---

## Version Policies

| Policy | Format | Example | Description |
|--------|--------|---------|-------------|
| Last N | `last-N` | `last-10` | Scan last 10 versions |
| Last Days | `last-Nd` | `last-180d` | Scan versions from last 180 days |
| All | `all` | `all` | Scan all versions |

---

## Database Schema

```sql
-- Main scan results
CREATE TABLE version_scans (
    id INTEGER PRIMARY KEY,
    package_name TEXT NOT NULL,
    version TEXT NOT NULL,
    scan_timestamp DATETIME NOT NULL,
    findings_count INTEGER NOT NULL,
    threat_score REAL NOT NULL,
    is_malicious BOOLEAN NOT NULL,
    scan_result_json TEXT,
    error TEXT,
    UNIQUE(package_name, version)
);
```

---

## Common Queries

```sql
-- Find malicious versions
SELECT package_name, version, threat_score, findings_count
FROM version_scans
WHERE is_malicious = 1
ORDER BY threat_score DESC;

-- Findings by package
SELECT package_name, SUM(findings_count) as total_findings
FROM version_scans
GROUP BY package_name
ORDER BY total_findings DESC;

-- Scan statistics
SELECT 
  COUNT(*) as total_scans,
  SUM(findings_count) as total_findings,
  COUNT(CASE WHEN is_malicious = 1 THEN 1 END) as malicious
FROM version_scans;
```

---

## Troubleshooting

### Rate Limiting

**Error:** `429 Too Many Requests`

**Fix:** Reduce workers
```bash
python3 background_scanner.py --workers 3 ...
```

### Scan Timeout

**Error:** `Timeout after 120s`

**Fix:** Use `last-N` policy instead of `all`
```bash
python3 background_scanner.py --policy last-5 ...
```

### Resume Failed Scan

```bash
python3 background_scanner.py \
  --packages packages.txt \
  --policy last-10 \
  --output results.db \
  --state scan-state.json \
  --resume
```

---

## Performance

| Scenario | Packages | Versions | Time | Workers |
|----------|----------|----------|------|---------|
| Small | 50 | 500 | ~2 min | 5 |
| Medium | 500 | 5,000 | ~20 min | 10 |
| Large | 5,000 | 50,000 | ~3 hours | 20 |

---

## Files Generated

| File | Purpose |
|------|---------|
| `packages-*.txt` | Sampled package list |
| `results.db` | SQLite scan results |
| `scan-state.json` | Checkpoint state |
| `scan.log` | Progress log |
| `version-scan-report.md` | Auto-generated report |

---

**For complete documentation, see:** `../USER-GUIDE.md`
