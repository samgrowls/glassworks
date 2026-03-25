# Questions Answered - Implementation Ready

**Date:** March 25, 2026
**Status:** ✅ **ALL QUESTIONS ANSWERED - READY TO IMPLEMENT**
**Source:** PROMPT9.md (Comprehensive Approval & Guidance)

---

## Summary

**PROMPT9.md** provides complete answers to all questions from `AUTORESEARCH-QUESTIONS.md`. 

**Status:** ✅ **APPROVED FOR IMPLEMENTATION**

**Timeline:** 4-6 days confirmed
**Tag:** v0.43.0-autoresearch-complete (upon success)

---

## Questions Resolution Table

| # | Question | Answer Summary | Status |
|---|----------|----------------|--------|
| 1 | **Implementation Approach** | ✅ Custom Rust script in `glassware-tools/src/bin/autoresearch.rs` | ANSWERED |
| 2 | **LLM API Rate Limits** | ✅ Phase 1: NO LLM, Phase 2: Add LLM after convergence | ANSWERED |
| 3 | **Benchmark Package Selection** | ✅ 96 clean packages provided (use 50 for iteration, 96 for validation) | ANSWERED |
| 4 | **Parameter Constraints** | ✅ Full list of modifiable vs. fixed parameters with ranges | ANSWERED |
| 5 | **Timeline** | ✅ 4-6 days approved (March 31 - April 5) | ANSWERED |

---

## Detailed Answers

### 1. Implementation Approach ✅

**Decision:** Custom Rust implementation

**Location:** `glassware-tools/src/bin/autoresearch.rs`

**Structure:**
```rust
mod config;        // Parameter ranges, optimization settings
mod benchmark;     // Package scanning, result collection
mod optimizer;     // Configuration proposal logic
mod metrics;       // F1 score, FP rate calculation
mod report;        // Iteration logging, final report

fn main() -> Result<()> {
    let config = load_config("autoresearch.toml")?;
    let engine = AutoresearchEngine::new(config);
    let result = engine.run()?;
    generate_report(&result)?;
    Ok(()
}
```

**Why Rust:**
- Glassworks is 100% Rust - keeps ecosystem consistent
- Direct integration with scoring engine (no FFI needed)
- Type-safe configuration handling
- Can be shipped as part of glassware-tools crate
- Future tuning can reuse same infrastructure

---

### 2. LLM API Rate Limits ✅

**Strategy:** Two-phase approach

**Phase 1:** Optimize WITHOUT LLM
- Pure scoring optimization
- Avoids rate limit issues entirely
- ~80% reduction in API calls

**Phase 2:** Add LLM after scoring converged
- Use cached responses where possible
- Rate limit: Space requests (1/sec)
- Backoff on 429 responses (exponential, max 5 retries)

**Implementation:**
```rust
// Rate limiter (only needed in Phase 2)
pub struct LLMRateLimiter {
    cerebras_requests: SlidingWindow,
    cerebras_limit_per_min: u32, // ~20-60 depending on tier
}

impl LLMRateLimiter {
    pub async fn wait_for_cerebras(&self) {
        while self.cerebras_requests.count() >= self.cerebras_limit_per_min {
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
        self.cerebras_requests.record();
    }
}
```

---

### 3. Benchmark Package Selection ✅

**Clean Package Set:** 96 packages total

**Categories:**
- UI Frameworks (8): react, vue, @angular/core, svelte, preact, solid-js, alpinejs, lit
- Build Tools (8): webpack, vite, rollup, esbuild, parcel, @swc/core, @babel/core, turbo
- Languages (4): typescript, @babel/core, @swc/core, coffeescript
- Testing (6): jest, vitest, mocha, chai, cypress, @playwright/test
- HTTP (5): axios, node-fetch, got, request, superagent
- Database (6): prisma, @prisma/client, mongoose, sequelize, knex, typeorm
- Blockchain (5): @solana/web3.js, ethers, viem, web3, @ethersproject/contracts
- Cloud SDKs (5): firebase, @aws-sdk/client-s3, @azure/storage-blob, @google-cloud/storage, @supabase/supabase-js
- Monitoring (4): @sentry/node, newrelic, datadog-metrics, opentelemetry-api
- Utilities (10): lodash, moment, dayjs, date-fns, chalk, commander, uuid, dotenv, debug, async
- Web Frameworks (8): express, fastify, koa, hapi, next, nuxt, gatsby, remix
- State Management (5): redux, vuex, mobx, zustand, recoil
- Styling (5): tailwindcss, styled-components, emotion, sass, postcss
- Misc High-Download (5): socket.io, jsonwebtoken, uuid, dotenv, winston

**Usage:**
- **Iteration:** Use 50-package subset for speed
- **Final Validation:** Use all 96 packages

**Explicit Exclusions:**
- Any package with score ≥8.0 in Wave 11 (use for evidence, not clean set)
- Packages <6 months old
- Packages <1K weekly downloads
- Packages with existing npm advisories

---

### 4. Parameter Constraints ✅

**Modifiable Parameters (11 total):**

| Parameter | Current | Range | Step | Priority |
|-----------|---------|-------|------|----------|
| malicious_threshold | 8.0 | 6.5 - 9.0 | 0.25 | CRITICAL |
| suspicious_threshold | 4.0 | 3.0 - 5.5 | 0.25 | HIGH |
| llm_override_threshold | 0.95 | 0.85 - 0.99 | 0.02 | MEDIUM |
| llm_multiplier_min | 0.2 | 0.1 - 0.5 | 0.05 | MEDIUM |
| reputation_tier_1 | 0.3 | 0.2 - 0.5 | 0.05 | HIGH |
| reputation_tier_2 | 0.5 | 0.3 - 0.7 | 0.05 | HIGH |
| category_cap_1 | 5.0 | 4.0 - 6.5 | 0.25 | MEDIUM |
| category_cap_2 | 7.0 | 6.0 - 8.5 | 0.25 | LOW |
| category_cap_3 | 8.5 | 7.5 - 9.5 | 0.25 | LOW |
| dedup_similarity | 0.8 | 0.6 - 0.9 | 0.05 | MEDIUM |
| log_weight_base | 1.0 | 0.5 - 1.5 | 0.1 | LOW |

**Fixed Constraints (DO NOT MODIFY):**
- Evidence detection rate: ≥90% (hard constraint)
- Known C2 exception score: ≥9.0 (fixed)
- Steganography+decoder exception score: ≥8.5 (fixed)
- Detector weights: 1.0 each (fixed)
- LLM provider endpoints: Fixed
- Evidence package list: Fixed (23 packages)

**Hard Constraints:**
- Evidence detection: ≥90% (reject if violated)
- FP rate: ≤5% (optimization target)
- Scan speed: ≥30k LOC/sec (warn if <30k, reject if <20k)
- Malicious threshold: 6.5-9.0 (don't allow too permissive/strict)

---

### 5. Timeline ✅

**Approved:** 4-6 days (March 31 - April 5, 2026)

**Detailed Schedule:**

| Day | Phase | Deliverables | Hours |
|-----|-------|--------------|-------|
| 1 | Infrastructure Setup | autoresearch.rs skeleton, config loading, benchmark suite, test on 5 packages | 6-8 |
| 2 | Optimization Engine | Parameter sampler, F1 calculator, iteration loop, logging/reporting | 6-8 |
| 3 | Run Optimization (Part 1) | 25-30 iterations, mid-point analysis, adjust if not converging | 4-6 |
| 4 | Run Optimization (Part 2) | 25-30 more iterations, convergence verification, best config identified | 4-6 |
| 5 | Validation | Run Phase A packages, verify FP rate ≤5%, verify evidence ≥90%, document results | 4-6 |
| 6 | Buffer/Phase B Prep | Catch up if behind, prepare Phase B config, final report | 4-6 |

**Total Estimated Hours:** 28-40 hours over 6 days

**Daily Check-in Format:**
```markdown
# Daily Autoresearch Progress Report - Day X

## Summary
- Iterations completed: X/50-100
- Best F1 score so far: 0.XX
- Best FP rate: X.X%
- Best evidence detection: XX%

## Top 3 Configurations
| Rank | FP Rate | Detection | F1 | Key Parameters |
|------|---------|-----------|----|----------------|
| 1 | X.X% | XX% | 0.XX | mal=7.5, rep1=0.3, ... |
| 2 | X.X% | XX% | 0.XX | ... |
| 3 | X.X% | XX% | 0.XX | ... |

## Convergence Status
- Trend: Improving / Stable / Needs adjustment
- Estimated iterations to convergence: X

## Blockers/Issues
- [Any issues encountered]

## Tomorrow's Plan
- [Next steps]
```

---

## Success Criteria

### Primary Metrics (Must Achieve)

| Metric | Target |
|--------|--------|
| FP Rate on clean benchmark | ≤5% |
| Evidence detection rate | ≥90% (ideally 100%) |
| F1 Score | ≥0.90 |
| Scan speed | ≥30k LOC/sec |

### Secondary Metrics (Nice to Have)

| Package | Target Score |
|---------|--------------|
| @prisma/client | <5.0 |
| @solana/web3.js | <5.0 |
| typescript | <5.0 |
| webpack | <5.0 |
| firebase | <5.0 |

### Phase B Readiness (After Autoresearch)

- ✅ All primary metrics met
- ✅ Configuration documented and committed
- ✅ Phase B config prepared
- ✅ Rollback procedure documented
- ✅ Daily reporting format established

---

## Risk Mitigation

### Risk 1: Optimization doesn't converge to ≤5% FP

**Mitigation:**
- Expand parameter ranges if stuck
- Try different optimization algorithm (simulated annealing vs Bayesian)
- Accept 6-7% FP if evidence detection is 100%
- Document as "best achievable" and proceed to Phase B with manual review

### Risk 2: LLM rate limits block progress

**Mitigation:**
- Phase 1: Run WITHOUT LLM (pure scoring)
- Phase 2: Add LLM after scoring converged
- Cache all LLM responses
- Use Cerebras only (faster, cheaper) for optimization

### Risk 3: Benchmark set contaminated with malicious packages

**Mitigation:**
- Use only packages with 0 npm advisories
- Use only packages >1 year old
- Use only packages >100K weekly downloads
- Cross-reference with Socket.dev clean list

### Risk 4: Optimization overfits to benchmark set

**Mitigation:**
- Hold out 20% of clean packages for final validation
- Optimize on 80%, validate on 20%
- If gap >2%, reduce overfitting (simpler config)

### Risk 5: Takes longer than 6 days

**Mitigation:**
- Parallelize benchmark scanning (8 concurrent)
- Use subset (50 packages) for iteration, full (96) for validation
- Skip LLM during optimization
- Accept Day 7-8 if needed for quality

---

## Rollback Procedure

```bash
# If autoresearch fails or produces worse results

# 1. Save current best config
cp config/scoring.toml config/scoring-autoresearch-backup.toml

# 2. Revert to pre-autoresearch config
git checkout v0.42.0-autoresearch-plan -- config/scoring.toml

# 3. Document what went wrong
echo "$(date): Autoresearch failed - [reason]" >> INCIDENT_LOG.md

# 4. Analyze iteration logs
cat output/autoresearch/iteration-*.json | jq '.config, .metrics' > failed-iterations.json

# 5. Decide: retry with adjustments or proceed with manual config
```

---

## Git Workflow

1. **Create branch:** `feature/autoresearch-implementation`
2. **Commit incrementally:** Each major component
3. **Tag on completion:** `v0.43.0-autoresearch-complete`
4. **Merge to main:** After validation passes

---

## Implementation Checklist

### Day 1: Infrastructure Setup

- [ ] Create `glassware-tools/src/bin/autoresearch.rs` skeleton
- [ ] Implement config loading (`autoresearch.toml`)
- [ ] Create benchmark suite structure
- [ ] Download 96 clean packages
- [ ] Test on 5 packages (verify infrastructure works)

### Day 2: Optimization Engine

- [ ] Implement parameter sampler
- [ ] Implement F1 calculator
- [ ] Implement iteration loop
- [ ] Implement logging/reporting
- [ ] Test full iteration cycle

### Day 3-4: Run Optimization

- [ ] Run 50-100 iterations
- [ ] Monitor convergence
- [ ] Adjust if not converging
- [ ] Identify best configuration

### Day 5: Validation

- [ ] Apply best configuration
- [ ] Run Phase A packages (200 packages)
- [ ] Verify FP rate ≤5%
- [ ] Verify evidence detection ≥90%
- [ ] Document results

### Day 6: Buffer/Phase B Prep

- [ ] Catch up if behind
- [ ] Prepare Phase B config
- [ ] Create final report
- [ ] Create git tag v0.43.0-autoresearch-complete

---

## Official Approval

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         OFFICIAL APPROVAL                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Project: Glassworks Autoresearch Loop Implementation                       │
│  Status: ✅ APPROVED FOR IMPLEMENTATION                                     │
│                                                                             │
│  Timeline: 4-6 days (March 31 - April 5, 2026)                              │
│  Tag: v0.43.0-autoresearch-complete                                         │
│                                                                             │
│  Conditions:                                                                │
│  ✓ Custom Rust implementation (no pi-autoresearch dependency)              │
│  ✓ Phase 1: Optimize WITHOUT LLM (avoid rate limits)                       │
│  ✓ Phase 2: Add LLM after scoring converged                                │
│  ✓ Clean benchmark: 96 packages (use 50 for iteration, 96 for validation)  │
│  ✓ Evidence benchmark: 23 packages (must maintain ≥90% detection)          │
│  ✓ Parameter constraints respected                                         │
│  ✓ Daily progress reports in specified format                              │
│  ✓ Rollback procedure documented                                           │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Next Action

**✅ READY TO IMPLEMENT**

1. Create feature branch: `feature/autoresearch-implementation`
2. Start Day 1 tasks (Infrastructure Setup)
3. Provide Daily Progress Report at end of Day 1

---

**Document Created By:** Glassworks Development Agent
**Date:** March 25, 2026
**Status:** ✅ **ALL QUESTIONS ANSWERED - READY TO IMPLEMENT**
**Source:** PROMPT9.md (Comprehensive Approval & Guidance)
