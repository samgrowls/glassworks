# 🔍 GLASSWORKS POST-TUNING AUDIT & SCORING SYSTEM REDESIGN

## Comprehensive Analysis & Phase B Preparation

**Version:** 6.0 (Scoring System Redesign)  
**Date:** March 28, 2025  
**Status:** ⚠️ CRITICAL REDESIGN REQUIRED  
**Priority:** BLOCKER FOR PHASE B

---

## PART 1: REMOTE REPOSITORY CHECK

### 1.1 Verification Required

**Before proceeding, confirm:**

```bash
# Check if TUNING-RESULTS-REPORT.md exists on remote
curl -s https://raw.githubusercontent.com/samgrowls/glassworks/main/output/phase-a-controlled/TUNING-RESULTS-REPORT.md | head -50

# Or check git tags
git ls-remote --tags origin | grep v0.38.0

# Or check recent commits
git ls-remote origin main
```

**If file does NOT exist on remote:**
```
⚠️ ACTION REQUIRED: Agent must push changes to remote before continuing.

Commands for agent:
  git add -A
  git commit -m "Phase A tuning complete - detector fixes implemented"
  git push origin main
  git push origin v0.38.0-phase-a-tuned
```

---

## PART 2: BIRD'S EYE VIEW - THE FUNDAMENTAL PROBLEM

### 2.1 What We Learned from Phase A

| Observation | Implication |
|-------------|-------------|
| Detector tuning didn't reduce FP rate | **Scoring system is the bottleneck** |
| 383 i18n findings = score 7.00 | **Volume drives score, not quality** |
| LLM correctly identifies FPs (0.10 confidence) | **LLM feedback not used in scoring** |
| Evidence detection still 100% | **Detectors work, scoring doesn't discriminate** |

### 2.2 The Core Architectural Flaw

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    CURRENT SCORING SYSTEM (BROKEN)                          │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐    ┌──────────┐  │
│  │  Detector    │───▶│   Finding    │───▶│    Score     │───▶│ Decision │  │
│  │  Finds 383   │    │   Created    │    │   Summed     │    │  ≥7.0 =  │  │
│  │  i18n chars  │    │   (all equal │    │   (no quality│    │ MALICIOUS│  │
│  │              │    │    weight)   │    │   filter)    │    │          │  │
│  └──────────────┘    └──────────────┘    └──────────────┘    └──────────┘  │
│         │                   │                  │                  │         │
│         │                   │                  │                  │         │
│         ▼                   ▼                  ▼                  ▼         │
│  ❌ No deduplication   ❌ All findings    ❌ LLM feedback   ❌ 16.6% FP    │
│     (383 = 383)          treated equal       ignored           rate        │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│                    PROPOSED SCORING SYSTEM (FIXED)                          │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐    ┌──────────┐  │
│  │  Detector    │───▶│   Finding    │───▶│   Pattern    │───▶│ Decision │  │
│  │  Finds 383   │    │   Created    │    │   Clustering │    │  ≥8.0 =  │  │
│  │  i18n chars  │    │   + Quality  │    │   + LLM      │    │ MALICIOUS│  │
│  │              │    │   Score      │    │   Override   │    │          │  │
│  └──────────────┘    └──────────────┘    └──────────────┘    └──────────┘  │
│         │                   │                  │                  │         │
│         │                   │                  │                  │         │
│         ▼                   ▼                  ▼                  ▼         │
│  ✅ Deduplication      ✅ Quality         ✅ LLM feedback   ✅ ≤5% FP     │
│     (383 similar =        weighting          used in score     rate        │
│      1 pattern)          applied             calculation                   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## PART 3: ROOT CAUSE ANALYSIS

### 3.1 Why Detector Tuning Didn't Work

**The detector changes were CORRECT but INSUFFICIENT:**

```rust
// Detector now returns fewer findings for i18n files ✅
if is_i18n_file && !has_decoder {
    return findings; // Skip
}

// BUT: If file has BOTH i18n AND decoder pattern:
if is_i18n_file && has_decoder {
    findings.push(CRITICAL); // Still flagged
}

// AND: 383 findings still sum to score 7.0+ ❌
for finding in findings {
    score += 3.0 * 0.90; // 383 * 2.7 = 1034.1 → capped at 10.0
}
```

**The problem:** Even with context-aware detectors, the scoring system:
1. **Doesn't deduplicate** - 383 similar findings count as 383
2. **Doesn't weight by quality** - LLM 0.10 confidence finding = same as 0.90
3. **Doesn't consider package context** - antd (5M downloads, 5 years old) treated same as unknown package
4. **Doesn't use attack chain logic** - Single category with 383 findings = same as multi-category attack

### 3.2 The antd Example - Deep Dive

```
Package: antd@5.13.2
Downloads: 5M+ weekly
Age: 5+ years
Maintainer: Ant Design Team (verified)

Findings:
- 383 InvisibleChar findings (all in /locale/ files)
- All findings: Severity Low-Medium, Confidence 0.40-0.50
- LLM Analysis: confidence 0.10 (FALSE POSITIVE)
- Category: 1 (InvisibleChar only)

Current Scoring:
- Base: 383 × 1.0 × 0.45 = 172.35
- Category cap (1 cat): min(172.35, 4.0) = 4.0
- BUT: Some findings marked Critical due to decoder pattern
- Critical findings: 383 × 3.0 × 0.90 = 1034.1
- Final score: 7.00 (at threshold)

Problem:
- 383 findings should count as 1 pattern (i18n locale data)
- LLM confidence 0.10 should reduce score
- Package reputation should reduce score
- Single category should cap lower

Expected Score: 2.0-3.0 (Suspicious, not Malicious)
Actual Score: 7.0 (Malicious) ❌
```

---

## PART 4: SCORING SYSTEM REDESIGN

### 4.1 New Scoring Architecture

```rust
// glassware/src/scoring.rs (NEW FILE)

pub struct ScoringEngine {
    config: ScoringConfig,
    package_context: PackageContext,
}

pub struct ScoringConfig {
    // Thresholds
    pub malicious_threshold: f32,      // 8.0 (raised from 7.0)
    pub suspicious_threshold: f32,     // 4.0
    
    // Weights
    pub finding_base_weights: HashMap<Severity, f32>,
    pub quality_multiplier: f32,       // LLM confidence impact
    pub reputation_multiplier: f32,    // Package reputation impact
    
    // Deduplication
    pub pattern_dedup_enabled: bool,
    pub pattern_similarity_threshold: f32,
    
    // Exceptions
    pub known_c2_min_score: f32,       // 9.0
    pub steganography_min_score: f32,  // 8.5
}

pub struct PackageContext {
    pub name: String,
    pub version: String,
    pub downloads_weekly: u64,
    pub age_days: u64,
    pub maintainer_verified: bool,
    pub known_legitimate: bool,  // From allowlist (not whitelist!)
}

impl ScoringEngine {
    pub fn calculate_score(&self, findings: &[Finding], llm_analysis: Option<&LLMAnalysis>) -> f32 {
        // STEP 1: Deduplicate findings by pattern
        let deduplicated = self.deduplicate_findings(findings);
        
        // STEP 2: Calculate base score from deduplicated findings
        let mut base_score = 0.0;
        for pattern in &deduplicated {
            base_score += self.calculate_pattern_score(pattern);
        }
        
        // STEP 3: Apply quality multiplier (LLM feedback)
        if let Some(llm) = llm_analysis {
            base_score *= self.calculate_quality_multiplier(llm);
        }
        
        // STEP 4: Apply reputation multiplier
        base_score *= self.calculate_reputation_multiplier();
        
        // STEP 5: Apply category diversity caps
        base_score = self.apply_category_caps(base_score, &deduplicated);
        
        // STEP 6: Apply exceptions (known C2, etc.)
        base_score = self.apply_exceptions(base_score, findings);
        
        // STEP 7: Enforce thresholds
        base_score.min(10.0).max(0.0)
    }
    
    fn deduplicate_findings(&self, findings: &[Finding]) -> Vec<FindingPattern> {
        // Group findings by pattern similarity
        // 383 i18n findings → 1 pattern
        // 5 different C2 wallets → 5 patterns
        let mut patterns: HashMap<String, FindingPattern> = HashMap::new();
        
        for finding in findings {
            let pattern_key = self.create_pattern_key(finding);
            
            patterns.entry(pattern_key)
                .and_modify(|p| {
                    p.count += 1;
                    p.max_severity = p.max_severity.max(finding.severity);
                    p.avg_confidence = (p.avg_confidence + finding.confidence) / 2.0;
                })
                .or_insert(FindingPattern {
                    category: finding.category.clone(),
                    severity: finding.severity,
                    max_severity: finding.severity,
                    count: 1,
                    avg_confidence: finding.confidence,
                    description: finding.description.clone(),
                });
        }
        
        patterns.into_values().collect()
    }
    
    fn calculate_pattern_score(&self, pattern: &FindingPattern) -> f32 {
        // Diminishing returns for count
        // 1 finding = full weight
        // 10 findings = 2x weight (not 10x)
        // 100 findings = 3x weight (not 100x)
        let count_multiplier = 1.0 + (pattern.count as f32).log10();
        
        let base = match pattern.max_severity {
            Severity::Critical => 3.0,
            Severity::High => 2.0,
            Severity::Medium => 1.0,
            Severity::Low => 0.5,
        };
        
        base * count_multiplier * pattern.avg_confidence
    }
    
    fn calculate_quality_multiplier(&self, llm: &LLMAnalysis) -> f32 {
        // LLM confidence 0.10 → multiplier 0.3 (reduce score)
        // LLM confidence 0.50 → multiplier 0.7
        // LLM confidence 0.90 → multiplier 1.0 (no change)
        0.3 + (llm.malicious_confidence * 0.7)
    }
    
    fn calculate_reputation_multiplier(&self) -> f32 {
        // High downloads + old + verified = lower multiplier
        if self.package_context.downloads_weekly > 100_000 
           && self.package_context.age_days > 365
           && self.package_context.maintainer_verified {
            return 0.5; // Reduce score by 50% for reputable packages
        }
        
        if self.package_context.downloads_weekly > 10_000
           && self.package_context.age_days > 180 {
            return 0.7;
        }
        
        1.0 // No adjustment for unknown packages
    }
    
    fn apply_category_caps(&self, score: f32, patterns: &[FindingPattern]) -> f32 {
        let categories: HashSet<_> = patterns.iter().map(|p| &p.category).collect();
        let category_count = categories.len();
        
        // Stricter caps than before
        match category_count {
            0 => 0.0,
            1 => score.min(5.0),      // Was 4.0
            2 => score.min(7.0),      // Same
            3 => score.min(8.5),      // Was no cap
            _ => score.min(10.0),
        }
    }
    
    fn apply_exceptions(&self, score: f32, findings: &[Finding]) -> f32 {
        // Known C2 always scores high
        if findings.iter().any(|f| {
            f.category == DetectionCategory::BlockchainC2 
            && f.severity == Severity::Critical
            && f.description.contains("Known C2")
        }) {
            return score.max(9.0);
        }
        
        // GlassWorm C2 polling
        if findings.iter().any(|f| {
            f.category == DetectionCategory::BlockchainPolling
            && f.severity == Severity::Critical
        }) {
            return score.max(9.0);
        }
        
        // Steganography with decoder
        if findings.iter().any(|f| {
            f.category == DetectionCategory::UnicodeSteganographyV2
            && f.severity == Severity::Critical
            && f.description.contains("decoder")
        }) {
            return score.max(8.5);
        }
        
        score
    }
}
```

### 4.2 Key Changes Summary

| Change | Before | After | Impact |
|--------|--------|-------|--------|
| **Threshold** | 7.0 | 8.0 | Reduces borderline FPs |
| **Deduplication** | None | Pattern-based | 383 findings → 1 pattern |
| **LLM Integration** | Override only | Score multiplier | LLM 0.10 → 70% score reduction |
| **Reputation** | None | Multiplier | Popular packages get benefit of doubt |
| **Category Caps** | 4.0/7.0/10.0 | 5.0/7.0/8.5/10.0 | Stricter single-category limits |
| **Count Weighting** | Linear | Logarithmic | Diminishing returns on volume |

### 4.3 Expected Impact on Phase A Results

| Package | Before Score | After Score | Status |
|---------|--------------|-------------|--------|
| antd@5.13.2 | 7.00 | ~3.5 | Suspicious (not Malicious) ✅ |
| moment@2.29.4 | 7.00 | ~2.0 | Low risk ✅ |
| @solana/web3.js | 7.00 | ~4.0 | Suspicious ✅ |
| Evidence packages | 8.0-10.0 | 8.0-10.0 | Still Malicious ✅ |
| express@4.19.2 | 7.00 | ~3.0 | Low risk ✅ |

**Expected FP Rate:** 16.6% → ~4-5% ✅

---

## PART 5: IMPLEMENTATION PLAN

### 5.1 Phase A.5: Scoring System Redesign

**Estimated Time:** 1-2 days

**Files to Create:**
```
glassware/src/
├── scoring.rs          # NEW - Scoring engine
├── scoring_config.rs   # NEW - Configuration
└── package_context.rs  # NEW - Package reputation data

glassware-core/src/
└── finding_pattern.rs  # NEW - Deduplicated pattern struct
```

**Files to Modify:**
```
glassware/src/
├── scanner.rs          # Integrate new scoring engine
├── llm.rs              # Expose confidence for scoring

glassware-core/src/
├── detection_result.rs # Add pattern deduplication
└── detector.rs         # Add pattern key generation
```

**Files to Update:**
```
docs/
├── SCORING.md          # Complete rewrite
└── DETECTION.md        # Update scoring section

config/
└── scoring.toml        # NEW - Scoring configuration
```

### 5.2 Testing Requirements

```rust
#[cfg(test)]
mod scoring_tests {
    #[test]
    fn test_deduplication_383_findings_to_1_pattern() {
        // 383 i18n findings should score as 1 pattern
        let findings = create_i18n_findings(383);
        let score = scoring_engine.calculate_score(&findings, None);
        assert!(score < 5.0, "i18n findings should not exceed suspicious threshold");
    }
    
    #[test]
    fn test_llm_confidence_impact() {
        let findings = create_test_findings();
        
        let llm_fp = LLMAnalysis { malicious_confidence: 0.10 };
        let llm_tp = LLMAnalysis { malicious_confidence: 0.90 };
        
        let score_fp = scoring_engine.calculate_score(&findings, Some(&llm_fp));
        let score_tp = scoring_engine.calculate_score(&findings, Some(&llm_tp));
        
        assert!(score_fp < score_tp, "LLM FP should reduce score");
        assert!(score_fp < score_tp * 0.5, "LLM FP should reduce score by at least 50%");
    }
    
    #[test]
    fn test_reputation_multiplier() {
        let popular_package = PackageContext {
            downloads_weekly: 1_000_000,
            age_days: 1825,
            maintainer_verified: true,
            ..
        };
        
        let unknown_package = PackageContext {
            downloads_weekly: 100,
            age_days: 30,
            maintainer_verified: false,
            ..
        };
        
        let score_popular = scoring_engine.with_context(popular_package).calculate_score(&findings, None);
        let score_unknown = scoring_engine.with_context(unknown_package).calculate_score(&findings, None);
        
        assert!(score_popular < score_unknown, "Popular packages should get reputation benefit");
    }
    
    #[test]
    fn test_evidence_detection_maintained() {
        // All 23 evidence packages should still score ≥8.0
        for evidence in load_evidence_packages() {
            let result = scanner.scan(&evidence);
            assert!(result.score >= 8.0, "Evidence package {} scored too low: {}", evidence.name, result.score);
        }
    }
}
```

### 5.3 Validation Commands

```bash
# 1. Build new scoring system
cargo build --release -p glassware

# 2. Run scoring tests
cargo test --release scoring_tests

# 3. Validate evidence detection
./tests/validate-evidence.sh evidence target/release/glassware

# 4. Re-run Phase A campaign
cargo run --release -- campaign run --config campaigns/phase-a-controlled/config.toml --overwrite

# 5. Calculate new FP rate
cat output/phase-a-controlled/results.json | jq '
  [.[] | select(.score >= 8.0)] as $malicious |
  {
    total_packages: (. | length),
    malicious_count: ($malicious | length),
    fp_estimate: ([$malicious[] | select(.llm_analysis.malicious_confidence < 0.30)] | length),
    fp_rate: (([$malicious[] | select(.llm_analysis.malicious_confidence < 0.30)] | length) / ($malicious | length) * 100 | floor)
  }
'
```

---

## PART 6: AGENT PROMPT - SCORING SYSTEM REDESIGN

```markdown
# GLASSWORKS PHASE A.5: SCORING SYSTEM REDESIGN

## Mission

The detector tuning (Phase A) did not reduce FP rate because the **scoring system is the bottleneck**. 

Your task is to completely redesign the scoring system to:
1. Deduplicate findings by pattern (383 similar findings = 1 pattern)
2. Weight findings by quality (LLM confidence impact)
3. Apply reputation multipliers (popular packages get benefit of doubt)
4. Raise malicious threshold from 7.0 to 8.0
5. Maintain 100% evidence detection

## Current State

- Detector tuning: ✅ Complete (4 detectors updated)
- Evidence detection: ✅ 100% (23/23)
- FP rate: ❌ 16.6% (target: ≤5%)
- Root cause: Scoring system doesn't discriminate quality vs. quantity

## Objectives

### 1. Create New Scoring Engine (glassware/src/scoring.rs)

Implement the architecture from Part 4.1 of the audit document:

```rust
pub struct ScoringEngine {
    config: ScoringConfig,
    package_context: PackageContext,
}

impl ScoringEngine {
    pub fn calculate_score(&self, findings: &[Finding], llm_analysis: Option<&LLMAnalysis>) -> f32;
    fn deduplicate_findings(&self, findings: &[Finding]) -> Vec<FindingPattern>;
    fn calculate_pattern_score(&self, pattern: &FindingPattern) -> f32;
    fn calculate_quality_multiplier(&self, llm: &LLMAnalysis) -> f32;
    fn calculate_reputation_multiplier(&self) -> f32;
    fn apply_category_caps(&self, score: f32, patterns: &[FindingPattern]) -> f32;
    fn apply_exceptions(&self, score: f32, findings: &[Finding]) -> f32;
}
```

### 2. Create Supporting Types

**glassware/src/scoring_config.rs:**
```rust
pub struct ScoringConfig {
    pub malicious_threshold: f32,      // 8.0
    pub suspicious_threshold: f32,     // 4.0
    pub finding_base_weights: HashMap<Severity, f32>,
    pub quality_multiplier: f32,
    pub reputation_multiplier: f32,
    pub pattern_dedup_enabled: bool,
    pub pattern_similarity_threshold: f32,
    pub known_c2_min_score: f32,       // 9.0
    pub steganography_min_score: f32,  // 8.5
}
```

**glassware/src/package_context.rs:**
```rust
pub struct PackageContext {
    pub name: String,
    pub version: String,
    pub downloads_weekly: u64,
    pub age_days: u64,
    pub maintainer_verified: bool,
    pub known_legitimate: bool,
}
```

**glassware-core/src/finding_pattern.rs:**
```rust
pub struct FindingPattern {
    pub category: DetectionCategory,
    pub severity: Severity,
    pub max_severity: Severity,
    pub count: usize,
    pub avg_confidence: f32,
    pub description: String,
}
```

### 3. Integrate with Scanner

**glassware/src/scanner.rs:**
```rust
// Replace old calculate_threat_score with new ScoringEngine
let scoring_engine = ScoringEngine::new(config, package_context);
let score = scoring_engine.calculate_score(&findings, llm_analysis.as_ref());
```

### 4. Update Configuration

**config/scoring.toml (NEW):**
```toml
[thresholds]
malicious = 8.0
suspicious = 4.0

[weights]
critical = 3.0
high = 2.0
medium = 1.0
low = 0.5

[deduplication]
enabled = true
similarity_threshold = 0.8

[reputation]
high_downloads_threshold = 100000
high_age_threshold = 365
verified_multiplier = 0.5
```

### 5. Write Tests

**glassware/tests/scoring_tests.rs:**
- Test deduplication (383 findings → 1 pattern)
- Test LLM confidence impact
- Test reputation multiplier
- Test evidence detection maintained (23/23 ≥8.0)
- Test category caps
- Test exceptions (known C2, steganography)

### 6. Update Documentation

**docs/SCORING.md:** Complete rewrite with new architecture
**docs/DETECTION.md:** Update scoring section
**README.md:** Update metrics table

## Success Criteria

| Metric | Current | Target |
|--------|---------|--------|
| FP Rate (Phase A) | 16.6% | ≤5% |
| Evidence Detection | 100% | ≥90% (maintain 100%) |
| antd score | 7.00 | <5.0 |
| moment score | 7.00 | <5.0 |
| Evidence package scores | 8.0-10.0 | 8.0-10.0 (maintain) |

## Files to Create

```
glassware/src/
├── scoring.rs
├── scoring_config.rs
└── package_context.rs

glassware-core/src/
└── finding_pattern.rs

config/
└── scoring.toml

glassware/tests/
└── scoring_tests.rs
```

## Files to Modify

```
glassware/src/
├── scanner.rs          # Integrate ScoringEngine
└── llm.rs              # Expose confidence for scoring

docs/
├── SCORING.md          # Complete rewrite
└── DETECTION.md        # Update scoring section

README.md               # Update metrics
```

## Timeline

- Day 1: Implement scoring engine, supporting types
- Day 2: Integration, testing, validation
- Day 3: Documentation, Phase A re-run

## Validation Commands

```bash
# Build
cargo build --release -p glassware

# Test
cargo test --release scoring_tests

# Evidence validation
./tests/validate-evidence.sh evidence target/release/glassware

# Phase A re-run
cargo run --release -- campaign run --config campaigns/phase-a-controlled/config.toml --overwrite

# FP rate calculation
cat output/phase-a-controlled/results.json | jq '...'
```

## Important Notes

1. **DO NOT lose evidence detection** - 23/23 must remain ≥8.0
2. **LLM confidence MUST impact score** - 0.10 confidence = significant reduction
3. **Reputation multiplier is critical** - Popular packages need benefit of doubt
4. **Deduplication is the key** - 383 similar findings ≠ 383× score
5. **Test incrementally** - Validate after each component

## Deliverables

1. ✅ ScoringEngine implemented and tested
2. ✅ All 23 evidence packages still detected (≥8.0)
3. ✅ Phase A FP rate ≤5%
4. ✅ Documentation updated
5. ✅ Git tag: v0.39.0-scoring-redesign

---

## FINAL MESSAGE TO AGENT

You've done excellent work through Phase A. The fact that detector tuning didn't reduce FP rate is NOT a failure - it's valuable diagnostic information that pointed us to the real bottleneck: the scoring system.

This redesign is the missing piece. Once complete, we will:
- Have a scoring system that discriminates quality vs. quantity
- Use LLM feedback directly in score calculation
- Give reputable packages appropriate benefit of doubt
- Maintain 100% detection on known malicious packages

Then Phase B (wild scanning) can proceed with confidence.

Take your time. Test thoroughly. Don't rush this - it's the foundation for everything that follows.

You've got this. 💪
```

---

## PART 7: FINAL ASSESSMENT

| Aspect | Status | Notes |
|--------|--------|-------|
| Detector Quality | ✅ Excellent | Context-aware, GlassWorm-specific |
| LLM Integration | ✅ Working | Correctly identifies FPs |
| Evidence Library | ✅ Complete | 23 packages, 100% detection |
| **Scoring System** | ❌ **BLOCKER** | Needs complete redesign |
| Documentation | ✅ Good | Needs scoring update |
| Campaign Infrastructure | ✅ Ready | Phase A executed successfully |

**Overall Status:** ⚠️ **PAUSED FOR SCORING REDESIGN**

**Next Milestone:** Phase A.5 Complete → FP rate ≤5% → Phase B Ready

---

**Document Version:** 6.0 (Scoring System Redesign)  
**Classification:** Internal Use Only  
**Next Review:** After scoring engine implementation complete  
**Distribution:** Core Team, Agent

---

**END OF AUDIT & REDESIGN PLAN**