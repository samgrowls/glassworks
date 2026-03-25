# GlassWorm FP/Success Rate Tuning with pi-autoresearch

**Date:** 2026-03-25  
**Version:** v0.56.0  
**Status:** Ready for autoresearch session

---

## Current Baseline (Post-Whitelist Removal)

**Benchmark:** `./benchmarks/fp-success-benchmark.sh --quick`

| Metric | Value | Target |
|--------|-------|--------|
| **Evidence Detection Rate** | 100% (23/23) | 100% |
| **False Positive Rate** | 10% (1/10) | <5% |
| **Combined Score** | 0.96 | >0.98 |

### Root Cause Analysis: 10% FP Rate

**Culprit:** `moment@2.30.1` flagged as malicious (threat score: 7.00)

**Why:** InvisibleCharacter detector finding zero-width characters (ZWNJ, ZWJ) in locale/i18n data files
- These are **legitimate** Unicode characters used for Persian/Arabic script rendering
- This is exactly the false positive we need to fix through detector tuning

**Fix Strategy:** Tune InvisibleCharacter detector to skip i18n/locale data files

### Evidence Packages (23 total)

**Original tarballs (4):**
- react-native-country-select-0.3.91.tgz ✅
- react-native-international-phone-number-0.11.8.tgz ✅
- iflow-mcp-watercrawl-watercrawl-mcp-1.3.4.tgz ✅
- aifabrix-miso-client-4.7.2.tgz ✅

**Synthetic - blockchain_c2 (4):**
- glassworm-c2-001.tgz through glassworm-c2-004.tgz

**Synthetic - combined (4):**
- glassworm-combo-001.tgz through glassworm-combo-004.tgz

**Synthetic - exfiltration (4):**
- glassworm-exfil-001.tgz through glassworm-exfil-004.tgz

**Synthetic - steganography (4):**
- glassworm-steg-001.tgz through glassworm-steg-004.tgz

**Synthetic - time_delay (3):**
- glassworm-evasion-001.tgz through glassworm-evasion-003.tgz

### Clean Packages (10 in quick mode)

express, lodash, axios, chalk, debug, moment, uuid, async, glob, ws

---

## Starting Autoresearch Session

### Prerequisites

1. **NVIDIA API Key** - Set in environment:
   ```bash
   export NVIDIA_NIM_API_KEY="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"
   ```

2. **pi coding agent** - Installed globally:
   ```bash
   pi --version  # Should show 0.62.0 or later
   ```

3. **Extensions installed:**
   - pi-autoresearch
   - pi-nvidia-nim

### Start Session

```bash
cd /home/shva/samgrowls/glassworks-v0.41
export NVIDIA_NIM_API_KEY="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"

# Start pi with autoresearch skill
/skill:autoresearch-create
```

### Session Parameters

When prompted, provide:

**Goal:**
```
Reduce false positive rate from 10% to 0% while maintaining 100% evidence detection
```

**Command:**
```
./benchmarks/fp-success-benchmark.sh --quick
```

**Metric:**
```
combined_score (maximize)
```

**Files in Scope:**
```
glassware-core/src/*.rs
glassware/src/scanner.rs
glassware/src/config.rs
```

---

## Experimentation Cycles

### Cycle 1: FP Reduction (Current Priority)

**Focus:** Fix the 10% false positive rate

**Hypotheses to test:**
1. Increase malicious_threshold from 7.0 to 8.0
2. Adjust category diversity scoring caps
3. Tune specific detector sensitivity (InvisibleCharacter, GlasswarePattern)
4. Add file type awareness to detectors

**Expected outcome:** FP rate drops to 0%, combined score improves to 1.0

### Cycle 2: Detector Sensitivity Tuning

**Focus:** Fine-tune individual detectors

**Detectors to tune:**
1. **InvisibleCharacter** - Skip i18n data files more aggressively
2. **GlasswarePattern** - Better minified code detection
3. **TimeDelay** - CI bypass + delay combination detection
4. **BlockchainC2** - Known C2 wallets vs generic API calls

### Cycle 3: Scoring System Optimization

**Focus:** Optimize scoring formula

**Experiments:**
1. Adjust category_weight, critical_weight, high_weight
2. Test different malicious_threshold values
3. Add detector-specific weights

---

## Configuration Files

### autoresearch.config.json

```json
{
  "workingDir": "/home/shva/samgrowls/glassworks-v0.41",
  "maxIterations": 30,
  "checks_timeout_seconds": 600,
  "benchmark_script": "benchmarks/fp-success-benchmark.sh --quick",
  "optimization_goal": "maximize",
  "metric": "combined_score"
}
```

### Benchmark Script

Location: `benchmarks/fp-success-benchmark.sh`

**Quick mode (10 clean packages):**
```bash
./benchmarks/fp-success-benchmark.sh --quick
```

**Full mode (20 clean packages):**
```bash
./benchmarks/fp-success-benchmark.sh
```

---

## Monitoring Progress

### Session Files

- `autoresearch.md` - Session document with experiment log
- `autoresearch.sh` - Benchmark script wrapper
- `autoresearch.jsonl` - Experiment results (JSONL format)

### View Results

```bash
# View latest experiment results
tail -20 autoresearch.jsonl | jq .

# View session document
cat autoresearch.md

# Check benchmark results
ls -la benchmarks/results/
```

### Confidence Scoring

pi-autoresearch uses MAD-based confidence:

| Confidence | Interpretation | Action |
|------------|----------------|--------|
| ≥ 2.0× (Green) | Improvement likely real | Keep change |
| 1.0-2.0× (Yellow) | Marginal improvement | Re-run to confirm |
| < 1.0× (Red) | Within noise | Discard or re-run |

---

## Expected Timeline

| Phase | Iterations | Duration | Outcome |
|-------|------------|----------|---------|
| Baseline | 1 | 5 min | Current performance |
| FP Reduction | 5-10 | 30-60 min | 0% FP rate |
| Detector Tuning | 10-15 | 60-90 min | Optimized detectors |
| Scoring Optimization | 5-10 | 30-60 min | Tuned scoring |

**Total:** ~2-3 hours for complete optimization cycle

---

## Troubleshooting

### Benchmark fails

```bash
# Check binary exists
ls -la target/release/glassware

# Rebuild if needed
cargo build --release -p glassware

# Test benchmark manually
./benchmarks/fp-success-benchmark.sh --quick
```

### LLM API errors

```bash
# Verify NVIDIA API key
echo $NVIDIA_NIM_API_KEY

# Test API connectivity
curl -H "Authorization: Bearer $NVIDIA_NIM_API_KEY" https://api.nvcf.nvidia.com/v2/nvcf/functions
```

### Extension not loading

```bash
# Check extensions
pi list

# Reload if needed
pi config
```

---

## Success Criteria

### Phase 1 Complete (FP Elimination)
- [x] Whitelist removed from codebase
- [x] Benchmark script created
- [x] Baseline established (100% detection, 10% FP)
- [ ] FP rate reduced to 0%
- [ ] Combined score ≥ 0.98

### Phase 2 Complete (Detector Tuning)
- [ ] All detectors tuned for context-awareness
- [ ] Evidence detection maintained at 100%
- [ ] Clean package false positives eliminated

### Phase 3 Complete (Scoring Optimization)
- [ ] Scoring system configurable via TOML
- [ ] Detector weights tunable without recompilation
- [ ] Documentation updated

---

## Next Steps After Autoresearch

1. **Validate on full Wave 10** - Run full campaign with 1000+ packages
2. **Manual review** - Inspect borderline cases
3. **Expand evidence library** - Add more real-world malicious packages
4. **Document findings** - Update HANDOFF documentation

---

**Author:** Qwen-Coder  
**Last Updated:** 2026-03-25 14:47 UTC
