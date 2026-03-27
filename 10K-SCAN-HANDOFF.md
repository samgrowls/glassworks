# 10,000 Package Scan - Handoff Instructions

**Date:** 2026-03-27
**Version:** v0.78.0-10k-ready
**Estimated Duration:** 8-12 hours on standard VM, 3-5 hours on high-performance VM

---

## Executive Summary

This document provides complete instructions for running a 10,000 package GlassWorm detection scan on a high-performance VM. The scan uses the `glassware` campaign orchestration system with context-aware file filtering.

**Key Configuration Fix Applied:**
- Obfuscation detector moved from Tier 1 to Tier 2
- Now requires Tier 1 signal (invisible chars or GlassWorm pattern) for high scores
- Prevents false positives from minified/bundled code

---

## Prerequisites

### Target VM Requirements

| Resource | Minimum | Recommended |
|----------|---------|-------------|
| **CPU** | 4 cores | 8+ cores |
| **RAM** | 8 GB | 16+ GB |
| **Disk** | 50 GB free | 100+ GB SSD |
| **Network** | 10 Mbps | 100+ Mbps |
| **OS** | Linux (Ubuntu 20.04+) | Ubuntu 22.04 LTS |

### Software Requirements

- **Rust:** 1.70+ (install via rustup)
- **Git:** For repository checkout
- **Build tools:** `build-essential` package

---

## Setup Instructions

### Step 1: Clone Repository

```bash
cd ~
git clone https://github.com/samgrowls/glassworks.git
cd glassworks
git checkout v0.78.0-10k-ready
```

### Step 2: Install Rust (if not installed)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env
rustup default stable
rustc --version  # Should show 1.70+
```

### Step 3: Install Build Dependencies

```bash
sudo apt update
sudo apt install -y build-essential pkg-config libssl-dev
```

### Step 4: Build Release Binary

```bash
# This takes 15-20 minutes
cargo build -p glassware --release

# Verify build
./target/release/glassware --version
```

### Step 5: Create Campaign Configuration

The 10k scan uses multiple wave files. Create `campaigns/wave-10k-master.toml`:

```toml
[campaign]
name = "10k Package GlassWorm Hunt"
description = "Large-scale GlassWorm detection across 10,000 npm packages"
created_by = "security-team"
priority = "critical"
tags = ["hunt", "10k", "large-scale"]

[settings]
concurrency = 20  # Adjust based on CPU cores
rate_limit_npm = 10.0
cache_enabled = true
cache_ttl_days = 7

[settings.scoring]
malicious_threshold = 7.0
suspicious_threshold = 4.0
tier_config = { mode = "tiered" }

# Tier 1: Primary signals (REQUIRED for high scores)
[[settings.scoring.tiers]]
tier = 1
detectors = ["invisible_char", "glassware_pattern"]
threshold = 0.0
weight_multiplier = 1.0

# Tier 2: Secondary confirmation
[[settings.scoring.tiers]]
tier = 2
detectors = ["header_c2", "exfil_schema", "blockchain_c2", "obfuscation"]
threshold = 2.0
weight_multiplier = 0.8

# Tier 3: Behavioral indicators
[[settings.scoring.tiers]]
tier = 3
detectors = ["locale_geofencing", "time_delay_sandbox_evasion"]
threshold = 10.0
weight_multiplier = 0.8

[settings.scoring.weights]
invisible_char = 6.0
glassware_pattern = 10.0
obfuscation = 10.0
blockchain_c2 = 5.0
header_c2 = 4.0
exfil_schema = 6.0
locale_geofencing = 7.0
time_delay_sandbox_evasion = 7.0

[settings.llm]
tier1_enabled = false  # Disabled due to rate limiting
tier2_enabled = false  # Enable for case studies only

[settings.output]
formats = ["json", "markdown"]
evidence_collection = true
evidence_dir = "evidence/10k-scan"
report_dir = "reports/10k-scan"

# Define waves with package categories
[[waves]]
id = "wave_1"
name = "Evidence Validation"
depends_on = []
mode = "validate"

[[waves.sources]]
type = "tarballs"
list = [
    "evidence/iflow-mcp-watercrawl-watercrawl-mcp-1.3.4.tgz",
]

[waves.expectations]
must_flag_all = true
min_threat_score = 7.0

[[waves]]
id = "wave_2"
name = "Web Frameworks (2500 packages)"
depends_on = ["wave_1"]
mode = "hunt"

[[waves.sources]]
type = "npm_search"
keywords = ["react"]
samples_per_keyword = 500
max_downloads = 500000

[[waves.sources]]
type = "npm_search"
keywords = ["vue"]
samples_per_keyword = 500
max_downloads = 500000

[[waves.sources]]
type = "npm_search"
keywords = ["angular"]
samples_per_keyword = 500
max_downloads = 500000

[[waves.sources]]
type = "npm_search"
keywords = ["svelte"]
samples_per_keyword = 500
max_downloads = 500000

[[waves.sources]]
type = "npm_search"
keywords = ["next"]
samples_per_keyword = 500
max_downloads = 500000

[waves.expectations]
must_flag_all = false
max_malicious_count = 50

# Add more waves for other categories...
```

### Step 6: Create Log Directory

```bash
mkdir -p logs/10k-scan
mkdir -p reports/10k-scan
mkdir -p evidence/10k-scan
```

---

## Running the Scan

### Start the Campaign

```bash
cd ~/glassworks

# Start campaign with logging
nohup ./target/release/glassware campaign run campaigns/wave-10k-master.toml \
    > logs/10k-scan/campaign.log 2>&1 &

# Record PID for monitoring
echo $! > logs/10k-scan/campaign.pid
```

### Monitor Progress

```bash
# View live progress
tail -f logs/10k-scan/campaign.log | grep -E "Wave.*completed|packages scanned|malicious"

# Check package count
grep -c "Package.*scanned:" logs/10k-scan/campaign.log

# Check malicious count
grep -c "flagged as malicious" logs/10k-scan/campaign.log

# Check process status
ps aux | grep glassware | grep -v grep
```

### Monitor System Resources

```bash
# CPU and memory usage
watch -n 5 'ps aux | grep glassware | grep -v grep'

# Disk usage
df -h .

# Network activity
iftop -P -n 2>/dev/null || nethogs 2>/dev/null
```

---

## Expected Results

### Scan Statistics

| Metric | Expected Value |
|--------|----------------|
| **Total Packages** | 10,000 |
| **Scan Duration** | 8-12 hours |
| **Packages/Hour** | ~800-1200 |
| **Flagged Rate** | 5-15% (500-1500 packages) |
| **Malicious Rate** | 0.1-0.5% (10-50 packages) |

### Output Files

```
glassworks/
├── logs/10k-scan/
│   └── campaign.log          # Full campaign log
├── reports/10k-scan/
│   ├── campaign-summary.md   # Markdown summary
│   └── campaign-results.json # JSON results
├── evidence/10k-scan/
│   └── [flagged packages]    # Preserved evidence
└── .glassware-checkpoints.db  # Progress database
```

---

## Handling Interruptions

### Resume Interrupted Scan

```bash
# If scan is interrupted, resume from checkpoint
./target/release/glassware campaign resume <case-id>

# Find case-id from logs
grep "case:" logs/10k-scan/campaign.log | tail -1
```

### Cancel Scan

```bash
# Graceful shutdown
kill $(cat logs/10k-scan/campaign.pid)

# Force kill if needed
pkill -9 glassware
```

---

## Post-Scan Actions

### 1. Generate Report

```bash
./target/release/glassware campaign report <case-id>
```

### 2. Review Malicious Packages

```bash
# List all malicious packages
grep "flagged as malicious" logs/10k-scan/campaign.log

# Extract package names
grep "flagged as malicious" logs/10k-scan/campaign.log | \
    sed 's/.*Package \([^ ]*\) flagged.*/\1/' > malicious-packages.txt
```

### 3. Preserve Evidence

All flagged packages are automatically saved to `evidence/10k-scan/`. Review and document findings.

### 4. Export Results

```bash
# Copy results for analysis
scp -r ~/glassworks/reports/10k-scan user@analysis-machine:/path/to/results
scp -r ~/glassworks/evidence/10k-scan user@analysis-machine:/path/to/evidence
```

---

## Troubleshooting

### Issue: Out of Disk Space

```bash
# Clear npm cache
npm cache clean --force

# Clear glassware cache
rm -f .glassware-orchestrator-cache.db

# Remove old checkpoints
rm -f .glassware-checkpoints.db.*
```

### Issue: Rate Limiting from npm

```bash
# Reduce concurrency in config
# Edit campaigns/wave-10k-master.toml
[settings]
concurrency = 10  # Reduce from 20

# Reduce rate limit
rate_limit_npm = 5.0  # Reduce from 10.0
```

### Issue: Scan Too Slow

```bash
# Increase concurrency (if CPU allows)
[settings]
concurrency = 30  # Increase from 20

# Use release build (already specified above)
```

### Issue: Too Many False Positives

Check tiered scoring config:
```toml
# Ensure obfuscation is Tier 2, not Tier 1
[[settings.scoring.tiers]]
tier = 2
detectors = ["header_c2", "exfil_schema", "blockchain_c2", "obfuscation"]
```

---

## Contact & Support

**For Issues:**
- Check logs: `logs/10k-scan/campaign.log`
- Review this document
- Contact security team

**Expected Completion:** 8-12 hours from start

**Success Criteria:**
- ✅ All 10,000 packages scanned
- ✅ Evidence preserved for flagged packages
- ✅ Report generated
- ✅ Malicious packages documented

---

**Last Updated:** 2026-03-27
**Version:** v0.78.0-10k-ready
**Status:** READY FOR EXECUTION
