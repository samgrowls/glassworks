# Phase 2/3 Handoff — Architecture & Scope Guide

**Date:** 2026-03-21  
**Version:** v0.9.1  
**Author:** Primary agent (Phase 1 implementer)  
**For:** Phase 2/3 implementer (Binary Scanning + Host Forensics)

---

## 1. Architecture Overview

### Detection Engine Flow

```
Input (file path + content)
    ↓
FileIR::build() — Unified IR construction
    ├── Pre-parsed JSON (for package.json)
    ├── Pre-parsed AST (for JS/TS, semantic feature)
    ├── Unicode analysis results
    └── File metadata (minified, bundled detection)
    ↓
ScanEngine::scan() — Detector orchestration
    ├── Tier 1: Always run (invisible, homoglyph, bidi)
    ├── Tier 2: Run if Tier 1 finds OR file not minified
    └── Tier 3: Run only if Tier 1+2 find (behavioral)
    ↓
Vec<Finding> — Detection results
    ↓
Attack Graph (optional) — Correlate findings into chains
    ↓
Risk Scoring — Context-aware severity multipliers
    ↓
Report Output — JSON/SARIF/pretty print
```

### Key Types

**`Finding`** (`src/finding.rs`):
```rust
pub struct Finding {
    pub file: String,              // File path
    pub line: usize,               // Line number
    pub column: usize,             // Column
    pub code_point: u32,           // Unicode code point (if applicable)
    pub character: String,         // Character string
    pub raw_bytes: Option<String>, // Raw bytes (if applicable)
    pub category: DetectionCategory, // What was detected
    pub severity: Severity,        // Info/Low/Medium/High/Critical
    pub description: String,       // Human-readable description
    pub remediation: String,       // What to do about it
    pub cwe_id: Option<String>,    // CWE ID
    pub references: Vec<String>,   // Reference URLs
    pub context: Option<String>,   // Additional context
    pub decoded_payload: Option<DecodedPayload>, // If stego payload decoded
    pub confidence: Option<f64>,   // 0.0-1.0 confidence score
}
```

**`DetectionCategory`** (`src/finding.rs`):
```rust
pub enum DetectionCategory {
    InvisibleCharacter,
    Homoglyph,
    BidirectionalOverride,
    UnicodeTag,
    GlasswarePattern,
    SteganoPayload,
    DecoderFunction,
    EncryptedPayload,
    HeaderC2,
    BlockchainC2,
    // ... and more (see finding.rs for full list)
}
```

**`Detector` trait** (`src/detector.rs`):
```rust
pub trait Detector: Send + Sync {
    fn name(&self) -> &str;
    fn tier(&self) -> DetectorTier { DetectorTier::Tier1Primary }
    fn cost(&self) -> u8 { 5 }  // 1-10, lower = cheaper
    fn signal_strength(&self) -> u8 { 5 }  // 1-10, higher = stronger signal
    fn detect(&self, ir: &FileIR) -> Vec<Finding>;
}
```

### Detector Registration

Detectors are registered in `ScanEngine::default_detectors()` (`src/engine.rs`):

```rust
pub fn default_detectors() -> Self {
    let mut engine = Self::new();
    
    // Tier 1
    engine.register(Box::new(UnicodeDetector::new()));
    engine.register(Box::new(EncryptedPayloadDetector::new()));
    engine.register(Box::new(HeaderC2Detector::new()));
    
    // Tier 2
    engine.register(Box::new(RddDetector::new()));
    engine.register(Box::new(JpdAuthorDetector::new()));
    engine.register(Box::new(ForceMemoDetector::new()));
    
    // Tier 3
    engine.register(Box::new(LocaleGeofencingDetector::new()));
    engine.register(Box::new(TimeDelayDetector::new()));
    engine.register(Box::new(BlockchainC2Detector::new()));
    
    // Phase 0 (E3)
    engine.register(Box::new(crate::detectors::browser_kill::BrowserKillDetector::new()));
    
    // Phase 1 (G3, G4, G5)
    engine.register(Box::new(crate::detectors::typo_attribution::TypoAttributionDetector::new()));
    engine.register(Box::new(crate::detectors::exfil_schema::ExfilSchemaDetector::new()));
    engine.register(Box::new(crate::detectors::socketio_c2::SocketIOC2Detector::new()));
    
    engine
}
```

### Module Discovery

Detectors live in `src/detectors/` and are exported via `src/detectors/mod.rs`:

```rust
pub mod typo_attribution;
pub mod exfil_schema;
pub mod socketio_c2;

pub use typo_attribution::TypoAttributionDetector;
pub use exfil_schema::ExfilSchemaDetector;
pub use socketio_c2::SocketIOC2Detector;
```

### Severity Levels

```rust
pub enum Severity {
    Info,       // Heuristic pattern match
    Low,        // Minimal security concern
    Medium,     // Moderate security concern
    High,       // Significant security concern
    Critical,   // Immediate attention required
}
```

**Severity stacking:** Use **additive scoring with diminishing returns**, NOT multiplicative stacking. See `src/detectors/socketio_c2.rs` for the reference implementation — it scores across 3 signal groups and caps per-category contributions.

### Taint Tracking

Taint tracking hooks in via `src/taint.rs` and `src/cross_file_taint.rs`. It tracks data flow from sources (user input, network) to sinks (eval, exec, network send). Semantic detectors use this for flow-based detection (e.g., stego → decode → exec).

### Campaign Intel

Campaign intelligence is attached via `src/campaign.rs` and `src/correlation.rs`. It clusters findings into campaigns (GlassWorm, PhantomRaven, ForceMemo) based on infrastructure reuse, code similarity (MinHash), and attack patterns.

---

## 2. What Phase 1 Added

### G3: Typo Attribution Detector

**File:** `src/detectors/typo_attribution.rs`

**Detects:** Verified GlassWorm typo fingerprints from PART5.md:
- `LoadLibararyFail` — memexec crate typo
- `Invlaid` — data.dll V10 path typo
- `NtAllocVmErr` — index loader typo

**Severity:** HIGH for any match (unique fingerprints)

**Tests:** 6 tests in `src/detectors/typo_attribution.rs::tests`

---

### G4: Exfil Schema Detector

**File:** `src/detectors/exfil_schema.rs`

**Detects:** GlassWorm exfil JSON schema from PART5.md

**Key categories:**
- **High-Signal (7 keys):** `sync_oauth_token`, `send_tab_private_key`, `walletCount`, `creditCardCount`, `cookieCount`, `loginCount`
- **Medium-Signal (5 keys):** `master_key`, `app_bound_key`, `dpapi_key`, `session_token`, `profile_oauth_token`
- **Low-Signal (11 keys):** `user_agent`, `email`, `uid`, `verified`, array keys (context only)

**Threshold:** ≥3 signal keys (configurable via `GLASSWARE_EXFIL_THRESHOLD` env var)

**Severity:**
- HIGH: ≥3 keys including ≥1 high-signal
- MEDIUM: ≥4 keys without high-signal

**Tests:** 7 tests in `src/detectors/exfil_schema.rs::tests`

---

### G5: Socket.IO C2 Detector

**File:** `src/detectors/socketio_c2.rs`

**Detects:** GlassWorm Socket.IO C2 transport pattern

**CRITICAL:** Compound pattern matcher across 3 signal groups:

- **Group A (Transport):** `io(`, `socket.connect(`, `socket.io-client`, `.on('connect')`
- **Group B (Endpoint):** `:4789`, `:5000`, `.onion`, hardcoded IPs, dynamic DNS
- **Group C (Tunnel):** `tunnel-agent`, `socks-proxy-agent`, `atob(`, `btoa(`, `extraHeaders`

**Scoring:**
- INFO: 1 group only (likely legitimate)
- MEDIUM: 2 groups (suspicious)
- HIGH: All 3 groups (GlassWorm signature)

**Reference Implementation:** This detector implements additive scoring with diminishing returns correctly. Future detectors should follow this pattern, NOT multiplicative stacking.

**Tests:** 7 tests in `src/detectors/socketio_c2.rs::tests`

---

### Running Phase 1 Tests

```bash
# Run all Phase 1 detector tests
cargo test -p glassware-core --lib typo_attribution exfil_schema socketio_c2

# Run specific test
cargo test -p glassware-core --lib test_detect_all_three_groups
```

---

## 3. Phase 2 Scope — Binary Scanning (G6–G9, G11)

### Context

`.node` files are **renamed native shared libraries**:
- Windows: PE format (`.dll`)
- Linux: ELF format (`.so`)
- macOS: Mach-O format (`.dylib`)

Scanning a `.node` file = parsing a standard binary format and extracting features.

### Key Crate: `goblin`

Add to `glassware-core/Cargo.toml`:
```toml
[dependencies]
goblin = "0.7"      # PE/ELF/Mach-O parser, zero-copy
memmap2 = "0.9"     # Zero-copy memory mapping (optional)
```

`goblin` is pure Rust, no C bindings, no unsafe. It's battle-tested and fast. Parsing a PE header takes microseconds.

### Binary Scanner Architecture

Create `src/binary/` module:

```
src/binary/
├── mod.rs           # BinaryScanner trait, orchestrator integration
├── extractor.rs     # goblin parse + string extract + entropy
├── xorshift.rs      # G6
├── ielevator.rs     # G7
├── apc_injection.rs # G8
├── memexec.rs       # G9
└── metadata.rs      # G11
```

**`BinaryFeatures` struct** (`extractor.rs`):
```rust
pub struct BinaryFeatures {
    pub format: BinaryFormat,          // PE / ELF / MachO
    pub imports: Vec<ImportEntry>,     // function imports
    pub exports: Vec<ExportEntry>,     // function exports
    pub strings: Vec<ExtractedString>, // printable ASCII/UTF-8, min 4 chars
    pub sections: Vec<SectionInfo>,    // name, entropy, size, flags
    pub debug_info: Option<DebugInfo>, // PDB path, Cargo paths
}
```

**Orchestrator Integration** (`src/scanner.rs` or `src/engine.rs`):
```rust
if path.extension() == Some("node") {
    let features = BinaryExtractor::extract(&bytes)?;
    let findings = BinaryScanner::scan(&features)?;
    return findings;
}
```

The binary scanner produces the **same `Finding` / IR types** as JS detectors. It's a new **input pathway**, not a new **output model**.

---

### G6: XorShift128 Obfuscation

**File:** `src/binary/xorshift.rs`

**Detects:** XorShift obfuscation used in GlassWorm binaries

**DO NOT use hardcoded byte patterns** like `[0x41, 0x69, 0x2E, 0x27, 0x62, 0x6C]` — that decodes to `Ai.'bl` which looks invented.

**Use heuristics instead:**
- High entropy sections (>7.5 bits/byte)
- XorShift instruction patterns (SHL/SHR pairs with characteristic constants: 13, 7, 17)
- Position-dependent key derivation patterns
- Multi-round decode loops

**Signals:**
```rust
// Signal 1: High entropy section
let high_entropy = features.sections.iter()
    .any(|s| s.entropy > 7.5);

// Signal 2: XorShift instruction patterns (from imports/disassembly)
let has_xorshift = features.imports.iter()
    .any(|i| i.name.contains("xor") || i.name.contains("shift"));

// Signal 3: Position-dependent key derivation (from strings)
let has_position_key = features.strings.iter()
    .any(|s| s.contains("idx") && s.contains("xor"));

// Threshold: ≥2 signals
```

**Severity:** HIGH if ≥2 signals

**Intel Source:** PART5.md section on xorshift obfuscation in `c_x64.node`

---

### G7: IElevator COM CLSID Detection

**File:** `src/binary/ielevator.rs`

**Detects:** IElevator COM interface usage for App-Bound key extraction

**VERIFIED CLSIDs from PART5.md:**
```rust
const IELEVATOR_CLSIDS: &[(&str, &str)] = &[
    // Chrome (first 12/16 digits match public docs)
    ("{BDB57FF2-79B9-4205-9447-F5FE85F37312}", "Chrome IElevator"),
    
    // Edge (first 8/16 match)
    ("{1F8B4C0E-79B9-420C-985A-71CAB25D78A8}", "Edge IElevator"),
    
    // Brave (from PART5 — verify exact value)
    ("{BRAVE-CLSID}", "Brave IElevator"),
];
```

**⚠️ WARNING:** A previous attempt had wrong CLSIDs. The Chrome CLSID above is from the audit (`BDB57FF2-79B9-4205-9447-F5FE85F37312`). **Re-read PART5.md to extract the exact Brave CLSID** before implementing.

**Detection:** String extraction from binary
```rust
if features.strings.iter().any(|s| {
    IELEVATOR_CLSIDS.iter().any(|(clsid, _)| s.contains(clsid))
}) {
    // Flag: IElevator COM for App-Bound key extraction
}
```

**Severity:** CRITICAL (confirmed GlassWorm TTP)

---

### G8: APC Injection Signatures

**File:** `src/binary/apc_injection.rs`

**Detects:** APC (Asynchronous Procedure Call) injection imports

**Native API imports to detect:**
```rust
const APC_IMPORTS: &[&str] = &[
    "NtAllocateVirtualMemory",
    "NtProtectVirtualMemory",
    "NtQueueApcThread",
    "NtCreateThreadEx",
    "NtWriteVirtualMemory",
];
```

**Detection:** Import table analysis
```rust
if features.imports.iter().any(|i| {
    APC_IMPORTS.iter().any(|&api| i.name.contains(api))
}) {
    // Flag: APC injection imports
}
```

**Severity:** HIGH (process injection technique)

**Intel Source:** PART4.md, PART5.md on APC injection in GlassWorm binaries

---

### G9: memexec Detection

**File:** `src/binary/memexec.rs`

**Detects:** memexec crate usage for fileless PE loading

**Detection:** String extraction
```rust
if features.strings.iter().any(|s| {
    s.contains("memexec 0.2.0") ||
    s.contains("memexec::") ||
    s.contains("LoadLibararyFail")  // memexec typo fingerprint
}) {
    // Flag: memexec fileless loader
}
```

**Severity:** HIGH (fileless execution technique)

**Intel Source:** PART4.md, PART5.md, REPORT_index.md

---

### G11: .node Binary Metadata

**File:** `src/binary/metadata.rs`

**Detects:** PDB paths, Cargo registry paths, internal DLL names

**Extraction:**
```rust
// PDB paths from debug info
if let Some(debug) = &features.debug_info {
    if let Some(pdb_path) = &debug.pdb_path {
        if pdb_path.contains("Administrator") {
            // Flag: Built on shared Windows admin account
        }
        if pdb_path.contains("N:\\work\\chrome_current") {
            // Flag: Known GlassWorm build path
        }
    }
}

// Cargo registry paths from strings
if features.strings.iter().any(|s| {
    s.contains(".cargo/registry/src") && s.contains("neon-")
}) {
    // Flag: Neon framework (Node.js native addon)
}
```

**Severity:** INFO/MEDIUM (attribution intelligence, not direct threat)

**Intel Source:** PART5.md section on developer forensics

---

### Binary Scanner Tests

Create test fixtures in `src/binary/tests/`:
- Mock PE file with known imports
- Mock ELF file with known sections
- Mock Mach-O file with known strings

**Test structure:**
```rust
#[test]
fn test_detect_xorshift_heuristic() {
    let features = BinaryFeatures {
        sections: vec![SectionInfo {
            name: ".text".to_string(),
            entropy: 7.8,  // High entropy
            ..Default::default()
        }],
        strings: vec![ExtractedString {
            content: "idx xor shift".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };
    
    let detector = XorShiftDetector::new();
    let findings = detector.detect(&features);
    assert!(!findings.is_empty());
    assert_eq!(findings[0].severity, Severity::High);
}
```

---

## 4. Phase 3 Scope — Host Forensics (G1, G2, E2)

### Context

Host forensics is a **new scanning mode** — incident response, not supply chain monitoring. It should be a **separate subcommand**, not part of package scanning.

**CLI:**
```bash
glassware host-scan --path / --chrome-prefs --output report.json
```

---

### G1: Filesystem IoC Scanner

**File:** `src/host/ioc_scanner.rs`

**Detects:** GlassWorm filesystem artifacts

**Known IoC paths from intel:**
```rust
const IOC_PATHS: &[&str] = &[
    // GlassWorm working directories
    "**/jucku/**",
    "**/myextension/**",
    
    // Temp directories (Windows)
    "%TEMP%\\SsWolTaQA\\eBmnoe",
    
    // Chrome extension paths
    "**/Chrome/Extensions/**/myextension",
    
    // Persistence artifacts
    "**/Library/LaunchAgents/com.system.update.plist",
];
```

**Implementation:**
```rust
pub struct HostScanner {
    ioc_patterns: Vec<IocPattern>,
}

impl HostScanner {
    pub fn scan(&self, root_path: &Path) -> Result<HostScanReport> {
        let mut findings = Vec::new();
        
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
        }
        
        Ok(HostScanReport { findings, .. })
    }
}
```

**⚠️ Performance:** Do NOT walk entire filesystem recursively. Use targeted path lists:
- Chrome profile directories
- Temp directories
- User app data
- Known GlassWorm paths

**Severity:** CRITICAL for confirmed IoCs

**Intel Source:** PART4.md, PART5.md

---

### G2 + E2: Chrome Secure Prefs Forensic

**File:** `src/host/chrome_prefs.rs`

**Detects:** Tampered Chrome Secure Preferences (sideloaded extensions)

**Key fields:**
```json
{
  "extensions": {
    "<extension_id>": {
      "location": 4,              // 4 = sideloaded
      "creation_flags": 38,       // 38 = suspicious combo
      "from_webstore": false,     // Not from Chrome Web Store
      "content_verification": {   // Missing/tampered = suspicious
        "verified_contents": "..."
      }
    }
  }
}
```

**Severity logic:**
```rust
// from_webstore: false ALONE = INFO (legitimate for dev/enterprise)
if from_webstore == Some(false) {
    findings.push(Finding::new(
        ...,
        Severity::Info,
        "Extension not from webstore",
        "Legitimate for dev/enterprise extensions",
    ));
}

// location=4 + creation_flags=38 + from_webstore=false = CRITICAL
if location == Some(4) && creation_flags == Some(38) && from_webstore == Some(false) {
    findings.push(Finding::new(
        ...,
        Severity::Critical,
        "GlassWorm extension signature",
        "location=4 + creation_flags=38 + from_webstore=false is GlassWorm fingerprint",
    ));
}
```

**⚠️ CRITICAL:** `from_webstore: false` alone should be **INFO or LOW**, not HIGH. Many legitimate extensions aren't from the web store (dev mode, enterprise policy). Only flag as HIGH/CRITICAL when combined with other signals.

**Intel Source:** PART5.md section on Chrome Secure Preferences bypass

---

### E2: Host Artifact Correlation

**File:** `src/host/correlation.rs`

**Correlates:** Multiple host artifacts into unified threat picture

**Example:**
```rust
if findings.iter().any(|f| f.category == "jucku_directory")
    && findings.iter().any(|f| f.category == "chrome_prefs_tampered")
    && findings.iter().any(|f| f.category == "persistence_artifact")
{
    // Correlate into single high-confidence finding
    findings.push(Finding::new(
        ...,
        Severity::Critical,
        "GlassWorm infection confirmed (multiple artifacts)",
        "Jucku directory + Chrome prefs tampering + persistence = active infection",
    ));
}
```

---

## 5. Known Issues & Gotchas

### Pre-Existing Test Failures (NOT Phase 1)

These failures predate Phase 1 and are **not your responsibility to fix**:

1. **`adversarial::polymorphic::tests::test_exec_pattern_variation`**
   - File: `src/adversarial/polymorphic.rs:474`
   - Error: "Should have exec patterns"
   - Guess: Polymorphic engine test, likely needs pattern update

2. **`adversarial::polymorphic::tests::test_variable_renaming_variation`**
   - File: `src/adversarial/polymorphic.rs:456`
   - Error: "Should have variable renaming variations"
   - Guess: Same as above

3. **`adversarial::strategies::variable::tests::test_variable_renaming_decoder`**
   - File: `src/adversarial/strategies/variable.rs:79`
   - Guess: Variable renaming strategy test

4. **`engine::tests::test_engine_default_detectors`** (FIXED in Phase 1)
   - Was: Expected 9 detectors, got 13
   - Fixed: Updated to expect 14 (with semantic) / 10 (without)

5. **`detectors::exfil_schema::tests::test_detect_threshold_met`** (FIXED in Phase 1)
   - Was: Test fixture didn't meet threshold
   - Fixed: Added more signal keys to test

### Stale Documentation

**`CURRENT-STATUS-AND-NEXT-STEPS.md`** is **STALE** and contradicts the integration plan.

**Action:** Either update it to align with `GLASSWORM-INTEGRATION-PLAN-V0.9-REVISED.md`, or deprecate it with a pointer to the revised plan.

### Severity Stacking

**CORRECT:** Additive scoring with diminishing returns (see `src/detectors/socketio_c2.rs`)

**INCORRECT:** Multiplicative stacking (e.g., `2.0 × 1.5 × 2.0 × 1.5 × 1.3 = 11.7`)

G5 already implements this correctly. Use it as a reference for future detectors.

### Rough Edges / "I Would Have Done This Differently"

1. **Detector registration is manual** — you add detectors to `default_detectors()` by hand. Could be macro-based, but it works.

2. **`DetectionCategory::Unknown`** — I used this for new detectors (G3, G4, G5) since there wasn't a perfect fit. Consider adding `AttributionFingerprint`, `ExfilSchema`, `SocketIOC2` variants.

3. **Binary scanner should be opt-in** — add a `--scan-binaries` flag to avoid overhead when not needed.

4. **Host forensics needs path configuration** — hardcoded paths won't work across Windows/macOS/Linux. Use a config file or platform-specific path lists.

---

## 6. Intel Sources

### Primary Source

**GlassWorm Writeup:** https://codeberg.org/tip-o-deincognito/glassworm-writeup

**Parts:**
- **PART1.md** — Infection chain overview
- **PART2.md** — Infrastructure rotation
- **PART3.md** — Chrome extension C2
- **PART4.md** — Binary analysis (index loaders, sideloaders)
- **PART5.md** — **MOST IMPORTANT FOR PHASE 2** (binary internals, CLSIDs, obfuscation)

**Companion Reports:**
- `REPORT_c_x64.md` — c_x64.node disassembly
- `REPORT_data.md` — data.dll analysis
- `REPORT_w.md` — w.node (Windows sideloader)
- `REPORT_m.md` — m.node (macOS sideloader)
- `REPORT_f_ex86.md` — f_ex86.node (VS Code harvester)
- `REPORT_index.md` — index loaders
- `REPORT_led_win32.md` — led-win32 (Ledger/Trezor phisher)

### Phase 2-Relevant Sections

**PART5.md:**
- Line 337: `LoadLibararyFail` typo
- Line 339: `Invlaid` typo in data.dll
- Line 468: `NtAllocVmErr` string
- Section on xorshift obfuscation (search for "xorshift")
- Section on IElevator COM (search for "IElevator")
- Section on Chrome Secure Preferences bypass
- Full exfil JSON schema

**PART4.md:**
- APC injection patterns
- memexec crate usage
- Native API imports

**REPORT_data.md:**
- V10 vs V20 encryption key paths
- Debug log format strings with typos

**REPORT_index.md:**
- memexec error enum with `LoadLibararyFail`
- `NtAllocVmErr` string

---

## 7. Getting Started

### Environment Setup

```bash
# Clone repo
git clone https://github.com/samgrowls/glassworks.git
cd glassworks

# Add goblin dependency to glassware-core/Cargo.toml
# [dependencies]
# goblin = "0.7"
# memmap2 = "0.9"

# Build
cargo build -p glassware-core

# Run tests
cargo test -p glassware-core --lib
```

### Development Workflow

1. **Create detector module** in `src/binary/` or `src/host/`
2. **Implement `Detector` trait** (or `BinaryScanner` trait for binary detectors)
3. **Add tests** in module's `tests` submodule
4. **Register detector** in `src/detectors/mod.rs` and `src/engine.rs`
5. **Run tests** — ensure no new failures introduced
6. **Update handoff doc** if you discover new patterns/conventions

### Testing Binary Detectors

Create mock binary fixtures:
```rust
// Test fixture: PE file with known imports
const TEST_PE_DATA: &[u8] = include_bytes!("fixtures/test.dll");

#[test]
fn test_detect_apc_injection() {
    let features = BinaryExtractor::extract(TEST_PE_DATA).unwrap();
    let detector = ApcInjectionDetector::new();
    let findings = detector.detect(&features);
    assert!(!findings.is_empty());
}
```

---

## 8. Sign-Off

**Phase 1 Status:** ✅ Complete (G3, G4, G5 detectors implemented and tested)

**Phase 2 Scope:** Binary scanning (G6-G9, G11) — `.node` file analysis

**Phase 3 Scope:** Host forensics (G1, G2, E2) — filesystem IoC + Chrome prefs

**Timeline:** 4 weeks total (2 weeks per phase)

**Contact:** Primary agent (via project issues/PRs)

---

**Good luck! You've got a solid foundation to build on. The architecture is clean, the patterns are established, and the intel sources are comprehensive. 🎯**

---

**End of Handoff**
