# GlassWorm FP/Success Rate Tuning Strategy with pi-autoresearch

**Date:** 2026-03-25  
**Version:** v0.56.0  
**Status:** Ready for execution

---

## Executive Summary

This document outlines the strategy for using [pi-autoresearch](https://github.com/davebcn87/pi-autoresearch) to systematically tune GlassWorm's false positive (FP) rate and detection success rate through automated experimentation.

---

## Current State Assessment

### Detection Performance (v0.41.0-handoff-documentation)

| Metric | Value | Assessment |
|--------|-------|------------|
| **False Positive Rate** | ~0% | ✅ Achieved via whitelisting |
| **Evidence Detection** | 50% (1/2) | ❌ Unacceptable for security tool |
| **Malicious Detection Rate** | ~0% | ⚠️ Suspiciously low |
| **Category Diversity** | Working | ✅ Proper tuning approach |

### Key Problems Identified

1. **Dangerous Whitelist Entries** - High-value targets (UI frameworks, build tools) are whitelisted
2. **Insufficient Evidence Library** - Only 2 known malicious packages
3. **Detector Over-Correction** - Detectors skip entire categories (build tools, SDKs)
4. **Scoring System Limitations** - Category diversity caps may miss single-vector attacks

---

## pi-autoresearch Integration Plan

### What pi-autoresearch Does

pi-autoresearch is an **automated experimentation framework** for pi coding agent that:

1. **Generates hypotheses** about code changes that could improve metrics
2. **Runs experiments** by applying changes and measuring outcomes
3. **Tracks results** with confidence scoring (MAD-based noise detection)
4. **Iterates** on successful changes

### Key Concepts

| Concept | Description |
|---------|-------------|
| **Experiment** | A code change + benchmark run |
| **Metric** | Quantitative measure (FP rate, detection rate, etc.) |
| **Confidence Score** | `|improvement| / MAD` - distinguishes signal from noise |
| **Session** | Collection of experiments toward a goal |

---

## Tuning Strategy

### Phase 1: Infrastructure Setup (COMPLETED)

- [x] Install pi coding agent (`@mariozechner/pi-coding-agent`)
- [x] Install pi-autoresearch extension
- [x] Install pi-nvidia-nim extension for NVIDIA LLM access
- [x] Configure NVIDIA API key from `~/.env`
- [x] Verify extensions load without conflicts

### Phase 2: Benchmark Definition (TODO)

**Goal:** Create reliable, fast benchmarks for measuring FP/success rate

#### Benchmark Types

1. **Evidence Detection Benchmark** (5-10 min)
   - Scan all evidence packages
   - Measure: % detected as malicious
   - Target: 100%

2. **False Positive Benchmark** (15-30 min)
   - Scan known-clean packages (top 100 npm)
   - Measure: % incorrectly flagged
   - Target: 0%

3. **Wave 10 Mini Benchmark** (30-60 min)
   - Run subset of wave10 (100 packages)
   - Measure: FP rate + detection rate
   - Target: 0% FP, >80% detection

#### Benchmark Script Structure

```bash
#!/bin/bash
# benchmarks/fp-success-benchmark.sh

# 1. Build release
cargo build --release -p glassware-orchestrator

# 2. Run evidence detection test
./target/release/glassware-orchestrator scan-tarball evidence/*.tgz --json > evidence-results.json

# 3. Run clean package test
./target/release/glassware-orchestrator scan-npm express lodash axios ... --json > clean-results.json

# 4. Calculate metrics
# - Evidence detection rate
# - False positive rate
# - Combined score

# 5. Output JSON for autoresearch
echo '{"evidence_detection_rate": 0.XX, "fp_rate": 0.XX, "combined_score": X.X}'
```

### Phase 3: Autoresearch Configuration

Create `autoresearch.config.json` in session directory:

```json
{
  "workingDir": "/home/shva/samgrowls/glassworks-v0.41",
  "maxIterations": 30,
  "checks_timeout_seconds": 600,
  "benchmark_script": "benchmarks/fp-success-benchmark.sh",
  "optimization_goal": "maximize",
  "metric": "combined_score"
}
```

### Phase 4: Experimentation Cycles

#### Cycle 1: Whitelist Removal (Priority: HIGH)

**Hypothesis:** Removing dangerous whitelist entries will improve detection without significantly increasing FPs

**Changes to test:**
1. Remove UI frameworks from whitelist
2. Remove build tools from whitelist  
3. Remove SDK packages from whitelist

**Expected outcome:** Detection rate improves, FP rate may increase slightly

#### Cycle 2: Detector Tuning (Priority: HIGH)

**Hypothesis:** Context-aware detector fixes will maintain 0% FP while improving detection

**Changes to test:**
1. InvisibleCharacter: Add file type awareness (skip .json, .d.ts)
2. TimeDelay: Detect CI bypass + delay combination (not just delay)
3. BlockchainC2: Skip generic API calls, flag known C2 indicators

**Expected outcome:** Detection improves, FP stays low

#### Cycle 3: Scoring System (Priority: MEDIUM)

**Hypothesis:** Configurable scoring weights will allow fine-tuning without code changes

**Changes to test:**
1. Move category diversity caps to config
2. Add per-detector weight configuration
3. Test different scoring formulas (linear, quadratic)

**Expected outcome:** Easier tuning, better optimization

#### Cycle 4: Evidence Expansion (Priority: MEDIUM)

**Hypothesis:** More evidence packages will improve tuning accuracy

**Actions:**
1. Search npm for packages similar to confirmed malicious
2. Create synthetic test cases for each attack type
3. Reach out to security firms for samples

---

## Running Autoresearch Sessions

### Start Session

```bash
cd /home/shva/samgrowls/glassworks-v0.41
export NVIDIA_NIM_API_KEY="nvapi-..."
/skill:autoresearch-create
```

### Command Format

```
/autoresearch <goal>, <command>, <metric>
```

### Example Sessions

#### Session 1: Evidence Detection

```
/autoresearch improve evidence detection rate, run benchmarks/fp-success-benchmark.sh and measure evidence_detection_rate, maximize evidence_detection_rate
```

#### Session 2: FP Reduction

```
/autoresearch reduce false positive rate, run benchmarks/fp-success-benchmark.sh and measure fp_rate, minimize fp_rate
```

#### Session 3: Combined Optimization

```
/autoresearch optimize combined FP/success score, run benchmarks/fp-success-benchmark.sh and measure combined_score, maximize combined_score
```

---

## Metrics Definition

### Evidence Detection Rate

```
evidence_detection_rate = detected_malicious / total_evidence
```

**Target:** 1.0 (100%)

### False Positive Rate

```
fp_rate = flagged_clean / total_clean
```

**Target:** 0.0 (0%)

### Combined Score

```
combined_score = (evidence_detection_rate * 0.6) + ((1 - fp_rate) * 0.4)
```

**Target:** 1.0 (perfect)

**Weights:** Adjust based on priorities (security vs. usability)

---

## Confidence Scoring

pi-autoresearch uses **Median Absolute Deviation (MAD)** to distinguish real improvements from noise:

```
Confidence = |best_improvement| / MAD
```

| Confidence | Interpretation | Action |
|------------|----------------|--------|
| ≥ 2.0× (Green) | Improvement likely real | Keep change |
| 1.0-2.0× (Yellow) | Above noise but marginal | Re-run to confirm |
| < 1.0× (Red) | Within noise | Discard or re-run |

---

## Expected Timeline

| Phase | Duration | Outcome |
|-------|----------|---------|
| Phase 2: Benchmark | 2-4 hours | Reliable benchmark script |
| Phase 4 Cycle 1 | 4-8 hours | Whitelist removal results |
| Phase 4 Cycle 2 | 8-16 hours | Detector tuning results |
| Phase 4 Cycle 3 | 4-8 hours | Scoring system improvements |
| Phase 4 Cycle 4 | Ongoing | Evidence library expansion |

**Total:** 1-2 days for initial results

---

## Risk Mitigation

### Risk 1: Benchmark Too Slow

**Mitigation:** Use mini-benchmarks (100 packages) for iteration, full benchmarks for validation

### Risk 2: No Clear Improvement

**Mitigation:** Expand evidence library, adjust metric weights, try different optimization goals

### Risk 3: Overfitting to Benchmarks

**Mitigation:** Validate on full Wave 10/12 campaigns, manual review of borderline cases

### Risk 4: LLM Costs

**Mitigation:** Use tier1_threshold config, limit maxIterations, monitor API usage

---

## Next Steps

1. **Create benchmark script** - `benchmarks/fp-success-benchmark.sh`
2. **Define metric calculation** - JSON output format
3. **Start first autoresearch session** - Whitelist removal cycle
4. **Document results** - Track all experiments in `autoresearch.jsonl`
5. **Iterate** - Continue until targets met

---

## Appendix: File Locations

| File | Purpose |
|------|---------|
| `~/.pi/agent/settings.json` | pi extension configuration |
| `~/.env` | NVIDIA API key |
| `glassworks-v0.41/benchmarks/` | Benchmark scripts |
| `glassworks-v0.41/evidence/` | Known malicious packages |
| `glassworks-v0.41/campaigns/wave10-1000plus.toml` | Test campaign |
| `~/.pi/agent/session/` | Autoresearch session state |
| `autoresearch.config.json` | Autoresearch configuration |
| `autoresearch.jsonl` | Experiment log |

---

## Appendix: Useful Commands

```bash
# Build glassware
cargo build --release -p glassware-orchestrator

# Scan single package
./target/release/glassware-orchestrator scan-npm <package>@<version>

# Scan evidence tarballs
./target/release/glassware-orchestrator scan-tarball evidence/*.tgz

# Run campaign
./target/release/glassware-orchestrator campaign run campaigns/wave10-1000plus.toml

# Check results
grep "flagged as" logs/wave10-*.log

# Clear cache
rm -f .glassware-orchestrator-cache.db

# Start autoresearch
/skill:autoresearch-create

# View experiment log
cat autoresearch.jsonl | jq .
```

---

**Author:** Qwen-Coder  
**Review Date:** After Phase 2 completion
