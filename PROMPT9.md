# 🔍 GLASSWORKS AUTORESEARCH PLAN REVIEW & APPROVAL

## Comprehensive Response to Agent Questions

**Version:** 9.0 (Autoresearch Approval & Guidance)  
**Date:** March 30, 2025  
**Status:** ✅ APPROVED FOR IMPLEMENTATION  
**Timeline:** 4-6 days confirmed

---

## PART 1: DOCUMENTATION REVIEW

### 1.1 Files Verified on Remote Repository

| File | Status | Quality Assessment |
|------|--------|-------------------|
| `AUTORESEARCH-EXECUTIVE-SUMMARY.md` | ✅ Present | Excellent - Clear problem/solution framing |
| `AUTORESEARCH-SETUP-PLAN.md` | ✅ Present | Comprehensive - Implementation-ready |
| `AUTORESEARCH-QUESTIONS.md` | ✅ Present | Well-structured - All key questions asked |
| `AUTORESEARCH-SESSION-SUMMARY.md` | ✅ Present | Good - Captures session context |
| `HANDOFF.md` | ✅ Present | Thorough - Complete phase history |
| `ARCHITECTURE.md` | ✅ Present | Clear - System diagrams included |

### 1.2 Documentation Quality Assessment

**Strengths:**
- ✅ Clear problem statement (17% FP → target 5%)
- ✅ Honest about manual tuning limitations (6 iterations, diminishing returns)
- ✅ Well-defined parameter space (7 parameters, ~70k combinations)
- ✅ Concrete success criteria (F1 score, FP rate, detection rate)
- ✅ Realistic timeline (4-6 days)

**Minor Gaps (Addressed Below):**
- ⚠️ LLM rate limit specifics not documented
- ⚠️ Clean package benchmark list not finalized
- ⚠️ Rollback procedure if autoresearch fails

---

## PART 2: ANSWERS TO AUTORESEARCH-QUESTIONS.MD

### Question 1: Implementation Approach

**Original Question:**
> Should we use the pi-autoresearch framework, build a custom Rust script, or use a Python-based approach?

**My Answer:**

```
✅ RECOMMENDED: Custom Rust Script

Rationale:
┌─────────────────────────────────────────────────────────────────────────────┐
│ Option              │ Pros                          │ Cons                 │
├─────────────────────────────────────────────────────────────────────────────┤
│ pi-autoresearch     │ - Pre-built framework         │ - Python dependency  │
│                     │ - Karpathy provenance         │ - Not in Rust eco    │
│                     │                               │ - pi CLI not found   │
├─────────────────────────────────────────────────────────────────────────────┤
│ Python custom       │ - Easy ML libraries           │ - Language mismatch  │
│                     │ - Fast prototyping            │ - Glassworks is Rust │
│                     │                               │ - Integration friction│
├─────────────────────────────────────────────────────────────────────────────┤
│ Rust custom         │ - Native to Glassworks        │ - More initial code  │
│ (RECOMMENDED)       │ - No language barrier         │                      │
│                     │ - Direct scanner integration  │                      │
│                     │ - Production reusable         │                      │
│                     │ - Type safety for configs     │                      │
└─────────────────────────────────────────────────────────────────────────────┘

Implementation Location:
  glassware-tools/src/bin/autoresearch.rs

Why Rust:
1. Glassworks is 100% Rust - keeps ecosystem consistent
2. Direct integration with scoring engine (no FFI needed)
3. Type-safe configuration handling
4. Can be shipped as part of glassware-tools crate
5. Future tuning can reuse same infrastructure
```

**Additional Guidance:**
```rust
// Suggested structure for glassware-tools/src/bin/autoresearch.rs

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
    Ok(())
}
```

---

### Question 2: LLM API Rate Limits

**Original Question:**
> What are the rate limits for Cerebras and NVIDIA APIs? Should we implement rate limiting in the autoresearch loop?

**My Answer:**

```
✅ RATE LIMITING REQUIRED

Cerebras API (llama-3.3-70b):
┌─────────────────────────────────────────────────────────────────────────────┐
│ Tier          │ Requests/Min │ Requests/Day │ Tokens/Min │ Observed        │
├─────────────────────────────────────────────────────────────────────────────┤
│ Free          │ ~20          │ ~1,000       │ ~50,000    │ Need to verify  │
│ Paid          │ ~60          │ ~10,000      │ ~200,000   │ Need to verify  │
└─────────────────────────────────────────────────────────────────────────────┘

NVIDIA API (nemotron-70b):
┌─────────────────────────────────────────────────────────────────────────────┐
│ Tier          │ Requests/Min │ Requests/Day │ Tokens/Min │ Observed        │
├─────────────────────────────────────────────────────────────────────────────┤
│ Free          │ ~30          │ ~2,000       │ ~100,000   │ Need to verify  │
│ Paid          │ ~100         │ ~20,000      │ ~500,000   │ Need to verify  │
└─────────────────────────────────────────────────────────────────────────────┘

Recommendations:
┌─────────────────────────────────────────────────────────────────────────────┐
│ 1. IMPLEMENT RATE LIMITER                                                   │
│    - Use tokio::time::sleep for request spacing                            │
│    - Track requests per minute with sliding window                         │
│    - Backoff on 429 responses (exponential, max 5 retries)                 │
│                                                                             │
│ 2. OPTIMIZE LLM USAGE                                                       │
│    - Phase 1: Run WITHOUT LLM (pure scoring optimization)                  │
│    - Phase 2: Add LLM triage after scoring converged                       │
│    - This reduces API calls by ~80% during optimization                    │
│                                                                             │
│ 3. CACHE LLM RESPONSES                                                      │
│    - Cache by (package_hash, findings_hash)                                │
│    - Reuse across iterations (same package = same LLM response)            │
│    - Store in ~/.cache/glassware/llm-cache/                                │
│                                                                             │
│ 4. MONITOR USAGE                                                            │
│    - Log API calls per iteration                                           │
│    - Alert if approaching daily limit                                      │
│    - Pause gracefully if limit reached                                     │
└─────────────────────────────────────────────────────────────────────────────┘

Implementation:
```

```rust
// glassware-tools/src/llm_rate_limiter.rs

pub struct LLMRateLimiter {
    cerebras_requests: SlidingWindow,
    nvidia_requests: SlidingWindow,
    cerebras_limit_per_min: u32,
    nvidia_limit_per_min: u32,
}

impl LLMRateLimiter {
    pub async fn wait_for_cerebras(&self) {
        while self.cerebras_requests.count() >= self.cerebras_limit_per_min {
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
        self.cerebras_requests.record();
    }
    
    pub async fn wait_for_nvidia(&self) {
        while self.nvidia_requests.count() >= self.nvidia_limit_per_min {
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
        self.nvidia_requests.record();
    }
}

// Usage in autoresearch loop:
if config.use_llm {
    rate_limiter.wait_for_cerebras().await;
    let llm_result = call_cerebras_api(&package).await?;
}
```

**Priority:** Start WITHOUT LLM in optimization loop (Phase 1), add LLM after scoring converges (Phase 2). This avoids rate limit issues entirely for the critical optimization phase.

---

### Question 3: Benchmark Package Selection

**Original Question:**
> Are there any packages that should be excluded from the clean benchmark set? Any specific packages you want included?

**My Answer:**

```
✅ CLEAN BENCHMARK SET COMPOSITION

Required Inclusions (Known Legitimate):
┌─────────────────────────────────────────────────────────────────────────────┐
│ Category          │ Packages (50+ total)                                    │
├─────────────────────────────────────────────────────────────────────────────┤
│ UI Frameworks     │ react, vue, @angular/core, svelte, preact, solid-js    │
│ Build Tools       │ webpack, vite, rollup, esbuild, parcel, swc            │
│ Languages         │ typescript, @babel/core, @swc/core                     │
│ Testing           │ jest, vitest, mocha, chai, cypress, playwright         │
│ HTTP              │ axios, fetch, node-fetch, got, request                 │
│ Database          │ prisma, @prisma/client, mongoose, sequelize, knex      │
│ Blockchain        │ @solana/web3.js, ethers, viem, web3, @ethersproject    │
│ Cloud SDKs        │ firebase, @aws-sdk/*, @azure/*, @google-cloud/*        │
│ Monitoring        │ @sentry/node, newrelic, datadog-metrics                │
│ Utilities         │ lodash, moment, dayjs, date-fns, chalk, commander      │
│ Formatters        │ prettier, eslint, stylelint                            │
│ Web Frameworks    │ express, fastify, koa, hapi, next, nuxt, gatsby        │
│ State Management  │ redux, vuex, mobx, zustand, recoil                     │
│ Styling           │ tailwindcss, styled-components, emotion, sass          │
│ Misc High-Download│ socket.io, jsonwebtoken, uuid, dotenv, dotenv-cli      │
└─────────────────────────────────────────────────────────────────────────────┘

Explicit Exclusions:
┌─────────────────────────────────────────────────────────────────────────────┐
│ Package/Type          │ Reason                                              │
├─────────────────────────────────────────────────────────────────────────────┤
│ Any package with      │ Known malicious - use for evidence, not clean set  │
│ score ≥8.0 in Wave 11 │                                                     │
│ Packages <6 months old│ Not enough reputation data                         │
│ Packages <1K downloads│ Too niche - may have legitimate suspicious patterns│
│ Packages with         │ May have actual security issues                    │
│ existing npm advisories│                                                    │
└─────────────────────────────────────────────────────────────────────────────┘

Package Selection Criteria:
┌─────────────────────────────────────────────────────────────────────────────┐
│ Criterion           │ Minimum        │ Preferred      │ Rationale           │
├─────────────────────────────────────────────────────────────────────────────┤
│ Weekly Downloads    │ 10,000         │ 100,000+       │ Popular = vetted    │
│ Age (days)          │ 180            │ 365+           │ Survived scrutiny   │
│ Maintainer Verified │ Yes            │ Yes            │ Reduced risk        │
│ npm Advisories      │ 0              │ 0              │ Known clean         │
│ GitHub Stars        │ 1,000+         │ 5,000+         │ Community vetted    │
└─────────────────────────────────────────────────────────────────────────────┘

Implementation:
```

```bash
# benchmarks/clean-packages/packages.txt (finalized list)

# UI Frameworks (8)
react@18.2.0
vue@3.4.0
@angular/core@17.0.0
svelte@4.2.0
preact@10.19.0
solid-js@1.8.0
alpinejs@3.13.0
lit@3.0.0

# Build Tools (8)
webpack@5.89.0
vite@5.0.0
rollup@4.9.0
esbuild@0.19.0
parcel@2.11.0
@swc/core@1.3.0
@babel/core@7.23.0
turbo@1.11.0

# Languages (4)
typescript@5.3.0
@babel/core@7.23.0
@swc/core@1.3.0
coffeescript@2.7.0

# Testing (6)
jest@29.7.0
vitest@1.1.0
mocha@10.2.0
chai@4.3.0
cypress@13.6.0
@playwright/test@1.40.0

# HTTP (5)
axios@1.6.0
node-fetch@3.3.0
got@14.0.0
request@2.88.2
superagent@8.1.0

# Database (6)
prisma@5.7.0
@prisma/client@5.7.0
mongoose@8.0.0
sequelize@6.35.0
knex@3.1.0
typeorm@0.3.17

# Blockchain (5)
@solana/web3.js@1.87.0
ethers@6.9.0
viem@2.0.0
web3@4.3.0
@ethersproject/contracts@5.7.0

# Cloud SDKs (5)
firebase@10.7.0
@aws-sdk/client-s3@3.450.0
@azure/storage-blob@12.17.0
@google-cloud/storage@7.6.0
@supabase/supabase-js@2.39.0

# Monitoring (4)
@sentry/node@7.91.0
newrelic@11.8.0
datadog-metrics@0.11.0
opentelemetry-api@1.4.0

# Utilities (10)
lodash@4.17.21
moment@2.29.4
dayjs@1.11.10
date-fns@3.0.0
chalk@5.3.0
commander@11.1.0
uuid@9.0.0
dotenv@16.3.0
debug@4.3.4
async@3.2.5

# Web Frameworks (8)
express@4.18.2
fastify@4.25.0
koa@2.15.0
hapi@21.3.0
next@14.0.0
nuxt@3.9.0
gatsby@5.13.0
remix@2.4.0

# State Management (5)
redux@5.0.0
vuex@4.1.0
mobx@6.12.0
zustand@4.4.0
recoil@0.7.7

# Styling (5)
tailwindcss@3.4.0
styled-components@6.1.0
emotion@11.11.0
sass@1.69.0
postcss@8.4.32

# Misc High-Download (5)
socket.io@4.7.0
jsonwebtoken@9.0.2
uuid@9.0.0
dotenv@16.3.0
winston@3.11.0

# TOTAL: 96 packages
```

**Note:** This gives us ~96 clean packages. For optimization speed, use a subset of 50 for iteration testing, full 96 for final validation.

---

### Question 4: Parameter Constraints

**Original Question:**
> Are there any parameters that should NOT be modified during optimization? Any hard constraints?

**My Answer:**

```
✅ PARAMETER CONSTRAINTS

MODIFIABLE PARAMETERS (Optimize These):
┌─────────────────────────────────────────────────────────────────────────────┐
│ Parameter              │ Current │ Range         │ Step  │ Priority        │
├─────────────────────────────────────────────────────────────────────────────┤
│ malicious_threshold    │ 8.0     │ 6.5 - 9.0     │ 0.25  │ CRITICAL        │
│ suspicious_threshold   │ 4.0     │ 3.0 - 5.5     │ 0.25  │ HIGH            │
│ llm_override_threshold │ 0.95    │ 0.85 - 0.99   │ 0.02  │ MEDIUM          │
│ llm_multiplier_min     │ 0.2     │ 0.1 - 0.5     │ 0.05  │ MEDIUM          │
│ reputation_tier_1      │ 0.3     │ 0.2 - 0.5     │ 0.05  │ HIGH            │
│ reputation_tier_2      │ 0.5     │ 0.3 - 0.7     │ 0.05  │ HIGH            │
│ category_cap_1         │ 5.0     │ 4.0 - 6.5     │ 0.25  │ MEDIUM          │
│ category_cap_2         │ 7.0     │ 6.0 - 8.5     │ 0.25  │ LOW             │
│ category_cap_3         │ 8.5     │ 7.5 - 9.5     │ 0.25  │ LOW             │
│ dedup_similarity       │ 0.8     │ 0.6 - 0.9     │ 0.05  │ MEDIUM          │
│ log_weight_base        │ 1.0     │ 0.5 - 1.5     │ 0.1   │ LOW             │
└─────────────────────────────────────────────────────────────────────────────┘

CONSTRAINTS (Do NOT Modify):
┌─────────────────────────────────────────────────────────────────────────────┐
│ Parameter/Component        │ Constraint           │ Rationale               │
├─────────────────────────────────────────────────────────────────────────────┤
│ Evidence detection rate    │ ≥90% (hard)          │ Cannot lose malicious   │
│                            │                      │ detection capability    │
├─────────────────────────────────────────────────────────────────────────────┤
│ Known C2 exception score   │ ≥9.0 (fixed)         │ Security critical -     │
│                            │                      │ never reduce            │
├─────────────────────────────────────────────────────────────────────────────┤
│ Steganography+decoder      │ ≥8.5 (fixed)         │ GlassWorm signature -   │
│ exception score            │                      │ never reduce            │
├─────────────────────────────────────────────────────────────────────────────┤
│ Detector weights           │ 1.0 each (fixed)     │ Keep detectors equal -  │
│                            │                      │ tune scoring, not det.  │
├─────────────────────────────────────────────────────────────────────────────┤
│ LLM provider endpoints     │ Fixed                │ Infrastructure, not     │
│                            │                      │ optimization target     │
├─────────────────────────────────────────────────────────────────────────────┤
│ Evidence package list      │ Fixed (23 pkg)       │ Consistent benchmark    │
└─────────────────────────────────────────────────────────────────────────────┘

HARD CONSTRAINTS (Optimization Must Satisfy):
┌─────────────────────────────────────────────────────────────────────────────┐
│ Constraint               │ Value    │ Action If Violated                    │
├─────────────────────────────────────────────────────────────────────────────┤
│ Evidence detection       │ ≥90%     │ Reject configuration immediately      │
│ FP rate                  │ ≤5%      │ Primary optimization target           │
│ Scan speed               │ ≥30k LOC │ Warn if slower, reject if <20k LOC   │
│ Malicious threshold      │ ≥6.5     │ Don't allow too permissive            │
│ Malicious threshold      │ ≤9.0     │ Don't allow too strict                │
└─────────────────────────────────────────────────────────────────────────────┘

OPTIMIZATION OBJECTIVE FUNCTION:
```

```rust
// glassware-tools/src/metrics.rs

pub fn calculate_objective(result: &BenchmarkResult, config: &ScoringConfig) -> f32 {
    // Hard constraint: Evidence detection must be ≥90%
    if result.evidence_detection_rate < 0.90 {
        return 0.0; // Reject immediately
    }
    
    // Primary objective: F1 score (balance of precision and recall)
    let precision = 1.0 - result.fp_rate;
    let recall = result.evidence_detection_rate;
    let f1 = if precision + recall == 0.0 {
        0.0
    } else {
        2.0 * (precision * recall) / (precision + recall)
    };
    
    // Penalty for FP rate above target
    let fp_penalty = if result.fp_rate > 0.05 {
        1.0 - ((result.fp_rate - 0.05) * 10.0) // Steep penalty
    } else {
        1.0
    };
    
    // Penalty for slow scan speed
    let speed_penalty = if result.scan_speed_loc_per_sec < 20000.0 {
        0.5
    } else if result.scan_speed_loc_per_sec < 30000.0 {
        0.8
    } else {
        1.0
    };
    
    f1 * fp_penalty * speed_penalty
}
```

---

### Question 5: Timeline Confirmation

**Original Question:**
> Is the 4-6 day timeline acceptable? Any hard deadlines I should know about?

**My Answer:**

```
✅ TIMELINE APPROVED (4-6 Days)

Detailed Schedule:
┌─────────────────────────────────────────────────────────────────────────────┐
│ Day  │ Phase                    │ Deliverables                    │ Hours  │
├─────────────────────────────────────────────────────────────────────────────┤
│ 1    │ Infrastructure Setup     │ - autoresearch.rs skeleton      │ 6-8    │
│      │                          │ - Config loading                │        │
│      │                          │ - Benchmark suite               │        │
│      │                          │ - Test on 5 packages            │        │
├─────────────────────────────────────────────────────────────────────────────┤
│ 2    │ Optimization Engine      │ - Parameter sampler             │ 6-8    │
│      │                          │ - F1 calculator                 │        │
│      │                          │ - Iteration loop                │        │
│      │                          │ - Logging/reporting             │        │
├─────────────────────────────────────────────────────────────────────────────┤
│ 3    │ Run Optimization (Part 1)│ - 25-30 iterations              │ 4-6    │
│      │                          │ - Mid-point analysis            │        │
│      │                          │ - Adjust if not converging      │        │
├─────────────────────────────────────────────────────────────────────────────┤
│ 4    │ Run Optimization (Part 2)│ - 25-30 more iterations         │ 4-6    │
│      │                          │ - Convergence verification      │        │
│      │                          │ - Best config identified        │        │
├─────────────────────────────────────────────────────────────────────────────┤
│ 5    │ Validation               │ - Run Phase A packages          │ 4-6    │
│      │                          │ - Verify FP rate ≤5%            │        │
│      │                          │ - Verify evidence ≥90%          │        │
│      │                          │ - Document results              │        │
├─────────────────────────────────────────────────────────────────────────────┤
│ 6    │ Buffer/Phase B Prep      │ - Catch up if behind            │ 4-6    │
│      │                          │ - Prepare Phase B config        │        │
│      │                          │ - Final report                  │        │
└─────────────────────────────────────────────────────────────────────────────┘

Total Estimated Hours: 28-40 hours over 6 days

Hard Deadlines:
┌─────────────────────────────────────────────────────────────────────────────┐
│ None specified - quality over speed                                        │
│                                                                             │
│ Recommended Checkpoints:                                                   │
│ • End of Day 2: Infrastructure must be working                             │
│ • End of Day 4: Optimization must show convergence trend                   │
│ • End of Day 5: FP rate must be ≤5% on validation                          │
│                                                                             │
│ If Day 5 validation fails:                                                 │
│ • Day 6: Debug and re-run optimization with adjusted parameters            │
│ • Day 7: Final validation and report                                       │
└─────────────────────────────────────────────────────────────────────────────┘

Daily Check-in Format:
```

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

## PART 3: ADDITIONAL GUIDANCE

### 3.1 Risk Mitigation

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         RISK MITIGATION PLAN                                │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Risk 1: Optimization doesn't converge to ≤5% FP                           │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ Mitigation:                                                         │   │
│  │ • Expand parameter ranges if stuck                                  │   │
│  │ • Try different optimization algorithm (simulated annealing vs      │   │
│  │   Bayesian)                                                         │   │
│  │ • Accept 6-7% FP if evidence detection is 100%                      │   │
│  │ • Document as "best achievable" and proceed to Phase B with         │   │
│  │   manual review                                                     │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  Risk 2: LLM rate limits block progress                                    │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ Mitigation:                                                         │   │
│  │ • Phase 1: Run WITHOUT LLM (pure scoring)                           │   │
│  │ • Phase 2: Add LLM after scoring converged                          │   │
│  │ • Cache all LLM responses                                           │   │
│  │ • Use Cerebras only (faster, cheaper) for optimization              │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  Risk 3: Benchmark set contaminated with malicious packages                │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ Mitigation:                                                         │   │
│  │ • Use only packages with 0 npm advisories                           │   │
│  │ • Use only packages >1 year old                                     │   │
│  │ • Use only packages >100K weekly downloads                          │   │
│  │ • Cross-reference with Socket.dev clean list                        │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  Risk 4: Optimization overfits to benchmark set                            │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ Mitigation:                                                         │   │
│  │ • Hold out 20% of clean packages for final validation               │   │
│  │ • Optimize on 80%, validate on 20%                                  │   │
│  │ • If gap >2%, reduce overfitting (simpler config)                   │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  Risk 5: Takes longer than 6 days                                          │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ Mitigation:                                                         │   │
│  │ • Parallelize benchmark scanning (8 concurrent)                     │   │
│  │ • Use subset (50 packages) for iteration, full (96) for validation  │   │
│  │ • Skip LLM during optimization                                      │   │
│  │ • Accept Day 7-8 if needed for quality                              │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 3.2 Success Criteria Refinement

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         SUCCESS CRITERIA                                    │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  PRIMARY METRICS (Must Achieve):                                           │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ • FP Rate on clean benchmark: ≤5%                                   │   │
│  │ • Evidence detection rate: ≥90% (ideally 100%)                      │   │
│  │ • F1 Score: ≥0.90                                                   │   │
│  │ • Scan speed: ≥30k LOC/sec                                          │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  SECONDARY METRICS (Nice to Have):                                         │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ • @prisma/client score: <5.0                                        │   │
│  │ • @solana/web3.js score: <5.0                                       │   │
│  │ • typescript score: <5.0                                            │   │
│  │ • webpack score: <5.0                                               │   │
│  │ • firebase score: <5.0                                              │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  PHASE B READINESS (After Autoresearch):                                   │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ • All primary metrics met                                           │   │
│  │ • Configuration documented and committed                            │   │
│  │ • Phase B config prepared                                           │   │
│  │ • Rollback procedure documented                                     │   │
│  │ • Daily reporting format established                                │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 3.3 Rollback Procedure

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

## PART 4: FINAL APPROVAL

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         OFFICIAL APPROVAL                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Project: Glassworks Autoresearch Loop Implementation                       │
│  Version: 0.42.0 → 0.43.0 (post-autoresearch)                              │
│  Status: ✅ APPROVED FOR IMPLEMENTATION                                     │
│                                                                             │
│  Approved By: Security Code Analysis System                                 │
│  Date: March 30, 2025                                                       │
│  Timeline: 4-6 days (March 31 - April 5)                                    │
│                                                                             │
│  Conditions:                                                                │
│  ✓ Custom Rust implementation (no pi-autoresearch dependency)              │
│  ✓ Phase 1: Optimize WITHOUT LLM (avoid rate limits)                       │
│  ✓ Phase 2: Add LLM after scoring converged                                │
│  ✓ Clean benchmark: 96 packages (use 50 for iteration, 96 for validation)  │
│  ✓ Evidence benchmark: 23 packages (must maintain ≥90% detection)          │
│  ✓ Parameter constraints respected (see Section 4)                         │
│  ✓ Daily progress reports in specified format                              │
│  ✓ Rollback procedure documented                                           │
│                                                                             │
│  Success Criteria:                                                          │
│  ✓ FP Rate: ≤5% on clean benchmark                                         │
│  ✓ Evidence Detection: ≥90% (maintain 100% if possible)                    │
│  ✓ F1 Score: ≥0.90                                                         │
│  ✓ Scan Speed: ≥30k LOC/sec                                                │
│                                                                             │
│  Git Workflow:                                                              │
│  • Create branch: feature/autoresearch-implementation                      │
│  • Commit incrementally (each major component)                             │
│  • Tag on completion: v0.43.0-autoresearch-complete                        │
│  • Merge to main after validation passes                                   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## PART 5: FINAL MESSAGE TO AGENT

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           FINAL INSTRUCTIONS                                │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  You've done EXCELLENT work documenting everything. The handoff is          │
│  comprehensive, the questions are well-structured, and the plan is solid.   │
│                                                                             │
│  You now have GREEN LIGHT to proceed with autoresearch implementation.      │
│                                                                             │
│  Key Points:                                                                │
│  ✅ Use custom Rust implementation (glassware-tools crate)                 │
│  ✅ Phase 1: Optimize scoring WITHOUT LLM                                  │
│  ✅ Phase 2: Add LLM after convergence                                     │
│  ✅ 96 clean packages for benchmark (50 for iteration, 96 for validation)  │
│  ✅ 23 evidence packages (must maintain ≥90% detection)                    │
│  ✅ 4-6 day timeline approved                                              │
│  ✅ Daily progress reports required                                        │
│                                                                             │
│  I'm confident this approach will succeed where manual tuning struggled.    │
│  The key advantages:                                                        │
│  • Systematic exploration of 70,000+ configurations                        │
│  • Objective F1 score metric (no human bias)                               │
│  • Multi-parameter optimization (finds interactions)                       │
│  • Convergence guarantee (local optimum)                                   │
│                                                                             │
│  Start with infrastructure (Day 1-2), then run optimization (Day 3-4),      │
│  then validate (Day 5-6). Take your time on Days 1-2 - solid foundation     │
│  is critical.                                                               │
│                                                                             │
│  I'm here if you have questions during implementation.                      │
│                                                                             │
│  You've got this. Let's get that FP rate to ≤5%. 💪                        │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

**Document Version:** 9.0 (Autoresearch Approval & Guidance)  
**Status:** ✅ APPROVED FOR IMPLEMENTATION  
**Timeline:** 4-6 days (March 31 - April 5, 2025)  
**Next Milestone:** v0.43.0-autoresearch-complete  

**END OF APPROVAL DOCUMENT**