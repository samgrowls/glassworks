# glassware Detector Reference

**Version:** v0.67.0
**Last Updated:** 2026-03-26

---

## Overview

glassware uses 13+ specialized detectors organized in three tiers. Each detector analyzes source code for specific GlassWorm attack indicators.

---

## Tier 1: Primary Indicators

These detectors run on all files and identify core GlassWorm patterns.

### InvisibleCharacter

**File:** `glassware-core/src/detectors/invisible.rs`

**Detects:** Invisible Unicode characters used for steganography

| Character | Code Point | Name |
|-----------|------------|------|
| ZWSP | U+200B | Zero Width Space |
| ZWNJ | U+200C | Zero Width Non-Joiner |
| ZWJ | U+200D | Zero Width Joiner |
| Word Joiner | U+2060 | Word Joiner |
| VS1-16 | U+FE00-FE0F | Variation Selectors |
| VS17-256 | U+E0100-E01EF | Variation Selectors Supplement |

**Severity:** High

**Context-Aware:**
- Skips i18n/translation files (legitimate usage)
- Flags i18n files with decoder patterns (steganography)

**Example Finding:**
```
[high] file.js:42 - Invisible character U+200B (ZWSP) detected
```

---

### Homoglyph

**File:** `glassware-core/src/detectors/homoglyph.rs`

**Detects:** Confusable Unicode characters that look like ASCII

**Examples:**
- Cyrillic `а` (U+0430) vs Latin `a` (U+0061)
- Greek `ο` (U+03BF) vs Latin `o` (U+006F)

**Severity:** Medium

**Example Finding:**
```
[medium] file.js:15 - Homoglyph detected: Cyrillic 'а' (U+0430) looks like Latin 'a'
```

---

### BidirectionalOverride

**File:** `glassware-core/src/detectors/bidi.rs`

**Detects:** Unicode bidirectional text override characters

**Characters:**
- U+202A: Left-to-Right Embedding
- U+202B: Right-to-Left Embedding
- U+202C: Pop Directional Formatting
- U+202D: Left-to-Right Override
- U+202E: Right-to-Left Override

**Severity:** Medium

**Example Finding:**
```
[medium] file.js:8 - Bidirectional override U+202E detected
```

---

### GlasswarePattern

**File:** `glassware-core/src/detectors/glassware.rs`

**Detects:** Combined steganography + decoder patterns

**Detection Logic:**
1. Invisible Unicode characters (REQUIRED)
2. VS-specific decoder patterns:
   - `codePointAt(0xFE00)` - Variation Selector decoding
   - `.filter(c => 0x200B)` - Invisible char filtering
   - `fromCodePoint + map + 0xFE0` - VS reconstruction

**Confidence Calculation:**
- Invisible char count: up to 40%
- Decoder indicators: up to 60%
- Combined confidence >= 70% = flag

**Severity:** Critical (when confidence >= 90%)

**Example Finding:**
```
[critical] file.js:78 - GlassWorm steganography detected: 15 invisible chars + decoder (confidence: 90%)
```

---

## Tier 2: Secondary Confirmation

These detectors run only if Tier 1 score >= threshold.

### EncryptedPayload

**File:** `glassware-core/src/encrypted_payload_detector.rs`

**Detects:** High-entropy encoded blobs combined with dynamic execution

**Detection Logic:**
1. High-entropy blob (entropy > 4.5 bits/byte)
2. Dynamic execution: `eval()`, `Function()`, `vm.runInContext()`
3. Decryption patterns: `createDecipheriv()`, `atob()`, `Buffer.from()`

**Context-Aware:**
- Skips build tool output (webpack, babel, etc.)
- Skips minified/bundled files

**Severity:** High

**Example Finding:**
```
[high] file.js:156 - High-entropy blob combined with decrypt→exec flow — potential encrypted payload loader
```

---

### HeaderC2

**File:** `glassware-core/src/header_c2_detector.rs`

**Detects:** HTTP header-based C2 communication

**Detection Logic (ALL THREE required):**
1. HTTP header extraction: `headers[`, `headers.get()`, `getHeader()`
2. Decryption: `createDecipheriv()`, `crypto.subtle.decrypt()`
3. Dynamic execution: `eval()`, `Function()`

**Telemetry Whitelist:**
- X-Telemetry, X-Analytics, X-Build-Id
- X-Prisma-, X-Firebase-, X-Vercel-
- X-Sentry-, X-NewRelic-, X-Datadog-

**C2 Header Patterns (always suspicious):**
- X-Exfil, X-Session-Token, X-Data-Payload
- X-Command, X-Exec, X-Eval

**Context-Aware:**
- Skips build tool output

**Severity:** Critical

**Example Finding:**
```
[critical] file.js:234 - HTTP header data extraction combined with decryption and dynamic execution — potential C2 payload delivery (GlassWare Wave 4-5)
```

---

### ExfilSchema

**File:** `glassware-core/src/detectors/exfil_schema.rs`

**Detects:** Data exfiltration patterns

**Patterns:**
- Custom HTTP headers for data exfiltration
- DNS TXT record queries
- GitHub API for data drops
- Blockchain transfers with memos

**Severity:** High

**Example Finding:**
```
[high] file.js:89 - Data exfiltration pattern detected: custom HTTP headers with encoded data
```

---

## Tier 3: Behavioral Analysis

These detectors run only if Tier 1+2 score >= threshold.

### BlockchainC2

**File:** `glassware-core/src/blockchain_c2_detector.rs`

**Detects:** Blockchain-based C2 communication

**Detection Logic:**

**CRITICAL (always flag):**
- Known C2 wallets (from GlassWorm research)
- Known C2 IPs (from GlassWorm research)

**HIGH (GlassWorm signature):**
- `getSignaturesForAddress` + `setInterval` + 300000ms (5 min)

**Context-Aware:**
- Skips generic SDK usage
- Only flags known malicious indicators

**Known C2 Wallets:**
```
28PKnu7RzizxBzFPoLp69HLXp9bJL3JFtT2s5QzHsEA2  // Primary GlassWorm
BjVeAjPrSKFiingBn4vZvghsGj9KCE8AJVtbc9S8o8SC  // ForceMemo C2
```

**Known C2 IPs:**
```
104.238.191.54    // Vultr AS20473
108.61.208.161    // Vultr AS20473
45.150.34.158     // Non-Vultr C2
```

**Severity:** Critical (known wallets/IPs), High (signature)

**Example Finding:**
```
[critical] file.js:45 - Known GlassWorm C2 wallet address: 28PKnu7RzizxBzFPoLp69HLXp9bJL3JFtT2s5QzHsEA2
```

---

### BlockchainPolling

**File:** `glassware-core/src/detectors/blockchain_polling.rs`

**Detects:** GlassWorm-specific blockchain polling patterns

**Detection Logic:**
- `getSignaturesForAddress` + `setInterval` + 5-minute polling
- Transaction memo instruction parsing
- Command extraction patterns (`decodeCommand`, `executeCommand`)

**Context-Aware:**
- Skips SDK source code
- Only flags GlassWorm-specific patterns

**Severity:** Critical

**Example Finding:**
```
[critical] file.js:123 - GlassWorm C2 polling pattern detected (getSignaturesForAddress + setInterval + 5min)
```

---

### TimeDelaySandboxEvasion

**File:** `glassware-core/src/time_delay_detector.rs`

**Detects:** Time-delay sandbox evasion with CI bypass

**Detection Logic:**
- CI environment check: `process.env.CI`, `GITHUB_ACTIONS`
- Long delay: `setTimeout(..., 300000+)` (5+ minutes)
- Conditional execution: `if (!isCI) setTimeout(...)`

**Context-Aware:**
- Skips build tool output (CI checks are optimization, not evasion)

**Severity:** Critical (CI + delay combination)

**Example Finding:**
```
[critical] file.js:67 - CI-aware time delay detected (sandbox evasion with CI bypass)
```

---

### LocaleGeofencing

**File:** `glassware-core/src/locale_detector.rs`

**Detects:** Geographic targeting (Russia skip)

**Detection Logic:**
- Locale check: `ru`, `RU`, `russian`
- Early exit: `process.exit(0)` after locale check
- Combined pattern: locale check + conditional exit

**Severity:** Critical

**Example Finding:**
```
[critical] file.js:34 - Geographic targeting detected: Russian locale check followed by early exit
```

---

## Detector Configuration

### Enable/Disable Detectors

```toml
[settings.detectors.invisible_char]
enabled = true
weight = 6.0

[settings.detectors.blockchain_c2]
enabled = true
weight = 5.0
```

### Adjust Weights

```toml
[settings.scoring.weights]
invisible_char = 6.0
glassware_pattern = 10.0
blockchain_c2 = 5.0
```

### Tier Configuration

```toml
[settings.scoring.tier_config]
mode = "tiered"

[[settings.scoring.tiers]]
tier = 1
detectors = ["invisible_char", "glassware_pattern"]
threshold = 0.0

[[settings.scoring.tiers]]
tier = 2
detectors = ["header_c2", "exfil_schema"]
threshold = 2.0
```

---

## Detector Development

### Adding a New Detector

1. Create file in `glassware-core/src/detectors/<name>.rs`
2. Implement `Detector` trait:

```rust
impl Detector for MyDetector {
    fn name(&self) -> &str { "my_detector" }
    fn tier(&self) -> DetectorTier { DetectorTier::Tier1 }
    fn detect(&self, ir: &FileIR) -> Vec<Finding> { ... }
}
```

3. Register in `glassware-core/src/engine.rs`:
```rust
engine.register(Box::new(MyDetector::new()));
```

4. Add to campaign config:
```toml
[[settings.scoring.tiers]]
tier = 1
detectors = ["my_detector"]
```

---

## References

- [Architecture](ARCHITECTURE.md)
- [User Guide](USER-GUIDE.md)
- [Developer Guide](DEVELOPER-GUIDE.md)
