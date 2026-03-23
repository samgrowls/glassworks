# Architectural Considerations

**Date:** March 22, 2026
**Purpose:** Long-term architectural questions and directions

---

## Question 1: Modular Detector Architecture

### Current State

Detectors are currently implemented in `glassware-core/src/` with varying degrees of modularity:

```
glassware-core/src/
├── detector.rs          # Detector trait (well-defined interface)
├── engine.rs            # Detector registration and execution
├── finding.rs           # Finding types and categories
├── rdd_detector.rs      # RDD detection
├── jpd_author_detector.rs # JPD signature detection
├── forcememo_detector.rs # Python injection detection
├── unicode_detector.rs  # Unicode attacks
├── blockchain_c2_detector.rs # Blockchain C2
└── ...
```

**The `Detector` trait is already well-designed:**

```rust
pub trait Detector: Send + Sync {
    fn name(&self) -> &str;
    fn tier(&self) -> DetectorTier { DetectorTier::Tier1Primary }
    fn detect(&self, ir: &FileIR) -> Vec<Finding>;
    fn cost(&self) -> u8 { 5 }
    fn signal_strength(&self) -> u8 { 5 }
    fn should_run(&self, other_findings: &[Finding]) -> bool { true }
}
```

### Path to Modularity

**Phase A: Plugin Architecture (Long-term)**

```rust
// Future: External detector plugins
pub trait DetectorPlugin: Send + Sync {
    fn metadata(&self) -> PluginMetadata;
    fn create_detector(&self, config: Value) -> Box<dyn Detector>;
}

// Detectors could be loaded from:
// - Built-in (current)
// - External crates (glassware-detector-*)
// - WASM plugins (future)
```

**Benefits:**
- Third-party detector development
- Community contributions
- Specialized detectors (financial fraud, IP theft, etc.)

**Costs:**
- Plugin ABI stability
- Version compatibility
- Security (untrusted detector code)

**Recommendation:** Wait until core detector set stabilizes. Current trait-based design already enables easy addition.

---

## Question 2: Broader Attack Detection Scope

### Current Focus

GlassWorm/GlassWare steganographic attacks:
- Unicode invisibles
- Homoglyphs
- Bidirectional text
- Behavioral evasion (locale, time delay, blockchain C2)

### Potential Expansion Areas

| Category | Examples | Fit with Current Architecture |
|----------|----------|-------------------------------|
| **Supply Chain** | Typosquats, dependency confusion | ✅ Good fit (RDD detector exists) |
| **Malicious Code** | Backdoors, data exfiltration | ⚠️ Needs semantic analysis |
| **License Violations** | GPL in proprietary code | ⚠️ Different detection paradigm |
| **Secrets Detection** | API keys, credentials | ✅ Good fit (pattern-based) |
| **Code Quality** | Security anti-patterns | ⚠️ Different audience/purpose |

### Recommendation

**Stay focused on steganographic + behavioral attacks for now.**

**Rationale:**
1. Clear differentiation from existing tools (Semgrep, CodeQL)
2. Deep expertise in Unicode/GlassWare domain
3. Manageable scope for Phase 1-2

**Future expansion criteria:**
- Leverages existing Unicode/behavioral expertise
- Pattern-based detection (not full program analysis)
- Clear signal/noise ratio
- Addresses npm supply chain specifically

---

## Question 3: TUI Design Possibilities

### Core Use Cases

1. **Progress Monitoring** (primary)
   - Real-time campaign progress
   - Wave status
   - ETA calculation

2. **Drill-Down Analysis** (secondary)
   - View individual package findings
   - Examine specific detections
   - LLM analysis integration

3. **Interactive Steering** (tertiary)
   - Pause/resume campaigns
   - Skip waves
   - Adjust parameters

### TUI Architecture Options

#### Option A: Read-Only Dashboard (Recommended for Phase 3)

```
┌──────────────────────────────────────────────────────────┐
│  Campaign: Wave 6 Calibration                  [Running] │
├──────────────────────────────────────────────────────────┤
│  Progress: ████████░░░░ 67%  ETA: 2m 30s                 │
├──────────────────────────────────────────────────────────┤
│  Waves:                                                   │
│  ✅ 6A: Known Malicious    2/2 scanned, 2 flagged       │
│  🟡 6B: Clean Baseline     3/5 scanned, 0 flagged       │
│  ⏳ 6C: React Native       0/4 scanned, 0 flagged       │
├──────────────────────────────────────────────────────────┤
│  Active: express@4.19.2 (scanning...)                    │
├──────────────────────────────────────────────────────────┤
│  Recent Findings:                                         │
│  [HIGH] express/package.json: RDD pattern detected       │
│  [MED]  express/index.js: Time delay pattern              │
└──────────────────────────────────────────────────────────┘
  [p] Pause  [x] Cancel  [s] Skip  [c] Concurrency  [q] Quit
```

**Implementation:**
- Subscribe to event bus
- Render state snapshots
- Send commands via channel

**Complexity:** Low
**Value:** High

---

#### Option B: Interactive Report Browser (Future)

```
┌──────────────────────────────────────────────────────────┐
│  Campaign: Wave 6 Calibration  > Packages > express@4.19 │
├──────────────────────────────────────────────────────────┤
│  Package: express@4.19.2                                  │
│  Status: ✅ Clean (threat score: 1.5)                     │
├──────────────────────────────────────────────────────────┤
│  Findings (2):                                            │
│  ┌────────────────────────────────────────────────────┐  │
│  │ [MED] Time Delay Sandbox Evasion                   │  │
│  │ File: package.json:42                              │  │
│  │ Context: "postinstall": "node scripts/delay.js"    │  │
│  │                                                     │  │
│  │ [LLM Analysis]                                     │  │
│  │ "Likely false positive - common test pattern"      │  │
│  │ Confidence: 0.85                                   │  │
│  └────────────────────────────────────────────────────┘  │
├──────────────────────────────────────────────────────────┤
│  [↑↓] Navigate  [Enter] Details  [L] LLM  [q] Back       │
└──────────────────────────────────────────────────────────┘
```

**Implementation:**
- Store findings in queryable format
- Navigation state machine
- LLM integration on-demand

**Complexity:** Medium
**Value:** Medium-High

---

#### Option C: LLM Query Interface (Future Future)

```
┌──────────────────────────────────────────────────────────┐
│  Campaign: Wave 6 Calibration  > LLM Query               │
├──────────────────────────────────────────────────────────┤
│  Query: "Show me all packages with blockchain C2 patterns"│
│  ─────────────────────────────────────────────────────  │
│  Results (1):                                             │
│  ┌────────────────────────────────────────────────────┐  │
│  │ crypto-wallet-helper@1.0.0                         │  │
│  │ Finding: Blockchain C2 pattern detected            │  │
│  │ File: src/index.js:150                             │  │
│  │ LLM: "High confidence malicious C2 pattern"        │  │
│  └────────────────────────────────────────────────────┘  │
├──────────────────────────────────────────────────────────┤
│  [n] New Query  [r] Refine  [q] Back                     │
└──────────────────────────────────────────────────────────┘
```

**Implementation:**
- Natural language → query translation
- Finding database with embeddings
- LLM integration for query processing

**Complexity:** High
**Value:** Uncertain (novelty vs utility)

---

## Recommendation: Phased TUI Approach

### Phase 3A: Read-Only Dashboard
- Progress monitoring
- Basic command steering (pause, cancel, skip)
- Recent findings display

**Duration:** 1-2 days
**Risk:** Low
**Value:** High

### Phase 3B: Report Browser
- Package drill-down
- Finding details
- LLM analysis on-demand

**Duration:** 2-3 days
**Risk:** Medium
**Value:** Medium

### Phase 3C: LLM Query (Experimental)
- Natural language queries
- Finding correlation
- Pattern discovery

**Duration:** 1 week+
**Risk:** High
**Value:** Uncertain

---

## Question 4: Detector Expansion Strategy

### Current Detector Organization

```
glassware-core/src/
├── detectors/
│   ├── invisible_char.rs
│   ├── homoglyph.rs
│   ├── bidi.rs
│   └── ...
├── behavioral/
│   ├── locale_detector.rs
│   ├── time_delay_detector.rs
│   └── blockchain_c2_detector.rs
└── semantic/
    ├── gw005_semantic.rs
    └── gw006_semantic.rs
```

### Expansion Strategy

**Keep current organization. It's already modular.**

**For new detectors:**
1. Create file in appropriate category
2. Implement `Detector` trait
3. Register in `engine.rs`
4. Add finding category to `finding.rs`
5. Write tests

**Example: Adding a secrets detector**

```rust
// detectors/secrets_detector.rs
pub struct SecretsDetector;

impl Detector for SecretsDetector {
    fn name(&self) -> &str { "secrets_detector" }
    fn tier(&self) -> DetectorTier { DetectorTier::Tier2Secondary }
    
    fn detect(&self, ir: &FileIR) -> Vec<Finding> {
        let mut findings = Vec::new();
        
        // Check for API key patterns
        if let Some(content) = ir.content() {
            if let Some(m) = API_KEY_PATTERN.find(content) {
                findings.push(Finding {
                    category: DetectionCategory::HardcodedSecret,
                    severity: Severity::High,
                    // ...
                });
            }
        }
        
        findings
    }
}
```

**No architectural changes needed.**

---

## Summary

| Question | Recommendation |
|----------|----------------|
| **Modular detectors** | Current trait design is good. Plugin system can wait. |
| **Broader scope** | Stay focused on steganographic/behavioral for now. |
| **TUI possibilities** | Phase 3A: Dashboard first. Drill-down and LLM query later. |
| **Detector expansion** | Current organization supports easy addition. |

---

**Bottom Line:** The architecture is already well-designed for growth. Focus on completing Phase 2 (resume, commands, reports) and Phase 3A (TUI dashboard) before considering major architectural changes.
