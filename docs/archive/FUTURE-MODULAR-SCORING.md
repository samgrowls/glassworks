# Modular Scoring Functions - Future Design

**Date:** 2026-03-24
**Status:** 📋 Design Proposal
**Priority:** Medium-Term Enhancement

---

## Motivation

Current scoring system uses a fixed linear formula:

```rust
score = (categories × category_weight) + 
        (critical_hits × critical_weight) + 
        (high_hits × high_weight)
```

This works well for general use, but different campaigns may benefit from different scoring strategies:

- **Conservative campaigns** (production use): Minimize false positives
- **Aggressive campaigns** (research/hunting): Catch more potential threats
- **Evidence-focused campaigns**: Weight obfuscation patterns heavily
- **Compliance campaigns**: Follow specific security frameworks

---

## Design Goals

1. **Configurable:** Scoring formula selectable via TOML config
2. **Extensible:** Easy to add new formulas without code changes
3. **Transparent:** Users can see exactly how scores are calculated
4. **Safe:** Sensible defaults, validation of custom formulas
5. **Performant:** Minimal overhead vs hardcoded formula

---

## Proposed Implementation

### Option 1: Predefined Formula Presets (Recommended)

Define several scoring presets in config:

```toml
[scoring]
# Choose a preset: "conservative", "balanced", "aggressive", "evidence"
preset = "balanced"

# Or override specific weights
category_weight = 2.0
critical_weight = 3.0
high_weight = 1.5
```

**Preset Definitions:**

| Preset | category_weight | critical_weight | high_weight | Use Case |
|--------|----------------|-----------------|-------------|----------|
| `conservative` | 1.5 | 2.5 | 1.0 | Production, minimize FPs |
| `balanced` | 2.0 | 3.0 | 1.5 | Default, general use |
| `aggressive` | 2.5 | 4.0 | 2.0 | Security research, hunting |
| `evidence` | 3.0 | 5.0 | 2.5 | Evidence detection, obfuscation focus |
| `compliance` | 2.0 | 3.5 | 1.5 | Regulatory compliance scanning |

### Option 2: Custom Formula Engine (Advanced)

Allow users to define custom scoring formulas:

```toml
[scoring]
# Custom formula using Lua scripting
custom_script = "scoring/my_formula.lua"

# Or simple expression (safer but less flexible)
# formula = "categories * 2.0 + critical * 3.0 + high * 1.5 + (categories * critical) * 0.5"
```

**Lua Script Example:**
```lua
-- scoring/aggressive.lua
function calculate_score(categories, critical_hits, high_hits, medium_hits)
    -- Base score
    local score = (categories * 2.5) + (critical_hits * 4.0) + (high_hits * 2.0)
    
    -- Bonus for multiple categories (diversity penalty)
    if categories >= 3 then
        score = score * 1.2
    end
    
    -- Cap at 10.0
    return math.min(score, 10.0)
end
```

### Option 3: Hybrid Approach (Best of Both)

Combine presets with limited customization:

```toml
[scoring]
# Start with a preset
preset = "balanced"

# Apply multipliers or adjustments
critical_multiplier = 1.2  # 20% more weight to critical findings
category_diversity_bonus = true  # Add bonus for 3+ categories

# Custom thresholds
malicious_threshold = 6.0  # Lower than default 7.0
suspicious_threshold = 3.0
```

---

## Implementation Plan

### Phase 1: Preset System (1-2 days)

**Files to modify:**
- `glassware-core/src/config.rs` - Add `ScoringPreset` enum
- `glassware-core/src/scorer.rs` - Implement preset scoring
- `glassware/src/config.rs` - Parse preset from TOML

**Code Structure:**
```rust
// glassware-core/src/config.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScoringPreset {
    Conservative,
    Balanced,
    Aggressive,
    Evidence,
    Compliance,
}

impl ScoringPreset {
    pub fn default_weights(&self) -> ScoringWeights {
        match self {
            ScoringPreset::Conservative => ScoringWeights {
                category_weight: 1.5,
                critical_weight: 2.5,
                high_weight: 1.0,
            },
            ScoringPreset::Balanced => ScoringWeights {
                category_weight: 2.0,
                critical_weight: 3.0,
                high_weight: 1.5,
            },
            // ... etc
        }
    }
}
```

### Phase 2: Formula Validation (1 day)

Add validation to ensure scoring produces reasonable results:

```rust
pub fn validate(&self) -> Result<(), ScoringError> {
    // Test with sample inputs
    let test_score = self.calculate(3, 2, 5, 10);
    
    if test_score < 0.0 || test_score > 15.0 {
        return Err(ScoringError::UnreasonableScore(test_score));
    }
    
    // Check weights are non-negative
    if self.category_weight < 0.0 || self.critical_weight < 0.0 {
        return Err(ScoringError::NegativeWeight);
    }
    
    Ok(())
}
```

### Phase 3: Documentation (1 day)

- Document each preset and its use cases
- Provide examples of when to use each preset
- Create scoring tuning guide
- Add preset selection to campaign user guide

---

## Configuration Examples

### Example 1: Production Scanning (Conservative)

```toml
# campaigns/production-scan.toml
[settings.scoring]
preset = "conservative"
malicious_threshold = 8.0  # Higher threshold, fewer FPs
suspicious_threshold = 4.0

[settings.whitelist]
# Extensive whitelist for production
packages = ["lodash", "express", "react", ...]
```

**Expected behavior:**
- Only flag high-confidence malicious packages
- Minimal false positives
- May miss some sophisticated attacks

### Example 2: Security Research (Aggressive)

```toml
# campaigns/research-hunt.toml
[settings.scoring]
preset = "aggressive"
malicious_threshold = 5.0  # Lower threshold, catch more
suspicious_threshold = 2.0

[settings.reporting]
include_clean_summary = false  # Focus on findings
```

**Expected behavior:**
- Flag more packages for review
- Higher false positive rate acceptable
- Catch sophisticated attacks

### Example 3: Evidence Detection

```toml
# campaigns/evidence-validation.toml
[settings.scoring]
preset = "evidence"
malicious_threshold = 7.0
category_diversity_bonus = true  # Bonus for multiple attack categories

[settings.detectors.glassware_pattern]
weight = 5.0  # Extra weight for confirmed patterns
```

**Expected behavior:**
- Heavy weighting for obfuscation patterns
- Catch evidence archive packages
- Focus on GlassWare-specific attacks

---

## Benefits

### For Users

1. **Simpler configuration:** Choose preset instead of tuning weights
2. **Better outcomes:** Presets optimized for specific use cases
3. **Transparency:** Clear documentation of what each preset does
4. **Flexibility:** Override preset weights if needed

### For Developers

1. **Easier tuning:** Adjust presets instead of code
2. **Testing:** Validate presets with known datasets
3. **Extensibility:** Add new presets without breaking changes
4. **Debugging:** Easier to reproduce scoring issues

---

## Migration Path

### Current Users

No breaking changes - existing configs continue to work:

```toml
# Old style (still works)
[settings.scoring]
category_weight = 2.0
critical_weight = 3.0
high_weight = 1.5

# New style (recommended)
[settings.scoring]
preset = "balanced"
# Optional overrides
# category_weight = 2.5  # Override preset default
```

### Default Behavior

If no preset specified, use `balanced` preset (matches current defaults).

---

## Testing Strategy

### Unit Tests

```rust
#[test]
fn test_conservative_preset() {
    let preset = ScoringPreset::Conservative;
    let weights = preset.default_weights();
    
    // Test with sample findings
    let score = weights.calculate(2, 1, 3, 5);
    assert!(score < 7.0);  // Should not flag as malicious
}

#[test]
fn test_aggressive_preset() {
    let preset = ScoringPreset::Aggressive;
    let weights = preset.default_weights();
    
    let score = weights.calculate(2, 1, 3, 5);
    assert!(score >= 5.0);  // Should flag as suspicious
}
```

### Integration Tests

1. Run Wave 8/9/10 with each preset
2. Compare false positive rates
3. Compare true positive rates (evidence detection)
4. Document results for users

---

## Future Enhancements

### Machine Learning-Based Scoring

Train a model on labeled dataset to optimize weights:

```toml
[scoring]
# Use ML-optimized weights from training
ml_model = "v1-production"
```

### Per-Detector Scoring

Allow detectors to have custom scoring logic:

```toml
[scoring.detector_scores]
invisible_char = { weight = 1.0, cap = 3.0 }  # Cap at 3.0 for this detector
glassware_pattern = { weight = 5.0, min_for_malicious = 2.0 }
```

### Contextual Scoring

Adjust scoring based on package context:

```toml
[scoring.context]
# Higher thresholds for popular packages
popular_package_threshold_bonus = 2.0

# Lower thresholds for high-risk categories
high_risk_categories = ["sms", "auth", "crypto"]
high_risk_threshold_penalty = -1.0
```

---

## Recommendation

**Implement Option 1 (Preset System) first:**
- Lowest complexity
- Immediate value for users
- Foundation for future enhancements

**Then consider Option 3 (Hybrid) if users need more customization.**

**Option 2 (Custom Formula) is probably overkill for most users and adds significant complexity.**

---

**Last Updated:** 2026-03-24
**Author:** Qwen-Coder
**Status:** Ready for implementation when prioritized
