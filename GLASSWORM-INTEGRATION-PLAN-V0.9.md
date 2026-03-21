# GlassWorm Intelligence Integration Plan — v0.9.0 Sprint

**Date:** 2026-03-21  
**Target Release:** v0.9.0.0  
**Timeline:** 4 weeks (Mar 24 - Apr 25)  
**Priority:** CRITICAL

---

## Executive Summary

Based on comprehensive review of GlassWorm intelligence (Parts 1-5, newintel.md audit), this plan addresses:

- **11 capability gaps** (G1-G11) between current detection and GlassWorm TTPs
- **5 enhancements** (E1-E5) to existing capabilities
- **Severity scoring overhaul** for context-aware risk assessment
- **Binary scanning foundation** for `.node` file analysis

**Key Insight:** The biggest ROI is **binary scanning** (Phase 2) — it unlocks detection of the actual native payloads where GlassWorm logic lives, not just the JS wrappers. Rust + `goblin` crate makes this tractable in 5-8 days.

---

## Current State Assessment

### ✅ What We Have (Verified)

**18 Detectors:**
- 13 Tier 1-3 (L1 regex + L3 behavioral)
- 4 Semantic (L2 flow analysis)
- 1 LLM analyzer (L3)

**7 Subsystems:**
- Attack graph correlation
- Campaign intelligence
- Cross-file taint tracking
- Tiered detection engine
- Unified IR
- Risk scoring (basic)
- Adversarial testing

### ❌ What's Missing (11 Gaps)

| Gap | Description | Priority | Effort |
|-----|-------------|----------|--------|
| **G1** | Filesystem IoC scanner (`jucku/`, `myextension/`) | P0 | 3 days |
| **G2** | Chrome Secure Prefs forensic (`location:4`, `creation_flags:38`) | P0 | 2 days |
| **G3** | Typo attribution strings (`Invlaid`, `LoadLibararyFail`) | P1 | 1 day |
| **G4** | Exfil JSON schema matcher | P1 | 2 days |
| **G5** | Socket.IO / TCP tunnel C2 | P1 | 2 days |
| **G6** | XorShift obfuscation signature | P2 | 3 days |
| **G7** | IElevator COM CLSID detection | P2 | 1 day |
| **G8** | APC injection signatures | P2 | 2 days |
| **G9** | memexec detection | P3 | 1 day |
| **G10** | Solana Memo decoder (query chain) | P3 | 3 days |
| **G11** | `.node` binary metadata extraction | P3 | 2 days |

### ⚠️ Enhancements Needed (5 Items)

| Enhancement | Description | Priority | Effort |
|-------------|-------------|----------|--------|
| **E1** | Wallet/IP list completeness | P0 | 0.5 days |
| **E2** | `creation_flags`/`location` heuristic | P1 | 1 day |
| **E3** | Browser-kill pattern detection | P1 | 1 day |
| **E4** | PhantomRaven auto-matching (126 pkg list) | P2 | 2 days |
| **E5** | YARA rule export | P2 | 2 days |

---

## Severity Scoring Overhaul

### Current System

```rust
// Fixed severity per detector
Severity::Critical  // Always critical regardless of context
Severity::High
Severity::Medium
Severity::Low
```

**Problem:** A Critical finding in a 5-year-old package from a reputable author should be treated differently than the same finding in a 3-day-old package from an anonymous author.

### Proposed System: Context-Aware Severity

```rust
// Base severity from detector
let base_severity = detector.detect(...).severity;  // Critical, High, Medium, Low

// Context multipliers
let multiplier = SeverityContext {
    package_age_days: 3,           // ×2.0 (very new)
    author_reputation: Unknown,     // ×1.0
    ecosystem: GitHub,              // ×1.5 (less scrutiny)
    has_install_script: true,       // ×1.3
    has_network_calls: true,        // ×1.3
    has_eval_usage: true,           // ×1.5
    rapid_version_changes: true,    // ×1.2
}.calculate_multiplier();           // Total: ~7.0x

// Adjusted severity
let adjusted_severity = base_severity × multiplier;
// Critical × 7.0 = EXTREME (new severity level)
```

### Severity Levels (New)

| Level | Score Range | Action |
|-------|-------------|--------|
| **INFO** | 0.0-0.5 | Log only |
| **LOW** | 0.5-2.0 | Monitor |
| **MEDIUM** | 2.0-4.0 | Review |
| **HIGH** | 4.0-6.0 | Block + alert |
| **CRITICAL** | 6.0-8.0 | Block + immediate review |
| **EXTREME** | 8.0-10.0 | Block + auto-quarantine + escalate |

### Context Multipliers

```rust
pub struct SeverityContext {
    // Package metadata (from npm API)
    pub package_age_days: u32,
    pub author_reputation: ReputationScore,
    pub download_count: u64,
    pub ecosystem: Ecosystem,
    
    // Behavioral indicators (from static analysis)
    pub has_install_script: bool,
    pub has_network_calls: bool,
    pub has_eval_usage: bool,
    pub has_decrypt_usage: bool,
    pub has_obfuscation: bool,
    
    // Historical data (from scan history)
    pub previous_versions_clean: bool,
    pub rapid_version_changes: bool,
    pub typosquat_of_popular: Option<String>,
}

impl SeverityContext {
    pub fn calculate_multiplier(&self) -> f64 {
        let mut multiplier = 1.0;
        
        // Package age (strongest signal)
        multiplier *= match self.package_age_days {
            0..=7 => 2.0,      // Very new
            8..=30 => 1.5,     // Relatively new
            31..=90 => 1.2,    // Established
            _ => 1.0,          // Well-established
        };
        
        // Author reputation
        multiplier *= match self.author_reputation {
            ReputationScore::KnownBad => 3.0,
            ReputationScore::Suspicious => 2.0,
            ReputationScore::Unknown => 1.0,
            ReputationScore::KnownGood => 0.5,
        };
        
        // Ecosystem
        multiplier *= match self.ecosystem {
            Ecosystem::GitHub => 1.5,   // Less scrutiny
            Ecosystem::Npm => 1.0,      // Standard
            Ecosystem::PyPI => 1.2,     // Medium risk
        };
        
        // Behavioral combinations (multiplicative)
        if self.has_install_script && self.has_network_calls {
            multiplier *= 2.0;  // High-risk combo
        }
        if self.has_eval_usage {
            multiplier *= 1.5;
        }
        if self.has_decrypt_usage && self.has_network_calls {
            multiplier *= 2.5;  // Decrypt + exfil pattern
        }
        
        // Historical
        if !self.previous_versions_clean {
            multiplier *= 1.5;  // Previous versions had issues
        }
        if self.rapid_version_changes {
            multiplier *= 1.3;  // >3 versions in 7 days
        }
        
        // Typosquatting
        if self.typosquat_of_popular.is_some() {
            multiplier *= 3.0;  // Strong indicator
        }
        
        multiplier.min(10.0)  // Cap at 10x
    }
}
```

### Implementation Files

**New:**
- `glassware-core/src/severity_context.rs` (300 lines)
- `glassware-core/src/reputation.rs` (150 lines)

**Modified:**
- `glassware-core/src/finding.rs` - Add `adjusted_severity` field
- `glassware-core/src/risk_scorer.rs` - Integrate context
- `glassware-orchestrator/src/scanner.rs` - Pass context to findings

---

## Implementation Phases

### Phase 0: Trivial Wins (1-2 days)

**Scope:** E1, E3 — pure data updates

#### E1: Update IoC Lists

**Files:** `glassware-core/src/blockchain_c2_detector.rs`, `glassware-core/src/campaign.rs`

**Work:**
```rust
// Add to HARDCODED_WALLETS
"DSRUBTz..."  // Chrome RAT wallet from Part 4

// Add to HARDCODED_C2_IPS
"104.238.191.54",
"108.61.208.161",
"45.150.34.158",  // led-win32 exfil server
```

**Test:** Verify detectors flag these values

#### E3: Browser-Kill Patterns

**Files:** `glassware-core/src/time_delay_detector.rs` (or new `behavioral_detector.rs`)

**Work:**
```rust
const BROWSER_KILL_PATTERNS: &[&str] = &[
    "taskkill /F /IM chrome.exe",
    "taskkill /F /IM msedge.exe",
    "taskkill /F /IM brave.exe",
    "pkill -9 -f \"Google Chrome\"",
    "pkill -9 -f \"Microsoft Edge\"",
    "Stop-Process -Name chrome -Force",
];
```

**Test:** Match against Part 5 examples

**Deliverables:**
- ✅ IoC lists updated
- ✅ Browser-kill patterns added
- ✅ Tests passing

---

### Phase 1: JS-Level Detector Additions (3-5 days)

**Scope:** G3, G4, G5 — new detectors fitting existing model

#### G3: Typo Attribution Strings

**File:** `glassware-core/src/detectors/attribution.rs` (NEW)

**Work:**
```rust
pub struct AttributionDetector;

impl Detector for AttributionDetector {
    fn detect(&self, file: &Path, content: &str) -> Vec<Finding> {
        let mut findings = Vec::new();
        
        const TYPO_FINGERPRINTS: &[(&str, &str)] = &[
            ("Invlaid", "GlassWorm data.dll typo fingerprint"),
            ("LoadLibararyFail", "GlassWorm memexec crate typo"),
            ("NtAllocVmErr", "GlassWorm APC injection typo"),
            ("Corpartion", "GlassWorm led-win32 typo"),
            ("ErorrMessage", "GlassWorm led-win32 typo"),
            ("complite", "GlassWorm extension typo"),
        ];
        
        for (typo, description) in TYPO_FINGERPRINTS {
            if content.contains(typo) {
                findings.push(Finding::new(
                    file,
                    0, 0, 0, '\0',
                    DetectionCategory::AttributionFingerprint,
                    Severity::High,  // High confidence indicator
                    format!("GlassWorm attribution typo found: '{}'", typo),
                    "This typo is a known GlassWorm campaign fingerprint",
                ));
            }
        }
        
        findings
    }
}
```

**Test:** Create fixtures with typo strings

#### G4: Exfil JSON Schema Matcher

**File:** `glassware-core/src/detectors/exfil_schema.rs` (NEW)

**Work:**
```rust
pub struct ExfilSchemaDetector;

impl Detector for ExfilSchemaDetector {
    fn detect(&self, file: &Path, content: &str) -> Vec<Finding> {
        let mut findings = Vec::new();
        
        // Keys from Part 5 exfil JSON schema
        const EXFIL_KEYS: &[&str] = &[
            "cookieCount", "loginCount", "creditCardCount",
            "sync_oauth_token", "send_tab_private_key",
            "app_bound_key", "dpapi_key", "master_key",
        ];
        
        let mut found_keys = Vec::new();
        for key in EXFIL_KEYS {
            if content.contains(&format!("\"{}\"", key)) {
                found_keys.push(key);
            }
        }
        
        if found_keys.len() >= 3 {
            findings.push(Finding::new(
                file,
                0, 0, 0, '\0',
                DetectionCategory::ExfiltrationSchema,
                Severity::Critical,
                format!("Exfiltration JSON schema detected ({} keys: {:?})", 
                       found_keys.len(), found_keys),
                "Matches GlassWorm exfiltration output format",
            ));
        }
        
        findings
    }
}
```

**Test:** Create fixture with exfil schema JSON

#### G5: Socket.IO / TCP Tunnel C2

**File:** `glassware-core/src/blockchain_c2_detector.rs` (EXTEND)

**Work:**
```rust
// Add to existing BlockchainC2Detector
const SOCKET_IO_PATTERNS: &[&str] = &[
    "socket.io-client",
    "io(", "socket(",
    ":5000",  // GlassWorm C2 port
    "tunnel", "proxy",
];

// Add detection logic similar to blockchain patterns
```

**Test:** Create fixture with Socket.IO C2 pattern

**Deliverables:**
- ✅ G3 attribution detector (typo fingerprints)
- ✅ G4 exfil schema detector
- ✅ G5 Socket.IO C2 detection
- ✅ All tests passing

---

### Phase 2: Binary Scanning Foundation (5-8 days)

**Scope:** Binary scanner module + G6, G7, G8, G9, G11

**This is the highest-ROI phase** — unlocks detection of native payloads.

#### Day 1-2: Foundation

**File:** `glassware-core/src/binary/mod.rs` (NEW)

**Dependencies:**
```toml
[dependencies]
goblin = "0.7"      # PE/ELF/Mach-O parser
memmap2 = "0.9"     # Zero-copy memory mapping
```

**Core Extractor:**
```rust
pub struct BinaryExtractor;

pub struct BinaryFeatures {
    pub format: BinaryFormat,          // PE / ELF / MachO
    pub imports: Vec<ImportEntry>,     // function imports
    pub exports: Vec<ExportEntry>,     // function exports
    pub strings: Vec<ExtractedString>, // printable strings
    pub sections: Vec<SectionInfo>,    // name, entropy, size
    pub debug_info: Option<DebugInfo>, // PDB path, Cargo paths
}

impl BinaryExtractor {
    pub fn extract(&self, bytes: &[u8]) -> Result<BinaryFeatures> {
        // Use goblin to parse format
        // Extract strings (printable ASCII/UTF-8, min 4 chars)
        // Calculate section entropy
        // Extract debug info
    }
}
```

**Orchestrator Integration:**
```rust
// In scanner.rs
if path.extension() == Some("node") {
    let features = BinaryExtractor::extract(&bytes)?;
    let findings = BinaryScanner::scan(&features)?;
    return findings;
}
```

#### Day 3-5: Detectors

**G11: Metadata Extraction** (simplest, validates pipeline)
```rust
// Look for: PDB paths, Cargo registry paths, internal DLL names
if features.debug_info.pdb_path.contains("Administrator") {
    // Flag: built on shared Windows admin account
}
if features.strings.iter().any(|s| s.contains("neon-")) {
    // Flag: Neon framework (Node.js native addon)
}
```

**G9: memexec Detection**
```rust
if features.strings.iter().any(|s| s.contains("memexec 0.2.0")) {
    // Flag: memexec crate for fileless PE loading
}
```

**G7: IElevator CLSID**
```rust
const IELEVATOR_CLSIDS: &[&str] = &[
    "{BDB57FF2-79B9-420C-985A-71CAB25D78A8}",  // Chrome
    "{1F8B4C0E-79B9-420C-985A-71CAB25D78A8}",  // Edge (partial match)
];

if features.strings.iter().any(|s| IELEVATOR_CLSIDS.contains(&s.as_str())) {
    // Flag: IElevator COM for App-Bound key extraction
}
```

**G8: APC Injection**
```rust
const APC_IMPORTS: &[&str] = &[
    "NtAllocateVirtualMemory",
    "NtProtectVirtualMemory",
    "NtQueueApcThread",
    "NtCreateThreadEx",
];

if features.imports.iter().any(|i| APC_IMPORTS.contains(&i.name.as_str())) {
    // Flag: APC injection imports
}
```

**G6: XorShift Obfuscation** (most complex)
```rust
// Byte-level pattern scan for xorshift constants
const XORSHIFT_PATTERN: &[u8] = &[0x41, 0x69, 0x2E, 0x27, 0x62, 0x6C];

// Also check for high-entropy sections (>7.0 bits/byte)
if features.sections.iter().any(|s| s.entropy > 7.0) {
    // Flag: packed/encrypted section
}
```

#### Day 6-8: Integration + Testing

**Test Fixtures:**
- Create mock `.node` files with known features
- Test against actual GlassWorm samples if available
- Verify findings flow into attack graph correctly

**Cross-Reference:**
```rust
// If JS has: require('./build/Release/addon.node')
// AND .node has: APC injection imports
// THEN: Elevated correlation score
```

**Deliverables:**
- ✅ Binary scanner module
- ✅ G6-G9, G11 detectors
- ✅ Integration tests
- ✅ Attack graph cross-reference

---

### Phase 3: Host Forensics Mode (5-7 days)

**Scope:** G1, G2, E2 — new scanning mode (incident response)

**Key Decision:** This is a **separate subcommand**, not part of package scanning.

#### G1: Filesystem IoC Scanner

**CLI:**
```bash
glassware host-scan --path / --chrome-prefs --output report.json
```

**Implementation:**
```rust
pub struct HostScanner {
    ioc_patterns: Vec<IocPattern>,
}

struct IocPattern {
    path_pattern: String,      // e.g., "**/jucku/**"
    description: String,
    severity: Severity,
}

impl HostScanner {
    pub fn scan(&self, root_path: &Path) -> Result<HostScanReport> {
        let mut findings = Vec::new();
        
        // Walk filesystem
        for entry in WalkDir::new(root_path) {
            let path = entry?.path();
            
            // Check for IoC directories
            if path.to_string_lossy().contains("jucku/") {
                findings.push(HostFinding::new(
                    path,
                    "GlassWorm working directory (jucku/)",
                    Severity::Critical,
                ));
            }
            
            // Check for myextension/ in Chrome paths
            if path.to_string_lossy().contains("myextension/") {
                findings.push(HostFinding::new(
                    path,
                    "GlassWorm extension directory (myextension/)",
                    Severity::Critical,
                ));
            }
        }
        
        Ok(HostScanReport { findings, .. })
    }
}
```

#### G2 + E2: Chrome Secure Prefs Forensic

**Implementation:**
```rust
pub struct ChromePrefsScanner;

impl ChromePrefsScanner {
    pub fn scan_secure_prefs(&self, prefs_path: &Path) -> Result<Vec<Finding>> {
        let content = fs::read_to_string(prefs_path)?;
        let prefs: Value = serde_json::from_str(&content)?;
        
        let mut findings = Vec::new();
        
        // Check extensions
        if let Some(extensions) = prefs.get("extensions").and_then(|e| e.as_object()) {
            for (ext_id, ext_data) in extensions {
                let location = ext_data.get("location").and_then(|v| v.as_u64());
                let creation_flags = ext_data.get("creation_flags").and_then(|v| v.as_u64());
                let from_webstore = ext_data.get("from_webstore").and_then(|v| v.as_bool());
                
                // Heuristic: location=4 (sideloaded) + creation_flags=38
                if location == Some(4) && creation_flags == Some(38) {
                    findings.push(Finding::new(
                        prefs_path,
                        0, 0, 0, '\0',
                        DetectionCategory::ChromeSideload,
                        Severity::Critical,
                        format!("Sideloaded extension with suspicious flags: {}", ext_id),
                        "location=4 + creation_flags=38 is GlassWorm signature",
                    ));
                }
                
                // from_webstore=false is also suspicious
                if from_webstore == Some(false) {
                    findings.push(Finding::new(
                        prefs_path,
                        0, 0, 0, '\0',
                        DetectionCategory::ChromeSideload,
                        Severity::High,
                        format!("Extension not from webstore: {}", ext_id),
                        "Manual review recommended",
                    ));
                }
            }
        }
        
        findings
    }
}
```

**Deliverables:**
- ✅ `host-scan` subcommand
- ✅ G1 filesystem IoC scanner
- ✅ G2 Chrome prefs forensic
- ✅ E2 creation_flags heuristic
- ✅ Cross-platform path handling

---

### Phase 4: Advanced Capabilities (5-8 days)

**Scope:** G10, E4, E5

#### G10: Solana Memo Decoder

**File:** `glassware-core/src/solana_decoder.rs` (NEW)

**Dependencies:**
```toml
[dependencies]
reqwest = { version = "0.11", features = ["json"] }
serde_json = "1.0"
```

**Implementation:**
```rust
pub struct SolanaMemoDecoder {
    rpc_endpoint: String,
    rate_limiter: RateLimiter,
}

impl SolanaMemoDecoder {
    pub fn new(rpc_endpoint: Option<String>) -> Self {
        Self {
            rpc_endpoint: rpc_endpoint.unwrap_or_else(|| 
                "https://api.mainnet-beta.solana.com".to_string()),
            rate_limiter: RateLimiter::new(10),  // 10 req/min
        }
    }
    
    pub async fn query_memo(&self, wallet_address: &str) -> Result<Vec<String>> {
        // Query Solana RPC for transactions involving this wallet
        // Extract memo field from each transaction
        // Decode and return C2 URLs
    }
}
```

**CLI:**
```bash
glassware check-chain --wallet DSRUBTz... --rpc https://solana-api.com
```

#### E4: PhantomRaven Auto-Matching

**File:** `glassware-core/src/campaign.rs` (EXTEND)

**Work:**
```rust
// Embed 126-package PhantomRaven list
const PHANTOM_RAVEN_PACKAGES: &[&str] = &[
    "@aifabrix/miso-client",
    "@iflow-mcp/ref-tools-mcp",
    // ... 124 more
];

impl CampaignIntel {
    pub fn is_known_phantom_raven(&self, package_name: &str) -> bool {
        PHANTOM_RAVEN_PACKAGES.contains(&package_name)
    }
    
    pub fn fuzzy_match_typosquat(&self, package_name: &str) -> Option<&str> {
        // Check for typosquats of known packages
        // e.g., "lodahs" → "lodash"
    }
}
```

#### E5: YARA Rule Export

**File:** `glassware-core/src/yara_export.rs` (NEW)

**Implementation:**
```rust
pub struct YaraExporter;

impl YaraExporter {
    pub fn export_rules(&self, detectors: &[&dyn Detector]) -> Result<String> {
        let mut yara = String::new();
        
        yara.push_str("rule GlassWorm_Detectors {\n");
        yara.push_str("  meta:\n");
        yara.push_str("    description = \"GlassWorm campaign detectors\"\n");
        yara.push_str("    author = \"glassware\"\n");
        yara.push_str("  strings:\n");
        
        for detector in detectors {
            for pattern in detector.yara_patterns() {
                yara.push_str(&format!("    {}\n", pattern));
            }
        }
        
        yara.push_str("  condition:\n");
        yara.push_str("    any of them\n");
        yara.push_str("}\n");
        
        Ok(yara)
    }
}
```

**CLI:**
```bash
glassware export-yara --output glassworm-rules.yar
```

**Deliverables:**
- ✅ G10 Solana memo decoder (opt-in)
- ✅ E4 PhantomRaven auto-matching
- ✅ E5 YARA export
- ✅ All tests passing

---

## Timeline Summary

```
Week 1 (Mar 24-28)
├── Phase 0: E1, E3 (1-2 days)
└── Phase 1: G3, G4, G5 (3 days)

Week 2 (Mar 31 - Apr 4)
├── Phase 1 complete (1 day)
└── Phase 2: Binary foundation (4 days)

Week 3 (Apr 7-11)
├── Phase 2: Binary detectors (3 days)
└── Phase 3: Host forensics (2 days)

Week 4 (Apr 14-18)
├── Phase 3 complete (2 days)
└── Phase 4: Advanced capabilities (3 days)

Buffer Week (Apr 21-25)
├── Integration testing
├── Documentation
└── Release v0.9.0.0
```

---

## Testing Strategy

### Unit Tests

Each detector gets:
- True positive test (known GlassWorm pattern)
- False positive test (legitimate code)
- Edge case test (obfuscated/evasion attempt)

### Integration Tests

- End-to-end scan of known malicious package
- End-to-end host scan of infected system (simulated)
- Cross-detector correlation (attack graph)

### Performance Benchmarks

- Binary scanning overhead: <5% total scan time
- Host scan speed: >100 files/sec
- Solana RPC calls: <1s latency (cached)

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| No `.node` samples for testing | Medium | High | Use mock files, request samples from community |
| G6 xorshift has variants | Medium | Medium | Make pattern configurable, tune after testing |
| Host scan cross-platform issues | High | Medium | Test on Windows, macOS, Linux VMs |
| Solana RPC rate limits | Low | Low | Rate limiting, caching, configurable endpoint |
| Severity multipliers too aggressive | Medium | Medium | Conservative defaults, tunable via config |

---

## Success Metrics

| Metric | Current | Target |
|--------|---------|--------|
| True positive rate | 95% | 98%+ |
| False positive rate | 2% | <1% |
| GlassWorm detection | 100% | 100% |
| Binary payload detection | 0% | 95%+ |
| Behavioral chain detection | Limited | Comprehensive |
| Context awareness | None | Full |

---

## Resource Requirements

**Development:**
- 1 Rust developer (full-time, 4 weeks)
- 1 Security researcher (part-time, for threat intel validation)

**Infrastructure:**
- Test VMs (Windows, macOS, Linux)
- Solana RPC access (public or self-hosted)
- GlassWorm sample repository (secure storage)

**Dependencies:**
- `goblin` 0.7 (binary parsing)
- `memmap2` 0.9 (zero-copy mapping)
- `reqwest` 0.11 (HTTP client)

---

## Next Steps

1. ✅ Review and approve this plan
2. ⏳ Set up development environment
3. ⏳ Begin Phase 0 (E1, E3) — 1-2 days
4. ⏳ Weekly check-ins on progress
5. ⏳ Integration testing (Week 4)
6. ⏳ Release v0.9.0.0 (Apr 25)

---

**Ready to begin implementation upon approval.**

---

**End of Plan**
