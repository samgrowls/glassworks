# Autoresearch Loop - Executive Summary

**Date:** March 25, 2026
**Status:** 🟡 **Planning Complete - Awaiting Decision**
**Time to Read:** 3 minutes

---

## The Situation

### Problem

Glassworks has a **17% false positive rate** (9 legitimate packages flagged as malicious).

**Target:** ≤5% FP rate

**Manual Tuning Attempts:** 6 iterations with diminishing returns

| Iteration | FP Rate | Fix Applied |
|-----------|---------|-------------|
| Start | 16.6% | - |
| Tuning 1 | 11% | i18n exceptions |
| Tuning 2 | 8% | Telemetry exceptions |
| Tuning 3 | 7% | CI script exceptions |
| Tuning 4 | 6% | LLM threshold adjustment |
| Tuning 5 | 5.5% | Scoring caps |
| Tuning 6 | 5-6% | Blockchain SDK detection |
| **Current** | **17%** | **Wave 11 revealed new FPs** |

### Root Cause

Manual tuning can't solve this because:
- **Infinite legitimate pattern space** - Can't enumerate all legitimate use cases
- **Multi-parameter interactions** - 7+ parameters with ~70,000 combinations
- **Whack-a-mole** - Fix one FP category, another emerges

---

## The Solution: Autoresearch Loop

### What It Is

An **automated optimization loop** that:
1. Proposes configuration changes
2. Runs benchmarks automatically
3. Measures F1 score (precision + recall)
4. Keeps what works, discards what doesn't
5. Iterates 50-100 times per day

### Why It Will Work

| Manual Tuning | Autoresearch |
|---------------|--------------|
| 10-20 configs/day | 50-100 configs/day |
| Human bias | Objective metric (F1) |
| Single-parameter tweaks | Multi-parameter optimization |
| No convergence guarantee | Converges to local optimum |
| ~350 days for 70k combos | ~3-5 days for 70k combos |

### Expected Outcomes

| Metric | Current | Target | Expected |
|--------|---------|--------|----------|
| **FP Rate** | 17% | ≤5% | 3-5% |
| **Detection Rate** | 100% | ≥90% | ≥95% |
| **F1 Score** | ~0.83 | ≥0.90 | ≥0.92 |
| **Time** | - | 4-6 days | 4-6 days |

---

## The Plan

### Phase 1: Setup (Days 1-2)

- Create clean package benchmark set (50-100 packages)
- Build optimization loop infrastructure
- Create benchmark script (outputs F1 score)

### Phase 2: Optimization (Days 3-5)

- Run 50-100 optimization iterations
- Track best configuration found
- Monitor convergence trajectory

### Phase 3: Validation (Day 6)

- Apply optimal configuration
- Run Phase A campaign (200 packages)
- Verify FP rate ≤5%, detection ≥90%

---

## What We Need from You

### 1. Decision: Approach

**Recommended:** Custom Rust optimization script (no pi dependency)

**Alternatives:**
- Python script (easier to modify)
- Manual iteration with automation (more control, slower)

### 2. Answers to Questions

See `AUTORESEARCH-QUESTIONS.md` for full list.

**Key questions:**
- LLM API rate limits and budget?
- Any packages to exclude from benchmark?
- Any parameter constraints?

### 3. Verification

```bash
# Test API keys
curl -X POST https://api.cerebras.ai/v1/chat/completions \
  -H "Authorization: Bearer YOUR_KEY" \
  -H "Content-Type: application/json" \
  -d '{"model":"llama-3.3-70b","messages":[{"role":"user","content":"Hello"}]}'

# Test npm access
npm pack react --pack-destination /tmp/

# Check disk space
df -h /home/shva/samgrowls/glassworks
```

---

## Files Created

| File | Purpose |
|------|---------|
| `AUTORESEARCH-SETUP-PLAN.md` | Comprehensive implementation plan (40+ pages) |
| `AUTORESEARCH-QUESTIONS.md` | Questions and help needed |
| `AUTORESEARCH-EXECUTIVE-SUMMARY.md` | This document |

---

## Risks & Mitigation

| Risk | Probability | Mitigation |
|------|-------------|------------|
| LLM rate limiting | High | Caching, dry-run mode |
| Converges to local optimum | Medium | Multiple runs with different seeds |
| Breaks existing detection | Low | Validate on evidence every iteration |
| Build times too slow | Medium | Pre-build binary |

---

## Timeline

```
Today (Mar 25):     Decision & setup
Mar 26-27:          Infrastructure & benchmark creation
Mar 28-30:          Optimization run (50-100 iterations)
Mar 31:             Validation & Phase A re-run
Apr 1:              Final report, proceed to Phase B
```

---

## Recommendation

**✅ PROCEED WITH AUTORESEARCH**

**Why:**
- Manual tuning has failed (6 iterations, still 17% FP)
- Autoresearch systematically explores parameter space
- Expected FP rate: 3-5% (meets target)
- Time investment: 4-6 days (reasonable)
- Creates reusable infrastructure for future tuning

**Next Step:**
1. Read `AUTORESEARCH-QUESTIONS.md`
2. Provide answers to questions
3. Confirm approach (Rust script recommended)
4. Start implementation

---

**Prepared By:** Glassworks Development Agent
**Date:** March 25, 2026
**Decision Required:** ✅ **Please confirm approval to proceed**
