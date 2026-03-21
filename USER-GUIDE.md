# Glassware Orchestrator - Complete Reference

**Version:** v0.8.75  
**Last Updated:** 2026-03-21

---

## Table of Contents

1. [Quick Start](#quick-start)
2. [Installation](#installation)
3. [CLI Commands](#cli-commands)
4. [Scan Management](#scan-management)
5. [Version Scanning](#version-scanning)
6. [LLM Analysis](#llm-analysis)
7. [Configuration](#configuration)
8. [Output Formats](#output-formats)
9. [Troubleshooting](#troubleshooting)
10. [Best Practices](#best-practices)

---

## Quick Start

```bash
# Install
cargo install --path glassware-orchestrator

# Scan a package
glassware-orchestrator scan-npm express

# Scan with version history
glassware-orchestrator scan-npm --versions last-10 lodash

# Check scan history
glassware-orchestrator scan-list
```

---

## Installation

### From Source
```bash
cd glassworks
cargo build --release -p glassware-orchestrator
./target/release/glassware-orchestrator --help
```

### Cargo Install
```bash
cargo install --path glassware-orchestrator
```

### Verify Installation
```bash
glassware-orchestrator --version
# Output: glassware-orchestrator 0.8.75
```

---

## CLI Commands

### scan-npm
Scan npm packages.

```bash
# Single package
glassware-orchestrator scan-npm <package>

# Multiple packages
glassware-orchestrator scan-npm express lodash axios

# With version history
glassware-orchestrator scan-npm --versions last-10 express

# With LLM analysis
glassware-orchestrator --llm scan-npm suspicious-pkg
```

**Options:**
- `--versions <POLICY>` - Version policy (last-10, last-180d, all, or comma-separated)

### scan-github
Scan GitHub repositories.

```bash
# Single repo
glassware-orchestrator scan-github owner/repo

# Multiple repos
glassware-orchestrator scan-github owner/repo org/project

# With specific branch
glassware-orchestrator scan-github -r main owner/repo
```

**Options:**
- `-r, --ref <REF>` - Git reference (branch, tag, or commit)

### scan-file
Scan packages from a file list.

```bash
# One package per line
glassware-orchestrator scan-file packages.txt

# With version scanning
glassware-orchestrator scan-file packages.txt --versions last-5
```

**File Format:**
```
express
lodash@4.17.21
@scope/package
owner/repo
```

### scan-list
List scan history.

```bash
# All scans
glassware-orchestrator scan-list

# Running scans only
glassware-orchestrator scan-list --status running

# Last 50 completed
glassware-orchestrator scan-list --status completed --limit 50
```

**Options:**
- `--status <STATUS>` - Filter by status (running, completed, failed, cancelled)
- `--limit <N>` - Limit results (default: 20)

### scan-show
Show scan details.

```bash
glassware-orchestrator scan-show <scan-id>
```

### scan-cancel
Cancel a running scan.

```bash
glassware-orchestrator scan-cancel <scan-id>
```

### cache-stats
Show cache statistics.

```bash
glassware-orchestrator cache-stats

# With JSON output
glassware-orchestrator --format json cache-stats

# Clear cache after showing stats
glassware-orchestrator cache-stats --clear
```

### cache-cleanup
Clean up expired cache entries.

```bash
glassware-orchestrator cache-cleanup
```

---

## Scan Management

### View Scan Registry

The scan registry tracks all scans in `.glassware-scan-registry.json`:

```json
{
  "scans": [
    {
      "id": "4e12cab4-bd88-4fe0-961a-f70b14cbbc4d",
      "started_at": "2026-03-21T06:12:47.691808442Z",
      "completed_at": "2026-03-21T06:12:48.181995449Z",
      "status": "completed",
      "command": "scan-npm",
      "packages": ["express"],
      "version_policy": null,
      "findings_count": 0,
      "malicious_count": 0,
      "error": null
    }
  ]
}
```

### Query Scans

```bash
# Using jq
cat .glassware-scan-registry.json | jq '.scans[] | select(.status == "running")'

# Count total scans
cat .glassware-scan-registry.json | jq '.scans | length'

# Get failed scans
cat .glassware-scan-registry.json | jq '.scans[] | select(.status == "failed")'
```

---

## Version Scanning

### Version Policies

| Policy | Format | Example | Description |
|--------|--------|---------|-------------|
| Last N | `last-N` | `last-10` | Scan last N versions |
| Last Days | `last-Nd` | `last-180d` | Scan versions from last N days |
| All | `all` | `all` | Scan all versions |
| Specific | CSV | `1.0.0,2.0.0` | Scan specific versions |

### Examples

```bash
# Scan last 10 versions
glassware-orchestrator scan-npm --versions last-10 lodash

# Scan versions from last 6 months
glassware-orchestrator scan-npm --versions last-180d axios

# Scan all versions (use cautiously!)
glassware-orchestrator scan-npm --versions all suspicious-pkg

# Scan specific versions
glassware-orchestrator scan-npm --versions "1.0.0,1.1.0,2.0.0" pkg
```

### Output

```
============================================================
VERSION SCAN SUMMARY
============================================================
Packages scanned: 1
Total versions: 5
Total findings: 15
Malicious versions: 2
============================================================

🚨 Malicious versions detected!
```

---

## LLM Analysis

### Configuration

```bash
# Set environment variables
export GLASSWARE_LLM_BASE_URL="https://api.cerebras.ai/v1"
export GLASSWARE_LLM_API_KEY="csk-..."
export GLASSWARE_LLM_MODEL="llama-3.3-70b"

# Or use .env file
cp .env.example .env
# Edit .env with your credentials
```

### Usage

```bash
# Scan with LLM analysis
glassware-orchestrator --llm scan-npm suspicious-pkg

# With version scanning
glassware-orchestrator --llm scan-npm --versions last-5 pkg

# With specific severity
glassware-orchestrator --llm --severity high scan-npm pkg
```

### Supported Providers

| Provider | Base URL | Model | Cost |
|----------|----------|-------|------|
| Cerebras | `https://api.cerebras.ai/v1` | `llama-3.3-70b` | ~$0.10/1M tokens |
| Groq | `https://api.groq.com/openai/v1` | `llama-3.3-70b-versatile` | Free tier |
| NVIDIA NIM | `https://integrate.api.nvidia.com/v1` | `meta/llama-3.3-70b-instruct` | Pay/token |
| OpenAI | `https://api.openai.com/v1` | `gpt-4o` | ~$2.50/1M tokens |

---

## Configuration

### Global Options

| Option | Description | Default |
|--------|-------------|---------|
| `--format, -f` | Output format (pretty, json, sarif) | `pretty` |
| `--severity, -s` | Minimum severity (info, low, medium, high, critical) | `low` |
| `--concurrency, -c` | Max concurrent operations | `10` |
| `--cache-db` | Cache database path | `.glassware-orchestrator-cache.db` |
| `--no-cache` | Disable caching | `false` |
| `--llm` | Enable LLM analysis | `false` |
| `--quiet, -q` | Quiet mode (minimal output) | `false` |
| `--verbose` | Verbose mode | `false` |
| `--log-level` | Log level (trace, debug, info, warn, error) | `info` |

### Environment Variables

```bash
# LLM Configuration
GLASSWARE_LLM_BASE_URL=https://api.cerebras.ai/v1
GLASSWARE_LLM_API_KEY=csk-...
GLASSWARE_LLM_MODEL=llama-3.3-70b
GLASSWARE_LLM_RPM=30
GLASSWARE_LLM_TPM=60000

# GitHub Token (for private repos)
GITHUB_TOKEN=ghp_...

# Evidence Directory
GLASSWARE_EVIDENCE_DIR=data/evidence
```

---

## Output Formats

### Pretty (Default)
```
============================================================
SCAN SUMMARY
============================================================
Total packages scanned: 1
Malicious packages: 0
Total findings: 0
Average threat score: 0.00
============================================================

✅ No security issues detected
```

### JSON
```bash
glassware-orchestrator --format json scan-npm express
```

```json
{
  "summary": {
    "total_packages": 1,
    "malicious_packages": 0,
    "total_findings": 0,
    "average_threat_score": 0.0
  },
  "results": [...],
  "errors": []
}
```

### SARIF
```bash
glassware-orchestrator --format sarif scan-npm pkg > results.sarif
```

For GitHub Advanced Security integration.

---

## Troubleshooting

### LLM Not Working

**Error:** `--llm requires GLASSWARE_LLM_BASE_URL`

**Fix:**
```bash
export GLASSWARE_LLM_BASE_URL="https://api.cerebras.ai/v1"
export GLASSWARE_LLM_API_KEY="csk-..."
```

### Cache Issues

**Error:** `Failed to connect to cache database`

**Fix:**
```bash
# Remove corrupted cache
rm .glassware-orchestrator-cache.db

# Or use a different path
glassware-orchestrator --cache-db /tmp/cache.db scan-npm pkg
```

### Version Scan Failures

**Error:** `Package not found: pkg@1.0.0`

**Cause:** Old versions may be unpublished from npm.

**Fix:** Use `last-N` policy instead of `all`:
```bash
glassware-orchestrator scan-npm --versions last-10 pkg
```

### Rate Limiting

**Error:** `429 Too Many Requests`

**Fix:**
```bash
# Reduce concurrency
glassware-orchestrator --concurrency 5 scan-file packages.txt

# Add delay between requests
# (Use Python harness for better rate limit control)
```

---

## Best Practices

### 1. Use Caching
```bash
# Enable caching for 10x speedup on re-scans
glassware-orchestrator --cache-db .glassware-cache.json scan-file packages.txt

# Re-scan uses cache automatically
glassware-orchestrator --cache-db .glassware-cache.json scan-file packages.txt
```

### 2. Tiered Scanning
```
1. Quick scan (Tier 1 only): 0.5s/pkg
2. Full scan (all tiers): 2s/pkg
3. LLM analysis (flagged only): +5s/finding
```

### 3. Batch Processing
```bash
# Scan in batches of 50
split -l 50 all-packages.txt batch-
for batch in batch-*; do
    glassware-orchestrator scan-file "$batch" &
done
wait
```

### 4. Monitor Long Scans
```bash
# Start scan in background
glassware-orchestrator scan-file packages.txt &

# Check progress
watch -n 5 'glassware-orchestrator scan-list --status running'
```

### 5. Export Results
```bash
# SARIF for GitHub
glassware-orchestrator --format sarif scan-file packages.txt > results.sarif

# JSON for analysis
glassware-orchestrator --format json scan-file packages.txt > results.json

# Analyze with jq
cat results.json | jq '.results[] | select(.is_malicious)'
```

---

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success, no findings at threshold |
| 1 | Findings detected |
| 2 | Error (validation, file not found, etc.) |

---

**End of Reference**
