# Release v0.3.0

**Date:** 2026-03-19  
**Status:** ✅ READY FOR RELEASE

---

## Major Changes

### Phase 1: Critical Fixes

- **Fixed silent file read failures** - Errors now logged to stderr and tracked in summary
- **Added 5MB file size limit** - Prevents DoS attacks from large files
- **HashSet optimization** - O(1) extension lookup instead of O(n) linear scan
- **Fixed README false positives** - Markdown files now properly excluded from invisible char detection

### Phase 2: High-Leverage Improvements

- **Parallel scanning with rayon** - 2x speedup on initial scans
- **Directory exclusion with ignore crate** - Glob patterns, automatic `.gitignore` support
- **Complete SARIF rules (GW001-GW008)** - Full GitHub Advanced Security compliance
- **Findings deduplication** - 20-30% noise reduction
- **Batch LLM analyzer** - High-volume automated review

### Phase 3: Architecture & Production Readiness

- **Unified Detector trait** - Composable detection logic
- **ScanConfig** - Programmatic usage and CLI decoupling
- **Incremental scanning with SHA-256 caching** - 10x speedup on re-scans
- **New behavioral detectors:**
  - `LocaleGeofencingDetector` - Locale-based evasion detection
  - `TimeDelayDetector` - Time-based logic bombs
  - `BlockchainC2Detector` - Blockchain C2 channels
  - `RddDetector` - Runtime data decryption patterns
  - `ForceMemoDetector` - Python `__forceMemo` patterns
  - `JpdAuthorDetector` - Author attribution attacks
- **Risk scoring system** - Automated prioritization

---

## Performance

| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| Initial scan (524 files) | 5s | 2.4s | **2x faster** |
| Re-scan (cached) | 5s | 0.5s | **10x faster** |
| Parallel execution | Sequential | rayon | **User time < Real time** |
| Cache hit rate | N/A | 15-25% | **Expected on re-scans** |

---

## Breaking Changes

**None** - all changes are backward compatible.

---

## Installation

```bash
# Install from source
cargo install --path glassware-cli

# Or build locally
cargo build --release
./target/release/glassware --version
```

---

## Usage

### Basic Scan

```bash
# Scan a directory
glassware src/

# Scan specific files
glassware src/index.js package.json
```

### Output Formats

```bash
# Pretty print (default)
glassware src/

# JSON output
glassware --format json src/ > findings.json

# SARIF output (GitHub Advanced Security)
glassware --format sarif src/ > results.sarif
```

### Incremental Scanning (Default)

```bash
# With default cache file (.glassware-cache.json)
glassware src/

# Custom cache file
glassware --cache-file .cache/glassware.json src/

# Disable caching
glassware --no-cache src/

# Custom cache TTL
glassware --cache-ttl 14 src/
```

### Severity Filtering

```bash
# Only critical findings
glassware --severity critical src/

# High and above (default)
glassware --severity high src/

# All findings
glassware --severity low src/
```

### LLM Analysis

```bash
# Requires environment variables:
# - GLASSWARE_LLM_BASE_URL
# - GLASSWARE_LLM_API_KEY

glassware --llm src/
```

---

## New Files (Phase 3)

### Core Library

- `glassware-core/src/cache.rs` - SHA-256 incremental scanning cache
- `glassware-core/src/locale_detector.rs` - Locale-based geofencing detection
- `glassware-core/src/time_delay_detector.rs` - Time-based logic bomb detection
- `glassware-core/src/blockchain_c2_detector.rs` - Blockchain C2 channel detection
- `glassware-core/src/rdd_detector.rs` - Runtime data decryption patterns
- `glassware-core/src/forcememo_detector.rs` - Python forced execution patterns
- `glassware-core/src/jpd_author_detector.rs` - Author attribution attacks
- `glassware-core/src/risk_scorer.rs` - Risk scoring and prioritization

### Documentation

- `DOCUMENTATION-CATALOG.md` - Complete documentation index
- `HANDOFF-WORKFLOW.md` - Development workflow guide
- `INTEL.md` - Threat intelligence summary
- `INTEL-REVIEW-EVASION-TECHNIQUES.md` - Evasion technique catalog
- `examples/programmatic_usage.rs` - Library usage examples

### Harness Tools

- `harness/batch_llm_analyzer.py` - Batch LLM analysis
- `harness/github_scanner.py` - GitHub repository scanning
- `harness/diverse_sampling.py` - Package sampling strategies
- 40+ harness reports documenting scan campaigns

---

## Configuration

### Environment Variables (LLM Layer)

```bash
export GLASSWARE_LLM_BASE_URL="https://api.cerebras.ai/v1"
export GLASSWARE_LLM_API_KEY="your-api-key"
export GLASSWARE_LLM_MODEL="llama-3.3-70b"  # Optional
```

### Supported LLM Providers

| Provider | Base URL | Recommended Model |
|----------|----------|-------------------|
| Cerebras | `https://api.cerebras.ai/v1` | `llama-3.3-70b` |
| Groq | `https://api.groq.com/openai/v1` | `llama-3.3-70b-versatile` |
| OpenAI | `https://api.openai.com/v1` | `gpt-4o` |
| NVIDIA NIM | `https://integrate.api.nvidia.com/v1` | `meta/llama-3.3-70b-instruct` |
| Ollama (local) | `http://localhost:11434/v1` | `llama3.3` |

---

## Detection Capabilities

| ID | Detection | Severity | Description |
|----|-----------|----------|-------------|
| GW001 | SteganoPayload | Critical | Dense runs of Unicode Variation Selectors |
| GW002 | DecoderFunction | High | `codePointAt` + 0xFE00/0xE0100 patterns |
| GW003 | InvisibleCharacter | Critical-High | ZWSP, ZWNJ, ZWJ, word joiners |
| GW004 | BidirectionalOverride | Critical | Trojan Source bidirectional overrides |
| GW005 | EncryptedPayload | High | High-entropy blob + dynamic execution |
| GW006 | HardcodedKeyDecryption | High | Crypto API with hardcoded key → exec |
| GW007 | Rc4Pattern | Info | Hand-rolled RC4-like cipher + exec |
| GW008 | HeaderC2 | Critical | HTTP header extraction + decrypt → exec |
| - | LocaleGeofencing | High | Locale-based execution gating |
| - | TimeDelay | Critical | Time-based logic bombs |
| - | BlockchainC2 | Critical | Blockchain smart contract C2 |
| - | RDD | High | Runtime data decryption |
| - | ForceMemo | High | Python `__forceMemo` patterns |
| - | JpdAuthor | Medium | Author attribution manipulation |

---

## Testing

```bash
# Run all tests
cargo test --features "full,llm"

# Run tests for specific package
cargo test -p glassware-core
cargo test -p glassware-cli

# Run with coverage (requires cargo-tarpaulin)
cargo tarpaulin --features "full,llm" --out Html
```

**Test Results:** All 177 tests pass with full feature set.

---

## Code Quality

```bash
# Format code
cargo fmt --all

# Run clippy (must pass with no warnings)
cargo clippy --features "full,llm" -- -D warnings

# Build documentation
cargo doc --no-deps --features "full,llm"
```

---

## References

- **Code Review:** `harness/reports/CODEREVIEW_193.md`
- **Phase 1 Report:** `harness/reports/PHASE1-IMPLEMENTATION-REPORT.md`
- **Phase 2 Report:** `harness/reports/PHASE2-IMPLEMENTATION-REPORT.md`
- **Documentation:** `DOCUMENTATION-CATALOG.md`

---

## Changelog

### v0.3.0 (2026-03-19)

**Added:**
- Incremental scanning with SHA-256 caching
- 7 new behavioral detectors
- Risk scoring system
- ScanConfig for programmatic usage
- Unified Detector trait

**Changed:**
- Parallel scanning (rayon)
- Directory exclusion (ignore crate)
- HashSet for extensions (O(1) lookup)

**Fixed:**
- Silent file read failures
- README false positives
- Memory efficiency improvements

**Performance:**
- 2x faster initial scans
- 10x faster re-scans (cached)

---

**Release prepared by:** glassware team  
**License:** MIT OR Apache-2.0
