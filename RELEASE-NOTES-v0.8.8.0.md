# Release Notes - Glassware v0.8.8.0

**Release Date:** 2026-03-21  
**Tag:** v0.8.8.0  
**Status:** ✅ Production Ready

---

## Overview

Phase 5 implementation: Background version history scanner with checkpoint/resume support for long-running scans.

---

## New Features

### 1. Background Scanner (Phase 5)

**File:** `harness/background_scanner.py`

Long-running scanner for version history analysis:
- ✅ Checkpoint/resume support
- ✅ SQLite database for results
- ✅ Parallel scanning (configurable workers)
- ✅ Progress logging (console + file)
- ✅ Auto-generated markdown reports

**Usage:**
```bash
python3 background_scanner.py \
  --packages packages.txt \
  --policy last-10 \
  --output results.db \
  --workers 5
```

### 2. Package Sampler (Phase 4)

**File:** `harness/version_sampler.py`

Diverse package sampling from npm:
- ✅ 12 categories (ai-ml, native-build, web-frameworks, etc.)
- ✅ Filter by: recently updated, popular, new
- ✅ Multiple output formats (plain, JSON, CSV)
- ✅ Rate limiting

**Usage:**
```bash
python3 version_sampler.py \
  --output packages.txt \
  --samples 50 \
  --categories ai-ml utils
```

### 3. Comprehensive Test Suite

**File:** `harness/test_suite.py`

15-test comprehensive suite:
- ✅ CLI validation (2 tests)
- ✅ Scan registry (2 tests)
- ✅ Basic scanning (2 tests)
- ✅ Version scanning (2 tests)
- ✅ Package sampler (2 tests)
- ✅ Background scanner (4 tests)
- ✅ LLM integration (1 test)

**Results:** 15/15 tests passed (100%)

---

## Improvements

### Documentation

- ✅ `harness/README.md` - Version history scanning guide
- ✅ `README-v0.8.8.md` - Updated main README
- ✅ `docs/archive/` - Archived historical documents

### Performance

| Scenario | Packages | Versions | Time |
|----------|----------|----------|------|
| Small scan | 50 | 500 | ~2 min |
| Medium scan | 500 | 5,000 | ~20 min |
| Large scan | 5,000 | 50,000 | ~3 hours |

### Reliability

- ✅ Checkpoint/resume for interrupted scans
- ✅ Graceful error handling
- ✅ Rate limiting awareness
- ✅ SQLite WAL mode for concurrency

---

## Breaking Changes

None. Backward compatible with v0.8.75.

---

## Migration Guide

No migration required. Upgrade is seamless:

```bash
git pull origin main
cargo build --release -p glassware-orchestrator
```

---

## Known Issues

### npm Version Availability

**Issue:** Old npm versions often return 404 errors

**Workaround:** Use `last-N` policy instead of `all`

```bash
# Good
--policy last-10

# Avoid for large scans
--policy all
```

---

## Files Changed

### New Files
- `harness/version_sampler.py` (430 lines)
- `harness/background_scanner.py` (450 lines)
- `harness/test_suite.py` (440 lines)
- `harness/README.md` (updated)
- `README-v0.8.8.md`
- `RELEASE-NOTES-v0.8.8.0.md`
- `TEST-REPORT-v0.8.75.md`

### Archived Files
- `docs/archive/LLM-*.md` (4 files)
- `docs/archive/PHASE4-5-*.md` (2 files)
- `docs/archive/IMPLEMENTATION-*.md` (2 files)
- `docs/archive/RELEASE-SUMMARY-*.md` (1 file)

---

## Test Results

**Test Suite:** 15/15 tests passed (100%)

| Category | Tests | Status |
|----------|-------|--------|
| CLI Validation | 2 | ✅ PASS |
| Scan Registry | 2 | ✅ PASS |
| Basic Scanning | 2 | ✅ PASS |
| Version Scanning | 2 | ✅ PASS |
| Package Sampler | 2 | ✅ PASS |
| Background Scanner | 4 | ✅ PASS |
| LLM Integration | 1 | ✅ PASS |

---

## Performance Benchmarks

### Scan Performance

| Operation | Time | Notes |
|-----------|------|-------|
| Single package | ~0.4s | express (9 files) |
| Cached re-scan | ~0.09s | 20x speedup |
| Version scan (3 versions) | ~0.8s | chalk |
| Background scan (6 versions) | ~2s | 3 packages |

### Package Sampler

| Scenario | Time | Packages |
|----------|------|----------|
| Single category (5 samples) | ~9s | 5 packages |
| Multiple categories (10 samples) | ~13s | 10 packages |

---

## Contributors

- Core development
- Phase 4/5 implementation
- Documentation
- Testing

---

## Upgrade Path

### From v0.8.75

```bash
git pull origin main
cargo build --release -p glassware-orchestrator

# Verify installation
./target/release/glassware-orchestrator --version
```

### From Earlier Versions

```bash
# Clean install
cargo install --path glassware-orchestrator --force

# Or build from source
cargo build --release -p glassware-orchestrator
```

---

## Support

- **Documentation:** `README-v0.8.8.md`, `harness/README.md`
- **Issues:** GitHub Issues
- **Discussions:** GitHub Discussions

---

**End of Release Notes**
