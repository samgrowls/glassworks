# Glassworks — Project Context

## Project Overview

**Glassworks** is a production-ready Rust-based campaign orchestration system for detecting **GlassWare steganographic attacks** in npm packages and GitHub repositories. It identifies invisible Unicode characters, homoglyphs, bidirectional text overrides, and behavioral evasion patterns used in trojan source attacks.

---

## ✅ CURRENT STATE (2026-03-26)

**Version:** v0.70.0-wave17-complete

**✅ PRODUCTION READY:** Detector tuning achieved < 1% FP rate without whitelisting.

**Current Performance:**
- Wave15 (197 packages): 5.6% FP rate → Fixed with Tier 1 signal requirement
- Wave16 (403 packages): 0.5% FP rate ✅
- Wave17 (607 packages): 0.66% FP rate ✅
- **Average FP Rate: 0.61%** (well below 1% target)
- Evidence detection: 100% (1/1)

**Key Fixes Applied:**
1. Tier 1 signal requirement - Without InvisibleCharacter/GlasswarePattern, max score capped at 3.5
2. i18n/locale data skip - Skip locale-data/, cldr/ directories (intl package FP fix)
3. BlockchainC2 specificity - Require decodeCommand + executeCommand + 5-min polling
4. Build tool detection - Skip build output for TimeDelay, EncryptedPayload detectors

**Known FPs (documented):**
- three.js - Unicode in shader code, minified patterns
- @builder.io/qwik - Variation selectors in minified output

**See:** `docs/WAVE17-VALIDATION-REPORT.md` for full analysis.

---

### Key Capabilities

- **Campaign Orchestration** — Run large-scale scanning campaigns (100k+ packages) with wave-based execution
- **Checkpoint/Resume** — Reliable interruption recovery via SQLite persistence
- **Interactive TUI** — Live monitoring with command palette, package drill-down, and LLM analysis
- **LLM-Powered Analysis** — Natural language queries about findings (Cerebras/NVIDIA providers)
- **Markdown Reports** — Professional stakeholder reports with Tera templates
- **13+ Detectors** — Unicode, behavioral, semantic, and binary analysis

### Architecture

```
glassworks/
├── glassware-core/              # Detection engine library (detectors, findings, scan engine)
├── glassware-orchestrator/      # Campaign orchestrator binary (TUI, campaigns, reports)
├── glassware-cli/               # Simple scanner binary (to be consolidated)
├── campaigns/                   # TOML campaign configurations
├── docs/                        # User documentation
├── design/                      # Architecture specifications
├── HANDOFF/                     # Developer handoff & roadmap
└── tests/                       # Integration tests
```

### Tech Stack

| Category | Technologies |
|----------|-------------|
| **Language** | Rust 2021 (MSRV 1.70) |
| **Async** | tokio, futures |
| **CLI** | clap (derive) |
| **TUI** | ratatui, crossterm |
| **HTTP** | reqwest (rustls) |
| **Database** | sqlx (SQLite), rusqlite |
| **Serialization** | serde, serde_json, toml |
| **Logging** | tracing, tracing-subscriber |
| **Rate Limiting** | governor |
| **Templates** | tera |
| **Unicode** | unicode-script |

---

## Building and Running

### Prerequisites

- Rust 1.70+
- Optional: LLM API keys (Cerebras, NVIDIA) for LLM-powered analysis

### Build Commands

```bash
# Debug build
cargo build -p glassware-orchestrator

# Release build (optimized)
cargo build -p glassware-orchestrator --release

# Check only (fast)
cargo check -p glassware-orchestrator

# Build all workspace members
cargo build --workspace
```

### Run Commands

```bash
# Run campaign from TOML config
./target/release/glassware-orchestrator campaign run campaigns/wave6.toml

# TUI demo mode
./target/release/glassware-orchestrator campaign demo

# Monitor live campaign
./target/release/glassware-orchestrator campaign monitor <case-id>

# Generate markdown report
./target/release/glassware-orchestrator campaign report <case-id>

# Ask LLM questions about campaign
./target/release/glassware-orchestrator campaign query <case-id> "Why was express flagged?"

# Simple file/directory scan (glassware-cli)
./target/release/glassware scan /path/to/code
```

### Test Commands

```bash
# Run campaign module tests
cargo test -p glassware-orchestrator campaign

# Run all tests (requires features)
cargo test --features "full,llm"

# Run core detector tests
cargo test -p glassware-core
```

### Environment Variables

```bash
# Tier 1 LLM (Cerebras - fast triage)
export GLASSWARE_LLM_BASE_URL="https://api.cerebras.ai/v1"
export GLASSWARE_LLM_API_KEY="csk-..."
export GLASSWARE_LLM_MODEL="llama-3.3-70b"

# Tier 2 LLM (NVIDIA - deep analysis)
export NVIDIA_API_KEY="nvapi-..."

# GitHub (private repos)
export GITHUB_TOKEN="ghp_..."
```

Copy `.env.example` to `.env` and configure as needed.

---

## Development Conventions

### Code Style

- **Rust Edition:** 2021
- **Error Handling:** `anyhow` for application, `thiserror` for libraries
- **Async:** tokio runtime with structured concurrency
- **Logging:** `tracing` with env-filter for dynamic log levels
- **Module Organization:** Feature-gated modules (e.g., `#[cfg(feature = "semantic")]`)

### Project Structure Conventions

| Directory | Purpose |
|-----------|---------|
| `src/campaign/` | Campaign orchestration (event bus, state manager, executor) |
| `src/tui/` | Terminal UI implementation |
| `src/formatters/` | Output formatters (JSON, Markdown, SARIF) |
| `glassware-core/src/detectors/` | Individual detector implementations |

### Testing Practices

- **Integration Tests:** Located in `tests/` directory with `full` feature
- **Unit Tests:** Co-located with source in `#[cfg(test)]` modules
- **Test Fixtures:** Campaign test fixtures in `glassware-core/tests/`
- **Smoke Tests:** `tests/smoke-tests.sh` for quick validation

### Feature Flags

| Feature | Description |
|---------|-------------|
| `full` | All features (regex, serde, cache, semantic, binary) |
| `minimal` | Invisible chars and bidi only |
| `semantic` | OXC-based semantic analysis (JS/TS) |
| `llm` | LLM analysis layer |
| `binary` | Binary analysis (.node files) |
| `cache` | Hash-based incremental scanning |

### Commit Conventions

- Clear, concise commit messages focused on "why" not "what"
- Reference issues/PRs where applicable
- Group related changes logically

---

## Key Documentation

| Document | Purpose |
|----------|---------|
| [`README.md`](README.md) | Project overview and quick start |
| [`HANDOFF/README.md`](HANDOFF/README.md) | Developer handoff and onboarding |
| [`docs/CAMPAIGN-USER-GUIDE.md`](docs/CAMPAIGN-USER-GUIDE.md) | Complete user guide for campaigns |
| [`design/CAMPAIGN-ARCHITECTURE.md`](design/CAMPAIGN-ARCHITECTURE.md) | Technical architecture specification |
| [`HANDOFF/FINAL-SESSION-SUMMARY.md`](HANDOFF/FINAL-SESSION-SUMMARY.md) | Session completion summary |
| [`HANDOFF/FUTURE/ROADMAP-2026.md`](HANDOFF/FUTURE/ROADMAP-2026.md) | Strategic roadmap |
| [`HANDOFF/FUTURE/BINARY-CONSOLIDATION.md`](HANDOFF/FUTURE/BINARY-CONSOLIDATION.md) | Binary consolidation plan (next major task) |

---

## TUI Keyboard Shortcuts

### Global
| Key | Action |
|-----|--------|
| `q` | Quit TUI |
| `Ctrl+C` | Pause campaign |
| `Tab` | Switch tabs |

### Campaign Control
| Key | Action |
|-----|--------|
| `p` | Pause/Resume |
| `x` | Cancel campaign |
| `r` | Resume campaign |
| `s` | Skip wave |
| `c` | Adjust concurrency |

### Package Drill-Down (Findings Tab)
| Key | Action |
|-----|--------|
| `Enter` | Open package detail view |
| `l` | Run LLM analysis on package |
| `↑/↓` or `j/k` | Navigate package list |
| `?` | Ask question about package |
| `q`/`Esc` | Close detail view |

---

## Detection Capabilities

### L1 Detectors (Primary)
- Invisible character detection
- Homoglyph/confusable character detection
- Bidirectional text override detection
- Unicode tag character detection

### L2 Detectors (Secondary)
- GlassWare pattern detection
- Encrypted payload detection
- RDD (URL dependency) detection
- JPD author signature detection

### L3 Detectors (Behavioral)
- Locale geofencing detection
- Time delay sandbox evasion
- Blockchain C2 detection

---

## Performance Metrics

| Metric | Value |
|--------|-------|
| Binary size | ~25MB |
| Scan speed | ~50k LOC/sec |
| npm scan | ~0.5s per package |
| GitHub scan | ~5-20s per repo |
| Memory usage | ~50MB during scan |

---

## Known Limitations

| Limitation | Impact | Workaround |
|------------|--------|------------|
| No partial wave resume | Mid-wave interruption loses progress | Run smaller waves |
| No checkpoint cleanup | Database grows over time | Manual: `rm .glassware-checkpoints.db` |
| LLM query single-shot | No conversational follow-up | Ask complete questions |
| Two binaries | User confusion, larger download | See BINARY-CONSOLIDATION.md |
| **Whitelist bypasses detection** | High-value targets not scanned | Remove whitelist entries, fix detectors |
| **Only 2 evidence packages** | Can't measure FN rate | Expand evidence library |

---

## Testing Workflows (From Recent Session)

### Individual Package Testing

```bash
# Scan single package
./target/release/glassware scan-npm <package>@<version>

# Scan with LLM analysis
./target/release/glassware scan-npm <package>@<version> --llm

# Scan with deep LLM analysis (NVIDIA Tier 2)
./target/release/glassware scan-npm <package>@<version> --deep-llm

# Scan evidence tarball
./target/release/glassware scan-tarball evidence/<package>.tgz

# Clear cache between tests
rm -f .glassware-orchestrator-cache.db
```

### Debugging Detections

```bash
# See findings breakdown
./target/release/glassware scan-npm <package> 2>&1 | grep -E "Findings by|category:"

# See individual findings
./target/release/glassware scan-npm <package> 2>&1 | tail -40

# Check LLM verdict
./target/release/glassware scan-npm <package> --llm 2>&1 | grep -E "LLM verdict|confidence"

# Extract tarball for manual inspection
cd /tmp && tar -xzf evidence/<package>.tgz
find package -name "*.js" | head -10
grep -rn "eval\|Function(" package/lib/
```

### Campaign Testing

```bash
# Run full campaign
./target/release/glassware campaign run campaigns/wave10-1000plus.toml --llm

# Monitor progress
tail -f logs/wave10-*.log | grep -E "Wave.*completed|Campaign completed"

# Check results
grep "flagged as malicious" logs/wave10-*.log | wc -l
grep "Malicious packages:" logs/wave10-*.log
```

---

## My Understanding of Detection Pipeline

```
npm package / tarball / directory
    ↓
Downloader (npm API / tarball extraction / file walk)
    ↓
Scanner (scan_directory / scan_tarball)
    ↓
ScanEngine (runs all detectors)
    ↓
For each file:
    - Build FileIR (AST + content)
    - Run detectors (Unicode, patterns, behavioral)
    - Collect findings
    ↓
Calculate Threat Score
    - Category diversity (1 cat = 4.0 max, 3+ cats = 10.0)
    - Critical/high hits weighted
    ↓
Apply Whitelist
    - Exact match, wildcard, prefix matching
    ↓
LLM Analysis (if enabled)
    - Confidence-based override
    - <0.25 = likely FP, >0.75 = trust LLM
    ↓
Return Results
```

### Key Insights

1. **Category diversity scoring works** - Real attacks involve multiple vectors
2. **Context matters** - setTimeout in build tool ≠ sandbox evasion
3. **LLM is underutilized** - Currently just confidence override
4. **Whitelisting is dangerous** - We whitelisted high-value targets

---

---

## Current Initiative: Binary Consolidation (In Progress)

**Priority:** HIGH | **Timeline:** 3 weeks (March 23 - April 13, 2026)
**Status:** 🟡 Planning Complete - Ready to Implement

Consolidate `glassware-cli` and `glassware-orchestrator` into a single unified `glassware` binary:

```bash
glassware scan /path/to/code          # Simple file/directory scan
glassware campaign run wave6.toml     # Campaign orchestration
glassware campaign demo               # TUI monitoring
glassware campaign query "..."        # LLM-powered queries
```

**Target Outcomes:**
| Metric | Before | Target | Improvement |
|--------|--------|--------|-------------|
| Binaries | 2 | 1 | -50% |
| Binary Size | ~36MB | 10-15MB | -60% |
| Memory Usage | ~50MB | 25-35MB | -40% |
| Scan Speed | ~50k LOC/s | ~65k LOC/s | +30% |

### Consolidation Documentation

Comprehensive planning documentation is available in `docs/binaryconsolidation/`:

| Document | Purpose |
|----------|---------|
| [`docs/binaryconsolidation/README.md`](docs/binaryconsolidation/README.md) | Documentation index |
| [`docs/binaryconsolidation/EXECUTIVE-SUMMARY.md`](docs/binaryconsolidation/EXECUTIVE-SUMMARY.md) | **Start here** - High-level overview |
| [`docs/binaryconsolidation/CONSOLIDATION-PLAN.md`](docs/binaryconsolidation/CONSOLIDATION-PLAN.md) | Master plan with full analysis |
| [`docs/binaryconsolidation/WORKSPACE-RESTRUCTURING.md`](docs/binaryconsolidation/WORKSPACE-RESTRUCTURING.md) | Step-by-step migration guide |
| [`docs/binaryconsolidation/IMPLEMENTATION-TRACKER.md`](docs/binaryconsolidation/IMPLEMENTATION-TRACKER.md) | Task tracking and progress |
| [`docs/binaryconsolidation/QUESTIONS.md`](docs/binaryconsolidation/QUESTIONS.md) | Questions for previous developer |

### Implementation Phases

**Week 1 (Mar 23-29):** Feature audit & code consolidation
**Week 2 (Mar 30 - Apr 5):** Size optimization (LTO, features, dependencies)
**Week 3 (Apr 6-12):** Performance optimization (memory, speed, PGO)
**Release (Apr 13):** v0.9.0 with deprecation notices

See [`docs/binaryconsolidation/EXECUTIVE-SUMMARY.md`](docs/binaryconsolidation/EXECUTIVE-SUMMARY.md) to get started.
