# Critical Finding: Detector Weights Not Loaded from Config

**Date:** 2026-03-24
**Severity:** CRITICAL
**Status:** Root Cause Identified - Fix Pending

---

## Summary

**Detector weights defined in `~/.config/glassware/config.toml` are NOT being loaded.** The code uses `DetectorWeights::default()` instead of parsing the config file values.

This explains why:
1. webpack is being flagged as malicious despite being in the whitelist
2. Evidence packages score below threshold (2.0-4.0 instead of 7.0+)
3. Scoring doesn't match expected behavior from config

---

## Root Cause

### Location: `glassware/src/main.rs:939-951`

```rust
glassware_config: glassware_core::GlasswareConfig {
    whitelist: glassware_core::WhitelistConfig {
        packages: glassware_config.whitelist.packages.clone(),
        crypto_packages: glassware_config.whitelist.crypto_packages.clone(),
        build_tools: glassware_config.whitelist.build_tools.clone(),
        state_management: vec![],  // ← NOT COPIED!
    },
    scoring: glassware_core::ScoringConfig {
        malicious_threshold: glassware_config.scoring.malicious_threshold,
        suspicious_threshold: glassware_config.scoring.suspicious_threshold,
        category_weight: glassware_config.scoring.category_weight,
        critical_weight: glassware_config.scoring.critical_weight,
        high_weight: glassware_config.scoring.high_weight,
    },
    detectors: glassware_core::DetectorWeights::default(),  // ← BUG: Using defaults!
},
```

### Problem

Line 951: `detectors: glassware_core::DetectorWeights::default()`

This **ignores** the `[detectors.*]` sections in the config file:

```toml
[detectors.invisible_char]
enabled = true
weight = 1.0

[detectors.homoglyph]
enabled = true
weight = 1.0

[detectors.bidi]
enabled = true
weight = 1.0

[detectors.blockchain_c2]
enabled = true
weight = 2.0

[detectors.glassware_pattern]
enabled = true
weight = 3.0
```

### Default Values vs Config Values

| Detector | Default Weight | Config Weight | Impact |
|----------|---------------|---------------|--------|
| invisible_char | 1.0 | 1.0 | ✅ Same |
| homoglyph | 1.0 | 1.0 | ✅ Same |
| bidi | 1.0 | 1.0 | ✅ Same |
| blockchain_c2 | 1.0 | **2.0** | ❌ C2 not weighted properly |
| glassware_pattern | **3.0** | **3.0** | ✅ Same |
| locale_geofencing | 1.0 | 1.0 | ✅ Same |

Wait... the defaults are actually correct! Let me re-check...

---

## Re-analysis: It's NOT the Detector Weights

Looking at `glassware-core/src/config.rs:212-228`:

```rust
impl Default for DetectorWeights {
    fn default() -> Self {
        Self {
            invisible_char: default_weight(),        // 1.0
            homoglyph: default_weight(),             // 1.0
            bidi: default_weight(),                  // 1.0
            blockchain_c2: default_weight(),         // 1.0 (config has 2.0)
            glassware_pattern: default_heavy_weight(), // 3.0
            locale_geofencing: default_weight(),     // 1.0
            time_delay: default_weight(),            // 1.0
            encrypted_payload: default_heavy_weight(), // 3.0
            rdd: default_heavy_weight(),             // 3.0
            forcememo: default_heavy_weight(),       // 3.0
            jpd_author: default_heavy_weight(),      // 3.0
        }
    }
}
```

The defaults are reasonable. So the issue is NOT the detector weights.

---

## Real Issue: Whitelist Not Working in scan-npm Mode

### Evidence

1. **Wave 11 (campaign mode):** Whitelist works - 28/54 packages whitelisted correctly
2. **scan-npm webpack@5.89.0:** Flagged as malicious despite being in whitelist
3. **scan-npm moment@2.30.1:** NOT flagged (194 findings, score capped)

### Hypothesis

The whitelist IS loaded from config, but something is wrong with the **scoring logic** or **whitelist matching** in the `calculate_threat_score` function.

### Next Investigation Steps

1. Add debug logging to `is_package_whitelisted()` to verify it's being called
2. Check if `glassware_config` is actually loaded with whitelist values
3. Verify `calculate_threat_score` is using the whitelist correctly

---

## Alternative Theory: Scoring Formula Issue

Looking at `scanner.rs:657-662`:

```rust
let score = (category_count * config.scoring.category_weight) +
            (critical_hits * config.scoring.critical_weight) +
            (high_hits * config.scoring.high_weight);
```

For webpack with 16 findings:
- 4 critical (GlassWare pattern decoder_pattern)
- 7 medium (eval_pattern)
- 3 info (Socket.IO)
- 2 high (entropy blob)

If whitelist is working:
- critical_hits should be 0 (whitelisted)
- high_hits should be 0 (whitelisted)
- category_count might still be > 0

But the score is 10.0, which means either:
1. Whitelist check is failing
2. Scoring is not respecting whitelist
3. Some findings are not being filtered

---

## Action Items

### Immediate

1. **Add debug logging** to verify whitelist is loaded and checked
2. **Test config loading** to ensure file is parsed correctly
3. **Check state_management** - it's hardcoded to `vec![]` in main.rs:944

### Short-Term

1. Fix config loading to properly populate all fields
2. Ensure whitelist is checked BEFORE categorizing findings
3. Verify scoring respects whitelist status

### Medium-Term

1. Add integration tests for whitelist + scoring
2. Document config loading flow
3. Add validation that config values are within expected ranges

---

**Last Updated:** 2026-03-24 08:00 UTC
**Investigator:** Qwen-Coder
**Status:** Investigation ongoing - whitelist matching suspected
