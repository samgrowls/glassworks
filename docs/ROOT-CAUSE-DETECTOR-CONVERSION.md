# Root Cause Identified: Detector Weights Not Converted

**Date:** 2026-03-24
**Severity:** CRITICAL
**Status:** Root Cause Identified - Fix Ready

---

## Summary

**Root Cause:** In `glassware/src/main.rs:951`, the detector weights loaded from `~/.config/glassware/config.toml` are **completely ignored** and replaced with `DetectorWeights::default()`.

This is a **simple conversion bug** - the config is loaded properly but not passed to the scanner.

---

## The Bug

### Location: `glassware/src/main.rs:939-951`

```rust
glassware_config: glassware_core::GlasswareConfig {
    whitelist: glassware_core::WhitelistConfig {
        packages: glassware_config.whitelist.packages.clone(),
        crypto_packages: glassware_config.whitelist.crypto_packages.clone(),
        build_tools: glassware_config.whitelist.build_tools.clone(),
        state_management: vec![],  // ← BUG 1: Not copied
    },
    scoring: glassware_core::ScoringConfig {
        malicious_threshold: glassware_config.scoring.malicious_threshold,
        suspicious_threshold: glassware_config.scoring.suspicious_threshold,
        category_weight: glassware_config.scoring.category_weight,
        critical_weight: glassware_config.scoring.critical_weight,
        high_weight: glassware_config.scoring.high_weight,
    },
    detectors: glassware_core::DetectorWeights::default(),  // ← BUG 2: Using defaults!
},
```

### What Should Happen

The `glassware_config.detectors` (loaded from TOML) should be converted to `glassware_core::DetectorWeights`:

```rust
detectors: glassware_core::DetectorWeights {
    invisible_char: glassware_config.detectors.invisible_char.weight,
    homoglyph: glassware_config.detectors.homoglyph.weight,
    bidi: glassware_config.detectors.bidi.weight,
    blockchain_c2: glassware_config.detectors.blockchain_c2.weight,
    glassware_pattern: glassware_config.detectors.glassware_pattern.weight,
    locale_geofencing: glassware_config.detectors.locale_geofencing.weight,
    // ... etc
},
```

---

## Config Structure Mismatch

### Source: `crate::config::DetectorConfig`

```rust
pub struct DetectorConfig {
    pub invisible_char: DetectorWeightConfig,
    pub homoglyph: DetectorWeightConfig,
    pub bidi: DetectorWeightConfig,
    pub blockchain_c2: DetectorWeightConfig,
    pub glassware_pattern: DetectorWeightConfig,
    pub locale_geofencing: LocaleGeofencingConfig,
}

pub struct DetectorWeightConfig {
    pub enabled: bool,
    pub weight: f32,
}
```

### Target: `glassware_core::DetectorWeights`

```rust
pub struct DetectorWeights {
    pub invisible_char: f32,
    pub homoglyph: f32,
    pub bidi: f32,
    pub blockchain_c2: f32,
    pub glassware_pattern: f32,
    pub locale_geofencing: f32,
    pub time_delay: f32,
    pub encrypted_payload: f32,
    pub rdd: f32,
    pub forcememo: f32,
    pub jpd_author: f32,
}
```

### Missing Conversion

There is **no `From` or `Into` trait** to convert between these types. The conversion must be done manually in `main.rs`.

---

## Impact

### Current Behavior

- Config file has `blockchain_c2.weight = 2.0`
- Code uses `DetectorWeights::default()` which has `blockchain_c2 = 1.0`
- **Result:** C2 detection is half as effective as configured

### Affected Detectors

| Detector | Config Weight | Default Weight | Difference |
|----------|--------------|----------------|------------|
| invisible_char | 1.0 | 1.0 | ✅ Same |
| homoglyph | 1.0 | 1.0 | ✅ Same |
| bidi | 1.0 | 1.0 | ✅ Same |
| blockchain_c2 | **2.0** | **1.0** | ❌ 50% reduction |
| glassware_pattern | 3.0 | 3.0 | ✅ Same |
| locale_geofencing | 1.0 | 1.0 | ✅ Same |

**Note:** Most detectors have the same weight, so the impact is limited. The main issue is `blockchain_c2`.

---

## Additional Bugs Found

### Bug 1: `state_management` Not Copied

Line 944: `state_management: vec![]`

The `state_management` whitelist category is hardcoded to empty instead of copying from config:

```rust
state_management: glassware_config.whitelist.state_management.clone(),
```

### Bug 2: Whitelist Matching May Be Broken

Even with the whitelist loaded, webpack is still being flagged. This suggests the whitelist matching logic itself may have issues.

**Hypothesis:** The whitelist check happens in `calculate_threat_score()` but the scoring might be happening BEFORE the whitelist check, or the whitelist check is failing for some reason.

---

## Fix Plan

### Step 1: Add Conversion Function

Add a helper function to convert `crate::config::DetectorConfig` to `glassware_core::DetectorWeights`:

```rust
fn convert_detector_weights(
    config: &crate::config::DetectorConfig
) -> glassware_core::DetectorWeights {
    glassware_core::DetectorWeights {
        invisible_char: config.invisible_char.weight,
        homoglyph: config.homoglyph.weight,
        bidi: config.bidi.weight,
        blockchain_c2: config.blockchain_c2.weight,
        glassware_pattern: config.glassware_pattern.weight,
        locale_geofencing: config.locale_geofencing.weight,
        time_delay: 1.0,  // Default (not in crate::config)
        encrypted_payload: 3.0,  // Default
        rdd: 3.0,  // Default
        forcememo: 3.0,  // Default
        jpd_author: 3.0,  // Default
    }
}
```

### Step 2: Fix `state_management` Copy

Change line 944:
```rust
state_management: glassware_config.whitelist.state_management.clone(),
```

### Step 3: Debug Whitelist Matching

Add logging to verify:
1. Config is loaded with correct whitelist values
2. `is_package_whitelisted()` is called with correct package name
3. Matching logic works for "webpack"

### Step 4: Test

```bash
# Test whitelist
./target/release/glassware scan-npm webpack@5.89.0
# Expected: NOT flagged as malicious

# Test blockchain C2 detection
./target/release/glassware scan-npm ethers@6.9.2
# Expected: Proper weight applied
```

---

## Long-Term Design: Configurable Scoring Functions

As discussed, the scoring system should be modular and parametrized. Current design:

```rust
score = (categories × category_weight) +
        (critical_hits × critical_weight) +
        (high_hits × high_weight)
```

### Proposed Enhancement

Allow users to define custom scoring functions via config:

```toml
[scoring]
# Formula: "linear", "quadratic", "custom"
formula = "linear"

# Linear: score = a*categories + b*critical + c*high
linear_weights = { a = 2.0, b = 3.0, c = 1.5 }

# Or custom formula with Lua/Python scripting
# custom_script = "scoring/my_formula.lua"
```

### Benefits

1. **Flexibility:** Different campaigns can use different scoring strategies
2. **Tuning:** Easy to adjust without code changes
3. **Experimentation:** Test different formulas without recompiling
4. **Transparency:** Users can see exactly how scores are calculated

### Implementation

1. Add `ScoringFormula` enum with variants for different formulas
2. Implement formula evaluator (simple enum match or embed Lua)
3. Add formula selection to config
4. Document formula options and use cases

---

## Next Actions

### Immediate (This Session)

1. ✅ Document root cause
2. ⏳ Fix detector weight conversion in `main.rs`
3. ⏳ Fix `state_management` copy
4. ⏳ Add debug logging for whitelist
5. ⏳ Test with webpack scan

### Short-Term

1. Run Wave 8/9/10 to verify whitelist working
2. Test evidence detection with proper weights
3. Document scoring formula for users

### Medium-Term

1. Implement configurable scoring formulas
2. Add scoring formula presets (conservative, balanced, aggressive)
3. Document scoring tuning guide

---

**Last Updated:** 2026-03-24 08:30 UTC
**Investigator:** Qwen-Coder
**Status:** Ready to implement fix
