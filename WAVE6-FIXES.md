# Wave 6 Fixes Summary

**Date:** March 22, 2026
**Status:** ✅ Complete - Ready for Wave 6 scanning

---

## Executive Summary

Fixed critical issues identified in the code audit that were blocking Wave 6 scanning. The Rust orchestrator now has:
- ✅ **LLM analysis integrated** into the scan flow (automatic with `--llm` flag)
- ✅ **Parallel scanning** with configurable concurrency
- ✅ **Package deduplication** to avoid redundant scans
- ✅ **Fixed hardcoded paths** in Python harness scripts

---

## Two-Tier LLM Architecture

The GlassWorm scanner uses a **two-tier LLM strategy** for cost-effective analysis:

```
┌─────────────────────────────────────────────────────────────┐
│  Tier 1: Cerebras (Fast Triage)                             │
│  - During scan with --llm flag                              │
│  - Speed: ~2-5 seconds per package                          │
│  - Purpose: Quick classification of all packages            │
│  - Config: GLASSWARE_LLM_BASE_URL, GLASSWARE_LLM_API_KEY    │
│  - Model: llama-3.3-70b or qwen-3-235b-a22b-instruct-2507   │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  Tier 2: NVIDIA (Deep Analysis)                             │
│  - After scan with --deep-llm flag                          │
│  - Speed: ~15-30 seconds per package                        │
│  - Purpose: Detailed analysis of suspicious packages        │
│  - Config: NVIDIA_API_KEY                                   │
│  - Models: Qwen 3.5 (397B) → Kimi K2.5 → GLM-5 → Llama 3.3  │
└─────────────────────────────────────────────────────────────┘
```

### Usage

```bash
# Tier 1: Fast triage on all packages
./target/release/glassware-orchestrator --llm scan-file packages.txt

# Tier 2: Deep analysis on flagged packages
./target/release/glassware-orchestrator --deep-llm scan-file flagged.txt

# Both tiers in sequence (recommended for Wave 6)
./target/release/glassware-orchestrator --llm scan-file packages.txt -o results.json
cat results.json | jq -r '.[] | select(.is_malicious or .threat_score >= 5.0) | .package_name' > flagged.txt
./target/release/glassware-orchestrator --deep-llm scan-file flagged.txt
```

### Why Two Tiers?

| Tier | Speed | Cost | Models | Use Case |
|------|-------|------|--------|----------|
| **Cerebras** | Fast (~2-5s) | Low | llama-3.3-70b | Scan 1000s of packages quickly |
| **NVIDIA** | Slower (~15-30s) | Higher | Qwen 397B, Kimi K2.5, GLM-5 | Deep dive on 10s of flagged packages |

### Model Fallback Chain (NVIDIA)

The NVIDIA deep analysis tries models in order until one succeeds:

1. **qwen/qwen3.5-397b-a17b** (397B parameters) - Strongest model
2. **moonshotai/kimi-k2.5** - Kimi K2.5
3. **z-ai/glm5** - GLM-5
4. **meta/llama-3.3-70b-instruct** - Fallback (70B)

This ensures analysis completes even if some models are unavailable.

---

## Changes Made

### 1. LLM Analysis Integration (Critical)

**Files Modified:**
- `glassware-orchestrator/src/scanner.rs` - Added `LlmPackageVerdict` struct and `llm_verdict` field
- `glassware-orchestrator/src/orchestrator.rs` - Added automatic LLM analysis after scanning
- `glassware-orchestrator/src/llm.rs` - Added `config()` getter method

**What Changed:**
- LLM analysis now runs automatically when `--llm` flag is used
- Verdicts are stored in `PackageScanResult.llm_verdict`
- Results are cached along with scan results
- Fallback chain works (NVIDIA models: qwen3.5-397b → kimi-k2.5 → glm5 → llama-3.3-70b)

**Usage:**
```bash
# Scan with LLM analysis
./target/release/glassware-orchestrator --llm scan-npm express lodash axios

# Results include LLM verdict
cat results.json | jq '.[] | {package, llm_verdict}'
```

---

### 2. Parallel Scanning (High Priority)

**Files Modified:**
- `glassware-orchestrator/src/orchestrator.rs` - Rewrote `scan_npm_packages()` to use `futures::stream`

**What Changed:**
- Changed from sequential to parallel scanning
- Uses `futures::stream::buffered()` for controlled concurrency
- Respects `max_concurrent` config setting (default: 10)

**Before (sequential):**
```rust
for package in packages {
    match self.scan_npm_package(package).await {
        Ok(result) => results.push(result),
        Err(e) => results.push(Err(e)),
    }
}
```

**After (parallel):**
```rust
let results = stream::iter(unique_packages)
    .map(|package| async move {
        self.scan_npm_package(&package).await
    })
    .buffered(concurrency)
    .collect::<Vec<_>>()
    .await;
```

---

### 3. Package Deduplication (High Priority)

**Files Modified:**
- `glassware-orchestrator/src/orchestrator.rs` - Added HashSet-based deduplication

**What Changed:**
- Duplicate packages in input list are automatically removed
- Logs deduplication statistics

**Example:**
```
info: Deduplicated 100 packages to 95 (removed 5 duplicates)
```

---

### 4. Fixed Hardcoded Paths (Critical)

**Files Modified:**
- `harness/optimized_scanner.py`
- `harness/batch_llm_analyzer.py`

**What Changed:**
- Changed from hardcoded `/home/property.sightlines/...` to environment variables with sensible defaults

**Before:**
```python
GLASSWARE = "/home/property.sightlines/samgrowls/glassworks/harness/glassware-scanner"
```

**After:**
```python
GLASSWARE = os.environ.get(
    "GLASSWARE_BINARY",
    str(Path(__file__).parent.parent / "target" / "release" / "glassware-orchestrator")
)
```

**Environment Variables:**
- `GLASSWARE_BINARY` - Path to Rust orchestrator binary
- `LLM_ANALYZER_SCRIPT` - Path to Python LLM analyzer script
- `GLASSWARE_EVIDENCE_DIR` - Evidence output directory

---

## Configuration

### LLM Configuration (~/.env)

```bash
# Tier 1: Cerebras (Fast Triage) - Used during scan with --llm
export GLASSWARE_LLM_BASE_URL="https://api.cerebras.ai/v1"
export GLASSWARE_LLM_API_KEY="csk-..."
export GLASSWARE_LLM_MODEL="llama-3.3-70b"  # or qwen-3-235b-a22b-instruct-2507

# Tier 2: NVIDIA (Deep Analysis) - Used for batch_llm_analyzer.py
export NVIDIA_API_KEY="nvapi-..."
```

### TOML Configuration (~/.config/glassware/config.toml)

```toml
[scoring]
malicious_threshold = 7.0
category_weight = 2.0
critical_weight = 3.0
high_weight = 1.5

[performance]
concurrency = 10  # Parallel scan concurrency
npm_rate_limit = 10.0

[llm]
# Provider for --llm flag (Tier 1)
provider = "cerebras"  # or "nvidia" for deep analysis
```

### Your Current Configuration

Based on `~/.env`:
- **Tier 1 (Cerebras)**: ✅ Configured with `qwen-3-235b-a22b-instruct-2507`
- **Tier 2 (NVIDIA)**: ✅ API key available for deep analysis

---

## Testing

### Build Verification

```bash
cd /home/property.sightlines/samgrowls/glassworks
cargo build -p glassware-orchestrator --release
# ✅ Build succeeded
```

### Binary Verification

```bash
./target/release/glassware-orchestrator --help
# ✅ Shows help with all commands
```

### Quick Scan Test

```bash
# Scan a single package
./target/release/glassware-orchestrator scan-npm express

# Scan with LLM
./target/release/glassware-orchestrator --llm scan-npm express lodash

# Scan from file
echo -e "express\nlodash\naxios" > packages.txt
./target/release/glassware-orchestrator scan-file packages.txt

# With JSON output
./target/release/glassware-orchestrator --format json scan-npm express > results.json
```

---

## Wave 6 Workflow

### Recommended Workflow (Two-Tier LLM)

1. **Prepare package list:**
   ```bash
   # Use Python harness for diverse sampling
   cd harness
   python3 diverse_sampling.py --samples-per-keyword 20 -o wave6-packages.txt
   ```

2. **Step 1 - Scan with Cerebras triage:**
   ```bash
   cd /home/property.sightlines/samgrowls/glassworks
   ./target/release/glassware-orchestrator \
     --llm \
     --format json \
     --output wave6-results.json \
     scan-file ../harness/wave6-packages.txt
   ```
   
   This runs fast LLM triage (~2-5s per package) on all packages.

3. **Step 2 - Identify suspicious packages:**
   ```bash
   # Find malicious or high-threat packages
   cat wave6-results.json | jq '[.[] | select(.is_malicious or .threat_score >= 5.0)] | length'
   
   # Extract package names for deep analysis
   cat wave6-results.json | jq -r '.[] | select(.is_malicious or .threat_score >= 5.0) | "\(.package_name)@\(.version)"' > flagged.txt
   ```

4. **Step 3 - Deep NVIDIA analysis (optional, on flagged only):**
   ```bash
   cd harness
   python3 batch_llm_analyzer.py flagged.txt --output wave6-deep-results.json
   ```
   
   This runs deep analysis (~15-30s per package) only on suspicious packages.

5. **Review results:**
   ```bash
   # View Cerebras triage verdicts
   cat wave6-results.json | jq '.[] | select(.llm_verdict != null) | {package, is_malicious, threat_score, llm_verdict}'
   
   # View NVIDIA deep analysis
   cat wave6-deep-results.json | jq '.[] | {package, llm_classification, llm_confidence_tier}'
   ```

---

## Known Limitations

1. **LLM rate limiting:** NVIDIA API has rate limits. For large batches (>100 packages), consider using Cerebras for triage first.

2. **Cache growth:** SQLite cache can grow large. Run periodic cleanup:
   ```bash
   ./target/release/glassware-orchestrator cache-cleanup
   ```

3. **Progress tracking:** The parallel scanner doesn't update progress tracker in real-time. This is a known limitation.

---

## Next Steps

1. **Run Wave 6 scan** with target of 500 packages
2. **Monitor LLM usage** and adjust rate limits if needed
3. **Review flagged packages** manually
4. **Consider evidence integrity** enhancements for future waves

---

## Files Changed

| File | Changes |
|------|---------|
| `glassware-orchestrator/src/scanner.rs` | Added `LlmPackageVerdict`, `llm_verdict` field |
| `glassware-orchestrator/src/orchestrator.rs` | LLM integration, parallel scanning, deduplication |
| `glassware-orchestrator/src/llm.rs` | Added `config()` getter |
| `harness/optimized_scanner.py` | Fixed hardcoded paths |
| `harness/batch_llm_analyzer.py` | Fixed hardcoded paths |

---

**Status:** ✅ Ready for Wave 6 scanning
