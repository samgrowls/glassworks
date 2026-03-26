# Whitelist Integration - Implementation Plan

**Date:** March 23, 2026
**Goal:** Proper whitelist integration for public open source release
**Status:** 📋 Planning Complete - Ready to Implement

---

## Architecture Overview

### Config Hierarchy

```
┌─────────────────────────────────────────────────────────────┐
│  Campaign TOML (campaigns/wave9.toml)                       │
│  [settings.whitelist]                                        │
│  packages = ["moment", "lodash", ...]                       │
│  crypto_packages = ["node-forge", "crypto-js", ...]         │
│  build_tools = ["webpack", "vite", ...]                     │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│  CampaignSettings (glassware/src/campaign/config.rs)        │
│  pub whitelist: WhitelistConfig                             │
│  pub scoring: ScoringConfig                                 │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│  ScannerConfig (glassware/src/scanner.rs)                   │
│  pub glassware_config: glassware_core::GlasswareConfig      │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│  GlasswareConfig (glassware-core/src/config.rs)             │
│  pub whitelist: WhitelistConfig                             │
│  pub scoring: ScoringConfig                                 │
│  pub detectors: DetectorWeights                             │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│  ScanEngine (glassware-core/src/engine.rs)                  │
│  Checks whitelist during scanning                           │
│  Applies scoring weights                                    │
└─────────────────────────────────────────────────────────────┘
```

---

## Implementation Steps

### Phase 1: Core Config Types (glassware-core)

**Files:**
- `glassware-core/src/config.rs` - Add config types
- `glassware-core/src/lib.rs` - Export types
- `glassware-core/src/engine.rs` - Use whitelist during scan
- `glassware-core/src/scanner.rs` - Apply whitelist in scan logic

**Types to Add:**
```rust
// glassware-core/src/config.rs
pub struct GlasswareConfig {
    pub whitelist: WhitelistConfig,
    pub scoring: ScoringConfig,
    pub detectors: DetectorWeights,
}

pub struct WhitelistConfig {
    pub packages: Vec<String>,
    pub crypto_packages: Vec<String>,
    pub build_tools: Vec<String>,
    pub state_management: Vec<String>,
}

pub struct ScoringConfig {
    pub malicious_threshold: f32,
    pub suspicious_threshold: f32,
    pub category_weight: f32,
    pub critical_weight: f32,
    pub high_weight: f32,
}

pub struct DetectorWeights {
    pub invisible_char: f32,
    pub homoglyph: f32,
    pub bidi: f32,
    pub blockchain_c2: f32,
    pub glassware_pattern: f32,
}
```

**Implementation:**
- [ ] Add types to `config.rs`
- [ ] Implement `Default` traits
- [ ] Add `serde` derives (with `#[cfg(feature = "serde")]`)
- [ ] Export from `lib.rs`
- [ ] Update `engine.rs` to check whitelist
- [ ] Update `scanner.rs` to apply whitelist

---

### Phase 2: Campaign Config (glassware)

**Files:**
- `glassware/src/campaign/config.rs` - CampaignSettings with whitelist
- `glassware/src/campaign/wave.rs` - Pass settings to scanner
- `glassware/src/campaign/executor.rs` - Clone settings for waves

**Changes:**
```rust
// glassware/src/campaign/config.rs
pub struct CampaignSettings {
    // ... existing fields
    pub whitelist: WhitelistConfig,
    pub scoring: ScoringConfig,
}
```

**Implementation:**
- [ ] Add fields to `CampaignSettings`
- [ ] Implement `Default` with empty whitelist
- [ ] Add `serde` derives for TOML parsing
- [ ] Update wave executor to use settings

---

### Phase 3: Scanner Integration (glassware)

**Files:**
- `glassware/src/scanner.rs` - Use glassware_core::GlasswareConfig
- `glassware/src/main.rs` - Convert CLI config to core config

**Changes:**
```rust
// glassware/src/scanner.rs
pub struct ScannerConfig {
    // ... existing fields
    pub glassware_config: glassware_core::GlasswareConfig,
}
```

**Implementation:**
- [ ] Change `glassware_config` type to `glassware_core::GlasswareConfig`
- [ ] Update `Default` impl
- [ ] Update `From<crate::config::GlasswareConfig>` conversion
- [ ] Update `main.rs` to convert properly

---

### Phase 4: CLI Config (glassware)

**Files:**
- `glassware/src/config.rs` - Local GlasswareConfig
- `glassware/src/main.rs` - Load and convert config

**Changes:**
```rust
// glassware/src/config.rs
pub struct GlasswareConfig {
    pub whitelist: WhitelistConfig,
    pub scoring: ScoringConfig,
    pub performance: PerformanceConfig,
    // ... existing fields
}
```

**Implementation:**
- [ ] Add whitelist/scoring to local config
- [ ] Update `load()` to parse from TOML
- [ ] Update conversion to `glassware_core::GlasswareConfig`
- [ ] Update `main.rs` to use new fields

---

### Phase 5: Testing & Validation

**Tests:**
- [ ] Unit test: Whitelist prevents flagging
- [ ] Unit test: Non-whitelisted packages still flagged
- [ ] Integration test: Wave 8 with whitelist
- [ ] Integration test: Known malicious still detected

**Validation:**
- [ ] Re-run Wave 8 - expect 0 false positives
- [ ] Verify node-forge NOT flagged
- [ ] Verify underscore NOT flagged
- [ ] Verify webpack NOT flagged
- [ ] Verify crypto-js NOT flagged
- [ ] Verify known malicious STILL flagged

---

### Phase 6: Documentation

**Documentation:**
- [ ] Document `WhitelistConfig` fields
- [ ] Document `ScoringConfig` thresholds
- [ ] Add example TOML config
- [ ] Update `CAMPAIGN-USER-GUIDE.md`
- [ ] Add whitelist section to README

**Example Config:**
```toml
[settings.whitelist]
# i18n libraries (naturally contain Unicode)
packages = [
    "moment", "moment-timezone",
    "lodash", "underscore",
    "i18next", "react-intl",
]

# Crypto libraries (legitimate API usage)
crypto_packages = [
    "node-forge", "crypto-js",
    "ethers", "web3",
]

# Build tools (complex patterns are legitimate)
build_tools = [
    "webpack", "vite", "rollup",
]

[settings.scoring]
malicious_threshold = 7.0
suspicious_threshold = 4.0
```

---

## Code Quality Standards

### Documentation

Every public type and function should have:
- `///` doc comments
- Example usage where helpful
- Explanation of fields

Example:
```rust
/// Package whitelist configuration.
///
/// Packages listed here are never flagged as malicious,
/// regardless of findings. This prevents false positives
/// for known legitimate packages like i18n libraries,
/// crypto libraries, and build tools.
///
/// # Example
///
/// ```toml
/// [settings.whitelist]
/// packages = ["moment", "lodash"]
/// crypto_packages = ["node-forge", "crypto-js"]
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhitelistConfig {
    // ...
}
```

### Error Handling

- Use `thiserror` for error types
- Provide helpful error messages
- Log at appropriate levels (info, warn, error)

### Testing

- Unit tests for config parsing
- Unit tests for whitelist logic
- Integration tests for full campaign runs

---

## Risk Mitigation

### Potential Issues

| Issue | Mitigation |
|-------|------------|
| Breaking changes | Keep existing APIs, add new fields as optional |
| Config parsing errors | Provide clear error messages with file/line |
| Performance impact | Whitelist check is O(n) but n is small (<50) |
| Missing malicious | Test with known malicious packages |

### Rollback Plan

If issues arise:
1. Revert to previous commit
2. Run Wave 8 without whitelist
3. Debug in separate branch
4. Re-apply when fixed

---

## Timeline

| Phase | Estimated Time | Dependencies |
|-------|---------------|--------------|
| Phase 1: Core Config | 1-2 hours | None |
| Phase 2: Campaign Config | 1 hour | Phase 1 |
| Phase 3: Scanner Integration | 1-2 hours | Phase 1, 2 |
| Phase 4: CLI Config | 1 hour | Phase 3 |
| Phase 5: Testing | 1-2 hours | Phase 4 |
| Phase 6: Documentation | 1 hour | Phase 5 |
| **Total** | **6-9 hours** | |

**Suggested Schedule:**
- **Today:** Phases 1-3 (Core + Campaign + Scanner)
- **Tomorrow:** Phases 4-6 (CLI + Testing + Docs)
- **Day 3:** Wave 9 execution with whitelist

---

## Success Criteria

### Functional
- [ ] Whitelist prevents false positives
- [ ] Malicious packages still detected
- [ ] Config loads from TOML without errors
- [ ] All tests pass

### Code Quality
- [ ] All public types documented
- [ ] Error messages are helpful
- [ ] Code follows Rust idioms
- [ ] No clippy warnings

### Performance
- [ ] No significant slowdown (<10% overhead)
- [ ] Memory usage unchanged
- [ ] Scan speed remains ~0.18s/package

---

## Next Action

**Start Phase 1:** Implement core config types in `glassware-core/src/config.rs`

**Files to modify first:**
1. `glassware-core/src/config.rs` - Add types
2. `glassware-core/src/lib.rs` - Export types
3. `glassware-core/src/engine.rs` - Use whitelist

Let's begin!
