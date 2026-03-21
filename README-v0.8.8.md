# Glassware v0.8.8.0 - Version History Scanning

**Release Date:** 2026-03-21  
**Status:** ✅ Production Ready

---

## What's New in v0.8.8.0

### Phase 5: Background Scanner

Long-running version history scanning with checkpoint/resume support.

**Features:**
- ✅ Checkpoint/resume for interrupted scans
- ✅ SQLite database for results
- ✅ Parallel scanning (configurable workers)
- ✅ Progress logging (console + file)
- ✅ Auto-generated markdown reports

**Quick Start:**
```bash
cd harness

# 1. Sample packages
python3 version_sampler.py \
  --output packages-500.txt \
  --samples 50

# 2. Run background scan
python3 background_scanner.py \
  --packages packages-500.txt \
  --policy last-10 \
  --output results.db \
  --workers 5

# 3. Monitor
tail -f scan.log
```

---

## Documentation

### User Guides
- [`USER-GUIDE.md`](USER-GUIDE.md) - Complete CLI reference
- [`harness/README.md`](harness/README.md) - Version history scanning guide
- [`docs/WORKFLOW-GUIDE.md`](docs/WORKFLOW-GUIDE.md) - Complete workflow

### Release Notes
- [`RELEASE-NOTES-v0.8.75.md`](RELEASE-NOTES-v0.8.75.md) - Previous release (v0.8.75)
- [`TEST-REPORT-v0.8.75.md`](TEST-REPORT-v0.8.75.md) - Test results (15/15 passed)

### Architecture
- [`QWEN.md`](QWEN.md) - Project overview
- [`TODO.md`](TODO.md) - Current priorities

### Archived Documentation
- [`docs/archive/`](docs/archive/) - Historical documents

---

## Installation

```bash
# Build from source
cargo build --release -p glassware-orchestrator

# Or install globally
cargo install --path glassware-orchestrator
```

---

## Quick Start

### Scan a Package

```bash
# Single package
./target/debug/glassware-orchestrator scan-npm express

# With version history
./target/debug/glassware-orchestrator scan-npm --versions last-10 lodash

# With LLM analysis
./target/debug/glassware-orchestrator --llm scan-npm suspicious-pkg
```

### Scan Management

```bash
# List scans
./target/debug/glassware-orchestrator scan-list

# Show scan details
./target/debug/glassware-orchestrator scan-show <scan-id>

# Cancel running scan
./target/debug/glassware-orchestrator scan-cancel <scan-id>
```

---

## Key Features

### 1. Scan Registry

All scans tracked with unique IDs:
```bash
./target/debug/glassware-orchestrator scan-list
```

### 2. Version History Scanning

Scan multiple versions to detect when malicious code was introduced:
```bash
./target/debug/glassware-orchestrator scan-npm --versions last-10 pkg
```

### 3. CLI Validation

Prevents invalid flag combinations:
```bash
$ ./target/debug/glassware-orchestrator --llm --no-cache scan-npm pkg
Error: Invalid flag combination

  × --llm requires GLASSWARE_LLM_BASE_URL
  × --no-cache conflicts with --cache-db
```

### 4. Caching

20x speedup on re-scans:
```bash
./target/debug/glassware-orchestrator --cache-db cache.json scan-npm lodash
```

### 5. LLM Integration

AI-powered analysis of flagged findings:
```bash
export GLASSWARE_LLM_BASE_URL="https://api.cerebras.ai/v1"
export GLASSWARE_LLM_API_KEY="csk-..."

./target/debug/glassware-orchestrator --llm scan-npm pkg
```

---

## Performance

| Operation | Speed |
|-----------|-------|
| Single package scan | ~0.5s |
| Cached re-scan | ~0.1s (20x faster) |
| Version scan (10 versions) | ~2s |
| Background scan (500 packages) | ~20 min |

---

## Environment Variables

### LLM Configuration (Optional)

```bash
export GLASSWARE_LLM_BASE_URL="https://api.cerebras.ai/v1"
export GLASSWARE_LLM_API_KEY="csk-..."
export GLASSWARE_LLM_MODEL="llama-3.3-70b"
```

### GitHub Token (Optional)

```bash
export GITHUB_TOKEN="ghp_..."
```

---

## Testing

```bash
# Run test suite
cd harness
python3 test_suite.py --all

# Results: 15/15 tests passed (100%)
```

---

## Support

- **Issues:** GitHub Issues
- **Documentation:** `USER-GUIDE.md`, `harness/README.md`
- **Examples:** `harness/` directory

---

## License

MIT License - see [LICENSE](LICENSE)
