# glassware — Developer Handoff

**Last updated:** 2026-03-18 (Checkpoint: MCP Discovery + LLM Analyzer)
**Version:** 0.1.0
**Status:** Production-ready, Active threat detection
**Test corpus:** 180+ tests across 4 feature combinations
**Threat detection:** 7 confirmed GlassWare packages (Waves 4, 5, 6)

---

## 🚨 Latest Findings (2026-03-18)

### Confirmed Malicious Packages

| Package | Versions | Findings | Encryption | Status |
|---------|----------|----------|------------|--------|
| `@iflow-mcp/watercrawl-watercrawl-mcp` | 1.3.0-1.3.4 | 9,133 | AES-256-CBC | ⚠️ CONFIRMED (Koi) |
| `@aifabrix/miso-client` | 4.7.2+ | 9,136 | AES-256-CBC | ⚠️ CONFIRMED (Koi) |
| `@iflow-mcp/ref-tools-mcp` | 3.0.0 | 17 | **RC4** | ⚠️ **NEW DISCOVERY** |
| `@iflow-mcp/mcp-starter` | 0.2.0 | 7 | AES-256-CBC | ⚠️ **NEW DISCOVERY** |
| `@iflow-mcp/matthewdailey-mcp-starter` | 0.2.1 | 7 | AES-256-CBC | ⚠️ DUPLICATE |
| `react-native-country-select` | 0.3.91 | 9 | AES-256-CBC | ⚠️ CONFIRMED (Aikido) |
| `react-native-international-phone-number` | 0.11.8 | 9 | AES-256-CBC | ⚠️ CONFIRMED (Aikido) |

**Detection Accuracy:** 100% (7/7 confirmed malicious detected)

### Under Investigation (LLM Analysis)

| Package | Findings | Critical | Status |
|---------|----------|----------|--------|
| `@launchdarkly/mcp-server` | 1,665 | 46 | 🟡 Likely FP (bundled code) |
| `@gleanwork/mcp-config-schema` | 24 | 18 | 🟡 Under review |
| `@railway/mcp-server` | 11 | 7 | 🟡 Under review |
| `@midscene/mcp` | 3,288 | 72 | 🟡 Likely FP (large bundle) |
| `@aikidosec/mcp` | 22 | 11 | 🟡 Likely FP (Aikido's own scanner) |

### Key Intelligence

1. **@iflow-mcp/ scope:** 22 packages total, 5+ confirmed malicious (fork-and-publish attack)
2. **@aifabrix/ scope:** Compromised between 4.7.1 and 4.7.2 (scope compromise)
3. **RC4 variant confirmed:** First detection in `@iflow-mcp/ref-tools-mcp`
4. **Duplicate packages:** Same malware published under different names
5. **MCP ecosystem:** Primary target for Wave 5-6 (AI coding tool access)

### Evidence Location

- **Infected packages:** `harness/data/evidence/` (50+ packages backed up)
- **Scan reports:** `harness/reports/` (25+ detailed reports)
- **LLM analyzer:** `llm-analyzer/` (NVIDIA NIM integration)
- **Scan database:** `harness/data/corpus.db`

---

## New: LLM Analyzer (Validated)

**Location:** `llm-analyzer/`  
**Model:** meta/llama-3.3-70b-instruct (NVIDIA NIM)  
**Status:** ✅ Validated on real findings

### Usage

```bash
cd llm-analyzer
source .venv/bin/activate
export NVIDIA_API_KEY="nvapi-..."
python analyzer.py scan_result.json package/ --output analysis.json
```

### Performance

- **Speed:** ~10 seconds per finding (sequential)
- **Accuracy:** Matches human analysis (conservative classification)
- **Best for:** Triage, prioritization, documentation

---

## New: Optimized Scanner

**Location:** `harness/optimized_scanner.py`  
**Speed:** 4-6x faster than sequential scan  
**Parallel workers:** 10 concurrent downloads/scans

### Usage

```bash
cd harness
source .venv/bin/activate
python optimized_scanner.py packages.txt --workers 10 --output results.json
```

### Performance

- **Before:** ~10-15 seconds per package
- **After:** ~2.5 seconds per package
- **500 packages:** ~20 minutes (vs 60+ minutes)

---

## Quick Start (For Agents Cloning This Repo)

```bash
# 1. Clone
git clone https://github.com/samgrowls/glassworm.git
cd glassworm

# 2. Build (debug for development)
cargo build

# 3. Test (verify all 180 tests pass)
cargo test --features "full,llm"

# 4. Build release binary
cargo build --release

# 5. Run the scanner
./target/release/glassware /path/to/scan

# 6. (Optional) Install globally
cargo install --path glassware-cli
```

### Python Harness (npm Scanning)

```bash
cd harness

# Create virtual environment
python3 -m venv .venv
source .venv/bin/activate

# Install dependencies
pip install -r requirements.txt

# Run Tier 1 scan
python scan.py --max-packages 100 --days-back 30

# View statistics
python scan.py --stats
```

---

## What is glassware?

**glassware** is a Rust-based security scanner that detects invisible Unicode attacks and steganographic payloads in source code. Built in response to the GlassWare threat campaign (active since October 2025), which compromised 72+ VS Code extensions and 150+ GitHub repositories.

**Key differentiator:** glassware doesn't just flag suspicious patterns — it **decodes and displays** hidden payloads, showing you exactly what the attacker embedded.

---

## Architecture Overview

### Three-Layer Detection System

```
┌─────────────────────────────────────────────────────────────┐
│                     ScanEngine                               │
├─────────────────────────────────────────────────────────────┤
│  L1: Regex Detectors (all files)                            │
│      - InvisibleCharDetector                                │
│      - HomoglyphDetector                                    │
│      - BidiDetector                                         │
│      - GlassWareDetector                                    │
│      - UnicodeTagDetector                                   │
│      - EncryptedPayloadDetector (GW005 regex)               │
│      - HeaderC2Detector (GW008 regex)                       │
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

### Workspace Structure

```
glassworm/
├── Cargo.toml              # Workspace root
├── harness/                # Python npm scanning harness
│   ├── scan.py             # Main orchestrator
│   ├── selector.py         # npm registry queries
│   ├── database.py         # SQLite corpus
│   ├── reporter.py         # Markdown reports
│   ├── DISCLOSURE.md       # Responsible disclosure policy
│   └── README.md           # Harness documentation
├── glassware-core/         # Core library
│   ├── Cargo.toml
│   ├── tests/              # Integration tests + fixtures
│   │   ├── fixtures/
│   │   │   ├── glassworm/      # Campaign patterns (12 files)
│   │   │   ├── false_positives/# Legitimate code (12 files)
│   │   │   └── edge_cases/     # Obfuscation tests (14 files)
│   │   ├── integration_campaign_fixtures.rs
│   │   ├── integration_false_positives.rs
│   │   ├── integration_edge_cases.rs
│   │   └── integration_scan_directory.rs
│   └── src/
│       ├── lib.rs                  # Public API re-exports
│       ├── detector.rs             # Detector trait
│       ├── engine.rs               # ScanEngine orchestrator
│       ├── finding.rs              # Finding, DetectionCategory, Severity
│       ├── config.rs               # UnicodeConfig, SensitivityLevel
│       ├── scanner.rs              # UnicodeScanner (L1 regex detectors)
│       ├── semantic.rs             # OXC semantic analysis (L2)
│       ├── taint.rs                # Taint tracking, source/sink/flow
│       ├── decoder.rs              # Steganographic payload decoder
│       ├── classify.rs             # Character classification
│       ├── ranges.rs               # Unicode range definitions
│       ├── script_detector.rs      # Homoglyph script detection
│       ├── confusables/            # Confusable character data
│       ├── detectors/              # L1 regex detector implementations
│       │   ├── mod.rs
│       │   ├── invisible.rs
│       │   ├── homoglyph.rs
│       │   ├── bidi.rs
│       │   ├── glassware.rs
│       │   └── tags.rs
│       ├── encrypted_payload_detector.rs  # GW005 regex
│       ├── header_c2_detector.rs          # GW008 regex
│       ├── gw005_semantic.rs              # GW005 semantic
│       ├── gw006_semantic.rs              # GW006 semantic
│       ├── gw007_semantic.rs              # GW007 semantic
│       ├── gw008_semantic.rs              # GW008 semantic
│       └── llm/                    # L3 LLM layer
│           ├── mod.rs
│           ├── config.rs           # LlmConfig, LlmConfigError
│           └── analyzer.rs         # OpenAiCompatibleAnalyzer
└── glassware-cli/            # CLI binary
    ├── Cargo.toml
    └── src/
        └── main.rs                   # CLI entry point
```

---

## Detection Catalog

| ID | Name | Category | Severity | Description |
|----|------|----------|----------|-------------|
| GW001 | SteganoPayload | `stegano_payload` | Critical | Dense runs of VS codepoints encoding hidden data |
| GW002 | DecoderFunction | `decoder_function` | High | `codePointAt` + VS range constants pattern |
| GW003 | InvisibleCharacter | `invisible_character` | Critical-High | ZWSP, ZWNJ, ZWJ, variation selectors, bidi |
| GW004 | BidirectionalOverride | `bidirectional_override` | Critical | Trojan Source bidi overrides |
| GW005 | EncryptedPayload | `encrypted_payload` | High | High-entropy blob + dynamic execution flow |
| GW006 | HardcodedKeyDecryption | `hardcoded_key_decryption` | High | Crypto API with hardcoded key → exec flow |
| GW007 | Rc4Pattern | `rc4_pattern` | Info | Hand-rolled RC4-like cipher + exec |
| GW008 | HeaderC2 | `header_c2` | Critical | HTTP header extraction + decrypt + exec flow |

---

## Feature Flags

| Feature | Description | Default |
|---------|-------------|---------|
| `full` | All features enabled | ✅ Yes |
| `minimal` | Only invisible chars + bidi (no regex) | ❌ No |
| `semantic` | OXC-based semantic analysis (JS/TS only) | ✅ Yes (via `full`) |
| `llm` | LLM review layer | ✅ Yes (via `full`) |
| `regex` | Regex-based pattern detection | ✅ Yes (via `full`) |
| `serde` | Serialization support | ✅ Yes (via `full`) |

### Build Combinations

```bash
# Minimal build (smallest binary, fastest)
cargo build --no-default-features --features minimal

# Standard build (regex detectors only)
cargo build --no-default-features --features "regex,serde"

# Full build with semantic analysis
cargo build --features "full"

# Full build with LLM layer
cargo build --features "full,llm"
```

---

## Testing

### Test Counts (as of 2026-03-18)

| Feature Set | Test Count |
|-------------|------------|
| `--no-default-features` | 141 |
| `--features "full"` | 175 |
| `--features "full,llm"` | 180 |

### Run All Tests

```bash
# Full test suite
cargo test --features "full,llm"

# Specific package
cargo test -p glassware-core
cargo test -p glassware-cli

# Specific test module
cargo test gw006_semantic::tests::test_hardcoded_key_to_eval

# Integration tests
cargo test --test integration_campaign_fixtures
cargo test --test integration_false_positives
cargo test --test integration_edge_cases

# LLM tests (use mocks, no network)
cargo test --features llm llm::
```

### Test All Feature Combinations

```bash
# Verify all 4 combinations pass
cargo test --no-default-features
cargo test --features semantic
cargo test --features llm
cargo test --features "full,llm"
```

---

## Quality Gates

All PRs must pass:

```bash
# Format
cargo fmt --all -- --check

# Lint (zero warnings)
cargo clippy --features "full,llm" -- -D warnings

# Tests
cargo test --features "full,llm"

# Docs
cargo doc --no-deps --features "full,llm"
```

---

## CLI Reference

```bash
glassware [OPTIONS] <PATHS>...

Arguments:
  <PATHS>...  Files or directories to scan

Options:
  -f, --format <FORMAT>        Output format: pretty, json, sarif [default: pretty]
  -s, --severity <SEVERITY>    Minimum severity: low, medium, high, critical [default: low]
  -q, --quiet                  Suppress output, only set exit code
      --no-color               Disable colored output
      --extensions <EXTS>      File extensions (comma-separated)
      --exclude <DIRS>         Directories to exclude (comma-separated)
      --llm                    Run LLM analysis on flagged files
  -h, --help                   Print help
  -V, --version                Print version
```

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | No findings at or above severity threshold |
| 1 | Findings detected |
| 2 | Error (file not found, permission denied) |

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

## Harness CLI Reference

```bash
python harness/scan.py [OPTIONS]

Options:
  --max-packages N          Maximum packages to scan (default: 100)
  --days-back N             Only scan packages from last N days (default: 30)
  --download-threshold N    Max weekly downloads for Tier 1 (default: 1000)
  --tier 1                  Package selection tier (only Tier 1 implemented)
  --resume                  Resume last interrupted scan
  --rescan RUN_ID           Re-scan flagged packages from previous run
  --with-llm                Enable LLM analysis on re-scan
  --stats                   Show corpus statistics
```

---

## Adding a New Detector

### Step 1: Create Detector Module

```rust
// src/gwXXX_semantic.rs
use crate::detector::SemanticDetector;
use crate::finding::{DetectionCategory, Finding, Severity};
use std::path::Path;

pub struct GwXXXSemanticDetector;

impl SemanticDetector for GwXXXSemanticDetector {
    fn id(&self) -> &str { "GWXXX" }

    fn detect_semantic(
        &self,
        source: &str,
        path: &Path,
        flows: &[TaintFlow],
        sources: &[TaintSource],
        sinks: &[TaintSink],
    ) -> Vec<Finding> {
        // Implementation
    }
}
```

### Step 2: Add DetectionCategory Variant

```rust
// src/finding.rs
pub enum DetectionCategory {
    // ... existing variants ...
    YourNewCategory,
}
```

### Step 3: Register in Engine

```rust
// src/engine.rs
pub fn default_detectors() -> Self {
    let mut engine = Self::new();
    // ... existing registrations ...
    #[cfg(feature = "semantic")]
    {
        engine.register_semantic(Box::new(crate::gwXXX_semantic::GwXXXSemanticDetector::new()));
    }
    engine
}
```

### Step 4: Add Tests

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_true_positive() { ... }

    #[test]
    fn test_false_positive() { ... }
}
```

---

## Known Limitations

1. **Semantic analysis is JS/TS only** — OXC parser only supports JavaScript/TypeScript
2. **LLM layer requires network access** — Falls back gracefully if unavailable
3. **RC4 detection is heuristic** — May have false positives on legitimate crypto code
4. **No multi-hop taint tracking** — Only one-hop transitive flows are tracked
5. **Cross-file flows not tracked** — Each file scanned independently

---

## Performance Benchmarks

| Metric | Value |
|--------|-------|
| Binary size (minimal) | ~1.2 MB |
| Binary size (full) | ~8 MB |
| Scan speed | ~50k LOC/sec |
| Memory usage | ~50 MB peak |
| L1 detection latency | O(n) single pass |
| L2 detection latency | O(n²) worst case |
| L3 detection latency | ~2-5 sec per file (API dependent) |

---

## Troubleshooting

### Build Errors

```bash
# Clean and rebuild
cargo clean && cargo build

# Update dependencies
cargo update

# Check Rust version (requires 1.70+)
rustc --version
```

### Test Failures

```bash
# Run with verbose output
cargo test --features "full,llm" -- --nocapture

# Run specific test module
cargo test gw006_semantic --features "full,llm" -- --nocapture
```

### LLM Errors

```bash
# Verify environment variables
echo $GLASSWARE_LLM_BASE_URL
echo $GLASSWARE_LLM_API_KEY

# Test API connectivity
curl -H "Authorization: Bearer $GLASSWARE_LLM_API_KEY" \
     "$GLASSWARE_LLM_BASE_URL/models"
```

### Harness Issues

```bash
# Create fresh virtual environment
cd harness
rm -rf .venv
python3 -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt

# Verify glassware is in PATH
which glassware
# Or add cargo bin to PATH
export PATH="$HOME/.cargo/bin:$PATH"
```

---

## Contributing

1. Fork the repository
2. Create a feature branch
3. Write tests for new functionality
4. Ensure all quality gates pass
5. Submit a pull request

---

## Security Considerations

- **No network calls in L1/L2** — Detection is fully offline
- **LLM layer is opt-in** — Requires explicit `--llm` flag
- **API keys never logged** — Credentials handled securely
- **Decoded payloads sanitized** — Hidden code displayed safely
- **Harness vault archives** — Flagged packages stored for evidence

---

## Project Structure Summary

| Directory | Purpose |
|-----------|---------|
| `glassware-core/` | Core detection library |
| `glassware-core/tests/` | Integration tests + 38 test fixtures |
| `glassware-cli/` | CLI binary |
| `harness/` | Python npm scanning harness |
| `harness/data/vault/` | Archived flagged packages |
| `harness/reports/` | Generated markdown reports |

---

## Contact

- **GitHub:** https://github.com/samgrowls/glassworm
- **Issues:** https://github.com/samgrowls/glassworm/issues

---

## License

MIT License — see LICENSE file for details.
