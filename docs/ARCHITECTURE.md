# glassware Architecture

**Version:** v0.67.0
**Last Updated:** 2026-03-26

---

## Overview

glassware is a production-ready Rust-based scanner for detecting GlassWorm steganographic attacks. The system is designed for large-scale scanning campaigns (100k+ packages) with reliable checkpoint/resume capabilities.

---

## System Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           glassware Binary                               │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                    CLI Layer                                     │   │
│  │  scan-npm | scan-tarball | campaign | cache-* | config          │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                              │                                          │
│  ┌───────────────────────────▼───────────────────────────────────────┐ │
│  │                    Orchestrator                                    │ │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐               │ │
│  │  │ Campaign    │  │ Scanner     │  │ Checkpoint  │               │ │
│  │  │ Executor    │  │ Engine      │  │ Manager     │               │ │
│  │  └─────────────┘  └─────────────┘  └─────────────┘               │ │
│  └───────────────────────────────────────────────────────────────────┘ │
│                              │                                          │
│  ┌───────────────────────────▼───────────────────────────────────────┐ │
│  │                    glassware-core Library                          │ │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐               │ │
│  │  │ Detectors   │  │ Scoring     │  │ IR Builder  │               │ │
│  │  │ (13+)       │  │ Engine      │  │             │               │ │
│  │  └─────────────┘  └─────────────┘  └─────────────┘               │ │
│  └───────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Component Architecture

### 1. CLI Layer (`glassware/src/cli.rs`)

Command-line interface providing:
- `scan-npm` - Scan npm packages
- `scan-tarball` - Scan local tarballs
- `campaign run` - Execute campaigns
- `campaign monitor` - TUI monitoring
- `cache-clear` - Cache management

### 2. Orchestrator (`glassware/src/orchestrator.rs`)

Campaign orchestration layer:
- **Campaign Executor** - Manages wave execution
- **Scanner Engine** - Coordinates scanning
- **Checkpoint Manager** - Handles resume capability
- **Event Bus** - Pub/sub for state updates

### 3. glassware-core Library (`glassware-core/src/`)

Core detection engine:

#### Detectors (`glassware-core/src/detectors/`)

| Detector | Tier | Purpose |
|----------|------|---------|
| `invisible.rs` | Tier 1 | ZWSP, ZWNJ, Variation Selectors |
| `homoglyph.rs` | Tier 1 | Confusable Unicode characters |
| `bidi.rs` | Tier 1 | Bidirectional text override |
| `glassware.rs` | Tier 1 | Steganography + decoder |
| `encrypted_payload.rs` | Tier 2 | High-entropy + dynamic exec |
| `header_c2.rs` | Tier 2 | HTTP header C2 pattern |
| `exfil_schema.rs` | Tier 2 | Data exfiltration |
| `blockchain_c2.rs` | Tier 3 | Known C2 wallets/IPs |
| `blockchain_polling.rs` | Tier 3 | 5-minute polling pattern |
| `time_delay.rs` | Tier 3 | CI bypass + delays |
| `locale.rs` | Tier 3 | Geographic targeting |

#### Scoring Engine (`glassware-core/src/scoring.rs`)

Tiered scoring with category diversity caps:

```rust
pub struct ScoringEngine {
    config: ScoringConfig,
    package_context: PackageContext,
}

impl ScoringEngine {
    pub fn calculate_score(&self, findings: &[Finding]) -> f32 {
        // 1. Deduplicate findings
        // 2. Calculate base score
        // 3. Apply category caps
        // 4. Apply LLM multiplier
        // 5. Apply reputation multiplier
        // 6. Apply exceptions
    }
}
```

#### IR Builder (`glassware-core/src/ir.rs`)

File intermediate representation:
- Content analysis
- AST parsing (for JS/TS)
- Unicode analysis
- Metadata extraction

---

## Data Flow

### Single Package Scan

```
npm API / Tarball
       │
       ▼
┌─────────────┐
│ Downloader  │
└─────────────┘
       │
       ▼
┌─────────────┐
│ Extractor   │
└─────────────┘
       │
       ▼
┌─────────────┐
│ IR Builder  │
└─────────────┘
       │
       ▼
┌─────────────┐
│ Detectors   │◄── Tier 1 → Tier 2 → Tier 3
└─────────────┘
       │
       ▼
┌─────────────┐
│ Scoring     │
│ Engine      │
└─────────────┘
       │
       ▼
┌─────────────┐
│ LLM         │ (optional)
│ Analysis    │
└─────────────┘
       │
       ▼
┌─────────────┐
│ Results     │
└─────────────┘
```

### Campaign Execution

```
┌─────────────┐
│ Campaign    │
│ Config      │
└─────────────┘
       │
       ▼
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│ Wave 1      │────►│ Wave 2      │────►│ Wave 3      │
│ (evidence)  │     │ (clean)     │     │ (hunt)      │
└─────────────┘     └─────────────┘     └─────────────┘
       │                  │                  │
       ▼                  ▼                  ▼
┌─────────────────────────────────────────────────────────┐
│                    Event Bus                             │
│  (state updates, progress, findings)                    │
└─────────────────────────────────────────────────────────┘
       │                  │                  │
       ▼                  ▼                  ▼
┌─────────────┐   ┌─────────────┐   ┌─────────────┐
│ State       │   │ Command     │   │ TUI         │
│ Manager     │   │ Channel     │   │ Display     │
└─────────────┘   └─────────────┘   └─────────────┘
```

---

## Scoring System

### Tiered Execution

```toml
[settings.scoring.tier_config]
mode = "tiered"

# Tier 1: Always runs
[[settings.scoring.tiers]]
tier = 1
detectors = ["invisible_char", "glassware_pattern"]
threshold = 0.0

# Tier 2: Only if Tier 1 score >= 2.0
[[settings.scoring.tiers]]
tier = 2
detectors = ["header_c2", "exfil_schema"]
threshold = 2.0

# Tier 3: Only if Tier 1+2 score >= 10.0
[[settings.scoring.tiers]]
tier = 3
detectors = ["locale_geofencing", "time_delay"]
threshold = 10.0
```

### Category Diversity Caps

| Categories | Max Score | Interpretation |
|------------|-----------|----------------|
| 1 | 5.0 | Single signal = suspicious |
| 2 | 7.0 | Two signals = borderline |
| 3 | 8.5 | Three signals = likely |
| 4+ | 10.0+ | Four+ signals = confirmed |

### Conditional Rules

```toml
[[settings.scoring.conditional_rules]]
name = "glassworm_signature"
condition = "invisible_char.count >= 10 AND blockchain_c2.count >= 1"
action = "final_score = 25.0"
```

---

## Cache System

### Cache Layers

1. **Scan Cache** (`.glassware-orchestrator-cache.db`)
   - Stores scan results by package hash
   - TTL-based expiration (default: 7 days)
   - SQLite with WAL mode

2. **Checkpoint DB** (`.glassware-checkpoints.db`)
   - Campaign state persistence
   - Wave progress tracking
   - Resume capability

3. **LLM Cache** (`.glassware-llm-cache.json`)
   - LLM response caching
   - Reduces API costs

### Cache Invalidation

```bash
# Clear all caches
./target/release/glassware cache-clear

# Or manually
rm -rf .glassware-orchestrator-cache.db
rm -rf .glassware-checkpoints.db
rm -rf .glassware-llm-cache.json
```

---

## TUI Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    TUI Application                           │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │ App State   │  │ UI Renderer │  │ Event Loop  │         │
│  │             │  │ (ratatui)   │  │ (crossterm) │         │
│  └─────────────┘  └─────────────┘  └─────────────┘         │
│         │                │                  │               │
│         ▼                ▼                  ▼               │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              Event Bus Subscription                  │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `q` | Quit |
| `Tab` | Switch tabs |
| `p` | Pause/Resume |
| `x` | Cancel |
| `c` | Adjust concurrency |
| `Enter` | Package drill-down |
| `l` | Run LLM analysis |
| `?` | Ask question |

---

## Performance Characteristics

| Metric | Value |
|--------|-------|
| Binary size | ~25MB |
| Scan speed | ~50k LOC/sec |
| npm scan | ~0.5s per package |
| GitHub scan | ~5-20s per repo |
| Memory usage | ~50MB during scan |
| Campaign throughput | ~200 packages/min (concurrency=20) |

---

## Extension Points

### Adding New Detectors

1. Create detector in `glassware-core/src/detectors/<name>.rs`
2. Implement `Detector` trait
3. Register in `glassware-core/src/engine.rs`
4. Add to tier config in campaign TOML

### Custom Scoring Rules

```toml
[[settings.scoring.conditional_rules]]
name = "my_rule"
condition = "detector_x.count >= 5"
action = "final_score *= 1.5"
```

### Custom Report Formats

Implement `ReportFormatter` trait and register in `glassware/src/formatters/`.

---

## Security Considerations

1. **Sandbox Execution:** Scanned code is never executed
2. **Network Isolation:** Downloads from npm only
3. **Cache Security:** SQLite with WAL mode, no SQL injection
4. **API Key Handling:** Environment variables only, never logged

---

## References

- [User Guide](USER-GUIDE.md)
- [Developer Guide](DEVELOPER-GUIDE.md)
- [Detectors](DETECTORS.md)
- [Campaign Operator Guide](CAMPAIGN-OPERATOR-GUIDE.md)
