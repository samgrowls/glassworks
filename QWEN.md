# Glassworks — Project Context

**Autonomous GlassWare Detection System** — Multi-provider LLM orchestration, GitHub repo scanning, and comprehensive threat intelligence for detecting GlassWare/steganographic attacks in source code.

---

## Project Overview

**glassworks** is a security scanning tool that detects:
- Steganographic payloads hidden in Unicode characters
- Invisible character attacks (zero-width, variation selectors)
- Bidirectional text override attacks
- Behavioral evasion patterns (locale geofencing, time delays, blockchain C2)
- RDD (Registry-Dependent Dependencies) + JPD author signature attacks
- Python repository injection (ForceMemo campaign)

### Architecture

```
glassworks/
├── glassware-core/          # Core Rust detection library (22+ detectors)
├── glassware-cli/           # Single-file CLI scanner (glassware binary)
├── glassware-orchestrator/  # Rust orchestrator for npm/GitHub scanning
├── harness/                 # Python scanning tools & wave campaigns
├── llm-analyzer/            # LLM analysis module (NVIDIA/Cerebras)
├── docs/                    # Documentation
└── tests/                   # Integration tests
```

### Detection Tiers

| Tier | Detectors | When They Run | Examples |
|------|-----------|---------------|----------|
| **Tier 1** | Primary | Always | Invisible chars, homoglyphs, bidi |
| **Tier 2** | Secondary | If Tier 1 finds something | GlassWare patterns, encrypted payload |
| **Tier 3** | Behavioral | If Tier 1+2 find something | Locale, time delay, blockchain C2 |

---

## Building and Running

### Prerequisites

- Rust 1.70+
- Python 3.10+
- NVIDIA/Cerebras API key (optional, for LLM analysis)

### Build Commands

```bash
# Build entire workspace
cargo build --release

# Build specific crate
cargo build -p glassware-cli --release
cargo build -p glassware-orchestrator --release

# Build with all features
cargo build --features "full,llm"

# Run tests
cargo test --features "full,llm"
```

### Running Scans

```bash
# Quick directory scan (CLI)
./target/release/glassware /path/to/project

# Scan npm packages (Orchestrator)
./target/release/glassware-orchestrator scan-npm express lodash axios

# Scan from file
./target/release/glassware-orchestrator scan-file packages.txt

# With LLM triage (Cerebras)
./target/release/glassware-orchestrator --llm scan-npm express

# SARIF output for GitHub Security
./target/release/glassware-orchestrator --format sarif scan-npm express > results.sarif
```

### Python Harness (Wave Campaigns)

```bash
cd harness

# Sample packages
python3 diverse_sampling.py --samples-per-keyword 10 -o packages.txt

# Run wave campaign
python3 -m core.orchestrator run-wave --wave 0

# With NVIDIA LLM analysis
python3 -m core.orchestrator run-wave --wave 0 --llm

# Background version scanning
python3 background_scanner.py --packages packages.txt --policy last-10 --output results.db
```

---

## Configuration

### Environment Variables

```bash
# Cerebras LLM (Rust orchestrator triage)
export GLASSWARE_LLM_BASE_URL="https://api.cerebras.ai/v1"
export GLASSWARE_LLM_API_KEY="csk-..."
export GLASSWARE_LLM_MODEL="llama-3.3-70b"

# NVIDIA LLM (Python harness deep analysis)
export NVIDIA_API_KEY="nvapi-..."
export NVIDIA_BASE_URL="https://integrate.api.nvidia.com/v1"
export NVIDIA_MODELS="qwen/qwen3.5-397b-a17b,moonshotai/kimi-k2.5"

# GitHub API (private repos)
export GITHUB_TOKEN="ghp_..."
```

### Wave Configuration (`harness/waves.toml`)

Waves define scanning campaigns with targets and parameters:

```toml
[wave_0]
name = "Calibration"
packages_total = 50

[wave_0.known_malicious]
packages = ["react-native-country-select@0.3.91", ...]

[wave_0.clean_baseline]
count = 20
packages = ["express", "lodash", "axios", ...]
```

---

## Key Files

| File | Purpose |
|------|---------|
| `glassware-core/src/engine.rs` | Main detection engine, DAG execution |
| `glassware-core/src/detector.rs` | Detector trait definition |
| `glassware-core/src/finding.rs` | Finding types and categories |
| `glassware-core/src/rdd_detector.rs` | RDD (URL dependency) detection |
| `glassware-core/src/jpd_author_detector.rs` | JPD author signature detection |
| `glassware-core/src/forcememo_detector.rs` | Python injection detection |
| `glassware-orchestrator/src/main.rs` | Rust orchestrator CLI |
| `harness/diverse_sampling.py` | npm package sampling |
| `harness/optimized_scanner.py` | High-performance scanner |
| `harness/batch_llm_analyzer.py` | NVIDIA LLM batch analysis |
| `harness/waves.toml` | Wave campaign configuration |
| `docs/WORKFLOW-GUIDE.md` | Complete workflow documentation |
| `CODEREVIEW.md` | Comprehensive code review (v0.11.7) |

---

## Development Conventions

### Adding New Detectors

1. Create detector in `glassware-core/src/detectors/` or `glassware-core/src/`
2. Implement `Detector` trait (see `detector.rs`)
3. Register in `glassware-core/src/engine.rs`
4. Add category to `glassware-core/src/finding.rs`
5. Write tests in `glassware-core/tests/`
6. Update `CODEREVIEW.md` or `HANDOFF.md`

### Detector Implementation Pattern

```rust
use glassware_core::{Detector, DetectorTier, FileIR, Finding, Severity};

pub struct MyDetector;

impl Detector for MyDetector {
    fn name(&self) -> &str { "my_detector" }

    fn tier(&self) -> DetectorTier { DetectorTier::Tier1Primary }

    fn detect(&self, ir: &FileIR) -> Vec<Finding> {
        // Access pre-parsed content via ir.content()
        // Access pre-parsed JSON via ir.json()
        // Access Unicode analysis via ir.unicode()
        vec![]
    }

    fn cost(&self) -> u8 { 3 }  // 1-10, lower = cheaper

    fn signal_strength(&self) -> u8 { 8 }  // 1-10, higher = lower FP rate
}
```

### Testing Practices

```bash
# Run all tests
cargo test --features "full,llm"

# Test specific detector
cargo test --lib rdd_detector
cargo test --lib forcememo_detector

# Integration tests
cargo test --test integration_campaign_fixtures
cargo test --test integration_false_positives
```

### Code Style

- **Rust**: Follow standard Rust conventions (rustfmt)
- **Python**: Use type hints, follow PEP 8
- **Error handling**: Use `anyhow`/`thiserror` in Rust
- **Logging**: Use `tracing` crate with `tracing-subscriber`

---

## Known Issues (from CODEREVIEW.md)

### P0 - Critical (Fix Before Large-Scale Scans)

1. **Parallel workflow bug** - Agent compaction causes duplicate registry entries
2. **Hardcoded paths** in Python scripts (`/home/property.sightlines/...`)
3. **Version validation gap** - 47% failure rate due to unvalidated versions

### P1 - High Priority

1. **LLM fallback chain incomplete** - Only first model used, fallbacks ignored
2. **Threat score edge cases** - No per-component capping
3. **Evidence integrity** - No SHA-256 hashing or chain of custody

### P2 - Medium Priority

1. **DAG execution disabled by default** - Wastes CPU on negative scans
2. **Cache database** - No VACUUM/compaction for large scans
3. **Rate limiting** - Not enforced in Python harness

---

## Performance Metrics

| Metric | Value |
|--------|-------|
| Binary size | ~11 MB |
| Scan speed | ~50k LOC/sec |
| npm scan | ~0.5s per package (with cache) |
| GitHub scan | ~5-20s per repo |
| Rust vs Python | 1.5x faster (Rust orchestrator) |
| Test coverage | 223 tests passing |

---

## Quick Reference

### Common Commands

```bash
# Project scan
glassware /path/to/project --format json --severity high

# npm packages
glassware-orchestrator scan-npm express@4.19.2 lodash@4.17.21

# GitHub repo
glassware-orchestrator scan-github owner/repo

# Wave campaign
cd harness && python3 -m core.orchestrator run-wave --wave 0 --llm

# Resume interrupted scan
glassware-orchestrator resume npm --packages express lodash
```

### Output Formats

- **Pretty print** (default): Human-readable terminal output
- **JSON**: `--format json` for programmatic processing
- **SARIF**: `--format sarif` for GitHub Security tab

---

## Documentation

| Document | Purpose |
|----------|---------|
| [README.md](README.md) | Project overview and quick start |
| [docs/WORKFLOW-GUIDE.md](docs/WORKFLOW-GUIDE.md) | Complete scan/analyze/improve workflow |
| [CODEREVIEW.md](CODEREVIEW.md) | Comprehensive code review with fixes |
| [harness/README.md](harness/README.md) | Python harness tools documentation |
| [docs/USER-GUIDE.md](docs/USER-GUIDE.md) | End-user guide |

---

**Last Updated:** 2026-03-22
**Version:** 0.8.0 (workspace), 0.11.7 (scanner)
**Status:** Near production-ready (fix P0 issues first)
