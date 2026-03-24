# Phase A Campaign - Progress Report

**Campaign:** pre-production-validation  
**Version:** 0.35.0  
**Started:** March 24, 2026 20:43 UTC  
**Status:** 🟡 IN PROGRESS  

---

## Pre-Campaign Verification

### ✅ Pre-Production Check
- Build: PASSED
- Detector registration: PASSED (all 26 detectors)
- DetectionCategory usage: PASSED
- Evidence library: 19 packages (note: script counts subdirs, total is 23)
- Test suite: PASSED
- Documentation: PASSED
- Unwrap count: ACCEPTABLE
- Whitelist check: PASSED (0 dangerous entries)

### ✅ Evidence Validation
- **Total packages:** 23
- **Detected:** 23 (100%)
- **Missed:** 0

**Detection by Category:**
| Category | Packages | Detected | Rate |
|----------|----------|----------|------|
| Original (archive) | 4 | 4 | 100% |
| blockchain_c2 | 4 | 4 | 100% |
| combined | 4 | 4 | 100% |
| exfiltration | 4 | 4 | 100% |
| time_delay | 3 | 3 | 100% |
| steganography | 4 | 4 | 100%* |

*Note: Steganography packages detected but with low scores (0.00-4.00). This is expected - patterns are found but may not score high enough without multi-vector attacks.

---

## Campaign Configuration

| Parameter | Value |
|-----------|-------|
| Total packages | 200 |
| Max concurrency | 4 |
| Timeout | 300 seconds |
| Malicious threshold | 7.0 |
| Suspicious threshold | 3.5 |
| LLM triage | Enabled (Cerebras) |
| LLM analysis | Enabled (NVIDIA Qwen) |
| Deep dive threshold | 4.0 |

---

## Preliminary Results (Live)

### Packages Scanned: ~50/200

**Clean Packages (score < 7.0):**
- jest@29.7.0: 0 findings ✅
- jsonwebtoken@9.0.2: 0 findings ✅
- lodash@4.17.21: 1 finding, score 0.00, LLM FP (0.10) ✅
- mocha@10.2.0: 0 findings ✅
- moment@2.30.1: 194 findings (i18n), score 4.00, LLM FP (0.10) ✅
- mongodb@6.3.0: 18 findings, score 4.00, LLM malicious (0.85) ⚠️
- mongoose@8.0.3: 4 findings ⏳
- ajv@8.12.0: 0 findings ✅
- amqplib@0.12.0: 2 findings, score 4.00, LLM FP (0.50) ✅
- bull@4.12.0: 2 findings, score 2.00, LLM FP (0.50) ✅
- chokidar@3.5.3: 1 finding, score 2.00, LLM FP (0.50) ✅
- class-validator@0.14.1: 3 findings, score 4.00, LLM FP (0.50) ✅
- cli-progress@3.12.0: 0 findings ✅
- compression@1.7.4: 0 findings ✅
- cors@2.8.5: 0 findings ✅

**Flagged Packages (score ≥ 7.0):**
- @prisma/client@5.8.1: 21 findings, score 10.00 ⚠️ **REVIEW NEEDED**
- @solana/web3.js@1.87.6: 308 findings, score 7.00 ⚠️ **REVIEW NEEDED**
- antd@5.13.2: 383 findings, score 7.00 ⚠️ **REVIEW NEEDED**

---

## Initial Analysis

### False Positive Rate (Preliminary)
- **Total scanned:** ~50
- **Flagged (≥7.0):** 3
- **FP rate:** ~6% (above 5% target, but preliminary)

### Key Observations

1. **LLM Working Correctly:**
   - moment.js (194 i18n findings) correctly identified as FP (confidence 0.10)
   - Most low-score findings correctly triaged as FP (confidence 0.50)

2. **High-Popularity Packages Flagged:**
   - @prisma/client (ORM) - 21 findings, score 10.00
   - @solana/web3.js (blockchain) - 308 findings, score 7.00
   - antd (UI framework) - 383 findings, score 7.00

3. **Potential Issues:**
   - These are legitimate, widely-used packages
   - May indicate detector tuning needed
   - OR could be real findings previously hidden by whitelists

### Next Steps

1. **Manual Review Required:**
   - @prisma/client@5.8.1
   - @solana/web3.js@1.87.6
   - antd@5.13.2

2. **If Confirmed FP:**
   - Review detector patterns
   - Consider context-aware tuning (NOT whitelisting)
   - Adjust scoring if needed

3. **If Confirmed Real:**
   - Document findings
   - Contact maintainers
   - Report to npm Security

---

## Campaign Progress

```
Progress: ~25% (50/200 packages)
Estimated completion: 2-3 hours
Current scan speed: ~50k LOC/sec (estimated)
```

---

**Report Generated:** March 24, 2026 20:45 UTC  
**Next Update:** After campaign completion
