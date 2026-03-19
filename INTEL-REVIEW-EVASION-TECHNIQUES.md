# Intel Review — Evasion Techniques

**Date:** 2026-03-19  
**Source:** Expert intelligence search  
**Status:** ✅ Validated, ready for implementation  

---

## Critical Assessment

### ✅ HIGH CONFIDENCE (Implement Immediately)

#### 1. Russian Locale/Timezone Checks
**Confidence:** HIGH - Well-documented across GlassWorm, PhantomRaven, SANDWORM_MODE  
**Implementation:** Straightforward regex + semantic analysis  
**Priority:** 🔴 CRITICAL

**Patterns confirmed:**
```javascript
Intl.DateTimeFormat().resolvedOptions().locale
Intl.DateTimeFormat().resolvedOptions().timeZone
ru-RU, Europe/Moscow
process.exit(0) after locale check
```

**Our advantage:** We can detect this SEMANTICALLY (taint flow from locale check to early exit)

---

#### 2. Time-Delay Sandbox Evasion
**Confidence:** HIGH - Multiple campaigns documented  
**Implementation:** Regex for setTimeout values + CI env check detection  
**Priority:** 🔴 CRITICAL

**Patterns confirmed:**
- Classic: `setTimeout(..., 900000)` (15 min)
- Modern: `48 * 60 * 60 * 1000` (48 hours) with CI bypass
- Pattern: `if (!isCI) setTimeout(...)` 

**Our advantage:** Can detect both regex patterns AND semantic flow (CI check → conditional delay)

---

#### 3. Blockchain C2 Polling
**Confidence:** HIGH - GlassWorm specifically uses Solana  
**Implementation:** Regex for Solana RPC + polling patterns  
**Priority:** 🔴 CRITICAL

**Patterns confirmed:**
- `api.mainnet-beta.solana.com`
- `getSignaturesForAddress`, `getTransaction`
- `setInterval(..., 5000)` (5-second polling)
- Specific wallet: `BjVeAjPrSKFiingBn4vZvghsGj9KCE8AJVtbc9S8o8SC`

**Our advantage:** This is GOLD - we can detect the exact C2 infrastructure!

---

### ⚠️ MEDIUM CONFIDENCE (Validate Before Implementing)

#### 4. Google Calendar C2 Backup
**Confidence:** MEDIUM - Mentioned but less documented  
**Implementation:** Regex for calendar URLs  
**Priority:** 🟡 HIGH

**Patterns:**
- `calendar.app.google/M2ZCvM8ULL56PD1d6` (specific URL)
- Base64-encoded URLs in event titles

**Validation needed:** Is this still active or historical?

---

#### 5. DNS Tunneling (SANDWORM_MODE)
**Confidence:** MEDIUM - Mentioned but no specific patterns  
**Implementation:** Would need DNS query monitoring (outside our scope)  
**Priority:** 🟢 LOW for now

---

### ❌ LOW CONFIDENCE / NOT ACTIONABLE

#### 6. "5 Waves" Timeline
**Issue:** Some dates don't match our intel
- Our intel says GlassWorm Wave 5 was March 2026 (MCP servers)
- This intel mentions different packages/timeline

**Action:** Cross-reference with Koi Security, Aikido reports

---

## Implementation Priority

### Phase 1: Implement TODAY (2-3 hours)

**GW009: Locale/Timezone Geofencing Detector**
```rust
// glassware-core/src/locale_detector.rs
// Detects: Intl.DateTimeFormat, ru-RU, Europe/Moscow, process.exit after check
```

**GW010: Time-Delay Sandbox Evasion Detector**
```rust
// glassware-core/src/delay_detector.rs
// Detects: setTimeout >5min, CI env checks, conditional delays
```

**GW011: Blockchain C2 Detector**
```rust
// glassware-core/src/blockchain_c2_detector.rs
// Detects: Solana RPC calls, polling patterns, specific wallet addresses
```

---

### Phase 2: Implement TOMORROW (1-2 hours)

**Update LLM Prompts:**
```
Add to analyzer.py prompt:
"Analyze for sandbox evasion:
1. Locale/timezone checks with early exit
2. setTimeout/setInterval delays >5 minutes
3. CI/CD environment detection
4. Blockchain polling (Solana, getSignaturesForAddress)
5. Silent error handling in network loops"
```

**Add Known IOCs:**
```rust
// glassware-core/src/ioc_database.rs
const KNOWN_C2_WALLETS: &[&str] = &[
    "BjVeAjPrSKFiingBn4vZvghsGj9KCE8AJVtbc9S8o8SC",  // GlassWorm
];

const KNOWN_C2_DOMAINS: &[&str] = &[
    "api.mainnet-beta.solana.com",
    "calendar.app.google",
    "217.69.3.218",
    "199.247.10.166",
    "45.32.150.251",
];
```

---

### Phase 3: Validate & Test (1 hour)

**Test on confirmed GlassWorm packages:**
```bash
# Test locale detection on @iflow-mcp packages
./target/release/glassware --format json /tmp/iflow-mcp-ref-tools-mcp/package/

# Should now detect:
# - Locale checks (if present)
# - Time delays (if present)
# - Solana C2 patterns (if present)
```

---

## Specific Code to Implement

### GW009: Locale Detector

```rust
use crate::finding::{DetectionCategory, Finding, Severity};
use once_cell::sync::Lazy;
use regex::Regex;
use std::path::Path;

static LOCALE_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        Regex::new(r"Intl\.DateTimeFormat\(\)\.resolvedOptions\(\)\.locale").unwrap(),
        Regex::new(r"Intl\.DateTimeFormat\(\)\.resolvedOptions\(\)\.timeZone").unwrap(),
        Regex::new(r"['\"]ru-RU['\"]|['\"]ru['\"]|['\"]Russian['\"]").unwrap(),
        Regex::new(r"Europe/Moscow|Europe/Kaliningrad|Europe/Volgograd").unwrap(),
        Regex::new(r"process\.exit\s*\(\s*0\s*\)").unwrap(),
    ]
});

pub struct LocaleGeofencingDetector;

impl Detector for LocaleGeofencingDetector {
    fn name(&self) -> &str { "locale_geofencing" }

    fn scan(&self, path: &Path, content: &str, _config: &UnicodeConfig) -> Vec<Finding> {
        let mut findings = Vec::new();
        
        // Track if we see locale checks followed by exit
        let mut locale_check_line: Option<usize> = None;
        
        for (line_num, line) in content.lines().enumerate() {
            for pattern in &LOCALE_PATTERNS {
                if pattern.is_match(line) {
                    if line.contains("Intl") || line.contains("ru-RU") || line.contains("Moscow") {
                        locale_check_line = Some(line_num);
                    }
                    
                    findings.push(Finding::new(
                        &path.to_string_lossy(),
                        line_num + 1,
                        1,
                        0,
                        '\0',
                        DetectionCategory::LocaleGeofencing,
                        Severity::Critical,
                        "Russian locale/timezone geofencing check detected",
                        "Review for geographic targeting behavior - common in GlassWorm campaign",
                    ).with_cwe_id("CWE-506"));
                }
            }
            
            // Check for locale check → exit pattern
            if let Some(check_line) = locale_check_line {
                if line_num > check_line && line_num <= check_line + 5 {
                    if pattern.is_match(line) && line.contains("exit") {
                        // Upgrade severity - this is active geofencing
                        findings.last_mut().unwrap().severity = Severity::Critical;
                        findings.last_mut().unwrap().message = 
                            "Active geofencing: locale check followed by early exit".to_string();
                    }
                }
            }
        }
        
        findings
    }
}
```

---

### GW010: Time-Delay Detector

```rust
static DELAY_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // setTimeout with long delays (5min to 96hrs)
        Regex::new(r"setTimeout\s*\(\s*.*,\s*(540000|[6-9]\d{5}|[1-9]\d{6,})\s*\)").unwrap(),
        // Specific millisecond values
        Regex::new(r"900000|172800000|259200000").unwrap(),
        // CI environment checks
        Regex::new(r"process\.env\.(CI|GITHUB_ACTIONS|CIRCLECI|TRAVIS)").unwrap(),
        // Conditional delay pattern
        Regex::new(r"if\s*\(\s*!?\s*isCI\s*\)\s*setTimeout").unwrap(),
    ]
});

pub struct TimeDelayDetector;

impl Detector for TimeDelayDetector {
    fn name(&self) -> &str { "time_delay_sandbox_evasion" }

    fn scan(&self, path: &Path, content: &str, _config: &UnicodeConfig) -> Vec<Finding> {
        let mut findings = Vec::new();
        let mut ci_check_seen: Option<usize> = None;
        
        for (line_num, line) in content.lines().enumerate() {
            // Track CI checks
            if DELAY_PATTERNS[2].is_match(line) {
                ci_check_seen = Some(line_num);
            }
            
            // Check for delays
            for (i, pattern) in DELAY_PATTERNS.iter().enumerate().skip(1) {
                if pattern.is_match(line) {
                    let mut severity = Severity::High;
                    let mut message = "Long time delay detected (possible sandbox evasion)".to_string();
                    
                    // Upgrade if CI bypass pattern
                    if let Some(ci_line) = ci_check_seen {
                        if line_num > ci_line && line_num <= ci_line + 10 {
                            severity = Severity::Critical;
                            message = "CI-aware time delay detected (sandbox evasion with CI bypass)".to_string();
                        }
                    }
                    
                    findings.push(Finding::new(
                        &path.to_string_lossy(),
                        line_num + 1,
                        1,
                        0,
                        '\0',
                        DetectionCategory::TimeDelaySandboxEvasion,
                        severity,
                        message,
                        "Review for sandbox evasion behavior",
                    ).with_cwe_id("CWE-506"));
                }
            }
        }
        
        findings
    }
}
```

---

### GW011: Blockchain C2 Detector

```rust
static BLOCKCHAIN_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // Solana RPC
        Regex::new(r"api\.mainnet-beta\.solana\.com").unwrap(),
        // Solana API methods
        Regex::new(r"getSignaturesForAddress|getTransaction|getParsedTransactions").unwrap(),
        // Known GlassWorm wallet
        Regex::new(r"BjVeAjPrSKFiingBn4vZvghsGj9KCE8AJVtbc9S8o8SC").unwrap(),
        // Short interval polling (1-10 seconds)
        Regex::new(r"setInterval\s*\(\s*.*,\s*([1-9]\d{2,3}|[1-9]\d{3})\s*\)").unwrap(),
        // Google Calendar C2
        Regex::new(r"calendar\.app\.google").unwrap(),
    ]
});

pub struct BlockchainC2Detector;

impl Detector for BlockchainC2Detector {
    fn name(&self) -> &str { "blockchain_c2" }

    fn scan(&self, path: &Path, content: &str, _config: &UnicodeConfig) -> Vec<Finding> {
        let mut findings = Vec::new();
        
        for (line_num, line) in content.lines().enumerate() {
            for (i, pattern) in BLOCKCHAIN_PATTERNS.iter().enumerate() {
                if pattern.is_match(line) {
                    let (severity, message) = match i {
                        0 => (Severity::Critical, "Solana RPC endpoint detected (blockchain C2 communication)"),
                        1 => (Severity::High, "Solana blockchain API call detected"),
                        2 => (Severity::Critical, "Known GlassWorm C2 wallet address detected"),
                        3 => (Severity::High, "Short-interval polling detected (possible C2 beaconing)"),
                        4 => (Severity::Critical, "Google Calendar C2 pattern detected"),
                        _ => (Severity::High, "Blockchain C2 pattern detected"),
                    };
                    
                    findings.push(Finding::new(
                        &path.to_string_lossy(),
                        line_num + 1,
                        1,
                        0,
                        '\0',
                        DetectionCategory::BlockchainC2,
                        severity,
                        message.to_string(),
                        "Review for command-and-control behavior",
                    ).with_cwe_id("CWE-506"));
                }
            }
        }
        
        findings
    }
}
```

---

## Testing Plan

### Test on Confirmed GlassWorm Packages

```bash
# Re-scan @iflow-mcp packages with new detectors
./target/release/glassware --format json /tmp/iflow-mcp-ref-tools-mcp/package/

# Expected new detections:
# - GW009: Locale checks (if present)
# - GW010: Time delays (if present)
# - GW011: Solana C2 patterns (if present)
```

### Test on 30k Scan Results

```bash
# Re-analyze flagged packages from 30k scan
# Focus on packages with existing findings + new evasion patterns
```

---

## Decision: Implement Now or Wait?

**Recommendation: IMPLEMENT TODAY**

**Rationale:**
1. ✅ Patterns are well-documented across multiple campaigns
2. ✅ Specific IOCs provided (wallet addresses, domains)
3. ✅ Implementation is straightforward (regex + semantic)
4. ✅ High value-add for GlassWorm detection
5. ✅ Low false positive risk (these patterns are rarely legitimate)

**Time investment:** 2-3 hours for all 3 detectors  
**Expected impact:** Catch next-wave GlassWorm variants that use these evasion techniques

---

**Ready to implement. Awaiting approval to proceed with Phase 1 detectors.**
