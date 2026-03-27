# Modular Conditional Scoring Design

**Date:** 2026-03-26
**Status:** DESIGN PROPOSAL
**Goal:** Flexible, configurable scoring with configurable detector tiers and score spread

---

## Design Goals

1. **Modular:** Detector tiers configurable per wave via TOML
2. **Flexible:** Different waves can prioritize different detectors
3. **Calibrated:** Real attacks score much higher than FPs (spread: 0-30+)
4. **Backward Compatible:** Default config maintains current behavior

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                   ScoringConfig (per wave)                   │
│  ┌─────────────────────────────────────────────────────┐    │
│  │ detector_tiers: [                                    │    │
│  │   { tier: 1, detectors: ["blockchain_c2"], threshold: 0.0 }, │
│  │   { tier: 2, detectors: ["header_c2"], threshold: 3.0 },     │
│  │   { tier: 3, detectors: ["invisible_char"], threshold: 5.0 },│
│  │ ]                                                        │    │
│  │                                                          │    │
│  │ score_weights: {                                         │    │
│  │   blockchain_c2: 10.0,                                   │    │
│  │   header_c2: 8.0,                                        │    │
│  │   invisible_char: 5.0,                                   │    │
│  │ }                                                        │    │
│  │                                                          │    │
│  │ conditional_rules: [                                     │    │
│  │   { if: "tier_sum >= 5.0", then: "enable_tier_3" },      │    │
│  │ ]                                                        │    │
│  └─────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│                   ScoringEngine                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │  Tier 1      │  │  Tier 2      │  │  Tier 3      │      │
│  │  (C2)        │→ │  (Header)    │→ │  (Invisible) │      │
│  │  threshold:0 │  │  threshold:3 │  │  threshold:5 │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
│         ↓                  ↓                  ↓              │
│    weight: 10.0       weight: 8.0       weight: 5.0         │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│                   Final Score (0-30+)                        │
│  Clean package: 0-3                                         │
│  Suspicious: 3-8                                            │
│  Likely malicious: 8-15                                     │
│  Confirmed malicious: 15-30+                                │
└─────────────────────────────────────────────────────────────┘
```

---

## Configuration Schema (TOML)

### Default Config (Current Behavior)

```toml
[settings.scoring]
malicious_threshold = 7.0
suspicious_threshold = 4.0

# Default: all detectors run independently (current behavior)
[settings.scoring.tier_config]
mode = "independent"  # All detectors run, scores summed

# Detector weights (contribute to score spread)
[settings.scoring.weights]
invisible_char = 5.0
homoglyph = 3.0
glassware_pattern = 8.0
blockchain_c2 = 10.0
header_c2 = 8.0
exfil_schema = 6.0
locale_geofencing = 7.0
time_delay = 7.0
```

### Wave-Specific Config (Example: Blockchain Hunt)

```toml
[settings.scoring]
malicious_threshold = 10.0  # Higher threshold for aggressive scanning
suspicious_threshold = 5.0

# Tiered scoring with C2 as priority
[settings.scoring.tier_config]
mode = "tiered"

[[settings.scoring.tiers]]
tier = 1
detectors = ["blockchain_c2", "header_c2"]
threshold = 0.0  # Always run
weight_multiplier = 1.5  # Boost C2 scores for this wave

[[settings.scoring.tiers]]
tier = 2
detectors = ["invisible_char", "glassware_pattern", "obfuscation"]
threshold = 5.0  # Only run if tier 1 score >= 5.0
weight_multiplier = 1.0

[[settings.scoring.tiers]]
tier = 3
detectors = ["locale_geofencing", "time_delay", "sandbox_evasion"]
threshold = 10.0  # Only run if tier 1+2 score >= 10.0
weight_multiplier = 0.8

# Conditional rules
[[settings.scoring.conditional_rules]]
name = "c2_confirms_invisible"
description = "If invisible chars found, C2 score is doubled"
condition = "invisible_char.score > 0 AND blockchain_c2.score > 0"
action = "blockchain_c2.weight *= 2.0"

[[settings.scoring.conditional_rules]]
name = "glassworm_signature"
description = "Invisible + C2 + polling = confirmed GlassWorm"
condition = "invisible_char.score > 0 AND blockchain_c2.score > 0 AND blockchain_polling.score > 0"
action = "final_score = 30.0"  # Maximum score
```

### Wave-Specific Config (Example: Steganography Hunt)

```toml
[settings.scoring]
malicious_threshold = 8.0
suspicious_threshold = 4.0

# Tiered scoring with steganography as priority
[settings.scoring.tier_config]
mode = "tiered"

[[settings.scoring.tiers]]
tier = 1
detectors = ["invisible_char", "glassware_pattern", "obfuscation"]
threshold = 0.0  # Always run
weight_multiplier = 1.5  # Boost stego scores

[[settings.scoring.tiers]]
tier = 2
detectors = ["blockchain_c2", "header_c2"]
threshold = 5.0  # Only run if stego detected
weight_multiplier = 1.0

[[settings.scoring.tiers]]
tier = 3
detectors = ["locale_geofencing", "time_delay"]
threshold = 10.0
weight_multiplier = 0.8
```

---

## Score Spread Design

### Current Problem
- Clean packages: 0-10 (overlap with malicious)
- Malicious packages: 7-10 (no spread)
- **Result:** Hard to distinguish FPs from real attacks

### Target Score Distribution

| Category | Score Range | Description |
|----------|-------------|-------------|
| Clean | 0-3 | No suspicious patterns |
| Low suspicion | 3-6 | Single detector hit |
| Medium suspicion | 6-10 | Multiple detectors agree |
| Likely malicious | 10-15 | Strong indicators |
| Confirmed malicious | 15-25 | Multiple strong indicators |
| GlassWorm signature | 25-30+ | Definitive attack pattern |

### Weight Calibration

| Detector | Base Weight | Max Contribution | Notes |
|----------|-------------|------------------|-------|
| invisible_char | 5.0 | 15.0 | Scales with count |
| homoglyph | 3.0 | 6.0 | Scales with count |
| glassware_pattern | 8.0 | 16.0 | Stego + obfuscation |
| blockchain_c2 | 10.0 | 10.0 | Known wallets only |
| header_c2 | 8.0 | 8.0 | Header extraction |
| exfil_schema | 6.0 | 12.0 | Scales with indicators |
| locale_geofencing | 7.0 | 7.0 | Binary (yes/no) |
| time_delay | 7.0 | 7.0 | Binary (yes/no) |
| blockchain_polling | 10.0 | 10.0 | GlassWorm signature |

**Max theoretical score:** ~95 (all detectors maxed)
**Typical clean package:** 0-3
**Typical malicious package:** 15-30

---

## Implementation Plan

### Phase 1: ScoringConfig Schema

**File:** `glassware/src/scoring_config.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringConfig {
    pub malicious_threshold: f32,
    pub suspicious_threshold: f32,
    
    // Tier configuration
    #[serde(default)]
    pub tier_config: TierConfig,
    
    // Detector weights
    #[serde(default)]
    pub weights: DetectorWeights,
    
    // Conditional rules
    #[serde(default)]
    pub conditional_rules: Vec<ConditionalRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierConfig {
    #[serde(default = "default_mode")]
    pub mode: TierMode,  // "independent" or "tiered"
    
    #[serde(default)]
    pub tiers: Vec<TierDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierDefinition {
    pub tier: u8,
    pub detectors: Vec<String>,
    pub threshold: f32,  // Min score from previous tiers to run this tier
    #[serde(default = "default_multiplier")]
    pub weight_multiplier: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectorWeights {
    #[serde(default = "default_invisible")]
    pub invisible_char: f32,
    #[serde(default = "default_blockchain")]
    pub blockchain_c2: f32,
    // ... etc
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionalRule {
    pub name: String,
    pub description: String,
    pub condition: String,  // Expression language
    pub action: String,     // Action to take
}
```

### Phase 2: ScoringEngine Refactor

**File:** `glassware/src/scanner.rs` (scoring logic)

```rust
pub struct ScoringEngine {
    config: ScoringConfig,
    tier_detectors: HashMap<u8, Vec<Box<dyn Detector>>>,
}

impl ScoringEngine {
    pub fn calculate_score(&self, findings: &[Finding]) -> f32 {
        match self.config.tier_config.mode {
            TierMode::Independent => self.calculate_independent(findings),
            TierMode::Tiered => self.calculate_tiered(findings),
        }
    }
    
    fn calculate_tiered(&self, findings: &[Finding]) -> f32 {
        let mut tier_sum = 0.0;
        
        for tier_def in &self.config.tier_config.tiers {
            // Check if threshold met
            if tier_sum < tier_def.threshold {
                continue;  // Skip this tier
            }
            
            // Run detectors in this tier
            let tier_score = self.run_tier_detectors(tier_def, findings);
            tier_sum += tier_score * tier_def.weight_multiplier;
        }
        
        // Apply conditional rules
        for rule in &self.config.conditional_rules {
            if self.evaluate_condition(rule, &tier_sum) {
                self.apply_action(rule, &mut tier_sum);
            }
        }
        
        tier_sum
    }
}
```

### Phase 3: Update Wave Configs

**File:** `campaigns/wave14-long-horizon-starter.toml`

```toml
[settings.scoring]
malicious_threshold = 10.0  # Higher for long-horizon
suspicious_threshold = 5.0

[settings.scoring.tier_config]
mode = "tiered"

[[settings.scoring.tiers]]
tier = 1
detectors = ["invisible_char", "glassware_pattern", "obfuscation"]
threshold = 0.0
weight_multiplier = 1.5

[[settings.scoring.tiers]]
tier = 2
detectors = ["blockchain_c2", "header_c2"]
threshold = 5.0
weight_multiplier = 1.0

[[settings.scoring.tiers]]
tier = 3
detectors = ["locale_geofencing", "time_delay", "sandbox_evasion"]
threshold = 10.0
weight_multiplier = 0.8

[settings.scoring.weights]
invisible_char = 6.0
glassware_pattern = 10.0
blockchain_c2 = 8.0  # Reduced weight
header_c2 = 6.0      # Reduced weight
```

---

## Expression Language for Conditional Rules

### Simple Conditions

```toml
[[settings.scoring.conditional_rules]]
name = "c2_confirms_invisible"
condition = "invisible_char.score > 0 AND blockchain_c2.score > 0"
action = "blockchain_c2.weight *= 2.0"

[[settings.scoring.conditional_rules]]
name = "glassworm_signature"
condition = "invisible_char.count >= 10 AND blockchain_polling.detected"
action = "final_score = 30.0"
```

### Condition Syntax

```
<detector>.<field> <operator> <value>
<detector>.<field> <operator> <detector>.<field>
<condition> AND <condition>
<condition> OR <condition>
NOT <condition>
```

### Available Fields

| Field | Type | Description |
|-------|------|-------------|
| `<detector>.score` | f32 | Detector's contribution to score |
| `<detector>.count` | u32 | Number of findings from detector |
| `<detector>.detected` | bool | True if detector found anything |
| `final_score` | f32 | Running total score |

---

## Score Spread Examples

### Clean Package (express@4.19.2)

| Detector | Findings | Weight | Contribution |
|----------|----------|--------|--------------|
| invisible_char | 0 | 6.0 | 0.0 |
| glassware_pattern | 0 | 10.0 | 0.0 |
| blockchain_c2 | 0 | 8.0 | 0.0 |
| **Total** | | | **0.0** ✅ |

### False Positive (pre-fix @solana/web3.js)

| Detector | Findings | Weight | Contribution |
|----------|----------|--------|--------------|
| blockchain_c2 | 245 | 10.0 | 10.0 ❌ |
| **Total** | | | **10.0** (FP) |

### False Positive (post-fix @solana/web3.js)

| Detector | Findings | Weight | Contribution |
|----------|----------|--------|--------------|
| blockchain_c2 | 0 | 8.0 | 0.0 ✅ |
| **Total** | | | **0.0** (clean) |

### Real Attack (iflow-mcp)

| Detector | Findings | Weight | Contribution |
|----------|----------|--------|--------------|
| invisible_char | 50 | 6.0 | 12.0 |
| glassware_pattern | 3 | 10.0 | 15.0 |
| blockchain_c2 | 1 | 8.0 | 8.0 |
| **Total** | | | **35.0** ✅ |

### Real Attack (react-native-country-select)

| Detector | Findings | Weight | Contribution |
|----------|----------|--------|--------------|
| glassware_pattern (obfuscation) | 5 | 10.0 | 12.0 |
| header_c2 | 2 | 6.0 | 6.0 |
| **Total** | | | **18.0** ✅ |

---

## Migration Path

### Step 1: Add Config Schema (Backward Compatible)
- New fields are optional with defaults
- Existing configs work unchanged

### Step 2: Default Tiered Config
- Sensible defaults for most waves
- Tier 1: Primary signals (invisible, obfuscation)
- Tier 2: Secondary (C2, header)
- Tier 3: Behavioral (geofencing, time delay)

### Step 3: Update Wave Configs
- wave14: Use tiered config with stego priority
- wave15: Use tiered config with balanced approach
- Future waves: Customize per use case

### Step 4: Calibrate Weights
- Run on evidence packages
- Adjust weights to achieve target spread
- Validate on clean baseline

---

## Success Criteria

### Score Spread
- [ ] Clean packages: 0-3
- [ ] Suspicious: 3-8
- [ ] Likely malicious: 8-15
- [ ] Confirmed malicious: 15-30+

### Flexibility
- [ ] TOML can override tier order
- [ ] TOML can adjust weights
- [ ] TOML can add conditional rules

### Backward Compatibility
- [ ] Existing configs work unchanged
- [ ] Default behavior matches current (until updated)

---

**Status:** DESIGN READY FOR REVIEW
**Next:** Implement Phase 1 (ScoringConfig schema)
