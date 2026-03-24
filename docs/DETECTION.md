# Glassworks Detection Documentation

**Version:** 0.32.0
**Last Updated:** March 24, 2026

---

## Overview

Glassworks uses a multi-layered detection approach with 13+ detectors organized into 3 tiers:

| Tier | Purpose | Detectors |
|------|---------|-----------|
| **L1** | Primary indicators | InvisibleChar, Homoglyph, BidirectionalOverride, UnicodeTags |
| **L2** | Secondary patterns | GlasswarePattern, EncryptedPayload, RDD, JPD Author |
| **L3** | Behavioral analysis | TimeDelay, LocaleGeofencing, BlockchainC2, HeaderC2 |

---

## Detector Reference

### L1 Detectors (Primary Indicators)

#### InvisibleCharacter Detector

**File:** `glassware-core/src/detectors/invisible.rs`

**Detects:** Invisible Unicode characters used for steganography

| Unicode Range | Name | Severity |
|---------------|------|----------|
| U+FE00-U+FE0F | Variation Selectors | High |
| U+200B-U+200F | Zero-width chars | High |
| U+2060-U+206F | Word joiner | High |
| U+E0000-U+E007F | Tags | Critical |

**Context-Aware Detection (2026-03-24):**
- Invisible chars + decoder = **Critical** (steganography)
- High density invisible chars = **High**
- Invisible chars alone in i18n file = **Skip** (legitimate)

**Files Scanned:** All except .d.ts and i18n JSON

---

#### Homoglyph Detector

**File:** `glassware-core/src/confusables/`

**Detects:** Characters that look like ASCII but aren't

**Example:** Cyrillic 'а' (U+0430) vs Latin 'a' (U+0041)

**Severity:** High

---

#### BidirectionalOverride Detector

**File:** `glassware-core/src/detectors/`

**Detects:** Unicode bidi overrides that reorder text

**Example:** U+202E (Right-to-Left Override) used to hide file extensions

**Severity:** High

---

### L2 Detectors (Secondary Patterns)

#### GlasswarePattern Detector

**File:** `glassware-core/src/detectors/glassware.rs`

**Detects:** GlassWare-specific patterns

| Pattern | Severity |
|---------|----------|
| eval() + invisible chars | Critical |
| Function() + encoded payload | Critical |
| vsprintf() + user input | High |

**Files Scanned:** .js, .mjs, .cjs, .ts, .tsx, .jsx, .json

---

#### EncryptedPayload Detector

**File:** `glassware-core/src/encrypted_payload_detector.rs`

**Detects:** Encrypted/obfuscated payload loaders

| Pattern | Severity |
|---------|----------|
| XOR decryption routine | High |
| Base64 + eval chain | High |
| String.fromCharCode + eval | Medium |

---

#### RDD Attack Detector (Registry Dependency)

**File:** `glassware-core/src/rdd_detector.rs`

**Detects:** Registry dependency confusion attacks

**Weight:** 3.0 (heavy detector)

---

#### JPD Author Detector

**File:** `glassware-core/src/jpd_author_detector.rs`

**Detects:** JSON package descriptor author manipulation

**Weight:** 3.0 (heavy detector)

---

### L3 Detectors (Behavioral Analysis)

#### TimeDelay Sandbox Evasion Detector

**File:** `glassware-core/src/time_delay_detector.rs`

**Detects:** Time-based sandbox evasion

| Pattern | Severity | Context |
|---------|----------|---------|
| CI bypass + setTimeout >30s | **Critical** | Sandbox evasion |
| setTimeout >300000ms (5min) | High | Possible evasion |
| Specific delays (15min, 48hr, 72hr) | Critical | Known evasion |
| setTimeout alone | Low | Likely legitimate |

**Context-Aware Detection (2026-03-24):**
- **Before:** Skipped all build tools
- **After:** Scans all packages; CI bypass + delay = evasion

**Rationale:** Build tools ARE attack targets (Babel 2024, Webpack 2025)

---

#### BlockchainC2 Detector

**File:** `glassware-core/src/blockchain_c2_detector.rs`

**Detects:** Command-and-control via blockchain

| Indicator | Severity | Notes |
|-----------|----------|-------|
| Known C2 wallet | **Critical** | Always flagged |
| Known C2 IP | **Critical** | Always flagged |
| Solana RPC + polling | Medium | Needs context |
| 5-second polling | Critical | GlassWorm signature |

**Known C2 Wallets (GlassWorm):**
```
BjVeAjPrSKFiingBn4vZvghsGj9KCE8AJVtbc9S8o8SC
28PKnu7RzizxBzFPoLp69HLXp9bJL3JFtT2s5QzHsEA2
G2YxRa6wt1qePMwfJzdXZG62ej4qaTC7YURzuh2Lwd3t
DSRUBTziADDHSik7WQvSMjvwCHFsbsThrbbjWMoJPUiW
6YGcuyFRJKZtcaYCCFba9fScNUvPkGXodXE1mJiSzqDJ
```

**Known C2 IPs:**
```
104.238.191.54    # Vultr AS20473
108.61.208.161    # Vultr AS20473
45.150.34.158     # led-win32 exfil
45.32.150.251     # GlassWorm infra
45.32.151.157     # GlassWorm infra
70.34.242.255     # GlassWorm infra
217.69.3.152      # Exfil endpoint /wall
```

**Context-Aware Detection (2026-03-24):**
- **Before:** Skipped crypto packages and cloud SDKs
- **After:** Known C2 ALWAYS flagged; generic patterns use context

---

#### LocaleGeofencing Detector

**File:** `glassware-core/src/locale_detector.rs`

**Detects:** Geographic targeting (execute only in specific countries)

| Pattern | Severity |
|---------|----------|
| Intl.DateTimeFormat + country check | High |
| localeCompare + conditional exec | Medium |

---

#### HeaderC2 Detector

**File:** `glassware-core/src/header_c2_detector.rs`

**Detects:** HTTP header-based C2 communication

**Severity:** Critical

---

## Scoring System

### Base Score Calculation

```rust
score = (category_count * category_weight) +
        (critical_hits * critical_weight) +
        (high_hits * high_weight)
```

**Default Weights:**
- `category_weight`: 3.0 (per unique category)
- `critical_weight`: 4.0 (per critical finding)
- `high_weight`: 2.0 (per high severity finding)

### Category Diversity Caps

| Categories | Cap | Rationale |
|------------|-----|-----------|
| 1 | 4.0 | Single vector = suspicious, not malicious |
| 2 | 7.0 | Two vectors = borderline |
| 3+ | None | Multi-vector = very likely malicious |

### Scoring Exceptions (Phase 3)

Some patterns are so distinctive they override category caps:

| Exception | Min Score | Trigger |
|-----------|-----------|---------|
| **Known C2** | 8.5 | BlockchainC2 + "Known C2" or "GlassWorm" |
| **Steganography** | 8.0 | InvisibleChar + "decoder" |
| **High Confidence** | 7.5 | confidence ≥0.90 + Critical |

### Threat Classification

| Score | Classification | Action |
|-------|---------------|--------|
| ≥7.0 | **Malicious** | Block, report, investigate |
| 3.5-6.9 | **Suspicious** | Review manually |
| <3.5 | **Clean** | No action |

---

## Detection Pipeline

```
Package Input (npm/tarball/directory)
         ↓
    Downloader
         ↓
     Scanner
         ↓
  ScanEngine (parallel detectors)
         ↓
  For each file:
    - Build FileIR
    - Run L1 detectors
    - Run L2 detectors
    - Run L3 detectors
    - Collect findings
         ↓
  Calculate Threat Score
    - Apply category diversity
    - Apply exceptions
         ↓
  Apply Threshold
    - ≥7.0 = malicious
    - 3.5-6.9 = suspicious
         ↓
  Return Results
```

---

## Known Attack Patterns

### GlassWorm Campaign (2024-2026)

**Characteristics:**
- Unicode steganography (variation selectors)
- Solana blockchain C2
- ForceMemo Python injection
- 5-second polling interval

**Known Packages:**
- react-native-country-select (0.3.91)
- react-native-phone-input (1.3.7)
- react-native-otp-inputs (0.3.1)

---

### Shai-Hulud Campaign

**Characteristics:**
- CI bypass + time delay
- Sandbox evasion
- Conditional execution

---

### SANDWORM_MODE

**Characteristics:**
- 48-96 hour delays
- CI/CD bypass
- Multi-stage payload

---

## Tuning Guidelines

### When to Adjust Detector Weights

1. **Too many FPs:** Increase category diversity cap
2. **Missing attacks:** Lower threshold or add exceptions
3. **Specific detector noisy:** Reduce weight, not skip logic

### When NOT to Adjust

1. **Never** add package-level whitelists (use context-aware detection)
2. **Never** skip entire directories (malicious code lives in /dist/)
3. **Never** skip build tools (they're attack targets)

---

## Testing

### Evidence Validation

```bash
# Run evidence validation
./tests/validate-evidence.sh evidence target/release/glassware

# Target: ≥90% detection rate
```

### Individual Package Testing

```bash
# Scan single tarball
./target/release/glassware scan-tarball evidence/package.tgz

# Scan with verbose output
./target/release/glassware scan-tarball evidence/package.tgz --verbose
```

---

## References

- [Koi Security - GlassWorm Research](https://www.koisecurity.io/)
- [Aikido Security - GlassWorm Returns](https://www.aikido.dev/blog/glassworm-returns-unicode-attack-github-npm-vscode)
- [Sonatype - Supply Chain Reports](https://www.sonatype.com/)
- [Socket.dev - Real-time Detection](https://socket.dev/)
