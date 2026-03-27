# Tiered Scoring Fix - Correct Implementation Plan

**Date:** 2026-03-27
**Based on:** Actual codebase research (not assumptions)
**Checkpoint:** v0.79.0-tiered-scoring-review

---

## Root Causes Identified

### Root Cause #1: Campaign Config Struct Missing Fields

**File:** `glassware/src/campaign/config.rs` lines 494-510

**Current (BROKEN):**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringConfig {
    #[serde(default = "default_malicious_threshold")]
    pub malicious_threshold: f32,
    #[serde(default = "default_suspicious_threshold")]
    pub suspicious_threshold: f32,
    // ❌ MISSING: tier_config, weights, conditional_rules
}
```

**Impact:** When TOML is deserialized, `tier_config`, `tiers`, and `weights` are SILENTLY DROPPED.

---

### Root Cause #2: Scanner Ignores Campaign Config

**File:** `glassware/src/scanner.rs` lines 209 and 430

**Current (BROKEN):**
```rust
// Line 209 in scan_package()
let scoring_config = ScoringConfig::default();  // ❌ Ignores campaign config!

// Line 430 in scan_tarball()  
let scoring_config = ScoringConfig::default();  // ❌ Ignores campaign config!
```

**Impact:** Even if campaign config were fixed, scanner uses defaults anyway.

---

### Root Cause #3: Wave Executor References Non-Existent Fields

**File:** `glassware/src/campaign/wave.rs` lines 43-71

**Current (BROKEN):**
```rust
let scoring_config = crate::scoring_config::ScoringConfig {
    malicious_threshold: settings.scoring.malicious_threshold,
    suspicious_threshold: settings.scoring.suspicious_threshold,
    category_weight: 2.0,        // ❌ Hardcoded
    critical_weight: 3.0,        // ❌ Hardcoded
    high_weight: 1.5,            // ❌ Hardcoded
    tier_config: settings.scoring.tier_config.clone(),  // ❌ Field doesn't exist!
};
```

**Impact:** Compilation error when trying to access `settings.scoring.tier_config`.

---

## Correct Implementation Plan

### Phase 1: Fix Campaign Config Struct (30 min)

**File:** `glassware/src/campaign/config.rs`

**Action:** Add missing fields to `ScoringConfig` struct

```rust
// ADD these imports at top of file
use crate::scoring_config::{TierConfig, DetectorWeights, ConditionalRule};

// UPDATE ScoringConfig struct (lines 494-510)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringConfig {
    #[serde(default = "default_malicious_threshold")]
    pub malicious_threshold: f32,
    #[serde(default = "default_suspicious_threshold")]
    pub suspicious_threshold: f32,
    
    // ADD THESE FIELDS:
    #[serde(default)]
    pub tier_config: TierConfig,
    
    #[serde(default)]
    pub weights: DetectorWeights,
    
    #[serde(default)]
    pub conditional_rules: Vec<ConditionalRule>,
}
```

**Why this works:** TOML deserialization will now populate all fields instead of silently dropping them.

---

### Phase 2: Fix Wave Executor (30 min)

**File:** `glassware/src/campaign/wave.rs`

**Action:** Properly map campaign config to scoring config

```rust
// UPDATE lines 43-71
let scoring_config = crate::scoring_config::ScoringConfig {
    malicious_threshold: settings.scoring.malicious_threshold,
    suspicious_threshold: settings.scoring.suspicious_threshold,
    category_weight: 2.0,   // Keep hardcoded (not in campaign config)
    critical_weight: 3.0,   // Keep hardcoded
    high_weight: 1.5,       // Keep hardcoded
    tier_config: settings.scoring.tier_config.clone(),  // ✅ NOW WORKS!
    tiers: settings.scoring.tiers.clone(),              // ✅ ADD THIS
    weights: settings.scoring.weights.clone(),          // ✅ ADD THIS
    conditional_rules: settings.scoring.conditional_rules.clone(), // ✅ ADD THIS
};

// ADD logging for debugging
log::info!("Loaded scoring config: {:?}", scoring_config);
log::info!("Tier config mode: {:?}", scoring_config.tier_config.mode);
log::info!("Number of tiers: {}", scoring_config.tiers.len());
```

**Why this works:** All fields from campaign TOML are now properly passed to scoring config.

---

### Phase 3: Fix Scanner (1 hour)

**File:** `glassware/src/scanner.rs`

**Action:** Use config from ScannerConfig instead of default

**Step 1:** Add scoring config to ScannerConfig (around line 165)

```rust
// ADD to ScannerConfig struct
pub struct ScannerConfig {
    pub max_concurrent: usize,
    pub enable_llm: bool,
    pub threat_threshold: f32,
    pub scoring: crate::scoring_config::ScoringConfig,  // ADD THIS
}

// UPDATE ScannerConfig::default() (around line 180)
impl Default for ScannerConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 20,
            enable_llm: false,
            threat_threshold: 7.0,
            scoring: crate::scoring_config::ScoringConfig::default(),  // ADD THIS
        }
    }
}
```

**Step 2:** Update scan_package() to use config (line 209)

```rust
// REPLACE line 209
let scoring_config = ScoringConfig::default();  // ❌ OLD

// WITH THIS:
let scoring_config = self.config.scoring.clone();  // ✅ NEW - uses campaign config
```

**Step 3:** Update scan_tarball() to use config (line 430)

```rust
// REPLACE line 430
let scoring_config = ScoringConfig::default();  // ❌ OLD

// WITH THIS:
let scoring_config = self.config.scoring.clone();  // ✅ NEW - uses campaign config
```

**Why this works:** Scanner now uses the config passed from campaign instead of hardcoded defaults.

---

### Phase 4: Update Wave Executor to Pass Config (30 min)

**File:** `glassware/src/campaign/wave.rs`

**Action:** Pass scoring config when creating Scanner

**Find the scanner creation** (around line 53-71) and update:

```rust
let scanner = crate::scanner::Scanner::with_config(
    crate::scanner::ScannerConfig {
        max_concurrent: max_concurrency,
        glassware_config: glassware_core::GlasswareConfig {
            scoring: scoring_config,  // ✅ Pass the scoring config we created
            detectors: glassware_core::DetectorWeights {
                // ... existing detector weights ...
            },
        },
        scoring: scoring_config.clone(),  // ✅ ADD THIS - for scanner's own use
        ..Default::default()
    }
);
```

**Why this works:** Scanner receives scoring config from campaign and uses it for all package scans.

---

### Phase 5: Testing & Validation (2 hours)

**Test 1:** Compile check
```bash
cargo build -p glassware
# Should compile without errors
```

**Test 2:** Run wave25 validation
```bash
rm -f .glassware*.db*
./target/debug/glassware campaign run campaigns/wave25-scoring-validation.toml
```

**Expected Results:**
- `systemjs-plugin-babel@0.0.25` → Score should be ~4.0-5.0 (not 10.00)
- `babel-plugin-angularjs-annotate@0.10.0` → Score should be ~0-2.0 (not 9.00)
- Tiered scoring should be active (check logs for "Tier config mode: Tiered")

**Test 3:** Check logs for tiered scoring
```bash
grep -E "tier|Tier" /tmp/wave25-run.log | head -20
# Should show tiered scoring being used
```

---

## Files to Modify

| File | Changes | Risk |
|------|---------|------|
| `glassware/src/campaign/config.rs` | Add 3 fields to ScoringConfig | 🟡 Medium (TOML parsing) |
| `glassware/src/campaign/wave.rs` | Fix scoring config mapping | 🟢 Low (straightforward mapping) |
| `glassware/src/scanner.rs` | Use config from ScannerConfig | 🟡 Medium (core scanning logic) |

**Total Lines Changed:** ~30 lines across 3 files

---

## Why This Will Work

1. **TOML fields now match struct** - No more silent dropping of tier_config
2. **Config flows through entire pipeline** - Campaign → Wave → Scanner → ScoringEngine
3. **No defaults used** - All config comes from campaign TOML
4. **Tiered scoring will activate** - ScoringEngine checks `tier_config.mode` and uses tiered logic

---

## Verification Checklist

After implementation:

- [ ] Code compiles without errors
- [ ] `cargo test -p glassware` passes
- [ ] Wave25 runs without errors
- [ ] Logs show "Tier config mode: Tiered"
- [ ] systemjs-plugin-babel scores < 7.0
- [ ] babel-plugin-angularjs-annotate scores < 7.0
- [ ] Real GlassWorm evidence still scores ≥ 7.0
- [ ] No performance regression

---

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| TOML parsing breaks | Test with wave25 config first |
| Existing waves break | Run wave22-24 to verify backward compatibility |
| Performance regression | Benchmark before/after |
| Scores too low | Adjust tier thresholds in TOML configs |

---

**Ready to implement. This plan is based on ACTUAL code, not assumptions.**
