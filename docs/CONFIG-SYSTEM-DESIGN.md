# GlassWorm Configuration System Design

**Date:** 2026-03-22  
**Version:** v0.11.6 (planned)  
**Status:** Design Phase

---

## Problem Statement

Current configuration issues:

1. **Inconsistent thresholds:**
   - `ScannerConfig.threat_threshold` defaults to 5.0
   - CLI `--threat-threshold` defaults to 7.0
   - No single source of truth

2. **No persistence:**
   - Configuration must be specified on every command
   - No user preferences saved between runs

3. **Limited customization:**
   - Can't adjust per-detector weights
   - Can't enable/disable specific detectors
   - Can't customize whitelist

4. **No hierarchy:**
   - No system-wide defaults
   - No project-specific overrides

---

## Design Goals

1. **Single Source of Truth:** One configuration structure used everywhere
2. **Hierarchical:** CLI > Config File > Env Vars > Defaults
3. **Persistent:** User preferences saved between runs
4. **Customizable:** Per-detector weights, thresholds, whitelists
5. **Cross-Platform:** Standard config locations on all OSes
6. **Backward Compatible:** Existing CLI flags continue to work

---

## Configuration Hierarchy

```
┌─────────────────────────────────────────────────────────────┐
│                    Configuration Priority                    │
├─────────────────────────────────────────────────────────────┤
│  1. CLI Flags (highest priority - overrides everything)     │
│     --threat-threshold, --concurrency, etc.                 │
├─────────────────────────────────────────────────────────────┤
│  2. Project Config (.glassware.toml in current directory)   │
│     Project-specific overrides                              │
├─────────────────────────────────────────────────────────────┤
│  3. User Config (~/.glassware/config.toml)                  │
│     User preferences, persisted between runs                │
├─────────────────────────────────────────────────────────────┤
│  4. Environment Variables (GLASSWARE_*)                     │
│     CI/CD friendly, container-friendly                      │
├─────────────────────────────────────────────────────────────┤
│  5. Built-in Defaults (lowest priority)                     │
│     Hardcoded sensible defaults                             │
└─────────────────────────────────────────────────────────────┘
```

---

## Configuration File Structure

### User Config (`~/.glassware/config.toml`)

```toml
# GlassWorm User Configuration

[scoring]
# Threat score thresholds
malicious_threshold = 7.0    # Score >= this is "malicious"
suspicious_threshold = 3.0   # Score >= this is "suspicious"

# Signal stacking weights
category_weight = 2.0        # Weight per category present
critical_weight = 3.0        # Weight per critical finding
high_weight = 1.5            # Weight per high severity finding

[detectors]
# Enable/disable specific detectors
[detectrors.invisible_char]
enabled = true
weight = 1.0

[detectors.homoglyph]
enabled = true
weight = 1.0

[detectors.blockchain_c2]
enabled = true
weight = 2.0  # Higher weight for C2 indicators

[detectors.glassware_pattern]
enabled = true
weight = 3.0  # Highest weight for confirmed GlassWorm patterns

# Detector-specific settings
[detectors.locale_geofencing]
enabled = true
skip_for_packages = ["moment", "date-fns", "globalize"]

[whitelist]
# Packages to never flag (locale libraries, etc.)
packages = [
    "moment",
    "moment-timezone",
    "date-fns",
    "dayjs",
    "prettier",
    "typescript",
    "eslint",
    "@babel/*"
]

# Crypto libraries (blockchain API calls are legitimate)
crypto_packages = [
    "ethers",
    "web3",
    "viem",
    "wagmi",
    "@solana/web3.js",
    "bitcoinjs-lib"
]

# Build tools (time delays are legitimate)
build_tools = [
    "webpack",
    "vite",
    "rollup",
    "esbuild",
    "parcel"
]

[performance]
# Parallel scanning
concurrency = 10

# Rate limiting
npm_rate_limit = 10.0      # requests/second
github_rate_limit = 5.0    # requests/second

# Caching
cache_enabled = true
cache_ttl_days = 7

[llm]
# LLM provider configuration
provider = "cerebras"  # or "nvidia", "groq", "openai"

[llm.cerebras]
base_url = "https://api.cerebras.ai/v1"
model = "llama-3.3-70b"
# api_key from GLASSWARE_LLM_API_KEY env var

[llm.nvidia]
base_url = "https://integrate.api.nvidia.com/v1"
models = [
    "qwen/qwen3.5-397b-a17b",
    "moonshotai/kimi-k2.5",
    "z-ai/glm5",
    "meta/llama-3.3-70b-instruct"
]
# api_key from NVIDIA_API_KEY env var

[output]
# Default output format
format = "pretty"  # or "json", "sarif"

# Default severity filter
min_severity = "low"  # or "info", "medium", "high", "critical"

# Color output
color = true
```

### Project Config (`.glassware.toml`)

```toml
# Project-specific GlassWorm configuration
# Overrides user config for this project only

[scoring]
# Stricter thresholds for production projects
malicious_threshold = 6.0

[detectors.glassware_pattern]
# Extra weight for production code
weight = 4.0

[whitelist]
# Project-specific exclusions
packages = ["@company/internal-lib"]
```

---

## Configuration Locations (Cross-Platform)

| OS | User Config | Project Config |
|----|-------------|----------------|
| **Linux** | `~/.config/glassware/config.toml` | `.glassware.toml` |
| **macOS** | `~/Library/Application Support/glassware/config.toml` | `.glassware.toml` |
| **Windows** | `%APPDATA%\glassware\config.toml` | `.glassware.toml` |

Uses the `dirs` crate for cross-platform path resolution.

---

## Implementation Plan

### Phase 1: Core Configuration System

1. **Add dependencies:**
   ```toml
   [dependencies]
   toml = "0.8"
   serde = { version = "1.0", features = ["derive"] }
   dirs = "5.0"
   ```

2. **Create `config.rs` module:**
   ```rust
   pub struct GlasswareConfig {
       pub scoring: ScoringConfig,
       pub detectors: DetectorConfig,
       pub whitelist: WhitelistConfig,
       pub performance: PerformanceConfig,
       pub llm: LlmConfig,
       pub output: OutputConfig,
   }
   
   impl GlasswareConfig {
       /// Load configuration with hierarchy:
       /// CLI > Project > User > Env > Defaults
       pub fn load(cli_overrides: Option<CliOverrides>) -> Result<Self>;
       
       /// Save user configuration
       pub fn save_user_config(&self) -> Result<()>;
   }
   ```

3. **Update ScannerConfig to use GlasswareConfig:**
   ```rust
   pub struct ScannerConfig {
       pub threat_threshold: f32,  // From config.scoring.malicious_threshold
       // ... other fields
   }
   
   impl From<&GlasswareConfig> for ScannerConfig {
       fn from(config: &GlasswareConfig) -> Self {
           Self {
               threat_threshold: config.scoring.malicious_threshold,
               // ...
           }
       }
   }
   ```

### Phase 2: CLI Integration

1. **Add config subcommand:**
   ```bash
   glassware config init          # Create default config file
   glassware config show          # Show current config
   glassware config edit          # Open config in editor
   glassware config validate      # Validate config syntax
   ```

2. **CLI flags override config:**
   ```rust
   // In main.rs
   let config = GlasswareConfig::load(Some(cli.overrides))?;
   let scanner = Scanner::new(config.into());
   ```

### Phase 3: Detector Weights

1. **Update threat score calculation:**
   ```rust
   fn calculate_threat_score(
       findings: &[Finding],
       config: &DetectorConfig
   ) -> f32 {
       let mut score = 0.0;
       
       for finding in findings {
           let detector_weight = config.get_weight(&finding.category);
           let severity_weight = match finding.severity {
               Severity::Critical => config.scoring.critical_weight,
               Severity::High => config.scoring.high_weight,
               _ => 1.0,
           };
           
           score += detector_weight * severity_weight;
       }
       
       score.min(10.0)
   }
   ```

### Phase 4: Testing

1. **Unit tests for config loading**
2. **Integration tests for hierarchy**
3. **Smoke tests with custom config**

---

## Migration Path

### Current Behavior (v0.11.5)

```bash
# Hardcoded defaults
glassware-orchestrator scan-npm pkg1 pkg2
# Uses: threat_threshold = 5.0 (ScannerConfig) or 7.0 (CLI)
```

### New Behavior (v0.11.6)

```bash
# First run - creates default config
glassware config init
# Creates ~/.config/glassware/config.toml with defaults

# Uses config file thresholds
glassware-orchestrator scan-npm pkg1 pkg2
# Uses: threat_threshold from config (default 7.0)

# Override with CLI
glassware-orchestrator --threat-threshold 6.0 scan-npm pkg1 pkg2
# Uses: 6.0 (CLI overrides config)
```

---

## Benefits

1. **Consistency:** Single source of truth for all thresholds
2. **Flexibility:** Per-detector weights, custom whitelists
3. **Persistence:** Configuration saved between runs
4. **CI/CD Friendly:** Environment variables still work
5. **Project-Specific:** Different configs for different projects
6. **User-Friendly:** `glassware config init` creates sensible defaults

---

## Open Questions

1. **Config file format:** TOML vs YAML vs JSON?
   - TOML: Rust-native, human-readable ✓
   - YAML: More common for configs
   - JSON: Machine-friendly, less human-readable

2. **Config file name:** `.glassware.toml` vs `glassware.toml`?
   - `.glassware.toml`: Hidden file, consistent with `.rustfmt.toml`
   - `glassware.toml`: Visible, consistent with `Cargo.toml`

3. **Detector weights:** Expose to users or keep internal?
   - Expose: Maximum flexibility
   - Internal: Simpler, less chance of misconfiguration

4. **Config validation:** Strict or lenient?
   - Strict: Error on unknown fields
   - Lenient: Warn on unknown fields, ignore them

---

## Recommendation

1. **Use TOML** - Rust-native, human-readable
2. **Use `.glassware.toml`** - Consistent with Rust tooling
3. **Expose detector weights** - Power users can tune
4. **Strict validation** - Catch typos early

---

**Next Steps:**

1. Review and approve this design
2. Implement Phase 1 (Core Configuration System)
3. Update smoke tests to use config
4. Document configuration options
