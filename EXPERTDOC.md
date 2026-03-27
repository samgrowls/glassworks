```markdown
# Glassworks Tiered Scoring System - Expert Review & Implementation Plan

**Checkpoint:** `v0.79.0-tiered-scoring-review`  
**Priority:** 🔴 Critical  
**Estimated Effort:** 2-3 days  
**Risk Level:** 🟡 Medium  
**Status:** Ready for Implementation

---

## Executive Summary

### The Problem

Campaign TOML files define tiered scoring configurations (e.g., obfuscation penalties in Tier 2), but the scanner ignores these and uses `ScoringConfig::default()`. This causes bundled packages to incorrectly score **9.0-10.0** instead of the expected **4.0-5.0**.

### The Solution

Modify the scanner to load and apply the campaign-specific `ScoringConfig` from the TOML configuration. This requires changes to config loading, scanner initialization, and score calculation pipelines.

### Key Metrics

| Metric | Value |
|--------|-------|
| Scoring Tiers Defined | 3 |
| Config Currently Ignored | 100% |
| Score Variance (Expected vs Actual) | ~5.0 points |
| 10k Scan Status | 🚫 Blocked |

---

## Problem Analysis

### Root Cause

1. **`wave.rs` creates scanner with `ScoringConfig::default()`** - The campaign config is parsed but never passed to the scanner instance.

2. **Campaign TOML scoring config is parsed but never utilized** - Configuration data exists but flows nowhere.

3. **No validation that config was actually applied** - No runtime checks or logging to verify correct behavior.

### Impact

| Impact Area | Severity | Description |
|-------------|----------|-------------|
| 10k Scan | 🔴 Critical | Cannot proceed with production validation |
| Score Accuracy | 🔴 Critical | Bundled packages receive inflated scores (9-10 vs expected 4-5) |
| Tier Differentiation | 🟠 High | Tier-based scoring completely non-functional |
| Production Readiness | 🟠 High | Cannot ship with broken scoring |

### Affected Files

| File | Issue | Priority |
|------|-------|----------|
| `glassware/src/campaign/wave.rs` | Scanner initialized with default config | 🔴 Critical |
| `glassware/src/scoring/config.rs` | Config struct exists but not utilized | 🟠 High |
| `glassware/src/scanner/mod.rs` | No config parameter in scanner init | 🟠 High |
| `campaigns/wave25-scoring-validation.toml` | Test config exists, not being applied | 🟡 Medium |

---

## Architecture Review

### Current Architecture Issues

```
┌─────────────────────────────────────────────────────────────────┐
│                    CURRENT (BROKEN) FLOW                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Campaign TOML                                                  │
│       │                                                         │
│       ▼                                                         │
│  ┌─────────────┐                                                │
│  │   Parser    │ ──► ScoringConfig (parsed but discarded) ❌   │
│  └─────────────┘                                                │
│       │                                                         │
│       ▼                                                         │
│  ┌─────────────┐                                                │
│  │   Wave.rs   │                                                │
│  └─────────────┘                                                │
│       │                                                         │
│       ▼                                                         │
│  ┌─────────────┐      ScoringConfig::default()                  │
│  │   Scanner   │ ◄─────────────────────────────── ❌            │
│  └─────────────┘      (hardcoded, ignores campaign)             │
│       │                                                         │
│       ▼                                                         │
│  ┌─────────────┐                                                │
│  │   Scoring   │ ──► Wrong scores produced                     │
│  └─────────────┘                                                │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Recommended Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    RECOMMENDED (FIXED) FLOW                     │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Campaign TOML                                                  │
│       │                                                         │
│       ▼                                                         │
│  ┌─────────────┐                                                │
│  │   Parser    │ ──► ScoringConfig (validated & logged) ✅     │
│  └─────────────┘                                                │
│       │                                                         │
│       ▼                                                         │
│  ┌─────────────┐                                                │
│  │   Wave.rs   │ ──► Pass config to scanner ✅                 │
│  └─────────────┘                                                │
│       │                                                         │
│       ▼                                                         │
│  ┌─────────────┐      Injected ScoringConfig                    │
│  │   Scanner   │ ◄─────────────────────────────── ✅            │
│  └─────────────┘      (uses campaign config)                    │
│       │                                                         │
│       ▼                                                         │
│  ┌─────────────┐                                                │
│  │   Scoring   │ ──► Correct tiered scores produced ✅         │
│  └─────────────┘                                                │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Architectural Principle

> **Configuration should flow from campaign definition → wave manager → scanner → scoring engine.** Each layer should receive config from the layer above, never create defaults independently.

---

## Implementation Plan

### Phase 1: Config Loading & Validation (Day 1)

**Goal:** Ensure campaign TOML scoring config is properly parsed and validated.

- [ ] Verify `ScoringConfig` deserialization from TOML
- [ ] Add config validation logging in `wave.rs`
- [ ] Create unit tests for config parsing edge cases
- [ ] Add `Debug` trait to `ScoringConfig` for logging

**Deliverable:** Config loads correctly with validation and logging.

### Phase 2: Scanner Integration (Day 1-2)

**Goal:** Modify scanner to accept and use campaign scoring config.

- [ ] Update scanner constructor to accept `ScoringConfig` parameter
- [ ] Replace `ScoringConfig::default()` with injected config
- [ ] Update `wave.rs` to pass config to scanner
- [ ] Ensure scoring engine uses config for all calculations

**Deliverable:** Scanner uses campaign config instead of defaults.

### Phase 3: Validation Testing (Day 2)

**Goal:** Comprehensive testing to verify fix works correctly.

- [ ] Run `wave25-scoring-validation.toml` test suite
- [ ] Verify bundled packages score 4.0-5.0 (not 9.0-10.0)
- [ ] Test all three tiers produce differentiated scores
- [ ] Run 10k scan preparation validation
- [ ] Performance benchmark (ensure no regression)

**Deliverable:** All tests pass, scores are correct.

### Phase 4: Production Deploy (Day 3)

**Goal:** Deploy fix and enable 10k scan.

- [ ] Create PR with all changes and documentation
- [ ] Code review and approval
- [ ] Merge and deploy to staging
- [ ] Enable 10k scan in production
- [ ] Monitor initial scan results

**Deliverable:** Fix deployed, 10k scan unblocked.

---

## Required Code Changes

### 1. Modify `glassware/src/campaign/wave.rs`

**Purpose:** Load scoring config from campaign and pass to scanner constructor.

```rust
// BEFORE (Broken)
let scanner = Scanner::new(
    ScoringConfig::default(),  // ❌ Ignores campaign config
    detection_engine,
);

// AFTER (Fixed)
let scoring_config = campaign_config
    .scoring
    .clone()
    .unwrap_or_default();  // ✅ Load from TOML

log::info!("Loaded scoring config: {:?}", scoring_config);

// Validate config before use
if let Err(e) = scoring_config.validate() {
    log::error!("Invalid scoring config: {}", e);
    return Err(ScanError::InvalidConfig(e));
}

let scanner = Scanner::new(
    scoring_config,  // ✅ Use campaign config
    detection_engine,
);
```

### 2. Modify `glassware/src/scanner/mod.rs`

**Purpose:** Update scanner constructor to accept ScoringConfig parameter.

```rust
// BEFORE
impl Scanner {
    pub fn new(
        _config: ScoringConfig,  // Parameter exists but unused
        detection_engine: DetectionEngine,
    ) -> Self {
        Self {
            config: ScoringConfig::default(),  // ❌ Hardcoded
            detection_engine,
        }
    }
}

// AFTER
impl Scanner {
    pub fn new(
        config: ScoringConfig,  // ✅ Now used
        detection_engine: DetectionEngine,
    ) -> Self {
        Self {
            config,  // ✅ Store injected config
            detection_engine,
        }
    }
    
    // Add getter for debugging/validation
    pub fn config(&self) -> &ScoringConfig {
        &self.config
    }
}
```

### 3. Modify `glassware/src/scoring/config.rs`

**Purpose:** Add Debug trait and validation method for better logging.

```rust
// ADD to ScoringConfig struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringConfig {
    pub tiers: Vec<TierConfig>,
    pub weights: ScoringWeights,
    // ... existing fields
}

// ADD validation method
impl ScoringConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.tiers.is_empty() {
            return Err("At least one scoring tier required".to_string());
        }
        
        // Validate tier thresholds are ordered
        for i in 1..self.tiers.len() {
            if self.tiers[i].min_score <= self.tiers[i-1].min_score {
                return Err(format!(
                    "Tier {} min_score must be greater than tier {}",
                    i, i-1
                ));
            }
        }
        
        // Validate weights sum to 1.0 (or appropriate total)
        let weight_sum = self.weights.obfuscation 
            + self.weights.malware 
            + self.weights.suspicious;
        
        if (weight_sum - 1.0).abs() > 0.01 {
            log::warn!("Scoring weights sum to {}, expected 1.0", weight_sum);
        }
        
        Ok(())
    }
}
```

### 4. Add `tests/tiered_scoring_integration.rs`

**Purpose:** Integration test to verify tiered scoring works end-to-end.

```rust
// NEW TEST FILE
#[cfg(test)]
mod tiered_scoring_integration {
    use super::*;

    #[test]
    fn test_bundled_package_tier2_score() {
        // Load wave25-scoring-validation.toml
        let config = load_campaign_config("wave25-scoring-validation.toml");
        
        // Create scanner with config
        let scanner = Scanner::new(config.scoring, detection_engine);
        
        // Scan bundled package (should be Tier 2)
        let result = scanner.scan(bundled_package);
        
        // Verify score is in expected range (4.0-5.0)
        assert!(
            result.score >= 4.0 && result.score <= 5.0,
            "Bundled package should score 4.0-5.0, got {}", 
            result.score
        );
    }

    #[test]
    fn test_clean_package_tier1_score() {
        let config = load_campaign_config("wave25-scoring-validation.toml");
        let scanner = Scanner::new(config.scoring, detection_engine);
        
        let result = scanner.scan(clean_package);
        
        assert!(
            result.score >= 8.0 && result.score <= 10.0,
            "Clean package should score 8.0-10.0, got {}", 
            result.score
        );
    }

    #[test]
    fn test_malicious_package_tier3_score() {
        let config = load_campaign_config("wave25-scoring-validation.toml");
        let scanner = Scanner::new(config.scoring, detection_engine);
        
        let result = scanner.scan(malicious_package);
        
        assert!(
            result.score >= 0.0 && result.score <= 2.0,
            "Malicious package should score 0.0-2.0, got {}", 
            result.score
        );
    }
}
```

---

## Validation & Testing

### Test Categories

| Category | Purpose | Key Tests |
|----------|---------|-----------|
| **Unit Tests** | Test individual components in isolation | ScoringConfig deserialization, Tier threshold calculations, Weight application logic |
| **Integration Tests** | Test full scanning pipeline with config | Wave25 validation suite, End-to-end scan flow, Config propagation verification |
| **Performance Tests** | Ensure fix doesn't impact performance | 10k scan benchmark, Memory usage profiling, Config load time measurement |

### Expected Test Results

| Test Case | Package Type | Expected Score | Current (Broken) | Status |
|-----------|--------------|----------------|------------------|--------|
| Clean Package | Tier 1 | 8.0-10.0 | 8.0-10.0 ✓ | ✅ Working |
| Bundled Package | Tier 2 | 4.0-5.0 | 9.0-10.0 ✗ | 🔴 Broken |
| Obfuscated Package | Tier 2 | 3.0-4.0 | 8.0-9.0 ✗ | 🔴 Broken |
| Malicious Package | Tier 3 | 0.0-2.0 | 5.0-7.0 ✗ | 🔴 Broken |

### Validation Checklist

- [ ] All unit tests pass
- [ ] Integration tests pass with expected score ranges
- [ ] No performance regression (>5% slowdown)
- [ ] Config logging shows correct values
- [ ] 10k scan completes successfully
- [ ] Score distribution matches expected tier breakdown

---

## General Code Review Recommendations

### Strengths Observed

✅ **Well-structured TOML configs** - Campaign configurations are cleanly organized and easy to modify.

✅ **Type-safe Rust codebase** - Strong typing helps catch errors at compile time.

✅ **Modular architecture** - Clear separation between campaign, scanner, and scoring modules.

✅ **Documentation present** - `TIERED-SCORING-EXPERT-REVIEW.md` shows good documentation practices.

### Areas for Improvement

⚠️ **Add config validation** - Validate TOML configs on load to catch errors early.

⚠️ **Improve error messages** - Add context to errors for easier debugging.

⚠️ **Add integration tests** - More end-to-end tests for critical paths.

⚠️ **Config schema documentation** - Document all available TOML config options.

### Code Quality Checklist

- [ ] All functions have doc comments
- [ ] Error handling uses Result types consistently
- [ ] No `unwrap()` in production code (use `?` or proper error handling)
- [ ] Logging at appropriate levels (info, warn, error)
- [ ] Unit tests cover edge cases
- [ ] CI/CD pipeline runs all tests
- [ ] No unused variables or imports
- [ ] Consistent naming conventions

---

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Breaking existing scans | Low | High | Run full test suite before deploy |
| Performance regression | Low | Medium | Benchmark before/after |
| Config parsing errors | Medium | Medium | Add validation and fallback defaults |
| Tier threshold misconfiguration | Low | High | Validate thresholds in config load |

---

## Success Criteria

The fix is complete when:

1. ✅ Bundled packages score 4.0-5.0 (not 9.0-10.0)
2. ✅ All three tiers produce differentiated scores
3. ✅ 10k scan completes without errors
4. ✅ Config loading is logged and validated
5. ✅ No performance regression (>5%)
6. ✅ All existing tests still pass

---

## Appendix: Quick Reference

### Files to Modify

```
glassware/
├── src/
│   ├── campaign/
│   │   └── wave.rs              # PRIMARY: Pass config to scanner
│   ├── scanner/
│   │   └── mod.rs               # PRIMARY: Accept config parameter
│   └── scoring/
│       └── config.rs            # SECONDARY: Add validation/logging
└── tests/
    └── tiered_scoring_integration.rs  # NEW: Integration tests

campaigns/
└── wave25-scoring-validation.toml     # Existing test config
```

### Command Reference

```bash
# Run unit tests
cargo test --package glassware

# Run integration tests
cargo test --test tiered_scoring_integration

# Run with logging
RUST_LOG=info cargo run -- scan --campaign wave25

# Benchmark
cargo bench --package glassware
```

### Key Types

```rust
ScoringConfig {
    tiers: Vec<TierConfig>,
    weights: ScoringWeights,
}

TierConfig {
    name: String,
    min_score: f64,
    max_score: f64,
    penalties: Vec<Penalty>,
}

ScoringWeights {
    obfuscation: f64,
    malware: f64,
    suspicious: f64,
}
```

---

## Document Information

| Field | Value |
|-------|-------|
| Document Type | Expert Review & Implementation Plan |
| Version | 1.0 |
| Checkpoint | v0.79.0-tiered-scoring-review |
| Author | Software Architecture Expert |
| Date | 2024 |
| Status | Ready for Implementation |

---

*This document should be referenced during implementation. All code changes should be reviewed against the specifications herein.*
```