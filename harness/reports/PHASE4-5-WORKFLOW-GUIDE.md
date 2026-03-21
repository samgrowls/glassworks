# Phase 4/5 Implementation Guide

**Version History Scanning Workflow**

**Date:** 2026-03-21  
**Status:** ✅ Implementation Complete

---

## Overview

This guide covers the complete workflow for version history scanning:

1. **Sample packages** from diverse categories
2. **Scan multiple versions** of each package
3. **Track results** in SQLite database
4. **Analyze findings** and generate reports

---

## Quick Start

```bash
cd harness

# 1. Sample 500 packages
python3 version_sampler.py \
  --output packages-500.txt \
  --samples 50 \
  --days 30

# 2. Run background scanner
python3 background_scanner.py \
  --packages packages-500.txt \
  --policy last-10 \
  --output results.db \
  --state scan-state.json \
  --log scan.log \
  --workers 5

# 3. View results
sqlite3 results.db "SELECT * FROM version_scans WHERE is_malicious = 1;"
```

---

## Step 1: Package Sampling

### Basic Usage

```bash
python3 version_sampler.py \
  --output packages.txt \
  --samples 50 \
  --days 30
```

### Options

| Option | Default | Description |
|--------|---------|-------------|
| `--output, -o` | (required) | Output file path |
| `--samples` | 50 | Samples per category |
| `--categories` | all | Categories to sample |
| `--days` | 30 | Sample from last N days |
| `--delay` | 0.5 | Delay between searches (seconds) |
| `--format` | plain | Output format (plain, json, csv) |
| `--include-popular` | false | Include popular packages |
| `--include-new` | false | Include new packages |

### Categories

Available categories:
- `native-build` - node-gyp, bindings, prebuild
- `install-scripts` - preinstall, postinstall, husky
- `web-frameworks` - react, vue, angular, svelte
- `backend` - express, fastify, koa, hapi
- `database` - mongoose, sequelize, prisma
- `devtools` - eslint, prettier, typescript
- `testing` - jest, mocha, vitest, cypress
- `ai-ml` - ai, llm, langchain, openai
- `utils` - lodash, async, moment, axios
- `logging` - winston, pino, log4js
- `crypto` - bcrypt, jsonwebtoken, jose
- `security` - helmet, cors, rate-limit

### Examples

```bash
# Sample from specific categories
python3 version_sampler.py \
  --output ai-packages.txt \
  --categories ai-ml utils \
  --samples 100

# Include popular packages
python3 version_sampler.py \
  --output popular.txt \
  --include-popular \
  --samples 50

# JSON output with metadata
python3 version_sampler.py \
  --output packages.json \
  --format json \
  --samples 100
```

---

## Step 2: Background Scanning

### Basic Usage

```bash
python3 background_scanner.py \
  --packages packages.txt \
  --policy last-10 \
  --output results.db \
  --workers 5
```

### Options

| Option | Default | Description |
|--------|---------|-------------|
| `--packages, -p` | (required) | Package list file |
| `--policy` | last-10 | Version policy |
| `--output, -o` | (required) | Output database |
| `--state, -s` | scan-state.json | Checkpoint state file |
| `--log, -l` | scan.log | Log file |
| `--workers, -w` | 5 | Parallel workers |
| `--resume` | false | Resume from state file |

### Version Policies

| Policy | Format | Example | Description |
|--------|--------|---------|-------------|
| Last N | `last-N` | `last-10` | Scan last 10 versions |
| Last Days | `last-Nd` | `last-180d` | Scan versions from last 180 days |
| All | `all` | `all` | Scan all versions |

### Examples

```bash
# Scan last 10 versions with 10 workers
python3 background_scanner.py \
  --packages packages.txt \
  --policy last-10 \
  --output results.db \
  --workers 10

# Scan all versions (use cautiously!)
python3 background_scanner.py \
  --packages suspicious.txt \
  --policy all \
  --output all-versions.db

# Resume interrupted scan
python3 background_scanner.py \
  --packages packages.txt \
  --policy last-10 \
  --output results.db \
  --state scan-state.json \
  --resume
```

---

## Step 3: Monitoring Progress

### Console Output

```
[2026-03-21T06:30:00] Package 5/500 | Versions: 50 (failures: 2) | Findings: 15 | Malicious: 1 | Rate: 2.5 ver/s
[2026-03-21T06:30:10] Package 10/500 | Versions: 100 (failures: 3) | Findings: 25 | Malicious: 2 | Rate: 2.8 ver/s
```

### Log File

```bash
# View recent entries
tail -f scan.log

# Search for malicious
grep "🚨" scan.log

# Count findings
grep -c "✅" scan.log
```

### State File

```bash
# Check current state
cat scan-state.json | jq

# Check progress
cat scan-state.json | jq '.versions_scanned'
```

---

## Step 4: Analyzing Results

### SQLite Queries

```bash
# Open database
sqlite3 results.db

# Find all malicious versions
SELECT package_name, version, threat_score, findings_count
FROM version_scans
WHERE is_malicious = 1
ORDER BY threat_score DESC;

# Findings by package
SELECT package_name, SUM(findings_count) as total_findings
FROM version_scans
GROUP BY package_name
ORDER BY total_findings DESC;

# Scan timeline
SELECT date(scan_timestamp) as date, COUNT(*) as versions_scanned
FROM version_scans
GROUP BY date
ORDER BY date;

# Version-specific findings
SELECT package_name, version, findings_count
FROM version_scans
WHERE findings_count > 0
ORDER BY package_name, version;

# Statistics
SELECT 
  COUNT(*) as total_versions,
  SUM(findings_count) as total_findings,
  COUNT(CASE WHEN is_malicious = 1 THEN 1 END) as malicious_versions,
  AVG(threat_score) as avg_threat_score
FROM version_scans;
```

### Export Results

```bash
# Export to CSV
sqlite3 -header -csv results.db \
  "SELECT * FROM version_scans WHERE is_malicious = 1;" \
  > malicious.csv

# Export to JSON
sqlite3 results.db \
  "SELECT json_group_array(json_object('package', package_name, 'version', version, 'findings', findings_count)) FROM version_scans WHERE is_malicious = 1;" \
  > malicious.json
```

---

## Step 5: Generate Reports

### Automatic Report

The scanner generates `version-scan-report.md` automatically:

```markdown
# Version History Scan Report

**Generated:** 2026-03-21T06:30:00
**Policy:** last-10
**Workers:** 5

## Summary

| Metric | Value |
|--------|-------|
| Total packages | 500 |
| Total versions scanned | 5000 |
| Malicious versions | 3 |
| Malicious packages | 2 |
| Total findings | 45 |

## 🚨 Malicious Versions Detected

| Package | Version | Threat Score | Findings |
|---------|---------|--------------|----------|
| suspicious-pkg | 2.3.4 | 9.2 | 15 |
| evil-utils | 1.0.1 | 8.5 | 8 |
```

### Custom Analysis Script

```python
#!/usr/bin/env python3
"""Custom analysis script"""

import sqlite3
import json

conn = sqlite3.connect("results.db")
cursor = conn.cursor()

# Find packages with version-specific malicious code
cursor.execute("""
    SELECT package_name, 
           COUNT(*) as version_count,
           SUM(CASE WHEN is_malicious = 1 THEN 1 ELSE 0 END) as malicious_count
    FROM version_scans
    GROUP BY package_name
    HAVING malicious_count > 0 AND malicious_count < version_count
    ORDER BY malicious_count DESC
""")

print("Packages with version-specific malicious code:")
for pkg, total, malicious in cursor.fetchall():
    print(f"  {pkg}: {malicious}/{total} versions malicious")

conn.close()
```

---

## Complete Workflow Example

### 1. Sample Packages

```bash
cd harness

# Sample 500 packages from high-risk categories
python3 version_sampler.py \
  --output high-risk-packages.txt \
  --categories native-build install-scripts ai-ml crypto \
  --samples 125 \
  --days 30 \
  --format plain
```

### 2. Start Background Scan

```bash
# Start scan with checkpointing
python3 background_scanner.py \
  --packages high-risk-packages.txt \
  --policy last-10 \
  --output high-risk-results.db \
  --state high-risk-state.json \
  --log high-risk-scan.log \
  --workers 10 &

# Save PID for later
echo $! > scanner.pid
```

### 3. Monitor Progress

```bash
# Watch log file
tail -f high-risk-scan.log

# Check state
watch -n 60 'cat high-risk-state.json | jq .versions_scanned'

# Check database
watch -n 60 'sqlite3 high-risk-results.db "SELECT COUNT(*) FROM version_scans WHERE is_malicious = 1;"'
```

### 4. Analyze Results

```bash
# After scan completes
sqlite3 high-risk-results.db <<EOF
.mode column
.headers on

-- Summary
SELECT 
  COUNT(*) as total,
  SUM(findings_count) as findings,
  COUNT(CASE WHEN is_malicious = 1 THEN 1 END) as malicious
FROM version_scans;

-- Top malicious packages
SELECT package_name, version, threat_score, findings_count
FROM version_scans
WHERE is_malicious = 1
ORDER BY threat_score DESC
LIMIT 10;
EOF
```

### 5. Generate Report

```bash
# Report is auto-generated
cat version-scan-report.md

# Or create custom report
python3 custom_analysis.py > custom-report.md
```

---

## Troubleshooting

### Rate Limiting

**Error:** `429 Too Many Requests`

**Fix:**
```bash
# Reduce workers
python3 background_scanner.py --workers 3 ...

# Add delay in sampler
python3 version_sampler.py --delay 2.0 ...
```

### Scan Timeout

**Error:** `Timeout after 120s`

**Fix:**
```bash
# Reduce version count
python3 background_scanner.py --policy last-5 ...

# Increase timeout in background_scanner.py (line 171)
timeout=300  # Change from 120 to 300
```

### Database Locked

**Error:** `database is locked`

**Fix:**
```bash
# Close other connections
sqlite3 results.db ".quit"

# Or use WAL mode
sqlite3 results.db "PRAGMA journal_mode = WAL;"
```

### Resume Failed Scan

```bash
# Check state
cat scan-state.json | jq

# Resume from checkpoint
python3 background_scanner.py \
  --packages packages.txt \
  --policy last-10 \
  --output results.db \
  --state scan-state.json \
  --resume
```

---

## Performance Tips

### Optimal Worker Count

| System | Recommended Workers |
|--------|---------------------|
| 2 CPU, 4GB RAM | 3-5 |
| 4 CPU, 8GB RAM | 5-10 |
| 8 CPU, 16GB RAM | 10-20 |
| 16+ CPU, 32GB+ RAM | 20-50 |

### Batch Processing

```bash
# Split into batches
split -l 100 packages-500.txt batch-

# Process batches in parallel
for batch in batch-*; do
    python3 background_scanner.py \
      --packages "$batch" \
      --policy last-10 \
      --output "${batch}.db" \
      --workers 5 &
done
wait

# Merge results
for db in batch-*.db; do
    sqlite3 results.db "ATTACH '$db' AS db; INSERT INTO version_scans SELECT * FROM db.version_scans; DETACH db;"
done
```

### Caching

```bash
# Enable caching (automatic)
# Cache location: /tmp/glassware-version-cache.db

# Clear cache if needed
rm /tmp/glassware-version-cache.db
```

---

## Expected Performance

| Scenario | Packages | Versions | Time | Cost (LLM) |
|----------|----------|----------|------|------------|
| Small scan | 50 | 500 | ~30 min | $0.05 |
| Medium scan | 500 | 5,000 | ~5 hours | $0.50 |
| Large scan | 5,000 | 50,000 | ~2 days | $5.00 |

**Note:** Times vary based on workers, network, and LLM usage.

---

**End of Guide**
