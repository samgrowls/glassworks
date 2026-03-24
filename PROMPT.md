# 🛠️ GLASSWORKS REMEDIATION PLAYBOOK

## Comprehensive Implementation Guide for Security Scanner Fixes

**Version:** 1.0  
**Created:** March 24, 2025  
**Priority:** CRITICAL  
**Estimated Effort:** 5-7 days  

---

## 📋 TABLE OF CONTENTS

1. [Executive Summary](#1-executive-summary)
2. [Architecture Overview](#2-architecture-overview)
3. [Phase 1: Emergency Whitelist Removal](#3-phase-1-emergency-whitelist-removal)
4. [Phase 2: Detector Logic Fixes](#4-phase-2-detector-logic-fixes)
5. [Phase 3: Scoring System Revision](#5-phase-3-scoring-system-revision)
6. [Phase 4: Evidence Library Expansion](#6-phase-4-evidence-library-expansion)
7. [Phase 5: LLM Integration Enhancement](#7-phase-5-llm-integration-enhancement)
8. [Phase 6: Testing & Validation](#8-phase-6-testing--validation)
9. [Phase 7: Documentation Updates](#9-phase-7-documentation-updates)
10. [Verification Checklist](#10-verification-checklist)
11. [Rollback Procedures](#11-rollback-procedures)

---

## 1. EXECUTIVE SUMMARY

### 1.1 Current State Assessment

| Metric | Current | Target | Gap |
|--------|---------|--------|-----|
| Detection Rate | 50% | ≥90% | -40% |
| False Positive Rate | ~0% | ≤5% | +5% acceptable |
| Evidence Packages | 2 | 20+ | -18 |
| Whitelist Entries | 40+ | 5 (max) | -35 |
| Detector Skip Rules | 15+ | 0 | -15 |

### 1.2 Critical Issues

```
┌─────────────────────────────────────────────────────────────────────────┐
│  ⚠️  CRITICAL: Whitelist strategy fundamentally broken                 │
│  ⚠️  CRITICAL: Detector skip logic prevents real attack detection      │
│  ⚠️  HIGH: Evidence library inadequate for tuning                      │
│  ⚠️  HIGH: Scoring system penalizes single-vector attacks              │
│  ⚠️  MEDIUM: LLM underutilized                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 1.3 Success Criteria

- [ ] All dangerous whitelist entries removed
- [ ] All detector skip logic removed or converted to context-aware detection
- [ ] Evidence library contains 20+ confirmed malicious packages
- [ ] Detection rate ≥90% on evidence library
- [ ] False positive rate ≤5% on clean package sample
- [ ] All tests passing
- [ ] Documentation updated

---

## 2. ARCHITECTURE OVERVIEW

### 2.1 Detection Pipeline

```
┌──────────────┐    ┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│   Package    │───▶│  Downloader  │───▶│   Scanner    │───▶│  ScanEngine  │
│    Input     │    │              │    │              │    │              │
└──────────────┘    └──────────────┘    └──────────────┘    └──────────────┘
                                                                   │
                                                                   ▼
┌──────────────┐    ┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│   Results    │◀───│  LLM Analysis│◀───│ Scoring Engine│◀──│  Detectors   │
│    Output    │    │              │    │              │    │  (Parallel)  │
└──────────────┘    └──────────────┘    └──────────────┘    └──────────────┘
```

### 2.2 Detector Inventory

| Detector | File Path | Current Issue | Fix Priority |
|----------|-----------|---------------|--------------|
| TimeDelay | `src/scanner/detectors/time_delay_detector.rs` | Skips ALL build tools | CRITICAL |
| BlockchainC2 | `src/scanner/detectors/blockchain_c2_detector.rs` | Skips ALL crypto SDKs | CRITICAL |
| InvisibleChar | `src/scanner/detectors/invisible.rs` | Skips ALL /dist/ directories | CRITICAL |
| Steganography | `src/scanner/detectors/steganography.rs` | Needs tuning | HIGH |
| EncryptedPayload | `src/scanner/detectors/encrypted_payload.rs` | Needs tuning | HIGH |
| HeaderC2 | `src/scanner/detectors/header_c2_detector.rs` | Needs review | MEDIUM |

### 2.3 Key Configuration Files

| File | Purpose | Action Required |
|------|---------|-----------------|
| `~/.config/glassware/config.toml` | Global config | Remove dangerous whitelists |
| `campaigns/*/config.toml` | Campaign configs | Remove dangerous whitelists |
| `src/scanner/whitelist.rs` | Whitelist logic | Consider removal |
| `src/scanner/scanner.rs` | Main scanner | Fix scoring, whitelist checks |

---

## 3. PHASE 1: EMERGENCY WHITELIST REMOVAL

**Estimated Time:** 2-4 hours  
**Risk Level:** HIGH (will increase false positives temporarily)  
**Rollback:** Keep backup of all config files

### 3.1 Identify All Whitelist Locations

```bash
# Search for all whitelist references
grep -r "whitelist" --include="*.rs" --include="*.toml" .
grep -r "is_package_whitelisted" --include="*.rs" .
grep -r "crypto_packages" --include="*.toml" .
grep -r "build_tools" --include="*.toml" .
```

### 3.2 Document Current Whitelist Entries

**Action:** Create inventory file before making changes

```bash
# Create backup and inventory
mkdir -p /tmp/glassworks-backup-$(date +%Y%m%d)
cp ~/.config/glassware/config.toml /tmp/glassworks-backup-$(date +%Y%m%d)/
cp campaigns/*/config.toml /tmp/glassworks-backup-$(date +%Y%m%d)/

# Export current whitelist
grep -A 50 "\[whitelist\]" ~/.config/glassware/config.toml > /tmp/glassworks-backup-$(date +%Y%m%d)/whitelist-inventory.txt
```

### 3.3 Remove Dangerous Whitelist Entries

**File:** `~/.config/glassware/config.toml`

**BEFORE (Dangerous):**
```toml
[whitelist]
packages = [
    "ant-design-vue",
    "element-plus", 
    "vuetify",
    "quasar",
    "@angular-devkit/*",
    "webpack",
    "vite",
    "rollup",
]

crypto_packages = [
    "ethers",
    "web3",
    "@azure/*",
    "@aws-sdk/*",
    "@google-cloud/*",
    "@microsoft/*",
]

build_tools = [
    "webpack",
    "vite",
    "rollup",
    "esbuild",
    "@angular-devkit/*",
]
```

**AFTER (Safe):**
```toml
[whitelist]
# CRITICAL: Only whitelist packages with verified legitimate use of detected patterns
# DO NOT whitelist based on popularity or package type

packages = [
    # Legitimate i18n packages that use invisible chars intentionally
    # Must be verified case-by-case
]

# Crypto packages - only legitimate libraries, NOT cloud SDKs
crypto_packages = [
    "ethers",
    "web3",
    "viem",
    "@solana/web3.js",
]

# Build tools - removed (detectors should use context, not skip)
build_tools = []
```

### 3.4 Remove Whitelist Check from Scanner

**File:** `src/scanner/scanner.rs`

**Location:** Around lines 450-520

**BEFORE:**
```rust
fn is_package_whitelisted(&self, package_name: &str) -> bool {
    if self.config.whitelist.packages.iter().any(|p| {
        package_name == p || package_name.starts_with(&p.replace("/*", ""))
    }) {
        return true;
    }
    
    if self.config.whitelist.crypto_packages.iter().any(|p| {
        package_name == p || package_name.starts_with(&p.replace("/*", ""))
    }) {
        return true;
    }
    
    if self.config.whitelist.build_tools.iter().any(|p| {
        package_name == p || package_name.starts_with(&p.replace("/*", ""))
    }) {
        return true;
    }
    
    false
}
```

**AFTER (Option 1 - Remove entirely):**
```rust
// REMOVED: Package-level whitelisting is dangerous for supply chain security
// Attackers target popular packages specifically
// Use context-aware detection instead (see detector fixes in Phase 2)

fn is_package_whitelisted(&self, _package_name: &str) -> bool {
    // Always return false - no package-level whitelisting
    false
}
```

**AFTER (Option 2 - Keep minimal, log warnings):**
```rust
fn is_package_whitelisted(&self, package_name: &str) -> bool {
    // Only check minimal whitelist (verified legitimate patterns)
    // Log all whitelist hits for audit
    if self.config.whitelist.packages.iter().any(|p| package_name == p) {
        warn!("Package {} matched whitelist - verify this is intentional", package_name);
        return true;
    }
    false
}
```

### 3.5 Update Campaign Configurations

**Files:** `campaigns/*/config.toml`

**Action:** Apply same whitelist changes to all campaign configs

```bash
# Find all campaign configs
find campaigns -name "config.toml" -exec grep -l "whitelist" {} \;

# For each file, apply the same changes as 3.3
```

### 3.6 Verification Steps

```bash
# Verify whitelist entries removed
grep -r "ant-design-vue\|element-plus\|vuetify\|quasar" --include="*.toml" .
# Should return: (nothing)

grep -r "@azure/\|@aws-sdk/\|@google-cloud/" --include="*.toml" .
# Should return: (nothing, or only in evidence/test configs)

# Verify scanner still compiles
cargo build --release

# Run quick scan test
cargo run -- scan-npm --package test-package
```

### 3.7 Phase 1 Completion Checklist

- [ ] All config files backed up
- [ ] Current whitelist inventory created
- [ ] Dangerous entries removed from global config
- [ ] Dangerous entries removed from all campaign configs
- [ ] Scanner whitelist function modified
- [ ] Code compiles without errors
- [ ] Quick scan test passes
- [ ] Changes committed with descriptive message

---

## 4. PHASE 2: DETECTOR LOGIC FIXES

**Estimated Time:** 2-3 days  
**Risk Level:** HIGH (will increase detections)  
**Rollback:** Git revert of detector changes

### 4.1 TimeDelay Detector Fix

**File:** `src/scanner/detectors/time_delay_detector.rs`

**Current Issue:** Skips ALL build tools, missing CI bypass + delay evasion

**Location:** Lines 95-105

**BEFORE:**
```rust
fn detect(&self, file_path: &str, content: &str) -> Vec<Finding> {
    let path_lower = file_path.to_lowercase();
    
    // Skip build tools and development dependencies
    if path_lower.contains("@angular") ||
       path_lower.contains("@angular-devkit") ||
       path_lower.contains("webpack") ||
       path_lower.contains("vite") ||
       path_lower.contains("rollup") {
        return findings;  // Returns empty - NO DETECTION
    }
    
    // ... detection logic
}
```

**AFTER (Context-Aware Detection):**
```rust
fn detect(&self, file_path: &str, content: &str) -> Vec<Finding> {
    let path_lower = file_path.to_lowercase();
    let mut findings = Vec::new();
    
    // DO NOT skip based on package name
    // Instead, use context-aware pattern detection
    
    let lines: Vec<&str> = content.lines().collect();
    
    for (line_num, line) in lines.iter().enumerate() {
        let line_lower = line.to_lowercase();
        
        // Pattern 1: CI environment check + delay (HIGH SEVERITY)
        // This is sandbox evasion, not legitimate build tool behavior
        if line_lower.contains("process.env.ci") || 
           line_lower.contains("ci === 'true'") ||
           line_lower.contains("github_actions") ||
           line_lower.contains("travis") ||
           line_lower.contains("jenkins") {
            
            // Check for delay in same file (within 10 lines)
            let start = line_num.saturating_sub(5);
            let end = (line_num + 10).min(lines.len());
            let context: String = lines[start..end].join("\n").to_lowercase();
            
            if context.contains("settimeout") || 
               context.contains("setinterval") ||
               context.contains("sleep") ||
               context.contains("delay") {
                findings.push(Finding {
                    file_path: file_path.to_string(),
                    line_number: line_num + 1,
                    category: DetectionCategory::TimeDelay,
                    severity: Severity::Critical,
                    description: "CI environment check combined with delay - possible sandbox evasion".to_string(),
                    code_snippet: line.trim().to_string(),
                    confidence: 0.85,
                });
                continue;
            }
        }
        
        // Pattern 2: Long delay without CI context (MEDIUM SEVERITY)
        if line_lower.contains("settimeout") || line_lower.contains("setinterval") {
            // Extract delay value
            if let Some(delay_match) = Regex::new(r"set[time|interval]out\s*\(\s*[^,]+,\s*(\d+)").unwrap().captures(line) {
                if let Some(delay_str) = delay_match.get(1) {
                    if let Ok(delay_ms) = delay_str.as_str().parse::<u64>() {
                        // Delays > 30 seconds are suspicious
                        if delay_ms > 30000 {
                            findings.push(Finding {
                                file_path: file_path.to_string(),
                                line_number: line_num + 1,
                                category: DetectionCategory::TimeDelay,
                                severity: Severity::High,
                                description: format!("Long delay detected: {}ms", delay_ms),
                                code_snippet: line.trim().to_string(),
                                confidence: 0.60,
                            });
                        }
                    }
                }
            }
        }
        
        // Pattern 3: Date-based execution (MEDIUM SEVERITY)
        if line_lower.contains("new date()") && 
           (line_lower.contains("settimeout") || line_lower.contains("setinterval")) {
            findings.push(Finding {
                file_path: file_path.to_string(),
                line_number: line_num + 1,
                category: DetectionCategory::TimeDelay,
                severity: Severity::Medium,
                description: "Date-based scheduled execution detected".to_string(),
                code_snippet: line.trim().to_string(),
                confidence: 0.50,
            });
        }
    }
    
    findings
}
```

**Verification:**
```bash
# Test with evidence package containing time delay
cargo run -- scan-npm --package <time-delay-evidence-package>
# Should detect CI bypass + delay pattern
```

### 4.2 BlockchainC2 Detector Fix

**File:** `src/scanner/detectors/blockchain_c2_detector.rs`

**Current Issue:** Skips ALL crypto SDKs, missing real C2 wallet addresses

**Location:** Lines 52-65

**BEFORE:**
```rust
const CRYPTO_PACKAGE_WHITELIST: &[&str] = &[
    "ethers",
    "web3",
    "@azure/",
    "@microsoft/",
    "@aws-sdk/",
    "@google-cloud/",
    // ...
];

fn detect(&self, file_path: &str, content: &str, package_name: &str) -> Vec<Finding> {
    // Skip if package is in whitelist
    if CRYPTO_PACKAGE_WHITELIST.iter().any(|p| {
        package_name == *p || package_name.starts_with(&p.replace("/*", ""))
    }) {
        return findings;  // Returns empty - NO DETECTION
    }
    
    // ... detection logic
}
```

**AFTER (Known C2 Always Flagged):**
```rust
// KNOWN C2 INDICATORS - NEVER WHITELIST
// These are confirmed malicious from security research
const KNOWN_C2_WALLETS: &[&str] = &[
    // Solana wallets (GlassWorm campaign)
    "7nE9GdcnPSzC9X5K9K4...",  // Add actual known C2 wallets
    "9xQeWvG816bUx9EPjH...",
    // Ethereum wallets
    "0x742d35Cc6634C0532925...",
    // Add more from security research
];

const KNOWN_C2_IPS: &[&str] = &[
    // Add known C2 IP addresses
];

fn detect(&self, file_path: &str, content: &str, package_name: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    
    // ALWAYS check for known C2 indicators - no package exemptions
    for (line_num, line) in lines.iter().enumerate() {
        // Check for known C2 wallets (CRITICAL - always flag)
        for wallet in KNOWN_C2_WALLETS {
            if line.contains(wallet) {
                findings.push(Finding {
                    file_path: file_path.to_string(),
                    line_number: line_num + 1,
                    category: DetectionCategory::BlockchainC2,
                    severity: Severity::Critical,
                    description: format!("Known C2 wallet address detected: {}", wallet),
                    code_snippet: line.trim().to_string(),
                    confidence: 0.95,
                });
            }
        }
        
        // Check for known C2 IPs (CRITICAL - always flag)
        for ip in KNOWN_C2_IPS {
            if line.contains(ip) {
                findings.push(Finding {
                    file_path: file_path.to_string(),
                    line_number: line_num + 1,
                    category: DetectionCategory::BlockchainC2,
                    severity: Severity::Critical,
                    description: format!("Known C2 IP address detected: {}", ip),
                    code_snippet: line.trim().to_string(),
                    confidence: 0.95,
                });
            }
        }
        
        // Pattern: Blockchain polling without legitimate context (MEDIUM)
        if line.contains("getBalance") || 
           line.contains("getTokenAccounts") ||
           line.contains("getSignatures") {
            
            // Check for polling pattern (repeated calls in loop)
            let context_start = line_num.saturating_sub(10);
            let context_end = (line_num + 10).min(lines.len());
            let context: String = lines[context_start..context_end].join("\n");
            
            if context.contains("setinterval") || 
               context.contains("while") ||
               context.contains("for") {
                findings.push(Finding {
                    file_path: file_path.to_string(),
                    line_number: line_num + 1,
                    category: DetectionCategory::BlockchainC2,
                    severity: Severity::Medium,
                    description: "Blockchain polling pattern detected".to_string(),
                    code_snippet: line.trim().to_string(),
                    confidence: 0.50,
                });
            }
        }
        
        // Pattern: Transaction signing without user interaction (HIGH)
        if line.contains("signTransaction") || line.contains("signMessage") {
            // Check if there's user confirmation
            let context_start = line_num.saturating_sub(20);
            let context_end = (line_num + 20).min(lines.len());
            let context: String = lines[context_start..context_end].join("\n").to_lowercase();
            
            if !context.contains("confirm") && 
               !context.contains("prompt") &&
               !context.contains("user") {
                findings.push(Finding {
                    file_path: file_path.to_string(),
                    line_number: line_num + 1,
                    category: DetectionCategory::BlockchainC2,
                    severity: Severity::High,
                    description: "Transaction signing without apparent user confirmation".to_string(),
                    code_snippet: line.trim().to_string(),
                    confidence: 0.70,
                });
            }
        }
    }
    
    findings
}
```

**Verification:**
```bash
# Test with evidence package containing blockchain C2
cargo run -- scan-npm --package <blockchain-c2-evidence-package>
# Should detect known C2 wallets
```

### 4.3 InvisibleChar Detector Fix

**File:** `src/scanner/detectors/invisible.rs`

**Current Issue:** Skips ALL /dist/ directories, but that's where malicious code often lives

**Location:** Lines 67-85

**BEFORE:**
```rust
fn detect(&self, file_path: &str, content: &str) -> Vec<Finding> {
    let path_lower = file_path.to_lowercase();
    
    // Skip bundled/minified files
    if path_lower.contains("/dist/") || 
       path_lower.contains("/build/") ||
       path_lower.contains("/bundle/") {
        return findings;  // Returns empty - NO DETECTION
    }
    
    // ... detection logic
}
```

**AFTER (Content-Aware Detection):**
```rust
fn detect(&self, file_path: &str, content: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    
    // DO NOT skip based on directory
    // Malicious code is often in dist/build directories
    
    // Check for invisible Unicode characters
    let invisible_chars: Vec<char> = vec![
        '\u{200B}', // Zero-width space
        '\u{200C}', // Zero-width non-joiner
        '\u{200D}', // Zero-width joiner
        '\u{FEFF}', // Zero-width no-break space (BOM)
        '\u{2060}', // Word joiner
        '\u{2061}', // Function application
        '\u{2062}', // Invisible times
        '\u{2063}', // Invisible separator
        '\u{2064}', // Invisible plus
        '\u{115F}', // Hangul choseong filler
        '\u{1160}', // Hangul jungseong filler
        '\u{3164}', // Hangul filler
        '\u{2800}', // Braille pattern blank
    ];
    
    let mut invisible_found = Vec::new();
    
    for (char_idx, ch) in content.chars().enumerate() {
        if invisible_chars.contains(&ch) {
            invisible_found.push((char_idx, ch));
        }
    }
    
    if invisible_found.is_empty() {
        return findings;
    }
    
    // Count invisible characters
    let invisible_count = invisible_found.len();
    let total_chars = content.chars().count();
    let ratio = invisible_count as f32 / total_chars as f32;
    
    // Check for decoder patterns (indicates steganography)
    let has_decoder = content.contains("atob") ||
                      content.contains("Buffer.from") ||
                      content.contains("decodeURIComponent") ||
                      content.contains("fromCharCode") ||
                      content.contains("String.fromCharCode") ||
                      content.contains("eval(") ||
                      content.contains("Function(");
    
    // Check for suspicious variable names
    let has_suspicious_vars = content.contains("_0x") ||
                               content.contains("var _") ||
                               content.contains("const _");
    
    // Determine severity based on context
    let (severity, confidence, description) = if has_decoder && invisible_count > 10 {
        // Invisible chars + decoder = likely steganography
        (Severity::Critical, 0.90, "Invisible characters with decoder pattern - likely steganography")
    } else if invisible_count > 50 || ratio > 0.01 {
        // High density of invisible chars
        (Severity::High, 0.75, "High density of invisible Unicode characters")
    } else if has_decoder {
        // Decoder without many invisible chars
        (Severity::Medium, 0.60, "Invisible characters with decoder pattern")
    } else if has_suspicious_vars {
        // Obfuscation indicators
        (Severity::Medium, 0.55, "Invisible characters with obfuscated variable names")
    } else {
        // Just invisible chars - could be legitimate i18n
        (Severity::Low, 0.40, "Invisible Unicode characters detected - may be legitimate i18n")
    };
    
    // Only report if severity is at least Medium
    if severity >= Severity::Medium {
        // Find line numbers for report
        let first_invisible_line = content[..invisible_found[0].0].lines().count() + 1;
        
        findings.push(Finding {
            file_path: file_path.to_string(),
            line_number: first_invisible_line,
            category: DetectionCategory::InvisibleChar,
            severity,
            description: description.to_string(),
            code_snippet: format!("{} invisible characters found", invisible_count),
            confidence,
        });
    }
    
    findings
}
```

**Verification:**
```bash
# Test with evidence package containing invisible chars
cargo run -- scan-npm --package <invisible-char-evidence-package>
# Should detect invisible chars + decoder pattern
```

### 4.4 Steganography Detector Review

**File:** `src/scanner/detectors/steganography.rs`

**Action:** Review and ensure it works with InvisibleChar findings

**Key Check:** Ensure steganography detection considers:
- Invisible characters
- Base64 encoded data
- Unusual string patterns
- Decoder chains

### 4.5 EncryptedPayload Detector Review

**File:** `src/scanner/detectors/encrypted_payload.rs`

**Action:** Review and ensure it detects:
- Encrypted strings
- Decryption routines
- Payload execution patterns

### 4.6 Phase 2 Completion Checklist

- [ ] TimeDelay detector updated with context-aware logic
- [ ] BlockchainC2 detector updated to always check known C2
- [ ] InvisibleChar detector updated to check all files
- [ ] Steganography detector reviewed
- [ ] EncryptedPayload detector reviewed
- [ ] All detectors compile without errors
- [ ] Each detector tested with corresponding evidence package
- [ ] Changes committed with descriptive messages

---

## 5. PHASE 3: SCORING SYSTEM REVISION

**Estimated Time:** 1 day  
**Risk Level:** MEDIUM  
**Rollback:** Git revert of scoring changes

### 5.1 Current Scoring Issues

**File:** `src/scanner/scanner.rs`

**Location:** Lines 380-395

**Problem:** Category diversity cap penalizes single-vector attacks

**BEFORE:**
```rust
fn calculate_threat_score(&self, findings: &[Finding], package_name: &str) -> f32 {
    let mut score = 0.0;
    
    // Calculate base score from findings
    for finding in findings {
        score += match finding.severity {
            Severity::Critical => 3.0,
            Severity::High => 2.0,
            Severity::Medium => 1.0,
            Severity::Low => 0.5,
        } * finding.confidence;
    }
    
    // Category diversity scoring
    let categories: HashSet<_> = findings.iter().map(|f| &f.category).collect();
    let category_count = categories.len() as f32;
    
    // CAP SCORES BASED ON CATEGORY COUNT
    if category_count == 1.0 {
        score = score.min(4.0);  // Single category capped
    } else if category_count == 2.0 {
        score = score.min(7.0);  // Two categories capped
    }
    // 3+ categories = no cap (max 10.0)
    
    score
}
```

### 5.2 Revised Scoring with Exceptions

**AFTER:**
```rust
fn calculate_threat_score(&self, findings: &[Finding], package_name: &str) -> f32 {
    let mut score = 0.0;
    
    // Track specific high-severity indicators
    let mut has_known_c2 = false;
    let mut has_critical_invisible_decoder = false;
    let mut has_confirmed_malicious_pattern = false;
    
    // Calculate base score from findings
    for finding in findings {
        let base_score = match finding.severity {
            Severity::Critical => 3.0,
            Severity::High => 2.0,
            Severity::Medium => 1.0,
            Severity::Low => 0.5,
        };
        
        score += base_score * finding.confidence;
        
        // Track exceptional patterns
        if finding.category == DetectionCategory::BlockchainC2 
           && finding.severity == Severity::Critical
           && finding.description.contains("Known C2") {
            has_known_c2 = true;
        }
        
        if finding.category == DetectionCategory::InvisibleChar
           && finding.severity == Severity::Critical
           && finding.description.contains("decoder") {
            has_critical_invisible_decoder = true;
        }
        
        if finding.confidence >= 0.90 && finding.severity == Severity::Critical {
            has_confirmed_malicious_pattern = true;
        }
    }
    
    // EXCEPTION: Known C2 indicators always score high
    if has_known_c2 {
        score = score.max(8.5);
        return score.min(10.0);
    }
    
    // EXCEPTION: Critical invisible + decoder = high score
    if has_critical_invisible_decoder {
        score = score.max(8.0);
        return score.min(10.0);
    }
    
    // EXCEPTION: High confidence critical findings
    if has_confirmed_malicious_pattern {
        score = score.max(7.5);
        return score.min(10.0);
    }
    
    // Category diversity scoring (for non-exceptional cases)
    let categories: HashSet<_> = findings.iter().map(|f| &f.category).collect();
    let category_count = categories.len() as f32;
    
    if category_count == 1.0 {
        score = score.min(5.0);  // Raised from 4.0
    } else if category_count == 2.0 {
        score = score.min(7.5);  // Raised from 7.0
    }
    // 3+ categories = no cap (max 10.0)
    
    score.min(10.0)
}
```

### 5.3 Threshold Adjustments

**File:** `src/scanner/scanner.rs` or config

**Current Thresholds:**
```rust
const MALICIOUS_THRESHOLD: f32 = 7.0;
const SUSPICIOUS_THRESHOLD: f32 = 4.0;
```

**Recommended Thresholds:**
```rust
const MALICIOUS_THRESHOLD: f32 = 7.0;  // Keep same
const SUSPICIOUS_THRESHOLD: f32 = 3.5; // Lowered to catch more
```

### 5.4 Phase 3 Completion Checklist

- [ ] Scoring function updated with exceptions
- [ ] Known C2 exception implemented
- [ ] Invisible + decoder exception implemented
- [ ] High confidence exception implemented
- [ ] Thresholds adjusted
- [ ] Code compiles without errors
- [ ] Evidence packages score correctly
- [ ] Changes committed

---

## 6. PHASE 4: EVIDENCE LIBRARY EXPANSION

**Estimated Time:** 2-3 days  
**Risk Level:** LOW  
**Rollback:** N/A (additive only)

### 6.1 Target Evidence Count

| Category | Target Count | Current | Gap |
|----------|--------------|---------|-----|
| Steganography | 5 | 1 | -4 |
| BlockchainC2 | 5 | 0 | -5 |
| TimeDelay | 4 | 1 | -3 |
| EncryptedPayload | 4 | 0 | -4 |
| HeaderC2 | 2 | 0 | -2 |
| Combined Attacks | 3 | 0 | -3 |
| **Total** | **23** | **2** | **-21** |

### 6.2 Evidence Sources

**Primary Sources (Contact for samples):**
1. **Koi Security** - GlassWorm campaign research
2. **Aikido Security** - Supply chain attack analysis
3. **Socket.dev** - Real-time detection samples
4. **Sonatype** - Supply chain reports
5. **npm Security Advisories** - Official reports
6. **GitHub Security Lab** - Research disclosures

**Secondary Sources (Public):**
1. **malicious-packages npm org** - Quarantined packages
2. **CVE databases** - Supply chain CVEs
3. **Security blog posts** - Attack analyses
4. **Twitter/X security researchers** - Real-time disclosures

### 6.3 Evidence Package Structure

```
evidence/
├── README.md                    # Evidence library documentation
├── steganography/
│   ├── package-001/
│   │   ├── package.json
│   │   ├── src/
│   │   │   └── index.js        # Contains invisible chars + decoder
│   │   └── analysis.md         # Why this is malicious
│   ├── package-002/
│   │   └── ...
│   └── ...
├── blockchain_c2/
│   ├── package-001/
│   │   ├── package.json
│   │   ├── src/
│   │   │   └── index.js        # Contains C2 wallet polling
│   │   └── analysis.md
│   └── ...
├── time_delay/
│   ├── package-001/
│   │   ├── package.json
│   │   ├── src/
│   │   │   └── index.js        # Contains CI bypass + delay
│   │   └── analysis.md
│   └── ...
├── encrypted_payload/
│   ├── package-001/
│   │   ├── package.json
│   │   ├── src/
│   │   │   └── index.js        # Contains encrypted loader
│   │   └── analysis.md
│   └── ...
├── header_c2/
│   └── ...
└── combined_attacks/
    ├── package-001/
    │   ├── package.json
    │   ├── src/
    │   │   └── index.js        # Multiple attack vectors
    │   └── analysis.md
    └── ...
```

### 6.4 Synthetic Evidence Creation

For categories where real samples are unavailable, create synthetic test cases:

**Example: TimeDelay Synthetic Package**
```json
{
  "name": "evidence-time-delay-001",
  "version": "1.0.0",
  "description": "Evidence package: CI bypass + setTimeout evasion",
  "main": "src/index.js"
}
```

```javascript
// src/index.js
// EVIDENCE: CI environment check combined with delay
// This pattern is used to evade sandbox detection

function init() {
    // Check if running in CI environment
    if (process.env.CI === 'true' || process.env.CI === '1') {
        // Delay execution to evade time-based sandbox detection
        setTimeout(() => {
            // Malicious payload would execute here
            console.log("Payload executed after CI check");
        }, 60000); // 60 second delay
    }
}

module.exports = { init };
```

```markdown
# analysis.md

## Attack Pattern: CI Bypass + Time Delay

### Description
This package demonstrates a sandbox evasion technique where:
1. The code checks for CI environment variables
2. If in CI, it delays execution to evade time-limited sandboxes
3. After delay, malicious payload would execute

### Detection Indicators
- `process.env.CI` check
- `setTimeout` with delay > 30 seconds
- Both patterns in same file/function

### Expected Detection
- Category: TimeDelay
- Severity: Critical
- Confidence: 0.85+
- Score: 8.0+

### Source
Synthetic evidence created for Glassworks testing
```

### 6.5 Evidence Validation Script

Create automated validation:

**File:** `tests/evidence_validation.rs`

```rust
#[cfg(test)]
mod evidence_validation {
    use crate::scanner::{Scanner, ScanResult};
    use std::fs;
    use std::path::Path;
    
    #[test]
    fn test_all_evidence_detected() {
        let evidence_dir = Path::new("evidence");
        let mut results = Vec::new();
        
        for category_dir in fs::read_dir(evidence_dir).unwrap() {
            let category_dir = category_dir.unwrap();
            if !category_dir.path().is_dir() {
                continue;
            }
            
            let category = category_dir.file_name().to_string_lossy().to_string();
            if category == "README.md" {
                continue;
            }
            
            for package_dir in fs::read_dir(category_dir.path()).unwrap() {
                let package_dir = package_dir.unwrap();
                if !package_dir.path().is_dir() {
                    continue;
                }
                
                let package_name = package_dir.file_name().to_string_lossy().to_string();
                let result = scan_evidence_package(package_dir.path());
                
                results.push(EvidenceResult {
                    category: category.clone(),
                    package: package_name,
                    detected: result.is_malicious,
                    score: result.score,
                    findings: result.findings.len(),
                });
            }
        }
        
        // Calculate detection rate
        let total = results.len();
        let detected = results.iter().filter(|r| r.detected).count();
        let detection_rate = detected as f32 / total as f32;
        
        println!("Evidence Detection Report:");
        println!("Total packages: {}", total);
        println!("Detected: {}", detected);
        println!("Detection rate: {:.1}%", detection_rate * 100.0);
        
        // Assert minimum detection rate
        assert!(detection_rate >= 0.90, 
            "Detection rate too low: {:.1}% (target: 90%)", 
            detection_rate * 100.0);
        
        // Print missed packages
        for result in results.iter().filter(|r| !r.detected) {
            println!("MISSED: {}/{} (score: {:.2})", 
                result.category, result.package, result.score);
        }
    }
    
    fn scan_evidence_package(path: &Path) -> ScanResult {
        // Implementation using Scanner
        // ...
    }
    
    struct EvidenceResult {
        category: String,
        package: String,
        detected: bool,
        score: f32,
        findings: usize,
    }
}
```

### 6.6 Phase 4 Completion Checklist

- [ ] Evidence directory structure created
- [ ] 5+ steganography evidence packages
- [ ] 5+ blockchain C2 evidence packages
- [ ] 4+ time delay evidence packages
- [ ] 4+ encrypted payload evidence packages
- [ ] 2+ header C2 evidence packages
- [ ] 3+ combined attack evidence packages
- [ ] Each package has analysis.md
- [ ] Evidence validation test created
- [ ] Detection rate ≥90% on evidence library
- [ ] Changes committed

---

## 7. PHASE 5: LLM INTEGRATION ENHANCEMENT

**Estimated Time:** 1-2 days  
**Risk Level:** LOW  
**Rollback:** Git revert of LLM changes

### 7.1 Current LLM Usage

**Issue:** LLM only provides confidence score override

**Location:** `src/llm/` directory

### 7.2 Enhanced LLM Integration

**Proposed Multi-Stage LLM Pipeline:**

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         LLM INTEGRATION PIPELINE                        │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  Stage 1: TRIAGE (Cerebras - Fast, ~2s/pkg)                            │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │ Purpose: Identify obvious false positives                        │   │
│  │ Input: Package name, findings, code snippets                     │   │
│  │ Output: FP probability, skip recommendation                      │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                              │                                          │
│                              ▼                                          │
│  Stage 2: ANALYSIS (NVIDIA - Medium, ~15s/pkg)                         │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │ Purpose: Explain attack chain, assess severity                   │   │
│  │ Input: All findings, full context                                │   │
│  │ Output: Attack explanation, confidence score, severity adjust    │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                              │                                          │
│                              ▼                                          │
│  Stage 3: DEEP DIVE (NVIDIA - Slow, ~30s/pkg)                          │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │ Purpose: Manual review assistance for borderline cases           │   │
│  │ Input: Score 4.0-7.0 packages, full code context                 │   │
│  │ Output: Detailed analysis, remediation suggestions               │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 7.3 LLM Prompt Templates

**Triage Prompt:**
```
You are a supply chain security triage assistant.

Package: {package_name}
Version: {version}
Downloads: {downloads}
Age: {age_days} days

Findings:
{findings_json}

Code Snippets:
{code_snippets}

Task: Assess if this is likely a FALSE POSITIVE.

Consider:
1. Is this a well-known legitimate package?
2. Do the findings match legitimate use patterns?
3. Is the package age and download count consistent with malicious behavior?

Respond with JSON:
{
  "fp_probability": 0.0-1.0,
  "skip_recommendation": true/false,
  "reason": "brief explanation"
}
```

**Analysis Prompt:**
```
You are a supply chain security analyst.

Package: {package_name}
Version: {version}

Findings by Category:
{findings_by_category}

Full Code Context:
{code_context}

Task: Analyze this package for supply chain attack indicators.

Consider:
1. Do the findings form a coherent attack chain?
2. Are there legitimate explanations for the patterns?
3. What is the confidence level of malicious intent?

Respond with JSON:
{
  "attack_chain_explanation": "description of how findings connect",
  "malicious_confidence": 0.0-1.0,
  "severity_recommendation": "Critical/High/Medium/Low",
  "legitimate_explanation": "alternative explanation if any",
  "remediation": "suggested action"
}
```

### 7.4 LLM Configuration Updates

**File:** `~/.config/glassware/config.toml`

```toml
[llm]
# Enable multi-stage LLM pipeline
enabled = true

# Stage 1: Triage (fast)
triage = { provider = "cerebras", model = "llama-3.3-70b", enabled = true }

# Stage 2: Analysis (medium)
analysis = { provider = "nvidia", model = "nemotron-70b", enabled = true }

# Stage 3: Deep Dive (slow, only for borderline)
deep_dive = { provider = "nvidia", model = "nemotron-70b", enabled = true, score_threshold = 4.0 }

# Caching
cache_enabled = true
cache_ttl_hours = 24
```

### 7.5 Phase 5 Completion Checklist

- [ ] LLM pipeline architecture documented
- [ ] Triage prompt template created
- [ ] Analysis prompt template created
- [ ] Deep dive prompt template created
- [ ] LLM configuration updated
- [ ] LLM integration tested with evidence packages
- [ ] LLM responses cached properly
- [ ] Changes committed

---

## 8. PHASE 6: TESTING & VALIDATION

**Estimated Time:** 1-2 days  
**Risk Level:** LOW  
**Rollback:** N/A

### 8.1 Test Suite Updates

**File:** `tests/integration.rs`

```rust
#[cfg(test)]
mod integration {
    use crate::scanner::{Scanner, ScanResult, ThreatLevel};
    
    #[test]
    fn test_evidence_library_detection_rate() {
        // Run evidence validation
        // Target: ≥90% detection rate
    }
    
    #[test]
    fn test_clean_packages_false_positive_rate() {
        // Scan known clean packages
        // Target: ≤5% false positive rate
    }
    
    #[test]
    fn test_known_malicious_packages() {
        // Scan confirmed malicious packages
        // Target: 100% detection, score ≥7.0
    }
    
    #[test]
    fn test_detector_independence() {
        // Each detector should work independently
        // Test each detector in isolation
    }
    
    #[test]
    fn test_scoring_exceptions() {
        // Test known C2 exception
        // Test invisible+decoder exception
        // Test high confidence exception
    }
}
```

### 8.2 Performance Benchmarks

**File:** `benches/scanner_bench.rs`

```rust
use criterion::{criterion_group, criterion_main, Criterion};

fn bench_scan_package(c: &mut Criterion) {
    c.bench_function("scan_10k_loc_package", |b| {
        b.iter(|| {
            // Scan 10k LOC package
        })
    });
    
    c.bench_function("scan_100k_loc_package", |b| {
        b.iter(|| {
            // Scan 100k LOC package
        })
    });
}

criterion_group!(benches, bench_scan_package);
criterion_main!(benches);
```

**Target Performance:**
- 10k LOC: <1 second
- 100k LOC: <10 seconds
- 1M LOC: <2 minutes

### 8.3 Manual Testing Workflow

```bash
# 1. Scan individual evidence packages
for pkg in evidence/*/package-*; do
    echo "Scanning $pkg..."
    cargo run -- scan-npm --path "$pkg" --verbose
done

# 2. Run evidence validation test
cargo test evidence_validation -- --nocapture

# 3. Run full test suite
cargo test --release

# 4. Run performance benchmarks
cargo bench

# 5. Test campaign mode
cargo run -- campaign run --config campaigns/test/config.toml
```

### 8.4 Phase 6 Completion Checklist

- [ ] Integration tests updated
- [ ] Evidence validation test passing
- [ ] Clean package FP test passing
- [ ] Performance benchmarks run
- [ ] Manual testing completed
- [ ] All tests passing
- [ ] Changes committed

---

## 9. PHASE 7: DOCUMENTATION UPDATES

**Estimated Time:** 4-8 hours  
**Risk Level:** LOW  
**Rollback:** N/A

### 9.1 Files to Update

| File | Updates Required |
|------|-----------------|
| `README.md` | Remove critical state warning, update metrics |
| `QWEN.md` | Update detection pipeline, remove whitelist references |
| `HANDOFF/CRITICAL-STATE-MAR24.md` | Mark as resolved, create new status doc |
| `docs/DETECTION.md` | Document all detector patterns |
| `docs/SCORING.md` | Document scoring system with exceptions |
| `docs/EVIDENCE.md` | Document evidence library |
| `docs/LLM.md` | Document LLM integration |

### 9.2 README.md Updates

**BEFORE:**
```markdown
## ⚠️ CRITICAL STATE WARNING

This scanner is currently unusable for production due to...
```

**AFTER:**
```markdown
## Glassworks - Supply Chain Security Scanner

A high-performance Unicode and supply chain attack scanner for npm packages.

### Current Status: ✅ Production Ready

| Metric | Value | Target | Status |
|--------|-------|--------|--------|
| Detection Rate | 92% | ≥90% | ✅ |
| False Positive Rate | 3.5% | ≤5% | ✅ |
| Evidence Coverage | 23 packages | 20+ | ✅ |
| Scan Speed | 55k LOC/sec | 50k LOC/sec | ✅ |

### Quick Start

```bash
# Scan a single package
glassware scan-npm --package <package-name>

# Run a campaign
glassware campaign run --config campaigns/default/config.toml

# Scan with LLM analysis
glassware scan-npm --package <package-name> --llm
```
```

### 9.3 Detection Pipeline Documentation

**File:** `docs/DETECTION.md`

```markdown
# Detection Pipeline

## Overview

Glassworks uses a parallel detector architecture with 6 detection categories:

1. **TimeDelay** - Sandbox evasion via CI bypass + delays
2. **BlockchainC2** - Command & Control via blockchain
3. **InvisibleChar** - Unicode steganography
4. **Steganography** - Hidden data in code
5. **EncryptedPayload** - Encrypted malicious loaders
6. **HeaderC2** - HTTP header-based C2

## Detector Patterns

### TimeDelay

| Pattern | Severity | Confidence |
|---------|----------|------------|
| CI check + setTimeout >30s | Critical | 0.85 |
| setTimeout >30s alone | High | 0.60 |
| Date-based execution | Medium | 0.50 |

### BlockchainC2

| Pattern | Severity | Confidence |
|---------|----------|------------|
| Known C2 wallet | Critical | 0.95 |
| Known C2 IP | Critical | 0.95 |
| Transaction signing without confirmation | High | 0.70 |
| Blockchain polling | Medium | 0.50 |

### InvisibleChar

| Pattern | Severity | Confidence |
|---------|----------|------------|
| Invisible chars + decoder | Critical | 0.90 |
| High density invisible chars | High | 0.75 |
| Invisible chars + obfuscation | Medium | 0.55 |
| Invisible chars alone | Low | 0.40 |
```

### 9.4 Phase 7 Completion Checklist

- [ ] README.md updated
- [ ] QWEN.md updated
- [ ] CRITICAL-STATE-MAR24.md marked resolved
- [ ] DETECTION.md created/updated
- [ ] SCORING.md created/updated
- [ ] EVIDENCE.md created/updated
- [ ] LLM.md created/updated
- [ ] All documentation reviewed
- [ ] Changes committed

---

## 10. VERIFICATION CHECKLIST

### 10.1 Phase Completion Summary

| Phase | Status | Completion Date | Verified By |
|-------|--------|-----------------|-------------|
| Phase 1: Whitelist Removal | ☐ | | |
| Phase 2: Detector Fixes | ☐ | | |
| Phase 3: Scoring Revision | ☐ | | |
| Phase 4: Evidence Library | ☐ | | |
| Phase 5: LLM Enhancement | ☐ | | |
| Phase 6: Testing | ☐ | | |
| Phase 7: Documentation | ☐ | | |

### 10.2 Final Verification Tests

```bash
# 1. Build verification
cargo build --release
# Expected: Success, no warnings

# 2. Unit tests
cargo test --lib
# Expected: All tests passing

# 3. Integration tests
cargo test --test integration
# Expected: All tests passing

# 4. Evidence validation
cargo test evidence_validation
# Expected: Detection rate ≥90%

# 5. Performance benchmark
cargo bench
# Expected: Meets performance targets

# 6. Manual scan test
cargo run -- scan-npm --package <test-package>
# Expected: Completes without errors

# 7. Campaign test
cargo run -- campaign run --config campaigns/test/config.toml
# Expected: Completes without errors
```

### 10.3 Metrics Verification

| Metric | Target | Actual | Pass/Fail |
|--------|--------|--------|-----------|
| Detection Rate | ≥90% | | |
| False Positive Rate | ≤5% | | |
| Evidence Packages | 20+ | | |
| Whitelist Entries | ≤5 | | |
| Detector Skip Rules | 0 | | |
| Test Pass Rate | 100% | | |

### 10.4 Sign-Off

- [ ] All phases completed
- [ ] All tests passing
- [ ] All metrics met
- [ ] Documentation updated
- [ ] Code reviewed
- [ ] Ready for production

**Signed:** ___________________  
**Date:** ___________________

---

## 11. ROLLBACK PROCEDURES

### 11.1 Emergency Rollback

If critical issues are discovered after deployment:

```bash
# 1. Stop any running campaigns
pkill -f glassware

# 2. Revert to previous version
git revert HEAD~7..HEAD  # Revert all remediation commits

# 3. Restore config backups
cp /tmp/glassworks-backup-*/config.toml ~/.config/glassware/

# 4. Rebuild
cargo build --release

# 5. Verify rollback
cargo run -- scan-npm --package test-package
```

### 11.2 Partial Rollback

If only specific phases need rollback:

```bash
# Rollback Phase 1 (Whitelist)
git revert <commit-hash-phase-1>
cp /tmp/glassworks-backup-*/config.toml ~/.config/glassware/

# Rollback Phase 2 (Detectors)
git revert <commit-hash-phase-2>

# Rollback Phase 3 (Scoring)
git revert <commit-hash-phase-3>

# etc.
```

### 11.3 Rollback Verification

```bash
# Verify rollback successful
cargo build --release
cargo test
cargo run -- scan-npm --package test-package

# Check metrics returned to baseline
# (Detection rate will be lower, but system should be stable)
```

---

## APPENDIX A: CONTACT INFORMATION

### Security Research Organizations

| Organization | Contact | Purpose |
|--------------|---------|---------|
| Koi Security | security@koisecurity.io | GlassWorm research, evidence samples |
| Aikido Security | security@aikido.dev | Supply chain analysis |
| Socket.dev | security@socket.dev | Real-time detection samples |
| Sonatype | security@sonatype.org | Supply chain reports |
| npm Security | security@npmjs.com | Official advisories |

### Internal Contacts

| Role | Name | Contact |
|------|------|---------|
| Project Lead | | |
| Security Lead | | |
| DevOps | | |

---

## APPENDIX B: REFERENCE MATERIALS

### B.1 Known Attack Patterns

1. **GlassWorm Campaign** - Unicode steganography + blockchain C2
2. **SolarWinds** - Build tool compromise
3. **Codecov** - CI/CD tool compromise
4. **event-stream** - Supply chain injection
5. **ua-parser-js** - Cryptominer injection

### B.2 Detection Research

1. "Supply Chain Attacks on npm" - Koi Security
2. "Unicode Exploits in JavaScript" - Snyk
3. "Blockchain C2 Patterns" - Aikido Security
4. "Sandbox Evasion Techniques" - Various

### B.3 Tools & Resources

1. npm audit
2. Socket.dev
3. Snyk Advisor
4. GitHub Security Advisories
5. CVE Database

---

## APPENDIX C: GLOSSARY

| Term | Definition |
|------|------------|
| **C2** | Command and Control - attacker infrastructure |
| **FP** | False Positive - legitimate code flagged as malicious |
| **FN** | False Negative - malicious code not detected |
| **LOC** | Lines of Code |
| **i18n** | Internationalization |
| **Steganography** | Hiding data within other data |
| **Sandbox Evasion** | Techniques to avoid detection in analysis environments |

---

**END OF PLAYBOOK**

---

## 📊 ARCHITECTURE DIAGRAM

![Supply Chain Security Scanner Architecture](https://image.qwenlm.ai/public_source/d84b3965-a5b2-4d32-a305-9aff93d42040/167cc7968-2e05-4374-bca1-75b4380e904e.png)

---

**Document Version:** 1.0  
**Last Updated:** March 24, 2025  
**Next Review:** After Phase 7 completion