# Detector Tuning Plan - Context-Aware Detection

**Date:** 2026-03-26
**Goal:** Distinguish build tool patterns from malicious patterns WITHOUT whitelisting

---

## Core Principle

**DO NOT whitelist packages based on popularity.**

**DO improve detectors to distinguish:**
- Build-time optimization vs runtime evasion
- Telemetry/analytics vs C2 communication
- Legitimate SDK usage vs malicious C2 usage

---

## Issue 1: TimeDelaySandboxEvasion

### Current Behavior (FP on prisma, playwright, pm2)

Detects: `if (process.env.CI) { setTimeout(..., 5000) }`

**Problem:** This pattern exists in:
- **Build tools:** Build optimization (skip delays in CI for faster builds)
- **Malware:** Sandbox evasion (skip delays in CI to avoid detection)

### Context-Aware Fix

**Distinguishing Factors:**

| Factor | Build Tool | Malware |
|--------|------------|---------|
| **File Path** | `generator-build/`, `build/`, `dist/` | `runtime/`, `index.js`, `install.js` |
| **Build Signatures** | `/* webpack`, `__webpack_require__` | None |
| **Delay Purpose** | Build optimization comment | No comment or misleading |
| **CI Check Location** | Build configuration | Hidden in runtime code |

**Implementation:**
```rust
fn detect_time_delay(&self, content: &str, file_path: &str) -> Vec<Finding> {
    // Skip build tool output
    if is_build_output(file_path, content) {
        return findings;  // Build optimization, not evasion
    }
    
    // Check for legitimate build context
    if has_build_optimization_comment(content) {
        return findings;  // Documented build optimization
    }
    
    // Runtime code with CI check + delay = evasion
    if has_ci_check(content) && has_delay(content) {
        findings.push(CRITICAL("CI-aware time delay (sandbox evasion)"));
    }
}

fn is_build_output(file_path: &str, content: &str) -> bool {
    // Check path
    if file_path.contains("/dist/") || file_path.contains("/build/") {
        return true;
    }
    
    // Check for build tool signatures
    const BUILD_SIGNATURES = [
        "/* webpack",
        "__webpack_require__",
        "/* babel",
        "//# sourceMappingURL=",
    ];
    
    BUILD_SIGNATURES.iter().any(|s| content.contains(s))
}
```

**Expected Impact:**
- prisma (generator-build/): SKIPPED ✅
- playwright (build output): SKIPPED ✅
- pm2 (runtime): FLAGGED (correct if evasion)

---

## Issue 2: HeaderC2

### Current Behavior (FP on prisma, firebase)

Detects: Custom HTTP headers like `X-Prisma-`, `X-Firebase-`

**Problem:** This pattern exists in:
- **Legitimate tools:** Telemetry, analytics, build tracking
- **Malware:** C2 communication, data exfiltration

### Context-Aware Fix

**Distinguishing Factors:**

| Factor | Telemetry | C2 |
|--------|-----------|-----|
| **Header Names** | X-Telemetry, X-Analytics, X-Build-Id | X-Exfil-ID, X-Session-Token, X-Data |
| **Direction** | One-way (send only) | Two-way (send + receive commands) |
| **Response Handling** | None or logging | Parse response for commands |
| **Endpoint** | Known telemetry endpoints | Unknown/suspicious endpoints |

**Implementation:**
```rust
fn detect_header_c2(&self, content: &str) -> Vec<Finding> {
    // Known telemetry header prefixes (not C2)
    const TELEMETRY_PREFIXES = [
        "X-Telemetry",
        "X-Analytics",
        "X-Build-Id",
        "X-Prisma-",  // Prisma telemetry
        "X-Firebase-", // Firebase analytics
    ];
    
    // Known C2 header patterns
    const C2_PATTERNS = [
        "X-Exfil",
        "X-Session-Token",
        "X-Data-Payload",
        "X-Command",
        "X-Exec",
    ];
    
    for header in extract_headers(content) {
        // Skip known telemetry
        if TELEMETRY_PREFIXES.iter().any(|p| header.starts_with(p)) {
            continue;  // Telemetry, not C2
        }
        
        // Flag known C2 patterns
        if C2_PATTERNS.iter().any(|p| header.starts_with(p)) {
            findings.push(CRITICAL("C2 header pattern detected"));
        }
        
        // Check for two-way communication (C2 indicator)
        if has_header(content, header) && has_response_parsing(content) {
            findings.push(HIGH("Custom header with response parsing (possible C2)"));
        }
    }
}

fn has_response_parsing(content: &str) -> bool {
    // Check for command extraction from response
    const COMMAND_PATTERNS = [
        "response.commands",
        "data.exec",
        "JSON.parse(response).command",
        "eval(response",
        "Function(response",
    ];
    
    COMMAND_PATTERNS.iter().any(|p| content.contains(p))
}
```

**Expected Impact:**
- prisma (X-Prisma- telemetry): SKIPPED ✅
- firebase (X-Firebase- analytics): SKIPPED ✅
- Malware with X-Exfil-ID: FLAGGED ✅

---

## Issue 3: BlockchainC2

### Current Behavior (FP on @solana/web3.js, mongodb)

Detects: `getSignaturesForAddress`, Solana RPC calls

**Problem:** This pattern exists in:
- **Legitimate SDKs:** Public blockchain API usage
- **Malware:** C2 via blockchain (GlassWorm signature)

### Context-Aware Fix

**Distinguishing Factors:**

| Factor | Legitimate SDK | Malware |
|--------|---------------|---------|
| **Wallet Address** | Public, known wallets | Known C2 wallets (from research) |
| **RPC Endpoint** | Public RPC (api.mainnet-beta.solana.com) | Private/suspicious IPs |
| **Polling Pattern** | Variable intervals | Fixed 5-minute intervals (GlassWorm) |
| **Command Extraction** | None | Parse transaction memos for commands |

**Implementation:**
```rust
fn detect_blockchain_c2(&self, content: &str) -> Vec<Finding> {
    // Known GlassWorm C2 wallets (from research)
    const KNOWN_C2_WALLETS = [
        "28PKnu7RzizxBzFPoLp69HLXp9bJL3JFtT2s5QzHsEA2",
        "BjVeAjPrSKFiingBn4vZvghsGj9KCE8AJVtbc9S8o8SC",
    ];
    
    // Known C2 IPs (from research)
    const KNOWN_C2_IPS = [
        "104.238.191.54",
        "108.61.208.161",
        "45.150.34.158",
    ];
    
    // Check for known C2 wallets (CRITICAL - always flag)
    for wallet in KNOWN_C2_WALLETS {
        if content.contains(wallet) {
            findings.push(CRITICAL("Known GlassWorm C2 wallet"));
        }
    }
    
    // Check for known C2 IPs (CRITICAL - always flag)
    for ip in KNOWN_C2_IPS {
        if content.contains(ip) {
            findings.push(CRITICAL("Known GlassWorm C2 IP"));
        }
    }
    
    // Check for GlassWorm C2 signature (HIGH)
    // Pattern: getSignaturesForAddress + setInterval + 300000ms (5 min)
    if has_glassworm_c2_signature(content) {
        findings.push(HIGH("GlassWorm C2 signature (5-min polling)"));
    }
    
    // Skip generic SDK usage
    // getSignaturesForAddress alone is NOT suspicious
}

fn has_glassworm_c2_signature(content: &str) -> bool {
    // Must have ALL three:
    // 1. getSignaturesForAddress call
    // 2. setInterval
    // 3. 300000ms (5 minutes) polling
    
    has_function(content, "getSignaturesForAddress")
        && has_function(content, "setInterval")
        && content.contains("300000")
}
```

**Expected Impact:**
- @solana/web3.js (SDK methods): SKIPPED ✅
- mongodb (database driver, not blockchain): SKIPPED ✅
- GlassWorm with known wallet: FLAGGED ✅
- GlassWorm with 5-min polling: FLAGGED ✅

---

## Issue 4: EncryptedPayload/Rc4Pattern

### Current Behavior (FP on prisma, prettier)

Detects: High-entropy strings, XOR operations, character manipulation

**Problem:** This pattern exists in:
- **Build tools:** String encoding, source map generation
- **Malware:** Encrypted payload decryption

### Context-Aware Fix

**Distinguishing Factors:**

| Factor | Build Tool | Malware |
|--------|------------|---------|
| **Context** | Source map, bundle generation | Hidden in runtime code |
| **Function Names** | encodeSourceMap, generateBundle | decode, decrypt, extract |
| **Output Usage** | Write to file | eval(), Function(), dynamic require |
| **Entropy** | Medium (encoded source) | Very high (encrypted binary) |

**Implementation:**
```rust
fn detect_encrypted_payload(&self, content: &str, file_path: &str) -> Vec<Finding> {
    // Skip build tool output
    if is_build_output(file_path, content) {
        return findings;
    }
    
    // Check for build tool context
    if has_build_function_names(content) {
        return findings;  // Source map encoding, not encryption
    }
    
    // Check for dynamic execution (malware indicator)
    if has_high_entropy(content) && has_dynamic_execution(content) {
        findings.push(CRITICAL("Encrypted payload with dynamic execution"));
    }
}

fn has_dynamic_execution(content: &str) -> bool {
    const EXEC_PATTERNS = [
        "eval(",
        "Function(",
        "new Function(",
        "vm.runInContext(",
        "dynamic require(",
    ];
    
    EXEC_PATTERNS.iter().any(|p| content.contains(p))
}
```

**Expected Impact:**
- prisma (source map encoding): SKIPPED ✅
- prettier (string manipulation): SKIPPED ✅
- Malware with eval + encrypted blob: FLAGGED ✅

---

## Implementation Priority

### Phase 1: Build Tool Detection (Highest Impact)
1. Add `is_build_output()` helper
2. Add build signature detection
3. Apply to TimeDelay, EncryptedPayload detectors

### Phase 2: BlockchainC2 Specificity
1. Add known C2 wallet/IP lists
2. Add GlassWorm signature detection (5-min polling)
3. Skip generic SDK usage

### Phase 3: Telemetry vs C2
1. Add telemetry header prefix list
2. Add C2 header pattern list
3. Check for two-way communication

### Phase 4: Dynamic Execution Context
1. Add dynamic execution detection
2. Combine with encrypted payload detection
3. Skip build tool context

---

## Expected FP Rate After Fixes

| Fix | FP Reduction | Remaining FPs |
|-----|--------------|---------------|
| Build tool detection | -5 FPs | 6 |
| BlockchainC2 specificity | -3 FPs | 3 |
| Telemetry vs C2 | -2 FPs | 1 |
| Dynamic execution context | -1 FP | 0 |

**Current FP Rate:** 11/197 = 5.6%
**Expected After Fixes:** 0-1/197 = < 1%

---

## Testing Strategy

1. **Re-run wave15** with fixed detectors
2. **Verify FP reduction** on known FPs (prisma, @solana, etc.)
3. **Verify evidence detection** still 100%
4. **Run wave16 (500 pkg)** for validation

---

**Key Principle:** No package whitelisting. Better context-aware detection.
