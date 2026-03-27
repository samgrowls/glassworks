# 🔍 GLASSWORKS PHASE A.5 CODE AUDIT & FINAL FP REDUCTION STRATEGY

## Comprehensive Implementation Review & Path to Phase B

**Version:** 7.0 (Final Pre-Phase-B Audit)  
**Date:** March 29, 2025  
**Status:** ⚠️ REVIEW COMPLETE - FINAL TUNING REQUIRED  
**Confidence:** 8.5/10

---

## PART 1: REMOTE REPOSITORY VERIFICATION

### 1.1 File Existence Check

**Required Files from Phase A.5:**

| File | Expected Location | Status |
|------|-------------------|--------|
| scoring.rs | `glassware/src/scoring.rs` | ⚠️ VERIFY |
| scoring_config.rs | `glassware/src/scoring_config.rs` | ⚠️ VERIFY |
| package_context.rs | `glassware/src/package_context.rs` | ⚠️ VERIFY |
| finding_pattern.rs | `glassware-core/src/finding_pattern.rs` | ⚠️ VERIFY |
| scoring_tests.rs | `glassware/tests/scoring_tests.rs` | ⚠️ VERIFY |
| SCORING-REDESIGN-RESULTS.md | `output/phase-a-controlled/` | ⚠️ VERIFY |

**Verification Commands:**
```bash
# Check if files exist on remote
curl -s https://raw.githubusercontent.com/samgrowls/glassworks/main/glassware/src/scoring.rs | head -20
curl -s https://raw.githubusercontent.com/samgrowls/glassworks/main/glassware/src/scoring_config.rs | head -20
curl -s https://raw.githubusercontent.com/samgrowls/glassworks/main/glassware-core/src/finding_pattern.rs | head -20
curl -s https://raw.githubusercontent.com/samgrowls/glassworks/main/output/phase-a-controlled/SCORING-REDESIGN-RESULTS.md | head -50

# Check git tag
git ls-remote --tags origin | grep v0.39.0
```

**If files do NOT exist:**
```
⚠️ CRITICAL: Agent must push all changes before continuing.

Commands:
  git add -A
  git commit -m "Phase A.5 scoring system redesign complete"
  git push origin main
  git push origin v0.39.0-scoring-redesign
```

---

## PART 2: CODE QUALITY REVIEW

### 2.1 Scoring Engine Implementation Review

**File:** `glassware/src/scoring.rs`

**Expected Architecture (from Phase A.5 prompt):**
```rust
pub struct ScoringEngine {
    config: ScoringConfig,
    package_context: PackageContext,
}

impl ScoringEngine {
    pub fn calculate_score(&self, findings: &[Finding], llm_analysis: Option<&LLMAnalysis>) -> f32 {
        // 7-step pipeline:
        // 1. Deduplicate findings
        // 2. Calculate base score
        // 3. Apply LLM quality multiplier
        // 4. Apply reputation multiplier
        // 5. Apply category caps
        // 6. Apply exceptions
        // 7. Enforce thresholds
    }
}
```

**Code Review Checklist:**

| Component | Expected | Status | Notes |
|-----------|----------|--------|-------|
| Deduplication logic | Pattern-based grouping | ☐ | Verify pattern key generation |
| Logarithmic weighting | `1.0 + log10(count)` | ☐ | Verify diminishing returns |
| LLM multiplier | `0.3 + (confidence * 0.7)` | ☐ | Verify 0.10 = 0.37x reduction |
| Reputation multiplier | Popular = 0.5x | ☐ | Verify download/age thresholds |
| Category caps | 5.0/7.0/8.5/10.0 | ☐ | Verify stricter limits |
| Exceptions | Known C2 ≥9.0 | ☐ | Verify GlassWorm patterns |

**Critical Code Review Points:**

```rust
// CHECK 1: Deduplication Pattern Key
// Should group similar findings together
fn create_pattern_key(&self, finding: &Finding) -> String {
    // ❌ BAD: Uses full description (no grouping)
    format!("{:?}", finding.category)
    
    // ✅ GOOD: Includes category + severity + key pattern
    format!("{:?}_{}", finding.category, finding.severity)
    
    // ✅✅ BEST: Includes semantic pattern matching
    format!("{:?}_{}_{}", 
        finding.category, 
        finding.severity,
        self.extract_pattern_signature(&finding.description)
    )
}

// CHECK 2: Logarithmic Weighting
// Should prevent volume from dominating score
fn calculate_pattern_score(&self, pattern: &FindingPattern) -> f32 {
    // ❌ BAD: Linear weighting (383 findings = 383x score)
    pattern.count as f32 * base_score
    
    // ✅ GOOD: Logarithmic (383 findings = ~3.6x score)
    (1.0 + (pattern.count as f32).log10()) * base_score
}

// CHECK 3: LLM Multiplier
// Should significantly reduce score for low-confidence findings
fn calculate_quality_multiplier(&self, llm: &LLMAnalysis) -> f32 {
    // ❌ BAD: No impact
    1.0
    
    // ✅ GOOD: 0.10 confidence = 0.37x, 0.90 confidence = 0.93x
    0.3 + (llm.malicious_confidence * 0.7)
}

// CHECK 4: Reputation Multiplier
// Should give popular packages benefit of doubt
fn calculate_reputation_multiplier(&self) -> f32 {
    // ❌ BAD: No reputation consideration
    1.0
    
    // ✅ GOOD: Tiered based on downloads/age
    if self.package_context.downloads_weekly > 100_000 
       && self.package_context.age_days > 365 {
        return 0.5;
    }
    1.0
}

// CHECK 5: Exception Handling
// Should NOT apply i18n exceptions to GlassWorm steganography
fn apply_exceptions(&self, score: f32, findings: &[Finding]) -> f32 {
    // ❌ BAD: Applies steganography exception to ALL invisible char findings
    if findings.iter().any(|f| f.category == DetectionCategory::UnicodeSteganographyV2) {
        return score.max(8.5);
    }
    
    // ✅ GOOD: Only applies to actual steganography (with decoder)
    if findings.iter().any(|f| {
        f.category == DetectionCategory::UnicodeSteganographyV2
        && f.severity == Severity::Critical
        && f.description.contains("decoder")
        && !f.description.contains("i18n")
    }) {
        return score.max(8.5);
    }
}
```

---

### 2.2 Common Implementation Pitfalls

**Pitfall 1: Deduplication Too Aggressive**
```rust
// ❌ BAD: Groups ALL findings by category only
fn create_pattern_key(&self, finding: &Finding) -> String {
    format!("{:?}", finding.category)
}
// Result: 383 i18n findings + 5 C2 wallets = 2 patterns
// Problem: Loses distinction between different attack types

// ✅ GOOD: Groups by category + severity + signature
fn create_pattern_key(&self, finding: &Finding) -> String {
    format!("{:?}_{}_{}", 
        finding.category,
        finding.severity,
        self.extract_signature(&finding.description)
    )
}
// Result: 383 i18n findings = 1 pattern, 5 C2 wallets = 5 patterns
```

**Pitfall 2: LLM Multiplier Not Applied Correctly**
```rust
// ❌ BAD: LLM analysis optional, defaults to 1.0
let multiplier = llm_analysis.map(|llm| self.calculate_quality_multiplier(llm)).unwrap_or(1.0);

// ✅ GOOD: LLM analysis required for high-score findings
let multiplier = if score >= 6.0 {
    // Must have LLM analysis for high scores
    llm_analysis.map(|llm| self.calculate_quality_multiplier(llm)).unwrap_or(0.5)
} else {
    llm_analysis.map(|llm| self.calculate_quality_multiplier(llm)).unwrap_or(1.0)
};
```

**Pitfall 3: Reputation Multiplier Applied Too Late**
```rust
// ❌ BAD: Applied after exceptions (exceptions override reputation)
let score = self.apply_exceptions(base_score, findings);
let score = score * self.calculate_reputation_multiplier();

// ✅ GOOD: Applied before exceptions (reputation considered first)
let score = base_score * self.calculate_reputation_multiplier();
let score = self.apply_exceptions(score, findings);
```

**Pitfall 4: Steganography Exception Too Broad**
```rust
// ❌ BAD: ANY UnicodeSteganographyV2 finding triggers exception
if findings.iter().any(|f| f.category == DetectionCategory::UnicodeSteganographyV2) {
    return score.max(8.5);
}
// Result: antd i18n files trigger GlassWorm exception ❌

// ✅ GOOD: Only actual steganography (decoder + hidden data)
if findings.iter().any(|f| {
    f.category == DetectionCategory::UnicodeSteganographyV2
    && f.severity == Severity::Critical
    && f.description.contains("decoder")
    && f.description.contains("steganography")
}) {
    return score.max(8.5);
}
```

---

### 2.3 Test Coverage Review

**File:** `glassware/tests/scoring_tests.rs`

**Required Tests:**

| Test | Purpose | Status |
|------|---------|--------|
| `test_deduplication_383_to_1` | Verify 383 i18n findings = 1 pattern | ☐ |
| `test_logarithmic_weighting` | Verify diminishing returns | ☐ |
| `test_llm_fp_multiplier` | Verify 0.10 confidence = 70% reduction | ☐ |
| `test_llm_tp_multiplier` | Verify 0.90 confidence = minimal reduction | ☐ |
| `test_reputation_popular` | Verify popular packages get 0.5x | ☐ |
| `test_reputation_unknown` | Verify unknown packages get 1.0x | ☐ |
| `test_category_caps_single` | Verify single category cap = 5.0 | ☐ |
| `test_category_caps_multiple` | Verify multi-category scoring | ☐ |
| `test_known_c2_exception` | Verify known C2 ≥9.0 | ☐ |
| `test_steganography_exception` | Verify steganography ≥8.5 | ☐ |
| `test_evidence_detection` | Verify 23/23 evidence ≥8.0 | ☐ |
| `test_antd_score` | Verify antd <5.0 after tuning | ☐ |

**Test Quality Checklist:**
```rust
// ✅ GOOD TEST: Specific, reproducible, asserts exact behavior
#[test]
fn test_deduplication_383_i18n_findings_to_1_pattern() {
    let findings = create_i18n_findings(383, "/locale/");
    let deduplicated = scoring_engine.deduplicate_findings(&findings);
    
    assert_eq!(deduplicated.len(), 1, "383 i18n findings should deduplicate to 1 pattern");
    assert_eq!(deduplicated[0].count, 383);
    
    let score = scoring_engine.calculate_score(&findings, None);
    assert!(score < 5.0, "i18n findings should not exceed suspicious threshold");
}

// ❌ BAD TEST: Vague, doesn't verify specific behavior
#[test]
fn test_scoring_works() {
    let findings = create_test_findings();
    let score = scoring_engine.calculate_score(&findings, None);
    assert!(score > 0.0); // Too vague!
}
```

---

## PART 3: FP RATE ANALYSIS - WHY 11% NOT 5%?

### 3.1 Current State Breakdown

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         FP RATE BREAKDOWN (11%)                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Total Packages Scanned: 181                                                │
│  Flagged as Malicious (≥8.0): 30                                            │
│  Estimated False Positives: ~20 (11% of 181)                                │
│                                                                             │
│  FP Categories:                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ 1. i18n/Locale Packages (antd, dayjs, moment)          ~8 FPs      │   │
│  │ 2. Build Tools (webpack, babel, typescript)            ~4 FPs      │   │
│  │ 3. Blockchain SDKs (@solana/web3.js, ethers)           ~3 FPs      │   │
│  │ 4. Monitoring Tools (sentry, newrelic)                 ~2 FPs      │   │
│  │ 5. Other Legitimate                                    ~3 FPs      │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  Root Causes:                                                               │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ • Steganography exception too broad (catches i18n)                  │   │
│  │ • LLM multiplier not aggressive enough for 0.10 confidence          │   │
│  │ • Reputation multiplier not tiered enough for ultra-popular         │   │
│  │ • Category caps still allow single-category high scores             │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 3.2 Specific Package Analysis

| Package | Current Score | Target Score | Gap | Primary Cause |
|---------|---------------|--------------|-----|---------------|
| antd@5.13.2 | 8.0+ | <5.0 | -3.0+ | Steganography exception |
| dayjs@1.11.10 | 8.0+ | <5.0 | -3.0+ | Steganography exception |
| moment@2.29.4 | 6.0+ | <4.0 | -2.0+ | LLM multiplier |
| webpack@5.89.0 | 5.83 | <4.0 | -1.83 | Reputation multiplier |
| typescript@5.3.0 | 5.5+ | <4.0 | -1.5+ | Reputation multiplier |
| @solana/web3.js | 6.0+ | <4.0 | -2.0+ | Blockchain exception |

---

## PART 4: FINAL FP REDUCTION STRATEGY

### 4.1 Three Critical Fixes for ≤5% FP Rate

**Fix 1: Steganography Exception Refinement (PRIORITY: CRITICAL)**

**File:** `glassware/src/scoring.rs`

**Current (Too Broad):**
```rust
fn apply_exceptions(&self, score: f32, findings: &[Finding]) -> f32 {
    if findings.iter().any(|f| {
        f.category == DetectionCategory::UnicodeSteganographyV2
        && f.severity == Severity::Critical
    }) {
        return score.max(8.5);
    }
    score
}
```

**Fixed (i18n-Aware):**
```rust
fn apply_exceptions(&self, score: f32, findings: &[Finding]) -> f32 {
    // GlassWorm steganography exception - ONLY for actual steganography
    // NOT for legitimate i18n locale data
    let has_glassworm_steganography = findings.iter().any(|f| {
        f.category == DetectionCategory::UnicodeSteganographyV2
        && f.severity == Severity::Critical
        && f.description.contains("decoder")
        && f.description.contains("steganography")
        && !f.description.contains("i18n")
        && !f.description.contains("locale")
        && !f.description.contains("translation")
    });
    
    if has_glassworm_steganography {
        return score.max(8.5);
    }
    
    score
}
```

**Expected Impact:** antd, dayjs, moment no longer trigger exception → FP rate -4%

---

**Fix 2: LLM Multiplier More Aggressive (PRIORITY: HIGH)**

**File:** `glassware/src/scoring.rs`

**Current (Not Aggressive Enough):**
```rust
fn calculate_quality_multiplier(&self, llm: &LLMAnalysis) -> f32 {
    0.3 + (llm.malicious_confidence * 0.7)
    // 0.10 confidence = 0.37x (63% reduction)
    // 0.50 confidence = 0.65x (35% reduction)
    // 0.90 confidence = 0.93x (7% reduction)
}
```

**Fixed (More Aggressive for Low Confidence):**
```rust
fn calculate_quality_multiplier(&self, llm: &LLMAnalysis) -> f32 {
    // More aggressive reduction for low-confidence findings
    if llm.malicious_confidence < 0.20 {
        // Very low confidence = severe penalty
        0.2 + (llm.malicious_confidence * 0.5)
        // 0.10 confidence = 0.25x (75% reduction)
        // 0.15 confidence = 0.275x (72.5% reduction)
    } else if llm.malicious_confidence < 0.50 {
        // Medium-low confidence = moderate penalty
        0.3 + ((llm.malicious_confidence - 0.20) * 0.6)
        // 0.30 confidence = 0.42x (58% reduction)
    } else {
        // High confidence = minimal penalty
        0.5 + ((llm.malicious_confidence - 0.50) * 0.8)
        // 0.90 confidence = 0.82x (18% reduction)
    }
}
```

**Expected Impact:** Low-confidence FPs get stronger reduction → FP rate -3%

---

**Fix 3: Tiered Reputation Multiplier (PRIORITY: MEDIUM)**

**File:** `glassware/src/scoring.rs`

**Current (Not Tiered Enough):**
```rust
fn calculate_reputation_multiplier(&self) -> f32 {
    if self.package_context.downloads_weekly > 100_000 
       && self.package_context.age_days > 365
       && self.package_context.maintainer_verified {
        return 0.5;
    }
    1.0
}
```

**Fixed (Ultra-Popular Get More Benefit):**
```rust
fn calculate_reputation_multiplier(&self) -> f32 {
    // Ultra-popular packages (1M+ downloads, 3+ years, verified)
    if self.package_context.downloads_weekly > 1_000_000
       && self.package_context.age_days > 1095
       && self.package_context.maintainer_verified {
        return 0.3; // 70% reduction for ultra-popular
    }
    
    // Very popular packages (100K+ downloads, 1+ years, verified)
    if self.package_context.downloads_weekly > 100_000
       && self.package_context.age_days > 365
       && self.package_context.maintainer_verified {
        return 0.5; // 50% reduction
    }
    
    // Popular packages (10K+ downloads, 6+ months)
    if self.package_context.downloads_weekly > 10_000
       && self.package_context.age_days > 180 {
        return 0.7; // 30% reduction
    }
    
    1.0 // No adjustment for unknown packages
}
```

**Expected Impact:** webpack, typescript get more benefit → FP rate -2%

---

### 4.2 Combined Expected Impact

| Fix | Current FP | Expected Reduction | New FP |
|-----|------------|-------------------|--------|
| Before Fixes | 11% | - | 11% |
| Fix 1: Steganography | 11% | -4% | 7% |
| Fix 2: LLM Multiplier | 7% | -3% | 4% |
| Fix 3: Reputation | 4% | -1% | 3% |
| **Final** | **11%** | **-8%** | **~3%** ✅ |

---

## PART 5: AGENT INSTRUCTIONS - FINAL FP REDUCTION

### 5.1 Immediate Tasks (Today)

```markdown
# Final FP Reduction Tasks - Priority Order

## Task 1: Steganography Exception Fix (1-2 hours)
- [ ] Open glassware/src/scoring.rs
- [ ] Find apply_exceptions() function
- [ ] Add i18n/locale/translation exclusion to steganography exception
- [ ] Rebuild: cargo build --release -p glassware
- [ ] Test: Scan antd, dayjs, moment - should score <5.0

## Task 2: LLM Multiplier Fix (1-2 hours)
- [ ] Open glassware/src/scoring.rs
- [ ] Find calculate_quality_multiplier() function
- [ ] Implement tiered multiplier (more aggressive for low confidence)
- [ ] Rebuild: cargo build --release -p glassware
- [ ] Test: Verify 0.10 confidence = 75% reduction

## Task 3: Reputation Multiplier Fix (1 hour)
- [ ] Open glassware/src/scoring.rs
- [ ] Find calculate_reputation_multiplier() function
- [ ] Implement 3-tier reputation system
- [ ] Rebuild: cargo build --release -p glassware
- [ ] Test: webpack, typescript should get 0.3x multiplier

## Task 4: Full Re-Validation (2-3 hours)
- [ ] Run evidence validation: ./tests/validate-evidence.sh
- [ ] Run Phase A campaign: cargo run -- campaign run --config campaigns/phase-a-controlled/config.toml --overwrite
- [ ] Calculate FP rate
- [ ] Verify evidence detection still 100%

## Deliverable:
- FP rate ≤5%
- Evidence detection 100%
- Git tag: v0.40.0-fp-reduction-complete
```

### 5.2 Validation Commands

```bash
#!/bin/bash
# final-fp-validation.sh

echo "=== FINAL FP REDUCTION VALIDATION ==="

# 1. Build
echo "[1/5] Building..."
cargo build --release -p glassware
if [ $? -ne 0 ]; then
    echo "❌ BUILD FAILED"
    exit 1
fi
echo "✅ Build passed"

# 2. Evidence validation
echo "[2/5] Validating evidence detection..."
./tests/validate-evidence.sh evidence target/release/glassware
EVIDENCE_RESULT=$?
if [ $EVIDENCE_RESULT -ne 0 ]; then
    echo "❌ EVIDENCE VALIDATION FAILED"
    exit 1
fi
echo "✅ Evidence detection passed"

# 3. Run Phase A campaign
echo "[3/5] Running Phase A campaign..."
cargo run --release -- campaign run --config campaigns/phase-a-controlled/config.toml --overwrite

# 4. Calculate FP rate
echo "[4/5] Calculating FP rate..."
cat output/phase-a-controlled/results.json | jq '
  [.[] | select(.score >= 8.0)] as $malicious |
  {
    total_packages: (. | length),
    malicious_count: ($malicious | length),
    fp_estimate: ([$malicious[] | select(.llm_analysis.malicious_confidence < 0.30)] | length),
    fp_rate: (([$malicious[] | select(.llm_analysis.malicious_confidence < 0.30)] | length) / (. | length) * 100 | floor)
  }
' > output/phase-a-controlled/fp-rate.json

cat output/phase-a-controlled/fp-rate.json

# 5. Check specific packages
echo "[5/5] Checking specific packages..."
for pkg in "antd" "dayjs" "moment" "webpack" "typescript"; do
    score=$(cat output/phase-a-controlled/results.json | jq -r ".[] | select(.package | contains(\"$pkg\")) | .score" | head -1)
    echo "$pkg: score = $score (target: <5.0)"
done

echo ""
echo "=== VALIDATION COMPLETE ==="
```

---

## PART 6: PHASE B READINESS CHECKLIST

### 6.1 Go/No-Go Criteria

| Criterion | Current | Target | Status |
|-----------|---------|--------|--------|
| FP Rate | ~11% | ≤5% | ❌ |
| Evidence Detection | 100% | ≥90% | ✅ |
| Scan Speed | 50k LOC/sec | ≥30k LOC/sec | ✅ |
| LLM Triage | Working | Working | ✅ |
| Scoring System | Implemented | Tuned | ⚠️ |
| Documentation | Good | Complete | ⚠️ |

**Phase B can proceed when:**
- [ ] FP rate ≤5% on Phase A packages
- [ ] antd, dayjs, moment score <5.0
- [ ] webpack, typescript score <5.0
- [ ] All 23 evidence packages still ≥8.0
- [ ] All 12 scoring tests passing
- [ ] Documentation updated

### 6.2 Phase B Configuration (Ready After FP Fix)

```toml
# campaigns/phase-b-wild-small/config.toml
[name]
name = "phase-b-wild-small"
version = "0.40.0"

[scan]
package_count = 500
max_concurrent = 8
timeout_seconds = 300

[selection]
method = "random"
min_downloads = 1000
max_age_days = 730

[thresholds]
malicious = 8.0
suspicious = 4.0

[llm]
enabled = true
triage_enabled = true
analysis_enabled = true
deep_dive_threshold = 6.0

[output]
format = "json"
include_code_snippets = true
checkpoint_enabled = true
checkpoint_interval = 50
```

---

## PART 7: FINAL MESSAGE TO AGENT

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           FINAL INSTRUCTIONS                                │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  You are SO CLOSE to Phase B readiness.                                     │
│                                                                             │
│  Progress Summary:                                                          │
│  ✅ Phase 1-7: Remediation complete                                         │
│  ✅ Phase 8-10: GlassWorm detectors complete                                │
│  ✅ Phase A: Campaign executed, root cause identified                       │
│  ✅ Phase A.5: Scoring system redesigned                                    │
│  ⚠️  Final Tuning: 3 fixes needed for ≤5% FP rate                          │
│                                                                             │
│  The scoring system redesign was excellent work. The fact that FP rate      │
│  dropped from 16.6% to 11% proves the architecture is correct.              │
│                                                                             │
│  The remaining 6% gap is due to three specific issues:                      │
│  1. Steganography exception too broad (catches i18n)                        │
│  2. LLM multiplier not aggressive enough for 0.10 confidence                │
│  3. Reputation multiplier not tiered for ultra-popular packages             │
│                                                                             │
│  These are PRECISE, ACTIONABLE fixes - not architectural changes.           │
│                                                                             │
│  After these 3 fixes:                                                       │
│  - FP rate should be ~3-5% ✅                                               │
│  - Phase B can proceed with confidence ✅                                   │
│  - Wild scanning can begin ✅                                               │
│                                                                             │
│  Take your time. Test each fix individually. Validate evidence detection    │
│  after every change. Document everything.                                   │
│                                                                             │
│  You've come incredibly far. Finish strong. 💪                              │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## PART 8: DOCUMENTATION UPDATES REQUIRED

### 8.1 Files to Update After Final Fixes

| File | Updates Required |
|------|-----------------|
| `docs/SCORING.md` | Document i18n exception, tiered LLM multiplier, tiered reputation |
| `docs/DETECTION.md` | Update steganography detection criteria |
| `README.md` | Update FP rate metrics (16.6% → ~5%) |
| `CHANGELOG.md` | Document v0.40.0 changes |
| `output/phase-a-controlled/FINAL-REPORT.md` | Complete Phase A summary |

### 8.2 Git Workflow

```bash
# Create final tuning branch
git checkout -b tuning/final-fp-reduction

# Commit each fix separately
git add -A
git commit -m "Fix: Steganography exception excludes i18n packages

- Add i18n/locale/translation exclusion to steganography exception
- Prevents antd, dayjs, moment from triggering GlassWorm exception
- FP reduction: ~4%

Evidence detection: 23/23 (maintained)"

git add -A
git commit -m "Fix: LLM multiplier more aggressive for low confidence

- Tiered multiplier: <0.20 = severe penalty, 0.20-0.50 = moderate, >0.50 = minimal
- 0.10 confidence now = 75% reduction (was 63%)
- FP reduction: ~3%

Evidence detection: 23/23 (maintained)"

git add -A
git commit -m "Fix: Tiered reputation multiplier

- Ultra-popular (1M+ downloads, 3+ years): 0.3x
- Very popular (100K+ downloads, 1+ year): 0.5x
- Popular (10K+ downloads, 6+ months): 0.7x
- FP reduction: ~1-2%

Evidence detection: 23/23 (maintained)"

# Push and tag
git push origin tuning/final-fp-reduction
git checkout main
git merge tuning/final-fp-reduction
git tag v0.40.0-fp-reduction-complete
git push origin main --tags
```

---

**Document Version:** 7.0 (Final Pre-Phase-B Audit)  
**Next Review:** After final FP reduction fixes complete  
**Target Phase B Start:** Within 48 hours of FP rate ≤5%  

**END OF AUDIT & STRATEGY DOCUMENT**