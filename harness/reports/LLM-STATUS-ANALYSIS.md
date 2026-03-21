# LLM Integration Status Analysis

**Date:** 2026-03-21  
**Analysis:** Current State of LLM Integration

---

## Current LLM Architecture

### Two Separate LLM Systems

```
┌─────────────────────────────────────────────────────────────────┐
│                    GLASSWARE ECOSYSTEM                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                   │
│  ┌──────────────────┐         ┌──────────────────┐              │
│  │  glassware-core  │         │ glassware-orch   │              │
│  │     (Rust)       │         │     (Rust)       │              │
│  │                  │         │                  │              │
│  │  LlmConfig       │         │ LlmAnalyzerConfig│              │
│  │  from_env()      │         │ from_env()       │              │
│  │                  │         │                  │              │
│  │  GLASSWARE_LLM_* │         │ GLASSWARE_LLM_*  │              │
│  │  - BASE_URL      │         │  - BASE_URL      │              │
│  │  - API_KEY       │         │  - API_KEY       │              │
│  │  - MODEL         │         │  - MODEL         │              │
│  └────────┬─────────┘         └────────┬─────────┘              │
│           │                             │                        │
│           └──────────┬──────────────────┘                        │
│                      │                                           │
│                      ▼                                           │
│           ┌──────────────────────┐                              │
│           │   Cerebras API       │                              │
│           │   (Fast, Cheap)      │                              │
│           │   llama-3.3-70b      │                              │
│           │   RPM: 30, TPM: 60k  │                              │
│           └──────────────────────┘                              │
│                                                                   │
├─────────────────────────────────────────────────────────────────┤
│                    PYTHON HARNESS                                │
│                                                                   │
│  ┌──────────────────┐         ┌──────────────────┐              │
│  │ batch_llm_       │         │ llm_prioritizer  │              │
│  │ analyzer.py      │         │ .py              │              │
│  │                  │         │                  │              │
│  │ NVIDIA_API_KEY   │         │ NVIDIA_API_KEY   │              │
│  │                  │         │                  │              │
│  │ NVIDIA NIM       │         │ NVIDIA NIM       │              │
│  │ meta/llama-3.3-  │         │ meta/llama-3.3-  │              │
│  │ 70b-instruct     │         │ 70b-instruct     │              │
│  │ (Slower, more    │         │ (Slower, more    │              │
│  │  capable)        │         │  capable)        │              │
│  └──────────────────┘         └──────────────────┘              │
│                                                                   │
└─────────────────────────────────────────────────────────────────┘
```

---

## Environment Variables Currently Used

### Rust LLM (glassware-core + orchestrator)

```bash
GLASSWARE_LLM_BASE_URL=https://api.cerebras.ai/v1
GLASSWARE_LLM_API_KEY=csk-...
GLASSWARE_LLM_MODEL=qwen-3-235b-a22b-instruct-2507
GLASSWARE_LLM_RPM=30          # Rate limit: requests/minute
GLASSWARE_LLM_TPM=60000       # Rate limit: tokens/minute
```

### Python Harness LLM

```bash
NVIDIA_API_KEY=nvapi-...
# Uses hardcoded: meta/llama-3.3-70b-instruct
# No rate limiting configured
```

---

## Use Cases

### Cerebras (Fast Model) - Rust

**Purpose:** Quick triage during package scans  
**Model:** `qwen-3-235b-a22b-instruct-2507` (or `llama-3.3-70b`)  
**Speed:** ~2-5 seconds per finding  
**Rate Limits:** 30 RPM, 60,000 TPM (STRICT)  
**Cost:** ~$0.10-0.30/1M tokens  

**When Used:**
- `glassware --llm` CLI scans
- `glassware-orchestrator --llm` package scans
- Real-time analysis during scanning

**Rate Limit Strategy:**
- Token bucket algorithm in `glassware-core/src/llm/rate_limiter.rs`
- Blocks when limit reached (no queuing)
- No interleaving with other providers

---

### NVIDIA NIM (Slow Model) - Python

**Purpose:** Deep analysis of flagged packages  
**Model:** `meta/llama-3.3-70b-instruct` (hardcoded)  
**Speed:** ~10-30 seconds per finding  
**Rate Limits:** Unknown (appears unlimited)  
**Cost:** Unknown (likely higher than Cerebras)  

**When Used:**
- `harness/batch_llm_analyzer.py` - batch analysis
- `harness/llm_prioritizer.py` - detailed triage
- Post-scan deep analysis

**Rate Limit Strategy:**
- None configured
- Sequential processing
- No retry logic

---

## Issues Identified

### 1. ❌ Duplicate Configuration

Two separate env var systems:
- `GLASSWARE_LLM_*` for Rust
- `NVIDIA_API_KEY` for Python

**Impact:** Confusing, requires two API keys, can't switch providers easily

---

### 2. ❌ No Provider Interleaving

Cerebras has strict 30 RPM limit. When scanning many packages:
- Hits rate limit quickly
- Blocks until tokens refill
- No fallback to other providers

**Impact:** Slow scans when rate limited

---

### 3. ❌ Hardcoded Model (Python)

Python harness uses `meta/llama-3.3-70b-instruct` hardcoded.

**Impact:** Can't switch models without code changes

---

### 4. ❌ No Rate Limiting (Python)

Python harness has no rate limiting.

**Impact:** Could hit API limits unexpectedly

---

### 5. ❌ Different Providers for Different Tasks

- Rust → Cerebras (fast)
- Python → NVIDIA (slow)

**Impact:** Can't use fast model for batch analysis or slow model for real-time

---

## Recommendations

### Short-Term (Quick Wins)

1. **Unify Environment Variables**
   ```bash
   # Primary (fast) provider
   LLM_FAST_BASE_URL=https://api.cerebras.ai/v1
   LLM_FAST_API_KEY=csk-...
   LLM_FAST_MODEL=llama-3.3-70b
   LLM_FAST_RPM=30
   LLM_FAST_TPM=60000
   
   # Secondary (slow/capable) provider
   LLM_SLOW_BASE_URL=https://integrate.api.nvidia.com/v1
   LLM_SLOW_API_KEY=nvapi-...
   LLM_SLOW_MODEL=meta/llama-3.3-70b-instruct
   ```

2. **Add Groq as Third Provider**
   - Groq has excellent speed (faster than Cerebras)
   - Free tier available
   - Can interleave with Cerebras to avoid rate limits

3. **Add Rate Limiting to Python**
   - Reuse token bucket logic from Rust
   - Or add simple delay between requests

---

### Medium-Term (Architecture)

4. **Provider Pool with Round-Robin**
   ```rust
   struct LlmProviderPool {
       providers: Vec<LlmProvider>,
       current: usize,
   }
   
   impl LlmProviderPool {
       fn next(&mut self) -> &LlmProvider {
           // Round-robin with rate limit checking
       }
   }
   ```

5. **Model Capability Tags**
   ```bash
   LLM_PROVIDER_1_URL=...
   LLM_PROVIDER_1_CAPABILITIES=fast,triage
   LLM_PROVIDER_2_URL=...
   LLM_PROVIDER_2_CAPABILITIES=deep-analysis,reasoning
   ```

---

### Long-Term (Advanced)

6. **Adaptive Provider Selection**
   - Simple findings → Fast provider
   - Complex findings → Slow provider
   - Based on finding category/severity

7. **Queue-Based Processing**
   - Queue findings when rate limited
   - Process in background
   - Return results asynchronously

---

## Testing Plan

### Immediate Tests

1. **Sample 50 packages** from diverse categories
2. **Scan with `--llm` flag** to verify Cerebras integration
3. **Monitor rate limiting** - track when 30 RPM hits
4. **Verify LLM verdicts** appear in output

### Metrics to Track

| Metric | Target | Current |
|--------|--------|---------|
| Scan speed (pkg/sec) | >10 | ? |
| LLM latency (ms) | <5000 | ? |
| Rate limit hits | <5% | ? |
| FP reduction | >80% | ? |

---

## Next Steps

1. ✅ Sample 50 diverse packages
2. ✅ Run scan with `--llm` flag
3. ✅ Verify LLM API calls succeed
4. ⏳ Document actual performance
5. ⏳ Implement unified config (if needed)
6. ⏳ Add Groq provider (if needed)

---

**End of Analysis**
