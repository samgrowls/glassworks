# Glassworks Scoring System

**Version:** 0.32.0
**Last Updated:** March 24, 2026

---

## Overview

Glassworks uses a **signal stacking** approach to threat scoring. Multiple weak signals combine to form a strong indication of malicious behavior.

**Design Philosophy:**
- Single indicator = suspicious (could be FP)
- Multiple indicators = likely malicious (real attacks use multiple vectors)
- Certain patterns = always malicious (known C2, steganography)

---

## Score Calculation

### Base Formula

```rust
score = (category_count * category_weight) +
        (critical_hits * critical_weight) +
        (high_hits * high_weight)
```

### Default Weights (config)

```toml
[scoring]
category_weight = 3.0   # Weight per unique category
critical_weight = 4.0   # Weight per critical finding
high_weight = 2.0       # Weight per high severity finding
```

### Detector Weights

Some detectors have higher weights (heavy detectors):

| Detector | Weight | Rationale |
|----------|--------|-----------|
| RDD Attack | 3.0 | Supply chain attack indicator |
| ForceMemo | 3.0 | Python injection (GlassWorm) |
| JPD Author | 3.0 | Package metadata manipulation |
| GlasswarePattern | 3.0 | Direct GlassWare signature |
| EncryptedPayload | 3.0 | Payload obfuscation |
| Others | 1.0 | Standard weight |

---

## Signal Categories

Findings are grouped into 5 signal categories:

| Category | Detectors | Significance |
|----------|-----------|--------------|
| **obfuscation** | InvisibleChar, Homoglyph, Bidi, XorShift | Hiding something |
| **evasion** | LocaleGeofencing, TimeDelay | Avoiding detection |
| **c2** | BlockchainC2, SocketIOC2, HeaderC2, IElevatorCom | Command & Control |
| **c2_weak** | BlockchainC2 (INFO/MEDIUM) | Possible C2 (needs context) |
| **execution** | GlasswarePattern, EncryptedPayload, ApcInjection, MemexecLoader | Code execution |
| **persistence** | RDD, ForceMemo, JPD Author | Maintaining access |

### Category Counting

```rust
// Remove weak C2 if strong C2 present
if categories.contains("c2") {
    categories.remove("c2_weak");
}

let category_count = categories.len();  // Unique categories
```

---

## Category Diversity Caps

### Purpose

Prevent false positives from single-pattern detections. Legitimate libraries may have one suspicious pattern; real attacks typically have multiple.

### Caps

| Category Count | Score Cap | Classification |
|----------------|-----------|----------------|
| **1** | 4.0 | Suspicious (not malicious) |
| **2** | 7.0 | Borderline malicious |
| **3+** | None (max 10.0) | Very likely malicious |

### Example Calculations

**Example 1: Single Category (InvisibleChar only)**
```
findings: [InvisibleChar (High), InvisibleChar (High)]
categories: {obfuscation}
critical_hits: 0
high_hits: 2.0

score = (1 * 3.0) + (0 * 4.0) + (2.0 * 2.0) = 7.0
capped at: 4.0 (single category)
final score: 4.0 → SUSPICIOUS
```

**Example 2: Two Categories (InvisibleChar + TimeDelay)**
```
findings: [InvisibleChar (High), TimeDelay (Critical)]
categories: {obfuscation, evasion}
critical_hits: 1.0
high_hits: 1.0

score = (2 * 3.0) + (1.0 * 4.0) + (1.0 * 2.0) = 12.0
capped at: 7.0 (two categories)
final score: 7.0 → MALICIOUS
```

**Example 3: Three Categories (InvisibleChar + TimeDelay + BlockchainC2)**
```
findings: [InvisibleChar (High), TimeDelay (Critical), BlockchainC2 (Critical)]
categories: {obfuscation, evasion, c2}
critical_hits: 2.0
high_hits: 1.0

score = (3 * 3.0) + (2.0 * 4.0) + (1.0 * 2.0) = 19.0
no cap (3+ categories)
final score: 10.0 (capped at max) → MALICIOUS
```

---

## Scoring Exceptions (Phase 3)

### Purpose

Some attack patterns are so distinctive they should score high **regardless of category diversity**.

### Exception 1: Known C2 Indicators

**Min Score:** 8.5

**Trigger:**
```rust
finding.category == BlockchainC2
&& finding.severity == Severity::Critical
&& (finding.description.contains("Known C2") || finding.description.contains("GlassWorm"))
```

**Rationale:** Known C2 wallets/IPs are confirmed malicious infrastructure.

---

### Exception 2: Steganography (Invisible + Decoder)

**Min Score:** 8.0

**Trigger:**
```rust
finding.category == InvisibleCharacter
&& finding.severity == Severity::Critical
&& finding.description.contains("decoder")
```

**Rationale:** Invisible characters + decoder pattern is the GlassWare steganography signature.

---

### Exception 3: High Confidence Critical

**Min Score:** 7.5

**Trigger:**
```rust
finding.confidence >= 0.90 && finding.severity == Severity::Critical
```

**Rationale:** High confidence critical findings are likely real attacks even if single-vector.

---

## Thresholds

### Classification Thresholds

| Threshold | Value | Meaning |
|-----------|-------|---------|
| `malicious_threshold` | 7.0 | Score ≥7.0 = malicious |
| `suspicious_threshold` | 3.5 | Score ≥3.5 = suspicious |

### Configuration

```toml
[scoring]
malicious_threshold = 7.0
suspicious_threshold = 3.5
```

### Campaign-Specific Thresholds

Different campaigns may use different thresholds:

| Campaign | Malicious | Suspicious | Purpose |
|----------|-----------|------------|---------|
| wave6 | 4.0 | - | Calibration (low threshold) |
| wave10 | 7.0 | 4.0 | Production hunt |
| wave11 | 5.0 | 2.0 | Evidence validation (sensitive) |
| wave12 | 5.0 | 2.0 | Large-scale (5000 pkg) |

---

## Score Interpretation

| Score Range | Classification | Action |
|-------------|----------------|--------|
| **8.0-10.0** | **Confirmed Malicious** | Block, report, investigate immediately |
| **7.0-7.9** | **Likely Malicious** | Block, manual review |
| **5.0-6.9** | **Borderline** | Manual review required |
| **3.5-4.9** | **Suspicious** | Monitor, investigate if other signals |
| **0.0-3.4** | **Clean** | No action |

---

## Tuning Guidelines

### Adjusting Category Weights

**Increase `category_weight`:**
- Rewards multi-vector detection
- Better for catching sophisticated attacks
- May miss single-vector attacks

**Decrease `category_weight`:**
- More sensitive to single indicators
- Catches simpler attacks
- May increase false positives

### Adjusting Caps

**Increase single-category cap (e.g., 4.0 → 5.0):**
- Catches single-vector attacks
- Risk: More false positives

**Decrease single-category cap (e.g., 4.0 → 3.0):**
- Fewer false positives
- Risk: Miss single-vector attacks

### Adjusting Exceptions

**Add new exception:**
- For newly discovered attack patterns
- Must have very high confidence indicator

**Remove exception:**
- If causing too many false positives
- If pattern found in legitimate code

---

## Comparison: Before vs After Phase 3

### Before Phase 3 (v0.31.0)

| Scenario | Score | Classification |
|----------|-------|----------------|
| Known C2 wallet only | 4.0 (capped) | Suspicious ❌ |
| Steganography only | 4.0 (capped) | Suspicious ❌ |
| Multi-vector attack | 8.0+ | Malicious ✅ |

### After Phase 3 (v0.32.0)

| Scenario | Score | Classification |
|----------|-------|----------------|
| Known C2 wallet only | 8.5+ | Malicious ✅ |
| Steganography only | 8.0+ | Malicious ✅ |
| Multi-vector attack | 8.0+ | Malicious ✅ |

---

## Testing

### Score Verification

```bash
# Scan package and see score
./target/release/glassware scan-tarball evidence/package.tgz

# Expected output:
# Threat score: 8.50
# Classification: MALICIOUS
```

### Unit Tests

```rust
#[test]
fn test_known_c2_exception() {
    let findings = vec![
        Finding {
            category: DetectionCategory::BlockchainC2,
            severity: Severity::Critical,
            description: "Known C2 wallet address detected".to_string(),
            confidence: Some(0.95),
            ..
        }
    ];
    
    let score = calculate_threat_score(&findings, "test-package");
    assert!(score >= 8.5);  // Exception overrides cap
}
```

---

## References

- `glassware/src/scanner.rs::calculate_threat_score()` - Main implementation
- `glassware-core/src/config.rs::ScoringConfig` - Configuration
- `docs/DETECTION.md` - Detector reference
- PROMPT.md Phase 3 - Scoring revision specification
