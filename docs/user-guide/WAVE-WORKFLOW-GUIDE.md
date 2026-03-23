# GlassWorm Wave Scanning Workflow

**Version:** v0.11.7  
**Last Updated:** 2026-03-22  
**Status:** Production Ready

---

## Overview

This document describes the complete workflow for running GlassWorm package scanning waves to detect GlassWorm and similar supply chain attacks.

---

## Quick Start

```bash
# 1. Build and install release binaries
cd /path/to/glassworks
cargo build --release -p glassware-cli -p glassware-orchestrator
cp target/release/glassware ~/.local/bin/
cp target/release/glassware-orchestrator ~/.local/bin/

# 2. Configure API keys (for LLM analysis)
echo 'GLASSWARE_LLM_API_KEY="csk-..." >> ~/.env
echo 'NVIDIA_API_KEY="nvapi-..." >> ~/.env
source ~/.env

# 3. Initialize configuration
glassware-orchestrator config init

# 4. Run a wave scan
cd harness
./wave5_scan.sh --llm  # With Cerebras LLM triage
```

---

## Architecture

### Two-Tier LLM Strategy

```
┌─────────────────────────────────────────────────────────────┐
│                    Wave Scan (No LLM)                        │
│  Speed: ~30-60 min for 1000 packages                        │
│  Output: JSON with findings and threat scores               │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│              Review Results (score >= 5.0)                   │
│  Identify suspicious packages for deep analysis             │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│         Deep Analysis (NVIDIA - Optional)                    │
│  Speed: ~15-30 sec per package                              │
│  Output: LLM verdict with reasoning                         │
└─────────────────────────────────────────────────────────────┘
```

### LLM Providers

| Tier | Provider | Speed | Use Case |
|------|----------|-------|----------|
| **Tier 1** | Cerebras | ~2-5s | Fast triage during scan |
| **Tier 2** | NVIDIA | ~15-30s | Deep analysis of suspicious |

---

## Configuration

### File Location

- **User config:** `~/.config/glassware/config.toml`
- **Project config:** `.glassware.toml` (in project root)

### Key Settings

```toml
[scoring]
malicious_threshold = 7.0    # Score >= this is "malicious"
suspicious_threshold = 3.0   # Score >= this is "suspicious"
category_weight = 2.0        # Weight per category
critical_weight = 3.0        # Weight per critical finding
high_weight = 1.5            # Weight per high severity

[whitelist]
# Packages to never flag
packages = ["moment", "prettier", "typescript", "i18n", ...]
crypto_packages = ["ethers", "web3", "viem", ...]
build_tools = ["webpack", "vite", "rollup", ...]
state_management = ["mobx", "redux", "zustand", ...]
web_frameworks = ["fastify", "express", "koa", ...]
```

### Environment Variables

```bash
# Cerebras (Tier 1 - Fast Triage)
export GLASSWARE_LLM_BASE_URL="https://api.cerebras.ai/v1"
export GLASSWARE_LLM_API_KEY="csk-..."
export GLASSWARE_LLM_MODEL="llama-3.3-70b"

# NVIDIA (Tier 2 - Deep Analysis)
export GLASSWARE_LLM_BASE_URL="https://integrate.api.nvidia.com/v1"
export NVIDIA_API_KEY="nvapi-..."
export GLASSWARE_LLM_MODEL="qwen/qwen3.5-397b-a17b"
```

---

## Wave Scanning Scripts

### wave5_scan.sh — 1000 Package Scan

```bash
cd harness

# Basic scan (no LLM, fastest)
./wave5_scan.sh

# With Cerebras LLM triage (recommended)
./wave5_scan.sh --llm

# With deep NVIDIA analysis on specific packages
./wave5_scan.sh --llm --deep-llm suspicious-pkg@1.0.0 another@2.0.0
```

**Categories:**
- Wave 5A: React Native Ecosystem (50 packages)
- Wave 5B: MCP/AI Infrastructure (38 packages)
- Wave 5C: Unicode/Locale Heavy (42 packages)
- Wave 5D: Install Scripts/Native (41 packages)
- Wave 5E: Random Recent/Popular (107 packages)

### analyze_flagged.sh — Post-Scan Deep Analysis

```bash
# Analyze all flagged packages (score >= 5.0)
./analyze_flagged.sh data/wave5-results/wave5-npm-*.json

# Analyze ALL packages (not just flagged)
./analyze_flagged.sh data/wave5-results/wave5-npm-*.json --all
```

---

## Recommended Workflow

### Step 1: Pre-Scan Preparation

```bash
# 1. Verify binaries are installed
glassware-orchestrator --version

# 2. Verify configuration
glassware-orchestrator config show

# 3. Validate config
glassware-orchestrator config validate
```

### Step 2: Run Wave Scan

```bash
# WITHOUT LLM for speed (recommended for first pass)
cd harness
./wave5_scan.sh

# Expected duration: 30-60 minutes for 1000 packages
```

### Step 3: Review Results

```bash
# View summary
cat data/wave5-results/wave5-npm-*.json | jq '.summary'

# View malicious packages
cat data/wave5-results/wave5-npm-*.json | jq '.results[] | select(.is_malicious == true)'

# View suspicious packages (score >= 5.0)
cat data/wave5-results/wave5-npm-*.json | jq '.results[] | select(.threat_score >= 5.0)'
```

### Step 4: Deep Analysis (Optional)

```bash
# Run NVIDIA deep analysis on flagged packages
export NVIDIA_API_KEY="nvapi-..."
./analyze_flagged.sh data/wave5-results/wave5-npm-*.json

# Review LLM verdicts
cat data/wave5-results/wave5-deep-llm-*.json | jq '.results[] | {package: .package_name, verdict: .llm_verdict}'
```

### Step 5: Manual Review

For each flagged package:

1. **Check LLM verdict** (if available)
2. **Review findings:**
   ```bash
   cat results.json | jq '.results[] | select(.package_name == "pkg") | .findings[]'
   ```
3. **Download and inspect:**
   ```bash
   npm pack pkg@version
   tar -xzf pkg-version.tgz
   # Review suspicious files manually
   ```
4. **Decide:**
   - **False Positive:** Add to whitelist
   - **Suspicious:** Report to npm Security
   - **Confirmed Malicious:** Immediate report + disclosure

---

## Package Selection Best Practices

### DO: Verify Versions Before Scan

```bash
# Check if version exists
npm view package@version >/dev/null 2>&1
if [ $? -ne 0 ]; then
    # Get latest version
    LATEST=$(npm view package version)
    echo "Using $LATEST instead"
fi
```

### DON'T: Use Placeholder Versions

```bash
# ❌ Bad - will fail
"package@0.0.0"

# ✅ Good - verify first
"package@$(npm view package version)"
```

### High-Risk Categories (Priority Targets)

1. **React Native Ecosystem**
   - Country pickers, phone inputs, OTP inputs
   - Packages with locale/country data
   
2. **MCP/AI Infrastructure**
   - Model Context Protocol servers
   - AI connectors and wrappers
   
3. **Unicode/Locale Heavy**
   - i18n libraries
   - Date/time with locale data
   
4. **Install Scripts**
   - Packages with `install.js`, `preinstall.js`
   - Native module builders

---

## Troubleshooting

### Common Issues

**1. "Version not found" errors**

```bash
# Fix: Get correct version
npm view package versions | tail -5

# Update wave script with correct version
```

**2. "LLM requires GLASSWARE_LLM_BASE_URL"**

```bash
# Fix: Set environment variable
export GLASSWARE_LLM_BASE_URL="https://api.cerebras.ai/v1"
export GLASSWARE_LLM_API_KEY="csk-..."

# Or use --llm flag (auto-sets Cerebras)
./wave5_scan.sh --llm
```

**3. High false positive rate**

```bash
# Fix: Update whitelist in config
glassware-orchestrator config edit

# Add package to whitelist
[whitelist]
packages = ["moment", "prettier", "your-package", ...]
```

**4. Scan taking too long**

```bash
# Fix: Reduce concurrency
# Edit ~/.config/glassware/config.toml
[performance]
concurrency = 5  # Default: 10

# Or skip LLM for initial pass
./wave5_scan.sh  # Without --llm
```

---

## Performance Benchmarks

| Scan Type | Packages | Duration | Notes |
|-----------|----------|----------|-------|
| Wave 5 (no LLM) | 147 | ~15 min | Baseline |
| Wave 5 (Cerebras) | 147 | ~20 min | +5 min for LLM |
| Deep Analysis | 10 | ~5 min | NVIDIA only |
| Full 1000 (est.) | 1000 | 60-90 min | With LLM triage |

---

## Output Files

### Wave Scan Results

```
harness/data/wave5-results/
├── wave5-npm-YYYYMMDD-HHMMSS.json      # Full results
├── wave5-npm-log-YYYYMMDD-HHMMSS.txt   # Scan log
├── wave5-evidence-YYYYMMDD-HHMMSS.json # Evidence scan
└── wave5-deep-llm-YYYYMMDD-HHMMSS.json # Deep analysis (if run)
```

### JSON Structure

```json
{
  "results": [
    {
      "package_name": "pkg",
      "version": "1.0.0",
      "threat_score": 10.00,
      "is_malicious": true,
      "findings": [...],
      "llm_verdict": {
        "malicious": true,
        "reason": "..."
      }
    }
  ],
  "summary": {
    "total_packages": 147,
    "malicious_packages": 10,
    "average_threat_score": 0.95,
    "findings_by_category": {...},
    "findings_by_severity": {...}
  }
}
```

---

## Reporting

### False Positive Report

```bash
# If package is legitimate but flagged:
# 1. Add to whitelist
# 2. Document why it was flagged
# 3. Commit whitelist update

# Example: fastify was flagged for complex patterns
# Reason: Web framework with legitimate eval patterns
# Action: Added to build_tools whitelist
```

### Malicious Package Report

```bash
# If package is confirmed malicious:
# 1. Capture evidence (tarball)
# 2. Document findings
# 3. Report to npm Security: https://www.npmjs.com/support
# 4. Prepare disclosure

# Template:
# - Package name and version
# - Threat score and findings
# - LLM verdict (if available)
# - Manual analysis notes
# - Recommended action
```

---

## References

- [Configuration System Design](../docs/CONFIG-SYSTEM-DESIGN.md)
- [User Guide](../docs/USER-GUIDE.md)
- [Wave 5 Results](../harness/WAVE5-RESULTS-FINAL.md)
- [GlassWorm Writeup](https://codeberg.org/tip-o-deincognito/glassworm-writeup)

---

**Last Wave Run:** Wave 5 (147 packages) — 2026-03-22  
**Next Wave:** Wave 6 (500 packages with version validation)
