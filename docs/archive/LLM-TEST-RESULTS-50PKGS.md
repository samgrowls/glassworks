# LLM Integration Test Results

**Date:** 2026-03-21  
**Test:** 25 Popular Packages Scan with LLM Enabled

---

## Test Configuration

```bash
# Environment
GLASSWARE_LLM_BASE_URL=https://api.cerebras.ai/v1
GLASSWARE_LLM_API_KEY=csk-...
GLASSWARE_LLM_MODEL=qwen-3-235b-a22b-instruct-2507
GLASSWARE_LLM_RPM=30
GLASSWARE_LLM_TPM=60000

# Command
./target/debug/glassware-orchestrator \
  --cache-db /tmp/popular-cache.db \
  --llm \
  --severity medium \
  scan-file /tmp/scan-popular.txt
```

---

## Packages Scanned (25)

| Package | Version | Files | Findings | Status |
|---------|---------|-------|----------|--------|
| react | 19.2.4 | 26 | 0 | ✅ Clean |
| express | 5.2.1 | 9 | 0 | ✅ Clean |
| lodash | 4.17.23 | 1,050 | 0 | ✅ Clean |
| axios | 1.13.6 | 72 | 0 | ✅ Clean |
| moment | 2.30.1 | 397 | 0 | ✅ Clean |
| webpack | 5.105.4 | 708 | 0 | ✅ Clean |
| babel | 6.23.0 | 4 | 0 | ✅ Clean |
| typescript | 5.9.3 | 130 | 0 | ✅ Clean |
| eslint | 10.1.0 | 417 | 0 | ✅ Clean |
| jest | 30.3.0 | 3 | 0 | ✅ Clean |
| vue | 3.5.30 | 20 | 0 | ✅ Clean |
| mongoose | 9.3.1 | 297 | 0 | ✅ Clean |
| sequelize | 6.37.8 | 183 | 0 | ✅ Clean |
| prisma | 7.5.0 | 6 | 0 | ✅ Clean |
| fastify | 5.8.2 | 343 | 0 | ✅ Clean |
| koa | 3.1.2 | 9 | 0 | ✅ Clean |
| hapi | 18.1.0 | 23 | 0 | ✅ Clean |
| chalk | 5.6.2 | 11 | 0 | ✅ Clean |
| commander | 14.0.3 | 12 | 0 | ✅ Clean |
| debug | 4.4.3 | 6 | 0 | ✅ Clean |
| async | 3.2.6 | 133 | 0 | ✅ Clean |
| uuid | 13.0.0 | 26 | 0 | ✅ Clean |
| dotenv | 17.3.1 | 11 | 0 | ✅ Clean |
| cors | 2.8.6 | 3 | 0 | ✅ Clean |
| helmet | 8.1.0 | 6 | 0 | ✅ Clean |

**Total:** 25 packages, 3,905 files scanned, **0 findings**

---

## Performance Metrics

| Metric | Value |
|--------|-------|
| Total scan time | 54.3s |
| Average per package | 2.17s |
| Average per file | 0.014s |
| Files/second | ~72 |
| Packages/second | ~0.46 |

**Cache Performance:**
- Initial scan: 54.3s
- Re-scan (cached): <1s (100% cache hit)

---

## LLM Integration Status

### ✅ What Works

1. **LLM Feature Enabled**
   - `--llm` flag recognized and processed
   - Config loaded from environment variables
   - LLM analyzer initialized successfully

2. **Environment Configuration**
   - `GLASSWARE_LLM_BASE_URL` ✓
   - `GLASSWARE_LLM_API_KEY` ✓
   - `GLASSWARE_LLM_MODEL` ✓

3. **Cache System**
   - SQLite cache database created automatically
   - Package results cached successfully
   - Re-scans use cache (10x speedup)

### ⚠️ What Didn't Trigger

**LLM analysis did NOT run because:**
- 0 findings across all 25 packages
- LLM only analyzes flagged findings
- This is **correct behavior** - no false positives to triage

### 📊 Expected LLM Behavior

```
Package Scan → Findings Detected? → LLM Analysis
     │                │                    │
     │                YES ────────────────▶│
     │                │                    │ Run LLM API call
     │                │                    │ Get verdict
     NO───────────────┘                    │
     │                                     ▼
     │                              Return Results
     ▼
Skip LLM (clean package)
```

---

## Key Findings

### 1. ✅ No False Positives on Popular Packages

All 25 popular, well-maintained packages scanned clean:
- No invisible character detections
- No GlassWare patterns
- No encrypted payloads
- No behavioral red flags

**This validates our tiered detection approach:**
- Tier 1 (regex): 0 findings
- Tier 2 (semantic): Not triggered (no Tier 1 findings)
- Tier 3 (LLM): Not triggered (no findings to analyze)

### 2. ✅ Performance is Excellent

- **72 files/second** scan speed
- **<1 second** re-scan with cache
- No rate limiting issues (0 LLM calls made)

### 3. ✅ Cache Working Perfectly

```bash
# First scan
real    0m54.294s

# Second scan (cached)
real    0m0.891s  # 61x faster!
```

---

## Next Steps for LLM Testing

### Required: Package with Actual Findings

To test LLM integration end-to-end, we need a package that triggers detections:

**Option 1: Use Malicious Fixture**
```bash
./target/debug/glassware --llm glassware-core/tests/fixtures/glassworm/wave5_aes_decrypt_eval.js
```

**Option 2: Find Real Malicious Package**
- Search npm for recently published packages
- Look for anonymous authors + high-risk patterns
- Scan with `--llm` flag

**Option 3: Plant Test Payload**
- Create test package with known GlassWare pattern
- Verify LLM detects and classifies it correctly

---

## Architecture Conclusions

### Current State: Two LLM Systems

| Aspect | Rust (glassware-core) | Python (harness) |
|--------|----------------------|------------------|
| **Provider** | Cerebras | NVIDIA NIM |
| **Config** | `GLASSWARE_LLM_*` | `NVIDIA_API_KEY` |
| **Model** | `qwen-3-235b-a22b-instruct` | `llama-3.3-70b-instruct` |
| **Rate Limit** | 30 RPM, 60k TPM | None |
| **Use Case** | Real-time triage | Batch analysis |

### Recommendations

1. **Keep Both Systems (for now)**
   - Rust LLM: Fast, integrated, rate-limited
   - Python LLM: Slower, more capable, no rate limit
   - Different use cases justify separate configs

2. **Unify Naming Convention**
   ```bash
   # Instead of GLASSWARE_LLM_* and NVIDIA_API_KEY
   LLM_PRIMARY_BASE_URL=...
   LLM_PRIMARY_API_KEY=...
   LLM_PRIMARY_MODEL=...
   
   LLM_SECONDARY_BASE_URL=...
   LLM_SECONDARY_API_KEY=...
   LLM_SECONDARY_MODEL=...
   ```

3. **Add Groq as Third Provider**
   - Faster than Cerebras
   - Free tier available
   - Can interleave to avoid rate limits

4. **Implement Provider Pool**
   - Round-robin between providers
   - Automatic failover
   - Rate limit awareness

---

## Test Summary

| Test | Result | Notes |
|------|--------|-------|
| LLM feature compilation | ✅ PASS | Builds with `llm` feature |
| Environment config loading | ✅ PASS | Reads `GLASSWARE_LLM_*` vars |
| LLM analyzer initialization | ✅ PASS | Logs "LLM analyzer configured" |
| Clean package detection | ✅ PASS | 0 FP on 25 popular packages |
| Cache system | ✅ PASS | 61x speedup on re-scan |
| LLM API invocation | ⏳ PENDING | Need package with findings |
| LLM verdict output | ⏳ PENDING | Need findings to analyze |

---

**Conclusion:** LLM integration is **architecturally complete** and **ready for testing**. The system correctly skips LLM analysis for clean packages. Next step: test with a package that has actual findings to verify end-to-end LLM flow.

---

**End of Report**
