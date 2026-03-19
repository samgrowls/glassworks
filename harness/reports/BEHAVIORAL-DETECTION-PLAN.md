# Behavioral Detection Enhancement Plan

**Date:** 2026-03-19  
**Status:** Design proposal  
**Priority:** High (parallel to 30k scan)  

---

## Phase 1: Basic Behavioral Detectors (Implement Today)

### 1. Russian Locale Exclusion Detector

**Pattern to detect:**
```javascript
// Common patterns from intel
if (locale === 'ru_RU' || locale === 'ru-RU') return;
if (timezone.includes('Moscow')) return;
if (navigator.language === 'ru') return;
```

**Implementation:**
```rust
// glassware-core/src/locale_detector.rs
pub struct LocaleExclusionDetector;

impl Detector for LocaleExclusionDetector {
    fn scan(&self, path: &Path, content: &str, _config: &UnicodeConfig) -> Vec<Finding> {
        let mut findings = Vec::new();
        
        // Russian locale checks
        let patterns = [
            r"['\"]ru_RU['\"]",
            r"['\"]ru-RU['\"]",
            r"['\"]Russian['\"]",
            r"navigator\.language\s*[=!]+\s*['\"]ru",
            r"timezone.*Moscow",
        ];
        
        for (line_num, line) in content.lines().enumerate() {
            for pattern in &patterns {
                if pattern.is_match(line) {
                    findings.push(Finding::new(
                        path,
                        line_num + 1,
                        0,
                        0,
                        '\0',
                        DetectionCategory::LocaleExclusion,
                        Severity::High,
                        "Russian locale exclusion check detected",
                        "Review for geographic targeting behavior",
                    ));
                }
            }
        }
        
        findings
    }
}
```

**Effort:** 1-2 hours  
**Impact:** Catch GlassWare geographic targeting

---

### 2. Time-Based Delay Detector

**Pattern to detect:**
```javascript
// 15-minute delay to evade sandboxing
setTimeout(() => { /* malicious code */ }, 900000);
await sleep(900000);
```

**Implementation:**
```rust
// glassware-core/src/delay_detector.rs
pub struct TimeDelayDetector;

impl Detector for TimeDelayDetector {
    fn scan(&self, path: &Path, content: &str, _config: &UnicodeConfig) -> Vec<Finding> {
        let mut findings = Vec::new();
        
        // Look for long delays (>10 minutes = 600000ms)
        let delay_pattern = r"(?:setTimeout|sleep|wait)\s*\(\s*(\d{6,})";
        
        for (line_num, line) in content.lines().enumerate() {
            if let Some(caps) = delay_pattern.captures(line) {
                if let Some(delay_ms) = caps.get(1) {
                    if let Ok(delay) = delay_ms.as_str().parse::<u64>() {
                        if delay >= 600000 {  // 10+ minutes
                            findings.push(Finding::new(
                                path,
                                line_num + 1,
                                0,
                                0,
                                '\0',
                                DetectionCategory::TimeDelay,
                                Severity::Medium,
                                format!("Long delay detected: {}ms (possible sandbox evasion)", delay),
                                "Review for sandbox evasion behavior",
                            ));
                        }
                    }
                }
            }
        }
        
        findings
    }
}
```

**Effort:** 1-2 hours  
**Impact:** Catch sandbox evasion

---

### 3. Sandbox/VM Detection

**Pattern to detect:**
```javascript
// VM detection
if (process.env.CI) return;
if (process.env.TRAVIS) return;
if (debuggerAttached) return;
```

**Implementation:**
```rust
// glassware-core/src/sandbox_detector.rs
pub struct SandboxDetectionDetector;

impl Detector for SandboxDetectionDetector {
    fn scan(&self, path: &Path, content: &str, _config: &UnicodeConfig) -> Vec<Finding> {
        let mut findings = Vec::new();
        
        let patterns = [
            r"process\.env\.(?:CI|TRAVIS|JENKINS|CIRCLECI)",
            r"debuggerAttached",
            r"isDebugging\(\)",
            r"process\.argv\.includes.*debug",
        ];
        
        for (line_num, line) in content.lines().enumerate() {
            for pattern in &patterns {
                if pattern.is_match(line) {
                    findings.push(Finding::new(
                        path,
                        line_num + 1,
                        0,
                        0,
                        '\0',
                        DetectionCategory::SandboxDetection,
                        Severity::Medium,
                        "Sandbox/VM detection code detected",
                        "Review for sandbox evasion behavior",
                    ));
                }
            }
        }
        
        findings
    }
}
```

**Effort:** 1 hour  
**Impact:** Catch sandbox evasion

---

## Phase 2: Advanced Behavioral Detectors (After Expert Intel)

### 4. Multi-Stage Loader Detector

**Pattern:** Decrypt → Download → Decrypt → Exec chains

**Implementation:** Requires expert intel on specific patterns

### 5. C2 Communication Detector

**Patterns:**
- Solana memo queries
- Google Calendar API calls with specific patterns
- HTTP header extraction for decryption keys

**Implementation:** Requires expert intel on current C2 infrastructure

### 6. Dormant/Hibernation Trigger Detector

**Pattern:** Conditional execution based on:
- Date/time triggers
- Download count thresholds
- Specific environment variables

**Implementation:** Requires expert intel on trigger mechanisms

---

## Integration Plan

### Step 1: Register New Detectors

```rust
// glassware-core/src/engine.rs
impl ScanEngine {
    pub fn default_detectors() -> Self {
        let mut engine = Self::new();
        
        // Existing detectors
        engine.register(Box::new(UnicodeDetector::new()));
        engine.register(Box::new(EncryptedPayloadDetector::new()));
        
        // NEW: Behavioral detectors
        engine.register(Box::new(LocaleExclusionDetector::new()));
        engine.register(Box::new(TimeDelayDetector::new()));
        engine.register(Box::new(SandboxDetectionDetector::new()));
        
        engine
    }
}
```

### Step 2: Add Detection Categories

```rust
// glassware-core/src/finding.rs
pub enum DetectionCategory {
    // Existing...
    LocaleExclusion,
    TimeDelay,
    SandboxDetection,
}
```

### Step 3: Test on Known Malicious

```bash
# Test on confirmed GlassWare packages
./target/release/glassware --format json /tmp/iflow-mcp-ref-tools-mcp/package/

# Should now detect:
# - Russian locale checks (if present)
# - Time delays (if present)
# - Sandbox detection (if present)
```

---

## Timeline

| Phase | Tasks | ETA |
|-------|-------|-----|
| **Phase 1** | Basic detectors (locale, delay, sandbox) | 3-5 hours |
| **Expert Intel** | Wait for behavioral IOCs | 24-48 hours |
| **Phase 2** | Advanced detectors (multi-stage, C2, dormant) | 4-6 hours after intel |
| **Integration** | Register detectors, test, deploy | 1-2 hours |

---

## Recommendation

**Do both in parallel:**

1. **Start Phase 1 now** (3-5 hours) - Implement basic detectors
2. **Request expert intel** - Get behavioral IOCs within 24-48h
3. **Complete Phase 2** - Implement advanced detectors with expert intel

**Total investment:** ~10 hours for comprehensive behavioral detection

---

**Ready to implement Phase 1 immediately. Awaiting decision on expert intel request.**
