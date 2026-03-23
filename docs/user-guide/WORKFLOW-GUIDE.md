# GlassWorm Campaign Workflow Guide

**Version:** v0.11.5  
**Last Updated:** 2026-03-21

---

## Quick Start

### 5-Minute Scan

```bash
# Clone and build
git clone https://github.com/samgrowls/glassworks.git
cd glassworks
cargo build -p glassware-cli --release

# Scan a directory
./target/release/glassware /path/to/project

# Scan npm packages
./target/release/glassware-orchestrator scan-npm express lodash axios
```

---

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Choosing Your Tool](#choosing-your-tool)
3. [Quick Scans](#quick-scans)
4. [Campaign Scans](#campaign-scans)
5. [Wave-Based Scanning](#wave-based-scanning)
6. [LLM Analysis](#llm-analysis)
7. [Output Formats](#output-formats)
8. [Troubleshooting](#troubleshooting)

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                    GlassWorm Detection Engine                    │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────┐  │
│  │  glassware CLI   │  │   Rust           │  │   Python     │  │
│  │  (single file)   │  │   Orchestrator   │  │   Harness    │  │
│  │                  │  │                  │  │              │  │
│  │  - Quick scans   │  │  - Campaign scans│  │  - Wave scans│  │
│  │  - Directories   │  │  - npm/GitHub    │  │  - NVIDIA LLM│  │
│  │  - Fast          │  │  - SARIF output  │  │  - Flexible  │  │
│  └──────────────────┘  └──────────────────┘  └──────────────┘  │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │              glassware-core (Rust detection engine)       │   │
│  │  - 22+ detectors (Unicode, Binary, Behavioral, Host)     │   │
│  │  - L1/L2/L3 detection tiers                              │   │
│  │  - Campaign correlation                                  │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## Choosing Your Tool

| Use Case | Recommended Tool | Why |
|----------|-----------------|-----|
| Scan a project directory | `glassware` CLI | Fast, simple, direct |
| Scan specific npm packages | `glassware-orchestrator` | Version support, caching |
| Scan GitHub repositories | `glassware-orchestrator` | GitHub API integration |
| Wave-based campaigns | Python harness | waves.toml configuration |
| NVIDIA LLM analysis | Python harness | Model fallback, stronger models |
| SARIF output (GitHub Security) | `glassware-orchestrator` | Native SARIF support |
| Maximum performance | `glassware-orchestrator` | 1.5x faster than Python |

### Performance Comparison

| Metric | Rust Orchestrator | Python Harness |
|--------|-------------------|----------------|
| Scan speed (3 packages) | 9.5s | 14.5s |
| Packages/sec | 0.32 | 0.21 |
| Memory usage | ~50MB | ~150MB |
| Startup time | <1s | ~2s |

---

## Quick Scans

### Single File

```bash
# Scan a single file
glassware src/index.js

# With JSON output
glassware --format json src/index.js > results.json

# Only high/critical findings
glassware --severity high src/index.js
```

### Directory Scan

```bash
# Scan entire project
glassware /path/to/project

# With specific extensions
glassware --extensions js,ts,tsx,jsx /path/to/project

# Exclude directories
glassware --exclude node_modules,target,.git /path/to/project

# SARIF output for GitHub
glassware --format sarif /path/to/project > results.sarif
```

### npm Packages

```bash
# Scan specific packages (Rust)
glassware-orchestrator scan-npm express@4.19.2 lodash@4.17.21

# Scan with version policy
glassware-orchestrator scan-npm express --versions last-10

# Scan from file
echo -e "express\nlodash\naxios" > packages.txt
glassware-orchestrator scan-file packages.txt
```

---

## Campaign Scans

### What is a Campaign?

A **campaign** is a coordinated scanning effort targeting specific threat patterns:

- **GlassWorm**: Unicode steganography + behavioral evasion
- **PhantomRaven**: RDD (URL dependencies) + JPD author signature
- **ForceMemo**: Python repository injection

### Running a Campaign

#### Option 1: Rust Orchestrator (Recommended for speed)

```bash
# 1. Create package list
cat > campaign-packages.txt << EOF
react-native-country-select@0.3.91
react-native-international-phone-number@0.11.8
EOF

# 2. Run scan with SARIF output
glassware-orchestrator --format sarif \
  --output campaign-results.sarif \
  scan-file campaign-packages.txt

# 3. Review results
cat campaign-results.sarif | jq '.runs[0].results'
```

#### Option 2: Python Harness (Recommended for LLM analysis)

```bash
# 1. Configure wave in waves.toml (see Wave-Based Scanning)

# 2. Run wave
cd harness
python3 -m core.orchestrator run-wave --wave 0 --llm

# 3. Review report
cat reports/scan-<run_id>.md
```

---

## Wave-Based Scanning

### What is a Wave?

A **wave** is a pre-configured scanning campaign with:
- Target package categories
- Sample sizes
- Detection thresholds

### Configuring Waves

Edit `harness/waves.toml`:

```toml
[wave_0]
name = "Wave 0: Calibration"
description = "Validate pipeline with known malicious + clean packages"
packages_total = 50

[wave_0.known_malicious]
packages = [
    "react-native-country-select@0.3.91",
    "react-native-international-phone-number@0.11.8",
]

[wave_0.clean_baseline]
count = 20
packages = ["express", "lodash", "axios", ...]

[wave_1]
name = "Wave 1: Targeted Hunting"
description = "Hunt in GlassWorm active zones"
packages_total = 100

[wave_1.react_native]
count = 30
keywords = ["react-native-phone", "react-native-country"]

[wave_1.crypto_wallet]
count = 30
keywords = ["solana", "ethereum", "web3", "wallet"]
```

### Running Waves

```bash
cd harness

# Run Wave 0 (calibration)
python3 -m core.orchestrator run-wave --wave 0

# Run with LLM analysis
python3 -m core.orchestrator run-wave --wave 0 --llm

# Check status
python3 -m core.orchestrator status --wave 0

# Generate report
python3 -m core.orchestrator report --wave 0
```

### Wave Results

After running a wave, check:

```bash
# Markdown report
cat harness/reports/scan-<run_id>.md

# JSON data
cat harness/reports/scan-<run_id>.json | jq

# Database queries
sqlite3 harness/data/corpus.db \
  "SELECT name, version, finding_count FROM packages WHERE finding_count > 0;"
```

---

## LLM Analysis

### Configuration

Add to `~/.env`:

```bash
# NVIDIA API (for deep analysis)
export NVIDIA_API_KEY="nvapi-..."
export NVIDIA_BASE_URL="https://integrate.api.nvidia.com/v1"

# Model preference (comma-separated, fallback order)
export NVIDIA_MODELS="qwen/qwen3.5-397b-a17b,moonshotai/kimi-k2.5,z-ai/glm5,meta/llama3-70b-instruct"

# Cerebras API (for Rust orchestrator triage)
export GLASSWARE_LLM_BASE_URL="https://api.cerebras.ai/v1"
export GLASSWARE_LLM_API_KEY="csk-..."
```

### Running LLM Analysis

#### Python Harness (NVIDIA)

```bash
# Scan with LLM on flagged packages
python3 -m core.orchestrator run-wave --wave 0 --llm

# Results include LLM verdict
cat harness/reports/scan-<run_id>.json | jq '.packages[] | {name, llm_analysis}'
```

#### Rust Orchestrator (Cerebras)

```bash
# Scan with LLM triage
glassware-orchestrator --llm scan-npm express lodash
```

### LLM Output Format

```json
{
  "malicious": "no",
  "confidence": "high",
  "recommendation": "clean",
  "concerns": [
    "Socket.IO usage (likely false positive)",
    "Time delay (likely test infrastructure)"
  ],
  "reasoning": "Package matches official source code...",
  "model_used": "qwen/qwen3.5-397b-a17b"
}
```

---

## Output Formats

### Pretty Print (Default)

```bash
glassware project/
```

```
⚠ CRITICAL
  File: src/index.js
  Line: 42
  Type: glassware pattern
  GlassWare attack pattern detected
---

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
2 findings in 15 files (1 critical, 0 high, 1 medium, 0 low)
Scanned 15 files in 0.25s
```

### JSON

```bash
glassware --format json project/ > results.json
```

```json
{
  "findings": [
    {
      "file": "src/index.js",
      "line": 42,
      "severity": "critical",
      "category": "GlasswarePattern",
      "description": "GlassWare attack pattern detected"
    }
  ],
  "summary": {
    "files_scanned": 15,
    "findings_total": 2
  }
}
```

### SARIF (GitHub Security)

```bash
glassware-orchestrator --format sarif \
  --output results.sarif \
  scan-npm express lodash
```

Upload `results.sarif` to GitHub Security tab.

---

## Troubleshooting

### "Package not found"

**Problem:** Rust orchestrator can't find package with version.

**Solution:** Already fixed in v0.11.5+. Update:
```bash
git pull
cargo build -p glassware-orchestrator --release
```

### "NVIDIA_API_KEY not set"

**Problem:** LLM analysis fails.

**Solution:** Add to `~/.env`:
```bash
export NVIDIA_API_KEY="nvapi-..."
```

### Slow scanning

**Problem:** Scanning takes too long.

**Solutions:**
1. Increase concurrency: `--concurrency 20`
2. Use Rust orchestrator (1.5x faster)
3. Enable caching: Remove `--no-cache` flag
4. Exclude large directories: `--exclude node_modules`

### False positives

**Problem:** Legitimate packages flagged.

**Solutions:**
1. Check severity level: `--severity high` (skip INFO/LOW)
2. Review LLM analysis for context
3. Add to exclusion list if consistent FP

### Memory issues

**Problem:** Out of memory during large scans.

**Solutions:**
1. Reduce concurrency: `--concurrency 5`
2. Use Rust orchestrator (lower memory footprint)
3. Scan in batches

---

## Quick Reference

### Common Commands

```bash
# Quick project scan
glassware /path/to/project

# Scan npm packages
glassware-orchestrator scan-npm express lodash axios

# Scan GitHub repo
glassware-orchestrator scan-github owner/repo

# Run wave campaign
cd harness && python3 -m core.orchestrator run-wave --wave 0

# With LLM analysis
glassware-orchestrator --llm scan-npm express
python3 -m core.orchestrator run-wave --wave 0 --llm

# SARIF output
glassware-orchestrator --format sarif scan-npm express > results.sarif

# Resume interrupted scan
glassware-orchestrator resume npm --packages express lodash
```

### Environment Variables

```bash
# NVIDIA LLM
export NVIDIA_API_KEY="nvapi-..."
export NVIDIA_MODELS="qwen/qwen3.5-397b-a17b,..."

# Cerebras LLM (Rust)
export GLASSWARE_LLM_BASE_URL="https://api.cerebras.ai/v1"
export GLASSWARE_LLM_API_KEY="csk-..."

# GitHub API (private repos)
export GITHUB_TOKEN="ghp_..."
```

---

## Getting Help

- **Documentation:** `glassware --help`, `glassware-orchestrator --help`
- **Issues:** https://github.com/samgrowls/glassworks/issues
- **Intel Source:** https://codeberg.org/tip-o-deincognito/glassworm-writeup
