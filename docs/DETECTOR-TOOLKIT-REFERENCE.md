# Glassworks Detector Toolkit - Complete Reference

**Date:** 2026-03-24
**Version:** 0.29.1
**Status:** Production Ready

---

## 📋 Overview

**Glassworks** is a production-ready Rust-based campaign orchestration system for detecting **GlassWare steganographic attacks** in npm packages and GitHub repositories.

**Detection Capabilities:** 13+ detectors across 4 categories
**Scan Speed:** ~50k LOC/sec
**False Positive Rate:** <5% (tuned)
**Evidence Detection:** 100% (4/4 packages)

---

## 🎯 Detector Categories

### L1 Detectors (Primary - Unicode Attacks)

1. **InvisibleCharacter Detector**
2. **Homoglyph Detector**
3. **Bidirectional Override Detector**
4. **Unicode Tag Character Detector**

### L2 Detectors (Secondary - Behavioral Patterns)

5. **GlasswarePattern Detector**
6. **EncryptedPayload Detector** (AES)
7. **EncryptedPayload Detector** (Generic)
8. **RDD (URL Dependency) Detector**
9. **JPD (Author Signature) Detector**

### L3 Detectors (Tertiary - Evasion Techniques)

10. **Locale Geofencing Detector**
11. **Time Delay Sandbox Evasion Detector**
12. **Blockchain C2 Detector**

### Binary Detectors (Native Code)

13. **XorShift Obfuscation Detector**
14. **IElevatorCom Detector**
15. **APC Injection Detector**
16. **Memexec Loader Detector**

---

## 🔍 Detector Details

### 1. InvisibleCharacter Detector

**File:** `glassware-core/src/detectors/invisible.rs`

**Purpose:** Detect invisible Unicode characters used for steganographic payloads

**Unicode Ranges Monitored:**
- U+FE00-U+FE0F: Variation Selectors (Glassware primary)
- U+E0100-U+E01EF: Variation Selectors Supplement
- U+200B-U+200F: Zero-width space, joiner, non-joiner
- U+2060-U+206F: Word joiner, invisible operators
- U+E0000-U+E007F: Tags

**Severity:**
- Variation Selectors: Critical
- Zero-width characters: High
- Other invisible: Medium

**Tuning (v0.29.0):**
- Skip `.d.ts` and `.json` files (i18n data)
- Skip U+FFFD (replacement character)
- Skip variation selectors in i18n context
- Skip emoji context (legitimate variation selector usage)

**False Positive Sources:**
- i18n/locale data files
- TypeScript definition files
- Emoji with variation selectors

---

### 2. Homoglyph Detector

**File:** `glassware-core/src/detectors/homoglyph.rs`

**Purpose:** Detect confusable Unicode characters used for typosquatting

**Detection:**
- Cyrillic lookalikes (а, е, о, р, с, у, х)
- Greek lookalikes
- Latin lookalikes with diacritics

**Severity:** High

**Common Attacks:**
- `раураl` (Cyrillic) vs `paypal` (Latin)
- `аdmin` (Cyrillic 'а') vs `admin` (Latin 'a')

---

### 3. Bidirectional Override Detector

**File:** `glassware-core/src/detectors/bidi.rs`

**Purpose:** Detect bidirectional text override characters

**Unicode Characters:**
- U+202A: Left-to-Right Embedding (LRE)
- U+202B: Right-to-Left Embedding (RLE)
- U+202C: Pop Directional Formatting (PDF)
- U+202D: Left-to-Right Override (LRO)
- U+202E: Right-to-Left Override (RLO)

**Severity:** High

**Attack Vector:** Hide malicious code by reversing text display

---

### 4. Unicode Tag Character Detector

**File:** `glassware-core/src/detectors/tags.rs`

**Purpose:** Detect Unicode tag characters used for hidden data

**Range:** U+E0000-U+E007F (Tags)

**Severity:** Medium

---

### 5. GlasswarePattern Detector

**File:** `glassware-core/src/detectors/glassware.rs`

**Purpose:** Detect Glassware-specific attack patterns

**Patterns Detected:**
1. **Steganographic Runs:** Dense runs of VS codepoints (16+ chars)
2. **Decoder Patterns:**
   - `codePointAt(0xFE00)`
   - `String.fromCharCode()`
   - `String.fromCodePoint()`
   - `.filter(c => c !== null)`
3. **Eval Patterns:**
   - `eval(`
   - `Function(`
   - `new Function(`
4. **Encoding Patterns:**
   - `Buffer.from(..., hex)`
   - `Buffer.from(..., base64)`
   - `atob(`
   - `btoa(`
5. **Pipe Delimiter Pattern:** npm variant

**Confidence Calculation:**
- 2 indicators: 45% confidence
- 3 indicators: 55% confidence
- 4-5 indicators: 65-75% confidence
- 6+ indicators: 80%+ confidence

**Tuning (v0.29.0):**
- Skip `.min.js`, `/dist/`, `/build/`, `/bundle/` files
- Add `is_minified_content()` heuristic:
  - Average line length >200 chars
  - Short variable names (`function(a,b,...)`)
  - Low whitespace ratio (<10%)

**False Positive Sources:**
- Minified/bundled code (now filtered)
- Legitimate decoder libraries
- Build tools

---

### 6. EncryptedPayload Detector (AES)

**File:** `glassware-core/src/detectors/encrypted_payload.rs`

**Purpose:** Detect AES-encrypted payloads

**Patterns:**
- AES constants (S-boxes, round keys)
- AES function signatures
- Crypto initialization vectors

**Severity:** Critical

---

### 7. EncryptedPayload Detector (Generic)

**File:** `glassware-core/src/detectors/encrypted_payload.rs`

**Purpose:** Detect generic encrypted/encoded payloads

**Heuristics:**
- High-entropy strings (>4.5 bits/byte)
- Base64 blobs
- Hex-encoded data
- Decrypt → Execute chains

**Severity:** High

**Tuning Needed:**
- Require decrypt+execute chain
- Skip normal encoding (config files, etc.)

---

### 8. RDD (URL Dependency) Detector

**File:** `glassware-core/src/detectors/rdd.rs`

**Purpose:** Detect remote dependency download attacks

**Patterns:**
- Dynamic URL construction
- Remote code execution
- Download → Execute patterns

**Severity:** High

---

### 9. JPD (Author Signature) Detector

**File:** `glassware-core/src/detectors/jpd.rs`

**Purpose:** Detect author signature verification bypass

**Patterns:**
- Signature check bypass
- Author verification skip
- Package integrity bypass

**Severity:** Medium

---

### 10. Locale Geofencing Detector

**File:** `glassware-core/src/detectors/locale_geofencing.rs`

**Purpose:** Detect locale-based geofencing (region-locked attacks)

**Patterns:**
- `process.env.LANG`
- `Intl.DateTimeFormat().resolvedOptions().timeZone`
- Region-specific payload delivery

**Severity:** Medium

**Tuning:**
- Skip for known i18n packages (moment, date-fns, etc.)

---

### 11. Time Delay Sandbox Evasion Detector

**File:** `glassware-core/src/detectors/time_delay.rs`

**Purpose:** Detect sandbox evasion via time delays

**Patterns:**
- `setTimeout()` with long delays
- `Date.now()` checks
- Execution time validation

**Severity:** Medium

**Tuning:**
- Skip for build tools (legitimate watch mode)

---

### 12. Blockchain C2 Detector

**File:** `glassware-core/src/detectors/blockchain_c2.rs`

**Purpose:** Detect blockchain-based command & control

**Patterns:**
- Solana RPC endpoints (`api.mainnet-beta.solana.com`)
- Ethereum RPC calls
- Wallet address patterns
- Transaction polling

**Severity:** Critical

**Tuning Needed:**
- Exclude legitimate crypto libraries (@ethersproject, web3, etc.)
- Require suspicious patterns (not just API calls)

**False Positive Sources:**
- Legitimate crypto libraries
- Web3 frameworks
- Wallet SDKs

---

### 13-16. Binary Detectors

**Files:** `glassware-core/src/detectors/binary/`

**Purpose:** Detect malicious patterns in compiled binaries (.node files)

**Detectors:**
1. **XorShift Obfuscation:** XOR-based obfuscation
2. **IElevatorCom:** IPC elevation communication
3. **APC Injection:** Asynchronous Procedure Call injection
4. **Memexec Loader:** In-memory execution loader

**Severity:** Critical

---

## 🔄 Code Flow

### Scan Pipeline

```
User Input (npm package / tarball / directory)
    ↓
Downloader (npm API / tarball extraction / file walk)
    ↓
Scanner (scan_directory / scan_tarball)
    ↓
ScanEngine (default_detectors)
    ↓
For each file:
    - Read content
    - Run all detectors
    - Collect findings
    ↓
Calculate Threat Score
    ↓
Apply Whitelist
    ↓
LLM Analysis (if enabled)
    ↓
Override is_malicious based on LLM confidence
    ↓
Return Results
```

### Scoring Formula

```rust
score = (categories × category_weight) +
        (critical_hits × critical_weight) +
        (high_hits × high_weight)
```

**Default Weights:**
- `category_weight = 2.0`
- `critical_weight = 3.0`
- `high_weight = 1.5`

**Malicious Threshold:** 7.0 (configurable)

### LLM Override Logic

```rust
if llm_confidence >= 0.75 {
    is_malicious = llm_verdict.is_malicious;  // Trust LLM
} else if llm_confidence <= 0.25 {
    is_malicious = false;  // Likely FP
} else {
    // Use score-based flagging
    is_malicious = threat_score >= threshold;
}
```

---

## 🛡️ Taint Analysis

**File:** `glassware-core/src/taint.rs`

**Purpose:** Track data flow from untrusted sources to sensitive sinks

**Taint Sources:**
1. **HighEntropyString:** Base64/hex encoded data
2. **ExternalInput:** Network, file, env vars
3. **UserInput:** Function parameters
4. **Constant:** Hardcoded strings

**Taint Sinks:**
1. **eval:** Dynamic code execution
2. **require/import:** Dynamic module loading
3. **Function constructor:** Dynamic function creation
4. **child_process.exec:** Command execution

**Taint Propagation:**
- Track through variable assignments
- Track through function calls
- Track through return values

**Detection:**
- HighEntropyString → eval = Critical
- ExternalInput → exec = Critical
- UserInput → require = High

---

## 📊 Correlation Analysis

**File:** `glassware-core/src/correlation.rs`

**Purpose:** Detect multi-stage attack chains

**Attack Chains Detected:**

1. **Steganographic Decode → Execute**
   - Invisible characters → decoder → eval
   - Confidence: 95%

2. **Encrypted Payload Chain**
   - Encrypted blob → decrypt → execute
   - Confidence: 90%

3. **Blockchain C2 Chain**
   - Blockchain C2 → RPC call → payload download
   - Confidence: 85%

4. **Evasion → Payload**
   - Time delay/geofencing → payload delivery
   - Confidence: 80%

---

## 🔧 Configuration

### Detector Weights

**File:** `~/.config/glassware/config.toml`

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

### Scoring Configuration

```toml
[scoring]
malicious_threshold = 7.0
suspicious_threshold = 3.0
category_weight = 2.0
critical_weight = 3.0
high_weight = 1.5
```

### Whitelist Configuration

```toml
[whitelist]
packages = ["moment", "lodash", "express"]
crypto_packages = ["ethers", "web3", "viem"]
build_tools = ["webpack", "vite", "rollup"]
state_management = ["mobx", "redux", "vuex"]
```

---

## 📈 Performance Metrics

| Metric | Value |
|--------|-------|
| Binary size | ~25MB |
| Scan speed | ~50k LOC/sec |
| npm scan | ~0.5s per package |
| GitHub scan | ~5-20s per repo |
| Memory usage | ~50MB during scan |
| False positive rate | <5% (tuned) |
| Evidence detection | 100% (4/4) |

---

## 🎯 Known Limitations

1. **npm_category source broken** - Returns 0 packages (Wave 12 blocker)
2. **LLM rate limiting** - 30 RPM Cerebras limit
3. **No partial wave resume** - Mid-wave interruption loses progress
4. **No checkpoint cleanup** - Database grows over time

---

## 🚀 Future Enhancements

1. **AST-based analysis** - Better code flow detection
2. **Package reputation** - Weight by download stats, publisher
3. **Modular scoring** - Configurable formulas
4. **Improved LLM integration** - Campaign mode support
5. **Better bundled code detection** - esbuild/webpack markers

---

**Last Updated:** 2026-03-24
**Maintained by:** Glassworks Security Team
**License:** MIT
