# glassware User Guide

**Version:** v0.67.0
**Last Updated:** 2026-03-26

---

## Table of Contents

1. [Getting Started](#getting-started)
2. [Scanning Packages](#scanning-packages)
3. [Running Campaigns](#running-campaigns)
4. [LLM Analysis](#llm-analysis)
5. [Understanding Results](#understanding-results)
6. [Troubleshooting](#troubleshooting)

---

## Getting Started

### Installation

```bash
# Clone repository
git clone https://github.com/samgrowls/glassworks.git
cd glassworks

# Build release binary
cargo build --release

# Verify installation
./target/release/glassware --help
```

### Environment Setup

```bash
# Copy example environment file
cp .env.example .env

# Edit with your API keys (optional)
nano .env
```

**Required Environment Variables (for LLM features):**

```bash
# Tier 1 LLM (Cerebras)
export GLASSWARE_LLM_BASE_URL="https://api.cerebras.ai/v1"
export GLASSWARE_LLM_API_KEY="csk-..."
export GLASSWARE_LLM_MODEL="llama-3.3-70b"

# Tier 2 LLM (NVIDIA)
export NVIDIA_API_KEY="nvapi-..."

# GitHub (for private repos)
export GITHUB_TOKEN="ghp_..."
```

---

## Scanning Packages

### Single Package Scan

```bash
# Scan npm package
./target/release/glassware scan-npm <package>@<version>

# Examples
./target/release/glassware scan-npm express@4.19.2
./target/release/glassware scan-npm lodash@4.17.21

# Scan with LLM analysis
./target/release/glassware scan-npm express@4.19.2 --llm

# Scan with deep LLM analysis (Tier 2)
./target/release/glassware scan-npm express@4.19.2 --deep-llm
```

### Scan Tarball

```bash
# Scan local tarball
./target/release/glassware scan-tarball path/to/package.tgz

# Scan evidence package
./target/release/glassware scan-tarball evidence/iflow-mcp.tgz
```

### Scan Multiple Packages

```bash
# Scan multiple packages
./target/release/glassware scan-npm express@4.19.2 lodash@4.17.21 axios@1.6.7

# Scan from file (one package per line)
./target/release/glassware scan-file packages.txt
```

### Output Formats

```bash
# Pretty print (default)
./target/release/glassware scan-npm express@4.19.2

# JSON format
./target/release/glassware --format json scan-npm express@4.19.2

# JSON Lines (streaming)
./target/release/glassware --format jsonl scan-npm express@4.19.2

# SARIF (for GitHub Advanced Security)
./target/release/glassware --format sarif scan-npm express@4.19.2
```

---

## Running Campaigns

### What Is a Campaign?

A **campaign** is a large-scale scanning operation that processes hundreds or thousands of packages in organized waves.

### Campaign Structure

```toml
[campaign]
name = "My Campaign"
description = "Description"
created_by = "security-team"

[settings]
concurrency = 20
rate_limit_npm = 10.0
cache_enabled = true

[settings.scoring]
malicious_threshold = 7.0

[[waves]]
id = "wave_1"
name = "Evidence Validation"

[[waves.sources]]
type = "packages"
list = ["package@1.0.0"]
```

### Run a Campaign

```bash
# Run campaign
./target/release/glassware campaign run campaigns/wave15-validation.toml

# Run with LLM analysis
./target/release/glassware campaign run campaigns/wave15-validation.toml --llm

# Run with deep LLM analysis
./target/release/glassware campaign run campaigns/wave15-validation.toml --deep-llm
```

### Monitor Campaign Progress

```bash
# List recent campaigns
./target/release/glassware campaign list

# Show campaign status
./target/release/glassware campaign status <case-id>

# Live TUI monitoring
./target/release/glassware campaign monitor <case-id>

# Demo mode (sample data)
./target/release/glassware campaign demo
```

### Campaign Commands

```bash
# Pause campaign
./target/release/glassware campaign command <case-id> pause

# Resume campaign
./target/release/glassware campaign command <case-id> resume

# Cancel campaign
./target/release/glassware campaign command <case-id> cancel

# Skip current wave
./target/release/glassware campaign command <case-id> skip-wave

# Adjust concurrency
./target/release/glassware campaign command <case-id> set-concurrency 30
```

### Generate Reports

```bash
# Generate markdown report
./target/release/glassware campaign report <case-id>

# Generate JSON report
./target/release/glassware campaign report <case-id> --format json

# Save report to file
./target/release/glassware campaign report <case-id> --output report.md
```

### Query Campaign with LLM

```bash
# Ask question about campaign
./target/release/glassware campaign query <case-id> "Which packages were flagged as malicious?"

# Ask about specific package
./target/release/glassware campaign query <case-id> "Why was express flagged?"
```

---

## LLM Analysis

### Tier 1: Fast Triage (Cerebras)

```bash
# Enable Tier 1 LLM
./target/release/glassware scan-npm express@4.19.2 --llm

# Tier 1 analyzes packages scoring >= 6.0
# Provides quick malicious/benign classification
```

### Tier 2: Deep Analysis (NVIDIA)

```bash
# Enable Tier 2 LLM
./target/release/glassware scan-npm express@4.19.2 --deep-llm

# Tier 2 analyzes packages scoring >= 8.0
# Provides detailed analysis with recommendations
```

### LLM Output

```
LLM verdict: malicious=false, confidence=0.15
LLM analysis complete: is_malicious=false, confidence=0.15
LLM low confidence (0.15) - overriding is_malicious to false (likely FP)
```

---

## Understanding Results

### Scan Output

```
============================================================
SCAN SUMMARY
============================================================
Total packages scanned: 1
Malicious packages: 1
Total findings: 24
Average threat score: 10.00

Findings by severity:
  Critical: 10
  High: 3
  Medium: 7
  Info: 4

Findings by category:
  "HeaderC2": 8
  "TimeDelaySandboxEvasion": 5
  "EncryptedPayload": 3
  "Rc4Pattern": 2
============================================================
```

### Threat Score Interpretation

| Score Range | Interpretation |
|-------------|----------------|
| 0.0 - 3.0 | Clean / Suspicious |
| 3.0 - 5.0 | Suspicious |
| 5.0 - 7.0 | Borderline malicious |
| 7.0 - 10.0 | Likely malicious |
| 10.0+ | Very likely malicious |
| 25.0+ | GlassWorm signature confirmed |

### Finding Categories

| Category | Description |
|----------|-------------|
| **InvisibleCharacter** | Invisible Unicode characters detected |
| **GlasswarePattern** | Steganography + decoder combination |
| **EncryptedPayload** | High-entropy blob + dynamic execution |
| **HeaderC2** | HTTP header C2 pattern |
| **BlockchainC2** | Blockchain-based C2 communication |
| **TimeDelaySandboxEvasion** | CI bypass + time delays |
| **LocaleGeofencing** | Geographic targeting |

---

## Troubleshooting

### Cache Issues

**Problem:** Old findings persist after code changes

**Solution:**
```bash
# Clear all cache
./target/release/glassware cache-clear

# Or manually
rm -rf .glassware-orchestrator-cache.db
rm -rf .glassware-checkpoints.db
```

### Rate Limiting

**Problem:** LLM rate limit exceeded

**Solution:**
```bash
# Disable LLM for initial scan
./target/release/glassware scan-npm express@4.19.2

# Run LLM analysis separately on flagged packages
./target/release/glassware scan-npm express@4.19.2 --llm
```

### Package Not Found

**Problem:** Package not found on npm

**Solution:**
```bash
# Verify package name and version
npm view <package>@<version>

# Use tarball scan for local packages
./target/release/glassware scan-tarball package.tgz
```

### Campaign Stuck

**Problem:** Campaign not progressing

**Solution:**
```bash
# Check campaign status
./target/release/glassware campaign status <case-id>

# Resume if paused
./target/release/glassware campaign command <case-id> resume

# Cancel and restart if needed
./target/release/glassware campaign command <case-id> cancel
```

---

## Best Practices

### 1. Start Small

```bash
# Test on single package first
./target/release/glassware scan-npm express@4.19.2

# Then run small campaign
./target/release/glassware campaign run campaigns/wave15-validation.toml
```

### 2. Use Evidence Validation

```bash
# Always validate evidence detection first
./target/release/glassware scan-tarball evidence/iflow-mcp.tgz

# Should detect at 8.50+ score
```

### 3. Clear Cache After Changes

```bash
# After detector changes
./target/release/glassware cache-clear

# Then re-scan
./target/release/glassware scan-npm express@4.19.2
```

### 4. Use LLM for Borderline Cases

```bash
# Packages scoring 5.0-8.0 benefit from LLM analysis
./target/release/glassware scan-npm <package> --llm
```

### 5. Review Flagged Packages Manually

```bash
# Generate detailed report
./target/release/glassware campaign report <case-id>

# Review each flagged package
# Check for:
# - Invisible characters (GlassWorm signature)
# - Known C2 wallets/IPs
# - Build tool context (may be FP)
```

---

## Next Steps

- [Developer Guide](DEVELOPER-GUIDE.md) - For extending glassware
- [Architecture](ARCHITECTURE.md) - System design
- [Detectors](DETECTORS.md) - Detector reference
- [Campaign Operator Guide](CAMPAIGN-OPERATOR-GUIDE.md) - Campaign operations

---

**Support:** Open an issue on [GitHub](https://github.com/samgrowls/glassworks/issues)
