# Wave 2 — 50 Package Real-World Hunt

**Date:** 2026-03-22  
**Version:** v0.11.6  
**Status:** Ready to Launch

---

## Objectives

1. **Validate detection pipeline** with known malicious packages
2. **Establish FP baseline** with high-download legitimate packages
3. **Hunt for real threats** in diverse categories
4. **Test LLM integration** (Cerebras for triage, NVIDIA for deep analysis)

---

## Package Selection (50 Total)

### Known Malicious (4 packages) — MUST DETECT

| Package | Source | Expected Score |
|---------|--------|---------------|
| react-native-country-select@0.3.91 | Evidence | 10.00 |
| react-native-international-phone-number@0.11.8 | Evidence | 10.00 |
| aifabrix-miso-client-4.7.2 | Evidence | 8.00+ |
| iflow-mcp-watercrawl-mcp-1.3.4 | Evidence | 8.00+ |

### High-Download Legitimate (20 packages) — FP Baseline

| Package | Weekly Downloads | Expected Score |
|---------|-----------------|----------------|
| express@4.19.2 | 30M+ | < 3.0 |
| lodash@4.17.21 | 25M+ | < 3.0 |
| axios@1.6.7 | 20M+ | < 3.0 |
| chalk@5.3.0 | 15M+ | < 3.0 |
| moment@2.30.1 | 10M+ | < 3.0 |
| typescript@5.4.2 | 15M+ | < 3.0 |
| prettier@3.2.5 | 12M+ | < 3.0 |
| ... (13 more) | ... | < 3.0 |

### Diverse Categories (16 packages) — Risk-Based Sampling

| Category | Packages | Risk Level |
|----------|----------|------------|
| native-build | node-gyp, bindings, prebuild | Medium |
| install-scripts | core-js, esbuild | Medium |
| crypto | ethers, web3, bcrypt, jsonwebtoken | Medium |
| ai-ml | langchain, openai, anthropic | Low |
| devtools | eslint, webpack, vite, babel | Low |
| utils | dayjs, axios, got | Low |

### Recent Publishes (10 packages) — Last 3 Months

| Package | Published | Risk Level |
|---------|-----------|------------|
| next@14.1.0 | Feb 2026 | Low |
| nuxt@3.10.0 | Feb 2026 | Low |
| vite@5.1.0 | Jan 2026 | Low |
| prisma@5.10.0 | Feb 2026 | Low |
| ... (6 more) | ... | Low |

---

## LLM Strategy

### Cerebras (Fast Triage)

**Provider:** Cerebras  
**Model:** llama-3.3-70b  
**Speed:** ~2-5 seconds per analysis  
**Use Case:** Initial triage of all flagged packages

**Configuration:**
```toml
[llm]
provider = "cerebras"

[llm.cerebras]
base_url = "https://api.cerebras.ai/v1"
model = "llama-3.3-70b"
```

### NVIDIA (Deep Analysis)

**Provider:** NVIDIA NIM  
**Models:** Fallback chain (397B → Kimi K2.5 → GLM-5 → 70B)  
**Speed:** ~15-30 seconds per analysis  
**Use Case:** Deep analysis of high-severity detections

**Configuration:**
```toml
[llm.nvidia]
base_url = "https://integrate.api.nvidia.com/v1"
models = [
    "qwen/qwen3.5-397b-a17b",  # Strongest (397B)
    "moonshotai/kimi-k2.5",     # Kimi K2.5
    "z-ai/glm5",                # GLM-5
    "meta/llama-3.3-70b-instruct"  # Fallback
]
```

### LLM Usage Strategy

| Package Category | LLM Provider | Reason |
|-----------------|--------------|--------|
| Known malicious | NVIDIA | Deep analysis, confirm detection |
| High-download clean | None | Speed (FP baseline) |
| Crypto packages | NVIDIA | Verify legitimate crypto usage |
| Native build | Cerebras | Fast triage |
| Recent publishes | Cerebras | Fast triage |

---

## Execution Plan

### Step 1: Source Environment

```bash
source ~/.env
export GLASSWARE_LLM_API_KEY
export NVIDIA_API_KEY
export NVIDIA_MODELS
```

### Step 2: Run Wave 2 Scanner

```bash
cd harness
python3 wave2_scanner.py
```

### Step 3: Review Results

```bash
# View summary
cat data/wave2-results/wave2-results-*.json | jq '.malicious_detected'

# View malicious packages
cat data/wave2-results/wave2-results-*.json | jq '.results[] | select(.is_malicious == true)'

# View LLM verdicts
cat data/wave2-results/wave2-results-*.json | jq '.results[] | select(.llm_verdict != null)'
```

### Step 4: Generate Report

```bash
# Generate markdown report
python3 reporter.py --input data/wave2-results/wave2-results-*.json --output reports/WAVE2-REPORT.md
```

---

## Success Criteria

| Metric | Target | Actual |
|--------|--------|--------|
| Known malicious detected | 4/4 (100%) | TBD |
| False positive rate | < 5% (1/20) | TBD |
| LLM triage working | Cerebras responds | TBD |
| LLM deep analysis working | NVIDIA responds | TBD |
| Scan duration | < 30 minutes | TBD |

---

## Risk Mitigation

### API Rate Limits

| Provider | Limit | Mitigation |
|----------|-------|------------|
| Cerebras | 30 RPM, 60K TPM | Scan in batches, 2-second delay |
| NVIDIA | 60 RPM | Use only for high-severity |

### Scan Failures

- **Timeout:** 120 seconds per package
- **Retry:** 3 attempts with exponential backoff
- **Fallback:** Skip LLM if API unavailable

---

## Expected Outcomes

### Best Case
- All 4 known malicious detected
- 0 false positives on high-download packages
- LLM integration working smoothly
- Scan completes in < 20 minutes

### Worst Case
- Some known malicious missed (config tuning needed)
- 1-2 false positives (whitelist tuning needed)
- LLM API rate limiting (batch scanning needed)
- Scan takes > 45 minutes

### Likely Outcome
- 3-4/4 known malicious detected
- 0-1 false positives
- LLM working with occasional rate limiting
- Scan completes in 25-35 minutes

---

## Next Steps After Wave 2

1. **Analyze results** — Review false positives/negatives
2. **Tune config** — Adjust thresholds/weights if needed
3. **Wave 3 (100 packages)** — Expand diverse categories
4. **Wave 4 (500 packages)** — Broad sweep
5. **Wave 5 (1000 packages)** — Full hunt

---

**Ready to launch Wave 2.**
