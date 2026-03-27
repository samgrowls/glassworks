# Tiered Scoring Architectural Issue - Expert Review Request

**Date:** 2026-03-27
**Version:** v0.78.0-10k-ready
**Severity:** HIGH - Affects detection accuracy for 10k scan
**Requested:** Architectural review and fix implementation

---

## Executive Summary

**Problem:** Campaign configuration defines tiered scoring (to prevent false positives from bundled/minified code), but the tiered scoring is NOT being applied during package scanning. Packages like `systemjs-plugin-babel@0.0.25` score 10.00 (malicious) when they should score ~4.0-5.0 (suspicious but not malicious).

**Root Cause:** Architectural disconnect between campaign config and scoring engine.

**Impact:** High false positive rate on legitimate bundled packages, undermining detection credibility.

**Requested:** Expert review of scoring architecture and implementation of proper tiered scoring flow.

---

## What We're Trying to Achieve

### Goal: Tiered Scoring for GlassWorm Detection

GlassWorm attacks have a specific pattern:
1. **Invisible Unicode characters** (steganography) - PRIMARY indicator
2. **Decoder function** to extract hidden payload
3. **C2 communication** (often blockchain-based)
4. **Sandbox evasion** patterns

**Tiered scoring ensures:**
- Packages WITHOUT invisible Unicode CANNOT score high (max ~4.0-5.0)
- Obfuscation alone (common in legitimate bundles) should NOT trigger malicious flag
- Only packages with Tier 1 signal (invisible chars OR GlassWorm pattern) can reach malicious threshold (7.0+)

### Configuration (Working Correctly)

Campaign TOML files define tiered scoring:

```toml
[settings.scoring.tier_config]
mode = "tiered"

# Tier 1: REQUIRED for high scores
[[settings.scoring.tiers]]
tier = 1
detectors = ["invisible_char", "glassware_pattern"]
threshold = 0.0
weight_multiplier = 1.0

# Tier 2: Requires Tier 1 signal first
[[settings.scoring.tiers]]
tier = 2
detectors = ["header_c2", "exfil_schema", "blockchain_c2", "obfuscation"]
threshold = 2.0
weight_multiplier = 0.8
```

**Expected behavior:**
- Package with obfuscation ONLY → Tier 1 score = 0 → Tier 2 skipped → Final score ~0-4.0
- Package with invisible chars + obfuscation → Tier 1 score = 6.0 → Tier 2 runs → Final score 8.0-10.0

---

## Current State - What Works

### ✅ Campaign Config Parsing

File: `glassware/src/campaign/config.rs`

TOML parsing works correctly. The `ScoringConfig` struct in campaign config has:
```rust
pub struct ScoringConfig {
    pub malicious_threshold: f32,
    pub suspicious_threshold: f32,
    pub tier_config: TierConfig,  // ✅ Parsed correctly
}
```

### ✅ Scoring Engine Implementation

File: `glassware/src/scoring.rs`

The `ScoringEngine` has proper tiered scoring logic:
```rust
pub fn calculate_score(&self, findings: &[Finding], llm_verdict: Option<&LlmVerdict>) -> f32 {
    if self.config.tier_config.mode == TierMode::Tiered && !self.config.tier_config.tiers.is_empty() {
        return self.calculate_score_tiered(findings, llm_verdict);  // ✅ Tiered logic exists
    }
    self.calculate_score_independent(findings, llm_verdict);  // ❌ Falls back to independent
}
```

### ✅ Detector Implementation

All detectors work correctly:
- `InvisibleCharDetector` - Detects invisible Unicode
- `GlasswarePatternDetector` - Detects decoder + execution patterns
- `ObfuscationDetector` - Detects minified/obfuscated code
- etc.

---

## The Snag - Where It Breaks

### ❌ Issue 1: Campaign Config Not Passed to Scanner

**Location:** `glassware/src/campaign/wave.rs` lines 44-52

**Current Code:**
```rust
let scoring_config = crate::scoring_config::ScoringConfig {
    malicious_threshold: settings.scoring.malicious_threshold,
    suspicious_threshold: settings.scoring.suspicious_threshold,
    category_weight: 2.0,
    critical_weight: 3.0,
    high_weight: 1.5,
    // ❌ MISSING: tier_config from campaign settings
};
```

**Problem:** The `tier_config` from campaign TOML is NOT copied to the `ScoringConfig` passed to Scanner.

**Attempted Fix (Failed):**
```rust
tier_config: settings.scoring.tier_config.clone(),  // ❌ Compilation error
```

**Error:** `glassware_core::config::ScoringConfig` doesn't have `tier_config` field!

### ❌ Issue 2: Scanner Uses Default Config

**Location:** `glassware/src/scanner.rs` line 209-212

**Current Code:**
```rust
pub async fn scan_package(&self, package: &DownloadedPackage) -> Result<PackageScanResult> {
    // ... scan directory ...
    
    // ❌ Uses default config, ignores campaign config
    let scoring_config = ScoringConfig::default();
    let package_context = PackageContext::new(package.name.clone(), package.version.clone());
    let scoring_engine = ScoringEngine::new(scoring_config, package_context);
    let threat_score = scoring_engine.calculate_score(&findings, None);
    
    // ...
}
```

**Problem:** Even if campaign config had `tier_config`, `scan_package()` creates a NEW `ScoringConfig::default()` for each package!

### ❌ Issue 3: Two ScoringConfig Structs

**Location:**
- `glassware-core/src/config.rs` - `ScoringConfig` (NO tier_config field)
- `glassware/src/scoring_config.rs` - `ScoringConfig` (HAS tier_config field)

**Problem:** There are TWO different `ScoringConfig` structs:

```rust
// glassware-core/src/config.rs (used by detectors)
pub struct ScoringConfig {
    pub malicious_threshold: f32,
    pub suspicious_threshold: f32,
    pub category_weight: f32,
    pub critical_weight: f32,
    pub high_weight: f32,
    // ❌ NO tier_config field!
}

// glassware/src/scoring_config.rs (used by ScoringEngine)
pub struct ScoringConfig {
    pub malicious_threshold: f32,
    pub suspicious_threshold: f32,
    pub category_weight: f32,
    pub critical_weight: f32,
    pub high_weight: f32,
    pub tier_config: TierConfig,  // ✅ HAS tier_config
    pub tiers: Vec<TierDefinition>,
    pub conditional_rules: Vec<ConditionalRule>,
}
```

**Impact:** Cannot pass tiered config from campaign → scanner → scoring engine because the core library struct doesn't support it!

---

## What I Know vs What I Suspect

### What I Know (Facts)

1. **Campaign TOML parsing works** - `settings.scoring.tier_config` is correctly parsed
2. **ScoringEngine tiered logic exists** - `calculate_score_tiered()` is implemented
3. **Two ScoringConfig structs exist** - One in core (no tiers), one in glassware (has tiers)
4. **Scanner ignores campaign config** - Uses `ScoringConfig::default()` per package
5. **Test results show issue** - `systemjs-plugin-babel` scores 10.00 (should be ~4.0)

### What I Suspect

1. **Historical refactoring** - Tiered scoring was added to `glassware/src/scoring.rs` but not propagated to `glassware-core`
2. **Scanner architecture** - `scan_package()` was designed for simple scoring, not campaign-based tiered scoring
3. **Missing abstraction** - No clean way to pass campaign scoring config to individual package scans

### What I'm Uncertain About

1. **Why two ScoringConfig structs?** - Is this intentional separation or historical accident?
2. **Should Scanner use campaign config?** - Or should each package have its own scoring context?
3. **Best fix approach** - Merge structs? Add delegation? Create new abstraction?

---

## Evidence of the Problem

### Test Case: systemjs-plugin-babel@0.0.25

**Characteristics:**
- 1.7MB bundled file (`systemjs-babel-node.js`)
- Contains 4 ZWNJ characters (U+200C) in Unicode identifier ranges (legitimate i18n support)
- NO decoder function
- NO C2 patterns
- Obfuscation: YES (bundled code)

**Expected Score (with tiered scoring):**
- Tier 1 (invisible_char): 1 finding × 6.0 weight = 6.0 (but capped due to context)
- Tier 1 (glassware_pattern): 0 findings = 0.0
- **Tier 1 Total: ~2.0** (below Tier 2 threshold of 2.0)
- Tier 2 (obfuscation): SKIPPED (threshold not met)
- **Final Score: ~2.0-4.0** (suspicious, NOT malicious)

**Actual Score (current behavior):**
- **Score: 10.00** (malicious)
- Obfuscation detector fired independently
- No tier gating applied

### Test Case: babel-plugin-angularjs-annotate@0.10.0

**Characteristics:**
- Babel plugin (legitimate tooling)
- NO invisible Unicode characters
- NO decoder function
- NO C2 patterns
- Obfuscation: NO (readable source code)

**Expected Score:**
- Tier 1 (invisible_char): 0 findings = 0.0
- Tier 1 (glassware_pattern): 0 findings = 0.0
- **Tier 1 Total: 0.0**
- Tier 2: SKIPPED
- **Final Score: 0.0** (clean)

**Actual Score:**
- **Score: 9.00** (malicious)
- Some detector fired (likely obfuscation or encrypted_payload)
- No tier gating applied

---

## Proposed Fix Approaches

### Approach 1: Merge ScoringConfig Structs (Cleanest)

**Action:** Add `tier_config` field to `glassware-core/src/config.rs::ScoringConfig`

```rust
// glassware-core/src/config.rs
pub struct ScoringConfig {
    pub malicious_threshold: f32,
    pub suspicious_threshold: f32,
    pub category_weight: f32,
    pub critical_weight: f32,
    pub high_weight: f32,
    pub tier_config: TierConfig,  // ADD THIS
}
```

**Then:** Update `glassware/src/campaign/wave.rs` to copy `tier_config`:
```rust
let scoring_config = glassware_core::config::ScoringConfig {
    malicious_threshold: settings.scoring.malicious_threshold,
    suspicious_threshold: settings.scoring.suspicious_threshold,
    category_weight: 2.0,
    critical_weight: 3.0,
    high_weight: 1.5,
    tier_config: settings.scoring.tier_config.clone(),  // NOW WORKS
};
```

**Then:** Update `glassware/src/scanner.rs` to use config from Scanner:
```rust
pub async fn scan_package(&self, package: &DownloadedPackage) -> Result<PackageScanResult> {
    // ... scan ...
    
    // USE CONFIG FROM SCANNER, NOT DEFAULT
    let scoring_engine = ScoringEngine::new(self.config.scoring.clone(), package_context);
    let threat_score = scoring_engine.calculate_score(&findings, None);
    
    // ...
}
```

**Pros:** Clean, single source of truth
**Cons:** Requires changes in multiple files, potential breaking changes

### Approach 2: Add ScoringConfig to ScannerConfig

**Action:** Add scoring config to Scanner's config struct

```rust
// glassware/src/scanner.rs
pub struct ScannerConfig {
    pub max_concurrent: usize,
    pub enable_llm: bool,
    pub threat_threshold: f32,
    pub scoring: ScoringConfig,  // ADD THIS
}
```

**Then:** Pass campaign config to Scanner constructor

**Pros:** Clear separation, minimal core changes
**Cons:** More indirection, two ScoringConfig structs still exist

### Approach 3: Create ScoringContext Abstraction

**Action:** Create new abstraction that holds both package context AND scoring config

```rust
pub struct ScoringContext {
    pub package: PackageContext,
    pub config: ScoringConfig,
}
```

**Then:** Pass `ScoringContext` instead of separate params

**Pros:** Clean API, groups related data
**Cons:** More refactoring, new abstraction layer

---

## Recommended Approach

**I recommend Approach 1** (Merge ScoringConfig structs) because:

1. **Single source of truth** - No confusion about which struct to use
2. **Minimal indirection** - Direct field access
3. **Future-proof** - Easy to add more scoring features
4. **Aligns with intent** - Tiered scoring was clearly intended (code exists)

---

## Files Requiring Changes

### Must Modify

1. **`glassware-core/src/config.rs`**
   - Add `tier_config: TierConfig` field to `ScoringConfig`
   - Add `Default` impl for `TierConfig`
   - May need to add `TierConfig`, `TierDefinition`, `TierMode`, `ConditionalRule` structs to core

2. **`glassware/src/campaign/wave.rs`**
   - Copy `tier_config` from campaign settings to scoring config (line 44-52)

3. **`glassware/src/scanner.rs`**
   - Store scoring config in `ScannerConfig`
   - Use stored config in `scan_package()` instead of `ScoringConfig::default()`

### May Need to Modify

4. **`glassware-core/src/lib.rs`** - Export new types if moved to core
5. **`glassware/src/scoring_config.rs`** - May need to reconcile with core version
6. **Tests** - Update any tests that create `ScoringConfig`

---

## Testing Requirements

After fix, verify:

1. **systemjs-plugin-babel@0.0.25** scores < 7.0 (not malicious)
2. **babel-plugin-angularjs-annotate@0.10.0** scores < 7.0 (not malicious)
3. **iflow-mcp-watercrawl-watercrawl-mcp-1.3.4** (real attack) still scores ≥ 7.0
4. **Synthetic GlassWorm** packages still detected correctly
5. **Wave25 validation** shows 0 malicious for bundled packages

---

## Timeline Impact

**Current:** 10k scan blocked (will produce unreliable results)
**With Fix:** 10k scan can proceed with confidence
**Estimated Fix Time:** 2-4 hours for experienced Rust developer
**Estimated Test Time:** 1-2 hours

---

## Contact

**Repository:** github.com/samgrowls/glassworks
**Branch:** main
**Checkpoint:** v0.78.0-10k-ready
**Documentation:** See `10K-PREP-STATUS.md` for full context

**Questions?** Review the following files for context:
- `glassware/src/scoring.rs` - ScoringEngine implementation (has tiered logic)
- `glassware/src/scoring_config.rs` - ScoringConfig with tier_config
- `glassware-core/src/config.rs` - ScoringConfig WITHOUT tier_config
- `glassware/src/campaign/wave.rs` - Where config conversion fails
- `glassware/src/scanner.rs` - Where default config is used incorrectly

---

**Last Updated:** 2026-03-27
**Prepared By:** AI Agent
**Status:** AWAITING EXPERT REVIEW
