# GlassWorm Architecture

**Version:** v0.11.0+  
**Last Updated:** 2026-03-21

---

## System Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                    Input Sources                                 │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐        │
│  │  npm     │  │  GitHub  │  │ Tarball  │  │  Host    │        │
│  │ Registry │  │   API    │  │  Files   │  │  Scan    │        │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘        │
└───────┼─────────────┼─────────────┼─────────────┼───────────────┘
        │             │             │             │
        └─────────────┴─────────────┴─────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────────┐
│                    glassware-core                                │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │              ScanEngine (Detector Orchestrator)          │    │
│  │  ┌──────────────────────────────────────────────────┐   │    │
│  │  │  Tier 1: Regex Detectors (all files)             │   │    │
│  │  │  - InvisibleChar, Homoglyph, Bidi, UnicodeTag    │   │    │
│  │  │  - GlasswarePattern, EncryptedPayload, HeaderC2  │   │    │
│  │  │  - LocaleGeofencing, TimeDelay, BlockchainC2     │   │    │
│  │  │  - RDD, ForceMemo, JpdAuthor                     │   │    │
│  │  └──────────────────────────────────────────────────┘   │    │
│  │  ┌──────────────────────────────────────────────────┐   │    │
│  │  │  Tier 2: Semantic Detectors (JS/TS only)         │   │    │
│  │  │  - Gw005Semantic, Gw006Semantic                  │   │    │
│  │  │  - Gw007Semantic, Gw008Semantic                  │   │    │
│  │  └──────────────────────────────────────────────────┘   │    │
│  │  ┌──────────────────────────────────────────────────┐   │    │
│  │  │  Tier 3: Binary Detectors (.node files)          │   │    │
│  │  │  - XorShift128, IElevator, ApcInjection          │   │    │
│  │  │  - Memexec, Metadata                             │   │    │
│  │  └──────────────────────────────────────────────────┘   │    │
│  └─────────────────────────────────────────────────────────┘    │
│                              │                                   │
│                              ▼                                   │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │              Finding Output (Vec<Finding>)               │    │
│  └─────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Output Formatters                             │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐        │
│  │  Pretty  │  │   JSON   │  │   SARIF  │  │ Streaming│        │
│  │  (CLI)   │  │          │  │  (GitHub)│  │  (JSONL) │        │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘        │
└─────────────────────────────────────────────────────────────────┘
```

---

## Components

### glassware-core

**Location:** `glassware-core/`  
**Purpose:** Core detection library

| Module | Purpose |
|--------|---------|
| `detector.rs` | Detector trait definition |
| `engine.rs` | ScanEngine orchestrator |
| `finding.rs` | Finding, DetectionCategory, Severity types |
| `scanner.rs` | UnicodeScanner (L1 regex detectors) |
| `ir.rs` | Unified intermediate representation |
| `taint.rs` | Taint tracking (source/sink/flow) |
| `binary/` | Binary detectors (goblin-based) |
| `detectors/` | L1 regex detector implementations |

### glassware-cli

**Location:** `glassware-cli/`  
**Purpose:** CLI binary (`glassware`)

| Feature | Description |
|---------|-------------|
| Directory scanning | Recursive file scanning |
| Format options | pretty, json, sarif |
| Severity filtering | --severity flag |
| LLM integration | --llm flag (Cerebras) |
| Caching | --cache-file for incremental scans |

### glassware-orchestrator

**Location:** `glassware-orchestrator/`  
**Purpose:** Advanced orchestration

| Feature | Description |
|---------|-------------|
| npm scanning | Package download + scan |
| GitHub scanning | Repo search + clone + scan |
| Tarball scanning | Direct .tgz file scanning |
| SQLite caching | Persistent scan cache |
| Rate limiting | Configurable per-source limits |
| Checkpoint/resume | Long-running scan support |
| LLM analysis | NVIDIA API integration |

### harness (Python)

**Location:** `harness/`  
**Purpose:** Python-based scanning tools

| Module | Purpose |
|--------|---------|
| `core/store.py` | SQLite database layer |
| `core/fetcher.py` | npm package download |
| `core/scanner.py` | glassware CLI wrapper |
| `core/analyzer.py` | NVIDIA LLM analysis |
| `core/reporter.py` | Markdown/JSON reports |
| `core/orchestrator.py` | Wave-based pipeline |

---

## Detection Flow

### 1. Input Processing

```
File/Directory
    │
    ▼
┌─────────────────┐
│  FileIR::build  │  ← Parse once, use many times
│  - Split lines  │
│  - Parse JSON   │  (package.json)
│  - Parse AST    │  (JS/TS, semantic feature)
│  - Unicode analysis │
│  - Extract metadata │
└────────┬────────┘
         │
         ▼
```

### 2. Detector Execution

```
FileIR
    │
    ▼
┌─────────────────────────────────────────┐
│         ScanEngine::scan()              │
│                                         │
│  Tier 1 (Always run)                    │
│  ├─ UnicodeDetector                     │
│  ├─ HomoglyphDetector                   │
│  ├─ BidiDetector                        │
│  └─ ...                                 │
│                                         │
│  Tier 2 (If Tier 1 finds OR not minified)│
│  ├─ GlasswareDetector                   │
│  ├─ EncryptedPayloadDetector            │
│  └─ HeaderC2Detector                    │
│                                         │
│  Tier 3 (Only if Tier 1+2 find)         │
│  ├─ LocaleGeofencingDetector            │
│  ├─ TimeDelayDetector                   │
│  ├─ BlockchainC2Detector                │
│  └─ ...                                 │
└────────┬────────────────────────────────┘
         │
         ▼
```

### 3. Threat Scoring

```
Vec<Finding>
    │
    ▼
┌─────────────────────────────────────────┐
│      calculate_threat_score()           │
│                                         │
│  categories = unique categories         │
│  critical = count(Critical severity)    │
│  high = count(High severity)            │
│                                         │
│  score = (categories × 2.0) +           │
│          (critical × 3.0) +             │
│          (high × 1.5)                   │
│                                         │
│  return min(score, 10.0)                │
└────────┬────────────────────────────────┘
         │
         ▼
```

### 4. Output Generation

```
PackageScanResult
    │
    ▼
┌─────────────────────────────────────────┐
│         Format Output                   │
│                                         │
│  Pretty: Human-readable terminal        │
│  JSON:   Machine-readable JSON          │
│  SARIF:  GitHub Advanced Security       │
│  JSONL:  Streaming output               │
└─────────────────────────────────────────┘
```

---

## Data Structures

### Finding

```rust
pub struct Finding {
    pub file: String,              // File path
    pub line: usize,               // Line number
    pub column: usize,             // Column
    pub code_point: u32,           // Unicode code point
    pub character: String,         // Character
    pub category: DetectionCategory, // Detection category
    pub severity: Severity,        // Info/Low/Medium/High/Critical
    pub description: String,       // Human-readable description
    pub remediation: String,       // Remediation guidance
    pub confidence: Option<f64>,   // 0.0-1.0 confidence score
}
```

### DetectionCategory

```rust
pub enum DetectionCategory {
    // L1 Regex
    InvisibleCharacter,
    Homoglyph,
    BidirectionalOverride,
    UnicodeTag,
    GlasswarePattern,
    EncryptedPayload,
    HeaderC2,
    LocaleGeofencing,
    TimeDelaySandboxEvasion,
    BlockchainC2,
    RddAttack,
    ForceMemoPython,
    JpdAuthor,
    
    // L2 Semantic
    // (internal use)
    
    // L3 Binary
    XorShiftObfuscation,
    IElevatorCom,
    ApcInjection,
    MemexecLoader,
    
    // Unknown
    Unknown,
}
```

### Severity

```rust
pub enum Severity {
    Info,       // Informational
    Low,        // Low severity
    Medium,     // Medium severity
    High,       // High severity
    Critical,   // Critical severity
}
```

---

## Configuration

### Environment Variables

```bash
# Cerebras LLM (Rust CLI)
export GLASSWARE_LLM_BASE_URL="https://api.cerebras.ai/v1"
export GLASSWARE_LLM_API_KEY="csk-..."
export GLASSWARE_LLM_MODEL="llama-3.3-70b"

# NVIDIA LLM (Python harness)
export NVIDIA_API_KEY="nvapi-..."
export NVIDIA_BASE_URL="https://integrate.api.nvidia.com/v1"
export NVIDIA_MODELS="qwen/qwen3.5-397b-a17b,moonshotai/kimi-k2.5,z-ai/glm5,meta/llama-3.3-70b-instruct"

# GitHub API (optional, higher rate limits)
export GITHUB_TOKEN="ghp_..."
```

### CLI Flags

```bash
# glassware CLI
glassware --format json --severity high --llm /path/to/scan

# glassware-orchestrator
glassware-orchestrator scan-npm --llm --concurrency 10 package1 package2
glassware-orchestrator scan-tarball file1.tgz file2.tgz
glassware-orchestrator scan-github owner/repo
```

---

## Performance Characteristics

| Operation | Time Complexity | Notes |
|-----------|-----------------|-------|
| FileIR::build | O(n) | n = file size |
| L1 Detectors | O(n) | Single pass |
| L2 Detectors | O(n²) | AST analysis |
| L3 Detectors | O(n) | Binary parsing |
| Threat Score | O(f) | f = findings count |
| Total Scan | O(n) | Dominated by file I/O |

---

## References

- [GlassWorm Writeup](https://codeberg.org/tip-o-deincognito/glassworm-writeup)
- [Trojan Source Paper](https://trojan.source.ac/)
- [SARIF Specification](https://docs.oasis-open.org/sarif/sarif/v2.1.0/sarif-v2.1.0.html)
