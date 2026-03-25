# Two-Threshold LLM Configuration

**Date:** March 25, 2026  
**Version:** v0.40.2-two-thresholds  
**Status:** ✅ **PROPERLY CONFIGURED**

---

## The Two-Threshold System

### Tier 1 (Cerebras) - Fast Triage During Scan

**Purpose:** Quick false positive identification during the scan

**Configuration:**
```toml
[settings.llm]
tier1_enabled = true
tier1_provider = "cerebras"
tier1_threshold = 6.0  # Skip Tier 1 for score < 6.0
```

**Behavior:**
- Runs **during** package scan (in wave.rs::scan_package)
- Only analyzes packages with `threat_score >= tier1_threshold`
- Default threshold: **6.0**
- Speed: ~2s/pkg
- Rate Limit: 30 RPM / 60K TPM

**When It Runs:**
```
Package Downloaded → Scan (calculate score) → 
  IF score >= 6.0 → Tier 1 LLM → 
    IF confidence >= 0.75 → trust LLM
    IF confidence <= 0.25 → assume FP
```

---

### Tier 2 (NVIDIA) - Deep Analysis on Flagged Packages

**Purpose:** Detailed analysis for packages already flagged as malicious

**Configuration:**
```toml
[settings.llm]
tier2_enabled = true
tier2_threshold = 7.0  # Tier 2 for score >= 7.0
tier2_models = [
    "qwen/qwen3.5-397b-a17b",  # 397B - strongest
    "moonshotai/kimi-k2.5",
    "z-ai/glm5",
    "meta/llama-3.3-70b-instruct"  # fallback
]
```

**Behavior:**
- Runs **after** scan completes (orchestrator-level)
- Only analyzes packages with `threat_score >= tier2_threshold`
- Default threshold: **7.0**
- Speed: ~15s/pkg
- Rate Limit: Higher (not a bottleneck)

**When It Runs:**
```
Scan Complete → Filter score >= 7.0 → 
  Tier 2 LLM (with model fallback) → 
    Deep analysis with explanation
```

---

## Configuration Examples

### Example 1: Fast Scanning (Phase B - 500 packages)

```toml
[settings.llm]
tier1_enabled = true
tier1_threshold = 6.0  # Skip Tier 1 for low-score
tier2_enabled = true
tier2_threshold = 7.0  # Deep analysis for high-score only
```

**Expected:**
- ~80% reduction in Tier 1 API calls
- Scan time: ~1.5-2 hours for 500 packages
- FP rate: ≤5%

---

### Example 2: Maximum Speed (Phase C - 5000 packages)

```toml
[settings.llm]
tier1_enabled = false  # Skip Tier 1 entirely
tier2_enabled = true
tier2_threshold = 8.0  # Only deep-analyze very high scores
```

**Expected:**
- Minimal LLM API calls
- Scan time: ~8-10 hours for 5000 packages
- Only most suspicious packages get LLM analysis

---

### Example 3: Maximum Detection (Evidence Validation)

```toml
[settings.llm]
tier1_enabled = true
tier1_threshold = 0.0  # Run Tier 1 on ALL packages with findings
tier2_enabled = true
tier2_threshold = 5.0  # Deep analysis for moderate scores
```

**Expected:**
- Maximum LLM coverage
- Scan time: ~3+ hours for 200 packages
- Best for catching subtle attacks

---

### Example 4: Balanced (Production Default)

```toml
[settings.llm]
tier1_enabled = true
tier1_threshold = 6.0  # Skip Tier 1 for score < 6.0
tier2_enabled = true
tier2_threshold = 7.0  # Deep analysis for score >= 7.0
```

**Expected:**
- Good balance of speed and detection
- Scan time: ~1.5-2 hours for 500 packages
- Recommended for most campaigns

---

## Threshold Tuning Guide

### If FP Rate Too High (>10%)

**Option A: Raise tier1_threshold**
```toml
tier1_threshold = 7.0  # Was 6.0
```
- Fewer packages get Tier 1 LLM analysis
- Reduces LLM false positives
- May miss some subtle attacks

**Option B: Raise tier2_threshold**
```toml
tier2_threshold = 8.0  # Was 7.0
```
- Fewer packages get deep analysis
- Reduces overall false positives
- Faster scan times

---

### If Detection Rate Too Low (<90%)

**Option A: Lower tier1_threshold**
```toml
tier1_threshold = 4.0  # Was 6.0
```
- More packages get Tier 1 LLM analysis
- Catches more subtle attacks
- Slower scan times, more API calls

**Option B: Lower tier2_threshold**
```toml
tier2_threshold = 6.0  # Was 7.0
```
- More packages get deep analysis
- Better detection of sophisticated attacks
- Slower but more thorough

---

## Flow Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         PACKAGE SCAN FLOW                                    │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  1. Download Package                                                        │
│         │                                                                   │
│         ▼                                                                   │
│  2. Scan Package (calculate threat_score)                                   │
│         │                                                                   │
│         ▼                                                                   │
│  3. Check tier1_threshold                                                   │
│         │                                                                   │
│    ┌────┴────┐                                                              │
│    │         │                                                              │
│    ▼         ▼                                                              │
│ score < 6.0  score >= 6.0                                                   │
│    │         │                                                              │
│    │         ▼                                                              │
│    │    Run Tier 1 LLM (Cerebras)                                           │
│    │         │                                                              │
│    │         ▼                                                              │
│    │    LLM Verdict (confidence)                                            │
│    │         │                                                              │
│    │    ┌────┼────┐                                                         │
│    │    │    │    │                                                         │
│    │    ▼    ▼    ▼                                                         │
│    │  <0.25 0.25-0.75 >=0.75                                                │
│    │  (FP)  (uncertain) (TP)                                                │
│    │    │    │    │                                                         │
│    │    │    │    ▼                                                         │
│    │    │    │ Override is_malicious                                        │
│    │    │    │                                                              │
│    │    ▼    ▼                                                              │
│    │  Assume safe  Trust LLM                                                │
│    │                                                                          │
│    ▼ (skip Tier 1)                                                           │
│                                                                             │
│  4. Check if is_malicious (score-based or LLM override)                     │
│         │                                                                   │
│    ┌────┴────┐                                                              │
│    │         │                                                              │
│    ▼         ▼                                                              │
│  false     true                                                             │
│    │         │                                                              │
│    │         ▼                                                              │
│    │    Check tier2_threshold                                               │
│    │         │                                                              │
│    │    ┌────┴────┐                                                         │
│    │    │         │                                                         │
│    │    ▼         ▼                                                         │
│    │  < 7.0    >= 7.0                                                       │
│    │    │         │                                                         │
│    │    │         ▼                                                         │
│    │    │    Run Tier 2 LLM (NVIDIA)                                        │
│    │    │         │                                                         │
│    │    │         ▼                                                         │
│    │    │    Deep Analysis                                                  │
│    │    │         │                                                         │
│    │    │         ▼                                                         │
│    │    │    Detailed Explanation                                           │
│    │    │                                                                   │
│    ▼    ▼                                                                    │
│  Skip   Include in Report                                                    │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Files Modified

1. **glassware/src/campaign/config.rs**
   - Added `tier1_threshold` field to `LlmSettings`
   - Default: 6.0
   - Removed duplicate `default_tier2_threshold` function

2. **glassware/src/campaign/wave.rs**
   - Updated to use `tier1_threshold` instead of `analysis_threshold`
   - Added debug logging for skipped Tier 1 LLM

3. **campaigns/wave6.toml**
   - Added `tier1_threshold = 6.0`
   - Added `tier2_models` list

4. **campaigns/phase-a-controlled/config.toml**
   - Added `tier1_threshold = 6.0`
   - Updated `tier2_threshold = 7.0`

---

## Migration Guide

### From Old `analysis_threshold` to New `tier1_threshold`

**Old Config:**
```toml
[settings.llm]
tier1_enabled = true
analysis_threshold = 6.0  # Old field name
tier2_threshold = 7.0
```

**New Config:**
```toml
[settings.llm]
tier1_enabled = true
tier1_threshold = 6.0  # New field name (same default)
tier2_enabled = true
tier2_threshold = 7.0
```

**Note:** The old `analysis_threshold` field no longer exists. Update your configs.

---

## Recommended Defaults

| Campaign Type | tier1_threshold | tier2_threshold | Expected Scan Time |
|---------------|-----------------|-----------------|-------------------|
| **Phase A (Validation)** | 6.0 | 7.0 | ~1.5-2 hours (200 pkg) |
| **Phase B (Wild Small)** | 6.0 | 7.0 | ~1.5-2 hours (500 pkg) |
| **Phase C (Wild Large)** | disabled | 8.0 | ~8-10 hours (5000 pkg) |
| **Evidence Validation** | 0.0 | 5.0 | ~30 min (23 pkg) |

---

**Report By:** Glassworks Development Agent  
**Date:** March 25, 2026  
**Next Action:** Commit and push, then run Phase A re-run
