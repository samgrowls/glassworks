# Rust Orchestrator Two-Tier LLM Guide

**Date:** March 22, 2026
**Status:** ✅ Complete and Tested

---

## Overview

The Rust orchestrator now supports a complete **two-tier LLM strategy** for cost-effective security analysis:

- **Tier 1 (Cerebras)**: Fast triage during scanning
- **Tier 2 (NVIDIA)**: Deep analysis with model fallback

---

## Quick Start

### Tier 1: Fast Triage (All Packages)

```bash
# Scan with Cerebras fast triage
./target/release/glassware-orchestrator --llm scan-file packages.txt

# Output includes LLM verdicts
cat results.json | jq '.[] | {package, is_malicious, llm_verdict}'
```

### Tier 2: Deep Analysis (Flagged Packages Only)

```bash
# Extract flagged packages
cat results.json | jq -r '.[] | select(.is_malicious or .threat_score >= 5.0) | "\(.package_name)@\(.version)"' > flagged.txt

# Run deep NVIDIA analysis
./target/release/glassware-orchestrator --deep-llm scan-file flagged.txt
```

---

## Configuration

### Environment Variables (~/.env)

```bash
# Tier 1: Cerebras (Fast Triage)
GLASSWARE_LLM_BASE_URL="https://api.cerebras.ai/v1"
GLASSWARE_LLM_API_KEY="csk-..."
GLASSWARE_LLM_MODEL="qwen-3-235b-a22b-instruct-2507"

# Tier 2: NVIDIA (Deep Analysis)
NVIDIA_API_KEY="nvapi-..."
```

### Your Current Configuration

Based on `~/.env`:
- ✅ **Tier 1 (Cerebras)**: Configured with `qwen-3-235b-a22b-instruct-2507`
- ✅ **Tier 2 (NVIDIA)**: API key available for deep analysis

---

## LLM Model Details

### Tier 1: Cerebras Models

| Model | Parameters | Speed | Use Case |
|-------|------------|-------|----------|
| `llama-3.3-70b` | 70B | ~2s | Fast triage |
| `qwen-3-235b-a22b-instruct-2507` | 235B | ~3-5s | Better accuracy |

### Tier 2: NVIDIA Models (Fallback Chain)

| Order | Model | Parameters | Speed |
|-------|-------|------------|-------|
| 1 | `qwen/qwen3.5-397b-a17b` | 397B | ~15-20s |
| 2 | `moonshotai/kimi-k2.5` | ~100B | ~15s |
| 3 | `z-ai/glm5` | ~100B | ~15s |
| 4 | `meta/llama-3.3-70b-instruct` | 70B | ~10s |

The fallback chain ensures analysis completes even if stronger models are unavailable.

---

## Wave 6 Workflow

### Step 1: Sample Packages

```bash
cd harness
python3 diverse_sampling.py --samples-per-keyword 20 -o wave6-packages.txt
```

### Step 2: Tier 1 Triage Scan

```bash
cd /home/property.sightlines/samgrowls/glassworks
./target/release/glassware-orchestrator \
  --llm \
  --format json \
  --output wave6-tier1-results.json \
  scan-file ../harness/wave6-packages.txt
```

**Expected time:** ~30-45 minutes for 500 packages (parallel scanning enabled)

### Step 3: Identify Suspicious Packages

```bash
# Find malicious or high-threat packages
cat wave6-tier1-results.json | jq '[.[] | select(.is_malicious or .threat_score >= 5.0)]' > suspicious.json

# Count suspicious packages
cat suspicious.json | jq 'length'

# Extract package specs for Tier 2
cat wave6-tier1-results.json | jq -r '.[] | select(.is_malicious or .threat_score >= 5.0) | "\(.package_name)@\(.version)"' > wave6-flagged.txt
```

### Step 4: Tier 2 Deep Analysis

```bash
./target/release/glassware-orchestrator \
  --deep-llm \
  --format json \
  --output wave6-tier2-results.json \
  scan-file wave6-flagged.txt
```

**Expected time:** ~5-10 minutes for 20-30 flagged packages

### Step 5: Review Results

```bash
# View Tier 1 verdicts
cat wave6-tier1-results.json | jq '.[] | select(.llm_verdict != null) | {package, threat_score, verdict: .llm_verdict}'

# View Tier 2 deep analysis
cat wave6-tier2-results.json | jq '.[] | {package, verdict: .llm_verdict}'

# Generate summary report
cat wave6-tier2-results.json | jq -r '
  "=== Wave 6 Summary ===\n",
  "Total packages scanned: \(. | length)",
  "Malicious: ([.[] | select(.is_malicious)] | length)",
  "Suspicious: ([.[] | select(.threat_score >= 5.0 and .is_malicious == false)] | length)",
  "Clean: ([.[] | select(.threat_score < 5.0 and .is_malicious == false)] | length)"
'
```

---

## Command Reference

### Scan Commands

```bash
# Scan npm packages
./target/release/glassware-orchestrator --llm scan-npm express lodash axios

# Scan from file
./target/release/glassware-orchestrator --llm scan-file packages.txt

# Scan GitHub repositories
./target/release/glassware-orchestrator --llm scan-github owner/repo

# Scan tarballs
./target/release/glassware-orchestrator --llm scan-tarball package.tgz
```

### LLM Flags

| Flag | Provider | Use Case |
|------|----------|----------|
| `--llm` | Cerebras (from env) | Fast triage on all packages |
| `--deep-llm` | NVIDIA (from NVIDIA_API_KEY) | Deep analysis on flagged packages |

### Output Formats

```bash
# Pretty print (default)
./target/release/glassware-orchestrator --llm scan-file packages.txt

# JSON
./target/release/glassware-orchestrator --llm --format json scan-file packages.txt

# JSON Lines (streaming)
./target/release/glassware-orchestrator --llm --streaming scan-file packages.txt

# SARIF (GitHub Advanced Security)
./target/release/glassware-orchestrator --llm --format sarif scan-file packages.txt > results.sarif
```

---

## Performance Benchmarks

### Tier 1 (Cerebras)

| Packages | Time | Avg/Package |
|----------|------|-------------|
| 10 | ~30s | 3s |
| 100 | ~5 min | 3s |
| 500 | ~25 min | 3s |
| 1000 | ~50 min | 3s |

*Note: Parallel scanning (10 concurrent) significantly reduces total time*

### Tier 2 (NVIDIA)

| Packages | Time | Avg/Package |
|----------|------|-------------|
| 10 | ~3 min | 18s |
| 50 | ~15 min | 18s |
| 100 | ~30 min | 18s |

---

## Troubleshooting

### "LLM API key is empty"

**Problem:** API key not set in environment.

**Solution:**
```bash
# For Tier 1
export GLASSWARE_LLM_API_KEY="csk-..."

# For Tier 2
export NVIDIA_API_KEY="nvapi-..."
```

### "All LLM models failed"

**Problem:** NVIDIA API rate limit or all models unavailable.

**Solution:**
1. Wait and retry (rate limit resets after 1 minute)
2. Check API key is valid
3. Reduce concurrency: `--concurrency 5`

### Slow scanning

**Problem:** Scanning takes too long.

**Solutions:**
1. Increase concurrency: `--concurrency 20`
2. Disable caching: `--no-cache` (if cache is stale)
3. Use Tier 1 only for initial scan, Tier 2 for flagged only

---

## Comparison: Rust vs Python Orchestrator

| Feature | Rust Orchestrator | Python Orchestrator |
|---------|------------------|---------------------|
| **Tier 1 (Cerebras)** | ✅ Native `--llm` flag | ⚠️ Via Rust CLI |
| **Tier 2 (NVIDIA)** | ✅ Native `--deep-llm` flag | ✅ `batch_llm_analyzer.py` |
| **Parallel scanning** | ✅ Tokio async (10 concurrent) | ✅ ThreadPoolExecutor |
| **GitHub scanning** | ✅ Native support | ❌ Not implemented |
| **SARIF output** | ✅ Native support | ❌ Not implemented |
| **Wave campaigns** | ❌ Not implemented | ✅ `waves.toml` |
| **Markdown reports** | ❌ Not implemented | ✅ Rich reports |
| **Resume support** | ✅ Checkpoint files | ✅ Database checkpoints |
| **Cache management** | ✅ SQLite with TTL | ✅ SQLite dedup |

---

## Files Changed

| File | Changes |
|------|---------|
| `glassware-orchestrator/src/cli.rs` | Added `--deep-llm` flag |
| `glassware-orchestrator/src/llm.rs` | Added `nvidia_deep_analysis()` config |
| `glassware-orchestrator/src/main.rs` | Wire up `--deep-llm` to NVIDIA config |
| `glassware-orchestrator/src/orchestrator.rs` | LLM analysis in scan flow |
| `glassware-orchestrator/src/scanner.rs` | Added `LlmPackageVerdict` struct |

---

## Next Steps

1. **Run Wave 6** with the two-tier workflow
2. **Monitor API usage** for both Cerebras and NVIDIA
3. **Review flagged packages** manually after Tier 2 analysis
4. **Consider wave campaigns** for Rust (future enhancement)

---

**Status:** ✅ Ready for Wave 6 scanning with two-tier LLM support
