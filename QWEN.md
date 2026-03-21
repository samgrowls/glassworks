# glassworks - Project Context

**Last updated:** 2026-03-20
**Version:** 0.8.0

## Project Overview

**glassworks** is a comprehensive security tool for detecting steganographic payloads, invisible Unicode characters, bidirectional text attacks, and behavioral evasion patterns in source code and npm/GitHub packages.

The project was created in response to the GlassWare threat campaign (active since October 2025) which compromised 72+ VS Code extensions and 150+ GitHub repositories using invisible Unicode steganography.

### Architecture

This is a **Cargo workspace** with four members:

| Package | Description |
|---------|-------------|
| `glassware-core` | Core detection library with three-layer detection (regex, semantic, LLM) |
| `glassware-cli` | CLI binary (`glassware`) for scanning files/directories |
| `glassware-orchestrator` | Advanced orchestration with GitHub scanning, checkpointing, rate limiting |
| `llm-analyzer` | LLM analysis module for intent-level reasoning |

### Python Harness

The `harness/` directory contains Python-based scanning tools:

| Script | Purpose |
|--------|---------|
| `diverse_sampling.py` | Async npm registry queries with Tier 1 filtering |
| `optimized_scanner.py` | Main orchestrator with progress tracking |
| `github_scanner.py` | GitHub repository scanning |
| `batch_llm_analyzer.py` | LLM analysis on flagged packages |
| `database.py` | SQLite corpus management |
| `reporter.py` | Markdown report generation |

---

## Three-Layer Detection System

```
┌─────────────────────────────────────────────────────────────┐
│                     ScanEngine                               │
├─────────────────────────────────────────────────────────────┤
│  L1: Regex Detectors (all files)                            │
│      - InvisibleCharDetector                                │
│      - HomoglyphDetector                                    │
│      - BidiDetector                                         │
│      - GlasswareDetector                                    │
│      - UnicodeTagDetector                                   │
│      - EncryptedPayloadDetector (GW005 regex)               │
│      - HeaderC2Detector (GW008 regex)                       │
│      - LocaleGeofencingDetector                             │
│      - TimeDelayDetector                                    │
│      - BlockchainC2Detector                                 │
│      - RddDetector                                          │
│      - ForceMemoDetector                                    │
│      - JpdAuthorDetector                                    │
├─────────────────────────────────────────────────────────────┤
│  L2: Semantic Detectors (JS/TS only, requires OXC)          │
│      - Gw005SemanticDetector (stego → exec flow)            │
│      - Gw006SemanticDetector (hardcoded key → exec)         │
│      - Gw007SemanticDetector (RC4 cipher → exec)            │
│      - Gw008SemanticDetector (header C2 → decrypt → exec)   │
├─────────────────────────────────────────────────────────────┤
│  L3: LLM Review (flagged files only, optional)              │
│      - OpenAiCompatibleAnalyzer                             │
│      - Intent-level reasoning                               │
│      - False positive reduction                             │
└─────────────────────────────────────────────────────────────┘
```

### Tiered Detection (v0.3.1+)

| Tier | When Run | Detectors | FP Rate |
|------|----------|-----------|---------|
| **Tier 1** | Always | Invisible, Homoglyph, Bidi, UnicodeTag | <1% |
| **Tier 2** | If Tier 1 finds OR file not minified | Glassware, EncryptedPayload, HeaderC2 | ~2% |
| **Tier 3** | Only if Tier 1+2 find | Locale, TimeDelay, BlockchainC2, RDD, ForceMemo, JPD | ~5% |

---

## Key Detection Capabilities

### L1 Regex Detectors

| ID | Detector | Severity | Description |
|----|----------|----------|-------------|
| GW001 | InvisibleChar | Critical | Zero-width chars, variation selectors |
| GW002 | Homoglyph | Medium-High | Mixed-script identifiers (Cyrillic/Greek) |
| GW003 | BidiOverride | Critical | Trojan Source bidirectional overrides |
| GW004 | UnicodeTag | High | Unicode tag characters (U+E0001–U+E007F) |
| GW005 | GlasswarePattern | High | Stego decoder patterns |
| GW006 | EncryptedPayload | High | High-entropy blob + dynamic execution |
| GW007 | HeaderC2 | Critical | HTTP header extraction + decrypt → exec |
| GW008 | LocaleGeofencing | Medium | Russian locale checks |
| GW009 | TimeDelay | Low | Sandbox evasion delays |
| GW010 | BlockchainC2 | High | Solana/Google Calendar C2 |
| GW011 | RddDetector | High | URL dependencies (PhantomRaven) |
| GW012 | ForceMemo | Critical | Python repo injection |
| GW013 | JpdAuthor | Critical | "JPD" author signature |

### L2 Semantic Detectors (JS/TS Only)

| ID | Detector | Purpose |
|----|----------|---------|
| L2-GW005 | Gw005Semantic | Stego → exec flow |
| L2-GW006 | Gw006Semantic | Hardcoded key → exec |
| L2-GW007 | Gw007Semantic | RC4 cipher → exec |
| L2-GW008 | Gw008Semantic | Header C2 → decrypt → exec |

### L3 LLM Review

| Component | Purpose |
|-----------|---------|
| OpenAiCompatibleAnalyzer | Intent-level reasoning, FP reduction |

---

## Building and Running

### Prerequisites

- Rust 1.70 or later
- Python 3.10+ (for harness scripts)
- Optional: NVIDIA/Groq/Cerebras API key for LLM analysis

### Feature Flags

| Feature | Description | Default |
|---------|-------------|---------|
| `full` | All features enabled | ✅ Yes |
| `minimal` | Only invisible chars + bidi (no regex) | ❌ No |
| `semantic` | OXC-based semantic analysis (JS/TS only) | ✅ Yes (via `full`) |
| `llm` | LLM review layer | ✅ Yes (via `full`) |
| `cache` | Incremental scanning with caching | ✅ Yes (via `full`) |
| `serde` | Serialization support | ✅ Yes (via `full`) |

### Build Commands

```bash
# Build entire workspace (debug)
cargo build

# Build with all features
cargo build --features "full,llm"

# Build release (optimized binary)
cargo build --release

# Minimal build (smallest binary)
cargo build --no-default-features --features minimal

# Run CLI directly
cargo run -- project/

# Run tests
cargo test --features "full,llm"

# Run tests for specific package
cargo test -p glassware-core
cargo test -p glassware-cli

# Install CLI globally
cargo install --path glassware-cli
```

### CLI Usage

```bash
# Scan a directory
glassware .

# Scan specific files
glassware src/index.js package.json

# JSON output
glassware --format json .

# SARIF output (GitHub Advanced Security)
glassware --format sarif . > results.sarif

# Only critical/high findings
glassware --severity high .

# Silent mode — exit code only
glassware --quiet .

# With caching (10x re-scan speedup)
glassware --cache-file .glassware-cache.json .

# LLM analysis (requires API key)
glassware --llm .
```

### CLI Options

| Flag | Description | Default |
|------|-------------|---------|
| `--format`, `-f` | Output format: `pretty`, `json`, `sarif` | `pretty` |
| `--severity`, `-s` | Minimum severity: `info`, `low`, `medium`, `high`, `critical` | `low` |
| `--quiet`, `-q` | Suppress output, only set exit code | `false` |
| `--no-color` | Disable colored output | `false` |
| `--extensions` | File extensions to include (comma-separated) | `js,mjs,cjs,ts,tsx,jsx,py,rs,go,...` |
| `--exclude` | Directories to exclude (comma-separated) | `.git,node_modules,target,...` |
| `--cache-file` | Cache file for incremental scanning | `.glassware-cache.json` |
| `--no-cache` | Disable caching | `false` |
| `--llm` | Run LLM analysis on flagged files | `false` |

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | No findings at or above severity threshold |
| 1 | Findings detected |
| 2 | Error (file not found, permission denied) |

---

## Python Harness Usage

### Scan npm Packages

```bash
cd harness

# 1. Sample packages by category
python3 diverse_sampling.py \
  --categories ai-ml native-build install-scripts \
  --samples-per-keyword 20 \
  --output packages.txt

# 2. Scan packages
python3 optimized_scanner.py \
  packages.txt \
  -w 10 \              # Workers
  -e data/evidence/scan-1 \
  -o scan-1-results.json

# 3. Check results
cat results.json | jq '{scanned, flagged, errors}'
```

### Scan GitHub Repositories

```bash
cd harness

# Scan repositories
python3 github_scanner.py \
  --queries "mcp" "vscode" "cursor" \
  --repos-per-query 50 \
  --max-repos 200 \
  --output github-scan.json

# Check results
cat github-scan.json | jq '{scanned, flagged, errors}'
```

### LLM Analysis

```bash
# Set API key
export NVIDIA_API_KEY="nvapi-..."

# Run LLM on flagged packages
python3 batch_llm_analyzer.py \
  flagged.txt \
  -w 2 \
  -o llm-results.json
```

---

## LLM Layer Configuration

### Environment Variables

```bash
# Required for --llm flag
export GLASSWARE_LLM_BASE_URL="https://api.cerebras.ai/v1"
export GLASSWARE_LLM_API_KEY="your-api-key"

# Optional (defaults to llama-3.3-70b)
export GLASSWARE_LLM_MODEL="llama-3.3-70b"
```

### Supported Providers

| Provider | Base URL | Recommended Model |
|----------|----------|-------------------|
| Cerebras | `https://api.cerebras.ai/v1` | `llama-3.3-70b` |
| Groq | `https://api.groq.com/openai/v1` | `llama-3.3-70b-versatile` |
| OpenAI | `https://api.openai.com/v1` | `gpt-4o` |
| NVIDIA NIM | `https://integrate.api.nvidia.com/v1` | `meta/llama-3.3-70b-instruct` |
| Ollama (local) | `http://localhost:11434/v1` | `llama3.3` |

### .env File

```bash
cp .env.example .env
# Edit .env with your credentials
```

---

## Core Modules (glassware-core)

```
glassware-core/src/
├── lib.rs                      # Main library entry point
├── detector.rs                 # Detector trait with tier support
├── engine.rs                   # ScanEngine orchestrator
├── finding.rs                  # Finding, DetectionCategory, Severity
├── config.rs                   # UnicodeConfig, DetectorConfig, ScanConfig
├── scanner.rs                  # UnicodeScanner (L1 regex detectors)
├── minified.rs                 # Minified code detection
├── cache.rs                    # Incremental scanning (10x speedup)
├── semantic.rs                 # OXC semantic analysis (L2)
├── taint.rs                    # Taint tracking: source/sink/flow
├── cross_file_taint.rs         # Cross-file taint propagation
├── module_graph.rs             # ES6/CommonJS module graph
├── decoder.rs                  # Steganographic payload decoder
├── classify.rs                 # Character classification
├── ranges.rs                   # Unicode range definitions
├── script_detector.rs          # Script detection for homoglyphs
├── attack_graph.rs             # Attack chain correlation
├── campaign.rs                 # Campaign intelligence tracking
├── risk_scorer.rs              # Contextual risk scoring
├── ir.rs                       # Unified intermediate representation
├── correlation.rs              # Finding correlation
├── confusables/                # Confusable character data
├── detectors/                  # L1 regex detectors
│   ├── mod.rs
│   ├── invisible.rs
│   ├── homoglyph.rs
│   ├── bidi.rs
│   ├── glassware.rs
│   └── tags.rs
├── encrypted_payload_detector.rs   # GW005 regex detector
├── header_c2_detector.rs           # GW007 regex detector
├── locale_detector.rs              # Locale geofencing
├── time_delay_detector.rs          # Time delay detection
├── blockchain_c2_detector.rs       # Blockchain C2 detection
├── rdd_detector.rs                 # RDD (URL dependency) detection
├── forcememo_detector.rs           # Python repo injection
├── jpd_author_detector.rs          # JPD author signature
├── gw005_semantic.rs               # GW005 semantic detector
├── gw006_semantic.rs               # GW006 semantic detector
├── gw007_semantic.rs               # GW007 semantic detector
├── gw008_semantic.rs               # GW008 semantic detector
├── adversarial/                    # Adversarial testing framework
│   ├── mod.rs
│   ├── strategies.rs
│   └── generator.rs
└── llm/                            # L3 LLM layer
    ├── mod.rs
    ├── config.rs
    └── analyzer.rs
```

---

## Development Conventions

### Code Style

- **Edition**: Rust 2021
- **Documentation**: All public types, functions, and enum variants have doc comments
- **Public API**: Re-exported from `lib.rs` for convenient access
- **Features**: `full`, `minimal`, `semantic`, `llm`, `cache`, `serde` for conditional compilation
- **Linting**: `cargo clippy -- -D warnings` must pass with no warnings

### Testing Practices

- Unit tests in each module using `#[cfg(test)]`
- Integration tests in `glassware-core/tests/` with 38+ test fixtures
- Test fixtures cover: GlassWare campaign patterns, false positives, edge cases
- Decoder tests include round-trip encoding/decoding verification
- Entropy analysis tests for payload classification
- LLM tests use mocks (no network calls)
- **All tests must pass before merging** (with `full,llm` features)

### Adding New Detectors

#### L1 Regex Detector

1. Create new detector module in `glassware-core/src/detectors/`
2. Implement `Detector` trait
3. Add to `DetectorConfig` in `config.rs`
4. Register in `UnicodeScanner` in `scanner.rs`
5. Add to `DetectionCategory` enum in `finding.rs`
6. Assign appropriate tier (Tier1/Tier2/Tier3)

#### L2 Semantic Detector

1. Create `gwXXX_semantic.rs` in `glassware-core/src/`
2. Implement `SemanticDetector` trait
3. Register in `ScanEngine::default_detectors()`
4. Add to `DetectionCategory` enum in `finding.rs`
5. Write tests for true positives and false positives

### Quality Checks

```bash
# Format code
cargo fmt --all

# Run clippy (must pass with no warnings)
cargo clippy --features "full,llm" -- -D warnings

# Run all tests
cargo test --features "full,llm"

# Build documentation
cargo doc --no-deps --features "full,llm"

# Test all feature combinations
cargo test --no-default-features
cargo test --features "full"
cargo test --features "full,llm"
```

---

## Performance Benchmarks

| Metric | Value |
|--------|-------|
| Binary size (minimal) | ~1.2 MB |
| Binary size (full) | ~8-11 MB |
| Scan speed | ~50k LOC/sec |
| Memory usage | ~50 MB peak |
| L1 detection latency | O(n) single pass |
| L2 detection latency | O(n²) worst case |
| L3 detection latency | ~2-5 sec per file (API dependent) |
| Cache speedup (re-scan) | ~10x |
| npm scan | ~0.5s per package (with cache) |
| GitHub scan | ~5-20s per repo |

---

## Test Corpus

**147+ tests** across 4 feature combinations:

| Category | Count |
|----------|-------|
| Unit tests (glassware-core) | 120+ |
| Campaign fixture tests | 12 (3 ignored) |
| False positive tests | 13 |
| Edge case tests | 14 (4 ignored) |
| Scan directory tests | 6 |
| Doc tests | 4 |

**Test fixtures:** 38 files in `glassware-core/tests/fixtures/`
- `glassworm/` (12): Reconstructed GlassWare campaign patterns
- `false_positives/` (12): Legitimate code that should NOT trigger
- `edge_cases/` (14): Obfuscation techniques documenting limitations

---

## Campaign Intelligence

### Detected Campaigns

| Campaign | Detection Method | Coverage |
|----------|------------------|----------|
| **GlassWorm Core** | Unicode stego + behavioral | ✅ 100% |
| **PhantomRaven** | RDD + JPD author | ✅ 100% |
| **ForceMemo** | Python markers | ✅ 100% |
| **Chrome RAT** | Blockchain C2 | ✅ 100% |
| **React Native** | Encrypted payload | ✅ 100% |

### Attack Graph Engine

Correlates findings into attack chains:

| Attack Chain Type | Description |
|-------------------|-------------|
| GlassWareStego | Stego payload → decoder → exec |
| EncryptedExec | Encrypted blob → decrypt → exec |
| HeaderC2Chain | Header C2 → decrypt → exec |
| LocaleGeofencing | Locale check → conditional exec |
| TimeDelayEvasion | Delay → network call |
| BlockchainC2Chain | Blockchain fetch → decode → exec |

---

## Key Files

| File | Purpose |
|------|---------|
| `HANDOFF.md` | Current status & quick start |
| `HANDOFF-WORKFLOW.md` | Production workflow guide |
| `README.md` | Project overview |
| `RELEASE.md` | Release notes |
| `TODO.md` | Current priorities |
| `DOCUMENTATION-CATALOG.md` | All documents catalogued |
| `docs/WORKFLOW-GUIDE.md` | Complete scan/analyze/improve workflow |
| `harness/reports/` | Scan reports & analysis |

---

## Troubleshooting

### npm 429 Rate Limit

```bash
# Increase delay between requests
python3 diverse_sampling.py --delay-between-keywords 2.0

# Reduce retries
python3 diverse_sampling.py --npm-retries 2
```

### GitHub 403 Rate Limit

```bash
# Add token for higher rate limits
export GITHUB_TOKEN="ghp_..."

# Or wait 60s (automatic backoff built-in)
```

### High False Positive Rate

```bash
# Check if scanning minified files (should skip by default)
glassware src/

# If you need to scan bundled code
glassware --analyze-bundled src/

# Or disable tiered detection entirely
glassware --no-tiered src/
```

### Cache Not Working

```bash
# Verify cache file exists
ls -la .glassware-cache.json

# Check cache stats in output
glassware --cache-file .glassware-cache.json src/
# Look for "Cache: X hits, Y misses" in output
```

### LLM Not Working

```bash
# Verify environment variables
echo $GLASSWARE_LLM_BASE_URL
echo $GLASSWARE_LLM_API_KEY

# Test connection
curl -H "Authorization: Bearer $GLASSWARE_LLM_API_KEY" \
  $GLASSWARE_LLM_BASE_URL/models
```

---

## Related Projects

- **Coax**: Full code trust scanner (secrets detection, Unicode attacks, entropy analysis). glassware's detection engine originated from Coax.
- **anti-trojan-source**: JavaScript-based Trojan Source detector (less feature-complete than glassware)

---

## Security Notice

**This tool is for defensive security research only.**

- Use responsibly
- Respect rate limits
- Don't scan without permission
- Report findings responsibly to package maintainers and npm Security

---

## License

MIT License - see [LICENSE](LICENSE)
